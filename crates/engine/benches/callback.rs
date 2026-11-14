//! Audio callback benchmark: 32 voices with pitch, envelope, filter, drive and both sends,
//! a reverb and a delay busy (56), transport and metronome at 172 BPM, 128-sample block,
//! 48 kHz. Target: at most 50 % of the block budget.
//! Run: `cargo bench -p engine`.

mod support;

use std::io;
use std::sync::Arc;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::time::Instant;

use engine::rt::{Command, ProjectCommand, ProjectMessage, ProjectState, Realtime, Stats};
use engine::{Engine, MAX_VOICES};
use support::attribution::Trace;
use support::{Report, Timing, measure};

const SR: u32 = 48_000;
const BLOCK: usize = 128;
const FULL_BLOCKS: usize = 20_000;

fn run_size() -> (usize, u64) {
    if std::env::var_os("PTSR_BENCH_QUICK").is_some() {
        (2_000, 20)
    } else {
        (FULL_BLOCKS, 200)
    }
}

fn setup(mut send: impl FnMut(Command)) {
    send(Command::SetTempo { bpm: 172.0 });
    send(Command::SetMetronome { enabled: true });
    send(Command::TransportPlay);
    // A press, pitch change and envelope for every pad.
    // different pads: hitting one pad again would cut its previous voice
    for pad in 0..MAX_VOICES {
        send(Command::PadPress { pad, velocity: 1.0 });
        send(Command::SetPadPitch { pad, semitones: 7 });
        send(Command::SetPadEnvelope {
            pad,
            attack_ms: 20,
            decay_ms: 100,
            release_ms: 150,
        });
        // a resonant band filter on every voice (44)
        send(Command::SetPadFilter {
            pad,
            settings: engine::filter::FilterSettings {
                kind: engine::filter::FilterKind::Band,
                cutoff_hz: 900.0,
                resonance: 0.5,
            },
        });
        // the drive and both sends on every voice (56)
        send(Command::SetPadDrive { pad, drive: 0.6 });
        send(Command::SetPadSends {
            pad,
            sends: [0.3, 0.3],
        });
    }
    // both slots busy: a reverb and a delay
    send(Command::SetFx {
        slot: 0,
        settings: engine::fx::FxSettings::Reverb {
            size: 0.8,
            tone: 0.5,
        },
    });
    send(Command::SetFx {
        slot: 1,
        settings: engine::fx::FxSettings::Delay {
            time_ms: 375.0,
            feedback: 0.6,
        },
    });
}

fn main() -> io::Result<()> {
    let budget = BLOCK as f64 / SR as f64;
    let (blocks, switches) = run_size();
    println!("{}", support::TIMING_NOTES);
    const OVERHEAD_CALLS: usize = 2_000;
    let mut empty = Vec::with_capacity(OVERHEAD_CALLS);
    let start = Instant::now();
    for _ in 0..OVERHEAD_CALLS {
        empty.push(measure(|| std::hint::black_box(()))?);
    }
    let envelope_mean = start.elapsed().as_secs_f64() / OVERHEAD_CALLS as f64;
    support::write_overhead(
        &Report::new(&empty, budget)?,
        envelope_mean,
        &mut io::stdout().lock(),
    )?;

    let mut trace = Trace::new(budget)?;
    let mut observed_empty = Vec::with_capacity(OVERHEAD_CALLS);
    let start = Instant::now();
    for block in 0..OVERHEAD_CALLS {
        observed_empty.push(trace.measure("observation overhead", 0, block, || {
            std::hint::black_box(())
        })?);
    }
    let observed_envelope_mean = start.elapsed().as_secs_f64() / OVERHEAD_CALLS as f64;
    support::attribution::write_overhead(
        &Report::new(&observed_empty, budget)?,
        observed_envelope_mean,
        &mut io::stdout().lock(),
    )?;

    // The sound is longer than the run, so every voice plays to the end.
    let sound = (0..BLOCK * blocks * 2 + 1)
        .map(|i| (i as f32 * 0.01).sin() * 0.01)
        .collect();
    let (mut tx, rx) = rtrb::RingBuffer::new(8 * MAX_VOICES);
    setup(|cmd| tx.push(cmd).unwrap());
    let mut rt = Realtime::new(
        Engine::with_sounds(SR, engine::click(SR), vec![sound]),
        vec![rx],
        Arc::new(Stats::default()),
        SR,
    );
    let (mut l, mut r) = (vec![0.0; BLOCK], vec![0.0; BLOCK]);

    let samples = (0..blocks)
        .map(|block| trace.measure("steady playback", 0, block, || rt.process(&mut l, &mut r)))
        .collect::<io::Result<Vec<_>>>()?;
    let report = Report::new(&samples, budget)?;

    let pct = |t: f64| t / budget * 100.0;
    let mean = report.wall.mean;
    let p99 = report.wall.p99;
    let max = report.wall.max;
    println!(
        "{MAX_VOICES} voices with pitch, envelope, filter, drive and sends, reverb and delay, transport/metronome at 172 BPM, block {BLOCK} @ {SR} Hz, budget {:.3} ms: mean {:.2} %, p99 {:.2} %, max {:.2} %",
        budget * 1e3,
        pct(mean),
        pct(p99),
        pct(max)
    );
    report.write("steady playback", &mut io::stdout().lock())?;
    let switch_p99 = bench_switch(budget, &mut trace, switches)?;
    let (pattern_p99, dropped) = bench_pattern(budget, &mut trace, blocks)?;
    let recording_exceeded = bench_recording(budget, &mut trace, blocks)?;
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/bench-diagnostics");
    let (path, exceeded) = trace.finish(&directory, pct(p99), switch_p99)?;
    println!("Local callback trace: {}", path.display());
    if std::env::var_os("PTSR_BENCH_REPORT_ONLY").is_some() {
        println!(
            "CI report only: steady p99 {p99:.2} %, switch p99 {switch_p99:.2} %, dense pattern p99 {pattern_p99:.2} %, dropped hits {dropped}, recording/erasure limit exceeded {recording_exceeded}"
        );
        return Ok(());
    }
    if exceeded {
        eprintln!("EXCEEDED: steady callback p99 or switch peak p99 is over 50 % of the budget");
        std::process::exit(1);
    }
    if pattern_p99 > 50.0 || dropped != 0 {
        eprintln!("EXCEEDED: dense pattern wall p99 is over 50 % or pattern hits were dropped");
        std::process::exit(1);
    }
    if recording_exceeded {
        eprintln!("EXCEEDED: active recording or erasure maximum is over 50 % of the budget");
        std::process::exit(1);
    }
    Ok(())
}

fn bench_recording(budget: f64, trace: &mut Trace, blocks: usize) -> io::Result<bool> {
    use support::recording::{Kind, Workload, maximum_exceeded};
    let mut exceeded = false;
    for kind in [Kind::Recording, Kind::Erasure] {
        let mut workload = Workload::new(kind, blocks, |send| setup(send))?;
        let mut samples = Vec::with_capacity(blocks);
        for block in 0..blocks {
            workload.prepare(block);
            samples.push(trace.measure(kind.label(), 0, block, || workload.process())?);
            workload.collect()?;
        }
        let report = Report::new(&samples, budget)?;
        report.write(kind.label(), &mut io::stdout().lock())?;
        exceeded |= maximum_exceeded(&report);
        workload.release();
        let release = trace.measure(
            match kind {
                Kind::Recording => "recording release",
                Kind::Erasure => "erasure release",
            },
            0,
            blocks,
            || workload.process(),
        )?;
        let report = Report::new(&[release], budget)?;
        report.write(
            &format!("{} release", kind.label()),
            &mut io::stdout().lock(),
        )?;
        exceeded |= maximum_exceeded(&report);
        workload.finish();
        let finishing = trace.measure(
            match kind {
                Kind::Recording => "recording finishing",
                Kind::Erasure => "erasure finishing",
            },
            0,
            blocks + 1,
            || workload.process(),
        )?;
        let report = Report::new(&[finishing], budget)?;
        report.write(
            &format!("{} finishing", kind.label()),
            &mut io::stdout().lock(),
        )?;
        exceeded |= maximum_exceeded(&report);
        let evidence = workload.validate(blocks)?;
        println!(
            "{}: {} callbacks, {} voices, {} accepted onsets, {} completed Gates, {} erased rows; no capture losses, no dropped playback hits. Accepted Finishing boundary; UI reconstruction and acknowledgement are not measured.",
            kind.label(),
            blocks,
            evidence.voices,
            evidence.onsets,
            evidence.gates,
            evidence.erased
        );
    }
    Ok(exceeded)
}

fn bench_pattern(budget: f64, trace: &mut Trace, blocks: usize) -> io::Result<(f64, u64)> {
    use engine::pattern::{Grid, Hit, RawPattern};
    let sound = (0..SR as usize * 4)
        .map(|i| (i as f32 * 0.01).sin() * 0.001)
        .collect();
    let mut engine = Engine::with_sounds(SR, engine::click(SR), vec![sound]);
    let (garbage, mut retired) = rtrb::RingBuffer::new(1024);
    engine.set_garbage(garbage);
    setup(|command| {
        if !matches!(command, Command::PadPress { .. }) {
            engine.handle(0, command);
        }
    });
    // Exactly 4096 events: 32 pads at every 1/64 line of a two-bar loop.
    let hits = (0..128)
        .flat_map(|step| (0..MAX_VOICES).map(move |pad| Hit::new(step * 60, pad, 127, 0).unwrap()))
        .collect();
    let pattern = RawPattern::new(2, Grid::SixtyFourth, 100, hits)
        .unwrap()
        .compile();
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(Arc::new(pattern)),
        },
    );
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![], stats.clone(), SR);
    let (mut left, mut right) = ([0.0; BLOCK], [0.0; BLOCK]);
    let mut samples = Vec::with_capacity(blocks);
    for block in 0..blocks {
        samples.push(trace.measure("dense pattern playback", 0, block, || {
            rt.process(&mut left, &mut right)
        })?);
        while retired.pop().is_ok() {}
    }
    let report = Report::new(&samples, budget)?;
    let p99 = report.wall.p99 / budget * 100.0;
    let dropped = stats
        .transport(0)
        .expect("completed callback snapshot")
        .dropped_hits;
    println!(
        "dense pattern: 4096 hits, {MAX_VOICES} voices, 172 BPM, block {BLOCK} @ {SR} Hz, dropped hits {dropped}"
    );
    report.write("dense pattern playback", &mut io::stdout().lock())?;
    Ok((p99, dropped))
}

fn bench_switch(budget: f64, trace: &mut Trace, switches: u64) -> io::Result<f64> {
    let sound = Arc::new(engine::Sample::mono(
        (0..SR as usize * 4)
            .map(|i| (i as f32 * 0.01).sin() * 0.01)
            .collect(),
    ));
    let (garbage_tx, mut garbage_rx) = rtrb::RingBuffer::new(1024);
    let mut engine = Engine::with_sounds(SR, engine::click(SR), vec![]);
    engine.set_garbage(garbage_tx);
    let (mut tx, rx) = rtrb::RingBuffer::new(8 * MAX_VOICES);
    let state = Arc::new(ProjectState::default());
    let mut rt = Realtime::with_project(
        engine,
        vec![rx],
        Arc::new(Stats::default()),
        SR,
        state.clone(),
    );
    let (mut l, mut r) = (vec![0.0; BLOCK], vec![0.0; BLOCK]);
    let mut times = Vec::new();
    let mut peaks = Vec::new();
    let mut samples = Vec::new();
    let mut phase_times: [Vec<Timing>; 3] = std::array::from_fn(|_| Vec::new());
    for epoch in 1..=switches {
        for pad in 0..MAX_VOICES {
            tx.push(ProjectCommand {
                epoch: epoch - 1,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadSound {
                    pad,
                    sound: Some(sound.clone()),
                }),
            })
            .unwrap();
        }
        setup(|cmd| {
            tx.push(ProjectCommand {
                epoch: epoch - 1,
                ready_at_send: true,
                command: ProjectMessage::Command(cmd),
            })
            .unwrap()
        });
        // Fill the effects and let the voices reach their sustained state.
        for _ in 0..32 {
            rt.process(&mut l, &mut r);
        }
        while garbage_rx.pop().is_ok() {}
        state.epoch.store(epoch, Release);
        tx.push(ProjectCommand {
            epoch,
            ready_at_send: false,
            command: ProjectMessage::Ready,
        })
        .unwrap();
        let mut peak: f64 = 0.0;
        let mut blocks = 0;
        while state.ready.load(Acquire) != epoch {
            let phase = match blocks {
                0 => "first fade block",
                1 => "second fade block",
                _ => "clearing and restore",
            };
            let sample = trace.measure(phase, epoch, blocks, || rt.process(&mut l, &mut r))?;
            times.push(sample.wall);
            samples.push(sample);
            phase_times[blocks.min(2)].push(sample);
            peak = peak.max(sample.wall);
            blocks += 1;
            assert!(blocks < 1000, "project switch did not complete");
        }
        peaks.push(peak);
        while garbage_rx.pop().is_ok() {}
    }
    times.sort_by(f64::total_cmp);
    peaks.sort_by(f64::total_cmp);
    let pct = |time: f64| time / budget * 100.0;
    let p99 = pct(times[times.len() * 99 / 100]);
    let peak_p99 = pct(peaks[peaks.len() * 99 / 100]);
    println!(
        "project switch: {switches} switches, {} callbacks, mean {:.2} %, callback p99 {:.2} %, switch peak p99 {:.2} %, max {:.2} %",
        times.len(),
        pct(times.iter().sum::<f64>() / times.len() as f64),
        p99,
        peak_p99,
        pct(*times.last().unwrap())
    );
    Report::new(&samples, budget)?
        .write("all project-switch callbacks", &mut io::stdout().lock())?;
    for (label, phase) in [
        "first fade block",
        "second fade block",
        "clearing and restore",
    ]
    .into_iter()
    .zip(&mut phase_times)
    {
        let report = Report::new(phase, budget)?;
        println!(
            "  {label}: p99 {:.2} %, max {:.2} %",
            pct(report.wall.p99),
            pct(report.wall.max)
        );
        report.write(label, &mut io::stdout().lock())?;
    }
    Ok(peak_p99)
}

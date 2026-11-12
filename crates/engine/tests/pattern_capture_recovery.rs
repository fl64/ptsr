//! Interrupted-take recovery through production queues and retained audio snapshots.

use engine::capture::{CAPACITY, Event, Kind, Phase, Recovery, Snapshot};
use engine::pattern::{Grid, Hit, MAX_GATE_TICKS, MAX_HITS, RawPattern};
use engine::rt::{ProjectCommand, ProjectMessage, ProjectState, Realtime, Stats};
use engine::{Command, Engine, PadMode, Retired, Sample, checks};
use std::sync::{
    Arc,
    atomic::Ordering::{Acquire, Release},
};

const SR: u32 = 48_000;
type Wave = (Vec<f32>, Vec<f32>);

struct Rig {
    tx: rtrb::Producer<ProjectCommand>,
    events: rtrb::Consumer<Event>,
    recovery: Arc<Recovery>,
    garbage: rtrb::Consumer<Retired>,
    rt: Option<Realtime>,
    stats: Arc<Stats>,
    project: Arc<ProjectState>,
}

fn compiled(rows: Vec<Hit>) -> Command {
    Command::SetPattern {
        pattern: Some(Arc::new(
            RawPattern::new(1, Grid::Off, 100, rows).unwrap().compile(),
        )),
    }
}

fn press(pad: usize) -> Command {
    Command::PadPress {
        pad,
        velocity: 100.0 / 127.0,
    }
}

impl Rig {
    fn new() -> Self {
        Self::at_rate(SR, 120.0, vec![])
    }

    fn at_rate(sr: u32, bpm: f64, rows: Vec<Hit>) -> Self {
        let frames = sr as usize * 3;
        let tone = |hz: f32, gain: f32| {
            (0..frames)
                .map(|i| gain * (std::f32::consts::TAU * hz * i as f32 / sr as f32).cos())
                .collect::<Vec<_>>()
        };
        let sample = Arc::new(Sample {
            left: tone(55.0, 0.04).into_boxed_slice(),
            right: Some(tone(82.5, 0.035).into_boxed_slice()),
        });
        let mut engine = Engine::with_sounds(sr, vec![], vec![]);
        engine.handle(0, Command::SetTempo { bpm });
        engine.handle(0, compiled(rows));
        for pad in 0..3 {
            engine.handle(
                0,
                Command::SetPadSound {
                    pad,
                    sound: Some(sample.clone()),
                },
            );
            engine.handle(
                0,
                Command::SetPadMode {
                    pad,
                    mode: if pad == 2 {
                        PadMode::OneShot
                    } else {
                        PadMode::Gate
                    },
                },
            );
            engine.handle(
                0,
                Command::SetPadEnvelope {
                    pad,
                    attack_ms: 2,
                    decay_ms: 0,
                    release_ms: 10,
                },
            );
        }
        let events = engine.enable_capture();
        let recovery = engine.capture_recovery().unwrap();
        let (garbage_tx, garbage) = rtrb::RingBuffer::new(1024);
        engine.set_garbage(garbage_tx);
        let (tx, rx) = rtrb::RingBuffer::new(1024);
        let stats = Arc::new(Stats::default());
        let project = Arc::new(ProjectState::default());
        let rt = Some(Realtime::with_project(
            engine,
            vec![rx],
            stats.clone(),
            sr,
            project.clone(),
        ));
        Self {
            tx,
            events,
            recovery,
            garbage,
            rt,
            stats,
            project,
        }
    }

    fn tagged(&mut self, epoch: u64, ready: bool, command: ProjectMessage) {
        self.tx
            .push(ProjectCommand {
                epoch,
                ready_at_send: ready,
                command,
            })
            .unwrap();
    }

    fn send(&mut self, command: Command) {
        let epoch = self.project.epoch.load(Acquire);
        self.tagged(
            epoch,
            self.project.ready.load(Acquire) == epoch,
            ProjectMessage::Command(command),
        );
    }

    fn start(&mut self, pass: u64) {
        self.send(Command::TransportPlay);
        self.send(Command::PatternRecordStart { pass });
    }

    fn render(&mut self, frames: usize) -> Wave {
        let (mut left, mut right) = (vec![0.0; frames], vec![0.0; frames]);
        let rt = self.rt.as_mut().expect("audio must be present to process");
        if frames == 0 {
            rt.process(&mut [], &mut []);
        }
        for (l, r) in left.chunks_mut(128).zip(right.chunks_mut(128)) {
            while self.garbage.pop().is_ok() {}
            rt.process(l, r);
        }
        (left, right)
    }

    fn state(&self) -> Snapshot {
        self.stats
            .recording(self.project.epoch.load(Acquire))
            .unwrap()
    }

    fn drain(&mut self) -> Vec<Event> {
        let mut out = Vec::new();
        while let Ok(e) = self.events.pop() {
            out.push(e);
        }
        out
    }

    fn lose_audio(&mut self, request: Command) -> Snapshot {
        let boundary = self.state();
        self.send(request);
        // The queued request cannot run after destruction of this isolated callback owner.
        drop(self.rt.take());
        assert_eq!(self.state(), boundary);
        boundary
    }
}

fn append(to: &mut Wave, from: Wave) {
    to.0.extend(from.0);
    to.1.extend(from.1);
}

fn compare(actual: &Wave, reference: &Wave) {
    for (a, b) in [(&actual.0, &reference.0), (&actual.1, &reference.1)] {
        assert_eq!(a.len(), b.len());
        for (i, (&a, &b)) in a.iter().zip(b).enumerate() {
            assert!((a - b).abs() <= 0.000001, "frame {i}: {a} != {b}");
        }
        assert_eq!(checks::first_non_finite(a), None);
        assert!(checks::peak(a) < 1.0);
        assert!(checks::max_step(a) < 0.004);
    }
}

#[test]
fn interrupted_pending_gate_closes_without_a_final_callback_or_acknowledgement() {
    for request in [
        Command::PatternRecordEnd { pass: 7 },
        Command::TransportStop,
    ] {
        let mut rig = Rig::new();
        rig.start(7);
        rig.send(press(0));
        rig.render(6000);
        let onset = rig.drain()[0];
        assert_eq!(rig.recovery.completed(onset), None);
        let boundary = rig.lose_audio(request);
        assert_eq!(boundary.phase, Phase::Recording);
        assert!((boundary.ticks - 240.0).abs() < 1e-8);
        for _ in 0..3 {
            assert_eq!(
                rig.recovery.interrupted(onset, boundary),
                Some(Hit::new(0, 0, 100, 240).unwrap())
            );
            assert_eq!(
                rig.recovery.completed(onset),
                None,
                "recovery is not an audio completion"
            );
            assert_eq!(
                rig.state(),
                boundary,
                "no fabricated acknowledgement or phase change"
            );
        }
        assert!(
            rig.drain().is_empty(),
            "retry does not create capture events"
        );
    }
}

#[test]
fn interrupted_recovery_preserves_released_retriggered_and_one_shot_rows() {
    let mut rig = Rig::new();
    rig.start(7);
    rig.send(press(0));
    rig.render(3000);
    rig.send(Command::PadRelease { pad: 0 });
    rig.send(press(2));
    rig.send(press(1));
    rig.render(3000);
    rig.send(press(1));
    rig.render(6000);
    let events: Vec<_> = rig
        .drain()
        .into_iter()
        .filter(|e| matches!(e.kind, Kind::Onset { .. }))
        .collect();
    let boundary = rig.lose_audio(Command::PatternRecordEnd { pass: 7 });
    let rows: Vec<_> = events
        .iter()
        .map(|&e| rig.recovery.interrupted(e, boundary).unwrap())
        .collect();
    assert_eq!(
        rows,
        vec![
            Hit::new(0, 0, 100, 120).unwrap(),
            Hit::new(120, 2, 100, 0).unwrap(),
            Hit::new(120, 1, 100, 120).unwrap(),
            Hit::new(240, 1, 100, 240).unwrap()
        ]
    );
    assert_eq!(rig.recovery.completed(events[0]), Some(rows[0]));
    assert_eq!(rig.recovery.completed(events[2]), Some(rows[2]));
    assert_eq!(rig.recovery.completed(events[3]), None);
}

#[test]
fn interrupted_gate_duration_crosses_the_loop_seam_and_a_tempo_change() {
    let mut rig = Rig::new();
    rig.start(7);
    rig.render(90_000);
    rig.send(press(0));
    rig.render(6000);
    rig.send(Command::SetTempo { bpm: 240.0 });
    rig.render(3000);
    let onset = rig.drain()[0];
    let boundary = rig.lose_audio(Command::PatternRecordEnd { pass: 7 });
    assert_eq!(
        rig.recovery.interrupted(onset, boundary),
        Some(Hit::new(3600, 0, 100, 480).unwrap())
    );
}

#[test]
fn interrupted_stop_boundary_survives_restart_but_not_a_new_pass() {
    let mut rig = Rig::new();
    rig.start(7);
    rig.send(press(0));
    rig.render(6000);
    let onset = rig.drain()[0];
    rig.send(Command::TransportStop);
    rig.render(0);
    let boundary = rig.state();
    assert_eq!(boundary.phase, Phase::Finishing);
    assert_eq!(rig.stats.transport(0).unwrap().ticks, 0.0);
    let expected = Some(Hit::new(0, 0, 100, 240).unwrap());
    assert_eq!(rig.recovery.interrupted(onset, boundary), expected);
    rig.send(Command::TransportPlay);
    rig.render(128);
    assert_eq!(rig.state(), boundary);
    assert_eq!(rig.recovery.interrupted(onset, boundary), expected);
    rig.send(Command::PatternRecordAck {
        pass: 7,
        sequence: boundary.sequence,
    });
    rig.send(Command::PatternRecordStart { pass: 8 });
    rig.send(press(0));
    rig.render(128);
    let newer = rig.drain()[0];
    assert_eq!(rig.recovery.interrupted(onset, boundary), None);
    assert_eq!(rig.recovery.interrupted(newer, boundary), None);
    let fresh_boundary = rig.lose_audio(Command::PatternRecordEnd { pass: 8 });
    assert!(rig.recovery.interrupted(newer, fresh_boundary).is_some());
    assert_eq!(rig.recovery.interrupted(onset, fresh_boundary), None);
}

#[test]
fn interrupted_recovery_rejects_old_projects_and_count_in_cancellation() {
    let mut rig = Rig::new();
    rig.start(7);
    rig.send(press(0));
    rig.render(128);
    let onset = rig.drain()[0];
    let boundary = rig.state();
    rig.project.epoch.store(1, Release);
    rig.tagged(1, false, ProjectMessage::Command(compiled(vec![])));
    rig.tagged(1, false, ProjectMessage::Ready);
    for _ in 0..1000 {
        rig.render(128);
        if rig.project.ready.load(Acquire) == 1 {
            break;
        }
    }
    assert_eq!(rig.project.ready.load(Acquire), 1);
    assert_eq!(rig.recovery.interrupted(onset, boundary), None);
    rig.send(Command::PatternRecordStart { pass: 1 });
    rig.render(0);
    let count_in = rig.state();
    assert_eq!(count_in.phase, Phase::CountIn);
    assert_eq!(rig.recovery.interrupted(onset, count_in), None);
    rig.send(Command::PatternRecordEnd { pass: 1 });
    rig.render(0);
    assert_eq!(rig.state().phase, Phase::Idle);
    assert_eq!(rig.recovery.interrupted(onset, rig.state()), None);
}

#[test]
fn interrupted_boundary_refuses_invalid_phase_identity_time_and_sequence() {
    let mut rig = Rig::new();
    rig.start(7);
    rig.render(3000);
    rig.send(press(0));
    rig.render(3000);
    let onset = rig.drain()[0];
    let boundary = rig.lose_audio(Command::PatternRecordEnd { pass: 7 });
    for bad in [
        Snapshot {
            epoch: 1,
            ..boundary
        },
        Snapshot {
            pass: 0,
            ..boundary
        },
        Snapshot {
            pass: 8,
            ..boundary
        },
        Snapshot {
            phase: Phase::Idle,
            ..boundary
        },
        Snapshot {
            phase: Phase::CountIn,
            ..boundary
        },
        Snapshot {
            ticks: f64::NAN,
            ..boundary
        },
        Snapshot {
            ticks: f64::INFINITY,
            ..boundary
        },
        Snapshot {
            ticks: -1.0,
            ..boundary
        },
        Snapshot {
            ticks: onset.ticks - 1.0,
            ..boundary
        },
        Snapshot {
            loop_ticks: 0,
            ..boundary
        },
        Snapshot {
            loop_ticks: 7680,
            ..boundary
        },
        Snapshot {
            sequence: 0,
            ..boundary
        },
        Snapshot {
            captured_onsets: 0,
            ..boundary
        },
    ] {
        assert_eq!(rig.recovery.interrupted(onset, bad), None, "{bad:?}");
    }
    for bad in [
        Event { epoch: 1, ..onset },
        Event { pass: 0, ..onset },
        Event { pass: 8, ..onset },
        Event {
            sequence: 0,
            ..onset
        },
        Event {
            sequence: boundary.sequence + 1,
            ..onset
        },
        Event {
            ticks: f64::NAN,
            ..onset
        },
        Event {
            ticks: f64::INFINITY,
            ..onset
        },
        Event {
            ticks: -1.0,
            ..onset
        },
        Event {
            ticks: boundary.ticks + 1.0,
            ..onset
        },
        Event {
            gate: None,
            ..onset
        },
        Event {
            gate: Some(MAX_HITS),
            ..onset
        },
        Event {
            gate: Some(boundary.captured_onsets as usize),
            ..onset
        },
        Event {
            kind: Kind::Release { pad: 0 },
            ..onset
        },
    ] {
        assert_eq!(rig.recovery.interrupted(bad, boundary), None, "{bad:?}");
    }
    assert!(rig.recovery.interrupted(onset, boundary).is_some());
}

#[test]
fn interrupted_overflow_recovers_only_original_accepted_onsets() {
    let mut rig = Rig::new();
    rig.start(7);
    rig.send(press(0));
    rig.render(0);
    for _ in 1..CAPACITY {
        rig.send(Command::PadRelease { pad: 2 });
    }
    for _ in 0..3 {
        rig.render(0);
    }
    rig.render(6000);
    rig.send(Command::PadRelease { pad: 0 });
    rig.send(press(1));
    rig.render(128);
    let events = rig.drain();
    assert_eq!(events.len(), CAPACITY);
    let accepted: Vec<_> = events
        .iter()
        .copied()
        .filter(|e| matches!(e.kind, Kind::Onset { .. }))
        .collect();
    assert_eq!(accepted.len(), 1);
    let boundary = rig.lose_audio(Command::PatternRecordEnd { pass: 7 });
    assert_eq!(
        (
            boundary.dropped_events,
            boundary.dropped_onsets,
            boundary.captured_onsets
        ),
        (2, 1, 1)
    );
    assert_eq!(
        rig.recovery.interrupted(accepted[0], boundary),
        Some(Hit::new(0, 0, 100, 240).unwrap())
    );
    let omitted = Event {
        sequence: boundary.sequence,
        gate: Some(1),
        ..accepted[0]
    };
    assert_eq!(rig.recovery.interrupted(omitted, boundary), None);
    for event in events.into_iter().skip(1) {
        assert_eq!(rig.recovery.interrupted(event, boundary), None);
    }
    assert!(rig.drain().is_empty());
}

#[test]
fn interrupted_duration_uses_existing_minimum_midpoint_and_maximum_rules() {
    for (frames, ticks) in [(0, 1), (4, 1), (5, 1), (14, 1), (15, 2), (24, 2), (25, 3)] {
        let mut rig = Rig::at_rate(SR, 300.0, vec![]);
        rig.start(7);
        rig.send(press(0));
        rig.render(frames);
        let onset = rig.drain()[0];
        let boundary = rig.lose_audio(Command::PatternRecordEnd { pass: 7 });
        assert_eq!(
            rig.recovery
                .interrupted(onset, boundary)
                .unwrap()
                .gate_ticks(),
            ticks
        );
    }
    for (frames, ticks) in [
        (49_151, MAX_GATE_TICKS - 5),
        (49_152, MAX_GATE_TICKS),
        (49_153, MAX_GATE_TICKS),
    ] {
        let mut rig = Rig::at_rate(960, 300.0, vec![]);
        rig.start(7);
        rig.send(press(0));
        rig.render(frames);
        let onset = rig.drain()[0];
        let boundary = rig.lose_audio(Command::PatternRecordEnd { pass: 7 });
        assert_eq!(
            rig.recovery
                .interrupted(onset, boundary)
                .unwrap()
                .gate_ticks(),
            ticks
        );
    }
}

#[test]
fn interrupted_recovery_never_extends_a_known_gate_or_crosses_its_boundary() {
    let mut rig = Rig::new();
    rig.start(7);
    rig.send(press(0));
    rig.render(3000);
    let onset = rig.drain()[0];
    let earlier = rig.state();
    rig.render(3000);
    rig.send(Command::PadRelease { pad: 0 });
    rig.render(3000);
    let later = rig.lose_audio(Command::PatternRecordEnd { pass: 7 });
    assert_eq!(rig.recovery.completed(onset).unwrap().gate_ticks(), 240);
    assert_eq!(
        rig.recovery
            .interrupted(onset, earlier)
            .unwrap()
            .gate_ticks(),
        120
    );
    assert_eq!(
        rig.recovery.interrupted(onset, later).unwrap().gate_ticks(),
        240
    );
}

fn playback(row: Hit, laps: usize, frames: usize) -> Wave {
    let mut actual = Rig::at_rate(SR, 120.0, vec![row]);
    let mut reference = Rig::new();
    actual.send(Command::TransportPlay);
    reference.send(Command::TransportPlay);
    let mut commands = Vec::new();
    for lap in 0..laps {
        let at = lap * 96_000 + row.tick() as usize * 25;
        commands.push((at, press(row.pad())));
        commands.push((
            at + row.gate_ticks() as usize * 25,
            Command::PadRelease { pad: row.pad() },
        ));
    }
    commands.sort_by_key(|(frame, _)| *frame);
    let (mut wave, mut expected) = ((Vec::new(), Vec::new()), (Vec::new(), Vec::new()));
    let mut at = 0;
    for (frame, command) in commands {
        append(&mut wave, actual.render(frame - at));
        append(&mut expected, reference.render(frame - at));
        reference.send(command);
        at = frame;
    }
    append(&mut wave, actual.render(frames - at));
    append(&mut expected, reference.render(frames - at));
    compare(&wave, &expected);
    for channel in [&wave.0, &wave.1] {
        assert_eq!(checks::peak(&channel[frames - 4800..]), 0.0);
    }
    wave
}

#[test]
fn interrupted_owner_render_keeps_known_short_gate_and_repeats_recovered_long_gate() {
    let mut known = Rig::new();
    known.start(7);
    known.send(press(0));
    known.render(6000);
    let onset = known.drain()[0];
    known.send(Command::PadRelease { pad: 0 });
    known.send(Command::PatternRecordEnd { pass: 7 });
    known.render(0);
    let short = known.recovery.interrupted(onset, known.state()).unwrap();
    assert_eq!(short.gate_ticks(), 240);
    let mut interrupted = Rig::new();
    interrupted.start(8);
    interrupted.send(press(0));
    interrupted.render(12_000);
    let pending = interrupted.drain()[0];
    let boundary = interrupted.lose_audio(Command::PatternRecordEnd { pass: 8 });
    let long = interrupted.recovery.interrupted(pending, boundary).unwrap();
    assert_eq!(long.gate_ticks(), 480);
    let mut wave = playback(short, 1, 48_000);
    append(&mut wave, playback(long, 2, 168_000));
    for channel in [&wave.0, &wave.1] {
        assert_eq!(checks::first_non_finite(channel), None);
        assert!((0.02..0.04).contains(&checks::peak(channel)));
        assert!(checks::max_step(channel) < 0.004);
        let attacks = checks::onsets(channel, 0.000001, 1024);
        assert_eq!(attacks.len(), 3);
        for (got, wanted) in attacks.iter().zip([0, 48_000, 144_000]) {
            assert!(got.abs_diff(wanted) <= 1);
        }
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../renders/97-pattern-interrupted.wav");
    engine::offline::write_wav(&path, SR, &wave.0, &wave.1).unwrap();
    println!(
        "97-pattern-interrupted.wav: {} frames, stereo float32, 48 kHz; gates 240 / 480 / 480 ticks",
        wave.0.len()
    );
    println!(
        "peaks: {} / {}; maximum steps: {} / {}",
        checks::peak(&wave.0),
        checks::peak(&wave.1),
        checks::max_step(&wave.0),
        checks::max_step(&wave.1)
    );
}

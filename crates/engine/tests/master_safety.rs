use engine::checks::{max_step, peak};
use engine::offline::write_wav;
use engine::rt::{Realtime, Stats};
use engine::{Command, Engine, FX_SLOTS, MAX_VOICES};
use std::path::Path;
use std::sync::Arc;

const RATE: u32 = 48_000;
const DELAY: usize = 72;
const CEILING: f32 = 0.966_050_86;

fn tone(frames: usize, freq: f32, rate: u32, level: f32) -> Vec<f32> {
    (0..frames)
        .map(|i| {
            (i as f32 * std::f32::consts::TAU * freq / rate as f32 + std::f32::consts::FRAC_PI_4)
                .sin()
                * level
        })
        .collect()
}

fn note(engine: &mut Engine, pad: usize) {
    engine.handle(0, Command::PadPress { pad, velocity: 1.0 });
}

fn start(rate: u32, sound: Vec<f32>) -> Engine {
    let mut engine = Engine::with_sounds(rate, engine::click(rate), vec![sound]);
    engine.enable_master_output();
    engine
}

/// Independent 48-tap Blackman-windowed sinc monitor at four times the sample rate.
fn reference_peak(samples: &[f32]) -> f64 {
    let sinc = |x: f64| {
        let angle = x * std::f64::consts::PI;
        if x == 0.0 { 1.0 } else { angle.sin() / angle }
    };
    let coeff: [[f64; 48]; 3] = std::array::from_fn(|phase| {
        let mut coeff = std::array::from_fn(|tap| {
            let x = tap as f64 - 23.0 - (phase + 1) as f64 * 0.25;
            let a = std::f64::consts::PI * x / 24.0;
            sinc(x) * (0.42 + 0.5 * a.cos() + 0.08 * (2.0 * a).cos())
        });
        let sum: f64 = coeff.iter().sum();
        coeff.iter_mut().for_each(|weight| *weight /= sum);
        coeff
    });
    samples
        .windows(48)
        .fold(f64::from(peak(samples)), |peak, window| {
            coeff.iter().fold(peak, |peak, weights| {
                let value: f64 = window
                    .iter()
                    .zip(weights)
                    .map(|(&v, &w)| f64::from(v) * w)
                    .sum();
                peak.max(value.abs())
            })
        })
}

fn render(rt: &mut Engine, frames: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let (mut left, mut right) = (vec![0.0; frames], vec![0.0; frames]);
    let (mut master_l, mut master_r) = (Vec::with_capacity(frames), Vec::with_capacity(frames));
    for (l, r) in left.chunks_mut(128).zip(right.chunks_mut(128)) {
        rt.process_with_master(l, r, |_, l, r| {
            master_l.extend_from_slice(l);
            master_r.extend_from_slice(r);
        });
    }
    (left, right, master_l, master_r)
}

/// Scenario "The loud master stays below the sample ceiling".
#[test]
fn the_loud_master_stays_below_the_sample_ceiling() {
    let mut e = start(RATE, tone(12_000, 12_000.0, RATE, 0.043));
    let stats = Arc::new(Stats::default());
    e.set_stats(stats.clone());
    e.handle(0, Command::SetMetronome { enabled: true });
    e.handle(0, Command::TransportPlay);
    for pad in 0..MAX_VOICES {
        e.handle(
            0,
            Command::SetPadSends {
                pad,
                sends: [0.30; FX_SLOTS],
            },
        );
        note(&mut e, pad);
    }
    let (left, right, master_l, master_r) = render(&mut e, 24_000);
    for channel in [&master_l, &master_r] {
        assert!(peak(channel) <= CEILING);
        let actual = reference_peak(channel);
        assert!(actual <= 1.0, "4x reference peak {actual}");
    }
    assert!(peak(&left) <= 1.0 && peak(&right) <= 1.0);
    assert!(stats.take_limiting());
    assert!(!stats.take_limiting());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders");
    write_wav(&root.join("111-master-loud.wav"), RATE, &left, &right).unwrap();
    write_wav(
        &root.join("111-master-record.wav"),
        RATE,
        &master_l,
        &master_r,
    )
    .unwrap();
}

/// Scenario "A normal master keeps its level".
#[test]
fn a_normal_master_keeps_its_level() {
    for rate in [44_100, 48_000, 96_000] {
        let sound = tone(3500, 240.0, rate, 0.24);
        for block in [1, 128, 1024, 137] {
            let mut engine = start(rate, sound.clone());
            let mut reference = Engine::with_sounds(rate, engine::click(rate), vec![sound.clone()]);
            let stats = Arc::new(Stats::default());
            engine.set_stats(stats.clone());
            for target in [&mut engine, &mut reference] {
                target.handle(0, Command::SetMetronome { enabled: true });
                target.handle(0, Command::TransportPlay);
                note(target, 0);
            }
            let latency = (rate as f64 * 0.0015).round() as usize;
            let frames = 4096;
            let run = |target: &mut Engine| {
                let (mut left, mut right) = (vec![0.0; frames], vec![0.0; frames]);
                for (left, right) in left.chunks_mut(block).zip(right.chunks_mut(block)) {
                    target.process(left, right);
                }
                (left, right)
            };
            let (left, right) = run(&mut engine);
            let (ref_l, ref_r) = run(&mut reference);
            assert_eq!(peak(&left[..latency]), 0.0);
            assert_eq!(&left[latency..], &ref_l[..frames - latency]);
            assert_eq!(&right[latency..], &ref_r[..frames - latency]);
            assert!(!stats.take_limiting());
            if rate == RATE && block == 128 {
                let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders");
                write_wav(&root.join("111-master-normal.wav"), RATE, &left, &right).unwrap();
            }
        }
    }
}

/// Scenario "Panic meets its deadline through the limiter".
#[test]
fn panic_meets_its_deadline_through_the_limiter() {
    let (mut tx, rx) = rtrb::RingBuffer::new(16);
    let engine = start(RATE, tone(24_000, 160.0, RATE, 1.4));
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), RATE);
    for action in [
        Command::SetPadSends {
            pad: 0,
            sends: [1.0; FX_SLOTS],
        },
        Command::SetMetronome { enabled: true },
        Command::TransportPlay,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    ] {
        tx.push(action).unwrap();
    }
    let mut lead = [0.0; 512];
    let mut right = lead;
    rt.process(&mut lead, &mut right);
    assert!(peak(&lead) > 0.5);
    let mut stopped = [0.0; 512];
    let mut other = stopped;
    tx.push(Command::Panic).unwrap();
    rt.process(&mut stopped, &mut other);
    assert_eq!(peak(&stopped[240..]), 0.0);
    assert_eq!(peak(&other[240..]), 0.0);
    assert!(stopped[..DELAY].iter().any(|&value| value != 0.0));
    assert!(max_step(&stopped) < 0.2);
    assert_eq!(stats.transport(0).unwrap().ticks, 0.0);
    tx.push(Command::PadPress {
        pad: 0,
        velocity: 1.0,
    })
    .unwrap();
    rt.process(&mut lead, &mut right);
    assert!(peak(&lead[DELAY..]) > 0.3);
}

/// Scenario "A return changes only the selected effect".
#[test]
fn a_return_changes_only_the_selected_effect() {
    use engine::fx::FxSettings;
    let build = |sends| {
        let sample = tone(16_000, 185.0, RATE, 0.12);
        let mut engine = start(RATE, sample);
        for (slot, ms) in [18.0, 31.0].into_iter().enumerate() {
            engine.handle(
                0,
                Command::SetFx {
                    slot,
                    settings: FxSettings::Delay {
                        time_ms: ms,
                        feedback: 0.52,
                    },
                },
            );
        }
        engine.handle(0, Command::SetPadSends { pad: 0, sends });
        note(&mut engine, 0);
        engine
    };
    let mut changed = build([0.6, 0.6]);
    let mut full = build([0.6, 0.6]);
    let mut second = build([0.0, 0.6]);
    let (lead, lead_r, _, _) = render(&mut changed, 4_096);
    render(&mut full, 4_096);
    render(&mut second, 4_096);

    let edit = |engine: &mut Engine, gain| engine.handle(0, Command::SetFxReturn { slot: 0, gain });
    edit(&mut changed, 0.5);
    let (half, half_r, _, _) = render(&mut changed, 4_096);
    let (both, both_r, _, _) = render(&mut full, 4_096);
    let (one, one_r, _, _) = render(&mut second, 4_096);
    for (actual, full, other) in [(&half, &both, &one), (&half_r, &both_r, &one_r)] {
        let settled = DELAY + 240;
        assert!(
            full[settled..]
                .iter()
                .zip(&other[settled..])
                .any(|(a, b)| (a - b).abs() > 0.035)
        );
        for i in settled..actual.len() {
            assert!((actual[i] - (full[i] + other[i]) * 0.5).abs() < 1e-6);
        }
    }

    edit(&mut changed, 0.0);
    let (muted, muted_r, _, _) = render(&mut changed, 4_096);
    let (one, one_r, _, _) = render(&mut second, 4_096);
    let settled = DELAY + 240;
    assert_eq!(&muted[settled..], &one[settled..]);
    assert_eq!(&muted_r[settled..], &one_r[settled..]);

    let left = [lead, half, muted].concat();
    let right = [lead_r, half_r, muted_r].concat();
    assert!(max_step(&left[4_000..]) < 0.02 && max_step(&right[4_000..]) < 0.02);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders");
    write_wav(&root.join("111-fx-returns.wav"), RATE, &left, &right).unwrap();
}

#[test]
fn a_project_switch_drains_the_old_device_output_before_ready() {
    use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
    use std::sync::atomic::Ordering::{Acquire, Release};

    let (mut tx, rx) = rtrb::RingBuffer::new(16);
    let state = Arc::new(ProjectState::default());
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::with_project(
        start(RATE, tone(8_000, 240.0, RATE, 1.3)),
        vec![rx],
        stats,
        RATE,
        state.clone(),
    );
    let mut send = |epoch, command| {
        tx.push(ProjectCommand {
            epoch,
            ready_at_send: epoch == 0,
            command,
        })
        .unwrap()
    };
    send(
        0,
        ProjectMessage::Command(Command::PadPress {
            pad: 0,
            velocity: 1.0,
        }),
    );
    let mut left = [0.0; 512];
    let mut right = left;
    rt.process(&mut left, &mut right);
    assert!(peak(&left) > 0.5);

    send(1, ProjectMessage::Ready);
    state.epoch.store(1, Release);
    for frame in 0..4_096 {
        rt.process(&mut left[..1], &mut right[..1]);
        if frame >= 240 {
            assert_eq!((left[0], right[0]), (0.0, 0.0));
        }
        if state.ready.load(Acquire) == 1 {
            assert!(frame >= 240);
            assert_eq!((left[0], right[0]), (0.0, 0.0));
            return;
        }
    }
    panic!("project restore did not complete");
}

//! The pad filter (44): offline renders of a hat, a kick and a held tone with the filter.
//! Spec scenarios: "Behind a wall", "The rumble goes", "Off is untouched", "Turn while it
//! sounds". The renders go to `renders/44-*.wav` for the owner's ears.

use std::path::Path;

use engine::checks::{first_non_finite, max_step, peak};
use engine::filter::{FilterKind, FilterSettings};
use engine::offline::{Event, Scenario, render, write_wav};
use engine::{Command, Engine, PadMode};

const SR: u32 = 48_000;

fn at(frame: usize, cmd: Command) -> Event {
    Event { frame, cmd }
}

fn filter(kind: FilterKind, cutoff_hz: f32, resonance: f32) -> Command {
    Command::SetPadFilter {
        pad: 0,
        settings: FilterSettings {
            kind,
            cutoff_hz,
            resonance,
        },
    }
}

fn press() -> Command {
    Command::PadPress {
        pad: 0,
        velocity: 1.0,
    }
}

/// Renders pad 0 playing `sound` with the events, `frames` long.
fn run(sound: &[f32], frames: usize, events: Vec<Event>) -> Vec<f32> {
    let mut engine = Engine::with_sounds(SR, vec![], vec![sound.to_vec()]);
    let sc = Scenario {
        sample_rate: SR,
        frames,
        events,
    };
    render(&mut engine, &sc, 128).0
}

/// Energy of `buf` between `lo` and `hi` Hz: a plain DFT over the bins in that band.
fn band_energy(buf: &[f32], lo: f32, hi: f32) -> f64 {
    let n = buf.len();
    let bin = SR as f32 / n as f32;
    let (first, last) = ((lo / bin).ceil() as usize, (hi / bin).floor() as usize);
    (first.max(1)..=last)
        .map(|k| {
            let w = std::f64::consts::TAU * k as f64 / n as f64;
            let (re, im) = buf
                .iter()
                .enumerate()
                .fold((0.0, 0.0), |(re, im), (i, &s)| {
                    let a = w * i as f64;
                    (re + s as f64 * a.cos(), im - s as f64 * a.sin())
                });
            re * re + im * im
        })
        .sum()
}

/// A hi-hat: noise that dies in 60 ms (a fixed generator, the same every run).
fn hat() -> Vec<f32> {
    let mut x: u32 = 0x1234_5678;
    (0..SR as usize / 10)
        .map(|i| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            let noise = x as f32 / u32::MAX as f32 * 2.0 - 1.0;
            0.6 * noise * (-(i as f32) / (0.015 * SR as f32)).exp()
        })
        .collect()
}

/// A kick with a sub rumble: 25 Hz and 55 Hz dying over half a second.
fn kick() -> Vec<f32> {
    (0..SR as usize)
        .map(|i| {
            let t = i as f32 / SR as f32;
            let env = (-t / 0.25).exp();
            env * (0.4 * (std::f32::consts::TAU * 25.0 * t).sin()
                + 0.4 * (std::f32::consts::TAU * 55.0 * t).sin())
        })
        .collect()
}

fn out(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../renders")
        .join(name)
}

/// "Behind a wall": the low-pass at 1 kHz keeps less than a tenth of the hat above 4 kHz.
#[test]
fn behind_a_wall() {
    let hat = hat();
    let len = hat.len() + 1_000;
    let dry = run(&hat, len, vec![at(0, press())]);
    let wet = run(
        &hat,
        len,
        vec![
            at(0, filter(FilterKind::LowPass, 1_000.0, 0.0)),
            at(0, press()),
        ],
    );
    assert_eq!(first_non_finite(&wet), None);
    assert!(peak(&wet) < 1.0, "no clipping");
    let (d, w) = (
        band_energy(&dry, 4_000.0, 20_000.0),
        band_energy(&wet, 4_000.0, 20_000.0),
    );
    assert!(w < d / 10.0, "above 4 kHz: {w:.3} of {d:.3}");
    let mut both = dry.clone();
    both.extend(vec![0.0; SR as usize / 4]);
    both.extend(&wet);
    write_wav(&out("44-filter-hat.wav"), SR, &both, &both).unwrap();
}

/// "The rumble goes": the high-pass at 60 Hz keeps less than a tenth of the kick below 30 Hz.
#[test]
fn the_rumble_goes() {
    let kick = kick();
    let len = kick.len();
    let dry = run(&kick, len, vec![at(0, press())]);
    let wet = run(
        &kick,
        len,
        vec![
            at(0, filter(FilterKind::HighPass, 60.0, 0.0)),
            at(0, press()),
        ],
    );
    assert_eq!(first_non_finite(&wet), None);
    assert!(peak(&wet) < 1.0, "no clipping");
    let (d, w) = (band_energy(&dry, 1.0, 30.0), band_energy(&wet, 1.0, 30.0));
    assert!(w < d / 10.0, "below 30 Hz: {w:.3} of {d:.3}");
    let mut both = dry.clone();
    both.extend(vec![0.0; SR as usize / 4]);
    both.extend(&wet);
    write_wav(&out("44-filter-kick.wav"), SR, &both, &both).unwrap();
}

/// "Off is untouched": a pad with the filter off sounds sample for sample as one that never
/// heard of the filter, also after the filter was on and turned off again.
#[test]
fn off_is_untouched() {
    let hat = hat();
    let len = hat.len();
    let never = run(&hat, len, vec![at(0, press())]);
    let off = run(
        &hat,
        len,
        vec![
            at(0, filter(FilterKind::LowPass, 500.0, 0.5)),
            at(0, filter(FilterKind::Off, 500.0, 0.5)),
            at(0, press()),
        ],
    );
    assert_eq!(never, off);
}

/// "Turn while it sounds": a held tone under the low-pass at 200 Hz, the cutoff jumps to
/// 8 kHz: the sound opens up with no step larger than the tone's own. Without resonance:
/// a resonant sweep through the tone's pitch is louder on purpose (the render below).
#[test]
fn turn_while_it_sounds() {
    let tone: Vec<f32> = (0..SR as usize)
        .map(|i| 0.5 * (std::f32::consts::TAU * 440.0 * i as f32 / SR as f32).sin())
        .collect();
    let len = SR as usize / 2;
    let own = max_step(&run(&tone, len, vec![at(0, press())]));
    let wet = run(
        &tone,
        len,
        vec![
            at(
                0,
                Command::SetPadMode {
                    pad: 0,
                    mode: PadMode::Gate,
                },
            ),
            at(0, filter(FilterKind::LowPass, 200.0, 0.0)),
            at(0, press()),
            at(len / 2, filter(FilterKind::LowPass, 8_000.0, 0.0)),
        ],
    );
    assert_eq!(first_non_finite(&wet), None);
    let step = max_step(&wet);
    assert!(
        step <= own * 1.05,
        "step {step} against the tone's own {own}"
    );
    // it opened: the second half is louder than the first
    let half = len / 2;
    assert!(peak(&wet[half + 2_400..]) > 2.0 * peak(&wet[2_400..half]));
    write_wav(&out("44-filter-open.wav"), SR, &wet, &wet).unwrap();
}

/// For the ears: a held saw-like tone swept from 200 Hz to 8 kHz and back with the
/// resonance at 60 %; no NaN, no clipping.
#[test]
fn a_resonant_sweep() {
    let tone: Vec<f32> = (0..2 * SR as usize)
        .map(|i| {
            let t = i as f32 * 110.0 / SR as f32;
            0.3 * (2.0 * (t - t.floor()) - 1.0)
        })
        .collect();
    let len = 2 * SR as usize;
    let mut events = vec![
        at(
            0,
            Command::SetPadMode {
                pad: 0,
                mode: PadMode::Gate,
            },
        ),
        at(0, filter(FilterKind::LowPass, 200.0, 0.6)),
        at(0, press()),
    ];
    // a knob turned up and down in 40 steps
    for n in 0..40 {
        let x = if n < 20 {
            n as f32 / 19.0
        } else {
            (39 - n) as f32 / 19.0
        };
        let hz = 200.0 * 40f32.powf(x);
        events.push(at(n * len / 40, filter(FilterKind::LowPass, hz, 0.6)));
    }
    let wet = run(&tone, len, events);
    assert_eq!(first_non_finite(&wet), None);
    assert!(peak(&wet) < 1.0, "no clipping: {}", peak(&wet));
    write_wav(&out("44-filter-sweep.wav"), SR, &wet, &wet).unwrap();
}

use std::path::Path;

use engine::checks::{first_loud, first_non_finite, max_step, onsets, peak};
use engine::offline::{Event, Scenario, render, write_wav};
use engine::{Command, Engine, MAX_VOICES, PadMode};

const SR: u32 = 48_000;
const MS: usize = SR as usize / 1000;
/// The steepest slope of pad 0's test tone (110 Hz, amplitude 0.3) is about 0.004 per sample.
/// Cutting that tone abruptly jumps up to 0.3; a smooth fade stays within the slope.
const NO_CLICK: f32 = 0.01;

fn at(frame: usize, cmd: Command) -> Event {
    Event { frame, cmd }
}

fn press(pad: usize, velocity: f32) -> Command {
    Command::PadPress { pad, velocity }
}

fn run(frames: usize, events: Vec<Event>) -> Vec<f32> {
    let sc = Scenario {
        sample_rate: SR,
        frames,
        events,
    };
    render(&mut Engine::new(SR), &sc, 128).0
}

#[test]
fn pad_starts_on_time() {
    let out = run(500 * MS, vec![at(1000, press(3, 1.0))]);
    assert_eq!(first_loud(&out, 0..1000, 0.0), None);
    let got = onsets(&out, 1e-6, 100);
    assert!(got[0].abs_diff(1000) <= 1, "hit at {}", got[0]);
}

#[test]
fn velocity_scales_level() {
    let full = peak(&run(200 * MS, vec![at(0, press(0, 1.0))]));
    let half = peak(&run(200 * MS, vec![at(0, press(0, 0.5))]));
    assert!((half / full - 0.5).abs() < 0.01, "{half} / {full}");
}

/// Scenario "Gate by release" (pad-playback): the sound after the release, the control side is `gate_by_release`.
#[test]
fn gate_stops_on_release_without_click() {
    let release = 300 * MS + 37;
    let out = run(
        600 * MS,
        vec![
            at(
                0,
                Command::SetPadMode {
                    pad: 0,
                    mode: PadMode::Gate,
                },
            ),
            at(0, press(0, 1.0)),
            at(release, Command::PadRelease { pad: 0 }),
        ],
    );
    assert!(max_step(&out) < NO_CLICK, "jump {}", max_step(&out));
    assert_eq!(first_loud(&out, release + 6 * MS..out.len(), 0.0), None);
}

#[test]
fn one_shot_ignores_release() {
    let out = run(
        600 * MS,
        vec![
            at(0, press(0, 1.0)),
            at(300 * MS, Command::PadRelease { pad: 0 }),
        ],
    );
    assert!(peak(&out[400 * MS..500 * MS]) > 0.01);
}

/// Scenario "Fast roll" (pad-playback).
#[test]
fn retrigger_cuts_previous_voice_without_click() {
    let mut events = Vec::new();
    // a roll of 16 hits per second
    for i in 0..16 {
        events.push(at(i * 62 * MS + 11, press(0, 1.0)));
    }
    let out = run(1200 * MS, events);
    // the hit itself has a 2 ms attack, not a click; look for the previous voice being cut
    assert!(max_step(&out) < NO_CLICK, "jump {}", max_step(&out));
    assert!(peak(&out) <= 0.6, "voices are not cut: peak {}", peak(&out));
    assert_eq!(first_non_finite(&out), None);
    // every hit sounds: once the cut voice has faded, each one plays as a lone hit does
    let lone = run(100 * MS, vec![at(11, press(0, 1.0))]);
    for i in 0..16 {
        for k in 10 * MS..20 * MS {
            let (got, want) = (out[i * 62 * MS + k], lone[k]);
            assert!(
                (got - want).abs() < 1e-4,
                "hit {i}, frame {k}: {got} vs {want}"
            );
        }
    }
}

#[test]
fn voice_overflow_fades_oldest() {
    let mut e = Engine::with_sounds(SR, vec![], vec![vec![1.0; SR as usize]]);
    for pad in 0..MAX_VOICES + 1 {
        e.handle(0, press(pad, 0.01));
    }
    let (mut l, mut r) = (vec![0.0; 10 * MS], vec![0.0; 10 * MS]);
    e.process(&mut l, &mut r);
    // first sample: all 33 sound, the oldest has just started fading
    assert!((l[0] - 0.33).abs() < 1e-4, "{}", l[0]);
    // after 5 ms the oldest is silent, 32 sound
    assert!((l[6 * MS] - 0.32).abs() < 1e-4, "{}", l[6 * MS]);
    assert!(max_step(&l) < 0.01 / 100.0);
}

#[test]
fn pad_gain_changes_smoothly() {
    let out = run(
        400 * MS,
        vec![
            at(0, press(0, 1.0)),
            at(100 * MS + 5, Command::SetPadGain { pad: 0, gain: 0.1 }),
        ],
    );
    assert!(max_step(&out) < NO_CLICK);
    let before = peak(&out[80 * MS..100 * MS]);
    let after = peak(&out[110 * MS..130 * MS]);
    assert!(after < before * 0.15, "{after} vs {before}");
}

/// Render for the owner: velocity, all pads of bank 1, gate, bank 2.
#[test]
fn render_pads_velocity() {
    let mut events = Vec::new();
    let mut t = 0;
    for v in [1.0, 0.8, 0.5, 0.25, 0.12] {
        events.push(at(t, press(0, v)));
        t += 400 * MS;
    }
    for pad in 0..16 {
        events.push(at(t, press(pad, 0.8)));
        t += 150 * MS;
    }
    t += 600 * MS;
    events.push(at(
        t,
        Command::SetPadMode {
            pad: 5,
            mode: PadMode::Gate,
        },
    ));
    for hold in [80, 300] {
        events.push(at(t, press(5, 0.8)));
        events.push(at(t + hold * MS, Command::PadRelease { pad: 5 }));
        t += 600 * MS;
    }
    for pad in 16..32 {
        events.push(at(t, press(pad, 0.8)));
        t += 150 * MS;
    }
    let sc = Scenario {
        sample_rate: SR,
        frames: t + SR as usize,
        events,
    };
    let (l, r) = render(&mut Engine::new(SR), &sc, 128);
    assert_eq!(first_non_finite(&l), None);
    assert!(peak(&l) <= 0.9, "peak {}", peak(&l));
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/02-pads-velocity.wav");
    write_wav(&out, SR, &l, &r).unwrap();
}

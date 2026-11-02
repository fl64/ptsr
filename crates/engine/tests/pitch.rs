//! Pad pitch: a voice reads its sample faster or slower, a change on a sounding
//! voice glides. Spec scenarios: "An octave up", "Glide while it sounds" (change 19).

use std::path::Path;
use std::sync::Arc;

use engine::checks::{first_loud, first_non_finite, max_step, onsets, peak};
use engine::offline::{Event, Scenario, render, write_wav};
use engine::{Command, Engine, PITCH_LIMIT, Sample};

const SR: u32 = 48_000;
const MS: usize = SR as usize / 1000;

fn at(frame: usize, cmd: Command) -> Event {
    Event { frame, cmd }
}

fn press(pad: usize, velocity: f32) -> Command {
    Command::PadPress { pad, velocity }
}

fn pitch(pad: usize, semitones: i8) -> Command {
    Command::SetPadPitch { pad, semitones }
}

fn run(frames: usize, events: Vec<Event>) -> Vec<f32> {
    let sc = Scenario {
        sample_rate: SR,
        frames,
        events,
    };
    render(&mut Engine::new(SR), &sc, 128).0
}

/// Span of the sound in the buffer: from the first to the last sample louder than `eps`.
fn span(buf: &[f32], eps: f32) -> usize {
    let first = buf.iter().position(|s| s.abs() > eps).unwrap();
    let last = buf.iter().rposition(|s| s.abs() > eps).unwrap();
    last - first + 1
}

/// Rising zero crossings per second in the window: the frequency of a sine.
fn crossings_per_second(buf: &[f32], from: usize, to: usize) -> f64 {
    let n = buf[from..to]
        .windows(2)
        .filter(|w| w[0] < 0.0 && w[1] >= 0.0)
        .count();
    n as f64 * SR as f64 / (to - from) as f64
}

#[test]
fn all_benchmark_voices_stay_sounding() {
    let frames = 20_000 * 128;
    let sound = vec![0.001; frames * 2 + 1];
    let mut engine = Engine::with_sounds(SR, vec![], vec![sound]);
    for pad in 0..engine::MAX_VOICES {
        engine.handle(0, pitch(pad, 7));
        engine.handle(0, press(pad, 1.0));
    }
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    for _ in 0..20_000 {
        engine.process(&mut left, &mut right);
    }
    assert!(left.iter().all(|&s| s > 0.02 && s.is_finite()));
    assert!(right.iter().all(|&s| s > 0.02 && s.is_finite()));
}

/// An octave up: the same pad plays an octave higher and half as long.
#[test]
fn an_octave_up_plays_higher_and_half_as_long() {
    let base = run(2 * SR as usize, vec![at(10 * MS, press(0, 1.0))]);
    let up = run(
        2 * SR as usize,
        vec![at(0, pitch(0, 12)), at(10 * MS, press(0, 1.0))],
    );
    assert_eq!(first_non_finite(&up), None);
    assert_eq!(first_loud(&up, 0..10 * MS, 0.0), None);
    let got = onsets(&up, 1e-6, 100 * MS);
    assert!(got[0].abs_diff(10 * MS) <= 1, "hit at {}", got[0]);
    let (len0, len12) = (span(&base, 1e-6), span(&up, 1e-6));
    assert!(
        (len12 as f64 / len0 as f64 - 0.5).abs() < 0.05,
        "{len12} / {len0}"
    );
    let f0 = crossings_per_second(&base, 100 * MS, 300 * MS);
    let f12 = crossings_per_second(&up, 100 * MS, 300 * MS);
    assert!((f12 / f0 - 2.0).abs() < 0.1, "{f12} / {f0}");
}

/// Glide while it sounds: two pitch changes on a long voice, no jump anywhere.
#[test]
fn glide_while_it_sounds_without_click() {
    let tone = Sample::mono(
        (0..3 * SR as usize)
            .map(|i| 0.3 * (std::f32::consts::TAU * 110.0 * i as f32 / SR as f32).sin())
            .collect(),
    );
    let out = run(
        1200 * MS,
        vec![
            at(
                0,
                Command::SetPadSound {
                    pad: 0,
                    sound: Some(Arc::new(tone)),
                },
            ),
            at(50 * MS, press(0, 1.0)),
            at(400 * MS, pitch(0, 12)),
            at(800 * MS, pitch(0, -PITCH_LIMIT)),
        ],
    );
    assert_eq!(first_non_finite(&out), None);
    // the steepest slope of the 220 Hz tone is about 0.009 per sample;
    // an abrupt rate change would jump by up to 0.6
    assert!(max_step(&out) < 0.012, "jump {}", max_step(&out));
    assert!(peak(&out[900 * MS..1000 * MS]) > 0.01);
}

/// One pad at 0, +12 and −12 for the owner's ears, with the automated checks.
#[test]
fn render_one_pad_at_0_up12_and_down12() {
    let events = vec![
        at(50 * MS, press(0, 1.0)),
        at(1500 * MS, pitch(0, 12)),
        at(1550 * MS, press(0, 1.0)),
        at(3000 * MS, pitch(0, -12)),
        at(3050 * MS, press(0, 1.0)),
    ];
    let sc = Scenario {
        sample_rate: SR,
        frames: 6 * SR as usize,
        events,
    };
    let (left, right) = render(&mut Engine::new(SR), &sc, 128);
    assert_eq!(first_non_finite(&left), None);
    assert!(peak(&left) < 1.0);
    let hits = onsets(&left, 1e-6, 200 * MS);
    let wants = [50 * MS, 1550 * MS, 3050 * MS];
    assert_eq!(hits.len(), wants.len());
    for (got, want) in hits.iter().zip(wants) {
        assert!(got.abs_diff(want) <= 1, "hit at {got}, want {want}");
    }
    // silence in the gaps: the hit at +12 ends before 1100 ms of the next minute mark
    assert_eq!(first_loud(&left, 1100 * MS..1500 * MS, 0.0), None);
    assert_eq!(first_loud(&left, 2600 * MS..3000 * MS, 0.0), None);
    // length check: +12 is half, −12 is twice the pitch-0 hit
    let len0 = span(&left[50 * MS..1100 * MS], 1e-6);
    let len12 = span(&left[1550 * MS..2600 * MS], 1e-6);
    let lenm12 = span(&left[3050 * MS..5300 * MS], 1e-6);
    assert!(
        (len12 as f64 / len0 as f64 - 0.5).abs() < 0.05,
        "{len12}/{len0}"
    );
    assert!(
        (lenm12 as f64 / len0 as f64 - 2.0).abs() < 0.1,
        "{lenm12}/{len0}"
    );
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/19-pad-pitch.wav");
    write_wav(&out, SR, &left, &right).unwrap();
}

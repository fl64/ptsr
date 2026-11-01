//! Pad envelope: attack at the piece start, decay at its end, release after a gate
//! release. Spec scenarios: "Attack at the start", "Decay ends at the end of the
//! piece", "Release after the gate" (change 04b).

use std::path::Path;
use std::sync::Arc;

use engine::checks::{first_loud, first_non_finite, max_step, onsets, peak};
use engine::offline::{Event, Scenario, render, write_wav};
use engine::{Command, Engine, PadMode, Sample};

const SR: u32 = 48_000;
const MS: usize = SR as usize / 1000;

fn at(frame: usize, cmd: Command) -> Event {
    Event { frame, cmd }
}

fn press(pad: usize, velocity: f32) -> Command {
    Command::PadPress { pad, velocity }
}

fn envelope(pad: usize, attack_ms: u16, decay_ms: u16, release_ms: u16) -> Command {
    Command::SetPadEnvelope {
        pad,
        attack_ms,
        decay_ms,
        release_ms,
    }
}

fn run(frames: usize, events: Vec<Event>) -> Vec<f32> {
    let sc = Scenario {
        sample_rate: SR,
        frames,
        events,
    };
    render(&mut Engine::new(SR), &sc, 128).0
}

/// A constant sine of `secs` seconds at 0.5 amplitude: its own envelope is flat,
/// so every level change comes from the pad envelope.
fn sine(secs: f32) -> Sample {
    let n = (SR as f32 * secs) as usize;
    Sample::mono(
        (0..n)
            .map(|i| 0.5 * (std::f32::consts::TAU * 110.0 * i as f32 / SR as f32).sin())
            .collect(),
    )
}

fn with_tone(pad: usize, secs: f32) -> Command {
    Command::SetPadSound {
        pad,
        sound: Some(Arc::new(sine(secs))),
    }
}

/// Spec: "Attack at the start".
#[test]
fn attack_at_the_start_without_a_click() {
    let plain = run(
        500 * MS,
        vec![at(0, with_tone(0, 2.0)), at(10 * MS, press(0, 1.0))],
    );
    let soft = run(
        500 * MS,
        vec![
            at(0, with_tone(0, 2.0)),
            at(0, envelope(0, 50, 0, 0)),
            at(10 * MS, press(0, 1.0)),
        ],
    );
    assert_eq!(first_non_finite(&soft), None);
    // the plain pad starts at full level right away
    assert!(peak(&plain[10 * MS..25 * MS]) >= 0.45);
    // the attack ramps in: 25 ms into the hit it is still at most half level
    assert!(
        peak(&soft[10 * MS..25 * MS]) < 0.3,
        "too loud too early: {}",
        peak(&soft[10 * MS..25 * MS])
    );
    // after 50 ms the level is the same as without the attack
    assert!(peak(&soft[65 * MS..90 * MS]) >= 0.45);
    // the hit is on time and no step anywhere
    let got = onsets(&soft, 1e-6, 100 * MS);
    assert!(got[0].abs_diff(10 * MS) <= 1, "hit at {}", got[0]);
    assert!(max_step(&soft) < 0.012, "jump {}", max_step(&soft));
}

/// Spec: "Decay ends at the end of the piece".
#[test]
fn decay_ends_at_the_end_of_the_piece_in_silence() {
    let plain = run(
        2500 * MS,
        vec![at(0, with_tone(0, 2.0)), at(10 * MS, press(0, 1.0))],
    );
    let decayed = run(
        2500 * MS,
        vec![
            at(0, with_tone(0, 2.0)),
            at(0, envelope(0, 0, 200, 0)),
            at(10 * MS, press(0, 1.0)),
        ],
    );
    assert_eq!(first_non_finite(&decayed), None);
    // the decayed tail is much quieter: 110 ms before the end the envelope is at 0.55
    assert!(peak(&decayed[1900 * MS..2010 * MS]) < 0.3);
    assert!(peak(&plain[1900 * MS..2010 * MS]) >= 0.4);
    // the piece ends at the same place as before: never past it, the fade to
    // silence makes the last audible samples phase-dependent, so a window, not a point
    let end_of = |buf: &[f32]| {
        buf[..2020 * MS]
            .iter()
            .rposition(|s| s.abs() > 1e-4)
            .unwrap()
    };
    assert!(
        (2010 * MS - 100..=2010 * MS + 5).contains(&end_of(&decayed)),
        "ends at {}",
        end_of(&decayed)
    );
    assert!(end_of(&plain).abs_diff(2010 * MS) <= 5);
    // and in silence after it
    assert_eq!(first_loud(&decayed, 2011 * MS..2100 * MS, 1e-6), None);
    assert!(max_step(&decayed) < 0.012, "jump {}", max_step(&decayed));
}

/// Spec: "Release after the gate"; zero release keeps the short cut.
#[test]
fn release_after_the_gate_fades_without_a_click() {
    let long = run(
        1200 * MS,
        vec![
            at(0, with_tone(0, 3.0)),
            at(0, envelope(0, 0, 0, 300)),
            at(
                0,
                Command::SetPadMode {
                    pad: 0,
                    mode: PadMode::Gate,
                },
            ),
            at(10 * MS, press(0, 1.0)),
            at(400 * MS, Command::PadRelease { pad: 0 }),
        ],
    );
    assert_eq!(first_non_finite(&long), None);
    // the sound keeps going after the release and dies around 700 ms
    assert!(peak(&long[405 * MS..600 * MS]) > 0.01);
    // the last 10 ms of the ramp are below 2 % level
    assert!(peak(&long[690 * MS..709 * MS]) < 0.02);
    assert_eq!(first_loud(&long, 710 * MS..800 * MS, 1e-5), None);
    assert!(max_step(&long) < 0.012, "jump {}", max_step(&long));
    // zero release is the short cut it was before the envelope existed
    for zero in [false, true] {
        let mut events = vec![
            at(0, with_tone(0, 3.0)),
            at(
                0,
                Command::SetPadMode {
                    pad: 0,
                    mode: PadMode::Gate,
                },
            ),
            at(10 * MS, press(0, 1.0)),
            at(400 * MS, Command::PadRelease { pad: 0 }),
        ];
        if zero {
            events.insert(1, at(0, envelope(0, 0, 0, 0)));
        }
        let out = run(1200 * MS, events);
        assert_eq!(first_loud(&out, 415 * MS..500 * MS, 1e-5), None);
    }
}

/// An envelope change on a sounding pad follows smoothly, without a click.
#[test]
fn an_envelope_change_on_a_sounding_pad_does_not_click() {
    let out = run(
        1300 * MS,
        vec![
            at(0, with_tone(0, 3.0)),
            at(50 * MS, press(0, 1.0)),
            at(400 * MS, envelope(0, 500, 0, 0)),
            at(800 * MS, envelope(0, 0, 300, 0)),
        ],
    );
    assert_eq!(first_non_finite(&out), None);
    assert!(max_step(&out) < 0.012, "jump {}", max_step(&out));
    // back at full level after the attack is switched off again
    assert!(peak(&out[900 * MS..1000 * MS]) > 0.4);
}

/// One pad with an attack, one with a decay, one gate pad with a release,
/// for the owner's ears, with the automated checks.
#[test]
fn render_attack_decay_and_release_for_the_owner() {
    let events = vec![
        at(0, envelope(0, 50, 0, 0)),
        at(0, envelope(1, 0, 200, 0)),
        at(
            0,
            Command::SetPadMode {
                pad: 2,
                mode: PadMode::Gate,
            },
        ),
        at(0, envelope(2, 0, 0, 300)),
        at(50 * MS, press(0, 1.0)),
        at(1600 * MS, press(1, 1.0)),
        at(3100 * MS, press(2, 1.0)),
        at(3700 * MS, Command::PadRelease { pad: 2 }),
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
    let wants = [50 * MS, 1600 * MS, 3100 * MS];
    assert_eq!(hits.len(), wants.len());
    // the attack keeps the first frames near silence, so its onset is found late:
    // a fade-in starts on time even when it crosses the threshold a few frames in
    let allowed = [8, 1, 1];
    for (got, (want, diff)) in hits.iter().zip(wants.iter().zip(allowed)) {
        assert!(got.abs_diff(*want) <= diff, "hit at {got}, want {want}");
    }
    // silence in the gaps between the three sections
    assert_eq!(first_loud(&left, 1150 * MS..1590 * MS, 1e-6), None);
    assert_eq!(first_loud(&left, 2620 * MS..3090 * MS, 1e-6), None);
    // the decay dies into silence at the end of the piece, the release by 700 ms after it
    assert_eq!(first_loud(&left, 2610 * MS..2620 * MS, 5e-4), None);
    assert_eq!(first_loud(&left, 4100 * MS..4600 * MS, 1e-5), None);
    assert!(max_step(&left) < 0.012, "jump {}", max_step(&left));
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/04b-envelope.wav");
    write_wav(&out, SR, &left, &right).unwrap();
}

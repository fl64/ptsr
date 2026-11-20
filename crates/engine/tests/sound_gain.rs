//! The sound gain of a pad (138): it acts before the drive and the volume, and glides.

use engine::checks::{first_non_finite, max_step, peak};
use engine::offline::{Event, Scenario, render};
use engine::{Command, Engine};

const SR: u32 = 48_000;
const MS: usize = SR as usize / 1000;

fn at(frame: usize, cmd: Command) -> Event {
    Event { frame, cmd }
}

fn run(frames: usize, events: Vec<Event>) -> Vec<f32> {
    let sc = Scenario {
        sample_rate: SR,
        frames,
        events,
    };
    render(&mut Engine::new(SR), &sc, 128).0
}

fn db(gain: f32) -> f32 {
    10f32.powf(gain / 20.0)
}

/// Scenario "The sound gain raises the level before the volume" (pad-playback): +12 dB of
/// sound gain at a 0 dB volume plays 12 dB louder than 0 dB of gain, with no clipping and
/// no click; the volume still scales the result.
#[test]
fn the_sound_gain_raises_the_level_before_the_volume() {
    let hit = |gain_db: f32, volume: f32| {
        run(
            300 * MS,
            vec![
                at(
                    0,
                    Command::SetSoundGain {
                        pad: 0,
                        gain: db(gain_db),
                    },
                ),
                at(
                    0,
                    Command::SetPadGain {
                        pad: 0,
                        gain: volume,
                    },
                ),
                at(
                    10 * MS,
                    Command::PadPress {
                        pad: 0,
                        velocity: 0.25,
                    },
                ),
            ],
        )
    };
    let plain = hit(0.0, 1.0);
    let louder = hit(12.0, 1.0);
    assert_eq!(first_non_finite(&louder), None);
    let rise = 20.0 * (peak(&louder) / peak(&plain)).log10();
    assert!((rise - 12.0).abs() < 0.1, "{rise:.3} dB");
    assert!(peak(&louder) < 1.0, "no clipping: {}", peak(&louder));
    // a quarter hit keeps the louder tone below full scale; its slope stays near 0.004
    assert!(max_step(&louder) < 0.01, "no click: {}", max_step(&louder));
    let halved = hit(12.0, 0.5);
    assert!(
        (peak(&halved) / peak(&louder) - 0.5).abs() < 0.01,
        "the volume acts after"
    );
}

/// A sound gain change on a sounding pad glides as the volume does.
#[test]
fn the_sound_gain_changes_smoothly() {
    let out = run(
        400 * MS,
        vec![
            at(
                0,
                Command::PadPress {
                    pad: 0,
                    velocity: 1.0,
                },
            ),
            at(100 * MS + 5, Command::SetSoundGain { pad: 0, gain: 0.1 }),
        ],
    );
    assert!(max_step(&out) < 0.01, "no click: {}", max_step(&out));
    let before = peak(&out[80 * MS..100 * MS]);
    let after = peak(&out[110 * MS..130 * MS]);
    assert!(after < before * 0.15, "{after} vs {before}");
}

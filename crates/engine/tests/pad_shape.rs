use std::path::Path;
use std::sync::Arc;

use engine::checks::{first_non_finite, max_step, peak};
use engine::fx::FxSettings;
use engine::offline::{Event, Scenario, render, write_wav};
use engine::pad_shape::Shape;
use engine::{Command, Engine, PadMode, Sample};

const RATE: u32 = 48_000;
const MS: usize = RATE as usize / 1000;

fn event(frame: usize, cmd: Command) -> Event {
    Event { frame, cmd }
}

fn shape(pad: usize, shape: Shape) -> Command {
    Command::SetPadShape { pad, shape }
}

fn hit(pad: usize, velocity: f32) -> Command {
    Command::PadPress { pad, velocity }
}

fn env(pad: usize, attack_ms: u16, decay_ms: u16, release_ms: u16) -> Command {
    Command::SetPadEnvelope {
        pad,
        attack_ms,
        decay_ms,
        release_ms,
    }
}

fn sine(frames: usize, hz: f32, level: f32) -> Vec<f32> {
    (0..frames)
        .map(|t| (t as f32 * std::f32::consts::TAU * hz / RATE as f32).sin() * level)
        .collect()
}

fn engine(sample: Vec<f32>) -> Engine {
    Engine::with_sounds(RATE, vec![], vec![sample])
}

fn draw(e: &mut Engine, frames: usize, events: Vec<Event>) -> (Vec<f32>, Vec<f32>) {
    render(
        e,
        &Scenario {
            sample_rate: RATE,
            frames,
            events,
        },
        MS,
    )
}

fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 1e-6, "{actual} != {expected}");
}

fn wav(name: &str, (left, right): &(Vec<f32>, Vec<f32>)) {
    for signal in [left, right] {
        assert_eq!(first_non_finite(signal), None);
        assert!(peak(signal) < 1.0);
        assert!(max_step(signal) < 0.04, "sample step {}", max_step(signal));
        assert_eq!(peak(&signal[..MS]), 0.0);
        assert_eq!(peak(&signal[signal.len() - MS..]), 0.0);
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../renders")
        .join(name);
    write_wav(&path, RATE, left, right).unwrap();
}

/// Scenario "Pan moves a mono pad and its sends together".
#[test]
fn pan_moves_a_mono_pad_and_its_sends_together() {
    for pan in [-100, -50, 0, 50, 100] {
        let mut e = engine(vec![0.1; 5 * MS]);
        for (slot, time_ms) in [12.0, 22.0].into_iter().enumerate() {
            e.handle(
                0,
                Command::SetFx {
                    slot,
                    settings: FxSettings::Delay {
                        time_ms,
                        feedback: 0.0,
                    },
                },
            );
        }
        e.handle(
            0,
            Command::SetPadSends {
                pad: 0,
                sends: [1.0; 2],
            },
        );
        let (l, r) = draw(
            &mut e,
            35 * MS,
            vec![
                event(
                    0,
                    shape(
                        0,
                        Shape {
                            pan,
                            ..Shape::default()
                        },
                    ),
                ),
                event(0, hit(0, 1.0)),
            ],
        );
        let at = 2 * MS;
        close(l[at] * l[at] + r[at] * r[at], 0.02);
        if pan == 0 {
            close(l[at], 0.1);
            close(r[at], 0.1);
        }
        if pan.abs() == 100 {
            let (wet, silent) = if pan < 0 { (&l, &r) } else { (&r, &l) };
            assert!(peak(&wet[12 * MS..17 * MS]) > 0.01);
            assert!(peak(&wet[22 * MS..27 * MS]) > 0.01);
            assert_eq!(peak(silent), 0.0);
        }
    }

    let mut e = engine(sine(440 * MS, 190.0, 0.2));
    let audio = draw(
        &mut e,
        500 * MS,
        vec![
            event(
                0,
                shape(
                    0,
                    Shape {
                        pan: -100,
                        ..Shape::default()
                    },
                ),
            ),
            event(10 * MS, hit(0, 1.0)),
            event(150 * MS, shape(0, Shape::default())),
            event(
                300 * MS,
                shape(
                    0,
                    Shape {
                        pan: 100,
                        ..Shape::default()
                    },
                ),
            ),
            event(420 * MS, Command::Panic),
        ],
    );
    assert_eq!(peak(&audio.1[10 * MS..150 * MS]), 0.0);
    assert_eq!(peak(&audio.0[340 * MS..420 * MS]), 0.0);
    assert!(peak(&audio.1[340 * MS..420 * MS]) > 0.27);
    wav("144-pan.wav", &audio);

    let mut loud = engine(sine(20 * MS, 200.0, 1.0));
    loud.enable_master_output();
    let (_, r) = draw(
        &mut loud,
        30 * MS,
        vec![
            event(
                0,
                shape(
                    0,
                    Shape {
                        pan: 100,
                        ..Shape::default()
                    },
                ),
            ),
            event(0, hit(0, 1.0)),
        ],
    );
    assert!(peak(&r) <= 0.966_051);
    assert!(peak(&r) > 0.9);
}

/// Scenario "Stereo balance preserves the nearer channel".
#[test]
fn stereo_balance_preserves_the_nearer_channel() {
    let sample = Sample {
        left: vec![0.2; 8 * MS].into_boxed_slice(),
        right: Some(vec![0.05; 8 * MS].into_boxed_slice()),
    };
    let mut e = engine(vec![]);
    let (l, r) = draw(
        &mut e,
        10 * MS,
        vec![
            event(
                0,
                Command::SetPadSound {
                    pad: 0,
                    sound: Some(Arc::new(sample)),
                },
            ),
            event(
                0,
                shape(
                    0,
                    Shape {
                        pan: -75,
                        ..Shape::default()
                    },
                ),
            ),
            event(0, hit(0, 1.0)),
        ],
    );
    close(l[2 * MS], 0.2);
    close(r[2 * MS], 0.0125);
}

/// Scenario "Fine tune spans one semitone in each direction".
#[test]
fn fine_tune_spans_one_semitone_in_each_direction() {
    let mut e = engine(sine(1600 * MS, 440.0, 0.28));
    let audio = draw(
        &mut e,
        1400 * MS,
        vec![
            event(10 * MS, hit(0, 1.0)),
            event(
                10 * MS,
                Command::SetPadPitch {
                    pad: 0,
                    semitones: 3,
                },
            ),
            event(
                430 * MS,
                shape(
                    0,
                    Shape {
                        cents: -100,
                        ..Shape::default()
                    },
                ),
            ),
            event(
                850 * MS,
                shape(
                    0,
                    Shape {
                        cents: 100,
                        ..Shape::default()
                    },
                ),
            ),
            event(1300 * MS, Command::Panic),
        ],
    );
    for (start, semitones) in [(50, 3), (470, 2), (890, 4)] {
        let signal = &audio.0[start * MS..(start + 320) * MS];
        let crossings: Vec<_> = signal
            .windows(2)
            .enumerate()
            .filter(|(_, pair)| pair[0] <= 0.0 && pair[1] > 0.0)
            .map(|(i, pair)| i as f32 - pair[0] / (pair[1] - pair[0]))
            .collect();
        let period = (crossings[crossings.len() - 1] - crossings[0]) / (crossings.len() - 1) as f32;
        let hz = RATE as f32 / period;
        assert!((hz - 440.0 * 2f32.powf(semitones as f32 / 12.0)).abs() < 0.02);
    }
    wav("144-fine-tune.wav", &audio);
}

/// Scenario "Hold delays the selected decay curve".
#[test]
fn hold_delays_the_selected_decay_curve() {
    for exponential in [false, true] {
        let mut e = engine(vec![0.25; 500 * MS]);
        let (l, _) = draw(
            &mut e,
            300 * MS,
            vec![
                event(
                    0,
                    shape(
                        0,
                        Shape {
                            hold_ms: 40,
                            exponential,
                            ..Shape::default()
                        },
                    ),
                ),
                event(0, env(0, 10, 80, 0)),
                event(
                    0,
                    Command::SetPadPitch {
                        pad: 0,
                        semitones: 12,
                    },
                ),
                event(0, hit(0, 1.0)),
            ],
        );
        close(l[5 * MS], 0.125);
        assert_eq!(l[10 * MS], 0.25);
        assert_eq!(l[50 * MS], 0.25);
        let halfway = if exponential {
            0.25 * 0.001f32.sqrt()
        } else {
            0.125
        };
        close(l[90 * MS], halfway);
        close(l[130 * MS], if exponential { 0.00025 } else { 0.0 });
        assert_eq!(peak(&l[132 * MS..]), 0.0);
        assert!(max_step(&l) < 0.001);
    }

    let mut e = engine(sine(780 * MS, 205.0, 0.25));
    let audio = draw(
        &mut e,
        1000 * MS,
        vec![
            event(0, env(0, 30, 220, 0)),
            event(
                0,
                shape(
                    0,
                    Shape {
                        hold_ms: 80,
                        ..Shape::default()
                    },
                ),
            ),
            event(20 * MS, hit(0, 1.0)),
            event(
                570 * MS,
                shape(
                    0,
                    Shape {
                        hold_ms: 80,
                        exponential: true,
                        ..Shape::default()
                    },
                ),
            ),
            event(570 * MS, hit(0, 1.0)),
        ],
    );
    wav("144-envelope.wav", &audio);
}

/// Scenario "Gate release preserves Panic and group cuts".
#[test]
fn gate_release_preserves_panic_and_group_cuts() {
    for cut in [None, Some(Command::Panic), Some(hit(1, 1.0))] {
        let mut e = engine(vec![0.25; 400 * MS]);
        let mut events = vec![
            event(
                0,
                Command::SetPadSound {
                    pad: 1,
                    sound: None,
                },
            ),
            event(0, Command::SetPadGroup { pad: 0, group: 1 }),
            event(0, Command::SetPadGroup { pad: 1, group: 1 }),
            event(
                0,
                Command::SetPadMode {
                    pad: 0,
                    mode: PadMode::Gate,
                },
            ),
            event(
                0,
                shape(
                    0,
                    Shape {
                        exponential: true,
                        ..Shape::default()
                    },
                ),
            ),
            event(0, env(0, 0, 0, 100)),
            event(0, hit(0, 1.0)),
            event(50 * MS, Command::PadRelease { pad: 0 }),
        ];
        let expect_cut = cut.is_some();
        if let Some(cut) = cut {
            events.push(event(80 * MS, cut));
        }
        let (l, _) = draw(&mut e, 200 * MS, events);
        if expect_cut {
            assert_eq!(peak(&l[85 * MS..]), 0.0);
            assert!((l[80 * MS] - l[80 * MS - 1]).abs() < 0.001);
        } else {
            close(l[100 * MS], 0.25 * 0.001f32.sqrt());
            close(l[150 * MS], 0.00025);
            assert_eq!(peak(&l[152 * MS..]), 0.0);
        }
        assert!(max_step(&l[50 * MS..]) < 0.001);
    }
}

#[test]
fn a_legacy_tail_keeps_its_gain_window_and_linear_velocity() {
    let mut e = engine(vec![0.25; 100 * MS]);
    let (l, _) = draw(
        &mut e,
        115 * MS,
        vec![event(0, env(0, 0, 40, 0)), event(0, hit(0, 64.0 / 127.0))],
    );
    close(l[59 * MS], 0.25 * 64.0 / 127.0);
    close(l[80 * MS], 0.25 * 32.0 / 127.0);
    assert_eq!(peak(&l[100 * MS..]), 0.0);

    let mut e = engine(sine(320 * MS, 205.0, 0.25));
    let audio = draw(
        &mut e,
        355 * MS,
        vec![
            event(0, env(0, 10, 140, 0)),
            event(10 * MS, hit(0, 64.0 / 127.0)),
        ],
    );
    wav("144-legacy.wav", &audio);
}

/// Scenario "The velocity range changes both manual and replayed hits".
#[test]
fn the_velocity_range_changes_both_manual_and_replayed_hits() {
    use engine::pattern::{Grid, Hit, RawPattern};

    let hits = [1, 64, 127];
    let mut demo = (Vec::new(), Vec::new());
    for db in [0, 30, 60] {
        let sets = Shape {
            velocity_db: Some(db),
            ..Shape::default()
        };
        let mut e = engine(vec![0.2; 4 * MS]);
        let rows: Vec<_> = hits
            .into_iter()
            .enumerate()
            .map(|(i, v)| Hit::new((i as u32 + 1) * 96, i, v, 0).unwrap())
            .collect();
        let compiled = Arc::new(RawPattern::new(1, Grid::Off, 0, rows).unwrap().compile());
        let mut events = vec![
            event(0, Command::SetTempo { bpm: 120.0 }),
            event(
                0,
                Command::SetPattern {
                    pattern: Some(compiled.clone()),
                },
            ),
            event(0, Command::TransportPlay),
        ];
        for (i, velocity) in hits.into_iter().enumerate() {
            events.push(event(0, shape(i, sets)));
            events.push(event(
                (200 + i * 50) * MS,
                hit(i, f32::from(velocity) / 127.0),
            ));
        }
        events.push(event(350 * MS, hit(0, 0.0)));
        let (l, r) = draw(&mut e, 370 * MS, events);
        for (i, expected) in [
            10f32.powf(-f32::from(db) / 20.0),
            10f32.powf(-f32::from(db) / 40.0),
            1.0,
        ]
        .into_iter()
        .enumerate()
        {
            let played = (51 + i * 50) * MS;
            let manual = (201 + i * 50) * MS;
            close(l[played], 0.2 * expected);
            close(l[manual], l[played]);
            close(r[manual], l[manual]);
        }
        assert_eq!(peak(&l[350 * MS..]), 0.0);
        assert_eq!(
            compiled
                .events()
                .iter()
                .map(|hit| hit.velocity())
                .collect::<Vec<_>>(),
            hits
        );

        let mut e = engine(sine(15 * MS, 200.0, 0.28));
        let mut events = vec![event(0, shape(0, sets))];
        for v in 1u8..=127 {
            events.push(event(
                (20 + (v as usize - 1) * 20) * MS,
                hit(0, f32::from(v) / 127.0),
            ));
        }
        let (l, r) = draw(&mut e, 2_580 * MS, events);
        demo.0.extend(l);
        demo.1.extend(r);
    }
    wav("144-velocity.wav", &demo);
}

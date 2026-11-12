//! Pattern timing and sound through the production engine, without devices.

use std::sync::Arc;

use engine::pattern::{Grid, Hit, RawPattern};
use engine::{Command, Engine, PadMode, Region, Sample};

const SR: u32 = 48_000;
const FADE: usize = 240;

fn pattern(bars: u8, grid: Grid, rows: &[[u32; 4]]) -> Arc<engine::pattern::Pattern> {
    Arc::new(
        RawPattern::new(
            bars,
            grid,
            0,
            rows.iter()
                .map(|&[t, p, v, g]| Hit::new(t, p as usize, v as u8, g).unwrap())
                .collect(),
        )
        .unwrap()
        .compile(),
    )
}

fn engine(bpm: f64) -> Engine {
    let mut e = Engine::with_sounds(SR, vec![0.01], vec![Vec::new()]);
    e.handle(0, Command::SetTempo { bpm });
    e
}

fn sound(e: &mut Engine, pad: usize, value: f32, frames: usize) {
    e.handle(
        0,
        Command::SetPadSound {
            pad,
            sound: Some(Arc::new(Sample::mono(vec![value; frames]))),
        },
    );
}

fn start(e: &mut Engine, p: Arc<engine::pattern::Pattern>) {
    e.handle(0, Command::SetPattern { pattern: Some(p) });
    e.handle(0, Command::TransportPlay);
}

fn frames(ticks: f64, bpm: f64) -> usize {
    (ticks * 60.0 * f64::from(SR) / (bpm * 960.0)).round() as usize
}

fn render(e: &mut Engine, n: usize, sizes: &[usize]) -> Vec<f32> {
    render_stereo(e, n, sizes).0
}

fn render_stereo(e: &mut Engine, n: usize, sizes: &[usize]) -> (Vec<f32>, Vec<f32>) {
    let mut l = vec![0.0; n];
    let mut r = vec![0.0; n];
    let mut at = 0;
    let mut index = 0;
    while at < n {
        let end = (at + sizes[index % sizes.len()]).min(n);
        e.process(&mut l[at..end], &mut r[at..end]);
        index += 1;
        at = end;
    }
    assert!(l.iter().chain(&r).all(|s| s.is_finite() && s.abs() < 1.0));
    (l, r)
}

fn near(a: &[f32], b: &[f32]) {
    assert_eq!(a.len(), b.len());
    for (i, (a, b)) in a.iter().zip(b).enumerate() {
        assert!((a - b).abs() <= 0.000001, "frame {i}: {a} != {b}");
    }
}

fn onsets(wave: &[f32]) -> Vec<usize> {
    wave.iter()
        .enumerate()
        .filter_map(|(i, &v)| (v != 0.0 && (i == 0 || wave[i - 1] == 0.0)).then_some(i))
        .collect()
}

/// Scenarios "Pattern repeats from its first hit" and "Pattern timing is independent of callback size".
#[test]
fn pattern_timing_is_independent_of_callback_size() {
    let ticks = [0, 1, 239, 3840, 7679];
    let rows: Vec<_> = ticks
        .iter()
        .enumerate()
        .map(|(i, &t)| [t, i as u32, 127, 0])
        .collect();
    for bpm in [40.0, 120.0, 172.0, 300.0] {
        let n = frames(4.0 * 7680.0, bpm);
        let expected: Vec<_> = (0..4)
            .flat_map(|lap| ticks.map(|t| frames(f64::from(lap * 7680 + t), bpm)))
            .collect();
        for sizes in [&[1][..], &[128], &[1024], &[3, 127, 1, 991, 47, 1024]] {
            let mut e = engine(bpm);
            for p in 0..ticks.len() {
                sound(&mut e, p, 0.02, 1);
            }
            start(&mut e, pattern(2, Grid::Off, &rows));
            let split = n / 2;
            let mut wave = render(&mut e, split, sizes);
            let before = e.transport();
            e.handle(0, Command::TransportPlay);
            assert_eq!(e.transport(), before, "explicit Play is idempotent");
            wave.extend(render(&mut e, n - split, sizes));
            let got = onsets(&wave);
            assert_eq!(got.len(), expected.len(), "{bpm}/{sizes:?}: {got:?}");
            for (got, expected) in got.iter().zip(&expected) {
                assert!(
                    got.abs_diff(*expected) <= 1,
                    "{bpm}/{sizes:?}: {got}/{expected}"
                );
            }
            assert_eq!(e.dropped_pattern_hits(), 0);
        }
    }
}

/// Scenario "Pattern empty blocks preserve its first event".
#[test]
fn pattern_empty_blocks_preserve_its_first_event() {
    let mut e = engine(120.0);
    sound(&mut e, 0, 0.1, 48_000);
    e.handle(
        0,
        Command::SetPadMode {
            pad: 0,
            mode: PadMode::Gate,
        },
    );
    start(&mut e, pattern(1, Grid::Off, &[[0, 0, 127, 10]]));
    for _ in 0..5 {
        e.process(&mut [], &mut []);
    }
    assert_eq!(e.transport().ticks, 0.0);
    let first = render(&mut e, 100, &[100]);
    assert_eq!(first, vec![0.1; 100]);
    let clock = e.transport();
    for _ in 0..5 {
        e.process(&mut [], &mut []);
    }
    assert_eq!(e.transport(), clock);
    let rest = render(&mut e, 500, &[128]);
    assert_eq!(rest[149], 0.1);
    assert!(rest[151] < 0.1);
    assert!(rest[400..].iter().all(|s| *s == 0.0));
}

/// Scenario "Empty pattern does not stop the beat clock".
#[test]
fn empty_pattern_does_not_stop_the_beat_clock() {
    let mut control = engine(172.0);
    control.handle(0, Command::SetMetronome { enabled: true });
    control.handle(0, Command::TransportPlay);
    let expected = render(&mut control, 100_000, &[128]);
    let mut e = engine(172.0);
    e.handle(0, Command::SetMetronome { enabled: true });
    start(&mut e, pattern(1, Grid::Off, &[]));
    near(&render(&mut e, 100_000, &[128]), &expected);
    assert_eq!(e.transport().pattern_ticks, 3840);
    assert_eq!(control.transport().pattern_ticks, 0);
    assert_eq!(e.transport().ticks, control.transport().ticks);
    assert!(e.transport().playing && e.transport().metronome);
}

/// Scenario "Pattern cuts do not start before the event".
#[test]
fn pattern_cuts_do_not_start_before_the_event() {
    for grouped in [false, true] {
        let mut e = engine(120.0);
        sound(&mut e, 0, 0.1, 48_000);
        sound(&mut e, 1, 0.2, 48_000);
        if grouped {
            for pad in [0, 1] {
                e.handle(0, Command::SetPadGroup { pad, group: 7 });
            }
        }
        let next = u32::from(grouped);
        start(
            &mut e,
            pattern(1, Grid::Off, &[[0, 0, 127, 0], [7, next, 127, 0]]),
        );
        let wave = render(&mut e, 512, &[512]);
        assert!(wave[..175].iter().all(|&v| v == 0.1));
        let added = if grouped { 0.2 } else { 0.1 };
        for i in 0..FADE {
            assert!(
                (wave[175 + i] - (added + 0.1 * (FADE - i) as f32 / FADE as f32)).abs() < 0.000001
            );
        }
        assert!((wave[175 + FADE] - added).abs() < 0.000001);
    }
}

/// Scenario "Simultaneous pattern hits use a stable order".
#[test]
fn simultaneous_pattern_hits_use_a_stable_order() {
    let mut e = engine(120.0);
    for p in 0..3 {
        sound(&mut e, p, 0.01 * (p + 1) as f32, 48_000);
        e.handle(0, Command::SetPadGroup { pad: p, group: 1 });
    }
    e.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    start(
        &mut e,
        pattern(
            1,
            Grid::Off,
            &[[0, 2, 127, 0], [0, 1, 127, 0], [0, 0, 127, 0]],
        ),
    );
    let wave = render(&mut e, 512, &[512]);
    assert!((wave[0] - 0.07).abs() < 0.000001);
    assert!(wave[FADE..].iter().all(|v| (*v - 0.03).abs() < 0.000001));
}

/// Scenario "Unavailable pattern pads never replay missed hits".
#[test]
fn unavailable_pattern_pads_never_replay_missed_hits() {
    let mut e = engine(120.0);
    start(
        &mut e,
        pattern(1, Grid::Off, &[[0, 127, 127, 0], [100, 127, 64, 0]]),
    );
    assert!(render(&mut e, 128, &[128]).iter().all(|&v| v == 0.0));
    sound(&mut e, 127, 0.1, 100);
    let wave = render(&mut e, 3000, &[128]);
    assert_eq!(onsets(&wave), [2500 - 128]);
    assert!((wave[2500 - 128] - 0.1 * 64.0 / 127.0).abs() < 0.000001);
}

/// Scenario "Pattern gate length crosses a loop seam safely".
#[test]
fn pattern_gate_length_crosses_a_loop_seam_safely() {
    let mut e = engine(120.0);
    for pad in 0..3 {
        sound(&mut e, pad, 0.1, 100_000);
    }
    for pad in [0, 1] {
        e.handle(
            0,
            Command::SetPadMode {
                pad,
                mode: PadMode::Gate,
            },
        );
    }
    start(
        &mut e,
        pattern(
            1,
            Grid::Off,
            &[[3800, 0, 127, 100], [0, 1, 127, 0], [0, 2, 127, 0]],
        ),
    );
    let wave = render(&mut e, 100_000, &[1024]);
    assert_eq!(wave[0], 0.1, "zero gate is silent, zero one-shot is not");
    assert!(
        (wave[frames(3899.0, 120.0)] - 0.2).abs() < 0.000001,
        "seam retriggers one-shot while gate continues"
    );
    let cut = frames(3900.0, 120.0);
    assert!(wave[cut + 1] < wave[cut]);
    assert!((wave[cut + FADE] - 0.1).abs() < 0.000001);
}

/// Scenario "Old pattern release cannot cut a new manual hit".
#[test]
fn old_pattern_release_cannot_cut_a_new_manual_hit() {
    let mut e = engine(120.0);
    sound(&mut e, 0, 0.1, 48_000);
    e.handle(
        0,
        Command::SetPadMode {
            pad: 0,
            mode: PadMode::Gate,
        },
    );
    start(&mut e, pattern(1, Grid::Off, &[[0, 0, 127, 100]]));
    render(&mut e, 1000, &[128]);
    e.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let wave = render(&mut e, 4000, &[128]);
    assert!(wave[FADE..].iter().all(|v| *v == 0.1));
    e.handle(0, Command::PadRelease { pad: 0 });
    assert_eq!(render(&mut e, 512, &[128])[FADE], 0.0);
}

/// Scenario "Manual release cannot cut a newer pattern note".
#[test]
fn manual_release_cannot_cut_a_newer_pattern_note() {
    let mut e = engine(120.0);
    sound(&mut e, 0, 0.1, 48_000);
    e.handle(
        0,
        Command::SetPadMode {
            pad: 0,
            mode: PadMode::Gate,
        },
    );
    e.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    start(&mut e, pattern(1, Grid::Off, &[[10, 0, 127, 100]]));
    render(&mut e, 1000, &[128]);
    e.handle(0, Command::PadRelease { pad: 0 });
    let wave = render(&mut e, 2500, &[128]);
    assert!(wave[..1750].iter().all(|v| *v == 0.1));
    assert_eq!(wave[1750 + FADE], 0.0);
}

/// Scenario "Pattern replacement keeps phase without a duplicate hit".
#[test]
fn pattern_replacement_keeps_phase_without_a_duplicate_hit() {
    let mut e = engine(120.0);
    sound(&mut e, 0, 0.1, 1);
    start(&mut e, pattern(1, Grid::Off, &[[0, 0, 127, 0]]));
    assert_eq!(onsets(&render(&mut e, 2500, &[128])), [0]);
    let clock = e.transport();
    e.handle(
        0,
        Command::SetPattern {
            pattern: Some(pattern(
                1,
                Grid::Off,
                &[[99, 0, 127, 0], [100, 0, 127, 0], [101, 0, 127, 0]],
            )),
        },
    );
    assert_eq!(e.transport(), clock);
    assert_eq!(onsets(&render(&mut e, 128, &[128])), [0, 25]);
    e.handle(0, Command::SetPattern { pattern: None });
    let before = e.transport().ticks;
    assert!(render(&mut e, 100_000, &[128]).iter().all(|&v| v == 0.0));
    assert!(e.transport().ticks > before && e.transport().playing);
}

#[test]
fn replacement_and_swap_before_the_first_frame_keep_tick_zero() {
    for swap in [false, true] {
        let mut e = engine(120.0);
        sound(&mut e, 0, 0.02, 1);
        sound(&mut e, 1, 0.03, 1);
        start(&mut e, pattern(1, Grid::Off, &[[0, 0, 127, 0]]));
        let replacement = Some(pattern(1, Grid::Off, &[[0, 1, 127, 0]]));
        e.handle(
            0,
            if swap {
                Command::SwapPatternPads {
                    a: 0,
                    b: 1,
                    pattern: replacement,
                }
            } else {
                Command::SetPattern {
                    pattern: replacement,
                }
            },
        );
        e.process(&mut [], &mut []);
        assert_eq!(e.transport().ticks, 0.0);
        let wave = render_stereo(&mut e, 128, &[128]);
        for channel in [&wave.0, &wave.1] {
            assert_eq!(onsets(channel), [0]);
            assert_eq!(channel[0], if swap { 0.02 } else { 0.03 });
        }
        assert_eq!(e.dropped_pattern_hits(), 0);
    }
}

/// Scenario "Pattern tempo changes preserve pending musical deadlines".
#[test]
fn pattern_tempo_changes_preserve_pending_musical_deadlines() {
    let mut e = engine(120.0);
    sound(&mut e, 0, 0.1, 100_000);
    e.handle(
        0,
        Command::SetPadMode {
            pad: 0,
            mode: PadMode::Gate,
        },
    );
    start(&mut e, pattern(1, Grid::Off, &[[0, 0, 127, 960]]));
    render(&mut e, 12_000, &[128]);
    let ticks = e.transport().ticks;
    e.handle(0, Command::SetTempo { bpm: 240.0 });
    assert_eq!(e.transport().ticks, ticks);
    let wave = render(&mut e, 7000, &[128]);
    assert!(wave[..6000].iter().all(|v| *v == 0.1));
    assert_eq!(wave[6000], 0.1);
    assert!(wave[6001] < 0.1);
    assert_eq!(wave[6000 + FADE], 0.0);
}

/// Scenario "Pattern sound matches ordinary pad playing".
#[test]
fn pattern_sound_matches_ordinary_pad_playing() {
    fn configured() -> Engine {
        let mut e = engine(120.0);
        for pad in [0, 1, 2] {
            let wave = (0..48_000)
                .map(|i| 0.02 * (i as f32 * 0.031 * (pad + 1) as f32).sin())
                .collect();
            e.handle(
                0,
                Command::SetPadSound {
                    pad,
                    sound: Some(Arc::new(Sample::mono(wave))),
                },
            );
            e.handle(
                0,
                Command::SetPadMode {
                    pad,
                    mode: PadMode::Gate,
                },
            );
            e.handle(0, Command::SetPadPitch { pad, semitones: 3 });
            e.handle(0, Command::SetPadGain { pad, gain: 0.7 });
            e.handle(
                0,
                Command::SetPadEnvelope {
                    pad,
                    attack_ms: 2,
                    decay_ms: 10,
                    release_ms: 20,
                },
            );
            e.handle(
                0,
                Command::SetPadFilter {
                    pad,
                    settings: engine::filter::FilterSettings::default(),
                },
            );
            e.handle(0, Command::SetPadDrive { pad, drive: 0.3 });
            e.handle(
                0,
                Command::SetPadSends {
                    pad,
                    sends: [0.2, 0.3],
                },
            );
            e.handle(
                0,
                Command::SetPadGroup {
                    pad,
                    group: if pad == 2 { 0 } else { 1 },
                },
            );
        }
        render(&mut e, 128, &[128]);
        e
    }
    let mut manual = configured();
    let mut sequenced = configured();
    let p = pattern(
        1,
        Grid::Off,
        &[[0, 0, 100, 100], [0, 2, 80, 100], [100, 1, 120, 100]],
    );
    start(&mut sequenced, p);
    manual.handle(0, Command::TransportPlay);
    let mut reference = (Vec::new(), Vec::new());
    for (at, commands) in [
        (
            0,
            vec![
                Command::PadPress {
                    pad: 0,
                    velocity: 100.0 / 127.0,
                },
                Command::PadPress {
                    pad: 2,
                    velocity: 80.0 / 127.0,
                },
            ],
        ),
        (
            2500,
            vec![
                Command::PadRelease { pad: 0 },
                Command::PadRelease { pad: 2 },
                Command::PadPress {
                    pad: 1,
                    velocity: 120.0 / 127.0,
                },
            ],
        ),
        (5000, vec![Command::PadRelease { pad: 1 }]),
    ] {
        if reference.0.len() < at {
            let n = at - reference.0.len();
            let (l, r) = render_stereo(&mut manual, n, &[128]);
            reference.0.extend(l);
            reference.1.extend(r);
        }
        for c in commands {
            manual.handle(0, c);
        }
    }
    let (l, r) = render_stereo(&mut manual, 8000 - reference.0.len(), &[128]);
    reference.0.extend(l);
    reference.1.extend(r);
    let actual = render_stereo(&mut sequenced, 8000, &[128]);
    near(&actual.0, &reference.0);
    near(&actual.1, &reference.1);
}

/// Scenario "Pitch Gate cuts at empty straight and triplet steps".
#[test]
fn pitch_gate_cuts_at_each_selected_grid_and_off_fallback() {
    for grid in [
        Grid::Off,
        Grid::Quarter,
        Grid::Eighth,
        Grid::Sixteenth,
        Grid::ThirtySecond,
        Grid::SixtyFourth,
        Grid::EighthTriplet,
        Grid::SixteenthTriplet,
        Grid::ThirtySecondTriplet,
    ] {
        for strength in [0, 50, 100] {
            let mut e = engine(172.0);
            sound(&mut e, 0, 0.1, 200_000);
            e.handle(
                0,
                Command::SetPadPitch {
                    pad: 0,
                    semitones: 1,
                },
            );
            e.handle(
                0,
                Command::SetPadPitchGate {
                    pad: 0,
                    enabled: true,
                    linked_slice: true,
                },
            );
            let raw =
                RawPattern::new(1, grid, strength, vec![Hit::new(1, 0, 127, 0).unwrap()]).unwrap();
            start(&mut e, Arc::new(raw.compile()));
            let cut = frames(f64::from(grid.gate_spacing()), 172.0);
            let wave = render(&mut e, cut + FADE + 10, &[128]);
            assert!(
                wave[frames(1.0, 172.0)..cut].iter().all(|v| *v == 0.1),
                "{grid:?}"
            );
            for i in 0..FADE {
                assert!(
                    (wave[cut + i] - 0.1 * (FADE - i) as f32 / FADE as f32).abs() < 0.000001,
                    "{grid:?}, {i}"
                );
            }
            assert!(wave[cut + FADE..].iter().all(|&v| v == 0.0));
        }
    }
}

/// Scenario "Pattern Pitch Gate eligibility preserves manual sound".
#[test]
fn pattern_pitch_gate_eligibility_preserves_manual_sound() {
    for (source, linked, region) in [
        ("whole file", false, Region::WHOLE),
        (
            "detached piece",
            false,
            Region {
                start: 2000,
                end: 90_000,
            },
        ),
        (
            "linked slice",
            true,
            Region {
                start: 2000,
                end: 90_000,
            },
        ),
    ] {
        for pitch in [-12, 0, 12] {
            for origin in ["manual", "preview", "pattern"] {
                let mut waves = Vec::new();
                for enabled in [false, true] {
                    let mut e = engine(120.0);
                    let sample = Arc::new(Sample::mono(vec![0.05; 100_000]));
                    for command in [
                        Command::SetPadSound {
                            pad: 0,
                            sound: Some(sample.clone()),
                        },
                        Command::SetPadRegion { pad: 0, region },
                        Command::SetPadPitch {
                            pad: 0,
                            semitones: pitch,
                        },
                        Command::SetPadPitchGate {
                            pad: 0,
                            enabled,
                            linked_slice: linked,
                        },
                        Command::SetPadEnvelope {
                            pad: 0,
                            attack_ms: 10,
                            decay_ms: 0,
                            release_ms: 20,
                        },
                    ] {
                        e.handle(0, command);
                    }
                    match origin {
                        "manual" => e.handle(
                            0,
                            Command::PadPress {
                                pad: 0,
                                velocity: 1.0,
                            },
                        ),
                        "preview" => {
                            e.handle(0, Command::PreviewAs { pad: Some(0) });
                            e.handle(
                                0,
                                Command::Preview {
                                    sound: Some(sample),
                                    region,
                                },
                            );
                        }
                        _ => start(&mut e, pattern(1, Grid::Sixteenth, &[[0, 0, 127, 0]])),
                    }
                    waves.push(render_stereo(&mut e, 12_000, &[128]));
                }
                let cuts = origin == "pattern" && linked && pitch > 0;
                if cuts {
                    assert!(waves[0].0[7000..].iter().all(|sample| *sample == 0.05));
                    assert!(
                        waves[1].0[6000 + FADE..]
                            .iter()
                            .all(|sample| *sample == 0.0)
                    );
                    near(&waves[0].0[..6000], &waves[1].0[..6000]);
                    near(&waves[0].1[..6000], &waves[1].1[..6000]);
                } else {
                    near(&waves[0].0, &waves[1].0);
                    near(&waves[0].1, &waves[1].1);
                    assert!(
                        waves[1].0[7000..].iter().any(|sample| *sample != 0.0),
                        "{source}/{pitch}/{origin}"
                    );
                }
            }
        }
    }
}

/// Scenario "Pitch Gate changes apply to future note eligibility".
#[test]
fn pitch_gate_eligibility_is_onset_owned_and_manual_play_is_exempt() {
    for (linked, enabled, pitch) in [
        (true, true, -1),
        (true, true, 0),
        (true, true, 1),
        (false, true, 1),
        (true, false, 1),
    ] {
        for manual in [false, true] {
            let mut e = engine(120.0);
            sound(&mut e, 0, 0.1, 100_000);
            e.handle(
                0,
                Command::SetPadPitch {
                    pad: 0,
                    semitones: pitch,
                },
            );
            e.handle(
                0,
                Command::SetPadPitchGate {
                    pad: 0,
                    enabled,
                    linked_slice: linked,
                },
            );
            if manual {
                e.handle(
                    0,
                    Command::PadPress {
                        pad: 0,
                        velocity: 1.0,
                    },
                );
            } else {
                start(
                    &mut e,
                    pattern(1, Grid::Sixteenth, &[[0, 0, 127, 0], [480, 0, 127, 0]]),
                );
            }
            render(&mut e, 128, &[128]);
            e.handle(
                0,
                Command::SetPadPitchGate {
                    pad: 0,
                    enabled: !enabled,
                    linked_slice: !linked,
                },
            );
            e.handle(
                0,
                Command::SetPadPitch {
                    pad: 0,
                    semitones: -pitch,
                },
            );
            e.handle(
                0,
                Command::SetPadRegion {
                    pad: 0,
                    region: Region { start: 1, end: 2 },
                },
            );
            let tail = render(&mut e, 7000, &[128]);
            let cuts = !manual && linked && enabled && pitch > 0;
            assert_eq!(
                tail[6500],
                if cuts { 0.0 } else { 0.1 },
                "{linked}/{enabled}/{pitch}/{manual}"
            );
            if !manual {
                sound(&mut e, 0, 0.2, 100_000);
                e.handle(
                    0,
                    Command::SetPadPitch {
                        pad: 0,
                        semitones: 1,
                    },
                );
                e.handle(
                    0,
                    Command::SetPadPitchGate {
                        pad: 0,
                        enabled: true,
                        linked_slice: true,
                    },
                );
                let next = render(&mut e, 20_000, &[128]);
                let onset = 12_000 - 7128;
                assert!(next[onset + FADE..18_000 - 7128].iter().all(|v| *v == 0.2));
                assert!(next[18_000 - 7128 + FADE..].iter().all(|v| *v == 0.0));
            }
        }
    }
}

/// Scenario "Pitch Gate caps an ordinary release tail".
#[test]
fn pitch_gate_caps_an_ordinary_release_without_raising_gain() {
    for gate in [120, 240, 480] {
        let mut e = engine(120.0);
        sound(&mut e, 0, 0.1, 100_000);
        for command in [
            Command::SetPadMode {
                pad: 0,
                mode: PadMode::Gate,
            },
            Command::SetPadPitch {
                pad: 0,
                semitones: 1,
            },
            Command::SetPadPitchGate {
                pad: 0,
                enabled: true,
                linked_slice: true,
            },
            Command::SetPadEnvelope {
                pad: 0,
                attack_ms: 0,
                decay_ms: 0,
                release_ms: 1000,
            },
        ] {
            e.handle(0, command);
        }
        start(&mut e, pattern(1, Grid::Sixteenth, &[[0, 0, 127, gate]]));
        let wave = render(&mut e, 7000, &[128]);
        let ordinary = frames(f64::from(gate), 120.0);
        let cut = frames(240.0, 120.0);
        let gain = (48_000 - cut.saturating_sub(ordinary)) as f32 / 48_000.0;
        for i in 0..FADE {
            assert!(
                (wave[cut + i] - 0.1 * gain * (FADE - i) as f32 / FADE as f32).abs() < 0.000001
            );
        }
        assert!(wave[cut + FADE..].iter().all(|&v| v == 0.0));
    }
}

/// Scenario "Pitch Gate cuts without extending a slice or changing attack".
#[test]
fn pitch_gate_cuts_without_extending_a_slice_or_changing_attack() {
    for end in [100, 40_000] {
        for attack_ms in [0, 3] {
            let configured = || {
                let mut e = engine(120.0);
                sound(&mut e, 0, 0.1, 100_000);
                sound(&mut e, 1, 0.01, 100_000);
                for command in [
                    Command::SetPadRegion {
                        pad: 0,
                        region: Region { start: 40, end },
                    },
                    Command::SetPadPitch {
                        pad: 0,
                        semitones: 1,
                    },
                    Command::SetPadMode {
                        pad: 0,
                        mode: PadMode::Gate,
                    },
                    Command::SetPadEnvelope {
                        pad: 0,
                        attack_ms,
                        decay_ms: 0,
                        release_ms: 0,
                    },
                    Command::SetPadGroup { pad: 0, group: 3 },
                    Command::SetPadGroup { pad: 1, group: 4 },
                ] {
                    e.handle(0, command);
                }
                e.handle(
                    0,
                    Command::PadPress {
                        pad: 1,
                        velocity: 1.0,
                    },
                );
                e
            };
            let mut manual = configured();
            manual.handle(
                0,
                Command::PadPress {
                    pad: 0,
                    velocity: 1.0,
                },
            );
            let mut reference = render(&mut manual, 6000, &[128]);
            manual.handle(0, Command::PadRelease { pad: 0 });
            reference.extend(render(&mut manual, 512, &[128]));
            let mut e = configured();
            e.handle(
                0,
                Command::SetPadPitchGate {
                    pad: 0,
                    enabled: true,
                    linked_slice: true,
                },
            );
            start(&mut e, pattern(1, Grid::Sixteenth, &[[0, 0, 127, 1000]]));
            near(&render(&mut e, 6512, &[128]), &reference);
            assert!(reference[6240..].iter().all(|v| *v == 0.01));
            if end == 100 {
                assert!(reference[100..].iter().all(|v| *v == 0.01));
            }
        }
    }
}

/// Scenario "Pitch Gate respects off-grid onsets and the loop end".
#[test]
fn pitch_gate_respects_off_grid_onsets_and_the_loop_end() {
    for tick in [0, 240, 241, 3839] {
        let mut e = engine(120.0);
        sound(&mut e, 0, 0.1, 200_000);
        e.handle(
            0,
            Command::SetPadPitch {
                pad: 0,
                semitones: 1,
            },
        );
        e.handle(
            0,
            Command::SetPadPitchGate {
                pad: 0,
                enabled: true,
                linked_slice: true,
            },
        );
        let rows = if tick == 3839 {
            vec![[0, 0, 127, 0], [tick, 0, 127, 0]]
        } else {
            vec![[tick, 0, 127, 0]]
        };
        start(&mut e, pattern(1, Grid::Sixteenth, &rows));
        let cut = frames(f64::from((tick / 240 + 1) * 240), 120.0);
        let wave = render(&mut e, cut + FADE + 300, &[1024]);
        if tick == 3839 {
            // The seam releases the old note, then starts the new tick-zero note.
            assert!((wave[cut] - 0.2).abs() < 0.000001);
            assert!(wave[cut + FADE..].iter().all(|v| *v == 0.1));
        } else {
            assert_eq!(wave[cut], 0.1);
            assert!(wave[cut + 1] < 0.1);
            assert!(wave[cut + FADE..].iter().all(|v| *v == 0.0));
        }
    }
}

/// Scenario "Pattern deadlines cannot follow a reused voice slot".
#[test]
fn pattern_deadlines_cannot_follow_a_reused_voice_slot() {
    let mut e = engine(120.0);
    for pad in 0..64 {
        sound(&mut e, pad, 0.001, 100_000);
        e.handle(
            0,
            Command::SetPadMode {
                pad,
                mode: PadMode::Gate,
            },
        );
    }
    let rows: Vec<_> = (0..64).map(|pad| [0, pad, 127, 100]).collect();
    start(&mut e, pattern(1, Grid::Off, &rows));
    render(&mut e, 128, &[128]);
    // Replace the source and force reuse after stealing more than all 48 voice slots.
    sound(&mut e, 0, 0.002, 100_000);
    for _ in 0..64 {
        e.handle(
            0,
            Command::PadPress {
                pad: 0,
                velocity: 1.0,
            },
        );
    }
    let wave = render(&mut e, 4000, &[128]);
    assert!(
        wave[FADE..2500 - 128]
            .iter()
            .all(|v| (*v - 0.033).abs() < 0.000001)
    );
    assert!(
        wave[3000..].iter().all(|v| *v == 0.002),
        "old deadlines must not cut the reused manual slot"
    );
}

/// Scenario "Pattern stop preserves unrelated tails and clears deadlines".
#[test]
fn pattern_stop_preserves_unrelated_tails_and_clears_deadlines() {
    for kind in [
        "gate",
        "pitch gate",
        "one-shot",
        "manual",
        "preview",
        "effects",
    ] {
        let configured = || {
            let mut e = engine(120.0);
            sound(
                &mut e,
                0,
                0.1,
                if kind == "effects" { 480 } else { 100_000 },
            );
            if kind == "gate" {
                e.handle(
                    0,
                    Command::SetPadMode {
                        pad: 0,
                        mode: PadMode::Gate,
                    },
                );
            }
            if kind == "pitch gate" {
                e.handle(
                    0,
                    Command::SetPadPitch {
                        pad: 0,
                        semitones: 1,
                    },
                );
                e.handle(
                    0,
                    Command::SetPadPitchGate {
                        pad: 0,
                        enabled: true,
                        linked_slice: true,
                    },
                );
            }
            if kind == "effects" {
                e.handle(
                    0,
                    Command::SetPadSends {
                        pad: 0,
                        sends: [0.0, 0.5],
                    },
                );
                e.handle(
                    0,
                    Command::SetFx {
                        slot: 1,
                        settings: engine::fx::FxSettings::Delay {
                            time_ms: 20.0,
                            feedback: 0.5,
                        },
                    },
                );
                render(&mut e, 128, &[128]);
            }
            let rows = if kind == "manual" || kind == "preview" {
                vec![]
            } else {
                vec![[0, 0, 127, 1000]]
            };
            start(&mut e, pattern(1, Grid::Sixteenth, &rows));
            if kind == "manual" {
                e.handle(
                    0,
                    Command::PadPress {
                        pad: 0,
                        velocity: 1.0,
                    },
                );
            }
            if kind == "preview" {
                e.handle(
                    0,
                    Command::Preview {
                        sound: Some(Arc::new(Sample::mono(vec![0.1; 100_000]))),
                        region: Region::WHOLE,
                    },
                );
            }
            render(&mut e, 1000, &[128]);
            e
        };
        let mut control = configured();
        let mut e = configured();
        e.handle(0, Command::TransportStop);
        assert!(!e.transport().playing && e.transport().ticks == 0.0);
        let wave = render(&mut e, 1024, &[128]);
        if kind == "gate" || kind == "pitch gate" {
            assert!(wave[FADE..].iter().all(|v| *v == 0.0));
            e.handle(0, Command::TransportPlay);
            let first = render(&mut e, 128, &[128]);
            assert!(first.iter().all(|v| *v == 0.1));
        } else {
            near(&wave, &render(&mut control, 1024, &[128]));
            assert!(wave.iter().any(|v| *v != 0.0), "{kind} tail survives");
        }
    }
}

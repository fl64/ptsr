//! Required listening files use generated sound and the production engine only.

use std::sync::Arc;

use engine::offline::{Event, write_wav};
use engine::pattern::{Grid, Hit, RawPattern};
use engine::{Command, Engine, PadMode, Region, Sample, checks};

const SR: u32 = 48_000;
const FADE: usize = 240;
type Wave = (Vec<f32>, Vec<f32>);

fn pattern(bars: u8, grid: Grid, rows: &[[u32; 4]]) -> Arc<engine::pattern::Pattern> {
    Arc::new(
        RawPattern::new(
            bars,
            grid,
            0,
            rows.iter()
                .map(|&[tick, pad, velocity, gate]| {
                    Hit::new(tick, pad as usize, velocity as u8, gate).unwrap()
                })
                .collect(),
        )
        .unwrap()
        .compile(),
    )
}

fn event(events: &mut Vec<Event>, frame: usize, cmd: Command) {
    events.push(Event { frame, cmd });
}

fn ticks(tick: u32, bpm: f64) -> usize {
    (f64::from(tick) * 60.0 * f64::from(SR) / (960.0 * bpm)).round() as usize
}

fn render(mut engine: Engine, frames: usize, mut events: Vec<Event>) -> Wave {
    events.sort_by_key(|event| event.frame);
    let (mut left, mut right) = (vec![0.0; frames], vec![0.0; frames]);
    let (mut at, mut next) = (0, 0);
    while at < frames {
        while next < events.len() && events[next].frame == at {
            engine.handle(0, events[next].cmd.clone());
            next += 1;
        }
        let end = (at + 128)
            .min(frames)
            .min(events.get(next).map_or(frames, |event| event.frame));
        assert!(end > at);
        engine.process(&mut left[at..end], &mut right[at..end]);
        at = end;
    }
    for channel in [&left, &right] {
        assert_eq!(checks::first_non_finite(channel), None);
        assert!(checks::peak(channel) < 1.0, "clipping");
    }
    (left, right)
}

fn same(actual: &Wave, reference: &Wave) {
    for (actual, reference) in [(&actual.0, &reference.0), (&actual.1, &reference.1)] {
        assert_eq!(actual.len(), reference.len());
        for (frame, (actual, reference)) in actual.iter().zip(reference).enumerate() {
            assert!(
                (actual - reference).abs() <= 0.000001,
                "frame {frame}: {actual} != {reference}"
            );
        }
    }
}

fn save(name: &str, wave: &Wave) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../renders")
        .join(name);
    write_wav(&path, SR, &wave.0, &wave.1).unwrap();
}

fn drum_engine() -> Engine {
    let mut engine = Engine::with_sounds(SR, vec![], vec![]);
    engine.handle(0, Command::SetTempo { bpm: 120.0 });
    for (pad, frequency, length, level) in [(0, 70.0, 4800, 0.08), (1, 700.0, 2880, 0.06)] {
        let sound = (0..length)
            .map(|frame| {
                let fade = 1.0 - frame as f32 / (length - 1) as f32;
                level
                    * fade.powi(3)
                    * (std::f32::consts::TAU * frequency * frame as f32 / SR as f32).cos()
            })
            .collect();
        engine.handle(
            0,
            Command::SetPadSound {
                pad,
                sound: Some(Arc::new(Sample::mono(sound))),
            },
        );
    }
    let bass = (0..SR * 4)
        .map(|frame| 0.025 * (std::f32::consts::TAU * 110.0 * frame as f32 / SR as f32).cos())
        .collect();
    engine.handle(
        0,
        Command::SetPadSound {
            pad: 2,
            sound: Some(Arc::new(Sample::mono(bass))),
        },
    );
    engine.handle(
        0,
        Command::SetPadMode {
            pad: 2,
            mode: PadMode::Gate,
        },
    );
    engine.handle(
        0,
        Command::SetPadEnvelope {
            pad: 2,
            attack_ms: 0,
            decay_ms: 0,
            release_ms: 80,
        },
    );
    engine
}

#[test]
fn required_pattern_play_render_matches_manual_hits_gates_and_four_loops() {
    let first = [
        [0, 0, 110, 0],
        [480, 2, 90, 480],
        [960, 1, 100, 0],
        [1920, 0, 110, 0],
        [2880, 1, 100, 0],
        [3840, 0, 110, 0],
        [4320, 2, 90, 480],
        [4800, 1, 100, 0],
        [5760, 0, 110, 0],
        [6720, 1, 100, 0],
    ];
    let mut last = first;
    for hit in last.iter_mut().filter(|hit| hit[1] == 1) {
        hit[0] += 480;
    }
    let mut controls = Vec::new();
    let mut manual = Vec::new();
    let mut percussion_manual = Vec::new();
    let mut expected_onsets = Vec::new();
    let mut at = 0;
    for lap in 0..4 {
        let bpm = if lap < 2 { 120.0 } else { 150.0 };
        let rows = if lap == 3 { &last } else { &first };
        if lap == 0 {
            event(
                &mut controls,
                at,
                Command::SetPattern {
                    pattern: Some(pattern(2, Grid::Sixteenth, rows)),
                },
            );
            for events in [&mut controls, &mut manual, &mut percussion_manual] {
                event(events, at, Command::TransportPlay);
            }
        }
        if lap == 2 {
            for events in [&mut controls, &mut manual, &mut percussion_manual] {
                event(events, at, Command::SetTempo { bpm });
            }
        }
        if lap == 3 {
            event(
                &mut controls,
                at,
                Command::SetPattern {
                    pattern: Some(pattern(2, Grid::Sixteenth, rows)),
                },
            );
        }
        for &[tick, pad, velocity, gate] in rows {
            let frame = at + ticks(tick, bpm);
            let command = Command::PadPress {
                pad: pad as usize,
                velocity: velocity as f32 / 127.0,
            };
            event(&mut manual, frame, command.clone());
            if pad == 2 {
                event(
                    &mut manual,
                    frame + ticks(gate, bpm),
                    Command::PadRelease { pad: 2 },
                );
            } else {
                expected_onsets.push(frame);
                event(&mut percussion_manual, frame, command);
            }
        }
        at += ticks(7680, bpm);
    }
    assert_eq!(at, 691_200);
    for events in [&mut controls, &mut manual, &mut percussion_manual] {
        event(events, at, Command::TransportStop);
    }
    let actual = render(drum_engine(), at + SR as usize / 2, controls);
    let reference = render(drum_engine(), at + SR as usize / 2, manual);
    same(&actual, &reference);
    let percussion = render(drum_engine(), at + SR as usize / 2, percussion_manual);
    for channel in [&percussion.0, &percussion.1] {
        let onsets = checks::onsets(channel, 0.000001, 64);
        assert_eq!(onsets.len(), expected_onsets.len());
        for (actual, expected) in onsets.iter().zip(&expected_onsets) {
            assert!(actual.abs_diff(*expected) <= 1);
        }
    }
    for channel in [&actual.0, &actual.1] {
        assert!((0.06..0.12).contains(&checks::peak(channel)));
        assert!(channel[at..].iter().all(|&sample| sample == 0.0));
    }
    save("96-pattern-play.wav", &actual);
}

fn pitch_engine(enabled: bool) -> Engine {
    let mut engine = Engine::with_sounds(SR, vec![], vec![]);
    let tone = (0..SR * 8)
        .map(|frame| 0.04 * (std::f32::consts::TAU * 220.0 * frame as f32 / SR as f32).cos())
        .collect();
    engine.handle(
        0,
        Command::SetPadSound {
            pad: 0,
            sound: Some(Arc::new(Sample::mono(tone))),
        },
    );
    engine.handle(
        0,
        Command::SetPadRegion {
            pad: 0,
            region: Region::WHOLE,
        },
    );
    engine.handle(
        0,
        Command::SetPadPitch {
            pad: 0,
            semitones: 12,
        },
    );
    engine.handle(
        0,
        Command::SetPadPitchGate {
            pad: 0,
            enabled,
            linked_slice: true,
        },
    );
    engine.handle(0, Command::SetTempo { bpm: 120.0 });
    engine
}

fn pitch_events(sequenced: bool, reference_cuts: bool) -> Vec<Event> {
    let mut events = Vec::new();
    event(&mut events, 0, Command::TransportPlay);
    if sequenced {
        event(
            &mut events,
            0,
            Command::SetPattern {
                pattern: Some(pattern(
                    1,
                    Grid::Sixteenth,
                    &[[0, 0, 127, 0], [3720, 0, 127, 0]],
                )),
            },
        );
    } else {
        for (onset, cut) in [(0, 6000), (93_000, 96_000), (96_000, 102_000)] {
            event(
                &mut events,
                onset,
                Command::PadPress {
                    pad: 0,
                    velocity: 1.0,
                },
            );
            if reference_cuts {
                // A silent retrigger gives the existing short same-pad cut as a reference.
                event(
                    &mut events,
                    cut,
                    Command::PadPress {
                        pad: 0,
                        velocity: 0.0,
                    },
                );
            }
        }
    }
    event(&mut events, 108_000, Command::TransportStop);
    event(
        &mut events,
        108_000,
        Command::PadPress {
            pad: 0,
            velocity: 0.0,
        },
    );
    events
}

#[test]
fn required_pitch_gate_render_matches_existing_cuts_and_exempts_manual_hits() {
    let on = render(pitch_engine(true), 120_000, pitch_events(true, false));
    let off = render(pitch_engine(false), 120_000, pitch_events(true, false));
    let reference = render(pitch_engine(true), 120_000, pitch_events(false, true));
    let manual_on = render(pitch_engine(true), 120_000, pitch_events(false, false));
    let manual_off = render(pitch_engine(false), 120_000, pitch_events(false, false));
    same(&on, &reference);
    same(&off, &manual_on);
    same(&manual_on, &manual_off);
    for (on, off) in [(&on.0, &off.0), (&on.1, &off.1)] {
        assert_eq!(&on[..6000], &off[..6000], "unchanged attack and prefix");
        let changed = on
            .iter()
            .zip(off)
            .position(|(a, b)| (a - b).abs() > 0.000001)
            .unwrap();
        assert!(
            changed.abs_diff(6000) <= 1,
            "first changed cut frame {changed}"
        );
        assert!(on[6000 + FADE..93_000].iter().all(|&sample| sample == 0.0));
        assert!(on[102_000 + FADE..].iter().all(|&sample| sample == 0.0));
        assert!(
            off[6000 + FADE..93_000]
                .iter()
                .any(|sample| sample.abs() > 0.03)
        );
        assert!(off[108_000 + FADE..].iter().all(|&sample| sample == 0.0));
        assert!((0.035..0.081).contains(&checks::peak(on)));
    }
    let mut combined = on;
    combined.0.extend(off.0);
    combined.1.extend(off.1);
    combined.0.extend(manual_on.0);
    combined.1.extend(manual_on.1);
    save("96-pitch-gate.wav", &combined);
}

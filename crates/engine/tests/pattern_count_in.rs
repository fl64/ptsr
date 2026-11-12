//! Audio-timed count-in through production project queues, without devices or App input.

use engine::capture::{Event, Kind, Phase, Snapshot};
use engine::checks::{first_non_finite, max_step, onsets, peak};
use engine::pattern::{Grid, Hit, RawPattern};
use engine::rt::{ProjectCommand, ProjectMessage, ProjectState, Realtime, Stats};
use engine::{Command, Engine, PadMode, Retired, Sample};
use std::sync::{
    Arc,
    atomic::Ordering::{Acquire, Release},
};

const SR: u32 = 48_000;
type Audio = (Vec<f32>, Vec<f32>);

struct Rig {
    tx: rtrb::Producer<ProjectCommand>,
    events: rtrb::Consumer<Event>,
    garbage: rtrb::Consumer<Retired>,
    rt: Realtime,
    stats: Arc<Stats>,
    project: Arc<ProjectState>,
}

fn tone(sr: u32, frames: usize, hz: f32, gain: f32) -> Vec<f32> {
    (0..frames)
        .map(|i| {
            let attack = (i as f32 / (sr as f32 * 0.002)).min(1.0);
            let tail = ((frames - i) as f32 / (sr as f32 * 0.01)).min(1.0);
            gain * attack * tail * (std::f32::consts::TAU * hz * i as f32 / sr as f32).sin()
        })
        .collect()
}

fn a1(rows: Vec<Hit>) -> Command {
    Command::SetPattern {
        pattern: Some(Arc::new(
            RawPattern::new(1, Grid::Off, 100, rows).unwrap().compile(),
        )),
    }
}

fn rhythm() -> Vec<Hit> {
    vec![
        Hit::new(0, 0, 127, 0).unwrap(),
        Hit::new(960, 0, 127, 0).unwrap(),
    ]
}

impl Rig {
    fn new(sr: u32, bpm: f64, metro: bool, rows: Vec<Hit>) -> Self {
        let mut engine = Engine::with_sounds(sr, engine::click(sr), vec![]);
        for (pad, hz, frames, gain) in [
            (0, 220.0, sr as usize * 8 / 100, 0.06),
            (1, 55.0, sr as usize * 3, 0.025),
        ] {
            engine.handle(
                0,
                Command::SetPadSound {
                    pad,
                    sound: Some(Arc::new(Sample {
                        left: tone(sr, frames, hz, gain).into_boxed_slice(),
                        right: Some(tone(sr, frames, hz * 1.5, gain).into_boxed_slice()),
                    })),
                },
            );
        }
        engine.handle(
            0,
            Command::SetPadMode {
                pad: 1,
                mode: PadMode::Gate,
            },
        );
        engine.handle(
            0,
            Command::SetPadEnvelope {
                pad: 1,
                attack_ms: 0,
                decay_ms: 0,
                release_ms: 100,
            },
        );
        engine.handle(0, Command::SetTempo { bpm });
        engine.handle(0, Command::SetMetronome { enabled: metro });
        engine.handle(0, a1(rows));
        let events = engine.enable_capture();
        let (tx, rx) = rtrb::RingBuffer::new(1024);
        let (garbage_tx, garbage) = rtrb::RingBuffer::new(1024);
        engine.set_garbage(garbage_tx);
        let stats = Arc::new(Stats::default());
        let project = Arc::new(ProjectState::default());
        let rt = Realtime::with_project(engine, vec![rx], stats.clone(), sr, project.clone());
        Self {
            tx,
            events,
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

    fn render(&mut self, frames: usize, blocks: &[usize]) -> Audio {
        let (mut left, mut right) = (vec![0.0; frames], vec![0.0; frames]);
        if frames == 0 {
            self.rt.process(&mut [], &mut []);
        }
        let mut at = 0;
        for &size in blocks.iter().cycle() {
            if at == frames {
                break;
            }
            let end = (at + size).min(frames);
            while self.garbage.pop().is_ok() {}
            self.rt.process(&mut left[at..end], &mut right[at..end]);
            at = end;
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
}

fn append(out: &mut Audio, next: Audio) {
    out.0.extend(next.0);
    out.1.extend(next.1);
}

fn compare(got: &Audio, expected: &Audio) {
    assert_eq!(got.0.len(), expected.0.len());
    assert_eq!(got.1.len(), expected.1.len());
    for (channel, (a, b)) in [(&got.0, &expected.0), (&got.1, &expected.1)]
        .into_iter()
        .enumerate()
    {
        for (i, (&a, &b)) in a.iter().zip(b.iter()).enumerate() {
            assert!(
                (a - b).abs() <= 0.000001,
                "channel {channel} frame {i}: {a} != {b}"
            );
        }
        assert_eq!(first_non_finite(a), None);
        assert!(peak(a) < 1.0);
        assert!(max_step(a) < 0.11);
    }
}

/// Scenario "Count in before the first recorded beat".
#[test]
fn count_in_before_the_first_recorded_beat() {
    for sr in [44_100, 48_000, 96_000] {
        for bpm in [40.0, 120.0, 172.0, 300.0] {
            for blocks in [&[1][..], &[128][..], &[1024][..], &[13, 128, 1024, 71][..]] {
                let mut actual = Rig::new(sr, bpm, false, rhythm());
                let mut reference = Rig::new(sr, bpm, true, vec![]);
                actual.send(Command::PatternRecordStart { pass: 1 });
                reference.send(Command::TransportPlay);
                for rig in [&mut actual, &mut reference] {
                    rig.send(Command::SetPadEnvelope {
                        pad: 1,
                        attack_ms: 0,
                        decay_ms: 0,
                        release_ms: 5,
                    });
                    rig.send(Command::PadPress {
                        pad: 1,
                        velocity: 1.0,
                    });
                    rig.send(Command::PadRelease { pad: 1 });
                }
                actual.render(0, blocks);
                assert_eq!(actual.state().phase, Phase::CountIn);
                assert_eq!(actual.state().count_in_beat, 4);
                let beat = f64::from(sr) * 60.0 / bpm;
                let zero = (4.0 * beat).round() as usize;
                let mut out = actual.render(zero, blocks);
                let mut expected = reference.render(zero, blocks);
                assert!(matches!(actual.state().phase, Phase::Recording));
                assert_eq!(actual.state().ticks, 0.0);
                assert_eq!(actual.state().count_in_beat, 0);
                assert_eq!(
                    actual.stats.pattern_hits(0).unwrap()[0],
                    0,
                    "zero's audio frame has not played"
                );
                assert!(!actual.stats.transport(0).unwrap().metronome);
                let attacks = onsets(&out.0, 1e-6, engine::click(sr).len());
                assert_eq!(attacks.len(), 4, "{sr} {bpm} {blocks:?}: {attacks:?}");
                for (i, &at) in attacks.iter().enumerate() {
                    assert!(at.abs_diff((i as f64 * beat).round() as usize) <= 1);
                }
                assert!(actual.drain().is_empty());
                reference.send(Command::TransportStop);
                reference.send(Command::SetMetronome { enabled: false });
                reference.send(Command::PadPress {
                    pad: 0,
                    velocity: 1.0,
                });
                append(&mut out, actual.render(beat.round() as usize, blocks));
                append(
                    &mut expected,
                    reference.render(beat.round() as usize, blocks),
                );
                reference.send(Command::PadPress {
                    pad: 0,
                    velocity: 1.0,
                });
                append(&mut out, actual.render(sr as usize / 4, blocks));
                append(&mut expected, reference.render(sr as usize / 4, blocks));
                compare(&out, &expected);
                assert!(actual.stats.pattern_hits(0).unwrap()[0] > 0);
                assert_eq!(actual.state().phase, Phase::Recording);
                assert!(
                    actual.drain().is_empty(),
                    "sequenced hits and count-in clicks are not captured"
                );
            }
        }
    }
}

#[test]
fn warm_up_gate_spans_zero_without_becoming_a_recorded_onset() {
    let mut actual = Rig::new(SR, 120.0, false, vec![]);
    let mut reference = Rig::new(SR, 120.0, true, vec![]);
    actual.send(Command::PatternRecordStart { pass: 1 });
    reference.send(Command::TransportPlay);
    let mut out = actual.render(48_000, &[128]);
    let mut expected = reference.render(48_000, &[128]);
    for rig in [&mut actual, &mut reference] {
        rig.send(Command::PadPress {
            pad: 1,
            velocity: 1.0,
        });
    }
    append(&mut out, actual.render(48_000, &[1024, 71]));
    append(&mut expected, reference.render(48_000, &[1024, 71]));
    assert_eq!(actual.state().phase, Phase::Recording);
    assert_eq!(actual.state().captured_onsets, 0);
    assert!(actual.drain().is_empty());
    reference.send(Command::TransportStop);
    for rig in [&mut actual, &mut reference] {
        rig.send(Command::SetMetronome { enabled: false });
    }
    append(&mut out, actual.render(6000, &[128]));
    append(&mut expected, reference.render(6000, &[128]));
    assert!(
        peak(&out.0[96_000..102_000]) > 0.02,
        "warm-up Gate survives the recording start"
    );
    for rig in [&mut actual, &mut reference] {
        rig.send(Command::PadRelease { pad: 1 });
    }
    append(&mut out, actual.render(12_000, &[128]));
    append(&mut expected, reference.render(12_000, &[128]));
    compare(&out, &expected);
    assert_eq!(peak(&out.0[108_000..]), 0.0);
    assert_eq!(actual.state().captured_onsets, 0);
    assert!(
        actual
            .drain()
            .iter()
            .all(|e| !matches!(e.kind, Kind::Onset { .. }))
    );
    actual.send(Command::PadPress {
        pad: 0,
        velocity: 0.5,
    });
    actual.render(0, &[128]);
    let onset = actual.drain().pop().unwrap();
    assert_eq!((onset.pass, onset.run), (1, 1));
    assert!((onset.ticks - 720.0).abs() < 1e-8);
    assert!(matches!(onset.kind, Kind::Onset { tick: 720, .. }));
}

/// Scenario "Change tempo during count in".
#[test]
fn change_tempo_during_count_in() {
    for bpm in [40.0, 90.0, 172.0, 300.0] {
        for metro in [false, true] {
            let mut actual = Rig::new(SR, 120.0, metro, rhythm());
            let mut reference = Rig::new(SR, 120.0, true, vec![]);
            actual.send(Command::PatternRecordStart { pass: 1 });
            reference.send(Command::TransportPlay);
            let mut out = actual.render(12_000, &[128, 13]);
            let mut expected = reference.render(12_000, &[128, 13]);
            let before = actual.state();
            for rig in [&mut actual, &mut reference] {
                rig.send(Command::SetTempo { bpm });
            }
            actual.render(0, &[128]);
            assert_eq!(actual.state(), before);
            let beat = f64::from(SR) * 60.0 / bpm;
            let remaining = (3.5 * beat).round() as usize;
            append(&mut out, actual.render(remaining, &[1024, 71]));
            append(&mut expected, reference.render(remaining, &[1024, 71]));
            assert_eq!(
                (actual.state().phase, actual.state().ticks),
                (Phase::Recording, 0.0)
            );
            let clicks = onsets(&out.0, 1e-6, engine::click(SR).len());
            assert_eq!(clicks.len(), 4);
            for (i, &at) in clicks.iter().enumerate() {
                let wanted = if i == 0 {
                    0
                } else {
                    12_000 + ((i as f64 - 0.5) * beat).round() as usize
                };
                assert!(at.abs_diff(wanted) <= 1, "{bpm}: {at} != {wanted}");
            }
            reference.send(Command::TransportStop);
            reference.send(Command::SetMetronome { enabled: metro });
            reference.send(Command::TransportPlay);
            reference.send(Command::PadPress {
                pad: 0,
                velocity: 1.0,
            });
            append(&mut out, actual.render(beat.round() as usize, &[128]));
            append(
                &mut expected,
                reference.render(beat.round() as usize, &[128]),
            );
            reference.send(Command::PadPress {
                pad: 0,
                velocity: 1.0,
            });
            append(&mut out, actual.render(4000, &[128]));
            append(&mut expected, reference.render(4000, &[128]));
            compare(&out, &expected);
            assert_eq!(actual.stats.transport(0).unwrap().metronome, metro);
            assert!(actual.drain().is_empty());
        }
    }
}

#[test]
fn cancellation_keeps_the_warm_up_gate_and_its_original_release() {
    for at in [0, 1, 24_000, 95_999] {
        let mut actual = Rig::new(SR, 120.0, false, vec![]);
        let mut reference = Rig::new(SR, 120.0, true, vec![]);
        actual.send(Command::PatternRecordStart { pass: 7 });
        reference.send(Command::TransportPlay);
        for rig in [&mut actual, &mut reference] {
            rig.send(Command::PadPress {
                pad: 1,
                velocity: 1.0,
            });
        }
        let mut out = actual.render(at, &[128]);
        let mut expected = reference.render(at, &[128]);
        actual.send(Command::PatternRecordEnd { pass: 7 });
        reference.send(Command::TransportStop);
        append(&mut out, actual.render(6000, &[128]));
        append(&mut expected, reference.render(6000, &[128]));
        assert_eq!(
            (
                actual.state().phase,
                actual.state().count_in_beat,
                actual.state().sequence
            ),
            (Phase::Idle, 0, 0)
        );
        assert!(!actual.stats.transport(0).unwrap().playing);
        assert_eq!(actual.stats.transport(0).unwrap().pattern_ticks, 3840);
        assert!(peak(&out.0[at..]) > 0.02);
        for rig in [&mut actual, &mut reference] {
            rig.send(Command::PadRelease { pad: 1 });
        }
        append(&mut out, actual.render(110_000, &[1024]));
        append(&mut expected, reference.render(110_000, &[1024]));
        compare(&out, &expected);
        assert_eq!(peak(&out.0[out.0.len() - 4800..]), 0.0);
        assert!(actual.drain().is_empty());
        actual.send(Command::PatternRecordStart { pass: 7 });
        actual.send(Command::PatternRecordEnd { pass: 7 });
        actual.render(0, &[128]);
        assert_eq!(
            actual.state().phase,
            Phase::Idle,
            "cancelled identity cannot reopen"
        );
        actual.send(Command::PatternRecordStart { pass: 8 });
        actual.render(0, &[128]);
        assert_eq!(
            (actual.state().phase, actual.state().count_in_beat),
            (Phase::CountIn, 4)
        );
    }
}

/// Scenario "Transport requests cannot start recording early".
#[test]
fn transport_requests_cannot_start_recording_early() {
    for command in [
        Command::TransportStop,
        Command::TransportPlay,
        Command::TransportToggle,
    ] {
        let playing = !matches!(command, Command::TransportStop);
        let mut rig = Rig::new(SR, 120.0, false, rhythm());
        rig.send(Command::PatternRecordStart { pass: 1 });
        rig.render(30_000, &[128]);
        rig.send(command);
        rig.render(0, &[128]);
        assert_eq!(rig.state().phase, Phase::Idle);
        assert_eq!(rig.stats.transport(0).unwrap().playing, playing);
        assert_eq!(rig.stats.transport(0).unwrap().ticks, 0.0);
        rig.send(Command::PadPress {
            pad: 1,
            velocity: 1.0,
        });
        rig.render(120_000, &[1024]);
        assert_eq!(rig.state().phase, Phase::Idle);
        assert!(rig.drain().is_empty());
        assert_eq!(rig.stats.pattern_hits(0).unwrap()[0] > 0, playing);
    }
}

/// Scenario "Repeated recording requests preserve the count in".
#[test]
fn repeated_recording_requests_preserve_the_count_in() {
    let mut rig = Rig::new(SR, 120.0, false, rhythm());
    rig.send(Command::PatternRecordStart { pass: 5 });
    rig.render(12_000, &[128]);
    let state = rig.state();
    for cmd in [
        Command::PatternRecordStart { pass: 5 },
        Command::PatternRecordStart { pass: 6 },
        Command::PatternRecordEnd { pass: 4 },
        Command::PatternRecordAck {
            pass: 5,
            sequence: 0,
        },
    ] {
        rig.send(cmd);
    }
    for _ in 0..4 {
        rig.render(0, &[128]);
        assert_eq!(rig.state(), state);
    }
    for (frames, beat) in [(12_000, 3), (24_000, 2), (24_000, 1), (23_999, 1)] {
        rig.render(frames, &[1024]);
        assert_eq!(rig.state().count_in_beat, beat);
        let transport = rig.stats.transport(0).unwrap();
        assert!(!transport.playing);
        assert_eq!(transport.ticks, 0.0);
    }
    assert_eq!(rig.state().phase, Phase::CountIn);
    assert_eq!(rig.state().count_in_beat, 1);
    rig.render(1, &[1]);
    assert_eq!(
        (rig.state().phase, rig.state().pass, rig.state().ticks),
        (Phase::Recording, 5, 0.0)
    );
    rig.send(Command::PadPress {
        pad: 0,
        velocity: 1.0,
    });
    rig.render(0, &[128]);
    let onset = rig.drain().pop().unwrap();
    assert_eq!((onset.ticks, onset.run, onset.sequence), (0.0, 1, 1));
    rig.send(Command::TransportStop);
    rig.send(Command::PatternRecordAck {
        pass: 5,
        sequence: 1,
    });
    rig.send(Command::PatternRecordStart { pass: 6 });
    rig.render(96_000, &[1024]);
    assert_eq!(
        (rig.state().phase, rig.state().ticks),
        (Phase::Recording, 0.0)
    );
    rig.send(Command::PadPress {
        pad: 0,
        velocity: 1.0,
    });
    rig.render(0, &[128]);
    let next = rig.drain().pop().unwrap();
    assert_eq!(
        (next.pass, next.run, next.ticks, next.sequence),
        (6, 2, 0.0, 1)
    );
}

#[test]
fn metronome_preference_changes_do_not_mute_or_restart_count_in() {
    for initial in [false, true] {
        let mut actual = Rig::new(SR, 120.0, initial, vec![]);
        let mut reference = Rig::new(SR, 120.0, true, vec![]);
        actual.send(Command::PatternRecordStart { pass: 1 });
        reference.send(Command::TransportPlay);
        let mut out = actual.render(12_000, &[128]);
        let mut expected = reference.render(12_000, &[128]);
        actual.send(Command::SetMetronome { enabled: !initial });
        append(&mut out, actual.render(84_000, &[1024]));
        append(&mut expected, reference.render(84_000, &[1024]));
        compare(&out, &expected);
        assert_eq!(actual.stats.transport(0).unwrap().metronome, !initial);
        reference.send(Command::TransportStop);
        reference.send(Command::SetMetronome { enabled: !initial });
        reference.send(Command::TransportPlay);
        compare(
            &actual.render(48_000, &[128]),
            &reference.render(48_000, &[128]),
        );
    }
}

#[test]
fn count_in_boundary_inside_a_callback_does_not_delay_the_first_a1_hit() {
    for block in [128, 1024] {
        for before in [1, block / 2, block - 1] {
            let mut actual = Rig::new(SR, 120.0, false, rhythm());
            let mut reference = Rig::new(SR, 120.0, false, vec![]);
            actual.send(Command::PatternRecordStart { pass: 1 });
            actual.render(96_000 - before, &[128]);
            let out = actual.render(block, &[block]);
            let mut expected = reference.render(before, &[block]);
            reference.send(Command::PadPress {
                pad: 0,
                velocity: 1.0,
            });
            append(&mut expected, reference.render(block - before, &[block]));
            compare(&out, &expected);
            assert_eq!(actual.state().phase, Phase::Recording);
            assert!((actual.state().ticks - (block - before) as f64 * 0.04).abs() < 1e-8);
            assert!(actual.drain().is_empty());
        }
    }
}

#[test]
fn count_in_preserves_all_32_manual_pad_voices() {
    let mut actual = Rig::new(SR, 120.0, false, vec![]);
    let mut reference = Rig::new(SR, 120.0, false, vec![]);
    let mut clicks = Rig::new(SR, 120.0, true, vec![]);
    actual.send(Command::PatternRecordStart { pass: 1 });
    clicks.send(Command::TransportPlay);
    for pad in 0..engine::MAX_VOICES {
        for rig in [&mut actual, &mut reference] {
            rig.send(Command::SetPadSound {
                pad,
                sound: Some(Arc::new(Sample::mono(vec![0.001; 120_000]))),
            });
            rig.send(Command::PadPress { pad, velocity: 1.0 });
        }
    }
    let out = actual.render(96_000, &[128]);
    let mut expected = reference.render(96_000, &[128]);
    let metronome = clicks.render(96_000, &[128]);
    for (a, b) in expected.0.iter_mut().zip(metronome.0) {
        *a += b;
    }
    for (a, b) in expected.1.iter_mut().zip(metronome.1) {
        *a += b;
    }
    compare(&out, &expected);
    assert_eq!(actual.state().phase, Phase::Recording);
    assert!(actual.drain().is_empty());
    let tail = actual.render(128, &[128]);
    assert!(tail.0.iter().all(|&v| (v - 0.032).abs() < 1e-7));
}

#[test]
fn project_change_discards_count_in_and_rejects_old_or_unready_requests() {
    let mut rig = Rig::new(SR, 120.0, false, rhythm());
    rig.send(Command::PatternRecordStart { pass: 1 });
    rig.render(24_000, &[128]);
    rig.project.epoch.store(1, Release);
    for command in [
        Command::PatternRecordStart { pass: 2 },
        Command::PatternRecordEnd { pass: 1 },
        Command::TransportPlay,
    ] {
        rig.tagged(0, true, ProjectMessage::Command(command));
    }
    rig.tagged(
        1,
        false,
        ProjectMessage::Command(Command::PatternRecordStart { pass: 1 }),
    );
    rig.tagged(1, false, ProjectMessage::Command(a1(vec![])));
    rig.tagged(1, false, ProjectMessage::Ready);
    for _ in 0..1000 {
        rig.render(128, &[128]);
        if rig.project.ready.load(Acquire) == 1 {
            break;
        }
    }
    assert_eq!(rig.project.ready.load(Acquire), 1);
    assert_eq!(
        rig.state(),
        Snapshot {
            epoch: 1,
            ..Snapshot::default()
        }
    );
    assert_eq!(rig.stats.recording(0), None);
    let silent = rig.render(120_000, &[1024]);
    assert_eq!((peak(&silent.0), peak(&silent.1)), (0.0, 0.0));
    assert!(rig.drain().is_empty());
    rig.send(Command::PatternRecordStart { pass: 1 });
    rig.render(0, &[128]);
    assert_eq!(
        (rig.state().phase, rig.state().count_in_beat),
        (Phase::CountIn, 4)
    );
}

fn section(change: bool, cancel: bool) -> Audio {
    let mut actual = Rig::new(SR, 120.0, false, if cancel { vec![] } else { rhythm() });
    let mut reference = Rig::new(SR, 120.0, true, vec![]);
    actual.send(Command::PatternRecordStart { pass: 1 });
    reference.send(Command::TransportPlay);
    let total = if change { 192_000 } else { 144_000 };
    let zero = if change { 124_000 } else { 96_000 };
    let beat = if change { 32_000 } else { 24_000 };
    let mut actions = Vec::new();
    if change {
        actions.push((12_000, Command::SetTempo { bpm: 90.0 }, true));
    }
    if cancel {
        actions.extend([
            (
                36_000,
                Command::PadPress {
                    pad: 1,
                    velocity: 1.0,
                },
                true,
            ),
            (42_000, Command::PatternRecordEnd { pass: 1 }, false),
            (54_000, Command::PadRelease { pad: 1 }, true),
        ]);
    } else {
        actions.extend([
            (
                zero,
                Command::PadPress {
                    pad: 0,
                    velocity: 1.0,
                },
                false,
            ),
            (
                zero + beat,
                Command::PadPress {
                    pad: 0,
                    velocity: 1.0,
                },
                false,
            ),
        ]);
    }
    let (mut out, mut expected) = ((Vec::new(), Vec::new()), (Vec::new(), Vec::new()));
    let mut at = 0;
    for (frame, cmd, both) in actions {
        append(&mut out, actual.render(frame - at, &[128]));
        append(&mut expected, reference.render(frame - at, &[128]));
        if both {
            actual.send(cmd.clone());
            reference.send(cmd);
        } else if cancel {
            actual.send(cmd);
            reference.send(Command::TransportStop);
        } else {
            if frame == zero {
                reference.send(Command::TransportStop);
                reference.send(Command::SetMetronome { enabled: false });
            }
            reference.send(cmd);
        }
        at = frame;
    }
    // End after the second A1 note, before the next bar can repeat.
    if !cancel {
        let end = zero + beat + 6000;
        append(&mut out, actual.render(end - at, &[128]));
        append(&mut expected, reference.render(end - at, &[128]));
        actual.send(Command::TransportStop);
        at = end;
    }
    append(&mut out, actual.render(total - at, &[128]));
    append(&mut expected, reference.render(total - at, &[128]));
    compare(&out, &expected);
    assert!(actual.drain().is_empty());
    for channel in [&out.0, &out.1] {
        assert_eq!(peak(&channel[channel.len() - 4800..]), 0.0);
    }
    out
}

#[test]
fn count_in_owner_render_has_four_beats_tempo_change_and_safe_cancellation() {
    let mut out = section(false, false);
    append(&mut out, section(true, false));
    append(&mut out, section(false, true));
    for channel in [&out.0, &out.1] {
        assert_eq!(first_non_finite(channel), None);
        assert!((0.4..0.6).contains(&peak(channel)));
        assert!(max_step(channel) < 0.11);
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../renders/97-pattern-count-in.wav");
    engine::offline::write_wav(&path, SR, &out.0, &out.1).unwrap();
    println!(
        "97-pattern-count-in.wav: {} frames, 48 kHz stereo, sections 0-3 / 3-7 / 7-10 s",
        out.0.len()
    );
    println!(
        "peaks: {} / {}; max steps: {} / {}",
        peak(&out.0),
        peak(&out.1),
        max_step(&out.0),
        max_step(&out.1)
    );
}

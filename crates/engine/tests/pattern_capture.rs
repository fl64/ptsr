//! Capture-protocol tests. Count-in, Gate completion and App publication have separate suites.

use engine::capture::{CAPACITY, Event, Kind, Phase, Snapshot};
use engine::pattern::{Grid, Hit, RawPattern};
use engine::rt::{ProjectCommand, ProjectMessage, ProjectState, Realtime, Stats};
use engine::{Command, Engine, PadMode, Retired, Sample};
use std::sync::{
    Arc,
    atomic::Ordering::{Acquire, Release},
};

const SR: u32 = 48_000;

fn pattern(rows: Vec<Hit>) -> Command {
    Command::SetPattern {
        pattern: Some(Arc::new(
            RawPattern::new(1, Grid::Off, 100, rows).unwrap().compile(),
        )),
    }
}

struct Rig {
    tx: rtrb::Producer<ProjectCommand>,
    events: rtrb::Consumer<Event>,
    garbage: rtrb::Consumer<Retired>,
    rt: Realtime,
    stats: Arc<Stats>,
    project: Arc<ProjectState>,
}

impl Rig {
    fn new(rows: Vec<Hit>) -> Self {
        let mut engine = Engine::with_sounds(SR, engine::click(SR), vec![vec![0.001; SR as usize]]);
        engine.handle(0, pattern(rows));
        let events = engine.enable_capture();
        let (garbage_tx, garbage) = rtrb::RingBuffer::new(1024);
        engine.set_garbage(garbage_tx);
        let (tx, rx) = rtrb::RingBuffer::new(1024);
        let stats = Arc::new(Stats::default());
        let project = Arc::new(ProjectState::default());
        let rt = Realtime::with_project(engine, vec![rx], stats.clone(), SR, project.clone());
        Self {
            tx,
            events,
            garbage,
            rt,
            stats,
            project,
        }
    }

    fn tagged(&mut self, epoch: u64, ready_at_send: bool, command: Command) {
        self.tx
            .push(ProjectCommand {
                epoch,
                ready_at_send,
                command: ProjectMessage::Command(command),
            })
            .unwrap();
    }

    fn send(&mut self, command: Command) {
        let epoch = self.project.epoch.load(Acquire);
        self.tagged(epoch, self.project.ready.load(Acquire) == epoch, command);
    }

    fn block(&mut self, frames: usize) -> (Vec<f32>, Vec<f32>) {
        while self.garbage.pop().is_ok() {}
        let (mut left, mut right) = (vec![0.0; frames], vec![0.0; frames]);
        self.rt.process(&mut left, &mut right);
        (left, right)
    }

    fn state(&self) -> Snapshot {
        self.stats
            .recording(self.project.epoch.load(Acquire))
            .unwrap()
    }

    fn start(&mut self, pass: u64) {
        self.send(Command::TransportPlay);
        self.send(Command::PatternRecordStart { pass });
    }

    fn drain(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok(event) = self.events.pop() {
            events.push(event);
        }
        events
    }
}

#[test]
fn capture_start_requires_setup_pattern_and_fresh_nonzero_identity() {
    let mut engine = Engine::with_sounds(SR, vec![], vec![]);
    engine.handle(0, pattern(vec![]));
    engine.handle(0, Command::TransportPlay);
    engine.handle(0, Command::PatternRecordStart { pass: 1 });
    assert_eq!(engine.recording().phase, Phase::Idle, "no feedback queue");
    let _events = engine.enable_capture();
    engine.handle(0, Command::TransportStop);
    engine.handle(0, Command::PatternRecordStart { pass: 1 });
    assert_eq!(engine.recording().phase, Phase::CountIn);
    engine.handle(0, Command::PatternRecordEnd { pass: 1 });
    assert_eq!(engine.recording().phase, Phase::Idle);
    engine.handle(0, Command::TransportPlay);
    engine.handle(0, Command::SetPattern { pattern: None });
    engine.handle(0, Command::PatternRecordStart { pass: 2 });
    assert_eq!(engine.recording().phase, Phase::Idle, "no accepted A1");
    engine.handle(0, pattern(vec![]));
    engine.handle(0, Command::PatternRecordStart { pass: 0 });
    engine.handle(0, Command::PatternRecordStart { pass: 1 });
    assert_eq!(
        engine.recording().phase,
        Phase::Idle,
        "cancelled pass stays consumed"
    );
    engine.handle(0, Command::PatternRecordStart { pass: 2 });
    assert_eq!(engine.recording().phase, Phase::Recording);
}

#[test]
fn accepted_manual_capture_excludes_pattern_preview_click_and_invalid_pad() {
    let mut rig = Rig::new(vec![Hit::new(0, 2, 127, 0).unwrap()]);
    rig.send(Command::SetPadMode {
        pad: 127,
        mode: PadMode::Gate,
    });
    rig.send(Command::SetPadSound {
        pad: 127,
        sound: None,
    });
    rig.send(Command::SetPadGain { pad: 0, gain: 0.0 });
    rig.start(3);
    rig.send(Command::SetMetronome { enabled: true });
    rig.send(Command::Click { velocity: 1.0 });
    rig.send(Command::Preview {
        sound: Some(Arc::new(Sample::mono(vec![0.001; 128]))),
        region: Default::default(),
    });
    rig.send(Command::PadPress {
        pad: 127,
        velocity: 81.0 / 127.0,
    });
    rig.send(Command::PadRelease { pad: 127 });
    rig.send(Command::PadPress {
        pad: 0,
        velocity: 1.0 / 127.0,
    });
    rig.send(Command::PadPress {
        pad: 1,
        velocity: 0.0,
    });
    rig.send(Command::PadPress {
        pad: 128,
        velocity: 1.0,
    });
    rig.send(Command::PadRelease { pad: 128 });
    let (left, right) = rig.block(128);
    assert!(
        left.iter()
            .chain(&right)
            .all(|v| v.is_finite() && v.abs() < 1.0)
    );
    assert!(left.iter().any(|v| *v != 0.0));
    let events = rig.drain();
    assert_eq!(events.len(), 3);
    for (i, event) in events.iter().enumerate() {
        assert_eq!(
            (event.epoch, event.pass, event.sequence, event.ticks),
            (0, 3, i as u64 + 1, 0.0)
        );
    }
    assert_eq!(
        events[0].kind,
        Kind::Onset {
            pad: 127,
            velocity: 81,
            mode: PadMode::Gate,
            tick: 0
        }
    );
    assert_eq!(events[1].kind, Kind::Release { pad: 127 });
    assert_eq!(
        events[2].kind,
        Kind::Onset {
            pad: 0,
            velocity: 1,
            mode: PadMode::OneShot,
            tick: 0
        }
    );
    assert_eq!(
        rig.stats.pattern_hits(0).unwrap()[127],
        0,
        "empty pad is not audible"
    );
    assert_eq!(
        rig.stats.pattern_hits(0).unwrap()[0],
        0,
        "muted pad is not audible"
    );
    assert!(
        rig.stats.pattern_hits(0).unwrap()[2] > 0,
        "sequencer really played"
    );
    rig.block(4096);
    assert!(
        rig.drain().is_empty(),
        "sound processing cannot create capture"
    );
}

#[test]
fn capture_ticks_follow_accepted_blocks_with_or_without_feedback_draining() {
    for delayed in [false, true] {
        let mut rig = Rig::new(vec![]);
        rig.start(1);
        let mut elapsed = 0;
        let mut received = Vec::new();
        for frames in [13, 128, 1024, 71] {
            rig.send(Command::PadPress {
                pad: 16,
                velocity: 0.5,
            });
            rig.block(frames);
            if !delayed {
                received.extend(rig.drain());
            }
            elapsed += frames;
            let state = rig.state();
            assert!((state.ticks - elapsed as f64 * 0.04).abs() < 1e-8);
            for _ in 0..3 {
                rig.block(0);
                assert_eq!(
                    rig.state(),
                    state,
                    "empty processing changes no musical time"
                );
            }
        }
        received.extend(rig.drain());
        assert_eq!(received.len(), 4);
        for (event, frame) in received.iter().zip([0, 13, 141, 1165]) {
            assert!((event.ticks - frame as f64 * 0.04).abs() < 1e-8);
            assert_eq!(
                event.kind,
                Kind::Onset {
                    pad: 16,
                    velocity: 64,
                    mode: PadMode::OneShot,
                    tick: (frame as f64 * 0.04).round() as u32,
                }
            );
        }
    }
}

#[test]
fn offline_capture_rounds_midpoints_later_and_wraps_only_loop_position() {
    let mut engine = Engine::with_sounds(SR, vec![], vec![]);
    let mut events = engine.enable_capture();
    engine.handle(0, pattern(vec![]));
    engine.handle(0, Command::SetTempo { bpm: 300.0 });
    engine.handle(0, Command::TransportPlay);
    engine.handle(0, Command::PatternRecordStart { pass: 1 });
    for (offset, gain, tick, velocity) in [
        (4, 0.0001, 0, 1),
        (5, 0.5, 1, 64),
        (6, 2.0, 1, 127),
        (38_394, 1.0, 3839, 127),
        (38_395, 1.0, 0, 127),
        (38_406, 1.0, 1, 127),
    ] {
        engine.handle(
            offset,
            Command::PadPress {
                pad: 127,
                velocity: gain,
            },
        );
        let event = events.pop().unwrap();
        assert!((event.ticks - offset as f64 / 10.0).abs() < 1e-9);
        assert_eq!(
            event.kind,
            Kind::Onset {
                pad: 127,
                velocity,
                mode: PadMode::OneShot,
                tick
            }
        );
    }
    for gain in [0.0, -0.1, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        engine.handle(
            0,
            Command::PadPress {
                pad: 0,
                velocity: gain,
            },
        );
        assert!(events.pop().is_err());
    }
}

#[test]
fn pass_completion_requires_matching_identity_sequence_and_drained_feedback() {
    let mut rig = Rig::new(vec![]);
    rig.start(7);
    rig.block(128);
    rig.send(Command::PatternRecordStart { pass: 7 });
    rig.send(Command::PatternRecordStart { pass: 8 });
    rig.send(Command::PatternRecordEnd { pass: 6 });
    rig.send(Command::PadPress {
        pad: 0,
        velocity: 1.0,
    });
    rig.send(Command::PatternRecordEnd { pass: 7 });
    rig.send(Command::PatternRecordStart { pass: 8 });
    rig.block(128);
    let end = rig.state();
    assert_eq!(
        (end.phase, end.pass, end.sequence),
        (Phase::Finishing, 7, 1)
    );
    assert!((end.ticks - 5.12).abs() < 1e-9);
    rig.send(Command::PatternRecordEnd { pass: 7 });
    rig.send(Command::PatternRecordAck {
        pass: 7,
        sequence: 1,
    });
    rig.block(128);
    assert_eq!(rig.state(), end, "unread event keeps the terminal boundary");
    assert_eq!(rig.drain().len(), 1);
    for (pass, sequence) in [(6, 1), (7, 0), (7, 2)] {
        rig.send(Command::PatternRecordAck { pass, sequence });
        rig.block(128);
        assert_eq!(rig.state(), end);
    }
    rig.send(Command::PatternRecordAck {
        pass: 7,
        sequence: 1,
    });
    rig.block(128);
    assert_eq!(rig.state().phase, Phase::Idle);
    rig.send(Command::PatternRecordStart { pass: 7 });
    rig.send(Command::PatternRecordStart { pass: 6 });
    rig.block(128);
    assert_eq!(
        rig.state().phase,
        Phase::Idle,
        "old starts cannot resume a pass"
    );
    rig.send(Command::PatternRecordStart { pass: 8 });
    rig.send(Command::PatternRecordEnd { pass: 7 });
    rig.send(Command::PatternRecordAck {
        pass: 7,
        sequence: 1,
    });
    rig.block(128);
    assert_eq!(
        (rig.state().phase, rig.state().pass, rig.state().sequence),
        (Phase::Recording, 8, 0)
    );
}

#[test]
fn full_capture_queue_retains_terminal_boundary_counts_losses_and_never_replays() {
    let mut rig = Rig::new(vec![]);
    rig.start(1);
    for batch in 0..6 {
        for hit in 0..100 {
            rig.send(Command::PadPress {
                pad: (batch * 100 + hit) % 128,
                velocity: 1.0,
            });
        }
        let (left, right) = rig.block(128);
        assert!(
            left.iter()
                .chain(&right)
                .all(|v| v.is_finite() && v.abs() < 1.0)
        );
        assert!(
            left.iter().any(|v| *v > 0.0),
            "manual sound continues under pressure"
        );
    }
    for pad in 0..3 {
        rig.send(Command::PadRelease { pad });
    }
    rig.send(Command::PatternRecordEnd { pass: 1 });
    rig.block(128);
    let end = rig.state();
    assert_eq!(end.phase, Phase::Finishing);
    assert_eq!(end.sequence, 603);
    assert_eq!(end.dropped_onsets, 600 - CAPACITY as u64);
    assert_eq!(end.dropped_events, 603 - CAPACITY as u64);
    rig.send(Command::PatternRecordStart { pass: 2 });
    for _ in 0..3 {
        rig.block(128);
        assert_eq!(rig.state(), end);
    }
    let events = rig.drain();
    assert_eq!(events.len(), CAPACITY);
    assert_eq!(
        events
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        (1..=512).collect::<Vec<_>>()
    );
    rig.block(128);
    assert!(rig.drain().is_empty(), "omitted events stay omitted");
    rig.send(Command::PatternRecordAck {
        pass: 1,
        sequence: 603,
    });
    rig.send(Command::PatternRecordStart { pass: 2 });
    rig.send(Command::PadPress {
        pad: 127,
        velocity: 1.0,
    });
    rig.block(128);
    let events = rig.drain();
    assert_eq!(events.len(), 1);
    assert_eq!((events[0].pass, events[0].sequence), (2, 1));
    assert_eq!(
        (rig.state().dropped_onsets, rig.state().dropped_events),
        (0, 0)
    );
}

#[test]
fn capture_pressure_preserves_existing_pattern_and_manual_output_sample_for_sample() {
    let rows = vec![
        Hit::new(0, 126, 50, 0).unwrap(),
        Hit::new(20, 127, 70, 0).unwrap(),
    ];
    let mut recording = Rig::new(rows.clone());
    let mut control = Rig::new(rows);
    recording.start(1);
    control.send(Command::TransportPlay);
    for _ in 0..8 {
        for pad in 0..80 {
            let command = Command::PadPress {
                pad: pad % 16,
                velocity: 0.5,
            };
            recording.send(command.clone());
            control.send(command);
        }
        let actual = recording.block(128);
        let expected = control.block(128);
        assert_eq!(actual, expected);
        assert!(
            actual
                .0
                .iter()
                .chain(&actual.1)
                .all(|v| v.is_finite() && v.abs() < 1.0)
        );
    }
    assert!(recording.state().dropped_onsets > 0);
    let hits = recording.stats.pattern_hits(0).unwrap();
    assert!(
        hits[126] > 0 && hits[127] > 0,
        "both scheduled notes really played"
    );
    assert_eq!(hits, control.stats.pattern_hits(0).unwrap());
    recording.drain();
    assert_eq!(recording.block(4096), control.block(4096));
    assert!(
        recording.drain().is_empty(),
        "draining cannot replay omitted capture"
    );
}

#[test]
fn an_empty_pass_and_maximum_pass_identity_finish_without_reopening() {
    let mut rig = Rig::new(vec![]);
    rig.start(u64::MAX);
    rig.block(0);
    assert_eq!(rig.state().phase, Phase::Recording);
    rig.send(Command::PatternRecordEnd { pass: u64::MAX });
    rig.block(0);
    assert_eq!(rig.state().phase, Phase::Finishing);
    assert_eq!(rig.state().sequence, 0);
    assert!(rig.drain().is_empty());
    rig.send(Command::PatternRecordAck {
        pass: u64::MAX,
        sequence: 0,
    });
    rig.send(Command::PatternRecordStart { pass: u64::MAX });
    rig.send(Command::PatternRecordStart { pass: 0 });
    rig.block(0);
    assert_eq!(rig.state().phase, Phase::Idle);
}

#[test]
fn stop_freezes_capture_before_clock_reset_without_changing_manual_tails() {
    for stop in [Command::TransportStop, Command::TransportToggle] {
        let mut rig = Rig::new(vec![]);
        let mut control = Rig::new(vec![]);
        rig.start(1);
        control.send(Command::TransportPlay);
        for target in [&mut rig, &mut control] {
            target.send(Command::PadPress {
                pad: 0,
                velocity: 1.0,
            });
            target.block(128);
            target.block(128);
        }
        let ticks = rig.state().ticks;
        rig.send(stop.clone());
        control.send(stop);
        assert_eq!(rig.block(128), control.block(128));
        assert_eq!(rig.stats.transport(0).unwrap().ticks, 0.0);
        assert_eq!(
            (rig.state().phase, rig.state().ticks),
            (Phase::Finishing, ticks)
        );
        rig.send(Command::TransportPlay);
        control.send(Command::TransportPlay);
        assert_eq!(rig.block(128), control.block(128));
        assert_eq!(rig.state().ticks, ticks, "restart cannot move the old end");
        assert_eq!(rig.drain().len(), 1);
    }
}

#[test]
fn project_epochs_reject_stale_and_unready_capture_commands() {
    let mut rig = Rig::new(vec![]);
    rig.start(1);
    rig.send(Command::PadPress {
        pad: 0,
        velocity: 1.0,
    });
    rig.block(128);
    rig.tagged(0, true, Command::PatternRecordEnd { pass: 1 });
    rig.tagged(
        0,
        true,
        Command::PatternRecordAck {
            pass: 1,
            sequence: 1,
        },
    );
    rig.tagged(0, true, Command::PatternRecordStart { pass: 2 });
    rig.tagged(
        0,
        true,
        Command::PadPress {
            pad: 127,
            velocity: 1.0,
        },
    );
    rig.project.epoch.store(1, Release);
    rig.tagged(1, false, Command::PatternRecordStart { pass: 1 });
    rig.tagged(1, true, Command::PatternRecordStart { pass: 2 });
    rig.tagged(1, false, pattern(vec![]));
    rig.tx
        .push(ProjectCommand {
            epoch: 1,
            ready_at_send: false,
            command: ProjectMessage::Ready,
        })
        .unwrap();
    for _ in 0..300 {
        rig.block(128);
        if rig.project.ready.load(Acquire) == 1 {
            break;
        }
    }
    assert_eq!(rig.project.ready.load(Acquire), 1);
    assert_eq!(rig.stats.recording(0), None);
    assert_eq!(
        rig.state(),
        Snapshot {
            epoch: 1,
            ..Snapshot::default()
        }
    );
    let old = rig.drain();
    assert_eq!(old.len(), 1);
    assert_eq!(
        (old[0].epoch, old[0].pass),
        (0, 1),
        "queued events keep their old identity"
    );
    rig.start(1);
    rig.tagged(1, false, Command::PatternRecordEnd { pass: 1 });
    rig.tagged(0, true, Command::PatternRecordEnd { pass: 1 });
    rig.send(Command::PadPress {
        pad: 127,
        velocity: 0.5,
    });
    rig.block(128);
    assert_eq!(rig.state().phase, Phase::Recording);
    let events = rig.drain();
    assert_eq!(events.len(), 1);
    assert_eq!(
        (events[0].epoch, events[0].pass, events[0].sequence),
        (1, 1, 1)
    );
}

#[test]
fn a_full_command_queue_cannot_create_a_capture_for_a_refused_onset() {
    let mut rig = Rig::new(vec![]);
    rig.start(1);
    rig.block(128);
    for _ in 0..1024 {
        rig.send(Command::SetTempo { bpm: 120.0 });
    }
    assert!(
        rig.tx
            .push(ProjectCommand {
                epoch: 0,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::PadPress {
                    pad: 0,
                    velocity: 1.0
                }),
            })
            .is_err()
    );
    for _ in 0..5 {
        rig.block(128);
    }
    assert_eq!(rig.state().sequence, 0);
    assert!(rig.drain().is_empty());
}

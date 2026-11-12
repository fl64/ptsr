//! Accepted erasure boundaries and bounded retained feedback through the production engine.

use std::sync::Arc;

use engine::capture::{Event, Phase, Recovery};
use engine::pattern::{Grid, Hit, PlaybackRow, RawPattern, RowId};
use engine::{Command, Engine};

struct Rig {
    engine: Engine,
    events: rtrb::Consumer<Event>,
    recovery: Arc<Recovery>,
}

impl Rig {
    fn new(rows: &[[u32; 4]]) -> Self {
        Self::with_rate(rows, 48_000)
    }

    fn with_rate(rows: &[[u32; 4]], sample_rate: u32) -> Self {
        let mut engine = Engine::with_sounds(sample_rate, vec![], vec![vec![0.02; 4800]; 2]);
        let events = engine.enable_capture();
        let recovery = engine.capture_recovery().unwrap();
        let raw = RawPattern::new(
            1,
            Grid::Off,
            0,
            rows.iter()
                .map(|&[tick, pad, velocity, gate]| {
                    Hit::new(tick, pad as usize, velocity as u8, gate).unwrap()
                })
                .collect(),
        )
        .unwrap();
        let identities: Vec<_> = (0..rows.len())
            .map(|slot| PlaybackRow::existing(RowId::new(slot, 1).unwrap()))
            .collect();
        engine.handle(0, Command::SetTempo { bpm: 120.0 });
        engine.handle(
            0,
            Command::SetPattern {
                pattern: Some(Arc::new(raw.compile_recorded(&identities).unwrap())),
            },
        );
        engine.handle(0, Command::TransportPlay);
        engine.handle(0, Command::PatternRecordStart { pass: 1 });
        Self {
            engine,
            events,
            recovery,
        }
    }

    fn publish(&mut self, rows: &[[u32; 4]], grid: Grid, strength: u8, generation: u64) {
        let raw = RawPattern::new(
            1,
            grid,
            strength,
            rows.iter()
                .map(|&[t, p, v, g]| Hit::new(t, p as usize, v as u8, g).unwrap())
                .collect(),
        )
        .unwrap();
        let identities: Vec<_> = (0..rows.len())
            .map(|slot| PlaybackRow::existing(RowId::new(slot, generation).unwrap()))
            .collect();
        let boundary = self.engine.recording();
        self.engine.handle(
            0,
            Command::PublishCapturedPattern {
                pattern: Arc::new(raw.compile_recorded(&identities).unwrap()),
                pass: boundary.pass,
                sequence: boundary.sequence,
            },
        );
    }

    fn frames(&mut self, mut frames: usize) {
        while frames > 0 {
            let n = frames.min(128);
            self.engine
                .process(&mut [0.0; 128][..n], &mut [0.0; 128][..n]);
            frames -= n;
        }
    }
}

#[test]
fn engine_erase_silences_all_collisions_and_retains_half_open_intervals() {
    let mut rig = Rig::new(&[
        [0, 0, 127, 0],
        [960, 0, 70, 0],
        [960, 0, 127, 0],
        [1920, 0, 70, 0],
        [960, 1, 60, 0],
        [6000, 0, 127, 0],
    ]);
    rig.frames(12_000);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    assert!(rig.engine.recording().erasing);
    rig.frames(36_000);
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    rig.frames(1);
    let boundary = rig.engine.recording();
    assert_eq!(boundary.phase, Phase::Recording);
    assert_eq!(boundary.erase_targets, [0, 0]);
    assert_eq!(boundary.erased_rows, 2);
    assert_eq!(boundary.captured_onsets, 0, "targets never record an onset");
    for slot in 0..6 {
        assert_eq!(
            rig.recovery.erased(RowId::new(slot, 1).unwrap(), boundary),
            Some(matches!(slot, 1 | 2)),
            "only traversed collision rows are removed"
        );
    }
    let interval = rig.recovery.erase_interval(0, boundary).unwrap();
    assert!((interval.start - 480.0).abs() < 1e-8);
    assert!((interval.end - 1920.0).abs() < 1e-8);
    assert!(interval.complete);
    assert_eq!(interval.pad, 0);
    assert!(
        rig.events.pop().is_ok(),
        "ordinary release feedback still works"
    );
}

#[test]
fn engine_erase_same_frame_release_excludes_the_onset_and_later_progress() {
    let mut rig = Rig::new(&[[0, 0, 127, 0]]);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    rig.frames(1);
    let boundary = rig.engine.recording();
    assert_eq!(boundary.erased_rows, 0);
    let interval = rig.recovery.erase_interval(0, boundary).unwrap();
    assert_eq!(
        (interval.start, interval.end, interval.complete),
        (0.0, 0.0, true)
    );
    rig.frames(3 * 96_000);
    assert_eq!(
        rig.recovery
            .erase_interval(0, rig.engine.recording())
            .unwrap(),
        interval
    );
    assert_eq!(rig.engine.recording().erased_rows, 0);
}

#[test]
fn engine_erase_seam_and_delayed_publication_do_not_revive_collision_rows() {
    let mut rig = Rig::new(&[
        [0, 0, 80, 0],
        [0, 0, 127, 0],
        [3600, 0, 80, 0],
        [960, 1, 127, 0],
    ]);
    rig.frames(84_000);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    rig.frames(18_000);
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    let frozen = rig.engine.recording();
    assert_eq!(frozen.erased_rows, 3);
    let interval = rig.recovery.erase_interval(0, frozen).unwrap();
    assert!((interval.start - 3360.0).abs() < 1e-7);
    assert!((interval.end - 4080.0).abs() < 1e-7);
    for _ in 0..3 {
        rig.publish(
            &[
                [0, 0, 80, 0],
                [0, 0, 127, 0],
                [3600, 0, 80, 0],
                [960, 1, 127, 0],
            ],
            Grid::Off,
            0,
            1,
        );
        let mut left = vec![0.0; 96_000];
        let mut right = vec![0.0; 96_000];
        rig.engine.process(&mut left, &mut right);
        // Only the other pad at the next quarter note can sound.
        assert!(left[48_000..78_000].iter().all(|s| *s == 0.0));
        assert_eq!(rig.engine.recording().erased_rows, 3);
    }
    assert_eq!(
        rig.recovery
            .erase_interval(0, rig.engine.recording())
            .unwrap(),
        interval
    );
    assert_eq!(
        rig.recovery.erased(RowId::new(2, 1).unwrap(), frozen),
        Some(true)
    );
}

#[test]
fn engine_erase_never_converts_an_ordinary_hold_and_exit_never_synthesizes_sound() {
    let mut rig = Rig::new(&[]);
    rig.engine.handle(
        0,
        Command::SetPadMode {
            pad: 0,
            mode: engine::PadMode::Gate,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 0.5,
        },
    );
    let onset = rig.events.pop().unwrap();
    rig.frames(6000);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    assert_eq!(rig.engine.recording().erase_targets, [0, 0]);
    assert_eq!(rig.engine.recording().captured_onsets, 1);
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    assert_eq!(rig.recovery.completed(onset).unwrap().gate_ticks(), 240);
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 1,
            velocity: 1.0,
        },
    );
    assert_eq!(rig.engine.recording().erase_targets, [2, 0]);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: false,
        },
    );
    assert_eq!(rig.engine.recording().erase_targets, [0, 0]);
    assert_eq!(rig.engine.recording().captured_onsets, 1);
    rig.engine.handle(0, Command::PadRelease { pad: 1 });
    assert_eq!(rig.engine.recording().captured_onsets, 1);
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 1,
            velocity: 1.0,
        },
    );
    assert_eq!(
        rig.engine.recording().captured_onsets,
        2,
        "only a fresh press returns to recording"
    );
}

#[test]
fn engine_erase_one_loop_fallback_uses_musical_time_and_excludes_its_endpoint() {
    let mut rig = Rig::new(&[
        [0, 0, 127, 0],
        [960, 0, 127, 0],
        [1920, 0, 127, 0],
        [2880, 0, 127, 0],
    ]);
    rig.frames(6000);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PatternEraseTarget {
            pass: 1,
            pad: 0,
            one_loop: true,
        },
    );
    rig.frames(18_000);
    rig.engine.handle(0, Command::SetTempo { bpm: 240.0 });
    rig.frames(39_000);
    let boundary = rig.engine.recording();
    assert!((boundary.ticks - 4080.0).abs() < 1e-7);
    assert_eq!(
        boundary.erase_targets,
        [0, 0],
        "expiry boundary: {boundary:?}"
    );
    let interval = rig.recovery.erase_interval(0, boundary).unwrap();
    assert!((interval.start - 240.0).abs() < 1e-7);
    assert!((interval.end - 4080.0).abs() < 1e-7);
    assert!(interval.complete);
    assert_eq!(boundary.erased_rows, 4);
    rig.frames(96_000);
    assert_eq!(rig.engine.recording().erase_intervals, 1);
}

#[test]
fn engine_erase_grid_publication_splits_an_active_target_without_resetting_expiry() {
    let mut rig = Rig::new(&[]);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PatternEraseTarget {
            pass: 1,
            pad: 0,
            one_loop: true,
        },
    );
    rig.frames(6000);
    rig.publish(&[], Grid::Sixteenth, 50, 1);
    rig.frames(6000);
    let boundary = rig.engine.recording();
    assert_eq!(boundary.erase_intervals, 2);
    let first = rig.recovery.erase_interval(0, boundary).unwrap();
    let second = rig.recovery.erase_interval(1, boundary).unwrap();
    assert_eq!(
        (first.grid, first.strength, first.complete),
        (Grid::Off, 0, true)
    );
    assert_eq!(
        (second.grid, second.strength, second.complete),
        (Grid::Sixteenth, 50, false)
    );
    assert!((first.end - second.start).abs() < 1e-9);
    rig.frames(84_000);
    let interval = rig
        .recovery
        .erase_interval(1, rig.engine.recording())
        .unwrap();
    assert!((interval.end - 3840.0).abs() < 1e-7);
    assert!(
        interval.complete,
        "expiry interval: {interval:?}; boundary: {:?}",
        rig.engine.recording()
    );
    assert_eq!(rig.engine.recording().erase_targets, [0, 0]);
}

#[test]
fn engine_erase_pass_ack_waits_for_confirmed_removal_and_consumed_intervals() {
    let mut rig = Rig::new(&[[0, 0, 127, 0]]);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    rig.frames(1);
    rig.engine.handle(0, Command::PatternRecordEnd { pass: 1 });
    while rig.events.pop().is_ok() {}
    let boundary = rig.engine.recording();
    assert!(!boundary.erasing);
    assert_eq!(boundary.erase_targets, [0, 0]);
    let interval = rig.recovery.erase_interval(0, boundary).unwrap();
    rig.engine.handle(
        0,
        Command::PatternRecordAck {
            pass: 1,
            sequence: boundary.sequence,
        },
    );
    assert_eq!(rig.engine.recording().phase, Phase::Finishing);
    rig.publish(&[], Grid::Off, 0, 1);
    rig.engine.handle(
        0,
        Command::PatternRecordAck {
            pass: 1,
            sequence: boundary.sequence,
        },
    );
    assert_eq!(rig.engine.recording().phase, Phase::Finishing);
    rig.engine.handle(
        0,
        Command::PatternEraseConsumed {
            pass: 1,
            slot: 0,
            sequence: interval.sequence,
        },
    );
    rig.engine.handle(
        0,
        Command::PatternRecordAck {
            pass: 1,
            sequence: boundary.sequence,
        },
    );
    assert_eq!(rig.engine.recording().phase, Phase::Idle);
    rig.engine
        .handle(0, Command::PatternRecordStart { pass: 2 });
    assert_eq!(rig.engine.recording().phase, Phase::Recording);
    assert!(!rig.engine.recording().erasing);
    assert_eq!(rig.engine.recording().erase_intervals, 0);
}

/// Scenario "Erasure pressure cannot create an untracked interval".
#[test]
fn engine_erase_full_interval_storage_refuses_silently_and_stale_consumption_cannot_free_a_new_hold()
 {
    let mut rig = Rig::new(&[]);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    for _ in 0..4096 {
        rig.engine.handle(
            0,
            Command::PadPress {
                pad: 0,
                velocity: 1.0,
            },
        );
        rig.engine.handle(0, Command::PadRelease { pad: 0 });
    }
    let before = rig.engine.recording();
    let first = rig.recovery.erase_interval(0, before).unwrap();
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let full = rig.engine.recording();
    assert_eq!(full.erase_intervals, 4096);
    assert_eq!(full.erase_targets, [0, 0]);
    assert_eq!(full.captured_onsets, 0);
    assert_eq!(full.erased_rows, 0);
    assert_eq!(full.dropped_events, before.dropped_events + 1);
    rig.engine.handle(
        0,
        Command::PatternEraseConsumed {
            pass: 2,
            slot: 0,
            sequence: first.sequence,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    assert_eq!(rig.engine.recording().erase_targets, [0, 0]);
    rig.engine.handle(
        0,
        Command::PatternEraseConsumed {
            pass: 1,
            slot: 0,
            sequence: first.sequence,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let next = rig
        .recovery
        .erase_interval(0, rig.engine.recording())
        .unwrap();
    assert!(next.sequence > first.sequence && next.index > first.index);
    rig.engine.handle(
        0,
        Command::PatternEraseConsumed {
            pass: 1,
            slot: 0,
            sequence: first.sequence,
        },
    );
    rig.engine.handle(
        0,
        Command::PatternEraseConsumed {
            pass: 1,
            slot: 0,
            sequence: next.sequence,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 1,
            velocity: 1.0,
        },
    );
    assert_eq!(
        rig.engine.recording().erase_targets,
        [1, 0],
        "an active interval cannot be reclaimed"
    );
}

/// Scenario "Reused recording capacity keeps earlier Gates separate".
#[test]
fn engine_erase_reuses_gate_storage_after_4096_onsets_without_changing_an_earlier_press() {
    let mut rig = Rig::new(&[]);
    rig.engine.handle(
        0,
        Command::SetPadMode {
            pad: 0,
            mode: engine::PadMode::Gate,
        },
    );
    let mut onsets = Vec::new();
    for _ in 0..4096 {
        rig.engine.handle(
            0,
            Command::PadPress {
                pad: 0,
                velocity: 1.0,
            },
        );
        onsets.push(rig.events.pop().unwrap());
    }
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    let old = *onsets.last().unwrap();
    let saved = rig.recovery.completed(old).unwrap();
    for onset in &onsets {
        assert_eq!(rig.recovery.completed(*onset).unwrap().gate_ticks(), 1);
        rig.engine.handle(
            0,
            Command::PatternGateConsumed {
                pass: 1,
                slot: onset.gate.unwrap(),
                sequence: onset.sequence,
            },
        );
    }
    while rig.events.pop().is_ok() {}
    rig.publish(&vec![[0, 0, 127, 1]; 4096], Grid::Off, 0, 2);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    rig.frames(1);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: false,
        },
    );
    let erased = rig.engine.recording();
    assert_eq!(erased.reserved_rows, 0);
    assert_eq!(erased.erased_rows, 4096);
    rig.publish(&[], Grid::Off, 0, 2);
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let newer = rig.events.pop().unwrap();
    assert_eq!(newer.gate, old.gate);
    assert_eq!(rig.engine.recording().captured_onsets, 4097);
    assert_eq!(rig.recovery.completed(old), None);
    rig.engine.handle(
        0,
        Command::PatternRecordRows {
            pass: 1,
            sequence: erased.sequence,
            captured_onsets: erased.captured_onsets,
            erased_rows: erased.erased_rows,
            raw_rows: 0,
        },
    );
    assert_eq!(
        rig.engine.recording().reserved_rows,
        1,
        "late UI confirmation keeps newer reservations"
    );
    rig.engine.handle(
        0,
        Command::PatternGateConsumed {
            pass: 1,
            slot: old.gate.unwrap(),
            sequence: old.sequence,
        },
    );
    rig.frames(6000);
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    assert_eq!(rig.recovery.completed(newer).unwrap().gate_ticks(), 240);
    assert_eq!(saved.gate_ticks(), 1);
}

#[test]
fn engine_erase_frozen_prefix_cannot_acquire_later_progress_or_removals() {
    let mut rig = Rig::new(&[[960, 0, 127, 0]]);
    let before = rig.engine.recording();
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    rig.frames(6000);
    let prefix = rig.engine.recording();
    rig.frames(24_000);
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    let later = rig.engine.recording();
    assert_eq!(
        rig.recovery.erased(RowId::new(0, 1).unwrap(), prefix),
        Some(false)
    );
    assert_eq!(
        rig.recovery.erased(RowId::new(0, 1).unwrap(), later),
        Some(true)
    );
    assert_eq!(rig.recovery.erase_interval(0, before), None);
    let recovered = rig.recovery.erase_interval(0, prefix).unwrap();
    assert_eq!(recovered.end, prefix.ticks);
    assert!(!recovered.complete);
    let completed = rig.recovery.erase_interval(0, later).unwrap();
    assert_eq!(completed.end, later.ticks);
    assert!(completed.complete);
}

#[test]
fn engine_erase_fresh_row_generation_does_not_inherit_a_removed_row() {
    let mut rig = Rig::new(&[[0, 0, 127, 0]]);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    rig.frames(1);
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: false,
        },
    );
    while rig.events.pop().is_ok() {}
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let onset = rig.events.pop().unwrap();
    let pattern = RawPattern::new(1, Grid::Off, 0, vec![Hit::new(0, 0, 127, 0).unwrap()])
        .unwrap()
        .compile_recorded(&[PlaybackRow::recorded(
            RowId::new(0, 2).unwrap(),
            onset.origin(3840).unwrap(),
            false,
        )])
        .unwrap();
    rig.engine.handle(
        0,
        Command::PublishCapturedPattern {
            pattern: Arc::new(pattern),
            pass: 1,
            sequence: rig.engine.recording().sequence,
        },
    );
    assert_eq!(
        rig.recovery
            .erased(RowId::new(0, 2).unwrap(), rig.engine.recording()),
        Some(false)
    );
    rig.frames(95_999);
    let mut left = [0.0; 128];
    let mut right = [0.0; 128];
    rig.engine.process(&mut left, &mut right);
    assert!(
        left.iter().any(|s| *s > 0.0),
        "a fresh generation plays at its next eligible loop"
    );
}

#[test]
fn engine_erase_terminal_pressure_never_overwrites_an_unconsumed_gate() {
    let mut rig = Rig::new(&[]);
    rig.engine.handle(
        0,
        Command::SetPadMode {
            pad: 0,
            mode: engine::PadMode::Gate,
        },
    );
    let mut last = None;
    for _ in 0..4096 {
        rig.engine.handle(
            0,
            Command::PadPress {
                pad: 0,
                velocity: 1.0,
            },
        );
        last = Some(rig.events.pop().unwrap());
    }
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    while rig.events.pop().is_ok() {}
    let old = last.unwrap();
    let saved = rig.recovery.completed(old).unwrap();
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    rig.frames(1);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: false,
        },
    );
    let boundary = rig.engine.recording();
    // The UI can erase captured rows before their first sequencer publication.
    rig.engine.handle(
        0,
        Command::PatternRecordRows {
            pass: 1,
            sequence: boundary.sequence,
            captured_onsets: boundary.captured_onsets,
            erased_rows: 0,
            raw_rows: 0,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    assert_eq!(rig.engine.recording().captured_onsets, 4096);
    assert_eq!(rig.engine.recording().dropped_onsets, 1);
    assert_eq!(rig.recovery.completed(old), Some(saved));
    rig.engine.handle(
        0,
        Command::PatternGateConsumed {
            pass: 1,
            slot: old.gate.unwrap(),
            sequence: old.sequence,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    assert_eq!(rig.engine.recording().captured_onsets, 4097);
    assert_eq!(rig.recovery.completed(old), None);
}

#[test]
fn engine_erase_reservation_confirmation_preserves_later_onsets_and_removals_and_rejects_stale_data()
 {
    let mut rig = Rig::new(&[[0, 0, 127, 0], [960, 1, 127, 0]]);
    let before = rig.engine.recording();
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 1,
            velocity: 1.0,
        },
    );
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    rig.frames(1);
    let now = rig.engine.recording();
    assert_eq!(now.reserved_rows, 2);
    rig.engine.handle(
        0,
        Command::PatternRecordRows {
            pass: 1,
            sequence: before.sequence,
            captured_onsets: 0,
            erased_rows: 0,
            raw_rows: 2,
        },
    );
    assert_eq!(rig.engine.recording().reserved_rows, 2);
    rig.engine.handle(
        0,
        Command::PatternRecordRows {
            pass: 1,
            sequence: now.sequence,
            captured_onsets: now.captured_onsets,
            erased_rows: now.erased_rows,
            raw_rows: 2,
        },
    );
    for (pass, sequence, captured_onsets, erased_rows, raw_rows) in [
        (1, before.sequence, 0, 0, 0),
        (2, now.sequence, now.captured_onsets, now.erased_rows, 0),
        (1, now.sequence + 1, now.captured_onsets, now.erased_rows, 0),
        (1, now.sequence, u32::MAX, now.erased_rows, 0),
        (1, now.sequence, now.captured_onsets, u64::MAX, 0),
        (1, now.sequence, now.captured_onsets, now.erased_rows, 4097),
    ] {
        rig.engine.handle(
            0,
            Command::PatternRecordRows {
                pass,
                sequence,
                captured_onsets,
                erased_rows,
                raw_rows,
            },
        );
        assert_eq!(rig.engine.recording().reserved_rows, 2);
    }
}

#[test]
fn engine_erase_only_accepts_matching_recording_phase_and_stop_clears_targets() {
    let mut rig = Rig::new(&[]);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 2,
            enabled: true,
        },
    );
    assert!(!rig.engine.recording().erasing);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let first = rig.engine.recording();
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    assert_eq!(
        rig.engine.recording().erase_intervals,
        first.erase_intervals
    );
    rig.frames(6000);
    rig.engine.handle(0, Command::TransportStop);
    let stopped = rig.engine.recording();
    assert_eq!(stopped.phase, Phase::Finishing);
    assert!(!stopped.erasing);
    assert_eq!(stopped.erase_targets, [0, 0]);
    assert!((stopped.ticks - 240.0).abs() < 1e-8);
    assert_eq!(rig.engine.transport().ticks, 0.0);
    rig.engine.handle(0, Command::SetTempo { bpm: 300.0 });
    assert_eq!(
        rig.engine.recording(),
        stopped,
        "terminal recovery keeps its accepted frame duration"
    );
    let interval = rig.recovery.erase_interval(0, stopped).unwrap();
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    assert!(!rig.engine.recording().erasing);
    rig.engine.handle(
        0,
        Command::PatternEraseConsumed {
            pass: 1,
            slot: interval.slot,
            sequence: interval.sequence,
        },
    );
    rig.engine.handle(
        0,
        Command::PatternRecordAck {
            pass: 1,
            sequence: stopped.sequence,
        },
    );
    rig.engine
        .handle(0, Command::PatternRecordStart { pass: 2 });
    assert_eq!(rig.engine.recording().phase, Phase::CountIn);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 2,
            enabled: true,
        },
    );
    assert!(!rig.engine.recording().erasing);
    rig.engine.handle(
        0,
        Command::PatternEraseTarget {
            pass: 1,
            pad: 0,
            one_loop: true,
        },
    );
    assert_eq!(rig.engine.recording().erase_targets, [0, 0]);
}

#[test]
fn engine_erase_expiry_follows_nearest_output_frames_across_formats_tempos_and_buffers() {
    for sample_rate in [44_100, 48_000, 96_000] {
        for bpm in [40.0, 120.0, 172.0, 300.0] {
            for block in [40, 128, 1024] {
                let mut rig = Rig::with_rate(&[], sample_rate);
                rig.engine.handle(0, Command::SetTempo { bpm });
                rig.engine.handle(
                    0,
                    Command::PatternErase {
                        pass: 1,
                        enabled: true,
                    },
                );
                rig.engine.handle(
                    0,
                    Command::PatternEraseTarget {
                        pass: 1,
                        pad: 127,
                        one_loop: true,
                    },
                );
                let duration = (f64::from(sample_rate) * 240.0 / bpm - 0.5 - 1e-9).ceil() as usize;
                let mut frames = duration - 1;
                while frames > 0 {
                    let n = frames.min(block);
                    rig.engine
                        .process(&mut [0.0; 1024][..n], &mut [0.0; 1024][..n]);
                    frames -= n;
                }
                assert_eq!(
                    rig.engine.recording().erase_targets,
                    [0, 1 << 63],
                    "{sample_rate}/{bpm}/{block}"
                );
                rig.frames(1);
                let boundary = rig.engine.recording();
                assert_eq!(
                    boundary.erase_targets,
                    [0, 0],
                    "{sample_rate}/{bpm}/{block}: {boundary:?}"
                );
                let interval = rig.recovery.erase_interval(0, boundary).unwrap();
                assert!(interval.complete);
                assert!((interval.end - 3840.0).abs() <= boundary.frame_ticks * 0.5 + 1e-7);
            }
        }
    }
}

#[test]
fn engine_erase_project_tagging_rejects_unready_and_stale_commands_without_leaking_holds() {
    use engine::rt::{ProjectCommand, ProjectMessage, ProjectState, Realtime, Stats};
    use std::sync::atomic::Ordering::{Acquire, Release};
    let rig = Rig::new(&[[960, 0, 127, 0]]);
    let (mut tx, rx) = rtrb::RingBuffer::new(32);
    let stats = Arc::new(Stats::default());
    let project = Arc::new(ProjectState::default());
    let mut rt =
        Realtime::with_project(rig.engine, vec![rx], stats.clone(), 48_000, project.clone());
    tx.push(ProjectCommand {
        epoch: 0,
        ready_at_send: false,
        command: ProjectMessage::Command(Command::PatternErase {
            pass: 1,
            enabled: true,
        }),
    })
    .unwrap();
    rt.process(&mut [0.0; 128], &mut [0.0; 128]);
    assert!(!stats.recording(0).unwrap().erasing);
    for command in [
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
        Command::PatternEraseTarget {
            pass: 1,
            pad: 0,
            one_loop: true,
        },
    ] {
        tx.push(ProjectCommand {
            epoch: 0,
            ready_at_send: true,
            command: ProjectMessage::Command(command),
        })
        .unwrap();
    }
    rt.process(&mut [0.0; 128], &mut [0.0; 128]);
    assert_eq!(stats.recording(0).unwrap().erase_targets, [1, 0]);
    project.epoch.store(1, Release);
    for command in [
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
        Command::PatternEraseTarget {
            pass: 1,
            pad: 127,
            one_loop: true,
        },
        Command::PatternEraseConsumed {
            pass: 1,
            slot: 0,
            sequence: 1,
        },
        Command::PatternGateConsumed {
            pass: 1,
            slot: 0,
            sequence: 1,
        },
        Command::PatternRecordRows {
            pass: 1,
            sequence: 1,
            captured_onsets: 0,
            erased_rows: 0,
            raw_rows: 0,
        },
    ] {
        tx.push(ProjectCommand {
            epoch: 0,
            ready_at_send: true,
            command: ProjectMessage::Command(command),
        })
        .unwrap();
    }
    rt.process(&mut [0.0; 128], &mut [0.0; 128]);
    let boundary = stats.recording(project.epoch.load(Acquire)).unwrap();
    assert_eq!(boundary.phase, Phase::Idle);
    assert!(!boundary.erasing);
    assert_eq!(boundary.erase_targets, [0, 0]);
    assert_eq!(rig.recovery.erase_interval(0, boundary), None);
}

#[test]
fn engine_erase_invalid_pad_or_velocity_cannot_create_a_target() {
    let mut rig = Rig::new(&[[0, 0, 127, 0]]);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    for velocity in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        rig.engine.handle(0, Command::PadPress { pad: 0, velocity });
    }
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 128,
            velocity: 1.0,
        },
    );
    let boundary = rig.engine.recording();
    assert_eq!(boundary.erase_intervals, 0);
    assert_eq!(boundary.erase_targets, [0, 0]);
    rig.frames(1);
    assert_eq!(rig.engine.recording().erased_rows, 0);
}

#[test]
fn engine_erase_one_loop_keeps_the_last_matching_onset_when_expiry_shares_its_output_frame() {
    let rows: Vec<_> = (0..3840).map(|tick| [tick, 0, 1, 0]).collect();
    let mut rig = Rig::with_rate(&rows, 44_100);
    rig.engine.handle(0, Command::SetTempo { bpm: 172.0 });
    rig.frames(401);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PatternEraseTarget {
            pass: 1,
            pad: 0,
            one_loop: true,
        },
    );
    rig.frames(61_536);
    let boundary = rig.engine.recording();
    assert_eq!(boundary.erase_targets, [0, 0]);
    assert_eq!(
        boundary.erased_rows, 3840,
        "every raw tick occurs once within a full musical loop"
    );
    assert_eq!(
        rig.recovery.erased(RowId::new(25, 1).unwrap(), boundary),
        Some(true)
    );
}

#[test]
fn engine_erase_release_keeps_an_earlier_compiled_onset_that_shares_its_output_frame() {
    let mut rig = Rig::with_rate(&[[25, 0, 127, 0]], 44_100);
    rig.engine.handle(0, Command::SetTempo { bpm: 172.0 });
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    rig.frames(401);
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    rig.frames(1);
    let boundary = rig.engine.recording();
    assert_eq!(
        boundary.erased_rows, 1,
        "tick 25 precedes the accepted release at tick 25.023854875"
    );
}

#[test]
fn engine_erase_automatic_expiry_retains_the_full_musical_interval_before_ui_consumption() {
    let mut rig = Rig::with_rate(&[], 96_000);
    rig.engine.handle(0, Command::SetTempo { bpm: 172.0 });
    rig.frames(401);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PatternEraseTarget {
            pass: 1,
            pad: 0,
            one_loop: true,
        },
    );
    rig.frames(133_954);
    rig.frames(1);
    let boundary = rig.engine.recording();
    let interval = rig.recovery.erase_interval(0, boundary).unwrap();
    assert!(interval.complete);
    assert!(
        (interval.end - interval.start - 3840.0).abs() < 1e-8,
        "{interval:?}"
    );
}

#[test]
fn engine_erase_disjoint_offset_holds_do_not_erase_the_gap_or_release_endpoints() {
    let mut rig = Rig::with_rate(
        &[
            [10, 0, 127, 0],
            [20, 0, 127, 0],
            [30, 0, 127, 0],
            [40, 0, 127, 0],
        ],
        960,
    );
    rig.engine.handle(0, Command::SetTempo { bpm: 300.0 });
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    for (press, release) in [(1, 3), (5, 7)] {
        rig.engine.handle(
            press,
            Command::PadPress {
                pad: 0,
                velocity: 1.0,
            },
        );
        rig.engine.handle(release, Command::PadRelease { pad: 0 });
    }
    rig.frames(10);
    let boundary = rig.engine.recording();
    assert_eq!(boundary.erased_rows, 2);
    for slot in 0..4 {
        assert_eq!(
            rig.recovery.erased(RowId::new(slot, 1).unwrap(), boundary),
            Some(slot == 0 || slot == 2)
        );
    }
}

#[test]
fn engine_erase_end_and_stop_freeze_the_last_matching_row_before_the_terminal_sequence() {
    for stop in [false, true] {
        let mut rig = Rig::with_rate(&[[25, 0, 127, 0]], 44_100);
        rig.engine.handle(0, Command::SetTempo { bpm: 172.0 });
        rig.engine.handle(
            0,
            Command::PatternErase {
                pass: 1,
                enabled: true,
            },
        );
        rig.engine.handle(
            0,
            Command::PadPress {
                pad: 0,
                velocity: 1.0,
            },
        );
        rig.frames(401);
        rig.engine.handle(
            0,
            if stop {
                Command::TransportStop
            } else {
                Command::PatternRecordEnd { pass: 1 }
            },
        );
        let terminal = rig.engine.recording();
        assert_eq!(terminal.erased_rows, 1);
        assert_eq!(terminal.phase, Phase::Finishing);
        rig.frames(128);
        assert_eq!(
            rig.engine.recording(),
            terminal,
            "later scheduling cannot extend terminal capture"
        );
    }
}

#[test]
fn engine_erase_a_new_capture_cannot_inherit_an_earlier_closed_window() {
    let mut rig = Rig::with_rate(&[[25, 0, 127, 0]], 44_100);
    rig.engine.handle(0, Command::SetTempo { bpm: 172.0 });
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    rig.frames(401);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: false,
        },
    );
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    while rig.events.pop().is_ok() {}
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let onset = rig.events.pop().unwrap();
    let raw = RawPattern::new(1, Grid::Off, 0, vec![Hit::new(25, 0, 127, 0).unwrap(); 2]).unwrap();
    let pattern = raw
        .compile_recorded(&[
            PlaybackRow::existing(RowId::new(0, 1).unwrap()),
            PlaybackRow::recorded(
                RowId::new(1, 1).unwrap(),
                onset.origin(3840).unwrap(),
                false,
            ),
        ])
        .unwrap();
    let boundary = rig.engine.recording();
    rig.engine.handle(
        0,
        Command::PublishCapturedPattern {
            pattern: Arc::new(pattern),
            pass: 1,
            sequence: boundary.sequence,
        },
    );
    rig.frames(1);
    let boundary = rig.engine.recording();
    assert_eq!(
        rig.recovery.erased(RowId::new(0, 1).unwrap(), boundary),
        Some(true)
    );
    assert_eq!(
        rig.recovery.erased(RowId::new(1, 1).unwrap(), boundary),
        Some(false)
    );
    assert_eq!(boundary.captured_onsets, 1);
}

#[test]
fn engine_erase_grid_changes_do_not_apply_an_old_window_to_new_compiled_positions() {
    let mut rig = Rig::new(&[[200, 0, 70, 0], [400, 0, 127, 0]]);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    rig.frames(7200);
    rig.publish(&[[200, 0, 70, 0], [400, 0, 127, 0]], Grid::Quarter, 100, 1);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: false,
        },
    );
    let boundary = rig.engine.recording();
    assert_eq!(
        rig.recovery.erased(RowId::new(0, 1).unwrap(), boundary),
        Some(true)
    );
    assert_eq!(
        rig.recovery.erased(RowId::new(1, 1).unwrap(), boundary),
        Some(false)
    );
    rig.frames(88_800);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    rig.engine.process(&mut left, &mut right);
    assert!(
        left.iter().any(|sample| *sample > 0.0),
        "the untraversed raw row survives the grid change"
    );
}

#[test]
fn engine_erase_rejects_a_stale_publication_after_its_closed_window_has_expired() {
    let mut rig = Rig::new(&[[0, 1, 127, 0]]);
    rig.frames(6000);
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let onset = rig.events.pop().unwrap();
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    let stale = rig.engine.recording();
    let raw = RawPattern::new(
        1,
        Grid::Off,
        0,
        vec![
            Hit::new(0, 1, 127, 0).unwrap(),
            Hit::new(240, 0, 127, 0).unwrap(),
        ],
    )
    .unwrap();
    let delayed = Arc::new(
        raw.compile_recorded(&[
            PlaybackRow::existing(RowId::new(0, 1).unwrap()),
            PlaybackRow::recorded(
                RowId::new(1, 1).unwrap(),
                onset.origin(3840).unwrap(),
                false,
            ),
        ])
        .unwrap(),
    );
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PatternEraseTarget {
            pass: 1,
            pad: 0,
            one_loop: true,
        },
    );
    rig.frames(96_000);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: false,
        },
    );
    rig.frames(1);
    let boundary = rig.engine.recording();
    assert!(boundary.erasure_sequence > stale.sequence);
    for command in [
        Command::PublishCapturedPattern {
            pattern: delayed.clone(),
            pass: 1,
            sequence: stale.sequence,
        },
        Command::PublishCapturedPattern {
            pattern: delayed.clone(),
            pass: 2,
            sequence: boundary.sequence,
        },
        Command::PublishCapturedPattern {
            pattern: delayed.clone(),
            pass: 1,
            sequence: boundary.sequence + 1,
        },
        Command::PublishRecordedPattern { pattern: delayed },
    ] {
        rig.engine.handle(0, command);
    }
    rig.frames(95_999);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    rig.engine.process(&mut left, &mut right);
    assert!(
        left.iter().chain(&right).all(|sample| *sample == 0.0),
        "a stale UI row cannot return on a later loop"
    );
}

#[test]
fn engine_erase_a_publication_applies_live_progress_that_followed_its_snapshot() {
    let mut rig = Rig::new(&[[0, 1, 127, 0]]);
    rig.publish(&[[0, 1, 127, 0]], Grid::Quarter, 100, 1);
    rig.frames(20_000);
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let onset = rig.events.pop().unwrap();
    rig.engine.handle(0, Command::PadRelease { pad: 0 });
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let earlier = rig.engine.recording();
    rig.frames(5000);
    let raw = RawPattern::new(
        1,
        Grid::Quarter,
        100,
        vec![
            Hit::new(0, 1, 127, 0).unwrap(),
            Hit::new(800, 0, 127, 0).unwrap(),
        ],
    )
    .unwrap();
    let delayed = raw
        .compile_recorded(&[
            PlaybackRow::existing(RowId::new(0, 1).unwrap()),
            PlaybackRow::recorded(
                RowId::new(1, 1).unwrap(),
                onset.origin(3840).unwrap(),
                false,
            ),
        ])
        .unwrap();
    rig.engine.handle(
        0,
        Command::PublishCapturedPattern {
            pattern: Arc::new(delayed),
            pass: 1,
            sequence: earlier.sequence,
        },
    );
    let later = rig.engine.recording();
    assert_eq!(later.erased_rows, 1);
    assert_eq!(
        rig.recovery.erased(RowId::new(1, 1).unwrap(), later),
        Some(true)
    );
    assert!(later.erasure_sequence > earlier.erasure_sequence);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: false,
        },
    );
    rig.frames(95_000);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    rig.engine.process(&mut left, &mut right);
    assert!(left.iter().chain(&right).all(|sample| *sample == 0.0));
}

#[test]
fn engine_erase_end_at_an_early_rounded_expiry_freezes_its_last_row() {
    let mut rig = Rig::with_rate(&[[1, 0, 127, 0]], 96_000);
    rig.engine.handle(0, Command::SetTempo { bpm: 172.0 });
    rig.frames(35);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PatternEraseTarget {
            pass: 1,
            pad: 0,
            one_loop: true,
        },
    );
    rig.frames(133_953);
    rig.engine.handle(0, Command::PatternRecordEnd { pass: 1 });
    let terminal = rig.engine.recording();
    assert_eq!(terminal.erased_rows, 1);
    rig.frames(1);
    assert_eq!(rig.engine.recording(), terminal);
}

#[test]
fn engine_erase_a_capture_after_early_expiry_survives_the_same_output_frame() {
    let mut rig = Rig::with_rate(&[], 96_000);
    rig.engine.handle(0, Command::SetTempo { bpm: 172.0 });
    rig.frames(35);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    rig.engine.handle(
        0,
        Command::PatternEraseTarget {
            pass: 1,
            pad: 0,
            one_loop: true,
        },
    );
    rig.frames(133_953);
    let expiry = rig.engine.recording();
    let interval = rig.recovery.erase_interval(0, expiry).unwrap();
    assert!(interval.complete);
    rig.engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: false,
        },
    );
    rig.engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let onset = rig.events.pop().unwrap();
    assert!(onset.sequence > interval.closed_sequence && onset.ticks < interval.end);
    let pattern = RawPattern::new(1, Grid::Off, 0, vec![Hit::new(1, 0, 127, 0).unwrap()])
        .unwrap()
        .compile_recorded(&[PlaybackRow::recorded(
            RowId::new(0, 1).unwrap(),
            onset.origin(3840).unwrap(),
            false,
        )])
        .unwrap();
    rig.engine.handle(
        0,
        Command::PublishCapturedPattern {
            pattern: Arc::new(pattern),
            pass: 1,
            sequence: rig.engine.recording().sequence,
        },
    );
    rig.frames(1);
    assert_eq!(
        rig.recovery
            .erased(RowId::new(0, 1).unwrap(), rig.engine.recording()),
        Some(false)
    );
}

fn erasure_render_engine(capture: bool) -> (Engine, Option<rtrb::Consumer<Event>>) {
    let mut engine = Engine::with_sounds(48_000, vec![], vec![]);
    for (pad, frequencies, levels) in [
        (0, [60.0_f32, 80.0], [0.04_f32, 0.03]),
        (1, [900.0, 1100.0], [0.01, 0.015]),
    ] {
        let channel = |frequency: f32, level: f32| -> Box<[f32]> {
            (0..4800)
                .map(|frame| {
                    let envelope = (std::f32::consts::PI * frame as f32 / 4799.0).sin().powi(2);
                    level
                        * envelope
                        * (std::f32::consts::TAU * frequency * frame as f32 / 48_000.0).sin()
                })
                .collect()
        };
        engine.handle(
            0,
            Command::SetPadSound {
                pad,
                sound: Some(Arc::new(engine::Sample {
                    left: channel(frequencies[0], levels[0]),
                    right: Some(channel(frequencies[1], levels[1])),
                })),
            },
        );
    }
    let mut hits: Vec<_> = (0..4)
        .filter(|beat| capture || *beat != 1)
        .map(|beat| Hit::new(beat * 960, 0, 127, 0).unwrap())
        .collect();
    hits.extend((0..8).map(|beat| Hit::new(beat * 480, 1, 127, 0).unwrap()));
    let raw = RawPattern::new(1, Grid::Off, 0, hits).unwrap();
    let pattern = if capture {
        let rows: Vec<_> = (0..raw.hits().len())
            .map(|slot| PlaybackRow::existing(RowId::new(slot, 1).unwrap()))
            .collect();
        raw.compile_recorded(&rows).unwrap()
    } else {
        raw.compile()
    };
    let events = capture.then(|| engine.enable_capture());
    engine.handle(0, Command::SetTempo { bpm: 120.0 });
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(Arc::new(pattern)),
        },
    );
    engine.handle(0, Command::TransportPlay);
    if capture {
        engine.handle(0, Command::PatternRecordStart { pass: 1 });
    }
    (engine, events)
}

fn erasure_render(
    engine: &mut Engine,
    commands: &[(usize, Command)],
    block: usize,
) -> (Vec<f32>, Vec<f32>) {
    let frames = 6 * 48_000;
    let (mut left, mut right) = (vec![0.0; frames], vec![0.0; frames]);
    let (mut frame, mut next) = (0, 0);
    while frame < frames {
        while commands.get(next).is_some_and(|event| event.0 == frame) {
            engine.handle(0, commands[next].1.clone());
            next += 1;
        }
        let end = (frame + block)
            .min(frames)
            .min(commands.get(next).map_or(frames, |event| event.0));
        assert!(end > frame);
        engine.process(&mut left[frame..end], &mut right[frame..end]);
        frame = end;
    }
    (left, right)
}

#[test]
fn engine_erase_render_keeps_other_pads_and_excludes_the_release_boundary_without_late_repeats() {
    let (mut reference_engine, _) = erasure_render_engine(false);
    let reference = erasure_render(&mut reference_engine, &[], 128);
    let commands = [
        (
            12_000,
            Command::PatternErase {
                pass: 1,
                enabled: true,
            },
        ),
        (
            12_000,
            Command::PadPress {
                pad: 0,
                velocity: 1.0,
            },
        ),
        (48_000, Command::PadRelease { pad: 0 }),
        (
            48_000,
            Command::PatternErase {
                pass: 1,
                enabled: false,
            },
        ),
    ];
    for block in [1, 128, 1024] {
        let (mut engine, _events) = erasure_render_engine(true);
        let actual = erasure_render(&mut engine, &commands, block);
        for (channel, expected) in [(&actual.0, &reference.0), (&actual.1, &reference.1)] {
            assert!(engine::checks::first_non_finite(channel).is_none());
            let peak = engine::checks::peak(channel);
            assert!((0.01..0.1).contains(&peak), "peak={peak}");
            assert!(engine::checks::max_step(channel) < 0.004);
            for (frame, (&sample, &wanted)) in channel.iter().zip(expected.iter()).enumerate() {
                assert!(
                    (sample - wanted).abs() <= 1e-6,
                    "block={block}, frame={frame}: {sample} != {wanted}"
                );
            }
            let onsets = engine::checks::onsets(channel, 1e-12, 6000);
            assert_eq!(onsets.len(), 24);
            for (beat, onset) in onsets.iter().enumerate() {
                assert!(onset.abs_diff(beat * 12_000 + 1) <= 1, "{onsets:?}");
            }
            assert!(
                channel[channel.len() - 4800..]
                    .iter()
                    .all(|sample| *sample == 0.0)
            );
        }
        assert_eq!(engine.recording().erased_rows, 1);
        assert_eq!(engine.recording().captured_onsets, 0);
        if block == 128 {
            engine::offline::write_wav(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../renders/97-pattern-erase.wav"),
                48_000,
                &actual.0,
                &actual.1,
            )
            .unwrap();
        }
    }
}

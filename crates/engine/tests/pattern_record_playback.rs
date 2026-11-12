//! Recorded-row eligibility through production queues, compared with manual sound.

use std::sync::{
    Arc,
    atomic::Ordering::{Acquire, Release},
};

use engine::capture::{Event, Kind, Recovery};
use engine::pattern::{Error, Grid, Hit, MAX_HITS, PlaybackRow, RawPattern, RowId, TICKS_PER_BAR};
use engine::rt::{ProjectCommand, ProjectMessage, ProjectState, Realtime, Stats};
use engine::{Command, Engine, PadMode, Retired, Sample, checks};

const SR: u32 = 48_000;
const LAP: usize = 96_000;
type Wave = (Vec<f32>, Vec<f32>);

fn id(slot: usize, generation: u64) -> RowId {
    RowId::new(slot, generation).unwrap()
}

fn hit(tick: u32, velocity: u8, gate: u32) -> Hit {
    Hit::new(tick, 0, velocity, gate).unwrap()
}

fn press(velocity: u8) -> Command {
    Command::PadPress {
        pad: 0,
        velocity: f32::from(velocity) / 127.0,
    }
}

fn sample(tone: bool) -> Arc<Sample> {
    if tone {
        let channel = |freq: f32| {
            (0..4800)
                .map(|i| {
                    let attack = ((i + 1) as f32 / 96.0).min(1.0);
                    let decay = (1.0 - i as f32 / 4800.0).powi(2);
                    0.05 * attack
                        * decay
                        * (std::f32::consts::TAU * freq * i as f32 / SR as f32).cos()
                })
                .collect::<Vec<_>>()
                .into_boxed_slice()
        };
        Arc::new(Sample {
            left: channel(110.0),
            right: Some(channel(165.0)),
        })
    } else {
        Arc::new(Sample {
            left: vec![0.04].into_boxed_slice(),
            right: Some(vec![0.02].into_boxed_slice()),
        })
    }
}

struct Rig {
    tx: rtrb::Producer<ProjectCommand>,
    events: rtrb::Consumer<Event>,
    recovery: Arc<Recovery>,
    garbage: rtrb::Consumer<Retired>,
    rt: Realtime,
    stats: Arc<Stats>,
    project: Arc<ProjectState>,
    wave: Wave,
}

impl Rig {
    fn new(rows: &[Hit], tone: bool) -> Self {
        let mut engine = Engine::with_sounds(SR, vec![], vec![]);
        engine.handle(
            0,
            Command::SetPadSound {
                pad: 0,
                sound: Some(sample(tone)),
            },
        );
        let descriptors: Vec<_> = (0..rows.len())
            .map(|i| PlaybackRow::existing(id(i, 1)))
            .collect();
        engine.handle(
            0,
            Command::SetPattern {
                pattern: Some(Arc::new(
                    RawPattern::new(1, Grid::Sixteenth, 100, rows.to_vec())
                        .unwrap()
                        .compile_recorded(&descriptors)
                        .unwrap(),
                )),
            },
        );
        let events = engine.enable_capture();
        let recovery = engine.capture_recovery().unwrap();
        let (garbage_tx, garbage) = rtrb::RingBuffer::new(1024);
        engine.set_garbage(garbage_tx);
        let (tx, rx) = rtrb::RingBuffer::new(1024);
        let stats = Arc::new(Stats::default());
        let project = Arc::new(ProjectState::default());
        let rt = Realtime::with_project(engine, vec![rx], stats.clone(), SR, project.clone());
        let mut rig = Self {
            tx,
            events,
            recovery,
            garbage,
            rt,
            stats,
            project,
            wave: (vec![], vec![]),
        };
        rig.send(Command::TransportPlay);
        rig.block(0);
        rig
    }

    fn send(&mut self, command: Command) {
        self.tx
            .push(ProjectCommand {
                epoch: self.project.epoch.load(Acquire),
                ready_at_send: true,
                command: ProjectMessage::Command(command),
            })
            .unwrap();
    }

    fn block(&mut self, n: usize) {
        while self.garbage.pop().is_ok() {}
        let (mut left, mut right) = (vec![0.0; n], vec![0.0; n]);
        self.rt.process(&mut left, &mut right);
        assert!(
            left.iter()
                .chain(&right)
                .all(|v| v.is_finite() && v.abs() < 1.0)
        );
        self.wave.0.extend(left);
        self.wave.1.extend(right);
    }

    fn to(&mut self, frame: usize, sizes: &[usize]) {
        assert!(frame >= self.wave.0.len());
        let mut index = 0;
        while self.wave.0.len() < frame {
            self.block(sizes[index % sizes.len()].min(frame - self.wave.0.len()));
            index += 1;
        }
    }

    fn onset(&mut self) -> Event {
        self.block(0);
        let event = self.events.pop().unwrap();
        assert!(matches!(event.kind, Kind::Onset { .. }));
        event
    }

    fn publish(&mut self, grid: Grid, strength: u8, rows: &[Hit], descriptors: &[PlaybackRow]) {
        let raw = RawPattern::new(1, grid, strength, rows.to_vec()).unwrap();
        let before = raw.clone();
        let pattern = raw.compile_recorded(descriptors).unwrap();
        assert_eq!(raw, before, "compilation must not change the raw take");
        self.send(Command::SetPattern {
            pattern: Some(Arc::new(pattern)),
        });
        self.block(0);
    }
}

fn pair_to(actual: &mut Rig, reference: &mut Rig, frame: usize, sizes: &[usize]) {
    actual.to(frame, sizes);
    reference.to(frame, sizes);
}

fn both(actual: &mut Rig, reference: &mut Rig, command: Command) {
    actual.send(command.clone());
    reference.send(command);
}

fn compare(actual: &Rig, reference: &Rig) {
    for (a, b) in [
        (&actual.wave.0, &reference.wave.0),
        (&actual.wave.1, &reference.wave.1),
    ] {
        assert_eq!(a.len(), b.len());
        for (frame, (a, b)) in a.iter().zip(b).enumerate() {
            assert!((a - b).abs() <= 0.000001, "frame {frame}: {a} != {b}");
        }
    }
    assert_eq!(
        actual
            .stats
            .transport(actual.project.epoch.load(Acquire))
            .unwrap()
            .dropped_hits,
        0
    );
}

fn expect_onsets(rig: &Rig, expected: &[usize]) {
    for channel in [&rig.wave.0, &rig.wave.1] {
        let got = checks::onsets(channel, 0.0000001, 0);
        assert_eq!(got.len(), expected.len(), "{got:?} != {expected:?}");
        for (got, expected) in got.iter().zip(expected) {
            assert!(got.abs_diff(*expected) <= 1, "{got} != {expected}");
        }
    }
}

fn fresh(
    actual: &mut Rig,
    reference: &mut Rig,
    frame: usize,
    velocity: u8,
    sizes: &[usize],
) -> (Event, Hit) {
    actual.send(Command::PatternRecordStart { pass: 1 });
    pair_to(actual, reference, frame, sizes);
    both(actual, reference, press(velocity));
    let onset = actual.onset();
    reference.block(0);
    let row = actual.recovery.completed(onset).unwrap();
    (onset, row)
}

fn forward(tone: bool, sizes: &[usize]) -> Rig {
    let mut actual = Rig::new(&[], tone);
    let mut reference = Rig::new(&[], tone);
    let (onset, row) = fresh(&mut actual, &mut reference, 5040, 100, sizes);
    // the audio thread places the hit with the 1/16 grid of its capture (98c)
    assert_eq!(row.tick(), 240);
    actual.publish(
        Grid::Sixteenth,
        100,
        &[row],
        &[PlaybackRow::recorded(
            id(0, 1),
            onset.origin(TICKS_PER_BAR).unwrap(),
            false,
        )],
    );
    for frame in [LAP + 6000, 2 * LAP + 6000] {
        pair_to(&mut actual, &mut reference, frame, sizes);
        reference.send(press(100));
    }
    pair_to(&mut actual, &mut reference, 210_000, sizes);
    compare(&actual, &reference);
    if !tone {
        expect_onsets(&actual, &[5040, 102_000, 198_000]);
    }
    actual
}

/// Scenario "A hit before the grid sounds once in its capture lap".
#[test]
fn a_hit_before_the_grid_sounds_once_in_its_capture_lap() {
    for sizes in [&[1][..], &[128], &[1024], &[3, 127, 1, 991, 47]] {
        forward(false, sizes);
    }
}

/// Scenario "A recorded seam hit first repeats at the next loop start".
#[test]
fn a_recorded_seam_hit_first_repeats_at_the_next_loop_start() {
    for frame in [LAP - 10, LAP - 1, LAP, LAP + 1, LAP + 10] {
        for grid in [Grid::Off, Grid::Sixteenth] {
            let mut actual = Rig::new(&[], false);
            let mut reference = Rig::new(&[], false);
            let (onset, row) = fresh(&mut actual, &mut reference, frame, 100, &[128]);
            assert_eq!(row.tick(), 0);
            let origin = onset.origin(TICKS_PER_BAR).unwrap();
            assert_eq!(
                origin.capture_lap(),
                u64::from(frame >= LAP),
                "frame {frame}, accepted tick {}",
                onset.ticks
            );
            actual.publish(
                grid,
                100,
                &[row],
                &[PlaybackRow::recorded(id(0, 1), origin, false)],
            );
            let first = if frame < LAP { LAP } else { 2 * LAP };
            for at in [first, first + LAP] {
                pair_to(&mut actual, &mut reference, at, &[1024]);
                reference.send(press(100));
            }
            pair_to(&mut actual, &mut reference, first + LAP + 128, &[128]);
            compare(&actual, &reference);
            expect_onsets(&actual, &[frame, first, first + LAP]);
        }
    }
}

#[test]
fn seam_eligibility_uses_the_audio_frame_at_other_tempos() {
    for bpm in [40.0, 172.0, 300.0] {
        let period = f64::from(SR) * 240.0 / bpm;
        let seam = period.round() as usize;
        for frame in [seam - 1, seam, seam + 1] {
            let mut actual = Rig::new(&[], false);
            let mut reference = Rig::new(&[], false);
            both(&mut actual, &mut reference, Command::SetTempo { bpm });
            let (onset, row) = fresh(&mut actual, &mut reference, frame, 100, &[1024]);
            let origin = onset.origin(TICKS_PER_BAR).unwrap();
            assert_eq!(
                origin.capture_lap(),
                u64::from(frame >= seam),
                "bpm {bpm}, frame {frame}, tick {}",
                onset.ticks
            );
            actual.publish(
                Grid::Sixteenth,
                100,
                &[row],
                &[PlaybackRow::recorded(id(0, 1), origin, false)],
            );
            let first = (period * if frame < seam { 1.0 } else { 2.0 }).round() as usize;
            pair_to(&mut actual, &mut reference, first, &[128]);
            reference.send(press(100));
            pair_to(&mut actual, &mut reference, first + 128, &[128]);
            compare(&actual, &reference);
            expect_onsets(&actual, &[frame, first]);
        }
    }
}

/// Scenario "Delayed capture publication does not produce a catch up burst".
#[test]
fn delayed_capture_publication_does_not_produce_a_catch_up_burst() {
    let mut actual = Rig::new(&[], false);
    let mut reference = Rig::new(&[], false);
    let (onset, row) = fresh(&mut actual, &mut reference, 15_000, 100, &[128]);
    let rows = [PlaybackRow::recorded(
        id(0, 1),
        onset.origin(TICKS_PER_BAR).unwrap(),
        false,
    )];
    pair_to(&mut actual, &mut reference, 3 * LAP + 20_000, &[1024]);
    actual.publish(Grid::Sixteenth, 100, &[row], &rows);
    actual.publish(Grid::Quarter, 100, &[row], &rows);
    pair_to(&mut actual, &mut reference, 4 * LAP + 24_000, &[128]);
    reference.send(press(100));
    pair_to(&mut actual, &mut reference, 4 * LAP + 25_000, &[128]);
    actual.publish(Grid::Quarter, 100, &[row], &rows);
    actual.publish(Grid::Eighth, 100, &[row], &rows);
    actual.publish(Grid::Quarter, 100, &[row], &rows);
    pair_to(&mut actual, &mut reference, 5 * LAP + 24_000, &[128]);
    reference.send(press(100));
    pair_to(&mut actual, &mut reference, 5 * LAP + 25_000, &[128]);
    compare(&actual, &reference);
    expect_onsets(&actual, &[15_000, 408_000, 504_000]);
}

#[test]
fn grid_change_cannot_repeat_consumed_rows_or_a_different_collision_winner() {
    let mut actual = Rig::new(&[], false);
    let mut reference = Rig::new(&[], false);
    let (onset, row) = fresh(&mut actual, &mut reference, 15_000, 100, &[128]);
    let origin = onset.origin(TICKS_PER_BAR).unwrap();
    let rows = [
        PlaybackRow::recorded(id(0, 1), origin, false),
        PlaybackRow::recorded(id(1, 1), origin, false),
    ];
    let hits = [row, hit(650, 80, 0)];
    actual.publish(Grid::Sixteenth, 100, &hits, &rows);
    pair_to(&mut actual, &mut reference, LAP + 18_000, &[128]);
    reference.send(press(100));
    pair_to(&mut actual, &mut reference, LAP + 20_000, &[128]);
    let stronger = [row, hit(650, 120, 0)];
    actual.publish(Grid::Quarter, 100, &stronger, &rows);
    pair_to(&mut actual, &mut reference, 2 * LAP + 24_000, &[128]);
    reference.send(press(120));
    pair_to(&mut actual, &mut reference, 2 * LAP + 25_000, &[128]);
    compare(&actual, &reference);
    expect_onsets(&actual, &[15_000, 114_000, 216_000]);
}

/// Since 98c the fresh row saves its placed tick: a later grid change does not separate it
/// from an old row on the same line, and each lap plays the louder one once.
#[test]
fn collision_rows_stay_together_after_a_later_grid_change() {
    let old = hit(240, 40, 0);
    let mut actual = Rig::new(&[old], false);
    let mut reference = Rig::new(&[], false);
    let (onset, row) = fresh(&mut actual, &mut reference, 5040, 100, &[128]);
    assert_eq!(row.tick(), 240);
    let descriptors = [
        PlaybackRow::existing(id(0, 1)),
        PlaybackRow::recorded(id(1, 1), onset.origin(TICKS_PER_BAR).unwrap(), false),
    ];
    actual.publish(Grid::Sixteenth, 100, &[old, row], &descriptors);
    pair_to(&mut actual, &mut reference, 6000, &[128]);
    reference.send(press(40));
    pair_to(&mut actual, &mut reference, LAP + 6000, &[128]);
    reference.send(press(100));
    pair_to(&mut actual, &mut reference, LAP + 7500, &[128]);
    actual.publish(Grid::Off, 100, &[old, row], &descriptors);
    pair_to(&mut actual, &mut reference, 2 * LAP + 6000, &[128]);
    reference.send(press(100));
    pair_to(&mut actual, &mut reference, 2 * LAP + 6128, &[128]);
    compare(&actual, &reference);
    expect_onsets(&actual, &[5040, 6000, 102_000, 198_000]);
}

fn collision(old_velocity: u8, fresh_velocity: u8) {
    let old = hit(240, old_velocity, 0);
    let mut actual = Rig::new(&[old], false);
    let mut reference = Rig::new(&[], false);
    let (onset, row) = fresh(&mut actual, &mut reference, 5040, fresh_velocity, &[128]);
    actual.publish(
        Grid::Sixteenth,
        100,
        &[old, row],
        &[
            PlaybackRow::existing(id(0, 1)),
            PlaybackRow::recorded(id(1, 1), onset.origin(TICKS_PER_BAR).unwrap(), false),
        ],
    );
    pair_to(&mut actual, &mut reference, 6000, &[128]);
    reference.send(press(old_velocity));
    pair_to(&mut actual, &mut reference, LAP + 6000, &[128]);
    reference.send(press(old_velocity.max(fresh_velocity)));
    pair_to(&mut actual, &mut reference, LAP + 6128, &[128]);
    compare(&actual, &reference);
    expect_onsets(&actual, &[5040, 6000, 102_000]);
}

/// Scenario "An ineligible overdub keeps the previous rhythm".
#[test]
fn an_ineligible_overdub_keeps_the_previous_rhythm() {
    collision(40, 100);
}

/// Scenario "A manual hit does not suppress an existing pattern hit".
#[test]
fn a_manual_hit_does_not_suppress_an_existing_pattern_hit() {
    collision(100, 40);
}

#[test]
fn pending_gates_stay_silent_until_completion_without_replaying_a_consumed_lap() {
    for release_tick in [600, 800] {
        let mut actual = Rig::new(&[], false);
        let mut reference = Rig::new(&[], false);
        both(
            &mut actual,
            &mut reference,
            Command::SetPadMode {
                pad: 0,
                mode: PadMode::Gate,
            },
        );
        actual.send(Command::PatternRecordStart { pass: 1 });
        pair_to(&mut actual, &mut reference, 15_000, &[128]);
        both(&mut actual, &mut reference, press(100));
        let onset = actual.onset();
        reference.block(0);
        assert_eq!(actual.recovery.completed(onset), None);
        let origin = onset.origin(TICKS_PER_BAR).unwrap();
        actual.publish(
            Grid::Sixteenth,
            100,
            &[hit(600, 100, 0)],
            &[PlaybackRow::recorded(id(0, 1), origin, true)],
        );
        pair_to(&mut actual, &mut reference, LAP + release_tick * 25, &[128]);
        both(&mut actual, &mut reference, Command::PadRelease { pad: 0 });
        actual.block(0);
        reference.block(0);
        let completed = actual.recovery.completed(onset).unwrap();
        actual.publish(
            Grid::Sixteenth,
            100,
            &[completed],
            &[PlaybackRow::recorded(id(0, 1), origin, false)],
        );
        let (first, grid) = if release_tick < 720 {
            (LAP + 18_000, Grid::Sixteenth)
        } else {
            (2 * LAP + 24_000, Grid::Quarter)
        };
        actual.publish(
            grid,
            100,
            &[completed],
            &[PlaybackRow::recorded(id(0, 1), origin, false)],
        );
        pair_to(&mut actual, &mut reference, first, &[128]);
        reference.send(press(100));
        pair_to(&mut actual, &mut reference, first + 128, &[128]);
        compare(&actual, &reference);
        expect_onsets(&actual, &[15_000, first]);
    }
}

#[test]
fn stop_restart_clears_old_origins_and_a_new_capture_uses_a_new_run() {
    for stop in [Command::TransportStop, Command::TransportToggle] {
        let mut actual = Rig::new(&[], false);
        let mut reference = Rig::new(&[], false);
        let (onset, row) = fresh(&mut actual, &mut reference, 3 * LAP + 2500, 100, &[1024]);
        actual.publish(
            Grid::Quarter,
            100,
            &[row],
            &[PlaybackRow::recorded(
                id(0, 1),
                onset.origin(TICKS_PER_BAR).unwrap(),
                false,
            )],
        );
        both(&mut actual, &mut reference, stop.clone());
        actual.block(0);
        reference.block(0);
        let end = actual.stats.recording(0).unwrap();
        actual.send(Command::PatternRecordAck {
            pass: 1,
            sequence: end.sequence,
        });
        both(&mut actual, &mut reference, Command::TransportPlay);
        reference.send(press(100));
        actual.send(Command::PatternRecordStart { pass: 2 });
        both(&mut actual, &mut reference, press(80));
        let newer = actual.onset();
        reference.block(0);
        assert_ne!(newer.run, onset.run);
        pair_to(&mut actual, &mut reference, 3 * LAP + 2628, &[128]);
        compare(&actual, &reference);
    }
}

#[test]
fn reused_runtime_slot_does_not_inherit_another_generation() {
    for generation in [1, 2] {
        let mut actual = Rig::new(&[hit(240, 100, 0)], false);
        let mut reference = Rig::new(&[], false);
        pair_to(&mut actual, &mut reference, 6000, &[128]);
        reference.send(press(100));
        pair_to(&mut actual, &mut reference, 7000, &[128]);
        actual.publish(
            Grid::Off,
            100,
            &[hit(480, 80, 0)],
            &[PlaybackRow::existing(id(0, generation))],
        );
        pair_to(&mut actual, &mut reference, 12_000, &[128]);
        if generation == 2 {
            reference.send(press(80));
        }
        pair_to(&mut actual, &mut reference, 12_128, &[128]);
        compare(&actual, &reference);
    }
}

#[test]
fn existing_rows_get_history_before_the_first_recording_grid_edit() {
    let row = hit(600, 100, 0);
    let mut actual = Rig::new(&[], false);
    let mut reference = Rig::new(&[], false);
    actual.send(Command::SetPattern {
        pattern: Some(Arc::new(
            RawPattern::new(1, Grid::Sixteenth, 100, vec![row])
                .unwrap()
                .compile(),
        )),
    });
    pair_to(&mut actual, &mut reference, 18_000, &[128]);
    reference.send(press(100));
    pair_to(&mut actual, &mut reference, 20_000, &[128]);
    actual.send(Command::PatternRecordStart { pass: 1 });
    let descriptors = [PlaybackRow::existing(id(0, 1))];
    actual.publish(Grid::Sixteenth, 100, &[row], &descriptors);
    actual.publish(Grid::Quarter, 100, &[row], &descriptors);
    pair_to(&mut actual, &mut reference, LAP + 24_000, &[128]);
    reference.send(press(100));
    pair_to(&mut actual, &mut reference, LAP + 24_128, &[128]);
    compare(&actual, &reference);
    expect_onsets(&actual, &[18_000, 120_000]);
}

#[test]
fn recorded_replacements_and_empty_callbacks_preserve_the_first_frame() {
    let row = hit(0, 100, 0);
    let mut actual = Rig::new(&[row], false);
    let mut reference = Rig::new(&[], false);
    for _ in 0..5 {
        actual.publish(
            Grid::Quarter,
            100,
            &[row],
            &[PlaybackRow::existing(id(0, 1))],
        );
        actual.block(0);
    }
    reference.send(press(100));
    pair_to(&mut actual, &mut reference, 128, &[128]);
    actual.publish(
        Grid::Quarter,
        100,
        &[row],
        &[PlaybackRow::existing(id(0, 1))],
    );
    pair_to(&mut actual, &mut reference, 256, &[128]);
    compare(&actual, &reference);
    expect_onsets(&actual, &[0]);
}

#[test]
fn recorded_compilation_checks_identities_and_preserves_candidates_and_raw_rows() {
    assert_eq!(RowId::new(MAX_HITS, 1), Err(Error::RowIdentity));
    assert_eq!(RowId::new(0, 0), Err(Error::RowIdentity));
    let raw = RawPattern::new(
        1,
        Grid::Quarter,
        100,
        vec![
            hit(100, 90, 100),
            hit(101, 100, 2),
            hit(102, 100, 3),
            hit(3840, 127, 0),
        ],
    )
    .unwrap();
    let before = raw.clone();
    let descriptors: Vec<_> = (0..4).map(|i| PlaybackRow::existing(id(i, 1))).collect();
    assert_eq!(
        raw.compile_recorded(&descriptors[..3]),
        Err(Error::PlaybackRows)
    );
    let mut duplicate = descriptors.clone();
    duplicate[3] = PlaybackRow::existing(id(0, 2));
    assert_eq!(
        raw.compile_recorded(&duplicate),
        Err(Error::RowIdentity),
        "hidden rows also need distinct identities"
    );
    let compiled = raw.compile_recorded(&descriptors).unwrap();
    assert_eq!(compiled.raw_rows(), 4);
    assert_eq!(
        compiled.events(),
        &[hit(0, 100, 3), hit(0, 100, 2), hit(0, 90, 100)]
    );
    assert_eq!(raw, before);
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
            let raw = RawPattern::new(1, grid, strength, vec![hit(201, 100, 0)]).unwrap();
            assert_eq!(
                raw.compile_recorded(&descriptors[..1]).unwrap().events(),
                raw.compile().events()
            );
        }
    }
}

#[test]
fn project_reset_clears_history_and_stale_recorded_replacements_stay_obsolete() {
    let mut actual = Rig::new(&[hit(0, 100, 0)], false);
    actual.block(128);
    let stale = RawPattern::new(1, Grid::Off, 100, vec![hit(0, 40, 0)])
        .unwrap()
        .compile_recorded(&[PlaybackRow::existing(id(0, 1))])
        .unwrap();
    actual.send(Command::SetPattern {
        pattern: Some(Arc::new(stale)),
    });
    actual.project.epoch.store(1, Release);
    for command in [
        Command::SetPadSound {
            pad: 0,
            sound: Some(sample(false)),
        },
        Command::SetPattern {
            pattern: Some(Arc::new(
                RawPattern::new(1, Grid::Off, 100, vec![hit(0, 80, 0)])
                    .unwrap()
                    .compile_recorded(&[PlaybackRow::existing(id(0, 1))])
                    .unwrap(),
            )),
        },
    ] {
        actual
            .tx
            .push(ProjectCommand {
                epoch: 1,
                ready_at_send: false,
                command: ProjectMessage::Command(command),
            })
            .unwrap();
    }
    actual
        .tx
        .push(ProjectCommand {
            epoch: 1,
            ready_at_send: false,
            command: ProjectMessage::Ready,
        })
        .unwrap();
    for _ in 0..1000 {
        actual.block(128);
        if actual.project.ready.load(Acquire) == 1 {
            break;
        }
    }
    assert_eq!(actual.project.ready.load(Acquire), 1);
    let start = actual.wave.0.len();
    actual.send(Command::TransportPlay);
    actual.block(128);
    assert!((actual.wave.0[start] - 0.04 * 80.0 / 127.0).abs() < 0.000001);
    assert_eq!(checks::onsets(&actual.wave.0[start..], 0.0000001, 1), [0]);
}

#[test]
fn required_recorded_repeat_render_matches_manual_sound_and_has_no_extra_attacks() {
    let actual = forward(true, &[128]);
    for channel in [&actual.wave.0, &actual.wave.1] {
        assert_eq!(
            checks::onsets(channel, 0.0000001, 4800),
            [5040, 102_000, 198_000]
        );
        assert_eq!(checks::first_non_finite(channel), None);
        assert!((0.02..0.05).contains(&checks::peak(channel)));
        assert!(checks::max_step(channel) < 0.004);
        assert!(channel[channel.len() - 4800..].iter().all(|v| *v == 0.0));
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../renders/97-pattern-repeat.wav");
    engine::offline::write_wav(&path, SR, &actual.wave.0, &actual.wave.1).unwrap();
}

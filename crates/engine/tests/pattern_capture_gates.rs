//! Gate capture and completed-take playback through production queues, without devices.

use std::sync::{Arc, atomic::Ordering::Acquire};

use engine::capture::{CAPACITY, Event, Kind, Phase, Recovery, Snapshot};
use engine::pattern::{Grid, Hit, MAX_GATE_TICKS, MAX_HITS, RawPattern, TICKS_PER_BAR};
use engine::rt::{ProjectCommand, ProjectMessage, ProjectState, Realtime, Stats};
use engine::{Command, Engine, PadMode, Retired, checks};

const SR: u32 = 48_000;
type Wave = (Vec<f32>, Vec<f32>);

struct Rig {
    tx: rtrb::Producer<ProjectCommand>,
    events: rtrb::Consumer<Event>,
    recovery: Arc<Recovery>,
    garbage: rtrb::Consumer<Retired>,
    rt: Realtime,
    stats: Arc<Stats>,
    project: Arc<ProjectState>,
}

fn compiled(rows: Vec<Hit>) -> Arc<engine::pattern::Pattern> {
    Arc::new(RawPattern::new(1, Grid::Off, 100, rows).unwrap().compile())
}

fn press(pad: usize, velocity: u8) -> Command {
    Command::PadPress {
        pad,
        velocity: f32::from(velocity) / 127.0,
    }
}

impl Rig {
    fn new(rows: Vec<Hit>) -> Self {
        Self::at_rate(rows, SR)
    }

    fn at_rate(rows: Vec<Hit>, sr: u32) -> Self {
        let tone = (0..sr)
            .map(|frame| 0.04 * (std::f32::consts::TAU * 55.0 * frame as f32 / sr as f32).cos())
            .collect();
        let mut engine = Engine::with_sounds(sr, vec![], vec![tone]);
        engine.handle(
            0,
            Command::SetPattern {
                pattern: Some(compiled(rows)),
            },
        );
        for pad in [0, 1, 3] {
            engine.handle(
                0,
                Command::SetPadMode {
                    pad,
                    mode: PadMode::Gate,
                },
            );
            engine.handle(
                0,
                Command::SetPadEnvelope {
                    pad,
                    attack_ms: 2,
                    decay_ms: 0,
                    release_ms: 10,
                },
            );
        }
        let events = engine.enable_capture();
        let recovery = engine.capture_recovery().unwrap();
        let (garbage_tx, garbage) = rtrb::RingBuffer::new(1024);
        engine.set_garbage(garbage_tx);
        let (tx, rx) = rtrb::RingBuffer::new(1024);
        let stats = Arc::new(Stats::default());
        let project = Arc::new(ProjectState::default());
        let rt = Realtime::with_project(engine, vec![rx], stats.clone(), sr, project.clone());
        Self {
            tx,
            events,
            recovery,
            garbage,
            rt,
            stats,
            project,
        }
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

    fn start(&mut self, pass: u64) {
        self.send(Command::TransportPlay);
        self.send(Command::PatternRecordStart { pass });
    }

    fn block(&mut self, frames: usize) -> Wave {
        while self.garbage.pop().is_ok() {}
        let (mut left, mut right) = (vec![0.0; frames], vec![0.0; frames]);
        self.rt.process(&mut left, &mut right);
        assert!(
            left.iter()
                .chain(&right)
                .all(|v| v.is_finite() && v.abs() < 1.0)
        );
        (left, right)
    }

    fn advance(&mut self, frames: usize) -> Wave {
        let mut wave = (Vec::new(), Vec::new());
        let mut at = 0;
        while at < frames {
            let end = (at + 128).min(frames);
            append(&mut wave, self.block(end - at));
            at = end;
        }
        if frames == 0 {
            self.block(0);
        }
        wave
    }

    fn drain(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok(event) = self.events.pop() {
            events.push(event);
        }
        events
    }

    fn state(&self) -> Snapshot {
        self.stats
            .recording(self.project.epoch.load(Acquire))
            .unwrap()
    }
}

fn append(to: &mut Wave, from: Wave) {
    to.0.extend(from.0);
    to.1.extend(from.1);
}

fn compare(actual: &Wave, reference: &Wave) {
    for (actual, reference) in [(&actual.0, &reference.0), (&actual.1, &reference.1)] {
        assert_eq!(actual.len(), reference.len());
        for (frame, (a, b)) in actual.iter().zip(reference).enumerate() {
            assert!((a - b).abs() <= 0.000001, "frame {frame}: {a} != {b}");
        }
    }
}

fn paired(recording: &mut Rig, control: &mut Rig, frames: usize) -> Wave {
    let actual = recording.advance(frames);
    compare(&actual, &control.advance(frames));
    actual
}

fn both(recording: &mut Rig, control: &mut Rig, command: Command) {
    recording.send(command.clone());
    control.send(command);
}

fn row(recovery: &Recovery, event: Event, tick: u32, duration: u32) -> Hit {
    let hit = recovery.completed(event).expect("complete accepted onset");
    assert_eq!((hit.tick(), hit.gate_ticks()), (tick, duration));
    hit
}

fn saved_playback(rows: &[Hit], laps: u32) -> Wave {
    let mut actual = Rig::new(rows.to_vec());
    let mut reference = Rig::new(vec![]);
    both(&mut actual, &mut reference, Command::TransportPlay);
    let ticks = laps * TICKS_PER_BAR + 960;
    let mut events = Vec::new();
    for lap in 0..laps {
        for hit in rows {
            let start = (lap * TICKS_PER_BAR + hit.tick()) as usize * 25;
            events.push((start, press(hit.pad(), hit.velocity())));
            if hit.gate_ticks() > 0 {
                events.push((
                    start + hit.gate_ticks() as usize * 25,
                    Command::PadRelease { pad: hit.pad() },
                ));
            }
        }
    }
    events.sort_by_key(|(frame, _)| *frame);
    let mut wave = (Vec::new(), Vec::new());
    let mut reference_wave = (Vec::new(), Vec::new());
    let (mut at, mut next) = (0, 0);
    let end = ticks as usize * 25;
    // A seam Gate must finish before Stop. These fixtures have no intervening next-lap onset.
    let stop = (laps as usize * TICKS_PER_BAR as usize * 25)
        .max(events.last().map_or(0, |(frame, _)| frame + 1024));
    let next_onset =
        (laps * TICKS_PER_BAR + rows.iter().map(|h| h.tick()).min().unwrap()) as usize * 25;
    assert!(stop <= next_onset && stop < end);
    while at < end {
        while next < events.len() && events[next].0 == at {
            reference.send(events[next].1.clone());
            next += 1;
        }
        if at == stop {
            both(&mut actual, &mut reference, Command::TransportStop);
        }
        let next_event = events.get(next).map_or(end, |(frame, _)| *frame);
        let boundary = if at < stop { stop } else { end };
        let until = (at + 128).min(next_event).min(boundary).min(end);
        assert!(until > at);
        append(&mut wave, actual.block(until - at));
        append(&mut reference_wave, reference.block(until - at));
        at = until;
    }
    compare(&wave, &reference_wave);
    for channel in [&wave.0, &wave.1] {
        assert_eq!(checks::first_non_finite(channel), None);
        assert!(checks::peak(channel) < 0.1);
        assert!(checks::max_step(channel) < 0.004, "release click");
        assert!(channel[end - 4800..].iter().all(|&v| v == 0.0));
    }
    wave
}

/// Scenario "Record a gate across the loop seam and a tempo change".
#[test]
fn record_a_gate_across_the_loop_seam_and_a_tempo_change() {
    let mut recording = Rig::new(vec![]);
    let mut control = Rig::new(vec![]);
    recording.start(1);
    control.send(Command::TransportPlay);
    let mut take = paired(&mut recording, &mut control, 90_000);
    both(&mut recording, &mut control, press(0, 100));
    append(&mut take, paired(&mut recording, &mut control, 6000));
    let onset = recording.drain()[0];
    assert_eq!(
        recording.recovery.completed(onset),
        None,
        "pending gates cannot replay"
    );
    both(
        &mut recording,
        &mut control,
        Command::SetTempo { bpm: 240.0 },
    );
    append(&mut take, paired(&mut recording, &mut control, 3000));
    both(&mut recording, &mut control, Command::PadRelease { pad: 0 });
    recording.send(Command::PatternRecordEnd { pass: 1 });
    append(&mut take, paired(&mut recording, &mut control, 6000));
    let hit = row(&recording.recovery, onset, 3600, 480);
    assert_eq!((hit.pad(), hit.velocity()), (0, 100));
    assert_eq!(recording.state().phase, Phase::Finishing);
    assert!(recording.stats.transport(0).unwrap().playing);
    let repeats = saved_playback(&[hit], 2);
    for channel in [&repeats.0, &repeats.1] {
        let starts = checks::onsets(channel, 0.000001, 1024);
        assert_eq!(starts.len(), 2);
        for (got, expected) in starts.iter().zip([90_000, 186_000]) {
            assert!(got.abs_diff(expected) <= 1);
        }
        assert!((0.03..0.04).contains(&checks::peak(channel)));
    }
    append(
        &mut take,
        (vec![0.0; SR as usize / 4], vec![0.0; SR as usize / 4]),
    );
    append(&mut take, repeats);
    for channel in [&take.0, &take.1] {
        assert_eq!(checks::first_non_finite(channel), None);
        assert!(checks::peak(channel) < 1.0);
        assert!(checks::max_step(channel) < 0.004);
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../renders/97-pattern-capture-gates.wav");
    engine::offline::write_wav(&path, SR, &take.0, &take.1).unwrap();
}

/// Scenario "End recording before the gate releases".
#[test]
fn end_recording_before_the_gate_releases() {
    let mut recording = Rig::new(vec![]);
    let mut control = Rig::new(vec![]);
    recording.start(10);
    control.send(Command::TransportPlay);
    both(&mut recording, &mut control, press(0, 100));
    paired(&mut recording, &mut control, 2400);
    let onset = recording.drain()[0];
    recording.send(Command::PatternRecordEnd { pass: 10 });
    let tail = paired(&mut recording, &mut control, 128);
    assert!(
        checks::peak(&tail.0) > 0.02,
        "record end must not release manual sound"
    );
    let original = row(&recording.recovery, onset, 0, 96);
    let final_sequence = recording.state().sequence;
    recording.send(Command::PatternRecordAck {
        pass: 10,
        sequence: final_sequence,
    });
    recording.send(Command::PatternRecordStart { pass: 11 });
    both(&mut recording, &mut control, Command::PadRelease { pad: 0 });
    paired(&mut recording, &mut control, 128);
    assert_eq!(
        recording.state().captured_onsets,
        0,
        "old release is not a new onset"
    );
    assert_eq!(
        recording.recovery.completed(onset),
        None,
        "old pass is obsolete"
    );
    both(&mut recording, &mut control, press(0, 90));
    paired(&mut recording, &mut control, 1200);
    let newer = recording
        .drain()
        .into_iter()
        .find(|e| matches!(e.kind, Kind::Onset { .. }))
        .unwrap();
    assert_eq!(recording.recovery.completed(newer), None);
    recording.send(Command::PatternRecordEnd { pass: 10 });
    recording.send(Command::PatternRecordAck {
        pass: 10,
        sequence: final_sequence,
    });
    paired(&mut recording, &mut control, 0);
    assert_eq!(recording.state().phase, Phase::Recording);
    assert_eq!(recording.recovery.completed(newer), None);
    both(&mut recording, &mut control, Command::PadRelease { pad: 0 });
    paired(&mut recording, &mut control, 128);
    assert_eq!(
        recording.recovery.completed(newer).unwrap().gate_ticks(),
        48
    );
    assert_eq!(original.gate_ticks(), 96, "saved row is unchanged");
    saved_playback(&[original], 2);
}

#[test]
fn retrigger_closes_only_the_previous_gate_and_preserves_manual_sound() {
    let mut recording = Rig::new(vec![]);
    let mut control = Rig::new(vec![]);
    recording.start(1);
    control.send(Command::TransportPlay);
    both(&mut recording, &mut control, press(0, 100));
    paired(&mut recording, &mut control, 2400);
    let first = recording.drain()[0];
    both(&mut recording, &mut control, press(0, 80));
    paired(&mut recording, &mut control, 1200);
    let second = recording.drain()[0];
    let a = row(&recording.recovery, first, 0, 96);
    assert_ne!(first.gate, second.gate);
    assert_eq!(recording.recovery.completed(second), None);
    recording.send(Command::PatternRecordEnd { pass: 1 });
    paired(&mut recording, &mut control, 0);
    let b = row(&recording.recovery, second, 96, 48);
    both(&mut recording, &mut control, Command::PadRelease { pad: 0 });
    paired(&mut recording, &mut control, 1024);
    assert_eq!(recording.recovery.completed(first), Some(a));
    assert_eq!(recording.recovery.completed(second), Some(b));
    saved_playback(&[a, b], 2);
}

#[test]
fn gate_duration_rounds_ties_later_clamps_and_uses_zero_only_for_one_shots() {
    for (frames, duration) in [(0, 1), (4, 1), (5, 1), (14, 1), (15, 2), (24, 2), (25, 3)] {
        let mut engine = Engine::with_sounds(SR, vec![], vec![]);
        let mut events = engine.enable_capture();
        let recovery = engine.capture_recovery().unwrap();
        for command in [
            Command::SetPattern {
                pattern: Some(compiled(vec![])),
            },
            Command::SetTempo { bpm: 300.0 },
            Command::TransportPlay,
            Command::SetPadMode {
                pad: 0,
                mode: PadMode::Gate,
            },
            Command::PatternRecordStart { pass: 1 },
            press(0, 127),
        ] {
            engine.handle(0, command);
        }
        let onset = events.pop().unwrap();
        engine.handle(frames, Command::PadRelease { pad: 0 });
        row(&recovery, onset, 0, duration);
    }
    for duration in [MAX_GATE_TICKS - 2, MAX_GATE_TICKS, MAX_GATE_TICKS + 2] {
        let mut rig = Rig::at_rate(vec![], 960);
        rig.start(1);
        rig.send(press(0, 127));
        // At 120 BPM and 960 Hz each frame advances exactly two musical ticks.
        rig.advance((duration / 2) as usize);
        let onset = rig.drain()[0];
        rig.send(Command::PadRelease { pad: 0 });
        rig.block(0);
        row(
            &rig.recovery,
            onset,
            0,
            (duration / 2 * 2).min(MAX_GATE_TICKS),
        );
    }
    let mut rig = Rig::new(vec![]);
    rig.start(1);
    rig.send(press(2, 127));
    rig.block(0);
    let onset = rig.drain()[0];
    assert_eq!(onset.gate, None);
    row(&rig.recovery, onset, 0, 0);
}

#[test]
fn lost_release_retrigger_and_end_feedback_recover_only_accepted_onsets() {
    let old = vec![Hit::new(0, 127, 20, 0).unwrap()];
    let mut recording = Rig::new(old.clone());
    let mut control = Rig::new(old);
    recording.start(1);
    control.send(Command::TransportPlay);
    for pad in [0, 1, 3] {
        both(&mut recording, &mut control, press(pad, 40));
    }
    paired(&mut recording, &mut control, 128);
    let accepted = recording.drain();
    assert_eq!(accepted.len(), 3);
    for _ in 0..4 {
        for _ in 0..CAPACITY / 4 {
            both(&mut recording, &mut control, press(2, 1));
        }
        paired(&mut recording, &mut control, 128);
    }
    both(&mut recording, &mut control, Command::PadRelease { pad: 0 });
    both(&mut recording, &mut control, press(1, 50));
    both(&mut recording, &mut control, press(0, 60));
    recording.send(Command::PatternRecordEnd { pass: 1 });
    paired(&mut recording, &mut control, 128);
    let end = recording.state();
    assert_eq!(end.phase, Phase::Finishing);
    assert_eq!((end.dropped_onsets, end.dropped_events), (2, 3));
    assert_eq!(end.limit_refusals, 0);
    assert_eq!(end.captured_onsets, CAPACITY as u32 + 3);
    assert_eq!(end.reserved_rows, CAPACITY as u32 + 4);
    for onset in &accepted {
        row(&recording.recovery, *onset, 0, 26);
    }
    let feedback = recording.drain();
    assert_eq!(feedback.len(), CAPACITY);
    assert!(
        feedback
            .iter()
            .all(|e| matches!(e.kind, Kind::Onset { pad: 2, .. }))
    );
    assert!(
        feedback
            .iter()
            .all(|e| recording.recovery.completed(*e).is_some())
    );
    paired(&mut recording, &mut control, 4096);
    assert!(
        recording.drain().is_empty(),
        "lost onsets never arrive later"
    );
    assert_eq!(recording.state(), end);
    assert_eq!(
        recording.stats.pattern_hits(0),
        control.stats.pattern_hits(0)
    );
    let rows: Vec<_> = accepted
        .iter()
        .map(|e| recording.recovery.completed(*e).unwrap())
        .collect();
    saved_playback(&rows, 2);
}

#[test]
fn hidden_rows_collisions_and_pending_gates_share_the_raw_hit_limit() {
    for count in [MAX_HITS - 1, MAX_HITS] {
        let rows: Vec<_> = (0..count)
            .map(|i| Hit::new(if i % 2 == 0 { 0 } else { TICKS_PER_BAR + 1 }, 127, 20, 0).unwrap())
            .collect();
        let pattern = compiled(rows.clone());
        assert_eq!(pattern.raw_rows(), count);
        assert_eq!(pattern.events().len(), 1);
        let mut recording = Rig::new(rows.clone());
        let mut control = Rig::new(rows);
        recording.start(1);
        control.send(Command::TransportPlay);
        both(&mut recording, &mut control, press(0, 50));
        // Measure after the existing 20 ms envelope glide, not only its first 128 frames.
        let sound = paired(&mut recording, &mut control, 960);
        assert!(checks::peak(&sound.0) > 0.01);
        both(&mut recording, &mut control, press(0, 60));
        both(&mut recording, &mut control, press(1, 70));
        paired(&mut recording, &mut control, 128);
        recording.send(Command::PatternRecordEnd { pass: 1 });
        paired(&mut recording, &mut control, 0);
        let state = recording.state();
        assert_eq!(state.reserved_rows, MAX_HITS as u32);
        assert_eq!(state.captured_onsets, (MAX_HITS - count) as u32);
        assert_eq!(state.limit_refusals, (3 - (MAX_HITS - count)) as u64);
        assert_eq!(state.dropped_onsets, state.limit_refusals);
        let events = recording.drain();
        assert_eq!(events.len(), MAX_HITS - count);
        if let Some(event) = events.first() {
            let accepted = row(&recording.recovery, *event, 0, 38);
            saved_playback(&[accepted], 2);
        }
        assert_eq!(pattern.raw_rows(), count);
        assert_eq!(pattern.events().len(), 1);
        paired(&mut recording, &mut control, 4096);
        assert!(recording.drain().is_empty());
    }
}

#[test]
fn stop_retains_complete_gate_at_pre_reset_tick_and_ignores_duplicate_end() {
    for stop in [Command::TransportStop, Command::TransportToggle] {
        let mut recording = Rig::new(vec![]);
        let mut control = Rig::new(vec![]);
        recording.start(1);
        control.send(Command::TransportPlay);
        both(&mut recording, &mut control, press(0, 90));
        paired(&mut recording, &mut control, 2400);
        let onset = recording.drain()[0];
        both(&mut recording, &mut control, stop);
        let tail = paired(&mut recording, &mut control, 128);
        assert!(checks::peak(&tail.0) > 0.02);
        assert_eq!(recording.stats.transport(0).unwrap().ticks, 0.0);
        let hit = row(&recording.recovery, onset, 0, 96);
        let end = recording.state();
        recording.send(Command::PatternRecordEnd { pass: 1 });
        both(&mut recording, &mut control, Command::TransportPlay);
        paired(&mut recording, &mut control, 128);
        assert_eq!(recording.state(), end);
        assert_eq!(recording.recovery.completed(onset), Some(hit));
    }
}

#[test]
fn project_reset_invalidates_terminal_data_even_before_old_slots_are_reused() {
    use std::sync::atomic::Ordering::Release;
    let mut rig = Rig::new(vec![]);
    rig.start(1);
    rig.send(press(0, 90));
    rig.advance(2400);
    let onset = rig.drain()[0];
    rig.send(Command::PadRelease { pad: 0 });
    rig.block(0);
    row(&rig.recovery, onset, 0, 96);
    rig.project.epoch.store(1, Release);
    rig.block(128);
    assert_eq!(rig.recovery.completed(onset), None);
    assert_eq!(rig.state().phase, Phase::Idle);
    assert_eq!(rig.state().reserved_rows, 0);
}

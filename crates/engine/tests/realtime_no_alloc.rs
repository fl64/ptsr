//! The audio callback does not allocate: count allocator calls around `Realtime::process`.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::Arc;

use engine::Engine;
use engine::rt::{Command, Realtime, Stats};

struct Counting;

thread_local! {
    // per thread: tests run in parallel and must not count each other's allocations
    static ALLOCS: Cell<usize> = const { Cell::new(0) };
}

fn allocs() -> usize {
    ALLOCS.with(Cell::get)
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.with(|c| c.set(c.get() + 1));
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        ALLOCS.with(|c| c.set(c.get() + 1));
        unsafe { System.dealloc(p, l) }
    }
}

#[global_allocator]
static A: Counting = Counting;

fn compiled_pattern(bars: u8, rows: &[[u32; 4]]) -> Arc<engine::pattern::Pattern> {
    use engine::pattern::{Grid, Hit, RawPattern};
    Arc::new(
        RawPattern::new(
            bars,
            Grid::Off,
            0,
            rows.iter()
                .map(|&[t, p, v, g]| Hit::new(t, p as usize, v as u8, g).unwrap())
                .collect(),
        )
        .unwrap()
        .compile(),
    )
}

/// Scenario "Pattern retirement pressure does not block or free in the callback": atomic replacement and swap.
#[test]
fn full_retirement_defers_whole_pattern_swap_and_replacement() {
    for swap in [false, true] {
        let (mut tx, rx) = rtrb::RingBuffer::new(16);
        let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
        garbage
            .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
            .unwrap();
        let old = compiled_pattern(1, &[[0, 0, 127, 0]]);
        let old_weak = Arc::downgrade(&old);
        let next = compiled_pattern(2, &[[0, 127, 127, 0]]);
        let next_weak = Arc::downgrade(&next);
        let mut e = Engine::with_sounds(48_000, vec![], vec![vec![0.01; 48_000]]);
        e.handle(0, Command::SetPattern { pattern: Some(old) });
        e.handle(0, Command::TransportPlay);
        e.set_garbage(garbage);
        let stats = Arc::new(Stats::default());
        let mut rt = Realtime::new(e, vec![rx], stats.clone(), 48_000);
        let (mut l, mut r) = ([0.0; 128], [0.0; 128]);
        let before = allocs();
        rt.process(&mut l, &mut r);
        assert_eq!(allocs(), before);
        assert!(l.iter().all(|v| *v == 0.01));
        tx.push(if swap {
            Command::SwapPatternPads {
                a: 0,
                b: 127,
                pattern: Some(next),
            }
        } else {
            Command::SetPattern {
                pattern: Some(next),
            }
        })
        .unwrap();
        for _ in 0..5 {
            let before = allocs();
            rt.process(&mut l, &mut r);
            assert_eq!(allocs(), before);
            assert_eq!(stats.transport(0).unwrap().pattern_ticks, 3840);
            assert!(l.iter().all(|v| *v == 0.01));
        }
        assert!(old_weak.upgrade().is_some() && next_weak.upgrade().is_some());
        drop(trash.pop().unwrap());
        let before = allocs();
        rt.process(&mut l, &mut r);
        assert_eq!(allocs(), before);
        assert_eq!(stats.transport(0).unwrap().pattern_ticks, 7680);
        assert!(
            old_weak.upgrade().is_some(),
            "last reference waits outside callback"
        );
        drop(trash.pop().unwrap());
        assert!(old_weak.upgrade().is_none());
        tx.push(Command::SetPattern { pattern: None }).unwrap();
        let before = allocs();
        rt.process(&mut l, &mut r);
        assert_eq!(allocs(), before);
        assert_eq!(stats.transport(0).unwrap().pattern_ticks, 0);
        assert!(next_weak.upgrade().is_some());
        drop(trash.pop().unwrap());
        assert!(next_weak.upgrade().is_none());
    }
}

#[test]
fn recorded_publication_full_rows_and_retirement_preserve_voices_without_allocation() {
    use engine::pattern::{Grid, Hit, MAX_HITS, PlaybackRow, RawPattern, RowId};
    let rows: Vec<_> = (0..engine::MAX_VOICES)
        .map(|pad| [0, pad as u32, 127, 960])
        .collect();
    let mut engine = Engine::with_sounds(48_000, vec![], vec![vec![0.0001; 48_000]]);
    let old = compiled_pattern(1, &rows);
    let weak = Arc::downgrade(&old);
    for pad in 0..engine::MAX_VOICES {
        engine.handle(
            0,
            Command::SetPadMode {
                pad,
                mode: engine::PadMode::Gate,
            },
        );
    }
    engine.handle(0, Command::SetPattern { pattern: Some(old) });
    engine.handle(0, Command::TransportPlay);
    let next_rows = (0..MAX_HITS)
        .map(|i| Hit::new(0, i % engine::MAX_VOICES, 127, 1).unwrap())
        .collect();
    let descriptors: Vec<_> = (0..MAX_HITS)
        .map(|i| PlaybackRow::existing(RowId::new(i, 1).unwrap()))
        .collect();
    let next = Arc::new(
        RawPattern::new(1, Grid::Quarter, 100, next_rows)
            .unwrap()
            .compile_recorded(&descriptors)
            .unwrap(),
    );
    let (mut tx, rx) = rtrb::RingBuffer::new(4);
    let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
    garbage
        .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
        .unwrap();
    engine.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 48_000);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    tx.push(Command::PublishRecordedPattern { pattern: next })
        .unwrap();
    for _ in 0..4 {
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        assert_eq!(tx.slots(), 3, "full retirement retains publication");
    }
    drop(trash.pop().unwrap());
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    assert_eq!(tx.slots(), 4);
    assert!(weak.upgrade().is_some());
    for channel in [&left, &right] {
        assert!(
            channel.iter().all(|&v| (v - 0.0032).abs() < 1e-6),
            "old 32-voice Gate tail survives publication"
        );
    }
    drop(trash.pop().unwrap());
    assert!(
        weak.upgrade().is_none(),
        "old pattern frees outside callback"
    );
    tx.push(Command::TransportStop).unwrap();
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
}

#[test]
fn obsolete_recorded_publication_waits_for_retirement_without_allocation() {
    use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
    use std::sync::atomic::Ordering::Release;
    for refusal in ["stale epoch", "unready at send", "unready at receive"] {
        let (mut tx, rx) = rtrb::RingBuffer::new(4);
        let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
        garbage
            .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
            .unwrap();
        let mut engine = Engine::with_sounds(48_000, vec![], vec![]);
        engine.handle(
            0,
            Command::SetPattern {
                pattern: Some(compiled_pattern(1, &[])),
            },
        );
        engine.set_garbage(garbage);
        let next = compiled_pattern(2, &[]);
        let weak = Arc::downgrade(&next);
        let project = Arc::new(ProjectState::default());
        let epoch = u64::from(refusal == "unready at receive");
        if epoch != 0 {
            project.epoch.store(epoch, Release);
        }
        tx.push(ProjectCommand {
            epoch: if refusal == "stale epoch" { 9 } else { epoch },
            ready_at_send: refusal != "unready at send",
            command: ProjectMessage::Command(Command::PublishRecordedPattern { pattern: next }),
        })
        .unwrap();
        let stats = Arc::new(Stats::default());
        let mut rt = Realtime::with_project(engine, vec![rx], stats.clone(), 48_000, project);
        for _ in 0..4 {
            let before = allocs();
            rt.process(&mut [0.0; 128], &mut [0.0; 128]);
            assert_eq!(allocs(), before);
            assert_eq!(tx.slots(), 3);
        }
        // One retirement slot also drains all 128 pads from the previous project.
        for _ in 0..512 {
            while trash.pop().is_ok() {}
            let before = allocs();
            rt.process(&mut [0.0; 128], &mut [0.0; 128]);
            assert_eq!(allocs(), before);
            if tx.slots() == 4 {
                break;
            }
        }
        assert_eq!(
            stats.transport(epoch).unwrap().pattern_ticks,
            if epoch == 0 { 3840 } else { 0 },
            "{refusal}"
        );
        assert_eq!(tx.slots(), 4);
        assert!(weak.upgrade().is_some());
        drop(trash.pop().unwrap());
        assert!(weak.upgrade().is_none());
    }
}

/// Scenario "Pattern retirement pressure does not block or free in the callback": omission count and on-time recovery.
#[test]
fn automatic_pressure_omits_hits_without_wait_free_or_late_replay() {
    let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
    garbage
        .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
        .unwrap();
    let rows: Vec<_> = (0..100).map(|tick| [tick, 0, 127, 0]).collect();
    let mut e = Engine::with_sounds(48_000, vec![], vec![vec![0.001; 48_000]]);
    e.handle(0, Command::SetTempo { bpm: 300.0 });
    e.handle(
        0,
        Command::SetPattern {
            pattern: Some(compiled_pattern(1, &rows)),
        },
    );
    e.handle(0, Command::TransportPlay);
    e.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(e, vec![], stats.clone(), 48_000);
    let (mut l, mut r) = ([0.0; 128], [0.0; 128]);
    for _ in 0..20 {
        let before = allocs();
        rt.process(&mut l, &mut r);
        assert_eq!(allocs(), before);
        assert!(l.iter().chain(&r).all(|v| v.is_finite() && v.abs() < 1.0));
    }
    let state = stats.transport(0).unwrap();
    assert!(state.dropped_hits > 0);
    let dropped = state.dropped_hits;
    for _ in 0..100 {
        while trash.pop().is_ok() {}
        let before = allocs();
        rt.process(&mut l, &mut r);
        assert_eq!(allocs(), before);
        assert_eq!(stats.transport(0).unwrap().dropped_hits, dropped);
    }
    // Before the next loop there are no new events. Omitted hits cannot become voices.
    assert!(l.iter().all(|v| *v == 0.001));
    while stats.transport(0).unwrap().ticks < 3700.0 {
        while trash.pop().is_ok() {}
        let before = allocs();
        rt.process(&mut l, &mut r);
        assert_eq!(allocs(), before);
    }
    let at = stats.transport(0).unwrap().ticks;
    assert!(at < 3840.0);
    let mut heard_next_lap = false;
    // One retirement slot cannot hold the references from several hits per block.
    // Drain between single-frame callbacks to remove pressure for the recovery control.
    for _ in 0..12_800 {
        while trash.pop().is_ok() {}
        let before = allocs();
        rt.process(&mut l[..1], &mut r[..1]);
        assert_eq!(allocs(), before);
        if stats.transport(0).unwrap().ticks > 3840.0 && l[0] > 0.0011 {
            heard_next_lap = true;
        }
    }
    assert!(
        heard_next_lap,
        "future on-time events recover after retirement drains"
    );
    assert_eq!(stats.transport(0).unwrap().dropped_hits, dropped);
}

/// Scenario "Pattern retirement pressure does not block or free in the callback": stale work and project transition.
#[test]
fn stale_patterns_and_swaps_retire_outside_callback_during_project_restore() {
    use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
    use std::sync::atomic::Ordering::{Acquire, Release};
    let (mut tx, rx) = rtrb::RingBuffer::new(16);
    let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
    garbage
        .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
        .unwrap();
    let state = Arc::new(ProjectState::default());
    let stats = Arc::new(Stats::default());
    let old = compiled_pattern(1, &[[0, 0, 127, 960]]);
    let stale = compiled_pattern(2, &[[0, 0, 127, 960]]);
    let late = compiled_pattern(3, &[]);
    let target = compiled_pattern(4, &[]);
    let weak = [
        Arc::downgrade(&old),
        Arc::downgrade(&stale),
        Arc::downgrade(&late),
    ];
    let mut e = Engine::with_sounds(48_000, vec![], vec![vec![0.01; 48_000]]);
    e.handle(0, Command::SetPattern { pattern: Some(old) });
    e.handle(0, Command::TransportPlay);
    e.set_garbage(garbage);
    let mut rt = Realtime::with_project(e, vec![rx], stats.clone(), 48_000, state.clone());
    let (mut l, mut r) = ([0.0; 128], [0.0; 128]);
    rt.process(&mut l, &mut r);
    for (epoch, command) in [
        (
            0,
            ProjectMessage::Command(Command::SetPattern {
                pattern: Some(stale),
            }),
        ),
        (
            0,
            ProjectMessage::Command(Command::SwapPatternPads {
                a: 0,
                b: 1,
                pattern: Some(late),
            }),
        ),
        (
            1,
            ProjectMessage::Command(Command::SetPattern {
                pattern: Some(target),
            }),
        ),
        (1, ProjectMessage::Ready),
    ] {
        tx.push(ProjectCommand {
            epoch,
            ready_at_send: false,
            command,
        })
        .unwrap();
    }
    state.epoch.store(1, Release);
    for _ in 0..300 {
        let before = allocs();
        rt.process(&mut l, &mut r);
        assert_eq!(allocs(), before);
    }
    assert_ne!(state.ready.load(Acquire), 1);
    assert!(
        l.iter().all(|v| *v == 0.0),
        "old project is silent despite full retirement"
    );
    assert_eq!(stats.transport(1).unwrap().pattern_ticks, 0);
    assert!(weak.iter().all(|w| w.upgrade().is_some()));
    for _ in 0..1000 {
        while trash.pop().is_ok() {}
        let before = allocs();
        rt.process(&mut l, &mut r);
        assert_eq!(allocs(), before);
        if state.ready.load(Acquire) == 1 {
            break;
        }
    }
    while trash.pop().is_ok() {}
    assert_eq!(state.ready.load(Acquire), 1);
    assert_eq!(stats.transport(1).unwrap().pattern_ticks, 4 * 3840);
    assert!(!stats.transport(1).unwrap().playing);
    assert!(weak.iter().all(|w| w.upgrade().is_none()));
    assert!(l.iter().all(|v| *v == 0.0));
}

/// Scenario "Transport remains allocation free under active control".
#[test]
fn transport_remains_allocation_free_under_active_control() {
    let (mut tx, rx) = rtrb::RingBuffer::new(64);
    let mut rt = Realtime::new(
        Engine::new(48_000),
        vec![rx],
        Arc::new(Stats::default()),
        48_000,
    );
    let (mut l, mut r) = ([0.0; 128], [0.0; 128]);
    let before = allocs();
    for block in 0..2000 {
        for command in [
            Command::SetTempo {
                bpm: 40.0 + (block % 260) as f64,
            },
            Command::SetMetronome {
                enabled: block % 3 != 0,
            },
            Command::TransportPlay,
        ] {
            tx.push(command).unwrap();
        }
        if block % 137 == 0 {
            tx.push(Command::TransportStop).unwrap();
            tx.push(Command::TransportToggle).unwrap();
        }
        rt.process(&mut l, &mut r);
    }
    assert_eq!(allocs(), before);
}

#[test]
fn pattern_capture_and_full_feedback_neither_allocate_nor_free() {
    use engine::capture::{CAPACITY, Phase};
    let (mut tx, rx) = rtrb::RingBuffer::new(256);
    let mut engine = Engine::with_sounds(48_000, vec![], vec![vec![0.001; 48_000]]);
    let mut capture = engine.enable_capture();
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(compiled_pattern(1, &[])),
        },
    );
    let (garbage, mut trash) = rtrb::RingBuffer::new(1024);
    engine.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 48_000);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    tx.push(Command::TransportPlay).unwrap();
    tx.push(Command::PatternRecordStart { pass: 1 }).unwrap();
    for block in 0..20 {
        for pad in 0..64 {
            tx.push(Command::PadPress { pad, velocity: 1.0 }).unwrap();
            tx.push(Command::PadRelease { pad }).unwrap();
        }
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before, "capture block {block}");
        assert!(
            left.iter()
                .chain(&right)
                .all(|v| v.is_finite() && v.abs() < 1.0)
        );
        while trash.pop().is_ok() {}
    }
    tx.push(Command::TransportStop).unwrap();
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    let end = stats.recording(0).unwrap();
    assert_eq!(end.phase, Phase::Finishing);
    assert_eq!(end.sequence, 2560);
    assert_eq!(end.dropped_events, 2560 - CAPACITY as u64);
    assert_eq!(end.dropped_onsets, 1280 - CAPACITY as u64 / 2);
    assert!(end.ticks > 0.0);
    let mut received = 0;
    while capture.pop().is_ok() {
        received += 1;
    }
    assert_eq!(received, CAPACITY);
    tx.push(Command::PatternRecordAck {
        pass: 1,
        sequence: end.sequence,
    })
    .unwrap();
    tx.push(Command::TransportPlay).unwrap();
    tx.push(Command::PatternRecordStart { pass: 2 }).unwrap();
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().phase, Phase::Recording);
    assert_eq!(stats.recording(0).unwrap().ticks, 0.0);
}

#[test]
fn all_gate_slots_limit_refusal_and_lost_releases_neither_allocate_nor_free() {
    use engine::capture::Phase;
    use engine::pattern::{MAX_GATE_TICKS, MAX_HITS};
    let (mut tx, rx) = rtrb::RingBuffer::new(256);
    let mut engine = Engine::with_sounds(48_000, vec![], vec![vec![0.001; 48_000]]);
    let mut capture = engine.enable_capture();
    let recovery = engine.capture_recovery().unwrap();
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(compiled_pattern(1, &[])),
        },
    );
    for pad in 0..engine::PADS {
        engine.handle(
            0,
            Command::SetPadMode {
                pad,
                mode: engine::PadMode::Gate,
            },
        );
    }
    let (garbage, mut trash) = rtrb::RingBuffer::new(1024);
    engine.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 48_000);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    let mut accepted = Vec::with_capacity(MAX_HITS);
    tx.push(Command::TransportPlay).unwrap();
    tx.push(Command::PatternRecordStart { pass: 1 }).unwrap();
    for _ in 0..MAX_HITS / engine::PADS {
        for pad in 0..engine::PADS {
            tx.push(Command::PadPress {
                pad,
                velocity: 0.25,
            })
            .unwrap();
        }
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        assert!(
            left.iter()
                .chain(&right)
                .all(|v| v.is_finite() && v.abs() < 1.0)
        );
        while let Ok(event) = capture.pop() {
            accepted.push(event);
        }
        while trash.pop().is_ok() {}
    }
    assert_eq!(accepted.len(), MAX_HITS);
    assert_eq!(accepted.last().unwrap().gate, Some(MAX_HITS - 1));
    assert_eq!(stats.recording(0).unwrap().reserved_rows, MAX_HITS as u32);
    for _ in 0..4 {
        for _ in 0..128 {
            tx.push(Command::PadRelease { pad: 127 }).unwrap();
        }
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        while trash.pop().is_ok() {}
    }
    tx.push(Command::PadPress {
        pad: 0,
        velocity: 0.5,
    })
    .unwrap();
    tx.push(Command::PadRelease { pad: 1 }).unwrap();
    tx.push(Command::TransportStop).unwrap();
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    let end = stats.recording(0).unwrap();
    assert_eq!(end.phase, Phase::Finishing);
    assert_eq!((end.captured_onsets, end.reserved_rows), (4096, 4096));
    assert_eq!(
        (end.dropped_onsets, end.dropped_events, end.limit_refusals),
        (1, 2, 1)
    );
    for event in &accepted {
        let hit = recovery
            .completed(*event)
            .expect("all accepted gates complete");
        assert!((1..=MAX_GATE_TICKS).contains(&hit.gate_ticks()));
    }
    while capture.pop().is_ok() {}
    while trash.pop().is_ok() {}
    tx.push(Command::PatternRecordAck {
        pass: 1,
        sequence: end.sequence,
    })
    .unwrap();
    tx.push(Command::TransportPlay).unwrap();
    tx.push(Command::PatternRecordStart { pass: 2 }).unwrap();
    tx.push(Command::PadPress {
        pad: 0,
        velocity: 0.5,
    })
    .unwrap();
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    assert_eq!(recovery.completed(accepted[0]), None);
    let newer = capture.pop().unwrap();
    assert_eq!(
        newer.gate, accepted[0].gate,
        "reuse occurs only in another pass"
    );
    assert_eq!(recovery.completed(newer), None, "new gate is still pending");
}

#[test]
fn capture_project_reset_and_stale_commands_do_not_allocate_or_free() {
    use engine::capture::Phase;
    use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
    use std::sync::atomic::Ordering::{Acquire, Release};
    let (mut tx, rx) = rtrb::RingBuffer::new(256);
    let mut engine = Engine::with_sounds(48_000, vec![], vec![]);
    engine.handle(
        0,
        Command::SetPadMode {
            pad: 0,
            mode: engine::PadMode::Gate,
        },
    );
    let mut capture = engine.enable_capture();
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(compiled_pattern(1, &[])),
        },
    );
    let (garbage, mut trash) = rtrb::RingBuffer::new(1024);
    engine.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let project = Arc::new(ProjectState::default());
    let mut rt = Realtime::with_project(engine, vec![rx], stats.clone(), 48_000, project.clone());
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    for command in [
        Command::TransportPlay,
        Command::PatternRecordStart { pass: 1 },
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    ] {
        tx.push(ProjectCommand {
            epoch: 0,
            ready_at_send: true,
            command: ProjectMessage::Command(command),
        })
        .unwrap();
    }
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    for command in [
        Command::PatternRecordEnd { pass: 1 },
        Command::PatternRecordStart { pass: 2 },
        Command::PatternRecordAck {
            pass: 1,
            sequence: 1,
        },
    ] {
        tx.push(ProjectCommand {
            epoch: 0,
            ready_at_send: true,
            command: ProjectMessage::Command(command),
        })
        .unwrap();
    }
    project.epoch.store(1, Release);
    tx.push(ProjectCommand {
        epoch: 1,
        ready_at_send: false,
        command: ProjectMessage::Ready,
    })
    .unwrap();
    for _ in 0..300 {
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        while trash.pop().is_ok() {}
        if project.ready.load(Acquire) == 1 {
            break;
        }
    }
    assert_eq!(project.ready.load(Acquire), 1);
    assert_eq!(stats.recording(1).unwrap().phase, Phase::Idle);
    assert_eq!(capture.pop().unwrap().epoch, 0);
    assert!(capture.pop().is_err());
}

#[test]
fn recorded_candidates_history_and_saturated_retirement_neither_allocate_nor_free() {
    use engine::pattern::{Grid, Hit, MAX_HITS, PlaybackRow, RawPattern, RowId};
    let (mut tx, rx) = rtrb::RingBuffer::new(16);
    let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
    garbage
        .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
        .unwrap();
    let mut engine = Engine::with_sounds(960, vec![], vec![]);
    let mut capture = engine.enable_capture();
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(compiled_pattern(1, &[])),
        },
    );
    engine.handle(0, Command::TransportPlay);
    engine.handle(0, Command::PatternRecordStart { pass: 1 });
    engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let origin = capture.pop().unwrap().origin(3840).unwrap();
    let rows: Vec<_> = (0..MAX_HITS)
        .map(|i| Hit::new(0, 0, if i == 0 { 10 } else { 127 }, 0).unwrap())
        .collect();
    let descriptors: Vec<_> = (0..MAX_HITS)
        .map(|i| {
            let id = RowId::new(i, 1).unwrap();
            if i == 0 {
                PlaybackRow::existing(id)
            } else {
                PlaybackRow::recorded(id, origin, false)
            }
        })
        .collect();
    let compile = |grid| {
        Arc::new(
            RawPattern::new(1, grid, 100, rows.clone())
                .unwrap()
                .compile_recorded(&descriptors)
                .unwrap(),
        )
    };
    let first = compile(Grid::Sixteenth);
    let weak = Arc::downgrade(&first);
    let next = compile(Grid::Quarter);
    tx.push(Command::SetPattern {
        pattern: Some(first),
    })
    .unwrap();
    engine.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 960);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    for _ in 0..4 {
        let before = allocs();
        rt.process(&mut [], &mut []);
        assert_eq!(allocs(), before);
        assert_eq!(stats.pattern_hits(0).unwrap()[0], 0);
    }
    drop(trash.pop().unwrap());
    for _ in 0..64 {
        while trash.pop().is_ok() {}
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
    }
    tx.push(Command::SetPattern {
        pattern: Some(next),
    })
    .unwrap();
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    assert!(
        weak.upgrade().is_some(),
        "compiled metadata must retire outside audio"
    );
    while trash.pop().is_ok() {}
    assert!(weak.upgrade().is_none());
    for command in [
        Command::TransportStop,
        Command::TransportPlay,
        Command::SetPattern { pattern: None },
    ] {
        tx.push(command).unwrap();
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        while trash.pop().is_ok() {}
    }
}

#[test]
fn count_in_full_polyphony_and_retirement_pressure_neither_allocate_nor_free() {
    use engine::capture::Phase;
    let (mut tx, rx) = rtrb::RingBuffer::new(256);
    let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
    garbage
        .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
        .unwrap();
    let sound = Arc::new(engine::Sample::mono(vec![0.001; 192_000]));
    let mut engine = Engine::with_sounds(48_000, engine::click(48_000), vec![]);
    for pad in 0..engine::MAX_VOICES {
        engine.handle(
            0,
            Command::SetPadSound {
                pad,
                sound: Some(sound.clone()),
            },
        );
    }
    engine.handle(
        0,
        Command::SetPadMode {
            pad: 0,
            mode: engine::PadMode::Gate,
        },
    );
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(compiled_pattern(1, &[])),
        },
    );
    let mut capture = engine.enable_capture();
    engine.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 48_000);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    tx.push(Command::SetTempo { bpm: 300.0 }).unwrap();
    tx.push(Command::PatternRecordStart { pass: 1 }).unwrap();
    for pad in 0..engine::MAX_VOICES {
        tx.push(Command::PadPress { pad, velocity: 1.0 }).unwrap();
    }
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().phase, Phase::CountIn);
    for command in [
        Command::PatternRecordEnd { pass: 1 },
        Command::PatternRecordStart { pass: 2 },
        Command::TransportPlay,
        Command::TransportStop,
        Command::PatternRecordStart { pass: 3 },
        Command::PatternRecordStart { pass: 4 },
        Command::PatternRecordEnd { pass: 2 },
    ] {
        tx.push(command).unwrap();
        let before = allocs();
        rt.process(&mut [], &mut []);
        assert_eq!(allocs(), before);
    }
    assert_eq!(
        (
            stats.recording(0).unwrap().phase,
            stats.recording(0).unwrap().pass
        ),
        (Phase::CountIn, 3)
    );
    for block in 0..1000 {
        if block == 100 {
            tx.push(Command::SetTempo { bpm: 240.0 }).unwrap();
        }
        tx.push(Command::SetMetronome {
            enabled: block % 2 == 0,
        })
        .unwrap();
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        assert!(
            left.iter()
                .chain(&right)
                .all(|v| v.is_finite() && v.abs() < 1.0)
        );
        for pad in 0..engine::MAX_VOICES {
            assert!(
                stats.pad_position(&sound, 0, |p| p == pad).is_some(),
                "warm-up pad {pad} survives"
            );
        }
        if stats.recording(0).unwrap().phase == Phase::Recording {
            break;
        }
    }
    assert_eq!(stats.recording(0).unwrap().phase, Phase::Recording);
    assert!(capture.pop().is_err());
    for command in [
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
        Command::TransportStop,
    ] {
        tx.push(command).unwrap();
    }
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().phase, Phase::Finishing);
    assert_eq!(capture.pop().unwrap().pass, 3);
    tx.push(Command::PatternRecordAck {
        pass: 3,
        sequence: 1,
    })
    .unwrap();
    tx.push(Command::PatternRecordStart { pass: 4 }).unwrap();
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().phase, Phase::CountIn);
    drop(trash.pop().unwrap());
}

#[test]
fn count_in_project_reset_with_full_retirement_neither_allocates_nor_frees() {
    use engine::capture::Phase;
    use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
    use std::sync::atomic::Ordering::{Acquire, Release};
    let (mut tx, rx) = rtrb::RingBuffer::new(16);
    let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
    garbage
        .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
        .unwrap();
    let mut engine = Engine::with_sounds(48_000, engine::click(48_000), vec![vec![0.001; 192_000]]);
    let mut capture = engine.enable_capture();
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(compiled_pattern(1, &[])),
        },
    );
    engine.handle(0, Command::PatternRecordStart { pass: 1 });
    engine.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let project = Arc::new(ProjectState::default());
    let mut rt = Realtime::with_project(engine, vec![rx], stats.clone(), 48_000, project.clone());
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().phase, Phase::CountIn);
    project.epoch.store(1, Release);
    for (epoch, command) in [
        (
            0,
            ProjectMessage::Command(Command::PatternRecordStart { pass: 2 }),
        ),
        (0, ProjectMessage::Command(Command::TransportPlay)),
        (1, ProjectMessage::Ready),
    ] {
        tx.push(ProjectCommand {
            epoch,
            ready_at_send: false,
            command,
        })
        .unwrap();
    }
    for _ in 0..800 {
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        assert_eq!(stats.recording(1).unwrap().phase, Phase::Idle);
    }
    assert_eq!(
        project.ready.load(Acquire),
        0,
        "retirement pressure delays restoration, not cancellation"
    );
    for _ in 0..1000 {
        while trash.pop().is_ok() {}
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        if project.ready.load(Acquire) == 1 {
            break;
        }
    }
    assert_eq!(project.ready.load(Acquire), 1);
    assert!(!stats.transport(1).unwrap().playing);
    assert!(capture.pop().is_err());
    assert!(left.iter().chain(&right).all(|&s| s == 0.0));
}

#[test]
fn interrupted_recovery_of_all_terminal_slots_neither_allocates_nor_frees() {
    use engine::capture::{Kind, Phase};
    use engine::pattern::MAX_HITS;
    let (mut tx, rx) = rtrb::RingBuffer::new(256);
    let mut engine = Engine::with_sounds(960, vec![], vec![vec![0.001; 16_000]]);
    engine.handle(0, Command::SetTempo { bpm: 300.0 });
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(compiled_pattern(1, &[])),
        },
    );
    for pad in 0..engine::PADS {
        engine.handle(
            0,
            Command::SetPadMode {
                pad,
                mode: engine::PadMode::Gate,
            },
        );
    }
    let mut events = engine.enable_capture();
    let recovery = engine.capture_recovery().unwrap();
    let (garbage, mut trash) = rtrb::RingBuffer::new(1024);
    engine.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 960);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    let mut accepted = Vec::with_capacity(MAX_HITS);
    tx.push(Command::TransportPlay).unwrap();
    tx.push(Command::PatternRecordStart { pass: 7 }).unwrap();
    for _ in 0..MAX_HITS / engine::PADS {
        for pad in 0..engine::PADS {
            tx.push(Command::PadPress { pad, velocity: 1.0 }).unwrap();
        }
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        while let Ok(event) = events.pop() {
            assert!(matches!(event.kind, Kind::Onset { .. }));
            accepted.push(event);
        }
        while trash.pop().is_ok() {}
    }
    assert_eq!(accepted.len(), MAX_HITS);
    let boundary = stats.recording(0).unwrap();
    assert_eq!(boundary.captured_onsets, MAX_HITS as u32);
    assert_eq!(recovery.completed(*accepted.last().unwrap()), None);
    tx.push(Command::PatternRecordEnd { pass: 7 }).unwrap();
    let before = allocs();
    for _ in 0..3 {
        for &onset in &accepted {
            let row = recovery.interrupted(onset, boundary).unwrap();
            assert_eq!(row.gate_ticks(), 640);
        }
    }
    assert_eq!(
        allocs(),
        before,
        "bounded read-side recovery of all 4096 slots"
    );
    assert_eq!(stats.recording(0).unwrap(), boundary);
    assert_eq!(recovery.completed(*accepted.last().unwrap()), None);
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().phase, Phase::Finishing);
    for command in [
        Command::TransportStop,
        Command::PatternRecordAck {
            pass: 7,
            sequence: boundary.sequence,
        },
        Command::TransportPlay,
        Command::PatternRecordStart { pass: 8 },
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    ] {
        tx.push(command).unwrap();
    }
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    assert_eq!(recovery.interrupted(accepted[0], boundary), None);
    let fresh = events.pop().unwrap();
    assert_eq!(fresh.pass, 8);
    let final_boundary = stats.recording(0).unwrap();
    let before = allocs();
    assert_eq!(
        recovery
            .interrupted(fresh, final_boundary)
            .unwrap()
            .gate_ticks(),
        640
    );
    assert_eq!(allocs(), before);
}

#[test]
fn process_does_not_allocate() {
    let (mut tx, rx) = rtrb::RingBuffer::new(64);
    let mut rt = Realtime::new(
        Engine::new(48_000),
        vec![rx],
        Arc::new(Stats::default()),
        48_000,
    );
    let (mut l, mut r) = (vec![0.0; 128], vec![0.0; 128]);
    for _ in 0..40 {
        tx.push(Command::Click { velocity: 1.0 }).unwrap();
    }

    let before = allocs();
    for _ in 0..1000 {
        rt.process(&mut l, &mut r);
    }
    assert_eq!(allocs() - before, 0);
}

/// Replacing pad samples while they play: neither allocation nor freeing in the callback.
#[test]
fn sample_swaps_do_not_allocate_or_free() {
    let (mut tx, rx) = rtrb::RingBuffer::new(64);
    let (garbage_tx, mut garbage_rx) = rtrb::RingBuffer::new(64);
    let mut engine = Engine::new(48_000);
    engine.set_garbage(garbage_tx);
    let mut rt = Realtime::new(engine, vec![rx], Arc::new(Stats::default()), 48_000);
    let (mut l, mut r) = (vec![0.0; 128], vec![0.0; 128]);
    for i in 0..8 {
        let sound = Arc::new(engine::Sample::mono(vec![0.1; 300 + i]));
        tx.push(Command::SetPadSound {
            pad: 0,
            sound: Some(sound),
        })
        .unwrap();
        tx.push(Command::PadPress {
            pad: 0,
            velocity: 1.0,
        })
        .unwrap();
    }

    let before = allocs();
    for _ in 0..100 {
        rt.process(&mut l, &mut r);
    }
    assert_eq!(allocs() - before, 0);
    // freed here, outside the callback
    while garbage_rx.pop().is_ok() {}
}

#[test]
fn project_switch_saturated_garbage_releases_every_sample_outside_callback() {
    use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
    use std::sync::atomic::Ordering::{Acquire, Release};
    let state = Arc::new(ProjectState::default());
    let (mut tx, rx) = rtrb::RingBuffer::new(512);
    let (garbage, mut trash) = rtrb::RingBuffer::new(1);
    let mut e = Engine::with_sounds(48_000, vec![], vec![vec![0.0; 16]]);
    e.set_garbage(garbage);
    let mut weak = Vec::new();
    for pad in 0..128 {
        let s = Arc::new(engine::Sample::mono(vec![0.1; 4096]));
        weak.push(Arc::downgrade(&s));
        tx.push(ProjectCommand {
            epoch: 0,
            ready_at_send: true,
            command: ProjectMessage::Command(Command::SetPadSound {
                pad,
                sound: Some(s),
            }),
        })
        .unwrap();
    }
    let mut rt = Realtime::with_project(
        e,
        vec![rx],
        Arc::new(Stats::default()),
        48_000,
        state.clone(),
    );
    let (mut l, mut r) = ([0.0; 128], [0.0; 128]);
    for _ in 0..300 {
        rt.process(&mut l, &mut r);
        while trash.pop().is_ok() {}
    }
    state.epoch.store(1, Release);
    tx.push(ProjectCommand {
        epoch: 1,
        ready_at_send: false,
        command: ProjectMessage::Ready,
    })
    .unwrap();
    for _ in 0..300 {
        let before = allocs();
        rt.process(&mut l, &mut r);
        assert_eq!(allocs(), before);
    }
    for _ in 0..2000 {
        while trash.pop().is_ok() {}
        let before = allocs();
        rt.process(&mut l, &mut r);
        assert_eq!(allocs(), before);
        if state.ready.load(Acquire) == 1 {
            break;
        }
    }
    while trash.pop().is_ok() {}
    assert_eq!(state.ready.load(Acquire), 1);
    assert!(weak.iter().all(|w| w.upgrade().is_none()));
}

#[test]
fn full_garbage_holds_commands_and_completed_voices_until_drained() {
    let (mut tx, rx) = rtrb::RingBuffer::new(8);
    let (garbage, mut trash) = rtrb::RingBuffer::new(1);
    let mut engine = Engine::with_sounds(48_000, vec![], vec![vec![0.0; 8]]);
    engine.set_garbage(garbage);
    let sound = Arc::new(engine::Sample::mono(vec![0.5; 8]));
    let weak = Arc::downgrade(&sound);
    tx.push(Command::SetPadSound {
        pad: 0,
        sound: Some(sound),
    })
    .unwrap();
    tx.push(Command::PadPress {
        pad: 0,
        velocity: 1.0,
    })
    .unwrap();
    tx.push(Command::SetPadSound {
        pad: 0,
        sound: None,
    })
    .unwrap();
    let mut rt = Realtime::new(engine, vec![rx], Arc::new(Stats::default()), 48_000);
    let (mut l, mut r) = ([0.0; 128], [0.0; 128]);
    for _ in 0..5 {
        let before = allocs();
        rt.process(&mut l, &mut r);
        assert_eq!(allocs(), before);
    }
    assert!(weak.upgrade().is_some());
    for _ in 0..10 {
        while trash.pop().is_ok() {}
        let before = allocs();
        rt.process(&mut l, &mut r);
        assert_eq!(allocs(), before);
    }
    while trash.pop().is_ok() {}
    assert!(weak.upgrade().is_none());
}

#[test]
fn erasure_full_rows_intervals_voices_and_retirement_neither_allocate_nor_free() {
    use engine::pattern::{Grid, Hit, MAX_HITS, PlaybackRow, RawPattern, RowId};
    let (mut tx, rx) = rtrb::RingBuffer::new(256);
    let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
    garbage
        .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
        .unwrap();
    let sound = Arc::new(engine::Sample::mono(vec![0.0001; 192_000]));
    let mut engine = Engine::with_sounds(48_000, vec![], vec![]);
    for pad in 0..engine::MAX_VOICES {
        engine.handle(
            0,
            Command::SetPadSound {
                pad,
                sound: Some(sound.clone()),
            },
        );
        engine.handle(0, Command::PadPress { pad, velocity: 1.0 });
    }
    let rows: Vec<_> = (0..MAX_HITS)
        .map(|slot| PlaybackRow::existing(RowId::new(slot, 1).unwrap()))
        .collect();
    let raw = RawPattern::new(
        1,
        Grid::Off,
        0,
        vec![Hit::new(0, 127, 127, 0).unwrap(); MAX_HITS],
    )
    .unwrap();
    let pattern = Arc::new(raw.compile_recorded(&rows).unwrap());
    let empty = Arc::new(
        RawPattern::new(1, Grid::Off, 0, vec![])
            .unwrap()
            .compile_recorded(&[])
            .unwrap(),
    );
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(pattern),
        },
    );
    let mut events = engine.enable_capture();
    let recovery = engine.capture_recovery().unwrap();
    engine.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 48_000);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    for command in [
        Command::TransportPlay,
        Command::PatternRecordStart { pass: 1 },
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
        Command::PadPress {
            pad: 127,
            velocity: 1.0,
        },
    ] {
        tx.push(command).unwrap();
    }
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().erased_rows, MAX_HITS as u64);
    tx.push(Command::PadRelease { pad: 127 }).unwrap();
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    for batch in 0..32 {
        let count = (MAX_HITS - 1 - batch * 128).min(128);
        for _ in 0..count {
            tx.push(Command::PadPress {
                pad: 127,
                velocity: 1.0,
            })
            .unwrap();
            tx.push(Command::PadRelease { pad: 127 }).unwrap();
        }
        let before = allocs();
        rt.process(&mut [], &mut []);
        assert_eq!(allocs(), before);
    }
    tx.push(Command::PadPress {
        pad: 127,
        velocity: 1.0,
    })
    .unwrap();
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    let full = stats.recording(0).unwrap();
    assert_eq!(full.erase_intervals, MAX_HITS as u64);
    assert_eq!(full.erase_refusals, 1);
    assert_eq!(full.erase_targets, [0, 0]);
    for pad in 0..engine::MAX_VOICES {
        assert!(
            stats
                .pad_position(&sound, 0, |candidate| candidate == pad)
                .is_some()
        );
    }
    let before = allocs();
    for slot in 0..MAX_HITS {
        assert_eq!(
            recovery.erased(RowId::new(slot, 1).unwrap(), full),
            Some(true)
        );
        assert!(recovery.erase_interval(slot, full).unwrap().complete);
    }
    assert_eq!(
        allocs(),
        before,
        "all retained erasure reads stay bounded and allocation-free"
    );
    tx.push(Command::PatternRecordEnd { pass: 1 }).unwrap();
    tx.push(Command::PublishCapturedPattern {
        pattern: empty,
        pass: 1,
        sequence: full.sequence,
    })
    .unwrap();
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    assert_eq!(
        stats.recording(0).unwrap().phase,
        engine::capture::Phase::Finishing
    );
    drop(trash.pop().unwrap());
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    while trash.pop().is_ok() {}
    for batch in 0..32 {
        for slot in batch * 128..(batch + 1) * 128 {
            let interval = recovery.erase_interval(slot, full).unwrap();
            tx.push(Command::PatternEraseConsumed {
                pass: 1,
                slot,
                sequence: interval.sequence,
            })
            .unwrap();
        }
        let before = allocs();
        rt.process(&mut [], &mut []);
        assert_eq!(allocs(), before);
    }
    while events.pop().is_ok() {}
    let end = stats.recording(0).unwrap();
    tx.push(Command::PatternRecordAck {
        pass: 1,
        sequence: end.sequence,
    })
    .unwrap();
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    assert_eq!(
        stats.recording(0).unwrap().phase,
        engine::capture::Phase::Idle
    );
}

#[test]
fn erasure_does_not_bypass_retirement_for_another_sequenced_pad() {
    use engine::pattern::{Grid, Hit, PlaybackRow, RawPattern, RowId};
    let (mut tx, rx) = rtrb::RingBuffer::new(16);
    let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
    garbage
        .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
        .unwrap();
    let sound = Arc::new(engine::Sample::mono(vec![0.0001; 192_000]));
    let mut engine = Engine::with_sounds(48_000, vec![], vec![]);
    for pad in 1..34 {
        engine.handle(
            0,
            Command::SetPadSound {
                pad,
                sound: Some(sound.clone()),
            },
        );
        if pad >= 2 {
            engine.handle(0, Command::PadPress { pad, velocity: 1.0 });
        }
    }
    for pad in 2..18 {
        engine.handle(0, Command::PadPress { pad, velocity: 1.0 });
    }
    let pattern = RawPattern::new(1, Grid::Off, 0, vec![Hit::new(0, 1, 127, 0).unwrap()])
        .unwrap()
        .compile_recorded(&[PlaybackRow::existing(RowId::new(0, 1).unwrap())])
        .unwrap();
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(Arc::new(pattern)),
        },
    );
    let _events = engine.enable_capture();
    engine.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 48_000);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    for command in [
        Command::TransportPlay,
        Command::PatternRecordStart { pass: 1 },
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    ] {
        tx.push(command).unwrap();
    }
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    assert_eq!(stats.pattern_hits(0).unwrap()[1], 0);
    for pad in 2..34 {
        assert!(
            stats
                .pad_position(&sound, 0, |candidate| candidate == pad)
                .is_some()
        );
    }
    drop(trash.pop().unwrap());
    for _ in 0..750 {
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
    }
    assert!(
        stats.pattern_hits(0).unwrap()[1] > 0,
        "the following occurrence uses normal retirement capacity"
    );
}

#[test]
fn stale_captured_publications_retire_outside_the_callback_and_cannot_block_project_ready() {
    use engine::pattern::{Grid, RawPattern};
    use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
    use std::sync::atomic::Ordering::{Acquire, Release};
    let (mut tx, rx) = rtrb::RingBuffer::new(16);
    let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
    garbage
        .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
        .unwrap();
    let mut engine = Engine::with_sounds(48_000, vec![], vec![]);
    let _events = engine.enable_capture();
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(Arc::new(
                RawPattern::new(1, Grid::Off, 0, vec![])
                    .unwrap()
                    .compile_recorded(&[])
                    .unwrap(),
            )),
        },
    );
    engine.handle(0, Command::TransportPlay);
    engine.handle(0, Command::PatternRecordStart { pass: 1 });
    engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    engine.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    engine.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let project = Arc::new(ProjectState::default());
    let mut rt = Realtime::with_project(engine, vec![rx], stats.clone(), 48_000, project.clone());
    let stale = compiled_pattern(1, &[[0, 0, 127, 0]]);
    let weak = Arc::downgrade(&stale);
    for command in [
        Command::PublishCapturedPattern {
            pattern: stale,
            pass: 1,
            sequence: 0,
        },
        Command::PadPress {
            pad: 1,
            velocity: 1.0,
        },
    ] {
        tx.push(ProjectCommand {
            epoch: 0,
            ready_at_send: true,
            command: ProjectMessage::Command(command),
        })
        .unwrap();
    }
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().erase_targets, [1, 0]);
    assert!(weak.upgrade().is_some());
    drop(trash.pop().unwrap());
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().erase_targets, [3, 0]);
    assert!(
        weak.upgrade().is_some(),
        "the rejected pattern waits in garbage"
    );
    drop(trash.pop().unwrap());
    assert!(weak.upgrade().is_none());
    project.epoch.store(1, Release);
    for (epoch, ready_at_send) in [(0, true), (1, false)] {
        tx.push(ProjectCommand {
            epoch,
            ready_at_send,
            command: ProjectMessage::Command(Command::PublishCapturedPattern {
                pattern: compiled_pattern(1, &[[0, 0, 127, 0]]),
                pass: 1,
                sequence: 1,
            }),
        })
        .unwrap();
    }
    tx.push(ProjectCommand {
        epoch: 1,
        ready_at_send: false,
        command: ProjectMessage::Ready,
    })
    .unwrap();
    for _ in 0..1000 {
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        while trash.pop().is_ok() {}
        if project.ready.load(Acquire) == 1 {
            break;
        }
    }
    assert_eq!(project.ready.load(Acquire), 1);
    assert_eq!(
        stats.recording(1).unwrap().phase,
        engine::capture::Phase::Idle
    );
    assert!(left.iter().chain(&right).all(|sample| *sample == 0.0));
}

#[test]
fn erasure_recycles_gate_and_interval_slots_through_5000_onsets_without_callback_allocation() {
    use engine::pattern::{Grid, RawPattern};
    let (mut tx, rx) = rtrb::RingBuffer::new(32);
    let (garbage, mut trash) = rtrb::RingBuffer::new(1024);
    let mut engine = Engine::with_sounds(960, vec![], vec![vec![0.001; 19_200]]);
    engine.handle(0, Command::SetTempo { bpm: 300.0 });
    engine.handle(
        0,
        Command::SetPadMode {
            pad: 0,
            mode: engine::PadMode::Gate,
        },
    );
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(Arc::new(
                RawPattern::new(1, Grid::Off, 0, vec![])
                    .unwrap()
                    .compile_recorded(&[])
                    .unwrap(),
            )),
        },
    );
    let mut events = engine.enable_capture();
    let recovery = engine.capture_recovery().unwrap();
    engine.set_garbage(garbage);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 960);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    tx.push(Command::TransportPlay).unwrap();
    tx.push(Command::PatternRecordStart { pass: 1 }).unwrap();
    let mut previous = None;
    for turn in 0..5000 {
        tx.push(Command::PadPress {
            pad: 0,
            velocity: 1.0,
        })
        .unwrap();
        tx.push(Command::PadRelease { pad: 0 }).unwrap();
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        let onset = events.pop().unwrap();
        while events.pop().is_ok() {}
        assert_eq!(recovery.completed(onset).unwrap().gate_ticks(), 1);
        if let Some(old) = previous {
            assert_eq!(recovery.completed(old), None);
        }
        previous = Some(onset);
        tx.push(Command::PatternGateConsumed {
            pass: 1,
            slot: onset.gate.unwrap(),
            sequence: onset.sequence,
        })
        .unwrap();
        tx.push(Command::PatternErase {
            pass: 1,
            enabled: true,
        })
        .unwrap();
        tx.push(Command::PatternEraseTarget {
            pass: 1,
            pad: 0,
            one_loop: true,
        })
        .unwrap();
        for _ in 0..6 {
            let before = allocs();
            rt.process(&mut left, &mut right);
            assert_eq!(allocs(), before);
            while trash.pop().is_ok() {}
        }
        tx.push(Command::PatternErase {
            pass: 1,
            enabled: false,
        })
        .unwrap();
        let before = allocs();
        rt.process(&mut [], &mut []);
        assert_eq!(allocs(), before);
        let boundary = stats.recording(0).unwrap();
        let interval = recovery.erase_interval(0, boundary).unwrap();
        assert!(interval.complete);
        // A full accepted lap covers the still-unpublished raw onset.
        tx.push(Command::PatternEraseConsumed {
            pass: 1,
            slot: interval.slot,
            sequence: interval.sequence,
        })
        .unwrap();
        tx.push(Command::PatternRecordRows {
            pass: 1,
            sequence: boundary.sequence,
            captured_onsets: boundary.captured_onsets,
            erased_rows: boundary.erased_rows,
            raw_rows: 0,
        })
        .unwrap();
        let before = allocs();
        rt.process(&mut [], &mut []);
        assert_eq!(allocs(), before);
        assert_eq!(stats.recording(0).unwrap().reserved_rows, 0);
        assert_eq!(stats.recording(0).unwrap().captured_onsets, turn + 1);
    }
    let final_state = stats.recording(0).unwrap();
    assert_eq!(final_state.captured_onsets, 5000);
    assert_eq!(final_state.erase_intervals, 5000);
    assert_eq!(final_state.dropped_onsets, 0);
    assert_eq!(final_state.dropped_events, 0);
}

fn erasure_hold_engine() -> (Engine, rtrb::Consumer<engine::capture::Event>) {
    use engine::pattern::{Grid, Hit, PlaybackRow, RawPattern, RowId};
    let mut engine = Engine::with_sounds(48_000, vec![], vec![]);
    let events = engine.enable_capture();
    let rows = [[0, 0, 90, 0], [240, 0, 70, 0]];
    let pattern = RawPattern::new(
        1,
        Grid::Off,
        0,
        rows.into_iter()
            .map(|[t, p, v, g]| Hit::new(t, p as usize, v as u8, g).unwrap())
            .collect(),
    )
    .unwrap()
    .compile_recorded(&[
        PlaybackRow::existing(RowId::new(0, 1).unwrap()),
        PlaybackRow::existing(RowId::new(1, 1).unwrap()),
    ])
    .unwrap();
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(Arc::new(pattern)),
        },
    );
    engine.handle(0, Command::TransportPlay);
    engine.handle(0, Command::PatternRecordStart { pass: 1 });
    engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
    );
    (engine, events)
}

#[test]
fn erasure_signal_closes_with_full_command_and_retirement_queues_without_allocating_or_freeing() {
    use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
    for early in [false, true] {
        let (mut engine, _events) = erasure_hold_engine();
        let released = Arc::new(AtomicBool::new(early));
        let hold = Arc::new(engine::capture::EraseHold::new(released.clone()));
        let weak = Arc::downgrade(&hold);
        let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
        garbage
            .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
            .unwrap();
        engine.set_garbage(garbage);
        let (mut tx, rx) = rtrb::RingBuffer::new(4);
        let stats = Arc::new(Stats::default());
        let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 48_000);
        tx.push(Command::PatternPadPress {
            pass: 1,
            pad: 0,
            velocity: 1.0,
            hold: hold.clone(),
        })
        .unwrap();
        let before = allocs();
        rt.process(&mut [], &mut []);
        assert_eq!(allocs(), before);
        assert_eq!(
            stats.recording(0).unwrap().erase_targets,
            if early { [0, 0] } else { [1, 0] }
        );
        let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
        for start in (0..6000).step_by(128) {
            let n = (6000 - start).min(128);
            let before = allocs();
            rt.process(&mut left[..n], &mut right[..n]);
            assert_eq!(allocs(), before);
        }
        let rows = stats.recording(0).unwrap().erased_rows;
        assert_eq!(rows, u64::from(!early));
        // The first command cannot retire its old sample. No queue slot remains.
        tx.push(Command::SetPadSound {
            pad: 127,
            sound: None,
        })
        .unwrap();
        tx.push(Command::PatternPadRelease { pad: 0, hold })
            .unwrap();
        for _ in 0..2 {
            tx.push(Command::SetMasterGain { gain: 1.0 }).unwrap();
        }
        assert_eq!(tx.slots(), 0);
        released.store(true, SeqCst);
        let before = allocs();
        rt.process(&mut [], &mut []);
        assert_eq!(allocs(), before);
        assert_eq!(stats.recording(0).unwrap().erase_targets, [0, 0]);
        assert_eq!(
            tx.slots(),
            0,
            "the release does not depend on queue consumption"
        );
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        assert_eq!(
            stats.recording(0).unwrap().erased_rows,
            rows,
            "tick 240 remains outside the interval"
        );
        assert!(weak.upgrade().is_some());
        for _ in 0..8 {
            while trash.pop().is_ok() {}
            let before = allocs();
            rt.process(&mut [], &mut []);
            assert_eq!(allocs(), before);
        }
        while trash.pop().is_ok() {}
        assert_eq!(tx.slots(), 4);
        assert!(
            weak.upgrade().is_none(),
            "the last handle frees outside audio"
        );
    }
}

#[test]
fn old_erasure_generation_cannot_close_a_new_target_while_retirement_is_full() {
    use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
    let (mut engine, _events) = erasure_hold_engine();
    let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
    garbage
        .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
        .unwrap();
    engine.set_garbage(garbage);
    let (mut tx, rx) = rtrb::RingBuffer::new(16);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 48_000);
    let old = Arc::new(AtomicBool::new(false));
    let second = Arc::new(AtomicBool::new(false));
    for released in [old.clone(), Arc::new(AtomicBool::new(false))] {
        tx.push(Command::PatternPadPress {
            pass: 1,
            pad: 0,
            velocity: 1.0,
            hold: Arc::new(engine::capture::EraseHold::new(released)),
        })
        .unwrap();
    }
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    for command in [
        Command::PatternErase {
            pass: 1,
            enabled: false,
        },
        Command::PatternErase {
            pass: 1,
            enabled: true,
        },
        Command::PatternPadPress {
            pass: 1,
            pad: 0,
            velocity: 1.0,
            hold: Arc::new(engine::capture::EraseHold::new(second.clone())),
        },
    ] {
        tx.push(command).unwrap();
    }
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    old.store(true, SeqCst);
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().erase_targets, [1, 0]);
    second.store(true, SeqCst);
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().erase_targets, [0, 0]);
    for _ in 0..6 {
        while trash.pop().is_ok() {}
        let before = allocs();
        rt.process(&mut [], &mut []);
        assert_eq!(allocs(), before);
    }
}

#[test]
fn all_4096_erasure_handles_and_one_refusal_neither_allocate_nor_free() {
    use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
    let (mut engine, _events) = erasure_hold_engine();
    let (garbage, mut trash) = rtrb::RingBuffer::new(8192);
    engine.set_garbage(garbage);
    let (mut tx, rx) = rtrb::RingBuffer::new(256);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 48_000);
    let released: Vec<_> = (0..4096)
        .map(|_| Arc::new(AtomicBool::new(false)))
        .collect();
    let mut weak = Vec::new();
    for batch in released.chunks(128) {
        for release in batch {
            let hold = Arc::new(engine::capture::EraseHold::new(release.clone()));
            weak.push(Arc::downgrade(&hold));
            tx.push(Command::PatternPadPress {
                pass: 1,
                pad: 0,
                velocity: 1.0,
                hold,
            })
            .unwrap();
        }
        let before = allocs();
        rt.process(&mut [], &mut []);
        assert_eq!(allocs(), before);
    }
    assert_eq!(stats.recording(0).unwrap().erase_targets, [1, 0]);
    assert!(weak.iter().all(|hold| hold.upgrade().is_some()));
    tx.push(Command::PatternPadPress {
        pass: 1,
        pad: 127,
        velocity: 1.0,
        hold: Arc::new(engine::capture::EraseHold::new(Arc::new(AtomicBool::new(
            false,
        )))),
    })
    .unwrap();
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().erase_refusals, 1);
    assert_eq!(stats.recording(0).unwrap().erase_targets, [1, 0]);
    released[2048].store(true, SeqCst);
    for _ in 0..2 {
        let before = allocs();
        rt.process(&mut [], &mut []);
        assert_eq!(allocs(), before);
    }
    assert_eq!(
        stats.recording(0).unwrap().erase_targets,
        [0, 0],
        "the first permitted release closes the shared pad target"
    );
    assert!(weak.iter().all(|hold| hold.upgrade().is_some()));
    while trash.pop().is_ok() {}
    assert!(weak.iter().all(|hold| hold.upgrade().is_none()));
}

#[test]
fn stale_or_unready_erasure_handles_retire_outside_the_callback() {
    use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
    use std::sync::atomic::AtomicBool;
    for release in [false, true] {
        for (epoch, ready) in [(9, true), (0, false)] {
            let (mut engine, _events) = erasure_hold_engine();
            let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
            garbage
                .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
                .unwrap();
            engine.set_garbage(garbage);
            let (mut tx, rx) = rtrb::RingBuffer::new(4);
            let stats = Arc::new(Stats::default());
            let mut rt = Realtime::with_project(
                engine,
                vec![rx],
                stats.clone(),
                48_000,
                Arc::new(ProjectState::default()),
            );
            let hold = Arc::new(engine::capture::EraseHold::new(Arc::new(AtomicBool::new(
                false,
            ))));
            let weak = Arc::downgrade(&hold);
            let command = if release {
                Command::PatternPadRelease { pad: 0, hold }
            } else {
                Command::PatternPadPress {
                    pass: 1,
                    pad: 0,
                    velocity: 1.0,
                    hold,
                }
            };
            tx.push(ProjectCommand {
                epoch,
                ready_at_send: ready,
                command: ProjectMessage::Command(command),
            })
            .unwrap();
            let before = allocs();
            rt.process(&mut [], &mut []);
            assert_eq!(allocs(), before);
            assert_eq!(tx.slots(), 3);
            drop(trash.pop().unwrap());
            let before = allocs();
            rt.process(&mut [], &mut []);
            assert_eq!(allocs(), before);
            assert_eq!(tx.slots(), 4);
            assert_eq!(stats.recording(0).unwrap().erase_targets, [0, 0]);
            assert_eq!(stats.recording(0).unwrap().captured_onsets, 0);
            assert!(weak.upgrade().is_some());
            drop(trash.pop().unwrap());
            assert!(weak.upgrade().is_none());
        }
    }
}

#[test]
fn erasure_hold_project_reset_retains_last_references_until_non_audio_retirement() {
    use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
    use std::sync::atomic::{
        AtomicBool,
        Ordering::{Acquire, Release, SeqCst},
    };
    let (mut engine, _events) = erasure_hold_engine();
    let (mut garbage, mut trash) = rtrb::RingBuffer::new(1);
    garbage
        .push(engine::Retired::Sample(Arc::new(engine::Sample::default())))
        .unwrap();
    engine.set_garbage(garbage);
    let (mut tx, rx) = rtrb::RingBuffer::new(4);
    let stats = Arc::new(Stats::default());
    let project = Arc::new(ProjectState::default());
    let mut rt = Realtime::with_project(engine, vec![rx], stats.clone(), 48_000, project.clone());
    let signal = Arc::new(AtomicBool::new(false));
    let hold = Arc::new(engine::capture::EraseHold::new(signal.clone()));
    let weak = Arc::downgrade(&hold);
    tx.push(ProjectCommand {
        epoch: 0,
        ready_at_send: true,
        command: ProjectMessage::Command(Command::PatternPadPress {
            pass: 1,
            pad: 0,
            velocity: 1.0,
            hold,
        }),
    })
    .unwrap();
    let before = allocs();
    rt.process(&mut [], &mut []);
    assert_eq!(allocs(), before);
    assert_eq!(stats.recording(0).unwrap().erase_targets, [1, 0]);
    project.epoch.store(1, Release);
    tx.push(ProjectCommand {
        epoch: 1,
        ready_at_send: false,
        command: ProjectMessage::Ready,
    })
    .unwrap();
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    for _ in 0..128 {
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        assert_eq!(stats.recording(1).unwrap().erase_targets, [0, 0]);
    }
    assert!(
        weak.upgrade().is_some(),
        "reset cannot free the last audio reference"
    );
    signal.store(true, SeqCst);
    for _ in 0..1000 {
        while trash.pop().is_ok() {}
        let before = allocs();
        rt.process(&mut left, &mut right);
        assert_eq!(allocs(), before);
        if project.ready.load(Acquire) == 1 {
            break;
        }
    }
    while trash.pop().is_ok() {}
    assert_eq!(project.ready.load(Acquire), 1);
    assert!(weak.upgrade().is_none());
    assert_eq!(
        stats.recording(1).unwrap().phase,
        engine::capture::Phase::Idle
    );
    assert!(left.iter().chain(&right).all(|sample| *sample == 0.0));
}

#[test]
fn ordinary_pattern_hold_keeps_existing_gate_release_order_without_callback_memory_operations() {
    use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
    let (mut engine, _events) = erasure_hold_engine();
    engine.handle(
        0,
        Command::PatternErase {
            pass: 1,
            enabled: false,
        },
    );
    engine.handle(
        0,
        Command::SetPattern {
            pattern: Some(compiled_pattern(1, &[])),
        },
    );
    engine.handle(
        0,
        Command::SetPadSound {
            pad: 0,
            sound: Some(Arc::new(engine::Sample::mono(vec![0.03; 48000]))),
        },
    );
    engine.handle(
        0,
        Command::SetPadMode {
            pad: 0,
            mode: engine::PadMode::Gate,
        },
    );
    let (garbage, mut trash) = rtrb::RingBuffer::new(1);
    engine.set_garbage(garbage);
    let (mut tx, rx) = rtrb::RingBuffer::new(4);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine, vec![rx], stats.clone(), 48000);
    let released = Arc::new(AtomicBool::new(false));
    let hold = Arc::new(engine::capture::EraseHold::new(released.clone()));
    let weak = Arc::downgrade(&hold);
    tx.push(Command::PatternPadPress {
        pass: 1,
        pad: 0,
        velocity: 1.0,
        hold: hold.clone(),
    })
    .unwrap();
    let (mut left, mut right) = ([0.0; 512], [0.0; 512]);
    let before = allocs();
    rt.process(&mut left[..128], &mut right[..128]);
    assert_eq!(allocs(), before);
    assert!(left[..128].iter().all(|sample| *sample == 0.03));
    released.store(true, SeqCst);
    let before = allocs();
    rt.process(&mut left[..128], &mut right[..128]);
    assert_eq!(allocs(), before);
    assert!(
        left[..128].iter().all(|sample| *sample == 0.03),
        "ordinary sound retains its existing UI release route"
    );
    tx.push(Command::PatternPadRelease { pad: 0, hold })
        .unwrap();
    drop(trash.pop().unwrap());
    let before = allocs();
    rt.process(&mut left, &mut right);
    assert_eq!(allocs(), before);
    assert!(
        left[240..]
            .iter()
            .chain(&right[240..])
            .all(|sample| *sample == 0.0)
    );
    assert_eq!(stats.recording(0).unwrap().captured_onsets, 1);
    assert!(weak.upgrade().is_some());
    drop(trash.pop().unwrap());
    assert!(weak.upgrade().is_none());
}

/// The drive, the sends and both slots, a change of kind among them (56): the callback
/// neither allocates nor frees.
#[test]
fn effects_do_not_allocate() {
    use engine::fx::FxSettings;
    let (mut tx, rx) = rtrb::RingBuffer::new(64);
    let mut rt = Realtime::new(
        Engine::new(48_000),
        vec![rx],
        Arc::new(Stats::default()),
        48_000,
    );
    let (mut l, mut r) = (vec![0.0; 128], vec![0.0; 128]);
    let commands = [
        Command::SetPadDrive { pad: 0, drive: 0.7 },
        Command::SetPadSends {
            pad: 0,
            sends: [0.5, 0.5],
        },
        Command::SetFx {
            slot: 0,
            settings: FxSettings::Reverb {
                size: 0.8,
                tone: 0.3,
            },
        },
        Command::SetFx {
            slot: 1,
            settings: FxSettings::Delay {
                time_ms: 300.0,
                feedback: 0.6,
            },
        },
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    ];
    for c in commands {
        tx.push(c).unwrap();
    }
    let before = allocs();
    for i in 0..1000 {
        if i == 500 {
            // the delay slot turns to a reverb while it echoes
            tx.push(Command::SetFx {
                slot: 1,
                settings: FxSettings::Reverb {
                    size: 0.5,
                    tone: 0.5,
                },
            })
            .unwrap();
        }
        rt.process(&mut l, &mut r);
    }
    assert_eq!(allocs() - before, 0);
}

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

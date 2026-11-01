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

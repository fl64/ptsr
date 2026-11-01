//! Output meters: the engine reports the peaks of exactly what it played.

use std::sync::Arc;

use engine::offline::{Event, Scenario, render};
use engine::rt::Stats;
use engine::{Command, Engine, checks};

#[test]
fn output_peaks_keep_the_max_and_the_sign_off() {
    let stats = Stats::default();
    stats.record_output(0.25, -0.5);
    stats.record_output(-0.75, 0.1);
    assert_eq!(stats.take_output(), (0.75, 0.5));
    // taking resets: nothing was played since
    assert_eq!(stats.take_output(), (0.0, 0.0));
}

#[test]
fn reported_peaks_equal_the_rendered_sound() {
    let stats = Arc::new(Stats::default());
    let mut engine = Engine::new(48_000);
    engine.set_stats(stats.clone());
    let scenario = Scenario {
        sample_rate: 48_000,
        frames: 48_000,
        events: vec![Event {
            frame: 0,
            cmd: Command::Click { velocity: 1.0 },
        }],
    };
    let (l, r) = render(&mut engine, &scenario, 128);
    // the meters read the same buffers that go into the WAV
    let (sl, sr) = stats.take_output();
    assert_eq!(sl, checks::peak(&l));
    assert_eq!(sr, checks::peak(&r));
}

#[test]
fn realtime_wires_the_stats_into_the_engine() {
    // a Realtime built with a Stats reports its output without extra wiring
    let (_tx, rx) = rtrb::RingBuffer::new(16);
    let stats = Arc::new(Stats::default());
    let mut rt = engine::rt::Realtime::new(Engine::new(48_000), vec![rx], stats.clone(), 48_000);
    let (mut l, mut r) = (vec![0.0; 128], vec![0.0; 128]);
    rt.process(&mut l, &mut r);
    // the built-in click is silent until a command arrives, but silence is recorded too
    assert_eq!(stats.take_output(), (0.0, 0.0));
}

#[test]
fn history_keeps_blocks_oldest_first() {
    let stats = Stats::default();
    for i in 0..8u32 {
        stats.record_output(i as f32 / 10.0, 0.0);
    }
    assert_eq!(
        stats.output_history(4),
        vec![0.4, 0.5, 0.6, 0.7],
        "the newest values come last"
    );
    assert_eq!(stats.output_history(usize::MAX).len(), 8);
    // the ring wraps: after a full lap the oldest values are gone
    for i in 8u32..(8 + 1024) {
        stats.record_output(i as f32 / 10.0, 0.0);
    }
    let all = stats.output_history(usize::MAX);
    assert_eq!(all.len(), 1024);
    assert_eq!(all.last(), Some(&((8 + 1024 - 1) as f32 / 10.0)));
    assert_eq!(all.first(), Some(&0.8), "only the last HIST values survive");
}

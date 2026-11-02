//! Samples on pads: stereo, replacing a sample while it plays, and memory handed back
//! instead of being freed in the audio thread.

use std::sync::Arc;

use engine::{Command, Engine, Region, Sample};

const SR: u32 = 48_000;

fn stereo(left: f32, right: f32, len: usize) -> Arc<Sample> {
    Arc::new(Sample {
        left: vec![left; len].into(),
        right: Some(vec![right; len].into()),
    })
}

fn block(e: &mut Engine, n: usize) -> (Vec<f32>, Vec<f32>) {
    let (mut l, mut r) = (vec![0.0; n], vec![0.0; n]);
    e.process(&mut l, &mut r);
    (l, r)
}

#[test]
fn stereo_sample_keeps_its_sides() {
    let mut e = Engine::new(SR);
    e.handle(
        0,
        Command::SetPadSound {
            pad: 3,
            sound: Some(stereo(0.5, -0.25, 1000)),
        },
    );
    e.handle(
        0,
        Command::PadPress {
            pad: 3,
            velocity: 1.0,
        },
    );
    let (l, r) = block(&mut e, 64);
    assert_eq!((l[10], r[10]), (0.5, -0.25));
}

#[test]
fn mono_sample_plays_on_both_sides() {
    let mut e = Engine::new(SR);
    e.handle(
        0,
        Command::SetPadSound {
            pad: 0,
            sound: Some(Arc::new(Sample::mono(vec![0.3; 100]))),
        },
    );
    e.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let (l, r) = block(&mut e, 32);
    assert_eq!((l[5], r[5]), (0.3, 0.3));
}

/// Scenario "New sample on a sounding pad" (sample-loading).
#[test]
fn replacing_a_sample_keeps_the_playing_voice() {
    let mut e = Engine::new(SR);
    e.handle(
        0,
        Command::SetPadSound {
            pad: 1,
            sound: Some(stereo(0.1, 0.1, 10_000)),
        },
    );
    e.handle(
        0,
        Command::PadPress {
            pad: 1,
            velocity: 1.0,
        },
    );
    block(&mut e, 64);
    e.handle(
        0,
        Command::SetPadSound {
            pad: 1,
            sound: Some(stereo(0.7, 0.7, 10_000)),
        },
    );
    let (l, _) = block(&mut e, 64);
    assert!((l[0] - 0.1).abs() < 1e-6, "the old voice was cut: {}", l[0]);
    // the next hit plays the new sample (the old voice fades out over 5 ms)
    e.handle(
        0,
        Command::PadPress {
            pad: 1,
            velocity: 1.0,
        },
    );
    let (l, _) = block(&mut e, 512);
    assert!((l[500] - 0.7).abs() < 1e-6, "{}", l[500]);
}

#[test]
fn empty_pad_is_silent() {
    let mut e = Engine::new(SR);
    e.handle(
        0,
        Command::SetPadSound {
            pad: 2,
            sound: None,
        },
    );
    e.handle(
        0,
        Command::PadPress {
            pad: 2,
            velocity: 1.0,
        },
    );
    let (l, r) = block(&mut e, 256);
    assert!(l.iter().chain(&r).all(|s| *s == 0.0));
}

#[test]
fn released_samples_go_to_the_garbage_queue() {
    let (tx, mut rx) = rtrb::RingBuffer::new(8);
    let mut e = Engine::new(SR);
    e.set_garbage(tx);
    let old = stereo(0.2, 0.2, 100);
    let weak = Arc::downgrade(&old);
    e.handle(
        0,
        Command::SetPadSound {
            pad: 4,
            sound: Some(old),
        },
    );
    e.handle(
        0,
        Command::PadPress {
            pad: 4,
            velocity: 1.0,
        },
    );
    // the pad gets a new sample while the voice still plays the old one
    e.handle(
        0,
        Command::SetPadSound {
            pad: 4,
            sound: Some(stereo(0.9, 0.9, 100)),
        },
    );
    block(&mut e, 256);
    // the engine let go of the old sample, but it is not freed: it waits in the queue
    let mut back = Vec::new();
    while let Ok(s) = rx.pop() {
        back.push(s);
    }
    assert!(weak.upgrade().is_some(), "freed inside the engine");
    assert!(back.iter().any(|s| s.left[0] == 0.2));
    drop(back);
    assert!(weak.upgrade().is_none());
}

#[test]
fn full_garbage_queue_blocks_sample_replacement() {
    let (tx, mut rx) = rtrb::RingBuffer::new(1);
    let mut e = Engine::new(SR);
    e.set_garbage(tx);
    let first = stereo(0.2, 0.0, 10);
    let weak = Arc::downgrade(&first);
    e.handle(
        0,
        Command::SetPadSound {
            pad: 5,
            sound: Some(first),
        },
    );
    let next = Command::SetPadSound {
        pad: 5,
        sound: Some(stereo(0.3, 0.0, 10)),
    };
    assert!(!e.can_handle(&next));
    drop(rx.pop().unwrap());
    assert!(e.can_handle(&next));
    e.handle(0, next);
    assert!(weak.upgrade().is_some());
    drop(rx.pop().unwrap());
    assert!(weak.upgrade().is_none());
}

#[test]
fn playing_sounds_stop_at_a_project_switch_during_long_release() {
    let mut e = Engine::new(SR);
    e.handle(
        0,
        Command::SetPadSound {
            pad: 0,
            sound: Some(stereo(0.5, 0.5, SR as usize * 3)),
        },
    );
    e.handle(
        0,
        Command::SetPadMode {
            pad: 0,
            mode: engine::PadMode::Gate,
        },
    );
    e.handle(
        0,
        Command::SetPadEnvelope {
            pad: 0,
            attack_ms: 0,
            decay_ms: 0,
            release_ms: 2000,
        },
    );
    e.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    block(&mut e, 512);
    e.handle(0, Command::PadRelease { pad: 0 });
    let (before, _) = block(&mut e, 4800);
    e.switch_project();
    let (after, _) = block(&mut e, 512);
    assert!((before[before.len() - 1] - after[0]).abs() < 0.001);
    assert!(after[240..].iter().all(|s| *s == 0.0));
}

#[test]
fn preview_switches_files_without_a_click_and_stops() {
    let mut e = Engine::new(SR);
    e.handle(
        0,
        Command::Preview {
            sound: Some(stereo(0.4, 0.4, 48_000)),
            region: Region::WHOLE,
        },
    );
    let (l, _) = block(&mut e, 256);
    assert!((l[100] - 0.4).abs() < 1e-6);
    // the next file: the old one fades out over 5 ms while the new one starts
    e.handle(
        0,
        Command::Preview {
            sound: Some(stereo(0.2, 0.2, 48_000)),
            region: Region::WHOLE,
        },
    );
    let (l, _) = block(&mut e, 512);
    assert!(
        engine::checks::max_step(&l) < 0.25,
        "jump {}",
        engine::checks::max_step(&l)
    );
    assert!((l[400] - 0.2).abs() < 1e-6, "{}", l[400]);
    // pads are not touched by the preview
    e.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    e.handle(
        0,
        Command::Preview {
            sound: None,
            region: Region::WHOLE,
        },
    );
    let (l, _) = block(&mut e, 512);
    assert!(l[400].abs() > 0.0, "a pad sounds on");
    e.handle(0, Command::PadRelease { pad: 0 });
    let mut e2 = Engine::new(SR);
    e2.handle(
        0,
        Command::Preview {
            sound: Some(stereo(0.4, 0.4, 48_000)),
            region: Region::WHOLE,
        },
    );
    block(&mut e2, 64);
    e2.handle(
        0,
        Command::Preview {
            sound: None,
            region: Region::WHOLE,
        },
    );
    let (l, _) = block(&mut e2, 512);
    assert_eq!(l[300], 0.0, "the preview stops");
}

/// Scenario "Quieter while playing" and "Knob with soft takeover" (master-volume): the gain glides.
#[test]
fn master_gain_changes_smoothly() {
    let mut e = Engine::new(SR);
    e.handle(
        0,
        Command::SetPadSound {
            pad: 0,
            sound: Some(stereo(0.5, 0.5, 48_000)),
        },
    );
    e.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let (before, _) = block(&mut e, 128);
    e.handle(0, Command::SetMasterGain { gain: 0.1 });
    let (ramp, _) = block(&mut e, 128);
    let (after, _) = block(&mut e, 128);
    assert!((before[64] - 0.5).abs() < 1e-6);
    assert!((after[64] - 0.05).abs() < 1e-6);
    // no jump inside the block that changes the gain, and none at its edges
    assert!(engine::checks::max_step(&[&before[..], &ramp[..], &after[..]].concat()) < 0.01);
}

#[test]
fn swapped_pads_trade_samples_without_a_click() {
    let mut e = Engine::new(SR);
    e.handle(
        0,
        Command::SetPadSound {
            pad: 0,
            sound: Some(stereo(0.5, 0.5, 48_000)),
        },
    );
    e.handle(
        0,
        Command::SetPadSound {
            pad: 1,
            sound: Some(stereo(0.2, 0.2, 48_000)),
        },
    );
    e.handle(0, Command::SetPadGain { pad: 1, gain: 0.5 });
    e.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let (before, _) = block(&mut e, 128);
    e.handle(0, Command::SwapPads { a: 0, b: 1 });
    let (fade, _) = block(&mut e, 512);
    assert!(
        engine::checks::max_step(&[&before[..], &fade[..]].concat()) < 0.01,
        "click on swap"
    );
    // pad 0 now plays pad 1's sample with pad 1's volume, and the other way round
    e.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    let (l, _) = block(&mut e, 64);
    assert!((l[32] - 0.1).abs() < 1e-6, "{}", l[32]);
    e.handle(0, Command::PadRelease { pad: 0 });
    let mut e2 = Engine::new(SR);
    e2.handle(
        0,
        Command::SetPadSound {
            pad: 0,
            sound: Some(stereo(0.5, 0.5, 48_000)),
        },
    );
    e2.handle(0, Command::SwapPads { a: 0, b: 1 });
    e2.handle(
        0,
        Command::PadPress {
            pad: 1,
            velocity: 1.0,
        },
    );
    let (l, _) = block(&mut e2, 64);
    assert!((l[32] - 0.5).abs() < 1e-6, "{}", l[32]);
}

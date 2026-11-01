//! Change 24: the chop screen opens in silence — the pad voices fade without a click,
//! the preview voice is not touched, and a pad hit after the hush sounds again.

use std::path::Path;
use std::sync::Arc;

use engine::checks::{first_loud, first_non_finite, max_step, peak};
use engine::offline::{Event, Scenario, render, write_wav};
use engine::{Command, Engine, Region, Sample};

const SR: u32 = 48_000;
const MS: usize = SR as usize / 1000;
/// The pad test tone (110 Hz, amplitude 0.3) rises by at most 0.004 per sample; a
/// 5 ms fade of that tone moves 0.00125 per sample. Anything steeper is a click.
const NO_CLICK: f32 = 0.01;

fn at(frame: usize, cmd: Command) -> Event {
    Event { frame, cmd }
}

fn press(pad: usize) -> Command {
    Command::PadPress { pad, velocity: 1.0 }
}

/// A sine that starts at zero, so its own start is not a step.
fn sine(hz: f32, amp: f32, frames: usize) -> Arc<Sample> {
    Arc::new(Sample::mono(
        (0..frames)
            .map(|i| (i as f32 * hz * std::f32::consts::TAU / SR as f32).sin() * amp)
            .collect(),
    ))
}

/// Scenario "A long pad fades on entry" (24): a one-shot pad plays a long sample and
/// the chop entry hushes it. The fade is click-free, stays inside the range and ends
/// in silence.
#[test]
fn a_long_pad_fades_on_entry() {
    let hush = 300 * MS;
    let sc = Scenario {
        sample_rate: SR,
        frames: 500 * MS,
        events: vec![at(0, press(0)), at(hush, Command::HushPads)],
    };
    let (left, right) = render(&mut Engine::new(SR), &sc, 128);
    assert!(first_non_finite(&left).is_none() && first_non_finite(&right).is_none());
    assert!(peak(&left) <= 1.0 && peak(&right) <= 1.0, "no clipping");
    assert!(peak(&left[100 * MS..hush]) > 0.05, "the pad sounds first");
    assert!(max_step(&left) < NO_CLICK, "jump {}", max_step(&left));
    // the fade lasts 5 ms: after it the pad voice is silent
    assert_eq!(first_loud(&left, hush + 20 * MS..left.len(), 0.0), None);
}

/// Scenario "Slices still play after the hush" (24): the chop screen opened with the
/// pads hushed, then a pad hit sounds its slice as usual.
#[test]
fn slices_still_play_after_the_hush() {
    let sc = Scenario {
        sample_rate: SR,
        frames: 500 * MS,
        events: vec![
            at(0, press(0)),
            at(200 * MS, Command::HushPads),
            at(300 * MS, press(1)),
        ],
    };
    let (left, right) = render(&mut Engine::new(SR), &sc, 128);
    assert!(first_non_finite(&left).is_none() && first_non_finite(&right).is_none());
    assert!(peak(&left) <= 1.0 && peak(&right) <= 1.0, "no clipping");
    assert!(max_step(&left) < NO_CLICK, "jump {}", max_step(&left));
    assert!(
        peak(&left[350 * MS..]) > 0.05,
        "the slice plays after the hush"
    );
}

/// The chop screen's own sample playback goes through the preview voice: the hush
/// fades the pads and leaves it alone. The preview is quiet, so the summed slope of
/// both tones stays under `NO_CLICK`.
#[test]
fn the_chop_playback_is_not_touched() {
    let preview = sine(220.0, 0.1, SR as usize);
    let sc = Scenario {
        sample_rate: SR,
        frames: 500 * MS,
        events: vec![
            at(0, press(0)),
            at(
                0,
                Command::Preview {
                    sound: Some(preview),
                    region: Region::WHOLE,
                },
            ),
            at(300 * MS, Command::HushPads),
        ],
    };
    let (left, _) = render(&mut Engine::new(SR), &sc, 128);
    assert!(max_step(&left) < NO_CLICK, "jump {}", max_step(&left));
    // after the fade only the preview is left, at its own level
    let rest = peak(&left[330 * MS..]);
    assert!(
        (rest - 0.1).abs() < 0.02,
        "the preview survived the hush: {rest}"
    );
}

/// The render for the owner: a pad tone, the chop entry fades it out, the slice keeps
/// playing, and a new pad hit sounds.
#[test]
fn render_for_the_owner() {
    let slice = sine(220.0, 0.25, SR as usize);
    let sc = Scenario {
        sample_rate: SR,
        frames: 1500 * MS,
        events: vec![
            at(0, press(0)),
            at(
                500 * MS,
                Command::Preview {
                    sound: Some(slice),
                    region: Region::WHOLE,
                },
            ),
            at(500 * MS, Command::HushPads),
            at(800 * MS, press(4)),
        ],
    };
    let (left, right) = render(&mut Engine::new(SR), &sc, 128);
    write_wav(
        Path::new("../../renders/24-chop-hush.wav"),
        SR,
        &left,
        &right,
    )
    .unwrap();
}

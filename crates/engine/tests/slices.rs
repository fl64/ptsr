//! Slices: a pad plays only its region of a shared sample, ends it without a click, and a
//! region change does not touch a voice that already plays.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

use engine::checks::{first_loud, first_non_finite, max_step, peak};
use engine::offline::{Event, Scenario, render, write_wav};
use engine::{Command, Engine, Region, Sample};

const SR: u32 = 48_000;
const MS: usize = SR as usize / 1000;

/// A ramp: each frame holds its own index scaled down, so the output shows which frames played.
fn ramp(len: usize) -> Arc<Sample> {
    Arc::new(Sample::mono((0..len).map(|i| i as f32 * 1e-5).collect()))
}

fn block(e: &mut Engine, n: usize) -> Vec<f32> {
    let (mut l, mut r) = (vec![0.0; n], vec![0.0; n]);
    e.process(&mut l, &mut r);
    l
}

fn put(e: &mut Engine, pad: usize, sound: &Arc<Sample>, start: usize, end: usize) {
    e.handle(
        0,
        Command::SetPadSound {
            pad,
            sound: Some(sound.clone()),
        },
    );
    e.handle(
        0,
        Command::SetPadRegion {
            pad,
            region: Region { start, end },
        },
    );
}

fn press(pad: usize) -> Command {
    Command::PadPress { pad, velocity: 1.0 }
}

#[test]
fn pad_plays_its_region_only() {
    let mut e = Engine::new(SR);
    let s = ramp(10_000);
    put(&mut e, 0, &s, 2_000, 3_000);
    e.handle(0, press(0));
    let out = block(&mut e, 2_000);
    assert_eq!(out[0], s.left[2_000]);
    assert_eq!(out[500], s.left[2_500]);
    assert_eq!(
        first_loud(&out, 1_000..2_000, 0.0),
        None,
        "plays past the end"
    );
}

#[test]
fn slice_end_fades_without_a_click() {
    let mut e = Engine::new(SR);
    let s = Arc::new(Sample::mono(vec![0.5; SR as usize]));
    put(&mut e, 0, &s, 0, 100 * MS);
    e.handle(0, press(0));
    let out = block(&mut e, 200 * MS);
    assert!(max_step(&out) < 0.01, "jump {}", max_step(&out));
    assert!(
        (out[50 * MS] - 0.5).abs() < 1e-6,
        "the slice body is untouched"
    );
    assert_eq!(first_loud(&out, 100 * MS..200 * MS, 0.0), None);
}

#[test]
fn region_change_leaves_the_playing_voice_alone() {
    let mut e = Engine::new(SR);
    let s = ramp(10_000);
    put(&mut e, 0, &s, 0, 4_000);
    e.handle(0, press(0));
    block(&mut e, 1_000);
    e.handle(
        0,
        Command::SetPadRegion {
            pad: 0,
            region: Region {
                start: 6_000,
                end: 7_000,
            },
        },
    );
    let out = block(&mut e, 1_000);
    assert_eq!(out[0], s.left[1_000], "the voice jumped to the new region");
    e.handle(0, press(0));
    let out = block(&mut e, 1_000);
    // the old voice fades out over 5 ms while the new one starts at 6000
    assert!((out[500] - s.left[6_500]).abs() < 1e-6, "{}", out[500]);
}

/// Scenario "Slice playing on a pad is replaced" (chopping): a plan puts another slice on a
/// sounding pad, as the apply sends it (the sound, then the region); the playing sound goes
/// on without a click, the next hit plays the new slice.
#[test]
fn slice_playing_on_a_pad_is_replaced() {
    let mut e = Engine::new(SR);
    let s = Arc::new(Sample::mono(
        (0..10_000)
            .map(|i| if i < 5_000 { 0.5 } else { -0.5 })
            .collect(),
    ));
    put(&mut e, 0, &s, 0, 4_000);
    e.handle(0, press(0));
    let mut out = block(&mut e, 1_000);
    put(&mut e, 0, &s, 6_000, 7_000);
    out.extend(block(&mut e, 1_000));
    assert_eq!(first_non_finite(&out), None);
    assert!(peak(&out) <= 1.0);
    assert!(max_step(&out) < 0.01, "a click: {}", max_step(&out));
    assert!((out[1_500] - 0.5).abs() < 1e-6, "the old sound plays on");
    e.handle(0, press(0));
    let out = block(&mut e, 1_000);
    assert!((out[500] + 0.5).abs() < 1e-6, "the new slice: {}", out[500]);
}

#[test]
fn a_new_sample_plays_whole_again() {
    let mut e = Engine::new(SR);
    let s = ramp(10_000);
    put(&mut e, 0, &s, 5_000, 6_000);
    e.handle(
        0,
        Command::SetPadSound {
            pad: 0,
            sound: Some(s.clone()),
        },
    );
    e.handle(0, press(0));
    let out = block(&mut e, 128);
    assert_eq!(out[10], s.left[10]);
}

#[test]
fn swapped_pads_keep_their_regions() {
    let mut e = Engine::new(SR);
    let s = ramp(10_000);
    put(&mut e, 0, &s, 1_000, 2_000);
    put(&mut e, 1, &s, 8_000, 9_000);
    e.handle(0, Command::SwapPads { a: 0, b: 1 });
    e.handle(0, press(0));
    assert_eq!(block(&mut e, 128)[10], s.left[8_010]);
}

#[test]
fn preview_plays_a_region_and_reports_its_position() {
    let mut e = Engine::new(SR);
    let at = Arc::new(AtomicUsize::new(0));
    e.set_preview_pos(at.clone());
    assert_eq!(at.load(Relaxed), usize::MAX);
    let s = ramp(10_000);
    e.handle(
        0,
        Command::Preview {
            sound: Some(s.clone()),
            region: Region {
                start: 3_000,
                end: 3_500,
            },
        },
    );
    let out = block(&mut e, 256);
    assert_eq!(out[0], s.left[3_000]);
    assert_eq!(at.load(Relaxed), 3_256);
    block(&mut e, 512);
    assert_eq!(at.load(Relaxed), usize::MAX, "the slice ended");
}

/// A two-bar break of eight hits (kick, hat, snare, hat ...), each with a sharp attack.
fn break_loop(beat: usize) -> Vec<f32> {
    let mut seed = 7u32;
    let mut noise = move || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) as f32 / (1 << 24) as f32 * 2.0 - 1.0
    };
    let mut out = vec![0.0; 8 * beat];
    for hit in 0..8 {
        for i in 0..beat {
            let t = i as f32 / SR as f32;
            let x = match hit % 4 {
                0 => {
                    let env = (-t * 12.0).exp();
                    0.7 * env
                        * (std::f32::consts::TAU * (50.0 + 90.0 * (-t * 35.0).exp()) * t).sin()
                }
                2 => 0.5 * (-t * 18.0).exp() * noise(),
                _ => 0.25 * (-t * 60.0).exp() * noise(),
            };
            // a 1 ms attack, as a recorded drum has
            out[hit * beat + i] = x * (i as f32 / MS as f32).min(1.0);
        }
    }
    out
}

/// Render for the owner: one break chopped into 8 slices on pads 1-8, played as a new beat.
#[test]
fn render_chopped_beat() {
    // 92 BPM, eighth notes
    let beat = SR as usize * 60 / 92 / 2;
    let source = Arc::new(Sample::mono(break_loop(beat)));
    let mut e = Engine::new(SR);
    for slice in 0..8 {
        put(&mut e, slice, &source, slice * beat, (slice + 1) * beat);
    }
    // the original order first, then a new pattern from the same slices
    let pattern = [0, 1, 2, 3, 4, 5, 6, 7, 0, 0, 2, 1, 0, 3, 2, 7];
    let events: Vec<Event> = pattern
        .iter()
        .enumerate()
        .map(|(step, &pad)| Event {
            frame: step * beat,
            cmd: press(pad),
        })
        .collect();
    let sc = Scenario {
        sample_rate: SR,
        frames: 17 * beat,
        events,
    };
    let (l, r) = render(&mut e, &sc, 128);
    assert_eq!(first_non_finite(&l), None);
    assert!(peak(&l) <= 0.95, "clipping");
    // a slice is one beat long, so the previous one has just ended and each step starts with
    // exactly the first frames of its slice; a repeated pad fades its own voice, so it is skipped
    for (step, &pad) in pattern.iter().enumerate() {
        if step > 0 && pattern[step - 1] == pad {
            continue;
        }
        for i in 0..MS {
            let (got, want) = (l[step * beat + i], source.left[pad * beat + i]);
            assert!(
                (got - want).abs() < 1e-6,
                "step {step} frame {i}: {got} vs {want}"
            );
        }
    }
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/04-chopped-beat.wav");
    write_wav(&out, SR, &l, &r).unwrap();
}

/// Render for the owner: two neighbouring slices on pads 1 and 2; halfway through the cut
/// between them moves 20 ms later, as when it is fixed after the layout. Both pads follow at
/// the next hit, the voices that already play are not touched.
#[test]
fn render_moved_cut() {
    let beat = SR as usize * 60 / 92 / 2;
    let source = Arc::new(Sample::mono(break_loop(beat)));
    let mut e = Engine::new(SR);
    put(&mut e, 0, &source, 0, beat);
    put(&mut e, 1, &source, beat, 2 * beat);
    let shift = 20 * MS;
    let mut events: Vec<Event> = (0..16)
        .map(|step| Event {
            frame: step * beat,
            cmd: press(step % 2),
        })
        .collect();
    // between steps 7 and 8, the cut moves: pad 1 ends later, pad 2 starts later
    let moved = 8 * beat - 100;
    for (pad, region) in [
        (
            0,
            Region {
                start: 0,
                end: beat + shift,
            },
        ),
        (
            1,
            Region {
                start: beat + shift,
                end: 2 * beat,
            },
        ),
    ] {
        events.push(Event {
            frame: moved,
            cmd: Command::SetPadRegion { pad, region },
        });
    }
    events.sort_by_key(|e| e.frame);
    let sc = Scenario {
        sample_rate: SR,
        frames: 17 * beat,
        events,
    };
    let (l, r) = render(&mut e, &sc, 128);
    assert_eq!(first_non_finite(&l), None);
    assert!(peak(&l) <= 0.95, "clipping");
    // before the move pad 2 starts at the old cut, after it at the new one
    for i in 0..MS {
        assert_eq!(l[3 * beat + i], source.left[beat + i], "old cut, frame {i}");
    }
    // step 9 is pad 2: pad 1 from step 8 now runs 20 ms into it, so compare after that
    let after = 9 * beat + shift;
    let (got, want) = (l[after + 10], source.left[beat + shift + shift + 10]);
    assert!((got - want).abs() < 1e-6, "new cut: {got} vs {want}");
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/04c-moved-cut.wav");
    write_wav(&out, SR, &l, &r).unwrap();
}

//! The loop on the chop screen (49): the preview plays a region over and over, takes a new
//! region at the next turn, fades at every seam. Spec scenarios: "The slice over and over",
//! "No click at the seam", "Move a bound while it loops". The render goes to
//! `renders/49-loop.wav` for the owner's ears.

use std::path::Path;
use std::sync::Arc;

use engine::checks::{first_non_finite, max_step, peak};
use engine::offline::{Event, Scenario, render, write_wav};
use engine::{Command, Engine, Region, Sample};

const SR: u32 = 48_000;

/// A tone loud to its last sample: a seam without a fade would click.
fn tone() -> Arc<Sample> {
    Arc::new(Sample::mono(
        (0..48_000)
            .map(|i| 0.5 * (std::f32::consts::TAU * 220.0 * i as f32 / SR as f32).sin())
            .collect(),
    ))
}

fn at(frame: usize, cmd: Command) -> Event {
    Event { frame, cmd }
}

fn run(frames: usize, events: Vec<Event>) -> Vec<f32> {
    let sc = Scenario {
        sample_rate: SR,
        frames,
        events,
    };
    render(&mut Engine::new(SR), &sc, 128).0
}

fn slice(start: usize, end: usize) -> Region {
    Region { start, end }
}

/// The start of a preview of `region` with the loop on.
fn looping(sound: &Arc<Sample>, region: Region) -> Vec<Event> {
    vec![
        at(
            0,
            Command::PreviewLoop {
                region: Some(region),
            },
        ),
        at(
            0,
            Command::Preview {
                sound: Some(sound.clone()),
                region,
            },
        ),
    ]
}

/// The output frames where the sound dips to near silence: the seams of the loop.
fn seams(buf: &[f32]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    for (w, chunk) in buf.chunks(48).enumerate() {
        let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();
        let at = w * 48;
        if rms < 0.05 && out.last().is_none_or(|&l| at > l + 1_000) {
            out.push(at);
        }
    }
    out
}

/// "The slice over and over": 10 000 frames looped plays on past three turns; with the
/// loop off it plays once.
#[test]
fn the_slice_over_and_over() {
    let sound = tone();
    let region = slice(10_000, 20_000);
    let buf = run(35_000, looping(&sound, region));
    for turn in 0..3 {
        let part = &buf[turn * 10_000 + 1_000..turn * 10_000 + 9_000];
        assert!(peak(part) > 0.4, "turn {} sounds", turn + 1);
    }
    assert!(peak(&buf[30_500..34_500]) > 0.4, "and goes on");
    let once = run(
        35_000,
        vec![at(
            0,
            Command::Preview {
                sound: Some(sound),
                region,
            },
        )],
    );
    assert_eq!(peak(&once[11_000..]), 0.0, "off: once");
}

/// "No click at the seam": the loop's steps are no larger than the tone's own.
#[test]
fn no_click_at_the_seam() {
    let sound = tone();
    let buf = run(35_000, looping(&sound, slice(10_000, 20_000)));
    assert_eq!(first_non_finite(&buf), None);
    assert!(peak(&buf) < 1.0, "no clipping");
    let own = max_step(&sound.left[10_000..20_000]);
    let step = max_step(&buf);
    assert!(step <= own * 1.01, "step {step}, the tone's own {own}");
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/49-loop.wav");
    write_wav(&out, SR, &buf, &buf).unwrap();
}

/// "Move a bound while it loops": the end moved left in the middle of the second turn
/// shortens the third, and the sound does not stop in between.
#[test]
fn move_a_bound_while_it_loops() {
    let sound = tone();
    let mut events = looping(&sound, slice(10_000, 20_000));
    events.push(at(
        15_000,
        Command::PreviewLoop {
            region: Some(slice(10_000, 15_000)),
        },
    ));
    let buf = run(36_000, events);
    let found = seams(&buf);
    let near = |want: usize| found.iter().any(|&s| s.abs_diff(want) < 200);
    assert!(near(10_000) && near(20_000), "the full turns: {found:?}");
    assert!(
        near(25_000) && near(30_000),
        "the short turns after the move: {found:?}"
    );
    assert!(
        peak(&buf[15_000..20_000]) > 0.4,
        "the second turn plays to its old end"
    );
}

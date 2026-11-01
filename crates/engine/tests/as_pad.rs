//! The chop screen plays a slice as its pad (50): the preview borrows the pad's pitch,
//! volume, envelope and filter. Spec scenarios: "Slice 3 as its pad", "Off plays bare".
//! The render goes to `renders/50-as-pad.wav` for the owner's ears.

use std::path::Path;
use std::sync::Arc;

use engine::checks::{first_non_finite, peak};
use engine::filter::{FilterKind, FilterSettings};
use engine::offline::{Event, Scenario, render, write_wav};
use engine::{Command, Engine, Region, Sample};

const SR: u32 = 48_000;
const PAD: usize = 6; // W

fn at(frame: usize, cmd: Command) -> Event {
    Event { frame, cmd }
}

/// A saw at 440 Hz: its pitch shows in the zero crossings, its overtones above 2 kHz.
fn saw() -> Arc<Sample> {
    Arc::new(Sample::mono(
        (0..SR as usize)
            .map(|i| {
                let t = i as f32 * 440.0 / SR as f32;
                0.4 * (2.0 * (t - t.floor()) - 1.0)
            })
            .collect(),
    ))
}

/// Pad W at −12 st behind a low-pass at 500 Hz.
fn pad_w() -> Vec<Event> {
    vec![
        at(
            0,
            Command::SetPadPitch {
                pad: PAD,
                semitones: -12,
            },
        ),
        at(
            0,
            Command::SetPadFilter {
                pad: PAD,
                settings: FilterSettings {
                    kind: FilterKind::LowPass,
                    cutoff_hz: 500.0,
                    resonance: 0.0,
                },
            },
        ),
    ]
}

fn run(frames: usize, events: Vec<Event>) -> Vec<f32> {
    let sc = Scenario {
        sample_rate: SR,
        frames,
        events,
    };
    render(&mut Engine::new(SR), &sc, 128).0
}

fn preview(sound: &Arc<Sample>, region: Region) -> Command {
    Command::Preview {
        sound: Some(sound.clone()),
        region,
    }
}

/// Rising zero crossings per second over the buffer.
fn crossings(buf: &[f32]) -> f32 {
    let n = buf.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count();
    n as f32 * SR as f32 / buf.len() as f32
}

/// Energy of `buf` between `lo` and `hi` Hz: a plain DFT over the bins in that band.
fn band_energy(buf: &[f32], lo: f32, hi: f32) -> f64 {
    let n = buf.len();
    let bin = SR as f32 / n as f32;
    let (first, last) = ((lo / bin).ceil() as usize, (hi / bin).floor() as usize);
    (first.max(1)..=last)
        .map(|k| {
            let w = std::f64::consts::TAU * k as f64 / n as f64;
            let (re, im) = buf
                .iter()
                .enumerate()
                .fold((0.0, 0.0), |(re, im), (i, &s)| {
                    let a = w * i as f64;
                    (re + s as f64 * a.cos(), im - s as f64 * a.sin())
                });
            re * re + im * im
        })
        .sum()
}

/// "Slice 3 as its pad": an octave lower, the overtones mostly gone.
#[test]
fn slice_3_as_its_pad() {
    let sound = saw();
    let region = Region {
        start: 0,
        end: 24_000,
    };
    let bare = run(24_000, vec![at(0, preview(&sound, region))]);
    let mut events = pad_w();
    events.push(at(0, Command::PreviewAs { pad: Some(PAD) }));
    events.push(at(0, preview(&sound, region)));
    let wet = run(24_000, events);
    assert_eq!(first_non_finite(&wet), None);
    assert!(peak(&wet) < 1.0, "no clipping");
    let (b, w) = (crossings(&bare[2_400..]), crossings(&wet[2_400..]));
    assert!(
        (w / b - 0.5).abs() < 0.05,
        "an octave lower: {b} -> {w} crossings a second"
    );
    let (b, w) = (
        band_energy(&bare[2_400..7_200], 2_000.0, 20_000.0),
        band_energy(&wet[2_400..7_200], 2_000.0, 20_000.0),
    );
    assert!(w < b / 10.0, "above 2 kHz: {w:.3} of {b:.3}");
    let mut both = bare.clone();
    both.extend(vec![0.0; 12_000]);
    both.extend(&wet);
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/50-as-pad.wav");
    write_wav(&out, SR, &both, &both).unwrap();
}

/// "Off plays bare": with the pad set up but no `PreviewAs`, the preview is the bare
/// sample, sample for sample; and `PreviewAs` holds for one preview only.
#[test]
fn off_plays_bare() {
    let sound = saw();
    let region = Region {
        start: 0,
        end: 12_000,
    };
    let bare = run(12_000, vec![at(0, preview(&sound, region))]);
    let mut events = pad_w();
    events.push(at(0, preview(&sound, region)));
    assert_eq!(run(12_000, events), bare, "no PreviewAs: bare");
    // the second preview after a PreviewAs plays bare again
    let mut events = pad_w();
    events.push(at(0, Command::PreviewAs { pad: Some(PAD) }));
    events.push(at(0, preview(&sound, region)));
    events.push(at(12_000, preview(&sound, region)));
    let second = run(24_000, events);
    assert_eq!(&second[12_000 + 240..], &bare[240..], "one preview only");
}

/// A hit on the pad does not cut the preview that plays as it.
#[test]
fn a_hit_does_not_cut_it() {
    let sound = saw();
    let region = Region {
        start: 0,
        end: 24_000,
    };
    let mut events = pad_w();
    events.push(at(0, Command::PreviewAs { pad: Some(PAD) }));
    events.push(at(0, preview(&sound, region)));
    events.push(at(
        6_000,
        Command::PadPress {
            pad: PAD,
            velocity: 0.0,
        },
    ));
    let buf = run(24_000, events);
    assert!(peak(&buf[12_000..18_000]) > 0.05, "the preview plays on");
}

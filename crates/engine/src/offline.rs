//! Offline render: the same engine, blocks computed in a loop without a sound card.

use std::path::Path;

use crate::{Command, Engine};

pub struct Event {
    /// Time in samples from the start of the render.
    pub frame: usize,
    pub cmd: Command,
}

pub struct Scenario {
    pub sample_rate: u32,
    /// Render length in samples.
    pub frames: usize,
    pub events: Vec<Event>,
}

impl Scenario {
    /// `count` clicks every `1/division` of a 4/4 bar at `bpm`, plus one second of silence at the end.
    pub fn clicks(sample_rate: u32, bpm: f64, division: u32, count: usize) -> Self {
        let step = sample_rate as f64 * 60.0 / bpm * 4.0 / division as f64;
        let events: Vec<_> = (0..count)
            .map(|i| Event {
                frame: (i as f64 * step).round() as usize,
                cmd: Command::Click { velocity: 1.0 },
            })
            .collect();
        let frames = (count as f64 * step) as usize + sample_rate as usize;
        Self {
            sample_rate,
            frames,
            events,
        }
    }
}

/// Renders the scenario in blocks of `block` samples; returns left and right channels.
pub fn render(engine: &mut Engine, scenario: &Scenario, block: usize) -> (Vec<f32>, Vec<f32>) {
    let mut left = vec![0.0; scenario.frames];
    let mut right = vec![0.0; scenario.frames];
    let mut start = 0;
    while start < scenario.frames {
        let end = (start + block).min(scenario.frames);
        for e in scenario
            .events
            .iter()
            .filter(|e| (start..end).contains(&e.frame))
        {
            engine.handle(e.frame - start, e.cmd.clone());
        }
        engine.process(&mut left[start..end], &mut right[start..end]);
        start = end;
    }
    (left, right)
}

/// Writes a stereo 32-bit float WAV.
pub fn write_wav(
    path: &Path,
    sample_rate: u32,
    left: &[f32],
    right: &[f32],
) -> Result<(), hound::Error> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec)?;
    for (l, r) in left.iter().zip(right) {
        w.write_sample(*l)?;
        w.write_sample(*r)?;
    }
    w.finalize()
}

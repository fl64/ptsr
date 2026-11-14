//! Loading samples on a background thread: decoding (symphonia) and resampling to the
//! system rate (rubato). The audio thread only ever gets ready samples.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};

use engine::Sample;
use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler};
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

/// File extensions the browser shows and the loader can decode.
pub const EXTENSIONS: [&str; 7] = ["wav", "flac", "aif", "aiff", "mp3", "ogg", "oga"];

pub struct Request {
    pub pad: usize,
    pub path: PathBuf,
    /// Ever-growing per pad: a reply with an older generation is stale.
    pub generation: u64,
}

pub enum Reply {
    Progress {
        pad: usize,
        percent: u8,
    },
    Loaded {
        pad: usize,
        generation: u64,
        path: PathBuf,
        sound: Arc<Sample>,
    },
    Failed {
        pad: usize,
        generation: u64,
        path: PathBuf,
        error: String,
    },
}

pub fn start(sample_rate: u32) -> (Sender<Request>, Receiver<Reply>) {
    let (req_tx, req_rx) = mpsc::channel::<Request>();
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("loader".into())
        .spawn(move || {
            while let Ok(first) = req_rx.recv() {
                let pending = latest_per_pad(std::iter::once(first).chain(req_rx.try_iter()));
                for r in pending {
                    load(r, sample_rate, &tx);
                }
            }
        })
        .expect("spawn loader");
    (req_tx, rx)
}

/// Keeps only the newest request of each pad (and of the preview), in order: scrolling fast
/// through files must not decode every file it passed.
fn latest_per_pad(requests: impl Iterator<Item = Request>) -> Vec<Request> {
    let mut out: Vec<Request> = Vec::new();
    for r in requests {
        out.retain(|o| o.pad != r.pad);
        out.push(r);
    }
    out
}

fn load(r: Request, sample_rate: u32, tx: &Sender<Reply>) {
    let pad = r.pad;
    let progress = |percent| {
        let _ = tx.send(Reply::Progress { pad, percent });
    };
    let reply = match decode(&r.path, sample_rate, progress) {
        Ok(s) => Reply::Loaded {
            pad,
            generation: r.generation,
            path: r.path,
            sound: Arc::new(s),
        },
        Err(error) => {
            log::warn!("{}: {error}", r.path.display());
            Reply::Failed {
                pad,
                generation: r.generation,
                path: r.path,
                error,
            }
        }
    };
    let _ = tx.send(reply);
}

/// Decodes a file into a mono or stereo sample at `sample_rate`; extra channels are dropped.
/// `progress` gets 0..100 now and then for long files.
pub fn decode(
    path: &Path,
    sample_rate: u32,
    mut progress: impl FnMut(u8),
) -> Result<Sample, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let mut format = symphonia::default::get_probe()
        .probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|e| format!("not an audio file ({e})"))?;
    let track = format
        .default_track(TrackType::Audio)
        .ok_or("no audio track")?;
    let params = track
        .codec_params
        .as_ref()
        .and_then(|p| p.audio())
        .ok_or("no audio track")?;
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(params, &AudioDecoderOptions::default())
        .map_err(|e| format!("unsupported codec ({e})"))?;
    let (track_id, total) = (track.id, track.num_frames.unwrap_or(0));

    let mut interleaved: Vec<f32> = Vec::new();
    let mut packet_buf: Vec<f32> = Vec::new();
    let (mut channels, mut rate) = (0, 0);
    let mut last_percent = 0;
    loop {
        let packet = match format.next_packet() {
            Ok(Some(p)) => p,
            Ok(None) => break,
            Err(e) => return Err(format!("cannot read ({e})")),
        };
        if packet.track_id != track_id {
            continue;
        }
        let buf = match decoder.decode(&packet) {
            Ok(b) => b,
            // a damaged packet is skipped, like players do
            Err(Error::DecodeError(_)) => continue,
            Err(e) => return Err(format!("cannot decode ({e})")),
        };
        let spec = buf.spec();
        let ch = spec.channels().count();
        (channels, rate) = (ch.min(2), spec.rate());
        packet_buf.resize(buf.samples_interleaved(), 0.0);
        buf.copy_to_slice_interleaved(&mut packet_buf);
        interleaved.extend(
            packet_buf
                .chunks(ch.max(1))
                .flat_map(|f| f.iter().take(2).copied()),
        );

        if let Some(percent) =
            ((interleaved.len() / channels.max(1)) as u64 * 100).checked_div(total)
        {
            let percent = percent.min(100) as u8;
            if percent >= last_percent + 5 {
                last_percent = percent;
                progress(percent);
            }
        }
    }
    if interleaved.is_empty() || channels == 0 {
        return Err("no audio in the file".into());
    }
    if rate != sample_rate {
        interleaved = resample(&interleaved, channels, rate, sample_rate)?;
    }
    Ok(if channels == 1 {
        Sample::mono(interleaved)
    } else {
        let left = interleaved.iter().step_by(2).copied().collect();
        let right = interleaved.iter().skip(1).step_by(2).copied().collect();
        Sample {
            left,
            right: Some(right),
        }
    })
}

/// A mini waveform: `cols` blocks, each the peak of its stretch of the sample, as `▁`..`█`.
/// Peak envelope of the sample: `cols` values, each the loudest of its stretch, 0..1.
pub fn envelope(sample: &Sample, cols: usize) -> Vec<f32> {
    envelope_region(sample, 0, sample.len(), cols)
}

/// Peak envelope of a piece of the sample, frames `start..end`: `cols` values over the
/// piece only, each the loudest of its stretch, 0..1.
pub fn envelope_region(sample: &Sample, start: usize, end: usize, cols: usize) -> Vec<f32> {
    let end = end.min(sample.len());
    let (start, end) = (start.min(end), end);
    if start == end || cols == 0 {
        return Vec::new();
    }
    let right = sample.right.as_deref().unwrap_or(&sample.left);
    let peaks: Vec<f32> = (0..cols)
        .map(|c| {
            let (a, b) = (
                start + c * (end - start) / cols,
                start
                    + (((c + 1) * (end - start) / cols).max(c * (end - start) / cols + 1))
                        .min(end - start),
            );
            sample.left[a..b]
                .iter()
                .chain(&right[a..b])
                .fold(0f32, |m, x| m.max(x.abs()))
        })
        .collect();
    let top = peaks.iter().fold(0f32, |m, p| m.max(*p)).max(1e-6);
    peaks.iter().map(|p| p / top).collect()
}

fn resample(interleaved: &[f32], channels: usize, from: u32, to: u32) -> Result<Vec<f32>, String> {
    let frames = interleaved.len() / channels;
    let mut r = Fft::<f32>::new(from as usize, to as usize, 1024, channels, FixedSync::Both)
        .map_err(|e| format!("resampler: {e}"))?;
    let input = InterleavedSlice::new(interleaved, channels, frames)
        .map_err(|e| format!("resampler: {e}"))?;
    let out = r
        .process_all(&input, frames, None)
        .map_err(|e| format!("resampler: {e}"))?;
    Ok(out.take_data())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ptsr-loader-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    fn write_wav(
        path: &Path,
        rate: u32,
        channels: u16,
        frames: usize,
        value: impl Fn(usize, u16) -> f32,
    ) {
        let spec = hound::WavSpec {
            channels,
            sample_rate: rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut w = hound::WavWriter::create(path, spec).unwrap();
        for i in 0..frames {
            for c in 0..channels {
                w.write_sample(value(i, c)).unwrap();
            }
        }
        w.finalize().unwrap();
    }

    #[test]
    fn mono_at_system_rate_is_exact() {
        let path = tmp("mono48.wav");
        write_wav(&path, 48_000, 1, 1000, |i, _| i as f32 / 1000.0);
        let s = decode(&path, 48_000, |_| {}).unwrap();
        assert!(s.right.is_none());
        assert_eq!(s.len(), 1000);
        assert_eq!(s.left[500], 0.5);
    }

    #[test]
    fn stereo_44k1_is_resampled_and_keeps_sides() {
        let path = tmp("stereo44.wav");
        // a quiet low tone, different on each side, so the resampler has a real signal
        let tone = |i: usize, c: u16| {
            let x = (i as f32 * 2.0 * std::f32::consts::PI * 100.0 / 44_100.0).sin() * 0.5;
            if c == 0 { x } else { -x }
        };
        write_wav(&path, 44_100, 2, 44_100, tone);
        let mut seen = Vec::new();
        let s = decode(&path, 48_000, |p| seen.push(p)).unwrap();
        // one second stays one second at the new rate
        assert!(s.len().abs_diff(48_000) <= 2, "{} frames", s.len());
        let right = s.right.as_ref().unwrap();
        let mid = 24_000;
        assert!((s.left[mid] + right[mid]).abs() < 1e-3, "sides mixed up");
        assert!(s.left.iter().fold(0f32, |m, x| m.max(x.abs())) > 0.45);
    }

    /// Scenario "Fast scrolling through long files" (sample-loading): the files passed are dropped, the last one loads.
    #[test]
    fn only_the_newest_request_per_pad_is_loaded() {
        let req = |pad, generation| Request {
            pad,
            path: "x".into(),
            generation,
        };
        let got = latest_per_pad([req(128, 1), req(3, 1), req(128, 2), req(128, 3)].into_iter());
        let got: Vec<_> = got.iter().map(|r| (r.pad, r.generation)).collect();
        assert_eq!(got, [(3, 1), (128, 3)]);
    }

    #[test]
    fn envelope_follows_the_loudness() {
        // loud, quiet, silent
        let s = Sample::mono((0..900).map(|i| [1.0, 0.1, 0.0][i / 300]).collect());
        assert_eq!(envelope(&s, 3), vec![1.0, 0.1, 0.0]);
        assert_eq!(envelope(&Sample::default(), 8), Vec::<f32>::new());
    }

    /// The big pad view draws a piece of the file: only the region is measured.
    #[test]
    fn envelope_region_covers_only_the_piece() {
        // first half loud, second half louder; the piece is the first half
        let s = Sample::mono((0..900).map(|i| [0.4, 1.0][i / 450]).collect());
        assert_eq!(envelope_region(&s, 0, 450, 3), vec![1.0; 3]);
        assert_eq!(envelope_region(&s, 450, 900, 3), vec![1.0; 3]);
        // the whole file normalizes against its loudest point
        assert_eq!(envelope(&s, 2), vec![0.4, 1.0]);
        // an end past the sample reads as "to the end" and stays inside
        assert_eq!(envelope_region(&s, 600, usize::MAX, 3), vec![1.0; 3]);
        assert_eq!(envelope_region(&s, 5, 5, 4), Vec::<f32>::new());
    }

    #[test]
    fn broken_file_is_an_error() {
        let path = tmp("broken.wav");
        std::fs::write(&path, b"RIFF this is not audio").unwrap();
        assert!(decode(&path, 48_000, |_| {}).is_err());
        assert!(decode(&tmp("does-not-exist.wav"), 48_000, |_| {}).is_err());
    }

    /// Render for the owner: files decoded by the loader, put on pads, played as a beat.
    #[test]
    fn render_pads_from_files() {
        use engine::offline::{Event, Scenario, render, write_wav as write_render};
        use engine::{Command, Engine, checks};

        let kick = tmp("kick44.wav");
        // a falling sine, 44.1 kHz mono: the loader resamples it
        write_wav(&kick, 44_100, 1, 13_000, |i, _| {
            let t = i as f32 / 44_100.0;
            let env = (1.0 - i as f32 / 13_000.0).powi(2);
            0.8 * env * (2.0 * std::f32::consts::PI * (50.0 + 120.0 * (-t * 30.0).exp()) * t).sin()
        });
        let shaker = tmp("shaker48.wav");
        // noise, 48 kHz stereo, louder on the right
        let mut seed = 1u32;
        let mut noise = move || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed >> 8) as f32 / (1 << 24) as f32 * 2.0 - 1.0
        };
        let noise: Vec<f32> = (0..4800 * 2).map(|_| noise()).collect();
        write_wav(&shaker, 48_000, 2, 4800, |i, c| {
            let env = (1.0 - i as f32 / 4800.0).powi(3);
            noise[i * 2 + c as usize] * env * if c == 0 { 0.15 } else { 0.3 }
        });

        let mut engine = Engine::new(48_000);
        for (pad, path) in [(0, &kick), (1, &shaker)] {
            let sound = Arc::new(decode(path, 48_000, |_| {}).unwrap());
            engine.handle(
                0,
                Command::SetPadSound {
                    pad,
                    sound: Some(sound),
                },
            );
        }
        // 2 bars at 100 BPM: kick on beats, shaker on eighths
        let eighth = 48_000 * 60 / 100 / 2;
        let mut events = Vec::new();
        for step in 0..16 {
            if step % 2 == 0 {
                events.push(Event {
                    frame: step * eighth,
                    cmd: Command::PadPress {
                        pad: 0,
                        velocity: 1.0,
                    },
                });
            }
            let accent = if step % 2 == 1 { 1.0 } else { 0.5 };
            events.push(Event {
                frame: step * eighth,
                cmd: Command::PadPress {
                    pad: 1,
                    velocity: accent,
                },
            });
        }
        let sc = Scenario {
            sample_rate: 48_000,
            frames: 17 * eighth,
            events,
        };
        let (l, r) = render(&mut engine, &sc, 128);
        assert_eq!(checks::first_non_finite(&l), None);
        assert!(checks::peak(&l).max(checks::peak(&r)) <= 0.95, "clipping");
        assert!(
            checks::peak(&r) > checks::peak(&l) * 0.9,
            "the shaker's right side got lost"
        );
        let out =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/03-pads-from-files.wav");
        write_render(&out, 48_000, &l, &r).unwrap();
    }
}

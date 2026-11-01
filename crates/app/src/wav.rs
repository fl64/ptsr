//! Writing 16-bit PCM WAV files, so recordings and later saved edits land on disk.

use std::path::Path;

use engine::Sample;

/// Writes `sample` as a 16-bit PCM WAV (mono stays mono, stereo stays stereo).
pub fn write(path: &Path, sample: &Sample, rate: u32) -> Result<(), String> {
    let channels = if sample.right.is_some() { 2 } else { 1 };
    let frames = sample.left.len();
    let data_len = frames * channels * 2;
    let mut out = Vec::with_capacity(44 + data_len);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len as u32).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&(channels as u16).to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * channels as u32 * 2).to_le_bytes());
    out.extend_from_slice(&(channels as u16 * 2).to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data_len as u32).to_le_bytes());
    let mut put = |v: f32| {
        let v = (v.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        out.extend_from_slice(&v.to_le_bytes());
    };
    match &sample.right {
        Some(right) => {
            for (l, r) in sample.left.iter().zip(right.iter()) {
                put(*l);
                put(*r);
            }
        }
        None => sample.left.iter().for_each(|&s| put(s)),
    }
    std::fs::write(path, &out).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("ptsr-wav-{name}.wav"));
        p
    }

    #[test]
    fn written_file_decodes_back_to_the_same_sound() {
        let path = tmp("round");
        let sample = Sample {
            left: vec![0.0, 0.5, -0.5, 1.0, -1.0].into(),
            right: Some(vec![0.25, -0.25, 0.75, -0.75, 0.0].into()),
        };
        write(&path, &sample, 48_000).unwrap();
        let back = crate::loader::decode(&path, 48_000, |_| {}).unwrap();
        assert_eq!(back.left.len(), sample.left.len());
        for (a, b) in back.left.iter().zip(&sample.left) {
            assert!((a - b).abs() < 1.0 / 32767.0, "{a} vs {b}");
        }
        let right = back.right.expect("stereo stays stereo");
        for (a, b) in right.iter().zip(&sample.right.unwrap()) {
            assert!((a - b).abs() < 1.0 / 32767.0, "{a} vs {b}");
        }
        std::fs::remove_file(&path).ok();
    }
}

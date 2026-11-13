//! Shared wave shapes and colors.

use super::*;

/// Waveform in Braille characters: each cell is 2 dot columns by 4 dot rows; every dot
/// column shows the peak of its stretch of `wave`, mirrored around the middle and
/// scaled to the wave's own peak.
pub fn braille(wave: &[f32], cols: usize, rows: usize) -> Vec<String> {
    let top = wave.iter().fold(0f32, |m, x| m.max(x.abs())).max(1e-6);
    braille_scaled(wave, cols, rows, top)
}

/// Braille at a fixed full scale: a quiet sound draws low, a loud one tall.
pub fn braille_abs(wave: &[f32], cols: usize, rows: usize) -> Vec<String> {
    braille_scaled(wave, cols, rows, 1.0)
}

fn braille_scaled(wave: &[f32], cols: usize, rows: usize, top: f32) -> Vec<String> {
    let (dx, dy) = (cols * 2, rows * 4);
    let mut cells = vec![vec![0u8; cols]; rows];
    if wave.is_empty() || dx == 0 || dy == 0 {
        return cells.iter().map(|r| " ".repeat(r.len())).collect();
    }
    // dot bits of a Braille cell, by (column, row)
    const BITS: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
    for (x, &peak) in dot_peaks(wave, dx, top).iter().enumerate() {
        // at least the middle dot, so silence still reads as a line
        let half = ((peak * dy as f32 / 2.0).round() as usize).max(1);
        let mid = dy / 2;
        for y in mid.saturating_sub(half)..(mid + half).min(dy) {
            cells[y / 4][x / 2] |= BITS[x % 2][y % 4];
        }
    }
    cells
        .iter()
        .map(|r| {
            r.iter()
                .map(|&bits| char::from_u32(0x2800 + bits as u32).unwrap_or(' '))
                .collect()
        })
        .collect()
}

/// The peak of every dot column (two per cell) of `wave`, scaled by `top`: the shape the
/// Braille wave and its kitty picture both draw.
pub(super) fn dot_peaks(wave: &[f32], dots: usize, top: f32) -> Vec<f32> {
    if wave.is_empty() || dots == 0 {
        return Vec::new();
    }
    (0..dots)
        .map(|x| {
            let (a, b) = (
                x * wave.len() / dots,
                ((x + 1) * wave.len() / dots).max(x * wave.len() / dots + 1),
            );
            wave[a..b.min(wave.len())]
                .iter()
                .fold(0f32, |m, v| m.max(v.abs()))
                / top
        })
        .collect()
}

/// A color without its hue, at its brightness: the slices that are not selected (40).
pub(super) fn grey(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => {
            let l = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32).round() as u8;
            Color::Rgb(l, l, l)
        }
        c => c,
    }
}

/// The tones of the three bands (38): the low red, the mid green, the high blue — the
/// owner asked for contrast after the softer palette merged into grey.
const BAND_TONES: [(f32, f32, f32); 3] = [
    (255.0, 70.0, 60.0),
    (80.0, 225.0, 100.0),
    (80.0, 130.0, 255.0),
];

/// A wave column colored by the band that stands out against its average over the file,
/// at `bright` from the background (1.0 full, less steps back). Each band is read against
/// its mean, and the shares go to the 4th power: a break whose bands are nearly even on
/// average still shows its kicks red and its hats blue, not a grey mix.
pub(super) fn band_color(b: [f32; 3], mean: [f32; 3], bright: f32) -> Color {
    let rel = [0, 1, 2].map(|i| (b[i] / mean[i].max(1e-6)).powi(4));
    let total = rel.iter().sum::<f32>().max(1e-9);
    let mut rgb = [0.0f32; 3];
    for (level, tone) in rel.iter().zip(BAND_TONES) {
        let w = level / total;
        rgb[0] += w * tone.0;
        rgb[1] += w * tone.1;
        rgb[2] += w * tone.2;
    }
    let bg = [7.0f32, 8.0, 10.0];
    let ch = |i: usize| {
        (bg[i] + (rgb[i] - bg[i]) * bright)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color::Rgb(ch(0), ch(1), ch(2))
}

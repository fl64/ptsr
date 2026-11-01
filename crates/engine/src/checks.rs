//! Render checks: the agent cannot hear, so it checks audio with numbers.

/// Index of the first NaN or infinity.
pub fn first_non_finite(buf: &[f32]) -> Option<usize> {
    buf.iter().position(|s| !s.is_finite())
}

pub fn peak(buf: &[f32]) -> f32 {
    buf.iter().fold(0.0, |m, s| m.max(s.abs()))
}

/// Sound onsets: a sample louder than `eps` preceded by at least `min_gap` silent samples.
pub fn onsets(buf: &[f32], eps: f32, min_gap: usize) -> Vec<usize> {
    let mut found = Vec::new();
    let mut last_loud: Option<usize> = None;
    for (i, s) in buf.iter().enumerate() {
        if s.abs() > eps {
            if last_loud.is_none_or(|l| i - l > min_gap) {
                found.push(i);
            }
            last_loud = Some(i);
        }
    }
    found
}

/// Index of the first sample louder than `eps` in the range, if the silence is broken.
pub fn first_loud(buf: &[f32], range: std::ops::Range<usize>, eps: f32) -> Option<usize> {
    let start = range.start;
    buf[range]
        .iter()
        .position(|s| s.abs() > eps)
        .map(|i| start + i)
}

/// Largest jump between neighbouring samples: a click is a sharp jump.
pub fn max_step(buf: &[f32]) -> f32 {
    buf.windows(2).fold(0.0, |m, w| m.max((w[1] - w[0]).abs()))
}

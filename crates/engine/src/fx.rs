//! The pad drive and the two shared effect slots (56). A pad drives its own voices and
//! sends a share of them to the slots; a slot holds a reverb or a delay and adds its wet
//! sound to the output. Every buffer is allocated with the engine: nothing allocates in the
//! audio thread.

/// What a slot holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FxKind {
    Reverb,
    Delay,
}

/// A slot as the owner sets it: the kind and its two parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FxSettings {
    /// `size` and `tone` 0..1: a small room to a hall, dark to bright.
    Reverb { size: f32, tone: f32 },
    /// `time_ms` 10..2000, `feedback` 0..0.9: the echo always dies out.
    Delay { time_ms: f32, feedback: f32 },
}

impl FxSettings {
    pub fn kind(self) -> FxKind {
        match self {
            FxSettings::Reverb { .. } => FxKind::Reverb,
            FxSettings::Delay { .. } => FxKind::Delay,
        }
    }
}

pub const DELAY_MS_MIN: f32 = 10.0;
pub const DELAY_MS_MAX: f32 = 2_000.0;
pub const FEEDBACK_MAX: f32 = 0.9;
/// Time a changed parameter of a slot takes to settle: the delay time slides, no click.
const PARAM_GLIDE_SECONDS: f32 = 0.050;

/// The level the drive keeps: a sound at this peak comes out at the same peak.
const DRIVE_REF: f32 = 0.3;
/// The input gain of the drive at full: grit, then the gain is taken back.
const DRIVE_GAIN: f32 = 24.0;

/// The drive of one sample, `drive` 0..1: at 0 the sample is untouched; up from there the
/// soft clip mixes in fully by a quarter and pushes harder to the top, the loudness kept.
pub fn drive(x: f32, drive: f32) -> f32 {
    Drive::new(drive).apply(x)
}

#[derive(Clone, Copy)]
pub(crate) struct Drive {
    k: f32,
    scale: f32,
    mix: f32,
}

impl Drive {
    pub(crate) fn new(drive: f32) -> Self {
        if drive <= 0.0 {
            return Self {
                k: 1.0,
                scale: 1.0,
                mix: 0.0,
            };
        }
        let k = 1.0 + DRIVE_GAIN * drive * drive;
        Self {
            k,
            scale: DRIVE_REF / (k * DRIVE_REF).tanh(),
            mix: (drive * 4.0).min(1.0),
        }
    }

    pub(crate) fn apply(self, x: f32) -> f32 {
        if self.mix == 0.0 {
            return x;
        }
        let shaped = (self.k * x).tanh() * self.scale;
        x + (shaped - x) * self.mix
    }
}

/// A stereo delay line: the wet echo only, the dry sound is the pad's own.
pub struct Delay {
    buf: [Vec<f32>; 2],
    write: usize,
    /// Delay in frames: `time` is the target, `time_now` slides to it.
    time: f32,
    time_now: f32,
    feedback: f32,
}

impl Delay {
    fn new(sample_rate: f32) -> Self {
        // room for the longest time plus the read interpolation
        let len = (DELAY_MS_MAX * 0.001 * sample_rate) as usize + 4;
        Self {
            buf: [vec![0.0; len], vec![0.0; len]],
            write: 0,
            time: 0.25 * sample_rate,
            time_now: 0.25 * sample_rate,
            feedback: 0.4,
        }
    }

    fn set(&mut self, time_ms: f32, feedback: f32, sample_rate: f32) {
        self.time = time_ms.clamp(DELAY_MS_MIN, DELAY_MS_MAX) * 0.001 * sample_rate;
        self.feedback = feedback.clamp(0.0, FEEDBACK_MAX);
    }

    fn clear(&mut self) {
        for b in self.buf.iter_mut() {
            b.fill(0.0);
        }
        self.time_now = self.time;
    }

    fn tick(&mut self, input: [f32; 2], glide: f32) -> [f32; 2] {
        self.time_now += (self.time - self.time_now) * glide;
        let len = self.buf[0].len();
        // the read point `time_now` frames back, between two frames
        let back = self.time_now.clamp(1.0, (len - 2) as f32);
        let whole = back as usize;
        let frac = back - whole as f32;
        let i0 = (self.write + len - whole) % len;
        let i1 = (i0 + len - 1) % len;
        let mut out = [0.0; 2];
        for (c, b) in self.buf.iter_mut().enumerate() {
            let y = b[i0] + (b[i1] - b[i0]) * frac;
            b[self.write] = input[c] + y * self.feedback;
            out[c] = y;
        }
        self.write = (self.write + 1) % len;
        out
    }
}

/// A comb filter of the reverb with damping in its loop.
struct Comb {
    buf: Vec<f32>,
    at: usize,
    store: f32,
}

impl Comb {
    fn tick(&mut self, x: f32, feedback: f32, damp: f32) -> f32 {
        let y = self.buf[self.at];
        self.store = y * (1.0 - damp) + self.store * damp;
        self.buf[self.at] = x + self.store * feedback;
        self.at = (self.at + 1) % self.buf.len();
        y
    }
}

struct Allpass {
    buf: Vec<f32>,
    at: usize,
}

impl Allpass {
    fn tick(&mut self, x: f32) -> f32 {
        let b = self.buf[self.at];
        self.buf[self.at] = x + b * 0.5;
        self.at = (self.at + 1) % self.buf.len();
        b - x
    }
}

/// Comb and allpass lengths of the classic Freeverb at 44.1 kHz; the right side is a
/// little longer, so the two sides do not sound the same.
const COMBS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASSES: [usize; 4] = [556, 441, 341, 225];
const SPREAD: usize = 23;
/// The level into the combs and out of the reverb.
const REVERB_IN: f32 = 0.015;
const REVERB_OUT: f32 = 3.0;

/// A Schroeder-Moorer reverb as in Freeverb: eight damped combs in parallel, four
/// allpasses in a row, per side.
pub struct Reverb {
    combs: [Vec<Comb>; 2],
    allpasses: [Vec<Allpass>; 2],
    /// Comb feedback (the size) and damping (the tone): targets and the values now.
    feedback: f32,
    damp: f32,
    feedback_now: f32,
    damp_now: f32,
}

impl Reverb {
    fn new(sample_rate: f32) -> Self {
        let scale = sample_rate / 44_100.0;
        let len = |n: usize, side: usize| (((n + side * SPREAD) as f32 * scale) as usize).max(1);
        let combs = std::array::from_fn(|side| {
            COMBS
                .iter()
                .map(|&n| Comb {
                    buf: vec![0.0; len(n, side)],
                    at: 0,
                    store: 0.0,
                })
                .collect()
        });
        let allpasses = std::array::from_fn(|side| {
            ALLPASSES
                .iter()
                .map(|&n| Allpass {
                    buf: vec![0.0; len(n, side)],
                    at: 0,
                })
                .collect()
        });
        let mut r = Self {
            combs,
            allpasses,
            feedback: 0.0,
            damp: 0.0,
            feedback_now: 0.0,
            damp_now: 0.0,
        };
        r.set(0.5, 0.5);
        r.feedback_now = r.feedback;
        r.damp_now = r.damp;
        r
    }

    /// `size` 0..1 sets the tail from a small room to a hall, `tone` 0..1 from dark to bright.
    fn set(&mut self, size: f32, tone: f32) {
        self.feedback = 0.70 + 0.28 * size.clamp(0.0, 1.0);
        self.damp = 0.05 + 0.6 * (1.0 - tone.clamp(0.0, 1.0));
    }

    fn clear(&mut self) {
        for side in 0..2 {
            for c in self.combs[side].iter_mut() {
                c.buf.fill(0.0);
                c.store = 0.0;
            }
            for a in self.allpasses[side].iter_mut() {
                a.buf.fill(0.0);
            }
        }
        self.feedback_now = self.feedback;
        self.damp_now = self.damp;
    }

    fn tick(&mut self, input: [f32; 2], glide: f32) -> [f32; 2] {
        self.feedback_now += (self.feedback - self.feedback_now) * glide;
        self.damp_now += (self.damp - self.damp_now) * glide;
        let x = (input[0] + input[1]) * REVERB_IN;
        let (fb, damp) = (self.feedback_now, self.damp_now);
        let mut out = [0.0; 2];
        for (side, o) in out.iter_mut().enumerate() {
            let mut y = 0.0;
            for c in self.combs[side].iter_mut() {
                y += c.tick(x, fb, damp);
            }
            for a in self.allpasses[side].iter_mut() {
                y = a.tick(y);
            }
            *o = y * REVERB_OUT;
        }
        out
    }
}

/// One shared slot: the effect it holds now and the tail of the one it held before, which
/// fades out after a change of kind. Each kind keeps its own state; the app keeps the
/// parameters of every kind.
pub struct Slot {
    kind: FxKind,
    reverb: Reverb,
    delay: Delay,
    /// The kind held before and the frames left of its fade-out.
    leaving: Option<(FxKind, u32)>,
    fade_len: u32,
    glide: f32,
    sample_rate: f32,
    /// Nothing reached the effect yet: a new setting takes hold at once, no slide.
    quiet: bool,
}

impl Slot {
    pub fn new(kind: FxKind, sample_rate: f32, fade_len: u32) -> Self {
        Self {
            kind,
            reverb: Reverb::new(sample_rate),
            delay: Delay::new(sample_rate),
            leaving: None,
            fade_len: fade_len.max(1),
            glide: 1.0 / (PARAM_GLIDE_SECONDS * sample_rate).max(1.0),
            sample_rate,
            quiet: true,
        }
    }

    pub fn set(&mut self, settings: FxSettings) {
        let kind = settings.kind();
        let mut fresh = self.quiet;
        if kind != self.kind {
            // back to the kind still fading out: it carries on; any other starts empty
            let back = matches!(self.leaving, Some((k, _)) if k == kind);
            if !back && !self.quiet {
                match kind {
                    FxKind::Reverb => self.reverb.clear(),
                    FxKind::Delay => self.delay.clear(),
                }
                fresh = true;
            }
            self.leaving = Some((self.kind, self.fade_len));
            self.kind = kind;
        }
        match settings {
            FxSettings::Reverb { size, tone } => {
                self.reverb.set(size, tone);
                if fresh {
                    (self.reverb.feedback_now, self.reverb.damp_now) =
                        (self.reverb.feedback, self.reverb.damp);
                }
            }
            FxSettings::Delay { time_ms, feedback } => {
                self.delay.set(time_ms, feedback, self.sample_rate);
                // an empty delay takes its time at once; a sounding one slides to it
                if fresh {
                    self.delay.time_now = self.delay.time;
                }
            }
        }
    }

    fn run(&mut self, kind: FxKind, input: [f32; 2]) -> [f32; 2] {
        match kind {
            FxKind::Reverb => self.reverb.tick(input, self.glide),
            FxKind::Delay => self.delay.tick(input, self.glide),
        }
    }

    /// Clear at most `budget` stored samples. Do not tick the slot while clearing.
    pub(crate) fn clear_step(&mut self, cursor: &mut usize, budget: usize) -> bool {
        let mut skip = *cursor;
        let mut left = budget;
        let mut clear = |buf: &mut [f32]| {
            if skip >= buf.len() {
                skip -= buf.len();
                return;
            }
            let n = (buf.len() - skip).min(left);
            buf[skip..skip + n].fill(0.0);
            left -= n;
            skip = 0;
        };
        for b in &mut self.delay.buf {
            clear(b);
        }
        for side in 0..2 {
            for c in &mut self.reverb.combs[side] {
                clear(&mut c.buf);
                c.store = 0.0;
            }
            for a in &mut self.reverb.allpasses[side] {
                clear(&mut a.buf);
            }
        }
        *cursor += budget - left;
        if left > 0 {
            self.leaving = None;
            self.quiet = true;
            self.delay.time_now = self.delay.time;
            self.reverb.feedback_now = self.reverb.feedback;
            self.reverb.damp_now = self.reverb.damp;
            true
        } else {
            false
        }
    }

    /// One frame: the wet sound of the slot for this input.
    pub fn tick(&mut self, input: [f32; 2]) -> [f32; 2] {
        if self.quiet && input == [0.0; 2] && self.leaving.is_none() {
            return [0.0; 2];
        }
        self.quiet = false;
        let mut out = self.run(self.kind, input);
        if let Some((k, left)) = self.leaving {
            let g = left as f32 / self.fade_len as f32;
            let tail = self.run(k, [0.0; 2]);
            out[0] += tail[0] * g;
            out[1] += tail[1] * g;
            self.leaving = (left > 1).then_some((k, left - 1));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_optimized_drive_matches_the_previous_sound() {
        fn previous(x: f32, drive: f32) -> f32 {
            if drive <= 0.0 {
                return x;
            }
            let k = 1.0 + 24.0 * drive * drive;
            let shaped = (k * x).tanh() * (0.3 / (k * 0.3).tanh());
            let mix = (drive * 4.0).min(1.0);
            x + (shaped - x) * mix
        }
        let rate = 48_000;
        let frames = rate * 2;
        let mut old = Vec::with_capacity(frames);
        let mut new = Vec::with_capacity(frames);
        for i in 0..frames {
            let x = (i as f32 * 0.031).sin() * 0.15;
            let d = if i < rate {
                0.6
            } else {
                (i - rate) as f32 / rate as f32
            };
            let a = previous(x, d);
            let b = super::Drive::new(d).apply(x);
            assert!((a - b).abs() <= 0.000001);
            assert!(a.is_finite() && b.is_finite() && a.abs() < 1.0 && b.abs() < 1.0);
            old.push(a);
            new.push(b);
        }
        let mut both = old;
        both.extend(std::iter::repeat_n(0.0, rate / 2));
        both.extend(new);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../renders/81-drive-comparison.wav");
        crate::offline::write_wav(&path, rate as u32, &both, &both).unwrap();
    }

    #[test]
    fn drive_off_is_the_sample_itself() {
        for x in [-0.9f32, -0.1, 0.0, 0.25, 1.0] {
            assert_eq!(drive(x, 0.0), x);
        }
    }

    #[test]
    fn drive_keeps_the_reference_peak() {
        for d in [0.25f32, 0.5, 1.0] {
            let y = drive(DRIVE_REF, d);
            assert!((y - DRIVE_REF).abs() < 1e-4, "{d}: {y}");
        }
    }
}

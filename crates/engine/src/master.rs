//! Stereo master and cue protection before the device output.

use std::collections::VecDeque;

pub const CEILING: f32 = 0.966_050_86;
const TRUE_PEAK_HEADROOM: f32 = 0.92;
const MARK_GAIN: f32 = 0.891_250_9;
const TAPS: usize = 8;
const DETECTOR_DELAY: usize = TAPS / 2;

fn interpolation() -> [[f32; TAPS]; 3] {
    let sinc = |x: f32| {
        let angle = x * std::f32::consts::PI;
        if x == 0.0 { 1.0 } else { angle.sin() / angle }
    };
    std::array::from_fn(|phase| {
        let mut weights = std::array::from_fn(|i| {
            let x = i as f32 - (TAPS / 2 - 1) as f32 - (phase + 1) as f32 / 4.0;
            sinc(x) * sinc(x / (TAPS / 2) as f32)
        });
        let sum: f32 = weights.iter().sum();
        weights.iter_mut().for_each(|w| *w /= sum);
        weights
    })
}

/// Hold detector events through the delay and average the leading edge of a drop.
struct Envelope {
    minimum: VecDeque<(u64, f32)>,
    window: u64,
    attack: Vec<f32>,
    at: usize,
    sum: f64,
    gain: f32,
    release: f32,
}

impl Envelope {
    fn new(delay: usize, sample_rate: u32) -> Self {
        let window = delay + DETECTOR_DELAY + 2;
        let attack = (delay.saturating_sub(DETECTOR_DELAY) / 2).max(1);
        Self {
            minimum: VecDeque::with_capacity(window + 1),
            window: window as u64,
            attack: vec![1.0; attack],
            at: 0,
            sum: attack as f64,
            gain: 1.0,
            release: 1.0 - (-1.0 / (sample_rate.max(1) as f32 * 0.075)).exp(),
        }
    }

    fn tick(&mut self, frame: u64, bound: f32) -> f32 {
        while self
            .minimum
            .front()
            .is_some_and(|&(at, _)| frame - at >= self.window)
        {
            self.minimum.pop_front();
        }
        while self.minimum.back().is_some_and(|&(_, old)| old >= bound) {
            self.minimum.pop_back();
        }
        self.minimum.push_back((frame, bound));
        let floor = self.minimum.front().map_or(1.0, |&(_, floor)| floor);
        self.sum += f64::from(floor) - f64::from(self.attack[self.at]);
        self.attack[self.at] = floor;
        self.at += 1;
        if self.at == self.attack.len() {
            self.at = 0;
        }
        let target = (self.sum / self.attack.len() as f64) as f32;
        self.gain = target.min(self.gain + (target - self.gain).max(0.0) * self.release);
        self.gain
    }
}

pub struct Output {
    delay: Vec<[f32; 3]>,
    at: usize,
    frames: u64,
    detector: [[f32; TAPS]; 2],
    weights: [[f32; TAPS]; 3],
    master: Envelope,
    cue: Envelope,
}

impl Output {
    pub fn new(rate: u32) -> Self {
        let delay = u64::from(rate) * 3 + 1000;
        let delay = (delay / 2000) as usize;
        Self {
            delay: vec![[0.0; 3]; delay.max(1)],
            at: 0,
            frames: 0,
            detector: [[0.0; TAPS]; 2],
            weights: interpolation(),
            master: Envelope::new(delay.max(1), rate),
            cue: Envelope::new(delay.max(1), rate),
        }
    }

    pub fn latency(&self) -> usize {
        self.delay.len()
    }

    #[inline]
    fn cue_bound(left: f32, right: f32, cue: f32) -> f32 {
        let size = cue.abs();
        let peak = if cue.is_sign_negative() {
            -left.min(right)
        } else {
            left.max(right)
        };
        let room = 1.0 - peak;
        if size <= room {
            1.0
        } else {
            (room / size).max(0.0)
        }
    }

    /// Replace all three streams with aligned, protected output. The cue remains separate.
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], cue: &mut [f32]) -> bool {
        let mut limiting = false;
        for ((left, right), cue) in left.iter_mut().zip(right).zip(cue) {
            let input =
                [*left, *right, *cue].map(|value| if value.is_finite() { value } else { 0.0 });
            for (samples, next) in self.detector.iter_mut().zip(input) {
                samples.copy_within(1.., 0);
                samples[TAPS - 1] = next;
            }
            let sample_peak = input[0].abs().max(input[1].abs());
            let mut bound = if sample_peak > CEILING {
                CEILING / sample_peak
            } else {
                1.0
            };
            for samples in &self.detector {
                if samples.iter().any(|sample| sample.abs() > 0.5) {
                    for weights in &self.weights {
                        let peak = samples
                            .iter()
                            .zip(weights)
                            .map(|(s, w)| s * w)
                            .sum::<f32>()
                            .abs();
                        if peak > TRUE_PEAK_HEADROOM {
                            bound = bound.min(TRUE_PEAK_HEADROOM / peak);
                        }
                    }
                }
            }
            let gain = self.master.tick(self.frames, bound);
            let cue_bound = Self::cue_bound(
                input[0].clamp(-CEILING, CEILING),
                input[1].clamp(-CEILING, CEILING),
                input[2],
            );
            let cue_gain = self.cue.tick(self.frames, cue_bound);
            let delayed = std::mem::replace(&mut self.delay[self.at], input);
            self.at += 1;
            if self.at == self.delay.len() {
                self.at = 0;
            }
            self.frames += 1;
            *left = (delayed[0] * gain).clamp(-CEILING, CEILING);
            *right = (delayed[1] * gain).clamp(-CEILING, CEILING);
            *cue = delayed[2] * cue_gain.min(Self::cue_bound(*left, *right, delayed[2]));
            limiting |= gain < MARK_GAIN && (*left != 0.0 || *right != 0.0);
        }
        limiting
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_block_preserves_the_fixed_delay() {
        let mut output = Output::new(48_000);
        let mut left = [0.05; 140];
        let mut right = [-0.05; 140];
        let mut cue = [0.12; 140];
        assert!(!output.process(&mut [], &mut [], &mut []));
        assert!(!output.process(&mut left, &mut right, &mut cue));
        assert_eq!(output.latency(), 72);
        assert_eq!(&left[..72], &[0.0; 72]);
        assert_eq!(&left[72..], &[0.05; 68]);
        assert_eq!(&right[72..], &[-0.05; 68]);
        assert_eq!(&cue[72..], &[0.12; 68]);
    }
}

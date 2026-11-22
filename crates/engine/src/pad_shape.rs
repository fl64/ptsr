//! Stereo position, velocity response and timed envelopes for a pad.

use crate::{Engine, PadMode, Voice};

pub const PAN_LIMIT: i8 = 100;
pub const CENT_LIMIT: i8 = 100;
pub const HOLD_LIMIT_MS: u16 = 2000;
pub const VELOCITY_LIMIT_DB: u8 = 60;
pub const DEFAULT_VELOCITY_DB: u8 = 30;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Shape {
    pub pan: i8,
    pub cents: i8,
    pub hold_ms: u16,
    pub exponential: bool,
    /// None keeps the original linear amplitude response.
    pub velocity_db: Option<u8>,
}

impl Shape {
    pub fn bounded(self) -> Self {
        Self {
            pan: self.pan.clamp(-PAN_LIMIT, PAN_LIMIT),
            cents: self.cents.clamp(-CENT_LIMIT, CENT_LIMIT),
            hold_ms: self.hold_ms.min(HOLD_LIMIT_MS),
            velocity_db: self.velocity_db.map(|db| db.min(VELOCITY_LIMIT_DB)),
            ..self
        }
    }

    fn velocity_gain(self, velocity: f32) -> f32 {
        let velocity = velocity.clamp(0.0, 1.0);
        match (self.velocity_db, velocity) {
            (Some(db), velocity) if velocity > 0.0 => {
                let fraction = ((velocity * 127.0 - 1.0) / 126.0).max(0.0);
                10f32.powf(f32::from(db) * (fraction - 1.0) / 20.0)
            }
            _ => velocity,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Pan {
    mono: [f32; 2],
    stereo: [f32; 2],
}

impl Default for Pan {
    fn default() -> Self {
        Self {
            mono: [1.0; 2],
            stereo: [1.0; 2],
        }
    }
}

impl Pan {
    fn new(pan: i8) -> Self {
        let fraction = f32::from(pan) / f32::from(PAN_LIMIT);
        let power = (fraction * std::f32::consts::FRAC_PI_2).sin();
        Self {
            mono: [(1.0 - power).sqrt(), (1.0 + power).sqrt()],
            stereo: [1.0 - fraction.max(0.0), 1.0 + fraction.min(0.0)],
        }
    }

    pub(super) fn gains(self, stereo: bool) -> [f32; 2] {
        if stereo { self.stereo } else { self.mono }
    }
}

// 2 raised to this power gives the minus 60 dB amplitude.
const FLOOR_LOG2: f32 = -9.965_784;
const FLOOR_GAIN: f32 = 0.001;

fn exp_gain(progress: f32) -> f32 {
    (FLOOR_LOG2 * progress).exp2()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TimedEnvelope {
    attack: u32,
    hold: u32,
    decay: u32,
    tail: u32,
    exponential: bool,
}

impl TimedEnvelope {
    fn new(shape: Shape, attack: f32, decay: f32, sample_rate: f32) -> Option<Self> {
        (shape.exponential || shape.hold_ms > 0).then(|| Self {
            attack: attack.round() as u32,
            hold: (f32::from(shape.hold_ms) * sample_rate / 1000.0).round() as u32,
            decay: decay.round() as u32,
            tail: tail_frames(sample_rate),
            exponential: shape.exponential,
        })
    }

    pub(super) fn finished(self, age: u32) -> bool {
        self.decay != 0
            && age
                >= self.attack
                    + self.hold
                    + self.decay
                    + if self.exponential { self.tail } else { 0 }
    }

    pub(super) fn gain(self, age: u32) -> f32 {
        if age < self.attack {
            return age as f32 / self.attack as f32;
        }
        let elapsed = age.saturating_sub(self.attack + self.hold);
        if elapsed == 0 || self.decay == 0 {
            return 1.0;
        }
        if self.exponential {
            if elapsed <= self.decay {
                exp_gain(elapsed as f32 / self.decay as f32)
            } else {
                FLOOR_GAIN * self.tail.saturating_sub(elapsed - self.decay) as f32
                    / self.tail as f32
            }
        } else {
            self.decay.saturating_sub(elapsed) as f32 / self.decay as f32
        }
    }
}

pub(super) fn tail_frames(sample_rate: f32) -> u32 {
    ((sample_rate * 0.002).floor() as u32).max(1)
}

impl Voice {
    pub(super) fn fade_level(&self, left: u32) -> f32 {
        let tail = self.exp_fade_tail;
        if tail == 0 {
            return self.fade_gain * left as f32 / self.fade_total.max(1) as f32;
        }
        if left >= tail {
            let duration = self.fade_total - tail;
            self.fade_gain * exp_gain((self.fade_total - left) as f32 / duration as f32)
        } else {
            self.fade_gain * FLOOR_GAIN * left as f32 / tail as f32
        }
    }

    pub(super) fn start_gate_fade(&mut self, duration: u32, short: u32, tail: u32) {
        if self.fade.is_some() {
            return;
        }
        self.exp_fade_tail = if self.exponential_release && duration > 0 {
            tail
        } else {
            0
        };
        self.fade_total = if duration == 0 { short } else { duration } + self.exp_fade_tail;
        self.fade = Some(self.fade_total);
    }

    pub(super) fn shorten_fade(&mut self, short: u32) {
        if self.fade.is_none_or(|left| left > short) {
            if let Some(left) = self.fade {
                self.fade_gain = self.fade_level(left);
            }
            self.exp_fade_tail = 0;
            self.fade = Some(short);
            self.fade_total = short;
        }
    }

    pub(super) fn set_timed_envelope(&mut self, envelope: Option<TimedEnvelope>) {
        if self.timed != envelope {
            self.timed = envelope;
            self.env_correction = envelope.map_or(0.0, |e| self.env - e.gain(self.age));
        }
    }
}

impl Engine {
    pub(super) fn set_pad_shape(&mut self, pad: usize, shape: Shape) {
        let shape = shape.bounded();
        let state = &mut self.pads[pad];
        let retune = state.shape.cents != shape.cents;
        state.pan = Pan::new(shape.pan);
        state.shape = shape;
        if retune {
            self.retune_pad(pad);
        }
        self.refresh_pad_envelope(pad);
    }

    pub(super) fn retune_pad(&mut self, pad: usize) {
        let state = &mut self.pads[pad];
        let pitch = f32::from(state.pitch) + f32::from(state.shape.cents) / 100.0;
        let rate = 2f32.powf(pitch / 12.0);
        state.rate = rate;
        let glide = self.glide_len;
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.sound.is_some() && (v.pad == Some(pad) || v.as_pad == Some(pad)))
        {
            voice.rate = rate;
            voice.rate_step = (rate - voice.rate_now) / glide as f32;
            voice.glide_left = glide;
        }
    }

    pub(super) fn refresh_pad_envelope(&mut self, pad: usize) {
        let state = &mut self.pads[pad];
        let timed = TimedEnvelope::new(state.shape, state.attack, state.decay, self.sample_rate);
        state.timed = timed;
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.sound.is_some() && (v.pad == Some(pad) || v.as_pad == Some(pad)))
        {
            voice.attack = state.attack;
            voice.decay = state.decay;
            voice.exponential_release = state.shape.exponential;
            voice.set_timed_envelope(timed);
        }
    }

    pub(super) fn shape_velocity(&self, pad: Option<usize>, velocity: f32) -> f32 {
        pad.map_or_else(
            || velocity.clamp(0.0, 1.0),
            |pad| self.pads[pad].shape.velocity_gain(velocity),
        )
    }

    pub(super) fn release_manual_gate(&mut self, pad: usize) {
        if self.pads[pad].mode != PadMode::Gate {
            return;
        }
        let duration = self.pads[pad].release_frames;
        let tail = tail_frames(self.sample_rate);
        for voice in &mut self.voices {
            if voice.sound.is_some() && voice.pad == Some(pad) && !voice.sequenced {
                voice.start_gate_fade(duration, self.fade_len, tail);
            }
        }
    }
}

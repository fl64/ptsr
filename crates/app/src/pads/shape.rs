use engine::Command;
use engine::pad_shape::{CENT_LIMIT, HOLD_LIMIT_MS, PAN_LIMIT, VELOCITY_LIMIT_DB};

use super::{ENV_FINE_MS, ENV_STEP_MS, Pads, Param};

pub(crate) const SHAPE_PARAMS: [Param; 5] = [
    Param::Pan,
    Param::FineTune,
    Param::Velocity,
    Param::Curve,
    Param::Hold,
];

impl Param {
    fn shape_range(self) -> (i32, i32) {
        match self {
            Self::Pan => (-i32::from(PAN_LIMIT), i32::from(PAN_LIMIT)),
            Self::FineTune => (-i32::from(CENT_LIMIT), i32::from(CENT_LIMIT)),
            Self::Velocity => (-1, i32::from(VELOCITY_LIMIT_DB)),
            Self::Curve => (0, 1),
            Self::Hold => (0, i32::from(HOLD_LIMIT_MS)),
            _ => (0, 1),
        }
    }
}

impl Pads {
    pub(crate) fn shape_text(&self, pad: usize, param: Param) -> String {
        let s = self.play_shape(pad);
        match param {
            Param::Pan => match s.pan {
                0 => "center".into(),
                n if n < 0 => format!("L {} %", n.unsigned_abs()),
                n => format!("R {n} %"),
            },
            Param::FineTune => format!("{:+} ct", s.cents),
            Param::Velocity => s
                .velocity_db
                .map_or("Linear".into(), |db| format!("{db} dB")),
            Param::Curve => if s.exponential {
                "exponential"
            } else {
                "linear"
            }
            .into(),
            Param::Hold => format!("{} ms", s.hold_ms),
            _ => String::new(),
        }
    }

    pub(crate) fn pitch_text(&self, pad: usize) -> String {
        format!("{:+} st {:+} ct", self.pitch[pad], self.shape[pad].cents)
    }

    fn shape_value(&self, pad: usize, param: Param) -> i32 {
        let s = self.play_shape(pad);
        match param {
            Param::Pan => i32::from(s.pan),
            Param::FineTune => i32::from(s.cents),
            Param::Velocity => s.velocity_db.map_or(-1, i32::from),
            Param::Curve => i32::from(s.exponential),
            Param::Hold => i32::from(s.hold_ms),
            _ => 0,
        }
    }

    pub(crate) fn shape_pos(&self, pad: usize, param: Param) -> f32 {
        let (min, max) = param.shape_range();
        let pos = (self.shape_value(pad, param) - min) as f32 / (max - min) as f32;
        if param == Param::Hold {
            pos.sqrt()
        } else {
            pos
        }
    }

    fn set_shape(&mut self, pad: usize, param: Param, value: i32) -> Option<Command> {
        let (min, max) = param.shape_range();
        let value = value.clamp(min, max);
        if self.shape_value(pad, param) == value {
            return None;
        }
        let s = &mut self.shape[pad];
        match param {
            Param::Pan => s.pan = value as i8,
            Param::FineTune => s.cents = value as i8,
            Param::Velocity => s.velocity_db = Some(value as i8),
            Param::Curve => s.exponential = value != 0,
            Param::Hold => s.hold_ms = value as u16,
            _ => return None,
        }
        Some(self.shape_command(pad))
    }

    pub(crate) fn set_shape_pos(&mut self, param: Param, position: f32) -> Vec<Command> {
        let (min, max) = param.shape_range();
        let pos = position.clamp(0.0, 1.0);
        let pos = if param == Param::Hold { pos * pos } else { pos };
        let value = min + (pos * (max - min) as f32).round() as i32;
        let delta = value - self.shape_value(self.selected_abs(), param);
        self.scope_pads()
            .into_iter()
            .filter_map(|pad| {
                let value = if param == Param::Curve {
                    value
                } else {
                    self.shape_value(pad, param) + delta
                };
                self.set_shape(pad, param, value)
            })
            .collect()
    }

    pub(super) fn step_shape(
        &mut self,
        pad: usize,
        param: Param,
        step: i8,
        fine: bool,
    ) -> Option<Command> {
        let scale = match param {
            Param::FineTune if !fine => 5,
            Param::Hold if fine => ENV_FINE_MS,
            Param::Hold => ENV_STEP_MS,
            _ => 1,
        };
        self.set_shape(
            pad,
            param,
            self.shape_value(pad, param) + i32::from(step) * scale,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DisplayEnvelope {
    attack_ms: u16,
    decay_ms: u16,
    pub hold_ms: u16,
    pub exponential: bool,
    cents: i16,
}

impl DisplayEnvelope {
    pub(crate) fn timed(self) -> bool {
        self.exponential || self.hold_ms > 0
    }

    pub(super) fn duration(self, source_ms: f32) -> f32 {
        if self.timed() {
            source_ms / 2f32.powf(f32::from(self.cents) / 1200.0)
        } else {
            source_ms
        }
    }

    pub(super) fn curve(self, duration_ms: f32, width: usize) -> Vec<f32> {
        if !self.timed() {
            return crate::env_curve(duration_ms, self.attack_ms, self.decay_ms, width);
        }
        let attack = f32::from(self.attack_ms);
        let hold_end = attack + f32::from(self.hold_ms);
        let decay = f32::from(self.decay_ms);
        (0..width)
            .map(|col| {
                let at = (col as f32 + 0.5) * duration_ms / width as f32;
                if at < attack {
                    at / attack
                } else if at <= hold_end || decay == 0.0 {
                    1.0
                } else if !self.exponential {
                    (1.0 - (at - hold_end) / decay).max(0.0)
                } else {
                    let time = at - hold_end;
                    if time <= decay {
                        10f32.powf(-3.0 * time / decay)
                    } else {
                        0.001 * (1.0 - (time - decay) / 2.0).max(0.0)
                    }
                }
            })
            .collect()
    }
}

impl Pads {
    pub(crate) fn display_envelope(&self, abs: usize) -> DisplayEnvelope {
        let shape = self.shape[abs];
        DisplayEnvelope {
            attack_ms: self.attack_ms[abs],
            decay_ms: self.decay_ms[abs],
            hold_ms: shape.hold_ms,
            exponential: shape.exponential,
            cents: i16::from(self.pitch[abs]) * 100 + i16::from(shape.cents),
        }
    }
}

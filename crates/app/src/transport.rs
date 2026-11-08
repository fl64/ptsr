//! Project transport preferences and accepted audio-thread feedback.

use super::{App, Command};
use engine::transport::{DEFAULT_BPM, MAX_BPM, MIN_BPM, Snapshot};
use std::cell::Cell;
use std::sync::atomic::Ordering::Acquire;

pub(super) const TEMPO_TARGET: u64 = u64::MAX - 1;

pub(super) struct State {
    pub bpm: f64,
    pub metronome: bool,
    accepted: Cell<(u64, Snapshot)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            bpm: DEFAULT_BPM,
            metronome: false,
            accepted: Cell::new((0, Snapshot::default())),
        }
    }
}

impl App {
    pub(super) fn transport_state(&self) -> Snapshot {
        let fallback = Snapshot {
            bpm: self.transport.bpm,
            metronome: self.transport.metronome,
            ..Snapshot::default()
        };
        let Some(audio) = self.audio_alive() else {
            return fallback;
        };
        let epoch = audio.project.epoch.load(Acquire);
        if audio.project.ready.load(Acquire) != epoch {
            return fallback;
        }
        if let Some(state) = audio.stats.transport(epoch) {
            self.transport.accepted.set((epoch, state));
            return state;
        }
        let (cached_epoch, state) = self.transport.accepted.get();
        if cached_epoch == epoch {
            state
        } else {
            fallback
        }
    }

    pub(super) fn transport_input_blocked(&self) -> bool {
        let focus = self.focus();
        focus.mode.text
            || focus.mode.picker
            || focus.mode.screen_learn
            || focus.mode.midi_learn
            || focus.mode.question
            || self.restoring_project
            || (self.screen == super::Screen::Chop && self.plan.is_some())
            || self.audio.as_ref().is_some_and(|audio| {
                audio.project.epoch.load(Acquire) != audio.project.ready.load(Acquire)
            })
    }

    pub(super) fn set_tempo(&mut self, bpm: f64, external: bool) {
        let Some(bpm) = engine::transport::tempo(bpm) else {
            return;
        };
        if bpm != self.transport.bpm {
            self.transport.bpm = bpm;
            if external {
                self.takeover.reset_target(TEMPO_TARGET);
            }
            self.send(Command::SetTempo { bpm });
            self.dirty();
        }
    }

    pub(super) fn tempo_position(&self) -> f32 {
        ((self.transport.bpm - MIN_BPM) / (MAX_BPM - MIN_BPM)) as f32
    }
}

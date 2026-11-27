//! Project transport preferences and accepted audio-thread feedback.

use super::{Action, App, Command, Screen};
use engine::transport::{DEFAULT_BPM, MAX_BPM, MIN_BPM, Snapshot};
use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::atomic::Ordering::Acquire;
use std::time::{Duration, Instant};

pub(super) const TEMPO_TARGET: u64 = u64::MAX - 1;
const TAP_WINDOW: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, PartialEq, Eq)]
enum OutputStop {
    Pausing,
    SaveAfterPause,
}

pub(super) struct State {
    pub bpm: f64,
    pub metronome: bool,
    pub(super) taps: VecDeque<Instant>,
    pub(super) tap_input_at: Option<Instant>,
    accepted: Cell<(u64, Snapshot)>,
    output_stop: Option<OutputStop>,
    pub(super) feedback: Option<(usize, u64)>,
    pub(super) pattern_hits: [u64; engine::PADS],
    pub(super) dropped_hits: u64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            bpm: DEFAULT_BPM,
            metronome: false,
            taps: VecDeque::new(),
            tap_input_at: None,
            accepted: Cell::new((0, Snapshot::default())),
            output_stop: None,
            feedback: None,
            pattern_hits: [0; engine::PADS],
            dropped_hits: 0,
        }
    }
}

impl App {
    pub(super) fn stop_all_sound(&mut self) {
        if self.audio_alive().is_some() {
            self.pattern_stop_requested();
            self.send(Command::Panic);
            self.post_notice(
                super::Level::Warn,
                ("All sound stopped.".into(), Instant::now()),
            );
        }
        self.record_stop();
    }

    pub(super) fn context_button(&mut self, action: Action) {
        let stopping_output = action == Action::StopContext
            && self
                .recording
                .is_some_and(|(source, _)| source == super::audio::SOURCE_OUTPUT);
        let focus = self.focus();
        if focus.mode.text
            || focus.mode.picker
            || focus.mode.screen_learn
            || focus.mode.midi_learn
            || self.restoring_project
            || (self.screen == Screen::Chop && self.plan.is_some() && !stopping_output)
            || self.audio.as_ref().is_some_and(|audio| {
                audio.project.epoch.load(Acquire) != audio.project.ready.load(Acquire)
            })
        {
            return;
        }
        if stopping_output {
            self.stop_output_recording();
            return;
        }
        match (self.screen, action) {
            (Screen::Files, Action::PlayContext) if self.preview_frame().is_none() => {
                self.preview_selected_once();
            }
            (Screen::Chop, Action::PlayContext) if self.playhead().is_none() => {
                if let Some(chop) = &self.chop {
                    let region = chop.play_region();
                    let looped = chop.looping.then_some(region);
                    let as_pad = self.as_pad_of(chop.selected);
                    let sound = chop.sound.clone();
                    self.send(Command::PreviewLoop { region: looped });
                    self.send(Command::PreviewAs { pad: as_pad });
                    self.loop_sent = looped;
                    self.send(Command::Preview {
                        sound: Some(sound),
                        region,
                    });
                }
            }
            (Screen::Files | Screen::Chop, Action::PlayContext) => {}
            (Screen::Files, Action::StopContext) => {
                let playing = self.preview_frame().is_some();
                self.stop_preview();
                if !playing && !self.transport_state().playing {
                    self.send(Command::TransportStopContext);
                }
            }
            (Screen::Chop, Action::StopContext) if self.preview_frame().is_some() => {
                self.stop_preview();
            }
            (Screen::Chop, Action::StopContext) => {
                if !self.transport_state().playing {
                    self.send(Command::TransportStopContext);
                }
            }
            (_, Action::PlayContext) => self.send(Command::TransportPlayContext),
            (_, Action::StopContext) => self.send(Command::TransportStopContext),
            _ => {}
        }
    }

    fn stop_output_recording(&mut self) {
        let preview_playing = self.preview_frame().is_some();
        if preview_playing || self.screen == Screen::Files {
            self.stop_preview();
        }
        if preview_playing || self.recording.is_none() {
            return;
        }
        if !self.transport_state().playing {
            self.record_stop();
        } else if self.transport.output_stop.is_some() {
            // A second context stop would reset the position after the first pauses A1.
            self.transport.output_stop = Some(OutputStop::SaveAfterPause);
        } else if self.try_send(Command::TransportStopContext) {
            self.transport.output_stop = Some(OutputStop::Pausing);
        }
    }

    pub(super) fn observe_transport_command(&mut self, command: &Command) {
        match command {
            Command::TransportStop | Command::TransportStopContext => self.pattern_stop_requested(),
            Command::TransportToggle | Command::TransportPlayContext => {
                self.finish_output_stop();
                self.clear_output_stop();
            }
            _ => {}
        }
    }

    pub(super) fn finish_audio_feedback(&mut self) {
        self.finish_manual_save();
        self.finish_output_stop();
    }

    pub(super) fn clear_output_stop(&mut self) {
        self.transport.output_stop = None;
    }

    fn finish_output_stop(&mut self) {
        if self.transport.output_stop.is_none() {
            return;
        }
        let Some(audio) = self.audio_alive() else {
            return;
        };
        let epoch = audio.project.epoch.load(Acquire);
        let paused = audio.project.ready.load(Acquire) == epoch
            && audio
                .stats
                .transport(epoch)
                .is_some_and(|state| !state.playing);
        if paused && self.transport.output_stop.take() == Some(OutputStop::SaveAfterPause) {
            self.record_stop();
        }
    }

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

    pub(super) fn pattern_tempo_available(&self) -> bool {
        self.screen == Screen::Pads && (self.pattern_view.overview || self.pattern_panel.is_some())
    }

    pub(super) fn tap_tempo_hint_at(&self, now: Instant) -> Option<&'static str> {
        let taps = &self.transport.taps;
        (taps.len() == 1
            && self.pattern_tempo_available()
            && taps
                .back()
                .is_some_and(|at| now.saturating_duration_since(*at) <= TAP_WINDOW))
        .then_some("TAP · 1 hit · tap again")
    }

    pub(super) fn tap_pattern_tempo(&mut self) {
        if !self.pattern_tempo_available() || self.transport_input_blocked() || self.help_open {
            return;
        }
        let now = self.transport.tap_input_at.unwrap_or_else(Instant::now);
        let taps = &mut self.transport.taps;
        if let Some(previous) = taps.back() {
            let Some(gap) = now.checked_duration_since(*previous) else {
                return;
            };
            if gap < Duration::from_millis(200) {
                return;
            }
            if gap > TAP_WINDOW {
                taps.clear();
            }
        }
        taps.push_back(now);
        if taps.len() > 4 {
            taps.pop_front();
        }
        if taps.len() > 1 {
            let beats = (taps.len() - 1) as f64;
            let seconds = now.duration_since(*taps.front().unwrap()).as_secs_f64();
            self.apply_tempo(beats * 60.0 / seconds, true);
            self.post_notice(
                super::Level::Done,
                (
                    format!("Tempo {:.1} BPM", self.transport.bpm),
                    Instant::now(),
                ),
            );
        }
    }

    pub(super) fn set_tempo(&mut self, bpm: f64, external: bool) {
        self.transport.taps.clear();
        self.apply_tempo(bpm, external);
    }

    fn apply_tempo(&mut self, bpm: f64, external: bool) {
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

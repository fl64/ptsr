//! The mode line (134, 138): the mode, the item in hand, the next step and the way out, in the
//! words of the last input device. The way out is always the last part, so the narrow line
//! keeps it.

use super::{App, PADS_PER_BANK, audio, chop, pad_name};
use engine::capture::Phase;
use std::time::{Duration, Instant};

/// A first tap waits this long for the second one (transport, "Pattern accepts taps").
const TAP_WAIT: Duration = Duration::from_secs(2);

impl App {
    pub(super) fn mode_line(&self, now: Instant) -> Option<String> {
        let smc = self.input_sources.last_is_smc();
        let (cancel, apply, close) = if smc {
            (
                "PAD BANK + 1 cancel",
                "PAD BANK + 4 apply",
                "PAD BANK + 1 close",
            )
        } else {
            ("Esc cancel", "Enter apply", "Esc close")
        };
        if matches!(self.ask, Some(super::Ask::ExitRecovery { .. })) {
            // the save failure answers keep the words of its question (F33)
            return Some(if smc {
                "ASK · PAD BANK + 4 retry save · PAD BANK + 1 back".into()
            } else {
                "ASK · Enter retry save · Esc back".into()
            });
        }
        if self.ask.is_some() {
            return Some(if smc {
                "ASK · pad 4 yes · pad 1 no".into()
            } else {
                "ASK · Enter yes · Esc no".into()
            });
        }
        if self.screen_learn.is_some() || self.learning.is_some() {
            return Some(if smc {
                "LEARN · touch a control · pad 4 save · pad 1 cancel".into()
            } else {
                "LEARN · touch a control · Esc leave".into()
            });
        }
        if smc && self.panel.panic_armed(now) {
            let cancel = if self.midi_gestures.smc_pad_bank_opens_panel {
                "release PAD BANK"
            } else {
                "pad 1 back"
            };
            return Some(format!("STOP again · {cancel} to cancel"));
        }
        if let Some((source, at)) = self.recording {
            let seconds = now.saturating_duration_since(at).as_secs();
            return Some(if source == audio::SOURCE_OUTPUT {
                format!(
                    "● REC OUTPUT {}:{:02} · {}",
                    seconds / 60,
                    seconds % 60,
                    self.output_stop_words(smc)
                )
            } else {
                let stop = if smc {
                    "release RECORD"
                } else {
                    "release the key"
                };
                format!("● REC MIC 0:{seconds:02} · stop: {stop}")
            });
        }
        if self.screen == input::Screen::Chop
            && let (Some(plan), Some(chop)) = (&self.plan, &self.chop)
        {
            return Some(assign_line(plan, chop, apply, cancel));
        }
        let place = |what: String| Some(format!("{what} · hit a pad · {cancel}"));
        if self.erase_pads || self.pattern_erasing() {
            let off = if smc { "PAD BANK + 1 off" } else { "Esc off" };
            return Some(format!("ERASE A1 · hit a pad to erase its hits · {off}"));
        }
        if let Some(line) = self.a1_pass_line(smc) {
            return Some(line);
        }
        if let Some(path) = &self.browser.in_hand {
            return place(format!("IN HAND {}", super::file_name(path)));
        }
        if let Some((source, _)) = &self.copy_in_hand {
            return place(format!(
                "COPY {} {}",
                pad_name(&self.pads, *source),
                bank_pad(*source)
            ));
        }
        if let Some(source) = self.pad_in_hand {
            return place(format!("SWAP pad {}", tui::pads::pad_tag(source)));
        }
        if self.edit && self.screen == input::Screen::Pads {
            let (page, _) = self.pads.page();
            // with the panel shown LEFT and RIGHT turn its views (127)
            let arrows = match (smc, self.panel.visible()) {
                (true, true) => "LEFT RIGHT view",
                (true, false) => "LEFT RIGHT page",
                (false, _) => "Tab page",
            };
            return Some(format!(
                "EDIT pad {} · {page} · {arrows} · {close}",
                tui::pads::pad_tag(self.pads.selected)
            ));
        }
        if self.edit
            && self.screen == input::Screen::Chop
            && let Some(chop) = &self.chop
        {
            return Some(format!(
                "EDIT slice {} · BOUNDS · {close}",
                chop.selected + 1
            ));
        }
        if self.pattern_view.edit {
            return Some(format!(
                "STEP {} · hit a pad to add or remove · {close}",
                self.pattern_view.cursor_cell + 1
            ));
        }
        if self.pattern_panel.is_some() {
            return Some(if smc {
                format!("A1 SETUP · knob 3 quantize · knob 4 strength · knob 5 bars · {close}")
            } else {
                format!("A1 SETUP · I K select · J L change · {close}")
            });
        }
        if self.pads.marking {
            let drop = if smc { "PAD BANK + 1 drop" } else { "Esc drop" };
            let marked = self.pads.marked.len();
            return Some(format!("MARK · hit pads · {marked} marked · {drop}"));
        }
        let taps = &self.transport.taps;
        if taps.len() == 1
            && taps
                .back()
                .is_some_and(|at| now.saturating_duration_since(*at) <= TAP_WAIT)
        {
            return Some("TAP · 1 hit · tap again".into());
        }
        (smc && self.panel.visible()).then(|| "STOP twice · stop all sound".into())
    }

    /// The A1 count-in and pass on the screen where they started (F45).
    fn a1_pass_line(&self, smc: bool) -> Option<String> {
        let state = self.pattern_record_state()?;
        match state.phase {
            Phase::CountIn => Some(format!("COUNT {}", state.count_in_beat)),
            Phase::Recording => Some(format!(
                "REC A1 · hit pads · {}",
                if smc { "RECORD stop" } else { "Ctrl+R stop" }
            )),
            _ => None,
        }
    }

    /// STOP first pauses what plays, so the recording keeps the tail; then it ends (138).
    pub(super) fn output_stop_words(&self, smc: bool) -> &'static str {
        if smc && self.panel.visible() {
            return "STOP twice · stop and save";
        }
        let playing = self.transport_state().playing || self.preview_frame().is_some();
        match (smc, playing) {
            (true, true) => "STOP stops play · STOP end rec",
            (true, false) => "STOP end rec",
            (false, _) => "Ctrl+T end rec",
        }
    }
}

/// The source of a copy as bank and pad: `B1/P1`, or `B1/Q` with key names.
fn bank_pad(abs: usize) -> String {
    let tag = tui::pads::pad_tag(abs);
    let prefix = if tui::pads::device_numbers() { "P" } else { "" };
    format!("B{}/{prefix}{tag}", abs / PADS_PER_BANK + 1)
}

fn assign_line(plan: &chop::Plan, chop: &chop::Chop, apply: &str, cancel: &str) -> String {
    let count = chop.slices().len();
    let placed = plan.targets(count).len();
    let next = match plan {
        chop::Plan::InOrder(None) => "hit the start pad".to_string(),
        _ => {
            let selected = chop.selected;
            let next = if plan
                .targets(count)
                .iter()
                .any(|(slice, _)| *slice == selected)
            {
                plan.next_unplanned(count, selected)
            } else {
                Some(selected)
            };
            next.map_or("all placed".into(), |slice| {
                format!("slice {} next · hit a pad", slice + 1)
            })
        }
    };
    format!("ASSIGN {placed}/{count} · {next} · {apply} · {cancel}")
}

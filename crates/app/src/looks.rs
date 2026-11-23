//! The looks of the panel cells (138): a grey cell cannot change anything now and a press
//! says why; an orange cell replaces or removes the owner's work.

use crate::panel::View;
use crate::*;

/// Why a cell is grey.
pub(super) enum Grey {
    /// The notice of a grey press.
    Why(String),
    /// The action refuses with its own notice and changes nothing: it runs.
    Itself,
}

impl App {
    /// Why the action of a panel cell of `view` cannot change anything now; `None` acts.
    pub(super) fn grey(&self, view: View, action: Action) -> Option<Grey> {
        let why = |text: &str| Some(Grey::Why(text.into()));
        let abs = self.pads.selected_abs();
        match (view, action) {
            (View::PadEdit, Action::PadsSourcePin) => {
                (!self.source_pin_available()).then(|| Grey::Why("select a loaded sample".into()))
            }
            (_, Action::Undo) => (!self.undo_waits()).then(|| Grey::Why("nothing to undo".into())),
            (_, Action::Redo) => self.redo_refusal().map(|text| Grey::Why(text.into())),
            (View::Pads | View::PadEdit, Action::PadCopy) => (self.pads.sound[abs].is_none()
                || self.pads.missing[abs]
                || self.pads.loading[abs].is_some())
            .then_some(Grey::Itself),
            (View::Pads, Action::PadTake) => self
                .swap_source_refusal(abs)
                .map(|reason| Grey::Why(reason.into())),
            (View::Pads, Action::ScreenChop) => {
                self.pads.sound[abs].is_none().then_some(Grey::Itself)
            }
            (View::Pads, Action::PadClear) if self.pads.marked.is_empty() => {
                if self.pads.loading[abs].is_some() {
                    why("pad is still loading")
                } else if self.pads.file[abs].is_none() {
                    why("the pad is empty")
                } else {
                    None
                }
            }
            (View::PadEdit, Action::PadClear) => (!self
                .pads
                .scope_pads()
                .into_iter()
                .any(|a| self.pads.file[a].is_some()))
            .then(|| Grey::Why("nothing to reset".into())),
            (View::Pattern, Action::PatternViewPrev | Action::PatternViewNext) => {
                let finer = action == Action::PatternViewNext;
                self.pattern_view.division.at_limit(finer).then(|| {
                    Grey::Why(format!(
                        "steps are already the {}",
                        if finer { "finest" } else { "widest" }
                    ))
                })
            }
            (
                View::Pattern,
                Action::PatternStepEdit | Action::PatternErase | Action::PatternClear,
            ) => self.pattern_grey(action).map(|text| Grey::Why(text.into())),
            (View::Files, Action::BrowserLoad | Action::BrowserHold | Action::ScreenChop) => {
                if action == Action::BrowserHold && self.browser.in_hand.is_some() {
                    return None;
                }
                match self.browser.current() {
                    None => why("select a file"),
                    Some(entry) if entry.is_dir => why("RIGHT opens the folder"),
                    Some(_) => {
                        let path = self.browser.selected_file()?;
                        (action == Action::BrowserLoad && same_whole_sound(&self.pads, abs, &path))
                            .then(|| Grey::Why(format!("{} already lies there", file_name(&path))))
                    }
                }
            }
            (View::Chop, Action::ChopCommit) if self.apply_waits() => {
                why("hit the start pad first")
            }
            (View::Chop, Action::ChopCut) => {
                let c = self.chop.as_ref()?;
                c.cut_refusal(self.playhead().unwrap_or(c.cursor))
                    .map(|text| Grey::Why(text.into()))
            }
            (View::Chop, Action::ChopCutsClear) => self
                .chop
                .as_ref()?
                .markers
                .is_empty()
                .then(|| Grey::Why("no cuts to remove".into())),
            (View::Chop, Action::ChopTrimSilence) => self
                .chop
                .as_ref()?
                .trim_silence_refusal()
                .map(|text| Grey::Why(text.into())),
            (View::Chop, Action::ChopHitsBack) => {
                self.chop.as_ref()?.hits_back_refusal().map(Grey::Why)
            }
            (View::ChopEdit, Action::ChopNormalize) => self
                .slice_pads()
                .is_empty()
                .then(|| Grey::Why("put the slice on a pad first".into())),
            (View::ChopEdit, Action::MarkerDelete | Action::SliceEndDelete) => {
                self.join_blocked(Some(action)).then_some(Grey::Itself)
            }
            _ => None,
        }
    }

    /// An undo step waits on this screen (138).
    fn undo_waits(&self) -> bool {
        match self.screen {
            Screen::Chop if self.plan.is_some() => !self.plan_undo.is_empty(),
            Screen::Chop => {
                self.chop.as_ref().is_some_and(chop::Chop::can_undo)
                    || self.fresh_undo.is_some()
                    || self.norm_is_next()
            }
            Screen::Pads | Screen::Files | Screen::Fx => {
                !self.history.is_empty()
                    || self
                        .audio
                        .as_ref()
                        .is_some_and(|audio| audio.pattern_capture.undo_possible())
            }
            _ => true,
        }
    }

    /// The availability of the A1 edit, erase and clear cells.
    fn pattern_grey(&self, action: Action) -> Option<&'static str> {
        use engine::capture::Phase;
        if action == Action::PatternStepEdit {
            return self.step_edit_refusal();
        }
        let phase = self.pattern_record_state().map(|s| s.phase);
        if action == Action::PatternErase {
            if self.erase_pads {
                return None;
            }
            if phase == Some(Phase::CountIn) {
                return Some("wait for the count-in");
            }
        }
        let hits = self
            .pads
            .pattern
            .as_ref()
            .is_some_and(|p| !p.hits.is_empty());
        (!hits && phase != Some(Phase::Recording)).then_some("A1 has no hits yet")
    }

    /// The current state that a state cell shows under its label (138).
    pub(super) fn cell_state(&self, action: Action) -> Option<&'static str> {
        let on = |on: bool| Some(if on { "on" } else { "off" });
        let abs = self.pads.selected_abs();
        match action {
            Action::PadsStripToggle => Some(self.source_view.view_name()),
            Action::PadsSourcePin => on(self.source_view.pinned(&self.pads)),
            Action::ChopPlayMode => self.chop.as_ref().map(|c| c.play.name()),
            Action::ChopLoopToggle => self.chop.as_ref().and_then(|c| on(c.looping)),
            Action::ChopSnapToggle => self.chop.as_ref().map(|c| match c.snap {
                chop::Snap::Off => "even",
                snap => snap.name(),
            }),
            Action::ChopAsPadToggle => self.chop.as_ref().and_then(|c| on(c.as_pad)),
            Action::PadModeNext => Some(if self.pads.gate[abs] {
                "gate"
            } else {
                "1 shot"
            }),
            Action::PadPitchGateToggle => on(self.pads.pitch_gate[abs]),
            Action::PadMarkToggle => on(self.pads.marking),
            Action::PatternOverviewEmptyToggle => Some(if self.pattern_view.hide_empty_banks {
                "hidden"
            } else {
                "shown"
            }),
            Action::BrowserPreviewToggle => on(self.preview_on),
            Action::BrowserFavorite => on(self.browser.favorite_now()),
            _ => None,
        }
    }

    /// The action of a panel cell replaces or removes the owner's work (138).
    pub(super) fn warns(view: View, action: Action) -> bool {
        matches!(
            (view, action),
            (View::Pads | View::PadEdit, Action::PadClear)
                | (View::Pattern, Action::PatternClear)
                | (View::Files, Action::BrowserLoad)
                | (
                    View::Chop,
                    Action::ChopParts
                        | Action::ChopEqualParts
                        | Action::ChopHits
                        | Action::ChopHitsBack
                        | Action::ChopCutsClear
                        | Action::ChopTrimSilence
                )
        )
    }
}

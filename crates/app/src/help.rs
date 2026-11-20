//! Local F1 navigation. Ordinary input retains its production route.

use super::{Action, App, Keymap, Screen, event};
use super::{NUDGE_FINE_MS, NUDGE_MS};

pub(super) fn describe(a: Action) -> String {
    match a {
        Action::PadPress { .. } | Action::PadRelease { .. } => "play pad".into(),
        Action::RecordMic(_) => "record the microphone: hold".into(),
        Action::RecordOutputToggle => "record the output: on / off".into(),
        Action::PatternRecord => {
            "record A1: count-in when stopped, overdub when playing. Press again to finish.".into()
        }
        Action::PatternErase => {
            "in a pass, held pads erase under the playhead; on Pads, a pad erases all its hits"
                .into()
        }
        Action::PatternUndo => "undo the A1 pass. Finish an active pass first. No redo.".into(),
        Action::PatternControls => {
            "pattern keys: A1 controls. Quantize and Strength set the next hits, not saved hits."
                .into()
        }
        Action::PatternParamPrev => "select the previous A1 control".into(),
        Action::PatternParamNext => "select the next A1 control".into(),
        Action::PatternValueDec => "decrease the selected A1 value with controls open".into(),
        Action::PatternValueInc => "increase the selected A1 value with controls open".into(),
        Action::PatternViewPrev => {
            "Pads or A1 overview: previous view division. Recorded timing stays unchanged.".into()
        }
        Action::PatternViewNext => {
            "Pads or A1 overview: next view division. Recorded timing stays unchanged.".into()
        }
        Action::PatternOverviewEmptyToggle => {
            "A1 overview: show or hide empty bank sections.".into()
        }
        Action::PatternStepEdit => "A1 overview: enter or leave step correction.".into(),
        Action::PatternClear => "clear all of A1 after a question. Ctrl+Z restores.".into(),
        Action::CursorMove { right, .. } => {
            format!(
                "chop: move the cursor {}",
                if right { "right" } else { "left" }
            )
        }
        Action::BankNext => "next bank".into(),
        Action::SmcList { next: false } => "previous item in the main list".into(),
        Action::SmcList { next: true } => "next item in the main list".into(),
        Action::BankPrev => "previous bank".into(),
        Action::BankSelect(b) => format!("bank {}", b + 1),
        Action::MasterVolume(s) if s < 0 => "master volume down 1 dB".into(),
        Action::MasterVolume(_) => "master volume up 1 dB".into(),
        Action::PadTake => "take the selected pad; the next pad hit swaps them".into(),
        Action::PadCopy => "copy the selected pad; the next pad hit gets a copy".into(),
        Action::PadRename => "pads: name the selected pad".into(),
        Action::PadSelect(d) => format!("select pad {}", format!("{d:?}").to_lowercase()),
        Action::PadModeNext => "pad mode: one-shot / gate".into(),
        Action::PadPitchGateToggle => "Pitch Gate: on / off for sequenced pitched slices".into(),
        Action::PadEdit => "pads: edit the selected pad; Esc closes".into(),
        Action::PadEditOnly => "pads: open edit only; do not place or confirm".into(),
        Action::PadParamNext => "pads: the next parameter".into(),
        Action::PadParam(s) if s < 0 => "pad: the picked parameter down a step".into(),
        Action::PadParam(_) => "pad: the picked parameter up a step".into(),
        Action::PadMarkBank => "pads: mark bank; again unmarks it".into(),
        Action::PadMarkToggle => "pads: marking on / off; a pad hit marks or unmarks it".into(),
        Action::PadClear => {
            "pads: take the sound off the selected pad; in the edit, reset its settings".into()
        }
        Action::ScreenPads => "pads screen".into(),
        Action::ScreenPattern => "pattern overview".into(),
        Action::ScreenFx => "shared FX screen".into(),
        Action::ScreenSettings => "audio settings".into(),
        Action::ScreenFiles => "files screen".into(),
        Action::ScreenChop => "chop the selected pad (or file on F6)".into(),
        Action::ChopPlay => "chop: play from the cursor / stop".into(),
        Action::ChopPlayMode => "chop: play to the end / to the cut / the slice".into(),
        Action::ChopLoopToggle => "chop: Space loops the play until the next Space".into(),
        Action::ChopAsPadToggle => {
            "chop: play a slice as its pad: pitch, volume, envelope, filter".into()
        }
        Action::CursorJump { right } => format!(
            "chop: the cursor to the {} cut",
            if right { "next" } else { "previous" }
        ),
        Action::ChopZoom { zoom_in: true } => "chop: zoom the wave in".into(),
        Action::ChopZoom { zoom_in: false } => "chop: zoom the wave out".into(),
        Action::ChopRegionStart => "chop: the region starts at the cursor".into(),
        Action::ChopRegionEnd => "chop: the region ends at the cursor".into(),
        Action::ChopRegionClear => "chop: the region is the whole sample".into(),
        Action::ChopSnapToggle => "chop: cursor in even steps / hit to hit / cut to cut".into(),
        Action::ChopNormalize => "chop: the pad volume puts its peak at −1 dB".into(),
        Action::ChopCut => "chop: cut at the cursor".into(),
        Action::ChopParts => {
            "chop: equal parts 2, 4, 8, 16, 32; assigning: in order / by hand".into()
        }
        Action::ChopEqualParts => "chop: equal parts 2, 4, 8, 16, 32".into(),
        Action::ChopHits => "chop: by hits, one threshold step lower per press (−6…−30 dB)".into(),
        Action::ChopHitsBack => "chop: by hits, one threshold step back (to −6 dB)".into(),
        Action::ThresholdStep(step) if step < 0 => "chop: hit threshold down 1 dB".into(),
        Action::ThresholdStep(_) => "chop: hit threshold up 1 dB".into(),
        Action::ChopAssignOrder => "assign: in order / by hand".into(),
        Action::SliceBound { end, right, .. } => format!(
            "chop: the slice {} {}",
            if end { "end" } else { "start" },
            if right { "right" } else { "left" }
        ),
        Action::ParamNext => "the next parameter; assigning: in order / by hand".into(),
        Action::ParamPrev => "the previous parameter".into(),
        Action::PageNext => "the next page of the edit".into(),
        Action::PagePrev => "the previous page of the edit".into(),
        Action::PadPage(page) => format!(
            "pad edit: the {} page",
            ["TONE", "ENV", "FILTER", "SOURCE", "FX"][usize::from(page).min(4)]
        ),
        Action::EditValue { up, fine } => format!(
            "the picked value {}{}",
            if up { "up" } else { "down" },
            if fine { ", fine" } else { "" }
        ),
        Action::ChopCommit => "chop: assign mode; in it, apply the plan".into(),
        Action::ChopEdit => "chop: edit the selected slice; assigning: apply the plan".into(),
        Action::ChopEditOnly => "chop: open slice edit only when no plan is pending".into(),
        Action::ChopAssign => "chop: the assign mode by hand from slice 1".into(),
        Action::MarkerSelect(s) if s < 0 => "chop: previous slice".into(),
        Action::MarkerSelect(_) => "chop: next slice".into(),
        Action::MarkerNudge { right, fine } => format!(
            "chop: move the slice start {} {} ms",
            if right { "right" } else { "left" },
            if fine { NUDGE_FINE_MS } else { NUDGE_MS }
        ),
        Action::MarkerDelete => "chop: join the slice with the previous one".into(),
        Action::SliceEndDelete => "chop: join the slice with the next one".into(),
        Action::Undo => "undo the last change".into(),
        Action::BrowserUp => "file up".into(),
        Action::BrowserDown => "file down".into(),
        Action::BrowserEnter => "open folder".into(),
        Action::BrowserParent => "parent folder".into(),
        Action::BrowserHold => "take file in hand / open folder".into(),
        Action::BrowserTakeOnly => {
            "files: take file or open folder; do not place or confirm".into()
        }
        Action::BrowserLoad => "the file loads onto the selected pad; a folder opens".into(),
        Action::BrowserPreviewToggle => "play the selected file: on / off".into(),
        Action::BrowserFilter => "filter the list by name".into(),
        Action::BrowserFavorite => "the selected folder: favorite on / off".into(),
        Action::HelpToggle => "this help".into(),
        Action::HelpPagePrev => "F1: previous key-table page".into(),
        Action::HelpPageNext => "F1: next key-table page".into(),
        Action::LearnToggle => "bind controller knobs and buttons to what the screen shows".into(),
        Action::ProjectPick => "open the project picker".into(),
        Action::PadsToggle => "hide the pad grid, the wave takes its place".into(),
        Action::Confirm => "confirm the open question or picker".into(),
        Action::Back => "back".into(),
        Action::Quit => "quit".into(),
        Action::SessionSave => "save the session now".into(),
        Action::Click => "test click".into(),
        Action::TransportToggle => "transport: play / stop".into(),
        Action::PlayContext => "start or continue transport, or start a stopped preview".into(),
        Action::StopContext => "pause transport or stop a preview; press again to reset".into(),
        Action::MetronomeToggle => "metronome: on / off".into(),
        Action::TapTempo => "tap tempo in Pattern".into(),
        Action::TempoStep { up, fine } => format!(
            "tempo {} {} BPM",
            if up { "up" } else { "down" },
            if fine { "0.1" } else { "1" }
        ),
        Action::SettingsRow(_) => "select row".into(),
        Action::SettingsValue(_) => "change value".into(),
        Action::SettingsTab => "switch tab: audio / input / MIDI".into(),
        Action::SettingsLearn => "MIDI tab: learn a control for the action".into(),
        Action::SettingsClear => "MIDI tab: clear the learned control".into(),
    }
}

/// The F1 tabs, in Tab order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Page {
    Help,
    Messages,
    Controller,
}

pub(super) struct State {
    pub page: Page,
    pub controller: super::controller_map::State,
    /// The Controller tab of this frame, built with the keys of the frame.
    pub controller_view: Option<tui::Controller>,
    pub scroll: usize,
    pub keys_scroll: usize,
    pub caption: Option<String>,
    pub navigation: String,
    key_rows: usize,
    pending_pages: Vec<bool>,
    pub size: (u16, u16),
}

impl Default for State {
    fn default() -> Self {
        Self {
            page: Page::Help,
            controller: Default::default(),
            controller_view: None,
            scroll: 0,
            keys_scroll: 0,
            caption: None,
            navigation: String::new(),
            key_rows: 0,
            pending_pages: Vec::new(),
            size: (80, 26),
        }
    }
}

impl State {
    pub fn messages(&self) -> bool {
        self.page == Page::Messages
    }
}

impl App {
    pub(super) fn toggle_help(&mut self) {
        self.help_open = !self.help_open;
        self.help_overlay.pending_pages.clear();
        if self.help_open {
            self.help_overlay.page = Page::Help;
            self.help_overlay.scroll = 0;
            self.help_overlay.keys_scroll = 0;
            self.help_overlay.key_rows = 0;
        }
    }

    pub(super) fn prepare_key_help(
        &mut self,
        keymap: &Keymap,
        rows: &[(String, String)],
        recording: Option<engine::capture::Snapshot>,
    ) {
        self.help_overlay.caption = self.help_recording_caption(recording);
        self.help_overlay.key_rows =
            tui::help_key_lines(rows, self.help_overlay.size.0.saturating_sub(2)).len();
        self.help_overlay.keys_scroll = self.help_overlay.keys_scroll.min(self.help_keys_max());
        for next in std::mem::take(&mut self.help_overlay.pending_pages) {
            self.step_key_page(next);
        }
        self.help_overlay.scroll = self.help_overlay.scroll.min(self.help_scroll_max());
        let key = |action| {
            self.help_bindings(keymap, action)
                .into_iter()
                .next()
                .unwrap_or_else(|| "none".into())
        };
        self.help_overlay.navigation = format!(
            "{} / {} page",
            key(Action::HelpPagePrev),
            key(Action::HelpPageNext)
        );
        self.help_overlay.controller_view = (self.help_open
            && self.help_overlay.page == Page::Controller)
            .then(|| self.controller_view(keymap));
    }

    pub(super) fn help_bindings(&self, keymap: &Keymap, action: Action) -> Vec<String> {
        let edit = self.edit_keys() || self.pattern_panel.is_some();
        let mut keys = keymap.keys_for(self.screen, action);
        if edit {
            keys.extend(keymap.edit_keys_for(action));
        }
        keys.retain(|key| {
            // Explicit lookup takes config names, not the help's display glyphs.
            let chord = key
                .split('+')
                .map(|name| match name {
                    "↑" => "Up",
                    "↓" => "Down",
                    "←" => "Left",
                    "→" => "Right",
                    "PgUp" => "PageUp",
                    "PgDn" => "PageDown",
                    "Bksp" => "Backspace",
                    name => name,
                })
                .collect::<Vec<_>>()
                .join("+");
            keymap.binding_for(self.screen, edit, &chord) == Some(Some(action))
        });
        keys.sort();
        keys.dedup();
        keys
    }

    fn help_navigation_allowed(&self) -> bool {
        self.help_open
            && self.ask.is_none()
            && self.path_picker.is_none()
            && self.project_picker.is_none()
            && self.pad_naming.is_none()
            && !(self.screen == Screen::Files && self.browser.editing)
            && self.screen_learn.is_none()
            && self.learning.is_none()
            && self.midi_run.is_none()
    }

    fn help_keys_max(&self) -> usize {
        self.help_overlay
            .key_rows
            .saturating_sub(tui::help_page_height(
                self.help_overlay.size.1,
                self.help_overlay.caption.is_some(),
            ))
    }

    pub(super) fn help_page(&mut self, next: bool) {
        if !self.help_navigation_allowed() || self.help_overlay.messages() {
            return;
        }
        if self.help_overlay.page == Page::Controller {
            self.controller_page(next);
            return;
        }
        if self.help_overlay.key_rows == 0 {
            self.help_overlay.pending_pages.push(next);
        } else {
            self.step_key_page(next);
        }
    }

    pub(super) fn help_panel_cell(&mut self, cell: u8) {
        use event::{KeyCode as K, KeyEvent, KeyModifiers as M};
        if !self.help_navigation_allowed() {
            return;
        }
        match cell {
            9 | 10 if self.help_overlay.page == Page::Help => self.help_page(cell == 10),
            9 | 10 => {
                let code = match (self.help_overlay.page, cell) {
                    (Page::Messages, 9) => K::PageUp,
                    (Page::Messages, _) => K::PageDown,
                    (_, 9) => K::Up,
                    _ => K::Down,
                };
                self.help_key(&KeyEvent::new(code, M::NONE));
            }
            _ => {}
        }
    }

    /// LEFT and RIGHT select the Help tabs (134).
    pub(super) fn help_tab(&mut self, next: bool) {
        use event::{KeyCode as K, KeyEvent, KeyModifiers as M};
        if self.help_navigation_allowed() {
            let modifiers = if next { M::NONE } else { M::SHIFT };
            self.help_key(&KeyEvent::new(K::Tab, modifiers));
        }
    }

    fn step_key_page(&mut self, next: bool) {
        let current = self.help_overlay.keys_scroll.min(self.help_keys_max());
        let step = tui::help_page_height(
            self.help_overlay.size.1,
            self.help_overlay.caption.is_some(),
        )
        .max(1);
        self.help_overlay.keys_scroll = if next {
            current.saturating_add(step).min(self.help_keys_max())
        } else {
            current.saturating_sub(step)
        };
    }

    pub(super) fn resize_help(&mut self, width: u16, height: u16) {
        self.help_overlay.size = (width, height);
        if self.help_open && self.help_overlay.messages() {
            self.help_overlay.scroll = self.help_overlay.scroll.min(self.help_scroll_max());
        }
    }

    fn help_scroll_max(&self) -> usize {
        let (width, height) = self.help_overlay.size;
        tui::message_lines(&self.message_history(), width.saturating_sub(2))
            .len()
            .saturating_sub(tui::help_page_height(
                height,
                self.help_overlay.caption.is_some(),
            ))
    }

    pub(super) fn help_key(&mut self, key: &event::KeyEvent) -> bool {
        use event::{KeyCode as K, KeyEventKind as Kind, KeyModifiers as M};
        if !self.help_navigation_allowed() || key.kind == Kind::Release {
            return false;
        }
        match (key.code, key.modifiers) {
            (code @ (K::Tab | K::BackTab), mods @ (M::NONE | M::SHIFT)) => {
                if key.kind == Kind::Press {
                    let back = code == K::BackTab || mods == M::SHIFT;
                    self.help_overlay.page = match (self.help_overlay.page, back) {
                        (Page::Help, false) | (Page::Controller, true) => Page::Messages,
                        (Page::Messages, false) | (Page::Help, true) => Page::Controller,
                        (Page::Controller, false) | (Page::Messages, true) => Page::Help,
                    };
                    if self.help_overlay.page == Page::Controller {
                        self.controller_start();
                    }
                    self.help_overlay.scroll = self.help_overlay.scroll.min(self.help_scroll_max());
                }
            }
            (code @ (K::Up | K::Down | K::Left | K::Right), M::NONE)
                if self.help_overlay.page == Page::Controller =>
            {
                let (dr, dc) = match code {
                    K::Up => (-1, 0),
                    K::Down => (1, 0),
                    K::Left => (0, -1),
                    _ => (0, 1),
                };
                self.controller_select(dr, dc);
            }
            (K::F(1) | K::Esc, M::NONE) => {
                if key.kind == Kind::Press {
                    self.help_open = false;
                    self.help_overlay.pending_pages.clear();
                }
            }
            (code @ (K::Up | K::Down | K::PageUp | K::PageDown), M::NONE)
                if self.help_overlay.messages() =>
            {
                let step = if matches!(code, K::PageUp | K::PageDown) {
                    tui::help_page_height(
                        self.help_overlay.size.1,
                        self.help_overlay.caption.is_some(),
                    )
                    .max(1)
                } else {
                    1
                };
                let current = self.help_overlay.scroll.min(self.help_scroll_max());
                self.help_overlay.scroll = if matches!(code, K::Up | K::PageUp) {
                    current.saturating_sub(step)
                } else {
                    current.saturating_add(step).min(self.help_scroll_max())
                };
            }
            _ => return false,
        }
        true
    }
}

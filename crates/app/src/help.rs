//! Local F1 navigation. Ordinary input retains its production route.

use super::{Action, App, Keymap, Screen, event};

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

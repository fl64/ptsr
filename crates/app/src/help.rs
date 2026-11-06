//! Local F1 navigation. Ordinary input retains its production route.

use super::{App, event};

pub(super) struct State {
    pub messages: bool,
    pub scroll: usize,
    size: (u16, u16),
}

impl Default for State {
    fn default() -> Self {
        Self {
            messages: false,
            scroll: 0,
            size: (80, 26),
        }
    }
}

impl App {
    pub(super) fn toggle_help(&mut self) {
        self.help_open = !self.help_open;
        if self.help_open {
            self.help_overlay.messages = false;
            self.help_overlay.scroll = 0;
        }
    }

    pub(super) fn resize_help(&mut self, width: u16, height: u16) {
        self.help_overlay.size = (width, height);
        if self.help_open && self.help_overlay.messages {
            self.help_overlay.scroll = self.help_overlay.scroll.min(self.help_scroll_max());
        }
    }

    fn help_scroll_max(&self) -> usize {
        let (width, height) = self.help_overlay.size;
        tui::message_lines(&self.message_history(), width.saturating_sub(2))
            .len()
            .saturating_sub(tui::messages_page_height(height))
    }

    pub(super) fn help_key(&mut self, key: &event::KeyEvent) -> bool {
        use event::{KeyCode as K, KeyEventKind as Kind, KeyModifiers as M};
        if !self.help_open
            || self.ask.is_some()
            || self.path_picker.is_some()
            || self.screen_learn.is_some()
            || self.learning.is_some()
            || self.midi_run.is_some()
            || key.kind == Kind::Release
        {
            return false;
        }
        match (key.code, key.modifiers) {
            (K::Tab | K::BackTab, M::NONE | M::SHIFT) => {
                if key.kind == Kind::Press {
                    self.help_overlay.messages = !self.help_overlay.messages;
                    self.help_overlay.scroll = self.help_overlay.scroll.min(self.help_scroll_max());
                }
            }
            (K::F(1) | K::Esc, M::NONE) => {
                if key.kind == Kind::Press {
                    self.help_open = false;
                }
            }
            (code @ (K::Up | K::Down | K::PageUp | K::PageDown), M::NONE)
                if self.help_overlay.messages =>
            {
                let step = if matches!(code, K::PageUp | K::PageDown) {
                    tui::messages_page_height(self.help_overlay.size.1).max(1)
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

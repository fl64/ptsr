//! The command panel belongs to the SMC-PAD connection that opened it.

use std::collections::{HashMap, HashSet};

use input::Action;
use ratatui::text::Line;

use crate::{App, Screen};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum View {
    Pads,
    PadEdit,
    Pattern,
    Files,
    Chop,
    ChopEdit,
    Fx,
    Settings,
}

const MAIN: [(&str, Option<Action>); 7] = [
    ("pads", Some(Action::ScreenPads)),
    ("files", Some(Action::ScreenFiles)),
    ("chop", Some(Action::ScreenChop)),
    ("A1", Some(Action::ScreenPattern)),
    ("fx", Some(Action::ScreenFx)),
    ("mixer", None),
    ("setup", Some(Action::ScreenSettings)),
];

impl View {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "pads" => Self::Pads,
            "pad_edit" => Self::PadEdit,
            "pattern" => Self::Pattern,
            "files" => Self::Files,
            "chop" => Self::Chop,
            "chop_edit" => Self::ChopEdit,
            "fx" => Self::Fx,
            "settings" => Self::Settings,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Cell {
    pub label: String,
    pub action: Action,
}

pub(super) struct Config {
    cells: HashMap<(View, usize), [Option<Cell>; 12]>,
    pages: HashMap<View, usize>,
    pub warnings: Vec<String>,
}

impl Config {
    pub(super) fn parse(file: &str, source: &str) -> Self {
        let mut config = Self {
            cells: HashMap::new(),
            pages: HashMap::new(),
            warnings: Vec::new(),
        };
        let root: toml::Value = match toml::from_str(source) {
            Ok(root) => root,
            Err(error) => {
                config.warnings.push(format!("{file}: {error}"));
                return config;
            }
        };
        let Some(views) = root.get("view").and_then(toml::Value::as_array) else {
            config.warnings.push(format!("{file}: no view list"));
            return config;
        };
        let mut seen = HashSet::new();
        for row in views {
            let Some(name) = row.get("name").and_then(toml::Value::as_str) else {
                config.warnings.push(format!("{file}: view has no name"));
                continue;
            };
            let Some(view) = View::parse(name) else {
                config.warnings.push(format!("{file}: unknown view {name}"));
                continue;
            };
            let Some(page) = row
                .get("page")
                .and_then(toml::Value::as_integer)
                .and_then(|page| usize::try_from(page).ok())
            else {
                config
                    .warnings
                    .push(format!("{file}: {name}: invalid page"));
                continue;
            };
            config
                .pages
                .entry(view)
                .and_modify(|count| *count = (*count).max(page + 1))
                .or_insert(page + 1);
            let Some(cells) = row.get("cells").and_then(toml::Value::as_array) else {
                config
                    .warnings
                    .push(format!("{file}: {name} page {page}: no cells"));
                continue;
            };
            for item in cells {
                let raw = item.get("cell").and_then(toml::Value::as_integer);
                let cell = raw.and_then(|cell| usize::try_from(cell).ok());
                let where_ = format!(
                    "{file}: {name} page {page} cell {}",
                    raw.map_or("?".into(), |n| n.to_string())
                );
                let Some(cell @ 1..=12) = cell else {
                    config
                        .warnings
                        .push(format!("{where_}: outside 1-12 or fixed bottom row"));
                    continue;
                };
                let key = (view, page, cell);
                let slots = config
                    .cells
                    .entry((view, page))
                    .or_insert_with(|| std::array::from_fn(|_| None));
                if !seen.insert(key) {
                    slots[cell - 1] = None;
                    config.warnings.push(format!("{where_}: repeated cell"));
                    continue;
                }
                if (page == 0 && !(9..=12).contains(&cell)) || (page > 0 && cell > 10) {
                    config.warnings.push(format!("{where_}: reserved cell"));
                    continue;
                }
                let Some(label) = item.get("label").and_then(toml::Value::as_str) else {
                    config.warnings.push(format!("{where_}: no label"));
                    continue;
                };
                if label.is_empty() || Line::raw(label).width() > 6 {
                    config
                        .warnings
                        .push(format!("{where_}: label exceeds six cells"));
                    continue;
                }
                let Some(name) = item.get("action").and_then(toml::Value::as_str) else {
                    config.warnings.push(format!("{where_}: no action"));
                    continue;
                };
                let Some(action) = Action::parse(name) else {
                    config
                        .warnings
                        .push(format!("{where_}: unknown action {name}"));
                    continue;
                };
                slots[cell - 1] = Some(Cell {
                    label: label.into(),
                    action,
                });
            }
        }
        config
    }

    pub(super) fn cell(&self, view: View, page: usize, cell: u8) -> Option<&Cell> {
        self.cells
            .get(&(view, page))?
            .get(usize::from(cell.checked_sub(1)?))?
            .as_ref()
    }

    pub(super) fn pages(&self, view: View) -> usize {
        self.pages.get(&view).copied().unwrap_or(1)
    }
}

pub(super) struct State {
    owner: Option<String>,
    page: usize,
    view: Option<View>,
    config: Config,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Press {
    Revealed,
    Cell(u8),
    OtherDevice,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Choice {
    Action(Action),
    Exit,
    Back,
    Page,
    Dim,
}

impl Default for State {
    fn default() -> Self {
        Self {
            owner: None,
            page: 0,
            view: None,
            config: Config::parse(
                "config/panel.toml",
                include_str!("../../../config/panel.toml"),
            ),
        }
    }
}

impl State {
    pub(super) fn load(file: &str) -> Self {
        let mut state = Self::default();
        match std::fs::read_to_string(file) {
            Ok(source) => state.config = Config::parse(file, &source),
            Err(error) => state.config.warnings.push(format!("{file}: {error}")),
        }
        state
    }

    pub(super) fn warnings(&self) -> &[String] {
        &self.config.warnings
    }

    pub(super) fn reveal(&mut self, port: &str) {
        if self.owner.is_none() {
            self.owner = Some(port.into());
            self.page = 0;
            self.view = None;
        }
    }

    pub(super) fn press(&mut self, port: &str, cell: u8) -> Press {
        match self.owner.as_deref() {
            None => {
                self.reveal(port);
                Press::Revealed
            }
            Some(owner) if owner == port => Press::Cell(cell),
            Some(_) => Press::OtherDevice,
        }
    }

    pub(super) fn visible(&self) -> bool {
        self.owner.is_some()
    }

    pub(super) fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    pub(super) fn page(&self) -> usize {
        self.page
    }

    pub(super) fn page_count(&self, view: View) -> usize {
        self.config.pages(view)
    }

    pub(super) fn choice(&mut self, view: View, cell: u8) -> Choice {
        match cell {
            13 => Choice::Back,
            14 => Choice::Action(Action::Undo),
            15 | 16 => Choice::Dim,
            1..=7 if self.page == 0 => MAIN[usize::from(cell - 1)]
                .1
                .map_or(Choice::Dim, Choice::Action),
            8 if self.page == 0 => {
                if self.page_count(view) > 1 {
                    self.page = 1;
                    Choice::Page
                } else {
                    Choice::Dim
                }
            }
            11 if self.page > 0 => {
                self.page = if self.page + 1 < self.page_count(view) {
                    self.page + 1
                } else {
                    1
                };
                Choice::Page
            }
            12 if self.page > 0 => {
                self.page = 0;
                Choice::Page
            }
            1..=12 => self
                .config
                .cell(view, self.page, cell)
                .map_or(Choice::Dim, |cell| {
                    if cell.action == Action::Quit {
                        Choice::Exit
                    } else {
                        Choice::Action(cell.action)
                    }
                }),
            _ => Choice::Dim,
        }
    }

    pub(super) fn label(&self, view: View, cell: u8) -> Option<&str> {
        match (self.page, cell) {
            (_, 13) => Some("back"),
            (_, 14) => Some("undo"),
            (_, 15) => Some("redo"),
            (_, 16) => Some("panic"),
            (0, 1..=7) => Some(MAIN[usize::from(cell - 1)].0),
            (0, 8) => Some("more"),
            (page, 11) if page > 0 => Some("next"),
            (page, 12) if page > 0 => Some("main"),
            _ => self
                .config
                .cell(view, self.page, cell)
                .map(|cell| cell.label.as_str()),
        }
    }

    pub(super) fn main_page(&mut self) {
        self.page = 0;
    }

    pub(super) fn select_context_page(&mut self, view: View, steps: i32) {
        if self.visible() && self.page > 0 {
            let pages = self.page_count(view).saturating_sub(1);
            if pages > 0 {
                self.page = 1
                    + ((self.page - 1) as i64 + i64::from(steps)).rem_euclid(pages as i64) as usize;
            }
        }
    }

    pub(super) fn sync_view(&mut self, view: View) {
        if self.view.is_some_and(|previous| previous != view) {
            self.page = 0;
        }
        self.view = Some(view);
    }

    pub(super) fn hide(&mut self) {
        self.owner = None;
        self.page = 0;
        self.view = None;
    }

    pub(super) fn disconnect(&mut self, port: &str) {
        if self.owner() == Some(port) {
            self.hide();
        }
    }
}

impl App {
    pub(super) fn panel_view(&self) -> View {
        match self.screen {
            Screen::Pads if self.pattern_view.overview => View::Pattern,
            Screen::Pads if self.edit => View::PadEdit,
            Screen::Pads => View::Pads,
            Screen::Files => View::Files,
            Screen::Chop if self.edit => View::ChopEdit,
            Screen::Chop => View::Chop,
            Screen::Fx => View::Fx,
            Screen::Settings => View::Settings,
        }
    }

    pub(super) fn panel_input_available(&self) -> bool {
        let mode = self.focus().mode;
        !self.help_open && !mode.text && !mode.picker && !mode.screen_learn && !mode.midi_learn
    }

    pub(super) fn panel_snapshot(&self) -> Option<tui::CommandPanel> {
        (self.panel.visible() && self.panel_input_available()).then(|| {
            let view = self.panel_view();
            let labels = std::array::from_fn(|index| {
                let cell = u8::try_from(index + 1).unwrap();
                self.panel.label(view, cell).map(str::to_owned)
            });
            let name = match view {
                View::Pads => "PADS",
                View::PadEdit => "PAD EDIT",
                View::Pattern => "PATTERN",
                View::Files => "FILES",
                View::Chop => "CHOP",
                View::ChopEdit => "CHOP EDIT",
                View::Fx => "FX",
                View::Settings => "SETTINGS",
            };
            tui::CommandPanel {
                labels,
                page: self.panel.page(),
                pages: self.panel.page_count(view),
                view: name,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario "Shipped labels fit the panel glossary" (command-panel).
    #[test]
    fn shipped_labels_fit_the_panel_glossary() {
        let allowed = [
            "A1", "as pad", "assign", "auto", "back", "chop", "clear", "commit", "copy", "cut",
            "cut -", "edit", "end", "end -", "enter", "erase", "exit", "files", "fx", "gate",
            "grid", "grid +", "grid -", "help", "hits", "hits -", "knobs", "learn", "level",
            "load", "loop", "main", "mark", "marks", "mixer", "mode", "more", "next", "open",
            "order", "output", "pads", "panic", "parent", "parts", "play", "record", "redo",
            "row +", "row -", "rows", "save", "setup", "snap", "star", "start", "step", "tab",
            "take", "undo", "whole",
        ];
        let mut panel = State::default();
        assert!(
            panel.config.warnings.is_empty(),
            "{:?}",
            panel.config.warnings
        );
        for (view, count) in panel.config.pages.clone() {
            for page in 0..count {
                panel.page = page;
                for cell in 1..=16 {
                    if let Some(label) = panel.label(view, cell) {
                        assert!(
                            Line::raw(label).width() <= 6 && allowed.contains(&label),
                            "{view:?} page {page}, cell {cell}: {label}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn main_cells_follow_the_current_view() {
        let config = Config::parse(
            "config/panel.toml",
            include_str!("../../../config/panel.toml"),
        );
        assert!(config.warnings.is_empty(), "{:?}", config.warnings);
        for view in [
            View::Pads,
            View::PadEdit,
            View::Pattern,
            View::Files,
            View::Chop,
            View::ChopEdit,
            View::Fx,
            View::Settings,
        ] {
            assert_eq!(
                config.pages(view),
                if matches!(view, View::Pads | View::Chop | View::ChopEdit) {
                    5
                } else {
                    4
                },
                "{view:?}"
            );
            let current_actions = (9..=12)
                .filter(|&cell| config.cell(view, 0, cell).is_some())
                .count();
            let editor = matches!(view, View::PadEdit | View::ChopEdit | View::Fx);
            assert_eq!(current_actions, if editor { 0 } else { 4 }, "{view:?}");
            assert!(
                (1..=10).any(|cell| config.cell(view, 1, cell).is_some()),
                "{view:?}"
            );
        }
    }

    /// Scenario "Layout rejects a split or reversed panel pair" (command-panel).
    #[test]
    fn layout_rejects_a_split_or_reversed_panel_pair() {
        let adjacent =
            |left: i64, right: i64| left + 1 == right && (left - 1) / 4 == (right - 1) / 4;
        let source: toml::Value =
            toml::from_str(include_str!("../../../config/panel.toml")).unwrap();
        let mut checked = 0;
        for view in source["view"].as_array().unwrap() {
            let mut pairs = HashMap::<&str, [Option<i64>; 2]>::new();
            for cell in view["cells"].as_array().unwrap() {
                let Some(pair) = cell.get("pair").and_then(toml::Value::as_str) else {
                    continue;
                };
                let (name, side) = pair.split_once(':').unwrap();
                let side = match side {
                    "lower" => 0,
                    "upper" => 1,
                    _ => panic!("unknown side for {pair}"),
                };
                let slot = cell["cell"].as_integer().unwrap();
                let previous = pairs.entry(name).or_default()[side].replace(slot);
                assert!(previous.is_none(), "repeated side in {pair}");
            }
            for (name, slots) in pairs {
                let [Some(left), Some(right)] = slots else {
                    panic!("missing side in {name}")
                };
                assert!(
                    adjacent(left, right),
                    "{name} uses cells {left} and {right}"
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 4);
        assert!(adjacent(7, 8), "No and Yes");
        assert!(adjacent(14, 15), "Undo and Redo");
        assert!(!adjacent(4, 5), "split row");
        assert!(!adjacent(8, 7), "reversed roles");
    }

    /// Scenario "Fixed cells stay on the bottom row" (command-panel).
    #[test]
    fn screen_and_context_pages_cover_more_than_16_commands() {
        let mut panel = State::default();
        panel.press("SMC-PAD-Master A", 8);
        assert_eq!(panel.page(), 0);
        assert_eq!(panel.choice(View::Pads, 8), Choice::Page);
        assert_eq!(panel.page(), 1);
        assert_eq!(panel.label(View::Pads, 1), Some("mode"));
        assert_eq!(panel.choice(View::Pads, 12), Choice::Page);
        assert_eq!(panel.page(), 0);
        assert_eq!(panel.label(View::Pads, 9), Some("edit"));
        for (cell, label) in [(13, "back"), (14, "undo"), (15, "redo"), (16, "panic")] {
            assert_eq!(panel.label(View::Pads, cell), Some(label));
        }
        assert_eq!(panel.choice(View::Pads, 15), Choice::Dim);
        assert_eq!(panel.choice(View::Pads, 16), Choice::Dim);
        panel.sync_view(View::Files);
        assert_eq!(panel.page(), 0);
        for (cell, label) in [(13, "back"), (14, "undo"), (15, "redo"), (16, "panic")] {
            assert_eq!(panel.label(View::Files, cell), Some(label));
        }
    }

    #[test]
    fn context_pages_cycle_to_a_third_page() {
        let config = Config::parse(
            "config/panel.toml",
            "[[view]]\nname='pads'\npage=0\ncells=[]\n\
             [[view]]\nname='pads'\npage=1\ncells=[]\n\
             [[view]]\nname='pads'\npage=2\ncells=[{cell=1,label='third',action='session.save'}]\n",
        );
        assert!(config.warnings.is_empty(), "{:?}", config.warnings);
        let mut panel = State {
            owner: Some("SMC-PAD-Master A".into()),
            page: 0,
            view: None,
            config,
        };
        assert_eq!(panel.choice(View::Pads, 8), Choice::Page);
        assert_eq!(panel.choice(View::Pads, 11), Choice::Page);
        assert_eq!(panel.page(), 2);
        assert_eq!(panel.label(View::Pads, 1), Some("third"));
        assert_eq!(panel.choice(View::Pads, 11), Choice::Page);
        assert_eq!(panel.page(), 1);
    }

    /// Scenario "Encoder selects a later context page" (controller-reach).
    #[test]
    fn encoder_page_selection_needs_a_visible_context_page() {
        let mut panel = State::default();
        panel.select_context_page(View::Pads, 1);
        assert_eq!(panel.page(), 0);
        panel.press("SMC-PAD-Master A", 8);
        panel.select_context_page(View::Pads, 1);
        assert_eq!(panel.page(), 0);
        panel.choice(View::Pads, 8);
        panel.select_context_page(View::Pads, 2);
        assert_eq!(panel.page(), 3);
        assert_eq!(panel.label(View::Pads, 5), Some("erase"));
        panel.select_context_page(View::Pads, -1);
        assert_eq!(panel.page(), 2);
        assert_eq!(panel.label(View::Pads, 10), Some("exit"));
    }

    #[test]
    fn invalid_panel_cells_do_not_disable_pad_play() {
        let config = Config::parse(
            "config/panel.toml",
            "[[view]]\nname='pads'\npage=0\ncells=[\n\
             {cell=9,label='good',action='screen.files'},\n\
             {cell=10,label='bad',action='missing.action'},\n\
             {cell=13,label='fixed',action='ui.back'},\n\
             {cell=9,label='twice',action='screen.chop'}]\n",
        );
        assert_eq!(config.warnings.len(), 3, "{:?}", config.warnings);
        assert!(
            config
                .warnings
                .iter()
                .all(|warning| warning.contains("config/panel.toml") && warning.contains("cell"))
        );
        assert!(config.cell(View::Pads, 0, 9).is_none());
        assert!(config.cell(View::Pads, 0, 10).is_none());
        assert!(config.cell(View::Pads, 0, 13).is_none());
    }

    #[test]
    fn first_second_bank_hit_only_reveals_the_panel() {
        let mut panel = State::default();
        assert_eq!(panel.press("SMC-PAD-Master A", 1), Press::Revealed);
        assert_eq!(panel.press("SMC-PAD-Master A", 2), Press::Cell(2));
    }

    #[test]
    fn two_controllers_keep_separate_routes_during_panel_use() {
        let mut panel = State::default();
        assert_eq!(panel.press("SMC-PAD-Master A", 1), Press::Revealed);
        assert_eq!(panel.press("X-Session", 1), Press::OtherDevice);
        assert_eq!(panel.owner(), Some("SMC-PAD-Master A"));
    }

    #[test]
    fn disconnect_clears_the_panel_without_a_stuck_gate() {
        let mut panel = State::default();
        panel.press("SMC-PAD-Master A", 1);
        panel.disconnect("X-Session");
        assert!(panel.visible());
        panel.disconnect("SMC-PAD-Master A");
        assert!(!panel.visible());
        assert_eq!(panel.press("SMC-PAD-Master A", 1), Press::Revealed);
    }
}

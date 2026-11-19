//! The command panel belongs to the SMC-PAD connection that opened it.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

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

/// The screens view (127): screens, Help and the global actions in cells 1–12.
const SCREENS: [(&str, Option<Action>); 12] = [
    ("pads", Some(Action::ScreenPads)),
    ("files", Some(Action::ScreenFiles)),
    ("chop", Some(Action::ScreenChop)),
    ("A1", Some(Action::ScreenPattern)),
    ("fx", Some(Action::ScreenFx)),
    ("mixer", None),
    ("setup", Some(Action::ScreenSettings)),
    ("help", Some(Action::HelpToggle)),
    ("save", Some(Action::SessionSave)),
    ("open", Some(Action::ProjectPick)),
    ("learn", Some(Action::LearnToggle)),
    ("exit", Some(Action::Quit)),
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
    cells: HashMap<View, [Option<Cell>; 12]>,
    pub warnings: Vec<String>,
}

impl Config {
    pub(super) fn parse(file: &str, source: &str) -> Self {
        let mut config = Self {
            cells: HashMap::new(),
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
            // Action pages start at 1; the screens view is fixed in the program.
            let Some(page) = row
                .get("page")
                .and_then(toml::Value::as_integer)
                .and_then(|page| usize::try_from(page).ok())
                .filter(|page| *page >= 1)
            else {
                config
                    .warnings
                    .push(format!("{file}: {name}: invalid page"));
                continue;
            };
            // A view has one page of 12 actions; more goes to its own screen or mode.
            if page > 1 {
                config.warnings.push(format!(
                    "{file}: {name} page {page}: a view has one action page"
                ));
                continue;
            }
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
                let key = (view, cell);
                let slots = config
                    .cells
                    .entry(view)
                    .or_insert_with(|| std::array::from_fn(|_| None));
                if !seen.insert(key) {
                    slots[cell - 1] = None;
                    config.warnings.push(format!("{where_}: repeated cell"));
                    continue;
                }
                let Some(label) = item.get("label").and_then(toml::Value::as_str) else {
                    config.warnings.push(format!("{where_}: no label"));
                    continue;
                };
                if label.is_empty() || Line::raw(label).width() > 7 {
                    config
                        .warnings
                        .push(format!("{where_}: label exceeds seven cells"));
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

    pub(super) fn cell(&self, view: View, cell: u8) -> Option<&Cell> {
        self.cells
            .get(&view)?
            .get(usize::from(cell.checked_sub(1)?))?
            .as_ref()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Copy,
    Swap,
    File,
    Assign,
}

impl Target {
    fn title(self) -> &'static str {
        match self {
            Self::Copy => "COPY · release, hit a pad",
            Self::Swap => "SWAP · release, hit a pad",
            Self::File => "FILE · release, hit a pad",
            Self::Assign => "ASSIGN · release, hit a pad",
        }
    }
}

pub(super) struct State {
    owner: Option<String>,
    /// The last cell hit and its time, lit for one pad hit flash (131).
    pub flash: Option<(u8, Instant)>,
    page: usize,
    view: Option<View>,
    pub target_menu: Option<Target>,
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
    Dim,
}

impl Default for State {
    fn default() -> Self {
        Self {
            owner: None,
            flash: None,
            page: 1,
            view: None,
            target_menu: None,
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
            self.page = 1;
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

    pub(super) fn choice(&mut self, view: View, cell: u8) -> Choice {
        let action = match cell {
            13 => return Choice::Back,
            14 => return Choice::Action(Action::Undo),
            1..=12 if self.page == 0 => SCREENS[usize::from(cell - 1)].1,
            1..=12 => self.config.cell(view, cell).map(|cell| cell.action),
            _ => None,
        };
        match action {
            Some(Action::Quit) => Choice::Exit,
            Some(action) => Choice::Action(action),
            None => Choice::Dim,
        }
    }

    pub(super) fn label(&self, view: View, cell: u8) -> Option<&str> {
        match (self.page, cell) {
            (_, 13) => Some("back"),
            (_, 14) => Some("undo"),
            (_, 15) => Some("redo"),
            (0, 1..=12) => Some(SCREENS[usize::from(cell - 1)].0),
            _ => self.config.cell(view, cell).map(|cell| cell.label.as_str()),
        }
    }

    /// Move between the screens view (0) and the actions (1), without a wrap.
    pub(super) fn move_view(&mut self, steps: i32) {
        self.page = (self.page as i64 + i64::from(steps)).clamp(0, 1) as usize;
    }

    /// Show action page 1, the view that a reveal and a screen change open.
    pub(super) fn first_actions(&mut self) {
        self.page = 1;
    }

    pub(super) fn sync_view(&mut self, view: View) {
        if self.view.is_some_and(|previous| previous != view) {
            self.page = 1;
            self.target_menu = None;
        }
        self.view = Some(view);
    }

    pub(super) fn hide(&mut self) {
        self.owner = None;
        self.flash = None;
        self.target_menu = None;
        self.page = 1;
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

    pub(super) fn panel_target(&self) -> Option<Target> {
        if self.pattern_view.edit || self.erase_pads || self.pattern_routes_pads() {
            return None;
        }
        if self.screen == Screen::Chop {
            self.plan.as_ref().map(|_| Target::Assign)
        } else if self.browser.in_hand.is_some() {
            Some(Target::File)
        } else if self.copy_in_hand.is_some() {
            Some(Target::Copy)
        } else if self.pad_in_hand.is_some() {
            Some(Target::Swap)
        } else {
            None
        }
    }

    /// During Assign, cell 4 of the Chop actions gives `order`: `assign` is already active.
    pub(super) fn assign_order_cell(&self, cell: u8) -> bool {
        cell == 4
            && self.panel.page() == 1
            && self.panel_view() == View::Chop
            && self.panel_target() == Some(Target::Assign)
    }

    pub(super) fn sync_panel_target(&mut self) {
        if self.panel.target_menu.is_some() && self.panel.target_menu != self.panel_target() {
            self.panel.target_menu = None;
        }
    }

    /// The cell index to light, while the last hit is within a pad hit flash.
    fn panel_flash(&self) -> Option<usize> {
        let (cell, at) = self.panel.flash?;
        (at.elapsed() < super::HIT_FLASH).then(|| usize::from(cell - 1))
    }

    fn target_snapshot(&self, target: Target) -> tui::CommandPanel {
        let labels = std::array::from_fn(|index| {
            let label = match index {
                3 if target == Target::Assign => "order",
                12 => "cancel",
                13 => "undo",
                14 => "menu",
                15 if target == Target::Assign => "apply",
                _ => return None,
            };
            Some(label.into())
        });
        tui::CommandPanel {
            labels,
            page: 0,
            view: target.title(),
            target: true,
            flash: self.panel_flash(),
        }
    }

    /// The panel of a mode that takes the screen (134): only the cells that act there.
    fn modal_snapshot(&self) -> Option<tui::CommandPanel> {
        let pending = self
            .screen_learn
            .as_ref()
            .is_some_and(|learn| learn.pending.is_some() || learn.clearing)
            || self
                .learning
                .as_ref()
                .is_some_and(|learn| learn.pending.is_some());
        let (view, cells): (&'static str, &[(usize, &str)]) = if self.help_open {
            (
                "HELP · LEFT RIGHT tabs",
                &[(8, "up"), (9, "down"), (12, "close")],
            )
        } else if self.project_picker.is_some() || self.path_picker.is_some() {
            ("LIST · knob 1 moves", &[(12, "close"), (15, "open")])
        } else if self.screen_learn.is_some() || self.learning.is_some() {
            if pending {
                ("LEARN", &[(12, "cancel"), (15, "save")])
            } else {
                ("LEARN · touch a control", &[(12, "back")])
            }
        } else {
            ("TEXT · type on the keyboard", &[(12, "back")])
        };
        Some(tui::CommandPanel {
            labels: std::array::from_fn(|index| {
                cells
                    .iter()
                    .find(|(cell, _)| *cell == index)
                    .map(|(_, label)| (*label).to_string())
            }),
            page: self.panel.page(),
            view,
            target: true,
            flash: self.panel_flash(),
        })
    }

    pub(super) fn panel_snapshot(&self) -> Option<tui::CommandPanel> {
        if self.panel.visible() && !self.panel_input_available() {
            return self.modal_snapshot();
        }
        (self.panel.visible() && self.panel_input_available()).then(|| {
            // Behind a question only its two answers show (134).
            if self.ask.is_some() {
                return tui::CommandPanel {
                    labels: std::array::from_fn(|index| match index {
                        12 => Some("no".into()),
                        15 => Some("yes".into()),
                        _ => None,
                    }),
                    page: self.panel.page(),
                    view: "QUESTION",
                    target: true,
                    flash: self.panel_flash(),
                };
            }
            let target = self.panel_target();
            if let Some(target) = target
                && self.ask.is_none()
                && self.panel.target_menu != Some(target)
            {
                return self.target_snapshot(target);
            }
            let view = self.panel_view();
            let overrides = self.smc_cell_overrides();
            let labels = std::array::from_fn(|index| {
                if let Some(owner) = &overrides[index] {
                    return Some(owner.clone());
                }
                if index == 14 && target.is_some() {
                    return Some("target".into());
                }
                let cell = u8::try_from(index + 1).unwrap();
                if self.assign_order_cell(cell) {
                    return Some("order".into());
                }
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
                view: name,
                target: false,
                flash: self.panel_flash(),
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
        // One meaning per word (134): the old `take`, `commit`, `knobs`, `output` are gone.
        let allowed = [
            "A1", "all", "apply", "as pad", "assign", "back", "cancel", "chop", "clear", "close",
            "copy", "cut", "cut -", "down", "edit", "end -", "enter", "erase", "exit", "files",
            "fill", "fx", "grid", "grid +", "grid -", "help", "hits", "hits -", "learn", "listen",
            "load", "loop", "mark", "menu", "mixer", "mode", "no", "normal", "open", "order",
            "p.gate", "pad", "pads", "params", "parent", "parts", "rec out", "record", "redo",
            "remove", "reset", "row +", "row -", "rows", "save", "setup", "snap", "star", "step",
            "swap", "tab", "tap", "target", "to pad", "undo", "undoA1", "unbind", "up", "used",
            "whole", "yes",
        ];
        let check = |label: &str| {
            assert!(
                Line::raw(label).width() <= 7 && allowed.contains(&label),
                "{label}"
            );
        };
        let mut panel = State::default();
        assert!(
            panel.config.warnings.is_empty(),
            "{:?}",
            panel.config.warnings
        );
        let views: Vec<View> = panel.config.cells.keys().copied().collect();
        for view in views {
            for page in 0..=1 {
                panel.page = page;
                for cell in 1..=16 {
                    if let Some(label) = panel.label(view, cell) {
                        check(label);
                    }
                }
            }
        }
        let app = crate::tests::test_app();
        for target in [Target::Copy, Target::Swap, Target::File, Target::Assign] {
            for label in app.target_snapshot(target).labels.into_iter().flatten() {
                check(&label);
            }
        }
    }

    /// Scenario "Shipped action pages match the table" (command-panel).
    #[test]
    fn shipped_action_pages_match_the_table() {
        let config = Config::parse(
            "config/panel.toml",
            include_str!("../../../config/panel.toml"),
        );
        assert!(config.warnings.is_empty(), "{:?}", config.warnings);
        let table: [(View, &[&str]); 8] = [
            (
                View::Pads,
                &[
                    "edit", "copy", "swap", "mark", "mode", "remove", "p.gate", "all", "grid",
                    "rec out",
                ],
            ),
            (View::PadEdit, &["mode", "reset", "p.gate", "copy"]),
            (
                View::Pattern,
                &[
                    "step", "grid -", "grid +", "params", "record", "erase", "undoA1", "clear",
                    "rows", "tap",
                ],
            ),
            (
                View::Files,
                &[
                    "to pad", "load", "parent", "star", "enter", "listen", "chop",
                ],
            ),
            (
                View::Chop,
                &[
                    "cut", "parts", "hits", "assign", "loop", "listen", "snap", "as pad", "whole",
                    "", "edit", "fill",
                ],
            ),
            (
                View::ChopEdit,
                &["normal", "hits -", "cut -", "end -", "whole"],
            ),
            (View::Fx, &[]),
            (
                View::Settings,
                &["tab", "learn", "unbind", "", "row -", "row +"],
            ),
        ];
        for (view, labels) in table {
            let shipped: Vec<&str> = (1..=12)
                .map(|cell| {
                    config
                        .cell(view, cell)
                        .map_or("", |cell| cell.label.as_str())
                })
                .collect();
            let mut expected = labels.to_vec();
            expected.resize(12, "");
            assert_eq!(shipped, expected, "{view:?}");
        }
    }

    /// Scenario "A second action page is rejected" (command-panel).
    #[test]
    fn a_second_action_page_is_rejected() {
        let config = Config::parse(
            "config/panel.toml",
            "[[view]]\nname='chop'\npage=1\ncells=[{cell=1,label='cut',action='chop.cut'}]\n\
             [[view]]\nname='chop'\npage=2\ncells=[{cell=1,label='edit',action='chop.edit'}]\n",
        );
        assert_eq!(
            config.warnings,
            ["config/panel.toml: chop page 2: a view has one action page"]
        );
        assert_eq!(config.cell(View::Chop, 1).unwrap().label, "cut");
        assert!(config.cell(View::Chop, 2).is_none());
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
        assert_eq!(checked, 2);
        assert!(adjacent(7, 8), "No and Yes");
        assert!(adjacent(14, 15), "Undo and Redo");
        assert!(!adjacent(4, 5), "split row");
        assert!(!adjacent(8, 7), "reversed roles");
    }

    /// Scenario "Fixed cells stay on the bottom row" (command-panel).
    #[test]
    fn fixed_cells_stay_on_the_bottom_row() {
        let mut panel = State::default();
        panel.press("SMC-PAD-Master A", 8);
        assert_eq!(panel.page(), 1);
        assert_eq!(panel.label(View::Pads, 1), Some("edit"));
        panel.move_view(-1);
        assert_eq!(panel.page(), 0);
        assert_eq!(panel.label(View::Pads, 1), Some("pads"));
        for view in [View::Pads, View::Files] {
            panel.sync_view(view);
            for (cell, label) in [(13, "back"), (14, "undo"), (15, "redo")] {
                assert_eq!(panel.label(view, cell), Some(label));
            }
            assert_eq!(panel.label(view, 16), None, "no cell shows Panic");
            assert_eq!(panel.choice(view, 15), Choice::Dim);
            assert_eq!(panel.choice(view, 16), Choice::Dim);
        }
        assert_eq!(panel.page(), 1, "a screen change shows the actions");
    }

    /// Scenario "LEFT and RIGHT move through the views" (command-panel).
    #[test]
    fn left_and_right_move_through_the_views() {
        let mut panel = State::default();
        panel.press("SMC-PAD-Master A", 8);
        for (steps, page) in [(-1, 0), (-1, 0), (1, 1), (1, 1), (-2, 0), (2, 1)] {
            panel.move_view(steps);
            assert_eq!(panel.page(), page, "after {steps}");
        }
        assert_eq!(panel.label(View::Pads, 1), Some("edit"));
        panel.move_view(-1);
        assert_eq!(panel.label(View::Pads, 12), Some("exit"));
        assert_eq!(panel.choice(View::Pads, 12), Choice::Exit);
    }

    #[test]
    fn invalid_panel_cells_do_not_disable_pad_play() {
        let config = Config::parse(
            "config/panel.toml",
            "[[view]]\nname='pads'\npage=1\ncells=[\n\
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
        assert!(config.cell(View::Pads, 9).is_none());
        assert!(config.cell(View::Pads, 10).is_none());
        assert!(config.cell(View::Pads, 13).is_none());
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

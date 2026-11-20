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
    // Bindings change from the keyboard only (138): no learn cell.
    ("", None),
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
                if label.is_empty() || !label_fits(label) {
                    config
                        .warnings
                        .push(format!("{where_}: label exceeds eight cells"));
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

    /// The action of an action cell; the screens view and the fixed row have none.
    pub(super) fn action(&self, view: View, cell: u8) -> Option<Action> {
        (self.page != 0 && cell <= 12)
            .then(|| self.config.cell(view, cell).map(|cell| cell.action))
            .flatten()
    }

    pub(super) fn label(&self, view: View, cell: u8) -> Option<&str> {
        match (self.page, cell) {
            (_, 13) => Some("back"),
            (_, 14) => Some("undo"),
            (_, 15) => Some("redo"),
            (0, 1..=12) => Some(SCREENS[usize::from(cell - 1)].0).filter(|l| !l.is_empty()),
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

    /// During Assign, the `put on pads` cell of the Chop actions gives `order`: `assign` is
    /// already active.
    pub(super) fn assign_order_cell(&self, cell: u8) -> bool {
        self.panel.action(View::Chop, cell) == Some(Action::ChopAssign)
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
            dim: [false; 16],
            // the method cell drops placed slices (138)
            warn: std::array::from_fn(|index| index == 3 && target == Target::Assign),
            current: [false; 16],
            states: [None; 16],
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
            dim: [false; 16],
            warn: [false; 16],
            current: [false; 16],
            states: [None; 16],
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
                    dim: [false; 16],
                    warn: [false; 16],
                    current: [false; 16],
                    states: [None; 16],
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
                // the pads toggle names what the press does now (138)
                match self.panel.label(view, cell) {
                    Some("hide pads") if self.ui.hide_pads => Some("show pads".into()),
                    label => label.map(str::to_owned),
                }
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
                dim: std::array::from_fn(|index| {
                    let action = match index {
                        13 => Some(Action::Undo),
                        _ => self.panel.action(view, u8::try_from(index + 1).unwrap()),
                    };
                    action.is_some_and(|action| self.grey(view, action).is_some())
                }),
                warn: std::array::from_fn(|index| {
                    let cell = u8::try_from(index + 1).unwrap();
                    self.assign_order_cell(cell)
                        || self
                            .panel
                            .action(view, cell)
                            .is_some_and(|action| Self::warns(view, action))
                }),
                current: std::array::from_fn(|index| {
                    let cell = u8::try_from(index + 1).unwrap();
                    view == View::PadEdit
                        && self.panel.action(view, cell)
                            == u8::try_from(self.pads.page_index())
                                .ok()
                                .map(Action::PadPage)
                }),
                // a learned cell keeps no shipped state (138)
                states: std::array::from_fn(|index| {
                    let cell = u8::try_from(index + 1).unwrap();
                    overrides[index]
                        .is_none()
                        .then(|| self.panel.action(view, cell))
                        .flatten()
                        .and_then(|action| self.cell_state(action))
                }),
            }
        })
    }

    /// A join without a neighbour slice cannot run: its cell is dim (135).
    pub(super) fn join_blocked(&self, action: Option<Action>) -> bool {
        let Some(c) = self.chop.as_ref() else {
            return false;
        };
        match action {
            Some(Action::MarkerDelete) => c.selected == 0,
            Some(Action::SliceEndDelete) => c.selected + 1 >= c.slices().len(),
            _ => false,
        }
    }
}

/// A label of two words shows each word on its own line (135, 138); each line keeps at
/// most eight cells.
fn label_fits(label: &str) -> bool {
    match label.split_once(' ') {
        Some((first, second)) => Line::raw(first).width() <= 8 && Line::raw(second).width() <= 8,
        None => Line::raw(label).width() <= 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario "Shipped labels fit the panel glossary" (command-panel).
    #[test]
    fn shipped_labels_fit_the_panel_glossary() {
        // One meaning per label (134); plain two-word action labels (138).
        let allowed = [
            "A1",
            "A1 setup",
            "apply",
            "back",
            "cancel",
            "chop",
            "chop file",
            "chop pad",
            "choose",
            "clear",
            "clear A1",
            "clear pad",
            "close",
            "copy pad",
            "cursor step",
            "cut by hits",
            "cut here",
            "down",
            "edit pad",
            "edit slice",
            "edit steps",
            "enter",
            "env page",
            "equal parts",
            "erase hits",
            "exit",
            "fewer hits",
            "files",
            "filter page",
            "fill pads",
            "fx",
            "fx page",
            "hear as pad",
            "help",
            "hide empty",
            "hide pads",
            "join left",
            "join right",
            "listen file",
            "load to pad",
            "loop play",
            "mark bank",
            "mark pad",
            "menu",
            "mixer",
            "no",
            "norm",
            "open",
            "order",
            "pad",
            "pads",
            "parent",
            "pitch gate",
            "play mode",
            "play range",
            "put on pads",
            "record A1",
            "record output",
            "redo",
            "reset pad",
            "row +",
            "row -",
            "save",
            "setup",
            "show pads",
            "source page",
            "star file",
            "steps finer",
            "steps wider",
            "swap pads",
            "tab",
            "take file",
            "tap tempo",
            "target",
            "tone page",
            "undo",
            "up",
            "used",
            "yes",
        ];
        let check = |label: &str| {
            assert!(label_fits(label) && allowed.contains(&label), "{label}");
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
        // Cells 1-12 top-down: device pads 13-16, 9-12, 5-8 (138).
        let table: [(View, &[&str]); 8] = [
            (
                View::Pads,
                &[
                    "pitch gate",
                    "record output",
                    "hide pads",
                    "",
                    "swap pads",
                    "mark pad",
                    "mark bank",
                    "play mode",
                    "edit pad",
                    "copy pad",
                    "chop pad",
                    "clear pad",
                ],
            ),
            (
                View::PadEdit,
                &[
                    "fx page",
                    "",
                    "",
                    "",
                    "tone page",
                    "env page",
                    "filter page",
                    "source page",
                    "play mode",
                    "copy pad",
                    "pitch gate",
                    "reset pad",
                ],
            ),
            (
                View::Pattern,
                &[
                    "hide empty",
                    "",
                    "",
                    "",
                    "steps wider",
                    "steps finer",
                    "A1 setup",
                    "clear A1",
                    "record A1",
                    "edit steps",
                    "erase hits",
                    "tap tempo",
                ],
            ),
            (
                View::Files,
                &[
                    "",
                    "",
                    "",
                    "",
                    "star file",
                    "",
                    "",
                    "",
                    "load to pad",
                    "take file",
                    "listen file",
                    "chop file",
                ],
            ),
            (
                View::Chop,
                &[
                    "",
                    "loop play",
                    "play range",
                    "cursor step",
                    "put on pads",
                    "fill pads",
                    "edit slice",
                    "",
                    "cut here",
                    "cut by hits",
                    "fewer hits",
                    "equal parts",
                ],
            ),
            (
                View::ChopEdit,
                &[
                    "",
                    "",
                    "",
                    "",
                    "",
                    "",
                    "",
                    "",
                    "norm",
                    "join left",
                    "join right",
                    "hear as pad",
                ],
            ),
            (View::Fx, &[]),
            (View::Settings, &["", "", "", "", "", "", "", "", "choose"]),
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
        assert_eq!(checked, 1);
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
        assert_eq!(panel.label(View::Pads, 9), Some("edit pad"));
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
        assert_eq!(panel.label(View::Pads, 9), Some("edit pad"));
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

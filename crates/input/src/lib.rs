//! Keyboard to logical actions. Pure logic: tested on recorded events without a terminal.

pub mod midi;

use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Screen to resolve the action for; shared keys live in `[global]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Screen {
    Pads,
    Settings,
    Files,
    Chop,
    Fx,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    /// Pad 0..16 of the current bank; velocity 0..1.
    PadPress {
        pad: usize,
        velocity: f32,
    },
    PadRelease {
        pad: usize,
    },
    BankNext,
    BankPrev,
    /// Jump to bank 0..8 (shown as 1..8).
    BankSelect(usize),
    /// Master volume step in dB.
    MasterVolume(i8),
    /// Take the selected pad in hand; the next pad hit swaps the two.
    PadTake,
    /// Take a copy of the selected loaded pad (83a); the next pad hit places it.
    PadCopy,
    PadRename,
    PadSelect(Dir),
    PadModeNext,
    /// Pads screen: the edit mode of the selected pad opens; `Esc` closes it.
    PadEdit,
    PadEditOnly,
    /// Pick the next parameter the pads screen changes, in the order of the edit.
    PadParamNext,
    /// Step the picked pad parameter: 1 dB of volume or 1 semitone of pitch.
    PadParam(i8),
    /// Mark the 16 pads of the open bank and turn marking on; unmark them when all are marked.
    PadMarkBank,
    /// Turn marking on or off; while it is on, a pad hit marks or unmarks the pad.
    PadMarkToggle,
    /// Take the sound off the selected pad.
    PadClear,
    /// The learn mode on the screen (47): on and off.
    LearnToggle,
    ScreenPads,
    ScreenSettings,
    ScreenFiles,
    ScreenChop,
    ScreenFx,
    /// Chop screen: play the sample from the start / stop.
    ChopPlay,
    /// Chop screen: the next play mode — to the end, to the cut, the slice.
    ChopPlayMode,
    /// The loop of the play on the chop screen (49): on and off.
    ChopLoopToggle,
    /// The chop screen plays a slice as its pad (50): on and off.
    ChopAsPadToggle,
    /// Chop screen: the cursor to the next (`right`) or previous cut, or the region's edge;
    /// assigning: the next or previous bank.
    CursorJump {
        right: bool,
    },
    /// Chop screen: a marker at the playback cursor.
    ChopCut,
    /// Chop screen: the next equal-parts count; assigning: in order / by hand.
    ChopParts,
    /// Chop screen: chop by hits, stepping the threshold one lower; like equal parts
    /// this replaces the cuts and is one undo step.
    ChopHits,
    /// Chop screen: chop by hits, stepping the threshold one higher, stopping at −6 dB
    /// without wrapping (83a); the same cuts, undo history and hit rule as `chop.hits`.
    ChopHitsBack,
    /// Chop screen: switch the assign plan between in order and by hand (83a); nothing
    /// in listen or slice edit, and it applies no plan.
    ChopAssignOrder,
    /// Chop screen: the next equal-parts count only (`L`); the assign mode ignores it.
    ChopEqualParts,
    /// The next or previous parameter of the selected object (`K`, `I`); `Tab` while
    /// assigning on the chop screen: in order / by hand.
    ParamNext,
    ParamPrev,
    /// The next or previous page of the edit (54): `Tab`, `Shift+Tab`.
    PageNext,
    PagePrev,
    /// The picked value of the selected object a step up or down; `fine` is the small step.
    EditValue {
        up: bool,
        fine: bool,
    },
    /// Chop screen: slices onto pads from the selected pad on.
    ChopCommit,
    /// Chop screen: in the listen mode the edit mode of the selected slice opens; in
    /// the assign mode the plan applies (the commit).
    ChopEdit,
    ChopEditOnly,
    /// Chop screen: the assign mode starts (slices onto pads), by hand from slice 1.
    ChopAssign,
    /// Chop screen: zoom the wave view in or out around the cursor.
    ChopZoom {
        zoom_in: bool,
    },
    /// Chop screen: the region (what equal parts and playback cover) starts at the cursor.
    ChopRegionStart,
    /// Chop screen: the region ends at the cursor.
    ChopRegionEnd,
    /// Chop screen: the region is the whole sample again.
    ChopRegionClear,
    /// Chop screen: the volume of the pad it was opened from puts its peak at −1 dB.
    ChopNormalize,
    /// Select the previous (-1) or next (1) marker.
    MarkerSelect(i8),
    /// Chop screen: the cursor moves in even steps or jumps from hit to hit.
    ChopSnapToggle,
    /// Move the editing cursor of the chop screen; `fine` is the small even step.
    CursorMove {
        right: bool,
        fine: bool,
    },
    /// Chop screen: the start (`end` false) or the end of the selected slice a step left
    /// or right; from the keyboard a tap is `fine`, a held key repeats the coarse step.
    SliceBound {
        end: bool,
        right: bool,
        fine: bool,
    },
    /// Move the selected marker; `fine` is the small step.
    MarkerNudge {
        right: bool,
        fine: bool,
    },
    MarkerDelete,
    /// Chop: delete the cut at the end of the selected slice.
    SliceEndDelete,
    Undo,
    BrowserUp,
    BrowserDown,
    /// Open the selected folder.
    BrowserEnter,
    BrowserParent,
    /// Ctrl+Enter on the selection: open a folder or take a file in hand.
    BrowserHold,
    BrowserTakeOnly,
    /// Files screen: the selected file loads onto the selected pad; a folder opens.
    BrowserLoad,
    /// Files screen: play the selected file automatically, on/off.
    BrowserPreviewToggle,
    /// Files screen: open the filter line.
    BrowserFilter,
    /// Files screen: the selected folder becomes a favorite or stops being one.
    BrowserFavorite,
    /// Any screen: open the picker of the projects root (11).
    ProjectPick,
    /// The screens with pads: hide the grid or show it again (32).
    PadsToggle,
    HelpToggle,
    Back,
    Quit,
    /// Save the session now; the manual save says that it did (22).
    SessionSave,
    Click,
    /// Recording: `on` starts the microphone recording, the release (or `false`) stops it.
    RecordMic(bool),
    /// Recording: toggles the program's output recording.
    RecordOutputToggle,
    /// Settings screen: row up/down, value back/forward.
    SettingsRow(i8),
    SettingsValue(i8),
    SettingsTab,
    /// MIDI tab: wait for a control for the selected action / clear its learned control.
    SettingsLearn,
    SettingsClear,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Action {
    /// Action name from `keymap.toml`. Actions of future screens (`chop.*`, `browser.*`) give `None`.
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "bank.next" => Self::BankNext,
            "bank.prev" => Self::BankPrev,
            "master.volume_down" => Self::MasterVolume(-1),
            "pad.take" => Self::PadTake,
            "pad.copy" => Self::PadCopy,
            "pad.rename" => Self::PadRename,
            "master.volume_up" => Self::MasterVolume(1),
            b if b.starts_with("bank.") => match b[5..].parse::<usize>() {
                Ok(n @ 1..=8) => Self::BankSelect(n - 1),
                _ => return None,
            },
            "pad.select_up" => Self::PadSelect(Dir::Up),
            "pad.select_down" => Self::PadSelect(Dir::Down),
            "pad.select_left" => Self::PadSelect(Dir::Left),
            "pad.select_right" => Self::PadSelect(Dir::Right),
            "pad.mode_next" => Self::PadModeNext,
            "pad.edit" => Self::PadEdit,
            "pad.edit_only" => Self::PadEditOnly,
            "pad.param_next" => Self::PadParamNext,
            "pad.param_dec" => Self::PadParam(-1),
            "pad.param_inc" => Self::PadParam(1),
            // `pad.scope_next` is the name before 76: an old keymap or binding keeps working
            "pad.mark_bank" | "pad.scope_next" => Self::PadMarkBank,
            "pad.mark_toggle" => Self::PadMarkToggle,
            "pad.clear" => Self::PadClear,
            "learn.toggle" => Self::LearnToggle,
            // a learned control strikes pad N of the current bank
            b if b.starts_with("pad.") => match b[4..].parse::<usize>() {
                Ok(n @ 1..=16) => Self::PadPress {
                    pad: n - 1,
                    velocity: 1.0,
                },
                _ => return None,
            },
            "screen.pads" => Self::ScreenPads,
            "screen.settings" => Self::ScreenSettings,
            "screen.files" => Self::ScreenFiles,
            "screen.chop" => Self::ScreenChop,
            "screen.fx" => Self::ScreenFx,
            "transport.play_stop" => Self::ChopPlay,
            "chop.cut" => Self::ChopCut,
            "chop.play_mode" => Self::ChopPlayMode,
            "chop.loop_toggle" => Self::ChopLoopToggle,
            "chop.as_pad_toggle" => Self::ChopAsPadToggle,
            "chop.cursor_next_cut" => Self::CursorJump { right: true },
            "chop.cursor_prev_cut" => Self::CursorJump { right: false },
            "chop.auto_mode_next" => Self::ChopParts,
            "chop.parts" => Self::ChopEqualParts,
            "chop.hits" => Self::ChopHits,
            "chop.hits_back" => Self::ChopHitsBack,
            "chop.assign_order" => Self::ChopAssignOrder,
            "edit.param_next" => Self::ParamNext,
            "edit.param_prev" => Self::ParamPrev,
            "edit.page_next" => Self::PageNext,
            "edit.page_prev" => Self::PagePrev,
            "edit.value_dec" => Self::EditValue {
                up: false,
                fine: false,
            },
            "edit.value_inc" => Self::EditValue {
                up: true,
                fine: false,
            },
            "edit.value_dec_fine" => Self::EditValue {
                up: false,
                fine: true,
            },
            "edit.value_inc_fine" => Self::EditValue {
                up: true,
                fine: true,
            },
            "chop.commit" => Self::ChopCommit,
            "chop.edit" => Self::ChopEdit,
            "chop.edit_only" => Self::ChopEditOnly,
            "chop.assign" => Self::ChopAssign,
            "chop.zoom_in" => Self::ChopZoom { zoom_in: true },
            "chop.zoom_out" => Self::ChopZoom { zoom_in: false },
            "chop.region_start" => Self::ChopRegionStart,
            "chop.region_end" => Self::ChopRegionEnd,
            "chop.region_clear" => Self::ChopRegionClear,
            "chop.normalize" => Self::ChopNormalize,
            "marker.prev" => Self::MarkerSelect(-1),
            "marker.next" => Self::MarkerSelect(1),
            "chop.cursor_left" => Self::CursorMove {
                right: false,
                fine: false,
            },
            "chop.cursor_right" => Self::CursorMove {
                right: true,
                fine: false,
            },
            "chop.cursor_left_fine" => Self::CursorMove {
                right: false,
                fine: true,
            },
            "chop.cursor_right_fine" => Self::CursorMove {
                right: true,
                fine: true,
            },
            "chop.snap_toggle" => Self::ChopSnapToggle,
            "marker.nudge_left" => Self::MarkerNudge {
                right: false,
                fine: false,
            },
            "marker.nudge_right" => Self::MarkerNudge {
                right: true,
                fine: false,
            },
            "marker.nudge_left_fine" => Self::MarkerNudge {
                right: false,
                fine: true,
            },
            "marker.nudge_right_fine" => Self::MarkerNudge {
                right: true,
                fine: true,
            },
            "slice.start_left" => Self::SliceBound {
                end: false,
                right: false,
                fine: false,
            },
            "slice.start_right" => Self::SliceBound {
                end: false,
                right: true,
                fine: false,
            },
            "slice.end_left" => Self::SliceBound {
                end: true,
                right: false,
                fine: false,
            },
            "slice.end_right" => Self::SliceBound {
                end: true,
                right: true,
                fine: false,
            },
            "marker.delete" => Self::MarkerDelete,
            "slice.end_delete" => Self::SliceEndDelete,
            "edit.undo" => Self::Undo,
            "browser.up" => Self::BrowserUp,
            "browser.down" => Self::BrowserDown,
            "browser.enter" => Self::BrowserEnter,
            "browser.parent" => Self::BrowserParent,
            "browser.hold" => Self::BrowserHold,
            "browser.take_only" => Self::BrowserTakeOnly,
            "browser.load" => Self::BrowserLoad,
            "browser.preview_toggle" => Self::BrowserPreviewToggle,
            "browser.filter" => Self::BrowserFilter,
            "browser.favorite_toggle" => Self::BrowserFavorite,
            "project.pick" => Self::ProjectPick,
            "ui.pads_toggle" => Self::PadsToggle,
            "help.toggle" => Self::HelpToggle,
            "ui.back" => Self::Back,
            "app.quit" => Self::Quit,
            "session.save" => Self::SessionSave,
            "metronome.click" => Self::Click,
            "record.mic" => Self::RecordMic(true),
            "record.output" => Self::RecordOutputToggle,
            "settings.row_prev" => Self::SettingsRow(-1),
            "settings.row_next" => Self::SettingsRow(1),
            "settings.value_prev" => Self::SettingsValue(-1),
            "settings.value_next" => Self::SettingsValue(1),
            "settings.tab_next" => Self::SettingsTab,
            "settings.learn" => Self::SettingsLearn,
            "settings.clear" => Self::SettingsClear,
            _ => return None,
        })
    }
}

/// Where an action works: on every screen, on one screen or in the edit layer of a
/// screen (28).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Group {
    Global,
    Pads,
    PadsEdit,
    Settings,
    Files,
    Chop,
    ChopEdit,
    Fx,
}

impl Group {
    /// Section name in keymap and config files; new keys are additive (28).
    pub fn key(self) -> &'static str {
        match self {
            Group::Global => "global",
            Group::Pads => "pads",
            Group::PadsEdit => "pads_edit",
            Group::Settings => "settings",
            Group::Files => "files",
            Group::Chop => "chop",
            Group::ChopEdit => "chop_edit",
            Group::Fx => "fx",
        }
    }

    /// Name in the MIDI learn list (28).
    pub fn title(self) -> &'static str {
        match self {
            Group::Global => "GLOBAL",
            Group::Pads => "PADS",
            Group::PadsEdit => "PADS · EDIT",
            Group::Settings => "SETTINGS",
            Group::Files => "FILES",
            Group::Chop => "CHOP",
            Group::ChopEdit => "CHOP · EDIT",
            Group::Fx => "FX",
        }
    }

    /// Where learned assignments of the group apply (28).
    pub fn scope(self) -> midi::Scope {
        match self {
            Group::Global => midi::Scope::Global,
            Group::Pads => midi::Scope::Screen(Screen::Pads),
            Group::PadsEdit => midi::Scope::Edit(Screen::Pads),
            Group::Settings => midi::Scope::Screen(Screen::Settings),
            Group::Files => midi::Scope::Screen(Screen::Files),
            Group::Chop => midi::Scope::Screen(Screen::Chop),
            Group::ChopEdit => midi::Scope::Edit(Screen::Chop),
            Group::Fx => midi::Scope::Screen(Screen::Fx),
        }
    }

    pub const ALL: [Group; 8] = [
        Group::Global,
        Group::Pads,
        Group::PadsEdit,
        Group::Settings,
        Group::Files,
        Group::Chop,
        Group::ChopEdit,
        Group::Fx,
    ];

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|g| g.key() == key)
    }
}

pub struct ActionInfo {
    /// Name used in keymap.toml, device profiles and the config.
    pub name: &'static str,
    pub label: &'static str,
    pub group: Group,
    /// Driven by a knob position, not by a press.
    pub knob: bool,
}

/// Every action a key or a controller can drive, by group. Quitting is left out on purpose:
/// a stray controller press must not close the program during a set.
pub const ACTIONS: &[ActionInfo] = &[
    ActionInfo {
        name: "screen.pads",
        label: "pads screen",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "screen.settings",
        label: "settings screen",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "screen.files",
        label: "files screen",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "screen.chop",
        label: "chop screen",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "screen.fx",
        label: "shared FX screen",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "help.toggle",
        label: "help",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "ui.back",
        label: "back",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "bank.prev",
        label: "bank previous",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "bank.next",
        label: "bank next",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "bank.1",
        label: "bank 1",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "bank.2",
        label: "bank 2",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "bank.3",
        label: "bank 3",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "bank.4",
        label: "bank 4",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "bank.5",
        label: "bank 5",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "bank.6",
        label: "bank 6",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "bank.7",
        label: "bank 7",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "bank.8",
        label: "bank 8",
        group: Group::Global,
        knob: false,
    },
    // pad hits: a learned control strikes pad N of the current bank on every screen
    ActionInfo {
        name: "pad.1",
        label: "pad 1",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.2",
        label: "pad 2",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.3",
        label: "pad 3",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.4",
        label: "pad 4",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.5",
        label: "pad 5",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.6",
        label: "pad 6",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.7",
        label: "pad 7",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.8",
        label: "pad 8",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.9",
        label: "pad 9",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.10",
        label: "pad 10",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.11",
        label: "pad 11",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.12",
        label: "pad 12",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.13",
        label: "pad 13",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.14",
        label: "pad 14",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.15",
        label: "pad 15",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.16",
        label: "pad 16",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "metronome.click",
        label: "test click",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "record.mic",
        label: "record microphone",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "record.output",
        label: "record output",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "project.pick",
        label: "project picker",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "ui.pads_toggle",
        label: "hide / show the pads",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.take",
        label: "take pad to swap",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "master.volume_down",
        label: "master volume -1 dB",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "master.volume_up",
        label: "master volume +1 dB",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "master.volume",
        label: "master volume (knob)",
        group: Group::Global,
        knob: true,
    },
    ActionInfo {
        name: "edit.param_next",
        label: "parameter next",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "edit.param_prev",
        label: "parameter previous",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "edit.page_next",
        label: "page next",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "edit.page_prev",
        label: "page previous",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "edit.value_dec",
        label: "value down",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "edit.value_inc",
        label: "value up",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "edit.value_dec_fine",
        label: "value down fine",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "edit.value_inc_fine",
        label: "value up fine",
        group: Group::Global,
        knob: false,
    },
    ActionInfo {
        name: "pad.select_left",
        label: "select pad left",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.select_right",
        label: "select pad right",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.select_up",
        label: "select pad up",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.select_down",
        label: "select pad down",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.copy",
        label: "copy pad",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.mode_next",
        label: "pad mode",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.edit",
        label: "edit pad",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.edit_only",
        label: "open pad edit only",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.param_next",
        label: "next parameter",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.param_dec",
        label: "picked parameter down",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.param_inc",
        label: "picked parameter up",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.mark_bank",
        label: "mark the bank",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.mark_toggle",
        label: "marking on / off",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.rename",
        label: "name the selected pad",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.clear",
        label: "clear the selected pad",
        group: Group::Pads,
        knob: false,
    },
    ActionInfo {
        name: "pad.volume",
        label: "pad volume (knob)",
        group: Group::Pads,
        knob: true,
    },
    ActionInfo {
        name: "pad.pitch",
        label: "pad pitch (knob)",
        group: Group::Pads,
        knob: true,
    },
    ActionInfo {
        name: "pad.attack",
        label: "pad attack (knob)",
        group: Group::Pads,
        knob: true,
    },
    ActionInfo {
        name: "pad.decay",
        label: "pad decay (knob)",
        group: Group::Pads,
        knob: true,
    },
    ActionInfo {
        name: "pad.release",
        label: "pad release (knob)",
        group: Group::Pads,
        knob: true,
    },
    ActionInfo {
        name: "pad.filter",
        label: "pad filter (knob)",
        group: Group::Pads,
        knob: true,
    },
    ActionInfo {
        name: "pad.cutoff",
        label: "pad cutoff (knob)",
        group: Group::Pads,
        knob: true,
    },
    ActionInfo {
        name: "pad.reso",
        label: "pad resonance (knob)",
        group: Group::Pads,
        knob: true,
    },
    ActionInfo {
        name: "pad.start",
        label: "pad start (knob)",
        group: Group::Pads,
        knob: true,
    },
    ActionInfo {
        name: "pad.end",
        label: "pad end (knob)",
        group: Group::Pads,
        knob: true,
    },
    // the edit layer of the pads screen (28): the page knobs pick the parameter of the
    // page shown, the buttons walk it and leave
    ActionInfo {
        name: "edit.param_1",
        label: "knob 1 of the page",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "edit.param_2",
        label: "knob 2 of the page",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "edit.param_3",
        label: "knob 3 of the page",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "edit.param_prev",
        label: "previous parameter",
        group: Group::PadsEdit,
        knob: false,
    },
    ActionInfo {
        name: "edit.param_next",
        label: "next parameter",
        group: Group::PadsEdit,
        knob: false,
    },
    ActionInfo {
        name: "edit.page_prev",
        label: "previous page",
        group: Group::PadsEdit,
        knob: false,
    },
    ActionInfo {
        name: "edit.page_next",
        label: "next page",
        group: Group::PadsEdit,
        knob: false,
    },
    ActionInfo {
        name: "edit.page",
        label: "page (knob)",
        group: Group::PadsEdit,
        knob: true,
    },
    // the parameters bound in the edit layer by F10 (57) show on F12 here
    ActionInfo {
        name: "pad.volume",
        label: "pad volume (knob)",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "pad.pitch",
        label: "pad pitch (knob)",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "pad.attack",
        label: "pad attack (knob)",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "pad.decay",
        label: "pad decay (knob)",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "pad.release",
        label: "pad release (knob)",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "pad.filter",
        label: "pad filter (knob)",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "pad.cutoff",
        label: "pad cutoff (knob)",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "pad.reso",
        label: "pad resonance (knob)",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "pad.start",
        label: "pad start (knob)",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "pad.end",
        label: "pad end (knob)",
        group: Group::PadsEdit,
        knob: true,
    },
    ActionInfo {
        name: "ui.back",
        label: "leave the edit",
        group: Group::PadsEdit,
        knob: false,
    },
    ActionInfo {
        name: "settings.row_prev",
        label: "row up",
        group: Group::Settings,
        knob: false,
    },
    ActionInfo {
        name: "settings.row_next",
        label: "row down",
        group: Group::Settings,
        knob: false,
    },
    ActionInfo {
        name: "settings.value_prev",
        label: "value back",
        group: Group::Settings,
        knob: false,
    },
    ActionInfo {
        name: "settings.value_next",
        label: "value forward",
        group: Group::Settings,
        knob: false,
    },
    ActionInfo {
        name: "settings.tab_next",
        label: "next tab",
        group: Group::Settings,
        knob: false,
    },
    ActionInfo {
        name: "settings.learn",
        label: "learn",
        group: Group::Settings,
        knob: false,
    },
    ActionInfo {
        name: "settings.clear",
        label: "clear",
        group: Group::Settings,
        knob: false,
    },
    ActionInfo {
        name: "browser.up",
        label: "file up",
        group: Group::Files,
        knob: false,
    },
    ActionInfo {
        name: "browser.down",
        label: "file down",
        group: Group::Files,
        knob: false,
    },
    ActionInfo {
        name: "browser.enter",
        label: "open folder",
        group: Group::Files,
        knob: false,
    },
    ActionInfo {
        name: "browser.parent",
        label: "parent folder",
        group: Group::Files,
        knob: false,
    },
    ActionInfo {
        name: "browser.hold",
        label: "take file / open folder",
        group: Group::Files,
        knob: false,
    },
    ActionInfo {
        name: "browser.take_only",
        label: "take file / open folder only",
        group: Group::Files,
        knob: false,
    },
    ActionInfo {
        name: "browser.load",
        label: "load the file onto the selected pad",
        group: Group::Files,
        knob: false,
    },
    ActionInfo {
        name: "browser.preview_toggle",
        label: "preview on/off",
        group: Group::Files,
        knob: false,
    },
    ActionInfo {
        name: "browser.filter",
        label: "filter by name",
        group: Group::Files,
        knob: false,
    },
    ActionInfo {
        name: "browser.favorite_toggle",
        label: "favorite folder on/off",
        group: Group::Files,
        knob: false,
    },
    ActionInfo {
        name: "transport.play_stop",
        label: "play / stop",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.cursor_next_cut",
        label: "cursor to the next cut",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.cursor_prev_cut",
        label: "cursor to the previous cut",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.play_mode",
        label: "play: to end / to cut / slice",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.loop_toggle",
        label: "loop the play",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.as_pad_toggle",
        label: "play as the pad",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.cut",
        label: "cut at the cursor",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.auto_mode_next",
        label: "equal parts 2-32",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.parts",
        label: "equal parts only",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.hits",
        label: "chop by hits",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.hits_back",
        label: "chop by hits, one step back",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.assign_order",
        label: "assign: in order / by hand",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "slice.start_left",
        label: "slice start left",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "slice.start_right",
        label: "slice start right",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "slice.end_left",
        label: "slice end left",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "slice.end_right",
        label: "slice end right",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "marker.prev",
        label: "slice previous",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.region_start",
        label: "region start",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.region_end",
        label: "region end",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.region_clear",
        label: "region clear",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.cursor_left",
        label: "cursor left",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.cursor_right",
        label: "cursor right",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.cursor_left_fine",
        label: "cursor left, fine",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.cursor_right_fine",
        label: "cursor right, fine",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.snap_toggle",
        label: "cursor: steps / hits / cuts",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "marker.next",
        label: "slice next",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "marker.nudge_left",
        label: "slice start left",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "marker.nudge_right",
        label: "slice start right",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "marker.nudge_left_fine",
        label: "slice start left, fine",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "marker.nudge_right_fine",
        label: "slice start right, fine",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "slice.start",
        label: "slice start (knob)",
        group: Group::Chop,
        knob: true,
    },
    ActionInfo {
        name: "slice.end",
        label: "slice end (knob)",
        group: Group::Chop,
        knob: true,
    },
    ActionInfo {
        name: "chop.zoom",
        label: "zoom (knob)",
        group: Group::Chop,
        knob: true,
    },
    ActionInfo {
        name: "chop.cursor_pos",
        label: "cursor (knob)",
        group: Group::Chop,
        knob: true,
    },
    ActionInfo {
        name: "chop.region_start_pos",
        label: "region start (knob)",
        group: Group::Chop,
        knob: true,
    },
    ActionInfo {
        name: "chop.region_end_pos",
        label: "region end (knob)",
        group: Group::Chop,
        knob: true,
    },
    ActionInfo {
        name: "chop.threshold",
        label: "chop threshold (knob)",
        group: Group::Chop,
        knob: true,
    },
    ActionInfo {
        name: "marker.delete",
        label: "delete the cut",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "slice.end_delete",
        label: "delete the slice end",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "edit.undo",
        label: "undo",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.commit",
        label: "assign mode / apply",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.edit",
        label: "edit slice / apply",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.edit_only",
        label: "open slice edit only",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.assign",
        label: "slices onto pads",
        group: Group::Chop,
        knob: false,
    },
    ActionInfo {
        name: "chop.normalize",
        label: "normalize the pad",
        group: Group::Chop,
        knob: false,
    },
    // the edit layer of the chop screen (28): the same layout of knobs and buttons,
    // BOUNDS has two parameters until 04d brings the threshold
    ActionInfo {
        name: "edit.param_1",
        label: "knob 1 of the page",
        group: Group::ChopEdit,
        knob: true,
    },
    ActionInfo {
        name: "edit.param_2",
        label: "knob 2 of the page",
        group: Group::ChopEdit,
        knob: true,
    },
    ActionInfo {
        name: "edit.param_3",
        label: "knob 3 of the page",
        group: Group::ChopEdit,
        knob: true,
    },
    ActionInfo {
        name: "edit.param_prev",
        label: "previous parameter",
        group: Group::ChopEdit,
        knob: false,
    },
    ActionInfo {
        name: "edit.param_next",
        label: "next parameter",
        group: Group::ChopEdit,
        knob: false,
    },
    // the parameters bound in the edit layer by F10 (57) show on F12 here
    ActionInfo {
        name: "slice.start",
        label: "slice start (knob)",
        group: Group::ChopEdit,
        knob: true,
    },
    ActionInfo {
        name: "slice.end",
        label: "slice end (knob)",
        group: Group::ChopEdit,
        knob: true,
    },
    ActionInfo {
        name: "chop.zoom",
        label: "zoom (knob)",
        group: Group::ChopEdit,
        knob: true,
    },
    ActionInfo {
        name: "chop.cursor_pos",
        label: "cursor (knob)",
        group: Group::ChopEdit,
        knob: true,
    },
    ActionInfo {
        name: "chop.region_start_pos",
        label: "region start (knob)",
        group: Group::ChopEdit,
        knob: true,
    },
    ActionInfo {
        name: "chop.region_end_pos",
        label: "region end (knob)",
        group: Group::ChopEdit,
        knob: true,
    },
    ActionInfo {
        name: "chop.threshold",
        label: "chop threshold (knob)",
        group: Group::ChopEdit,
        knob: true,
    },
    ActionInfo {
        name: "ui.back",
        label: "leave the edit",
        group: Group::ChopEdit,
        knob: false,
    },
    ActionInfo {
        name: "edit.param_1",
        label: "knob 1: effect type",
        group: Group::Fx,
        knob: true,
    },
    ActionInfo {
        name: "edit.param_2",
        label: "knob 2: first parameter",
        group: Group::Fx,
        knob: true,
    },
    ActionInfo {
        name: "edit.param_3",
        label: "knob 3: second parameter",
        group: Group::Fx,
        knob: true,
    },
    ActionInfo {
        name: "edit.page",
        label: "effect block (knob)",
        group: Group::Fx,
        knob: true,
    },
    ActionInfo {
        name: "edit.page_prev",
        label: "previous effect block",
        group: Group::Fx,
        knob: false,
    },
    ActionInfo {
        name: "edit.page_next",
        label: "next effect block",
        group: Group::Fx,
        knob: false,
    },
    ActionInfo {
        name: "edit.param_prev",
        label: "previous parameter",
        group: Group::Fx,
        knob: false,
    },
    ActionInfo {
        name: "edit.param_next",
        label: "next parameter",
        group: Group::Fx,
        knob: false,
    },
    ActionInfo {
        name: "edit.value_dec",
        label: "decrease value",
        group: Group::Fx,
        knob: false,
    },
    ActionInfo {
        name: "edit.value_inc",
        label: "increase value",
        group: Group::Fx,
        knob: false,
    },
];

/// A key with or without Ctrl, Alt and Shift; characters are lowercase Latin. `Shift+Tab`
/// is the `BackTab` code with `shift` false: it is its own key, not a shifted command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Chord {
    code: KeyCode,
    ctrl: bool,
    alt: bool,
    shift: bool,
}

/// The pad block in Latin, in pad order.
/// Only US and Russian (ЙЦУКЕН) are known: crossterm 0.29 drops the base layout key of the kitty
/// protocol, so other layouts need reading that key ourselves.
const PADS_US: [char; 16] = [
    '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v',
];
/// Without the kitty protocol, Shift+digit arrives as a symbol: US and Russian layouts.
const SHIFTED_DIGITS: [(char, char); 7] = [
    ('!', '1'),
    ('@', '2'),
    ('#', '3'),
    ('$', '4'),
    ('"', '2'),
    ('№', '3'),
    (';', '4'),
];

/// Key character to the Latin character of the same physical key (known layouts only).
fn normalize(c: char) -> char {
    let c = c.to_lowercase().next().unwrap_or(c);
    if let Some(&(_, d)) = SHIFTED_DIGITS.iter().find(|(s, _)| *s == c) {
        return d;
    }
    ru_to_us(c).unwrap_or(c)
}

fn ru_to_us(c: char) -> Option<char> {
    const RU: &str = "йцукенгшщзхъфывапролджэячсмитьбю";
    const US: &str = "qwertyuiop[]asdfghjkl;'zxcvbnm,.";
    RU.chars()
        .position(|r| r == c)
        .and_then(|i| US.chars().nth(i))
}

/// Which keys are down, to tell a held key from a new press. Ghostty sends a held key's
/// auto-repeat as plain presses; with the kitty protocol a release comes when the key goes
/// up, so a second press without a release is the repeat of a held key.
#[derive(Default)]
pub struct HeldKeys(std::collections::HashSet<KeyCode>);

impl HeldKeys {
    /// The event as it is meant: a press of a key that is still down becomes a repeat.
    pub fn repeat(&mut self, e: &KeyEvent) -> KeyEvent {
        let code = match e.code {
            KeyCode::Char(c) => KeyCode::Char(normalize(c)),
            c => c,
        };
        match e.kind {
            KeyEventKind::Release => {
                self.0.remove(&code);
                *e
            }
            KeyEventKind::Press if !self.0.insert(code) => KeyEvent {
                kind: KeyEventKind::Repeat,
                ..*e
            },
            _ => *e,
        }
    }
}

/// Which pad keys (and the record-microphone hold key) are held. Ghostty sends key
/// auto-repeat as plain presses (not as repeat events), so a held key would retrigger
/// the pad or stop the recording; only the first press counts.
#[derive(Default)]
pub struct HeldPads([bool; 16], bool);

impl HeldPads {
    /// Whether a pad key was accepted by the held-key filter and has not released.
    pub fn is_held(&self, pad: usize) -> bool {
        self.0.get(pad).copied().unwrap_or(false)
    }

    /// Drops presses of a key that is already held; passes everything else through.
    pub fn filter(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::PadPress { pad, .. } if pad < 16 => {
                (!std::mem::replace(&mut self.0[pad], true)).then_some(action)
            }
            Action::PadRelease { pad } if pad < 16 => {
                self.0[pad] = false;
                Some(action)
            }
            Action::RecordMic(on) => {
                let was_held = std::mem::replace(&mut self.1, on);
                (!(on && was_held)).then_some(action)
            }
            other => Some(other),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
}

/// Which modifier key is held, for the help line. The kitty protocol reports Ctrl, Alt
/// and Shift alone as their own press and release events.
#[derive(Default)]
pub struct HeldModifiers {
    ctrl: bool,
    alt: bool,
    shift: bool,
}

impl HeldModifiers {
    /// Ctrl wins over Alt (Ctrl+Alt is the fine step, listed under Ctrl), Alt over Shift.
    pub fn get(&self) -> Option<Modifier> {
        if self.ctrl {
            Some(Modifier::Ctrl)
        } else if self.alt {
            Some(Modifier::Alt)
        } else if self.shift {
            Some(Modifier::Shift)
        } else {
            None
        }
    }

    /// Follows one key event; `typing` (the text line is open) forgets both, Shift there
    /// only makes capitals.
    pub fn update(&mut self, e: &KeyEvent, typing: bool) {
        use crossterm::event::ModifierKeyCode as M;
        if typing {
            *self = Self::default();
            return;
        }
        let down = e.kind != KeyEventKind::Release;
        match e.code {
            KeyCode::Modifier(M::LeftControl | M::RightControl) => self.ctrl = down,
            KeyCode::Modifier(M::LeftShift | M::RightShift) => self.shift = down,
            KeyCode::Modifier(M::LeftAlt | M::RightAlt) => self.alt = down,
            KeyCode::Modifier(_) => {}
            // a key without the modifier: its release got lost (focus went away)
            _ => {
                self.ctrl &= e.modifiers.contains(KeyModifiers::CONTROL);
                self.shift &= e.modifiers.contains(KeyModifiers::SHIFT);
                self.alt &= e.modifiers.contains(KeyModifiers::ALT);
            }
        }
    }
}

pub struct Keymap {
    /// `None` is an explicit `none` of the keymap: it runs nothing and blocks the Shift
    /// and Tab fallbacks, unlike a chord that no entry names.
    global: HashMap<Chord, Option<Action>>,
    /// The edit layer: over the screen keys while an edit mode is open, the same on
    /// every screen.
    edit: HashMap<Chord, Option<Action>>,
    screens: HashMap<Screen, HashMap<Chord, Option<Action>>>,
    default_velocity: f32,
    shift_velocity: f32,
}

impl Keymap {
    /// Reads `keymap.toml`. The second part holds warnings: forbidden and unknown keys.
    pub fn from_toml(text: &str) -> Result<(Self, Vec<String>), String> {
        let mut k = Self {
            global: HashMap::new(),
            edit: HashMap::new(),
            screens: HashMap::new(),
            default_velocity: 80.0 / 127.0,
            shift_velocity: 1.0,
        };
        let warnings = k.overlay(text)?;
        Ok((k, warnings))
    }

    /// Lays a keymap file over this one (64): a key of a section replaces that key of the
    /// section, `"none"` keeps it silent, the rest stays. Returns the warnings; an error
    /// names the line and leaves the keymap as it was.
    pub fn overlay(&mut self, text: &str) -> Result<Vec<String>, String> {
        let t: toml::Table = text.parse().map_err(|e: toml::de::Error| {
            let line = e
                .span()
                .map_or(0, |s| text[..s.start].lines().count().max(1));
            format!("line {line}: {}", e.message())
        })?;
        let mut warnings = Vec::new();
        for (key, v) in [
            ("default_velocity", &mut self.default_velocity),
            ("shift_velocity", &mut self.shift_velocity),
        ] {
            if let Some(n) = t
                .get("pads")
                .and_then(|p| p.get(key))
                .and_then(|v| v.as_integer())
            {
                *v = n.clamp(1, 127) as f32 / 127.0;
            }
        }
        let mut section =
            |name: &str, table: Option<&toml::Value>, map: &mut HashMap<Chord, Option<Action>>| {
                for (key, action) in table.and_then(|v| v.as_table()).into_iter().flatten() {
                    let chord = match parse_chord(key) {
                        Ok(chord) => chord,
                        Err(e) => {
                            warnings.push(format!("keymap [{name}] {key}: {e}"));
                            continue;
                        }
                    };
                    match action.as_str() {
                        // `none` is remembered, not removed: it blocks the fallback of
                        // the shifted twin; a later overlay may bind the key again
                        Some("none") => {
                            map.insert(chord, None);
                        }
                        a => match a.and_then(Action::parse) {
                            Some(action) => {
                                map.insert(chord, Some(action));
                            }
                            // a binding to nothing is named, not skipped quietly (71)
                            None => warnings.push(format!(
                                "keymap [{name}] {key}: no such action {}",
                                a.unwrap_or("(not a name)")
                            )),
                        },
                    }
                }
            };
        section("global", t.get("global"), &mut self.global);
        section("edit", t.get("edit"), &mut self.edit);
        let screens_t = t.get("screens");
        for (screen, name) in [
            (Screen::Pads, "pads"),
            (Screen::Settings, "settings"),
            (Screen::Files, "files"),
            (Screen::Chop, "chop"),
            (Screen::Fx, "fx"),
        ] {
            let map = self.screens.entry(screen).or_default();
            section(name, screens_t.and_then(|s| s.get(name)), map);
        }
        Ok(warnings)
    }

    /// Action for a key event on `screen`.
    pub fn resolve(&self, screen: Screen, e: &KeyEvent) -> Option<Action> {
        self.resolve_in(screen, false, e)
    }

    /// Action for a key event on `screen`; with `editing` the edit layer comes first.
    pub fn resolve_in(&self, screen: Screen, editing: bool, e: &KeyEvent) -> Option<Action> {
        let ctrl = e.modifiers.contains(KeyModifiers::CONTROL);
        let alt = e.modifiers.contains(KeyModifiers::ALT);
        if let KeyCode::Char(c) = e.code
            && !ctrl
            && !alt
            && let Some(pad) = PADS_US.iter().position(|&p| p == normalize(c))
        {
            return match e.kind {
                KeyEventKind::Press => Some(Action::PadPress {
                    pad,
                    velocity: if e.modifiers.contains(KeyModifiers::SHIFT) || is_shifted(c) {
                        self.shift_velocity
                    } else {
                        self.default_velocity
                    },
                }),
                KeyEventKind::Release => Some(Action::PadRelease { pad }),
                // auto-repeat must not turn a held key into a roll
                KeyEventKind::Repeat => None,
            };
        }
        if e.kind == KeyEventKind::Repeat {
            // a held key walks the cursor, a cut, the zoom, the file list or the settings rows;
            // everything else acts once
            return self
                .resolve_in(
                    screen,
                    editing,
                    &KeyEvent {
                        kind: KeyEventKind::Press,
                        ..*e
                    },
                )
                .filter(|a| {
                    matches!(
                        a,
                        Action::CursorMove { .. }
                            | Action::MarkerNudge { .. }
                            | Action::EditValue { .. }
                            | Action::SliceBound { .. }
                            | Action::ChopZoom { .. }
                            | Action::BrowserUp
                            | Action::BrowserDown
                            | Action::SettingsRow(_)
                    )
                })
                .map(|a| with_step(a, false));
        }
        if e.kind != KeyEventKind::Press {
            // a hold action stops on the release of its key
            if e.kind == KeyEventKind::Release {
                let (code, shift) = chord_parts(e);
                // the Shift key may come up before or after the key of the chord, so
                // both twins answer the microphone question
                let mic = [shift, !shift].iter().any(|shift| {
                    let chord = Chord {
                        code,
                        ctrl,
                        alt,
                        shift: *shift,
                    };
                    matches!(
                        self.screens
                            .get(&screen)
                            .and_then(|m| m.get(&chord))
                            .or_else(|| self.global.get(&chord)),
                        Some(&Some(Action::RecordMic(_)))
                    )
                });
                if mic {
                    return Some(Action::RecordMic(false));
                }
            }
            return None;
        }
        let (code, shift) = chord_parts(e);
        self.bound(
            screen,
            editing,
            Chord {
                code,
                ctrl,
                alt,
                shift,
            },
        )
        .map(|a| with_step(a, true))
    }

    /// The entry of the chord: `Some(&Some(action))` when a section binds it, `Some(&None)`
    /// for an explicit `none`, `None` when no entry names it. The edit layer comes first
    /// when it is open, then the screen, then the shared keys.
    fn entry(&self, screen: Screen, editing: bool, c: Chord) -> Option<&Option<Action>> {
        editing
            .then(|| self.edit.get(&c))
            .flatten()
            .or_else(|| self.screens.get(&screen).and_then(|m| m.get(&c)))
            .or_else(|| self.global.get(&c))
    }

    /// The action the chord runs. A Shift chord no entry names acts as its plain chord,
    /// as before Shift commands; an explicit `none`, of either twin, runs nothing and
    /// blocks the fallback. `Shift+Tab` no entry names acts as `Tab`, as before the page
    /// keys (54); its explicit `none` blocks that fallback too.
    fn bound(&self, screen: Screen, editing: bool, c: Chord) -> Option<Action> {
        match self.entry(screen, editing, c) {
            Some(&Some(a)) => Some(a),
            Some(&None) => None,
            None => {
                if c.shift {
                    match self.entry(screen, editing, Chord { shift: false, ..c }) {
                        Some(&Some(a)) => Some(a),
                        _ => None,
                    }
                } else if c.code == KeyCode::BackTab {
                    self.entry(
                        screen,
                        editing,
                        Chord {
                            code: KeyCode::Tab,
                            shift: false,
                            ..c
                        },
                    )
                    .and_then(|a| *a)
                } else {
                    None
                }
            }
        }
    }

    /// Looks up an explicit chord in edit, screen and global priority order. An outer
    /// `None` means no entry (or an invalid chord); `Some(None)` means an explicit removal.
    /// Unlike resolution, this does not use Shift or Tab fallback or change action steps.
    pub fn binding_for(&self, screen: Screen, editing: bool, key: &str) -> Option<Option<Action>> {
        self.entry(screen, editing, parse_chord(key).ok()?).copied()
    }

    /// Keys of an action in the edit layer, for the help line of the edit mode.
    pub fn edit_keys_for(&self, action: Action) -> Vec<String> {
        let mut keys: Vec<String> = self
            .edit
            .iter()
            .filter(|(_, a)| **a == Some(action))
            .map(|(c, _)| chord_name(c))
            .collect();
        keys.sort();
        keys
    }

    /// Keys for an action on a screen, for the help line; screen keys first, then shared ones.
    pub fn keys_for(&self, screen: Screen, action: Action) -> Vec<String> {
        let mut keys: Vec<String> = self
            .screens
            .get(&screen)
            .into_iter()
            .chain([&self.global])
            .flat_map(|m| {
                m.iter()
                    .filter(|(_, a)| **a == Some(action))
                    .map(|(c, _)| chord_name(c))
            })
            .collect();
        keys.sort();
        keys.dedup();
        keys
    }

    /// All bindings of a screen plus shared ones, for the full help (F1).
    pub fn bindings(&self, screen: Screen) -> Vec<(String, Action)> {
        let mut all: Vec<_> = self
            .screens
            .get(&screen)
            .into_iter()
            .chain([&self.global])
            .flat_map(|m| {
                m.iter()
                    .filter_map(|(c, a)| a.as_ref().map(|a| (chord_name(c), *a)))
            })
            .collect();
        all.sort_by(|a, b| a.0.cmp(&b.0));
        all
    }
}

/// A character that, without the kitty protocol, means Shift+digit or an uppercase letter.
fn is_shifted(c: char) -> bool {
    c.is_uppercase() || SHIFTED_DIGITS.iter().any(|(s, _)| *s == c)
}

/// The chord of an event: the normalized key, and whether Shift is part of it. Without
/// the kitty protocol Shift arrives as an uppercase character; `Shift+Tab` is the
/// `BackTab` key in both protocols, never a shifted command.
fn chord_parts(e: &KeyEvent) -> (KeyCode, bool) {
    match e.code {
        KeyCode::Char(c) => (
            KeyCode::Char(normalize(c)),
            e.modifiers.contains(KeyModifiers::SHIFT) || is_shifted(c),
        ),
        // with the kitty protocol Shift+Tab may come as Tab with Shift
        KeyCode::Tab if e.modifiers.contains(KeyModifiers::SHIFT) => (KeyCode::BackTab, false),
        KeyCode::BackTab => (KeyCode::BackTab, false),
        c => (c, e.modifiers.contains(KeyModifiers::SHIFT)),
    }
}

/// A tap of a value or a bound key is the fine step, a held key repeats the coarse one;
/// the other actions pass as they are.
fn with_step(a: Action, tap: bool) -> Action {
    match a {
        Action::EditValue { up, .. } => Action::EditValue { up, fine: tap },
        Action::SliceBound { end, right, .. } => Action::SliceBound {
            end,
            right,
            fine: tap,
        },
        Action::CursorMove { right, .. } => Action::CursorMove { right, fine: tap },
        a => a,
    }
}

fn parse_chord(key: &str) -> Result<Chord, String> {
    // any order of the modifiers is fine: Ctrl+Alt+Left, Alt+Ctrl+Left, Ctrl+Shift+H;
    // `Shift+Tab` is its own key, not a shifted command
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut name = key;
    for _ in 0..3 {
        if let Some(rest) = name.strip_prefix("Ctrl+") {
            ctrl = true;
            name = rest;
        } else if let Some(rest) = name.strip_prefix("Alt+") {
            alt = true;
            name = rest;
        } else if name != "Shift+Tab"
            && let Some(rest) = name.strip_prefix("Shift+")
        {
            shift = true;
            name = rest;
        }
    }
    let code = match name {
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "Enter" => KeyCode::Enter,
        "Esc" => KeyCode::Esc,
        "Tab" => KeyCode::Tab,
        "Shift+Tab" => KeyCode::BackTab,
        "Backspace" => KeyCode::Backspace,
        "Space" => KeyCode::Char(' '),
        "Minus" => KeyCode::Char('-'),
        "Equal" => KeyCode::Char('='),
        "Comma" => KeyCode::Char(','),
        "Period" => KeyCode::Char('.'),
        "Semicolon" => KeyCode::Char(';'),
        "Quote" => KeyCode::Char('\''),
        "Slash" => KeyCode::Char('/'),
        f if f.starts_with('F') && f.len() > 1 => {
            KeyCode::F(f[1..].parse().map_err(|_| "unknown key".to_string())?)
        }
        c if c.chars().count() == 1 => KeyCode::Char(normalize(c.chars().next().unwrap_or(' '))),
        _ => return Err("unknown key".into()),
    };
    if let KeyCode::Char(c) = code
        && !ctrl
        && !alt
        && PADS_US.contains(&c)
    {
        return Err("pad keys cannot be commands".into());
    }
    Ok(Chord {
        code,
        ctrl,
        alt,
        shift,
    })
}

/// The normalized chord name of an event, independent of keyboard layout or event kind.
/// Uppercase legacy characters name Shift chords, as extended Shift events do.
pub fn key_name(e: &KeyEvent) -> String {
    let (code, shift) = chord_parts(e);
    chord_name(&Chord {
        code,
        ctrl: e.modifiers.contains(KeyModifiers::CONTROL),
        alt: e.modifiers.contains(KeyModifiers::ALT),
        shift,
    })
}

fn chord_name(c: &Chord) -> String {
    let key = match c.code {
        KeyCode::Up => "↑".into(),
        KeyCode::Down => "↓".into(),
        KeyCode::Left => "←".into(),
        KeyCode::Right => "→".into(),
        KeyCode::PageUp => "PgUp".into(),
        KeyCode::PageDown => "PgDn".into(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Esc => "Esc".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::BackTab => "Shift+Tab".into(),
        KeyCode::Backspace => "Bksp".into(),
        KeyCode::Char(' ') => "Space".into(),
        KeyCode::Char(ch) => ch.to_uppercase().to_string(),
        KeyCode::F(n) => format!("F{n}"),
        other => format!("{other:?}"),
    };
    if c.ctrl || c.alt || c.shift {
        let mut name = String::new();
        if c.ctrl {
            name.push_str("Ctrl+");
        }
        if c.alt {
            name.push_str("Alt+");
        }
        if c.shift {
            name.push_str("Shift+");
        }
        name.push_str(&key);
        name
    } else {
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventState;

    const PADS_RU: [char; 16] = [
        '1', '2', '3', '4', 'й', 'ц', 'у', 'к', 'ф', 'ы', 'в', 'а', 'я', 'ч', 'с', 'м',
    ];

    const KEYMAP: &str = include_str!("../../../config/keymap.toml");

    #[test]
    fn open_the_shared_fx_screen() {
        let k = keymap();
        for screen in [
            Screen::Pads,
            Screen::Chop,
            Screen::Files,
            Screen::Settings,
            Screen::Fx,
        ] {
            assert_eq!(
                k.resolve(
                    screen,
                    &ev(KeyCode::F(4), KeyModifiers::NONE, KeyEventKind::Press)
                ),
                Some(Action::ScreenFx)
            );
        }
        assert_eq!(Action::parse("screen.fx"), Some(Action::ScreenFx));
        assert_eq!(k.keys_for(Screen::Fx, Action::ScreenFx), ["F4"]);
    }

    #[test]
    fn select_the_next_shared_effect_in_both_layouts() {
        let k = keymap();
        for layout in [['i', 'k', 'j', 'l'], ['ш', 'л', 'о', 'д']] {
            for (c, action) in layout.into_iter().zip([
                Action::ParamPrev,
                Action::ParamNext,
                Action::EditValue {
                    up: false,
                    fine: true,
                },
                Action::EditValue {
                    up: true,
                    fine: true,
                },
            ]) {
                assert_eq!(k.resolve(Screen::Fx, &press(c)), Some(action));
            }
        }
        for (code, mods, action) in [
            (KeyCode::Tab, KeyModifiers::NONE, Action::PageNext),
            (KeyCode::Tab, KeyModifiers::SHIFT, Action::PagePrev),
            (KeyCode::BackTab, KeyModifiers::SHIFT, Action::PagePrev),
            (KeyCode::Up, KeyModifiers::NONE, Action::ParamPrev),
            (KeyCode::Down, KeyModifiers::NONE, Action::ParamNext),
            (
                KeyCode::Left,
                KeyModifiers::NONE,
                Action::EditValue {
                    up: false,
                    fine: true,
                },
            ),
            (
                KeyCode::Right,
                KeyModifiers::NONE,
                Action::EditValue {
                    up: true,
                    fine: true,
                },
            ),
        ] {
            assert_eq!(
                k.resolve(Screen::Fx, &ev(code, mods, KeyEventKind::Press)),
                Some(action)
            );
        }
        for c in ['z', 'я'] {
            assert_eq!(
                k.resolve(
                    Screen::Fx,
                    &ev(KeyCode::Char(c), KeyModifiers::CONTROL, KeyEventKind::Press)
                ),
                Some(Action::Undo)
            );
        }
        assert!(
            k.bindings(Screen::Fx)
                .iter()
                .any(|(_, a)| *a == Action::PageNext)
        );
    }

    #[test]
    fn play_pads_on_the_fx_screen() {
        let k = keymap();
        for layout in [PADS_US, PADS_RU] {
            for (pad, c) in layout.into_iter().enumerate() {
                for editing in [false, true] {
                    assert_eq!(
                        k.resolve_in(Screen::Fx, editing, &press(c)),
                        Some(Action::PadPress {
                            pad,
                            velocity: 80.0 / 127.0
                        })
                    );
                    assert_eq!(
                        k.resolve_in(
                            Screen::Fx,
                            editing,
                            &ev(KeyCode::Char(c), KeyModifiers::SHIFT, KeyEventKind::Press)
                        ),
                        Some(Action::PadPress { pad, velocity: 1.0 })
                    );
                    assert_eq!(
                        k.resolve_in(
                            Screen::Fx,
                            editing,
                            &ev(KeyCode::Char(c), KeyModifiers::NONE, KeyEventKind::Repeat)
                        ),
                        None
                    );
                    assert_eq!(
                        k.resolve_in(
                            Screen::Fx,
                            editing,
                            &ev(KeyCode::Char(c), KeyModifiers::NONE, KeyEventKind::Release)
                        ),
                        Some(Action::PadRelease { pad })
                    );
                }
            }
        }
    }

    #[test]
    fn fresh_fx_group_keeps_the_old_group_order() {
        assert_eq!(
            &Group::ALL[..7],
            &[
                Group::Global,
                Group::Pads,
                Group::PadsEdit,
                Group::Settings,
                Group::Files,
                Group::Chop,
                Group::ChopEdit
            ]
        );
        assert_eq!(Group::ALL[7], Group::Fx);
        assert_eq!(Group::from_key("fx"), Some(Group::Fx));
        assert_eq!(Group::Fx.title(), "FX");
        assert_eq!(Group::Fx.scope(), midi::Scope::Screen(Screen::Fx));
        for name in [
            "edit.param_1",
            "edit.param_2",
            "edit.param_3",
            "edit.page",
            "edit.page_next",
            "edit.page_prev",
            "edit.param_next",
            "edit.param_prev",
            "edit.value_inc",
            "edit.value_dec",
        ] {
            assert!(
                ACTIONS
                    .iter()
                    .any(|a| a.group == Group::Fx && a.name == name),
                "{name}"
            );
        }
    }

    #[test]
    fn inspect_entries_keeps_removals_and_layer_priority() {
        let (k, warnings) = Keymap::from_toml(
            "[global]\nH = \"help.toggle\"\n\"Shift+H\" = \"help.toggle\"\n[edit]\nH = \"none\"\n[screens.chop]\nH = \"chop.hits\"\n",
        )
        .unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(
            k.binding_for(Screen::Pads, false, "H"),
            Some(Some(Action::HelpToggle))
        );
        assert_eq!(
            k.binding_for(Screen::Chop, false, "Р"),
            Some(Some(Action::ChopHits))
        );
        assert_eq!(k.binding_for(Screen::Chop, true, "H"), Some(None));
        assert_eq!(
            k.binding_for(Screen::Pads, false, "Shift+H"),
            Some(Some(Action::HelpToggle))
        );
        assert_eq!(k.binding_for(Screen::Pads, false, "Shift+J"), None);
        assert_eq!(k.binding_for(Screen::Pads, false, "not a key"), None);
        assert_eq!(k.binding_for(Screen::Pads, false, "Q"), None);
    }

    #[test]
    fn event_names_use_the_same_chords_as_keymap_entries() {
        for (us, ru) in [('r', 'к'), ('n', 'т'), ('y', 'н'), ('h', 'р')] {
            for mods in [
                KeyModifiers::NONE,
                KeyModifiers::CONTROL,
                KeyModifiers::ALT,
                KeyModifiers::SHIFT,
            ] {
                assert_eq!(
                    key_name(&ev(KeyCode::Char(us), mods, KeyEventKind::Press)),
                    key_name(&ev(KeyCode::Char(ru), mods, KeyEventKind::Release))
                );
            }
        }
        for c in ['H', 'Р'] {
            assert_eq!(key_name(&press(c)), "Shift+H");
        }
        assert_eq!(
            key_name(&ev(KeyCode::Tab, KeyModifiers::SHIFT, KeyEventKind::Press)),
            "Shift+Tab"
        );
        assert_eq!(
            key_name(&ev(
                KeyCode::BackTab,
                KeyModifiers::NONE,
                KeyEventKind::Press
            )),
            "Shift+Tab"
        );
    }

    fn keymap() -> Keymap {
        let (k, warnings) = Keymap::from_toml(KEYMAP).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        k
    }

    fn ev(code: KeyCode, mods: KeyModifiers, kind: KeyEventKind) -> KeyEvent {
        KeyEvent::new_with_kind_and_state(code, mods, kind, KeyEventState::NONE)
    }

    fn press(c: char) -> KeyEvent {
        ev(KeyCode::Char(c), KeyModifiers::NONE, KeyEventKind::Press)
    }

    fn modifier(m: crossterm::event::ModifierKeyCode, kind: KeyEventKind) -> KeyEvent {
        use crossterm::event::ModifierKeyCode as M;
        let mods = match m {
            M::LeftShift | M::RightShift => KeyModifiers::SHIFT,
            _ => KeyModifiers::CONTROL,
        };
        ev(KeyCode::Modifier(m), mods, kind)
    }

    #[test]
    fn hold_ctrl_and_let_go() {
        use crossterm::event::ModifierKeyCode as M;
        let mut held = HeldModifiers::default();
        held.update(&modifier(M::LeftControl, KeyEventKind::Press), false);
        assert_eq!(held.get(), Some(Modifier::Ctrl));
        // Ghostty repeats a held modifier as presses; a Ctrl combo keeps it held
        held.update(&modifier(M::LeftControl, KeyEventKind::Press), false);
        let ctrl_r = ev(
            KeyCode::Char('r'),
            KeyModifiers::CONTROL,
            KeyEventKind::Press,
        );
        held.update(&ctrl_r, false);
        assert_eq!(held.get(), Some(Modifier::Ctrl));
        held.update(&modifier(M::LeftControl, KeyEventKind::Release), false);
        assert_eq!(held.get(), None);
    }

    #[test]
    fn hold_alt_and_let_go() {
        use crossterm::event::ModifierKeyCode as M;
        let mut held = HeldModifiers::default();
        held.update(&modifier(M::LeftAlt, KeyEventKind::Press), false);
        assert_eq!(held.get(), Some(Modifier::Alt));
        // Ctrl over Alt is the fine step: the Ctrl line lists it
        held.update(&modifier(M::LeftControl, KeyEventKind::Press), false);
        assert_eq!(held.get(), Some(Modifier::Ctrl));
        held.update(&modifier(M::LeftControl, KeyEventKind::Release), false);
        held.update(&modifier(M::LeftAlt, KeyEventKind::Release), false);
        assert_eq!(held.get(), None);
        // a lost release: the next key without Alt drops it
        held.update(&modifier(M::LeftAlt, KeyEventKind::Press), false);
        held.update(&press('q'), false);
        assert_eq!(held.get(), None);
    }

    #[test]
    fn hold_shift() {
        use crossterm::event::ModifierKeyCode as M;
        let mut held = HeldModifiers::default();
        held.update(&modifier(M::RightShift, KeyEventKind::Press), false);
        assert_eq!(held.get(), Some(Modifier::Shift));
        // a pad hit with Shift keeps the Shift line
        held.update(
            &ev(KeyCode::Char('Q'), KeyModifiers::SHIFT, KeyEventKind::Press),
            false,
        );
        assert_eq!(held.get(), Some(Modifier::Shift));
        held.update(&modifier(M::RightShift, KeyEventKind::Release), false);
        assert_eq!(held.get(), None);
    }

    #[test]
    fn a_lost_release() {
        use crossterm::event::ModifierKeyCode as M;
        let mut held = HeldModifiers::default();
        held.update(&modifier(M::LeftControl, KeyEventKind::Press), false);
        // the release went to another window; the next key comes without Ctrl
        held.update(&press('q'), false);
        assert_eq!(held.get(), None);
    }

    #[test]
    fn typing_a_filter() {
        use crossterm::event::ModifierKeyCode as M;
        let mut held = HeldModifiers::default();
        held.update(&modifier(M::LeftShift, KeyEventKind::Press), true);
        assert_eq!(
            held.get(),
            None,
            "Shift in the filter line only makes capitals"
        );
    }

    #[test]
    fn pad_block_in_us_and_russian_layouts() {
        let k = keymap();
        for (i, (us, ru)) in PADS_US.iter().zip(PADS_RU).enumerate() {
            for c in [*us, ru] {
                match k.resolve(Screen::Pads, &press(c)) {
                    Some(Action::PadPress { pad, .. }) => assert_eq!(pad, i, "{c}"),
                    other => panic!("{c}: {other:?}"),
                }
            }
        }
    }

    #[test]
    fn chop_arrows_move_the_cursor() {
        // a press is a tap: the fine step; the held key's repeats grow (21 "One speed")
        let k = keymap();
        match k.resolve(
            Screen::Chop,
            &ev(KeyCode::Left, KeyModifiers::NONE, KeyEventKind::Press),
        ) {
            Some(Action::CursorMove {
                right: false,
                fine: true,
            }) => {}
            other => panic!("Left: {other:?}"),
        }
        match k.resolve(
            Screen::Chop,
            &ev(KeyCode::Right, KeyModifiers::NONE, KeyEventKind::Press),
        ) {
            Some(Action::CursorMove {
                right: true,
                fine: true,
            }) => {}
            other => panic!("Right: {other:?}"),
        }
    }

    /// 32 "Hide the pads" and "The same on every screen": Ctrl+O is the switch on the
    /// screens with pads, in the Russian layout too (щ is the O key).
    #[test]
    fn ctrl_o_hides_the_pads_on_every_screen() {
        let k = keymap();
        for c in ['o', 'щ'] {
            let e = ev(KeyCode::Char(c), KeyModifiers::CONTROL, KeyEventKind::Press);
            for screen in [Screen::Pads, Screen::Files, Screen::Chop] {
                assert_eq!(
                    k.resolve(screen, &e),
                    Some(Action::PadsToggle),
                    "{c} on {screen:?}"
                );
            }
        }
    }

    #[test]
    fn record_keys_start_and_stop() {
        let k = keymap();
        let alt = |c, kind| ev(KeyCode::Char(c), KeyModifiers::ALT, kind);
        assert_eq!(
            k.resolve(Screen::Pads, &alt('r', KeyEventKind::Press)),
            Some(Action::RecordMic(true)),
            "pressing Alt+R starts the microphone"
        );
        assert_eq!(
            k.resolve(Screen::Pads, &alt('r', KeyEventKind::Release)),
            Some(Action::RecordMic(false)),
            "releasing Alt+R stops it"
        );
        assert_eq!(
            k.resolve(Screen::Files, &alt('r', KeyEventKind::Repeat)),
            None,
            "auto-repeat must not restart the recording"
        );
        let ctrl = |c, kind| ev(KeyCode::Char(c), KeyModifiers::CONTROL, kind);
        assert_eq!(
            k.resolve(Screen::Pads, &ctrl('t', KeyEventKind::Press)),
            Some(Action::RecordOutputToggle)
        );
        // the actions are listed for MIDI learn
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"record.mic") && names.contains(&"record.output"));
    }

    #[test]
    fn pad_hit_names_parse_and_are_listed() {
        assert_eq!(
            Action::parse("pad.1"),
            Some(Action::PadPress {
                pad: 0,
                velocity: 1.0
            })
        );
        assert_eq!(
            Action::parse("pad.16"),
            Some(Action::PadPress {
                pad: 15,
                velocity: 1.0
            })
        );
        assert_eq!(Action::parse("pad.0"), None);
        assert_eq!(Action::parse("pad.17"), None);
        // the other pad.* names keep their meaning
        assert_eq!(
            Action::parse("pad.select_up"),
            Some(Action::PadSelect(Dir::Up))
        );
        // all 16 are listed as global, press-like actions
        let hits: Vec<&ActionInfo> = ACTIONS
            .iter()
            .filter(|a| a.name.starts_with("pad.") && a.name[4..].parse::<usize>().is_ok())
            .collect();
        assert_eq!(hits.len(), 16);
        assert!(hits.iter().all(|a| a.group == Group::Global && !a.knob));
        assert_eq!(hits[0].name, "pad.1");
        assert_eq!(hits[15].label, "pad 16");
    }

    #[test]
    fn press_release_repeat_and_velocity() {
        let k = keymap();
        let shift = ev(KeyCode::Char('q'), KeyModifiers::SHIFT, KeyEventKind::Press);
        assert_eq!(
            k.resolve(Screen::Pads, &shift),
            Some(Action::PadPress {
                pad: 4,
                velocity: 1.0
            })
        );
        // without the kitty protocol Shift+й arrives as uppercase Й
        assert_eq!(
            k.resolve(Screen::Pads, &press('Й')),
            Some(Action::PadPress {
                pad: 4,
                velocity: 1.0
            })
        );
        assert_eq!(
            k.resolve(Screen::Pads, &press('!')),
            Some(Action::PadPress {
                pad: 0,
                velocity: 1.0
            })
        );
        let release = ev(
            KeyCode::Char('й'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        );
        assert_eq!(
            k.resolve(Screen::Settings, &release),
            Some(Action::PadRelease { pad: 4 })
        );
        let repeat = ev(KeyCode::Char('q'), KeyModifiers::NONE, KeyEventKind::Repeat);
        assert_eq!(k.resolve(Screen::Pads, &repeat), None);
        match k.resolve(Screen::Pads, &press('q')) {
            Some(Action::PadPress { velocity, .. }) => {
                assert!((velocity - 80.0 / 127.0).abs() < 1e-6)
            }
            other => panic!("{other:?}"),
        }
    }

    /// Scenario "Minus and equal are free" (editing): `-` and `=` on the pads screen are no command.
    #[test]
    fn commands_by_screen_and_ctrl_in_russian_layout() {
        let k = keymap();
        let key = |c| ev(c, KeyModifiers::NONE, KeyEventKind::Press);
        assert_eq!(
            k.resolve(Screen::Pads, &key(KeyCode::PageDown)),
            Some(Action::BankNext)
        );
        assert_eq!(
            k.resolve(Screen::Pads, &key(KeyCode::Enter)),
            Some(Action::PadEdit)
        );
        // H the pad mode, M the marking: the marker key (р ь in Russian)
        assert_eq!(
            k.resolve(Screen::Pads, &key(KeyCode::Char('h'))),
            Some(Action::PadModeNext)
        );
        assert_eq!(
            k.resolve(Screen::Pads, &press('р')),
            Some(Action::PadModeNext)
        );
        assert_eq!(
            k.resolve(Screen::Pads, &press('ь')),
            Some(Action::PadMarkToggle)
        );
        // 83a "Keyboard bank controls": U/O turn the banks, also in the Russian layout
        assert_eq!(k.resolve(Screen::Pads, &press('u')), Some(Action::BankPrev));
        assert_eq!(k.resolve(Screen::Pads, &press('г')), Some(Action::BankPrev));
        assert_eq!(k.resolve(Screen::Pads, &press('o')), Some(Action::BankNext));
        assert_eq!(k.resolve(Screen::Pads, &press('щ')), Some(Action::BankNext));
        assert_eq!(
            k.resolve(Screen::Settings, &key(KeyCode::Right)),
            Some(Action::SettingsValue(1))
        );
        // 83a: the direction keys change values, not banks, with the edit open or closed
        assert_eq!(
            k.resolve(Screen::Pads, &press('l')),
            Some(Action::EditValue {
                up: true,
                fine: true
            })
        );
        assert_eq!(
            k.resolve(Screen::Pads, &press('д')),
            Some(Action::EditValue {
                up: true,
                fine: true
            })
        );
        assert_eq!(
            k.resolve(Screen::Pads, &key(KeyCode::Left)),
            Some(Action::EditValue {
                up: false,
                fine: true
            })
        );
        // ↑ picks no pad: it walks the parameters, with the edit closed too (70)
        assert_eq!(
            k.resolve(Screen::Pads, &key(KeyCode::Up)),
            Some(Action::ParamPrev)
        );
        assert_eq!(k.resolve(Screen::Pads, &press('=')), None);
        assert_eq!(k.resolve(Screen::Pads, &press('-')), None);
        // master volume on [ and ], also as х and ъ in the Russian layout
        assert_eq!(
            k.resolve(Screen::Files, &press('[')),
            Some(Action::MasterVolume(-1))
        );
        assert_eq!(
            k.resolve(Screen::Pads, &press('ъ')),
            Some(Action::MasterVolume(1))
        );
        // Ctrl+digit picks a bank, the plain digit stays a pad
        let ctrl = |c| ev(KeyCode::Char(c), KeyModifiers::CONTROL, KeyEventKind::Press);
        assert_eq!(
            k.resolve(Screen::Pads, &ctrl('1')),
            Some(Action::BankSelect(0))
        );
        assert_eq!(
            k.resolve(Screen::Settings, &ctrl('8')),
            Some(Action::BankSelect(7))
        );
        assert!(matches!(
            k.resolve(Screen::Pads, &press('1')),
            Some(Action::PadPress { pad: 0, .. })
        ));
        let ctrl_q_ru = ev(
            KeyCode::Char('й'),
            KeyModifiers::CONTROL,
            KeyEventKind::Press,
        );
        assert_eq!(k.resolve(Screen::Pads, &ctrl_q_ru), Some(Action::Quit));
        let enter_release = ev(KeyCode::Enter, KeyModifiers::NONE, KeyEventKind::Release);
        assert_eq!(k.resolve(Screen::Pads, &enter_release), None);
    }

    /// Scenario "Pad block keys are not commands" (pad-playback): on every screen, with an
    /// edit open or not, a pad block key alone or with Shift plays a pad or does nothing;
    /// with Ctrl it is a command.
    #[test]
    fn pad_block_keys_are_not_commands() {
        let k = keymap();
        let screens = [Screen::Pads, Screen::Settings, Screen::Files, Screen::Chop];
        for c in PADS_US.iter().copied().chain(PADS_RU) {
            let up = c.to_uppercase().next().unwrap();
            let events = [
                press(c),
                ev(KeyCode::Char(c), KeyModifiers::SHIFT, KeyEventKind::Press),
                ev(KeyCode::Char(up), KeyModifiers::SHIFT, KeyEventKind::Press),
            ];
            for (screen, editing, e) in screens
                .iter()
                .flat_map(|&s| [(s, false), (s, true)])
                .flat_map(|(s, ed)| events.iter().map(move |e| (s, ed, e)))
            {
                let got = k.resolve_in(screen, editing, e);
                assert!(
                    matches!(got, None | Some(Action::PadPress { .. })),
                    "{screen:?} edit {editing} {e:?}: {got:?}"
                );
            }
        }
        let ctrl = |c| ev(KeyCode::Char(c), KeyModifiers::CONTROL, KeyEventKind::Press);
        assert_eq!(
            k.resolve(Screen::Pads, &ctrl('1')),
            Some(Action::BankSelect(0))
        );
        assert_eq!(
            k.resolve(Screen::Pads, &ctrl('8')),
            Some(Action::BankSelect(7))
        );
        assert_eq!(k.resolve(Screen::Pads, &ctrl('z')), Some(Action::Undo));
    }

    #[test]
    fn pad_keys_rejected_as_commands() {
        let (k, warnings) =
            Keymap::from_toml("[global]\nQ = \"bank.next\"\n\"Ctrl+Q\" = \"app.quit\"\n").unwrap();
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert_eq!(
            k.resolve(Screen::Pads, &press('q'))
                .map(|a| matches!(a, Action::PadPress { .. })),
            Some(true)
        );
    }

    /// Hold and speak: Ghostty repeats a held Alt+R as presses, they must not stop the recording.
    #[test]
    fn hold_and_speak_ignores_auto_repeat() {
        let k = keymap();
        let mut held = HeldPads::default();
        let alt_r = |kind| ev(KeyCode::Char('к'), KeyModifiers::ALT, kind);
        let mut got = Vec::new();
        for e in std::iter::repeat_n(alt_r(KeyEventKind::Press), 20)
            .chain([alt_r(KeyEventKind::Release), alt_r(KeyEventKind::Press)])
        {
            got.extend(k.resolve(Screen::Pads, &e).and_then(|a| held.filter(a)));
        }
        assert_eq!(
            got,
            [
                Action::RecordMic(true),
                Action::RecordMic(false),
                Action::RecordMic(true)
            ]
        );
    }

    #[test]
    fn held_pad_query_keeps_filter_state() {
        let mut held = HeldPads::default();
        assert!(!held.is_held(4));
        assert!(!held.is_held(16));
        let press = Action::PadPress {
            pad: 4,
            velocity: 1.0,
        };
        assert_eq!(held.filter(press), Some(press));
        assert!(held.is_held(4));
        assert!(held.is_held(4), "a query does not consume the hold");
        assert!(!held.is_held(5));
        assert_eq!(held.filter(press), None);
        held.filter(Action::PadRelease { pad: 4 });
        assert!(!held.is_held(4));
        assert_eq!(held.filter(press), Some(press));
    }

    /// Recorded from Ghostty, Russian layout, Shift held: key auto-repeat arrives as presses.
    #[test]
    fn ghostty_auto_repeat_does_not_retrigger() {
        let k = keymap();
        let mut held = HeldPads::default();
        let shift_q = |kind| ev(KeyCode::Char('й'), KeyModifiers::SHIFT, kind);
        let mut got = Vec::new();
        for e in std::iter::repeat_n(shift_q(KeyEventKind::Press), 31)
            .chain([shift_q(KeyEventKind::Release), shift_q(KeyEventKind::Press)])
        {
            got.extend(k.resolve(Screen::Pads, &e).and_then(|a| held.filter(a)));
        }
        assert_eq!(
            got,
            [
                Action::PadPress {
                    pad: 4,
                    velocity: 1.0
                },
                Action::PadRelease { pad: 4 },
                Action::PadPress {
                    pad: 4,
                    velocity: 1.0
                },
            ]
        );
    }

    #[test]
    fn safe_cue_actions_are_parsed_and_listed_in_their_screen_groups() {
        for (name, action, group) in [
            ("pad.edit_only", Action::PadEditOnly, Group::Pads),
            ("chop.edit_only", Action::ChopEditOnly, Group::Chop),
            ("browser.take_only", Action::BrowserTakeOnly, Group::Files),
        ] {
            assert_eq!(Action::parse(name), Some(action));
            assert!(
                ACTIONS
                    .iter()
                    .any(|row| { row.name == name && row.group == group && !row.knob })
            );
        }
        for (name, action) in [
            ("pad.edit", Action::PadEdit),
            ("chop.edit", Action::ChopEdit),
            ("browser.hold", Action::BrowserHold),
        ] {
            assert_eq!(Action::parse(name), Some(action));
        }
    }

    #[test]
    fn every_listed_action_is_known() {
        for a in ACTIONS {
            assert_eq!(Action::parse(a.name).is_some(), !a.knob, "{}", a.name);
        }
        assert!(ACTIONS.iter().all(|a| a.name != "app.quit"));
    }

    #[test]
    fn help_shows_keys() {
        let k = keymap();
        // 83a: U/O turn the banks on pads; the arrows change values there
        assert_eq!(k.keys_for(Screen::Pads, Action::BankNext), ["O", "PgDn"]);
        assert_eq!(k.keys_for(Screen::Pads, Action::BankPrev), ["PgUp", "U"]);
        let down = Action::EditValue {
            up: false,
            fine: false,
        };
        // 83a: the value keys sit on the screen layer too, with the edit closed
        assert_eq!(k.keys_for(Screen::Pads, down), ["J", "←"]);
        assert_eq!(k.edit_keys_for(down), ["J", "←"]);
    }

    #[test]
    fn hold_the_arrow() {
        let k = keymap();
        let held = |code| ev(code, KeyModifiers::NONE, KeyEventKind::Repeat);
        assert_eq!(
            k.resolve(Screen::Chop, &held(KeyCode::Right)),
            Some(Action::CursorMove {
                right: true,
                fine: false
            })
        );
        assert_eq!(
            k.resolve(
                Screen::Chop,
                &ev(KeyCode::Char('l'), KeyModifiers::ALT, KeyEventKind::Repeat)
            ),
            Some(Action::SliceBound {
                end: true,
                right: true,
                fine: false
            }),
            "the bound repeats the coarse step"
        );
        assert_eq!(
            k.resolve(Screen::Chop, &held(KeyCode::Char('i'))),
            Some(Action::ChopZoom { zoom_in: true })
        );
        // the rest acts once per press
        assert_eq!(k.resolve(Screen::Chop, &held(KeyCode::Backspace)), None);
        assert_eq!(k.resolve(Screen::Chop, &held(KeyCode::Enter)), None);
    }

    /// Scenario "Holding the arrow scrolls the list" (30).
    #[test]
    fn holding_the_arrow_scrolls_the_list() {
        let k = keymap();
        let held = |code| ev(code, KeyModifiers::NONE, KeyEventKind::Repeat);
        assert_eq!(
            k.resolve(Screen::Files, &held(KeyCode::Down)),
            Some(Action::BrowserDown)
        );
        assert_eq!(
            k.resolve(Screen::Files, &held(KeyCode::Up)),
            Some(Action::BrowserUp)
        );
        assert_eq!(
            k.resolve(Screen::Files, &held(KeyCode::Char('k'))),
            Some(Action::BrowserDown)
        );
        // opening a folder still acts once per press
        assert_eq!(k.resolve(Screen::Files, &held(KeyCode::Right)), None);
    }

    #[test]
    fn a_held_pad_does_not_roll() {
        let k = keymap();
        let held = ev(KeyCode::Char('q'), KeyModifiers::NONE, KeyEventKind::Repeat);
        assert_eq!(k.resolve(Screen::Chop, &held), None);
        assert_eq!(k.resolve(Screen::Pads, &held), None);
    }

    /// 57: the learn mode is on F10, next to the settings on F12.
    #[test]
    fn the_learn_mode_on_f10() {
        let k = keymap();
        let key = |code| ev(code, KeyModifiers::NONE, KeyEventKind::Press);
        for screen in [Screen::Pads, Screen::Chop, Screen::Files] {
            assert_eq!(
                k.resolve(screen, &key(KeyCode::F(10))),
                Some(Action::LearnToggle)
            );
            assert_eq!(
                k.resolve(screen, &key(KeyCode::F(4))),
                Some(Action::ScreenFx)
            );
        }
    }

    /// 54: Shift+Tab where only Tab is bound acts as Tab, as before the page keys; the
    /// pads screen binds both since 70.
    #[test]
    fn shift_tab_falls_back_to_tab() {
        let k = keymap();
        let key = |code| ev(code, KeyModifiers::NONE, KeyEventKind::Press);
        assert_eq!(
            k.resolve(Screen::Chop, &key(KeyCode::BackTab)),
            k.resolve(Screen::Chop, &key(KeyCode::Tab))
        );
        assert_eq!(
            k.resolve(Screen::Settings, &key(KeyCode::BackTab)),
            k.resolve(Screen::Settings, &key(KeyCode::Tab))
        );
    }

    /// Scenario "An unbound Shift chord acts as its plain key" (keyboard, 83a): with no
    /// Shift command bound, the shifted press of a command key runs the plain key's
    /// action in both input forms and both layouts.
    #[test]
    fn an_unbound_shift_chord_acts_as_its_plain_key() {
        let k = keymap();
        let shift = |c| ev(KeyCode::Char(c), KeyModifiers::SHIFT, KeyEventKind::Press);
        // the extended protocol names the base key and the modifier
        assert_eq!(
            k.resolve(Screen::Pads, &shift('h')),
            Some(Action::PadModeNext)
        );
        // without it, Shift arrives as an uppercase character; Р is the Russian H key
        assert_eq!(
            k.resolve(Screen::Pads, &press('H')),
            Some(Action::PadModeNext)
        );
        assert_eq!(
            k.resolve(Screen::Pads, &press('Р')),
            Some(Action::PadModeNext)
        );
        assert_eq!(
            k.resolve(Screen::Pads, &shift('р')),
            Some(Action::PadModeNext)
        );
        // a Shift chord of the pad block stays a full-velocity pad, not a command
        assert_eq!(
            k.resolve(Screen::Pads, &press('Й')),
            Some(Action::PadPress {
                pad: 4,
                velocity: 1.0
            })
        );
        assert_eq!(
            k.resolve(Screen::Pads, &shift('й')),
            Some(Action::PadPress {
                pad: 4,
                velocity: 1.0
            })
        );
    }

    /// Scenario "An explicit none on a Shift chord blocks the fallback" (keyboard, 83a):
    /// the owner's `none` keeps the shifted press silent, of the Tab fallback too, and
    /// the plain key keeps its action without Shift.
    #[test]
    fn an_explicit_none_on_a_shift_chord_blocks_the_fallback() {
        let (mut k, warnings) = Keymap::from_toml(KEYMAP).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        k.overlay("[screens.pads]\n\"Shift+H\" = \"none\"\n[screens.settings]\n\"Shift+Tab\" = \"none\"\n")
            .unwrap();
        let shift = |c| ev(KeyCode::Char(c), KeyModifiers::SHIFT, KeyEventKind::Press);
        for e in [shift('h'), press('H'), press('Р'), shift('р')] {
            assert_eq!(k.resolve(Screen::Pads, &e), None, "{e:?}");
        }
        // the plain key keeps its action without Shift
        assert_eq!(
            k.resolve(Screen::Pads, &press('h')),
            Some(Action::PadModeNext)
        );
        // the `none` blocks the Tab fallback of Shift+Tab as well
        let back_tab = ev(KeyCode::BackTab, KeyModifiers::SHIFT, KeyEventKind::Press);
        assert_eq!(k.resolve(Screen::Settings, &back_tab), None);
        assert_eq!(
            k.resolve(
                Screen::Settings,
                &ev(KeyCode::Tab, KeyModifiers::NONE, KeyEventKind::Press)
            ),
            Some(Action::SettingsTab)
        );
        // a plain `none` silences the key for the shifted press too, and a later
        // overlay may bind the chord again
        k.overlay("[screens.pads]\nH = \"none\"\n").unwrap();
        assert_eq!(k.resolve(Screen::Pads, &press('h')), None);
        assert_eq!(k.resolve(Screen::Pads, &shift('h')), None);
        k.overlay("[screens.pads]\n\"Shift+H\" = \"pad.clear\"\n")
            .unwrap();
        assert_eq!(k.resolve(Screen::Pads, &shift('h')), Some(Action::PadClear));
        assert_eq!(k.resolve(Screen::Pads, &press('H')), Some(Action::PadClear));
    }

    /// Scenario "Shift H is a distinct command in both input forms" (keyboard, 83a), on
    /// the resolution level: an existing action stands in for the future `chop.hits_back`.
    #[test]
    fn a_bound_shift_chord_is_distinct_from_its_plain_key() {
        let (mut k, warnings) = Keymap::from_toml(KEYMAP).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        k.overlay("[screens.chop]\n\"Shift+H\" = \"chop.normalize\"\n")
            .unwrap();
        let shift = |c| ev(KeyCode::Char(c), KeyModifiers::SHIFT, KeyEventKind::Press);
        assert_eq!(
            k.resolve(Screen::Chop, &shift('h')),
            Some(Action::ChopNormalize)
        );
        // the legacy uppercase input, in both layouts
        assert_eq!(
            k.resolve(Screen::Chop, &press('H')),
            Some(Action::ChopNormalize)
        );
        assert_eq!(
            k.resolve(Screen::Chop, &press('Р')),
            Some(Action::ChopNormalize)
        );
        assert_eq!(
            k.resolve(Screen::Chop, &shift('р')),
            Some(Action::ChopNormalize)
        );
        // the plain key keeps its own command: H chops by hits on the chop screen (83a)
        assert_eq!(k.resolve(Screen::Chop, &press('h')), Some(Action::ChopHits));
        // another screen keeps its layers: on Pads the shift chord falls back to plain H
        assert_eq!(
            k.resolve(Screen::Pads, &shift('h')),
            Some(Action::PadModeNext)
        );
        // the help names the chord
        assert!(
            k.keys_for(Screen::Chop, Action::ChopNormalize)
                .contains(&"Shift+H".into())
        );
        // a held shifted key acts once, and its release runs nothing
        assert_eq!(
            k.resolve(
                Screen::Chop,
                &ev(
                    KeyCode::Char('h'),
                    KeyModifiers::SHIFT,
                    KeyEventKind::Repeat
                )
            ),
            None
        );
        assert_eq!(
            k.resolve(
                Screen::Chop,
                &ev(
                    KeyCode::Char('h'),
                    KeyModifiers::SHIFT,
                    KeyEventKind::Release
                )
            ),
            None
        );
        // Shift+Tab, the Ctrl/Alt chords and the pads keep their behavior; 83a: the chop
        // Tab fallback lands on the named assign order action
        assert_eq!(
            k.resolve(
                Screen::Chop,
                &ev(KeyCode::Tab, KeyModifiers::SHIFT, KeyEventKind::Press)
            ),
            Some(Action::ChopAssignOrder),
            "Shift+Tab falls back to Tab on the chop screen"
        );
        assert_eq!(
            k.resolve(
                Screen::Chop,
                &ev(
                    KeyCode::Char('j'),
                    KeyModifiers::CONTROL,
                    KeyEventKind::Press
                )
            ),
            Some(Action::SliceBound {
                end: false,
                right: false,
                fine: true
            })
        );
    }

    /// Shift chords parse for commands, keep `Shift+Tab` its own key and refuse the pad
    /// block, alone or with Shift, as commands.
    #[test]
    fn shift_chord_parsing() {
        let (k, warnings) = Keymap::from_toml(
            "[global]\n\"Shift+H\" = \"help.toggle\"\n\"Alt+Shift+H\" = \"help.toggle\"\n\"Ctrl+Shift+Tab\" = \"help.toggle\"\n",
        )
        .unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(
            k.resolve(
                Screen::Pads,
                &ev(KeyCode::Char('h'), KeyModifiers::SHIFT, KeyEventKind::Press)
            ),
            Some(Action::HelpToggle)
        );
        assert_eq!(
            k.resolve(
                Screen::Pads,
                &ev(
                    KeyCode::Char('h'),
                    KeyModifiers::ALT | KeyModifiers::SHIFT,
                    KeyEventKind::Press
                )
            ),
            Some(Action::HelpToggle)
        );
        assert_eq!(
            k.resolve(
                Screen::Pads,
                &ev(
                    KeyCode::Tab,
                    KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                    KeyEventKind::Press
                )
            ),
            Some(Action::HelpToggle),
            "Ctrl+Shift+Tab is the Ctrl chord of the BackTab key"
        );
        assert_eq!(
            k.bindings(Screen::Pads),
            [
                ("Alt+Shift+H".into(), Action::HelpToggle),
                ("Ctrl+Shift+Tab".into(), Action::HelpToggle),
                ("Shift+H".into(), Action::HelpToggle),
            ]
        );
        // the pad block stays out of the commands, with Shift too
        let (k, warnings) = Keymap::from_toml(
            "[global]\n\"Shift+Q\" = \"help.toggle\"\n\"Shift+!\" = \"help.toggle\"\nQ = \"help.toggle\"\n",
        )
        .unwrap();
        assert_eq!(warnings.len(), 3, "{warnings:?}");
        assert!(
            warnings
                .iter()
                .all(|w| w.ends_with("pad keys cannot be commands"))
        );
        assert_eq!(
            k.resolve(Screen::Pads, &press('q')),
            Some(Action::PadPress {
                pad: 4,
                velocity: 80.0 / 127.0
            })
        );
    }

    /// 21: "The same keys on a pad and on a slice" and "Arrows change the picked value".
    #[test]
    fn the_same_keys_on_a_pad_and_on_a_slice() {
        let k = keymap();
        let key = |code, mods| ev(code, mods, KeyEventKind::Press);
        let held = |code| ev(code, KeyModifiers::NONE, KeyEventKind::Repeat);
        let value = |up, fine| Some(Action::EditValue { up, fine });
        for screen in [Screen::Pads, Screen::Chop] {
            let edit = |code| k.resolve_in(screen, true, &key(code, KeyModifiers::NONE));
            // the letters and their arrow twins
            for (letter, arrow, want) in [
                ('i', KeyCode::Up, Some(Action::ParamPrev)),
                ('k', KeyCode::Down, Some(Action::ParamNext)),
                ('j', KeyCode::Left, value(false, true)),
                ('l', KeyCode::Right, value(true, true)),
            ] {
                assert_eq!(edit(KeyCode::Char(letter)), want, "{letter} on {screen:?}");
                assert_eq!(edit(arrow), want, "{arrow:?} on {screen:?}");
            }
            // 54: Tab and Shift+Tab turn a whole page
            assert_eq!(edit(KeyCode::Tab), Some(Action::PageNext));
            assert_eq!(edit(KeyCode::BackTab), Some(Action::PagePrev));
            assert_eq!(
                k.resolve_in(screen, true, &key(KeyCode::Tab, KeyModifiers::SHIFT)),
                Some(Action::PagePrev),
                "kitty sends Tab with Shift"
            );
            // 21 "A tap is fine, holding is coarse"
            assert_eq!(
                k.resolve_in(screen, true, &held(KeyCode::Char('l'))),
                value(true, false)
            );
            // 21 "Enter closes too"
            assert_eq!(edit(KeyCode::Enter), Some(Action::Back));
            // the edit layer leaves the pads and the way out alone
            assert!(matches!(
                k.resolve_in(screen, true, &press('q')),
                Some(Action::PadPress { .. })
            ));
            assert_eq!(edit(KeyCode::Esc), Some(Action::Back));
        }
    }

    /// Scenario "Minus and equal do not zoom" (chopping).
    #[test]
    fn zoom_is_not_on_minus_and_equal() {
        let k = keymap();
        let press = |c| ev(KeyCode::Char(c), KeyModifiers::NONE, KeyEventKind::Press);
        // − and = do not zoom anywhere (21)
        assert_eq!(k.resolve(Screen::Chop, &press('-')), None);
        assert_eq!(k.resolve(Screen::Chop, &press('=')), None);
        // 21 "Zoom on I and K": the zoom is on I K and their arrow twins
        let alt = |code| ev(code, KeyModifiers::ALT, KeyEventKind::Press);
        assert_eq!(
            k.resolve(Screen::Chop, &press('i')),
            Some(Action::ChopZoom { zoom_in: true })
        );
        assert_eq!(
            k.resolve(Screen::Chop, &press('л')),
            Some(Action::ChopZoom { zoom_in: false }),
            "K in the Russian layout"
        );
        assert_eq!(
            k.resolve(
                Screen::Chop,
                &ev(KeyCode::Down, KeyModifiers::NONE, KeyEventKind::Press)
            ),
            Some(Action::ChopZoom { zoom_in: false })
        );
        // Alt does not eat the pads: Alt+q is free for a command, plain q still plays
        assert_eq!(k.resolve(Screen::Pads, &alt(KeyCode::Char('q'))), None);
        assert!(matches!(
            k.resolve(Screen::Pads, &press('q')),
            Some(Action::PadPress { .. })
        ));
    }

    #[test]
    fn chop_keys_and_fine_nudge_with_ctrl() {
        let k = keymap();
        let ctrl = |c| ev(KeyCode::Char(c), KeyModifiers::CONTROL, KeyEventKind::Press);
        assert_eq!(k.resolve(Screen::Chop, &press('m')), Some(Action::ChopCut));
        // the Russian layout: the same physical key is ь
        assert_eq!(k.resolve(Screen::Chop, &press('ь')), Some(Action::ChopCut));
        assert_eq!(k.resolve(Screen::Chop, &press(' ')), Some(Action::ChopPlay));
        // − = are free on the chop screen (the zoom test below); the value keys are the arrows in the edit
        assert_eq!(k.resolve(Screen::Chop, &press('-')), None);
        assert_eq!(k.resolve(Screen::Chop, &press('=')), None);
        // P starts the assign; Enter opens the slice edit (the commit is the device key)
        assert_eq!(
            k.resolve(Screen::Chop, &press('p')),
            Some(Action::ChopAssign)
        );
        assert_eq!(
            k.resolve(
                Screen::Chop,
                &ev(KeyCode::Enter, KeyModifiers::NONE, KeyEventKind::Press)
            ),
            Some(Action::ChopEdit)
        );
        // 21 "Ctrl and Alt name the bound": the left thumb holds the modifier, a tap is
        // the fine step
        let chord = |c, m| ev(KeyCode::Char(c), m, KeyEventKind::Press);
        let bound = |end, right| {
            Some(Action::SliceBound {
                end,
                right,
                fine: true,
            })
        };
        assert_eq!(
            k.resolve(Screen::Chop, &chord('j', KeyModifiers::CONTROL)),
            bound(false, false)
        );
        assert_eq!(
            k.resolve(Screen::Chop, &chord('l', KeyModifiers::ALT)),
            bound(true, true)
        );
        assert_eq!(
            k.resolve(
                Screen::Chop,
                &ev(KeyCode::Right, KeyModifiers::CONTROL, KeyEventKind::Press)
            ),
            bound(false, true),
            "the arrow twin"
        );
        // the bottom row acts at the cursor: M cut, , region start, . region end
        assert_eq!(
            k.resolve(Screen::Chop, &press(',')),
            Some(Action::ChopRegionStart)
        );
        assert_eq!(
            k.resolve(Screen::Chop, &press('ю')),
            Some(Action::ChopRegionEnd)
        );
        assert_eq!(
            k.resolve(Screen::Chop, &ctrl(',')),
            Some(Action::ChopRegionClear)
        );
        // J L move the cursor, Y the equal parts (н in Russian), Tab in the assign
        assert_eq!(
            k.resolve(Screen::Chop, &press('j')),
            Some(Action::CursorMove {
                right: false,
                fine: true
            })
        );
        assert_eq!(
            k.resolve(Screen::Chop, &press('н')),
            Some(Action::ChopEqualParts)
        );
        assert_eq!(
            k.resolve(
                Screen::Chop,
                &ev(KeyCode::Tab, KeyModifiers::NONE, KeyEventKind::Press)
            ),
            // 83a: Tab is the named assign order action on the chop screen
            Some(Action::ChopAssignOrder)
        );
        assert_eq!(k.resolve(Screen::Chop, &ctrl('z')), Some(Action::Undo));
        assert_eq!(
            k.resolve(
                Screen::Pads,
                &ev(KeyCode::F(3), KeyModifiers::NONE, KeyEventKind::Press)
            ),
            Some(Action::ScreenChop)
        );
        // pads still play on the chop screen
        assert!(matches!(
            k.resolve(Screen::Chop, &press('w')),
            Some(Action::PadPress { pad: 5, .. })
        ));
    }

    /// 83a chopping "H cuts at the marks by default": H chops by hits on the chop
    /// screen; the old Ctrl+Y stays reserved for redo in 93a and runs nothing by default.
    #[test]
    fn h_chops_by_hits_and_ctrl_y_stays_free() {
        let k = keymap();
        assert_eq!(k.resolve(Screen::Chop, &press('h')), Some(Action::ChopHits));
        assert_eq!(k.resolve(Screen::Chop, &press('р')), Some(Action::ChopHits));
        let ctrl = |c, screen| {
            k.resolve(
                screen,
                &ev(KeyCode::Char(c), KeyModifiers::CONTROL, KeyEventKind::Press),
            )
        };
        assert_eq!(ctrl('y', Screen::Chop), None);
        assert_eq!(ctrl('y', Screen::Pads), None);
    }

    /// Recorded from Ghostty: holding Ctrl+L sends presses; the release ends the hold.
    #[test]
    fn a_held_key_speeds_up_in_ghostty() {
        let k = keymap();
        let mut held = HeldKeys::default();
        let ctrl_l = |kind| ev(KeyCode::Char('l'), KeyModifiers::CONTROL, kind);
        let mut got = Vec::new();
        for e in std::iter::repeat_n(ctrl_l(KeyEventKind::Press), 3)
            .chain([ctrl_l(KeyEventKind::Release), ctrl_l(KeyEventKind::Press)])
        {
            got.extend(k.resolve(Screen::Chop, &held.repeat(&e)));
        }
        let start_right = |fine| Action::SliceBound {
            end: false,
            right: true,
            fine,
        };
        assert_eq!(
            got,
            [
                start_right(true),
                start_right(false),
                start_right(false),
                start_right(true),
            ],
            "a tap, two repeats, a new tap after the release"
        );
    }
}

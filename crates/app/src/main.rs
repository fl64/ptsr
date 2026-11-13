mod audio;
mod browser;
mod chop;
mod config;
mod controller_map;
mod dispatch;
mod graphics;
mod help;
mod input_sources;
mod learn;
mod loader;
mod meters;
mod midi;
mod notices;
mod pads;
mod pattern_record;
mod pattern_view;
mod saving;
mod session;
mod transport;
mod values;
mod view;
mod wav;

#[cfg(test)]
mod input_tests;
#[cfg(test)]
mod saving_tests;
#[cfg(test)]
mod screens;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

use engine::rt::{ProjectCommand, ProjectMessage};
use engine::{Command, PADS, PADS_PER_BANK, Region, Sample, test_tone_name};
use input::midi::{Move, Param, Profile, Takeover};
use input::{Action, HeldPads, Keymap, Screen};
use ratatui::crossterm::event::{
    self, Event, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use ratatui::crossterm::{
    execute,
    terminal::{self, SetTitle},
};

use audio::{Audio, Routed};
use config::AudioConfig;
use notices::{Level, Notice, NoticeToken, Notices, Origin};

/// Metronome click velocity.
const CLICK_VELOCITY: f32 = 100.0 / 127.0;
/// The keymap shipped with the program; the owner's `keymap.toml` lies over it (64).
const DEFAULT_KEYMAP: &str = include_str!("../../../config/keymap.toml");
/// How long a pad stays lit after a hit.
const HIT_FLASH: Duration = Duration::from_millis(120);
/// How long a changed parameter stays lit after a knob turn or a value key.
const LIT_FOR: Duration = Duration::from_secs(1);

/// Repeats of the held cursor or bound key so far (0 for a tap): the chop turns them into
/// a step that grows with holding (`Chop::view_step`).
fn held_repeats(repeats: &mut u32, tap: bool) -> u32 {
    *repeats = if tap { 0 } else { *repeats + 1 };
    *repeats
}

/// What lights up after a change: a pad parameter, or a bound of the selected slice
/// (`true` is the end).
/// Pads with their link and slice number before a step unlinked them.
type PadLinks = Vec<(usize, Option<(u32, u32)>, Option<u16>)>;

/// A pad before its source changed: (pad, region, link, slice number).
/// One step of the pads' undo (61): what it was, the pads it touched as they were before,
/// the shared slots as they were if it touched them, and when, so a knob turn joins one step.
struct Step {
    what: String,
    pads: Vec<(usize, pads::PadSnap)>,
    fx: Option<[pads::FxSlot; engine::FX_SLOTS]>,
    swap: Option<(usize, usize)>,
    pattern: Option<pattern_record::Undo>,
    at: Instant,
}

/// How many steps the undo keeps (61).
const UNDO_STEPS: usize = 100;
/// Changes of one thing closer than this join one step: a knob turn sends many (61).
const UNDO_JOIN: Duration = Duration::from_secs(1);

/// Every pad and the shared slots, before an action, to see what it changed (61).
struct Before {
    pads: Vec<pads::PadSnap>,
    fx: [pads::FxSlot; engine::FX_SLOTS],
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Lit {
    Pad(pads::Param),
    Bound(bool),
}
const FRAME: Duration = Duration::from_millis(33);
/// How long the audio thread load peak stays on screen.
const LOAD_WINDOW: Duration = Duration::from_millis(500);
const BUFFER_SIZES: [u32; 5] = [64, 128, 256, 512, 1024];
const SETTINGS_ROWS: usize = 4;
const DEVICES: &str = "config/devices";
/// Lines kept on the input debug tab.
const INPUT_LOG: usize = 12;
/// How long the soft takeover hint stays after the knob stops.
const PICKUP_HINT: Duration = Duration::from_secs(2);
/// Loader slot of the browser preview, after the 128 pads.
const PREVIEW: usize = PADS;
/// Loader slot of a file opened on the chop screen from the files screen.
const CHOP: usize = PADS + 1;
/// Marker steps on the chop screen: coarse, and with Ctrl held.
const NUDGE_MS: i32 = 20;
const NUDGE_FINE_MS: i32 = 4;
/// Session changes are written this long after the last one.
const SESSION_SAVE: Duration = Duration::from_secs(1);

/// A step that loses what lies on pads, waiting for the owner's answer. `lines` is what the
/// question showed: if the pads change before Enter, the question is shown again.
enum Ask {
    Plan {
        lines: Vec<String>,
    },
    PutFile {
        abs: usize,
        path: PathBuf,
        lines: Vec<String>,
    },
    /// The MIDI tab: a device's bindings back to its profile (59).
    ResetProfile {
        name: String,
        port: String,
        lines: Vec<String>,
    },
    /// `Backspace` in the pad edit: these pads back to clean (76).
    ResetPads {
        pads: Vec<usize>,
        lines: Vec<String>,
    },
    /// `Backspace` with marks outside the edit: the sounds off these pads (76).
    ClearPads {
        pads: Vec<usize>,
        lines: Vec<String>,
    },
    /// A copy in hand replaces a pad that holds something (83a).
    CopyPad {
        source: usize,
        lines: Vec<String>,
    },
    /// Clear A1 from pattern controls: every hit goes, the settings stay (98b).
    ClearPattern {
        lines: Vec<String>,
    },
}

/// The folder picker of the settings Paths tab (11): which row opened it, the folder it
/// shows, the folders in it and where the cursor is.
struct PathPicker {
    /// 0 = the projects root row, 1 = the samples root row.
    which: usize,
    at: PathBuf,
    entries: Vec<PathBuf>,
    selected: usize,
}

impl PathPicker {
    /// Opens on the row's root if it exists, otherwise at home; lists its folders only:
    /// hidden entries are skipped, folders first by name, as the files screen sorts.
    fn open(which: usize, current: &Path) -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| "/".into());
        let at = if current.is_dir() {
            current.to_path_buf()
        } else {
            home
        };
        let entries = folders_of(&at);
        Self {
            which,
            at,
            entries,
            selected: 0,
        }
    }

    /// Steps the cursor; the ends hold.
    fn move_cursor(&mut self, step: isize) {
        self.selected = (self.selected as isize + step)
            .clamp(0, self.entries.len().saturating_sub(1) as isize)
            as usize;
    }
}

/// The learn mode on the screen (47, 52): the element picked by its key, a control moved
/// for it waiting for `Enter`, a clear waiting for `Enter`, the learner of the control.
struct ScreenLearn {
    picked: Option<(&'static str, String)>,
    pending: Option<(String, input::midi::Key)>,
    clearing: bool,
    learner: Option<learn::Learning>,
    /// The control just bound: not heard until another control moves (53).
    ignore: Option<input::midi::Key>,
    /// The wrong kind of control was moved (53, 57): a knob for a command, a button for a
    /// parameter; the top line says why.
    refused: bool,
    /// The layer the bindings go to (57): the edit group with the edit open.
    group: input::Group,
    /// The page of the edit the element was picked on, for the top line.
    page: Option<&'static str>,
    /// Opened by "do, then F10" (57): the mode closes after the binding.
    once: bool,
}

/// The last action done with its element, for "do, then F10" (57).
struct LastDone {
    at: Instant,
    name: &'static str,
    label: String,
    group: input::Group,
    page: Option<&'static str>,
}

/// How long `F10` still catches the last action (57).
const LAST_DONE: Duration = Duration::from_secs(5);

/// The learn list entry of an action, in whatever group the list keeps it.
fn action_info(name: &str) -> &'static input::ActionInfo {
    input::ACTIONS
        .iter()
        .find(|i| i.name == name)
        .unwrap_or(&input::ACTIONS[0])
}

/// A control as its device profile names it (`b9`), or its key (`note 69`).
fn control_name(profiles: &[Profile], profile: &str, key: input::midi::Key) -> String {
    profiles
        .iter()
        .find(|p| p.name == profile)
        .and_then(|p| p.name_of(key))
        .map_or_else(|| key.to_string(), str::to_string)
}

/// The project picker of Ctrl+P (11): the names of the folders of the projects root that
/// hold a `project.toml`, where the cursor is, and the name line while a new project is
/// being typed.
struct ProjectPicker {
    entries: Vec<String>,
    selected: usize,
    naming: Option<String>,
}

impl ProjectPicker {
    /// Lists the project folders of the root; sorted, as the files screen sorts its folders.
    fn new(root: &Path) -> Self {
        let entries = folders_of(root)
            .into_iter()
            .filter(|f| f.join("project.toml").is_file())
            .map(|f| file_name(&f))
            .collect();
        Self {
            entries,
            selected: 0,
            naming: None,
        }
    }
}

/// The open project (11): its name and folder; `None` — the scratch session.
struct Project {
    /// Shown in the window title while the project is open.
    name: String,
    folder: PathBuf,
}

/// (11) The session as it goes into a project's `project.toml`: every sample path (pads,
/// chops) and the browser folder inside the folder becomes relative to it, outside paths
/// stay absolute.
fn with_stored_paths(mut sess: session::Session, folder: &Path) -> session::Session {
    for pad in sess.pads.values_mut() {
        if let Some(file) = &mut pad.file {
            *file = session::to_stored_path(folder, file);
        }
    }
    sess.browser_dir = sess
        .browser_dir
        .map(|dir| session::to_stored_path(folder, &dir));
    let stored_key = |file: &str| {
        session::to_stored_path(folder, Path::new(file))
            .to_string_lossy()
            .into_owned()
    };
    sess.chops = sess
        .chops
        .into_iter()
        .map(|(file, st)| (stored_key(&file), st))
        .collect();
    sess.kept = sess
        .kept
        .into_iter()
        .map(|(file, states)| (stored_key(&file), states))
        .collect();
    sess
}

/// (11) Back from the stored form when a project's session is restored: relative paths
/// join the project folder, absolute ones stay as they are.
fn with_resolved_paths(mut sess: session::Session, project: Option<&Path>) -> session::Session {
    let Some(folder) = project else {
        return sess;
    };
    for pad in sess.pads.values_mut() {
        if let Some(file) = &mut pad.file {
            *file = session::from_stored_path(folder, file);
        }
    }
    sess.browser_dir = sess
        .browser_dir
        .map(|dir| session::from_stored_path(folder, &dir));
    let resolved_key = |file: &str| {
        session::from_stored_path(folder, Path::new(file))
            .to_string_lossy()
            .into_owned()
    };
    sess.chops = sess
        .chops
        .into_iter()
        .map(|(file, st)| (resolved_key(&file), st))
        .collect();
    sess.kept = sess
        .kept
        .into_iter()
        .map(|(file, states)| (resolved_key(&file), states))
        .collect();
    sess
}

/// (11) The project the start reopens: the config keeps the last opened name, and its
/// folder in the projects root must still hold a `project.toml`. Anything else — the
/// start is scratch.
fn initial_project(last: Option<&str>, paths: &config::PathsConfig) -> Option<Project> {
    let name = last?;
    let folder = paths.projects.join(name);
    folder.join("project.toml").is_file().then_some(Project {
        name: name.to_string(),
        folder,
    })
}

/// The folders of `dir`; an unreadable folder gives an empty list.
fn folders_of(dir: &Path) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.is_dir()
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| !n.starts_with('.'))
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    folders.sort_by_key(|p| {
        p.file_name()
            .map_or_else(|| p.as_os_str().to_owned(), |n| n.to_owned())
            .to_string_lossy()
            .to_lowercase()
    });
    folders
}

impl Ask {
    fn lines(&self) -> &[String] {
        match self {
            Ask::Plan { lines }
            | Ask::PutFile { lines, .. }
            | Ask::ResetProfile { lines, .. }
            | Ask::ResetPads { lines, .. }
            | Ask::ClearPads { lines, .. }
            | Ask::CopyPad { lines, .. }
            | Ask::ClearPattern { lines } => lines,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UiPadHold {
    abs: usize,
    deferred: bool,
    pattern: Option<Arc<engine::capture::EraseHold>>,
}

struct App {
    transport: transport::State,
    audio: Option<Audio>,
    pending_audio: std::collections::VecDeque<ProjectCommand>,
    pending_manual_save: Option<saving::PendingSave>,
    quit_requested: bool,
    audio_held: [Option<u64>; PADS],
    midi_ui_held: std::collections::HashMap<u64, Vec<UiPadHold>>,
    restoring_project: bool,
    midi_preview_blocked: bool,
    cfg: AudioConfig,
    /// Where to save choices; `None` means the user file is broken and left alone.
    user_path: Option<PathBuf>,
    routed: Routed,
    screen: Screen,
    /// Shared FX selection is independent of the selected pad and its parameter.
    fx_slot: usize,
    fx_param: usize,
    settings_row: usize,
    help_open: bool,
    help_overlay: help::State,
    value_popup: Option<values::Shown>,
    pattern_panel: Option<usize>,
    /// Whole-pad erasure on Pads outside a pass (98b).
    erase_pads: bool,
    pattern_view: pattern_view::State,
    /// Pad key presses accepted for editing must stay silent through their release.
    silent_pad_keys: [bool; PADS_PER_BANK],
    input_sources: input_sources::State,
    pads: pads::Pads,
    /// Output meters for the pads screen: bars, peak hold, the latching overload light.
    meters: meters::Meters,
    settings_tab: usize,
    /// The open folder picker of the Paths tab (11); while it is open the keys go to it
    /// before the screen, like the filter line of the files screen.
    path_picker: Option<PathPicker>,
    /// The project picker of Ctrl+P (11); while it is open the keys go to it before the
    /// screen, like the filter line of the files screen.
    project_picker: Option<ProjectPicker>,
    pad_naming: Option<(usize, String)>,
    takeover: Takeover,
    /// Soft takeover hint for the selected pad and when it was last updated.
    pickup: Option<(String, Instant)>,
    /// Connected MIDI devices: (port, profile name).
    midi_devices: Vec<(String, String)>,
    input_log: Vec<String>,
    midi: Option<midi::Midi>,
    /// Device profiles, kept here too to show assignments on the MIDI tab.
    profiles: Vec<Profile>,
    learned: config::Learned,
    /// Profiles whose roles are copied into `learned` (59): only `learned` acts for them.
    seeded: Vec<String>,
    midi_row: usize,
    /// The groups shown open on the MIDI tab; the rest fold into their row (28).
    midi_open: Vec<input::Group>,
    /// A run over the pads or the banks row (28).
    midi_run: Option<MidiRun>,
    learning: Option<learn::Learning>,
    browser: browser::Browser,
    loader: Option<(
        std::sync::mpsc::Sender<loader::Request>,
        std::sync::mpsc::Receiver<loader::Reply>,
    )>,
    /// Per pad: the generation of its latest load; older replies are stale.
    load_gen: Vec<u64>,
    /// Per pad: the slice it gets once its load finishes.
    load_region: Vec<Region>,
    /// Pads waiting for another pad's load of the same file: (loading pad, waiting pad,
    /// its generation). Slices of one break are read from disk once.
    followers: Vec<(usize, usize, u64)>,
    /// The files screen plays the selected file.
    preview_on: bool,
    master_db: i8,
    /// The pad taken with Ctrl+X, waiting for the pad to swap with.
    pad_in_hand: Option<usize>,
    /// A copy in hand (83a): the source pad it was taken from; the snapshot lives until
    /// placement, so a later change of the source does not change what is placed.
    copy_in_hand: Option<(usize, pads::PadSnap)>,
    /// "2.61 s ▁▃█▅▂" for the file being previewed.
    /// Duration and peak envelope of the selected file, once the preview loaded it.
    preview: Option<(f32, Vec<f32>)>,
    /// The session file: the manual save and the autosave write here.
    session_path: std::path::PathBuf,
    /// The open project (11): pads and chops save into its `project.toml`.
    project: Option<Project>,
    /// The config's `[project]` section (11): the last opened name, written when a
    /// project opens.
    project_cfg: config::ProjectConfig,
    /// A load refusal protects the source file from all save requests.
    session_read_error: Option<String>,
    /// A write refusal pauses automatic saving until a manual save succeeds.
    session_write_failed: bool,
    /// Something changed since this moment and is not saved yet.
    session_dirty: Option<Instant>,
    /// The chop screen state; kept after leaving the screen, so coming back to the same pad
    /// finds the markers where they were.
    chop: Option<chop::Chop>,
    /// The chops of other files, kept so their pads stay linked; by file.
    chops: std::collections::HashMap<PathBuf, chop::Chop>,
    /// Chops from the session whose sample is not loaded yet.
    pending_chops: std::collections::HashMap<PathBuf, chop::Saved>,
    /// Older chops of a file that pads still play slices of (45); the newest chop of a
    /// file is the open one or the one in `chops`.
    older: Vec<chop::Chop>,
    /// Older chops from the session whose sample is not loaded yet (45).
    pending_older: Vec<(PathBuf, chop::Saved)>,
    /// The region the chop loop plays now, as the engine has it (49).
    loop_sent: Option<Region>,
    /// The learn mode on the screen (47); `None` while it is closed.
    screen_learn: Option<ScreenLearn>,
    last_done: Option<LastDone>,
    /// A fresh chop just started in front of an older one (45): the file, the fresh epoch,
    /// the older epoch; an undo past the fresh chop's first step brings the older one back.
    fresh_undo: Option<(PathBuf, u32, u32)>,
    /// The pad the chop screen was opened from; `None` when it came from the files screen.
    chop_from: Option<usize>,
    /// Undo of the pads screen (27): a pad's region, link and slice number before its
    /// source changed.
    /// The undo of the pads (61), the newest step last.
    history: Vec<Step>,
    /// How the screens draw: the wave picture switch (04b).
    ui: config::UiConfig,
    /// The two roots of the Paths tab (11): the picker writes here and into the config.
    paths: config::PathsConfig,
    /// Pads a region left out (35): their file and their links before, for the undo.
    range_unlinks: Vec<(PathBuf, PadLinks)>,
    /// The chop screen is in the assign mode with this plan.
    plan: Option<chop::Plan>,
    /// Earlier plans with the slice selected then, for undo in the assign mode.
    plan_undo: Vec<(chop::Plan, usize)>,
    /// The edit mode of the selected object is open: the pad on the pads screen, the
    /// slice on the chop screen. `Esc` closes it; a screen switch drops it.
    edit: bool,
    /// The picked bound of the selected slice: the end (`true`) or the start.
    slice_end: bool,
    /// Repeats of the held bound key so far: the step grows with them.
    bound_repeats: u32,
    /// What a knob or a value key changed last, and when: lit on the screen for `LIT_FOR`.
    lit: Option<(Lit, Instant)>,
    /// Tests stand in for the engine's playhead.
    #[cfg(test)]
    fake_playhead: Option<usize>,
    /// Listen mode: which 16 slices the pads play (0 is slices 1-16).
    /// A step that would overwrite pads waits for Enter or Esc.
    ask: Option<Ask>,
    /// The engine sample rate.
    rate: u32,
    input_level: f32,
    notices: Notices,
    notice_origin: Origin,
    shown_error: Option<NoticeToken>,
    frame_notice_level: Option<Level>,
    chop_loading: Option<PathBuf>,
    keymap_warnings: std::collections::VecDeque<String>,
    keymap_warning_shown: bool,
    /// The splash in the wave strip at start (73): `true` until the first sound — a note
    /// that plays or a recording that starts.
    splash: bool,
    /// The running recording: its source and when it started.
    recording: Option<(u8, Instant)>,
    /// Test only: every command the app sent, for asserting on the engine's input.
    #[cfg(test)]
    sent: Vec<Command>,
}

impl App {
    fn audio_alive(&self) -> Option<&Audio> {
        self.audio.as_ref().filter(|a| !a.stopped.load(Relaxed))
    }

    /// The microphone is linked only while it is used (26): a microphone recording runs or
    /// the settings screen shows its meter.
    fn mic_wanted(&self) -> bool {
        self.screen == Screen::Settings || matches!(self.recording, Some((audio::SOURCE_MIC, _)))
    }

    fn reroute(&mut self) {
        let Some(a) = self.audio_alive() else {
            return;
        };
        let routed = a.route(
            self.cfg.output.as_deref(),
            self.cfg.input.as_deref(),
            self.mic_wanted(),
        );
        if routed != self.routed {
            log::info!(
                "route: output {:?}, input {:?}",
                routed.output,
                routed.input
            );
        }
        self.routed = routed;
    }

    fn save(&mut self) {
        let result = match &self.user_path {
            Some(p) => config::save_section(p, "audio", &self.cfg),
            None => Err("settings are not saved: fix the config file first".into()),
        };
        if let Err(e) = result {
            log::error!("{e}");
            self.post_notice(Level::Error, (e, Instant::now()));
        }
    }

    /// (11) Where the material of the program lives and the files screen starts: with a
    /// project open — its `samples/` folder (if it is suddenly gone, the browser survives
    /// the same way it survives `~/Music`), otherwise — the samples root of the settings.
    fn samples_dir(&self) -> PathBuf {
        match &self.project {
            Some(p) => p.folder.join("samples"),
            None => self.paths.samples.clone(),
        }
    }

    /// (11) A session becomes the whole app state: the browser folder, the master, the
    /// pads, the chops. The start and the project open/switch both come here; everything
    /// of the previous state is dropped — nothing leaks between projects.
    fn apply_session(&mut self, sess: session::Session) {
        self.cancel_manual_save();
        self.quit_requested = false;
        self.value_popup = None;
        self.pattern_panel = None;
        self.erase_pads = false;
        self.pattern_view = pattern_view::State::default();
        self.pending_audio.clear();
        self.audio_held.fill(None);
        self.midi_ui_held.clear();
        self.restoring_project = false;
        if let Some(audio) = &mut self.audio {
            audio.pattern_capture.reset();
            audio
                .project
                .epoch
                .fetch_add(1, std::sync::atomic::Ordering::Release);
        }
        let favorites = std::mem::take(&mut self.browser.favorites);
        let was = self.browser.dir.clone();
        let mut browser = browser::Browser::new(
            sess.browser_dir
                .clone()
                .or_else(|| Some(self.samples_dir())),
        );
        // a new project has nothing in its samples/ yet: the files screen stays where the
        // owner browsed (35)
        if sess.browser_dir.is_none() && self.project.is_some() && browser.entries.is_empty() {
            browser = browser::Browser::new(Some(was));
        }
        self.browser = browser;
        self.browser.favorites = favorites;
        self.browser.set_filter(String::new());
        self.browser.in_hand = None;
        self.pad_in_hand = None;
        self.copy_in_hand = None;
        self.stop_preview();
        self.edit = false;
        self.transport = transport::State::default();
        self.transport.bpm = sess.bpm;
        self.transport.metronome = sess.metronome;
        self.takeover.reset_target(transport::TEMPO_TARGET);
        self.reset_pattern_takeover();
        self.send(Command::SetTempo { bpm: sess.bpm });
        self.send(Command::SetMetronome {
            enabled: sess.metronome,
        });
        self.set_master(sess.master_db);
        self.pads = pads::Pads::default();
        self.pad_naming = None;
        self.chop_loading = None;
        self.chop = None;
        self.plan = None;
        self.plan_undo.clear();
        self.history.clear();
        // Replies from the previous session must not apply to this one.
        for generation in &mut self.load_gen {
            *generation += 1;
        }
        self.followers.clear();
        let cmds = self.pads.restore(&sess, self.rate);
        for cmd in cmds {
            self.send(cmd);
        }
        self.pending_chops = sess
            .chops
            .iter()
            .map(|(f, st)| (PathBuf::from(f), chop::Saved::from_state(st, self.rate)))
            .collect();
        // the chops of the previous project's files stay behind too
        self.chops.clear();
        self.older.clear();
        self.fresh_undo = None;
        let rate = self.rate;
        self.pending_older = sess
            .kept
            .iter()
            .flat_map(|(f, all)| {
                all.iter()
                    .map(move |st| (PathBuf::from(f), chop::Saved::from_state(st, rate)))
            })
            .collect();
        let mut first_pad: std::collections::HashMap<PathBuf, usize> = Default::default();
        for abs in 0..PADS {
            let Some(path) = self.pads.file[abs].clone() else {
                // an empty pad must not keep the previous project's sound
                self.send(Command::SetPadSound {
                    pad: abs,
                    sound: None,
                });
                continue;
            };
            let region = self.pads.region[abs];
            if let Some(&leader) = first_pad.get(&path) {
                self.follow(leader, abs, region);
            } else if path.exists() {
                self.load_pad(abs, path.clone(), region);
                if self.pads.loading[abs].is_some() {
                    first_pad.insert(path, abs);
                }
            } else {
                log::warn!("session: {} is gone", path.display());
                self.pads.missing[abs] = true;
                self.send(Command::SetPadSound {
                    pad: abs,
                    sound: None,
                });
            }
        }
        self.restoring_project = true;
        self.finish_project_restore();
        self.session_dirty = None;
        self.session_write_failed = false;
    }

    fn finish_project_restore(&mut self) {
        if !self.restoring_project || self.pads.loading.iter().any(Option::is_some) {
            return;
        }
        self.restoring_project = false;
        if let Some(audio) = &self.audio {
            self.pending_audio.push_back(ProjectCommand {
                epoch: audio
                    .project
                    .epoch
                    .load(std::sync::atomic::Ordering::Acquire),
                ready_at_send: false,
                command: ProjectMessage::Ready,
            });
            self.flush_audio();
        }
    }

    fn paths_set(&mut self, which: usize, at: PathBuf) {
        let what = if which == 0 {
            self.paths.projects = at.clone();
            "Projects root"
        } else {
            self.paths.samples = at.clone();
            "Samples root"
        };
        let result = match &self.user_path {
            Some(p) => config::save_section(p, "paths", &self.paths),
            None => Err("settings are not saved: fix the config file first".into()),
        };
        let shown = tilde(&at.display().to_string());
        self.result_notice(result, format!("{what}: {shown}"));
    }

    /// (11) The config's `[project]` section: the last opened project's name. An error
    /// speaks in the flash; the project itself stays open.
    fn save_project_cfg(&mut self) {
        let result = match &self.user_path {
            Some(p) => config::save_section(p, "project", &self.project_cfg),
            None => Err("settings are not saved: fix the config file first".into()),
        };
        if let Err(e) = result {
            log::error!("{e}");
            self.post_notice(Level::Error, (e, Instant::now()));
        }
    }

    fn send(&mut self, cmd: Command) {
        self.try_send(cmd);
    }

    fn try_send(&mut self, cmd: Command) -> bool {
        if let Command::PatternRecordStart { pass } = &cmd {
            use std::sync::atomic::Ordering::Acquire;
            if self.quit_requested {
                return false;
            }
            if self.pending_manual_save.is_some() {
                self.flush_audio();
                if self.pending_manual_save.is_some() {
                    return false;
                }
            }
            let can_start = self.audio.as_ref().is_some_and(|audio| {
                let epoch = audio.project.epoch.load(Acquire);
                !audio.stopped.load(Relaxed)
                    && self.recording.is_none()
                    && audio.pattern_capture.available()
                    && audio.project.ready.load(Acquire) == epoch
                    && self.pending_audio.is_empty()
                    && audio.stats.recording(epoch).is_some_and(|snapshot| {
                        snapshot.phase == engine::capture::Phase::Idle
                            && *pass != 0
                            && *pass > snapshot.pass
                    })
            });
            if !can_start {
                return false;
            }
            let created = self.pads.pattern.is_none();
            match self.pads.prepare_pattern_record() {
                Ok(pattern) => {
                    if created {
                        self.dirty();
                    }
                    let audio = self.audio.as_mut().unwrap();
                    let epoch = audio.project.epoch.load(Acquire);
                    if let Err(error) = audio.pattern_capture.begin(epoch, *pass, &self.pads) {
                        self.post_notice(Level::Error, (error, Instant::now()));
                        return false;
                    }
                    self.send(pattern);
                }
                Err(error) => {
                    self.post_notice(Level::Error, (error, Instant::now()));
                    return false;
                }
            }
        }
        if let Command::PatternRecordEnd { pass } = &cmd {
            self.pattern_end_requested(*pass);
        } else if matches!(&cmd, Command::TransportStop) {
            self.pattern_stop_requested();
        }
        // the first sound ends the splash (73): a note that can sound, or a preview
        let sounds = match &cmd {
            Command::PadPress { pad, .. } | Command::PatternPadPress { pad, .. } => {
                self.pad_sounds(*pad)
            }
            Command::Preview { sound: Some(_), .. } => true,
            _ => false,
        };
        #[cfg(test)]
        self.sent.push(cmd.clone());
        if let Some(a) = &self.audio {
            use std::sync::atomic::Ordering::Acquire;
            let epoch = a.project.epoch.load(Acquire);
            let ready = a.project.ready.load(Acquire) == epoch;
            if self.screen == Screen::Chop
                && matches!(&cmd, Command::Preview { sound: Some(_), .. })
                && (!ready || !self.pending_audio.is_empty() || self.midi_preview_blocked)
            {
                return false;
            }
            match &cmd {
                Command::PadPress { pad, .. } | Command::PatternPadPress { pad, .. } => {
                    if !ready || !self.pending_audio.is_empty() || *pad >= PADS {
                        return false;
                    }
                }
                Command::PadRelease { pad } => {
                    if *pad >= PADS || self.audio_held[*pad].take() != Some(epoch) || !ready {
                        return false;
                    }
                }
                Command::PatternPadRelease { pad, hold } => {
                    if *pad >= PADS || !ready {
                        return false;
                    }
                    if hold.may_release_sound() && self.audio_held[*pad].take() != Some(epoch) {
                        return false;
                    }
                }
                _ => {}
            }
            let press = match &cmd {
                Command::PadPress { pad, .. } | Command::PatternPadPress { pad, .. } => Some(*pad),
                _ => None,
            };
            let message = ProjectCommand {
                epoch,
                ready_at_send: ready,
                command: ProjectMessage::Command(cmd),
            };
            if let Some(pad) = press {
                if self.audio.as_mut().unwrap().commands.push(message).is_err() {
                    return false;
                }
                self.audio_held[pad] = Some(epoch);
                if sounds {
                    self.splash = false;
                }
            } else {
                self.pending_audio.push_back(message);
                self.flush_audio();
            }
            return true;
        }
        false
    }

    fn flush_audio(&mut self) {
        use std::sync::atomic::Ordering::Acquire;
        let previous_settings = self.pattern_setting_indexes();
        let Some(audio) = &mut self.audio else {
            self.finish_manual_save();
            return;
        };
        let epoch = audio.project.epoch.load(Acquire);
        let ready = audio.project.ready.load(Acquire) == epoch;
        let snapshot = audio.stats.recording(epoch);
        let stopped = audio.stopped.load(Relaxed);
        let update = if self.quit_requested {
            audio
                .pattern_capture
                .poll_for_exit(epoch, snapshot, stopped, &mut self.pads)
        } else {
            audio
                .pattern_capture
                .poll(epoch, snapshot, stopped, &mut self.pads)
        };
        match update {
            Ok(update) => {
                if update.changed {
                    self.dirty();
                }
                for command in update.commands {
                    #[cfg(test)]
                    self.sent.push(command.clone());
                    self.pending_audio.push_back(ProjectCommand {
                        epoch,
                        ready_at_send: ready,
                        command: ProjectMessage::Command(command),
                    });
                }
                if let Some(before) = update.undo {
                    self.push_pattern_step(before);
                }
                // a pass that a manual save or quit ends keeps the save notice
                if let Some((new, replaced)) = update
                    .finished
                    .filter(|_| self.pending_manual_save.is_none() && !self.quit_requested)
                {
                    let text = match (new, replaced) {
                        (0, _) => "A1 pass: no new hits.".to_string(),
                        (new, 0) => format!("A1 pass: {new} new hits."),
                        (new, replaced) => format!("A1 pass: {new} new hits, {replaced} replaced."),
                    };
                    self.post_notice(Level::Done, (text, Instant::now()));
                }
                if let Some(notice) = update.notice {
                    self.post_notice(Level::Warn, (notice, Instant::now()));
                }
            }
            Err(error) => self.post_notice(Level::Error, (error, Instant::now())),
        }
        if self.pattern_setting_indexes() != previous_settings {
            self.reset_pattern_takeover();
        }
        let audio = self.audio.as_mut().unwrap();
        while let Some(cmd) = self.pending_audio.pop_front() {
            let preview = cmd.ready_at_send
                && cmd.epoch
                    == audio
                        .project
                        .epoch
                        .load(std::sync::atomic::Ordering::Acquire)
                && cmd.epoch
                    == audio
                        .project
                        .ready
                        .load(std::sync::atomic::Ordering::Acquire)
                && matches!(
                    &cmd.command,
                    ProjectMessage::Command(Command::Preview { sound: Some(_), .. })
                );
            if let Err(rtrb::PushError::Full(cmd)) = audio.commands.push(cmd) {
                self.pending_audio.push_front(cmd);
                break;
            }
            if preview {
                self.splash = false;
            }
        }
        self.finish_manual_save();
    }

    fn set_master(&mut self, db: i8) {
        self.master_db = db.clamp(pads::MIN_DB, pads::MAX_DB);
        let gain = 10f32.powf(self.master_db as f32 / 20.0);
        self.send(Command::SetMasterGain { gain });
    }

    fn pad_name_key(&mut self, k: &event::KeyEvent) -> bool {
        use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
        if k.kind == KeyEventKind::Release {
            return false;
        }
        let Some((abs, mut name)) = self.pad_naming.take() else {
            return false;
        };
        match k.code {
            KeyCode::Esc => return true,
            KeyCode::Enter => {
                let before = self.before();
                self.pads.custom_name[abs] = (!name.is_empty()).then_some(name);
                self.note_step(before, "name".into());
                self.dirty();
                return true;
            }
            KeyCode::Backspace => {
                name.pop();
            }
            KeyCode::Char(c)
                if !k
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                name.push(c)
            }
            _ => {}
        }
        self.pad_naming = Some((abs, name));
        true
    }

    /// A key while the filter line is open; `false` lets it through (releases, Ctrl+key).
    fn filter_key(&mut self, k: &event::KeyEvent) -> bool {
        use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
        if k.kind == KeyEventKind::Release || k.modifiers.contains(KeyModifiers::CONTROL) {
            return false;
        }
        let mut filter = self.browser.filter.clone();
        match k.code {
            KeyCode::Esc => {
                filter.clear();
                self.browser.editing = false;
            }
            KeyCode::Enter => self.browser.editing = false,
            KeyCode::Backspace => {
                filter.pop();
            }
            KeyCode::Char(c) => filter.push(c),
            // modifier keys and the like
            _ => return true,
        }
        if filter != self.browser.filter {
            self.browser.set_filter(filter);
            self.preview_selection();
        }
        true
    }

    /// A key while the project picker is open; `false` lets it through (releases, Ctrl+key).
    /// A character with the list on screen starts the name of a new project; the name line
    /// then takes every character until Enter (create and open) or Esc (back to the list).
    fn project_picker_key(&mut self, k: &event::KeyEvent) -> bool {
        use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
        if k.kind == KeyEventKind::Release || k.modifiers.contains(KeyModifiers::CONTROL) {
            return false;
        }
        let Some(mut picker) = self.project_picker.take() else {
            return false;
        };
        if let Some(name) = &mut picker.naming {
            match k.code {
                KeyCode::Esc => picker.naming = None,
                KeyCode::Enter => {
                    if self.create_project(name.clone()) {
                        return true;
                    }
                }
                KeyCode::Backspace => {
                    name.pop();
                }
                KeyCode::Char(c) => name.push(c),
                // modifier keys and the like
                _ => {
                    self.project_picker = Some(picker);
                    return true;
                }
            }
            self.project_picker = Some(picker);
            return true;
        }
        match k.code {
            KeyCode::Up => picker.selected = picker.selected.saturating_sub(1),
            KeyCode::Down => {
                picker.selected = (picker.selected + 1).min(picker.entries.len().saturating_sub(1));
            }
            KeyCode::Enter => {
                if let Some(name) = picker.entries.get(picker.selected).cloned()
                    && self.open_project(self.paths.projects.join(name))
                {
                    return true;
                }
            }
            KeyCode::Esc => return true, // closed, nothing happens
            // any character is the start of a new project's name
            KeyCode::Char(c) => picker.naming = Some(c.to_string()),
            _ => {
                self.project_picker = Some(picker);
                return true;
            }
        }
        self.project_picker = Some(picker);
        true
    }

    /// (11) The window title: the open project joins the program's name while its folder
    /// is still on disk; a deleted folder drops the name.
    fn window_title(&self) -> String {
        match self.project.as_ref().filter(|p| p.folder.is_dir()) {
            Some(p) => format!("ptsr — {}", p.name),
            None => "ptsr".into(),
        }
    }

    fn stop_preview(&mut self) {
        // a load still on its way for the old selection becomes stale
        self.load_gen[PREVIEW] += 1;
        self.preview = None;
        if self.loop_sent.take().is_some() {
            self.send(Command::PreviewLoop { region: None });
        }
        self.send(Command::Preview {
            sound: None,
            region: Region::WHOLE,
        });
    }

    /// Starts recording from `source`; the status counts the seconds.
    fn record_start(&mut self, source: u8) {
        if self.refuse_audio_during_pattern() {
            return;
        }
        if self.recording.is_some() {
            return;
        }
        let Some(a) = self.audio.as_ref() else {
            self.post_notice(
                Level::Warn,
                (
                    "No sound: there is nothing to record from.".into(),
                    Instant::now(),
                ),
            );
            return;
        };
        if source == audio::SOURCE_MIC && self.routed.input.is_none() {
            self.post_notice(
                Level::Warn,
                (
                    "No microphone is connected: nothing to record from.".into(),
                    Instant::now(),
                ),
            );
            return;
        }
        if source == audio::SOURCE_OUTPUT {
            let tapped = a.tap_output(&self.routed.playback);
            if !tapped {
                self.post_notice(
                    Level::Warn,
                    (
                        "The output device has no monitor to record.".into(),
                        Instant::now(),
                    ),
                );
                return;
            }
        }
        self.recording = Some((source, Instant::now()));
        // a recording is sound too: the splash ends (73)
        self.splash = false;
        // the microphone joins the graph only now; the link settles within a block or two
        if source == audio::SOURCE_MIC {
            self.reroute();
        }
        if let Some(a) = self.audio.as_ref() {
            a.recorder.start(source);
        }
    }

    /// Stops the recording: a WAV in the samples root, then the file is in hand.
    fn record_stop(&mut self) {
        // no recording running: the ring still holds the previous one, which must not be kept twice
        let Some((source, _)) = self.recording.take() else {
            return;
        };
        let Some(a) = self.audio.as_ref() else {
            return;
        };
        let frames = a.recorder.stop();
        log::info!("recording stopped: {frames} frames");
        if source == audio::SOURCE_OUTPUT {
            a.untap_output();
        }
        let ring = a.recorder.take(frames);
        let capped = a.recorder.capped.load(Relaxed);
        if source == audio::SOURCE_MIC {
            self.reroute();
        }
        let root = self.samples_dir();
        self.keep_recording(&ring, capped, &root);
    }

    /// The ring of a stopped recording becomes a WAV in `root`, and the file goes in hand.
    fn keep_recording(&mut self, ring: &[f32], capped: bool, root: &Path) {
        let Some(sample) = sample_from_interleaved(ring, self.rate) else {
            self.post_notice(
                Level::Info,
                (
                    "The recording was silent: nothing is kept.".into(),
                    Instant::now(),
                ),
            );
            return;
        };
        if let Err(e) = std::fs::create_dir_all(root) {
            self.post_notice(
                Level::Error,
                (
                    format!("Cannot create {}: {e}", root.display()),
                    Instant::now(),
                ),
            );
            return;
        }
        let path = free_name(root, &stamp(std::time::SystemTime::now()));
        if let Err(e) = wav::write(&path, &sample, self.rate) {
            self.post_notice(Level::Error, (e, Instant::now()));
            return;
        }
        let secs = sample.left.len() as f32 / self.rate as f32;
        let cap = if capped {
            ", stopped at the 30 s cap"
        } else {
            ""
        };
        self.post_notice(
            if capped { Level::Warn } else { Level::Done },
            (
                format!("Recorded {secs:.1} s{cap} · hit a pad"),
                Instant::now(),
            ),
        );
        self.browser.in_hand = Some(path);
    }

    /// Plays the selected file if it is an audio file and the preview is on.
    fn preview_selection(&mut self) {
        let file = self
            .browser
            .current()
            .filter(|e| !e.is_dir)
            .map(|e| self.browser.dir.join(&e.name));
        match file {
            Some(path) if self.preview_on && self.screen == Screen::Files => {
                self.load_gen[PREVIEW] += 1;
                self.preview = None;
                if let Some((tx, _)) = &self.loader {
                    let _ = tx.send(loader::Request {
                        pad: PREVIEW,
                        path,
                        generation: self.load_gen[PREVIEW],
                    });
                }
            }
            _ => self.stop_preview(),
        }
    }

    fn fx_selected_param(&self) -> pads::Param {
        EFFECT_PARAMS[3 + self.fx_slot * 3 + self.fx_param]
    }

    fn select_fx(&mut self, slot: usize) {
        if self.fx_slot != slot {
            self.fx_slot = slot;
            self.fx_param = 0;
            self.pickup = None;
        }
    }

    fn param_step_name(&self, p: pads::Param) -> String {
        match p.slot() {
            Some(k) => format!("FX {} {}", k + 1, self.pads.name(p)),
            None => self.pads.name(p).into(),
        }
    }

    /// Switches the screen; the edit mode does not survive the switch.
    fn goto(&mut self, screen: Screen) {
        let from = self.screen;
        self.screen = screen;
        self.pattern_view.overview = false;
        self.pattern_view.edit = false;
        self.pattern_view.selected_event = None;
        self.pattern_panel = None;
        self.erase_pads &= screen == Screen::Pads;
        self.edit = false;
        self.midi_run = None;
        // a file saved in the folder while the program runs shows on the next visit (66)
        if screen == Screen::Files && from != Screen::Files {
            self.browser.reread();
        }
        if screen == Screen::Settings {
            // the MIDI tab opens folded, with the group of the screen it came from (28)
            self.midi_open = vec![if from == Screen::Settings {
                input::Group::Global
            } else {
                from.group()
            }];
            self.midi_row = self.midi_row.min(self.midi_list().len().saturating_sub(1));
        }
    }

    /// The profiles of the connected devices with the learned assignments over them.
    fn live_profiles(&self) -> Vec<Profile> {
        let maps = midi::parse_learned(&self.learned);
        let mut profiles: Vec<Profile> = Vec::new();
        for (port, name) in &self.midi_devices {
            // one device often has several ports with the same profile
            if name != "default" && profiles.iter().any(|p| p.name == *name) {
                continue;
            }
            let mut p = self
                .profiles
                .iter()
                .find(|p| p.name == *name)
                .cloned()
                .unwrap_or_else(|| Profile::default_for(port));
            midi::apply_learned(&mut p, &maps);
            p.set_seeded(self.seeded.contains(&p.name));
            profiles.push(p);
        }
        profiles
    }

    /// Copies the roles of a device's profile into its bindings once (59), or again with
    /// `reset`, which drops the owner's bindings of that device first; from then on only
    /// the bindings act for it.
    fn seed_device(&mut self, port: &str, name: &str, reset: bool) {
        if !reset && self.seeded.iter().any(|s| s == name) {
            self.migrate_fx_bindings();
            return;
        }
        let profile = self
            .profiles
            .iter()
            .find(|p| p.name == name)
            .cloned()
            .unwrap_or_else(|| Profile::default_for(port));
        if reset {
            self.learned.remove(name);
        }
        learn::seed(&mut self.learned, name, &profile.roles());
        if !self.seeded.iter().any(|s| s == name) {
            self.seeded.push(name.into());
        }
        for g in input::Group::ALL {
            self.sync_learned(name, g);
        }
        if let Some(m) = &self.midi {
            m.set_seeded(name);
        }
        for p in self.profiles.iter_mut().filter(|p| p.name == name) {
            p.set_seeded(true);
        }
        self.save_midi();
        let section = config::DevicesConfig {
            seeded: self.seeded.clone(),
        };
        if let Some(p) = &self.user_path
            && let Err(e) = config::save_section(p, "devices", &section)
        {
            log::error!("{e}");
        }
        log::info!("MIDI: {name} bindings from its profile");
    }

    /// Sends the learned controls of `profile` in `group` to the MIDI thread and the local copy.
    fn sync_learned(&mut self, profile: &str, group: input::Group) {
        let map = learn::group_map(&self.learned, profile, group);
        if let Some(m) = &self.midi {
            m.set_learned(profile, group, map.clone());
        }
        for p in self.profiles.iter_mut().filter(|p| p.name == profile) {
            p.set_learned(group.scope(), map.clone());
        }
    }

    fn migrate_fx_bindings(&mut self) {
        if learn::migrate_fx(&mut self.learned, &self.profiles, &self.seeded) {
            let names: Vec<_> = self.learned.keys().cloned().collect();
            for name in names {
                self.sync_learned(&name, input::Group::Fx);
            }
            self.save_midi();
        }
    }

    /// MIDI bindings of the config that name no action are dropped and the config saved
    /// once (71); the log lists them, the notice counts them. Nothing to drop: no notice.
    fn drop_dead_bindings(&mut self) -> Option<String> {
        let gone = learn::drop_dead(&mut self.learned);
        if gone.is_empty() {
            return None;
        }
        for g in &gone {
            log::warn!("MIDI binding to no action dropped: {g}");
        }
        self.save_midi();
        let n = gone.len();
        let s = if n == 1 { "" } else { "s" };
        Some(format!(
            "MIDI: {n} binding{s} to actions the program does not have dropped from the config"
        ))
    }

    fn save_midi(&mut self) {
        let result = match &self.user_path {
            Some(p) => config::save_section(p, "midi", &self.learned),
            None => Err("MIDI assignments are not saved: fix the config file first".into()),
        };
        if let Err(e) = result {
            log::error!("{e}");
            self.post_notice(Level::Error, (e, Instant::now()));
        }
    }

    /// A control decided on the MIDI tab waits on its row for Enter (74).
    fn learned(&mut self, profile: String, key: input::midi::Key) {
        if let Some(l) = self.learning.as_mut() {
            l.hold(profile, key);
        }
    }

    /// Enter on the MIDI tab with a control waiting: it is assigned and saved (74).
    fn learned_save(&mut self) {
        let Some(l) = self.learning.take_if(|l| l.pending.is_some()) else {
            return;
        };
        let Some((profile, key)) = l.pending else {
            return;
        };
        learn::assign(&mut self.learned, &profile, l.info.group, key, l.info.name);
        self.sync_learned(&profile, l.info.group);
        self.save_midi();
        log::info!("MIDI learn: {profile} {key} -> {}", l.info.name);
    }

    /// The rows of the MIDI learn list: a row per group and, under an open group, a row
    /// per its action; the pads and the banks stand in one row each (28).
    fn midi_list(&self) -> Vec<MidiListRow> {
        let sets = midi_sets();
        // the knob response first: one setting for every knob (58)
        let mut rows = vec![MidiListRow::Response];
        for (port, name) in &self.midi_devices {
            if !rows
                .iter()
                .any(|r| matches!(r, MidiListRow::Reset { name: n, .. } if n == name))
            {
                rows.push(MidiListRow::Reset {
                    name: name.clone(),
                    port: port.clone(),
                });
            }
        }
        let mut prev: Option<input::Group> = None;
        for info in input::ACTIONS {
            if prev != Some(info.group) {
                rows.push(MidiListRow::Group { group: info.group });
                prev = Some(info.group);
            }
            if !self.midi_open.contains(&info.group) {
                continue;
            }
            // the row of a set replaces the rows of the numbered pads and banks in it
            if let Some(&(name, infos)) = sets
                .iter()
                .find(|(_, s)| s.first().is_some_and(|f| f.name == info.name))
            {
                rows.push(MidiListRow::Set { name, infos });
                continue;
            }
            let numbered = (info.name.starts_with("pad.") || info.name.starts_with("bank."))
                && info
                    .name
                    .split('.')
                    .nth(1)
                    .is_some_and(|s| s.parse::<usize>().is_ok());
            if !numbered {
                rows.push(MidiListRow::Action { info });
            }
        }
        rows
    }

    /// Any profile has a learned control for `action` in `group`.
    fn learned_any(&self, group: input::Group, action: &str) -> bool {
        self.learned.values().any(|g| {
            g.get(group.key())
                .is_some_and(|m| m.values().any(|a| a == action))
        })
    }

    /// How many of the run row's actions have a learned control (28).
    fn learned_in(&self, infos: &[input::ActionInfo]) -> usize {
        infos
            .iter()
            .filter(|i| self.learned_any(i.group, i.name))
            .count()
    }

    /// How many actions of `group` have a learned control, over every profile.
    fn learned_count(&self, group: input::Group) -> usize {
        input::ACTIONS
            .iter()
            .filter(|i| i.group == group && self.learned_any(group, i.name))
            .count()
    }

    /// The rows the MIDI tab draws, from the list with its folded groups.
    fn midi_screen(&self) -> Vec<tui::MidiRow> {
        let profiles = self.live_profiles();
        let who = |info: &input::ActionInfo| -> String {
            let who: Vec<String> = profiles
                .iter()
                .filter_map(|p| {
                    Some(format!(
                        "{}: {}",
                        p.name,
                        p.assignment(info.group.scope(), info.name)?
                    ))
                })
                .collect();
            if who.is_empty() {
                "-".into()
            } else {
                who.join("; ")
            }
        };
        // a control waiting for Enter (74): its row shows `old → new` with what the control
        // keeps in the other groups, the row of its group that loses it is marked
        let pending = self
            .learning
            .as_ref()
            .and_then(|l| Some((l.info, l.pending.clone()?)));
        let taken = pending
            .as_ref()
            .and_then(|(i, (_, key))| self.taken_in(i.group, *key, i.name))
            .map(|t| t.0);
        let action = |info: &input::ActionInfo| {
            let mut row = (who(info), false, Vec::new());
            let Some((p, (profile, key))) = &pending else {
                return row;
            };
            let named = control_name(&profiles, profile, *key);
            if p.group != info.group {
                return row;
            }
            if p.name == info.name {
                let now = if row.0 == "-" { "none" } else { row.0.as_str() };
                row.0 = format!("{now} → {named}");
                row.2 = self
                    .bound_in(profile, *key, info.group, &input::Group::ALL)
                    .into_iter()
                    .map(|b| format!("{b} · stays"))
                    .collect();
            } else if taken == Some(info.name) {
                row.0 = format!("{}  ✗ loses {named}", row.0);
                row.1 = true;
            }
            row
        };
        self.midi_list()
            .iter()
            .map(|row| match row {
                MidiListRow::Group { group, .. } => tui::MidiRow::Group {
                    name: group.title(),
                    counts: format!(
                        "{} actions · {} learned",
                        input::ACTIONS.iter().filter(|i| i.group == *group).count(),
                        self.learned_count(*group)
                    ),
                },
                MidiListRow::Action { info } => {
                    let (who, taken, stays) = action(info);
                    tui::MidiRow::Action {
                        label: info.label.to_string(),
                        who,
                        taken,
                        stays,
                    }
                }
                MidiListRow::Reset { name, .. } => tui::MidiRow::Action {
                    label: "reset to profile".into(),
                    who: name.clone(),
                    taken: false,
                    stays: Vec::new(),
                },
                MidiListRow::Response => tui::MidiRow::Action {
                    label: "knob response".into(),
                    who: format!(
                        "{}   (jump / pickup / scale)",
                        self.takeover.response.name()
                    ),
                    taken: false,
                    stays: Vec::new(),
                },
                MidiListRow::Set { name, infos } => tui::MidiRow::Action {
                    label: name.to_string(),
                    who: format!("{} of {} learned", self.learned_in(infos), infos.len()),
                    taken: false,
                    stays: Vec::new(),
                },
            })
            .collect()
    }

    /// Starts the run of a pads or banks row at the first not learned one, or at 1 when
    /// all are (28).
    fn midi_run_start(&mut self, infos: &'static [input::ActionInfo]) {
        self.learning = None;
        let at = infos
            .iter()
            .position(|i| !self.learned_any(i.group, i.name))
            .unwrap_or(0);
        self.midi_run = Some(MidiRun {
            infos,
            at,
            taken: std::collections::HashSet::new(),
            learner: learn::Learning::new(&infos[at]),
        });
    }

    /// One control of the run decided: it is learned for the current action and the run
    /// moves on; it ends after the last one (28).
    fn midi_run_hit(&mut self, profile: String, key: input::midi::Key) {
        let Some(r) = self.midi_run.as_mut() else {
            return;
        };
        let infos = r.infos;
        let info = &infos[r.at];
        let last = r.at + 1 >= infos.len();
        r.taken.insert(key);
        if last {
            self.midi_run = None;
        } else {
            r.at += 1;
            r.learner = learn::Learning::new(&infos[r.at]);
        }
        learn::assign(&mut self.learned, &profile, info.group, key, info.name);
        self.sync_learned(&profile, info.group);
        self.save_midi();
        log::info!("MIDI learn: {profile} {key} -> {}", info.name);
    }

    /// The line of the MIDI tab about waiting for a control: a single learn or a run (28).
    fn learning_line(&self) -> Option<String> {
        if let Some(r) = &self.midi_run {
            let infos = r.infos;
            return Some(format!(
                " {} of {}: hit a control, Esc stops",
                infos[r.at].label,
                infos.len()
            ));
        }
        let l = self.learning.as_ref()?;
        Some(match &l.pending {
            Some((profile, key)) => format!(
                " {} for \"{}\" · Enter: assign and save · Esc: keep all · another control takes its place",
                control_name(&self.live_profiles(), profile, *key),
                l.info.label
            ),
            None => format!(" Move a control for \"{}\"... Esc cancels.", l.info.label),
        })
    }

    /// The next or previous knob response (58), applied at once and saved under `[knobs]`.
    fn step_response(&mut self, step: isize) {
        let all = input::midi::Response::ALL;
        let at = all
            .iter()
            .position(|r| *r == self.takeover.response)
            .unwrap_or(0) as isize;
        let next = all[(at + step).rem_euclid(all.len() as isize) as usize];
        self.takeover.response = next;
        let section = config::KnobsConfig {
            response: next.name().into(),
        };
        let result = match &self.user_path {
            Some(p) => config::save_section(p, "knobs", &section),
            None => Err("settings are not saved: fix the config file first".into()),
        };
        self.result_notice(result, format!("knobs: {}", next.name()));
    }

    /// Opens the group of the selected row (`true`) or closes it, and selects its row.
    fn midi_fold(&mut self, open: bool) {
        let list = self.midi_list();
        let group = match list.get(self.midi_row) {
            Some(MidiListRow::Group { group, .. }) => *group,
            Some(MidiListRow::Action { info }) => info.group,
            // the pads and the banks stand in GLOBAL: the fold covers their row
            Some(MidiListRow::Set { infos, .. }) => infos[0].group,
            Some(MidiListRow::Response | MidiListRow::Reset { .. }) | None => return,
        };
        if open {
            if !self.midi_open.contains(&group) {
                self.midi_open.push(group);
            }
        } else {
            self.midi_open.retain(|g| *g != group);
        }
        let list = self.midi_list();
        self.midi_row = list
            .iter()
            .position(|r| matches!(r, MidiListRow::Group { group: g, .. } if *g == group))
            .unwrap_or(self.midi_row);
    }

    /// A loop playing on the chop screen takes the bounds as they are now (49): a moved cut,
    /// slice bound or region edge is heard from the next turn on.
    fn follow_loop(&mut self) {
        let Some(sent) = self.loop_sent else {
            return;
        };
        let Some(region) = self
            .chop
            .as_ref()
            .filter(|c| c.looping)
            .map(|c| c.play_region())
        else {
            return;
        };
        if region != sent {
            self.send(Command::PreviewLoop {
                region: Some(region),
            });
            self.loop_sent = Some(region);
        }
    }

    /// After an undo on the chop screen: a region that had left pads out covers them
    /// again, they follow their slice again (35).
    fn relink_after_undo(&mut self) -> usize {
        let Some(c) = self.chop.as_ref() else {
            return 0;
        };
        let mut relinked = 0;
        let file = c.file.clone();
        // the region came back over pads it had left out: they follow their slice again
        let range = c.range;
        let fits = |r: Region| r.start >= range.start && r.end <= range.end;
        let (back, keep): (Vec<_>, Vec<_>) = std::mem::take(&mut self.range_unlinks)
            .into_iter()
            .partition(|(f, pads)| *f == file && pads.iter().all(|p| fits(self.pads.region[p.0])));
        self.range_unlinks = keep;
        for (_, pads) in back {
            for (a, link, slice) in pads {
                (self.pads.link[a], self.pads.slice[a]) = (link, slice);
                self.send(self.pads.pitch_gate_command(a));
                relinked += 1;
            }
        }
        relinked
    }

    /// The cuts are kept with the session also when no pad follows them (04e): any change
    /// of the open chop's cuts or range marks the session to be written.
    fn cuts_changed(&mut self, before: Option<chop::Saved>) {
        let now = self.chop.as_ref().map(|c| c.saved());
        // opening a chop changes nothing yet; another file opened over one marks it too,
        // which only writes the session once more
        if before.is_some() && now.is_some() && now != before {
            self.dirty();
        }
    }

    /// The slices of a file's cuts with the cut id at each slice start and the chop epoch,
    /// from the open chop, a kept one or one waiting in the session (27).
    fn file_slices(&self, file: &Path) -> Option<(Vec<Region>, Vec<u32>, u32)> {
        let saved = match self
            .chop
            .iter()
            .chain(self.chops.values())
            .find(|c| c.file == file)
        {
            Some(c) => c.saved(),
            None => self.pending_chops.get(file)?.clone(),
        };
        let mut bounds = vec![saved.range.start];
        bounds.extend(saved.cuts.iter().map(|&(_, at)| at));
        bounds.push(saved.range.end);
        let slices = bounds
            .windows(2)
            .map(|w| Region {
                start: w[0],
                end: w[1],
            })
            .collect();
        let ids = std::iter::once(0)
            .chain(saved.cuts.iter().map(|&(id, _)| id))
            .collect();
        Some((slices, ids, saved.epoch))
    }

    /// The sources a pad can step through: all, every slice of its file, its own bounds.
    fn source_list(&self, abs: usize) -> Vec<pads::Source> {
        let n = self.pads.file[abs]
            .as_deref()
            .and_then(|f| self.file_slices(f))
            .map_or(0, |(s, _, _)| s.len());
        let slices = (1..=n as u16).map(pads::Source::Slice);
        std::iter::once(pads::Source::All)
            .chain(slices)
            .chain(std::iter::once(pads::Source::Own))
            .collect()
    }

    /// The pad's start and end in frames and the sound's length; `None` without a sound.
    fn pad_bounds(&self, abs: usize) -> Option<(usize, usize, usize)> {
        let len = self.pads.sound[abs].as_ref()?.len().max(1);
        let r = self.pads.region[abs];
        Some((r.start.min(len), r.end.min(len), len))
    }

    /// The edit layer takes the keys: the edit is open and no question waits; a question
    /// takes Enter and Esc of the screen, where the edit's Enter closes the edit (76).
    fn edit_keys(&self) -> bool {
        self.edit && self.ask.is_none()
    }

    /// `Backspace` in the pad edit (76): asks before the pads of the scope with a sound go
    /// back to clean.
    fn ask_reset(&mut self) {
        let pads: Vec<usize> = self
            .pads
            .scope_pads()
            .into_iter()
            .filter(|&a| self.pads.file[a].is_some())
            .collect();
        let what = match pads.as_slice() {
            [] => return,
            [abs] => format!("pad {}", PAD_KEYS[abs % PADS_PER_BANK].to_ascii_uppercase()),
            many => format!("{} pads", many.len()),
        };
        self.ask = Some(Ask::ResetPads {
            pads,
            lines: vec![
                format!("Reset {what} to clean?"),
                "  Volume 0 dB, one-shot, all the rest at the defaults.".into(),
                "  The sound and its bounds stay. Ctrl+Z brings the settings back.".into(),
                "Enter: reset · Esc: keep".into(),
            ],
        });
    }

    /// `Backspace` with marks outside the edit (76): asks before the marked pads with a
    /// sound lose it.
    fn ask_clear(&mut self) {
        let pads: Vec<usize> = self
            .pads
            .marked
            .iter()
            .copied()
            .filter(|&a| self.pads.file[a].is_some())
            .collect();
        let what = match pads.len() {
            0 => return,
            1 => "1 pad".to_string(),
            n => format!("{n} pads"),
        };
        self.ask = Some(Ask::ClearPads {
            pads,
            lines: vec![
                format!("Clear {what}?"),
                "  The sounds leave the pads; their files on disk stay.".into(),
                "  Ctrl+Z brings them back.".into(),
                "Enter: clear · Esc: keep".into(),
            ],
        });
    }

    fn clear_pads(&mut self, pads: &[usize]) {
        for &abs in pads {
            // a file still loading for the pad would land after the clear
            self.load_gen[abs] += 1;
            let cmd = self.pads.clear(abs);
            self.send(cmd);
            self.send(self.pads.pitch_gate_command(abs));
        }
        self.dirty();
    }

    fn reset_pads(&mut self, pads: &[usize]) {
        for &abs in pads {
            for cmd in self.pads.reset_clean(abs) {
                self.send(cmd);
            }
        }
        self.dirty();
    }

    /// Every pad and the slots as they are now, for the undo (61).
    fn before(&self) -> Before {
        Before {
            pads: (0..engine::PADS).map(|a| self.pads.snapshot(a)).collect(),
            fx: self.pads.fx,
        }
    }

    /// What an action changes, as the undo says it (61).
    fn step_name(&self, action: Action) -> String {
        match action {
            Action::EditValue { .. } | Action::PadParam(_) => {
                let p = if self.screen == Screen::Fx {
                    self.fx_selected_param()
                } else {
                    self.pads.param
                };
                self.param_step_name(p)
            }
            Action::PadModeNext => "mode".into(),
            Action::PadPitchGateToggle => "Pitch Gate".into(),
            Action::PadClear => "clear".into(),
            Action::PadEdit if matches!(self.ask, Some(Ask::ResetPads { .. })) => "reset".into(),
            Action::PadEdit if matches!(self.ask, Some(Ask::ClearPads { .. })) => "clear".into(),
            Action::PadEdit if matches!(self.ask, Some(Ask::ClearPattern { .. })) => "clear".into(),
            _ if self.pad_in_hand.is_some() => "swap".into(),
            _ if self.browser.in_hand.is_some() => "file".into(),
            _ if self.copy_in_hand.is_some() => "copy".into(),
            _ => "change".into(),
        }
    }

    /// Keeps what an action changed of the pads as a step (61); a change of the same thing
    /// on the same pads within a second joins the step before, so a knob turn is one step.
    fn note_step(&mut self, b: Before, what: String) {
        // A pattern swap keeps its own inverse transaction, including identical sounds.
        if what == "swap" && self.pads.pattern.is_some() {
            return;
        }
        let pads: Vec<(usize, pads::PadSnap)> = b
            .pads
            .into_iter()
            .enumerate()
            .filter(|(a, s)| self.pads.snapshot(*a) != *s)
            .collect();
        let fx = (b.fx != self.pads.fx).then_some(b.fx);
        if pads.is_empty() && fx.is_none() {
            return;
        }
        self.push_step(pads, fx, what);
    }

    fn push_step(
        &mut self,
        pads: Vec<(usize, pads::PadSnap)>,
        fx: Option<[pads::FxSlot; engine::FX_SLOTS]>,
        what: String,
    ) {
        self.flush_audio();
        let now = Instant::now();
        let touched: Vec<usize> = pads.iter().map(|(a, _)| *a).collect();
        if let Some(last) = self.history.last_mut()
            && last.what == what
            && last.pattern.is_none()
            && what != "Pitch Gate"
            && !(what == "swap" && self.pads.pattern.is_some())
            && now.duration_since(last.at) < UNDO_JOIN
            && last
                .pads
                .iter()
                .map(|(a, _)| *a)
                .eq(touched.iter().copied())
            && last.fx.is_some() == fx.is_some()
        {
            last.at = now;
            return;
        }
        if self.history.len() == UNDO_STEPS {
            self.history.remove(0);
        }
        self.history.push(Step {
            what,
            pads,
            fx,
            swap: None,
            pattern: None,
            at: now,
        });
    }

    fn push_pattern_step(&mut self, before: pattern_record::Undo) {
        self.push_pattern_step_named(before, "pattern pass");
    }

    fn push_pattern_step_named(&mut self, before: pattern_record::Undo, what: &str) {
        if self.history.len() == UNDO_STEPS {
            self.history.remove(0);
        }
        self.history.push(Step {
            what: what.into(),
            pads: Vec::new(),
            fx: None,
            swap: None,
            pattern: Some(before),
            at: Instant::now(),
        });
    }

    /// `Ctrl+Z` on the pads (61): the last step comes back, every pad of it sounds as before.
    fn undo_pads(&mut self) {
        self.flush_audio();
        if let Some((before, what)) = self
            .history
            .last()
            .and_then(|step| Some((step.pattern.as_ref()?, step.what.clone())))
        {
            match before.restore(&mut self.pads) {
                Ok(command) => {
                    self.reset_pattern_takeover();
                    self.history.pop();
                    if self.audio.as_ref().is_some_and(|audio| {
                        !audio.stopped.load(Relaxed) && audio.pattern_capture.available()
                    }) {
                        self.send(command);
                    }
                    self.dirty();
                    // the step names itself: a pass, an erased pad or a clear (98i)
                    self.post_notice(Level::Done, (format!("undo: {what} on A1"), Instant::now()));
                }
                Err(error) => self.post_notice(Level::Error, (error, Instant::now())),
            }
            return;
        }
        let Some(step) = self.history.pop() else {
            self.post_notice(Level::Info, ("nothing to undo".into(), Instant::now()));
            return;
        };
        if let Some((a, b)) = step.swap {
            match self.pads.swap(a, b) {
                Ok(cmd) => {
                    self.load_gen[a] += 1;
                    self.load_gen[b] += 1;
                    self.send(cmd);
                }
                Err(error) => {
                    self.history.push(step);
                    self.post_notice(Level::Error, (error, Instant::now()));
                    return;
                }
            }
        } else {
            for (abs, snap) in &step.pads {
                // a file still loading for the pad would land over the undone one
                self.load_gen[*abs] += 1;
                for cmd in self.pads.put_back(*abs, snap) {
                    self.send(cmd);
                }
            }
        }
        if let Some(fx) = step.fx {
            self.pads.fx = fx;
            for k in 0..engine::FX_SLOTS {
                let cmd = self.pads.fx_command(k);
                self.send(cmd);
            }
        }
        self.dirty();
        let on = match step.pads.as_slice() {
            [] => "the shared slot".to_string(),
            [(abs, _)] => format!("pad {}", PAD_KEYS[abs % PADS_PER_BANK].to_ascii_uppercase()),
            many => format!("{} pads", many.len()),
        };
        self.post_notice(
            Level::Done,
            (format!("undo: {} on {on}", step.what), Instant::now()),
        );
    }

    fn set_pad_region(&mut self, abs: usize, region: Region) {
        self.pads.region[abs] = region;
        self.send(Command::SetPadRegion { pad: abs, region });
        self.send(self.pads.pitch_gate_command(abs));
        self.dirty();
    }

    /// Puts a pad on another source, one undo step (27).
    fn set_source(&mut self, abs: usize, source: pads::Source) {
        if source == self.pads.source(abs) {
            return;
        }
        let Some((start, end, _)) = self.pad_bounds(abs) else {
            return;
        };
        let region = match source {
            pads::Source::All => {
                (self.pads.link[abs], self.pads.slice[abs]) = (None, None);
                Region::WHOLE
            }
            pads::Source::Own => {
                (self.pads.link[abs], self.pads.slice[abs]) = (None, None);
                Region { start, end }
            }
            pads::Source::Slice(n) => {
                let file = self.pads.file[abs].clone();
                let Some((slices, ids, epoch)) = file.as_deref().and_then(|f| self.file_slices(f))
                else {
                    return;
                };
                let i = n as usize - 1;
                self.pads.link[abs] = Some((ids[i], epoch));
                self.pads.slice[abs] = Some(n);
                slices[i]
            }
        };
        self.set_pad_region(abs, region);
        self.lit = Some((Lit::Pad(pads::Param::Source), Instant::now()));
        self.post_notice(
            Level::Info,
            (format!("source: {}", source_word(source)), Instant::now()),
        );
    }

    /// Moves the pad's start or end to `to` frames; a pad that followed the file or its
    /// cuts becomes its own first, one undo step, and the status says so (27).
    fn move_bound(&mut self, abs: usize, end: bool, to: usize) {
        let Some((start, stop, len)) = self.pad_bounds(abs) else {
            return;
        };
        let gap = (self.rate as usize / 100).max(1); // 10 ms, as the chop region
        let region = if end {
            Region {
                start,
                end: to.clamp(start + gap, len),
            }
        } else {
            Region {
                start: to.min(stop.saturating_sub(gap)),
                end: stop,
            }
        };
        if (region.start, region.end) == (start, stop) {
            return;
        }
        let was = self.pads.source(abs);
        if was != pads::Source::Own {
            (self.pads.link[abs], self.pads.slice[abs]) = (None, None);
        }
        self.set_pad_region(abs, region);
        let what = if end {
            pads::Param::End
        } else {
            pads::Param::Start
        };
        self.lit = Some((Lit::Pad(what), Instant::now()));
        let key = PAD_KEYS[abs % PADS_PER_BANK].to_ascii_uppercase();
        let secs = |f: usize| f as f32 / self.rate.max(1) as f32;
        self.post_notice(
            Level::Info,
            (
                match was {
                    pads::Source::Slice(_) => {
                        format!("pad {key} no longer follows the cuts; Ctrl+Z links it back")
                    }
                    _ => format!(
                        "{}: {:.3} s",
                        if end { "end" } else { "start" },
                        secs(if end { region.end } else { region.start })
                    ),
                },
                Instant::now(),
            ),
        );
    }

    /// The value keys on the SOURCE page: the source steps through its list, a bound moves
    /// by the fine step (1 ms, a tap) or the coarse one (10 ms, held), as every value (27).
    fn source_value(&mut self, up: bool, fine: bool) {
        let abs = self.pads.selected_abs();
        let Some((start, stop, _)) = self.pad_bounds(abs) else {
            self.post_notice(
                Level::Warn,
                ("empty pad: nothing to play from".into(), Instant::now()),
            );
            return;
        };
        match self.pads.param {
            pads::Param::Source => {
                let list = self.source_list(abs);
                let at = list
                    .iter()
                    .position(|&s| s == self.pads.source(abs))
                    .unwrap_or(0);
                let to = if up {
                    (at + 1).min(list.len() - 1)
                } else {
                    at.saturating_sub(1)
                };
                self.set_source(abs, list[to]);
            }
            param => {
                let step = self.rate as usize / if fine { 1000 } else { 100 };
                let end = param == pads::Param::End;
                let from = if end { stop } else { start };
                let to = if up {
                    from + step
                } else {
                    from.saturating_sub(step)
                };
                self.move_bound(abs, end, to);
            }
        }
    }

    /// The parameter in knob slot `k` of the open edit page; `None` outside an edit mode
    /// or past the page.
    fn slot_param(&self, k: u8) -> Option<Param> {
        if self.screen == Screen::Fx {
            return (k < 3).then(|| Param::PadEffect(3 + self.fx_slot as u8 * 3 + k));
        }
        if !self.edit {
            return None;
        }
        let k = k as usize;
        match self.screen {
            Screen::Pads => Some(match self.pads.page().1.get(k)? {
                pads::Param::Volume => Param::PadVolume,
                pads::Param::Pitch => Param::PadPitch,
                pads::Param::Attack => Param::PadAttack,
                pads::Param::Decay => Param::PadDecay,
                pads::Param::Release => Param::PadRelease,
                pads::Param::Source => Param::PadSource,
                pads::Param::Start => Param::PadStart,
                pads::Param::End => Param::PadEnd,
                pads::Param::Filter => Param::PadFilter,
                pads::Param::Cutoff => Param::PadCutoff,
                pads::Param::Reso => Param::PadReso,
                p => Param::PadEffect(EFFECT_PARAMS.iter().position(|e| e == p)? as u8),
            }),
            Screen::Chop => [Param::SliceStart, Param::SliceEnd].get(k).copied(),
            _ => None,
        }
    }

    /// Left/Right on the settings screen: apply at once and save.
    fn change(&mut self, row: usize, step: isize) {
        if row == 3 {
            // the wave as a picture or Braille (04b), from the next start
            self.ui.graphics = !self.ui.graphics;
            let result = match &self.user_path {
                Some(p) => config::save_section(p, "ui", &self.ui),
                None => Err("settings are not saved: fix the config file first".into()),
            };
            let what = if self.ui.graphics {
                "picture"
            } else {
                "Braille"
            };
            self.result_notice(result, format!("wave: {what} from the next start"));
            return;
        }
        if row == 2 {
            let size = next_buffer_choice(self.cfg.buffer_size, step);
            if size == 0 {
                // auto: follow the server, nothing to ask for
                self.cfg.buffer_size = 0;
                self.save();
                return;
            }
            match self.audio_alive().map(|a| a.set_buffer_size(size)) {
                Some(Ok(())) => {
                    self.cfg.buffer_size = size;
                    self.save();
                }
                Some(Err(e)) => self.post_notice(Level::Error, (e, Instant::now())),
                None => {}
            }
            return;
        }
        let Some(devices) = self.audio_alive().and_then(|a| a.devices()) else {
            return;
        };
        let (list, current) = if row == 0 {
            (devices.outputs, &self.routed.output)
        } else {
            (devices.inputs, &self.routed.input)
        };
        let Some(next) = step_in(&list, current.as_deref(), step) else {
            return;
        };
        if row == 0 {
            self.cfg.output = Some(next);
        } else {
            self.cfg.input = Some(next);
        }
        self.reroute();
        self.save();
    }

    fn key_notice_blocked(&self) -> bool {
        self.pad_naming.is_some()
            || self.project_picker.is_some()
            || self.path_picker.is_some()
            || self.ask.is_some()
            || self.screen_learn.is_some()
            || self.learning.is_some()
            || self.help_open
            || (self.screen == Screen::Files && self.browser.editing)
    }

    fn moved_key_notice(&mut self, keymap: &Keymap, k: &event::KeyEvent) -> bool {
        if k.kind != event::KeyEventKind::Press || self.key_notice_blocked() {
            return false;
        }
        let key = input::key_name(k);
        if keymap
            .resolve_in(self.screen, self.edit_keys(), k)
            .is_some()
            || keymap
                .binding_for(self.screen, self.edit_keys(), &key)
                .is_some()
        {
            return false;
        }
        let row = MOVED_KEYS.iter().find(|(screen, old, _)| {
            screen.is_none_or(|screen| screen == self.screen) && *old == key
        });
        if let Some((_, _, message)) = row {
            self.post_notice(Level::Warn, ((*message).into(), Instant::now()));
            return true;
        }
        false
    }

    /// The first key event of the session clears the startup notices: the owner has read
    /// them (22).
    fn dismiss_startup(&mut self) {
        self.notices.dismiss_startup(Instant::now());
    }

    /// A pad that can sound now: a loaded sample, not still loading, not failed (73).
    fn pad_sounds(&self, pad: usize) -> bool {
        pad < PADS
            && self.pads.file[pad].is_some()
            && !self.pads.missing[pad]
            && self.pads.loading[pad].is_none()
    }

    /// A saved device that is missing now: (what, saved name, where audio goes).
    fn missing(&self) -> Option<(&'static str, &str, Option<&str>)> {
        self.audio_alive()?.devices()?;
        missing_device(&self.cfg.output, &self.routed.output)
            .map(|w| ("Output", w, self.routed.output.as_deref()))
            .or_else(|| {
                missing_device(&self.cfg.input, &self.routed.input)
                    .map(|w| ("Input", w, self.routed.input.as_deref()))
            })
    }
}

/// `~/…` instead of the full home path, so the files title stays short.
fn tilde(path: &str) -> String {
    let Ok(home) = std::env::var("HOME") else {
        return path.to_string();
    };
    if home.is_empty() {
        return path.to_string();
    }
    if path == home {
        return "~".into();
    }
    path.strip_prefix(&format!("{home}/"))
        .map(|rest| format!("~/{rest}"))
        .unwrap_or_else(|| path.to_string())
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// One short plain message when nothing plays; the raw reason goes to the log only.
const NO_AUDIO_SERVER: &str = "No sound: the audio server is not available.";
/// Same, when the fallback output carries the sound.
const AUDIO_FALLBACK: &str = "JACK did not answer; the sound goes through the fallback output.";
/// The startup warning on a terminal without the kitty keyboard protocol.
const NO_RELEASES_NOTICE: &str = "This terminal has no kitty keyboard protocol: no key releases, gate pads play as one-shot. Use Ghostty, kitty, WezTerm or foot.";

/// The saved device, if audio is not going to it right now.
fn missing_device<'a>(want: &'a Option<String>, got: &Option<String>) -> Option<&'a str> {
    want.as_deref().filter(|w| got.as_deref() != Some(*w))
}

/// Next device after the current one, wrapping around.
fn step_in(list: &[String], current: Option<&str>, step: isize) -> Option<String> {
    if list.is_empty() {
        return None;
    }
    let i = current
        .and_then(|c| list.iter().position(|d| d == c))
        .unwrap_or(0) as isize;
    Some(list[(i + step).rem_euclid(list.len() as isize) as usize].clone())
}

fn main() -> std::io::Result<()> {
    init_log();
    let loaded = config::load();
    let cfg = loaded.config.audio;
    let learned = loaded.config.midi;
    let seeded = loaded.config.devices.seeded;
    let favorites = loaded.config.browser.favorites;
    let ui = loaded.config.ui;
    let response =
        input::midi::Response::from_name(&loaded.config.knobs.response).unwrap_or_default();
    let paths = loaded.config.paths;
    // the roots appear empty on first use (11); a failure only speaks in the log
    for root in [&paths.projects, &paths.samples] {
        if let Err(e) = std::fs::create_dir_all(root) {
            log::warn!("cannot create {}: {e}", root.display());
        }
    }
    log::info!("start, {cfg:?}");

    let (audio, audio_msg) = match audio::start(cfg.backend, cfg.buffer_size) {
        Ok((a, notice)) => (Some(a), notice.map(|_| AUDIO_FALLBACK.to_string())),
        Err(e) => {
            log::error!("{e}");
            (None, Some(NO_AUDIO_SERVER.to_string()))
        }
    };
    // (11) the start reopens the last project; without one the start is scratch as today
    let start_project = initial_project(loaded.config.project.last.as_deref(), &paths);
    let (sess, session_notice, project_notice) = match &start_project {
        Some(p) => match session::load(&p.folder.join("project.toml")) {
            Ok(s) => (s, None, None),
            Err(e) => {
                log::error!("{e}");
                (session::Session::default(), Some(e.clone()), Some(e))
            }
        },
        None => match session::load(&session::path()) {
            Ok(s) => (s, None, None),
            Err(e) => {
                log::error!("{e}");
                (
                    session::Session::default(),
                    Some(format!("{e}; the session is not saved")),
                    None,
                )
            }
        },
    };
    // the project's stored paths resolve against its folder; the scratch session keeps
    // absolute paths as they are (11)
    let sess = with_resolved_paths(sess, start_project.as_ref().map(|p| p.folder.as_path()));
    let mut app = App {
        transport: transport::State::default(),
        audio,
        pending_audio: Default::default(),
        pending_manual_save: None,
        quit_requested: false,
        audio_held: [None; PADS],
        midi_ui_held: Default::default(),
        restoring_project: false,
        midi_preview_blocked: false,
        cfg,
        user_path: loaded.user_path,
        path_picker: None,
        project_picker: None,
        pad_naming: None,
        paths,
        routed: Routed::default(),
        screen: Screen::Pads,
        fx_slot: 0,
        fx_param: 0,
        settings_row: 0,
        help_open: false,
        help_overlay: help::State::default(),
        value_popup: None,
        pattern_panel: None,
        erase_pads: false,
        pattern_view: pattern_view::State::default(),
        silent_pad_keys: [false; PADS_PER_BANK],
        input_sources: input_sources::State::default(),
        pads: pads::Pads::default(),
        meters: meters::Meters::default(),
        settings_tab: 0,
        takeover: Takeover::with(response),
        pickup: None,
        midi_devices: Vec::new(),
        input_log: Vec::new(),
        midi: None,
        profiles: Vec::new(),
        learned,
        seeded,
        midi_row: 0,
        midi_open: Vec::new(),
        midi_run: None,
        learning: None,
        browser: browser::Browser::new(sess.browser_dir.clone()),
        loader: None,
        load_gen: vec![0; PADS + 2],
        load_region: vec![Region::WHOLE; PADS + 2],
        followers: Vec::new(),
        preview_on: true,
        master_db: 0,
        preview: None,
        pad_in_hand: None,
        copy_in_hand: None,
        session_path: session::path(),
        project: start_project,
        project_cfg: loaded.config.project,
        session_read_error: session_notice.clone(),
        session_write_failed: false,
        session_dirty: None,
        chop: None,
        chops: Default::default(),
        pending_chops: Default::default(),
        older: Vec::new(),
        pending_older: Vec::new(),
        fresh_undo: None,
        screen_learn: None,
        last_done: None,
        loop_sent: None,
        chop_from: None,
        history: Vec::new(),
        ui,
        range_unlinks: Vec::new(),
        plan: None,
        plan_undo: Vec::new(),
        edit: false,
        slice_end: false,
        bound_repeats: 0,
        lit: None,
        #[cfg(test)]
        fake_playhead: None,
        ask: None,
        rate: 48_000,
        input_level: 0.0,
        notices: Notices::new(Instant::now()),
        notice_origin: Origin::System,
        shown_error: None,
        frame_notice_level: None,
        chop_loading: None,
        keymap_warnings: Default::default(),
        keymap_warning_shown: false,
        // the splash shows from the first frame on and lives until the first sound (73)
        splash: true,
        recording: None,
        #[cfg(test)]
        sent: Vec::new(),
    };
    let sample_rate = app
        .audio
        .as_ref()
        .map(|a| a.stats.format().1)
        .filter(|r| *r > 0)
        .unwrap_or(48_000);
    app.rate = sample_rate;
    app.loader = Some(loader::start(sample_rate));
    // the start and the picker open a project through the same door (11); it keeps the
    // favorites the browser holds, so the saved ones go in first (03b)
    app.browser.favorites = favorites;
    app.apply_session(sess);
    // Session read protection below records this failure once at startup.
    if let Some(e) = project_notice.filter(|_| session_notice.is_none()) {
        app.post_startup(Level::Error, e);
    }
    let (keymap, keymap_notice, keymap_warnings) = load_keymap(
        &config::xdg_dir("XDG_CONFIG_HOME", ".config").join("keymap.toml"),
        DEFAULT_KEYMAP,
    );
    app.add_keymap_warnings(keymap_warnings);
    let (profiles, profile_errors) = midi::load_profiles(DEVICES);
    if let Some(msg) = audio_msg {
        app.post_startup(
            if app.audio.is_some() {
                Level::Warn
            } else {
                Level::Error
            },
            msg,
        );
    }
    for e in profile_errors {
        app.post_startup(
            Level::Warn,
            format!("{e}; that device uses the default profile"),
        );
    }
    let shared = Arc::new(midi::Shared::default());
    app.profiles = profiles.clone();
    if let Some(notice) = app.drop_dead_bindings() {
        app.post_startup(Level::Warn, notice);
    }
    app.migrate_fx_bindings();
    let maps = midi::parse_learned(&app.learned);
    for p in app.profiles.iter_mut() {
        midi::apply_learned(p, &maps);
        p.set_seeded(app.seeded.contains(&p.name));
    }
    app.publish_midi(&shared);
    app.midi = Some(midi::start(
        profiles,
        maps,
        app.seeded.clone(),
        app.audio.as_mut().and_then(|a| a.midi_commands.take()),
        app.audio.as_ref().map(|a| a.project.clone()),
        shared.clone(),
    ));
    if let Some(d) = app.audio.as_ref().and_then(|a| a.devices()) {
        log::info!("devices: outputs {:?}, inputs {:?}", d.outputs, d.inputs);
    }
    app.reroute();

    let mut term = ratatui::init();
    let kitty = terminal::supports_keyboard_enhancement().unwrap_or(false);
    if kitty {
        execute!(
            std::io::stdout(),
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                    | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
            )
        )?;
    }
    log::info!("kitty keyboard protocol: {kitty}");
    app.pads.track_keys = kitty;
    // Braille unless the owner switched the picture on (04b): only then ask the terminal
    let graphics = app.ui.graphics && graphics_support();
    let mut shown = graphics::State::default();
    let mut frame_text = view::FrameText::default();
    let mut status_text = tui::TextCache::default();
    // the startup notices live on the app: the first key event of the session clears them
    for notice in [loaded.error, session_notice, keymap_notice]
        .into_iter()
        .flatten()
    {
        app.post_startup(Level::Error, notice);
    }
    if !kitty {
        app.post_startup(Level::Warn, NO_RELEASES_NOTICE.into());
    }
    let mut held = HeldPads::default();
    let mut keys_down = input::HeldKeys::default();
    let mut mods = input::HeldModifiers::default();
    // the title is set only when the open project changes it (11)
    let mut title = app.window_title();
    let _ = execute!(std::io::stdout(), SetTitle(title.as_str()));

    let mut load_peak = 0.0;
    let mut load_taken = Instant::now();
    let result = loop {
        app.finish_project_restore();
        app.flush_audio();
        if app.quit_ready() {
            break Ok(());
        }
        let events: Vec<_> = app
            .midi
            .as_ref()
            .map(|m| m.events.try_iter().collect())
            .unwrap_or_default();
        for ev in events {
            app.midi_ui_event(ev);
        }
        if let Some((profile, key)) = app.learning.as_mut().and_then(|l| l.poll(Instant::now())) {
            app.learned(profile, key);
        }
        if let Some((profile, key)) = app
            .screen_learn
            .as_mut()
            .and_then(|l| l.learner.as_mut()?.poll(Instant::now()))
        {
            app.learn_bind(profile, key);
        }
        let replies: Vec<_> = app
            .loader
            .as_ref()
            .map(|(_, rx)| rx.try_iter().collect())
            .unwrap_or_default();
        for r in replies {
            app.loader_reply(r);
        }
        app.autosave(Instant::now());
        // the cap ends the recording in the audio thread: the status names it, the file goes in hand
        if app.recording.is_some()
            && app
                .audio
                .as_ref()
                .is_some_and(|a| a.recorder.stopped_by_cap())
        {
            app.record_stop();
        }
        app.publish_midi(&shared);
        if app
            .pickup
            .as_ref()
            .is_some_and(|(_, at)| at.elapsed() > PICKUP_HINT)
        {
            app.pickup = None;
        }
        if app
            .audio
            .as_ref()
            .is_some_and(|a| a.ports_changed.swap(false, Relaxed))
        {
            app.reroute();
        }
        if let Some(a) = &mut app.audio {
            a.free_garbage();
            if load_taken.elapsed() >= LOAD_WINDOW {
                load_peak = a.stats.take_load_peak();
                load_taken = Instant::now();
            }
            // smooth decay so the meter does not flicker between frames
            app.input_level = a.stats.take_input_peak().max(app.input_level * 0.8);
            app.meters.update(a.stats.take_output(), Instant::now());
        }
        let size = match term.size() {
            Ok(size) => size,
            Err(error) => break Err(error),
        };
        app.resize_help(size.width, size.height);
        let fresh = app.window_title();
        if fresh != title {
            title = fresh;
            let _ = execute!(std::io::stdout(), SetTitle(title.as_str()));
        }

        app.input_sources.advance(Instant::now());
        let notice = app.frame_notice();
        let (buffer_size, sample_rate) = app.audio.as_ref().map_or((0, 0), |a| a.stats.format());
        let text_at = Instant::now();
        let pattern_recording = app.pattern_record_state();
        let full = app.help_open.then(|| {
            let mut rows = app.full_help_recording_keys(&keymap, kitty, pattern_recording);
            if app.screen == Screen::Fx {
                rows.push((String::new(), format!("{} selected", app.help_state().page)));
            }
            rows
        });
        if let Some(rows) = &full {
            app.prepare_key_help(&keymap, rows, pattern_recording);
        }
        let status = tui::Status {
            badges: frame_text
                .badges_for(&app, text_at, pattern_recording)
                .to_vec(),
            save: app.save_status(),
            backend: app.audio_alive().map(|a| a.backend),
            recording: app.recording.is_some(),
            buffer_size,
            sample_rate,
            load_peak,
            xruns: app.audio.as_ref().map_or(0, |a| a.stats.xruns()),
            master_db: app.master_db,
            meters: (
                app.meters.level_db(),
                app.meters.peak_db(),
                app.meters.clip(),
            ),
            notice: notice.as_deref().map(|text| tui::NoticeLine {
                text,
                level: app.frame_notice_level.map(|level| match level {
                    Level::Info => tui::NoticeLevel::Info,
                    Level::Done => tui::NoticeLevel::Done,
                    Level::Warn => tui::NoticeLevel::Warn,
                    Level::Error => tui::NoticeLevel::Error,
                }),
            }),
            no_releases: !kitty,
            project: app
                .project
                .as_ref()
                .filter(|p| p.folder.is_dir())
                .map(|p| p.name.as_str()),
        };
        let device_lines: Vec<String> = app
            .midi_devices
            .iter()
            .map(|(port, profile)| format!("{port}  ({profile} profile)"))
            .collect();
        let midi_rows = app.midi_screen();
        let learning = app.learning_line();
        let dir = tilde(&app.browser.dir.display().to_string());
        let hand = app.browser.in_hand.as_deref().map(file_name);
        let out = out_wave(&app);
        // the roots of the Paths tab, `~`-shortened for the rows (11)
        let paths_view = (
            tilde(&app.paths.projects.display().to_string()),
            tilde(&app.paths.samples.display().to_string()),
        );
        // the project picker (11) is modal: while it is open the body shows it instead of
        // the screen, so the view replaces the whole screen on the display
        let project_picker = app.picker_view();
        let screen = match project_picker {
            Some(p) => tui::Screen::ProjectPicker(p),
            None => match (app.screen, &app.chop) {
                (Screen::Chop, Some(c)) => {
                    let mut view = app.chop_display_recording(c, pattern_recording);
                    // the keyboard of slices draws the slices' own waves
                    if view.slice_keys.is_none() {
                        bank_waves(&mut view.pads, &mut app.pads, app.rate);
                    }
                    view.which = app.which_chop();
                    app.learn_grid(&mut view.pads.pads);
                    tui::Screen::Chop(Box::new(app.mark_chop_recording(
                        view,
                        mods.get(),
                        pattern_recording,
                    )))
                }
                (Screen::Fx, _) => tui::Screen::Fx(app.fx_view()),
                (Screen::Files, _) => tui::Screen::Files(tui::Files {
                    dir: &dir,
                    entries: app
                        .browser
                        .entries
                        .iter()
                        .map(|e| (e.name.as_str(), e.is_dir, e.favorite.is_some()))
                        .collect(),
                    selected: app.browser.selected,
                    in_hand: hand.as_deref(),
                    error: app.browser.error.as_deref(),
                    pads: {
                        let mut v = tui::Pads {
                            feedback: app.pad_feedback(),
                            hidden: app.ui.hide_pads,
                            ..pads_view(&mut app.pads, None, app.pad_in_hand, out, None, app.rate)
                        };
                        app.learn_grid(&mut v.pads);
                        app.erase_pad_marks(&mut v, pattern_recording);
                        v.bank_pattern = app.bank_pattern();
                        v
                    },
                    filter: &app.browser.filter,
                    editing: app.browser.editing,
                    preview: app.preview.as_ref().map(|(s, w)| (*s, w.as_slice())),
                    playhead: app.preview_at(),
                }),
                (Screen::Pads | Screen::Chop, _) => {
                    let pickup = match app.pad_in_hand {
                        Some(abs) => Some(in_hand_line(&app.pads, abs)),
                        None => app.pickup.as_ref().map(|p| p.0.clone()),
                    };
                    let splash = app.splash_view();
                    let mut view = pads_view(
                        &mut app.pads,
                        pickup,
                        app.pad_in_hand,
                        out,
                        splash,
                        app.rate,
                    );
                    preview_pad_name(&mut view, app.pads.bank, app.pad_naming.as_ref());
                    if app.screen == Screen::Pads {
                        view.edit_key = keys(&keymap, Screen::Pads, &[Action::PadEdit]);
                    }
                    let mut view = app.mark_pads_recording(view, pattern_recording);
                    view.bank_pattern = app.bank_pattern();
                    if app.screen == Screen::Pads {
                        if app.pattern_view.overview {
                            view.overview = Some(app.pattern_overview_at(&keymap, size.width));
                        } else {
                            view.pattern = Some(app.pattern_strip(&keymap));
                        }
                    }
                    tui::Screen::Pads(view)
                }
                (Screen::Settings, _) => tui::Screen::Settings(tui::Settings {
                    row: app.settings_row,
                    output: app.routed.output.as_deref(),
                    output_missing: missing_device(&app.cfg.output, &app.routed.output),
                    input: app.routed.input.as_deref(),
                    input_missing: missing_device(&app.cfg.input, &app.routed.input),
                    input_level: app.input_level,
                    // the actual size the server runs; auto shows it in parentheses
                    buffer_size: app
                        .audio
                        .as_ref()
                        .map_or(app.cfg.buffer_size, |a| a.stats.format().0),
                    buffer_choice: app.cfg.buffer_size,
                    graphics: app.ui.graphics,
                    routing: app.audio_alive().is_some_and(|a| a.devices().is_some()),
                    backend_jack: matches!(app.cfg.backend, audio::Backend::Jack),
                    tab: app.settings_tab,
                    midi_devices: &device_lines,
                    input_log: &app.input_log,
                    midi_rows: &midi_rows,
                    midi_row: app.midi_row,
                    learning: learning.as_deref(),
                    paths: Some((paths_view.0.as_str(), paths_view.1.as_str())),
                    picker: app.path_picker.as_ref().map(|p| tui::PathPickerView {
                        title: if p.which == 0 {
                            " PICK THE PROJECTS ROOT "
                        } else {
                            " PICK THE SAMPLES ROOT "
                        },
                        entries: p
                            .entries
                            .iter()
                            .map(|e| {
                                e.file_name().map_or_else(
                                    || e.display().to_string(),
                                    |n| n.to_string_lossy().into_owned(),
                                )
                            })
                            .collect(),
                        selected: p.selected,
                    }),
                }),
            },
        };
        let groups = app.pattern_help_groups(
            &keymap,
            help_groups(&keymap, app.screen, &app.help_state()),
            pattern_recording,
            kitty,
        );
        let groups = match (&app.project_picker, mods.get()) {
            // the learn mode speaks in its own lines, the columns step aside (52)
            _ if app.screen_learn.is_some() => Vec::new(),
            // the picker takes the keys: the line says its own (35)
            (Some(p), _) => picker_help(p.naming.is_some()),
            (None, Some(m)) => modifier_groups(&keymap, app.screen, m, app.ui.hide_pads),
            (None, None) if app.pattern_view.overview => as_combos(groups),
            (None, None) if kitty => without_modifiers(groups),
            (None, None) => as_combos(groups),
        };
        let messages = app.message_history();
        let learn_lines = app.learning_view(&keymap);
        let question = app.question_view(&keymap);
        let feedback = frame_text.feedback_at(&app, text_at);
        let help = tui::Help {
            groups: &groups,
            hint: hint_for(&app),
            full: full
                .as_deref()
                .and_then(|rows| app.full_help_view(rows, &messages)),
            question: question.as_ref(),
            learn: learn_lines.as_ref(),
        };
        let value = app.current_value_at(Instant::now());
        let pattern_controls = app.pattern_controls_view(&keymap);
        let cell_px = if graphics {
            match terminal::window_size() {
                Ok(s) if s.columns > 0 && s.rows > 0 => (s.width / s.columns, s.height / s.rows),
                _ => (0, 0),
            }
        } else {
            (0, 0)
        };
        let mut want = Vec::new();
        let mut blockers = Vec::new();
        let drawn = term.try_draw(|f| -> std::io::Result<()> {
            blockers = tui::overlay_areas(f.area(), &screen, &help, value);
            if pattern_controls.is_some() {
                blockers.push(tui::pattern_controls_area(f.area(), &screen));
            }
            let commands = shown.before(graphics, app.screen, f.area(), cell_px, &blockers);
            graphics::send(&mut std::io::stdout(), &commands)?;
            tui::draw_cached(
                f,
                &status,
                &screen,
                &help,
                &feedback,
                &mut status_text,
                graphics.then_some(&mut want),
            );
            if let Some(value) = value {
                tui::value_popup(f, value);
            }
            if let Some(controls) = &pattern_controls {
                tui::pattern_controls(f, &screen, controls);
            }
            Ok(())
        });
        if let Err(e) = drawn {
            break Err(e);
        }
        // Only unobstructed pictures go out after the successful text frame.
        let commands = shown.after(&mut want, &blockers);
        if let Err(e) = graphics::send(&mut std::io::stdout(), &commands) {
            break Err(e);
        }

        match event::poll(FRAME) {
            Ok(false) => continue,
            Err(e) => break Err(e),
            Ok(true) => {}
        }
        let Event::Key(k) = (match event::read() {
            Ok(e) => e,
            Err(e) => break Err(e),
        }) else {
            continue;
        };
        if app.keyboard_event(&keymap, &k, kitty, &mut keys_down, &mut held, &mut mods) {
            break Ok(());
        }
    };

    app.save_on_exit();
    if kitty {
        let _ = execute!(std::io::stdout(), PopKeyboardEnhancementFlags);
    }
    if graphics {
        let mut out = std::io::stdout();
        let _ = out.write_all(shown.drop_all().concat().as_bytes());
        let _ = out.flush();
    }
    ratatui::restore();
    log::info!("exit");
    result
}

/// Asks the terminal whether it speaks kitty graphics (04b): a graphics reply that comes
/// before the device attributes reply means yes. Inside tmux, where escape codes reach
/// only the foreground pane, and without a pixel size the graphics stay off.
fn graphics_support() -> bool {
    if std::env::var_os("TMUX").is_some() {
        log::info!("kitty graphics: off, inside tmux");
        return false;
    }
    match terminal::window_size() {
        Ok(s) if s.width > 0 && s.height > 0 => {}
        _ => {
            log::info!("kitty graphics: off, the terminal does not report its size in pixels");
            return false;
        }
    }
    // the design's query, then the attributes query; the ack comes first and surfaces
    // through crossterm as Alt+_ — the whole window is drained, keys typed in it are lost
    let query = "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c";
    let mut out = std::io::stdout();
    if out
        .write_all(query.as_bytes())
        .and_then(|_| out.flush())
        .is_err()
    {
        log::info!("kitty graphics: off, the query did not go out");
        return false;
    }
    let deadline = Instant::now() + Duration::from_millis(200);
    let mut seen = false;
    while matches!(
        event::poll(deadline.saturating_duration_since(Instant::now())),
        Ok(true)
    ) {
        match event::read() {
            Ok(ev) => seen |= tui::is_graphics_ack(&ev),
            Err(_) => break,
        }
    }
    log::info!("kitty graphics: {seen}");
    seen
}

/// The hint line under the help groups; a short-lived message never removes it.
fn hint_for(app: &App) -> Option<&'static str> {
    match app.screen {
        Screen::Pads if app.pattern_view.edit => {
            Some("A1 step edit: pads remove one hit; empty adds none. Ctrl+Z restores.")
        }
        Screen::Pads if app.pattern_view.overview => Some(if app.pattern_view.hide_empty_banks {
            "A1 table: view only · pads play · Alt+H shows empty banks."
        } else {
            "A1 table: view only · pads play · Alt+H hides empty banks."
        }),
        Screen::Pads if app.pads.file.iter().all(Option::is_none) => {
            Some("Pads are empty: F6 opens files, Enter takes one, a pad hit puts it there.")
        }
        Screen::Pads => Some("Hit 1-4 Q-R A-F Z-V to play pads; Shift for full velocity."),
        Screen::Fx => Some("Shared by all pads: Tab selects FX 1 / FX 2; I K select, J L change."),
        Screen::Files if app.browser.editing => {
            Some("Type to filter; pads are silent. Enter keeps the filter, Esc clears it.")
        }
        Screen::Files => {
            Some("Enter takes a file, a pad hit puts it there; Ctrl+Enter: the selected pad.")
        }
        Screen::Settings => {
            Some("Left/Right applies at once and saves to ~/.config/ptsr/config.toml.")
        }
        Screen::Chop if app.plan.is_some() => Some(
            "Hit pads to plan where the slices go; J L pick the slice, PgUp/PgDn the bank; Enter applies.",
        ),
        // the next step of the chop (35): start it, lay it out, then fine-tune the bounds
        Screen::Chop if app.edit => None,
        Screen::Chop if app.chop.as_ref().is_some_and(|c| c.slices().len() > 1) => {
            if app.linked_pads() > 0 && app.slices_on_no_pad() > 0 {
                Some("○ slices lie on no pad: P, J L and a pad hit place them.")
            } else if app.linked_pads() > 0 {
                Some("Hold Ctrl or Alt and press J L to move a bound; the pads follow it.")
            } else {
                Some("P lays the slices on pads; hit a pad to hear its slice.")
            }
        }
        Screen::Chop => Some(
            "Start with Y (equal parts) or Space and M on the beat; P lays the slices on pads.",
        ),
    }
}

// Temporary migration hints: remove after version 0.2.
const MOVED_KEYS: &[(Option<Screen>, &str, &str)] = &[
    (Some(Screen::Chop), "Ctrl+N", "normalize moved to Ctrl+G"),
    (Some(Screen::Chop), "Ctrl+Y", "chop by hits moved to H"),
    (Some(Screen::Pads), "J", "previous bank moved to U"),
    (Some(Screen::Pads), "L", "next bank moved to O"),
];

const CLEANUP_DEFAULTS: &[(Option<Screen>, &str, &str)] = &[
    (None, "Space", "transport.toggle"),
    (None, "Ctrl+Up", "transport.tempo_up"),
    (None, "Ctrl+Down", "transport.tempo_down"),
    (None, "Ctrl+Shift+Up", "transport.tempo_up_fine"),
    (None, "Ctrl+Shift+Down", "transport.tempo_down_fine"),
    (None, "Alt+M", "metronome.toggle"),
    (None, "Alt+R", "record.mic"),
    (None, "Ctrl+R", "pattern.record"),
    (None, "Alt+E", "pattern.erase"),
    (None, "Alt+N", "pattern.controls"),
    (None, "Alt+Z", "pattern.undo"),
    (Some(Screen::Chop), "Ctrl+G", "chop.normalize"),
    (Some(Screen::Chop), "H", "chop.hits"),
    (Some(Screen::Chop), "Shift+H", "chop.hits_back"),
    (Some(Screen::Pads), "U", "bank.prev"),
    (Some(Screen::Pads), "O", "bank.next"),
    (Some(Screen::Pads), "Alt+G", "pad.pitch_gate_toggle"),
    (Some(Screen::Pads), "Alt+Down", "pattern.view_prev"),
    (Some(Screen::Pads), "Alt+Up", "pattern.view_next"),
    (Some(Screen::Pads), "Ctrl+C", "pad.copy"),
    (Some(Screen::Pads), "J", "edit.value_dec"),
    (Some(Screen::Pads), "L", "edit.value_inc"),
    (Some(Screen::Pads), "Left", "edit.value_dec"),
    (Some(Screen::Pads), "Right", "edit.value_inc"),
];

fn keymap_shadow_warnings(shipped: &Keymap, merged: &Keymap) -> Vec<String> {
    let mut warnings = Vec::new();
    for screen in [
        Screen::Pads,
        Screen::Chop,
        Screen::Files,
        Screen::Fx,
        Screen::Settings,
    ] {
        let screen_name = screen.key();
        for &(scope, key, name) in CLEANUP_DEFAULTS {
            if scope.is_some_and(|scope| scope != screen) {
                continue;
            }
            let Some(action) = Action::parse(name) else {
                continue;
            };
            // Existing edit bindings alone do not mean the new screen default has shipped.
            if shipped.binding_for(screen, false, key) != Some(Some(action)) {
                continue;
            }
            if [false, true].into_iter().any(|editing| {
                shipped.binding_for(screen, editing, key) == Some(Some(action))
                    && merged.binding_for(screen, editing, key) != Some(Some(action))
            }) {
                warnings.push(format!("keymap.toml: {key} on {screen_name} hides {name}"));
            }
        }
    }
    warnings
}

/// The shipped keymap with the owner's file `user` over it (64); the working folder does
/// not matter. A broken file is named with its line and the shipped keys stay.
fn load_keymap(user: &Path, defaults: &str) -> (Keymap, Option<String>, Vec<String>) {
    let (mut keymap, _) =
        Keymap::from_toml(defaults).expect("the shipped keymap parses (tested in input)");
    let shown = user.display();
    let (warnings, notice) = match std::fs::read_to_string(user) {
        Ok(text) => match keymap.overlay(&text) {
            Ok(w) => (w, None),
            Err(e) => (vec![], Some(format!("{shown} {e}; using the shipped keys"))),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (vec![], None),
        Err(e) => (
            vec![],
            Some(format!("{shown}: {e}; using the shipped keys")),
        ),
    };
    for w in &warnings {
        log::warn!("{w}");
    }
    let warning = warnings.into_iter().next().map(|w| format!("{shown}: {w}"));
    let (shipped, _) = Keymap::from_toml(defaults).expect("the shipped keymap parses");
    let shadows = keymap_shadow_warnings(&shipped, &keymap);
    for warning in &shadows {
        log::warn!("{warning}");
    }
    (keymap, notice.or(warning), shadows)
}

use pads::PAD_KEYS;

/// The curve of a pad filter for the strip (44): 256 points along the octaves.
fn filter_curve(s: engine::filter::FilterSettings) -> tui::FilterCurve {
    use engine::filter::{CUTOFF_MAX, CUTOFF_MIN, FilterKind, response};
    const POINTS: usize = 256;
    // half an octave of room at both ends: a hump at 20 Hz or 20 kHz stays on the screen
    let span = (CUTOFF_MIN / 2f32.sqrt(), CUTOFF_MAX * 2f32.sqrt());
    let levels = (0..POINTS)
        .map(|i| {
            let hz = span.0 * (span.1 / span.0).powf(i as f32 / (POINTS - 1) as f32);
            response(s, hz)
        })
        .collect();
    let caption = match s.kind {
        FilterKind::Off => "filter off".to_string(),
        k => format!(
            "{} · {} · reso {:.0} %",
            pads::kind_name(k),
            pads::cutoff_text(s.cutoff_hz),
            s.resonance * 100.0
        ),
    };
    tui::FilterCurve {
        caption,
        levels,
        span,
    }
}

/// The pad parameter of a filter knob.
fn pad_filter(param: Param) -> pads::Param {
    match param {
        Param::PadFilter => pads::Param::Filter,
        Param::PadCutoff => pads::Param::Cutoff,
        _ => pads::Param::Reso,
    }
}

/// The pad parameter of an envelope knob.
fn pad_env(param: Param) -> pads::Param {
    match param {
        Param::PadAttack => pads::Param::Attack,
        Param::PadDecay => pads::Param::Decay,
        _ => pads::Param::Release,
    }
}

/// Snapshot of the strip's history, newest at the right: what the recording writes while
/// one runs (25), the live output otherwise.
fn out_wave(app: &App) -> Vec<f32> {
    app.audio
        .as_ref()
        .map_or_else(Vec::new, |a| match app.recording {
            Some(_) => a.stats.rec_history(512),
            None => a.stats.output_history(512),
        })
}

/// What differs on a pad from a fresh one (55): its parameters off their defaults; the
/// source block when it plays a part of the file. An empty pad has nothing.
fn pad_changes(p: &pads::Pads, abs: usize) -> Vec<tui::PadParam> {
    use tui::PadParam as W;
    if p.file[abs].is_none() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut add = |on: bool, w: W| {
        if on {
            out.push(w);
        }
    };
    add(p.volume_db[abs] != 0, W::Volume);
    add(p.pitch[abs] != 0, W::Pitch);
    add(p.group[abs] != 0, W::Group);
    add(p.attack_ms[abs] != 0, W::Attack);
    add(p.decay_ms[abs] != 0, W::Decay);
    add(p.release_ms[abs] != 0, W::Release);
    // a filter that is off counts for nothing, whatever its cutoff and reso (55)
    let on = p.filter[abs] != engine::filter::FilterKind::Off;
    add(on, W::Filter);
    add(on && p.cutoff_hz[abs] != pads::CUTOFF_DEFAULT, W::Cutoff);
    add(on && p.reso[abs] != 0, W::Reso);
    let part = p.source(abs) != pads::Source::All;
    for w in [W::Source, W::Start, W::End] {
        add(part, w);
    }
    add(p.drive[abs] != 0, W::Drive);
    add(p.sends[abs][0] != 0, W::Send1);
    add(p.sends[abs][1] != 0, W::Send2);
    out
}

/// The mark at the end of a pad's mode line (77): `✱` when the pad has settings the line
/// does not show already: pitch, envelope, filter on, drive, sends. Not the volume, the
/// mode, the group (on the line) or the source (a slice shows in the name).
fn pad_glyphs(changed: &[tui::PadParam]) -> String {
    use tui::PadParam as W;
    let more = [
        W::Pitch,
        W::Attack,
        W::Decay,
        W::Release,
        W::Filter,
        W::Cutoff,
        W::Reso,
        W::Drive,
        W::Send1,
        W::Send2,
    ];
    if more.iter().any(|w| changed.contains(w)) {
        "✱".into()
    } else {
        String::new()
    }
}

fn preview_pad_name(view: &mut tui::Pads, bank: usize, naming: Option<&(usize, String)>) {
    if let Some((abs, name)) = naming
        && *abs / PADS_PER_BANK == bank
        && let Some(pad) = view.pads.get_mut(*abs % PADS_PER_BANK)
    {
        pad.name.clone_from(name);
        pad.naming = true;
    }
}

fn pads_view(
    p: &mut pads::Pads,
    pickup: Option<String>,
    in_hand: Option<usize>,
    out_wave: Vec<f32>,
    splash: Option<tui::Splash>,
    rate: u32,
) -> tui::Pads {
    let pads = (0..PADS_PER_BANK)
        .map(|i| {
            let abs = p.bank * PADS_PER_BANK + i;
            let changed = pad_changes(p, abs);
            tui::PadView {
                naming: false,
                glyphs: pad_glyphs(&changed),
                pitch_gate: view::pitch_gate(p, abs),
                changed,
                dim: false,
                dashed: false,
                source: source_word(p.source(abs)),
                bounds: p.sound[abs].as_ref().map_or((0.0, 0.0), |s| {
                    let r = p.region[abs];
                    let secs = |f: usize| f.min(s.len()) as f32 / rate.max(1) as f32;
                    (secs(r.start), secs(r.end))
                }),
                wave: p.wave(abs, rate, BIG_COLS),
                key: PAD_KEYS[i],
                name: if p.file[abs].is_some() {
                    pad_name(p, abs)
                } else {
                    "empty".into()
                },
                detail: Some(match p.file[abs].as_deref() {
                    None => format!("empty · test tone {}", test_tone_name(abs)),
                    Some(f) => format!("{} · {}", file_name(f), source_word(p.source(abs))),
                }),
                missing: p.missing[abs],
                loading: p.loading[abs],
                in_hand: in_hand == Some(abs),
                marked: p.marked.contains(&abs),
                note: None,
                gate: p.gate[abs],
                volume_db: p.volume_db[abs] as f32,
                pitch: p.pitch[abs],
                attack_ms: p.attack_ms[abs],
                decay_ms: p.decay_ms[abs],
                release_ms: p.release_ms[abs],
                group: p.group[abs],
                filter: [
                    pads::kind_name(p.filter[abs]).into(),
                    pads::cutoff_text(p.cutoff_hz[abs]),
                    format!("{} %", p.reso[abs]),
                ],
                flash: p.held[abs] || p.hit_at[abs].is_some_and(|t| t.elapsed() < HIT_FLASH),
            }
        })
        .collect();
    // the FX block of the selected pad (56): each send names the kind of its slot
    let sel = p.bank * PADS_PER_BANK + p.selected;
    let fx = [pads::Param::Drive, pads::Param::Send1, pads::Param::Send2]
        .into_iter()
        .map(|w| {
            let mut value = p.fx_text(sel, w);
            if let Some(k) = [pads::Param::Send1, pads::Param::Send2]
                .iter()
                .position(|s| *s == w)
            {
                value = format!("{value}  {}", pads::kind_word(p.fx[k].kind));
            }
            (tui_param(w), label8(p.name(w)), value)
        })
        .collect();
    tui::Pads {
        feedback: tui::PadFeedback::default(),
        pattern: None,
        overview: None,
        playing: false,
        fx,
        bank: p.bank,
        pads,
        selected: p.selected,
        pickup,
        param: tui_param(p.param),
        scope: p.scope_name(),
        bank_fill: bank_fill(p),
        bank_marks: bank_marks(p),
        bank_pattern: tui::BankPattern::default(),
        out_wave,
        splash,
        big: big_view(p, rate),
        curve: None,
        learn: Vec::new(),
        edit: None,
        pages: Vec::new(),
        knobs: Vec::new(),
        lit: None,
        hidden: false,
        edit_key: String::new(),
        slot: None,
    }
}

/// Resolution of the big pad view: more columns than any terminal row is wide, so the
/// tui can shrink it to any width without losing the shape.
const BIG_COLS: usize = 1200;

/// The piece of the selected pad with the pad's envelope over it, or nothing when the
/// pad is empty.
fn big_view(p: &pads::Pads, rate: u32) -> Option<tui::BigView> {
    let abs = p.bank * PADS_PER_BANK + p.selected;
    // an empty pad plays its test tone: the edit shows that piece (98h)
    let tone;
    let (sound, region) = match p.sound[abs].as_deref() {
        Some(sound) => (sound, p.region[abs]),
        None => {
            tone = Sample::mono(engine::test_tone(rate, abs));
            (&tone, Region::WHOLE)
        }
    };
    let (wave, env, dur_ms) = pads::piece(
        sound,
        region,
        rate,
        p.attack_ms[abs],
        p.decay_ms[abs],
        BIG_COLS,
    );
    let share = |ms: u16| f32::from(ms) / dur_ms.max(1.0);
    Some(tui::BigView {
        wave,
        env,
        attack: share(p.attack_ms[abs]).min(1.0),
        decay: share(p.decay_ms[abs]).min(1.0),
        release: share(p.release_ms[abs]),
    })
}

/// The pad's envelope as a multiplier over the piece, one value per column: attack
/// fades in over the first `attack_ms`, decay dies into silence over the last
/// `decay_ms` — the same ramps the audio voice follows.
fn env_curve(dur_ms: f32, attack_ms: u16, decay_ms: u16, cols: usize) -> Vec<f32> {
    (0..cols)
        .map(|c| {
            let t = (c as f32 + 0.5) / cols.max(1) as f32;
            let at = if attack_ms == 0 {
                1.0
            } else {
                (t * dur_ms / f32::from(attack_ms)).min(1.0)
            };
            let de = if decay_ms == 0 {
                1.0
            } else {
                ((1.0 - t) * dur_ms / f32::from(decay_ms)).min(1.0)
            };
            at * de
        })
        .collect()
}

/// The buffer row's next choice: 0 is auto (follow the server), then the fixed sizes.
fn next_buffer_choice(current: u32, step: isize) -> u32 {
    let pos = if current == 0 {
        0
    } else {
        BUFFER_SIZES
            .iter()
            .position(|&b| b >= current)
            .map_or(BUFFER_SIZES.len(), |i| i + 1)
    };
    let next = (pos as isize + step).clamp(0, BUFFER_SIZES.len() as isize) as usize;
    if next == 0 { 0 } else { BUFFER_SIZES[next - 1] }
}

/// Per bank, a bitmask of its 16 pads with a sound.
fn bank_fill(p: &pads::Pads) -> [u16; tui::BANKS] {
    let mut fill = [0u16; tui::BANKS];
    for abs in 0..PADS {
        if p.file[abs].is_some() {
            fill[abs / PADS_PER_BANK] |= 1 << (abs % PADS_PER_BANK);
        }
    }
    fill
}

/// Per bank, a bitmask of its marked pads (76).
fn bank_marks(p: &pads::Pads) -> [u16; tui::BANKS] {
    let mut marks = [0u16; tui::BANKS];
    for &abs in &p.marked {
        marks[abs / PADS_PER_BANK] |= 1 << (abs % PADS_PER_BANK);
    }
    marks
}

/// The pad already plays this whole file: putting it again changes nothing.
fn same_whole_sound(p: &pads::Pads, abs: usize, path: &Path) -> bool {
    p.file[abs].as_deref() == Some(path) && p.region[abs] == Region::WHOLE
}

#[derive(Debug, PartialEq)]
enum Rec {
    Start(u8),
    Stop,
}

/// What a record action does to the recording from `running`. One recording at a time:
/// the other source's key is ignored. A second press stops — a controller button or a
/// terminal without releases; with releases, held-key repeats are filtered before this.
fn record_step(action: Action, running: Option<u8>) -> Option<Rec> {
    let source = match action {
        Action::RecordMic(false) => {
            return (running == Some(audio::SOURCE_MIC)).then_some(Rec::Stop);
        }
        Action::RecordMic(true) => audio::SOURCE_MIC,
        Action::RecordOutputToggle => audio::SOURCE_OUTPUT,
        _ => return None,
    };
    match running {
        None => Some(Rec::Start(source)),
        Some(s) if s == source => Some(Rec::Stop),
        Some(_) => None,
    }
}

/// Splits interleaved stereo into a sample with 5 ms fades at both edges; `None` when
/// nothing was recorded or it is silent.
fn sample_from_interleaved(buf: &[f32], rate: u32) -> Option<Sample> {
    if buf.is_empty() {
        return None;
    }
    let mut left = Vec::with_capacity(buf.len() / 2);
    let mut right = Vec::with_capacity(buf.len() / 2);
    let mut peak = 0.0f32;
    for pair in buf.chunks(2) {
        let l = pair[0];
        let r = *pair.get(1).unwrap_or(&l);
        peak = peak.max(l.abs()).max(r.abs());
        left.push(l);
        right.push(r);
    }
    // quieter than -60 dB is silence: a muted microphone must not become a pad
    if peak < 0.001 {
        return None;
    }
    // the key is pressed in the middle of a sound: without the fades the pad would click
    let fade = (rate as usize / 200).max(1).min(left.len() / 2);
    let n = left.len();
    for i in 0..fade {
        let g = i as f32 / fade as f32;
        for ch in [&mut left, &mut right] {
            ch[i] *= g;
            ch[n - 1 - i] *= g;
        }
    }
    Some(Sample {
        left: left.into(),
        right: Some(right.into()),
    })
}

/// `rec-<stamp>.wav` in `root`; two recordings within one second get `-2`, `-3`… so the
/// second never overwrites a file a pad may already play.
fn free_name(root: &Path, stamp: &str) -> PathBuf {
    let mut path = root.join(format!("rec-{stamp}.wav"));
    let mut n = 2;
    while path.exists() {
        path = root.join(format!("rec-{stamp}-{n}.wav"));
        n += 1;
    }
    path
}

/// `YYYYMMDD-HHMMSS` of a unix time, no calendar libraries.
fn stamp(t: std::time::SystemTime) -> String {
    let secs = t
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
    let (h, m, s) = (rem / 3600, rem / 60 % 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let d = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * d + 2) / 153;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    if mo <= 2 {
        y += 1;
    }
    let day = d - (153 * mp + 2) / 5 + 1;
    format!("{y:04}{mo:02}{day:02}-{h:02}{m:02}{s:02}")
}

/// The end of every overwrite question.
fn overwrite_footer() -> [String; 3] {
    [
        "The old sounds leave these pads; their files on disk stay.".into(),
        "This cannot be undone.".into(),
        "Enter: replace · Esc: keep".into(),
    ]
}

/// One row of the MIDI learn list: a group with its actions folded in, or an action of
/// an open group (28).
enum MidiListRow {
    /// How the knobs catch a value (58): jump, pickup, scale.
    Response,
    /// A connected device's bindings back to its profile (59), after a question.
    Reset {
        name: String,
        port: String,
    },
    Group {
        group: input::Group,
    },
    Action {
        info: &'static input::ActionInfo,
    },
    Set {
        name: &'static str,
        infos: &'static [input::ActionInfo],
    },
}

/// A run over the pads or the banks row: every control pressed goes to the next one
/// (28).
struct MidiRun {
    infos: &'static [input::ActionInfo],
    /// The action waiting for a control now.
    at: usize,
    /// The controls already taken in this run: a repeated hit is ignored.
    taken: std::collections::HashSet<input::midi::Key>,
    learner: learn::Learning,
}

/// The tail of ACTIONS from `first`, `len` long: a run row of the MIDI tab (28).
fn action_range(first: &str, len: usize) -> &'static [input::ActionInfo] {
    let at = input::ACTIONS
        .iter()
        .position(|i| i.name == first)
        .expect("the pads and the banks are listed");
    &input::ACTIONS[at..at + len]
}

/// The pads and the banks of GLOBAL: one row each, learned in a run (28).
fn midi_sets() -> [(&'static str, &'static [input::ActionInfo]); 2] {
    [
        ("pads 1–16", action_range("pad.1", 16)),
        ("banks 1–8", action_range("bank.1", 8)),
    ]
}

/// The group row before or after `at` in the list; a jump never leaves the group rows.
fn group_jump(rows: &[MidiListRow], at: usize, next: bool) -> Option<usize> {
    let groups: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| matches!(r, MidiListRow::Group { .. }))
        .map(|(i, _)| i)
        .collect();
    // above the first group (the knob response row, 58): PgDn goes to the first group
    let Some(now) = groups.iter().rposition(|&g| g <= at) else {
        return next.then(|| groups.first().copied()).flatten();
    };
    let to = if next {
        (now + 1).min(groups.len() - 1)
    } else if groups[now] < at {
        now
    } else {
        now.saturating_sub(1)
    };
    groups.get(to).copied()
}

/// The file-list keys while a file is in hand: they move the pad selection instead.
/// "bank 1, pad Q": where a pad is, for questions.
fn pad_place(abs: usize) -> String {
    format!(
        "bank {}, pad {}",
        abs / PADS_PER_BANK + 1,
        PAD_KEYS[abs % PADS_PER_BANK].to_ascii_uppercase()
    )
}

/// "pad Q kick": the pad key and what lies on it.
fn pad_label(p: &pads::Pads, abs: usize) -> String {
    format!(
        "pad {} {}",
        PAD_KEYS[abs % PADS_PER_BANK].to_ascii_uppercase(),
        pad_name(p, abs)
    )
}

/// A pad as bank and key, `2-W`, as the chop screen labels it (27, the dash of 36).
fn bank_key(abs: usize) -> String {
    let key = PAD_KEYS[abs % PADS_PER_BANK].to_ascii_uppercase();
    format!("{}-{key}", abs / PADS_PER_BANK + 1)
}

/// The pad source in the words of an MPC pad layer (27): `all`, `slice 3`, `pad`.
fn source_word(s: pads::Source) -> String {
    match s {
        pads::Source::All => "all".into(),
        pads::Source::Slice(n) => format!("slice {n}"),
        pads::Source::Own => "pad".into(),
    }
}

/// "break 5" for a slice, the file name for a whole file, the test tone note for an empty pad.
fn pad_name(p: &pads::Pads, abs: usize) -> String {
    if let Some(name) = &p.custom_name[abs] {
        return name.clone();
    }
    let Some(stem) = p.file[abs].as_deref().and_then(|f| f.file_stem()) else {
        return test_tone_name(abs);
    };
    let stem = stem.to_string_lossy();
    match p.slice[abs] {
        Some(n) => format!("{stem} {n}"),
        None => stem.into_owned(),
    }
}

/// The chop screen snapshot: positions as fractions of what was opened.
/// The page of the keyboard of slices (36): the bank picked, but never past the last page
/// that has slices — bank 2 of a 4-slice chop shows slices 1–4, not an empty 17–32.
fn slice_page(bank: usize, slices: usize) -> usize {
    bank.min(slices.saturating_sub(1) / PADS_PER_BANK)
}

/// The chop screen shows the real pads of the bank (36): their waves come from the cache of
/// the pads screen, which needs the pads mutable, so they are filled after [`chop_view`].
fn bank_waves(view: &mut tui::Pads, p: &mut pads::Pads, rate: u32) {
    for (i, v) in view.pads.iter_mut().enumerate() {
        v.wave = p.wave(p.bank * PADS_PER_BANK + i, rate, BIG_COLS);
    }
}

#[cfg(test)]
fn chop_view<'a>(
    c: &'a chop::Chop,
    plan: Option<&chop::Plan>,
    playhead: Option<usize>,
    rate: u32,
    p: &pads::Pads,
) -> tui::Chop<'a> {
    chop_view_routed(c, plan, playhead, rate, p, false)
}

fn chop_view_routed<'a>(
    c: &'a chop::Chop,
    plan: Option<&chop::Plan>,
    playhead: Option<usize>,
    rate: u32,
    p: &pads::Pads,
    pattern_pads: bool,
) -> tui::Chop<'a> {
    let (start, end) = (c.view.start, c.view.end);
    let len = (end - start).max(1) as f32;
    let at = |frame: usize| frame.saturating_sub(start) as f32 / len;
    let secs = |frames: usize| frames as f32 / rate.max(1) as f32;
    let slices = c.slices();
    let sel = slices[c.selected.min(slices.len() - 1)];
    let info = format!(
        "slice {} · start {:.3} s · length {:.0} ms · cursor {:.3} s{}",
        c.selected + 1,
        secs(sel.start),
        secs(sel.end - sel.start) * 1000.0,
        secs(c.cursor),
        if slices.len() == 1 {
            "   (Y cuts into equal parts, Space and M cut by ear)"
        } else {
            ""
        }
    );
    let mut bounds: Vec<f32> = vec![at(c.range.start)];
    bounds.extend(c.markers.iter().map(|&m| at(m)));
    bounds.push(at(c.range.end));
    let file = c
        .file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    // before any slice lies on a pad the grid is the keyboard of slices, 16 a bank page (36)
    let laid = (0..PADS)
        .any(|a| p.file[a].as_ref() == Some(&c.file) && p.link[a].is_some_and(|l| l.1 == c.epoch));
    let mut slice_keys = None;
    let (mode, side, banks, pads) = match plan {
        None if !laid && !pattern_pads => {
            let page = slice_page(p.bank, slices.len());
            let pads = (0..PADS_PER_BANK)
                .map(|i| {
                    let n = page * PADS_PER_BANK + i;
                    let slice = slices.get(n);
                    tui::PadView {
                        naming: false,
                        glyphs: String::new(),
                        changed: Vec::new(),
                        dim: slice.is_none(),
                        pitch_gate: tui::PitchGate::NeedsSlice,
                        dashed: false,
                        source: String::new(),
                        bounds: (0.0, 0.0),
                        filter: Default::default(),
                        wave: slice.map_or_else(
                            || vec![0.0],
                            |r| loader::envelope_region(&c.sound, r.start, r.end, BIG_COLS),
                        ),
                        key: PAD_KEYS[i],
                        name: slice.map_or("-".into(), |_| format!("slice {}", n + 1)),
                        gate: false,
                        volume_db: 0.0,
                        pitch: 0,
                        attack_ms: 0,
                        decay_ms: 0,
                        release_ms: 0,
                        group: 0,
                        flash: false,
                        missing: false,
                        loading: None,
                        in_hand: false,
                        marked: false,
                        detail: None,
                        note: Some(slice.map_or(String::new(), |r| {
                            format!("{:.0} ms", secs(r.end - r.start) * 1000.0)
                        })),
                    }
                })
                .collect();
            let first = page * PADS_PER_BANK + 1;
            slice_keys = Some((first, (first + PADS_PER_BANK - 1).min(slices.len())));
            (
                "LISTEN".to_string(),
                vec![
                    "no slice lies on a pad yet".into(),
                    "Ctrl+1…8: more slices".into(),
                    "P: put them on pads".into(),
                ],
                None,
                tui::Pads {
                    pattern: None,
                    overview: None,
                    feedback: tui::PadFeedback::default(),
                    playing: false,
                    bank: page,
                    pads,
                    selected: if c.selected / PADS_PER_BANK == page {
                        c.selected % PADS_PER_BANK
                    } else {
                        usize::MAX
                    },
                    pickup: None,
                    param: tui::PadParam::Volume,
                    scope: String::new(),
                    bank_fill: [0; tui::BANKS],
                    bank_marks: [0; tui::BANKS],
                    bank_pattern: tui::BankPattern::default(),
                    out_wave: Vec::new(),
                    splash: None,
                    big: None,
                    curve: None,
                    learn: Vec::new(),
                    edit: None,
                    pages: Vec::new(),
                    knobs: Vec::new(),
                    lit: None,
                    hidden: false,
                    edit_key: String::new(),
                    fx: Vec::new(),
                    slot: None,
                },
            )
        }
        None => {
            // the real pads of the bank, as on the pads screen (36): a pad of this chop names
            // its slice, the rest steps back
            let ours = |abs: usize| {
                p.file[abs].as_ref() == Some(&c.file) && p.link[abs].is_some_and(|l| l.1 == c.epoch)
            };
            // the pad that plays the whole file: the source, marked ◆ (37)
            let whole = |abs: usize| {
                p.file[abs].as_ref() == Some(&c.file) && p.source(abs) == pads::Source::All
            };
            let mut marks = [(false, false); tui::BANKS];
            for (b, m) in marks.iter_mut().enumerate() {
                let bank = b * PADS_PER_BANK..(b + 1) * PADS_PER_BANK;
                m.0 = bank.clone().any(|a| p.file[a].is_some());
                m.1 = bank.into_iter().any(ours);
            }
            let pads = (0..PADS_PER_BANK)
                .map(|i| {
                    let abs = p.bank * PADS_PER_BANK + i;
                    tui::PadView {
                        naming: false,
                        glyphs: String::new(),
                        changed: Vec::new(),
                        dim: !pattern_pads && !ours(abs) && !whole(abs),
                        pitch_gate: view::pitch_gate(p, abs),
                        // the source stands apart from its slices (46)
                        dashed: whole(abs),
                        source: String::new(),
                        bounds: (0.0, 0.0),
                        filter: Default::default(),
                        wave: Vec::new(),
                        key: PAD_KEYS[i],
                        name: if p.file[abs].is_some() {
                            pad_name(p, abs)
                        } else {
                            "empty".into()
                        },
                        gate: p.gate[abs],
                        volume_db: p.volume_db[abs] as f32,
                        pitch: p.pitch[abs],
                        attack_ms: p.attack_ms[abs],
                        decay_ms: p.decay_ms[abs],
                        release_ms: p.release_ms[abs],
                        group: p.group[abs],
                        flash: false,
                        missing: p.missing[abs],
                        loading: p.loading[abs],
                        in_hand: false,
                        marked: false,
                        detail: None,
                        note: if ours(abs) {
                            p.slice[abs].map(|n| format!("slice {n}"))
                        } else if whole(abs) {
                            Some("◆ all".into())
                        } else if p.file[abs].is_none() {
                            Some(String::new())
                        } else {
                            None
                        },
                    }
                })
                .collect();
            let banks_with: Vec<String> = marks
                .iter()
                .enumerate()
                .filter(|(_, m)| m.1)
                .map(|(b, _)| (b + 1).to_string())
                .collect();
            let first = match banks_with.len() {
                0 => "no slice lies on a pad yet".to_string(),
                1 => format!("slices lie on bank {}", banks_with[0]),
                _ => format!("slices lie on banks {}", banks_with.join(", ")),
            };
            (
                "LISTEN".to_string(),
                vec![
                    if pattern_pads {
                        "A1: real pads".into()
                    } else {
                        first
                    },
                    "Ctrl+1…8: banks".into(),
                    if pattern_pads {
                        "end pattern work to assign".into()
                    } else {
                        "P: put them on pads".into()
                    },
                ],
                Some(marks),
                tui::Pads {
                    pattern: None,
                    overview: None,
                    playing: false,
                    bank: p.bank,
                    pads,
                    feedback: tui::PadFeedback::default(),
                    // the pad of the selected slice, when it is in this bank
                    selected: if pattern_pads {
                        p.selected
                    } else {
                        (0..PADS_PER_BANK)
                            .find(|&i| {
                                let abs = p.bank * PADS_PER_BANK + i;
                                ours(abs) && p.slice[abs] == Some(c.selected as u16 + 1)
                            })
                            .unwrap_or(usize::MAX)
                    },
                    pickup: None,
                    param: tui::PadParam::Volume,
                    scope: String::new(),
                    bank_fill: bank_fill(p),
                    bank_marks: bank_marks(p),
                    bank_pattern: tui::BankPattern::default(),
                    out_wave: Vec::new(),
                    splash: None,
                    big: None,
                    curve: None,
                    learn: Vec::new(),
                    edit: None,
                    pages: Vec::new(),
                    knobs: Vec::new(),
                    lit: None,
                    hidden: false,
                    edit_key: String::new(),
                    fx: Vec::new(),
                    slot: None,
                },
            )
        }
        Some(plan) => {
            let targets = plan.targets(slices.len());
            let taken = |abs: usize| p.file[abs].is_some();
            let mut marks = [(false, false); tui::BANKS];
            for (b, m) in marks.iter_mut().enumerate() {
                m.0 = (b * PADS_PER_BANK..(b + 1) * PADS_PER_BANK).any(taken);
            }
            for &(_, abs) in &targets {
                marks[abs / PADS_PER_BANK].1 = true;
            }
            let pads = (0..PADS_PER_BANK)
                .map(|i| {
                    let abs = p.bank * PADS_PER_BANK + i;
                    let planned = targets.iter().find(|t| t.1 == abs).map(|t| t.0);
                    tui::PadView {
                        pitch_gate: view::pitch_gate(p, abs),
                        naming: false,
                        glyphs: String::new(),
                        changed: Vec::new(),
                        dim: false,
                        dashed: false,
                        source: String::new(),
                        bounds: (0.0, 0.0),
                        filter: Default::default(),
                        wave: Vec::new(),
                        key: PAD_KEYS[i],
                        name: if taken(abs) {
                            pad_name(p, abs)
                        } else {
                            "empty".into()
                        },
                        gate: p.gate[abs],
                        volume_db: p.volume_db[abs] as f32,
                        pitch: p.pitch[abs],
                        attack_ms: p.attack_ms[abs],
                        decay_ms: p.decay_ms[abs],
                        release_ms: p.release_ms[abs],
                        group: p.group[abs],
                        flash: false,
                        missing: p.missing[abs],
                        loading: p.loading[abs],
                        // pink: something lies there and the plan replaces it
                        in_hand: planned.is_some() && taken(abs),
                        marked: false,
                        detail: None,
                        note: match planned {
                            Some(s) => Some(format!("→ {}", s + 1)),
                            // an empty pad: no mode line, "empty" says it all
                            None if !taken(abs) => Some(String::new()),
                            None => None,
                        },
                    }
                })
                .collect();
            let by_hand = matches!(plan, chop::Plan::ByHand(_));
            let summary = match chop::summary(&targets, slices.len(), taken) {
                Some(s) => format!(
                    "{} slice{} → {s}",
                    targets.len(),
                    if targets.len() == 1 { "" } else { "s" }
                ),
                None => "nothing planned yet".into(),
            };
            let (mode, how) = match plan {
                chop::Plan::ByHand(_) => (
                    "ASSIGN · BY HAND",
                    "J L pick the slice, a pad hit places it",
                ),
                chop::Plan::InOrder(None) => ("ASSIGN · IN ORDER", "hit the pad to start from"),
                chop::Plan::InOrder(Some(_)) => {
                    ("ASSIGN · IN ORDER", "another hit moves the start")
                }
            };
            let list = chop::overwrites(&targets, |a| taken(a).then(|| pad_label(p, a)));
            let mut side = vec![summary, how.into()];
            if !list.is_empty() {
                side.push(format!("overwrites {list}"));
            }
            side.push(
                if by_hand {
                    "Tab: all in order"
                } else {
                    "Tab: back to by hand"
                }
                .into(),
            );
            side.push("Enter: apply · Esc: back".into());
            (
                mode.to_string(),
                side,
                Some(marks),
                tui::Pads {
                    pattern: None,
                    overview: None,
                    feedback: tui::PadFeedback::default(),
                    playing: false,
                    bank: p.bank,
                    pads,
                    selected: usize::MAX,
                    pickup: None,
                    param: tui::PadParam::Volume,
                    scope: String::new(),
                    out_wave: Vec::new(),
                    splash: None,
                    big: None,
                    curve: None,
                    learn: Vec::new(),
                    bank_fill: bank_fill(p),
                    bank_marks: bank_marks(p),
                    bank_pattern: tui::BankPattern::default(),
                    edit: None,
                    pages: Vec::new(),
                    knobs: Vec::new(),
                    lit: None,
                    hidden: false,
                    edit_key: String::new(),
                    fx: Vec::new(),
                    slot: None,
                },
            )
        }
    };
    // the pads that follow each slice of this chop, as bank and key (27), and the bank of
    // the first one for the mark (36)
    let mut owners = vec![Vec::new(); c.slices().len()];
    let mut owner_bank = vec![None; c.slices().len()];
    for abs in 0..PADS {
        if p.file[abs].as_ref() == Some(&c.file)
            && p.link[abs].is_some_and(|l| l.1 == c.epoch)
            && let Some(n) = p.slice[abs]
            && let Some(o) = owners.get_mut(n as usize - 1)
        {
            o.push(bank_key(abs));
            owner_bank[n as usize - 1].get_or_insert(abs / PADS_PER_BANK);
        }
    }
    // the slice line names where the selected slice lies, also when its label does not fit
    // at the slice number (04e)
    let info = match owners.get(c.selected).filter(|o| !o.is_empty()) {
        Some(o) => format!("{info} · on {}", o.join(", ")),
        None => info,
    };
    let mut source_banks = [false; tui::BANKS];
    for abs in 0..PADS {
        if p.file[abs].as_ref() == Some(&c.file) && p.source(abs) == pads::Source::All {
            source_banks[abs / PADS_PER_BANK] = true;
        }
    }
    // the band levels of the blocks the view covers: the wave colors (38)
    let blocks = (start / chop::BAND_BLOCK).min(c.bands.len())
        ..end.div_ceil(chop::BAND_BLOCK).min(c.bands.len());
    tui::Chop {
        owners: owners.into_iter().map(|o| o.join(",")).collect(),
        owner_bank,
        source_banks,
        slice_keys,
        bands: &c.bands[blocks],
        band_mean: c.band_mean,
        file,
        // the listen header shows that the chop screen sounds (68)
        mode: if playhead.is_some() && plan.is_none() {
            format!("{mode} ▶")
        } else {
            mode
        },
        seconds: secs(c.range.end - c.range.start),
        which: None,
        // the hits mode marks where the cursor stops and Ctrl+Y cuts (51)
        hits: if c.snap == chop::Snap::Hits {
            c.strong_hits()
                .into_iter()
                .filter(|&h| h >= start && h < end)
                .map(at)
                .collect()
        } else {
            Vec::new()
        },
        wave: &c.sound.left[start..end],
        bounds,
        selected: c.selected,
        cursor: Some(c.cursor).filter(|p| (start..end).contains(p)).map(at),
        play: {
            let r = c.play_region();
            Some((at(r.start), at(r.end).min(1.0)))
        },
        playhead: playhead.filter(|p| (start..end).contains(p)).map(at),
        info,
        bounds_slice: c.selected + 1,
        edit_rows: [
            format!("{:.3} s", secs(sel.start)),
            format!("{:.3} s", secs(sel.end)),
            format!("{:.0} ms", secs(sel.end - sel.start) * 1000.0),
            format!("{:.3} s", secs(c.cursor)),
        ],
        side,
        banks,
        pads,
        edit: false,
        hot: None,
    }
}

fn tui_param(p: pads::Param) -> tui::PadParam {
    match p {
        pads::Param::Volume => tui::PadParam::Volume,
        pads::Param::Pitch => tui::PadParam::Pitch,
        pads::Param::Group => tui::PadParam::Group,
        pads::Param::Attack => tui::PadParam::Attack,
        pads::Param::Decay => tui::PadParam::Decay,
        pads::Param::Release => tui::PadParam::Release,
        pads::Param::Source => tui::PadParam::Source,
        pads::Param::Start => tui::PadParam::Start,
        pads::Param::End => tui::PadParam::End,
        pads::Param::Filter => tui::PadParam::Filter,
        pads::Param::Cutoff => tui::PadParam::Cutoff,
        pads::Param::Reso => tui::PadParam::Reso,
        pads::Param::Drive => tui::PadParam::Drive,
        pads::Param::Send1 => tui::PadParam::Send1,
        pads::Param::Send2 => tui::PadParam::Send2,
        pads::Param::Fx1Kind => tui::PadParam::Fx1Kind,
        pads::Param::Fx1A => tui::PadParam::Fx1A,
        pads::Param::Fx1B => tui::PadParam::Fx1B,
        pads::Param::Fx2Kind => tui::PadParam::Fx2Kind,
        pads::Param::Fx2A => tui::PadParam::Fx2A,
        pads::Param::Fx2B => tui::PadParam::Fx2B,
    }
}

/// A panel label padded to the width of the other rows ("cutoff  ").
fn label8(name: &str) -> &'static str {
    match name {
        "drive" => "drive   ",
        "send 1" => "send 1  ",
        "send 2" => "send 2  ",
        "type" => "type    ",
        "size" => "size    ",
        "tone" => "tone    ",
        "time" => "time    ",
        _ => "feedback",
    }
}

// the screen and the engine count the same banks
const _: () = assert!(tui::BANKS == engine::BANKS);

/// The parameters of the FX pages in the order of `Param::PadEffect` (56).
const EFFECT_PARAMS: [pads::Param; 10] = [
    pads::Param::Drive,
    pads::Param::Send1,
    pads::Param::Send2,
    pads::Param::Fx1Kind,
    pads::Param::Fx1A,
    pads::Param::Fx1B,
    pads::Param::Fx2Kind,
    pads::Param::Fx2A,
    pads::Param::Fx2B,
    // the mute group (70) rides the same knob path: knob 3 of TONE
    pads::Param::Group,
];

fn keys(keymap: &Keymap, screen: Screen, actions: &[Action]) -> String {
    let all: Vec<String> = actions
        .iter()
        .flat_map(|a| keymap.keys_for(screen, *a))
        .collect();
    letters_first(all).join(" ")
}

/// The help shows the letters the hands rest on; the arrows and PgUp PgDn that do the
/// same job are left for F1. Keys without a letter twin stay as they are.
fn letters_first(all: Vec<String>) -> Vec<String> {
    let bare = |k: &str| k.rsplit('+').next().unwrap_or(k).to_string();
    let twin = |k: &str| matches!(bare(k).as_str(), "←" | "→" | "↑" | "↓" | "PgUp" | "PgDn");
    let letter = |k: &str| {
        let b = bare(k);
        b.len() == 1 && b.chars().all(|c| c.is_ascii_alphabetic())
    };
    if all.iter().any(|k| letter(k)) {
        all.into_iter().filter(|k| !twin(k)).collect()
    } else {
        all
    }
}

/// "Ctrl J L": a chord the left thumb holds while the right hand presses, written without
/// "+" so the usual help line keeps it as a hint.
/// The panel line of a pad in hand (35): short enough for the panel; the help line says
/// what to do with it.
fn in_hand_line(p: &pads::Pads, abs: usize) -> String {
    let name = if p.file[abs].is_some() {
        pad_name(p, abs)
    } else {
        "empty".into()
    };
    format!(
        "in hand: pad {} · {name}",
        PAD_KEYS[abs % PADS_PER_BANK].to_ascii_uppercase()
    )
}

/// Without kitty a held modifier is not seen, so the hold hints of the help line read as
/// the combinations they are, like every other one: `Ctrl Z` → `Ctrl+Z` (35).
fn as_combos(mut groups: Vec<tui::HelpGroup>) -> Vec<tui::HelpGroup> {
    for g in &mut groups {
        for (keys, _) in &mut g.items {
            for m in ["Ctrl ", "Alt "] {
                if let Some(rest) = keys.strip_prefix(m) {
                    *keys = format!("{}+{rest}", m.trim_end());
                }
            }
        }
    }
    groups
}

fn hold_keys(all: Vec<String>) -> String {
    let all = letters_first(all);
    let Some((modifier, _)) = all.first().and_then(|k| k.split_once('+')) else {
        return all.join(" ");
    };
    let bare: Vec<&str> = all
        .iter()
        .filter_map(|k| k.split_once('+').map(|(_, b)| b))
        .collect();
    format!("{modifier} {}", bare.join(" "))
}

/// What the help line shows about the current states of the toggles.
struct HelpState {
    hand: Option<&'static str>,
    preview_on: bool,
    assign: bool,
    snap: chop::Snap,
    tab: usize,
    /// The parameter `Alt+←` `Alt+→` change right now: of the pad, or the slice bound.
    param_name: &'static str,
    /// The page of the edit mode: TONE, ENV, BOUNDS.
    page: &'static str,
    /// The play mode of the chop screen.
    play: &'static str,
    /// The chop loop is on (49).
    looping: bool,
    /// The chop screen plays as the pad (50).
    as_pad: bool,
    /// Marking is on: pad hits mark pads for the change scope.
    marking: bool,
    /// The change scope (`Ctrl+A`), named in the help line of the pads screen (68).
    scope: String,
    /// The edit mode of the selected object is open.
    edit: bool,
}

fn transport_help(keymap: &Keymap, screen: Screen) -> tui::HelpGroup {
    let item = |actions: &[Action], text: &str| {
        let keys = keymap
            .bindings(screen)
            .into_iter()
            .filter(|(_, action)| actions.contains(action))
            .map(|(key, _)| key)
            .collect();
        (letters_first(keys).join(" "), text.to_string())
    };
    tui::HelpGroup {
        name: "CLOCK",
        items: vec![
            item(&[Action::TransportToggle], "play/stop"),
            item(
                &[
                    Action::TempoStep {
                        up: false,
                        fine: false,
                    },
                    Action::TempoStep {
                        up: true,
                        fine: false,
                    },
                ],
                "BPM",
            ),
            item(
                &[
                    Action::TempoStep {
                        up: false,
                        fine: true,
                    },
                    Action::TempoStep {
                        up: true,
                        fine: true,
                    },
                ],
                "0.1 BPM",
            ),
            item(&[Action::MetronomeToggle], "metro"),
        ]
        .into_iter()
        .filter(|(key, _)| !key.is_empty())
        .collect(),
    }
}

fn help_groups(keymap: &Keymap, screen: Screen, st: &HelpState) -> Vec<tui::HelpGroup> {
    let item = |actions: &[Action], text: String| (keys(keymap, screen, actions), text);
    // the edit layer keys: the same on every screen, over the screen keys
    let edit_keys = |actions: &[Action]| {
        let all: Vec<String> = actions
            .iter()
            .flat_map(|a| keymap.edit_keys_for(*a))
            .collect();
        letters_first(all).join(" ")
    };
    let bank = || tui::HelpGroup {
        name: "BANK",
        items: vec![
            item(&[Action::BankPrev, Action::BankNext], "bank".into()),
            (
                format!("{}…8", keys(keymap, screen, &[Action::BankSelect(0)])),
                "bank 1-8".into(),
            ),
        ],
    };
    if st.edit && screen.rules().ui_edit_group.is_some() {
        // the edit line shows only the edit keys and the way out
        // a slice has one page (BOUNDS): no page keys to offer there (54)
        let walk = if screen == Screen::Chop {
            vec![(
                edit_keys(&[Action::ParamPrev, Action::ParamNext]),
                st.page.into(),
            )]
        } else {
            vec![
                (
                    edit_keys(&[Action::PageNext, Action::PagePrev]),
                    st.page.into(),
                ),
                (
                    edit_keys(&[Action::ParamPrev, Action::ParamNext]),
                    "parameter".into(),
                ),
            ]
        };
        return vec![tui::HelpGroup {
            name: "EDIT",
            items: walk
                .into_iter()
                .chain([
                    (
                        edit_keys(&[
                            Action::EditValue {
                                up: false,
                                fine: false,
                            },
                            Action::EditValue {
                                up: true,
                                fine: false,
                            },
                        ]),
                        st.param_name.into(),
                    ),
                    ("pad".into(), "listen".into()),
                ])
                // the pad edit only: the clear key resets the settings there (76)
                .chain((screen == Screen::Pads).then(|| {
                    (
                        keys(keymap, screen, &[Action::PadClear]),
                        "reset".to_string(),
                    )
                }))
                .chain([(
                    format!(
                        "{} {}",
                        edit_keys(&[Action::Back]),
                        keys(keymap, screen, &[Action::Back])
                    ),
                    "close".into(),
                )])
                .collect(),
        }];
    }
    let mut groups = match screen {
        Screen::Files | Screen::Pads if st.hand.is_some() => vec![
            tui::HelpGroup {
                name: "IN HAND",
                items: if screen == Screen::Files {
                    vec![
                        ("1-4 Q-R A-F Z-V".into(), st.hand.unwrap_or_default().into()),
                        item(&[Action::BrowserHold], "on the selected pad".into()),
                        item(&[Action::Back], "put back".into()),
                    ]
                } else {
                    vec![
                        ("1-4 Q-R A-F Z-V".into(), st.hand.unwrap_or_default().into()),
                        item(&[Action::Back], "put back".into()),
                    ]
                },
            },
            bank(),
        ],
        Screen::Fx => vec![
            tui::HelpGroup {
                name: "SHARED FX",
                items: vec![
                    item(
                        &[Action::PagePrev, Action::PageNext],
                        format!("{} selected", st.page),
                    ),
                    item(&[Action::ParamPrev, Action::ParamNext], "parameter".into()),
                    item(
                        &[
                            Action::EditValue {
                                up: false,
                                fine: false,
                            },
                            Action::EditValue {
                                up: true,
                                fine: false,
                            },
                        ],
                        st.param_name.into(),
                    ),
                    item(&[Action::Undo], "undo".into()),
                    ("pad".into(), "play".into()),
                ],
            },
            bank(),
        ],
        Screen::Pads => vec![
            tui::HelpGroup {
                name: "PAD",
                // the pad panel shows the mode and the picked parameter: the help line names only the keys
                items: vec![
                    item(&[Action::PadEdit], "edit".into()),
                    item(&[Action::PadRename], "name".into()),
                    item(&[Action::PadModeNext], "mode".into()),
                    item(&[Action::PadClear], "clear".into()),
                ],
            },
            tui::HelpGroup {
                name: "CHANGE",
                items: vec![
                    // the scope rides on the mark key: a kitty terminal hides the Ctrl+A item (68)
                    item(
                        &[Action::PadMarkToggle],
                        format!(
                            "{} · scope: {}",
                            if st.marking {
                                "mark: on, hit pads"
                            } else {
                                "mark: off"
                            },
                            st.scope
                        ),
                    ),
                    item(&[Action::PadMarkBank], "mark bank".into()),
                ],
            },
            bank(),
        ],
        Screen::Files => vec![
            tui::HelpGroup {
                name: "FILES",
                items: vec![
                    item(&[Action::BrowserUp, Action::BrowserDown], "select".into()),
                    item(&[Action::BrowserLoad], "load / open".into()),
                    item(&[Action::BrowserParent], "up".into()),
                    item(&[Action::BrowserHold], "take in hand".into()),
                    item(
                        &[Action::BrowserPreviewToggle],
                        format!("preview: {}", if st.preview_on { "on" } else { "off" }),
                    ),
                    item(&[Action::BrowserFilter], "filter".into()),
                    item(&[Action::BrowserFavorite], "favorite".into()),
                ],
            },
            bank(),
        ],
        Screen::Chop if st.assign => vec![
            tui::HelpGroup {
                name: "ASSIGN",
                items: vec![
                    ("1-4 Q-R A-F Z-V".into(), "pad".into()),
                    item(
                        &[
                            Action::CursorMove {
                                right: false,
                                fine: false,
                            },
                            Action::CursorMove {
                                right: true,
                                fine: false,
                            },
                        ],
                        "slice".into(),
                    ),
                    item(
                        &[Action::ParamNext, Action::ChopAssignOrder],
                        "in order / by hand".into(),
                    ),
                    item(&[Action::ChopEdit, Action::ChopCommit], "apply".into()),
                    item(&[Action::Back], "back".into()),
                    item(&[Action::Undo], "undo".into()),
                ],
            },
            bank(),
        ],
        Screen::Chop => {
            let hold = |end: bool| {
                let all = [false, true].map(|right| {
                    keymap.keys_for(
                        screen,
                        Action::SliceBound {
                            end,
                            right,
                            fine: false,
                        },
                    )
                });
                hold_keys(all.concat())
            };
            vec![
                tui::HelpGroup {
                    name: "MOVE",
                    items: vec![
                        item(
                            &[
                                Action::CursorMove {
                                    right: false,
                                    fine: false,
                                },
                                Action::CursorMove {
                                    right: true,
                                    fine: false,
                                },
                            ],
                            "cursor".into(),
                        ),
                        // Modified zoom aliases live in modifier help and F1, not this short row.
                        (
                            keys(
                                keymap,
                                screen,
                                &[
                                    Action::ChopZoom { zoom_in: true },
                                    Action::ChopZoom { zoom_in: false },
                                ],
                            )
                            .split_whitespace()
                            .filter(|key| !key.contains('+'))
                            .collect::<Vec<_>>()
                            .join(" "),
                            "zoom".into(),
                        ),
                        item(&[Action::ChopPlay], "play".into()),
                        item(
                            &[Action::ChopPlayMode],
                            // the loop's state stands by the play mode: its Alt key hides
                            // under kitty until Alt is held (49)
                            format!(
                                "{}{}{}",
                                st.play,
                                if st.looping { " · loop: on" } else { "" },
                                if st.as_pad { " · as pad" } else { "" }
                            ),
                        ),
                        item(
                            &[Action::ChopSnapToggle],
                            format!("snap: {}", st.snap.name()),
                        ),
                    ],
                },
                tui::HelpGroup {
                    name: "CUT",
                    items: vec![
                        item(&[Action::ChopCut], "cut".into()),
                        // only the plain key: a "Ctrl+" in it would hide the item under kitty
                        (
                            keys(keymap, screen, &[Action::MarkerDelete])
                                .split(' ')
                                .filter(|k| !k.contains('+'))
                                .collect::<Vec<_>>()
                                .join(" "),
                            "remove cut".into(),
                        ),
                        item(
                            &[Action::ChopRegionStart, Action::ChopRegionEnd],
                            "region".into(),
                        ),
                        // the ways back stay in sight: a hold hint has no "+", so the
                        // usual line keeps it
                        (
                            hold_keys(keymap.keys_for(screen, Action::ChopRegionClear)),
                            "whole".into(),
                        ),
                        item(
                            &[Action::ChopEqualParts, Action::ChopHits],
                            // one item: 83a added H, and the line must fit the band
                            "parts/hits".into(),
                        ),
                        // 83a: Shift+H; the kitty view drops the combo item (it waits for
                        // its modifier), F1 keeps the row
                        item(&[Action::ChopHitsBack], "hits back".into()),
                        (
                            hold_keys(keymap.keys_for(screen, Action::Undo)),
                            "undo".into(),
                        ),
                    ],
                },
                tui::HelpGroup {
                    name: "PADS",
                    items: vec![
                        item(&[Action::ChopAssign], "to pads".into()),
                        item(&[Action::ChopEdit], "edit".into()),
                    ],
                },
                tui::HelpGroup {
                    name: "BOUNDS",
                    items: vec![(hold(false), "left".into()), (hold(true), "right".into())],
                },
                // the banks turn on the chop screen too (43)
                bank(),
            ]
        }
        Screen::Settings => {
            let row = item(
                &[Action::SettingsRow(-1), Action::SettingsRow(1)],
                "row".into(),
            );
            let tabs = item(
                &[Action::SettingsTab],
                "audio / input / MIDI / paths".into(),
            );
            let back = item(&[Action::Back], "back".into());
            let items = match st.tab {
                0 => vec![
                    row,
                    item(
                        &[Action::SettingsValue(-1), Action::SettingsValue(1)],
                        "change".into(),
                    ),
                    tabs,
                    back,
                ],
                1 => vec![tabs, back],
                // the Paths tab (11): two rows, Enter opens the folder picker of one
                3 => vec![
                    row,
                    item(&[Action::SettingsLearn], "open".into()),
                    tabs,
                    back,
                ],
                _ => vec![
                    item(&[Action::SettingsLearn], "learn".into()),
                    item(&[Action::SettingsClear], "clear".into()),
                    row,
                    item(
                        &[Action::SettingsValue(-1), Action::SettingsValue(1)],
                        "open / close".into(),
                    ),
                    item(&[Action::BankPrev, Action::BankNext], "groups".into()),
                    tabs,
                    back,
                ],
            };
            vec![tui::HelpGroup {
                name: "SETTINGS",
                items,
            }]
        }
    };
    groups.push(transport_help(keymap, screen));
    groups.push(tui::HelpGroup {
        name: "REC",
        items: vec![
            item(&[Action::RecordMic(true)], "mic: hold".into()),
            item(&[Action::RecordOutputToggle], "output".into()),
        ],
    });
    groups.push(tui::HelpGroup {
        name: "",
        items: vec![
            item(&[Action::HelpToggle], "help".into()),
            item(&[Action::ScreenChop], "chop".into()),
            item(&[Action::ScreenFx], "FX".into()),
            item(&[Action::ScreenPattern], "pattern".into()),
            item(&[Action::ScreenFiles], "files".into()),
            item(&[Action::ScreenSettings], "settings".into()),
        ],
    });
    groups
}

/// The keys of the project picker for the help line (35): the list, or the name line.
fn picker_help(naming: bool) -> Vec<tui::HelpGroup> {
    let items = if naming {
        vec![
            ("a-z".into(), "name".into()),
            ("Enter".into(), "create".into()),
            ("Esc".into(), "back to the list".into()),
        ]
    } else {
        vec![
            ("↑ ↓".into(), "select".into()),
            ("Enter".into(), "open".into()),
            ("a-z".into(), "type a name to create".into()),
            ("Esc".into(), "close".into()),
        ]
    };
    vec![tui::HelpGroup {
        name: "PROJECTS",
        items,
    }]
}

/// The pads switch with its state, for the Ctrl view of the help line (32): the usual
/// line of the chop screen has no room left for it without kitty, F1 lists it there.
fn pads_state(hidden: bool) -> String {
    format!("pads: {}", if hidden { "hidden" } else { "shown" })
}

/// The usual help line where a held Ctrl can be seen: the Ctrl combos show only while it
/// is held, so this line keeps the keys without modifiers.
fn without_modifiers(mut groups: Vec<tui::HelpGroup>) -> Vec<tui::HelpGroup> {
    for g in &mut groups {
        // a combo waits for its modifier; a hold hint ("Ctrl J L") has no "+" and stays
        g.items
            .retain(|(k, _)| !k.split(' ').any(|t| t.contains('+') && t.len() > 1));
    }
    groups.retain(|g| !g.items.is_empty());
    groups
}

/// The help line while a modifier is held: only the keys that work with it.
fn modifier_groups(
    keymap: &Keymap,
    screen: Screen,
    m: input::Modifier,
    pads_hidden: bool,
) -> Vec<tui::HelpGroup> {
    // Page-only controls stay in F1's table and footer, not ordinary modifier help.
    let bindings = keymap
        .bindings(screen)
        .into_iter()
        .filter(|(_, action)| !matches!(action, Action::HelpPagePrev | Action::HelpPageNext));
    if m == input::Modifier::Shift {
        // 83a: a Shift command (as Shift+H on Chop) joins the full-velocity pads;
        // its text names what the shifted key does, not the unshifted command
        let mut items = vec![("1-4 Q-R A-F Z-V".into(), "pads at full velocity".into())];
        for (k, a) in bindings {
            let Some(rest) = k.strip_prefix("Shift+") else {
                continue;
            };
            // a Shift chord of the pad block stays a full-velocity pad, not a command
            if rest.chars().count() == 1 && "1234QWERASDFZXCV".contains(rest) {
                continue;
            }
            items.push((k.clone(), shift_text(a)));
        }
        items.sort();
        items.dedup();
        return vec![tui::HelpGroup {
            name: "SHIFT",
            items,
        }];
    }
    let (prefix, name) = match m {
        input::Modifier::Alt => ("Alt+", "ALT"),
        _ => ("Ctrl+", "CTRL"),
    };
    let mut items: Vec<(String, String)> = Vec::new();
    for (k, a) in bindings {
        let Some(key) = k.strip_prefix(prefix) else {
            continue;
        };
        let (key, text) = match a {
            Action::BankSelect(0) => ("1…8".to_string(), "bank".to_string()),
            Action::BankSelect(_) => continue,
            Action::PadsToggle => (key.to_string(), pads_state(pads_hidden)),
            a => (key.to_string(), ctrl_text(a)),
        };
        // keys of one kind (fine nudge left and right) share one item
        match items.iter_mut().find(|(_, t)| *t == text) {
            Some((keys, _)) => {
                keys.push(' ');
                keys.push_str(&key);
            }
            None => items.push((key, text)),
        }
    }
    // the letters the hands rest on; the arrow twins are left for F1
    for (keys, _) in &mut items {
        *keys = letters_first(keys.split(' ').map(String::from).collect()).join(" ");
    }
    vec![tui::HelpGroup { name, items }]
}

/// Short texts for the Ctrl view of the help line.
/// The text of a Shift command in the Shift view of the help line (83a): what the
/// shifted key does, not the unshifted command.
fn shift_text(a: Action) -> String {
    match a {
        Action::ChopHitsBack => "fewer hit cuts".into(),
        a => ctrl_text(a),
    }
}

fn ctrl_text(a: Action) -> String {
    match a {
        Action::RecordMic(_) => "mic: hold".into(),
        Action::RecordOutputToggle => "output".into(),
        Action::PatternRecord => "pattern record".into(),
        Action::PatternErase => "pattern erase".into(),
        Action::PatternUndo => "pattern undo".into(),
        Action::PatternControls => "pattern controls".into(),
        Action::PadTake => "take pad".into(),
        Action::PadCopy => "copy pad".into(),
        Action::Undo => "undo".into(),
        Action::MarkerNudge { fine: true, .. } => "nudge fine".into(),
        Action::MarkerNudge { fine: false, .. } => "slice start".into(),
        Action::ChopRegionClear => "region all".into(),
        Action::SliceBound { end: false, .. } => "left bound".into(),
        Action::SliceBound { end: true, .. } => "right bound".into(),
        Action::MarkerDelete => "delete left".into(),
        Action::SliceEndDelete => "delete right".into(),
        Action::ChopLoopToggle => "loop".into(),
        Action::ChopAsPadToggle => "as pad".into(),
        Action::PadParam(_) | Action::EditValue { fine: false, .. } => "value".into(),
        Action::EditValue { fine: true, .. } => "fine".into(),
        Action::ChopZoom { zoom_in: true } => "zoom in".into(),
        Action::ChopZoom { zoom_in: false } => "zoom out".into(),
        Action::BrowserFavorite => "favorite".into(),
        Action::MetronomeToggle => "metro".into(),
        a => describe(a),
    }
}

/// Chop-screen actions the assign mode swallows (21): a cut, a zoom or a cursor move
/// while the owner lays slices onto pads would ruin the wave under the plan. The pads,
/// Tab, Enter, Esc, the slice pick and the undo stay live.
fn assign_deaf(a: &Action) -> bool {
    matches!(
        a,
        Action::ChopPlay
            | Action::ChopPlayMode
            | Action::ChopCut
            | Action::ChopEqualParts
            | Action::ChopHits
            | Action::ChopHitsBack
            | Action::ChopAssign
            | Action::ChopZoom { .. }
            | Action::ChopRegionStart
            | Action::ChopRegionEnd
            | Action::ChopRegionClear
            | Action::ChopSnapToggle
            | Action::CursorMove { .. }
            | Action::CursorJump { .. }
            | Action::MarkerNudge { .. }
            | Action::EditValue { .. }
            | Action::SliceBound { .. }
            | Action::ParamPrev
            | Action::MarkerDelete
            | Action::SliceEndDelete
    )
}

fn describe(a: Action) -> String {
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
            "A1 controls. Quantize and Strength set the next hits, not saved hits.".into()
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
        Action::PadsToggle => "hide / show the pads".into(),
        Action::Back => "back".into(),
        Action::Quit => "quit".into(),
        Action::SessionSave => "save the session now".into(),
        Action::Click => "test click".into(),
        Action::TransportToggle => "transport: play / stop".into(),
        Action::MetronomeToggle => "metronome: on / off".into(),
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

/// The heading an action is listed under in the full help, in the order of the headings.
fn help_section(a: Action) -> usize {
    match a {
        Action::PadPress { .. }
        | Action::PadRelease { .. }
        | Action::PadSelect(_)
        | Action::PadModeNext
        | Action::PadPitchGateToggle
        | Action::PadEdit
        | Action::PadEditOnly
        | Action::PadParamNext
        | Action::PadParam(_)
        | Action::PadMarkBank
        | Action::PadMarkToggle
        | Action::PadClear
        | Action::PadTake
        | Action::PadCopy
        | Action::PadRename
        | Action::LearnToggle => 0,
        Action::BankNext | Action::BankPrev | Action::BankSelect(_) => 1,
        Action::BrowserUp
        | Action::BrowserDown
        | Action::BrowserEnter
        | Action::BrowserParent
        | Action::BrowserHold
        | Action::BrowserTakeOnly
        | Action::BrowserLoad
        | Action::BrowserPreviewToggle
        | Action::BrowserFilter
        | Action::BrowserFavorite => 2,
        Action::ChopPlay
        | Action::ChopPlayMode
        | Action::ChopLoopToggle
        | Action::ChopAsPadToggle
        | Action::CursorJump { .. }
        | Action::ChopCut
        | Action::ChopParts
        | Action::ChopEqualParts
        | Action::ChopHits
        | Action::ChopHitsBack
        | Action::ChopAssignOrder
        | Action::SliceBound { .. }
        | Action::ParamNext
        | Action::ParamPrev
        | Action::PageNext
        | Action::PagePrev
        | Action::EditValue { .. }
        | Action::ChopCommit
        | Action::ChopEdit
        | Action::ChopEditOnly
        | Action::ChopAssign
        | Action::ChopZoom { .. }
        | Action::ChopRegionStart
        | Action::ChopRegionEnd
        | Action::ChopRegionClear
        | Action::ChopSnapToggle
        | Action::ChopNormalize
        | Action::CursorMove { .. }
        | Action::MarkerSelect(_)
        | Action::MarkerNudge { .. }
        | Action::MarkerDelete
        | Action::SliceEndDelete
        | Action::Undo => 3,
        Action::SettingsRow(_)
        | Action::SettingsValue(_)
        | Action::SettingsTab
        | Action::SettingsLearn
        | Action::SettingsClear => 4,
        Action::MasterVolume(_)
        | Action::Click
        | Action::TransportToggle
        | Action::MetronomeToggle
        | Action::TempoStep { .. } => 5,
        Action::RecordMic(_)
        | Action::RecordOutputToggle
        | Action::PatternRecord
        | Action::PatternErase
        | Action::PatternUndo
        | Action::PatternControls
        | Action::PatternParamPrev
        | Action::PatternParamNext
        | Action::PatternValueDec
        | Action::PatternValueInc
        | Action::PatternViewPrev
        | Action::PatternViewNext
        | Action::PatternOverviewEmptyToggle
        | Action::PatternStepEdit
        | Action::PatternClear => 5,
        Action::ScreenPads
        | Action::ScreenPattern
        | Action::ScreenSettings
        | Action::ScreenFiles
        | Action::ScreenChop
        | Action::ScreenFx
        | Action::HelpToggle
        | Action::HelpPagePrev
        | Action::HelpPageNext
        | Action::ProjectPick
        | Action::PadsToggle
        | Action::Back
        | Action::Quit
        | Action::SessionSave => 6,
    }
}

const HELP_SECTIONS: [&str; 7] = [
    "PADS", "BANK", "FILES", "CHOP", "SETTINGS", "SOUND", "SCREENS",
];

/// Actions of one kind that F1 lists in one row: the row text and the place of the key in it.
fn pair(a: Action) -> Option<(String, i8)> {
    let side = |more: bool| if more { 1 } else { 0 };
    Some(match a {
        Action::SettingsValue(s) => ("change value".into(), s),
        Action::SettingsRow(s) => ("select row".into(), s),
        Action::BankPrev => ("previous / next bank".into(), 0),
        Action::BankNext => ("previous / next bank".into(), 1),
        Action::MarkerSelect(s) => ("chop: previous / next slice".into(), s),
        Action::MarkerNudge { right, fine } => (
            format!(
                "chop: move the slice start {} ms",
                if fine { NUDGE_FINE_MS } else { NUDGE_MS }
            ),
            side(right),
        ),
        Action::CursorMove { right, .. } => ("chop: move the cursor".into(), side(right)),
        Action::PadSelect(d) => ("select pad".into(), d as i8),
        Action::BrowserUp => ("file up / down".into(), 0),
        Action::BrowserDown => ("file up / down".into(), 1),
        Action::PadParam(s) => ("pad: the picked parameter down / up".into(), s),
        Action::SliceBound { end, right, .. } => (
            format!(
                "chop: the slice {} left / right",
                if end { "end" } else { "start" }
            ),
            side(right),
        ),
        Action::EditValue { up, fine } => (
            format!(
                "the picked value down / up{}",
                if fine { ", fine" } else { "" }
            ),
            side(up),
        ),
        Action::MasterVolume(s) => ("master volume down / up 1 dB".into(), s),
        _ => return None,
    })
}

/// An F1 row while it is built: its keys with their place in the row, and the text.
type HelpRow = (Vec<(i8, String)>, String);

/// F1: the keys of the screen under headings; a heading is a row without keys.
/// `kitty`: the terminal gives key releases; without them F1 says where they come from (68).
fn full_help(keymap: &Keymap, screen: Screen, kitty: bool) -> Vec<(String, String)> {
    // per section: rows of (keys with their place, text); pairs gather into one row
    let mut sections: Vec<Vec<HelpRow>> = vec![Vec::new(); HELP_SECTIONS.len()];
    sections[0].push((
        vec![(0, "1-4 Q-R A-F Z-V".to_string())],
        "play pads; Shift = full velocity".to_string(),
    ));
    for (k, a) in keymap.bindings(screen) {
        let section = &mut sections[help_section(a)];
        let (key, text) = match a {
            Action::RecordMic(_) if !kitty => (
                (0, k),
                "record the microphone: toggle on / off (no key releases)".into(),
            ),
            Action::PageNext | Action::PagePrev if screen == Screen::Fx => {
                ((0, k), "select the next / previous shared FX block".into())
            }
            Action::ParamNext | Action::ParamPrev if screen == Screen::Fx => {
                ((0, k), "select the next / previous effect parameter".into())
            }
            // eight banks are one row
            Action::BankSelect(0) => ((0, format!("{k}…8")), "bank 1–8".to_string()),
            Action::BankSelect(_) => continue,
            a => match pair(a) {
                Some((text, place)) => ((place, k), text),
                None => ((0, k), describe(a)),
            },
        };
        match section.iter_mut().find(|(_, t)| *t == text) {
            Some((keys, _)) => keys.push(key),
            None => section.push((vec![key], text)),
        }
    }
    let sections = sections.into_iter().map(|rows| {
        rows.into_iter()
            .map(|(mut keys, text)| {
                keys.sort_by_key(|k| k.0);
                let keys: Vec<String> = keys.into_iter().map(|k| k.1).collect();
                (keys.join(" "), text)
            })
            .collect::<Vec<_>>()
    });
    let mut rows = Vec::new();
    for (name, section) in HELP_SECTIONS.iter().zip(sections) {
        if !section.is_empty() {
            rows.push((String::new(), name.to_string()));
            rows.extend(section);
        }
    }
    // the mode line of a pad and the dots in the panel (55, 77)
    if matches!(screen, Screen::Pads | Screen::Files) {
        rows.push((String::new(), "GLYPHS ON A PAD".to_string()));
        for (glyph, text) in [
            (
                "✱",
                "more settings, see the panel: pitch, envelope, filter, drive or sends",
            ),
            (
                "mute N",
                "its group: a hit stops the other pads of the group",
            ),
            ("⇥", "gate: plays until the release stops it"),
            ("no mark", "one-shot: pressed, it goes on to the end"),
            (
                "●",
                "in the panel: a block with a change; changed values are bright",
            ),
        ] {
            rows.push((glyph.to_string(), text.to_string()));
        }
    }
    if !kitty {
        rows.push((String::new(), "NO KEY RELEASES".to_string()));
        rows.push((
            "gate pads".to_string(),
            "play one-shot: this terminal gives no key releases".to_string(),
        ));
        rows.push((
            "key releases".to_string(),
            "come in Ghostty, kitty, WezTerm and foot".to_string(),
        ));
    }
    rows
}

/// Log to `$XDG_STATE_HOME/ptsr/ptsr.log`. The audio thread never logs.
fn init_log() -> PathBuf {
    let dir = config::xdg_dir("XDG_STATE_HOME", ".local/state");
    let path = dir.join("ptsr.log");
    let file = std::fs::create_dir_all(&dir).and_then(|_| {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
    });
    // the log did not open: run without it; the terminal is taken by the UI
    if let Ok(f) = file {
        let _ =
            simplelog::WriteLogger::init(log::LevelFilter::Info, simplelog::Config::default(), f);
    }
    path
}

#[cfg(test)]
mod tests {
    use super::{BIG_COLS, HelpState, Region, Sample, big_view, step_in};
    use super::{Command, Screen, browser};
    use super::{Level, Notices};
    use input::Action;
    use std::time::Instant;

    /// Scenario "The envelope is visible", the data behind the drawing: the big view
    /// carries the piece (the region only) with the pad's fade-in and tail, and the
    /// curve follows the values.
    #[test]
    fn the_big_view_shows_the_fade_in_and_the_tail() {
        use std::sync::Arc;
        let mut p = super::pads::Pads::default();
        // half a second at 48 kHz: quiet first half, loud second; the piece is the first
        let half = 24_000;
        let sound = Sample::mono(
            (0..48_000)
                .map(|i| if i < half { 0.2 } else { 1.0 })
                .collect(),
        );
        p.sound[0] = Some(Arc::new(sound));
        p.region[0] = Region {
            start: 0,
            end: half,
        };
        p.attack_ms[0] = 250;
        p.decay_ms[0] = 250;
        let big = big_view(&p, 48_000).expect("a sounding pad has a big view");
        // only the piece is measured, scaled to the piece itself
        assert!(big.wave.iter().all(|w| (*w - 1.0).abs() < 1e-3));
        // fade-in over the first 250 ms, decay over the last 250 of the 500 ms piece
        assert!(big.env[0] < 0.01, "starts in silence");
        assert!(big.env[BIG_COLS - 1] < 0.01, "ends in silence");
        assert!(
            (big.env[BIG_COLS / 2] - 1.0).abs() < 1e-3,
            "the middle is full"
        );
        assert!(
            (big.env[BIG_COLS / 4] - 0.5).abs() < 0.02,
            "half of the attack is done at a quarter of the piece"
        );
        // the envelope moves when a value changes: attack off, the start is loud
        p.attack_ms[0] = 0;
        let big = big_view(&p, 48_000).unwrap();
        assert!(big.env[0] > 0.99, "no attack: the start is full");
        // an empty pad draws its test tone, whole (98h)
        p.sound[0] = None;
        let tone = big_view(&p, 48_000).expect("the test tone piece");
        assert!(tone.wave[0] > tone.wave[BIG_COLS - 1], "the tone decays");
    }

    #[test]
    fn buffer_choice_cycles_between_auto_and_the_sizes() {
        use super::next_buffer_choice;
        assert_eq!(next_buffer_choice(0, -1), 0, "auto is the left end");
        assert_eq!(next_buffer_choice(0, 1), 64, "auto steps into the sizes");
        assert_eq!(next_buffer_choice(64, 1), 128);
        assert_eq!(next_buffer_choice(256, 1), 512);
        assert_eq!(next_buffer_choice(512, 1), 1024, "1024 is reachable");
        assert_eq!(next_buffer_choice(1024, 1), 1024, "the right end holds");
        assert_eq!(next_buffer_choice(1024, -1), 512);
        assert_eq!(next_buffer_choice(64, -1), 0, "back into auto");
        assert_eq!(
            next_buffer_choice(300, 1),
            1024,
            "an odd saved value steps to the next size above it"
        );
    }

    /// F12 from the pads screen, on the MIDI tab of the settings.
    fn midi_tab(app: &mut super::App) {
        app.apply(Action::ScreenPads);
        app.apply(Action::ScreenSettings);
        app.settings_tab = 2;
    }

    /// PgDn until the CHOP group row is selected.
    fn jump_to_chop(app: &mut super::App) {
        for _ in 0..10 {
            if matches!(
                app.midi_list().get(app.midi_row),
                Some(super::MidiListRow::Group {
                    group: input::Group::Chop,
                    ..
                })
            ) {
                return;
            }
            app.apply(Action::BankNext);
        }
        panic!("PgDn never reaches the CHOP group");
    }

    /// 28 midi-learn: "To the chop group" — PgDn walks the group rows until the CHOP
    /// group row is selected; PgUp walks back through the group rows.
    #[test]
    fn to_the_chop_group() {
        let mut app = test_app();
        midi_tab(&mut app);
        jump_to_chop(&mut app);
        assert!(matches!(
            app.midi_list().get(app.midi_row),
            Some(super::MidiListRow::Group { .. })
        ));
        // PgUp from an action of CHOP lands on its row, PgUp again on the group before
        app.apply(Action::SettingsRow(1));
        app.apply(Action::BankPrev);
        assert!(matches!(
            app.midi_list().get(app.midi_row),
            Some(super::MidiListRow::Group {
                group: input::Group::Chop,
                ..
            })
        ));
        app.apply(Action::BankPrev);
        assert!(matches!(
            app.midi_list().get(app.midi_row),
            Some(super::MidiListRow::Group {
                group: input::Group::Files,
                ..
            })
        ));
    }

    /// 28 midi-learn: "Closed groups" — F12 from the pads screen shows PADS open and
    /// every other group as one row with its counts, like "CHOP · 31 actions · 2 learned".
    #[test]
    fn closed_groups() {
        let mut app = test_app();
        midi_tab(&mut app);
        let list = app.midi_list();
        let groups: Vec<_> = list
            .iter()
            .filter_map(|r| match r {
                super::MidiListRow::Group { group } => Some(*group),
                _ => None,
            })
            .collect();
        assert_eq!(groups.len(), input::Group::ALL.len(), "one row per group");
        // the actions under the rows belong to the pads group: it is the open one
        let mut under: Vec<input::Group> = Vec::new();
        for r in &list {
            if let super::MidiListRow::Action { info } = r {
                under.push(info.group);
            }
        }
        let pads_actions = input::ACTIONS
            .iter()
            .filter(|i| i.group == input::Group::Pads)
            .count();
        assert!(
            !under.is_empty() && under.iter().all(|g| *g == input::Group::Pads),
            "only the pads actions show, under their group"
        );
        assert_eq!(under.len(), pads_actions);
        // the drawn row counts two learned chop actions
        super::learn::assign(
            &mut app.learned,
            "xsession-pro",
            input::Group::Chop,
            input::midi::Key::Cc(40),
            "chop.cut",
        );
        super::learn::assign(
            &mut app.learned,
            "xsession-pro",
            input::Group::Chop,
            input::midi::Key::Note(41),
            "chop.edit",
        );
        let chop_actions = input::ACTIONS
            .iter()
            .filter(|i| i.group == input::Group::Chop)
            .count();
        let counts = app
            .midi_screen()
            .iter()
            .find_map(|r| match r {
                tui::MidiRow::Group {
                    name: "CHOP",
                    counts,
                } => Some(counts.clone()),
                _ => None,
            })
            .unwrap();
        assert_eq!(counts, format!("{chop_actions} actions · 2 learned"));
    }

    /// 83a midi-learn "Cleanup actions appear in their learn groups": CHOP lists
    /// `chop.assign_order` and `chop.hits_back` with labels, as buttons, not knobs.
    #[test]
    fn cleanup_actions_appear_in_their_learn_groups() {
        let mut app = test_app();
        midi_tab(&mut app);
        jump_to_chop(&mut app);
        // → opens the CHOP group: its actions stand under their row
        app.apply(Action::SettingsValue(1));
        let names: Vec<&str> = app
            .midi_list()
            .iter()
            .filter_map(|r| match r {
                super::MidiListRow::Action { info } => Some(info.name),
                _ => None,
            })
            .collect();
        assert!(
            names.contains(&"chop.assign_order") && names.contains(&"chop.hits_back"),
            "{names:?}"
        );
        for info in input::ACTIONS
            .iter()
            .filter(|i| i.name == "chop.assign_order" || i.name == "chop.hits_back")
        {
            assert_eq!(info.group, input::Group::Chop, "{}", info.name);
            assert!(!info.knob, "{} takes a button, not a knob", info.name);
            assert!(!info.label.is_empty(), "{}", info.name);
        }
    }

    /// 83a midi-learn "Pick assign order in screen learn": Tab on Chop picks
    /// `chop.assign_order` without opening or changing a plan.
    #[test]
    fn pick_assign_order_in_screen_learn() {
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(2));
        app.apply(Action::LearnToggle);
        learn_press(&mut app, ratatui::crossterm::event::KeyCode::Tab);
        assert_eq!(picked(&app).0, "chop.assign_order");
        assert!(app.plan.is_none(), "learning opens no plan");
    }

    /// 83a midi-learn "Pick hits back in screen learn": Shift+H on Chop picks
    /// `chop.hits_back` without a cut or threshold change.
    #[test]
    fn pick_hits_back_in_screen_learn() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        let markers = app.chop.as_ref().unwrap().markers.clone();
        app.apply(Action::LearnToggle);
        // the extended form; the legacy uppercase letter answers the same
        let mut k =
            ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('h'), KeyModifiers::SHIFT);
        assert!(app.learn_key(&keymap, &k), "the mode takes every key");
        assert_eq!(picked(&app).0, "chop.hits_back");
        assert_eq!(app.chop.as_ref().unwrap().markers, markers);
        k = ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('H'), KeyModifiers::NONE);
        assert!(app.learn_key(&keymap, &k));
        assert_eq!(picked(&app).0, "chop.hits_back");
        assert_eq!(app.chop.as_ref().unwrap().markers, markers);
    }

    /// 83a midi-learn "Learn a button for assign order": a learned button switches
    /// by-hand and in-order once while a plan is open and does nothing outside it.
    #[test]
    fn learn_a_button_for_assign_order() {
        use super::chop::Plan;
        let mut app = xsession_connected(&[(input::Group::Chop, 50, "chop.assign_order")]);
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(2));
        // outside a plan the button does nothing
        app.apply(Action::ChopAssign);
        let msg = input::midi::Msg::NoteOn {
            ch: 1,
            note: 50,
            velocity: 100,
        };
        assert_eq!(xs_live(&app, msg), Some(Action::ChopAssignOrder));
        app.apply(xs_live(&app, msg).unwrap());
        assert_eq!(app.plan, Some(Plan::InOrder(None)), "switched once");
        app.apply(xs_live(&app, msg).unwrap());
        assert!(matches!(app.plan, Some(Plan::ByHand(_))));
        app.plan = None;
        app.apply(xs_live(&app, msg).unwrap());
        assert_eq!(app.plan, None, "outside assignment it does nothing");
    }

    /// 83a midi-learn "Learn a button for hits back": a learned button raises the
    /// threshold one step with the cuts and undo rules of Shift+H, and stays ignored
    /// during assignment.
    #[test]
    fn learn_a_button_for_hits_back() {
        let mut app = xsession_connected(&[(input::Group::Chop, 51, "chop.hits_back")]);
        app.screen = Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        app.chop.as_mut().unwrap().set_threshold(18.0 / 40.0);
        let msg = input::midi::Msg::NoteOn {
            ch: 1,
            note: 51,
            velocity: 100,
        };
        assert_eq!(xs_live(&app, msg), Some(Action::ChopHitsBack));
        app.apply(xs_live(&app, msg).unwrap());
        assert_eq!(app.chop.as_ref().unwrap().threshold_db, -12.0);
        // while assigning the button is ignored like the key
        app.apply(Action::ChopAssign);
        let slices = app.chop.as_ref().unwrap().slices().to_vec();
        app.apply(xs_live(&app, msg).unwrap());
        assert_eq!(app.chop.as_ref().unwrap().slices().to_vec(), slices);
        assert_eq!(flash(&app), "assigning: Enter applies, Esc leaves");
    }

    /// 28 midi-learn: "Open a group" — → on the CHOP row shows its actions under it;
    /// ← on one of them closes CHOP and selects its row.
    #[test]
    fn open_a_group() {
        let mut app = test_app();
        midi_tab(&mut app);
        jump_to_chop(&mut app);
        app.apply(Action::SettingsValue(1));
        let at = app.midi_row;
        let list = app.midi_list();
        assert!(matches!(
            list[at],
            super::MidiListRow::Group {
                group: input::Group::Chop,
                ..
            }
        ));
        let chop_actions = input::ACTIONS
            .iter()
            .filter(|i| i.group == input::Group::Chop)
            .count();
        // the actions show under the row, until the next group row
        let under = list[at + 1..]
            .iter()
            .take_while(|r| matches!(r, super::MidiListRow::Action { .. }))
            .count();
        assert_eq!(under, chop_actions, "every chop action shows under the row");
        // ← on one of the actions closes CHOP and selects its row
        app.apply(Action::SettingsRow(1));
        app.apply(Action::SettingsValue(-1));
        let list = app.midi_list();
        let at = app.midi_row;
        assert!(matches!(
            list[at],
            super::MidiListRow::Group {
                group: input::Group::Chop,
                ..
            }
        ));
        assert!(
            matches!(
                list[at + 1],
                super::MidiListRow::Group {
                    group: input::Group::ChopEdit,
                    ..
                }
            ),
            "CHOP has no actions under it anymore"
        );
        // the pads group stays open, as F12 left it
        let actions = list
            .iter()
            .filter(|r| matches!(r, super::MidiListRow::Action { .. }))
            .count();
        let pads_actions = input::ACTIONS
            .iter()
            .filter(|i| i.group == input::Group::Pads)
            .count();
        assert_eq!(actions, pads_actions);
    }

    /// Opens GLOBAL on the MIDI tab, selects the pads row and starts its run (28).
    fn start_pad_run(app: &mut super::App) {
        midi_tab(app);
        app.midi_row = app
            .midi_list()
            .iter()
            .position(|r| {
                matches!(
                    r,
                    super::MidiListRow::Group {
                        group: input::Group::Global
                    }
                )
            })
            .expect("GLOBAL is in the list");
        app.apply(Action::SettingsValue(1));
        let at = app
            .midi_list()
            .iter()
            .position(|r| {
                matches!(
                    r,
                    super::MidiListRow::Set {
                        name: "pads 1–16",
                        ..
                    }
                )
            })
            .expect("the pads row is in the open GLOBAL");
        app.midi_row = at;
        app.apply(Action::SettingsLearn);
    }

    /// A controller hit while the MIDI tab works with the learn flow.
    fn hit_note(app: &mut super::App, note: u8) {
        app.midi_event(super::midi::Event::Message {
            port: "X".into(),
            profile: "xsession-pro".into(),
            msg: input::midi::Msg::NoteOn {
                ch: 1,
                note,
                velocity: 127,
            },
            meaning: String::new(),
        });
    }

    /// The `who` of the drawn pads row.
    fn pads_row_who(app: &super::App) -> String {
        app.midi_screen()
            .iter()
            .find_map(|r| match r {
                tui::MidiRow::Action { label, who, .. } if label == "pads 1–16" => {
                    Some(who.clone())
                }
                _ => None,
            })
            .expect("the pads row is drawn")
    }

    /// The learned control of a pad action, or `None`.
    fn learned_control(app: &super::App, action: &str) -> Option<String> {
        app.learned["xsession-pro"]["global"]
            .iter()
            .find_map(|(k, a)| (a == action).then_some(k.clone()))
    }

    /// 28 midi-learn: "Sixteen pads in a row" — Enter on "pads 1–16" and 16 hits learn
    /// pad k on the k-th control; the row says "16 of 16 learned"; Backspace clears
    /// them all.
    #[test]
    fn sixteen_pads_in_a_row() {
        let mut app = test_app();
        start_pad_run(&mut app);
        assert_eq!(
            app.learning_line().as_deref(),
            Some(" pad 1 of 16: hit a control, Esc stops")
        );
        for n in 0..16u8 {
            hit_note(&mut app, 40 + n);
        }
        for n in 0..16u8 {
            assert_eq!(
                learned_control(&app, &format!("pad.{}", n + 1)),
                Some(format!("note {}", 40 + n)),
                "controller pad {} plays pad {}",
                n + 1,
                n + 1
            );
        }
        assert_eq!(pads_row_who(&app), "16 of 16 learned");
        assert_eq!(app.learning_line(), None, "the run ended after the last");
        // Backspace on the row clears them all
        app.apply(Action::SettingsClear);
        assert_eq!(pads_row_who(&app), "0 of 16 learned");
        assert_eq!(learned_control(&app, "pad.1"), None);
        // the banks row is its own run and untouched
        let banks = app.midi_screen().iter().find_map(|r| match r {
            tui::MidiRow::Action { label, who, .. } if label == "banks 1–8" => Some(who.clone()),
            _ => None,
        });
        assert_eq!(banks, Some("0 of 8 learned".into()));
    }

    /// 28 midi-learn: "A double hit" — a control taken in the run is ignored: the same
    /// button twice does not take two pads.
    #[test]
    fn a_double_hit() {
        let mut app = test_app();
        start_pad_run(&mut app);
        hit_note(&mut app, 40);
        assert_eq!(learned_control(&app, "pad.1"), Some("note 40".into()));
        // the same button again: the run keeps waiting for another control
        hit_note(&mut app, 40);
        assert_eq!(
            app.learning_line().as_deref(),
            Some(" pad 2 of 16: hit a control, Esc stops")
        );
        hit_note(&mut app, 41);
        assert_eq!(learned_control(&app, "pad.2"), Some("note 41".into()));
        assert_eq!(
            app.learning_line().as_deref(),
            Some(" pad 3 of 16: hit a control, Esc stops"),
            "two pads took two controls"
        );
    }

    /// 28 midi-learn: "Stop in the middle" — Esc after 5 pads keeps them learned; the
    /// next run starts at the first not learned one.
    #[test]
    fn stop_in_the_middle() {
        let mut app = test_app();
        start_pad_run(&mut app);
        for n in 0..5u8 {
            hit_note(&mut app, 40 + n);
        }
        app.apply(Action::Back);
        assert_eq!(app.learning_line(), None, "Esc stops the run");
        assert_eq!(pads_row_who(&app), "5 of 16 learned");
        for n in 0..5u8 {
            assert_eq!(
                learned_control(&app, &format!("pad.{}", n + 1)),
                Some(format!("note {}", 40 + n))
            );
        }
        // the next run starts at the first not learned one
        start_pad_run(&mut app);
        assert_eq!(
            app.learning_line().as_deref(),
            Some(" pad 6 of 16: hit a control, Esc stops")
        );
    }

    #[test]
    fn the_same_whole_file_asked_nothing() {
        let mut p = super::pads::Pads::default();
        let kick = std::path::PathBuf::from("/m/kick.wav");
        p.file[0] = Some(kick.clone());
        p.region[0] = super::Region::WHOLE;
        assert!(super::same_whole_sound(&p, 0, &kick));
        // a slice of the same file is another sound: the question stays
        p.region[0] = super::Region {
            start: 100,
            end: 400,
        };
        assert!(!super::same_whole_sound(&p, 0, &kick));
        // another file is another sound
        assert!(!super::same_whole_sound(
            &p,
            0,
            std::path::Path::new("/m/snare.wav")
        ));
    }

    #[test]
    fn a_recording_becomes_a_stereo_sample_or_is_dropped_as_silence() {
        // one second: left 0.5, right -0.5
        let buf: Vec<f32> = (0..48_000).flat_map(|_| [0.5, -0.5]).collect();
        let s = super::sample_from_interleaved(&buf, 48_000).expect("kept");
        let right = s.right.as_ref().expect("stereo");
        assert_eq!((s.left.len(), right.len()), (48_000, 48_000));
        assert_eq!((s.left[24_000], right[24_000]), (0.5, -0.5));
        // quieter than -60 dB is silence: the file is not written
        assert!(super::sample_from_interleaved(&[0.0001; 4], 48_000).is_none());
        assert!(super::sample_from_interleaved(&[], 48_000).is_none());
    }

    #[test]
    fn no_click_at_the_edges() {
        // the key was pressed in the middle of a loud sound: full scale from the first frame
        let buf = vec![0.9f32; 48_000 * 2];
        let s = super::sample_from_interleaved(&buf, 48_000).expect("kept");
        let right = s.right.as_ref().unwrap();
        for ch in [&s.left[..], &right[..]] {
            assert_eq!(
                (ch[0], ch[ch.len() - 1]),
                (0.0, 0.0),
                "the edges start from silence"
            );
            // 0.9 over 5 ms (240 frames) is about 0.004 per frame; a click is a jump
            let step = engine::checks::max_step(ch);
            assert!(step < 0.01, "a step of {step} clicks");
        }
        // a recording shorter than two fades still fades, it does not panic
        assert!(super::sample_from_interleaved(&[0.9; 6], 48_000).is_some());
    }

    /// The program without a sound card: a real loader thread, no audio, no MIDI.
    pub(super) fn test_app() -> super::App {
        use super::*;
        App {
            transport: transport::State::default(),
            audio: None,
            pending_audio: Default::default(),
            pending_manual_save: None,
            quit_requested: false,
            audio_held: [None; PADS],
            midi_ui_held: Default::default(),
            restoring_project: false,
            midi_preview_blocked: false,
            cfg: AudioConfig::default(),
            user_path: None,
            path_picker: None,
            project_picker: None,
            pad_naming: None,
            paths: config::PathsConfig::default(),
            routed: Routed::default(),
            screen: Screen::Pads,
            fx_slot: 0,
            fx_param: 0,
            settings_row: 0,
            help_open: false,
            help_overlay: help::State::default(),
            value_popup: None,
            pattern_panel: None,
            erase_pads: false,
            pattern_view: pattern_view::State::default(),
            silent_pad_keys: [false; PADS_PER_BANK],
            input_sources: input_sources::State::default(),
            pads: pads::Pads::default(),
            meters: meters::Meters::default(),
            settings_tab: 0,
            takeover: Takeover::default(),
            pickup: None,
            midi_devices: Vec::new(),
            input_log: Vec::new(),
            midi: None,
            profiles: Vec::new(),
            learned: Default::default(),
            seeded: Vec::new(),
            midi_row: 0,
            midi_open: Vec::new(),
            midi_run: None,
            learning: None,
            browser: browser::Browser::new(None),
            loader: Some(loader::start(48_000)),
            load_gen: vec![0; PADS + 2],
            load_region: vec![Region::WHOLE; PADS + 2],
            followers: Vec::new(),
            preview_on: true,
            master_db: 0,
            preview: None,
            pad_in_hand: None,
            copy_in_hand: None,
            session_path: "session.toml".into(),
            project: None,
            project_cfg: config::ProjectConfig::default(),
            session_read_error: Some("test session is protected".into()),
            session_write_failed: false,
            session_dirty: None,
            chop: None,
            chops: Default::default(),
            pending_chops: Default::default(),
            older: Vec::new(),
            pending_older: Vec::new(),
            fresh_undo: None,
            screen_learn: None,
            last_done: None,
            loop_sent: None,
            chop_from: None,
            history: Vec::new(),
            ui: Default::default(),
            range_unlinks: Vec::new(),
            plan: None,
            plan_undo: Vec::new(),
            edit: false,
            slice_end: false,
            bound_repeats: 0,
            lit: None,
            fake_playhead: None,
            ask: None,
            rate: 48_000,
            input_level: 0.0,
            notices: Notices::new(Instant::now()),
            notice_origin: super::Origin::System,
            shown_error: None,
            frame_notice_level: None,
            chop_loading: None,
            keymap_warnings: Default::default(),
            keymap_warning_shown: false,
            splash: false,
            recording: None,
            #[cfg(test)]
            sent: Vec::new(),
        }
    }

    #[test]
    fn app_focus_keeps_overlapping_modes_below_overlays() {
        use input::{Focus, Mode};
        let mut app = test_app();
        assert_eq!(app.focus(), Focus::default());
        app.pads.bank = 7;
        app.screen = Screen::Chop;
        app.edit = true;
        app.pads.marking = true;
        app.pad_naming = Some((0, "name".into()));
        app.project_picker = Some(super::ProjectPicker {
            entries: Vec::new(),
            selected: 0,
            naming: None,
        });
        app.screen_learn = Some(app.learn_open());
        app.learning = Some(super::learn::Learning::new(super::action_info("pad.copy")));
        app.ask = Some(super::Ask::Plan { lines: Vec::new() });
        app.browser.in_hand = Some("break.wav".into());
        app.copy_in_hand = Some((0, app.pads.snapshot(0)));
        app.pad_in_hand = Some(0);
        let expected = Focus {
            screen: Screen::Chop,
            bank: 7,
            mode: Mode {
                edit: true,
                marking: true,
                text: true,
                picker: true,
                screen_learn: true,
                midi_learn: true,
                question: true,
                file_in_hand: true,
                copy_in_hand: true,
                swap_in_hand: true,
                pattern_pads: false,
                erase_pads: false,
                step_edit: false,
            },
        };
        assert_eq!(app.focus(), expected);
        assert_eq!(Focus::unpack(expected.pack().unwrap()), Some(expected));
        assert_eq!(input::route_pad(app.focus()), input::PadRoute::Ignore);
        app.screen_learn = None;
        assert_eq!(input::route_pad(app.focus()), input::PadRoute::Ui);
        app.pad_naming = None;
        assert!(!app.focus().mode.text);
        app.project_picker.as_mut().unwrap().naming = Some("new".into());
        assert!(app.focus().mode.text);
        app.project_picker = None;
        app.screen = Screen::Files;
        app.browser.editing = true;
        assert!(app.focus().mode.text);
        assert!(!app.focus().mode.picker);
        app.browser.editing = false;
        assert!(!app.focus().mode.text);
        app.path_picker = Some(super::PathPicker {
            which: 0,
            at: "target/85-picker".into(),
            entries: Vec::new(),
            selected: 0,
        });
        assert!(app.focus().mode.picker);
        app.learning = None;
        app.midi_run_start(super::midi_sets()[0].1);
        assert!(app.focus().mode.midi_learn);
    }

    #[test]
    fn shared_midi_publication_keeps_the_existing_routes() {
        use input::{PadRoute, route_pad};
        let shared = super::midi::Shared::default();
        let mut app = test_app();
        for (screen, code) in [
            (Screen::Pads, 0),
            (Screen::Settings, 1),
            (Screen::Files, 2),
            (Screen::Chop, 3),
            (Screen::Fx, 4),
        ] {
            app.screen = screen;
            for bank in [0, 7] {
                app.pads.bank = bank;
                for edit in [true, false] {
                    app.edit = edit;
                    for learn in [true, false] {
                        app.screen_learn = learn.then(|| app.learn_open());
                        for hand in [1, 2, 3, 4, 0] {
                            app.browser.in_hand = (hand == 1).then(|| "break.wav".into());
                            app.pad_in_hand = (hand == 2).then_some(4);
                            app.copy_in_hand = (hand == 3).then(|| (4, app.pads.snapshot(4)));
                            app.ask = (hand == 4).then(|| super::Ask::Plan { lines: Vec::new() });
                            app.publish_midi(&shared);
                            let focus = shared.focus().unwrap();
                            assert_eq!(focus.bank, bank);
                            assert_eq!(focus.screen as u8, code);
                            assert_eq!(focus.mode.edit, edit);
                            assert_eq!(focus.mode.screen_learn, learn);
                            let hand_route = focus.mode.file_in_hand
                                || focus.mode.swap_in_hand
                                || focus.mode.copy_in_hand
                                || focus.mode.question
                                || focus.screen == Screen::Chop;
                            assert_eq!(hand_route, hand != 0 || screen == Screen::Chop);
                            assert_eq!(
                                route_pad(focus),
                                if learn {
                                    PadRoute::Ignore
                                } else if hand_route {
                                    PadRoute::Ui
                                } else {
                                    PadRoute::Sound
                                }
                            );
                        }
                    }
                }
            }
        }
    }

    /// Feeds loader replies to the app until `done` holds; fails after 5 seconds.
    fn wait_for(app: &mut super::App, done: impl Fn(&super::App) -> bool) {
        let start = std::time::Instant::now();
        while !done(app) {
            assert!(start.elapsed().as_secs() < 5, "the loader did not answer");
            let reply = app
                .loader
                .as_ref()
                .unwrap()
                .1
                .recv_timeout(std::time::Duration::from_secs(1));
            if let Ok(r) = reply {
                app.loader_reply(r);
            }
        }
    }

    fn tmp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("ptsr-rec-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// 73 "The splash at start": the wave strip holds the splash until the first sound;
    /// the loading line names what loads now, then `ready`. No time and no key ends the
    /// splash by itself.
    #[test]
    fn the_splash_at_start() {
        let mut app = test_app();
        app.splash = true;
        let view = app
            .splash_view()
            .expect("the splash is up before any sound");
        assert_eq!(view.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(view.loading, "ready", "nothing loads yet");
        // while samples load, the line names the session and the number of samples
        app.pads.loading[2] = Some(40);
        app.pads.loading[5] = Some(0);
        assert_eq!(
            app.splash_view().unwrap().loading,
            "loading the session, 2 samples"
        );
        app.pads.loading[2] = None;
        app.pads.loading[5] = None;
        // a screen change alone does not end it: only sound does
        app.screen = Screen::Files;
        assert!(app.splash_view().is_some());
        app.screen = Screen::Pads;
        // the drawing of the strip is the tui test; nothing here waits on time
    }

    /// 73 "A pad hit during the splash": the key of pad `Q` plays at once and the splash
    /// ends — the strip shows the wave from then on.
    #[test]
    fn a_pad_hit_during_the_splash() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = laid_out_break();
        app.audio = Some(super::audio::offline(48_000).0);
        app.screen = Screen::Pads;
        app.splash = true;
        pads_key(&mut app, KeyCode::Char('q'), M::NONE);
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::PadPress { pad: 4, .. })),
            "pad Q (4) plays"
        );
        assert!(!app.splash, "the note ended the splash");
    }

    #[test]
    fn splash_waits_for_accepted_pad_and_preview() {
        use std::sync::atomic::Ordering::Release;
        let mut app = chop_from_pad(4);
        app.screen = Screen::Pads;
        app.splash = true;
        let press = || Command::PadPress {
            pad: 4,
            velocity: 1.0,
        };
        app.send(press());
        assert!(app.splash, "no audio");
        let (audio, mut rt) = super::audio::offline(48_000);
        app.audio = Some(audio);
        app.audio.as_ref().unwrap().project.epoch.store(1, Release);
        app.send(press());
        assert!(app.splash, "project not ready");
        app.audio.as_ref().unwrap().project.ready.store(1, Release);
        let queue = &mut app.audio.as_mut().unwrap().commands;
        while queue
            .push(super::ProjectCommand {
                epoch: 1,
                ready_at_send: true,
                command: super::ProjectMessage::Command(Command::PadRelease { pad: 0 }),
            })
            .is_ok()
        {}
        app.send(press());
        assert!(app.splash, "full queue");
        let sound = app.chop.as_ref().unwrap().sound.clone();
        app.send(Command::Preview {
            sound: Some(sound),
            region: Region::WHOLE,
        });
        assert!(app.splash, "preview still waiting for the queue");
        let mut left = [0.0; 128];
        let mut right = [0.0; 128];
        for _ in 0..8 {
            rt.process(&mut left, &mut right);
        }
        // The RT changes readiness on project transition; make it ready for this queue test.
        app.audio.as_ref().unwrap().project.ready.store(1, Release);
        app.flush_audio();
        assert!(!app.splash, "preview entered the queue");
    }

    /// 73 "A command key during the splash": F3 opens the chop screen and the splash
    /// stays until the first sound.
    #[test]
    fn a_command_key_during_the_splash() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = chop_from_pad(0);
        app.screen = Screen::Pads;
        app.splash = true;
        pads_key(&mut app, KeyCode::F(3), M::NONE);
        assert_eq!(app.screen, Screen::Chop);
        assert!(app.splash, "no sound went: the splash stays");
    }

    /// 73 "A missing sample on the splash": the session names a sample file that does
    /// not exist, the loading line names that file.
    #[test]
    fn a_missing_sample_on_the_splash() {
        let mut app = test_app();
        app.splash = true;
        let gone = tmp_dir("splash-gone").join("gone.wav");
        let sess = super::session::Session {
            pads: std::collections::BTreeMap::from([(
                "0".into(),
                super::session::PadState {
                    file: Some(gone),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        app.apply_session(sess);
        let view = app.splash_view().expect("the splash is up");
        assert!(view.loading.contains("gone.wav"), "{}", view.loading);
    }

    /// 73: a hit on a pad with no sound does not end the splash, and neither does a note
    /// of the controller on such a pad — no sound went.
    #[test]
    fn an_empty_pad_does_not_end_the_splash() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = laid_out_break();
        app.screen = Screen::Pads;
        app.splash = true;
        const PAD_KEYS: [char; 16] = [
            '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v',
        ];
        let empty = (0..16)
            .find(|&p| app.pads.file[p].is_none())
            .expect("an empty pad exists in the fixture");
        pads_key(&mut app, KeyCode::Char(PAD_KEYS[empty]), M::NONE);
        assert!(app.splash, "an empty pad makes no sound");
        app.midi_event(super::midi::Event::PadHit {
            sound: super::midi::PadSound::DirectRejected,
            abs: empty,
            velocity: 1.0,
            ui_press: None,
        });
        assert!(
            app.splash,
            "a controller note on an empty pad makes no sound"
        );
    }

    /// 73: a note of the controller on a pad with a sound ends the splash too.
    #[test]
    fn a_controller_note_ends_the_splash() {
        let mut app = laid_out_break();
        app.screen = Screen::Pads;
        app.splash = true;
        assert!(app.pads.file[4].is_some(), "pad Q has a sample");
        app.midi_event(super::midi::Event::PadHit {
            sound: super::midi::PadSound::DirectRejected,
            abs: 4,
            velocity: 1.0,
            ui_press: None,
        });
        assert!(app.splash, "a rejected controller note keeps the splash");
        app.midi_event(super::midi::Event::PadHit {
            sound: super::midi::PadSound::DirectAccepted,
            abs: 4,
            velocity: 1.0,
            ui_press: None,
        });
        assert!(!app.splash, "the controller note ended the splash");
    }

    fn startup_file_fixture(name: &str) -> (std::path::PathBuf, [std::path::PathBuf; 2]) {
        let root = tmp_dir(name);
        let bases = [root.join("config"), root.join("share")];
        let old_cfg = bases[0].join("arasaka-404");
        std::fs::create_dir_all(old_cfg.join("devices")).unwrap();
        std::fs::write(
            old_cfg.join("config.toml"),
            "[audio]\noutput = \"Legacy\"\n[midi.\"X-Session Pro\".pads]\n\"cc 31\" = \"pad.pitch\"\n",
        )
        .unwrap();
        std::fs::write(
            old_cfg.join("keymap.toml"),
            "[screens.pads]\n\"Ctrl+G\" = \"none\"\n",
        )
        .unwrap();
        std::fs::write(old_cfg.join("devices/my.toml"), "name = \"My\"\n").unwrap();
        let old_data = bases[1].join("arasaka-404");
        std::fs::create_dir_all(&old_data).unwrap();
        std::fs::write(
            old_data.join("session.toml"),
            "version = 3\nbrowser_dir = \"/legacy\"\n[pads.0]\nfile = \"/legacy.wav\"\n",
        )
        .unwrap();
        (root, bases)
    }

    #[test]
    fn current_ptsr_files_load() {
        let (root, bases) = startup_file_fixture("current-files");
        let snapshot: Vec<_> = [
            bases[0].join("arasaka-404/config.toml"),
            bases[0].join("arasaka-404/keymap.toml"),
            bases[0].join("arasaka-404/devices/my.toml"),
            bases[1].join("arasaka-404/session.toml"),
        ]
        .into_iter()
        .map(|p| {
            let bytes = std::fs::read(&p).unwrap();
            (p, bytes)
        })
        .collect();
        for base in &bases {
            std::fs::create_dir_all(base.join("ptsr")).unwrap();
        }
        let cfg = bases[0].join("ptsr/config.toml");
        let config_text = "[audio]\noutput = \"Current\"\n[midi.\"X-Session Pro\".pads]\n\"cc 31\" = \"pad.volume\"\n";
        std::fs::write(&cfg, config_text).unwrap();
        let keys = bases[0].join("ptsr/keymap.toml");
        let key_text = "[screens.pads]\n\"Ctrl+G\" = \"pad.mark_bank\"\n";
        std::fs::write(&keys, key_text).unwrap();
        let sess = bases[1].join("ptsr/session.toml");
        let session_text = "version = 3\nbrowser_dir = \"/current\"\n[pads.0]\nfile = \"/current.wav\"\ngate = true\npitch = 7\n";
        std::fs::write(&sess, session_text).unwrap();

        let loaded = super::config::load_from(&cfg);
        assert_eq!(loaded.error, None);
        assert_eq!(loaded.user_path, Some(cfg.clone()));
        assert_eq!(loaded.config.audio.output.as_deref(), Some("Current"));
        assert_eq!(
            loaded.config.midi["X-Session Pro"]["pads"]["cc 31"],
            "pad.volume"
        );
        let (keymap, notice, _) = super::load_keymap(&keys, super::DEFAULT_KEYMAP);
        assert_eq!(notice, None);
        assert_eq!(pads_ctrl(&keymap, 'g'), Some(Action::PadMarkBank));
        let session = super::session::load(&sess).unwrap();
        assert_eq!(session.browser_dir, Some("/current".into()));
        assert_eq!(session.pads["0"].file, Some("/current.wav".into()));
        assert!(session.pads["0"].gate);
        assert_eq!(session.pads["0"].pitch, 7);
        for (path, bytes) in snapshot {
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
        for (path, text) in [(cfg, config_text), (keys, key_text), (sess, session_text)] {
            assert_eq!(std::fs::read_to_string(path).unwrap(), text);
        }
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn legacy_folders_stay_unused_without_current_files() {
        let (root, bases) = startup_file_fixture("legacy-unused");
        let snapshot: Vec<_> = [
            bases[0].join("arasaka-404/config.toml"),
            bases[0].join("arasaka-404/keymap.toml"),
            bases[0].join("arasaka-404/devices/my.toml"),
            bases[1].join("arasaka-404/session.toml"),
        ]
        .into_iter()
        .map(|p| {
            let bytes = std::fs::read(&p).unwrap();
            (p, bytes)
        })
        .collect();
        let loaded = super::config::load_from(&bases[0].join("ptsr/config.toml"));
        let defaults = super::config::load_from(&root.join("absent/config.toml"));
        assert_eq!(loaded.error, None);
        assert_eq!(loaded.config.audio, defaults.config.audio);
        assert!(loaded.config.midi.is_empty());
        let (keymap, notice, _) =
            super::load_keymap(&bases[0].join("ptsr/keymap.toml"), super::DEFAULT_KEYMAP);
        assert_eq!(notice, None);
        assert_eq!(pads_ctrl(&keymap, 'g'), None);
        assert_eq!(pads_ctrl(&keymap, 'a'), Some(Action::PadMarkBank));
        assert_eq!(
            super::session::load(&bases[1].join("ptsr/session.toml")).unwrap(),
            super::session::Session::default()
        );
        for base in &bases {
            assert!(!base.join("ptsr").exists());
            assert!(!base.join("ptsr.part").exists());
        }
        for (path, bytes) in snapshot {
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// 68 sample-loading "Ctrl+B on a file": in "breaks" with a file selected the status
    /// names the folder of the file; the second Ctrl+B takes it back.
    #[test]
    fn ctrl_b_on_a_file() {
        let dir = tmp_dir("fav-file");
        std::fs::create_dir_all(dir.join("breaks")).unwrap();
        std::fs::write(dir.join("breaks/amen.wav"), b"").unwrap();
        let mut app = test_app();
        app.user_path = Some(dir.join("config.toml"));
        app.browser = super::browser::Browser::new(Some(dir.join("breaks")));
        app.screen = Screen::Files;
        assert_eq!(
            app.browser.current().map(|e| e.name.as_str()),
            Some("amen.wav")
        );
        app.apply(Action::BrowserFavorite);
        assert_eq!(flash(&app), "★ breaks: favorite (the folder of this file)");
        assert_eq!(app.browser.favorites, [dir.join("breaks")]);
        app.apply(Action::BrowserFavorite);
        assert_eq!(
            flash(&app),
            "breaks: no longer a favorite (the folder of this file)"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 68 chopping "The header shows play": Space plays, the header says `LISTEN ▶`; the
    /// second Space stops it and the header says `LISTEN`. The engine's playhead is stood
    /// in for: it runs while the preview voice plays.
    #[test]
    fn the_header_shows_play() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(2));
        let mode = |app: &super::App| {
            let c = app.chop.as_ref().unwrap();
            super::chop_view(c, app.plan.as_ref(), app.playhead(), 48_000, &app.pads).mode
        };
        assert_eq!(mode(&app), "LISTEN");
        chop_key(&mut app, KeyCode::Char(' '), M::NONE);
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::Preview { sound: Some(_), .. }))
        );
        app.fake_playhead = Some(1_000);
        assert_eq!(mode(&app), "LISTEN ▶");
        chop_key(&mut app, KeyCode::Char(' '), M::NONE);
        assert!(matches!(
            app.sent.last(),
            Some(Command::Preview { sound: None, .. })
        ));
        app.fake_playhead = None;
        assert_eq!(mode(&app), "LISTEN");
    }

    /// 68 screen-help "F1 says where releases come from": without key releases F1 names
    /// the terminals that give them; with them the rows are absent.
    #[test]
    fn f1_says_where_releases_come_from() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let text = |kitty| {
            super::full_help(&keymap, Screen::Pads, kitty)
                .into_iter()
                .map(|(k, t)| format!("{k} {t}"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let without = text(false);
        for name in ["Ghostty", "kitty", "WezTerm", "foot"] {
            assert!(without.contains(name), "{name}: {without}");
        }
        assert!(!text(true).contains("Ghostty"));
    }

    /// The owner's keymap `text` in a fresh folder, loaded as at start (64).
    fn user_keymap(name: &str, text: &str) -> (input::Keymap, Option<String>) {
        let dir = tmp_dir(name);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("keymap.toml");
        std::fs::write(&path, text).unwrap();
        let (keymap, notice, _) = super::load_keymap(&path, super::DEFAULT_KEYMAP);
        std::fs::remove_dir_all(&dir).ok();
        (keymap, notice)
    }

    fn pads_ctrl(k: &input::Keymap, c: char) -> Option<Action> {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        k.resolve(Screen::Pads, &KeyEvent::new(KeyCode::Char(c), M::CONTROL))
    }

    /// 71 input-devices "The shipped bindings all act": the keymap and the device profiles
    /// name only actions the program has; Ctrl+K and knobs k6–k10 of the chop screen are free.
    /// Scenario "Shipped profiles load without template errors" (input-devices).
    #[test]
    fn the_shipped_bindings_all_act() {
        let (k, warnings) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(pads_ctrl(&k, 'k'), None, "Ctrl+K is free");
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../config/devices");
        let (profiles, errors) = super::midi::load_profiles(dir);
        assert!(errors.is_empty(), "{errors:?}");
        assert!(!profiles.iter().any(|p| p.name == "SMC-PAD"));
        let x = profiles.iter().find(|p| p.name == "X-Session Pro").unwrap();
        for (scope, key, name) in x.roles() {
            assert!(input::midi::known(&name), "{key} {name}");
            if scope == input::midi::Scope::Screen(Screen::Chop) {
                for cc in [29, 31, 32, 33, 34] {
                    assert_ne!(key, input::midi::Key::Cc(cc), "k6–k10 are free: {name}");
                }
            }
        }
    }

    /// 71 input-devices "Dead bindings leave the config": X-Session cc 29 on the chop
    /// screen bound to slice.pitch is dropped and saved away, the notice counts one, the
    /// next start says nothing.
    #[test]
    fn dead_bindings_leave_the_config() {
        let dir = tmp_dir("dead-bindings");
        std::fs::create_dir_all(&dir).unwrap();
        let cfg = dir.join("config.toml");
        std::fs::write(
            &cfg,
            "[midi.\"X-Session Pro\".chop]\n\"cc 24\" = \"slice.start\"\n\"cc 29\" = \"slice.pitch\"\n",
        )
        .unwrap();
        let mut app = test_app();
        app.user_path = Some(cfg.clone());
        app.learned = super::config::load_from(&cfg).config.midi;
        assert_eq!(app.learned["X-Session Pro"]["chop"].len(), 2);
        let notice = app.drop_dead_bindings().unwrap();
        assert!(notice.contains("1 binding "), "{notice}");
        let text = std::fs::read_to_string(&cfg).unwrap();
        assert!(!text.contains("slice.pitch"), "{text}");
        assert!(text.contains("slice.start"), "{text}");
        let mut again = test_app();
        again.user_path = Some(cfg.clone());
        again.learned = super::config::load_from(&cfg).config.midi;
        assert_eq!(
            again.drop_dead_bindings(),
            None,
            "the next start says nothing"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 71 input-devices "An unknown action is named": Ctrl+G = pad.flyaway in the owner's
    /// keymap is named with the file and the key; the rest works.
    #[test]
    fn an_unknown_action_is_named() {
        let (k, notice) = user_keymap(
            "km-unknown",
            "[screens.pads]\n\"Ctrl+G\" = \"pad.flyaway\"\n",
        );
        let notice = notice.unwrap();
        for part in ["keymap.toml", "Ctrl+G", "pad.flyaway"] {
            assert!(notice.contains(part), "{part}: {notice}");
        }
        assert_eq!(pads_ctrl(&k, 'g'), None);
        assert_eq!(pads_ctrl(&k, 'a'), Some(Action::PadMarkBank));
    }

    /// 64 keyboard "A key of my own": `Ctrl+G` marks the bank, `Ctrl+A` still does.
    #[test]
    fn a_key_of_my_own() {
        let (k, notice) = user_keymap("km-own", "[screens.pads]\n\"Ctrl+G\" = \"pad.mark_bank\"\n");
        assert_eq!(notice, None);
        assert_eq!(pads_ctrl(&k, 'g'), Some(Action::PadMarkBank));
        assert_eq!(pads_ctrl(&k, 'a'), Some(Action::PadMarkBank));
    }

    /// 76 keyboard "An old action name": `pad.scope_next` is read as `pad.mark_bank`.
    #[test]
    fn an_old_action_name() {
        let (k, notice) = user_keymap(
            "km-old",
            "[screens.pads]\n\"Ctrl+G\" = \"pad.scope_next\"\n",
        );
        assert_eq!(notice, None);
        assert_eq!(pads_ctrl(&k, 'g'), Some(Action::PadMarkBank));
    }

    /// 64 keyboard "A key taken away": `"none"` unbinds `Ctrl+A` on the pads screen.
    #[test]
    fn a_key_taken_away() {
        let (k, _) = user_keymap("km-none", "[screens.pads]\n\"Ctrl+A\" = \"none\"\n");
        assert_eq!(pads_ctrl(&k, 'a'), None);
        assert_eq!(pads_ctrl(&k, 'z'), Some(Action::Undo), "the rest stays");
    }

    /// 64 keyboard "A broken file": the notice names the file and the line, every
    /// shipped key works.
    #[test]
    fn a_broken_file() {
        let (k, notice) = user_keymap("km-broken", "[screens.pads]\n\"Ctrl+G\" = \n");
        let notice = notice.unwrap();
        assert!(notice.contains("keymap.toml line 2"), "{notice}");
        assert_eq!(pads_ctrl(&k, 'a'), Some(Action::PadMarkBank));
        assert_eq!(pads_ctrl(&k, 'g'), None);
    }

    /// 64 keyboard "From any folder": the shipped keys are built in and the owner's file
    /// comes by its full path, so the start folder plays no part; tests run in the crate
    /// folder, where no `config/keymap.toml` lies.
    #[test]
    fn from_any_folder() {
        let (k, notice) = user_keymap(
            "km-away",
            "[screens.pads]\n\"Ctrl+G\" = \"pad.mark_bank\"\n",
        );
        assert_eq!(notice, None);
        assert_eq!(pads_ctrl(&k, 'g'), Some(Action::PadMarkBank));
        let (shipped, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        assert_eq!(
            k.bindings(Screen::Chop),
            shipped.bindings(Screen::Chop),
            "the shipped keys as from the project folder"
        );
    }

    /// 64 keyboard "Pad keys stay pads": `Q` bound to a command still plays its pad and
    /// the notice says a pad key cannot be a command; `Shift+Q` alike.
    #[test]
    fn pad_keys_stay_pads() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        for key in ["Q", "Shift+Q"] {
            let (k, notice) =
                user_keymap("km-pad", &format!("[global]\n\"{key}\" = \"app.quit\"\n"));
            let notice = notice.unwrap();
            assert!(notice.contains("pad keys cannot be commands"), "{notice}");
            assert!(matches!(
                k.resolve(Screen::Pads, &KeyEvent::new(KeyCode::Char('q'), M::NONE)),
                Some(Action::PadPress { pad: 4, .. })
            ));
        }
    }

    // Task 1.2 tests the migration layer before task 2.1 moves the shipped keys.
    // Every fixture binding uses an action that already has a production handler.
    const NOTICE_DEFAULTS: &str = r#"
[global]
"Alt+R" = "record.mic"
"Ctrl+T" = "record.output"
"Ctrl+Q" = "app.quit"
"Ctrl+Z" = "edit.undo"
PageDown = "bank.next"
F10 = "learn.toggle"
Esc = "ui.back"
[edit]
J = "edit.value_dec"
L = "edit.value_inc"
Left = "edit.value_dec"
Right = "edit.value_inc"
[screens.chop]
H = "chop.hits"
"Ctrl+G" = "chop.normalize"
Enter = "chop.edit"
[screens.pads]
J = "edit.value_dec"
L = "edit.value_inc"
Left = "edit.value_dec"
Right = "edit.value_inc"
U = "bank.prev"
O = "bank.next"
"Ctrl+N" = "pad.rename"
Enter = "pad.edit"
[screens.files]
Slash = "browser.filter"
Enter = "browser.hold"
"#;

    fn notice_map(defaults: &str, owner: &str) -> input::Keymap {
        let (mut keymap, warnings) = input::Keymap::from_toml(defaults).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!(keymap.overlay(owner).unwrap().is_empty());
        keymap
    }

    #[derive(Default)]
    struct NoticeInput {
        down: input::HeldKeys,
        pads: input::HeldPads,
        mods: input::HeldModifiers,
    }

    impl NoticeInput {
        fn dispatch(
            &mut self,
            app: &mut super::App,
            keymap: &input::Keymap,
            event: ratatui::crossterm::event::KeyEvent,
            kitty: bool,
        ) -> bool {
            app.keyboard_event(
                keymap,
                &event,
                kitty,
                &mut self.down,
                &mut self.pads,
                &mut self.mods,
            )
        }

        fn event(
            &mut self,
            app: &mut super::App,
            keymap: &input::Keymap,
            event: ratatui::crossterm::event::KeyEvent,
            kitty: bool,
        ) {
            app.frame_notice();
            assert!(!self.dispatch(app, keymap, event, kitty));
        }
    }

    fn assert_moved_notice(screen: Screen, latin: char, russian: char, ctrl: bool, message: &str) {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        let keymap = notice_map(NOTICE_DEFAULTS, "");
        for kitty in [false, true] {
            for c in [latin, russian] {
                let mut app = test_app();
                app.screen = screen;
                app.chop = Some(chop_with_parts(4));
                let slices = app.chop.as_ref().unwrap().slices();
                let gain = app.pads.volume_db;
                let mut input = NoticeInput::default();
                input.event(
                    &mut app,
                    &keymap,
                    KeyEvent::new(KeyCode::Char(c), if ctrl { M::CONTROL } else { M::NONE }),
                    kitty,
                );
                assert_eq!(flash(&app), message, "{c}, kitty {kitty}");
                assert_eq!(app.chop.as_ref().unwrap().slices(), slices);
                assert_eq!(app.pads.volume_db, gain);
                assert!(app.recording.is_none());
                assert!(app.sent.is_empty(), "a notice sends no audio commands");
                assert!(app.history.is_empty());
            }
        }
    }

    #[test]
    fn old_microphone_key_names_its_replacement() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        let keymap = notice_map(super::DEFAULT_KEYMAP, "[global]\n\"Ctrl+R\" = \"none\"\n");
        for screen in Screen::ALL {
            for kitty in [false, true] {
                for c in ['r', 'к'] {
                    let mut app = test_app();
                    app.screen = screen;
                    NoticeInput::default().event(
                        &mut app,
                        &keymap,
                        KeyEvent::new(KeyCode::Char(c), M::CONTROL),
                        kitty,
                    );
                    assert_eq!(flash(&app), "");
                    assert!(app.recording.is_none());
                    assert!(app.pads.pattern.is_none());
                    assert!(app.sent.is_empty());
                }
            }
        }
    }

    #[test]
    fn old_normalize_key_names_its_replacement() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        assert_moved_notice(Screen::Chop, 'n', 'т', true, "normalize moved to Ctrl+G");
        let keymap = notice_map(NOTICE_DEFAULTS, "");
        let mut app = test_app();
        app.pads.file[0] = Some("/samples/kick.wav".into());
        NoticeInput::default().event(
            &mut app,
            &keymap,
            KeyEvent::new(KeyCode::Char('n'), M::CONTROL),
            true,
        );
        assert!(
            app.pad_naming.is_some(),
            "Ctrl+N on Pads still opens naming"
        );
        assert!(!flash(&app).contains("moved"));
    }

    #[test]
    fn old_hit_chop_key_names_its_replacement() {
        assert_moved_notice(Screen::Chop, 'y', 'н', true, "chop by hits moved to H");
    }

    #[test]
    fn unbound_bank_keys_name_their_replacements() {
        let defaults = "[global]\nPageDown = \"bank.next\"\n";
        let keymap = notice_map(defaults, "");
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        for (key, message) in [
            ('j', "previous bank moved to U"),
            ('о', "previous bank moved to U"),
            ('l', "next bank moved to O"),
            ('д', "next bank moved to O"),
        ] {
            let mut app = test_app();
            NoticeInput::default().event(
                &mut app,
                &keymap,
                KeyEvent::new(KeyCode::Char(key), M::NONE),
                true,
            );
            assert_eq!(flash(&app), message);
            assert_eq!(app.pads.bank, 0);
        }
    }

    #[test]
    fn rebound_bank_keys_change_values_without_a_move_notice() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        let keymap = notice_map(NOTICE_DEFAULTS, "");
        for editing in [false, true] {
            for (key, change) in [('j', -1), ('о', -1), ('l', 1), ('д', 1)] {
                let mut app = test_app();
                app.edit = editing;
                app.pads.file[0] = Some("/samples/kick.wav".into());
                NoticeInput::default().event(
                    &mut app,
                    &keymap,
                    KeyEvent::new(KeyCode::Char(key), M::NONE),
                    true,
                );
                assert_eq!(app.pads.volume_db[0], change);
                assert_eq!(app.pads.bank, 0);
                assert!(!flash(&app).contains("moved"));
            }
        }
    }

    #[test]
    fn owner_old_key_wins_over_the_move_notice() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        let keymap = notice_map(NOTICE_DEFAULTS, "[global]\n\"Ctrl+R\" = \"record.mic\"\n");
        let mut app = test_app();
        NoticeInput::default().event(
            &mut app,
            &keymap,
            KeyEvent::new(KeyCode::Char('к'), M::CONTROL),
            true,
        );
        assert_eq!(
            flash(&app),
            "No sound: there is nothing to record from.",
            "the real microphone handler, not a moved-key notice"
        );
        let keymap = notice_map(
            NOTICE_DEFAULTS,
            "[screens.chop]\n\"Ctrl+Y\" = \"bank.next\"\n",
        );
        let mut app = test_app();
        app.screen = Screen::Chop;
        NoticeInput::default().event(
            &mut app,
            &keymap,
            KeyEvent::new(KeyCode::Char('y'), M::CONTROL),
            true,
        );
        assert_eq!(app.pads.bank, 1);
        assert!(!flash(&app).contains("moved"));
    }

    #[test]
    fn disabled_old_key_stays_silent() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        let keymap = notice_map(
            NOTICE_DEFAULTS,
            "[global]\n\"Ctrl+R\" = \"none\"\n[screens.chop]\n\"Ctrl+N\" = \"none\"\n\"Ctrl+Y\" = \"none\"\n[screens.pads]\nJ = \"none\"\nL = \"none\"\n",
        );
        for (screen, key, mods) in [
            (Screen::Chop, 'r', M::CONTROL),
            (Screen::Chop, 'т', M::CONTROL),
            (Screen::Chop, 'н', M::CONTROL),
            (Screen::Pads, 'j', M::NONE),
            (Screen::Pads, 'д', M::NONE),
        ] {
            let mut app = test_app();
            app.screen = screen;
            NoticeInput::default().event(
                &mut app,
                &keymap,
                KeyEvent::new(KeyCode::Char(key), mods),
                true,
            );
            assert_eq!(flash(&app), "");
            assert!(app.sent.is_empty());
            assert_eq!(app.pads.bank, 0);
        }
    }

    #[test]
    fn move_notices_do_not_interrupt_other_input_modes() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        let keymap = notice_map(NOTICE_DEFAULTS, "");
        for mode in 0..5 {
            let mut app = test_app();
            match mode {
                0 => {
                    app.screen = Screen::Files;
                    app.browser.editing = true;
                }
                1 => app.pad_naming = Some((0, "kick".into())),
                2 => {
                    app.ask = Some(super::Ask::ClearPads {
                        pads: vec![0],
                        lines: vec!["keep this question".into()],
                    })
                }
                3 => {
                    app.apply(Action::LearnToggle);
                }
                4 => {
                    app.project_picker = Some(super::ProjectPicker {
                        entries: vec![],
                        selected: 0,
                        naming: Some("project".into()),
                    })
                }
                _ => unreachable!(),
            }
            app.post_notice(Level::Warn, ("keep this notice".into(), Instant::now()));
            NoticeInput::default().event(
                &mut app,
                &keymap,
                KeyEvent::new(KeyCode::Char('r'), M::CONTROL),
                true,
            );
            assert_eq!(flash(&app), "keep this notice", "mode {mode}");
            assert!(app.recording.is_none());
        }
        // Repeats and releases are not moved-key presses, including Ghostty repeats.
        let mut app = test_app();
        let mut input = NoticeInput::default();
        let event = KeyEvent::new(KeyCode::Char('r'), M::CONTROL);
        input.event(&mut app, &keymap, event, true);
        app.notices = Notices::new(Instant::now());
        input.event(&mut app, &keymap, event, true);
        assert_eq!(flash(&app), "");
        input.event(
            &mut app,
            &keymap,
            KeyEvent {
                kind: super::event::KeyEventKind::Release,
                ..event
            },
            true,
        );
        assert_eq!(flash(&app), "");
    }

    fn loaded_notice_map(defaults: &str, owner: &str) -> (input::Keymap, Vec<String>) {
        let dir = tmp_dir("83a-notice-keymap");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("keymap.toml");
        std::fs::write(&path, owner).unwrap();
        let (keymap, notice, warnings) = super::load_keymap(&path, defaults);
        assert!(notice.is_none(), "{notice:?}");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            owner,
            "owner file stays unchanged"
        );
        std::fs::remove_dir_all(&dir).unwrap();
        (keymap, warnings)
    }

    #[test]
    fn an_owner_key_hides_a_new_default() {
        let (keymap, warnings) =
            loaded_notice_map(NOTICE_DEFAULTS, "[screens.chop]\nH = \"chop.normalize\"\n");
        assert_eq!(warnings, ["keymap.toml: H on chop hides chop.hits"]);
        assert_eq!(
            keymap.binding_for(Screen::Chop, false, "H"),
            Some(Some(Action::ChopNormalize))
        );
    }

    #[test]
    fn matching_owner_keys_do_not_warn() {
        let (_, warnings) = loaded_notice_map(
            NOTICE_DEFAULTS,
            "[global]\n\"Alt+R\" = \"record.mic\"\n[screens.chop]\nH = \"chop.hits\"\n[screens.pads]\nJ = \"edit.value_dec\"\n",
        );
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn all_changed_default_conflicts_are_reported() {
        let (keymap, warnings) = loaded_notice_map(
            NOTICE_DEFAULTS,
            "[global]\n\"Alt+R\" = \"none\"\n[screens.chop]\nH = \"none\"\n[screens.pads]\nJ = \"none\"\nL = \"bank.next\"\n",
        );
        assert_eq!(
            warnings.len(),
            8,
            "five global screens, Chop H and Pads J/L: {warnings:?}"
        );
        let distinct: std::collections::BTreeSet<_> = warnings.iter().collect();
        assert_eq!(distinct.len(), warnings.len());
        assert!(warnings.contains(&"keymap.toml: H on chop hides chop.hits".into()));
        assert!(warnings.contains(&"keymap.toml: J on pads hides edit.value_dec".into()));
        assert_eq!(keymap.binding_for(Screen::Chop, false, "H"), Some(None));
        assert_eq!(keymap.binding_for(Screen::Pads, false, "J"), Some(None));
        assert_eq!(
            keymap.binding_for(Screen::Pads, false, "L"),
            Some(Some(Action::BankNext))
        );
        let mut app = test_app();
        app.keymap_warnings.extend(warnings.clone());
        let mut input = NoticeInput::default();
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        for warning in warnings {
            assert_eq!(app.frame_notice(), Some(warning));
            input.event(
                &mut app,
                &keymap,
                KeyEvent::new(KeyCode::Char('5'), M::NONE),
                false,
            );
        }
        assert!(app.keymap_warnings.is_empty());
    }

    #[test]
    fn shadow_warnings_follow_effective_layers_and_normalized_chords() {
        let defaults = format!("{NOTICE_DEFAULTS}\n[screens.fx]\n\"Alt+R\" = \"record.mic\"\n");
        let (_, warnings) = loaded_notice_map(
            &defaults,
            "[global]\n\"Alt+R\" = \"none\"\n[edit]\n\"о\" = \"none\"\n",
        );
        assert_eq!(
            warnings.len(),
            5,
            "four global conflicts and one edit-layer conflict: {warnings:?}"
        );
        assert!(
            !warnings.iter().any(|w| w.contains(" on fx ")),
            "screen binding wins over global removal"
        );
        assert!(warnings.contains(&"keymap.toml: J on pads hides edit.value_dec".into()));
    }

    #[test]
    fn planned_defaults_do_not_produce_shadow_warnings() {
        // 83a task 4.1 ships every cleanup binding; an owner override of a shipped
        // default warns. No planned binding is left.
        let (_, warnings) = loaded_notice_map(
            super::DEFAULT_KEYMAP,
            "[screens.pads]\n\"Ctrl+C\" = \"none\"\n",
        );
        assert_eq!(warnings, ["keymap.toml: Ctrl+C on pads hides pad.copy"]);
    }

    /// 83a keyboard "An owner key hides a new default" on the shipped map: the owner's
    /// `none` wins and their file stays, while the startup names the hidden default.
    #[test]
    fn shipped_defaults_shadow_warn_when_the_owner_overrides_them() {
        let (keymap, warnings) =
            loaded_notice_map(super::DEFAULT_KEYMAP, "[global]\n\"Alt+R\" = \"none\"\n");
        assert_eq!(
            warnings,
            [
                "keymap.toml: Alt+R on pads hides record.mic",
                "keymap.toml: Alt+R on chop hides record.mic",
                "keymap.toml: Alt+R on files hides record.mic",
                "keymap.toml: Alt+R on fx hides record.mic",
                "keymap.toml: Alt+R on settings hides record.mic",
            ]
        );
        assert_eq!(
            keymap.binding_for(input::Screen::Pads, false, "Alt+R"),
            Some(None)
        );
    }

    #[test]
    fn only_a_visible_keymap_warning_is_dismissed() {
        use ratatui::crossterm::event::{
            KeyCode, KeyEvent, KeyEventKind as K, KeyModifiers as M, ModifierKeyCode,
        };
        let keymap = notice_map(NOTICE_DEFAULTS, "");
        let mut app = test_app();
        app.keymap_warnings
            .extend(["first warning".into(), "second warning".into()]);
        let mut input = NoticeInput::default();
        let key = KeyEvent::new(KeyCode::Char('5'), M::NONE);
        input.event(&mut app, &keymap, key, true);
        assert_eq!(app.keymap_warnings.len(), 1);
        input.event(&mut app, &keymap, key, true); // Ghostty repeat as a plain press
        input.event(
            &mut app,
            &keymap,
            KeyEvent {
                kind: K::Repeat,
                ..key
            },
            true,
        );
        input.event(
            &mut app,
            &keymap,
            KeyEvent {
                kind: K::Release,
                ..key
            },
            true,
        );
        input.event(
            &mut app,
            &keymap,
            KeyEvent::new(KeyCode::Modifier(ModifierKeyCode::LeftShift), M::SHIFT),
            true,
        );
        assert_eq!(
            app.keymap_warnings.len(),
            1,
            "repeats, release and modifier do not acknowledge it"
        );
        input.event(
            &mut app,
            &keymap,
            KeyEvent::new(KeyCode::PageDown, M::NONE),
            true,
        );
        assert!(app.keymap_warnings.is_empty());
        assert_eq!(app.pads.bank, 1, "the normal command still runs");
    }

    /// 83a "Only a visible keymap warning is dismissed": Enter closes pad naming and
    /// a Ghostty repeat of the still-held key must not count as the new press that
    /// dismisses the warning the next frame shows.
    #[test]
    fn a_repeat_after_a_mode_change_does_not_dismiss_a_warning() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        let keymap = notice_map(NOTICE_DEFAULTS, "");
        let mut app = test_app();
        app.pads.file[0] = Some("/samples/kick.wav".into());
        app.keymap_warnings.push_back("pending warning".into());
        let mut input = NoticeInput::default();
        let enter = KeyEvent::new(KeyCode::Enter, M::NONE);
        // Pad naming hides the warning, so Enter saves the name without dismissing it.
        app.pad_naming = Some((0, "kick".into()));
        input.event(&mut app, &keymap, enter, true);
        assert_eq!(app.keymap_warnings.len(), 1);
        assert_eq!(app.frame_notice().as_deref(), Some("pending warning"));
        // The next frame showed the warning, but Enter is still held: a Ghostty repeat
        // is not a new press and must not dismiss it.
        input.event(&mut app, &keymap, enter, true);
        assert_eq!(
            app.keymap_warnings.len(),
            1,
            "a Ghostty repeat is not a new press after a mode change"
        );
    }

    #[test]
    fn hidden_keymap_warnings_stay_pending() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        let keymap = notice_map(NOTICE_DEFAULTS, "");
        for mode in 0..8 {
            let mut app = test_app();
            app.keymap_warnings.push_back("pending warning".into());
            match mode {
                0 => {
                    app.screen = Screen::Files;
                    app.browser.editing = true;
                }
                1 => app.pad_naming = Some((0, "kick".into())),
                2 => {
                    app.ask = Some(super::Ask::ClearPads {
                        pads: vec![0],
                        lines: vec!["question".into()],
                    })
                }
                3 => {
                    app.apply(Action::LearnToggle);
                }
                4 => app.post_notice(Level::Warn, ("higher priority".into(), Instant::now())),
                5 => app.post_startup(Level::Warn, "old startup notice".into()),
                6 => app.pads.loading[0] = Some(0),
                7 => app.recording = Some((super::audio::SOURCE_OUTPUT, std::time::Instant::now())),
                _ => unreachable!(),
            }
            // Ctrl+U is unbound and is not one of the migrated chords.
            NoticeInput::default().event(
                &mut app,
                &keymap,
                KeyEvent::new(KeyCode::Char('u'), M::CONTROL),
                true,
            );
            assert_eq!(app.keymap_warnings.len(), 1, "hidden in mode {mode}");
            app.screen = Screen::Pads;
            app.browser.editing = false;
            app.pad_naming = None;
            app.ask = None;
            app.screen_learn = None;
            app.notices = Notices::new(Instant::now());
            app.pads.loading[0] = None;
            app.recording = None;
            assert!(
                !app.notices.has_startup(),
                "the old startup dismissal rule stays"
            );
            assert_eq!(app.frame_notice().as_deref(), Some("pending warning"));
        }
        // An expired flash must not acknowledge a warning that no frame showed yet.
        let mut app = test_app();
        app.keymap_warnings.push_back("not shown yet".into());
        app.post_notice(Level::Warn, ("flash".into(), Instant::now()));
        assert_eq!(app.frame_notice().as_deref(), Some("flash"));
        app.notices = Notices::new(Instant::now());
        let mut input = NoticeInput::default();
        assert!(!input.dispatch(
            &mut app,
            &keymap,
            KeyEvent::new(KeyCode::Char('5'), M::NONE),
            true
        ));
        assert_eq!(app.keymap_warnings.len(), 1);
    }

    // Outcome assertions inspect production history, not the priority-selected line.
    fn last_notice(app: &super::App) -> Option<&str> {
        app.notices
            .history()
            .front()
            .map(|notice| notice.text.as_str())
    }

    fn flash(app: &super::App) -> &str {
        last_notice(app).unwrap_or_default()
    }

    /// One second of a stereo tone as the ring holds it.
    fn ring() -> Vec<f32> {
        (0..48_000)
            .flat_map(|i| {
                let s = (i as f32 * 0.05).sin() * 0.5;
                [s, -s]
            })
            .collect()
    }

    #[test]
    fn bind_to_a_pad() {
        let dir = tmp_dir("bind");
        let mut app = test_app();
        app.keep_recording(&ring(), false, &dir);
        assert_eq!(flash(&app), "Recorded 1.0 s · hit a pad");
        let first = app
            .browser
            .in_hand
            .clone()
            .expect("the recording is in hand");
        assert!(first.starts_with(&dir) && first.exists());
        // pad Q is empty: the hit binds at once
        app.put_in_hand(4);
        assert!(app.ask.is_none() && app.browser.in_hand.is_none());
        wait_for(&mut app, |a| a.pads.loading[4].is_none());
        assert_eq!(app.pads.file[4].as_ref(), Some(&first));
        // a second recording within the same second gets its own name, the first is kept
        app.keep_recording(&ring(), false, &dir);
        let second = app.browser.in_hand.clone().unwrap();
        assert_ne!(second, first);
        assert!(first.exists() && second.exists());
        // pad Q holds the first one: the replace question comes first, Esc keeps the pad
        app.put_in_hand(4);
        assert!(app.ask.is_some(), "the replace question");
        assert_eq!(app.pads.file[4].as_ref(), Some(&first));
        app.apply(Action::Back);
        assert!(app.ask.is_none());
        assert_eq!(app.pads.file[4].as_ref(), Some(&first));
        // the hit again and Enter replace it
        app.put_in_hand(4);
        app.apply(Action::PadEdit);
        wait_for(&mut app, |a| a.pads.loading[4].is_none());
        assert_eq!(app.pads.file[4].as_ref(), Some(&second));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn kept_over_a_restart() {
        let dir = tmp_dir("restart");
        let mut app = test_app();
        app.keep_recording(&ring(), false, &dir);
        let path = app.browser.in_hand.clone().unwrap();
        app.put_in_hand(4);
        wait_for(&mut app, |a| a.pads.loading[4].is_none());
        let file = dir.join("session.toml");
        super::session::save(&file, &app.pads.to_session(None, 48_000)).unwrap();
        // the restart: a fresh program reads the session back
        let mut pads = super::pads::Pads::default();
        pads.restore(&super::session::load(&file).unwrap(), 48_000);
        assert_eq!(pads.file[4].as_ref(), Some(&path));
        assert!(path.exists(), "the file is in the samples root");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn silence_is_not_kept() {
        let dir = tmp_dir("silence");
        let mut app = test_app();
        app.keep_recording(&vec![0.0; 48_000 * 2], false, &dir);
        assert_eq!(flash(&app), "The recording was silent: nothing is kept.");
        assert!(app.browser.in_hand.is_none());
        assert!(!dir.exists(), "no file appears");
    }

    #[test]
    fn half_a_minute_names_the_cap() {
        let dir = tmp_dir("cap");
        let mut app = test_app();
        app.keep_recording(&ring(), true, &dir);
        assert_eq!(
            flash(&app),
            "Recorded 1.0 s, stopped at the 30 s cap · hit a pad"
        );
        assert!(app.browser.in_hand.is_some());
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Scenario "The manual save speaks" (22): `Ctrl+S` saves the session right away and
    /// the notice line says "Session saved" for 5 s.
    #[test]
    fn ctrl_s_saves_the_session_and_says_so() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = test_app();
        let file = tmp_dir("save").join("session.toml");
        app.session_path = file.clone();
        app.session_read_error = None;
        app.dirty();
        let ctrl_s = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Char('s'),
            ratatui::crossterm::event::KeyModifiers::CONTROL,
        );
        app.apply(keymap.resolve(input::Screen::Pads, &ctrl_s).unwrap());
        assert_eq!(flash(&app), "Session saved");
        assert!(file.exists(), "the session was written");
        std::fs::remove_dir_all(file.parent().unwrap()).ok();
    }

    /// Scenario "The autosave is silent" (22): the idle save writes the session and says
    /// nothing; the hints and the screen stay as they were.
    #[test]
    fn the_autosave_stays_silent() {
        let mut app = test_app();
        let file = tmp_dir("autosave").join("session.toml");
        app.session_path = file.clone();
        app.session_read_error = None;
        app.dirty();
        app.save_session().unwrap();
        assert_eq!(flash(&app), "", "no message after the autosave");
        assert!(file.exists());
        std::fs::remove_dir_all(file.parent().unwrap()).ok();
    }

    /// Scenario "Saved into the project" (11): with a project open the session goes into
    /// the project's `project.toml` with the paths inside the folder stored relative, and
    /// the scratch session file stays untouched.
    #[test]
    fn saved_into_the_project() {
        let tmp = tmp_dir("proj-save");
        let pj = tmp.join("pj");
        std::fs::create_dir_all(pj.join("samples")).unwrap();
        std::fs::write(pj.join("samples/a.wav"), b"").unwrap();
        let scratch = tmp.join("scratch");
        let mut app = test_app();
        app.project = Some(super::Project {
            name: "MyBeat".into(),
            folder: pj.clone(),
        });
        app.session_path = scratch.join("session.toml");
        app.session_read_error = None;
        app.pads.file[4] = Some(pj.join("samples/a.wav"));
        app.save_session().unwrap();
        let toml = std::fs::read_to_string(pj.join("project.toml")).unwrap();
        assert!(toml.contains("samples/a.wav"), "stored relative: {toml}");
        assert!(
            !toml.contains(pj.to_string_lossy().as_ref()),
            "no absolute paths of the project: {toml}"
        );
        assert!(
            !scratch.join("session.toml").exists(),
            "the scratch file is untouched"
        );
        std::fs::remove_dir_all(&tmp).ok();
    }

    /// Scenario "Restart" (11): the saved `project.toml` reads back through the stored
    /// paths of the project — the pad holds the absolute path inside the folder again.
    #[test]
    fn restart() {
        let tmp = tmp_dir("proj-restart");
        let pj = tmp.join("pj");
        std::fs::create_dir_all(pj.join("samples")).unwrap();
        std::fs::write(pj.join("samples/a.wav"), b"").unwrap();
        let mut app = test_app();
        app.project = Some(super::Project {
            name: "MyBeat".into(),
            folder: pj.clone(),
        });
        app.session_path = tmp.join("scratch/session.toml");
        app.session_read_error = None;
        app.pads.file[4] = Some(pj.join("samples/a.wav"));
        app.save_session().unwrap();
        let sess = super::with_resolved_paths(
            super::session::load(&pj.join("project.toml")).unwrap(),
            Some(pj.as_path()),
        );
        assert_eq!(
            sess.pads["4"].file.as_ref(),
            Some(&pj.join("samples/a.wav")),
            "absolute again after the round-trip"
        );
        std::fs::remove_dir_all(&tmp).ok();
    }

    /// The integration twin of "Outside stays absolute and untouched" (11): a sample
    /// outside the project folder keeps its absolute path in `project.toml` and is not
    /// copied into the project.
    #[test]
    fn outside_stays_absolute_in_the_project() {
        let tmp = tmp_dir("proj-outside");
        let pj = tmp.join("pj");
        let outside = tmp.join("outside");
        std::fs::create_dir_all(pj.join("samples")).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("amen.wav"), b"").unwrap();
        let mut app = test_app();
        app.project = Some(super::Project {
            name: "MyBeat".into(),
            folder: pj.clone(),
        });
        app.session_path = tmp.join("scratch/session.toml");
        app.session_read_error = None;
        app.pads.file[4] = Some(outside.join("amen.wav"));
        app.save_session().unwrap();
        let toml = std::fs::read_to_string(pj.join("project.toml")).unwrap();
        assert!(
            toml.contains(outside.join("amen.wav").to_string_lossy().as_ref()),
            "stored absolute: {toml}"
        );
        assert!(
            !pj.join("samples/amen.wav").exists(),
            "the file is not copied into the project"
        );
        std::fs::remove_dir_all(&tmp).ok();
    }

    /// Scenario "Restart with a project" (11): the config keeps the last opened name;
    /// the next start finds the folder with its `project.toml` in the projects root and
    /// restores the pads from the stored relative paths.
    #[test]
    fn restart_with_a_project() {
        let dir = tmp_dir("proj-reopen");
        let cfg = dir.join("config.toml");
        let root = dir.join("projects");
        let mut app = test_app();
        app.paths.projects = root.clone();
        app.user_path = Some(cfg.clone());
        app.create_project("MyBeat".into());
        assert_eq!(
            app.project.as_ref().map(|p| p.name.as_str()),
            Some("MyBeat"),
            "the project opened"
        );
        // a pad plays a sample inside the project, stored as a relative path
        let pj = root.join("MyBeat");
        std::fs::write(pj.join("samples/a.wav"), b"").unwrap();
        let mut pads = std::collections::BTreeMap::new();
        pads.insert(
            "0".to_string(),
            super::session::PadState {
                file: Some("samples/a.wav".into()),
                ..Default::default()
            },
        );
        super::session::save(
            &pj.join("project.toml"),
            &super::session::Session {
                pads,
                ..Default::default()
            },
        )
        .unwrap();

        // the program starts again: the config carries the name, the project is found
        let loaded: super::config::Config =
            toml::from_str(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
        assert_eq!(loaded.project.last.as_deref(), Some("MyBeat"));
        let paths = super::config::PathsConfig {
            projects: root.clone(),
            ..Default::default()
        };
        let start = super::initial_project(loaded.project.last.as_deref(), &paths)
            .expect("the project reopens");
        assert_eq!(start.name, "MyBeat");
        assert_eq!(start.folder, pj);

        // its session comes back: the stored relative path resolves against the folder
        let sess = super::with_resolved_paths(
            super::session::load(&start.folder.join("project.toml")).unwrap(),
            Some(start.folder.as_path()),
        );
        let mut app = test_app();
        app.project = Some(start);
        app.apply_session(sess);
        wait_for(&mut app, |a| a.pads.loading[0].is_none());
        assert_eq!(
            app.pads.file[0].as_ref(),
            Some(&pj.join("samples/a.wav")),
            "the pad plays the project's sample again"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Scenario "First start is scratch" (11): no project ever opened — and a name whose
    /// folder is gone — leave the start in the scratch session.
    #[test]
    fn first_start_is_scratch() {
        let root = tmp_dir("proj-scratch");
        let paths = super::config::PathsConfig {
            projects: root.clone(),
            ..Default::default()
        };
        assert!(super::initial_project(None, &paths).is_none());
        assert!(
            super::initial_project(Some("Gone"), &paths).is_none(),
            "a name without a folder on disk is not a project"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    /// Scenario "First start in the samples root" (11): no project is open and the
    /// session carries no folder — the files screen opens in the samples root.
    #[test]
    fn first_start_in_the_samples_root() {
        let root = tmp_dir("first-samples-root");
        std::fs::create_dir_all(&root).unwrap();
        let mut app = test_app();
        app.paths.samples = root.clone();
        app.apply_session(super::session::Session::default());
        assert_eq!(
            app.browser.dir, root,
            "the first start opens the samples root"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    /// Scenario "The project's samples come first" (11): with a project open the files
    /// screen starts in its `samples/` folder; a folder saved in the session — the one
    /// the user walked to — wins over the default.
    #[test]
    fn the_projects_samples_come_first() {
        let root = tmp_dir("proj-samples-first");
        let pj = root.join("MyBeat");
        std::fs::create_dir_all(pj.join("samples")).unwrap();
        // material in it: an empty samples/ keeps the folder browsed last (35)
        std::fs::write(pj.join("samples/rec.wav"), b"").unwrap();
        let walked = root.join("walked");
        std::fs::create_dir_all(&walked).unwrap();

        let mut app = test_app();
        app.project = Some(super::Project {
            name: "MyBeat".into(),
            folder: pj.clone(),
        });
        app.apply_session(super::session::Session::default());
        assert_eq!(
            app.browser.dir,
            pj.join("samples"),
            "the files screen starts in the project's samples"
        );

        app.apply_session(super::session::Session {
            browser_dir: Some(walked.clone()),
            ..Default::default()
        });
        assert_eq!(
            app.browser.dir, walked,
            "the folder saved in the session wins over the default"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    /// Scenario "Folder deleted while open" (11): the open project's folder vanishes; the
    /// save fails, the pads with files inside the folder are marked missing, the title
    /// drops the name and the program keeps working.
    #[test]
    fn folder_deleted_while_open() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let key = |code: KeyCode| KeyEvent::new(code, KeyModifiers::empty());
        let root = tmp_dir("proj-gone-root");
        let pj = root.join("MyBeat");
        std::fs::create_dir_all(pj.join("samples")).unwrap();
        std::fs::write(pj.join("samples/a.wav"), b"").unwrap();
        let mut pads = std::collections::BTreeMap::new();
        pads.insert(
            "0".to_string(),
            super::session::PadState {
                file: Some("samples/a.wav".into()),
                ..Default::default()
            },
        );
        super::session::save(
            &pj.join("project.toml"),
            &super::session::Session {
                pads,
                ..Default::default()
            },
        )
        .unwrap();
        let mut app = test_app();
        app.paths.projects = root.clone();
        app.apply(Action::ProjectPick);
        assert!(app.project_picker_key(&key(KeyCode::Enter)));
        wait_for(&mut app, |a| a.pads.loading[0].is_none());
        assert!(app.session_read_error.is_none());
        // a pad outside the project folder keeps its file through the deletion
        let outside_dir = tmp_dir("proj-gone-outside");
        std::fs::create_dir_all(&outside_dir).unwrap();
        let outside = outside_dir.join("keep.wav");
        std::fs::write(&outside, b"").unwrap();
        app.pads.file[1] = Some(outside.clone());

        std::fs::remove_dir_all(&pj).unwrap();
        app.dirty();
        assert!(
            app.save_session().is_err(),
            "the save fails without the folder"
        );
        assert!(!pj.exists(), "the save does not resurrect the folder");
        assert!(app.pads.missing[0], "the pad inside the folder is missing");
        assert!(!app.pads.missing[1], "the pad outside keeps its file");
        assert_eq!(app.window_title(), "ptsr", "the title drops the name");
        assert!(app.session_write_failed);
        assert!(app.session_dirty.is_some());
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&outside_dir).ok();
    }

    /// Scenario "Pick the roots" (11): Enter on a Paths row opens the folder picker,
    /// walking into a folder and Enter picks it — the root and the `[paths]` section
    /// of the config take the picked folder.
    #[test]
    fn pick_the_roots() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let key = |code: KeyCode| KeyEvent::new(code, KeyModifiers::empty());
        let mut app = test_app();
        let dir = tmp_dir("pick-roots");
        app.user_path = Some(dir.join("config.toml"));
        let root = tmp_dir("pick-roots-root");
        std::fs::create_dir_all(root.join("Field")).unwrap();
        app.paths.projects = root.clone();
        // F12, three Tabs to the Paths tab, Enter opens the picker of the projects root
        app.apply(
            keymap
                .resolve(input::Screen::Pads, &key(KeyCode::F(12)))
                .unwrap(),
        );
        for _ in 0..3 {
            app.apply(
                keymap
                    .resolve(input::Screen::Settings, &key(KeyCode::Tab))
                    .unwrap(),
            );
        }
        app.apply(
            keymap
                .resolve(input::Screen::Settings, &key(KeyCode::Enter))
                .unwrap(),
        );
        assert!(app.path_picker.is_some(), "the picker is open");
        // walk into Field and pick it
        app.apply(
            keymap
                .resolve(input::Screen::Settings, &key(KeyCode::Right))
                .unwrap(),
        );
        app.apply(
            keymap
                .resolve(input::Screen::Settings, &key(KeyCode::Enter))
                .unwrap(),
        );
        assert!(
            app.path_picker.is_none(),
            "the picker is closed after the pick"
        );
        assert_eq!(app.paths.projects, root.join("Field"));
        let text = std::fs::read_to_string(dir.join("config.toml")).unwrap();
        assert!(text.contains("[paths]"), "{text}");
        assert!(text.contains("Field"), "{text}");
        assert!(flash(&app).starts_with("Projects root"), "{}", flash(&app));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::remove_dir_all(&root).ok();
    }

    /// Scenario "The choice survives a restart" (11): the config written by the pick
    /// loads back with the same roots, the untouched one keeps its default.
    #[test]
    fn the_choice_survives_a_restart() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let key = |code: KeyCode| KeyEvent::new(code, KeyModifiers::empty());
        let mut app = test_app();
        let dir = tmp_dir("restart-roots");
        app.user_path = Some(dir.join("config.toml"));
        let root = tmp_dir("restart-roots-root");
        std::fs::create_dir_all(root.join("Field")).unwrap();
        app.paths.projects = root.clone();
        app.apply(
            keymap
                .resolve(input::Screen::Pads, &key(KeyCode::F(12)))
                .unwrap(),
        );
        for _ in 0..3 {
            app.apply(
                keymap
                    .resolve(input::Screen::Settings, &key(KeyCode::Tab))
                    .unwrap(),
            );
        }
        app.apply(
            keymap
                .resolve(input::Screen::Settings, &key(KeyCode::Enter))
                .unwrap(),
        );
        app.apply(
            keymap
                .resolve(input::Screen::Settings, &key(KeyCode::Right))
                .unwrap(),
        );
        app.apply(
            keymap
                .resolve(input::Screen::Settings, &key(KeyCode::Enter))
                .unwrap(),
        );
        let c: super::config::Config =
            toml::from_str(&std::fs::read_to_string(dir.join("config.toml")).unwrap()).unwrap();
        assert_eq!(c.paths.projects, root.join("Field"));
        assert_eq!(
            c.paths.samples,
            super::config::PathsConfig::default().samples,
            "the untouched root keeps its default"
        );
        std::fs::remove_dir_all(&dir).ok();
        std::fs::remove_dir_all(&root).ok();
    }

    /// Scenario "Open a project" (11): Ctrl+P lists the folders of the projects root that
    /// hold a `project.toml`; Enter opens one — the pad's stored relative path comes back
    /// absolute and the title carries the project's name.
    #[test]
    fn open_a_project() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let key = |code: KeyCode| KeyEvent::new(code, KeyModifiers::empty());
        let root = tmp_dir("proj-open-root");
        let pj = root.join("MyBeat");
        std::fs::create_dir_all(pj.join("samples")).unwrap();
        std::fs::write(pj.join("samples/a.wav"), b"").unwrap();
        let mut pads = std::collections::BTreeMap::new();
        pads.insert(
            "0".to_string(),
            super::session::PadState {
                file: Some("samples/a.wav".into()),
                ..Default::default()
            },
        );
        super::session::save(
            &pj.join("project.toml"),
            &super::session::Session {
                pads,
                ..Default::default()
            },
        )
        .unwrap();
        let mut app = test_app();
        app.paths.projects = root.clone();
        app.apply(Action::ProjectPick);
        let picker = app.project_picker.as_ref().expect("the picker is open");
        assert_eq!(picker.entries, vec!["MyBeat".to_string()]);
        assert!(app.project_picker_key(&key(KeyCode::Enter)));
        assert!(app.project_picker.is_none(), "the picker is closed");
        assert_eq!(
            app.project.as_ref().map(|p| p.name.as_str()),
            Some("MyBeat")
        );
        wait_for(&mut app, |a| a.pads.loading[0].is_none());
        assert_eq!(
            app.pads.file[0].as_ref(),
            Some(&pj.join("samples/a.wav")),
            "the stored relative path resolved against the folder"
        );
        assert!(
            app.window_title().contains("MyBeat"),
            "{}",
            app.window_title()
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn rename_a_pad_cancel_the_pad_name_and_restore_the_file_name() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let mut app = test_app();
        app.pads.file[0] = Some("kick.wav".into());
        app.apply(Action::PadRename);
        assert_eq!(app.pad_naming.as_ref().unwrap().1, "kick");
        for _ in 0..4 {
            app.pad_name_key(&KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        }
        for c in "Бочка".chars() {
            assert!(app.pad_name_key(&KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)));
        }
        app.pad_name_key(&KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(super::pad_name(&app.pads, 0), "Бочка");
        app.apply(Action::PadRename);
        app.pad_name_key(&KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        app.pad_name_key(&KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(super::pad_name(&app.pads, 0), "Бочка");
        app.apply(Action::PadRename);
        for _ in 0..5 {
            app.pad_name_key(&KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        }
        app.pad_name_key(&KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(super::pad_name(&app.pads, 0), "kick");
    }

    #[test]
    fn see_the_pad_name_while_typing() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let mut app = test_app();
        app.pads.file[0] = Some("kick.wav".into());
        app.apply(Action::PadRename);
        app.pad_name_key(&KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE));
        let mut view = super::pads_view(&mut app.pads, None, None, Vec::new(), None, app.rate);
        super::preview_pad_name(&mut view, app.pads.bank, app.pad_naming.as_ref());
        assert_eq!(view.pads[0].name, "kick!");
        assert!(view.pads[0].naming);
        assert_eq!(app.pads.custom_name[0], None);
        app.pad_name_key(&KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(super::pad_name(&app.pads, 0), "kick");
    }

    #[test]
    fn pad_name_after_restart() {
        let root = tmp_dir("pad-name-restart");
        let path = root.join("session.toml");
        let mut app = test_app();
        app.pads.file[0] = Some("kick.wav".into());
        app.pads.custom_name[0] = Some("Бочка".into());
        super::session::save(&path, &app.pads.to_session(None, app.rate)).unwrap();
        let session = super::session::load(&path).unwrap();
        let mut pads = super::pads::Pads::default();
        pads.restore(&session, app.rate);
        assert_eq!(super::pad_name(&pads, 0), "Бочка");
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn swap_named_pads_and_clear_and_undo_a_named_pad() {
        let mut app = test_app();
        app.pads.file[0] = Some("kick.wav".into());
        app.pads.custom_name[0] = Some("Бочка".into());
        app.pads.swap(0, 1).unwrap();
        assert_eq!(app.pads.custom_name[1].as_deref(), Some("Бочка"));
        let before = app.before();
        app.pads.clear(1);
        app.note_step(before, "clear".into());
        assert_eq!(app.pads.custom_name[1], None);
        app.undo_pads();
        assert_eq!(app.pads.custom_name[1].as_deref(), Some("Бочка"));
    }

    #[test]
    fn replace_a_named_pad_sound_and_a_failed_replacement_keeps_the_name() {
        let mut app = test_app();
        app.pads.file[0] = Some("old.wav".into());
        app.pads.custom_name[0] = Some("kick".into());
        let sound = std::sync::Arc::new(Sample::mono(vec![0.1; 64]));
        app.pads.sound[0] = Some(sound.clone());
        app.load_gen[0] = 1;
        app.loader_reply(super::loader::Reply::Failed {
            pad: 0,
            generation: 1,
            path: "new.wav".into(),
            error: "failed".into(),
        });
        assert_eq!(app.pads.custom_name[0].as_deref(), Some("kick"));
        app.loader_reply(super::loader::Reply::Loaded {
            pad: 0,
            generation: 1,
            path: "new.wav".into(),
            sound,
        });
        assert_eq!(app.pads.custom_name[0], None);
        assert_eq!(super::pad_name(&app.pads, 0), "new");
    }

    /// 79: A late sample from the old project must not fill or sound an empty pad.
    #[test]
    fn a_late_sample_from_the_old_project() {
        let root = tmp_dir("late-project-sample");
        let mut app = test_app();
        app.paths.projects = root.clone();
        app.project = Some(super::Project {
            name: "old".into(),
            folder: root.join("old"),
        });
        let (requests, incoming) = std::sync::mpsc::channel();
        let (_replies, replies) = std::sync::mpsc::channel();
        app.loader = Some((requests, replies));
        let (audio, mut rt) = super::audio::offline(48_000);
        app.audio = Some(audio);

        let path = root.join("old/sample.wav");
        app.load_pad(0, path.clone(), Region::WHOLE);
        let request = incoming.try_recv().unwrap();
        app.create_project("empty".into());
        assert_eq!(app.project.as_ref().unwrap().name, "empty");
        assert!(app.pads.sound[0].is_none());
        let mut left = vec![0.0; 4096];
        let mut right = vec![0.0; 4096];
        for _ in 0..16 {
            rt.process(&mut left, &mut right);
        }

        app.loader_reply(super::loader::Reply::Loaded {
            pad: request.pad,
            generation: request.generation,
            path,
            sound: std::sync::Arc::new(Sample::mono(vec![0.5; 2048])),
        });
        app.apply(Action::PadPress {
            pad: 0,
            velocity: 1.0,
        });
        rt.process(&mut left, &mut right);
        let peak = left
            .iter()
            .chain(&right)
            .map(|s| s.abs())
            .fold(0.0_f32, f32::max);
        let empty = app.pads.file[0].is_none() && app.pads.sound[0].is_none();
        std::fs::remove_dir_all(&root).ok();
        assert!(
            empty && peak < 1e-6,
            "late sample filled the pad: {}; peak: {peak}",
            !empty
        );
    }

    #[test]
    fn a_late_sample_from_the_old_project_does_not_consume_new_followers() {
        let mut app = test_app();
        let (requests, incoming) = std::sync::mpsc::channel();
        let (_replies, replies) = std::sync::mpsc::channel();
        app.loader = Some((requests, replies));
        let old = std::path::PathBuf::from("old.wav");
        let new = std::path::PathBuf::from("new.wav");
        app.load_pad(0, old.clone(), Region::WHOLE);
        let old_request = incoming.try_recv().unwrap();
        app.apply_session(super::session::Session::default());
        app.load_pad(0, new.clone(), Region::WHOLE);
        let new_request = incoming.try_recv().unwrap();
        app.follow(0, 1, Region::WHOLE);

        app.loader_reply(super::loader::Reply::Failed {
            pad: 0,
            generation: old_request.generation,
            path: old.clone(),
            error: "old error".into(),
        });
        app.loader_reply(super::loader::Reply::Loaded {
            pad: 0,
            generation: old_request.generation,
            path: old,
            sound: std::sync::Arc::new(Sample::mono(vec![0.5; 16])),
        });
        assert!(app.pads.sound[1].is_none());
        assert_eq!(app.followers.len(), 1);

        app.loader_reply(super::loader::Reply::Loaded {
            pad: 0,
            generation: new_request.generation,
            path: new.clone(),
            sound: std::sync::Arc::new(Sample::mono(vec![0.25; 16])),
        });
        assert_eq!(app.pads.file[1].as_ref(), Some(&new));
        assert!(app.pads.sound[1].is_some());
        assert!(app.followers.is_empty());
    }

    #[test]
    fn chop_preview_while_the_project_opens_does_not_play_later() {
        use std::sync::atomic::Ordering::Acquire;
        let mut app = test_app();
        let (audio, mut rt) = super::audio::offline(48_000);
        app.audio = Some(audio);
        app.apply_session(super::session::Session::default());
        app.screen = Screen::Chop;
        let mut chop = chop_with_parts(0);
        chop.sound = std::sync::Arc::new(Sample::mono(vec![0.5; 96_000]));
        app.chop = Some(chop);
        app.apply(Action::ChopPlay);
        app.apply(Action::PadPress {
            pad: 0,
            velocity: 1.0,
        });
        app.midi_event(super::midi::Event::PadHit {
            sound: super::midi::PadSound::DirectRejected,
            abs: 0,
            velocity: 1.0,
            ui_press: None,
        });
        let delayed = super::midi::UiEvent::stamped(
            super::midi::Event::Action(Action::ChopPlay),
            &app.audio.as_ref().unwrap().project,
        );
        let mut left = vec![0.0; 4096];
        let mut right = left.clone();
        for _ in 0..300 {
            app.flush_audio();
            rt.process(&mut left, &mut right);
            assert!(left.iter().chain(&right).all(|s| s.abs() < 1e-6));
            if app.audio.as_ref().unwrap().project.ready.load(Acquire) == 1
                && app.pending_audio.is_empty()
            {
                break;
            }
        }
        assert_eq!(app.audio.as_ref().unwrap().project.ready.load(Acquire), 1);
        app.midi_ui_event(delayed);
        rt.process(&mut left, &mut right);
        assert!(left.iter().chain(&right).all(|s| s.abs() < 1e-6));
        app.apply(Action::ChopPlay);
        rt.process(&mut left, &mut right);
        assert!(left.iter().chain(&right).any(|s| s.abs() > 1e-6));
    }

    #[test]
    fn project_ready_waits_for_loaded_slices_and_failed_followers() {
        use std::sync::atomic::Ordering::Acquire;
        let root = tmp_dir("project-ready-loads");
        std::fs::create_dir_all(&root).unwrap();
        let first = root.join("first.wav");
        let second = root.join("second.wav");
        std::fs::write(&first, b"placeholder").unwrap();
        std::fs::write(&second, b"placeholder").unwrap();
        let mut session = super::session::Session::default();
        for (pad, path) in [(0, &first), (1, &first), (2, &second)] {
            session.pads.insert(
                pad.to_string(),
                super::session::PadState {
                    file: Some(path.clone()),
                    start: Some(0.01),
                    end: Some(0.02),
                    ..Default::default()
                },
            );
        }
        let mut app = test_app();
        let (requests, incoming) = std::sync::mpsc::channel();
        let (_replies, replies) = std::sync::mpsc::channel();
        app.loader = Some((requests, replies));
        let (audio, mut rt) = super::audio::offline(48_000);
        app.audio = Some(audio);
        app.apply_session(session);
        let one = incoming.try_recv().unwrap();
        let two = incoming.try_recv().unwrap();
        let mut left = vec![0.0; 4096];
        let mut right = left.clone();
        for _ in 0..40 {
            app.flush_audio();
            rt.process(&mut left, &mut right);
        }
        assert_ne!(app.audio.as_ref().unwrap().project.ready.load(Acquire), 1);
        app.loader_reply(super::loader::Reply::Loaded {
            pad: one.pad,
            generation: one.generation,
            path: one.path,
            sound: std::sync::Arc::new(Sample::mono(vec![0.5; 2048])),
        });
        for _ in 0..40 {
            app.flush_audio();
            rt.process(&mut left, &mut right);
        }
        assert_ne!(app.audio.as_ref().unwrap().project.ready.load(Acquire), 1);
        app.loader_reply(super::loader::Reply::Failed {
            pad: two.pad,
            generation: two.generation,
            path: two.path,
            error: "bad file".into(),
        });
        for _ in 0..40 {
            app.flush_audio();
            rt.process(&mut left, &mut right);
        }
        assert_eq!(app.audio.as_ref().unwrap().project.ready.load(Acquire), 1);
        assert!(app.pads.missing[2]);
        for pad in [0, 1] {
            assert!(app.pads.sound[pad].is_some());
            assert_ne!(app.pads.region[pad], Region::WHOLE);
        }
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn project_ready_ignores_late_load_after_second_switch() {
        use std::sync::atomic::Ordering::Acquire;
        let root = tmp_dir("project-ready-switch");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("sound.wav");
        std::fs::write(&path, b"placeholder").unwrap();
        let mut session = super::session::Session::default();
        session.pads.insert(
            "0".into(),
            super::session::PadState {
                file: Some(path.clone()),
                ..Default::default()
            },
        );
        let mut app = test_app();
        let (requests, incoming) = std::sync::mpsc::channel();
        let (_replies, replies) = std::sync::mpsc::channel();
        app.loader = Some((requests, replies));
        let (audio, mut rt) = super::audio::offline(48_000);
        app.audio = Some(audio);
        app.apply_session(session);
        let old = incoming.try_recv().unwrap();
        app.apply_session(super::session::Session::default());
        app.loader_reply(super::loader::Reply::Loaded {
            pad: old.pad,
            generation: old.generation,
            path,
            sound: std::sync::Arc::new(Sample::mono(vec![0.5; 2048])),
        });
        let mut left = vec![0.0; 4096];
        let mut right = left.clone();
        for _ in 0..300 {
            app.flush_audio();
            rt.process(&mut left, &mut right);
            if app.audio.as_ref().unwrap().project.ready.load(Acquire) == 2 {
                break;
            }
        }
        assert_eq!(app.audio.as_ref().unwrap().project.ready.load(Acquire), 2);
        assert!(app.pads.sound[0].is_none());
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn project_ready_when_loader_is_unavailable() {
        let root = tmp_dir("project-ready-no-loader");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("sound.wav");
        std::fs::write(&path, b"placeholder").unwrap();
        let mut session = super::session::Session::default();
        for pad in 0..2 {
            session.pads.insert(
                pad.to_string(),
                super::session::PadState {
                    file: Some(path.clone()),
                    ..Default::default()
                },
            );
        }
        let mut app = test_app();
        app.loader = None;
        app.apply_session(session);
        assert!(!app.restoring_project);
        assert!(app.pads.missing[0] && app.pads.missing[1]);
        std::fs::remove_dir_all(root).ok();
    }

    /// 79: A sounding slice fades on a switch and plays again on return.
    #[test]
    fn playing_sounds_stop_at_a_project_switch_and_return_to_a_project_with_sounds() {
        use engine::checks::{first_loud, first_non_finite, max_step, peak};
        use std::sync::{Arc, atomic::Ordering::Acquire};

        const BLOCK: usize = 128;
        const RATE: u32 = 48_000;
        fn block(
            app: &mut super::App,
            rt: &mut engine::rt::Realtime,
            l: &mut Vec<f32>,
            r: &mut Vec<f32>,
        ) {
            app.flush_audio();
            let mut left = [0.0; BLOCK];
            let mut right = [0.0; BLOCK];
            rt.process(&mut left, &mut right);
            l.extend_from_slice(&left);
            r.extend_from_slice(&right);
            app.audio.as_mut().unwrap().free_garbage();
        }

        let root = tmp_dir("render-project-switch");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("slice.wav");
        std::fs::write(&path, b"placeholder").unwrap();
        let tone = Arc::new(Sample::mono(
            (0..RATE as usize * 2)
                .map(|i| 0.3 * (i as f32 * 110.0 * std::f32::consts::TAU / RATE as f32).sin())
                .collect(),
        ));
        let mut session = super::session::Session::default();
        session.pads.insert(
            "0".into(),
            super::session::PadState {
                file: Some(path.clone()),
                start: Some(0.1),
                end: Some(1.5),
                volume_db: -6,
                ..Default::default()
            },
        );
        let mut app = test_app();
        let (requests, incoming) = std::sync::mpsc::channel();
        let (_replies, replies) = std::sync::mpsc::channel();
        app.loader = Some((requests, replies));
        let (audio, mut rt) = super::audio::offline(RATE);
        app.audio = Some(audio);
        let mut left = Vec::new();
        let mut right = Vec::new();
        let load = |app: &mut super::App| {
            let request = incoming.try_recv().unwrap();
            app.loader_reply(super::loader::Reply::Loaded {
                pad: request.pad,
                generation: request.generation,
                path: request.path,
                sound: tone.clone(),
            });
        };
        let session_path = root.join("project.toml");
        super::session::save(&session_path, &session).unwrap();
        app.apply_session(super::session::load(&session_path).unwrap());
        load(&mut app);
        for _ in 0..1000 {
            block(&mut app, &mut rt, &mut left, &mut right);
            if app.audio.as_ref().unwrap().project.ready.load(Acquire) == 1 {
                break;
            }
        }
        assert_eq!(app.audio.as_ref().unwrap().project.ready.load(Acquire), 1);
        left.clear();
        right.clear();
        let press = |app: &mut super::App| {
            app.apply(Action::PadPress {
                pad: 0,
                velocity: 1.0,
            });
        };
        press(&mut app);
        for _ in 0..113 {
            block(&mut app, &mut rt, &mut left, &mut right);
        }
        let switch = left.len();
        assert!(
            peak(&left[switch - 1024..switch]) > 0.05,
            "slice is still playing"
        );
        app.apply_session(super::session::Session::default());
        press(&mut app); // A hit before readiness must not play later.
        for _ in 0..1000 {
            block(&mut app, &mut rt, &mut left, &mut right);
            if app.audio.as_ref().unwrap().project.ready.load(Acquire) == 2 {
                break;
            }
        }
        assert_eq!(app.audio.as_ref().unwrap().project.ready.load(Acquire), 2);
        for _ in 0..75 {
            block(&mut app, &mut rt, &mut left, &mut right);
        }
        press(&mut app);
        for _ in 0..75 {
            block(&mut app, &mut rt, &mut left, &mut right);
        }
        let return_switch = left.len();
        app.apply_session(session);
        load(&mut app);
        for _ in 0..1000 {
            block(&mut app, &mut rt, &mut left, &mut right);
            if app.audio.as_ref().unwrap().project.ready.load(Acquire) == 3 {
                break;
            }
        }
        assert_eq!(app.audio.as_ref().unwrap().project.ready.load(Acquire), 3);
        let return_hit = left.len();
        press(&mut app);
        for _ in 0..113 {
            block(&mut app, &mut rt, &mut left, &mut right);
        }
        for channel in [&left, &right] {
            assert!(first_non_finite(channel).is_none());
            assert!(peak(channel) <= 1.0, "no clipping");
            assert!(max_step(channel) < 0.01, "click: {}", max_step(channel));
            assert!(
                peak(&channel[switch + 960..return_hit]) < 1e-6,
                "silence after fade and empty-pad hit"
            );
            assert!(
                peak(&channel[return_hit + 1024..]) > 0.05,
                "slice plays after return"
            );
            let first = first_loud(channel, 0..1024, 1e-6).unwrap();
            let again =
                first_loud(channel, return_hit..return_hit + 1024, 1e-6).unwrap() - return_hit;
            assert!(
                first <= 2 && first.abs_diff(again) <= 1,
                "hit timing: {first} vs {again}"
            );
            assert!(
                channel[..1024]
                    .iter()
                    .zip(&channel[return_hit..return_hit + 1024])
                    .all(|(a, b)| (a - b).abs() < 1e-6),
                "saved slice and settings sound the same"
            );
            assert!(peak(&channel[return_switch..return_hit]) < 1e-6);
        }
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../renders/79-project-switch.wav");
        engine::offline::write_wav(&out, RATE, &left, &right).unwrap();
        std::fs::remove_dir_all(root).ok();
    }

    /// 79: Empty pads after a project switch, with the real UI queue full.
    #[test]
    fn empty_pads_after_a_project_switch() {
        use std::sync::Arc;

        let mut app = test_app();
        let (audio, mut rt) = super::audio::offline(48_000);
        app.audio = Some(audio);
        let sound = Arc::new(Sample::mono(vec![0.5; 2048]));
        app.send(Command::SetPadSound {
            pad: 0,
            sound: Some(sound),
        });
        app.apply(Action::PadPress {
            pad: 0,
            velocity: 1.0,
        });
        let mut left = vec![0.0; 4096];
        let mut right = vec![0.0; 4096];
        rt.process(&mut left, &mut right);
        assert!(
            left.iter().any(|s| s.abs() > 0.01),
            "old project must sound"
        );

        let queue = &mut app.audio.as_mut().unwrap().commands;
        let mut filled = 0;
        while queue
            .push(super::ProjectCommand {
                epoch: 0,
                ready_at_send: true,
                command: super::ProjectMessage::Command(Command::PadRelease { pad: 0 }),
            })
            .is_ok()
        {
            filled += 1;
        }
        assert_eq!(filled, 1024, "fill the real UI command queue");
        app.apply_session(super::session::Session::default());
        assert!(app.pads.sound[0].is_none(), "new project pad is empty");
        for _ in 0..300 {
            app.flush_audio();
            rt.process(&mut left, &mut right);
            if app.pending_audio.is_empty()
                && app
                    .audio
                    .as_ref()
                    .unwrap()
                    .project
                    .ready
                    .load(std::sync::atomic::Ordering::Acquire)
                    == 1
            {
                break;
            }
        }
        assert!(app.pending_audio.is_empty());
        assert_eq!(
            app.audio
                .as_ref()
                .unwrap()
                .project
                .ready
                .load(std::sync::atomic::Ordering::Acquire),
            1
        );

        app.apply(Action::PadPress {
            pad: 0,
            velocity: 1.0,
        });
        rt.process(&mut left, &mut right);
        let peak = left
            .iter()
            .chain(&right)
            .map(|s| s.abs())
            .fold(0.0_f32, f32::max);
        assert!(
            peak < 1e-6,
            "empty project pad played the old sound: peak {peak}"
        );
    }

    /// Scenario "Create a project from the picker" (11): typing a name in the picker opens
    /// the name line; Enter makes the project and opens it.
    #[test]
    fn create_a_project_from_the_picker() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let key = |code: KeyCode| KeyEvent::new(code, KeyModifiers::empty());
        let root = tmp_dir("proj-create-root");
        let mut app = test_app();
        app.paths.projects = root.clone();
        app.apply(Action::ProjectPick);
        for c in "NewBeat".chars() {
            assert!(
                app.project_picker_key(&KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()))
            );
        }
        assert_eq!(
            app.project_picker
                .as_ref()
                .and_then(|p| p.naming.as_deref()),
            Some("NewBeat"),
            "the name line holds what was typed"
        );
        assert!(app.project_picker_key(&key(KeyCode::Enter)));
        assert!(app.project_picker.is_none(), "the picker is closed");
        assert_eq!(
            app.project.as_ref().map(|p| p.name.as_str()),
            Some("NewBeat")
        );
        std::fs::remove_dir_all(&root).ok();
    }

    /// 35 pad-assignment "A pad in hand is named" in a short line: the panel keeps it whole.
    #[test]
    fn a_pad_in_hand_is_named_short() {
        let mut app = test_app();
        app.pads.file[2] = Some("/m/amen.wav".into());
        app.pads.slice[2] = Some(3);
        let text = super::in_hand_line(&app.pads, 2);
        assert_eq!(text, "in hand: pad 3 · amen 3");
        assert!(
            text.chars().count() <= 36,
            "fits the panel of a 100-column terminal"
        );
    }

    /// 35 screen-help "One way to write Ctrl": without kitty the chop line writes undo as
    /// `Ctrl+Z` like `Ctrl+Y`, the bounds as `Ctrl+J L` and `Alt+J L`.
    #[test]
    fn one_way_to_write_ctrl() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let st = HelpState {
            hand: None,
            preview_on: false,
            assign: false,
            snap: super::chop::Snap::Off,
            tab: 0,
            param_name: "volume",
            page: "TONE",
            play: "to end",
            looping: false,
            as_pad: false,
            marking: false,
            scope: String::new(),
            edit: false,
        };
        let items = line(&super::as_combos(super::help_groups(
            &keymap,
            input::Screen::Chop,
            &st,
        )));
        for (k, t) in [
            ("Ctrl+Z", "undo"),
            ("Y H", "parts/hits"),
            ("Ctrl+J L", "left"),
            ("Alt+J L", "right"),
        ] {
            assert!(items.contains(&(k.into(), t.into())), "{k} {t}: {items:?}");
        }
        assert!(
            !items
                .iter()
                .any(|(k, _)| k.starts_with("Ctrl ") || k.starts_with("Alt "))
        );
    }

    /// 35 pad-playback "Restart on bank 2": the open bank goes into the session and comes
    /// back on the next start; an old session without it opens on bank 1.
    #[test]
    fn restart_on_bank_2() {
        let mut app = test_app();
        app.apply(Action::BankNext);
        assert_eq!(app.pads.bank, 1);
        let sess = app.pads.to_session(None, 48_000);
        let text = toml::to_string(&sess).unwrap();
        let back: super::session::Session = toml::from_str(&text).unwrap();
        let mut app = test_app();
        app.apply_session(back);
        assert_eq!(app.pads.bank, 1, "bank 2 again");
        let old: super::session::Session = toml::from_str("version = 3\n").unwrap();
        app.apply_session(old);
        assert_eq!(app.pads.bank, 0, "an old session: bank 1");
    }

    /// 35 projects "Which one is open": the picker marks the open project and the help
    /// line names the picker keys, not the pads screen's.
    #[test]
    fn which_one_is_open() {
        let root = tmp_dir("picker-open");
        let mut app = test_app();
        app.paths.projects = root.clone();
        app.create_project("beat1".into());
        app.create_project("beat2".into());
        app.create_project("beat1".into());
        app.apply(Action::ProjectPick);
        let v = app.picker_view().unwrap();
        assert_eq!(v.entries, ["beat1  ● open", "beat2"]);
        let items = line(&super::picker_help(false));
        for want in ["select", "open", "type a name to create", "close"] {
            assert!(items.iter().any(|(_, t)| t == want), "{want}: {items:?}");
        }
        assert!(!items.iter().any(|(_, t)| t == "edit"), "no pad keys");
        std::fs::remove_dir_all(&root).ok();
    }

    /// 35 sample-loading "A new project starts where you browsed": the files screen of a
    /// new project with an empty samples/ shows the folder the owner browsed last; once the
    /// project has material, its samples/ comes first again.
    #[test]
    fn a_new_project_starts_where_you_browsed() {
        let root = tmp_dir("new-project-browse");
        let breaks = root.join("Music/breaks");
        std::fs::create_dir_all(&breaks).unwrap();
        std::fs::write(breaks.join("amen.wav"), b"").unwrap();
        let mut app = test_app();
        app.paths.projects = root.join("projects");
        app.browser = browser::Browser::new(Some(breaks.clone()));
        app.create_project("beat2".into());
        assert_eq!(app.browser.dir, breaks, "not the empty beat2/samples/");
        // with a recording in it, the project's samples/ comes first
        let samples = root.join("projects/beat2/samples");
        std::fs::write(samples.join("rec.wav"), b"").unwrap();
        // Reopen from another project: selecting the current one no longer reloads.
        assert!(app.create_project("away".into()));
        assert!(app.create_project("beat2".into()));
        assert_eq!(app.browser.dir, samples);
        std::fs::remove_dir_all(&root).ok();
    }

    /// 11 projects "A recording lands in the project": with "beat1" open the WAV goes to
    /// `beat1/samples/` and the session keeps it relative; without a project it goes to the
    /// samples root of the settings.
    #[test]
    fn a_recording_lands_in_the_project() {
        let root = tmp_dir("rec-project");
        let mut app = test_app();
        app.paths.projects = root.join("projects");
        app.paths.samples = root.join("samples");
        assert_eq!(
            app.samples_dir(),
            root.join("samples"),
            "no project: the samples root"
        );
        app.create_project("beat1".into());
        let here = app.samples_dir();
        assert_eq!(here, root.join("projects/beat1/samples"));
        app.keep_recording(&ring(), false, &here);
        let file = app.browser.in_hand.take().unwrap();
        assert!(file.starts_with(&here), "{}", file.display());
        app.put_file(0, file);
        wait_for(&mut app, |a| a.pads.sound[0].is_some());
        let folder = app.project.as_ref().unwrap().folder.clone();
        let stored = super::with_stored_paths(app.pads.to_session(None, 48_000), &folder);
        assert!(
            stored.pads.iter().any(|(_, p)| p
                .file
                .as_deref()
                .is_some_and(|f: &std::path::Path| f.is_relative())),
            "kept relative to the project"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    /// Scenario "A new project folder" (11): the created project is a folder with
    /// `project.toml` and an empty `samples/` in the projects root.
    #[test]
    fn a_new_project_folder() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let key = |code: KeyCode| KeyEvent::new(code, KeyModifiers::empty());
        let root = tmp_dir("proj-folder-root");
        let mut app = test_app();
        app.paths.projects = root.clone();
        app.apply(Action::ProjectPick);
        for c in "MyBeat".chars() {
            app.project_picker_key(&key(KeyCode::Char(c)));
        }
        app.project_picker_key(&key(KeyCode::Enter));
        let pj = root.join("MyBeat");
        assert!(pj.is_dir(), "the folder appeared");
        assert!(
            pj.join("project.toml").is_file(),
            "the session file is there"
        );
        let samples = pj.join("samples");
        assert!(
            samples.is_dir() && std::fs::read_dir(&samples).unwrap().next().is_none(),
            "samples/ exists and is empty"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    /// Scenario "Switch and switch back" (11): each project opens exactly what it saved —
    /// the pad of the previous project leaves no trace.
    #[test]
    fn switch_and_switch_back() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let key = |code: KeyCode| KeyEvent::new(code, KeyModifiers::empty());
        let root = tmp_dir("proj-switch-root");
        for (name, sample) in [("A", "a.wav"), ("B", "b.wav")] {
            let pj = root.join(name);
            std::fs::create_dir_all(pj.join("samples")).unwrap();
            std::fs::write(pj.join("samples").join(sample), b"").unwrap();
            let mut pads = std::collections::BTreeMap::new();
            pads.insert(
                "0".to_string(),
                super::session::PadState {
                    file: Some(super::PathBuf::from(format!("samples/{sample}"))),
                    ..Default::default()
                },
            );
            super::session::save(
                &pj.join("project.toml"),
                &super::session::Session {
                    pads,
                    ..Default::default()
                },
            )
            .unwrap();
        }
        let mut app = test_app();
        app.paths.projects = root.clone();
        let open = |app: &mut super::App, name: &str| {
            app.apply(Action::ProjectPick);
            app.project_picker.as_mut().unwrap().selected = app
                .project_picker
                .as_ref()
                .unwrap()
                .entries
                .iter()
                .position(|e| e == name)
                .unwrap();
            app.project_picker_key(&key(KeyCode::Enter));
            wait_for(app, |a| a.pads.loading[0].is_none());
            app.project.as_ref().map(|p| p.name.clone())
        };
        assert_eq!(open(&mut app, "A"), Some("A".to_string()));
        assert_eq!(
            app.pads.file[0]
                .as_ref()
                .and_then(|p| p.file_name().unwrap().to_str()),
            Some("a.wav")
        );
        assert_eq!(open(&mut app, "B"), Some("B".to_string()));
        assert_eq!(
            app.pads.file[0]
                .as_ref()
                .and_then(|p| p.file_name().unwrap().to_str()),
            Some("b.wav"),
            "the pad of A did not leak into B"
        );
        assert_eq!(open(&mut app, "A"), Some("A".to_string()));
        assert_eq!(
            app.pads.file[0]
                .as_ref()
                .and_then(|p| p.file_name().unwrap().to_str()),
            Some("a.wav")
        );
        std::fs::remove_dir_all(&root).ok();
    }

    /// Scenario "Switch and switch back" (11), heard: after a switch the pads sound as in
    /// the project opened alone; the pitch, volume, mode, envelope and sounds of the
    /// previous project stay behind (owner, 2026-10-31: pad D played 7 semitones down
    /// with +0 st on screen).
    #[test]
    fn switch_and_switch_back_nothing_leaks_into_the_sound() {
        use engine::offline::{Event, Scenario, render, write_wav};
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let key = |code: KeyCode| KeyEvent::new(code, KeyModifiers::empty());
        let root = tmp_dir("proj-leak-root");
        let tone: Vec<f32> = (0..24_000)
            .map(|i| 0.5 * (i as f32 * 220.0 / 48_000.0 * std::f32::consts::TAU).sin())
            .collect();
        let pad = |pitch, volume_db, gate, attack_ms| super::session::PadState {
            file: Some(super::PathBuf::from("samples/tone.wav")),
            pitch,
            volume_db,
            gate,
            attack_ms,
            ..Default::default()
        };
        // A: pad D (10) shifted, quieter, gate, slow attack; pad F (11) sounds.
        // B: pad D plain, pad F empty.
        let projects = [
            (
                "A",
                vec![
                    (
                        "10",
                        super::session::PadState {
                            // and a filter (44): it must stay behind too
                            filter: "low-pass".into(),
                            cutoff_hz: Some(300.0),
                            reso: 50,
                            ..pad(-7, -6, true, 50)
                        },
                    ),
                    ("11", pad(0, 0, false, 0)),
                ],
            ),
            ("B", vec![("10", pad(0, 0, false, 0))]),
        ];
        for (name, pads) in projects {
            let pj = root.join(name);
            std::fs::create_dir_all(pj.join("samples")).unwrap();
            write_wav(&pj.join("samples/tone.wav"), 48_000, &tone, &tone).unwrap();
            let pads = pads.into_iter().map(|(k, s)| (k.to_string(), s)).collect();
            super::session::save(
                &pj.join("project.toml"),
                &super::session::Session {
                    pads,
                    ..Default::default()
                },
            )
            .unwrap();
        }
        let open = |app: &mut super::App, name: &str| {
            app.apply(Action::ProjectPick);
            let picker = app.project_picker.as_mut().unwrap();
            picker.selected = picker.entries.iter().position(|e| e == name).unwrap();
            app.project_picker_key(&key(KeyCode::Enter));
            wait_for(app, |a| a.pads.loading.iter().all(Option::is_none));
        };
        // what the engine hears: every command the app sent, then pads D and F are hit
        let heard = |app: &super::App| {
            let mut events: Vec<Event> = app
                .sent
                .iter()
                .map(|c| Event {
                    frame: 0,
                    cmd: c.clone(),
                })
                .collect();
            for pad in [10, 11] {
                events.push(Event {
                    frame: 128,
                    cmd: Command::PadPress { pad, velocity: 1.0 },
                });
            }
            let sc = Scenario {
                sample_rate: 48_000,
                frames: 24_000,
                events,
            };
            render(&mut engine::Engine::new(48_000), &sc, 128).0
        };
        let mut switched = test_app();
        switched.paths.projects = root.clone();
        open(&mut switched, "A");
        open(&mut switched, "B");
        let mut alone = test_app();
        alone.paths.projects = root.clone();
        open(&mut alone, "B");
        let (after, expected) = (heard(&switched), heard(&alone));
        assert!(engine::checks::first_non_finite(&after).is_none());
        assert!(engine::checks::peak(&expected) > 0.4, "pad D sounds in B");
        let off = after
            .iter()
            .zip(&expected)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(off < 1e-6, "after A, B sounds as B alone; off by {off}");
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../renders/11-switch-nothing-leaks.wav");
        write_wav(&out, 48_000, &after, &after).unwrap();
        std::fs::remove_dir_all(&root).ok();
    }

    /// Scenario "Folders without a project" (11): the picker lists only the folders that
    /// hold a `project.toml`.
    #[test]
    fn folders_without_a_project() {
        let root = tmp_dir("proj-junk-root");
        let pj = root.join("MyBeat");
        std::fs::create_dir_all(&pj).unwrap();
        std::fs::write(pj.join("project.toml"), "").unwrap();
        std::fs::create_dir_all(root.join("Junk")).unwrap();
        let mut app = test_app();
        app.paths.projects = root.clone();
        app.apply(Action::ProjectPick);
        let picker = app.project_picker.as_ref().expect("the picker is open");
        assert_eq!(picker.entries, vec!["MyBeat".to_string()]);
        std::fs::remove_dir_all(&root).ok();
    }

    /// A plain `Enter` press on the files screen.
    fn enter_key() -> input::Action {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let enter = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Enter,
            ratatui::crossterm::event::KeyModifiers::empty(),
        );
        keymap.resolve(input::Screen::Files, &enter).unwrap()
    }

    /// A `Ctrl+Enter` press on the files screen.
    fn ctrl_enter_key() -> input::Action {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let ctrl_enter = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Enter,
            ratatui::crossterm::event::KeyModifiers::CONTROL,
        );
        keymap.resolve(input::Screen::Files, &ctrl_enter).unwrap()
    }

    /// Scenario "Ctrl+Enter puts the file on the selected pad" (41): `Ctrl+Enter` on a file
    /// loads it onto the selected pad and closes the files screen; the hand is not needed.
    #[test]
    fn ctrl_enter_puts_the_file_on_the_selected_pad() {
        let dir = tmp_dir("enter-load");
        let mut app = test_app();
        app.keep_recording(&ring(), false, &dir);
        let wav = app.browser.in_hand.clone().expect("the file is listed");
        app.browser.in_hand = None;
        app.browser = browser::Browser::new(Some(dir.clone()));
        app.goto(Screen::Files);
        app.apply(ctrl_enter_key());
        assert_eq!(app.screen, Screen::Pads, "the screen closed onto the pads");
        assert!(app.ask.is_none());
        assert!(app.browser.in_hand.is_none());
        let abs = app.pads.selected_abs();
        wait_for(&mut app, |a| a.pads.loading[abs].is_none());
        assert_eq!(app.pads.file[abs].as_ref(), Some(&wav));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Scenario "Enter takes the file, a hit places it" (41).
    /// Scenario "Put a break on a pad" (pad-assignment).
    #[test]
    fn enter_takes_the_file_a_hit_places_it() {
        let dir = tmp_dir("enter-take");
        let mut app = test_app();
        app.keep_recording(&ring(), false, &dir);
        let wav = app.browser.in_hand.take().unwrap();
        app.browser = browser::Browser::new(Some(dir.clone()));
        app.goto(Screen::Files);
        app.apply(enter_key());
        assert_eq!(
            app.browser.in_hand.as_ref(),
            Some(&wav),
            "in hand after Enter"
        );
        hit(&mut app, 5); // W
        wait_for(&mut app, |a| a.pads.loading[5].is_none());
        assert_eq!(app.pads.file[5].as_ref(), Some(&wav), "on W after the hit");
        let sound = app.pads.sound[5].clone().expect("pad 6 holds the file");
        assert!(app.sent.iter().any(|c| matches!(c,
            Command::SetPadSound { pad: 5, sound: Some(s) } if std::sync::Arc::ptr_eq(s, &sound))));
        app.sent.clear();
        hit(&mut app, 5);
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::PadPress { pad: 5, .. })),
            "the next hit on W plays it"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Scenario "See where the file goes" (41): `Enter` only takes the file in hand; the
    /// screen stays, nothing loads yet, and the help line offers the pads and Esc.
    #[test]
    fn see_where_the_file_goes() {
        let dir = tmp_dir("ctrl-enter");
        let mut app = test_app();
        app.keep_recording(&ring(), false, &dir);
        let wav = app.browser.in_hand.clone().unwrap();
        app.browser.in_hand = None;
        app.browser = browser::Browser::new(Some(dir.clone()));
        app.goto(Screen::Files);
        app.apply(enter_key());
        assert_eq!(app.browser.in_hand.as_ref(), Some(&wav), "the file in hand");
        assert_eq!(app.screen, Screen::Files, "the screen stays");
        assert!(app.pads.file.iter().all(Option::is_none), "nothing loaded");
        assert!(app.ask.is_none());
        // the help line points at the pads and the way back
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let groups = super::help_groups(
            &keymap,
            input::Screen::Files,
            &HelpState {
                hand: Some("the file"),
                preview_on: false,
                assign: false,
                snap: super::chop::Snap::Off,
                tab: 0,
                param_name: "volume",
                page: "TONE",
                play: "to end",
                looping: false,
                as_pad: false,
                marking: false,
                scope: String::new(),
                edit: false,
            },
        );
        let items = line(&groups);
        assert!(
            items
                .iter()
                .any(|(k, v)| k == "1-4 Q-R A-F Z-V" && v == "the file"),
            "the help line offers a pad hit"
        );
        assert!(items.iter().any(|(_, v)| v == "put back"), "and Esc");
        // a pad hit puts the file in hand onto the selected pad, as before (Enter on pads)
        app.goto(Screen::Pads);
        app.apply(Action::PadEdit);
        assert!(app.ask.is_none());
        let abs = app.pads.selected_abs();
        wait_for(&mut app, |a| a.pads.loading[abs].is_none());
        assert_eq!(app.pads.file[abs].as_ref(), Some(&wav));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Scenario "Into a folder and back" (23): `Enter` on a folder opens it, the list
    /// stays; the way back shows the parent with the same folder selected.
    #[test]
    fn into_a_folder_and_back() {
        let dir = tmp_dir("enter-dir");
        let mut app = test_app();
        app.keep_recording(&ring(), false, &dir);
        std::fs::create_dir(dir.join("kit")).unwrap();
        app.browser = browser::Browser::new(Some(dir.clone()));
        app.goto(Screen::Files);
        app.apply(enter_key());
        assert_eq!(app.browser.dir, dir.join("kit"), "the folder opened");
        assert_eq!(app.screen, Screen::Files, "the list stays open");
        assert!(app.pads.file.iter().all(Option::is_none), "nothing loaded");
        assert!(app.ask.is_none());
        // back with the parent key: the same folder stays selected
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let left = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Left,
            ratatui::crossterm::event::KeyModifiers::empty(),
        );
        app.apply(keymap.resolve(input::Screen::Files, &left).unwrap());
        assert_eq!(app.browser.dir, dir, "back to the parent folder");
        assert_eq!(
            app.browser.current().map(|e| e.name.as_str()),
            Some("kit"),
            "the folder we left stays selected"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A `Space` press on the files screen.
    fn space_key() -> input::Action {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let space = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Char(' '),
            ratatui::crossterm::event::KeyModifiers::empty(),
        );
        keymap.resolve(input::Screen::Files, &space).unwrap()
    }

    /// 27 "Source page" in the help line: the edit line names the SOURCE page and the
    /// picked bound, as it names TONE and ENV.
    #[test]
    fn the_source_page_in_the_help_line() {
        let mut app = laid_out_break(); // slice 3 on E of bank 1
        app.screen = Screen::Pads;
        app.pads.selected = 6;
        app.apply(Action::PadEdit);
        while app.pads.page().0 != "SOURCE" {
            app.apply(Action::ParamNext);
        }
        app.apply(Action::ParamNext); // source → start
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let items = line(&super::help_groups(
            &keymap,
            input::Screen::Pads,
            &app.help_state(),
        ));
        assert!(items.iter().any(|(_, t)| t == "SOURCE"), "{items:?}");
        assert!(items.iter().any(|(_, t)| t == "start"), "{items:?}");
    }

    /// screen-help "The last group of the help line": on every screen the screens group
    /// reads F1, F3, F6, F12 in that order.
    #[test]
    fn the_last_group_of_the_help_line() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let st = HelpState {
            hand: None,
            preview_on: false,
            assign: false,
            snap: super::chop::Snap::Off,
            tab: 0,
            param_name: "volume",
            page: "TONE",
            play: "to end",
            looping: false,
            as_pad: false,
            marking: false,
            scope: String::new(),
            edit: false,
        };
        for screen in [
            input::Screen::Pads,
            input::Screen::Files,
            input::Screen::Chop,
            input::Screen::Settings,
        ] {
            let groups = super::help_groups(&keymap, screen, &st);
            let keys: Vec<String> = groups
                .last()
                .unwrap()
                .items
                .iter()
                .map(|i| i.0.clone())
                .collect();
            assert_eq!(keys, ["F1", "F3", "F4", "F5", "F6", "F12"], "{screen:?}");
        }
    }

    /// screen-help "Audio tab" (and the Paths tab of 11): the help line of a settings tab
    /// lists only its own keys — no "learn" and no "clear" outside the MIDI tab.
    #[test]
    fn audio_tab() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let tab_line = |tab| {
            let st = HelpState {
                hand: None,
                preview_on: false,
                assign: false,
                snap: super::chop::Snap::Off,
                tab,
                param_name: "volume",
                page: "TONE",
                play: "to end",
                looping: false,
                as_pad: false,
                marking: false,
                scope: String::new(),
                edit: false,
            };
            line(&super::help_groups(&keymap, input::Screen::Settings, &st))
        };
        for tab in [0, 1, 3] {
            let items = tab_line(tab);
            assert!(
                !items.iter().any(|(_, t)| t == "learn" || t == "clear"),
                "tab {tab}: {items:?}"
            );
        }
        assert!(
            tab_line(3).iter().any(|(_, t)| t == "open"),
            "Paths: Enter opens"
        );
        assert!(
            tab_line(2).iter().any(|(_, t)| t == "learn"),
            "MIDI keeps them"
        );
    }

    /// The files help line items with the given preview state.
    fn files_line(preview_on: bool) -> Vec<(String, String)> {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let groups = super::help_groups(
            &keymap,
            input::Screen::Files,
            &HelpState {
                hand: None,
                preview_on,
                assign: false,
                snap: super::chop::Snap::Off,
                tab: 0,
                param_name: "volume",
                page: "TONE",
                play: "to end",
                looping: false,
                as_pad: false,
                marking: false,
                scope: String::new(),
                edit: false,
            },
        );
        line(&groups)
    }

    /// Scenario "Scrolling through files" (30): the preview is on by default; scrolling
    /// the list moves the selection and the new file sounds.
    #[test]
    fn scrolling_through_files() {
        let dir = tmp_dir("scroll");
        let mut app = test_app();
        app.keep_recording(&ring(), false, &dir);
        app.keep_recording(&ring(), false, &dir);
        app.browser = browser::Browser::new(Some(dir.clone()));
        app.apply(Action::ScreenFiles);
        assert!(app.preview_on, "the preview is on by default");
        wait_for(&mut app, |a| a.preview.is_some());
        let first = app.load_gen[super::PREVIEW];
        app.apply(Action::BrowserDown);
        assert_eq!(app.browser.selected, 1, "the selection moved");
        assert!(
            app.load_gen[super::PREVIEW] > first,
            "the next file is asked for"
        );
        wait_for(&mut app, |a| a.preview.is_some());
        assert!(app.pads.file.iter().all(Option::is_none));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 33 "The line runs with the sound" and "No line when silent": the files screen gets
    /// the place of the preview as a part of the file while it sounds, nothing otherwise.
    #[test]
    fn the_preview_playhead() {
        let dir = tmp_dir("preview-at");
        let mut app = test_app();
        app.keep_recording(&ring(), false, &dir); // 1 s at 48 kHz
        app.browser = browser::Browser::new(Some(dir.clone()));
        app.apply(Action::ScreenFiles);
        wait_for(&mut app, |a| a.preview.is_some());
        app.fake_playhead = Some(24_000);
        let at = app.preview_at().expect("the line while it sounds");
        assert!((at - 0.5).abs() < 0.01, "halfway: {at}");
        app.fake_playhead = None;
        assert_eq!(
            app.preview_at(),
            None,
            "the file played to its end: no line"
        );
        app.fake_playhead = Some(24_000);
        app.apply(space_key());
        assert_eq!(app.preview_at(), None, "the preview off: no line");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Scenario "Preview off" (30): Space stops the preview, the help line says
    /// "preview: off" and the list stays silent.
    /// Scenario "Toggle in the help" (screen-help).
    #[test]
    fn preview_off() {
        let dir = tmp_dir("preview-off");
        let mut app = test_app();
        app.keep_recording(&ring(), false, &dir);
        app.keep_recording(&ring(), false, &dir);
        app.browser = browser::Browser::new(Some(dir.clone()));
        app.apply(Action::ScreenFiles);
        wait_for(&mut app, |a| a.preview.is_some());
        app.apply(space_key());
        assert!(!app.preview_on);
        assert!(app.preview.is_none(), "the preview stops");
        assert!(
            files_line(false).iter().any(|(_, v)| v == "preview: off"),
            "the help line names the state"
        );
        app.apply(Action::BrowserDown);
        assert!(app.preview.is_none(), "the list stays silent");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Scenario "Preview on" (30): with the preview off, Space turns it back on, the
    /// current file sounds and the help line says "preview: on".
    #[test]
    fn preview_on() {
        let dir = tmp_dir("preview-on");
        let mut app = test_app();
        app.keep_recording(&ring(), false, &dir);
        app.browser = browser::Browser::new(Some(dir.clone()));
        app.apply(Action::ScreenFiles);
        app.apply(space_key());
        assert!(app.preview.is_none());
        app.apply(space_key());
        assert!(app.preview_on);
        wait_for(&mut app, |a| a.preview.is_some());
        assert!(
            files_line(true).iter().any(|(_, v)| v == "preview: on"),
            "the help line names the state"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Scenario "A long pad fades on entry" (24): opening the chop screen sends the hush
    /// once, from the pads and from the files screen alike; the sound itself is checked
    /// by the engine render (`a_long_pad_fades_on_entry`).
    #[test]
    fn the_chop_entry_hushes_the_pads() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = test_app();
        // the chop screen opens only over a pad with a sample
        app.pads.sound[0] = Some(std::sync::Arc::new(Sample::mono(vec![0.0; 480])));
        app.pads.file[0] = Some(std::path::PathBuf::from("/tmp/ptsr-hush/tone.wav"));
        let f3 = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::F(3),
            ratatui::crossterm::event::KeyModifiers::empty(),
        );
        app.apply(keymap.resolve(input::Screen::Pads, &f3).unwrap());
        assert_eq!(app.screen, Screen::Chop);
        assert_eq!(
            app.sent
                .iter()
                .filter(|c| matches!(c, Command::HushPads))
                .count(),
            1,
            "one hush at the screen switch"
        );
        // a pad hit on the chop screen sends no second hush
        app.apply(Action::PadPress {
            pad: 0,
            velocity: 0.8,
        });
        assert_eq!(
            app.sent
                .iter()
                .filter(|c| matches!(c, Command::HushPads))
                .count(),
            1
        );
        // back and in again: the hush comes once per entry
        app.apply(Action::ScreenPads);
        app.apply(keymap.resolve(input::Screen::Pads, &f3).unwrap());
        assert_eq!(
            app.sent
                .iter()
                .filter(|c| matches!(c, Command::HushPads))
                .count(),
            2
        );
    }

    /// Scenario "Empty session" (23): with no samples on the pads the hint line tells to
    /// open the files and press `Enter` on a file.
    #[test]
    fn empty_session() {
        let app = test_app();
        assert_eq!(
            super::hint_for(&app),
            Some("Pads are empty: F6 opens files, Enter takes one, a pad hit puts it there.")
        );
    }

    /// Scenario "A message passes" (23): a short-lived message does not remove the hint;
    /// it is there while the message shows and after it is gone.
    #[test]
    fn a_message_passes() {
        let mut app = test_app();
        app.post_notice(Level::Info, ("MIDI connected".into(), Instant::now()));
        assert!(
            super::hint_for(&app).is_some(),
            "the hint stays under the message"
        );
        app.notices = Notices::new(Instant::now());
        assert_eq!(
            super::hint_for(&app),
            Some("Pads are empty: F6 opens files, Enter takes one, a pad hit puts it there."),
            "the hint is back as soon as the message is gone"
        );
    }

    /// Scenario "Done with the hint" (23): once a pad holds a sample the first-steps
    /// hint gives way to the play hint.
    #[test]
    fn done_with_the_hint() {
        let dir = tmp_dir("hint-done");
        let mut app = test_app();
        app.keep_recording(&ring(), false, &dir);
        let wav = app.browser.in_hand.clone().unwrap();
        app.pads.file[0] = Some(wav);
        assert_eq!(
            super::hint_for(&app),
            Some("Hit 1-4 Q-R A-F Z-V to play pads; Shift for full velocity.")
        );
        std::fs::remove_dir_all(&dir).ok();
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 21: the first-steps hint of the chop screen names the keys of 21.
    #[test]
    fn the_chop_hint_names_the_keys_of_21() {
        let mut app = test_app();
        app.screen = Screen::Chop;
        assert_eq!(
            super::hint_for(&app),
            Some(
                "Start with Y (equal parts) or Space and M on the beat; P lays the slices on pads."
            )
        );
    }

    /// A pad that holds something asks first; the same `Enter` replaces the file, `Esc`
    /// keeps the pad. Guards the question of the load flow (the in-hand flow has its own
    /// test in `bind_to_a_pad`).
    #[test]
    fn overwrite_question() {
        let dir = tmp_dir("overwrite");
        let mut app = test_app();
        app.keep_recording(&ring(), false, &dir);
        let first = app.browser.in_hand.clone().unwrap();
        app.browser.in_hand = None;
        // the pad holds another file already, loaded without the list
        app.keep_recording(&ring(), false, &dir);
        let second = app.browser.in_hand.clone().unwrap();
        app.browser.in_hand = None;
        app.put_file(0, second.clone());
        wait_for(&mut app, |a| a.pads.loading[0].is_none());
        // the selection goes to the first file, the one not on the pad
        app.browser = browser::Browser::new(Some(dir.clone()));
        let idx = app
            .browser
            .entries
            .iter()
            .position(|e| app.browser.dir.join(&e.name) == first)
            .expect("the first file is listed");
        app.browser.selected = idx;
        app.goto(Screen::Files);
        // Ctrl+Enter puts the file straight on the selected pad (41)
        let enter = ctrl_enter_key();
        app.apply(enter);
        assert!(app.ask.is_some(), "the replace question");
        assert_eq!(app.screen, Screen::Files, "the screen waits");
        assert_eq!(app.pads.file[0].as_ref(), Some(&second), "nothing yet");
        app.apply(Action::Back);
        assert!(app.ask.is_none());
        assert_eq!(
            app.pads.file[0].as_ref(),
            Some(&second),
            "Esc keeps the pad"
        );
        app.apply(enter);
        assert!(app.ask.is_some(), "the question comes again");
        app.apply(enter);
        assert!(app.ask.is_none());
        assert_eq!(
            app.screen,
            Screen::Pads,
            "the screen closed after the replace"
        );
        wait_for(&mut app, |a| a.pads.loading[0].is_none());
        assert_eq!(app.pads.file[0].as_ref(), Some(&first));
        std::fs::remove_dir_all(&dir).ok();
    }

    fn line(groups: &[tui::HelpGroup]) -> Vec<(String, String)> {
        groups.iter().flat_map(|g| g.items.clone()).collect()
    }

    fn pads_line(kitty: bool) -> Vec<tui::HelpGroup> {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let groups = super::help_groups(
            &keymap,
            input::Screen::Pads,
            &HelpState {
                hand: None,
                preview_on: false,
                assign: false,
                snap: super::chop::Snap::Off,
                tab: 0,
                param_name: "volume",
                page: "TONE",
                play: "to end",
                looping: false,
                as_pad: false,
                marking: false,
                scope: String::new(),
                edit: false,
            },
        );
        if kitty {
            super::without_modifiers(groups)
        } else {
            groups
        }
    }

    #[test]
    fn hold_ctrl_shows_only_the_ctrl_combos() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let groups =
            super::modifier_groups(&keymap, input::Screen::Pads, input::Modifier::Ctrl, false);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "CTRL");
        let items = line(&groups);
        for want in [("T", "output"), ("1…8", "bank")] {
            assert!(
                items.contains(&(want.0.into(), want.1.into())),
                "{want:?} missing in {items:?}"
            );
        }
        // 83a: the microphone moved to Alt+R, so the Alt group names it
        let alt = line(&super::modifier_groups(
            &keymap,
            input::Screen::Pads,
            input::Modifier::Alt,
            false,
        ));
        assert!(alt.contains(&("R".into(), "mic: hold".into())), "{alt:?}");
        // every key there is a Ctrl combo, shown by its bare letter under CTRL
        for (k, _) in &items {
            assert!(
                !k.contains("Ctrl") && !k.starts_with('F'),
                "{k} is not a Ctrl combo"
            );
        }
        // the chop screen pairs the fine nudge keys
        let chop = line(&super::modifier_groups(
            &keymap,
            input::Screen::Chop,
            input::Modifier::Ctrl,
            false,
        ));
        assert!(
            chop.contains(&("J L".into(), "left bound".into())),
            "{chop:?}"
        );
    }

    /// Recorded: the cursor at 1 s, play running to 3 s, M, Space. 21: "Cut while playing",
    /// "The cursor survives a pause".
    #[test]
    fn cut_while_playing() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(0));
        app.chop.as_mut().unwrap().set_cursor(48_000);
        app.fake_playhead = Some(72_000);
        app.apply(input::Action::ChopCut);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.markers, [72_000], "the cut lands where the sound is");
        assert_eq!(c.cursor, 48_000, "the cursor stays");
        // Space while it plays stops it; the cursor stays where the owner put it
        app.apply(input::Action::ChopPlay);
        assert_eq!(app.chop.as_ref().unwrap().cursor, 48_000);
        // stopped: the cut is at the cursor again
        app.fake_playhead = None;
        app.chop.as_mut().unwrap().set_cursor(24_000);
        app.apply(input::Action::ChopCut);
        assert_eq!(app.chop.as_ref().unwrap().markers, [24_000, 72_000]);
    }

    /// Recorded: Ctrl+X on pad W, then a hit on pad E. pad-assignment 21: "Swap by a hit".
    #[test]
    fn swap_by_a_hit() {
        let mut app = test_app();
        app.pads.file[5] = Some("/s/w.wav".into());
        app.pads.file[6] = Some("/s/e.wav".into());
        app.pads.selected = 5;
        app.apply(input::Action::PadTake);
        assert_eq!(app.pad_in_hand, Some(5));
        app.apply(input::Action::PadPress {
            pad: 6,
            velocity: 1.0,
        });
        assert_eq!(
            app.pads.file[5].as_deref(),
            Some(std::path::Path::new("/s/e.wav"))
        );
        assert_eq!(
            app.pads.file[6].as_deref(),
            Some(std::path::Path::new("/s/w.wav"))
        );
        assert!(app.pad_in_hand.is_none(), "nothing in hand any more");
    }

    /// Recorded: O, then L into the next slice. 21: "The bar shows what plays".
    #[test]
    fn the_bar_shows_what_plays() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(1)); // 2 slices of 48_000
        let o = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Char('o'),
            ratatui::crossterm::event::KeyModifiers::NONE,
        );
        app.apply(keymap.resolve(input::Screen::Chop, &o).unwrap());
        let bar = |app: &super::App| {
            let c = app.chop.as_ref().unwrap();
            super::chop_view(c, None, None, 48_000, &app.pads).play
        };
        assert_eq!(bar(&app), Some((0.0, 0.5)), "to the cut of slice 1");
        let help = line(&super::help_groups(
            &keymap,
            input::Screen::Chop,
            &app.help_state(),
        ));
        assert!(help.contains(&("O".into(), "to cut".into())), "{help:?}");
        app.chop.as_mut().unwrap().set_cursor(60_000);
        let (from, to) = bar(&app).unwrap();
        assert!(
            (from - 0.625).abs() < 1e-3 && to == 1.0,
            "follows the cursor: {from} {to}"
        );
    }

    /// The key `code` with `m` on `screen`, through the edit layer when an edit is open.
    fn screen_key(
        app: &mut super::App,
        screen: Screen,
        code: ratatui::crossterm::event::KeyCode,
        m: ratatui::crossterm::event::KeyModifiers,
    ) {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let k = ratatui::crossterm::event::KeyEvent::new(code, m);
        let action = keymap.resolve_in(screen, app.edit, &k).unwrap();
        app.apply(action);
    }

    /// Recorded: PgDn, PgUp on the chop screen in the listen mode. 43 "The next bank while
    /// chopping".
    #[test]
    fn the_next_bank_while_chopping() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(1)); // a cut at 48_000
        app.chop.as_mut().unwrap().set_cursor(10_000);
        screen_key(&mut app, Screen::Chop, KeyCode::PageDown, M::NONE);
        assert_eq!(app.pads.bank, 1, "bank 2");
        assert_eq!(
            app.chop.as_ref().unwrap().cursor,
            10_000,
            "the cursor stays"
        );
        screen_key(&mut app, Screen::Chop, KeyCode::PageUp, M::NONE);
        assert_eq!(app.pads.bank, 0, "back to bank 1");
    }

    /// Recorded: P, then PgDn. 43 "The same keys while assigning".
    #[test]
    fn the_same_keys_while_assigning() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(1));
        app.apply(Action::ChopAssign);
        assert!(app.plan.is_some());
        screen_key(&mut app, Screen::Chop, KeyCode::PageDown, M::NONE);
        assert_eq!(app.pads.bank, 1, "bank 2, as in the listen mode");
        assert!(app.plan.is_some(), "still assigning");
    }

    /// Recorded: PgDn with 20 slices on no pad, then the first pad key. 43 "Pages of the
    /// keyboard of slices".
    #[test]
    fn pages_of_the_keyboard_of_slices() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        let mut c = chop_with_parts(0);
        for n in 1..20 {
            c.cut(n * 4_800);
        }
        assert_eq!(c.slices().len(), 20);
        app.chop = Some(c);
        screen_key(&mut app, Screen::Chop, KeyCode::PageDown, M::NONE);
        let c = app.chop.as_ref().unwrap();
        let v = super::chop_view(c, None, None, 48_000, &app.pads);
        assert_eq!(v.slice_keys, Some((17, 20)), "slices 17–20");
        hit(&mut app, 0);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.selected, 16, "the first pad key: slice 17");
        assert_eq!(last_played(&app), Some(c.slices()[16]));
    }

    /// Recorded: Enter on pad W, PgDn, L. 43 "The next bank while editing a pad".
    #[test]
    fn the_next_bank_while_editing_a_pad() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.pads.selected = 5; // W
        screen_key(&mut app, Screen::Pads, KeyCode::Enter, M::NONE);
        assert!(app.edit);
        screen_key(&mut app, Screen::Pads, KeyCode::PageDown, M::NONE);
        assert_eq!(app.pads.bank, 1, "bank 2");
        assert!(app.edit, "the edit stays open");
        assert_eq!(app.pads.selected, 5, "on W of bank 2");
        let volume = app.pads.volume_db[16 + 5];
        screen_key(&mut app, Screen::Pads, KeyCode::Char('l'), M::NONE);
        assert_eq!(app.pads.bank, 1, "L does not turn the bank in the edit");
        assert_eq!(
            app.pads.volume_db[16 + 5],
            volume + 1,
            "L changes the value"
        );
    }

    /// Recorded: PgDn on the files screen. 43 "The same keys on the files screen".
    #[test]
    fn the_same_keys_on_the_files_screen() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Files;
        let before = app.browser.selected;
        screen_key(&mut app, Screen::Files, KeyCode::PageDown, M::NONE);
        assert_eq!(app.pads.bank, 1, "bank 2 on the grid");
        assert_eq!(app.browser.selected, before, "the list does not move");
    }

    /// screen-help 43: "Narrow terminal" — with the kitty protocol at 140 columns every
    /// group of the real lines is in the band.
    #[test]
    fn every_line_fits_the_band_at_140() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = test_app();
        app.chop = Some(chop_with_parts(1));
        for (screen, edit) in [
            (input::Screen::Chop, false),
            (input::Screen::Chop, true),
            (input::Screen::Pads, false),
            (input::Screen::Pads, true),
            (input::Screen::Files, false),
        ] {
            app.screen = screen;
            app.edit = edit;
            let groups = || super::help_groups(&keymap, screen, &app.help_state());
            let g = super::without_modifiers(groups());
            // the chop listen line drops only its last group, the screens; F1 has them
            let drops = usize::from(screen == input::Screen::Chop && !edit);
            assert_eq!(
                tui::help_dropped(&g, 140),
                drops,
                "{screen:?} edit {edit}: {:?}",
                line(&g)
            );
            if drops == 1 {
                assert_eq!(
                    g.last().unwrap().name,
                    "",
                    "the dropped group is the screens"
                );
            }
        }
    }

    /// The help groups of `screen` as the kitty terminal shows them (no combos).
    fn kitty_groups(screen: input::Screen) -> Vec<tui::HelpGroup> {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = test_app();
        app.chop = Some(chop_with_parts(1));
        app.screen = screen;
        super::without_modifiers(super::help_groups(&keymap, screen, &app.help_state()))
    }

    /// 43 "Columns on the pads screen".
    #[test]
    fn columns_on_the_pads_screen() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let app = test_app();
        let groups = super::help_groups(&keymap, input::Screen::Pads, &app.help_state());
        let names: Vec<_> = groups.iter().map(|g| g.name).collect();
        assert_eq!(names, ["PAD", "CHANGE", "BANK", "CLOCK", "REC", ""]);
        assert_eq!(tui::help_dropped(&super::without_modifiers(groups), 140), 0);
    }

    /// 43 "The same order on every screen": BANK right after the screen's own groups.
    #[test]
    fn the_same_order_on_every_screen() {
        for screen in [
            input::Screen::Pads,
            input::Screen::Chop,
            input::Screen::Files,
        ] {
            let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
            let mut app = test_app();
            app.chop = Some(chop_with_parts(1));
            app.screen = screen;
            let names: Vec<_> = super::help_groups(&keymap, screen, &app.help_state())
                .iter()
                .map(|g| g.name)
                .collect();
            let n = names.len();
            assert_eq!(
                names[n - 4..],
                ["BANK", "CLOCK", "REC", ""],
                "{screen:?}: {names:?}"
            );
        }
    }

    /// 43 "Too narrow for all": the files screen at 100 columns keeps its groups and BANK
    /// whole and lets the screens go.
    #[test]
    fn too_narrow_for_all() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = test_app();
        app.screen = input::Screen::Files;
        let groups = super::help_groups(&keymap, input::Screen::Files, &app.help_state());
        let dropped = tui::help_dropped(&groups, 100);
        assert!(dropped >= 1, "too narrow for all");
        let kept: Vec<_> = groups[..groups.len() - dropped]
            .iter()
            .map(|g| g.name)
            .collect();
        assert!(
            kept.contains(&"FILES") && kept.contains(&"BANK"),
            "{kept:?}"
        );
        assert!(!kept.contains(&""), "the screens group is gone");
    }

    /// 43 "The owner looks for a way to remove a cut": under kitty the chop line names
    /// Backspace "remove cut" in the CUT group.
    #[test]
    fn the_owner_looks_for_a_way_to_remove_a_cut() {
        let groups = kitty_groups(input::Screen::Chop);
        let cut = groups.iter().find(|g| g.name == "CUT").unwrap();
        assert!(
            cut.items.contains(&("Bksp".into(), "remove cut".into())),
            "{:?}",
            cut.items
        );
    }

    /// screen-help 21: "The chop line reads as jobs", with the kitty protocol.
    #[test]
    fn the_chop_line_reads_as_jobs() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(1));
        let groups = super::without_modifiers(super::help_groups(
            &keymap,
            input::Screen::Chop,
            &app.help_state(),
        ));
        let names: Vec<_> = groups.iter().map(|g| g.name).collect();
        assert_eq!(&names[..4], ["MOVE", "CUT", "PADS", "BOUNDS"], "{names:?}");
        let items = line(&groups);
        for want in [
            ("J L", "cursor"),
            ("I K", "zoom"),
            ("M", "cut"),
            ("Y H", "parts/hits"),
            ("Ctrl J L", "left"),
            ("Alt J L", "right"),
            ("Ctrl ,", "whole"),
            ("Ctrl Z", "undo"),
            ("P", "to pads"),
            ("Enter", "edit"),
        ] {
            assert!(
                items.contains(&(want.0.into(), want.1.into())),
                "{want:?} missing in {items:?}"
            );
        }
        assert!(
            items.iter().all(|(_, t)| t.split(' ').count() <= 2),
            "a key and two words at most: {items:?}"
        );
    }

    /// screen-help 21: "Hold Alt".
    #[test]
    fn hold_alt() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let items = line(&super::modifier_groups(
            &keymap,
            input::Screen::Chop,
            input::Modifier::Alt,
            false,
        ));
        assert_eq!(
            items,
            [
                ("Bksp".to_string(), "delete right".to_string()),
                ("E".to_string(), "pattern erase".to_string()),
                ("I".to_string(), "zoom in".to_string()),
                ("J L".to_string(), "right bound".to_string()),
                ("K".to_string(), "zoom out".to_string()),
                ("M".to_string(), "metro".to_string()),
                ("N".to_string(), "pattern controls".to_string()),
                // 49: the loop of the play; 50: play as the pad
                ("O".to_string(), "loop".to_string()),
                ("P".to_string(), "as pad".to_string()),
                ("R".to_string(), "mic: hold".to_string()),
                ("Z".to_string(), "pattern undo".to_string())
            ],
            "the Alt combos of the chop screen and nothing else"
        );
        // the usual line with the kitty protocol has no Alt key
        assert!(
            line(&pads_line(true))
                .iter()
                .all(|(k, _)| !k.contains("Alt+")),
            "{:?}",
            line(&pads_line(true))
        );
    }

    #[test]
    fn hold_shift_shows_only_velocity() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let groups =
            super::modifier_groups(&keymap, input::Screen::Files, input::Modifier::Shift, false);
        assert_eq!(
            line(&groups),
            [(
                "1-4 Q-R A-F Z-V".to_string(),
                "pads at full velocity".to_string()
            )]
        );
    }

    #[test]
    fn one_row_on_the_pads_screen() {
        let groups = pads_line(true);
        let items = line(&groups);
        assert!(
            items.iter().all(|(k, _)| !k.contains("Ctrl+")),
            "no Ctrl combos while Ctrl is not held: {items:?}"
        );
        assert!(
            items.iter().all(|(k, _)| k != "Ctrl"),
            "nothing about modifiers"
        );
        let longest = groups.iter().map(|g| g.items.len()).max().unwrap().min(3);
        assert_eq!(
            tui::help_rows(&groups, 140),
            longest,
            "the columns take the rows of their longest group"
        );
    }

    #[test]
    fn state_shown_once() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let groups = super::help_groups(
            &keymap,
            input::Screen::Pads,
            &HelpState {
                hand: None,
                preview_on: false,
                assign: false,
                snap: super::chop::Snap::Off,
                tab: 0,
                param_name: "volume",
                page: "TONE",
                play: "to end",
                looping: false,
                as_pad: false,
                marking: false,
                scope: String::new(),
                edit: false,
            },
        );
        let items = line(&groups);
        assert!(items.contains(&("H".into(), "mode".into())), "{items:?}");
        assert!(
            items.contains(&("Enter".into(), "edit".into())),
            "{items:?}"
        );
        assert!(items.contains(&("U O".into(), "bank".into())), "{items:?}");
        assert!(
            items
                .iter()
                .all(|(_, t)| !t.contains("gate") && !t.contains("dB")),
            "the pad panel shows the mode and the volume: {items:?}"
        );
        // the files screen shows the preview state nowhere else
        let files = super::help_groups(
            &keymap,
            input::Screen::Files,
            &HelpState {
                hand: None,
                preview_on: false,
                assign: false,
                snap: super::chop::Snap::Off,
                tab: 0,
                param_name: "volume",
                page: "TONE",
                play: "to end",
                looping: false,
                as_pad: false,
                marking: false,
                scope: String::new(),
                edit: false,
            },
        );
        assert!(line(&files).contains(&("Space".into(), "preview: off".into())));
    }

    #[test]
    fn the_cursor_mode_is_in_the_help_line() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        use super::chop::Snap;
        for (snap, text) in [
            (Snap::Off, "snap: off"),
            (Snap::Hits, "snap: hits"),
            (Snap::Cuts, "snap: cuts"),
        ] {
            let groups = super::help_groups(
                &keymap,
                input::Screen::Chop,
                &HelpState {
                    hand: None,
                    preview_on: false,
                    assign: false,
                    snap,
                    tab: 0,
                    param_name: "volume",
                    page: "TONE",
                    play: "to end",
                    looping: false,
                    as_pad: false,
                    marking: false,
                    scope: String::new(),
                    edit: false,
                },
            );
            assert!(line(&groups).contains(&("N".into(), text.into())));
        }
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(super::chop::Chop::new(
            "b.wav".into(),
            std::sync::Arc::new(engine::Sample::mono(vec![0.0; 48_000])),
            48_000,
        ));
        app.apply(input::Action::ChopSnapToggle);
        assert_eq!(
            app.chop.as_ref().unwrap().snap,
            Snap::Hits,
            "N turns hits mode on"
        );
    }

    /// Recorded: K held for five repeats on the MIDI tab. 43 "Down the MIDI list".
    #[test]
    fn down_the_midi_list() {
        use ratatui::crossterm::event::{
            KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
        };
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = test_app();
        app.screen = Screen::Settings;
        app.settings_tab = 2;
        assert!(app.midi_list().len() > 6);
        let key = |kind| {
            KeyEvent::new_with_kind_and_state(
                KeyCode::Char('k'),
                KeyModifiers::NONE,
                kind,
                KeyEventState::NONE,
            )
        };
        let mut events = vec![key(KeyEventKind::Press)];
        events.extend([key(KeyEventKind::Repeat); 5]);
        for e in events {
            if let Some(a) = keymap.resolve_in(Screen::Settings, false, &e) {
                app.apply(a);
            }
        }
        assert_eq!(app.midi_row, 6, "the press and every repeat");
    }

    /// The app with a default controller on "X" and the learn mode open on `screen`.
    fn learning_on(screen: Screen, edit: bool) -> super::App {
        let mut app = test_app();
        app.midi_devices.push(("X".into(), "default".into()));
        app.screen = screen;
        app.edit = edit;
        app.apply(Action::LearnToggle);
        assert!(app.screen_learn.is_some(), "F10 opens the learn mode");
        app
    }

    /// A key in the learn mode.
    fn learn_press(app: &mut super::App, code: ratatui::crossterm::event::KeyCode) {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let k = ratatui::crossterm::event::KeyEvent::new(
            code,
            ratatui::crossterm::event::KeyModifiers::NONE,
        );
        assert!(app.learn_key(&keymap, &k), "the mode takes every key");
    }

    /// A controller message from "X".
    fn controller(app: &mut super::App, msg: input::midi::Msg) {
        app.midi_event(super::midi::Event::Message {
            port: "X".into(),
            profile: "default".into(),
            msg,
            meaning: String::new(),
        });
    }

    /// A knob turned on "X": a few values, then the learn window passes.
    fn turn_knob(app: &mut super::App, cc: u8) {
        for value in [60, 61, 62, 63] {
            controller(app, input::midi::Msg::Cc { ch: 1, cc, value });
        }
        let later = std::time::Instant::now() + std::time::Duration::from_millis(600);
        if let Some((profile, key)) = app
            .screen_learn
            .as_mut()
            .and_then(|l| l.learner.as_mut()?.poll(later))
        {
            app.learn_bind(profile, key);
        }
    }

    /// The picked element of the learn mode: (action, label).
    fn picked(app: &super::App) -> (&'static str, String) {
        app.learn_picked().expect("something is picked")
    }

    /// What a message from "X" does on `screen` now, with the learned assignments.
    fn resolved(
        app: &super::App,
        screen: input::Screen,
        edit: bool,
        msg: input::midi::Msg,
    ) -> Option<input::midi::Resolved> {
        app.live_profiles()[0].resolve_in(screen, edit, msg)
    }

    /// 52 "The cutoff on a knob": k5 learned for the cutoff turns the cutoff with the edit
    /// closed and with it open on ENV; the decay does not move.
    #[test]
    fn the_cutoff_on_a_knob() {
        let mut app = test_app();
        app.midi_devices.push(("X".into(), "default".into()));
        let k5 = input::midi::Key::Cc(28);
        super::learn::assign(
            &mut app.learned,
            "default",
            input::Group::Pads,
            k5,
            "pad.cutoff",
        );
        let turn = |app: &mut super::App, edit: bool, value: u8| {
            let msg = input::midi::Msg::Cc {
                ch: 1,
                cc: 28,
                value,
            };
            let Some(input::midi::Resolved::Param { param, value, .. }) =
                resolved(app, input::Screen::Pads, edit, msg)
            else {
                panic!("k5 drives a parameter");
            };
            assert_eq!(param, input::midi::Param::PadCutoff, "edit {edit}");
            app.midi_event(super::midi::Event::Param {
                port: "X".into(),
                control: "cc 28".into(),
                param,
                value,
                takeover: false,
            });
        };
        let before = app.pads.cutoff_hz[0];
        turn(&mut app, false, 20);
        let closed = app.pads.cutoff_hz[0];
        assert!(closed < before, "the edit closed: {before} -> {closed}");
        app.apply(Action::PadEdit);
        while app.pads.page().0 != "ENV" {
            app.apply(Action::ParamNext);
        }
        let decay = app.pads.decay_ms[0];
        turn(&mut app, true, 100);
        assert!(
            app.pads.cutoff_hz[0] > closed,
            "the edit open on ENV: the cutoff moves"
        );
        assert_eq!(app.pads.decay_ms[0], decay, "the decay does not");
    }

    /// 52 "Pick by the pad key": the pad key picks the pad, lights it, plays nothing, and
    /// the top line says what is picked.
    #[test]
    fn pick_by_the_pad_key() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = learning_on(Screen::Pads, false);
        app.sent.clear();
        learn_press(&mut app, KeyCode::Char('q'));
        assert_eq!(picked(&app), ("pad.5", "pad Q".to_string()));
        assert!(
            !app.sent
                .iter()
                .any(|c| matches!(c, Command::PadPress { .. })),
            "silent"
        );
        let lines = app.learn_lines().unwrap();
        assert!(lines[0].contains("picked: pad Q"), "{lines:?}");
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        assert!(app.mark_pads(v).pads[4].in_hand, "pad Q is lit");
    }

    /// 57 "A turn waits for Enter": controller pad 5 hit for pad Q stands next to "none" in
    /// the pending colour and binds nothing; Enter binds it and picks pad W; the same
    /// controller pad hit again does not stand for pad W, another one does.
    #[test]
    fn a_turn_waits_for_enter() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = learning_on(Screen::Pads, false);
        learn_press(&mut app, KeyCode::Char('q'));
        let hit = input::midi::Msg::NoteOn {
            ch: 1,
            note: 90,
            velocity: 100,
        };
        let plays = |app: &super::App, msg| match resolved(app, input::Screen::Pads, false, msg) {
            Some(input::midi::Resolved::Action(Action::PadPress { pad, .. })) => Some(pad),
            _ => None,
        };
        controller(&mut app, hit);
        assert_eq!(plays(&app, hit), None, "nothing is bound before Enter");
        let shown = app.learn_shown(&app.live_profiles(), "pad.5");
        assert_eq!(shown.as_deref(), Some("none → note 90"));
        assert!(app.learn_lines().unwrap()[0].contains("Enter bind"));
        learn_press(&mut app, KeyCode::Enter);
        assert_eq!(plays(&app, hit), Some(4), "Enter binds pad Q");
        assert_eq!(picked(&app), ("pad.6", "pad W".to_string()), "the next pad");
        controller(&mut app, hit);
        assert!(
            app.screen_learn.as_ref().unwrap().pending.is_none(),
            "the control just bound is not heard"
        );
        let other = input::midi::Msg::NoteOn {
            ch: 1,
            note: 91,
            velocity: 100,
        };
        controller(&mut app, other);
        learn_press(&mut app, KeyCode::Enter);
        assert_eq!(plays(&app, other), Some(5));
    }

    /// 57 "The binding is shown" and "The panel row shows the pending control".
    #[test]
    fn the_panel_row_shows_the_pending_control() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = learning_on(Screen::Pads, false);
        super::learn::assign(
            &mut app.learned,
            "default",
            input::Group::Pads,
            input::midi::Key::Cc(24),
            "pad.pitch",
        );
        learn_press(&mut app, KeyCode::Char('q'));
        assert_eq!(
            app.learn_shown(&app.live_profiles(), "pad.5").as_deref(),
            Some("none"),
            "a picked pad with no control says none"
        );
        while app.learn_picked().map(|p| p.0) != Some("pad.pitch") {
            learn_press(&mut app, KeyCode::Char('k'));
        }
        assert!(app.learn_lines().unwrap()[0].contains("now cc 24"));
        turn_knob(&mut app, 26);
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        let v = app.mark_pads(v);
        assert!(
            v.learn
                .contains(&(tui::PadParam::Pitch, "cc 24 → cc 26".to_string())),
            "{:?}",
            v.learn
        );
        let cc = input::midi::Msg::Cc {
            ch: 1,
            cc: 24,
            value: 90,
        };
        assert!(
            matches!(
                resolved(&app, input::Screen::Pads, false, cc),
                Some(input::midi::Resolved::Param {
                    param: input::midi::Param::PadPitch,
                    ..
                })
            ),
            "the pitch still follows cc 24"
        );
    }

    /// 57 "Esc keeps the old one" and "Another control takes its place".
    #[test]
    fn esc_keeps_the_old_one() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = learning_on(Screen::Pads, false);
        super::learn::assign(
            &mut app.learned,
            "default",
            input::Group::Pads,
            input::midi::Key::Cc(20),
            "pad.cutoff",
        );
        while app.learn_picked().map(|p| p.0) != Some("pad.cutoff") {
            learn_press(&mut app, KeyCode::Char('k'));
        }
        turn_knob(&mut app, 26);
        assert_eq!(
            app.learn_shown(&app.live_profiles(), "pad.cutoff")
                .as_deref(),
            Some("cc 20 → cc 26")
        );
        turn_knob(&mut app, 27);
        assert_eq!(
            app.learn_shown(&app.live_profiles(), "pad.cutoff")
                .as_deref(),
            Some("cc 20 → cc 27"),
            "another control takes its place"
        );
        learn_press(&mut app, KeyCode::Esc);
        assert_eq!(
            app.learn_shown(&app.live_profiles(), "pad.cutoff")
                .as_deref(),
            Some("cc 20"),
            "the old one stays"
        );
        assert_eq!(picked(&app).0, "pad.cutoff", "the pick stays");
    }

    /// line asks for a button.
    #[test]
    fn a_knob_is_not_a_button() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = learning_on(Screen::Pads, false);
        learn_press(&mut app, KeyCode::PageDown);
        turn_knob(&mut app, 34);
        assert!(!app.learned.contains_key("default"), "nothing bound");
        assert!(app.learn_lines().unwrap()[0].contains("needs a button"));
    }

    /// 52 "A taken control": k5 turns the volume; the cutoff is picked with I K and k5
    /// turned: the volume row says it leaves; Esc keeps k5 on the volume, Enter moves it.
    #[test]
    fn a_taken_control() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = learning_on(Screen::Pads, false);
        let k5 = input::midi::Key::Cc(28);
        super::learn::assign(
            &mut app.learned,
            "default",
            input::Group::Pads,
            k5,
            "pad.volume",
        );
        while app.learn_picked().map(|p| p.0) != Some("pad.cutoff") {
            learn_press(&mut app, KeyCode::Char('k'));
        }
        turn_knob(&mut app, 28);
        assert_eq!(app.learn_taken().map(|t| t.0), Some("pad.volume"));
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        let v = app.mark_pads(v);
        assert!(
            v.learn
                .contains(&(tui::PadParam::Volume, "cc 28  ✗ leaves".to_string())),
            "{:?}",
            v.learn
        );
        learn_press(&mut app, KeyCode::Esc);
        let cc = input::midi::Msg::Cc {
            ch: 1,
            cc: 28,
            value: 90,
        };
        let param = |app: &super::App| match resolved(app, input::Screen::Pads, false, cc) {
            Some(input::midi::Resolved::Param { param, .. }) => Some(param),
            _ => None,
        };
        assert_eq!(
            param(&app),
            Some(input::midi::Param::PadVolume),
            "Esc: as before"
        );
        turn_knob(&mut app, 28);
        learn_press(&mut app, KeyCode::Enter);
        assert_eq!(
            param(&app),
            Some(input::midi::Param::PadCutoff),
            "Enter: moved"
        );
    }

    /// 52 "A command by its key": M on the chop screen picks "cut" and cuts nothing.
    #[test]
    fn a_command_by_its_key() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = test_app();
        app.chop = Some(chop_with_parts(1));
        app.screen = Screen::Chop;
        app.apply(Action::LearnToggle);
        let cuts = app.chop.as_ref().unwrap().markers.clone();
        learn_press(&mut app, KeyCode::Char('m'));
        assert_eq!(app.learn_picked().map(|p| p.0), Some("chop.cut"));
        assert_eq!(app.chop.as_ref().unwrap().markers, cuts, "no cut made");
    }

    /// 57 "Clear waits for Enter": Backspace on the bound cutoff shows `cc 28 → none`,
    /// Enter clears it.
    #[test]
    fn clear_asks_too() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = learning_on(Screen::Pads, false);
        let k5 = input::midi::Key::Cc(28);
        super::learn::assign(
            &mut app.learned,
            "default",
            input::Group::Pads,
            k5,
            "pad.cutoff",
        );
        while app.learn_picked().map(|p| p.0) != Some("pad.cutoff") {
            learn_press(&mut app, KeyCode::Char('k'));
        }
        learn_press(&mut app, KeyCode::Backspace);
        assert_eq!(
            app.learn_shown(&app.live_profiles(), "pad.cutoff")
                .as_deref(),
            Some("cc 28 → none")
        );
        assert!(app.learn_lines().unwrap()[0].contains("Enter clear"));
        assert!(
            app.learn_label(&app.live_profiles(), "pad.cutoff")
                .is_some(),
            "not yet"
        );
        learn_press(&mut app, KeyCode::Enter);
        assert_eq!(app.learn_label(&app.live_profiles(), "pad.cutoff"), None);
    }

    /// 47 "Silent while learning": pad keys and command keys pick, nothing sounds or acts;
    /// a controller pad hit plays nothing either.
    #[test]
    fn silent_while_learning() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = learning_on(Screen::Pads, false);
        app.sent.clear();
        learn_press(&mut app, KeyCode::Char('q'));
        learn_press(&mut app, KeyCode::Char('h')); // the pad mode key
        app.midi_event(super::midi::Event::PadHit {
            sound: super::midi::PadSound::DirectRejected,
            abs: 3,
            velocity: 1.0,
            ui_press: None,
        });
        assert!(
            !app.sent
                .iter()
                .any(|c| matches!(c, Command::PadPress { .. } | Command::SetPadMode { .. })),
            "{:?}",
            app.sent
        );
        assert_eq!(
            app.learn_picked().map(|p| p.0),
            Some("pad.mode_next"),
            "H picks its command"
        );
    }

    /// 57 "Do, then F10": the cutoff stepped with L in the pad edit, F10 within 5 s picks
    /// it in PADS · EDIT · FILTER; k5 waits for Enter, then turns the cutoff in the edit
    /// only, and the mode closes.
    #[test]
    fn do_then_f10() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = test_app();
        app.midi_devices.push(("X".into(), "default".into()));
        app.apply(Action::PadEdit);
        while app.pads.page().0 != "FILTER" {
            app.apply(Action::PageNext);
        }
        app.apply(Action::ParamNext); // the cutoff
        app.apply(Action::EditValue {
            up: true,
            fine: true,
        });
        app.apply(Action::LearnToggle);
        assert_eq!(picked(&app).0, "pad.cutoff");
        let lines = app.learn_lines().unwrap();
        assert!(
            lines[0].contains("cutoff (the last thing you did) · PADS · EDIT · FILTER"),
            "{lines:?}"
        );
        turn_knob(&mut app, 28);
        assert_eq!(
            app.learn_shown(&app.live_profiles(), "pad.cutoff")
                .as_deref(),
            Some("none → cc 28")
        );
        learn_press(&mut app, KeyCode::Enter);
        assert!(app.screen_learn.is_none(), "one element: the mode closes");
        assert_eq!(
            app.learned["default"]["pads_edit"]
                .get("cc 28")
                .map(String::as_str),
            Some("pad.cutoff")
        );
        let cc = input::midi::Msg::Cc {
            ch: 1,
            cc: 28,
            value: 90,
        };
        assert!(matches!(
            resolved(&app, input::Screen::Pads, true, cc),
            Some(input::midi::Resolved::Param {
                param: input::midi::Param::PadCutoff,
                ..
            })
        ));
        assert!(
            resolved(&app, input::Screen::Pads, false, cc).is_none(),
            "with the edit closed k5 keeps what it had: nothing"
        );
    }

    /// 57 "The cutoff in the edit on F12": a knob bound to the cutoff in the pad edit shows
    /// on the "pad cutoff" row of PADS · EDIT, and Backspace there clears it.
    #[test]
    fn the_cutoff_in_the_edit_on_f12() {
        let mut app = test_app();
        app.midi_devices.push(("X".into(), "default".into()));
        let k5 = input::midi::Key::Cc(28);
        super::learn::assign(
            &mut app.learned,
            "default",
            input::Group::PadsEdit,
            k5,
            "pad.cutoff",
        );
        midi_tab(&mut app);
        app.midi_open.push(input::Group::PadsEdit);
        let at = app
            .midi_list()
            .iter()
            .position(|r| {
                matches!(r, super::MidiListRow::Action { info }
                    if info.name == "pad.cutoff" && info.group == input::Group::PadsEdit)
            })
            .expect("a pad cutoff row in PADS · EDIT");
        match &app.midi_screen()[at] {
            tui::MidiRow::Action { who, .. } => assert!(who.contains("cc 28"), "{who}"),
            _ => panic!("an action row"),
        }
        app.midi_row = at;
        app.apply(Action::SettingsClear);
        assert!(
            !super::learn::group_map(&app.learned, "default", input::Group::PadsEdit)
                .contains_key(&k5),
            "Backspace clears it"
        );
    }

    /// 57 "The line says where the pick came from": F10, then Enter picks "edit pad" by its
    /// key; a knob is refused with the reason and how to pick another; a pick by "do, then
    /// F10" says it is the last thing done.
    #[test]
    fn the_line_says_where_the_pick_came_from() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = learning_on(Screen::Pads, false);
        learn_press(&mut app, KeyCode::Enter);
        assert_eq!(picked(&app).0, "pad.edit");
        turn_knob(&mut app, 30);
        let lines = app.learn_lines().unwrap().join(" ");
        assert!(lines.contains("needs a button"), "{lines}");
        assert!(lines.contains("I K pick a parameter"), "{lines}");
        assert!(!lines.contains("the last thing you did"), "{lines}");
        // do, then F10
        app.apply(Action::LearnToggle);
        app.apply(Action::BankNext);
        app.apply(Action::LearnToggle);
        turn_knob(&mut app, 30);
        let lines = app.learn_lines().unwrap().join(" ");
        assert!(lines.contains("the last thing you did"), "{lines}");
    }

    /// 57 "Too late for the last action": F10 10 s after PgDn opens with nothing picked.
    #[test]
    fn too_late_for_the_last_action() {
        let mut app = test_app();
        app.midi_devices.push(("X".into(), "default".into()));
        app.apply(Action::BankNext);
        if let Some(d) = app.last_done.as_mut() {
            d.at -= std::time::Duration::from_secs(10);
        }
        app.apply(Action::LearnToggle);
        assert!(app.learn_picked().is_none());
        // in time: PgDn picks the next bank
        app.apply(Action::LearnToggle);
        app.apply(Action::BankNext);
        app.apply(Action::LearnToggle);
        assert_eq!(picked(&app).0, "bank.next");
    }

    /// 57 "A button for a parameter": b6 for the cutoff stands for nothing, the line says
    /// why; "A knob is not a button" says why too.
    #[test]
    fn a_button_for_a_parameter() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = learning_on(Screen::Pads, false);
        while app.learn_picked().map(|p| p.0) != Some("pad.cutoff") {
            learn_press(&mut app, KeyCode::Char('k'));
        }
        controller(
            &mut app,
            input::midi::Msg::NoteOn {
                ch: 1,
                note: 60,
                velocity: 100,
            },
        );
        assert!(app.screen_learn.as_ref().unwrap().pending.is_none());
        let lines = app.learn_lines().unwrap().join(" ");
        assert!(lines.contains("needs a knob or a fader"), "{lines}");
        assert!(lines.contains("a button only presses"), "{lines}");
        learn_press(&mut app, KeyCode::PageDown);
        turn_knob(&mut app, 30);
        let lines = app.learn_lines().unwrap().join(" ");
        assert!(lines.contains("needs a button or a pad"), "{lines}");
        assert!(lines.contains("repeat it on every tick"), "{lines}");
    }

    /// 57 "Taken in this layer" and "Bound in another layer".
    #[test]
    fn taken_in_this_layer_and_bound_in_another() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = learning_on(Screen::Pads, true);
        let k5 = input::midi::Key::Cc(28);
        super::learn::assign(
            &mut app.learned,
            "default",
            input::Group::PadsEdit,
            k5,
            "pad.reso",
        );
        super::learn::assign(
            &mut app.learned,
            "default",
            input::Group::Pads,
            k5,
            "pad.volume",
        );
        while app.learn_picked().map(|p| p.0) != Some("pad.cutoff") {
            learn_press(&mut app, KeyCode::Char('k'));
        }
        turn_knob(&mut app, 28);
        let lines = app.learn_lines().unwrap().join(" ");
        assert!(lines.contains("taken on this layer"), "{lines}");
        assert_eq!(app.learn_taken().map(|t| t.0), Some("pad.reso"));
        learn_press(&mut app, KeyCode::Esc);
        // another layer only: a plain note, no warning
        super::learn::clear(&mut app.learned, input::Group::PadsEdit, "pad.reso");
        turn_knob(&mut app, 28);
        let lines = app.learn_lines().unwrap().join(" ");
        assert!(!lines.contains("taken on this layer"), "{lines}");
        assert!(lines.contains("cc 28 on PADS: pad volume"), "{lines}");
        learn_press(&mut app, KeyCode::Enter);
        let cc = input::midi::Msg::Cc {
            ch: 1,
            cc: 28,
            value: 90,
        };
        let param = |edit| match resolved(&app, input::Screen::Pads, edit, cc) {
            Some(input::midi::Resolved::Param { param, .. }) => Some(param),
            _ => None,
        };
        assert_eq!(param(true), Some(input::midi::Param::PadCutoff));
        assert_eq!(param(false), Some(input::midi::Param::PadVolume));
    }

    /// 47 "One screen at a time": b1 bound to the next bank on the pads screen (picked with
    /// PgDn) turns the bank there and does nothing new on the chop screen.
    #[test]
    fn one_screen_at_a_time() {
        use ratatui::crossterm::event::KeyCode;
        let mut app = learning_on(Screen::Pads, false);
        learn_press(&mut app, KeyCode::PageDown);
        assert_eq!(app.learn_picked().map(|p| p.0), Some("bank.next"));
        let b1 = input::midi::Msg::NoteOn {
            ch: 1,
            note: 90,
            velocity: 100,
        };
        controller(&mut app, b1);
        learn_press(&mut app, KeyCode::Enter);
        learn_press(&mut app, KeyCode::F(10));
        assert!(app.screen_learn.is_none());
        assert_eq!(
            resolved(&app, input::Screen::Pads, false, b1),
            Some(input::midi::Resolved::Action(Action::BankNext))
        );
        assert_eq!(resolved(&app, input::Screen::Chop, false, b1), None);
    }

    /// 49 "Space stops the loop" and "Off plays once", on recorded keys: Alt+O, Space
    /// plays the slice looped; Space again stops it; with the loop off Space plays once.
    #[test]
    fn space_loops_and_stops() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = chop_open();
        app.chop.as_mut().unwrap().play = super::chop::PlayMode::Slice;
        app.chop.as_mut().unwrap().set_cursor(30_000);
        let slice = app.chop.as_ref().unwrap().play_region();
        // off: once
        app.sent.clear();
        chop_key(&mut app, KeyCode::Char(' '), M::NONE);
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::PreviewLoop { region: None }))
        );
        assert!(
            !app.sent
                .iter()
                .any(|c| matches!(c, Command::PreviewLoop { region: Some(_) }))
        );
        // on: the loop, then Space stops it
        app.fake_playhead = None;
        chop_key(&mut app, KeyCode::Char('o'), M::ALT);
        assert!(app.chop.as_ref().unwrap().looping);
        app.sent.clear();
        chop_key(&mut app, KeyCode::Char(' '), M::NONE);
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::PreviewLoop { region: Some(r) } if *r == slice)),
            "{:?}",
            app.sent
        );
        app.fake_playhead = Some(slice.start + 10);
        app.sent.clear();
        chop_key(&mut app, KeyCode::Char(' '), M::NONE);
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::Preview { sound: None, .. }))
        );
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::PreviewLoop { region: None }))
        );
    }

    /// 49 "Move a bound while it loops", the app side: the end of the slice moved while the
    /// loop plays goes to the engine as the loop's new region.
    #[test]
    fn the_loop_follows_the_bounds() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = chop_open();
        app.chop.as_mut().unwrap().play = super::chop::PlayMode::Slice;
        app.chop.as_mut().unwrap().set_cursor(30_000);
        chop_key(&mut app, KeyCode::Char('o'), M::ALT);
        chop_key(&mut app, KeyCode::Char(' '), M::NONE);
        let before = app.chop.as_ref().unwrap().play_region();
        app.fake_playhead = Some(before.start + 10);
        app.sent.clear();
        chop_key(&mut app, KeyCode::Char('j'), M::ALT); // the end to the left
        let after = app.chop.as_ref().unwrap().play_region();
        assert!(after.end < before.end);
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::PreviewLoop { region: Some(r) } if *r == after)),
            "{:?}",
            app.sent
        );
        assert!(
            !app.sent
                .iter()
                .any(|c| matches!(c, Command::Preview { sound: None, .. })),
            "no stop"
        );
    }

    /// 50 "Slice 3 as its pad", the app side, and "A slice on no pad": with Alt+P on,
    /// Space on slice 3 plays it as the first pad that holds it; a slice on no pad plays
    /// bare; off, nothing is borrowed.
    #[test]
    fn space_plays_as_the_pad() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = laid_out_break(); // slices 1–8 on pads 4–11
        let borrowed = |app: &super::App| {
            app.sent.iter().find_map(|c| match c {
                Command::PreviewAs { pad } => Some(*pad),
                _ => None,
            })
        };
        let slices = app.chop.as_ref().unwrap().slices();
        app.chop.as_mut().unwrap().set_cursor(slices[2].start + 10);
        app.sent.clear();
        chop_key(&mut app, KeyCode::Char(' '), M::NONE);
        assert_eq!(borrowed(&app), Some(None), "off: bare");
        chop_key(&mut app, KeyCode::Char('p'), M::ALT);
        assert!(app.chop.as_ref().unwrap().as_pad);
        app.sent.clear();
        chop_key(&mut app, KeyCode::Char(' '), M::NONE);
        assert_eq!(borrowed(&app), Some(Some(6)), "slice 3 lies on pad 6 (E)");
        // a slice on no pad: take slice 3 off its pad
        app.pads.link[6] = None;
        app.pads.slice[6] = None;
        app.sent.clear();
        app.play_slice(2);
        assert_eq!(borrowed(&app), Some(None), "on no pad: bare");
    }

    /// 50 "The switch in the help line": Alt+P says "as pad" by the play mode; again, not.
    #[test]
    fn as_pad_in_the_help_line() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = chop_open();
        let o_item = |app: &super::App| {
            line(&super::help_groups(
                &keymap,
                input::Screen::Chop,
                &app.help_state(),
            ))
            .into_iter()
            .find(|(k, _)| k == "O")
            .map(|(_, t)| t)
        };
        chop_key(&mut app, KeyCode::Char('p'), M::ALT);
        assert_eq!(o_item(&app).as_deref(), Some("to end · as pad"));
        chop_key(&mut app, KeyCode::Char('p'), M::ALT);
        assert_eq!(o_item(&app).as_deref(), Some("to end"));
    }

    /// 49: the help line names the loop by the play mode while it is on.
    #[test]
    fn the_loop_in_the_help_line() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = chop_open();
        chop_key(&mut app, KeyCode::Char('o'), M::ALT);
        let items = line(&super::help_groups(
            &keymap,
            input::Screen::Chop,
            &app.help_state(),
        ));
        assert!(
            items.contains(&("O".into(), "to end · loop: on".into())),
            "{items:?}"
        );
    }

    /// 51 "Small rises are passed" and "The marks are the stops": hits at −4, −10 and
    /// −16 dB below the loudest; the −16 dB one is under the −12 dB threshold, the cursor
    /// passes it, and the marks are the three stops.
    #[test]
    fn hits_mode_stops_at_the_marks() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        chop_key(&mut app, KeyCode::Char('n'), M::NONE);
        let c = app.chop.as_ref().unwrap();
        let marks: Vec<usize> = super::chop_view(c, None, None, 48_000, &app.pads)
            .hits
            .iter()
            .map(|f| (f * (c.view.end - c.view.start) as f32).round() as usize + c.view.start)
            .collect();
        assert_eq!(
            marks,
            [24_000, 48_000, 72_000],
            "the −16 dB hit has no mark"
        );
        let mut stops = Vec::new();
        for _ in 0..3 {
            chop_key(&mut app, KeyCode::Char('l'), M::NONE);
            stops.push(app.chop.as_ref().unwrap().cursor);
        }
        assert_eq!(stops, marks, "the cursor stops at the marks");
        chop_key(&mut app, KeyCode::Char('l'), M::NONE);
        let next = app.chop.as_ref().unwrap().cursor;
        assert!(
            next > 72_000 && next != 96_000,
            "past the last mark: even steps, {next}"
        );
    }

    /// 51 chopping "H cuts at the marks by default": H lands the cuts on hit starts
    /// and "No marks in even steps": the cursor walks without marks.
    #[test]
    fn h_cuts_at_the_marks() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        let view = |app: &super::App| {
            let c = app.chop.as_ref().unwrap();
            super::chop_view(c, None, None, 48_000, &app.pads)
                .hits
                .len()
        };
        assert_eq!(view(&app), 0, "even steps: no marks");
        chop_key(&mut app, KeyCode::Char('h'), M::NONE); // −6 dB
        chop_key(&mut app, KeyCode::Char('n'), M::NONE);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.markers, [24_000, 48_000]);
        assert_eq!(
            c.strong_hits(),
            c.markers,
            "a mark at every cut and nowhere else"
        );
        assert_eq!(view(&app), 2);
    }

    /// 83a chopping "Ctrl+Y cuts at the marks": the old chord stays an owner-remap
    /// compatibility path; an explicit binding cuts at the hit starts as H does.
    #[test]
    fn ctrl_y_cuts_at_the_marks() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let (mut keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        keymap
            .overlay("[screens.chop]\n\"Ctrl+Y\" = \"chop.hits\"\n")
            .unwrap();
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        let k = ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('y'), M::CONTROL);
        app.apply(
            keymap
                .resolve_in(Screen::Chop, app.edit_keys(), &k)
                .unwrap(),
        );
        app.apply(Action::ChopSnapToggle);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.markers, [24_000, 48_000]);
        assert_eq!(c.strong_hits(), c.markers, "a mark at every cut");
    }

    /// 83a chopping "Reverse the hit threshold": −18 → Shift+H → −12, quieter cuts
    /// disappear, one Ctrl+Z restores the cuts and the −18 threshold.
    #[test]
    fn reverse_the_hit_threshold() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        app.chop.as_mut().unwrap().set_threshold(18.0 / 40.0);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.threshold_db, -18.0);
        assert_eq!(c.markers.len(), 4, "−18 keeps the 0.15 hit");
        // Shift+H as the extended event, then the legacy uppercase and Russian forms
        for code in [
            KeyCode::Char('h'),
            KeyCode::Char('H'),
            KeyCode::Char('р'),
            KeyCode::Char('Р'),
        ] {
            // the extended protocol names the base key and the modifier; without it
            // Shift arrives as an uppercase character (Р is the Russian H key)
            let m = if matches!(code, KeyCode::Char('h') | KeyCode::Char('р')) {
                M::SHIFT
            } else {
                M::NONE
            };
            app.chop.as_mut().unwrap().set_threshold(18.0 / 40.0);
            chop_key(&mut app, code, m);
            println!("flash: {}", flash(&app));
            let c = app.chop.as_ref().unwrap();
            assert_eq!(c.threshold_db, -12.0, "{code:?}");
            assert_eq!(c.markers, [24_000, 48_000, 72_000], "the 0.15 hit leaves");
            // were 5 slices: −18 keeps four cuts; −12 leaves three of them
            assert_eq!(
                flash(&app),
                "by hits at -12 dB: 4 slices (were 5) · Ctrl+Z brings them back",
                "{code:?}"
            );
        }
    }

    /// 83a chopping "Reverse hit threshold stops at the first step": −6 stays, no undo
    /// step is added and the flash names −6 dB.
    #[test]
    fn reverse_hit_threshold_stops_at_the_first_step() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        chop_key(&mut app, KeyCode::Char('h'), M::NONE); // −6, one undo step
        let cuts = app.chop.as_ref().unwrap().markers.clone();
        chop_key(&mut app, KeyCode::Char('h'), M::SHIFT);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.threshold_db, -6.0, "the first step holds");
        assert_eq!(c.markers, cuts);
        assert_eq!(flash(&app), "by hits stays at -6 dB");
        // the clamped step added no undo step: one Ctrl+Z undoes the −6 press, not it
        chop_key(&mut app, KeyCode::Char('z'), M::CONTROL);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.threshold_db, 0.0, "no explicit threshold again");
        assert_eq!(c.markers, [], "the whole region is one slice again");
    }

    /// 83a chopping "Reverse a continuous hit threshold": a knob at −15 steps to −12
    /// and the cuts follow the same hit rule.
    #[test]
    fn reverse_a_continuous_hit_threshold() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        app.chop.as_mut().unwrap().set_threshold(15.0 / 40.0);
        chop_key(&mut app, KeyCode::Char('h'), M::SHIFT);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.threshold_db, -12.0, "the next higher listed step");
        assert_eq!(c.markers, [24_000, 48_000, 72_000]);
    }

    /// 83a chopping "Reverse keeps a higher knob threshold": −3 stays and nothing changes.
    #[test]
    fn reverse_keeps_a_higher_knob_threshold() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        app.chop.as_mut().unwrap().set_threshold(3.0 / 40.0);
        let cuts = app.chop.as_ref().unwrap().markers.clone();
        chop_key(&mut app, KeyCode::Char('h'), M::SHIFT);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.threshold_db, -3.0);
        assert_eq!(c.markers, cuts, "no cuts change");
        assert_eq!(flash(&app), "by hits stays at -3 dB");
    }

    /// 83a chopping "First reverse hit command": without an explicit threshold Shift+H
    /// starts at −6 and the status names the threshold.
    #[test]
    fn first_reverse_hit_command() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        chop_key(&mut app, KeyCode::Char('h'), M::SHIFT);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.threshold_db, -6.0);
        assert_eq!(c.markers, [24_000, 48_000]);
        assert_eq!(flash(&app), "by hits at -6 dB: 3 slices");
    }

    /// 83a chopping "Tab has the named assign order action": Tab switches by-hand and
    /// in-order only while a plan is open and applies none of them.
    #[test]
    fn tab_has_the_named_assign_order_action() {
        use super::chop::Plan;
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(4));
        app.apply(Action::ChopAssign);
        assert!(matches!(app.plan, Some(Plan::ByHand(_))));
        chop_key(&mut app, KeyCode::Tab, M::NONE);
        assert_eq!(app.plan, Some(Plan::InOrder(None)));
        // the next Tab returns to an empty by-hand plan without applying either
        let files: Vec<_> = app.pads.file.iter().filter_map(Clone::clone).collect();
        chop_key(&mut app, KeyCode::Tab, M::NONE);
        assert_eq!(app.plan, Some(Plan::ByHand(Vec::new())));
        assert!(app.sent.is_empty(), "no slice sounds");
        assert_eq!(
            app.pads
                .file
                .iter()
                .filter_map(Clone::clone)
                .collect::<Vec<_>>(),
            files,
            "no plan was applied"
        );
        // outside a plan Tab does nothing: listen and slice edit keep their keys
        app.plan = None;
        app.edit = true;
        let slices = app.chop.as_ref().unwrap().slices().to_vec();
        chop_key(&mut app, KeyCode::Tab, M::NONE);
        assert_eq!(app.plan, None);
        assert_eq!(app.chop.as_ref().unwrap().slices().to_vec(), slices);
    }

    /// 83a chopping "Old parameter binding still switches assign order": `edit.param_next`
    /// keeps switching while assigning, as a learned controller button or an owner key.
    #[test]
    fn old_parameter_binding_still_switches_assign_order() {
        use super::chop::Plan;
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(4));
        app.apply(Action::ChopAssign);
        app.apply(Action::ParamNext);
        assert_eq!(app.plan, Some(Plan::InOrder(None)));
        app.apply(Action::ParamNext);
        assert!(matches!(app.plan, Some(Plan::ByHand(_))));
    }

    /// 83a chopping "Old auto mode binding still switches assign order": the former
    /// `chop.auto_mode_next` keeps switching by-hand and in-order while assigning.
    #[test]
    fn old_auto_mode_binding_still_switches_assign_order() {
        use super::chop::Plan;
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(4));
        app.apply(Action::ChopAssign);
        assert_eq!(
            Action::parse("chop.auto_mode_next"),
            Some(Action::ChopParts)
        );
        app.apply(Action::ChopParts);
        assert_eq!(app.plan, Some(Plan::InOrder(None)));
        app.apply(Action::ChopParts);
        assert!(matches!(app.plan, Some(Plan::ByHand(_))));
    }

    /// Recorded: N three times on the chop screen. 43 "N walks the three modes".
    #[test]
    fn n_walks_the_three_modes() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = chop_open();
        let mut said = Vec::new();
        for _ in 0..3 {
            screen_key(&mut app, Screen::Chop, KeyCode::Char('n'), M::NONE);
            said.push(format!("snap: {}", app.chop.as_ref().unwrap().snap.name()));
        }
        assert_eq!(said, ["snap: hits", "snap: cuts", "snap: off"]);
    }

    /// Recorded: N twice, then L L J inside slice 2 of 4. 43 "Cut to cut".
    #[test]
    fn cut_to_cut() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(2)); // cuts at 24_000, 48_000, 72_000
        app.chop.as_mut().unwrap().set_cursor(30_000);
        let slices = app.chop.as_ref().unwrap().slices();
        for _ in 0..2 {
            screen_key(&mut app, Screen::Chop, KeyCode::Char('n'), M::NONE);
        }
        let mut at = Vec::new();
        for key in ['l', 'l', 'j'] {
            screen_key(&mut app, Screen::Chop, KeyCode::Char(key), M::NONE);
            at.push(app.chop.as_ref().unwrap().cursor);
        }
        assert_eq!(at, [48_000, 72_000, 48_000], "slice 3, slice 4, back to 3");
        assert_eq!(
            played(&app),
            [slices[2], slices[3], slices[2]],
            "each slice plays"
        );
    }

    /// Recorded: cuts mode, L in the last slice, J in the first. 43 "To the ends".
    #[test]
    fn to_the_ends() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(2));
        app.chop.as_mut().unwrap().snap = super::chop::Snap::Cuts;
        app.chop.as_mut().unwrap().set_cursor(80_000);
        screen_key(&mut app, Screen::Chop, KeyCode::Char('l'), M::NONE);
        assert_eq!(
            app.chop.as_ref().unwrap().cursor,
            96_000,
            "the end of the region"
        );
        app.chop.as_mut().unwrap().set_cursor(10_000);
        screen_key(&mut app, Screen::Chop, KeyCode::Char('j'), M::NONE);
        assert_eq!(
            app.chop.as_ref().unwrap().cursor,
            0,
            "the start of the first slice"
        );
    }

    #[test]
    fn the_status_says_the_pads_changed() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        let mut c = super::chop::Chop::new(
            "break.wav".into(),
            std::sync::Arc::new(engine::Sample::mono(vec![0.0; 96_000])),
            48_000,
        );
        c.next_parts();
        for (i, slice) in c.slices().into_iter().enumerate() {
            app.pads.file[i] = Some(c.file.clone());
            app.pads.region[i] = slice;
            app.pads.link[i] = Some((c.start_id(i), c.epoch));
        }
        c.select_slice(1);
        app.chop = Some(c);
        let status = |app: &super::App| last_notice(app).map(str::to_owned);
        app.apply(input::Action::MarkerNudge {
            right: true,
            fine: false,
        });
        assert_eq!(
            status(&app).as_deref(),
            Some("2 pads changed; Ctrl+Z returns")
        );
        app.apply(input::Action::Undo);
        assert_eq!(status(&app).as_deref(), Some("2 pads changed back"));
        // an edit that no pad hears says nothing
        app.notices = Notices::new(Instant::now());
        app.apply(input::Action::ChopZoom { zoom_in: true });
        assert_eq!(status(&app), None);
    }

    /// A chop of "break.wav" with `parts` steps of equal parts applied.
    fn chop_with_parts(parts: usize) -> super::chop::Chop {
        let mut c = super::chop::Chop::new(
            "break.wav".into(),
            std::sync::Arc::new(engine::Sample::mono(vec![0.0; 96_000])),
            48_000,
        );
        for _ in 0..parts {
            c.next_parts();
        }
        c
    }

    /// A chop of a wave with one hit per amplitude at 24_000·(n+1), each decaying over
    /// 4_000 frames; the detector finds each start exactly.
    fn chop_hits(amps: &[f32]) -> super::chop::Chop {
        let mut wave = vec![0.0; 48_000 * 7];
        for (n, &a) in amps.iter().enumerate() {
            let at = 24_000 * (n + 1);
            for (i, v) in wave[at..at + 4_000].iter_mut().enumerate() {
                *v = a * (-(i as f32) / 800.0).exp();
            }
        }
        super::chop::Chop::new(
            "break.wav".into(),
            std::sync::Arc::new(engine::Sample::mono(wave)),
            48_000,
        )
    }

    /// 04d "Break by hits": Ctrl+Y lands the cuts on the starts of the loud hits and the
    /// status says how many.
    #[test]
    fn break_by_hits() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        // 0.55 is 4.3 dB below 0.9: both pass −6 dB; 0.3 is −9.5, 0.15 is −15.6
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        chop_key(&mut app, KeyCode::Char('h'), M::NONE);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.markers, [24_000, 48_000], "the two loudest hits at −6 dB");
        assert_eq!(c.threshold_db, -6.0);
        assert_eq!(last_notice(&app), Some("by hits at -6 dB: 3 slices"));
    }

    /// 68 chopping "By hits says what it replaced": 16 hand-made slices, Ctrl+Y says how
    /// many slices there are now and were, and Ctrl+Z brings the 16 back.
    #[test]
    fn by_hits_says_what_it_replaced() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        let mut c = chop_hits(&[0.9, 0.55, 0.3, 0.15]);
        for i in 1..16 {
            c.cut(i * 20_000 + 1_000);
        }
        let hand = c.markers.clone();
        assert_eq!(c.slices().len(), 16);
        app.chop = Some(c);
        chop_key(&mut app, KeyCode::Char('h'), M::NONE);
        assert_eq!(
            last_notice(&app),
            Some("by hits at -6 dB: 3 slices (were 16) · Ctrl+Z brings them back")
        );
        chop_key(&mut app, KeyCode::Char('z'), M::CONTROL);
        assert_eq!(app.chop.as_ref().unwrap().markers, hand, "the 16 are back");
    }

    /// 04d "More presses, more cuts": every press steps the threshold lower so quieter
    /// hits get their cuts; Ctrl+Z brings back the cuts of the press before; after −30
    /// the next press starts over at −6.
    #[test]
    fn more_presses_more_cuts() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        let h = |app: &mut super::App| chop_key(app, KeyCode::Char('h'), M::NONE);
        let state = |app: &super::App| {
            let c = app.chop.as_ref().unwrap();
            (c.markers.clone(), last_notice(app).map(str::to_owned))
        };
        h(&mut app);
        assert_eq!(state(&app).0, [24_000, 48_000]);
        h(&mut app);
        let (markers, flash) = state(&app);
        assert_eq!(
            markers,
            [24_000, 48_000, 72_000],
            "−12 dB: the −9.5 dB hit joins"
        );
        assert_eq!(
            flash.as_deref(),
            Some("by hits at -12 dB: 4 slices (were 3) · Ctrl+Z brings them back")
        );
        h(&mut app);
        let (markers, flash) = state(&app);
        assert_eq!(
            markers,
            [24_000, 48_000, 72_000, 96_000],
            "−18 dB: all four"
        );
        assert_eq!(
            flash.as_deref(),
            Some("by hits at -18 dB: 5 slices (were 4) · Ctrl+Z brings them back")
        );
        chop_key(&mut app, KeyCode::Char('z'), M::CONTROL);
        assert_eq!(
            state(&app).0,
            [24_000, 48_000, 72_000],
            "Ctrl+Z back a step"
        );
        h(&mut app);
        assert_eq!(
            state(&app).0,
            [24_000, 48_000, 72_000, 96_000],
            "−18 dB again, all four"
        );
        h(&mut app);
        assert_eq!(
            state(&app).0,
            [24_000, 48_000, 72_000, 96_000],
            "−24 dB holds all"
        );
        h(&mut app);
        assert_eq!(
            state(&app).0,
            [24_000, 48_000, 72_000, 96_000],
            "−30 dB holds all"
        );
        h(&mut app);
        let (markers, flash) = state(&app);
        assert_eq!(
            markers,
            [24_000, 48_000],
            "after −30 the step starts over at −6"
        );
        assert_eq!(
            flash.as_deref(),
            Some("by hits at -6 dB: 3 slices (were 5) · Ctrl+Z brings them back")
        );
    }

    /// 04d "Undo brings the threshold back": Ctrl+Z returns the threshold with the cuts,
    /// so the next press continues from the step before, with the same cuts again.
    #[test]
    fn undo_returns_the_threshold_with_the_cuts() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        let h = |app: &mut super::App| chop_key(app, KeyCode::Char('h'), M::NONE);
        h(&mut app);
        h(&mut app);
        let (markers, db) = {
            let c = app.chop.as_ref().unwrap();
            (c.markers.clone(), c.threshold_db)
        };
        assert_eq!(markers, [24_000, 48_000, 72_000], "−12 dB: three cuts");
        assert_eq!(db, -12.0);
        chop_key(&mut app, KeyCode::Char('z'), M::CONTROL);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.markers, [24_000, 48_000], "the cuts of the first press");
        assert_eq!(c.threshold_db, -6.0, "the threshold came back with them");
        h(&mut app);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(
            c.markers,
            [24_000, 48_000, 72_000],
            "the third press steps to −12 again, the cuts of the second"
        );
        assert_eq!(c.threshold_db, -12.0);
    }

    /// 04d "The threshold on a knob": k11 adds and removes cuts at once as it turns, with
    /// the soft takeover, and a whole turn is one undo step.
    #[test]
    fn the_threshold_on_a_knob() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let mut app = test_app();
        // the scenario is about the catch: the response pickup (58)
        app.takeover.response = input::midi::Response::Pickup;
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.55, 0.3, 0.15]));
        chop_key(&mut app, KeyCode::Char('h'), KeyModifiers::NONE);
        assert_eq!(app.chop.as_ref().unwrap().markers, [24_000, 48_000]);
        // the knob sits far from the −6 dB (0.15 of the way): it waits, naming the way
        xsession(&mut app, knob(K11, 127));
        assert_eq!(
            app.chop.as_ref().unwrap().markers,
            [24_000, 48_000],
            "no jump"
        );
        assert_eq!(last_notice(&app), Some("turn k11 down ↓"));
        // down through the current value: the knob picks up at 0 dB, only the loudest keeps a cut
        xsession(&mut app, knob(K11, 0));
        assert_eq!(app.chop.as_ref().unwrap().markers, [24_000]);
        assert_eq!(last_notice(&app), Some("1 cut at 0 dB"));
        // up to the quiet end: every hit the detector found gets its cut
        xsession(&mut app, knob(K11, 127));
        let (markers, db) = (
            app.chop.as_ref().unwrap().markers.clone(),
            app.chop.as_ref().unwrap().threshold_db,
        );
        assert_eq!(markers, [24_000, 48_000, 72_000, 96_000]);
        assert_eq!(db, -40.0);
        assert_eq!(last_notice(&app), Some("4 cuts at -40 dB"));
        // mid: the −15.6 dB hit is out, the −9.5 dB one stays
        xsession(&mut app, knob(K11, 32));
        assert_eq!(app.chop.as_ref().unwrap().markers, [24_000, 48_000, 72_000]);
        assert_eq!(last_notice(&app), Some("3 cuts at -10 dB"));
        // the whole turn so far is one undo step: back to the cuts Ctrl+Y made
        chop_key(&mut app, KeyCode::Char('z'), KeyModifiers::CONTROL);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(
            c.markers,
            [24_000, 48_000],
            "one undo step: the cuts Ctrl+Y made"
        );
        assert_eq!(
            c.threshold_db, -6.0,
            "the threshold came back with the cuts of Ctrl+Y"
        );
        assert!(
            (c.threshold_pos() - 6.0 / 40.0).abs() < 0.01,
            "the knob position and the threshold stay in step"
        );
    }

    /// 04d "Cut slightly off the hit": playing with the hits mode on, a cut 40 ms after
    /// a hit lands exactly on it and the status says so; with the mode off the cut stays
    /// where pressed.
    #[test]
    fn cut_slightly_off_the_hit() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_hits(&[0.9, 0.3]));
        app.fake_playhead = Some(48_000 + 1_920); // 40 ms after the hit at 48_000
        app.apply(input::Action::ChopSnapToggle);
        app.apply(input::Action::ChopCut);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.markers, [48_000], "the marker snapped to the hit");
        assert_eq!(last_notice(&app), Some("marker snapped to hit"));
        // the hits mode off: the cut stays where the sound is
        app.apply(input::Action::ChopSnapToggle);
        app.fake_playhead = Some(48_000 + 12_000);
        app.notices = Notices::new(Instant::now());
        app.apply(input::Action::ChopCut);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.markers, [48_000, 60_000], "where pressed, no snap");
        assert_eq!(
            last_notice(&app),
            None,
            "no snap status with the hits mode off"
        );
    }

    /// 04d for the owner: a synthetic break chopped by three presses of Ctrl+Y, the
    /// slices played one by one. Every slice starts on its hit within a block, the
    /// silence before the first hit stays silent, no NaN, no clipping.
    #[test]
    fn render_chopped_by_hits() {
        use engine::offline::{Event, Scenario, render, write_wav};
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let amps = [0.9, 0.55, 0.3, 0.15];
        let mut wave = vec![0.0; 48_000 * 7];
        for (n, &a) in amps.iter().enumerate() {
            let at = 24_000 * (n + 1);
            for (i, v) in wave[at..at + 4_000].iter_mut().enumerate() {
                *v = a
                    * (-(i as f32) / 800.0).exp()
                    * (i as f32 * 110.0 / 48_000.0 * std::f32::consts::TAU).sin();
            }
        }
        let sound = std::sync::Arc::new(Sample::mono(wave));
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(super::chop::Chop::new(
            "break.wav".into(),
            sound.clone(),
            48_000,
        ));
        for _ in 0..3 {
            chop_key(&mut app, KeyCode::Char('h'), M::NONE); // −6, −12, −18 dB
        }
        let slices = app.chop.as_ref().unwrap().slices();
        let hits = [24_000, 48_000, 72_000, 96_000];
        // the head of silence plus one piece per hit
        assert_eq!(slices.len(), hits.len() + 1, "−18 dB keeps all four hits");
        let mut left = Vec::new();
        let mut right = Vec::new();
        for (i, s) in slices.iter().enumerate() {
            if i > 0 {
                assert!(
                    s.start.abs_diff(hits[i - 1]) <= 256,
                    "slice {} starts on its hit, {} frames off",
                    i + 1,
                    s.start as i64 - hits[i - 1] as i64
                );
            }
            let frames = s.end - s.start + 128;
            let events = vec![
                Event {
                    frame: 0,
                    cmd: Command::SetPadSound {
                        pad: 0,
                        sound: Some(sound.clone()),
                    },
                },
                Event {
                    frame: 0,
                    cmd: Command::SetPadRegion { pad: 0, region: *s },
                },
                Event {
                    frame: 128,
                    cmd: Command::PadPress {
                        pad: 0,
                        velocity: 1.0,
                    },
                },
            ];
            let sc = Scenario {
                sample_rate: 48_000,
                frames,
                events,
            };
            let (l, r) = render(&mut engine::Engine::new(48_000), &sc, 128);
            assert!(engine::checks::first_non_finite(&l).is_none());
            assert!(
                engine::checks::peak(&l) < 1.0,
                "no clipping in slice {}",
                i + 1
            );
            if i == 0 {
                // the bleed past the region end is the next hit's onset fading, the head
                // itself is quiet
                assert_eq!(
                    engine::checks::peak(&l[..hits[0]]),
                    0.0,
                    "silence before the first hit"
                );
            } else {
                assert!(
                    engine::checks::peak(&l[..480]) > 0.5 * amps[i - 1],
                    "slice {} starts on its hit, loud from the head",
                    i + 1
                );
            }
            left.extend(l);
            right.extend(r);
        }
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../renders/04d-chopped-by-hits.wav");
        write_wav(&out, 48_000, &left, &right).unwrap();
    }

    /// The chop screen key `code` with `m`, as the keymap resolves it.
    fn chop_key(
        app: &mut super::App,
        code: ratatui::crossterm::event::KeyCode,
        m: ratatui::crossterm::event::KeyModifiers,
    ) {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let k = ratatui::crossterm::event::KeyEvent::new(code, m);
        app.apply(keymap.resolve(input::Screen::Chop, &k).unwrap());
    }

    /// Recorded: Alt+Backspace and Ctrl+Backspace in slice 3 of 4. 21: "Delete the lit bound".
    #[test]
    fn delete_the_lit_bound() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(2)); // cuts at 24_000, 48_000, 72_000
        app.chop.as_mut().unwrap().set_cursor(50_000);
        let cuts = |app: &super::App| app.chop.as_ref().unwrap().markers.clone();
        chop_key(&mut app, KeyCode::Backspace, M::ALT);
        assert_eq!(cuts(&app), [24_000, 48_000], "the end of slice 3 is gone");
        chop_key(&mut app, KeyCode::Char('z'), M::CONTROL);
        assert_eq!(cuts(&app), [24_000, 48_000, 72_000], "undo brings it back");
        chop_key(&mut app, KeyCode::Backspace, M::CONTROL);
        assert_eq!(cuts(&app), [24_000, 72_000], "Ctrl: the start of slice 3");
        // the last slice ends at the region's edge: nothing to delete
        app.chop.as_mut().unwrap().set_cursor(90_000);
        chop_key(&mut app, KeyCode::Backspace, M::ALT);
        assert_eq!(cuts(&app), [24_000, 72_000]);
    }

    /// A jump onto the first cut, then Backspace. 21: "The cursor on a cut".
    #[test]
    fn the_cursor_on_a_cut() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(2));
        app.chop.as_mut().unwrap().set_cursor(0);
        app.apply(Action::CursorJump { right: true });
        assert_eq!(app.chop.as_ref().unwrap().cursor, 24_000);
        chop_key(&mut app, KeyCode::Backspace, M::NONE);
        assert_eq!(app.chop.as_ref().unwrap().markers, [48_000, 72_000]);
    }

    /// Recorded: Enter, then Esc on the pads screen. 21: "Enter opens, Esc closes".
    #[test]
    fn enter_opens_esc_closes() {
        let mut app = test_app();
        app.apply(input::Action::PadEdit);
        assert!(app.edit, "the edit mode of the selected pad opens");
        // a screen switch drops the mode
        app.apply(input::Action::ScreenFiles);
        assert!(!app.edit, "the mode does not survive the switch");
        // Enter again, Esc closes and the screen stays
        app.apply(input::Action::ScreenPads);
        app.apply(input::Action::PadEdit);
        app.apply(input::Action::Back);
        assert!(!app.edit, "Esc closes the mode");
        assert_eq!(app.screen, input::Screen::Pads, "the screen stays");
        // the help line names the way out while the mode is open
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let st = super::HelpState {
            hand: None,
            preview_on: false,
            assign: false,
            snap: super::chop::Snap::Off,
            tab: 0,
            param_name: "volume",
            page: "TONE",
            play: "to end",
            looping: false,
            as_pad: false,
            marking: false,
            scope: String::new(),
            edit: true,
        };
        let items = line(&super::help_groups(&keymap, input::Screen::Pads, &st));
        assert!(
            items.contains(&("Enter Esc".into(), "close".into())),
            "{items:?}"
        );
        // and the page with the keys that walk it, nothing of the screen (21 "The edit line")
        assert!(
            items.contains(&("Tab Shift+Tab".into(), "TONE".into())),
            "{items:?}"
        );
        assert!(
            items.contains(&("J L".into(), "volume".into())),
            "{items:?}"
        );
        assert!(
            items.contains(&("pad".into(), "listen".into())),
            "{items:?}"
        );
        // 54: the pages on Tab, one parameter on I K
        assert!(
            items.contains(&("I K".into(), "parameter".into())),
            "{items:?}"
        );
        // 76: the clear key resets the pad there
        assert!(
            items.contains(&("Bksp".into(), "reset".into())),
            "{items:?}"
        );
        assert_eq!(items.len(), 6, "only the edit keys: {items:?}");
        assert!(
            !items.iter().any(|(_, t)| t == "select"),
            "the arrows do not select in the edit mode: {items:?}"
        );
    }

    /// Recorded: a pad hit with the edit mode open. 21: "The music plays on".
    #[test]
    fn the_music_plays_on() {
        let mut app = test_app();
        app.apply(input::Action::PadEdit);
        app.apply(input::Action::PadPress {
            pad: 5,
            velocity: 0.5,
        });
        assert_eq!(
            app.pads.selected, 5,
            "a pad hit selects its pad in the edit mode"
        );
        assert!(app.edit, "the mode stays open: the hit does not close it");
    }

    /// Recorded: G on a pad, then Enter. pad-playback 21: "Mode from the keyboard".
    #[test]
    fn mode_from_the_keyboard() {
        let mut app = test_app();
        let was = app.pads.gate[app.pads.selected_abs()];
        app.apply(input::Action::PadModeNext);
        assert_eq!(
            app.pads.gate[app.pads.selected_abs()],
            !was,
            "G toggles the mode in place"
        );
        // Enter opens the edit, it does not toggle the mode
        app.apply(input::Action::PadEdit);
        assert!(app.edit);
        assert_eq!(
            app.pads.gate[app.pads.selected_abs()],
            !was,
            "Enter does not touch the mode"
        );
    }

    /// Recorded: Enter on the chop screen in the listen mode. 21: "Enter opens on the
    /// selected slice" (the mode state; the BOUNDS naming comes with task 2).
    #[test]
    fn enter_opens_on_the_selected_slice() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        let mut c = chop_with_parts(1);
        c.select_slice(1);
        app.chop = Some(c);
        app.apply(input::Action::ChopEdit);
        assert!(app.edit, "Enter opens the slice edit in the listen mode");
        // the help line names the BOUNDS page and the picked bound
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let items = line(&super::help_groups(
            &keymap,
            input::Screen::Chop,
            &app.help_state(),
        ));
        assert!(
            items.contains(&("I K".into(), "BOUNDS".into())),
            "{items:?}"
        );
        assert!(items.contains(&("J L".into(), "start".into())), "{items:?}");
        app.apply(input::Action::Back);
        assert!(!app.edit, "Esc closes the slice edit");
    }

    /// Recorded: P on the chop screen in the listen mode. 21: "P starts the assign".
    #[test]
    fn p_starts_the_assign() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(1));
        app.apply(input::Action::ChopAssign);
        assert!(
            matches!(&app.plan, Some(super::chop::Plan::ByHand(_))),
            "by hand from slice 1"
        );
        assert_eq!(app.chop.as_ref().unwrap().selected, 0, "slice 1 first");
        // in the listen mode Enter is the edit now; only P and the device commit start
        // the assign
        app.apply(input::Action::Back);
        assert!(app.plan.is_none(), "Esc leaves the assign");
        app.apply(input::Action::ChopEdit);
        assert!(
            app.edit && app.plan.is_none(),
            "Enter opens the edit instead"
        );
    }

    /// Recorded: P, two pad hits, Enter. 21: "Commit is untouched".
    #[test]
    fn commit_is_untouched() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(1));
        app.apply(input::Action::ChopAssign);
        app.apply(input::Action::PadPress {
            pad: 2,
            velocity: 1.0,
        });
        app.apply(input::Action::PadPress {
            pad: 3,
            velocity: 1.0,
        });
        assert!(app.plan.is_some(), "two pads planned");
        // Enter applies the plan: the pads were empty, no question comes first
        app.apply(input::Action::ChopEdit);
        assert!(app.plan.is_none() && app.ask.is_none(), "the plan applied");
        assert!(app.pads.file[2].is_some() && app.pads.file[3].is_some());
        // over an occupied pad the question comes first; the same key applies after it
        app.screen = input::Screen::Chop;
        app.apply(input::Action::ChopAssign);
        app.apply(input::Action::PadPress {
            pad: 2,
            velocity: 1.0,
        });
        app.apply(input::Action::ChopEdit);
        assert!(app.ask.is_some(), "the replace question");
        app.apply(input::Action::ChopEdit);
        assert!(
            app.ask.is_none() && app.plan.is_none(),
            "the second Enter applies"
        );
    }

    /// Recorded: P, a pad hit, then m, Alt+←, the arrows and Alt+arrows. 21: "No stray
    /// cut during the assign".
    #[test]
    fn no_stray_cut_during_the_assign() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(1));
        app.apply(input::Action::ChopAssign);
        app.apply(input::Action::PadPress {
            pad: 2,
            velocity: 1.0,
        });
        let c = app.chop.as_ref().unwrap();
        let (cursor, markers, view, range, plan) = (
            c.cursor,
            c.markers.clone(),
            c.view,
            c.range,
            app.plan.clone(),
        );
        app.notices = Notices::new(Instant::now());
        for a in [
            input::Action::ChopCut,
            input::Action::MarkerNudge {
                right: false,
                fine: false,
            },
            // J L are the slice pick in the assign (35), not a stray key
            input::Action::ChopZoom { zoom_in: true },
            input::Action::ChopEqualParts,
            input::Action::ChopRegionStart,
            input::Action::ChopPlay,
            input::Action::ChopPlayMode,
        ] {
            app.apply(a);
        }
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.cursor, cursor, "the cursor did not move");
        assert_eq!(c.markers, markers, "nothing was cut or moved");
        assert_eq!(c.view, view, "the view did not zoom");
        assert_eq!(c.range, range, "the region stayed");
        assert_eq!(app.plan, plan, "the plan stayed as it was");
        assert_eq!(
            app.chop.as_ref().unwrap().play,
            super::chop::PlayMode::ToEnd,
            "the play mode stayed"
        );
        assert_eq!(flash(&app), "assigning: Enter applies, Esc leaves");
        // Esc leaves the assign
        app.apply(input::Action::Back);
        assert!(app.plan.is_none(), "Esc leaves the assign");
    }

    /// Recorded: Enter on a slice, then P and the device commit. 21: "No assign from the
    /// edit".
    #[test]
    fn no_assign_from_the_edit() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(1));
        app.apply(input::Action::ChopEdit);
        app.apply(input::Action::ChopAssign);
        app.apply(input::Action::ChopCommit);
        assert!(app.plan.is_none(), "the assign did not start");
        assert!(app.edit, "the slice edit stays open");
        // Esc first, then the device commit starts it as before
        app.apply(input::Action::Back);
        app.apply(input::Action::ChopCommit);
        assert!(app.plan.is_some());
    }

    /// Recorded: Enter on a pad, F1, Esc. 21: "Esc closes the help first".
    #[test]
    fn esc_closes_the_help_first() {
        let mut app = test_app();
        app.apply(input::Action::PadEdit);
        app.apply(input::Action::HelpToggle);
        app.apply(input::Action::Back);
        assert!(!app.help_open, "the help closes");
        assert!(app.edit, "the edit mode stays open");
    }

    /// Recorded: L in the listen mode. 21: "L steps the parts".
    #[test]
    fn l_steps_the_parts() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(0));
        app.apply(input::Action::ChopEqualParts);
        assert_eq!(app.chop.as_ref().unwrap().slices().len(), 2);
        // K in the listen mode does not cut: it picks the bound (task 3)
        app.apply(input::Action::ParamNext);
        assert_eq!(app.chop.as_ref().unwrap().slices().len(), 2);
        // in the assign mode Tab switches in order / by hand
        app.apply(input::Action::ChopAssign);
        app.apply(input::Action::ParamNext);
        assert!(matches!(app.plan, Some(super::chop::Plan::InOrder(_))));
    }

    /// chopping 21 "Coarse move": a held bound key moves like the cursor, by a share of the
    /// view that grows with the repeats.
    /// Scenario "Fine adjustment" (chopping).
    #[test]
    fn a_held_bound_speeds_up() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        let mut c = chop_with_parts(1);
        c.select_slice(1);
        app.chop = Some(c);
        let start = |app: &super::App| app.chop.as_ref().unwrap().slices()[1].start;
        let mut moved = Vec::new();
        for fine in [true, false, false, false, false, false, false, false] {
            let was = start(&app);
            app.apply(input::Action::SliceBound {
                end: false,
                right: true,
                fine,
            });
            moved.push(start(&app) - was);
        }
        let view = 96_000;
        assert_eq!(moved[0], view / 128, "a tap: about a cell");
        assert!(
            moved.windows(2).all(|w| w[1] >= w[0]),
            "it only grows: {moved:?}"
        );
        assert_eq!(moved[7], view / 32, "the top step after the ramp");
    }

    /// Scenario "Zoom between bound edits with the modifier held".
    #[test]
    fn bound_zoom_keeps_held_modifier_bounds_cursor_and_slice() {
        use ratatui::crossterm::event::{
            KeyCode, KeyEvent, KeyEventKind as Kind, KeyModifiers as M,
        };
        let (map, warnings) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        for (modifier, end, indicator) in [
            (M::CONTROL, false, input::Modifier::Ctrl),
            (M::ALT, true, input::Modifier::Alt),
        ] {
            for editing in [false, true] {
                for [zoom_in, zoom_out, right] in [['i', 'k', 'l'], ['ш', 'л', 'д']] {
                    let mut app = test_app();
                    app.screen = input::Screen::Chop;
                    let mut chop = chop_with_parts(2);
                    chop.select_slice(1);
                    app.chop = Some(chop);
                    app.edit = editing;
                    let mut held = input::HeldPads::default();
                    let mut mods = input::HeldModifiers::default();
                    let mut keys = input::HeldKeys::default();
                    let modifier_code = KeyCode::Modifier(if end {
                        ratatui::crossterm::event::ModifierKeyCode::LeftAlt
                    } else {
                        ratatui::crossterm::event::ModifierKeyCode::LeftControl
                    });
                    let down = KeyEvent::new(modifier_code, modifier);
                    assert!(
                        !app.keyboard_event(&map, &down, true, &mut keys, &mut held, &mut mods)
                    );
                    let mut send = |app: &mut super::App, letter, kind| {
                        let key = KeyEvent::new_with_kind(KeyCode::Char(letter), modifier, kind);
                        assert!(
                            !app.keyboard_event(&map, &key, true, &mut keys, &mut held, &mut mods)
                        );
                        assert_eq!(mods.get(), Some(indicator));
                    };
                    let bounds = |app: &super::App| app.chop.as_ref().unwrap().slices()[1];
                    let width = |app: &super::App| {
                        let v = app.chop.as_ref().unwrap().view;
                        v.end - v.start
                    };
                    let initial = bounds(&app);
                    send(&mut app, right, Kind::Press);
                    send(&mut app, right, Kind::Release);
                    let moved = bounds(&app);
                    if end {
                        assert!(moved.end > initial.end);
                        assert_eq!(moved.start, initial.start);
                    } else {
                        assert!(moved.start > initial.start);
                        assert_eq!(moved.end, initial.end);
                    }
                    let snapshot = |app: &super::App| {
                        let c = app.chop.as_ref().unwrap();
                        (c.saved(), c.slices(), c.cursor, c.selected, app.slice_end)
                    };
                    let before = snapshot(&app);
                    let whole = width(&app);
                    send(&mut app, zoom_in, Kind::Press);
                    let once = width(&app);
                    assert!(once < whole);
                    send(&mut app, zoom_in, Kind::Repeat);
                    let twice = width(&app);
                    assert!(twice < once);
                    send(&mut app, zoom_in, Kind::Release);
                    assert_eq!(width(&app), twice);
                    send(&mut app, zoom_out, Kind::Press);
                    assert_eq!(width(&app), once);
                    send(&mut app, zoom_out, Kind::Release);
                    assert_eq!(width(&app), once);
                    assert_eq!(
                        snapshot(&app),
                        before,
                        "zoom must not edit bounds or move the cursor"
                    );
                    send(&mut app, right, Kind::Press);
                    send(&mut app, right, Kind::Release);
                    let after = bounds(&app);
                    if end {
                        assert!(after.end > moved.end);
                        assert_eq!(after.start, moved.start);
                    } else {
                        assert!(after.start > moved.start);
                        assert_eq!(after.end, moved.end);
                    }
                    assert_eq!(app.edit, editing);
                    let up = KeyEvent::new_with_kind(modifier_code, M::NONE, Kind::Release);
                    assert!(!app.keyboard_event(&map, &up, true, &mut keys, &mut held, &mut mods));
                    assert_eq!(mods.get(), None);
                }
            }
        }
    }

    /// Legacy terminals can send Tab instead of a distinct Ctrl+I event.
    #[test]
    fn bound_zoom_legacy_alt_keeps_tab_and_slice_edit_roles() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        let (map, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        for editing in [false, true] {
            for [zoom_in, zoom_out] in [['i', 'k'], ['ш', 'л']] {
                let mut app = test_app();
                app.screen = input::Screen::Chop;
                app.chop = Some(chop_with_parts(2));
                app.edit = editing;
                let mut held = input::HeldPads::default();
                let mut mods = input::HeldModifiers::default();
                let mut keys = input::HeldKeys::default();
                let width = |app: &super::App| {
                    let v = app.chop.as_ref().unwrap().view;
                    v.end - v.start
                };
                let whole = width(&app);
                let tab = KeyEvent::new(KeyCode::Tab, M::NONE);
                assert!(!app.keyboard_event(&map, &tab, false, &mut keys, &mut held, &mut mods));
                assert_eq!(width(&app), whole);
                let saved = app.chop.as_ref().unwrap().saved();
                let cursor = app.chop.as_ref().unwrap().cursor;
                for (letter, expected) in [(zoom_in, whole / 2), (zoom_out, whole)] {
                    let key = KeyEvent::new(KeyCode::Char(letter), M::ALT);
                    assert!(
                        !app.keyboard_event(&map, &key, false, &mut keys, &mut held, &mut mods)
                    );
                    assert_eq!(width(&app), expected);
                    assert_eq!(app.chop.as_ref().unwrap().saved(), saved);
                    assert_eq!(app.chop.as_ref().unwrap().cursor, cursor);
                    assert_eq!(app.edit, editing);
                }
            }
        }
    }

    #[test]
    fn bound_zoom_respects_assignment_naming_and_learning() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        let (map, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        for blocker in ["assign", "name", "learn"] {
            for modifier in [M::CONTROL, M::ALT] {
                let mut app = test_app();
                app.screen = input::Screen::Chop;
                app.chop = Some(chop_with_parts(2));
                match blocker {
                    "assign" => app.plan = Some(super::chop::Plan::ByHand(vec![])),
                    "name" => app.pad_naming = Some((0, String::new())),
                    "learn" => app.screen_learn = Some(app.learn_open()),
                    _ => unreachable!(),
                }
                let before = app.chop.as_ref().unwrap().view;
                let mut held = input::HeldPads::default();
                let mut mods = input::HeldModifiers::default();
                let mut keys = input::HeldKeys::default();
                for letter in ['i', 'k'] {
                    let key = KeyEvent::new(KeyCode::Char(letter), modifier);
                    assert!(!app.keyboard_event(&map, &key, true, &mut keys, &mut held, &mut mods));
                    assert_eq!(app.chop.as_ref().unwrap().view, before, "{blocker}");
                }
            }
        }
    }

    /// Scenario "Held bound modifiers show working zoom keys".
    #[test]
    fn bound_zoom_help_names_only_effective_modified_bindings() {
        let (mut map, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let help = |map: &input::Keymap, modifier| {
            super::modifier_groups(map, input::Screen::Chop, modifier, false)
                .into_iter()
                .flat_map(|g| g.items)
                .collect::<Vec<_>>()
        };
        for modifier in [input::Modifier::Ctrl, input::Modifier::Alt] {
            let rows = help(&map, modifier);
            assert!(rows.contains(&("I".into(), "zoom in".into())));
            assert!(rows.contains(&("K".into(), "zoom out".into())));
            assert!(
                rows.iter()
                    .any(|(keys, text)| keys == "J L" && text.ends_with("bound"))
            );
        }
        for kitty in [false, true] {
            let rows = super::full_help(&map, input::Screen::Chop, kitty);
            for chord in ["Ctrl+I", "Ctrl+K", "Alt+I", "Alt+K"] {
                assert!(
                    rows.iter()
                        .any(|(keys, text)| keys.split_whitespace().any(|k| k == chord)
                            && text.starts_with("chop: zoom")),
                    "{rows:?}"
                );
            }
        }
        assert!(
            map.overlay("[screens.chop]\n'Ctrl+I' = 'none'\n'Alt+K' = 'chop.cut'\n")
                .unwrap()
                .is_empty()
        );
        assert!(
            !help(&map, input::Modifier::Ctrl)
                .iter()
                .any(|(_, text)| text == "zoom in")
        );
        assert!(
            help(&map, input::Modifier::Alt)
                .contains(&("K".into(), "chop: cut at the cursor".into()))
        );
        assert!(
            !help(&map, input::Modifier::Alt)
                .iter()
                .any(|(_, text)| text == "zoom out")
        );
        for kitty in [false, true] {
            let rows = super::full_help(&map, input::Screen::Chop, kitty);
            assert!(
                !rows
                    .iter()
                    .any(|(keys, _)| keys.split_whitespace().any(|k| k == "Ctrl+I"))
            );
            assert!(
                rows.iter()
                    .any(|(keys, text)| keys.split_whitespace().any(|k| k == "Alt+K")
                        && text.contains("cut"))
            );
        }
    }

    /// Recorded: I I K on the chop screen. 21: "Zoom on I and K".
    #[test]
    fn zoom_on_i_and_k() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let key = |c| {
            ratatui::crossterm::event::KeyEvent::new(
                ratatui::crossterm::event::KeyCode::Char(c),
                ratatui::crossterm::event::KeyModifiers::NONE,
            )
        };
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(1));
        let (whole, cursor) = {
            let c = app.chop.as_ref().unwrap();
            (c.view.end - c.view.start, c.cursor)
        };
        let width = |app: &super::App| {
            let v = app.chop.as_ref().unwrap().view;
            v.end - v.start
        };
        for c in ['i', 'i'] {
            app.apply(keymap.resolve(input::Screen::Chop, &key(c)).unwrap());
        }
        let twice = width(&app);
        assert!(twice * 3 < whole, "twice in: {twice} of {whole}");
        app.apply(keymap.resolve(input::Screen::Chop, &key('k')).unwrap());
        assert!(width(&app) > twice, "once out");
        assert_eq!(
            app.chop.as_ref().unwrap().cursor,
            cursor,
            "the cursor stays"
        );
    }

    /// Recorded: Ctrl+J, then Alt+L on slice 2. 21: "Ctrl and Alt name the bound".
    #[test]
    fn ctrl_and_alt_name_the_bound() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        let mut c = chop_with_parts(2);
        c.select_slice(1);
        app.chop = Some(c);
        let bounds = |app: &super::App| {
            let s = app.chop.as_ref().unwrap().slices()[1];
            (s.start, s.end)
        };
        let (start, end) = bounds(&app);
        let ctrl_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);
        app.apply(keymap.resolve(input::Screen::Chop, &ctrl_j).unwrap());
        assert!(bounds(&app).0 < start, "the start moved left");
        assert_eq!(bounds(&app).1, end, "the end stayed");
        let alt_l = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::ALT);
        app.apply(keymap.resolve(input::Screen::Chop, &alt_l).unwrap());
        assert!(bounds(&app).1 > end, "the end moved right");
        assert!(app.slice_end, "the moved bound is the picked one");
    }

    /// Recorded: , at 0.5 s, . at 1.5 s, Ctrl+, on the chop screen. 21: "Region on comma
    /// and dot".
    #[test]
    fn region_on_comma_and_dot() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(0));
        let press = |app: &mut super::App, c, m| {
            let a = keymap
                .resolve(input::Screen::Chop, &KeyEvent::new(KeyCode::Char(c), m))
                .unwrap();
            app.apply(a);
        };
        app.chop.as_mut().unwrap().set_cursor(24_000);
        press(&mut app, ',', KeyModifiers::NONE);
        app.chop.as_mut().unwrap().set_cursor(72_000);
        press(&mut app, '.', KeyModifiers::NONE);
        let r = app.chop.as_ref().unwrap().range;
        assert_eq!((r.start, r.end), (24_000, 72_000));
        press(&mut app, ',', KeyModifiers::CONTROL);
        let r = app.chop.as_ref().unwrap().range;
        assert_eq!((r.start, r.end), (0, 96_000), "the whole sample again");
    }

    /// 21: "The envelope page fills three knobs".
    #[test]
    fn the_envelope_page_fills_three_knobs() {
        use super::pads::Param;
        let mut app = test_app();
        for want in [Param::Attack, Param::Decay, Param::Release] {
            while app.pads.param != want {
                app.apply(input::Action::ParamNext);
            }
            assert_eq!(
                app.pads.page(),
                ("ENV", &[Param::Attack, Param::Decay, Param::Release][..])
            );
        }
    }

    /// Scenario "The page follows the picked parameter" (pad-playback): recorded `Enter`, then
    /// `K` until decay; the page is ENV and its three knobs are attack, decay, release.
    #[test]
    fn the_page_follows_the_picked_parameter() {
        use super::pads::Param;
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = test_app();
        let press = |app: &mut super::App, code| {
            let k = KeyEvent::new(code, KeyModifiers::NONE);
            let a = keymap.resolve_in(app.screen, app.edit, &k).unwrap();
            app.apply(a);
        };
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.pads.page().0, "TONE");
        for _ in 0..20 {
            if app.pads.param == Param::Decay {
                break;
            }
            press(&mut app, KeyCode::Char('k'));
        }
        assert_eq!(app.pads.param, Param::Decay, "K reaches the decay");
        assert_eq!(
            app.pads.page(),
            ("ENV", &[Param::Attack, Param::Decay, Param::Release][..])
        );
    }

    /// 54 "Up down walks the parameters" through every page; pad-playback 21: "The page
    /// follows the picked parameter".
    #[test]
    fn tab_moves_to_the_tone_page() {
        use super::pads::Param;
        let mut app = test_app();
        app.apply(input::Action::PadEdit);
        let mut pages = Vec::new();
        for _ in 0..15 {
            app.apply(input::Action::ParamNext);
            pages.push((app.pads.param, app.pads.page().0));
        }
        assert_eq!(
            pages,
            [
                (Param::Pitch, "TONE"),
                (Param::Group, "TONE"),
                (Param::Attack, "ENV"),
                (Param::Decay, "ENV"),
                (Param::Release, "ENV"),
                (Param::Filter, "FILTER"),
                (Param::Cutoff, "FILTER"),
                (Param::Reso, "FILTER"),
                (Param::Source, "SOURCE"),
                (Param::Start, "SOURCE"),
                (Param::End, "SOURCE"),
                (Param::Drive, "FX"),
                (Param::Send1, "FX"),
                (Param::Send2, "FX"),
                (Param::Volume, "TONE"),
            ]
        );
        assert_eq!(
            app.pads.page().1,
            [Param::Volume, Param::Pitch, Param::Group]
        );
    }

    /// 44 "The filter page": K past SOURCE opens FILTER; knobs 1–3 turn the kind, the
    /// cutoff and the resonance of the pad, and the pad sends its filter to the engine.
    #[test]
    fn the_filter_page() {
        use super::pads::Param;
        use engine::filter::FilterKind;
        let turn = |app: &mut super::App, k: u8, value: f32| {
            app.midi_event(super::midi::Event::Param {
                port: "X-Session".into(),
                control: format!("k{}", k + 1),
                param: input::midi::Param::Slot(k),
                value,
                takeover: false,
            })
        };
        let mut app = test_app();
        app.apply(input::Action::PadEdit);
        // past the end of ENV (48)
        while app.pads.param != Param::Release {
            app.apply(input::Action::ParamNext);
        }
        app.apply(input::Action::ParamNext);
        assert_eq!(
            app.pads.page(),
            ("FILTER", &[Param::Filter, Param::Cutoff, Param::Reso][..])
        );
        app.sent.clear();
        turn(&mut app, 0, 0.3); // the second quarter: low-pass
        turn(&mut app, 1, 0.5); // halfway along the octaves: about 632 Hz
        turn(&mut app, 2, 0.4);
        assert_eq!(app.pads.filter[0], FilterKind::LowPass);
        assert!(
            (app.pads.cutoff_hz[0] - 632.5).abs() < 1.0,
            "{}",
            app.pads.cutoff_hz[0]
        );
        assert_eq!(app.pads.reso[0], 40);
        let last = app.sent.iter().rev().find_map(|c| match c {
            Command::SetPadFilter { pad: 0, settings } => Some(*settings),
            _ => None,
        });
        assert_eq!(
            last,
            Some(app.pads.filter_settings(0)),
            "the engine hears it"
        );
    }

    /// The pad edit opened on the FILTER page of pad 1, with `kind`, `hz` and `reso` %.
    fn filter_page(kind: engine::filter::FilterKind, hz: f32, reso: u8) -> super::App {
        let mut app = test_app();
        app.pads.filter[0] = kind;
        app.pads.cutoff_hz[0] = hz;
        app.pads.reso[0] = reso;
        app.apply(input::Action::PadEdit);
        while app.pads.param != super::pads::Param::Filter {
            app.apply(input::Action::ParamNext);
        }
        app
    }

    /// The curve the strip shows now.
    fn curve(app: &mut super::App) -> tui::FilterCurve {
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        app.mark_pads(v)
            .curve
            .expect("the FILTER page shows a curve")
    }

    /// Where the curve first falls 3 dB below its start, as a fraction of its width.
    fn drop_at(c: &tui::FilterCurve) -> f32 {
        let i = c.levels.iter().position(|&l| l < 0.708).unwrap();
        i as f32 / c.levels.len() as f32
    }

    /// 44 "The cutoff moves on the curve": an octave down moves the drop left by an
    /// octave's share of the width, and the caption follows.
    #[test]
    fn the_cutoff_moves_on_the_curve() {
        use engine::filter::FilterKind;
        let mut app = filter_page(FilterKind::LowPass, 1_200.0, 0);
        let before = curve(&mut app);
        assert_eq!(before.caption, "low-pass · 1.2 kHz · reso 0 %");
        while app.pads.param != super::pads::Param::Cutoff {
            app.apply(input::Action::ParamNext);
        }
        app.apply(input::Action::EditValue {
            up: false,
            fine: false,
        });
        let after = curve(&mut app);
        assert_eq!(after.caption, "low-pass · 600 Hz · reso 0 %");
        // 20 Hz–20 kHz is ten octaves, and half an octave of room at both ends
        let octave = 1.0 / ((1_000f32).log2() + 1.0);
        let moved = drop_at(&before) - drop_at(&after);
        assert!(
            (moved - octave).abs() < 0.01,
            "moved {moved}, an octave is {octave}"
        );
    }

    /// 44 "The resonance is a hump": at 60 % the curve rises above full level at the cutoff.
    #[test]
    fn the_resonance_is_a_hump() {
        use engine::filter::FilterKind;
        let flat = curve(&mut filter_page(FilterKind::LowPass, 1_200.0, 0));
        let hump = curve(&mut filter_page(FilterKind::LowPass, 1_200.0, 60));
        let top = |c: &tui::FilterCurve| c.levels.iter().copied().fold(0.0, f32::max);
        assert!(
            top(&flat) <= 1.0 + 1e-3,
            "no hump without resonance: {}",
            top(&flat)
        );
        assert!(top(&hump) > 1.5, "a hump at 60 %: {}", top(&hump));
    }

    /// 44 "The hump stays on the screen": at both ends of the cutoff, at full resonance,
    /// the top of the hump lies inside the curve, not at its edge.
    #[test]
    fn the_hump_stays_on_the_screen() {
        use engine::filter::FilterKind;
        for hz in [20.0, 20_000.0] {
            let c = curve(&mut filter_page(FilterKind::LowPass, hz, 100));
            let top = (0..c.levels.len())
                .max_by(|&a, &b| c.levels[a].total_cmp(&c.levels[b]))
                .unwrap();
            let n = c.levels.len();
            assert!(
                top > n / 40 && top < n - n / 40,
                "{hz} Hz: the top at {top} of {n}"
            );
        }
    }

    /// 44 "Off is flat" and "Back to the wave".
    #[test]
    fn off_is_flat_and_tab_goes_back_to_the_wave() {
        use engine::filter::FilterKind;
        let mut app = filter_page(FilterKind::Off, 1_000.0, 0);
        let c = curve(&mut app);
        assert_eq!(c.caption, "filter off");
        assert!(c.levels.iter().all(|&l| l == 1.0));
        while app.pads.page().0 != "TONE" {
            app.apply(input::Action::ParamNext);
        }
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        assert!(
            app.mark_pads(v).curve.is_none(),
            "the wave and the envelope again"
        );
    }

    /// Recorded: M, hit Q and W, the cutoff page, J held on Q. 44 "Marked pads together".
    #[test]
    fn marked_pads_turn_the_cutoff_together() {
        use super::pads::Param;
        let mut app = test_app();
        app.pads.marked.extend([4, 5]); // Q W
        app.pads.selected = 4;
        app.apply(input::Action::PadEdit);
        while app.pads.param != Param::Cutoff {
            app.apply(input::Action::ParamNext);
        }
        let w = app.pads.cutoff_hz[5];
        // a held key: the coarse step, an octave
        app.apply(input::Action::EditValue {
            up: false,
            fine: false,
        });
        assert!((app.pads.cutoff_hz[4] - 500.0).abs() < 0.1);
        assert!((app.pads.cutoff_hz[5] - w / 2.0).abs() < 0.1, "W follows");
        // a tap: a semitone
        app.apply(input::Action::EditValue {
            up: true,
            fine: true,
        });
        assert!((app.pads.cutoff_hz[4] - 500.0 * 2f32.powf(1.0 / 12.0)).abs() < 0.1);
    }

    /// 56 "SOURCE last" (as 56 has it): K from the reso of FILTER picks the source; K past
    /// SOURCE picks the drive on FX; K past send 2 comes back to the volume.
    #[test]
    fn source_last() {
        use super::pads::Param;
        let mut app = test_app();
        app.apply(input::Action::PadEdit);
        while app.pads.param != Param::Reso {
            app.apply(input::Action::ParamNext);
        }
        app.apply(input::Action::ParamNext);
        assert_eq!(
            (app.pads.param, app.pads.page().0),
            (Param::Source, "SOURCE")
        );
        while app.pads.param != Param::End {
            app.apply(input::Action::ParamNext);
        }
        app.apply(input::Action::ParamNext);
        assert_eq!((app.pads.param, app.pads.page().0), (Param::Drive, "FX"));
        while app.pads.param != Param::Send2 {
            app.apply(input::Action::ParamNext);
        }
        app.apply(input::Action::ParamNext);
        assert_eq!(app.pads.param, Param::Volume);
        assert_eq!(
            super::pads::page_names(),
            ["TONE", "ENV", "FILTER", "SOURCE", "FX"]
        );
    }

    /// Recorded: Enter, ↓, ↑ on a pad. 21: "Up down walks the parameters".
    #[test]
    fn up_down_walks_the_parameters() {
        use super::pads::Param;
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let key = |code| {
            ratatui::crossterm::event::KeyEvent::new(
                code,
                ratatui::crossterm::event::KeyModifiers::NONE,
            )
        };
        let mut app = test_app();
        let selected = app.pads.selected;
        for (code, want) in [
            (ratatui::crossterm::event::KeyCode::Enter, Param::Volume),
            (ratatui::crossterm::event::KeyCode::Down, Param::Pitch),
            (ratatui::crossterm::event::KeyCode::Up, Param::Volume),
        ] {
            let a = keymap.resolve_in(app.screen, app.edit, &key(code)).unwrap();
            app.apply(a);
            assert_eq!(app.pads.param, want);
        }
        assert_eq!(
            app.pads.selected, selected,
            "the arrows did not select a pad"
        );
    }

    /// Recorded: Enter, K to attack, → and Ctrl+→. 21: "Arrows change the picked value".
    #[test]
    fn arrows_change_the_picked_value() {
        let mut app = test_app();
        app.apply(input::Action::PadEdit);
        // volume, pitch, mute group (70), attack
        for _ in 0..3 {
            app.apply(input::Action::ParamNext);
        }
        let (abs, selected) = (app.pads.selected_abs(), app.pads.selected);
        let was = app.pads.attack_ms[abs];
        app.apply(input::Action::EditValue {
            up: true,
            fine: false,
        });
        assert_eq!(app.pads.attack_ms[abs], was + 5, "a step");
        app.apply(input::Action::EditValue {
            up: true,
            fine: true,
        });
        assert_eq!(app.pads.attack_ms[abs], was + 6, "a fine step");
        assert_eq!(app.pads.selected, selected);
        assert_eq!(flash(&app), "attack: 6 ms");
    }

    /// Recorded: Alt+← on a pad with volume picked. 21: "The quick way outside the mode";
    /// pad-playback 21: "Pad volume from the keyboard".
    #[test]
    fn the_quick_way_outside_the_mode() {
        let mut app = test_app();
        let (abs, selected) = (app.pads.selected_abs(), app.pads.selected);
        let was = app.pads.volume_db[abs];
        app.apply(input::Action::EditValue {
            up: false,
            fine: false,
        });
        assert_eq!(app.pads.volume_db[abs], was - 1, "1 dB down");
        assert_eq!(app.pads.selected, selected, "the selection stays");
        assert!(!app.edit, "no edit mode was needed");
        // the files screen has no value to change: a pad there stays as it was
        app.apply(input::Action::ScreenFiles);
        app.apply(input::Action::EditValue {
            up: false,
            fine: false,
        });
        assert_eq!(app.pads.volume_db[abs], was - 1);
    }

    /// Recorded: Alt+←, K, Ctrl+Alt+→ on slice 2. 21: "Alt moves the picked bound".
    #[test]
    fn alt_moves_the_picked_bound() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        let mut c = chop_with_parts(2);
        c.select_slice(1);
        app.chop = Some(c);
        let bounds = |app: &super::App| {
            let s = app.chop.as_ref().unwrap().slices()[1];
            (s.start, s.end)
        };
        let (start, end) = bounds(&app);
        app.apply(input::Action::EditValue {
            up: false,
            fine: false,
        });
        let step = start - bounds(&app).0;
        assert!(step > 0, "the start moved left");
        assert_eq!(bounds(&app).1, end, "the end stayed");
        app.apply(input::Action::ParamNext);
        app.apply(input::Action::EditValue {
            up: true,
            fine: true,
        });
        let fine = bounds(&app).1 - end;
        // the first repeat of a held key is as fine as a tap: the speed comes with holding
        assert_eq!(fine, step, "the end moved a fine step right");
    }

    /// Recorded: a pad volume knob, then Enter and → on attack. 21: "A knob lights what it
    /// turns", "The frame tells the mode", "The knob numbers".
    #[test]
    fn a_knob_lights_what_it_turns() {
        use super::pads::Param as P;
        let mut app = test_app();
        app.midi_event(super::midi::Event::Param {
            port: "X-Session".into(),
            control: "cc 24".into(),
            param: input::midi::Param::PadVolume,
            value: 0.2,
            takeover: false,
        });
        assert_eq!(
            app.lit_now(),
            Some(super::Lit::Pad(P::Volume)),
            "outside the mode too"
        );
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        let v = app.mark_pads(v);
        assert_eq!(v.lit, Some(tui::PadParam::Volume));
        assert_eq!(v.edit, None, "no frame while the mode is closed");
        // the edit mode on the ENV page: the frame names it, the knobs carry numbers
        app.apply(input::Action::PadEdit);
        // volume, pitch, mute group (70), attack
        for _ in 0..3 {
            app.apply(input::Action::ParamNext);
        }
        app.apply(input::Action::EditValue {
            up: true,
            fine: false,
        });
        assert_eq!(app.lit_now(), Some(super::Lit::Pad(P::Attack)));
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        let v = app.mark_pads(v);
        assert_eq!(v.edit, Some("ENV"));
        assert_eq!(
            v.knobs,
            [
                tui::PadParam::Attack,
                tui::PadParam::Decay,
                tui::PadParam::Release
            ]
        );
        // the help line of the edit mode says how to listen
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let items = line(&super::help_groups(
            &keymap,
            input::Screen::Pads,
            &app.help_state(),
        ));
        assert!(
            items.contains(&("pad".into(), "listen".into())),
            "{items:?}"
        );
    }

    /// Recorded: K, Alt+→ on a slice. 21: "The picked bound is visible".
    #[test]
    fn the_picked_bound_is_visible() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        let mut c = chop_with_parts(2);
        c.select_slice(1);
        app.chop = Some(c);
        app.apply(input::Action::ParamNext);
        app.apply(input::Action::EditValue {
            up: true,
            fine: false,
        });
        assert_eq!(app.lit_now(), Some(super::Lit::Bound(true)));
        let c = app.chop.as_ref().unwrap();
        let view = || super::chop_view(c, None, None, 48_000, &app.pads);
        let v = app.mark_chop(view(), None);
        assert_eq!(v.hot, Some(true), "the end just moved: it is in play");
        assert!(!v.edit);
        // 21 "A held modifier lights its bound": Ctrl the start, Alt the end
        assert_eq!(
            app.mark_chop(view(), Some(input::Modifier::Ctrl)).hot,
            Some(false)
        );
        assert_eq!(
            app.mark_chop(view(), Some(input::Modifier::Alt)).hot,
            Some(true)
        );
    }

    /// The default keymap. 21: "No command on the left half".
    #[test]
    fn no_command_on_the_left_half() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let left = "12345QWERTASDFGZXCVB";
        let mut letters: std::collections::HashMap<String, input::Action> = Default::default();
        for screen in [
            input::Screen::Pads,
            input::Screen::Chop,
            input::Screen::Files,
            input::Screen::Settings,
        ] {
            for (k, a) in keymap.bindings(screen) {
                if k.chars().count() != 1 {
                    continue;
                }
                assert!(!left.contains(&k), "{k} is on the left half ({screen:?})");
                // the direction letters act like the arrows: their meaning follows the screen;
                // M is the marker key: a pad mark here, a cut marker there;
                // 83a: H is the pad mode here and hits there, O picks the play mode and
                // turns the bank
                if k.chars().all(|c| c.is_ascii_alphabetic())
                    && !"JLIKMHOU".contains(&k)
                    && let Some(was) = letters.insert(k.clone(), a)
                {
                    assert_eq!(was, a, "{k} means two things");
                }
            }
        }
    }

    #[test]
    fn no_kitty_protocol_keeps_the_ctrl_combos() {
        let items = line(&pads_line(false));
        assert!(items.iter().any(|(k, _)| k == "Alt+R"), "{items:?}");
    }

    #[test]
    fn a_button_records_press_twice() {
        use super::{Rec, record_step};
        use crate::audio::SOURCE_MIC;
        // a controller sends presses only: the first starts, the second stops
        let mic = Action::RecordMic(true);
        assert_eq!(record_step(mic, None), Some(Rec::Start(SOURCE_MIC)));
        assert_eq!(record_step(mic, Some(SOURCE_MIC)), Some(Rec::Stop));
        let out = Action::RecordOutputToggle;
        assert_eq!(
            record_step(out, None),
            Some(Rec::Start(crate::audio::SOURCE_OUTPUT))
        );
        assert_eq!(
            record_step(out, Some(crate::audio::SOURCE_OUTPUT)),
            Some(Rec::Stop)
        );
    }

    #[test]
    fn no_microphone_release_keeps_nothing() {
        // the press found no microphone, nothing runs: the release must not save the old ring
        assert_eq!(super::record_step(Action::RecordMic(false), None), None);
    }

    #[test]
    fn one_recording_at_a_time() {
        use super::{Rec, record_step};
        use crate::audio::{SOURCE_MIC, SOURCE_OUTPUT};
        // Ctrl+R pressed and released during the output recording leaves it running
        assert_eq!(
            record_step(Action::RecordMic(true), Some(SOURCE_OUTPUT)),
            None
        );
        assert_eq!(
            record_step(Action::RecordMic(false), Some(SOURCE_OUTPUT)),
            None
        );
        // and Ctrl+T during the microphone recording
        assert_eq!(
            record_step(Action::RecordOutputToggle, Some(SOURCE_MIC)),
            None
        );
        assert_eq!(
            record_step(Action::RecordMic(false), Some(SOURCE_MIC)),
            Some(Rec::Stop)
        );
    }

    #[test]
    fn recording_names_carry_the_time() {
        // 2026-10-28 18:45:00 UTC = 1793213100
        let t = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_793_213_100);
        assert_eq!(super::stamp(t), "20261028-184500");
        assert_eq!(super::stamp(std::time::UNIX_EPOCH), "19700101-000000");
    }

    #[test]
    fn audio_trouble_is_said_in_plain_words_once() {
        // the corner indicator carries the state; the status line gets only these words,
        // and they are flashed once instead of living in the startup notices
        for msg in [super::NO_AUDIO_SERVER, super::AUDIO_FALLBACK] {
            assert!(
                !msg.contains("ClientStatus"),
                "no internal error text: {msg}"
            );
            assert!(!msg.contains("unavailable: "), "no raw error text: {msg}");
            assert!(msg.ends_with('.'), "{msg}");
        }
        assert!(super::NO_AUDIO_SERVER.starts_with("No sound:"));
    }

    /// 77 "F1 names the mark": `✱` and the mode glyphs, no frame glyph.
    #[test]
    fn f1_names_the_mark() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let rows = super::full_help(&keymap, input::Screen::Pads, true);
        for (glyph, word) in [
            ("✱", "more settings"),
            ("no mark", "one-shot"),
            ("⇥", "gate"),
        ] {
            assert!(
                rows.iter().any(|(k, t)| k == glyph && t.contains(word)),
                "{glyph}: {rows:?}"
            );
        }
        for old in ["⇕", "⩘", "∿", "◻", "↯", "◌", "⋯"] {
            assert!(!rows.iter().any(|(k, _)| k == old), "{old}: {rows:?}");
        }
    }

    /// Scenario "Terminal without the kitty protocol" (22): the warning is shown until
    /// the first keypress of any kind, then the hint line is back, and the status keeps
    /// the "no releases" marker for the session.
    #[test]
    fn terminal_without_the_kitty_protocol() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut app = test_app();
        app.post_startup(Level::Warn, super::NO_RELEASES_NOTICE.into());
        assert_eq!(app.notice().as_deref(), Some(super::NO_RELEASES_NOTICE));
        // what the event loop does with every key event: dismiss first, resolve after
        let feed = |app: &mut super::App, e: KeyEvent| {
            app.dismiss_startup();
            if let Some(action) = keymap.resolve(input::Screen::Pads, &e) {
                app.apply(action);
            }
        };
        // a key that is no command for anyone still clears the warning
        feed(&mut app, KeyEvent::new(KeyCode::Char('5'), M::NONE));
        assert_eq!(app.notice(), None, "the warning is gone, the hint is back");
        assert!(
            !app.notices.has_startup(),
            "the whole list is cleared at once"
        );
        // and further presses keep it that way
        feed(&mut app, KeyEvent::new(KeyCode::Char('h'), M::NONE));
        assert_eq!(app.notice(), None);
    }

    /// Scenario "Banks in one row" (screen-help).
    #[test]
    fn full_help_is_grouped_and_banks_are_one_row() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let rows = super::full_help(&keymap, input::Screen::Pads, true);
        let headings: Vec<&str> = rows
            .iter()
            .filter(|r| r.0.is_empty())
            .map(|r| r.1.as_str())
            .collect();
        assert_eq!(
            headings,
            [
                "PADS",
                "BANK",
                "CHOP",
                "SOUND",
                "SCREENS",
                "GLYPHS ON A PAD"
            ]
        );
        let banks: Vec<_> = rows.iter().filter(|r| r.1.starts_with("bank ")).collect();
        assert_eq!(banks.len(), 1, "{banks:?}");
        assert_eq!(banks[0].0, "Ctrl+1…8");
        assert!(
            rows.iter()
                .all(|r| r.0.chars().count() + r.1.chars().count() < 96)
        );
    }

    #[test]
    fn pairs_in_one_row() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let rows = super::full_help(&keymap, input::Screen::Settings, true);
        let row = |keys: &str| rows.iter().find(|r| r.0 == keys).map(|r| r.1.as_str());
        // F1 names the letters and their arrow twins in one row
        assert_eq!(row("J ← L →"), Some("change value"), "{rows:?}");
        assert_eq!(row("I ↑ K ↓"), Some("select row"));
        assert_eq!(row("PgUp PgDn"), Some("previous / next bank"));
        // no key of a pair is left on a row of its own
        for lone in ["←", "→", "↑", "↓", "PgUp", "PgDn", "J", "L"] {
            assert_eq!(row(lone), None, "{lone} alone");
        }
        let pads = super::full_help(&keymap, input::Screen::Pads, true);
        assert!(
            pads.iter()
                .any(|r| r.0.contains('U') && r.0.contains('O') && r.1 == "previous / next bank"),
            "{pads:?}"
        );
        let chop = super::full_help(&keymap, input::Screen::Chop, true);
        assert!(
            chop.iter()
                .any(|r| r.0.contains("Ctrl+J") && r.1 == "chop: the slice start left / right"),
            "{chop:?}"
        );
    }

    #[test]
    fn a_text_does_not_repeat_its_key() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let rows = super::full_help(&keymap, input::Screen::Pads, true);
        assert!(rows.contains(&("Alt+R".into(), "record the microphone: hold".into())));
        for (keys, text) in rows.iter().filter(|r| !r.0.is_empty()) {
            for k in keys.split(' ').filter(|k| k.len() > 1) {
                assert!(!text.contains(k), "{text} repeats {k}");
            }
        }
    }

    #[test]
    fn step_wraps_and_starts_from_unknown() {
        let l = ["A".to_string(), "B".into(), "C".into()];
        assert_eq!(step_in(&l, Some("C"), 1).as_deref(), Some("A"));
        assert_eq!(step_in(&l, Some("A"), -1).as_deref(), Some("C"));
        assert_eq!(step_in(&l, Some("gone"), 1).as_deref(), Some("B"));
        assert_eq!(step_in(&[], None, 1), None);
    }

    /// Applies what a profile resolved, as the MIDI thread feeds it to the app.
    fn feed_resolved(app: &mut super::App, resolved: Option<input::midi::Resolved>) {
        use input::midi::Resolved;
        match resolved {
            Some(Resolved::Action(a)) => {
                app.apply(a);
            }
            Some(Resolved::Param {
                control,
                param,
                value,
                takeover,
            }) => app.midi_event(super::midi::Event::Param {
                port: "X-Session".into(),
                control,
                param,
                value,
                takeover,
            }),
            None => {}
        }
    }

    /// A recorded X-Session Pro message as the MIDI thread feeds it: the shipped profile,
    /// the app's screen and edit mode.
    fn xsession(app: &mut super::App, msg: input::midi::Msg) {
        if let Some(p) = app
            .live_profiles()
            .into_iter()
            .find(|p| p.name == "X-Session Pro")
        {
            feed_resolved(app, p.resolve_in(app.screen, app.edit, msg));
            return;
        }
        let p = input::midi::Profile::from_toml(
            "xsession-pro.toml",
            include_str!("../../../config/devices/xsession-pro.toml"),
        )
        .unwrap();
        feed_resolved(app, p.resolve_in(app.screen, app.edit, msg));
    }

    /// An app that knows the X-Session Pro profile, the device connected (59).
    fn xsession_connected(learned: &[(input::Group, u8, &str)]) -> super::App {
        let mut app = test_app();
        app.profiles = vec![
            input::midi::Profile::from_toml(
                "xsession-pro.toml",
                include_str!("../../../config/devices/xsession-pro.toml"),
            )
            .unwrap(),
        ];
        for (group, note, action) in learned {
            super::learn::assign(
                &mut app.learned,
                "X-Session Pro",
                *group,
                input::midi::Key::Note(*note),
                action,
            );
        }
        app.midi_event(super::midi::Event::Connected {
            port: "USB X-Session MIDI 1".into(),
            profile: "X-Session Pro".into(),
        });
        app
    }

    /// What the X-Session Pro message does now, with the app's bindings (59).
    fn xs_live(app: &super::App, msg: input::midi::Msg) -> Option<Action> {
        let p = app
            .live_profiles()
            .into_iter()
            .find(|p| p.name == "X-Session Pro")?;
        match p.resolve_in(app.screen, app.edit, msg)? {
            input::midi::Resolved::Action(a) => Some(a),
            _ => None,
        }
    }

    /// 59 input-devices: "First connect".
    #[test]
    fn first_connect() {
        let app = xsession_connected(&[]);
        assert!(app.seeded.iter().any(|s| s == "X-Session Pro"));
        let pads = &app.learned["X-Session Pro"]["pads"];
        assert_eq!(
            pads.get("note 58").map(String::as_str),
            Some("pad.edit_only")
        );
        assert_eq!(xs_live(&app, press(B7)), Some(Action::PadEditOnly));
        // the MIDI tab names it as the profile does, no "(learned)"
        let p = app.live_profiles().remove(0);
        assert_eq!(
            p.assignment(input::midi::Scope::Screen(Screen::Pads), "pad.edit_only")
                .as_deref(),
            Some("b7 (note 58)")
        );
        // the edit layer went into both edit groups
        assert_eq!(
            app.learned["X-Session Pro"]["chop_edit"]
                .get("note 46")
                .map(String::as_str),
            Some("edit.page_prev")
        );
        assert_eq!(xs_live(&app, press(B10)), None);
        assert_eq!(xs_live(&app, press(B9)), None);
        assert_eq!(xs_live(&app, press(B3)), Some(Action::ScreenFx));
        assert!(
            !app.learned["X-Session Pro"]
                .values()
                .any(|group| { group.values().any(|action| action == "pad.mode_next") })
        );
    }

    /// 59 input-devices: "Learned wins".
    #[test]
    fn learned_wins() {
        let app = xsession_connected(&[(input::Group::Pads, B3, "bank.next")]);
        assert_eq!(xs_live(&app, press(B3)), Some(Action::BankNext));
        assert_ne!(
            app.learned["X-Session Pro"]["pads"]
                .get("note 45")
                .map(String::as_str),
            Some("screen.fx")
        );
        assert_eq!(xs_live(&app, press(B7)), Some(Action::PadEditOnly));
    }

    fn xs_raw(app: &mut super::App, bytes: &[u8]) -> Option<input::midi::Resolved> {
        let msg = input::midi::Msg::parse(bytes).expect("a factory MIDI message");
        let result = resolved(app, app.screen, app.edit, msg);
        feed_resolved(app, result.clone());
        result
    }

    fn xs_button(app: &mut super::App, note: u8) {
        xs_raw(app, &[0x90, note, 127]);
        for status in [0x80, 0x90] {
            assert_eq!(xs_raw(app, &[status, note, 0]), None, "release {note}");
        }
    }

    #[test]
    fn x_session_physical_navigation_and_preview() {
        fn walkthrough(mut app: super::App) {
            use super::pads::Param as P;
            let dir = tmp_dir("xs-physical-navigation");
            std::fs::create_dir_all(&dir).unwrap();
            let sound = Sample::mono(vec![0.1; 48_000]);
            for name in ["a.wav", "b.wav"] {
                super::wav::write(&dir.join(name), &sound, 48_000).unwrap();
            }
            app.browser = browser::Browser::new(Some(dir.clone()));
            for note in [B9, B10] {
                assert_eq!(xs_raw(&mut app, &[0x90, note, 127]), None);
                assert!(!app.edit);
            }
            xs_button(&mut app, B1);
            assert_eq!(app.pads.selected, 1);
            xs_button(&mut app, B4);
            assert_eq!(app.pads.selected, 0);
            xs_button(&mut app, 57);
            assert_eq!(app.pads.bank, 1);
            xs_button(&mut app, 56);
            assert_eq!(app.pads.bank, 0);
            xs_button(&mut app, B7);
            assert!(app.edit);
            xs_button(&mut app, B1);
            assert_eq!(app.pads.page().0, "ENV");
            xs_raw(&mut app, &[0xB0, K1, 64]);
            assert!(app.pads.attack_ms[0] > 0);
            assert_eq!(app.lit_now(), Some(super::Lit::Pad(P::Attack)));
            xs_button(&mut app, B1);
            assert_eq!(app.pads.page().0, "FILTER");
            xs_button(&mut app, B4);
            assert_eq!(app.pads.page().0, "ENV");
            for note in [B9, B10] {
                assert_eq!(xs_raw(&mut app, &[0x90, note, 127]), None);
                assert!(app.edit);
            }
            xs_button(&mut app, B8);
            assert!(!app.edit);
            xs_button(&mut app, B7);
            assert!(app.edit);
            xs_button(&mut app, B7);
            assert!(!app.edit);

            xs_button(&mut app, 44);
            assert_eq!(app.screen, Screen::Files);
            assert_eq!(app.browser.selected, 0);
            xs_button(&mut app, B1);
            assert_eq!(app.browser.selected, 1);
            wait_for(&mut app, |a| a.preview.is_some());
            xs_button(&mut app, B4);
            assert_eq!(app.browser.selected, 0);
            wait_for(&mut app, |a| a.preview.is_some());
            for note in [B10, B9] {
                xs_button(&mut app, note);
                assert!(!app.preview_on);
                assert!(app.preview.is_none());
                xs_button(&mut app, note);
                assert!(app.preview_on);
                wait_for(&mut app, |a| a.preview.is_some());
            }
            xs_button(&mut app, B7);
            assert_eq!(app.browser.in_hand, Some(dir.join("a.wav")));
            xs_button(&mut app, B3);
            assert_eq!(app.screen, Screen::Fx);
            xs_button(&mut app, B8);
            assert!(app.browser.in_hand.is_none());
            xs_button(&mut app, 44);
            assert_eq!(app.screen, Screen::Files);
            xs_button(&mut app, B8);
            wait_for(&mut app, |a| a.screen == Screen::Chop);
            assert!(!app.edit);
            assert!(app.plan.is_none());
            let cuts = app.chop.as_ref().unwrap().markers.clone();
            xs_raw(&mut app, &[0xB0, K3, 64]);
            let cursor = app.chop.as_ref().unwrap().cursor;
            assert!(cursor > 0 && cursor < 48_000);
            xs_button(&mut app, B7);
            assert_eq!(app.chop.as_ref().unwrap().markers.len(), cuts.len() + 1);
            assert!(app.chop.as_ref().unwrap().markers.contains(&cursor));
            let selected = app.chop.as_ref().unwrap().selected;
            xs_button(&mut app, B4);
            assert_eq!(app.chop.as_ref().unwrap().selected, selected - 1);
            xs_button(&mut app, B1);
            assert_eq!(app.chop.as_ref().unwrap().selected, selected);
            xs_button(&mut app, B8);
            assert!(app.edit);
            assert!(app.plan.is_none());
            let start = app.chop.as_ref().unwrap().slices()[selected].start;
            xs_raw(&mut app, &[0xB0, K1, 80]);
            assert_ne!(app.chop.as_ref().unwrap().slices()[selected].start, start);
            for edit in [true, false] {
                assert_eq!(app.edit, edit);
                for note in [B10, B9] {
                    app.fake_playhead = None;
                    app.sent.clear();
                    xs_button(&mut app, note);
                    assert!(last_played(&app).is_some());
                    app.fake_playhead = Some(1);
                    app.sent.clear();
                    xs_button(&mut app, note);
                    assert!(
                        app.sent
                            .iter()
                            .any(|c| matches!(c, Command::Preview { sound: None, .. }))
                    );
                    assert!(app.plan.is_none());
                }
                app.fake_playhead = None;
                if edit {
                    xs_button(&mut app, B7);
                    assert!(!app.edit);
                }
            }
            xs_button(&mut app, B8);
            assert!(app.edit);
            xs_button(&mut app, B8);
            assert!(!app.edit);
            xs_button(&mut app, B3);
            assert_eq!(app.screen, Screen::Fx);
            let one = app.pads.fx[0];
            xs_button(&mut app, B1);
            assert_eq!(app.fx_slot, 1);
            assert_eq!(app.pads.fx[0], one);
            xs_raw(&mut app, &[0xB0, K1, 10]);
            xs_raw(&mut app, &[0xB0, K2, 89]);
            xs_raw(&mut app, &[0xB0, K3, 25]);
            assert_eq!(app.pads.fx[1].kind, engine::fx::FxKind::Reverb);
            assert_eq!((app.pads.fx[1].size, app.pads.fx[1].tone), (70, 20));
            assert_eq!(app.pads.fx[0], one);
            xs_button(&mut app, B4);
            assert_eq!(app.fx_slot, 0);
            for note in [B9, B10] {
                assert_eq!(xs_raw(&mut app, &[0x90, note, 127]), None);
            }
            xs_button(&mut app, B8);
            assert_eq!(app.screen, Screen::Pads);
            assert!(!app.edit);
            std::fs::remove_dir_all(dir).unwrap();
        }
        let mut unseeded = test_app();
        unseeded.profiles.push(
            input::midi::Profile::from_toml(
                "xsession-pro.toml",
                include_str!("../../../config/devices/xsession-pro.toml"),
            )
            .unwrap(),
        );
        unseeded
            .midi_devices
            .push(("USB X-Session MIDI 1".into(), "X-Session Pro".into()));
        walkthrough(unseeded);
        walkthrough(xsession_connected(&[]));
    }

    #[test]
    fn x_session_faders_use_physical_roles() {
        use input::midi::{Param as P, Resolved};
        let mut app = xsession_connected(&[]);
        for (screen, edit) in [
            (Screen::Pads, false),
            (Screen::Pads, true),
            (Screen::Files, false),
            (Screen::Chop, false),
            (Screen::Chop, true),
            (Screen::Fx, false),
            (Screen::Settings, false),
        ] {
            app.goto(screen);
            app.edit = edit;
            app.pads.selected = 3;
            let abs = app.pads.selected_abs();
            for value in [0, 127] {
                for (cc, param) in [(11, P::PadVolume), (12, P::PadPitch), (14, P::MasterVolume)] {
                    assert!(
                        matches!(
                            xs_raw(&mut app, &[0xB0, cc, value]),
                            Some(Resolved::Param { param: actual, .. }) if actual == param
                        ),
                        "{screen:?} edit {edit}: CC{cc}"
                    );
                }
                assert_eq!(
                    app.pads.volume_db[abs],
                    if value == 0 {
                        super::pads::MIN_DB
                    } else {
                        super::pads::MAX_DB
                    }
                );
                assert_eq!(app.pads.pitch[abs], if value == 0 { -24 } else { 24 });
                assert_eq!(app.master_db, if value == 0 { -40 } else { 6 });
            }
            let before = (
                app.before().pads,
                app.pads.fx,
                app.master_db,
                app.sent.len(),
            );
            for cc in [15, 17, 20] {
                for value in [0, 127] {
                    assert_eq!(xs_raw(&mut app, &[0xB0, cc, value]), None);
                }
            }
            assert_eq!(
                (
                    app.before().pads,
                    app.pads.fx,
                    app.master_db,
                    app.sent.len()
                ),
                before
            );
        }
    }

    #[test]
    fn x_session_safe_default_buttons() {
        use input::midi::Resolved;
        let mut app = xsession_connected(&[]);
        for screen in [
            Screen::Pads,
            Screen::Files,
            Screen::Chop,
            Screen::Fx,
            Screen::Settings,
        ] {
            for edit in [false, true] {
                for note in [43, 44, 45, 46, 56, 57, 58, 59, 69, 70] {
                    let msg = input::midi::Msg::parse(&[0x90, note, 127]).unwrap();
                    assert!(
                        !matches!(
                            resolved(&app, screen, edit, msg),
                            Some(Resolved::Action(
                                Action::MarkerDelete
                                    | Action::ChopCommit
                                    | Action::BrowserLoad
                                    | Action::SettingsLearn
                            ))
                        ),
                        "{screen:?} edit {edit}: note {note}"
                    );
                    for status in [0x80, 0x90] {
                        let release = input::midi::Msg::parse(&[status, note, 0]).unwrap();
                        assert_eq!(resolved(&app, screen, edit, release), None);
                    }
                }
            }
        }

        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(4));
        app.pads.file[0] = Some("old.wav".into());
        app.browser.in_hand = Some("held.wav".into());
        app.plan = Some(super::chop::Plan::InOrder(Some(0)));
        let pads = app.before().pads;
        xs_button(&mut app, B8);
        assert!(app.plan.is_some(), "Cue must not apply a pending plan");
        assert!(!app.edit, "edit and a pending plan must not coexist");
        assert!(app.ask.is_none());
        assert_eq!(app.before().pads, pads);

        app.apply(Action::ChopCommit);
        assert!(matches!(app.ask, Some(super::Ask::Plan { .. })));
        let question = app.ask.as_ref().unwrap().lines().to_vec();
        for note in [B7, B8] {
            xs_button(&mut app, note);
            assert!(app.plan.is_some());
            assert_eq!(app.ask.as_ref().unwrap().lines(), question);
            assert_eq!(app.before().pads, pads);
        }
        app.ask = None;
        app.plan = None;
        xs_button(&mut app, B8);
        assert!(app.edit, "Cue opens slice edit when no plan is pending");
        xs_button(&mut app, B8);
        assert!(
            app.browser.in_hand.is_none(),
            "Back releases the held file first"
        );
        assert!(app.edit);
        xs_button(&mut app, B8);
        assert!(!app.edit);
        app.browser.in_hand = Some("held.wav".into());

        for screen in [Screen::Pads, Screen::Files] {
            app.screen = screen;
            app.edit = false;
            let held = app.browser.in_hand.clone();
            xs_button(&mut app, B7);
            assert_eq!(app.browser.in_hand, held, "{screen:?}: no placement");
            assert_eq!(app.before().pads, pads);
            assert!(app.ask.is_none());
            assert_eq!(app.edit, screen == Screen::Pads);
            app.edit = false;
            app.put_in_hand(0);
            assert!(matches!(app.ask, Some(super::Ask::PutFile { .. })));
            let question = app.ask.as_ref().unwrap().lines().to_vec();
            xs_button(&mut app, B7);
            assert_eq!(app.ask.as_ref().unwrap().lines(), question);
            assert_eq!(app.browser.in_hand, held);
            assert_eq!(app.before().pads, pads);
            assert!(!app.edit);
            app.ask = None;
        }
        app.pads.selected = 1;
        xs_button(&mut app, B7);
        assert!(app.pads.file[1].is_none());
        assert_eq!(app.browser.in_hand, Some("held.wav".into()));
        assert!(app.ask.is_none());
        app.screen = Screen::Pads;
        app.pad_in_hand = Some(0);
        xs_button(&mut app, B7);
        assert!(app.edit);
        assert_eq!(app.pad_in_hand, Some(0), "Cue must not swap a held pad");
        assert_eq!(app.before().pads, pads);
        app.pad_in_hand = None;
        app.edit = false;
        for (screen, action) in [
            (Screen::Files, Action::PadEditOnly),
            (Screen::Pads, Action::ChopEditOnly),
            (Screen::Pads, Action::BrowserTakeOnly),
            (Screen::Fx, Action::PadEditOnly),
            (Screen::Fx, Action::ChopEditOnly),
            (Screen::Fx, Action::BrowserTakeOnly),
        ] {
            app.screen = screen;
            app.apply(action);
            assert!(!app.edit);
            assert!(app.ask.is_none());
            assert_eq!(app.browser.in_hand, Some("held.wav".into()));
            assert_eq!(app.before().pads, pads);
        }
        for ask in [
            super::Ask::ResetPads {
                pads: vec![0],
                lines: vec!["reset?".into()],
            },
            super::Ask::ClearPads {
                pads: vec![0],
                lines: vec!["clear?".into()],
            },
            super::Ask::ResetProfile {
                name: "X-Session Pro".into(),
                port: "X".into(),
                lines: vec!["profile?".into()],
            },
        ] {
            app.ask = Some(ask);
            let question = app.ask.as_ref().unwrap().lines().to_vec();
            let learned = app.learned.clone();
            for (screen, note) in [(Screen::Pads, B7), (Screen::Chop, B8), (Screen::Files, B7)] {
                app.screen = screen;
                xs_button(&mut app, note);
                assert_eq!(app.ask.as_ref().unwrap().lines(), question);
                assert_eq!(app.before().pads, pads);
                assert_eq!(app.learned, learned);
            }
        }
    }

    #[test]
    fn x_session_replacement_preserves_existing_assignments() {
        use input::midi::Key;
        let dir = tmp_dir("xs-legacy-map");
        std::fs::create_dir_all(&dir).unwrap();
        let cfg = dir.join("config.toml");
        let mut app = xsession_connected(&[]);
        app.user_path = Some(cfg.clone());
        app.learned.clear();
        for group in input::Group::ALL {
            app.learned
                .entry("X-Session Pro".into())
                .or_default()
                .insert(group.key().into(), Default::default());
        }
        for (group, note, action) in [
            (input::Group::Pads, B10, "pad.edit"),
            (input::Group::Chop, B9, "chop.commit"),
            (input::Group::Chop, 56, "marker.delete"),
        ] {
            super::learn::assign(
                &mut app.learned,
                "X-Session Pro",
                group,
                Key::Note(note),
                action,
            );
        }
        app.save_midi();
        super::config::save_section(
            &cfg,
            "devices",
            &super::config::DevicesConfig {
                seeded: app.seeded.clone(),
            },
        )
        .unwrap();
        let legacy = app.learned.clone();
        let profiles = app.profiles.clone();
        let load = || {
            let loaded = super::config::load_from(&cfg);
            assert!(loaded.error.is_none());
            let mut next = test_app();
            next.profiles = profiles.clone();
            next.user_path = Some(cfg.clone());
            next.learned = loaded.config.midi;
            next.seeded = loaded.config.devices.seeded;
            next.midi_event(super::midi::Event::Connected {
                port: "USB X-Session MIDI 1".into(),
                profile: "X-Session Pro".into(),
            });
            next
        };
        let mut app = load();
        for reconnect in [false, true] {
            if reconnect {
                app.midi_event(super::midi::Event::Disconnected {
                    port: "USB X-Session MIDI 1".into(),
                });
                app.midi_event(super::midi::Event::Connected {
                    port: "USB X-Session MIDI 1".into(),
                    profile: "X-Session Pro".into(),
                });
            }
            assert_eq!(app.learned, legacy);
            assert_eq!(xs_on(&mut app, Screen::Pads, B10), Some(Action::PadEdit));
            assert_eq!(xs_on(&mut app, Screen::Pads, B7), None);
            assert_eq!(xs_on(&mut app, Screen::Chop, B9), Some(Action::ChopCommit));
            assert_eq!(
                xs_on(&mut app, Screen::Chop, 56),
                Some(Action::MarkerDelete)
            );
        }
        app.apply(Action::ScreenSettings);
        app.settings_tab = 2;
        app.midi_row = app
            .midi_list()
            .iter()
            .position(|r| matches!(r, super::MidiListRow::Reset { .. }))
            .unwrap();
        let before = std::fs::read_to_string(&cfg).unwrap();
        app.apply(Action::SettingsLearn);
        assert!(matches!(app.ask, Some(super::Ask::ResetProfile { .. })));
        app.apply(Action::Back);
        assert!(app.ask.is_none());
        assert_eq!(app.learned, legacy);
        assert_eq!(std::fs::read_to_string(&cfg).unwrap(), before);
        let mut reloaded = load();
        assert_eq!(reloaded.learned, legacy);
        assert_eq!(
            xs_on(&mut reloaded, Screen::Pads, B10),
            Some(Action::PadEdit)
        );
        assert_eq!(
            xs_on(&mut reloaded, Screen::Chop, B9),
            Some(Action::ChopCommit)
        );
        assert_eq!(
            xs_on(&mut reloaded, Screen::Chop, 56),
            Some(Action::MarkerDelete)
        );
        app.apply(Action::SettingsLearn);
        assert!(app.ask.is_some());
        assert_eq!(app.learned, legacy);
        app.apply(Action::SettingsLearn);
        assert!(app.ask.is_none());
        let defaults = xsession_connected(&[]).learned;
        assert_eq!(app.learned, defaults);
        for app in [&mut app, &mut load()] {
            assert_eq!(app.learned, defaults);
            assert_eq!(xs_on(app, Screen::Pads, B10), None);
            assert_eq!(xs_on(app, Screen::Pads, B7), Some(Action::PadEditOnly));
            assert_eq!(xs_on(app, Screen::Pads, B3), Some(Action::ScreenFx));
            assert_eq!(xs_on(app, Screen::Chop, B9), Some(Action::ChopPlay));
            assert_eq!(xs_on(app, Screen::Chop, 56), Some(Action::BankPrev));
            assert_eq!(xs_on(app, Screen::Chop, B8), Some(Action::ChopEditOnly));
            app.edit = true;
            assert_eq!(xs_live(app, press(B7)), Some(Action::Back));
            assert_eq!(xs_live(app, press(B8)), Some(Action::Back));
            assert_eq!(xs_live(app, press(B4)), Some(Action::PagePrev));
            assert_eq!(xs_live(app, press(B1)), Some(Action::PageNext));
        }
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// 59 midi-learn: "Clear a profile role" and "Cleared stays cleared".
    #[test]
    fn clear_a_profile_role() {
        let mut app = xsession_connected(&[]);
        assert_eq!(xs_live(&app, press(B7)), Some(Action::PadEditOnly));
        super::learn::clear(&mut app.learned, input::Group::Pads, "pad.edit_only");
        app.sync_learned("X-Session Pro", input::Group::Pads);
        assert_eq!(xs_live(&app, press(B7)), None, "no profile role under it");
        // every row cleared, the device connects again: nothing is copied back
        app.learned
            .insert("X-Session Pro".into(), Default::default());
        for g in input::Group::ALL {
            app.sync_learned("X-Session Pro", g);
        }
        app.midi_event(super::midi::Event::Connected {
            port: "USB X-Session MIDI 1".into(),
            profile: "X-Session Pro".into(),
        });
        assert!(app.learned["X-Session Pro"].values().all(|m| m.is_empty()));
        assert_eq!(xs_live(&app, press(B7)), None);
    }

    /// 59 midi-learn: "Reset to profile".
    #[test]
    fn reset_to_profile() {
        let mut app = xsession_connected(&[]);
        super::learn::clear(&mut app.learned, input::Group::Pads, "pad.edit_only");
        super::learn::assign(
            &mut app.learned,
            "X-Session Pro",
            input::Group::Pads,
            input::midi::Key::Note(B3),
            "bank.next",
        );
        app.sync_learned("X-Session Pro", input::Group::Pads);
        assert_eq!(xs_live(&app, press(B7)), None);
        app.apply(Action::ScreenSettings);
        app.settings_tab = 2;
        app.midi_row = app
            .midi_list()
            .iter()
            .position(|r| matches!(r, super::MidiListRow::Reset { .. }))
            .expect("a reset row for the connected device");
        app.apply(Action::SettingsLearn);
        assert!(app.ask.is_some(), "it asks first");
        app.apply(Action::Back);
        assert!(app.ask.is_none());
        assert_eq!(
            app.learned["X-Session Pro"]["pads"]
                .get("note 45")
                .map(String::as_str),
            Some("bank.next"),
            "Esc keeps the bindings"
        );
        app.apply(Action::SettingsLearn);
        app.apply(Action::SettingsLearn);
        assert!(app.ask.is_none());
        app.screen = Screen::Pads;
        assert_eq!(xs_live(&app, press(B7)), Some(Action::PadEditOnly));
        assert_eq!(xs_live(&app, press(B3)), Some(Action::ScreenFx));
    }

    fn press(note: u8) -> input::midi::Msg {
        input::midi::Msg::NoteOn {
            ch: 1,
            note,
            velocity: 127,
        }
    }

    fn knob(cc: u8, value: u8) -> input::midi::Msg {
        input::midi::Msg::Cc { ch: 1, cc, value }
    }

    const B1: u8 = 43;
    const B3: u8 = 45;
    const B4: u8 = 46;
    const B7: u8 = 58;
    const B8: u8 = 59;
    const B9: u8 = 69;
    const B10: u8 = 70;
    const K1: u8 = 24;
    const K2: u8 = 25;
    const K3: u8 = 26;
    /// The X-Session knob k11 of the shipped profile: the chop threshold since 04d.
    const K11: u8 = 35;
    /// The X-Session knob k12 of the shipped profile: the chop zoom since 17.
    const K12: u8 = 36;

    /// A recorded message through controls learned for the chop screen, as the MIDI
    /// thread feeds it: the shipped profile with `map` on top, the app's screen.
    fn learned(app: &mut super::App, map: &[(input::midi::Key, &str)], msg: input::midi::Msg) {
        use input::midi::{Profile, Scope};
        let mut p = Profile::from_toml(
            "xsession-pro.toml",
            include_str!("../../../config/devices/xsession-pro.toml"),
        )
        .unwrap();
        p.set_learned(
            Scope::Screen(input::Screen::Chop),
            map.iter().map(|(k, a)| (*k, a.to_string())).collect(),
        );
        feed_resolved(app, p.resolve_in(app.screen, app.edit, msg));
    }

    /// A recorded message through controls learned on the pads screen (`screen` map) and
    /// in its edit layer (`edit` map), as the MIDI thread feeds it: the shipped profile
    /// with the maps on top, the app's screen and edit mode.
    fn learned_pads(
        app: &mut super::App,
        screen: &[(input::midi::Key, &str)],
        edit: &[(input::midi::Key, &str)],
        msg: input::midi::Msg,
    ) {
        use input::midi::{Profile, Scope};
        let mut p = Profile::from_toml(
            "xsession-pro.toml",
            include_str!("../../../config/devices/xsession-pro.toml"),
        )
        .unwrap();
        let set = |p: &mut Profile, scope, m: &[(input::midi::Key, &str)]| {
            p.set_learned(scope, m.iter().map(|(k, a)| (*k, a.to_string())).collect());
        };
        set(&mut p, Scope::Screen(input::Screen::Pads), screen);
        set(&mut p, Scope::Edit(Screen::Pads), edit);
        feed_resolved(app, p.resolve_in(app.screen, app.edit, msg));
    }

    /// 21 input-devices: "Edit a pad from the controller only", "X-Session Pro in the pad
    /// edit mode".
    /// Scenario "X-Session Pro in the pad edit mode" (input-devices).
    #[test]
    fn edit_a_pad_from_the_controller_only() {
        use super::pads::Param as P;
        let mut app = test_app();
        let abs = app.pads.selected_abs();
        xsession(&mut app, press(B7));
        assert!(app.edit, "left Cue opens the edit");
        xsession(&mut app, press(B1));
        assert_eq!(app.pads.page().0, "ENV", "left forward turns the page");
        // k1 is the first slot of ENV: the attack, picked up at 0 ms
        xsession(&mut app, knob(K1, 0));
        xsession(&mut app, knob(K1, 64));
        assert!(app.pads.attack_ms[abs] > 0);
        assert_eq!(
            app.lit_now(),
            Some(super::Lit::Pad(P::Attack)),
            "the screen says which"
        );
        xsession(&mut app, press(B7));
        assert!(!app.edit, "left Cue closes the edit");
    }

    /// 54 input-devices: "Pages from the controller".
    #[test]
    fn pages_from_the_controller() {
        use super::pads::Param as P;
        let mut app = test_app();
        let abs = app.pads.selected_abs();
        xsession(&mut app, press(B7));
        let mut pages = Vec::new();
        for b in [B1, B1, B4] {
            xsession(&mut app, press(b));
            pages.push(app.pads.page().0);
        }
        assert_eq!(pages, ["ENV", "FILTER", "ENV"]);
        // the knobs turn the page shown: k3 is the release
        xsession(&mut app, knob(K3, 0));
        xsession(&mut app, knob(K3, 64));
        assert!(app.pads.release_ms[abs] > 0);
        assert_eq!(app.lit_now(), Some(super::Lit::Pad(P::Release)));
    }

    /// 54 editing: "A knob picks the page".
    #[test]
    fn a_knob_picks_the_page() {
        use super::pads::Param as P;
        let mut app = test_app();
        app.apply(input::Action::PadEdit);
        let page = |app: &mut super::App, value: f32| {
            app.midi_event(super::midi::Event::Param {
                port: "X-Session".into(),
                control: "k4".into(),
                param: input::midi::Param::Page,
                value,
                takeover: true,
            })
        };
        // Five equal parts: the middle selects FILTER, without pickup.
        page(&mut app, 0.5);
        assert_eq!(app.pads.page().0, "FILTER");
        assert_eq!(app.pads.param, P::Filter);
        // a turn inside the same part keeps the pick
        app.apply(input::Action::ParamNext);
        page(&mut app, 0.52);
        assert_eq!(app.pads.param, P::Cutoff);
        page(&mut app, 1.0);
        assert_eq!(app.pads.page().0, "FX");
        // a slice has one page: the knob keeps it
        app.apply(input::Action::Back);
        app.screen = input::Screen::Chop;
        app.edit = true;
        app.slice_end = true;
        page(&mut app, 0.0);
        assert!(app.slice_end);
    }

    /// 54 editing: "One page" — the slice edit has BOUNDS only, the page keys keep it.
    #[test]
    fn one_page() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.edit = true;
        app.slice_end = true;
        app.apply(input::Action::PageNext);
        assert!(app.slice_end, "the slice end stays picked");
        app.apply(input::Action::PagePrev);
        assert!(app.slice_end);
    }

    /// 21 input-devices: "The ENV page on the knobs".
    #[test]
    fn the_env_page_on_the_knobs() {
        use super::pads::Param as P;
        let mut app = test_app();
        let abs = app.pads.selected_abs();
        xsession(&mut app, press(B7));
        xsession(&mut app, press(B1));
        assert_eq!(app.pads.page().0, "ENV");
        // the response jump (58): the decay goes to the knob at once, no arrow
        xsession(&mut app, knob(K2, 64));
        assert!(app.pickup.is_none(), "nothing to wait for");
        // the square curve: the middle of the knob is a quarter of 5 s
        let pos = 64.0f32 / 127.0;
        let ms = (pos * pos * super::pads::ENV_MS_LIMIT as f32).round() as u16;
        assert_eq!(app.pads.decay_ms[abs], ms);
        assert_eq!(app.lit_now(), Some(super::Lit::Pad(P::Decay)));
        assert_eq!(app.pads.attack_ms[abs], 0, "k1 was not turned");
    }

    /// 21 input-devices: "The knob keeps its role outside the edit".
    #[test]
    fn the_knob_keeps_its_role_outside_the_edit() {
        let mut app = test_app();
        let abs = app.pads.selected_abs();
        xsession(&mut app, knob(K1, 110));
        xsession(&mut app, knob(K1, 127));
        assert_eq!(
            app.pads.volume_db[abs],
            super::pads::MAX_DB,
            "k1 is the pad volume"
        );
        xsession(&mut app, knob(K2, 0));
        xsession(&mut app, knob(K2, 127));
        assert_eq!(app.pads.pitch[abs], 0, "k2 has no pads role");
        assert!(!app.edit);
    }

    /// 28 midi-learn: "Same knob, two layers" — a knob learned for "pad volume" in PADS
    /// and for "knob 1 of the page" in PADS · EDIT turns the volume with the edit closed
    /// and the attack with the edit open on the ENV page.
    #[test]
    fn same_knob_two_layers() {
        use input::midi::Key;
        let mut app = test_app();
        let abs = app.pads.selected_abs();
        let pads = [(Key::Cc(K1), "pad.volume")];
        let edit = [(Key::Cc(K1), "edit.param_1")];
        // the edit is closed: the knob turns the volume to the top
        learned_pads(&mut app, &pads, &edit, knob(K1, 127));
        assert_eq!(app.pads.volume_db[abs], super::pads::MAX_DB);
        // the edit opens on the ENV page: the same knob is now the first slot, the attack
        xsession(&mut app, press(B7));
        xsession(&mut app, press(B1));
        assert_eq!(app.pads.page().0, "ENV");
        learned_pads(&mut app, &pads, &edit, knob(K1, 64));
        // the square curve: the middle of the knob is a quarter of 5 s
        let pos = 64.0f32 / 127.0;
        let ms = (pos * pos * super::pads::ENV_MS_LIMIT as f32).round() as u16;
        assert_eq!(app.pads.attack_ms[abs], ms);
        assert_eq!(
            app.pads.volume_db[abs],
            super::pads::MAX_DB,
            "the volume stays"
        );
    }

    /// 28 midi-learn: "Outside the edit" — a button learned only for "next parameter"
    /// in PADS · EDIT does nothing while no edit mode is open.
    #[test]
    fn outside_the_edit() {
        use super::pads::Param as P;
        use input::midi::Key;
        let mut app = test_app();
        let edit = [(Key::Note(B4), "edit.param_next")];
        learned_pads(&mut app, &[], &edit, press(B4));
        assert_eq!(app.pads.param, P::Volume, "no edit open: nothing happens");
        assert!(!app.edit);
        // with the edit open the same button picks the next parameter
        xsession(&mut app, press(B7));
        learned_pads(&mut app, &[], &edit, press(B4));
        assert_eq!(app.pads.param, P::Pitch);
    }

    /// 26 recording: "Start leaves the microphone alone", "A recording links the
    /// microphone" — the state that routing follows; the links themselves need the sound
    /// server and are checked live (26, task 2).
    #[test]
    fn a_recording_links_the_microphone() {
        let mut app = test_app();
        assert!(!app.mic_wanted(), "at start the microphone stays unlinked");
        app.recording = Some((super::audio::SOURCE_MIC, std::time::Instant::now()));
        assert!(app.mic_wanted(), "a microphone recording links it");
        app.recording = Some((super::audio::SOURCE_OUTPUT, std::time::Instant::now()));
        assert!(
            !app.mic_wanted(),
            "the output recording needs no microphone"
        );
        app.recording = None;
        assert!(
            !app.mic_wanted(),
            "after the recording it is unlinked again"
        );
    }

    /// 26 audio-settings: "Leaving the settings unlinks the microphone".
    #[test]
    fn leaving_the_settings_unlinks_the_microphone() {
        let mut app = test_app();
        app.apply(Action::ScreenSettings);
        assert!(
            app.mic_wanted(),
            "F12 shows the meter: the microphone is linked"
        );
        app.apply(Action::ScreenPads);
        assert!(!app.mic_wanted(), "back on the pads: unlinked");
    }

    /// 17 "Zoom knob": the k12 knob of the shipped profile spans the view from the
    /// whole file to a close-up, and grabbing the knob from another position does not
    /// jump the view.
    #[test]
    fn zoom_knob() {
        let mut app = test_app();
        // the scenario is about the catch: the response pickup (58)
        app.takeover.response = input::midi::Response::Pickup;
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(0));
        let view_len = |app: &super::App| {
            let c = app.chop.as_ref().unwrap();
            c.view.end - c.view.start
        };
        // the knob stands at 127 while the view is the whole file: the grab changes nothing
        xsession(&mut app, knob(K12, 127));
        assert_eq!(view_len(&app), 96_000, "the grab does not jump the view");
        // picked up at 0, the knob sweeps the log range to the close-up and back
        xsession(&mut app, knob(K12, 0));
        assert_eq!(view_len(&app), 96_000, "0 is the whole file");
        xsession(&mut app, knob(K12, 127));
        assert_eq!(view_len(&app), 256, "127 is the closest view");
        xsession(&mut app, knob(K12, 64));
        let mid = view_len(&app) as f32;
        // the knob position maps onto the log scale: len = 256 · (96000/256)^(1−pos)
        let span = (96_000f32 / 256.0).ln();
        let want = 256.0 * (span * (1.0 - 64.0 / 127.0)).exp();
        assert!(
            (mid - want).abs() < 4.0,
            "the knob follows the log scale: {mid} vs {want}"
        );
    }

    /// 17 "Region knobs": the learned edges follow with soft takeover, and the region
    /// never gets shorter than 10 ms.
    #[test]
    fn region_knobs() {
        let mut app = test_app();
        // the scenario is about the catch: the response pickup (58)
        app.takeover.response = input::midi::Response::Pickup;
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(0));
        let map = [
            (input::midi::Key::Cc(40), "chop.region_start_pos"),
            (input::midi::Key::Cc(41), "chop.region_end_pos"),
        ];
        // the start knob grabbed at 127 while the edge is at 0: nothing moves
        learned(&mut app, &map, knob(40, 127));
        assert_eq!(app.chop.as_ref().unwrap().range.start, 0, "no jump");
        // picked up at 0, the start follows to the middle of the file
        learned(&mut app, &map, knob(40, 0));
        learned(&mut app, &map, knob(40, 64));
        assert_eq!(app.chop.as_ref().unwrap().range.start, 48_378);
        // the end knob, picked up at its far end, hits the 10 ms floor (480 frames)
        learned(&mut app, &map, knob(41, 127));
        learned(&mut app, &map, knob(41, 1));
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.range.end - c.range.start, 480, "the 10 ms floor");
    }

    /// 17 "Cursor knob": the learned cursor knob walks the cursor inside the region,
    /// the slice under it becomes the selected one.
    #[test]
    fn cursor_knob() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(2)); // cuts at 24_000, 48_000, 72_000
        let map = [(input::midi::Key::Cc(42), "chop.cursor_pos")];
        learned(&mut app, &map, knob(42, 0));
        assert_eq!(app.chop.as_ref().unwrap().cursor, 0, "picked up at 0");
        learned(&mut app, &map, knob(42, 64));
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.cursor, 48_378, "the middle of the region");
        assert_eq!(c.selected, 2, "the slice under the cursor is selected");
        learned(&mut app, &map, knob(42, 127));
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.cursor, c.range.end, "the cursor stays inside the region");
        assert_eq!(c.selected, 3);
    }

    /// 17 "Cursor buttons": the learned "cursor right" button steps the cursor right,
    /// as `L` does.
    #[test]
    fn cursor_buttons() {
        let mut app = test_app();
        app.screen = input::Screen::Chop;
        app.chop = Some(chop_with_parts(2));
        let map = [(input::midi::Key::Note(60), "chop.cursor_right")];
        let step = app.chop.as_ref().unwrap().view_step(1);
        learned(&mut app, &map, press(60));
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.cursor, step, "one step right");
        assert_eq!(c.selected, c.slice_at(c.cursor), "the slice follows");
    }

    /// A pad with a 440 Hz tone at `peak`, ready for the chop screen.
    fn quiet_pad(app: &mut super::App, peak: f32) {
        let wave: Vec<f32> = (0..96_000)
            .map(|i| (i as f32 * 440.0 / 48_000.0 * std::f32::consts::TAU).sin() * peak)
            .collect();
        app.pads.sound[0] = Some(std::sync::Arc::new(Sample::mono(wave)));
        app.pads.file[0] = Some(std::path::PathBuf::from("break.wav"));
    }

    /// Opens the chop screen from the pads screen, as F3 does.
    fn open_chop_from_pads(app: &mut super::App) {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let f3 = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::F(3),
            ratatui::crossterm::event::KeyModifiers::empty(),
        );
        app.apply(keymap.resolve(input::Screen::Pads, &f3).unwrap());
    }

    /// 17 "Quiet break": the pad plays its quiet region 6 dB below full scale while the
    /// file behind is louder; `Ctrl+N` reads the peak of what the pad plays and puts the
    /// volume at +5 dB, the status shows it. The sound itself is the engine render
    /// (`render_the_normalized_pad`).
    #[test]
    fn quiet_break() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        quiet_pad(&mut app, 0.5);
        // the pad plays the quiet first quarter of a louder file
        app.pads.region[0] = Region {
            start: 0,
            end: 24_000,
        };
        open_chop_from_pads(&mut app);
        assert_eq!(app.chop_from, Some(0));
        chop_key(&mut app, KeyCode::Char('g'), M::CONTROL);
        assert_eq!(app.pads.volume_db[0], 5, "0.5 is −6 dB, so +5 dB");
        assert!(flash(&app).contains("+5 dB"), "{}", flash(&app));
        let gain = app
            .sent
            .iter()
            .find_map(|c| match c {
                Command::SetPadGain { pad: 0, gain } => Some(*gain),
                _ => None,
            })
            .expect("the engine hears the new gain");
        assert!((gain - 10f32.powf(0.25)).abs() < 1e-4, "{gain}");
    }

    /// 17 "Very quiet break": peaks 20 dB below full scale want more than the pad
    /// volume has; the +6 dB limit stops it and the status says so.
    #[test]
    fn very_quiet_break() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        quiet_pad(&mut app, 0.1);
        open_chop_from_pads(&mut app);
        chop_key(&mut app, KeyCode::Char('g'), M::CONTROL);
        assert_eq!(app.pads.volume_db[0], super::pads::MAX_DB, "the limit");
        assert_eq!(
            flash(&app),
            "normalized: +6 dB, the pad volume limit stopped it"
        );
    }

    /// 17 "Twice in a row": the gain is measured from the sound again, so the level
    /// does not climb twice.
    #[test]
    fn twice_in_a_row() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        quiet_pad(&mut app, 0.5);
        open_chop_from_pads(&mut app);
        chop_key(&mut app, KeyCode::Char('g'), M::CONTROL);
        let gains = |app: &super::App| {
            app.sent
                .iter()
                .filter(|c| matches!(c, Command::SetPadGain { .. }))
                .count()
        };
        let after_first = gains(&app);
        chop_key(&mut app, KeyCode::Char('g'), M::CONTROL);
        assert_eq!(app.pads.volume_db[0], 5, "the level did not climb");
        assert_eq!(
            gains(&app),
            after_first,
            "the same gain does not go to the engine twice"
        );
        assert!(flash(&app).contains("+5 dB"), "{}", flash(&app));
    }

    /// 17 "Chop without a pad": the chop screen opened from the files screen has no pad
    /// to set the volume on.
    #[test]
    fn chop_without_a_pad() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        quiet_pad(&mut app, 0.5);
        open_chop_from_pads(&mut app);
        assert_eq!(app.chop_from, Some(0));
        // the owner takes the same file to the chop screen from the files screen
        let sound = app.pads.sound[0].clone().unwrap();
        app.show_file_chop(std::path::PathBuf::from("break.wav"), sound);
        assert_eq!(app.chop_from, None);
        chop_key(&mut app, KeyCode::Char('g'), M::CONTROL);
        assert_eq!(flash(&app), "no pad to set the volume on");
        assert_eq!(app.pads.volume_db[0], 0, "nothing changed");
    }

    /// 17 "Quiet break", the sound side: the pad normalized by `Ctrl+N` renders with its
    /// peak at −1 dB and without clipping → `renders/17-normalize.wav`.
    #[test]
    fn render_the_normalized_pad() {
        use engine::Engine;
        use engine::checks::{first_non_finite, peak};
        use engine::offline::{Event, Scenario, render, write_wav};
        use std::path::Path;

        let sr = 48_000;
        let ms = |n: usize| sr as usize / 1000 * n;
        // the app side decides the gain: the quiet pad, Ctrl+G (83a)
        let mut app = test_app();
        quiet_pad(&mut app, 0.5);
        open_chop_from_pads(&mut app);
        chop_key(
            &mut app,
            ratatui::crossterm::event::KeyCode::Char('g'),
            ratatui::crossterm::event::KeyModifiers::CONTROL,
        );
        let db = app.pads.volume_db[0];
        let gain = 10f32.powf(db as f32 / 20.0);
        // the engine side: the same quiet pad with that gain, one hit at the full velocity
        let sound = std::sync::Arc::new(Sample::mono(
            (0..sr as usize * 2)
                .map(|i| (i as f32 * 440.0 * std::f32::consts::TAU / sr as f32).sin() * 0.5)
                .collect(),
        ));
        let sc = Scenario {
            sample_rate: sr,
            frames: ms(600),
            events: vec![
                Event {
                    frame: 0,
                    cmd: Command::SetPadSound {
                        pad: 0,
                        sound: Some(sound),
                    },
                },
                Event {
                    frame: 0,
                    cmd: Command::SetPadGain { pad: 0, gain },
                },
                Event {
                    frame: ms(100),
                    cmd: Command::PadPress {
                        pad: 0,
                        velocity: 1.0,
                    },
                },
            ],
        };
        let (l, r) = render(&mut Engine::new(sr), &sc, 128);
        assert_eq!(first_non_finite(&l), None, "no NaN or infinities");
        let p = peak(&l[ms(150)..]);
        assert!(p < 1.0, "no clipping: peak {p}");
        // the sample peak (≈0.4998) times the gain lands at −1 dB
        assert!((p - 0.5 * gain).abs() < 0.01, "peak {p} vs {}", 0.5 * gain);
        let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/17-normalize.wav");
        write_wav(&out, sr, &l, &r).unwrap();
    }

    /// 17 "Cursor knob" with soft takeover: one learned control drives the volume of pad
    /// 3 on the pads screen and the chop cursor on the chop screen; after the switch it
    /// catches up with the cursor instead of jumping it.
    #[test]
    fn a_knob_from_the_pads_catches_up_with_the_cursor() {
        let turn = |app: &mut super::App, param, value| {
            app.midi_event(super::midi::Event::Param {
                port: "X-Session".into(),
                control: "cc 50".into(),
                param,
                value,
                takeover: true,
            })
        };
        let mut app = test_app();
        // the scenario is about the catch: the response pickup (58)
        app.takeover.response = input::midi::Response::Pickup;
        app.pads.selected = 2;
        let at = app.pads.volume_pos();
        turn(&mut app, input::midi::Param::PadVolume, at);
        turn(&mut app, input::midi::Param::PadVolume, 0.9);
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(1));
        app.chop.as_mut().unwrap().set_cursor(0);
        turn(&mut app, input::midi::Param::Cursor, 0.9);
        assert_eq!(
            app.chop.as_ref().unwrap().cursor,
            0,
            "no jump: the knob must catch up"
        );
    }

    /// Slices 1–8 of a break on pads 5–12, as the assign lays them out, and pad 1 with
    /// the whole break.
    fn laid_out_break() -> super::App {
        let mut app = test_app();
        app.screen = Screen::Chop;
        let mut c = chop_with_parts(3); // 8 slices of break.wav
        for (i, slice) in c.slices().into_iter().enumerate() {
            app.pads.file[4 + i] = Some(c.file.clone());
            app.pads.region[4 + i] = slice;
            app.pads.link[4 + i] = Some((c.start_id(i), c.epoch));
        }
        app.pads.file[0] = Some(c.file.clone());
        c.select_slice(0);
        app.chop = Some(c);
        app.sync_links();
        app
    }

    /// The panel as the screen draws it now, the edit marks included.
    fn panel_now(app: &mut super::App) -> tui::Pads {
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        app.mark_pads(v)
    }

    fn fx_key(
        app: &mut super::App,
        code: ratatui::crossterm::event::KeyCode,
        modifiers: ratatui::crossterm::event::KeyModifiers,
    ) {
        screen_key(app, app.screen, code, modifiers);
    }

    /// Scenario "Open the shared FX screen" (keyboard).
    #[test]
    fn open_the_shared_fx_screen() {
        use ratatui::crossterm::event::{KeyCode as K, KeyModifiers as M};
        let mut app = test_app();
        app.pads.param = super::pads::Param::Send1;
        fx_key(&mut app, K::Enter, M::NONE);
        fx_key(&mut app, K::F(4), M::NONE);
        assert_eq!(app.screen, Screen::Fx);
        assert!(!app.edit);
        let v = app.fx_view();
        assert_eq!((v.selected, v.param), (0, 0));
        assert_eq!(v.blocks[0].rows[1].0, "size");
        assert_eq!(v.blocks[1].rows[1].0, "time");
        let (map, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let help = line(&super::help_groups(&map, app.screen, &app.help_state()));
        assert!(help.iter().any(|(_, t)| t.contains("FX 1 selected")));
        let full = super::full_help(&map, Screen::Fx, true);
        for text in [
            "shared FX screen",
            "shared FX block",
            "effect parameter",
            "picked value",
        ] {
            assert!(full.iter().any(|(_, t)| t.contains(text)), "{text}");
        }
        fx_key(&mut app, K::F(1), M::NONE);
        assert!(app.help_open);
        fx_key(&mut app, K::F(2), M::NONE);
        assert_eq!(app.screen, Screen::Pads);
        assert_eq!(app.pads.param, super::pads::Param::Send1);
    }

    #[test]
    fn shared_fx_screen_renders_the_emitted_controls() {
        use engine::Engine;
        use engine::checks::{first_loud, first_non_finite, peak};
        use engine::fx::FxSettings;
        use engine::offline::{Event, Scenario, render, write_wav};
        use ratatui::crossterm::event::{KeyCode as K, KeyModifiers as M};
        use std::path::Path;

        let sr = 48_000;
        let ms = |n: usize| sr as usize / 1000 * n;
        let mut phase = 0.0f32;
        let kick = (0..ms(300))
            .map(|i| {
                let t = i as f32 / sr as f32;
                phase += std::f32::consts::TAU * (50.0 + 100.0 * (-t * 30.0).exp()) / sr as f32;
                0.35 * phase.sin() * (-t * 9.0).exp()
            })
            .collect::<Vec<_>>();
        let tone = engine::test_tone(sr, 4);
        let sounds = vec![kick, tone];
        let reverb = |size, tone| Command::SetFx {
            slot: 0,
            settings: FxSettings::Reverb { size, tone },
        };
        let delay = |time_ms, feedback| Command::SetFx {
            slot: 1,
            settings: FxSettings::Delay { time_ms, feedback },
        };
        let hit = |pad| Command::PadPress { pad, velocity: 1.0 };
        let mut app = test_app();
        let mut events = Vec::new();
        let mut expected = Vec::new();
        let capture = |app: &mut super::App, events: &mut Vec<Event>, frame| {
            events.extend(
                std::mem::take(&mut app.sent)
                    .into_iter()
                    .map(|cmd| Event { frame, cmd }),
            );
        };
        for (pad, sends) in [(0, [0.65, 0.0]), (1, [0.0, 0.65])] {
            let cmd = Command::SetPadSends { pad, sends };
            app.send(cmd.clone());
            expected.push(Event { frame: 0, cmd });
        }
        fx_key(&mut app, K::F(4), M::NONE);
        assert_eq!(app.screen, Screen::Fx);
        page_knob(&mut app, 0, 0.0);
        page_knob(&mut app, 1, 0.7);
        page_knob(&mut app, 2, 0.3);
        capture(&mut app, &mut events, 0);
        expected.extend([reverb(0.7, 0.5), reverb(0.7, 0.3)].map(|cmd| Event { frame: 0, cmd }));

        fx_key(&mut app, K::Char('1'), M::SHIFT);
        capture(&mut app, &mut events, ms(100));
        expected.push(Event {
            frame: ms(100),
            cmd: hit(0),
        });
        app.apply(Action::PadRelease { pad: 0 });
        capture(&mut app, &mut events, ms(120));
        expected.push(Event {
            frame: ms(120),
            cmd: Command::PadRelease { pad: 0 },
        });

        fx_key(&mut app, K::Tab, M::NONE);
        assert_eq!(app.fx_slot, 1);
        assert!(app.sent.is_empty(), "slot selection sends no audio command");
        page_knob(&mut app, 0, 1.0);
        page_knob(&mut app, 1, 0.4);
        page_knob(&mut app, 2, 0.4);
        capture(&mut app, &mut events, ms(1000));
        expected.extend([delay(328.0, 0.4), delay(328.0, 0.36)].map(|cmd| Event {
            frame: ms(1000),
            cmd,
        }));

        for (at, key, commands) in [
            (1100, None, Vec::new()),
            (2300, Some(K::Char('l')), vec![delay(338.0, 0.36)]),
            (
                3500,
                Some(K::Char('z')),
                vec![reverb(0.7, 0.3), delay(328.0, 0.36)],
            ),
        ] {
            if let Some(key) = key {
                if key == K::Char('l') {
                    fx_key(&mut app, K::Char('k'), M::NONE);
                    fx_key(&mut app, key, M::SHIFT);
                } else {
                    fx_key(&mut app, key, M::CONTROL);
                }
                capture(&mut app, &mut events, ms(at - 100));
                expected.extend(commands.into_iter().map(|cmd| Event {
                    frame: ms(at - 100),
                    cmd,
                }));
            }
            fx_key(&mut app, K::Char('2'), M::SHIFT);
            capture(&mut app, &mut events, ms(at));
            expected.push(Event {
                frame: ms(at),
                cmd: hit(1),
            });
            app.apply(Action::PadRelease { pad: 1 });
            capture(&mut app, &mut events, ms(at + 20));
            expected.push(Event {
                frame: ms(at + 20),
                cmd: Command::PadRelease { pad: 1 },
            });
        }
        assert_eq!(app.pads.fx[1].time_ms, 328, "undo restores the delay time");
        fx_key(&mut app, K::BackTab, M::SHIFT);
        assert_eq!(app.fx_slot, 0);
        assert!(
            app.sent.is_empty(),
            "returning to FX 1 sends no audio command"
        );
        page_knob(&mut app, 1, 0.9);
        capture(&mut app, &mut events, ms(4600));
        expected.push(Event {
            frame: ms(4600),
            cmd: reverb(0.9, 0.3),
        });
        fx_key(&mut app, K::Char('1'), M::SHIFT);
        capture(&mut app, &mut events, ms(4700));
        expected.push(Event {
            frame: ms(4700),
            cmd: hit(0),
        });
        app.apply(Action::PadRelease { pad: 0 });
        capture(&mut app, &mut events, ms(4720));
        expected.push(Event {
            frame: ms(4720),
            cmd: Command::PadRelease { pad: 0 },
        });

        let frames = ms(7500);
        let run = |events| {
            render(
                &mut Engine::with_sounds(sr, Vec::new(), sounds.clone()),
                &Scenario {
                    sample_rate: sr,
                    frames,
                    events,
                },
                128,
            )
        };
        let (left, right) = run(events
            .iter()
            .map(|e| Event {
                frame: e.frame,
                cmd: e.cmd.clone(),
            })
            .collect());
        let (want_left, want_right) = run(expected);
        for (channel, want) in [(&left, &want_left), (&right, &want_right)] {
            assert_eq!(first_non_finite(channel), None, "no NaN or infinity");
            assert!(peak(channel) < 1.0, "no clipping");
            assert_eq!(first_loud(channel, 0..ms(100), 0.0), None);
            for (frame, (got, want)) in channel.iter().zip(want).enumerate() {
                assert_eq!(got, want, "UI and direct controls differ at frame {frame}");
            }
        }
        assert!(
            peak(&left[ms(450)..ms(900)]) > 1e-4,
            "the reverb has a tail"
        );
        assert!(
            peak(&left[ms(2100)..ms(2200)]) > 1e-4,
            "the delay has a tail"
        );
        for at in [100, 1100, 2300, 3500, 4700] {
            let frame = ms(at);
            let without_hit = events
                .iter()
                .filter(|e| !(e.frame == frame && matches!(e.cmd, Command::PadPress { .. })))
                .map(|e| Event {
                    frame: e.frame,
                    cmd: e.cmd.clone(),
                })
                .collect();
            let (without_left, without_right) = run(without_hit);
            for (channel, without) in [(&left, &without_left), (&right, &without_right)] {
                assert!(
                    channel[..frame]
                        .iter()
                        .zip(&without[..frame])
                        .all(|(a, b)| a == b),
                    "no early hit at {at} ms"
                );
                let difference = channel[frame..frame + 3]
                    .iter()
                    .zip(&without[frame..frame + 3])
                    .map(|(got, without)| got - without)
                    .collect::<Vec<_>>();
                let onset = first_loud(&difference, 0..difference.len(), 1e-7)
                    .expect("the pad hit must reach the output");
                assert!(onset <= 1, "hit at {at} ms is {onset} samples late");
            }
        }
        let out =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/82-shared-fx-screen.wav");
        write_wav(&out, sr, &left, &right).unwrap();
    }

    /// Scenario "Select the next shared effect" (pad-playback).
    #[test]
    fn select_the_next_shared_effect() {
        use ratatui::crossterm::event::{KeyCode as K, KeyModifiers as M};
        let mut app = test_app();
        fx_key(&mut app, K::F(4), M::NONE);
        let before = app.pads.fx;
        app.sent.clear();
        fx_key(&mut app, K::Tab, M::NONE);
        assert_eq!(app.fx_view().selected, 1);
        assert_eq!(app.help_state().page, "FX 2");
        fx_key(&mut app, K::BackTab, M::SHIFT);
        assert_eq!(app.fx_view().selected, 0);
        assert_eq!(app.pads.fx, before);
        assert!(app.sent.is_empty());
        assert!(app.history.is_empty());
        assert!(app.session_dirty.is_none());
    }

    /// Scenario "The same knobs control the selected effect" (pad-playback).
    #[test]
    fn the_same_knobs_control_the_selected_effect() {
        use ratatui::crossterm::event::{KeyCode as K, KeyModifiers as M};
        let mut app = test_app();
        let pads = app.before().pads;
        fx_key(&mut app, K::F(4), M::NONE);
        page_knob(&mut app, 0, 0.8);
        page_knob(&mut app, 1, 0.8);
        page_knob(&mut app, 2, 0.3);
        let one = app.pads.fx[0];
        assert_eq!(one.kind, engine::fx::FxKind::Delay);
        assert!(one.time_ms > 250);
        assert_eq!(one.feedback, 27);
        fx_key(&mut app, K::Tab, M::NONE);
        page_knob(&mut app, 0, 0.1);
        page_knob(&mut app, 1, 0.7);
        page_knob(&mut app, 2, 0.2);
        assert_eq!(app.pads.fx[0], one);
        assert_eq!(app.pads.fx[1].kind, engine::fx::FxKind::Reverb);
        assert_eq!((app.pads.fx[1].size, app.pads.fx[1].tone), (70, 20));
        assert_eq!(app.before().pads, pads);
        assert!(app.session_dirty.is_some());
        assert_eq!(app.pads.param, super::pads::Param::Volume);
        // The page selector acts without an edit mode or pickup and keeps the pick
        // when its position stays in the same half.
        fx_key(&mut app, K::Char('k'), M::NONE);
        for value in [0.9, 0.0] {
            app.midi_event(super::midi::Event::Param {
                port: "X".into(),
                control: "page".into(),
                param: input::midi::Param::Page,
                value,
                takeover: true,
            });
            assert_eq!(app.fx_slot, usize::from(value >= 0.5));
            assert_eq!(app.fx_param, usize::from(value >= 0.5));
        }
        let before = app.pads.fx;
        page_knob(&mut app, 3, 1.0);
        assert_eq!(app.pads.fx, before, "there is no fourth parameter");
    }

    /// Scenario "Takeover is separate for shared effects" (pad-playback).
    #[test]
    fn takeover_is_separate_for_shared_effects() {
        use input::midi::{Param as P, Response};
        let turn = |app: &mut super::App, port: &str, param, value| {
            app.midi_event(super::midi::Event::Param {
                port: port.into(),
                control: "knob".into(),
                param,
                value,
                takeover: true,
            });
        };
        let mut app = test_app();
        app.apply(Action::ScreenFx);
        app.takeover.response = Response::Pickup;
        let current = app.pads.fx_pos(super::pads::Param::Fx1A);
        turn(&mut app, "A", P::Slot(1), current);
        turn(&mut app, "A", P::Slot(1), 0.1);
        assert_eq!(app.pads.fx[0].size, 10);
        app.apply(Action::PageNext);
        let before = app.pads.fx;
        turn(&mut app, "A", P::Slot(1), 0.95);
        assert_eq!(app.pads.fx, before);
        assert!(app.fx_view().pickup.is_some());
        let current = app.pads.fx_pos(super::pads::Param::Fx2A);
        turn(&mut app, "A", P::Slot(1), current);
        turn(&mut app, "A", P::Slot(1), 0.6);
        assert_ne!(app.pads.fx[1], before[1]);
        let caught = app.pads.fx;
        turn(&mut app, "B", P::Slot(1), 0.95);
        assert_eq!(app.pads.fx, caught, "another device must catch its value");
        turn(&mut app, "A", P::Slot(2), 0.95);
        assert_eq!(
            app.pads.fx, caught,
            "another parameter must catch its value"
        );
        // The same control must catch again when time becomes size.
        turn(&mut app, "A", P::Slot(1), 0.6);
        page_knob(&mut app, 0, 0.0);
        let changed = app.pads.fx;
        turn(&mut app, "A", P::Slot(1), 0.95);
        assert_eq!(
            app.pads.fx, changed,
            "a different effect type is a new target"
        );
    }

    #[test]
    fn fx_scale_and_jump_follow_the_selected_slot() {
        use input::midi::{Param, Response};
        let turn = |app: &mut super::App, value| {
            app.midi_event(super::midi::Event::Param {
                port: "A".into(),
                control: "k2".into(),
                param: Param::Slot(1),
                value,
                takeover: true,
            });
        };
        for response in [Response::Scale, Response::Jump] {
            let mut app = test_app();
            app.apply(Action::ScreenFx);
            app.takeover.response = response;
            turn(&mut app, 0.1);
            turn(&mut app, 0.2);
            let one = app.pads.fx[0];
            app.apply(Action::PageNext);
            let two = app.pads.fx[1];
            turn(&mut app, 0.9);
            if response == Response::Scale {
                assert_eq!(app.pads.fx[1], two, "new target starts from its own value");
            } else {
                assert_ne!(app.pads.fx[1], two, "jump applies the first position");
            }
            turn(&mut app, 1.0);
            assert_eq!(app.pads.fx[1].time_ms, 2_000);
            assert_eq!(app.pads.fx[0], one);
        }
    }

    #[test]
    fn fx_keyboard_values_and_learn_use_the_fx_selection() {
        use ratatui::crossterm::event::{KeyCode as K, KeyModifiers as M};
        let mut app = test_app();
        app.pads.param = super::pads::Param::Send2;
        fx_key(&mut app, K::F(4), M::NONE);
        fx_key(&mut app, K::Tab, M::NONE);
        fx_key(&mut app, K::Char('k'), M::NONE);
        let time = app.pads.fx[1].time_ms;
        fx_key(&mut app, K::Char('l'), M::SHIFT);
        assert_eq!(app.pads.fx[1].time_ms, time + 10);
        fx_key(&mut app, K::Char('j'), M::SHIFT);
        assert_eq!(app.pads.fx[1].time_ms, time);
        fx_key(&mut app, K::F(10), M::NONE);
        assert_eq!(app.learn_group(), input::Group::Fx);
        assert_eq!(picked(&app), ("edit.param_2", "time".into()));
        assert_eq!(app.fx_view().lit, Some(1));
        assert_eq!(app.pads.param, super::pads::Param::Send2);
    }

    /// Scenario "Undo a shared effect on the FX screen" (pad-playback): keyboard,
    /// MIDI, distinct slot undo steps, and project save/reload use the same settings.
    #[test]
    fn undo_a_shared_effect_on_the_fx_screen() {
        use ratatui::crossterm::event::{KeyCode as K, KeyModifiers as M};
        let mut app = test_app();
        fx_key(&mut app, K::F(4), M::NONE);
        let original = app.pads.fx;
        // Russian physical K, L and I select and change the same parameters.
        fx_key(&mut app, K::Char('л'), M::NONE);
        fx_key(&mut app, K::Char('д'), M::NONE);
        assert_eq!(app.fx_param, 1);
        assert_ne!(app.pads.fx, original);
        fx_key(&mut app, K::Char('ш'), M::NONE);
        assert_eq!(app.fx_param, 0);
        fx_key(&mut app, K::Char('я'), M::CONTROL);
        assert_eq!(app.pads.fx, original);
        page_knob(&mut app, 2, 0.2);
        let first = app.pads.fx;
        fx_key(&mut app, K::Tab, M::NONE);
        page_knob(&mut app, 2, 0.8);
        assert_eq!(
            app.history.len(),
            2,
            "different slots do not merge undo steps"
        );
        fx_key(&mut app, K::Char('z'), M::CONTROL);
        assert_eq!(app.pads.fx, first);
        let dir = tmp_dir("fx-reload");
        std::fs::create_dir_all(&dir).unwrap();
        app.session_path = dir.join("project.toml");
        app.session_read_error = None;
        app.save_session().unwrap();
        let mut restored = test_app();
        restored.apply_session(super::session::load(&app.session_path).unwrap());
        assert_eq!(restored.pads.fx, first);
        fx_key(&mut app, K::Char('z'), M::CONTROL);
        assert_eq!(app.pads.fx, original);
        app.save_session().unwrap();
        restored.apply_session(super::session::load(&app.session_path).unwrap());
        assert_eq!(restored.pads.fx, original);
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::SetFx { slot: 1, .. }))
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// Scenario "Play pads on the FX screen" (keyboard): hits and releases retain
    /// their bank, velocity and shared effect selection in either keyboard layout.
    #[test]
    fn play_pads_on_the_fx_screen() {
        use ratatui::crossterm::event::{KeyCode as K, KeyEvent, KeyEventKind, KeyModifiers as M};
        let mut app = test_app();
        let (map, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        fx_key(&mut app, K::F(4), M::NONE);
        fx_key(&mut app, K::Tab, M::NONE);
        fx_key(&mut app, K::Char('k'), M::NONE);
        app.apply(Action::BankSelect(2));
        app.sent.clear();
        for c in ['q', 'й'] {
            fx_key(&mut app, K::Char(c), M::SHIFT);
            let release = KeyEvent::new_with_kind(K::Char(c), M::NONE, KeyEventKind::Release);
            app.apply(map.resolve(app.screen, &release).unwrap());
        }
        assert_eq!((app.fx_slot, app.fx_param), (1, 1));
        assert_eq!(app.pads.param, super::pads::Param::Volume);
        assert_eq!(
            app.sent
                .iter()
                .filter(|c| matches!(c,
            Command::PadPress { pad: 36, velocity } if *velocity == 1.0))
                .count(),
            2
        );
        assert_eq!(
            app.sent
                .iter()
                .filter(|c| matches!(c, Command::PadRelease { pad: 36 }))
                .count(),
            2
        );
        app.midi_event(super::midi::Event::PadHit {
            abs: 37,
            sound: super::midi::PadSound::DirectAccepted,
            velocity: 1.0,
            ui_press: None,
        });
        assert_eq!(app.pads.selected_abs(), 37);
        assert_eq!((app.fx_slot, app.fx_param), (1, 1));
    }

    #[test]
    fn clear_a_shared_effect_knob_assignment() {
        use input::midi::{Param as P, Resolved};
        use ratatui::crossterm::event::KeyCode as K;
        for reassign in [false, true] {
            let dir = tmp_dir(if reassign {
                "fx-reassign-clear"
            } else {
                "fx-clear"
            });
            std::fs::create_dir_all(&dir).unwrap();
            let cfg = dir.join("config.toml");
            std::fs::write(&cfg, "[devices]\nseeded = [\"default\"]\n[midi.default.chop_edit]\n\"cc 24\" = \"edit.param_1\"\n[midi.default.pads_edit]\n\"cc 80\" = \"edit.param_1\"\n").unwrap();
            let mut app = test_app();
            app.user_path = Some(cfg.clone());
            let loaded = super::config::load_from(&cfg).config;
            app.learned = loaded.midi;
            app.seeded = loaded.devices.seeded;
            app.migrate_fx_bindings();
            app.midi_devices.push(("X".into(), "default".into()));
            app.apply(Action::ScreenFx);
            assert!(matches!(
                resolved(&app, Screen::Fx, false, knob(80, 127)),
                Some(Resolved::Param {
                    param: P::Slot(0),
                    ..
                })
            ));
            assert_eq!(resolved(&app, Screen::Fx, false, knob(24, 127)), None);
            app.apply(Action::LearnToggle);
            app.learn_pick("edit.param_1", "effect type".into());
            if reassign {
                turn_knob(&mut app, 81);
                learn_press(&mut app, K::Enter);
                app.learn_pick("edit.param_1", "effect type".into());
                assert_eq!(resolved(&app, Screen::Fx, false, knob(80, 127)), None);
            }
            learn_press(&mut app, K::Backspace);
            learn_press(&mut app, K::Enter);
            learn_press(&mut app, K::Esc);
            assert!(app.learned["default"]["fx"].is_empty());
            for cc in [24, 80, 81] {
                assert_eq!(resolved(&app, Screen::Fx, false, knob(cc, 127)), None);
            }
            let mut again = test_app();
            again.user_path = Some(cfg.clone());
            let loaded = super::config::load_from(&cfg).config;
            again.learned = loaded.midi;
            again.seeded = loaded.devices.seeded;
            assert_eq!(again.drop_dead_bindings(), None);
            again.migrate_fx_bindings();
            again.midi_event(super::midi::Event::Connected {
                port: "X".into(),
                profile: "default".into(),
            });
            assert!(again.learned["default"]["fx"].is_empty());
            for cc in [24, 80, 81] {
                assert_eq!(resolved(&again, Screen::Fx, false, knob(cc, 127)), None);
            }
            assert_eq!(
                again.live_profiles()[0].assignment(input::Group::Fx.scope(), "edit.param_1"),
                None
            );
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn global_learned_controls_change_and_clear_on_fx() {
        use input::midi::{Key, Param as P, Resolved};
        let mut app = xsession_connected(&[]);
        let states = [
            (Screen::Pads, false),
            (Screen::Pads, true),
            (Screen::Files, false),
            (Screen::Chop, false),
            (Screen::Chop, true),
            (Screen::Fx, false),
            (Screen::Settings, false),
        ];
        for (name, old_cc, new_cc, param) in [
            ("pad.volume", 11, 81, P::PadVolume),
            ("pad.pitch", 12, 82, P::PadPitch),
            ("master.volume", 14, 83, P::MasterVolume),
        ] {
            super::learn::assign(
                &mut app.learned,
                "X-Session Pro",
                input::Group::Global,
                Key::Cc(new_cc),
                name,
            );
            app.sync_learned("X-Session Pro", input::Group::Global);
            for (screen, edit) in states {
                app.screen = screen;
                app.edit = edit;
                assert_eq!(
                    xs_raw(&mut app, &[0xB0, old_cc, 0]),
                    None,
                    "{name}: {screen:?}"
                );
                assert!(matches!(xs_raw(&mut app, &[0xB0, new_cc, 0]),
                    Some(Resolved::Param { param: actual, .. }) if actual == param));
                match param {
                    P::PadVolume => assert_eq!(
                        app.pads.volume_db[app.pads.selected_abs()],
                        super::pads::MIN_DB
                    ),
                    P::PadPitch => assert_eq!(app.pads.pitch[app.pads.selected_abs()], -24),
                    P::MasterVolume => assert_eq!(app.master_db, -40),
                    _ => unreachable!(),
                }
            }
            super::learn::clear(&mut app.learned, input::Group::Global, name);
            app.sync_learned("X-Session Pro", input::Group::Global);
            for (screen, edit) in states {
                app.screen = screen;
                app.edit = edit;
                for cc in [old_cc, new_cc] {
                    assert_eq!(
                        xs_raw(&mut app, &[0xB0, cc, 127]),
                        None,
                        "{name}: {screen:?}"
                    );
                }
            }
        }
        learn_button(&mut app, input::Group::Global, "bank.next", 90);
        for (screen, edit) in states {
            app.screen = screen;
            app.edit = edit;
            app.pads.bank = 0;
            assert_eq!(
                xs_raw(&mut app, &[0x90, 57, 127]),
                None,
                "old bank control: {screen:?}"
            );
            assert_eq!(app.pads.bank, 0);
            assert_eq!(
                xs_raw(&mut app, &[0x90, 90, 127]),
                Some(Resolved::Action(Action::BankNext)),
                "new bank control: {screen:?}",
            );
            assert_eq!(app.pads.bank, usize::from(screen != Screen::Settings));
        }
        learn_row(&mut app, input::Group::Global, "bank.next");
        app.apply(Action::Back);
        app.apply(Action::SettingsClear);
        for (screen, edit) in states {
            app.screen = screen;
            app.edit = edit;
            for note in [57, 90] {
                assert_eq!(
                    xs_raw(&mut app, &[0x90, note, 127]),
                    None,
                    "cleared bank control: {screen:?}"
                );
            }
        }
        learn_button(&mut app, input::Group::Global, "screen.files", 91);
        for (screen, edit) in states {
            app.screen = screen;
            app.edit = edit;
            assert_eq!(
                xs_raw(&mut app, &[0x90, 44, 127]),
                None,
                "old Files control: {screen:?}"
            );
            xs_button(&mut app, 91);
            assert_eq!(app.screen, Screen::Files, "new Files control: {screen:?}");
        }
        learn_row(&mut app, input::Group::Global, "screen.files");
        app.apply(Action::Back);
        app.apply(Action::SettingsClear);
        for (screen, edit) in states {
            app.screen = screen;
            app.edit = edit;
            for note in [44, 91] {
                assert_eq!(
                    xs_raw(&mut app, &[0x90, note, 127]),
                    None,
                    "cleared Files control: {screen:?}"
                );
            }
        }
        let fx = &app.learned["X-Session Pro"]["fx"];
        assert!(!fx.values().any(|action| matches!(
            action.as_str(),
            "pad.volume" | "pad.pitch" | "master.volume" | "bank.next" | "screen.files"
        )));
    }

    #[test]
    fn replaced_page_knobs_stay_replaced() {
        use input::midi::{Key, Param as P, Resolved};
        for seeded in [false, true] {
            let mut app = test_app();
            app.profiles.push(
                input::midi::Profile::from_toml(
                    "xsession-pro.toml",
                    include_str!("../../../config/devices/xsession-pro.toml"),
                )
                .unwrap(),
            );
            super::learn::assign(
                &mut app.learned,
                "X-Session Pro",
                input::Group::PadsEdit,
                Key::Cc(80),
                "edit.param_1",
            );
            if seeded {
                app.seeded.push("X-Session Pro".into());
            }
            app.midi_event(super::midi::Event::Connected {
                port: "X".into(),
                profile: "X-Session Pro".into(),
            });
            assert_eq!(
                app.learned["X-Session Pro"]["fx"]
                    .get("cc 80")
                    .map(String::as_str),
                Some("edit.param_1")
            );
            assert!(matches!(
                resolved(&app, Screen::Fx, false, knob(80, 127)),
                Some(Resolved::Param {
                    param: P::Slot(0),
                    ..
                })
            ));
            assert_eq!(resolved(&app, Screen::Fx, false, knob(24, 127)), None);
            assert_eq!(
                app.live_profiles()[0].assignment(input::Group::Fx.scope(), "edit.param_1"),
                Some("cc 80".into())
            );
            assert!(app.seeded.iter().any(|name| name == "X-Session Pro"));
        }
    }

    #[test]
    fn fx_learned_page_controls_and_old_bindings_keep_their_meaning() {
        use input::midi::{Key, Param as P, Resolved};
        use ratatui::crossterm::event::KeyCode as K;
        let mut app = learning_on(Screen::Fx, false);
        assert_eq!(app.learn_group(), input::Group::Fx);
        super::learn::assign(
            &mut app.learned,
            "default",
            input::Group::PadsEdit,
            Key::Cc(90),
            "pad.fx1_size",
        );
        for (i, cc) in [80, 81, 82, 83].into_iter().enumerate() {
            if i == 0 {
                learn_press(&mut app, K::Char('k'));
            }
            assert_eq!(picked(&app).0, app.learn_params()[i].0);
            turn_knob(&mut app, cc);
            learn_press(&mut app, K::Enter);
        }
        assert!(app.learned["default"]["fx"].contains_key("cc 80"));
        learn_press(&mut app, K::Esc);
        for (cc, expected) in [
            (80, P::Slot(0)),
            (81, P::Slot(1)),
            (82, P::Slot(2)),
            (83, P::Page),
        ] {
            let r = resolved(&app, Screen::Fx, false, knob(cc, 127));
            assert!(matches!(r, Some(Resolved::Param { param, .. }) if param == expected));
            feed_resolved(&mut app, r);
        }
        assert_eq!(app.fx_slot, 1);
        // An older direct binding continues to address FX 1 on Pads and on FX.
        // The legacy action names are resolved by the profile, not by the FX pick.
        let old = app.live_profiles()[0].resolve_in(Screen::Fx, false, knob(90, 20));
        assert!(matches!(
            old,
            Some(Resolved::Param {
                param: P::PadEffect(4),
                ..
            })
        ));
        feed_resolved(&mut app, old);
        assert!((app.pads.fx_pos(super::pads::Param::Fx1A) - 20.0 / 127.0).abs() < 0.01);
        app.apply(Action::ScreenPads);
        let old = app.live_profiles()[0].resolve_in(Screen::Pads, true, knob(90, 90));
        feed_resolved(&mut app, old);
        assert!((app.pads.fx_pos(super::pads::Param::Fx1A) - 90.0 / 127.0).abs() < 0.01);
        assert_eq!(app.pads.param, super::pads::Param::Volume);
        app.apply(Action::ScreenFx);
        app.apply(Action::ScreenSettings);
        assert_eq!(app.midi_open, vec![input::Group::Fx]);
    }

    /// Turns knob k of the edit page to `value`.
    fn page_knob(app: &mut super::App, k: u8, value: f32) {
        app.midi_event(super::midi::Event::Param {
            port: "X-Session".into(),
            control: format!("k{}", k + 1),
            param: input::midi::Param::Slot(k),
            value,
            takeover: false,
        });
    }

    /// 56 "The FX page": knob 1 the drive, knob 2 send 1, knob 3 send 2; each send line
    /// names the kind of its slot; the drive reaches the engine.
    #[test]
    fn the_fx_page() {
        use super::pads::Param;
        let mut app = test_app();
        app.apply(Action::PadEdit);
        while app.pads.page().0 != "FX" {
            app.apply(Action::PageNext);
        }
        app.sent.clear();
        page_knob(&mut app, 0, 0.4);
        page_knob(&mut app, 1, 0.2);
        page_knob(&mut app, 2, 0.5);
        let abs = app.pads.selected_abs();
        assert_eq!((app.pads.drive[abs], app.pads.sends[abs]), (40, [20, 50]));
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::SetPadDrive { .. }))
        );
        let v = panel_now(&mut app);
        let row = |w| v.fx.iter().find(|r| r.0 == w).map(|r| r.2.clone());
        assert_eq!(row(tui::PadParam::Send1).as_deref(), Some("20 %  reverb"));
        assert_eq!(row(tui::PadParam::Send2).as_deref(), Some("50 %  delay"));
        assert_eq!(app.pads.param, Param::Drive, "the FX page picks the drive");
        assert!(v.slot.is_none(), "no slot block on the FX page");
    }

    /// 82 "A slot page": the shared FX screen edits a slot, not a pad.
    #[test]
    fn a_slot_page() {
        use engine::fx::FxKind;
        let mut app = test_app();
        app.apply(Action::ScreenFx);
        app.apply(Action::PageNext);
        assert_eq!(app.pads.fx[1].kind, FxKind::Delay);
        app.sent.clear();
        page_knob(&mut app, 0, 0.1);
        assert_eq!(app.pads.fx[1].kind, FxKind::Reverb);
        page_knob(&mut app, 1, 0.8);
        page_knob(&mut app, 2, 0.3);
        assert_eq!((app.pads.fx[1].size, app.pads.fx[1].tone), (80, 30));
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::SetFx { slot: 1, .. }))
        );
        let v = app.fx_view();
        assert_eq!(v.selected, 1);
        assert_eq!(v.blocks[1].rows[1].0, "size");
        // the slot keeps the delay's own values for when it comes back
        assert_eq!(app.pads.fx[1].time_ms, 250);
        app.apply(Action::ScreenPads);
        assert_eq!(app.pads.page().0, "TONE");
        assert!(panel_now(&mut app).slot.is_none());
    }

    /// 56 "Marked pads together": Q and W marked, send 1 on Q to 30 % moves W too; a slot
    /// page changes no pad.
    #[test]
    fn marked_pads_together() {
        let mut app = test_app();
        let (q, w) = (4, 5);
        app.pads.marked.insert(q);
        app.pads.marked.insert(w);
        app.pads.selected = q;
        app.apply(Action::PadEdit);
        while app.pads.page().0 != "FX" {
            app.apply(Action::PageNext);
        }
        page_knob(&mut app, 1, 0.3);
        assert_eq!((app.pads.sends[q][0], app.pads.sends[w][0]), (30, 30));
        let before = (app.pads.drive, app.pads.sends);
        app.apply(Action::ScreenFx);
        page_knob(&mut app, 1, 0.9);
        assert_eq!(
            (app.pads.drive, app.pads.sends),
            before,
            "a slot page changes no pad"
        );
    }

    /// 77 "A send marks the pad": a send above 0 or the drive gives the one mark `✱`.
    #[test]
    fn a_send_marks_the_pad() {
        let mut app = laid_out_break();
        app.pads.sends[0] = [0, 20];
        assert_eq!(glyphs(&mut app, 0), "✱");
        app.pads.sends[0] = [0, 0];
        app.pads.drive[0] = 10;
        assert_eq!(glyphs(&mut app, 0), "✱", "the drive too");
    }

    /// A pads-screen app with the X-Session knob 1 bound to the pad volume, for the undo.
    fn undo_app() -> super::App {
        let mut app = test_app();
        app.screen = Screen::Pads;
        app
    }

    /// Pad 7 (slice 3 of the break) with settings on it, its edit open.
    fn dressed_pad() -> (super::App, usize) {
        use engine::filter::FilterKind;
        let mut app = laid_out_break();
        app.screen = Screen::Pads;
        app.chop = None;
        let abs = 6;
        app.pads.selected = abs;
        app.pads.pitch[abs] = -5;
        app.pads.filter[abs] = FilterKind::LowPass;
        app.pads.volume_db[abs] = -6;
        app.pads.gate[abs] = true;
        app.pads.group[abs] = 2;
        app.apply(Action::PadEdit);
        (app, abs)
    }

    /// 76 pad-playback "A pad back to clean".
    #[test]
    fn a_pad_back_to_clean() {
        use engine::filter::FilterKind;
        let (mut app, abs) = dressed_pad();
        let source = app.pads.source(abs);
        let region = app.pads.region[abs];
        app.apply(Action::PadClear);
        app.apply(Action::PadEdit); // Enter answers the question
        assert!(app.ask.is_none());
        assert_eq!(app.pads.pitch[abs], 0);
        assert_eq!(app.pads.filter[abs], FilterKind::Off);
        assert_eq!(app.pads.volume_db[abs], 0);
        assert!(!app.pads.gate[abs], "one-shot");
        assert_eq!(app.pads.group[abs], 0);
        assert_eq!(app.pads.source(abs), source, "the slice stays");
        assert_eq!(app.pads.region[abs], region);
        assert!(app.pads.file[abs].is_some());
        assert!(app.edit, "the edit stays open");
        assert_eq!(glyphs(&mut app, abs), "", "no mark: a slice is no setting");
    }

    /// 76 pad-playback "The question names the pads" and "Esc keeps".
    #[test]
    fn the_question_names_the_pads() {
        let mut app = undo_app();
        app.apply(Action::PadMarkToggle);
        for abs in 0..17 {
            app.pads.file[abs] = Some(format!("/m/{abs}.wav").into());
            app.pads.pitch[abs] = 3;
            app.pads.marked.insert(abs);
        }
        app.pads.marked.insert(40); // an empty pad is skipped
        app.apply(Action::PadEdit);
        app.apply(Action::PadClear);
        let lines = app.ask.as_ref().expect("a question").lines().to_vec();
        assert_eq!(lines[0], "Reset 17 pads to clean?");
        assert_eq!(lines.last().unwrap(), "Enter: reset · Esc: keep");
        assert!(
            (0..17).all(|a| app.pads.pitch[a] == 3),
            "no pad has changed yet"
        );
        // Esc keeps
        app.apply(Action::Back);
        assert!(app.ask.is_none());
        assert!(app.edit, "the edit stays open");
        assert!((0..17).all(|a| app.pads.pitch[a] == 3));
        assert_eq!(app.pads.marked.len(), 18, "the marks stay");
    }

    /// 76 pad-playback "Undo a reset".
    #[test]
    fn undo_a_reset() {
        let mut app = undo_app();
        for abs in 0..16 {
            app.pads.file[abs] = Some(format!("/m/{abs}.wav").into());
            app.pads.pitch[abs] = abs as i8 - 8;
            app.pads.volume_db[abs] = -(abs as i8);
        }
        let before: Vec<_> = (0..16).map(|a| app.pads.snapshot(a)).collect();
        app.apply(Action::PadMarkBank);
        app.apply(Action::PadEdit);
        app.apply(Action::PadClear);
        app.apply(Action::PadEdit);
        assert!((0..16).all(|a| app.pads.pitch[a] == 0 && app.pads.volume_db[a] == 0));
        let said = undo(&mut app);
        assert_eq!(said, "undo: reset on 16 pads");
        let after: Vec<_> = (0..16).map(|a| app.pads.snapshot(a)).collect();
        assert_eq!(after, before, "every pad has its own settings back");
    }

    /// 76 pad-playback "Backspace outside the edit": only the selected pad is cleared,
    /// nothing asks.
    #[test]
    fn backspace_outside_the_edit() {
        let mut app = undo_app();
        app.pads.file[0] = Some("/m/kick.wav".into());
        app.pads.file[1] = Some("/m/snare.wav".into());
        app.apply(Action::PadClear);
        assert!(app.ask.is_none());
        assert!(app.pads.file[0].is_none());
        assert!(app.pads.file[1].is_some());
    }

    /// 76 pad-playback "Clear the marked pads": a question first, then Q W E are empty;
    /// one Ctrl+Z brings the three sounds back.
    #[test]
    fn clear_the_marked_pads() {
        let mut app = undo_app();
        for abs in [4, 5, 6, 7] {
            app.pads.file[abs] = Some(format!("/m/{abs}.wav").into());
        }
        app.pads.marked.extend([4, 5, 6, 12]); // pad Z is empty: skipped
        app.apply(Action::PadClear);
        let lines = app.ask.as_ref().expect("a question").lines().to_vec();
        assert_eq!(lines[0], "Clear 3 pads?");
        assert_eq!(lines.last().unwrap(), "Enter: clear · Esc: keep");
        assert!((4..7).all(|a| app.pads.file[a].is_some()), "nothing yet");
        app.apply(Action::PadEdit); // Enter
        assert!(app.ask.is_none());
        assert!((4..7).all(|a| app.pads.file[a].is_none()));
        assert!(app.pads.file[7].is_some(), "pad R is not marked");
        assert_eq!(undo(&mut app), "undo: clear on 3 pads");
        assert!((4..7).all(|a| app.pads.file[a].is_some()));
    }

    /// 76 pad-playback "Esc keeps the marked pads".
    #[test]
    fn esc_keeps_the_marked_pads() {
        let mut app = undo_app();
        app.pads.file[4] = Some("/m/kick.wav".into());
        app.pads.marked.insert(4);
        app.apply(Action::PadClear);
        assert!(app.ask.is_some());
        app.apply(Action::Back);
        assert!(app.ask.is_none());
        assert!(app.pads.file[4].is_some());
        assert_eq!(app.pads.marked.len(), 1, "the marks stay");
    }

    /// 76 pad-playback "The edit names the reset key", and Enter answers the question
    /// through the keys, not the edit layer (where Enter closes the edit).
    #[test]
    fn the_edit_names_the_reset_key() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let (mut app, abs) = dressed_pad();
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let groups = super::help_groups(&keymap, Screen::Pads, &app.help_state());
        assert!(
            line(&groups).contains(&("Bksp".into(), "reset".into())),
            "{:?}",
            line(&groups)
        );
        pads_key(&mut app, KeyCode::Backspace, M::NONE);
        assert!(app.ask.is_some());
        pads_key(&mut app, KeyCode::Enter, M::NONE);
        assert!(app.ask.is_none() && app.edit);
        assert_eq!(app.pads.pitch[abs], 0);
    }

    /// 76 pad-playback "A sounding pad resets without a click": the commands of the reset
    /// reach a sounding voice; the sound goes on, no step larger than the tone's own →
    /// `renders/76-reset-sounding.wav`.
    #[test]
    fn a_sounding_pad_resets_without_a_click() {
        use engine::Engine;
        use engine::checks::{first_non_finite, max_step, peak};
        use engine::filter::FilterKind;
        use engine::offline::{Event, Scenario, render, write_wav};
        use std::path::Path;

        let sr = 48_000;
        let ms = |n: usize| sr as usize / 1000 * n;
        let mut app = undo_app();
        app.pads.file[0] = Some("/m/tone.wav".into());
        app.pads.volume_db[0] = -12;
        app.pads.filter[0] = FilterKind::LowPass;
        app.pads.cutoff_hz[0] = 300.0;
        let snap = app.pads.snapshot(0);
        let dressed: Vec<Command> = app
            .pads
            .put_back(0, &snap)
            .into_iter()
            .filter(|c| {
                !matches!(
                    c,
                    Command::SetPadSound { .. } | Command::SetPadRegion { .. }
                )
            })
            .collect();
        app.apply(Action::PadEdit);
        app.apply(Action::PadClear);
        app.sent.clear();
        app.apply(Action::PadEdit);
        let reset = std::mem::take(&mut app.sent);
        assert!(!reset.is_empty());
        assert!(
            !reset
                .iter()
                .any(|c| matches!(c, Command::SetPadSound { .. })),
            "the sound is not sent again"
        );
        let tone: Vec<f32> = (0..sr as usize * 2)
            .map(|i| (i as f32 * 440.0 * std::f32::consts::TAU / sr as f32).sin() * 0.5)
            .collect();
        let at = |frame, cmd| Event { frame, cmd };
        let mut events = vec![at(
            0,
            Command::SetPadSound {
                pad: 0,
                sound: Some(std::sync::Arc::new(Sample::mono(tone.clone()))),
            },
        )];
        events.extend(dressed.into_iter().map(|c| at(0, c)));
        events.push(at(
            ms(10),
            Command::PadPress {
                pad: 0,
                velocity: 1.0,
            },
        ));
        events.extend(reset.into_iter().map(|c| at(ms(500), c)));
        let sc = Scenario {
            sample_rate: sr,
            frames: ms(1000),
            events,
        };
        let (l, r) = render(&mut Engine::new(sr), &sc, 128);
        assert_eq!(first_non_finite(&l), None, "no NaN or infinities");
        assert!(peak(&l) < 1.0, "no clipping");
        let quiet = peak(&l[ms(300)..ms(500)]);
        let open = peak(&l[ms(800)..]);
        assert!(
            open > quiet * 2.0,
            "the sound goes on, louder: {quiet} → {open}"
        );
        // a 440 Hz tone at 0.5 moves at most ≈0.029 a sample; a click jumps far more
        let own = 0.5 * (440.0 * std::f32::consts::TAU / sr as f32);
        let step = max_step(&l[ms(20)..]);
        assert!(
            step <= own * 1.1,
            "step {step} against the tone's own {own}"
        );
        let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/76-reset-sounding.wav");
        write_wav(&out, sr, &l, &r).unwrap();
    }

    /// 70/77 pad-playback "The group on the pad": Q in group 1 carries its group to the
    /// mode line (drawn in tui), W none, the frame glyphs have no digit; F1 explains
    /// `mute N`. 77 "F1 names the mode glyphs".
    #[test]
    fn the_group_on_the_pad() {
        let mut app = undo_app();
        app.pads.group[4] = 1;
        app.pads.group[6] = 8;
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        assert_eq!(
            (v.pads[4].group, v.pads[6].group, v.pads[5].group),
            (1, 8, 0)
        );
        assert_eq!(
            v.pads[4].glyphs, "",
            "the digit is not a glyph of the string"
        );
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let f1 = super::full_help(&keymap, Screen::Pads, true);
        assert!(
            f1.iter()
                .any(|(k, t)| k == "mute N" && t.contains("its group"))
        );
        for (k, word) in [("no mark", "one-shot"), ("⇥", "gate")] {
            assert!(f1.iter().any(|(g, t)| g == k && t.contains(word)), "{k}");
        }
    }

    /// 70: the closed panel names the key the keymap gives the pad edit.
    #[test]
    fn the_edit_key_comes_from_the_keymap() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        assert_eq!(
            super::keys(&keymap, Screen::Pads, &[Action::PadEdit]),
            "Enter"
        );
    }

    /// 70 editing "Up down with the edit closed": K K ↓ I walk pitch, mute group, attack
    /// and back, the edit stays closed.
    #[test]
    fn up_down_with_the_edit_closed() {
        use super::pads::Param;
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = undo_app();
        let mut seen = Vec::new();
        for code in [
            KeyCode::Char('k'),
            KeyCode::Char('k'),
            KeyCode::Down,
            KeyCode::Char('i'),
        ] {
            pads_key(&mut app, code, M::NONE);
            seen.push(app.pads.param);
        }
        assert_eq!(
            seen,
            [Param::Pitch, Param::Group, Param::Attack, Param::Group]
        );
        assert!(!app.edit);
    }

    /// 70 editing "Tab with the edit closed": Tab Tab Shift+Tab with the pad edit closed
    /// turn the pages: ENV, FILTER, back to ENV; Shift+Tab as Tab with Shift alike.
    #[test]
    fn tab_with_the_edit_closed() {
        use super::pads::Param;
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = undo_app();
        assert!(!app.edit);
        pads_key(&mut app, KeyCode::Tab, M::NONE);
        assert_eq!(app.pads.param, Param::Attack);
        pads_key(&mut app, KeyCode::Tab, M::NONE);
        assert_eq!(app.pads.param, Param::Filter);
        pads_key(&mut app, KeyCode::BackTab, M::SHIFT);
        assert_eq!(app.pads.param, Param::Attack, "back to ENV");
        pads_key(&mut app, KeyCode::Tab, M::SHIFT);
        assert_eq!(app.pads.param, Param::Volume, "back to TONE");
        assert!(!app.edit, "the edit stays closed");
    }

    /// Opens the edit of the selected pad and walks to the mute group (70).
    fn to_group(app: &mut super::App) {
        app.apply(Action::PadEdit);
        while app.pads.param != super::pads::Param::Group {
            app.apply(Action::ParamNext);
        }
        assert_eq!(app.pads.page().0, "TONE");
    }

    fn tap_up(app: &mut super::App) {
        app.apply(Action::EditValue {
            up: true,
            fine: true,
        });
    }

    /// 70 pad-playback "A group for the marked pads": M, Q W E, M, the edit on the mute
    /// group, one L: the three are in group 1, R is not, the panel says 1.
    #[test]
    fn a_group_for_the_marked_pads() {
        let mut app = undo_app();
        app.apply(Action::PadMarkToggle);
        for pad in [4, 5, 6] {
            app.apply(Action::PadPress { pad, velocity: 0.8 });
            app.apply(Action::PadRelease { pad });
        }
        app.apply(Action::PadMarkToggle);
        to_group(&mut app);
        app.sent.clear();
        tap_up(&mut app);
        assert_eq!(app.pads.group[4..8], [1, 1, 1, 0]);
        for pad in [4, 5, 6] {
            assert!(
                app.sent
                    .iter()
                    .any(|c| matches!(c, Command::SetPadGroup { pad: p, group: 1 } if *p == pad)),
                "the engine knows pad {pad}"
            );
        }
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        assert_eq!(v.pads[v.selected].group, 1);
        assert_eq!(app.pads.param_value(), "1");
    }

    /// 70 pad-playback "Undo a group": Q into group 2, Ctrl+Z, Q is off again.
    #[test]
    fn undo_a_group() {
        let mut app = undo_app();
        app.pads.selected = 4;
        to_group(&mut app);
        tap_up(&mut app);
        tap_up(&mut app);
        assert_eq!(app.pads.group[4], 2);
        app.sent.clear();
        undo(&mut app);
        assert_eq!(app.pads.group[4], 0);
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::SetPadGroup { pad: 4, group: 0 }))
        );
    }

    /// 70: the mute group stops at 8 and at off, and knob 3 of TONE sets it.
    #[test]
    fn the_group_by_keys_and_knob_3() {
        let mut app = undo_app();
        to_group(&mut app);
        for _ in 0..12 {
            tap_up(&mut app);
        }
        assert_eq!(app.pads.group[0], 8, "it stops at 8");
        let knob = app.slot_param(2).unwrap();
        app.midi_event(super::midi::Event::Param {
            port: "X-Session".into(),
            control: "cc 26".into(),
            param: knob,
            value: 3.0 / 8.0,
            takeover: false,
        });
        assert_eq!(app.pads.group[0], 3, "knob 3 of TONE");
        app.midi_event(super::midi::Event::Param {
            port: "X-Session".into(),
            control: "cc 26".into(),
            param: knob,
            value: 0.0,
            takeover: false,
        });
        assert_eq!(app.pads.param_value(), "off");
    }

    fn undo(app: &mut super::App) -> String {
        app.apply(Action::Undo);
        last_notice(app).unwrap_or_default().to_owned()
    }

    /// 67 "Enter with no pad hit": in order, Enter takes the selected pad of bank 2 as the
    /// start and shows the plan; the second Enter lays the slices from there.
    #[test]
    fn enter_with_no_pad_hit() {
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(1));
        let n = app.chop.as_ref().unwrap().slices().len();
        assert!(n >= 2);
        app.pads.bank = 1;
        app.pads.selected = 4; // pad Q
        app.apply(Action::ChopAssign);
        app.apply(Action::ChopParts); // in order, no start yet
        assert!(matches!(app.plan, Some(super::chop::Plan::InOrder(None))));
        app.apply(Action::ChopEdit);
        let q: usize = engine::PADS_PER_BANK + 4;
        assert!(matches!(app.plan, Some(super::chop::Plan::InOrder(Some(s))) if s == q));
        assert!(app.pads.file[q].is_none(), "shown, not applied yet");
        app.apply(Action::ChopEdit);
        assert!(app.plan.is_none(), "applied");
        assert!(
            (q..q + n).all(|a| app.pads.file[a].is_some()),
            "every slice from pad Q"
        );
    }

    /// 66 "A new file appears": a file saved in the folder while the program runs shows on
    /// the next visit to F6, the selection stays.
    #[test]
    fn a_new_file_appears() {
        let dir = tmp_dir("fresh-list");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.wav"), b"").unwrap();
        std::fs::write(dir.join("c.wav"), b"").unwrap();
        let mut app = test_app();
        app.browser = super::browser::Browser::new(Some(dir.clone()));
        app.apply(Action::ScreenFiles);
        app.apply(Action::BrowserDown); // c.wav
        let name = |app: &super::App| app.browser.current().map(|e| e.name.clone());
        assert_eq!(name(&app).as_deref(), Some("c.wav"));
        app.apply(Action::ScreenPads);
        std::fs::write(dir.join("b.wav"), b"").unwrap();
        app.apply(Action::ScreenFiles);
        assert!(app.browser.entries.iter().any(|e| e.name == "b.wav"));
        assert_eq!(name(&app).as_deref(), Some("c.wav"), "the selection stays");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 66 "Esc clears the filter": the first Esc on F6 drops the filter, the whole folder
    /// is listed; the next Esc leaves the screen.
    #[test]
    fn esc_clears_the_filter() {
        let dir = tmp_dir("filter-esc");
        std::fs::create_dir_all(&dir).unwrap();
        for f in ["snare.wav", "kick.wav"] {
            std::fs::write(dir.join(f), b"").unwrap();
        }
        let mut app = test_app();
        app.browser = super::browser::Browser::new(Some(dir.clone()));
        app.apply(Action::ScreenFiles);
        app.browser.filter = "sn".into();
        app.browser.reread();
        assert_eq!(app.browser.entries.len(), 1);
        app.apply(Action::Back);
        assert_eq!(app.screen, Screen::Files, "the first Esc drops the filter");
        assert!(app.browser.filter.is_empty());
        assert_eq!(app.browser.entries.len(), 2);
        app.apply(Action::Back);
        assert_eq!(app.screen, Screen::Pads);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 61 "Undo a knob turn": one turn of the volume knob is one step.
    #[test]
    fn undo_a_knob_turn() {
        let mut app = undo_app();
        let abs = app.pads.selected_abs();
        for v in [100, 80, 60, 40, 20] {
            xsession(&mut app, knob(K1, v));
        }
        assert!(app.pads.volume_db[abs] < 0);
        let said = undo(&mut app);
        assert_eq!(app.pads.volume_db[abs], 0, "one press brings it back");
        assert!(said.contains("undo: volume on pad"), "{said}");
        assert!(app.sent.iter().any(|c| matches!(
            c,
            Command::SetPadGain { pad, gain } if *pad == abs && (*gain - 1.0).abs() < 1e-6
        )));
        assert_eq!(undo(&mut app), "nothing to undo");
    }

    /// 61 "Undo over the scope": the cutoff of the 16 pads of the bank comes back at once.
    #[test]
    fn undo_over_the_scope() {
        use super::pads::Param;
        let mut app = undo_app();
        app.apply(Action::PadMarkBank); // the 16 pads of the bank marked
        app.apply(Action::PadEdit);
        while app.pads.param != Param::Cutoff {
            app.apply(Action::ParamNext);
        }
        for _ in 0..3 {
            app.apply(Action::EditValue {
                up: true,
                fine: false,
            });
        }
        assert!((0..16).all(|a| app.pads.cutoff_hz[a] > super::pads::CUTOFF_DEFAULT));
        let said = undo(&mut app);
        assert!(
            (0..16).all(|a| app.pads.cutoff_hz[a] == super::pads::CUTOFF_DEFAULT),
            "one step for the run of keys"
        );
        assert_eq!(said, "undo: cutoff on 16 pads");
    }

    /// 61 "Undo a swap" and "Undo a clear".
    #[test]
    fn undo_a_swap_and_a_clear() {
        let mut app = laid_out_break();
        app.screen = Screen::Pads;
        let (q, v) = (4, 0);
        let (sq, sv) = (app.pads.snapshot(q), app.pads.snapshot(v));
        app.pads.selected = q;
        app.apply(Action::PadTake);
        app.apply(Action::PadPress {
            pad: v,
            velocity: 1.0,
        });
        assert_ne!(app.pads.snapshot(q), sq, "swapped");
        let said = undo(&mut app);
        assert_eq!(
            (app.pads.snapshot(q), app.pads.snapshot(v)),
            (sq.clone(), sv)
        );
        assert!(said.contains("swap"), "{said}");
        app.pads.selected = q;
        app.apply(Action::PadClear);
        assert!(app.pads.file[q].is_none());
        undo(&mut app);
        assert_eq!(app.pads.snapshot(q), sq, "the file is back");
    }

    /// 61 "Undo a shared slot": FX 2 turned to a reverb comes back as the delay it was.
    #[test]
    fn undo_a_shared_slot() {
        use engine::fx::FxKind;
        let mut app = undo_app();
        app.apply(Action::ScreenFx);
        app.apply(Action::PageNext);
        app.apply(Action::EditValue {
            up: true,
            fine: true,
        });
        assert_eq!(app.pads.fx[1].kind, FxKind::Reverb);
        let said = undo(&mut app);
        assert_eq!(app.pads.fx[1].kind, FxKind::Delay);
        assert_eq!(app.pads.fx[1].time_ms, 250);
        assert!(said.contains("shared slot"), "{said}");
    }

    /// 61 "Nothing to undo" and "What changes no pad".
    #[test]
    fn what_changes_no_pad() {
        let mut app = undo_app();
        assert_eq!(undo(&mut app), "nothing to undo");
        app.apply(Action::PadParam(1)); // the volume up
        let abs = app.pads.selected_abs();
        assert_eq!(app.pads.volume_db[abs], 1);
        for _ in 0..2 {
            app.apply(Action::PadMarkBank);
        }
        app.apply(Action::BankNext);
        app.apply(Action::PadSelect(input::Dir::Right));
        let (bank, selected) = (app.pads.bank, app.pads.selected);
        undo(&mut app);
        assert_eq!(app.pads.volume_db[abs], 0, "the volume comes back");
        assert_eq!((app.pads.bank, app.pads.selected), (bank, selected));
    }

    fn glyphs(app: &mut super::App, abs: usize) -> String {
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        v.pads[abs].glyphs.clone()
    }

    /// 77 "A filter marks the pad", "A fresh pad has no mark".
    /// Scenario "The mark goes with the setting".
    #[test]
    fn a_filter_marks_the_pad() {
        use engine::filter::FilterKind;
        let mut app = laid_out_break();
        assert_eq!(
            glyphs(&mut app, 0),
            "",
            "a fresh pad: the whole file, nothing set"
        );
        app.pads.filter[0] = FilterKind::LowPass;
        assert_eq!(glyphs(&mut app, 0), "✱");
        app.pads.filter[0] = FilterKind::Off;
        app.pads.pitch[0] = 2;
        assert_eq!(glyphs(&mut app, 0), "✱");
        app.pads.pitch[0] = 0;
        assert_eq!(glyphs(&mut app, 0), "", "the mark goes with the setting");
        app.pads.decay_ms[0] = 40;
        assert_eq!(glyphs(&mut app, 0), "✱", "the envelope");
        assert_eq!(glyphs(&mut app, 3), "", "an empty pad has none");
    }

    /// 77 "A filter that is off marks nothing": the cutoff moved with the filter off gives
    /// no mark and no change.
    #[test]
    fn a_filter_that_is_off_marks_nothing() {
        let mut app = laid_out_break();
        app.pads.cutoff_hz[0] = 1_100.0;
        app.pads.reso[0] = 20;
        assert_eq!(glyphs(&mut app, 0), "");
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        assert!(v.pads[0].changed.is_empty(), "{:?}", v.pads[0].changed);
        app.pads.filter[0] = engine::filter::FilterKind::LowPass;
        assert_eq!(glyphs(&mut app, 0), "✱", "on: it counts");
    }

    /// 77 "A slice is not a setting", "Volume and group are not counted twice" and "The
    /// panel names the settings" (the values).
    #[test]
    fn a_slice_is_not_a_setting() {
        use tui::PadParam as W;
        let mut app = laid_out_break();
        for abs in 4..12 {
            assert_eq!(glyphs(&mut app, abs), "", "pad {abs} plays a slice");
        }
        app.pads.volume_db[4] = -6;
        app.pads.group[4] = 2;
        assert_eq!(
            glyphs(&mut app, 4),
            "",
            "volume and group are on the line already"
        );
        app.pads.pitch[0] = -5;
        app.pads.filter[0] = engine::filter::FilterKind::LowPass;
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        let changed = &v.pads[0].changed;
        assert!(changed.contains(&W::Pitch) && changed.contains(&W::Filter));
        assert!(!changed.contains(&W::Attack) && !changed.contains(&W::Source));
    }

    fn panel_line(app: &mut super::App, abs: usize) -> String {
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        v.pads[abs].detail.clone().unwrap_or_default()
    }

    /// 27 pad-playback: "Slices laid out".
    #[test]
    fn slices_laid_out() {
        let mut app = laid_out_break();
        assert_eq!(panel_line(&mut app, 6), "break.wav · slice 3");
        assert_eq!(app.pads.source(6), super::pads::Source::Slice(3));
    }

    /// 27 pad-playback: "The whole file".
    #[test]
    fn the_whole_file() {
        let mut app = laid_out_break();
        assert_eq!(panel_line(&mut app, 0), "break.wav · all");
    }

    /// 27 task 1: the slice number follows the cuts — a delete renumbers the later pads;
    /// a new chop with `Y` turns the pads into their own bounds, undo links them back.
    /// Scenario "Join two slices", "Chop again into equal parts" and "Undo the new chop" (chopping).
    #[test]
    fn the_source_follows_the_cuts() {
        use super::pads::Source;
        let mut app = laid_out_break();
        let (two, four) = (app.pads.region[5], app.pads.region[7]);
        // delete the cut at the start of slice 3: pads 6 and 7 play the joined slice 2
        app.chop.as_mut().unwrap().select_slice(2);
        app.apply(Action::MarkerDelete);
        assert_eq!(app.pads.source(6), Source::Slice(2));
        assert_eq!(
            app.pads.source(7),
            Source::Slice(3),
            "the later pads move down by one"
        );
        assert_eq!(app.pads.region[5], app.pads.region[6], "one joined slice");
        assert_eq!(app.pads.region[5].start, two.start);
        assert_eq!(app.pads.region[7], four, "the same sound");
        app.apply(Action::Undo);
        assert_eq!(app.pads.source(6), Source::Slice(3));
        // a new chop of the file: the pads keep their bounds and are their own now
        let region = app.pads.region[6];
        let cuts = app.chop.as_ref().unwrap().markers.clone();
        app.apply(Action::ChopParts);
        assert_eq!(app.pads.source(6), Source::Own);
        assert_eq!(app.pads.region[6], region, "it plays what it played");
        assert_eq!(panel_line(&mut app, 6), "break.wav · pad");
        assert!(
            flash(&app).starts_with("8 pads keep their old slices"),
            "{}",
            flash(&app)
        );
        app.apply(Action::Undo);
        assert_eq!(app.pads.source(6), Source::Slice(3), "undo links it back");
        assert_eq!(app.chop.as_ref().unwrap().markers, cuts, "the old cuts");
    }

    /// 68/76 screen-help "Scope in the help line": `Ctrl+A` on bank 2 shows
    /// `mark: on, hit pads · scope: 16 marked` at the mark key, also with the Ctrl combos
    /// hidden (kitty); after `Esc`, `mark: off · scope: pad 1`.
    #[test]
    fn scope_in_the_help_line() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Pads;
        app.pads.bank = 1;
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let said = |app: &super::App| {
            let groups = super::help_groups(&keymap, Screen::Pads, &app.help_state());
            line(&super::without_modifiers(groups))
        };
        pads_key(&mut app, KeyCode::Char('a'), M::CONTROL);
        assert!(
            said(&app).contains(&("M".into(), "mark: on, hit pads · scope: 16 marked".into())),
            "{:?}",
            said(&app)
        );
        app.apply(Action::Back);
        assert!(
            said(&app).contains(&("M".into(), "mark: off · scope: pad 1".into())),
            "{:?}",
            said(&app)
        );
    }

    /// 76 screen-help "The mark keys in the full help".
    #[test]
    fn the_mark_keys_in_the_full_help() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let rows = super::full_help(&keymap, input::Screen::Pads, true);
        assert!(
            rows.iter()
                .any(|(k, t)| k == "Ctrl+A" && t.contains("mark bank")),
            "{rows:?}"
        );
        assert!(!rows.iter().any(|(_, t)| t.contains("scope")), "{rows:?}");
    }

    /// 76 pad-playback "The mark plate": the title line says marking is on and how many
    /// pads are marked; with marking off and marks left it says so; after Esc it is gone.
    #[test]
    fn the_mark_plate() {
        let mut app = undo_app();
        let plate = |app: &mut super::App| {
            app.title_badges()
                .into_iter()
                .find(|badge| badge.normal.contains("MARK"))
                .map(|badge| badge.normal)
        };
        assert_eq!(plate(&mut app), None);
        app.apply(Action::PadMarkToggle);
        for pad in [0, 1, 2] {
            app.apply(Action::PadPress { pad, velocity: 0.8 });
            app.apply(Action::PadRelease { pad });
        }
        assert_eq!(
            plate(&mut app).as_deref(),
            Some("MARK · hit pads · 3 marked · ■ 120.0")
        );
        app.apply(Action::PadMarkToggle);
        assert_eq!(
            plate(&mut app).as_deref(),
            Some("3 MARKED · Esc drops · ■ 120.0")
        );
        app.apply(Action::Back);
        assert_eq!(plate(&mut app), None);
    }

    /// 76 pad-playback "Esc in the edit keeps the marks".
    #[test]
    fn esc_in_the_edit_keeps_the_marks() {
        let mut app = test_app();
        app.screen = Screen::Pads;
        app.apply(Action::PadMarkToggle);
        for pad in [0, 1, 2] {
            app.apply(Action::PadPress { pad, velocity: 0.8 });
            app.apply(Action::PadRelease { pad });
        }
        app.apply(Action::PadEdit);
        assert!(app.edit);
        app.apply(Action::Back);
        assert!(!app.edit, "the first Esc closes the edit");
        assert_eq!(app.pads.marked.len(), 3, "the marks stay");
        app.apply(Action::Back);
        assert!(app.pads.marked.is_empty(), "a second Esc drops them");
    }

    /// The pads screen key `code` with `m`, through the keymap and its edit layer.
    fn pads_key(
        app: &mut super::App,
        code: ratatui::crossterm::event::KeyCode,
        m: ratatui::crossterm::event::KeyModifiers,
    ) {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let k = ratatui::crossterm::event::KeyEvent::new(code, m);
        let action = keymap
            .resolve_in(Screen::Pads, app.edit_keys(), &k)
            .unwrap();
        app.apply(action);
    }

    /// The laid-out break on the pads screen, pad 7 (slice 3) selected, its edit on the
    /// SOURCE page at `param`.
    fn source_page(param: super::pads::Param) -> super::App {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = laid_out_break();
        let sound = app.chop.as_ref().unwrap().sound.clone();
        for abs in 0..12 {
            app.pads.sound[abs] = Some(sound.clone());
        }
        app.screen = Screen::Pads;
        app.pads.selected = 6;
        pads_key(&mut app, KeyCode::Enter, M::NONE);
        // K walks one parameter; Tab turns whole pages (54)
        while app.pads.param != param {
            pads_key(&mut app, KeyCode::Char('k'), M::NONE);
        }
        assert_eq!(app.pads.page().0, "SOURCE");
        app
    }

    /// 27 pad-playback: "Edit only this fragment".
    #[test]
    fn edit_only_this_fragment() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = source_page(super::pads::Param::Start);
        let (before, six, eight) = (app.pads.region[6], app.pads.region[5], app.pads.region[7]);
        pads_key(&mut app, KeyCode::Char('l'), M::NONE);
        // a tap is the fine step of the value keys (21): 1 ms
        assert_eq!(app.pads.region[6].start, before.start + 48, "1 ms later");
        assert_eq!(app.pads.region[6].end, before.end);
        assert_eq!(app.pads.source(6), super::pads::Source::Own);
        assert_eq!(panel_line(&mut app, 6), "break.wav · pad");
        assert_eq!(
            (app.pads.region[5], app.pads.region[7]),
            (six, eight),
            "the neighbours stay"
        );
        assert_eq!(
            flash(&app),
            "pad E no longer follows the cuts; Ctrl+Z links it back"
        );
    }

    /// 27 pad-playback: "Undo links it back".
    #[test]
    fn undo_links_it_back() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = source_page(super::pads::Param::Start);
        let before = app.pads.region[6];
        pads_key(&mut app, KeyCode::Char('l'), M::NONE);
        pads_key(&mut app, KeyCode::Char('z'), M::CONTROL);
        assert_eq!(app.pads.source(6), super::pads::Source::Slice(3));
        assert_eq!(app.pads.region[6], before, "it plays slice 3 again");
    }

    /// 27 pad-playback: "Pick another slice".
    #[test]
    fn pick_another_slice() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = source_page(super::pads::Param::Source);
        app.pads.selected = 0; // pad 1 plays the whole break
        assert_eq!(app.pads.source(0), super::pads::Source::All);
        for _ in 0..2 {
            pads_key(&mut app, KeyCode::Char('l'), M::NONE);
        }
        assert_eq!(app.pads.source(0), super::pads::Source::Slice(2));
        let slice2 = app.chop.as_ref().unwrap().slices()[1];
        assert_eq!(app.pads.region[0], slice2);
        // it follows the cuts from now on
        app.screen = Screen::Chop;
        app.chop.as_mut().unwrap().select_slice(1);
        app.apply(Action::MarkerNudge {
            right: true,
            fine: false,
        });
        let moved = app.chop.as_ref().unwrap().slices()[1];
        assert_eq!(app.pads.region[0], moved);
    }

    /// 27 "Source page" on the knobs: k2 of the X-Session turns the start of the pad,
    /// after the pickup, and the pad becomes its own.
    #[test]
    fn the_source_page_on_the_knobs() {
        let mut app = source_page(super::pads::Param::Start);
        let before = app.pads.region[6];
        let len = app.pads.sound[6].as_ref().unwrap().len() as f32;
        let at = (before.start as f32 / len * 127.0).round() as u8;
        xsession(&mut app, knob(K2, at));
        xsession(&mut app, knob(K2, at + 2));
        assert!(
            app.pads.region[6].start > before.start,
            "the start follows k2"
        );
        assert_eq!(app.pads.source(6), super::pads::Source::Own);
    }

    /// The chop screen opened with F3 from pad `abs` of the laid-out break.
    fn chop_from_pad(abs: usize) -> super::App {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = laid_out_break();
        let sound = app.chop.as_ref().unwrap().sound.clone();
        for a in 0..12 {
            app.pads.sound[a] = Some(sound.clone());
        }
        app.screen = Screen::Pads;
        app.pads.selected = abs;
        pads_key(&mut app, KeyCode::F(3), M::NONE);
        assert_eq!(app.screen, Screen::Chop);
        app
    }

    /// 31 "The same frame on every screen" with 36: the chop grid shows the real pads of
    /// the bank with their cached waves, as the pads screen does; pads of this chop name
    /// their slice, the rest is dimmed.
    #[test]
    fn the_chop_pads_draw_their_waves() {
        let mut app = chop_from_pad(6);
        let c = app.chop.as_ref().unwrap();
        let mut view = super::chop_view(c, None, None, 48_000, &app.pads).pads;
        super::bank_waves(&mut view, &mut app.pads, 48_000);
        for i in 0..16 {
            let want = app.pads.wave(i, 48_000, super::BIG_COLS);
            assert_eq!(view.pads[i].wave, want, "pad {i} draws its own wave");
        }
        assert!(!view.pads[6].dim && view.pads[6].note.as_deref() == Some("slice 3"));
        assert!(view.pads[1].dim, "an empty pad steps back");
    }

    /// The laid-out break of `laid_out_break` moved to bank 2 (pads 16..24), the whole
    /// break on pad 1 of bank 1, the pads screen on bank 1.
    fn break_on_bank_2() -> super::App {
        let mut app = laid_out_break();
        let sound = app.chop.as_ref().unwrap().sound.clone();
        for i in 0..8 {
            let (from, to) = (4 + i, 16 + i);
            app.pads.file[to] = app.pads.file[from].take();
            app.pads.region[to] = app.pads.region[from];
            app.pads.link[to] = app.pads.link[from].take();
            app.pads.slice[to] = app.pads.slice[from].take();
            app.pads.sound[to] = Some(sound.clone());
            app.pads.region[from] = Region::WHOLE;
        }
        app.pads.sound[0] = Some(sound);
        app.pads.bank = 0;
        app.pads.selected = 0;
        app.screen = Screen::Pads;
        app
    }

    /// The break's chop opened from the files screen with bank 1 shown: the same chop, its
    /// slices on bank 2 (F3 on the whole-file pad would start a fresh one, 42).
    fn open_break_from_files(app: &mut super::App) {
        app.screen = Screen::Files;
        let c = app.chop.take().unwrap();
        let (file, sound) = (c.file.clone(), c.sound.clone());
        app.chops.insert(file.clone(), c);
        app.show_file_chop(file, sound);
    }

    /// 36 chopping "Check a slice": the break opened from the files screen on bank 1 shows
    /// bank 2, where the slices lie; the third pad plays itself and slice 3 is selected.
    #[test]
    fn check_a_slice() {
        let mut app = break_on_bank_2();
        open_break_from_files(&mut app);
        assert_eq!(app.screen, Screen::Chop);
        assert_eq!(app.pads.bank, 1, "the grid shows bank 2");
        app.sent.clear();
        hit(&mut app, 2);
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::PadPress { pad: 18, .. })),
            "the real pad 2-3 plays: {:?}",
            app.sent
        );
        assert_eq!(
            app.chop.as_ref().unwrap().selected,
            2,
            "slice 3 is selected"
        );
        let c = app.chop.as_ref().unwrap();
        let v = super::chop_view(c, None, None, 48_000, &app.pads);
        assert_eq!(v.pads.pads[2].note.as_deref(), Some("slice 3"));
        assert!(
            v.pads.pads[12].dim,
            "a pad with no slice of this chop is dimmed"
        );
        assert_eq!(v.side[0], "slices lie on bank 2");
    }

    /// 36: the files screen opens the chop on the bank of its slices too, when the bank
    /// shown holds none of them (the pad path is `check_a_slice`).
    #[test]
    fn the_files_path_opens_on_the_bank_of_the_slices() {
        let mut app = break_on_bank_2();
        app.screen = Screen::Files;
        let c = app.chop.take().unwrap();
        let (file, sound) = (c.file.clone(), c.sound.clone());
        app.chops.insert(file.clone(), c);
        app.show_file_chop(file, sound);
        assert_eq!(app.screen, Screen::Chop);
        assert_eq!(
            app.pads.bank, 1,
            "the grid shows bank 2, where the slices lie"
        );
    }

    /// 36 chopping "Before the layout": no slice on a pad yet, the keys play the slices.
    #[test]
    fn before_the_layout() {
        let mut app = chop_open(); // 8 slices, none on a pad
        app.pads.bank = 1; // bank 2 open: still slices 1–8, there is no 17–32
        hit(&mut app, 2);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.selected, 2, "slice 3 is selected");
        assert_eq!(last_played(&app), Some(c.slices()[2]), "and plays");
        let v = super::chop_view(c, None, None, 48_000, &app.pads);
        assert_eq!(v.slice_keys, Some((1, 8)));
        assert_eq!(v.pads.pads[0].name, "slice 1");
        assert_eq!(v.pads.pads[7].name, "slice 8");
        assert!(v.pads.pads[8].dim, "no slice 9: the cell steps back");
        // once one slice lies on a pad, the grid shows the real pads
        app.pads.file[5] = Some(c.file.clone());
        app.pads.link[5] = Some((0, c.epoch));
        app.pads.slice[5] = Some(1);
        let c = app.chop.as_ref().unwrap();
        let v = super::chop_view(c, None, None, 48_000, &app.pads);
        assert_eq!(v.slice_keys, None);
    }

    /// 42/45 chopping "Open from the whole-file pad" and "Undo the fresh chop": F3 on the
    /// whole break starts a fresh chop; the laid-out pads play on and keep their slices,
    /// the older chop stays with them; undo opens the older chop again.
    #[test]
    fn open_from_the_whole_file_pad_starts_fresh() {
        use super::pads::Source;
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = break_on_bank_2(); // pad 1-1 whole, slices on 2-1…2-8
        let before: Vec<Region> = (16..24).map(|a| app.pads.region[a]).collect();
        pads_key(&mut app, KeyCode::F(3), M::NONE);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.slices().len(), 1, "no cuts");
        assert_eq!(
            flash(&app),
            "a new chop of this file; the older one keeps its pads, Ctrl+Z goes back to it"
        );
        for (i, a) in (16..24).enumerate() {
            assert_eq!(
                app.pads.region[a], before[i],
                "pad {a} plays what it played"
            );
            assert_eq!(
                app.pads.source(a),
                Source::Slice(i as u16 + 1),
                "and keeps its slice"
            );
        }
        app.apply(Action::Undo);
        assert_eq!(
            app.chop.as_ref().unwrap().slices().len(),
            8,
            "the older chop is open again"
        );
        assert_eq!(app.which_chop(), None, "the fresh chop had no pads: gone");
        for a in 16..24 {
            assert!(
                matches!(app.pads.source(a), Source::Slice(_)),
                "pad {a} follows"
            );
        }
    }

    /// A 2 s break written to `dir/break.wav` and an app on it: the chop cut in 8 laid on
    /// bank 2, pad 1-1 playing the whole file; the session goes to `dir/session.toml`.
    fn real_break(dir: &std::path::Path) -> (super::App, std::path::PathBuf) {
        std::fs::create_dir_all(dir).unwrap();
        let path = dir.join("break.wav");
        let wave: Vec<f32> = (0..96_000)
            .map(|i| 0.3 * (i as f32 * 0.05).sin() * (1.0 - (i % 12_000) as f32 / 12_000.0))
            .collect();
        engine::offline::write_wav(&path, 48_000, &wave, &wave).unwrap();
        let sound = std::sync::Arc::new(engine::Sample::mono(wave));
        let mut app = test_app();
        app.session_path = dir.join("session.toml");
        app.session_read_error = None;
        let mut c = super::chop::Chop::new(path.clone(), sound.clone(), 48_000);
        for _ in 0..3 {
            c.next_parts();
        }
        app.chop = Some(c);
        lay_out(&mut app, 16);
        app.pads.file[0] = Some(path.clone());
        app.pads.sound[0] = Some(sound);
        app.screen = Screen::Pads;
        (app, path)
    }

    /// Opens the session file `file` in a fresh app and waits for the samples.
    fn reopen(file: &std::path::Path) -> super::App {
        let mut app = test_app();
        app.apply_session(super::session::load(file).unwrap());
        wait_for(&mut app, |a| a.pads.loading.iter().all(Option::is_none));
        app
    }

    /// F3 on pad `abs`, from the pads screen.
    fn f3_on(app: &mut super::App, abs: usize) {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        app.screen = Screen::Pads;
        app.pads.bank = abs / 16;
        app.pads.selected = abs % 16;
        pads_key(app, KeyCode::F(3), M::NONE);
    }

    /// 45 chopping "Saved and opened": both chops of the break go to the project and come
    /// back; F3 on pad 2-3 opens the first, F3 on pad 3-2 the second.
    #[test]
    fn saved_and_opened_keeps_both_chops() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let dir = tmp_dir("two-chops");
        let (mut app, _) = real_break(&dir);
        f3_on(&mut app, 0); // fresh
        chop_key(&mut app, KeyCode::Char('y'), M::NONE);
        chop_key(&mut app, KeyCode::Char('y'), M::NONE);
        lay_out(&mut app, 32);
        app.save_session().unwrap();
        let text = std::fs::read_to_string(dir.join("session.toml")).unwrap();
        assert!(text.contains("[[kept."), "the older chop is saved: {text}");
        let mut back = reopen(&dir.join("session.toml"));
        f3_on(&mut back, 18);
        assert_eq!(
            back.chop.as_ref().unwrap().slices().len(),
            8,
            "pad 2-3: the first"
        );
        assert_eq!(back.chop.as_ref().unwrap().selected, 2);
        f3_on(&mut back, 33);
        assert_eq!(
            back.chop.as_ref().unwrap().slices().len(),
            4,
            "pad 3-2: the second"
        );
        assert_eq!(back.chop.as_ref().unwrap().selected, 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 45 chopping "A project saved before", the owner's project of 2026-10-31: pads of the
    /// chops of epochs 3 and 7 and a saved chop at 8 with no cuts. F3 on a pad of epoch 3
    /// opens its chop rebuilt from the pads' bounds, and they say their slices again.
    #[test]
    fn a_project_saved_before_rebuilds_its_chop() {
        use super::pads::Source;
        let dir = tmp_dir("lost-chop");
        let (_, path) = real_break(&dir);
        let f = path.to_string_lossy();
        // epoch 3: 4 pads at 0.5 s steps from 0.5 s; epoch 7: one pad 0–0.5 s; chop 8 empty
        let mut toml = String::from("version = 3\n");
        for (i, (start, end, id)) in [
            (0.5, 1.0, 3),
            (1.0, 1.5, 5),
            (1.5, 1.75, 12),
            (1.75, 2.0, 6),
        ]
        .into_iter()
        .enumerate()
        {
            toml += &format!(
                "[pads.{}]\nfile = \"{f}\"\nstart = {start}\nend = {end}\nlink = {id}\nepoch = 3\n",
                5 + i
            );
        }
        toml +=
            &format!("[pads.64]\nfile = \"{f}\"\nstart = 0.0\nend = 0.5\nlink = 0\nepoch = 7\n");
        toml += &format!(
            "[chops.\"{f}\"]\nstart = 0.0\nend = 2.0\ncuts = []\nids = []\nepoch = 8\nnext_id = 33\n"
        );
        std::fs::write(dir.join("old.toml"), toml).unwrap();
        let mut app = reopen(&dir.join("old.toml"));
        f3_on(&mut app, 6); // pad W of epoch 3
        let c = app.chop.as_ref().unwrap();
        let cuts: Vec<usize> = c.markers.clone();
        assert_eq!(
            cuts,
            [24_000, 48_000, 72_000, 84_000],
            "the cuts at the pads' bounds"
        );
        assert_eq!(c.selected, 2, "pad W plays the slice from 1.0 s");
        for (i, a) in (5..9).enumerate() {
            assert_eq!(app.pads.source(a), Source::Slice(i as u16 + 2), "pad {a}");
        }
        // the empty chop 8 is still the newest; pad 5-1 of epoch 7 rebuilds its own
        f3_on(&mut app, 64);
        assert_eq!(app.chop.as_ref().unwrap().markers, [24_000]);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 11 "Switch and switch back": the chops of the previous project's files stay behind.
    #[test]
    fn a_switch_leaves_the_chops_behind() {
        let mut app = break_on_bank_2();
        let c = app.chop.take().unwrap();
        app.chops.insert(c.file.clone(), c);
        app.apply_session(super::session::Session::default());
        assert!(app.chops.is_empty() && app.older.is_empty() && app.chop.is_none());
    }

    /// Links pads `from..` to the slices of the open chop, as a layout does.
    fn lay_out(app: &mut super::App, from: usize) {
        let c = app.chop.as_ref().unwrap();
        let (file, sound) = (c.file.clone(), c.sound.clone());
        for (i, slice) in c.slices().into_iter().enumerate() {
            app.pads.file[from + i] = Some(file.clone());
            app.pads.sound[from + i] = Some(sound.clone());
            app.pads.region[from + i] = slice;
            app.pads.link[from + i] = Some((c.start_id(i), c.epoch));
        }
        app.sync_links();
    }

    /// 45 chopping "Chop the same sample again", "Back to an older pad" and "A cut moves
    /// only its own pads": the break laid out on bank 2, chopped again in 4 from the
    /// whole-file pad onto bank 3; F3 on pad 2-3 opens the first chop, and its cut moves
    /// pad 2-3 only.
    /// Scenario "A cut moves only its own pads" (chopping).
    #[test]
    fn chop_the_same_sample_again_and_go_back() {
        use super::pads::Source;
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = break_on_bank_2();
        pads_key(&mut app, KeyCode::F(3), M::NONE); // fresh, from pad 1-1
        chop_key(&mut app, KeyCode::Char('y'), M::NONE); // 2 parts
        chop_key(&mut app, KeyCode::Char('y'), M::NONE); // 4 parts
        assert_eq!(app.chop.as_ref().unwrap().slices().len(), 4);
        lay_out(&mut app, 32); // bank 3
        for i in 0..8 {
            assert_eq!(
                app.pads.source(16 + i),
                Source::Slice(i as u16 + 1),
                "2-{}",
                i + 1
            );
        }
        for i in 0..4 {
            assert_eq!(
                app.pads.source(32 + i),
                Source::Slice(i as u16 + 1),
                "3-{}",
                i + 1
            );
        }
        let new_pads: Vec<Region> = (32..36).map(|a| app.pads.region[a]).collect();
        // back to pad 2-3
        app.screen = Screen::Pads;
        app.pads.bank = 1;
        app.pads.selected = 2;
        pads_key(&mut app, KeyCode::F(3), M::NONE);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.slices().len(), 8, "the first chop with its 8 slices");
        assert_eq!(c.selected, 2, "slice 3 selected");
        assert_eq!(app.which_chop(), Some((1, 2)), "chop 1 of 2");
        let before = app.pads.region[18];
        app.apply(Action::SliceBound {
            end: false,
            right: true,
            fine: true,
        });
        assert!(
            app.pads.region[18].start > before.start,
            "pad 2-3 starts later"
        );
        let after: Vec<Region> = (32..36).map(|a| app.pads.region[a]).collect();
        assert_eq!(after, new_pads, "pads 3-1…3-4 do not change");
        // and F3 on pad 3-2 opens the second chop again
        app.screen = Screen::Pads;
        app.pads.bank = 2;
        app.pads.selected = 1;
        pads_key(&mut app, KeyCode::F(3), M::NONE);
        assert_eq!(app.chop.as_ref().unwrap().slices().len(), 4);
        assert_eq!(app.which_chop(), Some((2, 2)));
    }

    /// 36 chopping "Fix the slice just heard": the hit on the pad of slice 6 selects it, the
    /// start moves and the pad follows.
    #[test]
    fn fix_the_slice_just_heard() {
        let mut app = break_on_bank_2();
        open_break_from_files(&mut app);
        hit(&mut app, 5); // pad 2-W holds slice 6
        assert_eq!(app.chop.as_ref().unwrap().selected, 5);
        let before = app.pads.region[21];
        app.apply(Action::SliceBound {
            end: false,
            right: true,
            fine: true,
        });
        assert!(
            app.pads.region[21].start > before.start,
            "the pad follows its start"
        );
    }

    /// 36 chopping "A cut leaves a slice on no pad": a cut inside slice 5 of a laid-out
    /// break: slice 5 wears ● of its bank, the new slice 6 a ○, the hint says how to place it.
    #[test]
    fn a_cut_leaves_a_slice_on_no_pad() {
        let mut app = laid_out_break(); // slices 1–8 on Q W E R A S D F of bank 1
        let s5 = app.chop.as_ref().unwrap().slices()[4];
        app.chop
            .as_mut()
            .unwrap()
            .set_cursor(s5.start + (s5.end - s5.start) / 2);
        app.apply(Action::ChopCut);
        let c = app.chop.as_ref().unwrap();
        let v = super::chop_view(c, None, None, 48_000, &app.pads);
        assert_eq!(v.owner_bank[4], Some(0), "slice 5: ● of bank 1");
        assert_eq!(v.owner_bank[5], None, "the new slice 6: ○");
        assert_eq!(
            v.owner_bank[6],
            Some(0),
            "the old slice 6, now 7, stays on its pad"
        );
        assert_eq!(
            super::hint_for(&app),
            Some("○ slices lie on no pad: P, J L and a pad hit place them.")
        );
    }

    /// 37 chopping "Open on your slice": F3 on the pad of slice 5 shows slice 5 in the
    /// middle with half of it on each side; zooming out reaches the whole file.
    #[test]
    fn open_on_your_slice() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = break_on_bank_2(); // slice 5 on pad 2-Q (abs 20)
        app.pads.bank = 1;
        app.pads.selected = 4;
        pads_key(&mut app, KeyCode::F(3), M::NONE);
        let c = app.chop.as_ref().unwrap();
        let s5 = c.slices()[4];
        let half = (s5.end - s5.start) / 2;
        assert_eq!(c.selected, 4);
        assert_eq!(c.view.start, s5.start - half, "half a slice before it");
        assert_eq!(c.view.end, s5.end + half, "half a slice after it");
        for _ in 0..12 {
            app.apply(Action::ChopZoom { zoom_in: false });
        }
        let c = app.chop.as_ref().unwrap();
        assert_eq!(
            (c.view.start, c.view.end),
            (0, c.sound.len()),
            "the whole file"
        );
    }

    /// 69 chopping "Open on your own piece": pad 7 played slice 3, the owner moved its
    /// start 2 ms on the SOURCE page; F3 on it shows its piece in the middle with half of
    /// it on each side, selects slice 3, where the piece starts, and keeps the cuts.
    #[test]
    fn open_on_your_own_piece() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = source_page(super::pads::Param::Start);
        let cuts = app.chop.as_ref().unwrap().markers.clone();
        pads_key(&mut app, KeyCode::Char('l'), M::NONE);
        pads_key(&mut app, KeyCode::Char('l'), M::NONE);
        assert_eq!(app.pads.source(6), super::pads::Source::Own);
        pads_key(&mut app, KeyCode::Esc, M::NONE);
        let piece = app.pads.region[6];
        pads_key(&mut app, KeyCode::F(3), M::NONE);
        assert_eq!(app.screen, Screen::Chop);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.markers, cuts, "the chop keeps its cuts");
        let half = (piece.end - piece.start) / 2;
        assert_eq!(c.view.start, piece.start - half, "half a piece before it");
        assert_eq!(c.view.end, piece.end + half, "half a piece after it");
        assert_eq!(c.selected, 2, "slice 3, where the piece starts");
    }

    /// 37: on a view opened on slice 5 the slice line says where slice 5 starts in the
    /// file, as the cursor does, not from the left edge of the picture.
    #[test]
    fn the_slice_start_is_in_the_file() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = break_on_bank_2();
        app.pads.bank = 1;
        app.pads.selected = 4; // slice 5
        pads_key(&mut app, KeyCode::F(3), M::NONE);
        let c = app.chop.as_ref().unwrap();
        let v = super::chop_view(c, None, None, 48_000, &app.pads);
        let s5 = c.slices()[4].start as f32 / 48_000.0;
        assert!(
            v.info.starts_with(&format!("slice 5 · start {s5:.3} s")),
            "{}",
            v.info
        );
    }

    /// 37 chopping "The source is marked": the bank of the whole-file pad wears ◆, and on
    /// its bank that pad says `◆ all` and is not dimmed.
    #[test]
    fn the_source_is_marked() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = break_on_bank_2(); // pad 1-1 plays the whole break
        app.pads.bank = 1;
        app.pads.selected = 4; // 2-Q, slice 5
        pads_key(&mut app, KeyCode::F(3), M::NONE);
        let view = |app: &super::App| {
            let c = app.chop.as_ref().unwrap();
            let v = super::chop_view(c, None, None, 48_000, &app.pads);
            (
                v.source_banks,
                v.pads.pads[0].note.clone(),
                v.pads.pads[0].dim,
            )
        };
        let (banks, _, _) = view(&app);
        let mut want = [false; tui::BANKS];
        want[0] = true;
        assert_eq!(banks, want);
        app.apply(Action::BankSelect(0));
        let (_, note, dim) = view(&app);
        assert_eq!(note.as_deref(), Some("◆ all"));
        assert!(!dim, "the source is not dimmed");
    }

    /// 46 "The whole file among its slices" and "Another file": on the chop screen the pad
    /// that plays the whole break has a dashed frame, its slice pads and a pad with another
    /// whole file solid ones.
    #[test]
    fn the_source_pad_stands_apart() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = break_on_bank_2(); // pad 1-1 whole, slices on 2-1…2-8
        app.pads.file[1] = Some("/other.wav".into()); // pad 1-2: the whole of another file
        app.pads.bank = 1;
        app.pads.selected = 0;
        pads_key(&mut app, KeyCode::F(3), M::NONE); // opens the break on its slice 1
        let view = |app: &super::App| {
            let c = app.chop.as_ref().unwrap();
            super::chop_view(c, None, None, 48_000, &app.pads).pads.pads
        };
        assert!(
            view(&app).iter().all(|v| !v.dashed),
            "bank 2: slices, solid"
        );
        app.apply(Action::BankSelect(0));
        let bank1 = view(&app);
        assert!(bank1[0].dashed, "pad 1-1, the whole break: dashed");
        assert!(!bank1[1].dashed, "pad 1-2, another file: solid");
    }

    /// 32 "The plan shows the grid": with the pads hidden the chop grid is hidden in the
    /// listen mode, shown while the assign mode is open, hidden again after it.
    #[test]
    fn the_plan_shows_the_grid() {
        let mut app = chop_from_pad(6);
        app.ui.hide_pads = true;
        let hidden = |app: &super::App| {
            let c = app.chop.as_ref().unwrap();
            let view = super::chop_view(c, app.plan.as_ref(), None, 48_000, &app.pads);
            app.mark_chop(view, None).pads.hidden
        };
        assert!(hidden(&app), "listen: hidden");
        app.apply(Action::ChopAssign);
        assert!(app.plan.is_some(), "the assign mode is open");
        assert!(!hidden(&app), "assign: the grid with the plan");
        app.apply(Action::Back);
        assert!(app.plan.is_none());
        assert!(hidden(&app), "closed: hidden again");
    }

    /// 32 chopping: "Selected slice with the pads hidden" — a hit selects the slice, and the
    /// pad line gets the pad and its slice (the line itself is drawn by the tui).
    #[test]
    fn selected_slice_with_the_pads_hidden() {
        let mut app = chop_from_pad(6); // slices 1–8 on Q W E R A S D F of bank 1
        app.ui.hide_pads = true;
        app.apply(Action::PadPress {
            pad: 5,
            velocity: 1.0,
        });
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.selected, 1, "pad W holds slice 2");
        let view = super::chop_view(c, None, None, 48_000, &app.pads);
        let v = app.mark_chop(view, None);
        assert!(v.pads.hidden);
        assert_eq!(v.pads.selected, 5);
        assert_eq!(v.pads.pads[5].note.as_deref(), Some("slice 2"));
    }

    #[test]
    fn change_of_mind() {
        let mut app = test_app();
        app.screen = Screen::Files;
        app.browser.in_hand = Some("/tmp/amen.wav".into());
        app.apply(Action::Back);
        assert!(app.browser.in_hand.is_none());
        app.sent.clear();
        hit(&mut app, 0);
        assert!(app.pads.file.iter().all(Option::is_none), "no pad changes");
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::PadPress { pad: 0, .. })),
            "the hit plays: {:?}",
            app.sent
        );
    }

    /// pad-assignment "File is gone": an assigned file deleted from disk marks its pad
    /// missing when the session opens again; the other pad keeps its sound. (A new file that
    /// fails to load leaves the pad as it was: nothing was assigned yet.)
    #[test]
    fn file_is_gone() {
        let dir = tmp_dir("gone");
        let mut app = test_app();
        for pad in [0, 1] {
            app.keep_recording(&ring(), false, &dir);
            let file = app.browser.in_hand.take().unwrap();
            app.put_file(pad, file);
        }
        wait_for(&mut app, |a| {
            a.pads.sound[0].is_some() && a.pads.sound[1].is_some()
        });
        let sess = app.pads.to_session(None, app.rate);
        std::fs::remove_file(app.pads.file[0].clone().unwrap()).unwrap();
        app.apply_session(sess);
        assert!(app.pads.missing[0], "the deleted file: missing");
        wait_for(&mut app, |a| a.pads.sound[1].is_some());
        assert!(!app.pads.missing[1], "the other pad works");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// pad-assignment "Swap while loading": a pad still loading is not swapped, the
    /// status says to wait.
    #[test]
    fn swap_while_loading() {
        let mut app = test_app();
        app.screen = Screen::Pads;
        app.pads.file[0] = Some("a.wav".into());
        app.pads.file[1] = Some("b.wav".into());
        app.pads.loading[1] = Some(40);
        app.pads.selected = 0;
        app.apply(Action::PadTake);
        hit(&mut app, 1);
        assert_eq!(
            app.pads.file[0].as_deref(),
            Some(std::path::Path::new("a.wav"))
        );
        assert_eq!(
            app.pads.file[1].as_deref(),
            Some(std::path::Path::new("b.wav"))
        );
        assert_eq!(flash(&app), "Wait until loading finishes, then swap.");
    }

    /// pad-playback "New pad": a pad without a sample says "empty", its info names the
    /// test tone.
    #[test]
    fn new_pad() {
        let mut app = test_app();
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        assert_eq!(v.pads[0].name, "empty");
        assert_eq!(v.pads[0].detail.as_deref(), Some("empty · test tone A2"));
    }

    /// The chop screen open on 8 equal slices of a silent break, nothing laid out.
    fn chop_open() -> super::App {
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(3));
        app
    }

    fn hit(app: &mut super::App, pad: usize) {
        app.apply(Action::PadPress { pad, velocity: 1.0 });
    }

    /// 83a pad-playback "U and O turn banks on Pads" and "U and O turn banks during pad edit": U/O turn banks with edit open or closed, keeping the pad position; J/L and the arrows change values, not banks.
    #[test]
    fn u_and_o_turn_banks_on_pads() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        for editing in [false, true] {
            let mut app = test_app();
            app.pads.selected = 6; // pad E
            app.pads.file[6] = Some("/samples/kick.wav".into());
            app.edit = editing;
            let _ = editing; // the walk runs twice: edit open and closed
            pads_key(&mut app, KeyCode::Char('o'), M::NONE);
            assert_eq!((app.pads.bank, app.pads.selected), (1, 6));
            pads_key(&mut app, KeyCode::Char('u'), M::NONE);
            assert_eq!((app.pads.bank, app.pads.selected), (0, 6));
            // the Russian twins: щ is the physical O, г is the physical U
            pads_key(&mut app, KeyCode::Char('щ'), M::NONE);
            assert_eq!(app.pads.bank, 1, "the Russian O turns the bank");
            pads_key(&mut app, KeyCode::Char('г'), M::NONE);
            assert_eq!(app.pads.bank, 0, "the Russian U turns the bank back");
            let volume = app.pads.volume_db[6];
            pads_key(&mut app, KeyCode::Char('l'), M::NONE);
            assert_eq!(app.pads.bank, 0, "L changes no bank");
            assert_eq!(app.pads.volume_db[6], volume + 1, "L changes the value");
            app.edit = editing; // a value key may close it; keep the state
            pads_key(&mut app, KeyCode::Right, M::NONE);
            assert_eq!(app.pads.bank, 0, "Right changes no bank");
        }
    }

    /// 83a pad-playback "Value keys work with pad edit closed" and "Value arrows work with pad edit closed": J/L and Left/Right change the selected value with the edit closed, without turning banks or picking another pad.
    #[test]
    fn value_keys_work_with_pad_edit_closed() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.pads.file[0] = Some("/samples/kick.wav".into());
        app.pads.selected = 0;
        let volume = app.pads.volume_db[0];
        pads_key(&mut app, KeyCode::Char('l'), M::NONE);
        assert_eq!(app.pads.volume_db[0], volume + 1);
        assert_eq!(app.pads.selected, 0, "no other pad is picked");
        assert_eq!(app.pads.bank, 0);
        pads_key(&mut app, KeyCode::Char('j'), M::NONE);
        assert_eq!(app.pads.volume_db[0], volume);
        pads_key(&mut app, KeyCode::Right, M::NONE);
        assert_eq!(app.pads.volume_db[0], volume + 1);
        pads_key(&mut app, KeyCode::Left, M::NONE);
        assert_eq!(app.pads.volume_db[0], volume);
    }

    /// 83a pad-playback "Undo one pad copy": one Ctrl+Z restores the target's full
    /// previous state in one step, without changing its source.
    #[test]
    fn undo_one_pad_copy() {
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.selected = 0;
        // something on W too: the undo restores its full previous state
        app.pads.sound[1] = app.pads.sound[0].clone();
        app.pads.file[1] = Some("snare.wav".into());
        app.pads.volume_db[1] = -6;
        let was = app.pads.snapshot(1);
        app.apply(Action::PadCopy);
        hit(&mut app, 1);
        // W is occupied: the question names it; Enter confirms, Esc would keep it
        app.pads.selected = 1;
        app.apply(Action::PadEdit); // Enter
        assert!(app.pads.file[1].is_some(), "the copy was placed");
        app.apply(Action::Undo);
        assert_eq!(app.pads.snapshot(1), was, "W's previous state is back");
        assert_eq!(app.pads.custom_name[0].as_deref(), Some("Бочка"), "Q stays");
        // one step: a second Ctrl+Z says there is nothing more
        app.apply(Action::Undo);
        assert_eq!(flash(&app), "nothing to undo");
    }

    /// 83a pad-assignment "Confirm a pad copy replacement" and "Decline a pad copy
    /// replacement": Enter places only the named target, Esc keeps the target and the
    /// copy stays in hand.
    #[test]
    fn confirm_and_decline_a_pad_copy_replacement() {
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.sound[1] = app.pads.sound[0].clone();
        app.pads.file[1] = Some("snare.wav".into());
        app.pads.selected = 0;
        let was = app.pads.snapshot(1);
        app.apply(Action::PadCopy);
        hit(&mut app, 1); // W, occupied: the question
        // Decline: the target stays, the copy remains in hand
        app.apply(Action::Back);
        assert_eq!(app.pads.snapshot(1), was);
        assert!(app.copy_in_hand.is_some(), "the copy stays in hand");
        // Confirm: Esc dropped nothing; a new hit, then Enter places it
        hit(&mut app, 1);
        app.pads.selected = 1;
        app.apply(Action::PadEdit); // Enter
        assert_eq!(app.pads.custom_name[1].as_deref(), Some("Бочка (copy 1)"));
        assert!(app.pads.file[0].is_some(), "the source stays");
    }

    /// 83a pad-assignment "A copied pad survives restart" and "Copy in hand is visible without a mode redesign": the placed copy saves into the session and comes back; the help line names the next step while a copy is in hand, on the same band.
    #[test]
    fn a_copied_pad_survives_restart() {
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.selected = 0;
        app.apply(Action::PadCopy);
        // the help line names the copy step
        assert_eq!(app.help_state().hand, Some("hit a pad to copy it there"));
        hit(&mut app, 1);
        assert_eq!(app.pads.custom_name[1].as_deref(), Some("Бочка (copy 1)"));
        // save and reload: the session keeps the copy's name, link and settings
        let session = app.pads.to_session(None, app.rate);
        let mut again = test_app();
        again.pads.restore(&session, app.rate);
        assert_eq!(again.pads.custom_name[1].as_deref(), Some("Бочка (copy 1)"));
        assert_eq!(again.pads.slice[1], Some(3), "the slice link saves");
        assert_eq!(again.pads.link[1], app.pads.link[1]);
        assert_eq!(again.pads.volume_db[1], app.pads.volume_db[1]);
    }

    /// 83a keyboard "Ctrl C copies instead of quitting": Ctrl+C on Pads in raw mode
    /// resolves `pad.copy`, requests no quit, and Ctrl+Q stays the quit command.
    #[test]
    fn ctrl_c_copies_instead_of_quitting() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.selected = 0;
        let mut input = NoticeInput::default();
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let ctrl_c = ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('c'), M::CONTROL);
        // raw mode with and without the extended protocol: Ctrl is the modifier either way
        assert_eq!(
            keymap.resolve_in(Screen::Pads, app.edit_keys(), &ctrl_c),
            Some(Action::PadCopy)
        );
        let ru = ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('с'), M::CONTROL);
        assert_eq!(
            keymap.resolve_in(Screen::Pads, app.edit_keys(), &ru),
            Some(Action::PadCopy),
            "the Russian C key copies too"
        );
        // through the production routing: no quit, the copy is taken
        let quit_before = app.apply(Action::Quit);
        assert!(quit_before, "quit works when asked");
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.selected = 0;
        assert!(!input.dispatch(&mut app, &keymap, ctrl_c, true), "no quit");
        assert!(app.copy_in_hand.is_some(), "the copy is taken");
        // Ctrl+Q stays the quit command
        let ctrl_q = ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('q'), M::CONTROL);
        assert!(
            input.dispatch(&mut app, &keymap, ctrl_q, true),
            "Ctrl+Q quits"
        );
    }

    /// Keyboard "Future commands stay unbound": Ctrl+R records A1, Ctrl+Y and
    /// Ctrl+. stay free, and Ctrl+B keeps the Files favorite.
    #[test]
    fn future_commands_stay_unbound() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let ctrl = |c: char, screen| {
            keymap.resolve(
                screen,
                &ratatui::crossterm::event::KeyEvent::new(KeyCode::Char(c), M::CONTROL),
            )
        };
        for screen in [
            Screen::Pads,
            Screen::Chop,
            Screen::Files,
            Screen::Fx,
            Screen::Settings,
        ] {
            assert_eq!(
                ctrl('r', screen),
                Some(Action::PatternRecord),
                "Ctrl+R records A1 ({screen:?})"
            );
            assert_eq!(ctrl('y', screen), None, "Ctrl+Y stays free ({screen:?})");
            assert_eq!(ctrl('.', screen), None, "Ctrl+. stays free ({screen:?})");
        }
        assert_eq!(
            ctrl('b', Screen::Files),
            Some(Action::BrowserFavorite),
            "the favorite keeps its key"
        );
    }

    /// 83a keyboard "Shipped modified chords have one role": every Ctrl/Alt chord of the
    /// shipped map keeps one role across screens and edit layers, with the exceptions
    /// the map comments name; no Ctrl+Shift+letter command, no plain/Shift pad command.
    #[test]
    fn shipped_modified_chords_have_one_role() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let mut roles: std::collections::HashMap<(char, bool), String> = Default::default();
        for screen in [
            input::Screen::Pads,
            input::Screen::Chop,
            input::Screen::Files,
            input::Screen::Fx,
            input::Screen::Settings,
        ] {
            for (k, a) in keymap.bindings(screen) {
                let Some(rest) = k.strip_prefix("Ctrl+").or_else(|| k.strip_prefix("Alt+")) else {
                    continue;
                };
                if rest.chars().count() != 1 {
                    continue;
                }
                let c = rest.chars().next().unwrap();
                let role = super::ctrl_text(a);
                let key = (c, k.starts_with("Alt+"));
                roles
                    .entry(key)
                    .and_modify(|was| assert_eq!(was, &role, "{k} has one role"))
                    .or_insert(role);
            }
        }
        // no Ctrl+Shift+letter on the shipped map, no command on the pad block
        let pad = "1234QWERASDFZXCV";
        for screen in [
            input::Screen::Pads,
            input::Screen::Chop,
            input::Screen::Settings,
        ] {
            for (k, a) in keymap.bindings(screen) {
                let fine_tempo = matches!(a, Action::TempoStep { fine: true, .. })
                    && matches!(k.as_str(), "Ctrl+Shift+↑" | "Ctrl+Shift+↓");
                assert!(
                    fine_tempo || (!k.starts_with("Ctrl+Shift+") && !k.starts_with("Alt+Shift+")),
                    "no double-modifier letter command: {k}"
                );
                if k.chars().count() == 1 && pad.contains(k.as_str()) {
                    panic!("a command on the pad block: {k} = {a:?}");
                }
            }
        }
    }

    /// 83a screen-help "Unbound Shift Tab keeps its fallback": Settings has no previous
    /// page action; Shift+Tab falls back to Tab there, and its `none` blocks that.
    #[test]
    fn unbound_shift_tab_keeps_its_fallback() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let tab = ratatui::crossterm::event::KeyEvent::new(KeyCode::Tab, M::NONE);
        let shift_tab = ratatui::crossterm::event::KeyEvent::new(KeyCode::Tab, M::SHIFT);
        let back_tab = ratatui::crossterm::event::KeyEvent::new(KeyCode::BackTab, M::SHIFT);
        let screen = Screen::Settings;
        assert_eq!(
            keymap.resolve_in(screen, false, &tab),
            Some(Action::SettingsTab)
        );
        // the fallback answers the shifted twins; no new navigation action is added
        for k in [shift_tab, back_tab] {
            assert_eq!(
                keymap.resolve_in(screen, false, &k),
                Some(Action::SettingsTab),
                "the Tab fallback stays"
            );
        }
        // an explicit none blocks the fallback
        let (mut k2, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        k2.overlay("[screens.settings]\n\"Shift+Tab\" = \"none\"\n")
            .unwrap();
        let back = ratatui::crossterm::event::KeyEvent::new(KeyCode::BackTab, M::SHIFT);
        assert_eq!(
            k2.resolve_in(screen, false, &back),
            None,
            "the none blocks it"
        );
    }

    /// 83a screen-help "Chop help names the moved commands", "Pads help names values and new bank keys", "Shift help names reverse hit chopping" and "Microphone help follows release support": the full help names
    /// H and Shift+H on Chop, U/O and the value keys on Pads, and the Shift view on Chop
    /// names the reverse hit command.
    #[test]
    fn chop_help_names_the_moved_commands() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        // the chop line of F1
        let chop = super::full_help(&keymap, input::Screen::Chop, true);
        assert!(
            chop.iter().any(|(k, t)| k == "H" && t.contains("by hits")),
            "the moved hit chop is named: {chop:?}"
        );
        assert!(
            chop.iter()
                .any(|(k, t)| k == "Shift+H" && t.contains("step back")),
            "the reverse hit chop is named: {chop:?}"
        );
        assert!(
            chop.iter()
                .any(|(k, t)| k == "Ctrl+G" && t.contains("−1 dB")),
            "the moved normalize is named: {chop:?}"
        );
        // the pads line names the value keys and U/O banks
        let pads = super::full_help(&keymap, input::Screen::Pads, true);
        assert!(
            pads.iter()
                .any(|(k, t)| k.contains('U') && k.contains('O') && t.contains("bank")),
            "{pads:?}"
        );
        assert!(
            pads.iter()
                .any(|(k, t)| k.contains("J ← L →") && t.contains("value")),
            "the value keys are named: {pads:?}"
        );
        // the Shift view on Chop names the reverse hit command (35: a Shift command
        // names its chord; the pad velocity hint stays)
        let shift =
            super::modifier_groups(&keymap, input::Screen::Chop, input::Modifier::Shift, false);
        assert_eq!(shift.len(), 1, "the Shift group");
        assert_eq!(shift[0].name, "SHIFT");
        let items: Vec<(String, String)> = shift[0].items.clone();
        assert!(
            items
                .iter()
                .any(|(k, t)| k == "Shift+H" && t.contains("fewer hit cuts")),
            "the reverse hit command is named, not the unshifted one: {items:?}"
        );
        assert!(
            items.iter().any(|(k, _)| k == "1-4 Q-R A-F Z-V"),
            "the full-velocity pads stay: {items:?}"
        );
    }

    /// 83a screen-help "Help respects an owner recording key" and "Microphone help follows
    /// release support": the full help names the microphone chord the owner bound, and
    /// its text follows whether the terminal sends release events.
    #[test]
    fn help_respects_an_owner_recording_key() {
        let (mut keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        keymap
            .overlay(
                "[global]
\"Ctrl+K\" = \"record.mic\"
",
            )
            .unwrap();
        let rows = super::full_help(&keymap, input::Screen::Pads, true);
        // the owner's chord joins the shipped one in one hold row
        assert!(
            rows.iter().any(|(k, t)| k.contains("Ctrl+K")
                && k.contains("Alt+R")
                && t.contains("microphone")
                && t.contains("hold")),
            "{rows:?}"
        );
    }

    /// 83a recording "Hold and speak", "Microphone toggle without release events",
    /// "Holding the moved microphone key does not toggle", "No microphone" and "One
    /// recording at a time" with the shipped Alt+R: a terminal without releases toggles
    /// on each press, repeats do not stop, and no input ports say so.
    #[test]
    fn microphone_toggle_without_release_events() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let alt = |kind| ratatui::crossterm::event::KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: M::ALT,
            kind,
            state: ratatui::crossterm::event::KeyEventState::NONE,
        };
        use ratatui::crossterm::event::KeyEventKind as K;
        // without a sound card the press names the trouble: the "No sound" case (26)
        let mut app = test_app();
        app.screen = Screen::Pads;
        let mut input = NoticeInput::default();
        input.event(&mut app, &keymap, alt(K::Press), true);
        assert_eq!(
            flash(&app),
            "No sound: there is nothing to record from.",
            "a press without a sound card starts nothing"
        );
        input.event(&mut app, &keymap, alt(K::Repeat), true);
        assert_eq!(app.recording, None, "a repeat starts nothing either");
        input.event(&mut app, &keymap, alt(K::Release), true);
        assert_eq!(app.recording, None, "a release saves no ring");
        // the toggle logic itself: without releases every press toggles (26)
        use super::{Rec, record_step};
        assert_eq!(
            record_step(Action::RecordMic(true), None),
            Some(Rec::Start(super::audio::SOURCE_MIC))
        );
        assert_eq!(
            record_step(Action::RecordMic(true), Some(super::audio::SOURCE_MIC)),
            Some(Rec::Stop),
            "the next press toggles it off without releases"
        );
        // an output recording goes on: one recording at a time
        assert_eq!(
            record_step(Action::RecordMic(true), Some(super::audio::SOURCE_OUTPUT)),
            None,
            "the output recording goes on"
        );
    }

    /// 83a chopping "New hit keys stay deaf while assigning" and "Assign order does not edit a listening slice": H, Shift+H and Tab do nothing while assigning and in listen mode outside a plan.
    #[test]
    fn new_hit_keys_stay_deaf_while_assigning() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(4));
        // listen mode, no plan: Tab edits nothing
        let slices = app.chop.as_ref().unwrap().slices().to_vec();
        chop_key(&mut app, KeyCode::Tab, M::NONE);
        assert_eq!(app.plan, None, "assign order does nothing in listen");
        assert_eq!(app.chop.as_ref().unwrap().slices().to_vec(), slices);
        // a plan open: the hit keys are deaf
        app.apply(Action::ChopAssign);
        let before = app.chop.as_ref().unwrap().slices().to_vec();
        chop_key(&mut app, KeyCode::Char('h'), M::NONE);
        assert_eq!(
            app.chop.as_ref().unwrap().slices().to_vec(),
            before,
            "H is deaf while assigning"
        );
        chop_key(&mut app, KeyCode::Char('h'), M::SHIFT);
        assert_eq!(
            app.chop.as_ref().unwrap().slices().to_vec(),
            before,
            "Shift+H is deaf while assigning"
        );
        assert_eq!(flash(&app), "assigning: Enter applies, Esc leaves");
    }

    /// 83a midi-learn "Cleanup does not reset X Session controls" and "Cleanup learned
    /// bindings survive restart": an initialized X-Session keeps its roles, the new
    /// cleanup bindings save into `learned` and come back after a reload.
    #[test]
    fn cleanup_learned_bindings_survive_restart() {
        let app = xsession_connected(&[
            (input::Group::Chop, 50, "chop.assign_order"),
            (input::Group::Chop, 51, "chop.hits_back"),
            (input::Group::Pads, 30, "pad.copy"),
        ]);
        // the seeded X-Session roles stay: no reset happened
        let p = app
            .live_profiles()
            .into_iter()
            .find(|p| p.name == "X-Session Pro")
            .unwrap();
        assert_eq!(
            p.assignment(input::midi::Scope::Screen(Screen::Pads), "pad.edit_only")
                .as_deref(),
            Some("b7 (note 58)"),
            "the seeded role stays"
        );
        // the cleanup bindings resolve to their actions
        let msg = |note: u8| input::midi::Msg::NoteOn {
            ch: 1,
            note,
            velocity: 100,
        };
        let mut app2 = xsession_connected(&[
            (input::Group::Chop, 50, "chop.assign_order"),
            (input::Group::Pads, 30, "pad.copy"),
        ]);
        app2.screen = Screen::Chop;
        app2.chop = Some(chop_with_parts(2));
        assert_eq!(xs_live(&app2, msg(50)), Some(Action::ChopAssignOrder));
        app2.screen = Screen::Pads;
        assert_eq!(xs_live(&app2, msg(30)), Some(Action::PadCopy));
    }

    /// 83a midi-learn "Learn a button for pad copy": a learned button takes the same copy
    /// as Ctrl+C, its release takes no second copy and plays no pad.
    #[test]
    fn learn_a_button_for_pad_copy() {
        let mut app = xsession_connected(&[(input::Group::Pads, 30, "pad.copy")]);
        app.screen = Screen::Pads;
        loaded_q(&mut app);
        app.pads.selected = 0;
        let msg = input::midi::Msg::NoteOn {
            ch: 1,
            note: 30,
            velocity: 100,
        };
        assert_eq!(xs_live(&app, msg), Some(Action::PadCopy));
        let action = xs_live(&app, msg).unwrap();
        app.apply(action);
        assert_eq!(flash(&app), "COPY · hit a pad");
        // the release adds nothing: a second press takes a fresh copy
        let msg = input::midi::Msg::NoteOff { ch: 1, note: 30 };
        assert_eq!(xs_live(&app, msg), None);
        assert!(app.copy_in_hand.is_some(), "the copy stays in hand");
        // a target from the controller gets it
        app.midi_event(super::midi::Event::PadHit {
            abs: 1,
            sound: super::midi::PadSound::DirectAccepted,
            velocity: 1.0,
            ui_press: None,
        });
        assert_eq!(app.pads.custom_name[1].as_deref(), Some("Бочка (copy 1)"));
        assert!(app.copy_in_hand.is_none());
    }

    /// 83a pad-assignment "The copy sounds like the source": the source and the copy
    /// render the same with matched velocity; the undone target restores its sound →
    /// `renders/83a-pad-copy.wav`.
    #[test]
    fn render_the_pad_copy() {
        use engine::Engine;
        use engine::checks::{first_non_finite, peak};
        use engine::offline::{Event, Scenario, render, write_wav};
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        use std::path::Path;

        let sr = 48_000;
        let ms = |n: usize| sr as usize / 1000 * n;
        let sound = std::sync::Arc::new(Sample::mono(
            (0..sr as usize * 2)
                .map(|i| (i as f32 * 220.0 * std::f32::consts::TAU / sr as f32).sin() * 0.5)
                .collect(),
        ));
        let mut app = test_app();
        app.screen = Screen::Pads;
        app.pads.sound[0] = Some(sound.clone());
        app.pads.file[0] = Some("kick.wav".into());
        app.pads.pitch[0] = 0;
        app.pads.selected = 0;
        // a real copy command: Ctrl+C, then a hit on empty W
        screen_key(&mut app, Screen::Pads, KeyCode::Char('c'), M::CONTROL);
        let snap = app.copy_in_hand.as_ref().map(|(_, s)| s.clone());
        assert!(snap.is_some(), "the copy is in hand");
        hit(&mut app, 1);
        let copy = app.pads.sound[1].clone().expect("the copy lies on W");
        assert_eq!(app.pads.pitch[1], app.pads.pitch[0], "the same settings");
        // the engine side: source and copy render the same, matched velocity
        let play = |snd: &Sample| {
            let events = vec![
                Event {
                    frame: 0,
                    cmd: Command::SetPadSound {
                        pad: 0,
                        sound: Some(std::sync::Arc::new(Sample {
                            left: snd.left.clone(),
                            right: snd.right.clone(),
                        })),
                    },
                },
                Event {
                    frame: ms(100),
                    cmd: Command::PadPress {
                        pad: 0,
                        velocity: 1.0,
                    },
                },
            ];
            let sc = Scenario {
                sample_rate: sr,
                frames: ms(600),
                events,
            };
            render(&mut Engine::new(sr), &sc, 128)
        };
        let (q_l, q_r) = play(&sound);
        let (w_l, w_r) = play(&copy);
        assert_eq!(q_l, w_l, "the copy sounds like the source");
        assert_eq!(q_r, w_r);
        assert_eq!(first_non_finite(&w_l), None);
        assert!(peak(&w_l) < 1.0, "no clipping");
        // the hit timing within one sample
        let onset = |l: &[f32]| l.iter().position(|x| x.abs() > 0.01).unwrap_or(usize::MAX);
        assert!((onset(&q_l) as isize - onset(&w_l) as isize).abs() <= 1);
        // the expected silence before the hit
        assert!(q_l[..ms(100)].iter().all(|x| x.abs() < 1e-6));
        // a real undo restores the target: the copy leaves W
        app.apply(Action::Undo);
        assert!(app.pads.file[1].is_none(), "the target is empty again");
        // one render with both voices: the source, then the copy
        let events = vec![
            Event {
                frame: 0,
                cmd: Command::SetPadSound {
                    pad: 0,
                    sound: Some(sound.clone()),
                },
            },
            Event {
                frame: 0,
                cmd: Command::SetPadSound {
                    pad: 1,
                    sound: Some(copy),
                },
            },
            Event {
                frame: ms(50),
                cmd: Command::PadPress {
                    pad: 0,
                    velocity: 1.0,
                },
            },
            // a softer hit for the copy: the two voices stay below full scale
            Event {
                frame: ms(300),
                cmd: Command::PadPress {
                    pad: 1,
                    velocity: 0.5,
                },
            },
        ];
        let sc = Scenario {
            sample_rate: sr,
            frames: ms(600),
            events,
        };
        let (l, r) = render(&mut Engine::new(sr), &sc, 128);
        assert_eq!(first_non_finite(&l), None);
        assert!(peak(&l) < 1.0, "no clipping");
        let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/83a-pad-copy.wav");
        write_wav(&out, sr, &l, &r).unwrap();
    }

    /// A pad loaded for the copy tests: bank 1, pad Q, with the settings the copy keeps.
    fn loaded_q(app: &mut super::App) {
        let wave: Vec<f32> = (0..48_000)
            .map(|i| (i as f32 * 440.0 / 48_000.0 * std::f32::consts::TAU).sin() * 0.8)
            .collect();
        app.pads.sound[0] = Some(std::sync::Arc::new(engine::Sample::mono(wave)));
        app.pads.file[0] = Some(std::path::PathBuf::from("kick.wav"));
        app.pads.custom_name[0] = Some("Бочка".into());
        app.pads.gate[0] = true;
        app.pads.volume_db[0] = -3;
        app.pads.pitch[0] = 2;
        app.pads.attack_ms[0] = 3;
        app.pads.sends[0] = [30, 20];
        app.pads.group[0] = 4;
        app.pads.slice[0] = Some(3);
        app.pads.link[0] = Some((1, 2));
    }

    /// A copy of the settings of pad `abs`, as `snapshot` has them.
    fn settings_of(app: &super::App, abs: usize) -> Vec<(String, String)> {
        let p = &app.pads;
        vec![
            ("volume".into(), p.volume_db[abs].to_string()),
            ("pitch".into(), p.pitch[abs].to_string()),
            ("gate".into(), p.gate[abs].to_string()),
            ("attack".into(), p.attack_ms[abs].to_string()),
            ("sends".into(), format!("{:?}", p.sends[abs])),
            ("group".into(), p.group[abs].to_string()),
            ("slice".into(), format!("{:?}", p.slice[abs])),
            ("link".into(), format!("{:?}", p.link[abs])),
            ("name".into(), format!("{:?}", p.custom_name[abs])),
        ]
    }

    /// 83a pad-assignment "Copy the selected pad with all settings": Ctrl+C takes a copy,
    /// a hit on an empty pad places it with a numbered copy name and plays nothing.
    #[test]
    fn copy_the_selected_pad_with_all_settings() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.selected = 0; // Q
        let q = settings_of(&app, 0);
        app.sent.clear();
        screen_key(&mut app, Screen::Pads, KeyCode::Char('c'), M::CONTROL);
        assert_eq!(flash(&app), "COPY · hit a pad");
        app.sent.clear();
        hit(&mut app, 1); // W
        assert!(
            !app.sent
                .iter()
                .any(|c| matches!(c, Command::PadPress { .. })),
            "the target hit does not play: {:?}",
            app.sent
        );
        // W keeps all Q's settings with the new numbered name.
        assert!(app.pads.file[1].is_some());
        assert_eq!(app.pads.custom_name[1].as_deref(), Some("Бочка (copy 1)"));
        assert!(app.copy_in_hand.is_none(), "the copy hand clears");
        let w = settings_of(&app, 1);
        // Only the name differs from the source settings.
        let q: Vec<_> = q.into_iter().filter(|(k, _)| k != "name").collect();
        let w: Vec<_> = w.into_iter().filter(|(k, _)| k != "name").collect();
        assert_eq!(q, w, "every setting but the name copies");
        // Q stays unchanged
        assert_eq!(app.pads.custom_name[0].as_deref(), Some("Бочка"));
        // one undo step for the target only: W comes back, Q stays
        app.apply(Action::Undo);
        assert!(app.pads.file[1].is_none(), "the target is back to empty");
        assert!(app.pads.file[0].is_some(), "the source stays");
    }

    /// 83a pad-assignment "Copy shares sample data": both pads point at the same sample.
    #[test]
    fn copy_shares_sample_data() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.selected = 0;
        screen_key(&mut app, Screen::Pads, KeyCode::Char('c'), M::CONTROL);
        hit(&mut app, 1);
        let (a, b) = (app.pads.sound[0].clone(), app.pads.sound[1].clone());
        assert!(matches!((a, b), (Some(x), Some(y)) if std::sync::Arc::ptr_eq(&x, &y)));
        let region = app.pads.region[0];
        assert_eq!(app.pads.region[1], region, "the bounds come along");
        assert_eq!(
            app.pads.link[1], app.pads.link[0],
            "the slice link comes along"
        );
    }

    /// 83a pad-assignment "Copy asks before replacing a pad", "Confirm a pad copy
    /// replacement" and "A changed copy target asks again".
    #[test]
    fn copy_asks_before_replacing_a_pad() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.selected = 0;
        loaded_q(&mut app); // Q again; put something on W too
        let wave: Vec<f32> = (0..48_000)
            .map(|i| (i as f32 * 220.0 / 48_000.0 * std::f32::consts::TAU).sin() * 0.7)
            .collect();
        app.pads.sound[1] = Some(std::sync::Arc::new(engine::Sample::mono(wave)));
        app.pads.file[1] = Some(std::path::PathBuf::from("snare.wav"));
        let was = app.pads.snapshot(1);
        screen_key(&mut app, Screen::Pads, KeyCode::Char('c'), M::CONTROL);
        hit(&mut app, 1); // W, occupied
        assert!(matches!(&app.ask, Some(super::Ask::CopyPad { .. })));
        // Esc keeps the target and the copy stays in hand
        app.apply(Action::Back);
        assert!(app.ask.is_none());
        assert_eq!(app.pads.snapshot(1), was, "the target stays");
        assert!(app.copy_in_hand.is_some(), "the copy stays in hand");
        // a hit on the source cancels
        hit(&mut app, 0);
        assert!(app.copy_in_hand.is_none(), "a hit on the source cancels");
        assert_eq!(app.pads.snapshot(1), was);
        // again: Enter confirms, only the named target is replaced
        screen_key(&mut app, Screen::Pads, KeyCode::Char('c'), M::CONTROL);
        hit(&mut app, 1);
        app.pads.selected = 1; // Enter answers about the selected pad
        app.apply(Action::PadEdit); // Enter
        assert!(app.ask.is_none(), "the question was answered");
        assert_eq!(app.pads.custom_name[1].as_deref(), Some("Бочка (copy 1)"));
        assert!(app.pads.file[0].is_some(), "the source stays");
        // a changed target asks again: a second Enter with a different selected pad
        // places nothing on it
        app.pads.sound[2] = app.pads.sound[0].clone();
        app.pads.file[2] = Some("hat.wav".into());
        screen_key(&mut app, Screen::Pads, KeyCode::Char('c'), M::CONTROL);
        hit(&mut app, 2); // occupied E: the question names E
        let before_e = app.pads.snapshot(2);
        app.pads.selected = 0; // the selection moved: Enter answers, the target changed
        app.apply(Action::PadEdit);
        assert!(
            matches!(&app.ask, Some(super::Ask::CopyPad { .. })),
            "it asks again"
        );
        assert_eq!(app.pads.snapshot(2), before_e, "nothing was replaced");
    }

    /// 83a pad-assignment "Esc drops a pending copy" and "Copy onto the source cancels".
    #[test]
    fn esc_drops_a_pending_copy() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.selected = 0;
        screen_key(&mut app, Screen::Pads, KeyCode::Char('c'), M::CONTROL);
        app.apply(Action::Back);
        assert!(app.copy_in_hand.is_none(), "Esc drops the copy");
        assert!(app.pads.file[1].is_none(), "no pad changed");
        assert!(app.history.is_empty(), "no undo step was added");
    }

    /// 83a pad-assignment "Copy refuses an unavailable source or loading target".
    #[test]
    fn copy_refuses_an_unavailable_source_or_loading_target() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.pads.selected = 0; // empty Q
        screen_key(&mut app, Screen::Pads, KeyCode::Char('c'), M::CONTROL);
        assert_eq!(flash(&app), "Copy what? This pad holds nothing.");
        assert!(app.copy_in_hand.is_none());
        // a loading source refuses
        loaded_q(&mut app);
        app.pads.loading[0] = Some(0);
        screen_key(&mut app, Screen::Pads, KeyCode::Char('c'), M::CONTROL);
        assert!(flash(&app).contains("Wait until loading finishes"));
        assert!(app.copy_in_hand.is_none());
        app.pads.loading[0] = None;
        // a copy in hand with a loading target waits
        screen_key(&mut app, Screen::Pads, KeyCode::Char('c'), M::CONTROL);
        assert!(app.copy_in_hand.is_some());
        app.pads.loading[1] = Some(0);
        hit(&mut app, 1);
        assert!(flash(&app).contains("Wait until loading finishes"));
        assert!(app.pads.file[1].is_none(), "nothing was placed");
        assert!(app.copy_in_hand.is_some(), "the copy waits");
    }

    /// 83a pad-assignment "Copy changes one target despite marks": the hit names one target.
    #[test]
    fn copy_changes_one_target_despite_marks() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.selected = 0;
        app.pads.marked = (0..super::PADS).collect();
        screen_key(&mut app, Screen::Pads, KeyCode::Char('c'), M::CONTROL);
        hit(&mut app, 1);
        assert_eq!(app.pads.custom_name[1].as_deref(), Some("Бочка (copy 1)"));
        assert!(app.pads.file[2].is_none(), "only W changed");
        assert!(!app.pads.marked.is_empty(), "the marks stay");
    }

    /// 83a pad-assignment "Copy keeps the settings from pickup": the snapshot is taken at
    /// Ctrl+C, later changes of the source do not change the pending copy.
    #[test]
    fn copy_keeps_the_settings_from_pickup() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.selected = 0;
        screen_key(&mut app, Screen::Pads, KeyCode::Char('c'), M::CONTROL);
        app.pads.pitch[0] = 9; // the source changes after the pickup
        hit(&mut app, 1);
        assert_eq!(app.pads.pitch[1], 2, "the pitch from pickup");
        assert_eq!(app.pads.pitch[0], 9, "the source keeps its change");
    }

    /// 83a pad-assignment "Copy target from a controller in another bank": a controller
    /// note places the copy without sound, and held releases keep working.
    #[test]
    fn copy_target_from_a_controller_in_another_bank() {
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.selected = 0;
        app.apply(Action::PadCopy);
        app.pads.bank = 1; // bank 2
        app.sent.clear();
        app.midi_event(super::midi::Event::PadHit {
            abs: 16 + 2, // bank 2, pad E
            sound: super::midi::PadSound::DirectAccepted,
            velocity: 1.0,
            ui_press: None,
        });
        assert!(
            !app.sent
                .iter()
                .any(|c| matches!(c, Command::PadPress { .. })),
            "the target hit plays nothing: {:?}",
            app.sent
        );
        assert_eq!(app.pads.custom_name[18].as_deref(), Some("Бочка (copy 1)"));
        assert!(app.pads.file[0].is_some(), "the source stays");
        assert!(app.copy_in_hand.is_none(), "the hand clears");
        // a release of a held pad still works: no crash, nothing else changes
        let before = app.pads.snapshot(4);
        app.midi_event(super::midi::Event::PadRelease {
            abs: 4,
            direct: false,
        });
        assert_eq!(app.pads.snapshot(4), before);
    }

    /// 83a midi-learn "Pick pad copy in screen learn" and "Learn a button for pad copy":
    /// Ctrl+C picks `pad.copy` and a learned button takes the same copy without sound.
    #[test]
    fn pick_pad_copy_in_screen_learn() {
        let mut app = test_app();
        loaded_q(&mut app);
        app.pads.selected = 0;
        app.apply(Action::LearnToggle);
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let k = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Char('c'),
            ratatui::crossterm::event::KeyModifiers::CONTROL,
        );
        assert!(app.learn_key(&keymap, &k), "the mode takes every key");
        assert_eq!(picked(&app).0, "pad.copy");
        assert!(app.copy_in_hand.is_none(), "learning takes no copy");
        assert!(app.pads.file[1].is_none(), "no pad changed");
    }

    /// The region of the last slice sent to the preview voice.
    fn last_played(app: &super::App) -> Option<Region> {
        app.sent.iter().rev().find_map(|c| match c {
            Command::Preview {
                sound: Some(_),
                region,
            } => Some(*region),
            _ => None,
        })
    }

    /// The slices sent to the preview voice, in order.
    fn played(app: &super::App) -> Vec<Region> {
        app.sent
            .iter()
            .filter_map(|c| match c {
                Command::Preview {
                    sound: Some(_),
                    region,
                } => Some(*region),
                _ => None,
            })
            .collect()
    }

    /// 21 chopping "The cursor selects": → into the next slice selects it and plays it
    /// once; a move inside it plays nothing more; the jump by cuts plays the slice too.
    #[test]
    fn the_cursor_selects() {
        let mut app = chop_open();
        let slices = app.chop.as_ref().unwrap().slices();
        app.chop.as_mut().unwrap().cursor = slices[1].start - 1;
        let right = Action::CursorMove {
            right: true,
            fine: true,
        };
        app.apply(right);
        assert_eq!(
            app.chop.as_ref().unwrap().selected,
            1,
            "the next slice is selected"
        );
        assert_eq!(played(&app), [slices[1]], "it plays once");
        app.apply(right);
        assert_eq!(played(&app), [slices[1]], "inside it: nothing more");
        app.apply(Action::CursorJump { right: true });
        assert_eq!(app.chop.as_ref().unwrap().selected, 2);
        assert_eq!(
            played(&app),
            [slices[1], slices[2]],
            "the jump plays the slice it lands in"
        );
    }

    /// 21 chopping "Select a slice": "next slice" from a controller selects the next slice
    /// and plays it once.
    #[test]
    fn select_a_slice() {
        let mut app = chop_open();
        let slices = app.chop.as_ref().unwrap().slices();
        app.apply(Action::MarkerSelect(1));
        assert_eq!(app.chop.as_ref().unwrap().selected, 1);
        assert_eq!(played(&app), [slices[1]]);
    }

    /// 21 input-devices "Lay out from the controller": the commit twice puts slices 1–8 on
    /// Q W E R A S D F of bank 1 from the selected Q; `P` still assigns by hand.
    #[test]
    fn lay_out_from_the_controller() {
        let mut app = chop_open();
        app.pads.selected = 4; // Q
        let map = [(input::midi::Key::Note(B9), "chop.commit")];
        learned(&mut app, &map, press(B9));
        assert!(
            matches!(app.plan, Some(super::chop::Plan::InOrder(Some(4)))),
            "in order from Q"
        );
        learned(&mut app, &map, press(B9));
        assert!(app.plan.is_none(), "applied");
        for slice in 0..8 {
            assert_eq!(
                app.pads.slice[4 + slice],
                Some(slice as u16 + 1),
                "slice {}",
                slice + 1
            );
        }
        let mut app = chop_open();
        app.apply(Action::ChopAssign);
        assert!(
            matches!(app.plan, Some(super::chop::Plan::ByHand(_))),
            "P: by hand"
        );
    }

    /// 35 chopping "Pick the slice for a pad": in the assign mode by hand J steps back to
    /// slice 2, a hit on C plans it there; the cuts stay.
    #[test]
    fn pick_the_slice_for_a_pad() {
        let mut app = chop_open();
        let cuts = app.chop.as_ref().unwrap().cut_ids().to_vec();
        app.apply(Action::ChopAssign);
        app.chop.as_mut().unwrap().select_slice(4); // slice 5
        let left = Action::CursorMove {
            right: false,
            fine: true,
        };
        for _ in 0..3 {
            app.apply(left);
        }
        assert_eq!(app.chop.as_ref().unwrap().selected, 1, "slice 2");
        let slices = app.chop.as_ref().unwrap().slices();
        assert_eq!(
            last_played(&app),
            Some(slices[1]),
            "the picked slice sounds"
        );
        hit(&mut app, 14); // C
        let targets = app.plan.as_ref().unwrap().targets(8);
        assert_eq!(targets, [(1, 14)], "slice 2 on pad C");
        assert_eq!(
            app.chop.as_ref().unwrap().cut_ids(),
            cuts,
            "the cuts did not move"
        );
        assert!(
            last_notice(&app).is_none() || flash(&app) != "assigning: Enter applies, Esc leaves"
        );
    }

    /// 35 chopping "A region over a laid-out break": the region start at slice 8 leaves
    /// the pads of slices 1–7 playing what they played (`pad`), the pad of slice 8 follows
    /// it; undo links them all back.
    #[test]
    fn a_region_over_a_laid_out_break() {
        let mut app = laid_out_break(); // slices 1–8 on Q W E R A S D F of bank 1 (4..12)
        let before: Vec<Region> = (4..12).map(|a| app.pads.region[a]).collect();
        let slices = app.chop.as_ref().unwrap().slices();
        app.chop.as_mut().unwrap().cursor = slices[7].start;
        app.apply(Action::ChopRegionStart);
        for (i, a) in (4..11).enumerate() {
            assert_eq!(app.pads.link[a], None, "pad {a} left out: no link");
            assert_eq!(
                app.pads.region[a], before[i],
                "pad {a} plays what it played"
            );
        }
        assert!(
            app.pads.link[11].is_some(),
            "the pad of slice 8 still follows"
        );
        assert_eq!(
            app.pads.slice[11],
            Some(1),
            "its slice is the first of the region"
        );
        assert_eq!(app.pads.region[11], before[7]);
        assert_eq!(flash(&app), "7 pads changed; Ctrl+Z returns");
        app.apply(Action::Undo);
        assert_eq!(flash(&app), "7 pads changed back");
        for (i, a) in (4..12).enumerate() {
            assert!(app.pads.link[a].is_some(), "pad {a} follows again");
            assert_eq!(app.pads.slice[a], Some(i as u16 + 1));
            assert_eq!(app.pads.region[a], before[i]);
        }
    }

    /// 35 chopping "Edit after an undo": the cursor stands in slice 8, a region is set and
    /// undone; the slice edit opens on slice 8, the slice under the cursor.
    #[test]
    fn edit_after_an_undo() {
        let mut app = chop_open();
        let slices = app.chop.as_ref().unwrap().slices();
        app.chop.as_mut().unwrap().cursor = slices[7].start + 100;
        app.apply(Action::ChopRegionStart);
        app.apply(Action::Undo);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.slices().len(), 8, "the region is undone");
        assert_eq!(c.selected, 7, "slice 8, under the cursor");
        app.apply(Action::ChopEdit);
        assert!(app.edit);
        assert_eq!(
            app.chop.as_ref().unwrap().selected,
            7,
            "the edit is on slice 8"
        );
    }

    /// 35 screen-help "The hint after a layout": the chop hint says the next step of the
    /// chop — start, lay out, move a bound — and nothing in the edit.
    #[test]
    fn the_hint_after_a_layout() {
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(0));
        assert!(
            super::hint_for(&app).unwrap().starts_with("Start with Y"),
            "no cuts yet"
        );
        let mut app = chop_open();
        assert!(
            super::hint_for(&app)
                .unwrap()
                .starts_with("P lays the slices"),
            "cut, not laid"
        );
        app.apply(Action::ChopCommit);
        app.apply(Action::ChopCommit); // laid in order from pad 1
        app.screen = Screen::Chop;
        let hint = super::hint_for(&app).unwrap();
        assert!(hint.contains("move a bound"), "{hint}");
        app.apply(Action::ChopEdit);
        assert_eq!(
            super::hint_for(&app),
            None,
            "the edit line speaks for itself"
        );
    }

    /// chopping "Empty pad": F3 on a pad without a sample keeps the screen and says so.
    #[test]
    fn empty_pad() {
        let mut app = test_app();
        app.screen = Screen::Pads;
        assert!(app.pads.file[app.pads.selected_abs()].is_none());
        app.apply(Action::ScreenChop);
        assert_eq!(app.screen, Screen::Pads, "the screen does not change");
        assert!(flash(&app).starts_with("pad is empty"), "{}", flash(&app));
    }

    /// chopping "Hear the plan": a hit in the assign mode plays the slice planned there.
    #[test]
    fn hear_the_plan() {
        let mut app = chop_open();
        app.apply(Action::ChopAssign);
        app.chop.as_mut().unwrap().select_slice(3);
        hit(&mut app, 0); // slice 4 goes on pad 1 and sounds
        let slices = app.chop.as_ref().unwrap().slices();
        assert_eq!(
            last_played(&app),
            Some(slices[3]),
            "the planned slice plays"
        );
        // a tap by hand puts the selected slice there: the next hit plans and plays that one
        app.sent.clear();
        hit(&mut app, 0);
        let planned = app.plan.as_ref().unwrap().targets(8);
        let on_pad1 = planned.iter().find(|t| t.1 == 0).unwrap().0;
        assert_eq!(
            last_played(&app),
            Some(slices[on_pad1]),
            "what is planned there plays"
        );
    }

    /// 04e chopping "All placed": with every slice planned, a hit plans nothing new and
    /// plays what is planned for that pad, if anything.
    #[test]
    fn all_placed() {
        let mut app = chop_open();
        app.apply(Action::ChopAssign);
        for pad in 0..8 {
            hit(&mut app, pad);
        }
        let targets = |app: &super::App| app.plan.as_ref().unwrap().targets(8);
        let before = targets(&app);
        assert_eq!(before.len(), 8);
        app.sent.clear();
        hit(&mut app, 9);
        assert_eq!(targets(&app), before, "nothing planned on pad 10");
        assert_eq!(flash(&app), "all 8 slices placed: Enter applies");
        assert_eq!(
            last_played(&app),
            None,
            "nothing is planned for pad 10: silence"
        );
        hit(&mut app, 2);
        let slice3 = app.chop.as_ref().unwrap().slices()[2];
        assert_eq!(last_played(&app), Some(slice3), "pad 3 plays its slice");
    }

    /// 04e chopping "Undo a tap": the last tap is undone and its slice selected again.
    #[test]
    fn undo_a_tap() {
        let mut app = chop_open();
        app.apply(Action::ChopAssign);
        hit(&mut app, 0);
        hit(&mut app, 1);
        assert_eq!(
            app.chop.as_ref().unwrap().selected,
            2,
            "the next slice is up"
        );
        app.apply(Action::Undo);
        let targets = app.plan.as_ref().unwrap().targets(8);
        assert_eq!(
            targets.len(),
            1,
            "slice 2 is no longer planned: {targets:?}"
        );
        assert_eq!(
            app.chop.as_ref().unwrap().selected,
            1,
            "slice 2 is selected again"
        );
    }

    /// chopping "Steps that can be undone": a cut, a plan and its undo ask nothing.
    #[test]
    fn steps_that_can_be_undone() {
        let mut app = chop_open();
        app.chop.as_mut().unwrap().cursor = 1_000;
        app.apply(Action::ChopCut);
        assert!(app.ask.is_none(), "a cut asks nothing");
        app.apply(Action::ChopAssign);
        hit(&mut app, 0);
        app.apply(Action::Undo);
        assert!(app.ask.is_none(), "a plan and its undo ask nothing");
    }

    /// chopping "Selected slice on the grid": with the pads shown, a hit on pad W in the
    /// listen mode selects it on the grid.
    #[test]
    fn selected_slice_on_the_grid() {
        let mut app = laid_out_break(); // slices 1–8 on Q W E R A S D F of bank 1
        hit(&mut app, 5);
        let c = app.chop.as_ref().unwrap();
        let v = app.mark_chop(super::chop_view(c, None, None, 48_000, &app.pads), None);
        assert!(!v.pads.hidden);
        assert_eq!(v.pads.selected, 5, "pad W is the selected cell");
    }

    /// chopping "Plain overwrite question": slice 3 planned on pad 3 of bank 1 that holds
    /// "kick" asks "bank 1, pad 3: kick → slice 3".
    #[test]
    fn plain_overwrite_question() {
        let mut app = chop_open();
        app.pads.file[2] = Some("kick.wav".into());
        app.apply(Action::ChopAssign);
        app.chop.as_mut().unwrap().select_slice(2);
        hit(&mut app, 2);
        app.apply(Action::ChopCommit);
        let Some(super::Ask::Plan { lines }) = &app.ask else {
            panic!("a question");
        };
        assert!(
            lines
                .iter()
                .any(|l| l.trim() == "bank 1, pad 3: kick → slice 3"),
            "{lines:?}"
        );
    }

    /// 27 chopping: "Open from a slice pad".
    /// Scenario "Open the whole break from a slice" (chopping).
    #[test]
    fn open_from_a_slice_pad() {
        let app = chop_from_pad(6);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.selected, 2, "slice 3 is selected");
        let v = super::chop_view(c, None, None, 48_000, &app.pads);
        // slices 1–8 lie on Q W E R A S D F of bank 1; pad 7 is E with slice 3
        assert_eq!(
            v.owners,
            ["1-Q", "1-W", "1-E", "1-R", "1-A", "1-S", "1-D", "1-F"],
            "every slice shows its pad"
        );
    }

    /// 04e chopping: "Chopped, nothing laid out" — a cut with no pad following it marks
    /// the session, and the written session keeps the cut for the next start.
    #[test]
    fn chopped_nothing_laid_out() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        let file = tmp_dir("chopped").join("session.toml");
        app.session_path = file.clone();
        app.session_read_error = None;
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(0));
        app.chop.as_mut().unwrap().set_cursor(30_000);
        assert!(app.session_dirty.is_none());
        chop_key(&mut app, KeyCode::Char('m'), M::NONE);
        assert!(app.session_dirty.is_some(), "the cut marks the session");
        app.save_session().unwrap();
        let s = super::session::load(&file).unwrap();
        let cuts: Vec<usize> = s.chops.values().map(|c| c.cuts.len()).collect();
        assert_eq!(cuts, [1], "the cut is kept with no pad on it");
        std::fs::remove_dir_all(file.parent().unwrap()).ok();
    }

    /// 04e chopping: "Assign the slice just heard".
    #[test]
    fn assign_the_slice_just_heard() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(3)); // 8 slices, none on a pad
        // slices on no pad are heard with the cursor (36): four jumps play slice 5
        for _ in 0..4 {
            app.apply(Action::CursorJump { right: true });
        }
        assert_eq!(app.chop.as_ref().unwrap().selected, 4);
        chop_key(&mut app, KeyCode::Char('p'), M::NONE);
        assert_eq!(
            app.chop.as_ref().unwrap().selected,
            4,
            "the assign starts there"
        );
        app.apply(Action::PadPress {
            pad: 4,
            velocity: 1.0,
        });
        chop_key(&mut app, KeyCode::Enter, M::NONE);
        assert_eq!(app.pads.slice[4], Some(5), "slice 5 lies on Q");
        let others = (0..super::PADS).filter(|&a| a != 4 && app.pads.file[a].is_some());
        assert_eq!(others.count(), 0, "no other pad changed");
    }

    /// 36 chopping: "Cut inside a slice on a pad" and "Undo the cut" — the pad keeps the
    /// left part, the slice that still starts at its cut, and says it is shorter.
    #[test]
    fn cut_inside_a_slice_on_a_pad() {
        use super::pads::Source;
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = laid_out_break(); // slice 1 on Q (abs 4), slice 2 on W (abs 5)
        let q = app.pads.region[4];
        let mid = q.start + (q.end - q.start) / 2;
        app.chop.as_mut().unwrap().set_cursor(mid);
        chop_key(&mut app, KeyCode::Char('m'), M::NONE);
        assert_eq!(
            app.pads.region[4],
            Region {
                start: q.start,
                end: mid
            },
            "Q plays from its start to the new cut"
        );
        assert_eq!(app.pads.source(4), Source::Slice(1), "Q stays linked");
        assert_eq!(panel_line(&mut app, 4), "break.wav · slice 1");
        assert_eq!(flash(&app), "pad 1-Q is shorter now; Ctrl+Z returns");
        assert_eq!(
            app.pads.source(5),
            Source::Slice(3),
            "W stays linked, one number up"
        );
        // undo the cut: Q follows slice 1 again
        chop_key(&mut app, KeyCode::Char('z'), M::CONTROL);
        assert_eq!(
            app.chop.as_ref().unwrap().cut_ids().len(),
            7,
            "the cut is gone"
        );
        assert_eq!(app.pads.source(4), Source::Slice(1));
        assert_eq!(app.pads.region[4], q);
        assert_eq!(app.pads.source(5), Source::Slice(2));
    }

    /// 36: the render for the owner — pad Q before the cut inside its slice and after it,
    /// back to back; after the cut Q plays only the left part.
    #[test]
    fn render_the_cut_inside() {
        use engine::offline::{Event, Scenario, render, write_wav};
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let tone: Vec<f32> = (0..96_000)
            .map(|i| (i as f32 * 220.0 / 48_000.0 * std::f32::consts::TAU).sin() * 0.5)
            .collect();
        let sound = std::sync::Arc::new(Sample::mono(tone));
        let mut app = laid_out_break();
        app.chop.as_mut().unwrap().sound = sound.clone();
        let q = app.pads.region[4];
        let mid = q.start + (q.end - q.start) / 2;
        app.chop.as_mut().unwrap().set_cursor(mid);
        chop_key(&mut app, KeyCode::Char('m'), M::NONE);
        let play = |region: Region| {
            let events = vec![
                Event {
                    frame: 0,
                    cmd: Command::SetPadSound {
                        pad: 0,
                        sound: Some(sound.clone()),
                    },
                },
                Event {
                    frame: 0,
                    cmd: Command::SetPadRegion { pad: 0, region },
                },
                Event {
                    frame: 128,
                    cmd: Command::PadPress {
                        pad: 0,
                        velocity: 1.0,
                    },
                },
            ];
            let sc = Scenario {
                sample_rate: 48_000,
                frames: 24_000,
                events,
            };
            render(&mut engine::Engine::new(48_000), &sc, 128)
        };
        let (before, after) = (play(q), play(app.pads.region[4]));
        // the left part sounds the same as the start of the whole slice up to the short
        // fade at its new end (no click), then it is silent
        let half = 128 + (mid - q.start);
        let fade = 48_000 * 10 / 1000;
        assert_eq!(
            before.0[..half - fade],
            after.0[..half - fade],
            "the left part is the start of the old piece"
        );
        assert!(
            after.0[half + fade..].iter().all(|x| x.abs() < 1e-6),
            "silent after the new cut"
        );
        assert!(
            before.0[half + fade..].iter().any(|x| x.abs() > 0.1),
            "the whole piece went on"
        );
        assert!(engine::checks::first_non_finite(&after.0).is_none());
        assert!(engine::checks::peak(&after.0) < 1.0, "no clipping");
        assert!(
            engine::checks::peak(&before.0) > 0.1,
            "the pad sounds, not silence"
        );
        let left: Vec<f32> = before.0.iter().chain(&after.0).copied().collect();
        let right: Vec<f32> = before.1.iter().chain(&after.1).copied().collect();
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../renders/36-cut-keeps-left.wav");
        write_wav(&out, 48_000, &left, &right).unwrap();
    }

    /// 04e chopping: "Two pads hold the slice".
    #[test]
    fn two_pads_hold_the_slice() {
        let mut app = laid_out_break(); // slice 3 on E of bank 1 (abs 6)
        let c = app.chop.as_ref().unwrap();
        app.pads.file[16 + 5] = Some(c.file.clone()); // W of bank 2
        app.pads.link[16 + 5] = Some((c.start_id(2), c.epoch));
        app.pads.slice[16 + 5] = Some(3);
        app.chop.as_mut().unwrap().select_slice(2);
        let c = app.chop.as_ref().unwrap();
        let v = super::chop_view(c, None, None, 48_000, &app.pads);
        assert!(v.info.ends_with(" · on 1-E, 2-W"), "{}", v.info);
        app.chop.as_mut().unwrap().select_slice(0);
        let c = app.chop.as_ref().unwrap();
        let v = super::chop_view(c, None, None, 48_000, &app.pads);
        assert!(v.info.ends_with(" · on 1-Q"), "{}", v.info);
    }

    /// 32 screen-help: "The help names the state" — under the held Ctrl the help line says
    /// `O pads: hidden` or `pads: shown`; F1 lists the switch too.
    #[test]
    fn the_help_names_the_state() {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        for screen in [
            input::Screen::Pads,
            input::Screen::Files,
            input::Screen::Chop,
        ] {
            for (hidden, text) in [(true, "pads: hidden"), (false, "pads: shown")] {
                let items = line(&super::modifier_groups(
                    &keymap,
                    screen,
                    input::Modifier::Ctrl,
                    hidden,
                ));
                assert!(
                    items.contains(&("O".into(), text.into())),
                    "{screen:?} {text}: {items:?}"
                );
            }
            let full = super::full_help(&keymap, screen, true);
            assert!(
                full.iter()
                    .any(|(k, d)| k.contains("Ctrl+O") && d == "hide / show the pads"),
                "{screen:?}: F1 lists the switch"
            );
        }
    }

    /// 32 "The choice survives a restart": Ctrl+O hides the pads and the choice is in the
    /// config for the next start; a config without it shows the pads.
    #[test]
    fn the_pads_choice_survives_a_restart() {
        let old: super::config::Config = toml::from_str("[ui]\ngraphics = true\n").unwrap();
        assert!(
            !old.ui.hide_pads,
            "a config without the field shows the pads"
        );
        let mut app = test_app();
        let file = tmp_dir("hide-pads").join("config.toml");
        app.user_path = Some(file.clone());
        let saved = || -> super::config::Config {
            toml::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap()
        };
        app.apply(Action::PadsToggle);
        assert!(app.ui.hide_pads);
        assert!(saved().ui.hide_pads, "kept for the next start");
        app.apply(Action::PadsToggle);
        assert!(!app.ui.hide_pads);
        assert!(!saved().ui.hide_pads, "shown again, and that is kept too");
        std::fs::remove_dir_all(file.parent().unwrap()).ok();
    }

    /// 58 input-devices: "Pick pickup" — the knob response row of the MIDI tab steps the
    /// response, the next knob turn follows it, and the config keeps it.
    #[test]
    fn pick_pickup() {
        use input::midi::Response;
        let mut app = test_app();
        let file = tmp_dir("knobs").join("config.toml");
        app.user_path = Some(file.clone());
        midi_tab(&mut app);
        app.midi_row = 0;
        assert!(matches!(
            app.midi_list().first(),
            Some(super::MidiListRow::Response)
        ));
        app.apply(Action::SettingsValue(1));
        assert_eq!(app.takeover.response, Response::Pickup);
        let saved: super::config::Config =
            toml::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!(saved.knobs.response, "pickup", "kept for the next start");
        assert_eq!(
            Response::from_name(&saved.knobs.response),
            Some(Response::Pickup)
        );
        // the next knob turn waits for the value
        let abs = app.pads.selected_abs();
        app.apply(Action::ScreenPads);
        xsession(&mut app, knob(K1, 127));
        assert_eq!(app.pads.volume_db[abs], 0, "pickup waits for the value");
        // PgDn from the row goes to the first group
        app.apply(Action::ScreenSettings);
        app.settings_tab = 2;
        app.midi_row = 0;
        app.apply(Action::BankNext);
        assert!(matches!(
            app.midi_list().get(app.midi_row),
            Some(super::MidiListRow::Group { .. })
        ));
        std::fs::remove_dir_all(file.parent().unwrap()).ok();
    }

    /// 58 input-devices: "An older config" — no knob response reads as jump.
    #[test]
    fn an_older_config_jumps() {
        let old: super::config::Config = toml::from_str("[audio]\nbuffer_size = 0\n").unwrap();
        assert_eq!(
            input::midi::Response::from_name(&old.knobs.response).unwrap_or_default(),
            input::midi::Response::Jump
        );
        assert!(test_app().takeover.response == input::midi::Response::Jump);
    }

    /// 04b chopping: "Braille by default" and "The picture switched on" — the switch on
    /// F12 is kept in the config for the next start; a config without it means Braille.
    #[test]
    fn braille_by_default_and_the_picture_switched_on() {
        let old: super::config::Config = toml::from_str("[audio]\nbuffer_size = 0\n").unwrap();
        assert!(!old.ui.graphics, "a config without [ui] draws Braille");
        let mut app = test_app();
        assert!(!app.ui.graphics);
        let file = tmp_dir("wave").join("config.toml");
        app.user_path = Some(file.clone());
        app.apply(Action::ScreenSettings);
        for _ in 0..3 {
            app.apply(Action::SettingsRow(1));
        }
        app.apply(Action::SettingsValue(1));
        assert!(app.ui.graphics, "Wave: picture");
        let saved: super::config::Config =
            toml::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        assert!(saved.ui.graphics, "kept for the next start");
        assert_eq!(flash(&app), "wave: picture from the next start");
        std::fs::remove_dir_all(file.parent().unwrap()).ok();
    }

    /// The chop screen as a 100×30 terminal draws it, with the open plan.
    fn chop_screen(app: &super::App) -> ratatui::buffer::Buffer {
        let c = app.chop.as_ref().unwrap();
        let view = super::chop_view(c, app.plan.as_ref(), None, 48_000, &app.pads);
        let status = tui::Status {
            badges: app.title_badges(),
            save: app.save_status(),
            backend: Some("jack"),
            recording: false,
            buffer_size: 128,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0; 2], [-60.0; 2], false),
            notice: None,
            no_releases: false,
            project: None,
        };
        let help = tui::Help {
            groups: &[],
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(100, 30)).unwrap();
        term.draw(|f| tui::draw(f, &status, &tui::Screen::Chop(Box::new(view)), &help))
            .unwrap();
        term.backend().buffer().clone()
    }

    fn screen_rows(buf: &ratatui::buffer::Buffer) -> Vec<String> {
        let w = buf.area.width as usize;
        buf.content()
            .chunks(w)
            .map(|row| row.iter().map(|c| c.symbol()).collect())
            .collect()
    }

    /// The row of the drawn wave: the first one with Braille dots in it.
    fn wave_row(rows: &[String]) -> usize {
        rows.iter()
            .position(|r| r.chars().any(|c| ('\u{2801}'..='\u{28ff}').contains(&c)))
            .expect("a Braille wave is drawn")
    }

    /// Scenario "Open from a pad" (chopping): F3 on a pad with a sample opens the chop
    /// screen on that sample, its wave drawn in Braille.
    #[test]
    fn open_from_a_pad() {
        let mut app = test_app();
        quiet_pad(&mut app, 0.5);
        open_chop_from_pads(&mut app);
        assert_eq!(app.screen, Screen::Chop);
        let c = app.chop.as_ref().unwrap();
        assert!(std::sync::Arc::ptr_eq(
            &c.sound,
            app.pads.sound[0].as_ref().unwrap()
        ));
        let rows = screen_rows(&chop_screen(&app));
        let wave = &rows[wave_row(&rows)];
        assert!(
            wave.chars().filter(|&c| c > '\u{2800}').count() > 50,
            "the wave runs across the screen: {wave}"
        );
    }

    /// Scenario "The cursor is visible" (chopping): the `!` stands above the wave at the
    /// cursor, the slice line names the cursor time.
    #[test]
    fn the_cursor_is_visible() {
        let mut app = test_app();
        quiet_pad(&mut app, 0.5);
        open_chop_from_pads(&mut app);
        app.chop.as_mut().unwrap().set_cursor(48_000);
        let rows = screen_rows(&chop_screen(&app));
        let w = wave_row(&rows);
        let mark = rows[..w]
            .iter()
            .find_map(|r| r.chars().position(|c| c == '!'))
            .expect("the ! above the wave");
        assert!(
            (49..=51).contains(&mark),
            "the middle of 100 columns: {mark}"
        );
        assert!(
            rows.iter().any(|r| r.contains("cursor 1.000 s")),
            "{}",
            rows.join("\n")
        );
    }

    /// Scenario "Eight parts" (chopping): Y three times gives 8 parts, 7 cuts at equal
    /// distances, and the header says "8 slices".
    #[test]
    fn eight_parts() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        quiet_pad(&mut app, 0.5);
        open_chop_from_pads(&mut app);
        for _ in 0..3 {
            chop_key(&mut app, KeyCode::Char('y'), M::NONE);
        }
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.markers, (1..8).map(|i| i * 12_000).collect::<Vec<_>>());
        let rows = screen_rows(&chop_screen(&app));
        assert!(
            rows.iter().any(|r| r.contains("· 8 slices")),
            "{}",
            rows.join("\n")
        );
    }

    /// Scenario "Another slice selected" (chopping): after another slice is picked the
    /// start knob far from its cut does nothing and the status says which way to turn;
    /// once it reaches the cut it moves it.
    #[test]
    fn another_slice_selected() {
        let turn = |app: &mut super::App, value| {
            app.midi_event(super::midi::Event::Param {
                port: "X-Session".into(),
                control: "k1".into(),
                param: input::midi::Param::SliceStart,
                value,
                takeover: true,
            })
        };
        let mut app = test_app();
        // the scenario is about the catch: the response pickup (58)
        app.takeover.response = input::midi::Response::Pickup;
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(2)); // cuts at 24_000, 48_000, 72_000
        app.chop.as_mut().unwrap().select_slice(1);
        let at = app.chop.as_ref().unwrap().edge_pos(false);
        turn(&mut app, at);
        turn(&mut app, 0.1);
        assert!(app.chop.as_ref().unwrap().markers[0] < 24_000, "caught");
        app.chop.as_mut().unwrap().select_slice(2);
        let cuts = app.chop.as_ref().unwrap().markers.clone();
        turn(&mut app, 0.95);
        assert_eq!(app.chop.as_ref().unwrap().markers, cuts, "no jump");
        assert_eq!(flash(&app), "turn k1 down ↓ to the cut");
        let at = app.chop.as_ref().unwrap().edge_pos(false);
        turn(&mut app, at);
        turn(&mut app, at - 0.1);
        assert!(
            app.chop.as_ref().unwrap().markers[1] < 48_000,
            "follows once it reached the cut"
        );
    }

    /// Scenario "See what is taken" (chopping): in the assign mode every pad shows what
    /// lies on it or "empty", the planned ones their slice, the replaced ones are lit,
    /// and the summary counts the banks and the replaced pads.
    #[test]
    fn see_what_is_taken() {
        let mut app = chop_open();
        app.pads.file[4] = Some("kick.wav".into());
        app.pads.file[7] = Some("snare.wav".into());
        app.apply(Action::ChopAssign);
        hit(&mut app, 4); // slice 1 over "kick"
        hit(&mut app, 5); // slice 2 on an empty pad
        let c = app.chop.as_ref().unwrap();
        let v = super::chop_view(c, app.plan.as_ref(), None, 48_000, &app.pads);
        let p = &v.pads.pads;
        assert_eq!(
            (p[4].name.as_str(), p[4].note.as_deref()),
            ("kick", Some("→ 1"))
        );
        assert!(p[4].in_hand, "kick would be replaced");
        assert_eq!(
            (p[5].name.as_str(), p[5].note.as_deref()),
            ("empty", Some("→ 2"))
        );
        assert!(!p[5].in_hand);
        assert_eq!((p[7].name.as_str(), p[7].note.as_deref()), ("snare", None));
        assert!(!p[7].in_hand, "not planned: not replaced");
        assert_eq!(p[0].name, "empty");
        assert_eq!(v.side[0], "2 slices → bank 1 · replaces 1 · 6 left out");
    }

    /// Scenario "Start the chain on another pad" (chopping): in order, nothing is planned
    /// before a pad is hit; pad 5 of bank 3 takes slice 1 and the rest follow.
    #[test]
    fn start_the_chain_on_another_pad() {
        let mut app = chop_open();
        app.apply(Action::ChopAssign);
        app.apply(Action::ParamNext);
        let targets = |app: &super::App| app.plan.as_ref().unwrap().targets(8);
        assert!(targets(&app).is_empty(), "no start pad yet");
        app.apply(Action::BankNext);
        app.apply(Action::BankNext);
        assert_eq!(app.pads.bank, 2);
        hit(&mut app, 4);
        assert_eq!(
            targets(&app),
            (0..8).map(|s| (s, 36 + s)).collect::<Vec<_>>()
        );
    }

    /// The chop screen on 9 slices of the silent break, nothing laid out.
    fn nine_slices_open() -> super::App {
        let mut app = test_app();
        app.screen = Screen::Chop;
        let mut c = chop_with_parts(0);
        for i in 1..9 {
            c.cut(i * 10_000);
        }
        assert_eq!(c.slices().len(), 9);
        c.select_slice(0);
        app.chop = Some(c);
        app
    }

    /// Scenario "Nine slices" (chopping): in order from pad 1 of an empty bank 3 the
    /// slices land on pads 1–9 of bank 3 at once, 10–16 stay empty.
    #[test]
    fn nine_slices() {
        let mut app = nine_slices_open();
        app.apply(Action::ChopAssign);
        app.apply(Action::ParamNext);
        app.pads.bank = 2;
        hit(&mut app, 0);
        app.apply(Action::ChopEdit);
        assert!(app.ask.is_none(), "an empty bank asks nothing");
        let slices = app.chop.as_ref().unwrap().slices();
        for (i, slice) in slices.iter().enumerate() {
            assert_eq!(app.pads.region[32 + i], *slice, "pad 3-{}", i + 1);
        }
        assert!(
            (41..48).all(|a| app.pads.file[a].is_none()),
            "10–16 untouched"
        );
        assert_eq!(flash(&app), "slices placed on bank 3");
    }

    /// Scenario "More slices than pads to the end of the bank" (chopping): 9 slices from
    /// pad 13 of bank 1 fill 13–16 of bank 1 and 1–5 of bank 2.
    #[test]
    fn more_slices_than_pads_to_the_end_of_the_bank() {
        let mut app = nine_slices_open();
        app.apply(Action::ChopAssign);
        app.apply(Action::ParamNext);
        hit(&mut app, 12);
        app.apply(Action::ChopEdit);
        let slices = app.chop.as_ref().unwrap().slices();
        for (i, slice) in slices.iter().enumerate() {
            assert_eq!(app.pads.region[12 + i], *slice);
        }
        assert!(app.pads.file[21].is_none() && app.pads.file[11].is_none());
        assert_eq!(flash(&app), "slices placed on banks 1–2");
    }

    /// Slices 1 and 2 planned on pads Q and W of bank 2, which hold "kick" and "amen 3".
    fn plan_over_kick_and_amen() -> super::App {
        let mut app = chop_open();
        app.pads.file[20] = Some("kick.wav".into());
        app.pads.file[21] = Some("amen 3.wav".into());
        app.apply(Action::ChopAssign);
        app.pads.bank = 1;
        hit(&mut app, 4);
        hit(&mut app, 5);
        app
    }

    /// Scenario "Overwrite only after a question" (chopping): applying asks first and
    /// changes nothing; Esc keeps the pads and the plan, Enter after the question places
    /// the slices.
    #[test]
    fn overwrite_only_after_a_question() {
        let mut app = plan_over_kick_and_amen();
        let plan = app.plan.clone();
        app.apply(Action::ChopEdit);
        let lines = app.ask.as_ref().expect("a question").lines().to_vec();
        for want in [
            "bank 2, pad Q: kick → slice 1",
            "bank 2, pad W: amen 3 → slice 2",
        ] {
            assert!(lines.iter().any(|l| l.trim() == want), "{lines:?}");
        }
        assert_eq!(app.pads.file[20].as_deref(), Some("kick.wav".as_ref()));
        app.apply(Action::Back);
        assert!(app.ask.is_none());
        assert_eq!(app.plan, plan, "the plan stays");
        assert_eq!(app.pads.file[21].as_deref(), Some("amen 3.wav".as_ref()));
        app.apply(Action::ChopEdit);
        app.apply(Action::ChopEdit);
        assert!(app.ask.is_none() && app.plan.is_none(), "applied");
        assert_eq!(app.pads.file[20].as_deref(), Some("break.wav".as_ref()));
        assert_eq!(app.pads.slice[21], Some(2));
    }

    /// Scenario "Apply a plan over taken pads" (chopping): the question lists both pads,
    /// says the files on disk stay and offers Enter and Esc.
    #[test]
    fn apply_a_plan_over_taken_pads() {
        let mut app = plan_over_kick_and_amen();
        app.apply(Action::ChopEdit);
        let lines: Vec<String> = app
            .ask
            .as_ref()
            .unwrap()
            .lines()
            .iter()
            .map(|l| l.trim().to_string())
            .collect();
        for want in [
            "bank 2, pad Q: kick → slice 1",
            "bank 2, pad W: amen 3 → slice 2",
            "The old sounds leave these pads; their files on disk stay.",
            "Enter: replace · Esc: keep",
        ] {
            assert!(lines.iter().any(|l| l == want), "{want}: {lines:?}");
        }
    }

    /// Scenario "Leave without changes" (chopping): Esc out of the assign mode with a plan
    /// leaves every pad as it was.
    #[test]
    fn leave_without_changes() {
        let mut app = plan_over_kick_and_amen();
        let before: Vec<_> = (0..super::PADS)
            .map(|a| (app.pads.file[a].clone(), app.pads.region[a]))
            .collect();
        app.apply(Action::Back);
        assert!(app.plan.is_none(), "the assign mode is closed");
        let after: Vec<_> = (0..super::PADS)
            .map(|a| (app.pads.file[a].clone(), app.pads.region[a]))
            .collect();
        assert_eq!(after, before);
    }

    /// Scenario "Learn the cut" (chopping): "chop.cut" picked in the CHOP group of the
    /// MIDI tab takes the next button; on the chop screen that button cuts.
    #[test]
    fn learn_the_cut() {
        let mut app = xsession_connected(&[]);
        midi_tab(&mut app);
        jump_to_chop(&mut app);
        app.apply(Action::SettingsValue(1));
        app.midi_row = app
            .midi_list()
            .iter()
            .position(
                |r| matches!(r, super::MidiListRow::Action { info } if info.name == "chop.cut"),
            )
            .expect("chop.cut under CHOP");
        app.apply(Action::SettingsLearn);
        app.midi_event(super::midi::Event::Message {
            port: "USB X-Session MIDI 1".into(),
            profile: "X-Session Pro".into(),
            msg: press(99),
            meaning: String::new(),
        });
        // the button waits on the row for Enter (74)
        app.apply(Action::SettingsLearn);
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(0));
        app.chop.as_mut().unwrap().set_cursor(30_000);
        let action = xs_live(&app, press(99));
        assert_eq!(action, Some(Action::ChopCut));
        app.apply(action.unwrap());
        assert_eq!(app.chop.as_ref().unwrap().markers, [30_000]);
    }

    /// Scenario "Fix a slice after the layout" (chopping): F3 on pad 3 of a laid-out break
    /// and the start of slice 3 moved 20 ms right: pad 2 is 20 ms longer, pad 3 starts
    /// 20 ms later.
    #[test]
    fn fix_a_slice_after_the_layout() {
        let mut app = chop_from_pad(6); // slices 1–8 on pads 5–12; pad 7 holds slice 3
        let (two, three) = (app.pads.region[5], app.pads.region[6]);
        app.apply(Action::MarkerNudge {
            right: true,
            fine: false,
        });
        assert_eq!(
            app.pads.region[5].end,
            two.end + 960,
            "pad 2 is 20 ms longer"
        );
        assert_eq!(app.pads.region[6].start, three.start + 960, "20 ms later");
        assert_eq!(app.pads.region[6].end, three.end);
    }

    /// Scenario "Chop a found break" (chopping): F3 on a file of the files screen opens it
    /// on the chop screen; no pad changes.
    #[test]
    fn chop_a_found_break() {
        let dir = tmp_dir("found-break");
        let path = dir.join("amen.wav");
        engine::offline::write_wav(&path, 48_000, &[0.1; 4_800], &[0.1; 4_800]).unwrap();
        let mut app = test_app();
        app.browser = super::browser::Browser::new(Some(dir.clone()));
        app.screen = Screen::Files;
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let f3 = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::F(3),
            ratatui::crossterm::event::KeyModifiers::empty(),
        );
        app.apply(keymap.resolve(Screen::Files, &f3).unwrap());
        wait_for(&mut app, |a| a.screen == Screen::Chop);
        assert_eq!(app.chop.as_ref().unwrap().file, path);
        assert!(app.pads.file.iter().all(|f| f.is_none()), "no pad changes");
        assert!(
            !app.sent
                .iter()
                .any(|c| matches!(c, Command::SetPadSound { .. })),
            "no pad hears it"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Scenario "Tap pads in turn" (chopping): 4 slices, P and hits on Q 1 W E place
    /// slices 1–4 there in turn; the status says all 4 are placed and Enter applies.
    #[test]
    fn tap_pads_in_turn() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(2));
        app.chop.as_mut().unwrap().select_slice(0);
        chop_key(&mut app, KeyCode::Char('p'), M::NONE);
        for pad in [4, 0, 5, 6] {
            hit(&mut app, pad);
        }
        let mut t = app.plan.as_ref().unwrap().targets(4);
        t.sort();
        assert_eq!(t, [(0, 4), (1, 0), (2, 5), (3, 6)]);
        assert_eq!(flash(&app), "all 4 slices placed: Enter applies");
    }

    /// Scenario "Put a file on a taken pad" (chopping): a hit with a file in hand on a pad
    /// with "kick" asks first; Esc keeps "kick" and the file in hand.
    #[test]
    fn put_a_file_on_a_taken_pad() {
        let mut app = test_app();
        app.screen = Screen::Files;
        app.pads.file[0] = Some("kick.wav".into());
        app.browser.in_hand = Some("/tmp/amen.wav".into());
        hit(&mut app, 0);
        let lines = app.ask.as_ref().expect("a question").lines().to_vec();
        assert!(
            lines
                .iter()
                .any(|l| l.trim() == "bank 1, pad 1: kick → amen.wav"),
            "{lines:?}"
        );
        app.apply(Action::Back);
        assert!(app.ask.is_none());
        assert_eq!(app.pads.file[0].as_deref(), Some("kick.wav".as_ref()));
        assert_eq!(
            app.browser.in_hand.as_deref(),
            Some("/tmp/amen.wav".as_ref())
        );
    }

    /// Scenario "Region from I to O" (chopping): `,` at 30_000, `.` at 60_000 and equal
    /// parts: the parts cover the region, the wave outside it is dimmed.
    #[test]
    fn region_from_i_to_o() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        quiet_pad(&mut app, 0.5);
        open_chop_from_pads(&mut app);
        app.chop.as_mut().unwrap().set_cursor(30_000);
        chop_key(&mut app, KeyCode::Char(','), M::NONE);
        app.chop.as_mut().unwrap().set_cursor(60_000);
        chop_key(&mut app, KeyCode::Char('.'), M::NONE);
        chop_key(&mut app, KeyCode::Char('y'), M::NONE);
        let s = app.chop.as_ref().unwrap().slices();
        assert_eq!((s[0].start, s[s.len() - 1].end), (30_000, 60_000));
        assert_eq!(s.len(), 2);
        let buf = chop_screen(&app);
        let w = wave_row(&screen_rows(&buf)) as u16;
        let light = |x: u16| match buf.cell((x, w)).unwrap().fg {
            ratatui::style::Color::Rgb(r, g, b) => r as u32 + g as u32 + b as u32,
            c => panic!("{c:?}"),
        };
        // 96_000 frames over 100 columns: the region is columns 31..62
        for (x, inside) in [(10, 40), (80, 55)] {
            assert!(light(x) * 2 < light(inside), "column {x} is dimmed");
        }
    }

    /// Scenario "Y steps the parts" (chopping): Y in the listen mode cuts into the next
    /// count of equal parts.
    #[test]
    fn y_steps_the_parts() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(0));
        chop_key(&mut app, KeyCode::Char('y'), M::NONE);
        assert_eq!(app.chop.as_ref().unwrap().slices().len(), 2);
        chop_key(&mut app, KeyCode::Char('y'), M::NONE);
        assert_eq!(app.chop.as_ref().unwrap().slices().len(), 4);
    }

    /// The pads screen as a 120×40 terminal draws it, one string per row.
    fn pads_screen(app: &mut super::App) -> Vec<String> {
        let view = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        let view = app.mark_pads(view);
        let status = tui::Status {
            badges: app.title_badges(),
            save: app.save_status(),
            backend: Some("jack"),
            recording: false,
            buffer_size: 128,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: app.master_db,
            meters: ([-60.0; 2], [-60.0; 2], false),
            notice: None,
            no_releases: false,
            project: None,
        };
        let help = tui::Help {
            groups: &[],
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 40)).unwrap();
        term.draw(|f| tui::draw(f, &status, &tui::Screen::Pads(view), &help))
            .unwrap();
        screen_rows(term.backend().buffer())
    }

    /// Scenario "Direct bank choice" (pad-playback): Ctrl+3 opens bank 3, the plain 3 still
    /// plays pad 3 of it.
    #[test]
    fn direct_bank_choice() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        pads_key(&mut app, KeyCode::Char('3'), M::CONTROL);
        assert_eq!(app.pads.bank, 2);
        app.sent.clear();
        pads_key(&mut app, KeyCode::Char('3'), M::NONE);
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::PadPress { pad: 34, .. })),
            "pad 3 of bank 3 plays"
        );
        assert_eq!(app.pads.bank, 2);
    }

    /// Scenario "Hit selects the pad" (pad-playback): W lights pad 6, selects it and the
    /// panel on the right names its mode, volume and pitch.
    #[test]
    fn hit_selects_the_pad() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.pads.gate[5] = true;
        app.pads.volume_db[5] = -7;
        app.pads.pitch[5] = 3;
        pads_key(&mut app, KeyCode::Char('w'), M::NONE);
        assert_eq!(app.pads.selected, 5);
        let v = super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000);
        assert!(v.pads[5].flash, "pad 6 flashes");
        assert_eq!(v.selected, 5);
        let rows = pads_screen(&mut app);
        let text = rows.join("\n");
        let right = |r: &String| r.chars().skip(80).collect::<String>();
        for want in ["pad W", "mode    gate", "volume  -7 dB", "pitch   +3 st"] {
            assert!(
                rows.iter().any(|r| right(r).contains(want)),
                "{want}:\n{text}"
            );
        }
    }

    /// Scenario "Held pad stays lit" (pad-playback): pad Q held for 2 seconds, from the
    /// keyboard or a controller, is lit all along and dark right after the release.
    #[test]
    fn held_pad_stays_lit() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let lit = |app: &mut super::App| {
            super::pads_view(&mut app.pads, None, None, Vec::new(), None, 48_000).pads[4].flash
        };
        let two_seconds_ago = std::time::Instant::now() - std::time::Duration::from_secs(2);
        let mut app = test_app();
        app.pads.track_keys = true;
        pads_key(&mut app, KeyCode::Char('q'), M::NONE);
        app.pads.hit_at[4] = Some(two_seconds_ago);
        assert!(lit(&mut app), "held from the keyboard");
        app.apply(Action::PadRelease { pad: 4 });
        assert!(!lit(&mut app), "dark after the release");
        app.midi_event(super::midi::Event::PadHit {
            sound: super::midi::PadSound::DirectRejected,
            abs: 4,
            velocity: 1.0,
            ui_press: None,
        });
        app.pads.hit_at[4] = Some(two_seconds_ago);
        assert!(lit(&mut app), "held from a controller");
        app.midi_event(super::midi::Event::PadRelease {
            abs: 4,
            direct: false,
        });
        assert!(!lit(&mut app));
    }

    /// Scenario "L turns to the next bank" (pad-playback): an explicit owner binding of L
    /// keeps the old bank turn; the shipped map has U/O for banks (83a).
    #[test]
    fn l_turns_to_the_next_bank() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let (mut keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        keymap
            .overlay("[screens.pads]\nL = \"bank.next\"\n")
            .unwrap();
        let mut app = test_app();
        app.pads.selected = 6;
        let k = ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('l'), M::NONE);
        app.apply(
            keymap
                .resolve_in(Screen::Pads, app.edit_keys(), &k)
                .unwrap(),
        );
        assert_eq!((app.pads.bank, app.pads.selected), (1, 6));
    }

    /// Scenario "Start without the selected device" (audio-settings): the saved device is
    /// named while the sound goes elsewhere; the routing half is the audio test of that name.
    #[test]
    fn start_without_the_selected_device() {
        let saved = Some("Buds".to_string());
        assert_eq!(
            super::missing_device(&saved, &Some("Speaker".into())),
            Some("Buds")
        );
        assert_eq!(super::missing_device(&saved, &saved.clone()), None);
    }

    /// Scenario "Controller pulled out during play" (input-devices), the screen side: the
    /// status names the disconnect, nothing is sent to the sound, and the device plugged in
    /// again acts by its profile. The MIDI thread side is the midi test of that name.
    #[test]
    fn controller_pulled_out_during_play() {
        let mut app = xsession_connected(&[]);
        app.sent.clear();
        app.midi_event(super::midi::Event::Disconnected {
            port: "USB X-Session MIDI 1".into(),
        });
        assert_eq!(flash(&app), "MIDI disconnected: USB X-Session MIDI 1");
        assert!(app.sent.is_empty(), "the sound is left alone");
        assert!(app.midi_devices.is_empty());
        app.midi_event(super::midi::Event::Connected {
            port: "USB X-Session MIDI 1".into(),
            profile: "X-Session Pro".into(),
        });
        assert_eq!(
            flash(&app),
            "MIDI connected: USB X-Session MIDI 1 (X-Session Pro profile)"
        );
        assert_eq!(xs_live(&app, press(B7)), Some(Action::PadEditOnly));
    }

    /// Picks the row of `name` in `group` on the MIDI tab and presses Enter on it.
    fn learn_row(app: &mut super::App, group: input::Group, name: &str) {
        midi_tab(app);
        if !app.midi_open.contains(&group) {
            app.midi_open.push(group);
        }
        app.midi_row = app
            .midi_list()
            .iter()
            .position(|r| {
                matches!(r, super::MidiListRow::Action { info }
                    if info.name == name && info.group == group)
            })
            .expect("the action row");
        app.apply(Action::SettingsLearn);
    }

    /// A message from the X-Session Pro as the MIDI thread hands it to the screen.
    fn xs_message(app: &mut super::App, msg: input::midi::Msg) {
        app.midi_event(super::midi::Event::Message {
            port: "USB X-Session MIDI 1".into(),
            profile: "X-Session Pro".into(),
            msg,
            meaning: String::new(),
        });
    }

    /// Learns the X-Session button `note` for `name` in `group`: the press waits on the
    /// row, Enter saves it (74).
    fn learn_button(app: &mut super::App, group: input::Group, name: &str, note: u8) {
        learn_row(app, group, name);
        xs_message(app, press(note));
        assert!(
            app.learning.as_ref().is_some_and(|l| l.pending.is_some()),
            "{name}: the button waits for Enter"
        );
        app.apply(Action::SettingsLearn);
        assert!(app.learning.is_none(), "{name}: the button is taken");
    }

    /// The MIDI tab row of `label`: (what drives it, taken, the `stays` lines).
    fn midi_row_of(app: &super::App, label: &str) -> (String, bool, Vec<String>) {
        app.midi_screen()
            .into_iter()
            .find_map(|r| match r {
                tui::MidiRow::Action {
                    label: l,
                    who,
                    taken,
                    stays,
                } if l == label => Some((who, taken, stays)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("the row {label}"))
    }

    /// What the X-Session button `note` does on `screen`.
    fn xs_on(app: &mut super::App, screen: Screen, note: u8) -> Option<Action> {
        app.screen = screen;
        xs_live(app, press(note))
    }

    /// Scenario "Knob for pad volume" (midi-learn): the knob that moved most is taken over
    /// a neighbour touched once, Enter saves it, the row names it, and it turns the selected
    /// pad volume with soft takeover.
    #[test]
    fn knob_for_pad_volume() {
        let mut app = xsession_connected(&[]);
        learn_row(&mut app, input::Group::Pads, "pad.volume");
        xs_message(&mut app, knob(30, 70));
        for v in 60..70 {
            xs_message(&mut app, knob(31, v));
        }
        assert!(app.learning.is_some(), "a knob is decided after a while");
        let later = std::time::Instant::now() + std::time::Duration::from_secs(1);
        let (profile, key) = app.learning.as_mut().unwrap().poll(later).unwrap();
        app.learned(profile, key);
        assert_eq!(
            app.learned["X-Session Pro"]["pads"].get("cc 31"),
            None,
            "the knob waits for Enter"
        );
        app.apply(Action::SettingsLearn);
        let row = app
            .midi_screen()
            .into_iter()
            .find_map(|r| match r {
                tui::MidiRow::Action { label, who, .. } if label == "pad volume (knob)" => {
                    Some(who)
                }
                _ => None,
            })
            .unwrap();
        assert!(row.contains("cc 31") && !row.contains("cc 30"), "{row}");
        // soft takeover: the knob far from the volume waits, then follows
        app.takeover.response = input::midi::Response::Pickup;
        app.screen = Screen::Pads;
        let turn = |app: &mut super::App, v| {
            let p = app
                .live_profiles()
                .into_iter()
                .find(|p| p.name == "X-Session Pro")
                .unwrap();
            let r = p.resolve_in(Screen::Pads, false, knob(31, v));
            feed_resolved(app, r);
        };
        let abs = app.pads.selected_abs();
        turn(&mut app, 127);
        assert_eq!(app.pads.volume_db[abs], 0, "no jump");
        let at = (app.pads.volume_pos() * 127.0).round() as u8;
        turn(&mut app, at);
        turn(&mut app, at - 20);
        assert!(app.pads.volume_db[abs] < 0, "follows once caught");
    }

    /// Scenario "Button for the next bank" (midi-learn): the button pressed, then Enter,
    /// turns the bank; the button the row had before no longer does, and a button pressed
    /// before Enter gives its place to the next one.
    #[test]
    fn button_for_the_next_bank() {
        let mut app = xsession_connected(&[]);
        learn_button(&mut app, input::Group::Global, "bank.next", 90);
        learn_row(&mut app, input::Group::Global, "bank.next");
        xs_message(&mut app, press(92));
        learn_button_press_enter(&mut app, 91);
        assert_eq!(xs_on(&mut app, Screen::Pads, 91), Some(Action::BankNext));
        assert_eq!(xs_on(&mut app, Screen::Pads, 90), None, "the old button");
        assert_eq!(xs_on(&mut app, Screen::Pads, 92), None, "the replaced one");
        app.apply(Action::BankNext);
        assert_eq!(app.pads.bank, 1);
    }

    /// The X-Session button `note` pressed with a row waiting, then Enter.
    fn learn_button_press_enter(app: &mut super::App, note: u8) {
        xs_message(app, press(note));
        app.apply(Action::SettingsLearn);
    }

    /// b9 on "pad mode" in PADS and on "bank next" in GLOBAL; the config in a fresh folder.
    fn b9_on_two_groups(name: &str) -> (super::App, std::path::PathBuf) {
        let mut app = xsession_connected(&[
            (input::Group::Pads, B9, "pad.mode_next"),
            (input::Group::Global, B9, "bank.next"),
        ]);
        let dir = tmp_dir(name);
        std::fs::create_dir_all(&dir).unwrap();
        app.user_path = Some(dir.join("config.toml"));
        app.save_midi();
        (app, dir.join("config.toml"))
    }

    /// Scenario "See what the button leaves before it is saved" (midi-learn): b9 waits on
    /// the "marking on / off" row, the "pad mode" row is marked as the one that loses it,
    /// GLOBAL keeps it in a grey line, and nothing is saved.
    #[test]
    fn see_what_the_button_leaves_before_it_is_saved() {
        let (mut app, cfg) = b9_on_two_groups("see-leaves");
        let before = (app.learned.clone(), std::fs::read_to_string(&cfg).unwrap());
        learn_row(&mut app, input::Group::Pads, "pad.mark_toggle");
        xs_message(&mut app, press(B9));
        let (who, taken, stays) = midi_row_of(&app, "marking on / off");
        assert_eq!(who, "none → b9");
        assert!(!taken);
        assert!(
            stays.contains(&"b9 on GLOBAL: bank next · stays".to_string()),
            "{stays:?}"
        );
        let (who, taken, stays) = midi_row_of(&app, "pad mode");
        assert!(taken, "the pad mode row is in the taken colour");
        assert!(who.ends_with("✗ loses b9"), "{who}");
        assert!(stays.is_empty());
        assert!(
            app.learning_line()
                .unwrap()
                .contains("Enter: assign and save")
        );
        assert_eq!(
            (app.learned.clone(), std::fs::read_to_string(&cfg).unwrap()),
            before,
            "nothing is saved yet"
        );
    }

    /// Scenario "Esc on F12 keeps all" (midi-learn): Esc drops the waiting b9, every
    /// binding stays and the config file is the same.
    #[test]
    fn esc_on_f12_keeps_all() {
        let (mut app, cfg) = b9_on_two_groups("esc-keeps");
        let before = (app.learned.clone(), std::fs::read_to_string(&cfg).unwrap());
        learn_row(&mut app, input::Group::Pads, "pad.mark_toggle");
        xs_message(&mut app, press(B9));
        app.apply(Action::Back);
        assert!(app.learning.is_none());
        assert!(!midi_row_of(&app, "pad mode").1, "no row is marked");
        assert_eq!(
            (app.learned.clone(), std::fs::read_to_string(&cfg).unwrap()),
            before
        );
        assert_eq!(xs_on(&mut app, Screen::Pads, B9), Some(Action::PadModeNext));
    }

    /// Scenario "Reassign a button" (midi-learn): b9 for "bank next" in GLOBAL is learned
    /// for "pad mode" in PADS: the row says `bank next · stays` before Enter; after it b9
    /// switches the pad mode on Pads and turns the bank in Settings; Files keeps preview.
    #[test]
    fn reassign_a_button() {
        let mut app = xsession_connected(&[(input::Group::Global, B9, "bank.next")]);
        learn_row(&mut app, input::Group::Pads, "pad.mode_next");
        xs_message(&mut app, press(B9));
        let (_, _, stays) = midi_row_of(&app, "pad mode");
        assert!(
            stays.contains(&"b9 on GLOBAL: bank next · stays".to_string()),
            "{stays:?}"
        );
        app.apply(Action::SettingsLearn);
        assert_eq!(xs_on(&mut app, Screen::Pads, B9), Some(Action::PadModeNext));
        assert_eq!(
            xs_on(&mut app, Screen::Settings, B9),
            Some(Action::BankNext)
        );
        assert_eq!(
            xs_on(&mut app, Screen::Files, B9),
            Some(Action::BrowserPreviewToggle)
        );
    }

    /// Scenario "Cancel and clear" (midi-learn): Esc while waiting changes nothing;
    /// Backspace on a row removes its control, a profile's one too.
    #[test]
    fn cancel_and_clear() {
        let mut app = xsession_connected(&[]);
        let before = app.learned.clone();
        learn_row(&mut app, input::Group::Global, "bank.next");
        app.apply(Action::Back);
        assert!(app.learning.is_none(), "waiting stops");
        xs_message(&mut app, press(90));
        assert_eq!(app.learned, before, "no changes");
        // b7 opens pad edit from the profile
        assert_eq!(xs_on(&mut app, Screen::Pads, B7), Some(Action::PadEditOnly));
        learn_row(&mut app, input::Group::Pads, "pad.edit_only");
        app.apply(Action::Back);
        app.apply(Action::SettingsClear);
        assert_eq!(xs_on(&mut app, Screen::Pads, B7), None, "no control left");
    }

    /// Scenario "Global action" (midi-learn): a button learned for help in GLOBAL opens
    /// the help on the pads screen and on the settings screen.
    #[test]
    fn global_action() {
        let mut app = xsession_connected(&[]);
        learn_button(&mut app, input::Group::Global, "help.toggle", 90);
        for screen in [Screen::Pads, Screen::Settings] {
            let a = xs_on(&mut app, screen, 90);
            assert_eq!(a, Some(Action::HelpToggle), "{screen:?}");
            app.help_open = false;
            app.apply(a.unwrap());
            assert!(app.help_open, "{screen:?}");
        }
    }

    /// Scenario "Screen group wins" (midi-learn): one button for help in GLOBAL and for
    /// the pad mode in PADS: the pads screen switches the mode, the settings screen opens
    /// the help.
    #[test]
    fn screen_group_wins() {
        let mut app = xsession_connected(&[]);
        learn_button(&mut app, input::Group::Global, "help.toggle", 90);
        learn_button(&mut app, input::Group::Pads, "pad.mode_next", 90);
        assert_eq!(xs_on(&mut app, Screen::Pads, 90), Some(Action::PadModeNext));
        assert_eq!(
            xs_on(&mut app, Screen::Settings, 90),
            Some(Action::HelpToggle)
        );
    }

    /// A key on the files screen as the main loop routes it: the open filter line takes
    /// it first, the keymap gets the rest.
    fn files_key(
        app: &mut super::App,
        code: ratatui::crossterm::event::KeyCode,
        m: ratatui::crossterm::event::KeyModifiers,
    ) {
        let k = ratatui::crossterm::event::KeyEvent::new(code, m);
        if app.browser.editing && app.screen == Screen::Files && app.filter_key(&k) {
            return;
        }
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        if let Some(a) = keymap.resolve_in(Screen::Files, false, &k) {
            app.apply(a);
        }
    }

    fn type_text(app: &mut super::App, text: &str) {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        for c in text.chars() {
            files_key(app, KeyCode::Char(c), M::NONE);
        }
    }

    fn pad_presses(app: &super::App) -> usize {
        app.sent
            .iter()
            .filter(|c| matches!(c, Command::PadPress { .. }))
            .count()
    }

    /// Scenario "Search by name" (sample-loading): "amen" typed in the filter leaves the
    /// files and folders with "amen" in the name, any case.
    #[test]
    fn search_by_name() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let dir = tmp_dir("search-amen");
        std::fs::create_dir_all(dir.join("Amen breaks")).unwrap();
        for f in ["amen-brother.wav", "kick.wav", "think.wav"] {
            std::fs::write(dir.join(f), b"").unwrap();
        }
        let mut app = test_app();
        app.preview_on = false;
        app.browser = super::browser::Browser::new(Some(dir.clone()));
        app.apply(Action::ScreenFiles);
        files_key(&mut app, KeyCode::Char('/'), M::NONE);
        type_text(&mut app, "amen");
        let names: Vec<_> = app
            .browser
            .entries
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        assert_eq!(names, ["Amen breaks", "amen-brother.wav"]);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Scenario "Typing text in the filter" (sample-loading): with the line open every key,
    /// the pad block in both layouts too, types and no pad sounds; Enter keeps the filter,
    /// Esc clears it, and after either the block keys play pads again.
    #[test]
    fn typing_text_in_the_filter() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.preview_on = false;
        app.browser = super::browser::Browser::new(Some(tmp_dir("filter-typing")));
        app.apply(Action::ScreenFiles);
        files_key(&mut app, KeyCode::Char('/'), M::NONE);
        assert!(app.browser.editing, "the line is open");
        app.sent.clear();
        type_text(&mut app, "qw1йц");
        files_key(&mut app, KeyCode::Char('Q'), M::SHIFT);
        assert_eq!(app.browser.filter, "qw1йцQ");
        assert_eq!(pad_presses(&app), 0, "no pad sounds");
        files_key(&mut app, KeyCode::Enter, M::NONE);
        assert!(!app.browser.editing);
        assert_eq!(app.browser.filter, "qw1йцQ", "Enter keeps the filter");
        files_key(&mut app, KeyCode::Char('q'), M::NONE);
        assert_eq!(pad_presses(&app), 1, "the block plays again");
        files_key(&mut app, KeyCode::Char('/'), M::NONE);
        type_text(&mut app, "zx");
        files_key(&mut app, KeyCode::Esc, M::NONE);
        assert!(
            !app.browser.editing && app.browser.filter.is_empty(),
            "Esc clears"
        );
        files_key(&mut app, KeyCode::Char('й'), M::NONE);
        assert_eq!(pad_presses(&app), 2, "Q in the Russian layout plays");
    }

    /// Scenario "Loading a long file during play" (sample-loading), on a 20-second file:
    /// the load runs aside while the pads keep playing, the status shows its progress, and
    /// the pad has the file once it is loaded.
    #[test]
    fn loading_a_long_file_during_play() {
        let dir = tmp_dir("long-load");
        let path = dir.join("long.wav");
        let tone: Vec<f32> = (0..48_000 * 20)
            .map(|i| ((i % 100) as f32 / 100.0) - 0.5)
            .collect();
        engine::offline::write_wav(&path, 48_000, &tone, &tone).unwrap();
        let mut app = test_app();
        app.load_pad(4, path.clone(), Region::WHOLE);
        app.sent.clear();
        hit(&mut app, 0);
        assert_eq!(pad_presses(&app), 1, "pad 1 plays while Q loads");
        let seen = std::cell::RefCell::new(Vec::new());
        wait_for(&mut app, |a| {
            seen.borrow_mut().extend(a.notice());
            a.pads.loading[4].is_none()
        });
        let seen = seen.into_inner();
        let percents: Vec<u8> = seen
            .iter()
            .filter_map(|n| n.strip_prefix("Loading ")?.strip_suffix('%'))
            .filter_map(|n| n.rsplit(' ').next()?.parse().ok())
            .collect();
        assert!(
            percents.len() > 2 && percents.windows(2).all(|w| w[0] <= w[1]),
            "the progress moves: {seen:?}"
        );
        assert_eq!(
            app.pads.sound[4].as_ref().map(|s| s.len()),
            Some(48_000 * 20)
        );
        assert!(
            app.sent.iter().any(|c| matches!(
                c,
                Command::SetPadSound {
                    pad: 4,
                    sound: Some(_)
                }
            )),
            "the file is on the pad"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Scenario "Swap two pads" (pad-assignment): Q selected, Ctrl+X and a hit on V trade
    /// the two pads with their modes and volumes; the session is to be saved.
    #[test]
    fn swap_two_pads() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.pads.file[4] = Some("/s/q.wav".into());
        app.pads.file[15] = Some("/s/v.wav".into());
        (app.pads.gate[4], app.pads.volume_db[4]) = (true, -6);
        (app.pads.gate[15], app.pads.volume_db[15]) = (false, 3);
        app.pads.selected = 4;
        pads_key(&mut app, KeyCode::Char('x'), M::CONTROL);
        assert_eq!(app.pad_in_hand, Some(4));
        hit(&mut app, 15);
        let pad = |a: usize| {
            (
                app.pads.file[a].clone().unwrap(),
                app.pads.gate[a],
                app.pads.volume_db[a],
            )
        };
        assert_eq!(pad(4), ("/s/v.wav".into(), false, 3));
        assert_eq!(pad(15), ("/s/q.wav".into(), true, -6));
        assert!(
            app.sent
                .iter()
                .any(|c| matches!(c, Command::SwapPads { a: 4, b: 15 })),
            "the sound follows"
        );
        assert!(app.session_dirty.is_some(), "the session is saved");
    }

    /// Scenario "Move to an empty pad" (pad-assignment): the pad in hand put on an empty
    /// pad moves there, the first pad is empty.
    #[test]
    fn move_to_an_empty_pad() {
        let mut app = test_app();
        app.pads.file[4] = Some("/s/q.wav".into());
        app.pads.selected = 4;
        app.apply(Action::PadTake);
        hit(&mut app, 9);
        assert_eq!(app.pads.file[9].as_deref(), Some("/s/q.wav".as_ref()));
        assert_eq!(app.pads.file[4], None, "the first pad is empty");
        assert!(app.pad_in_hand.is_none());
    }

    fn master_gains(app: &super::App) -> Vec<f32> {
        app.sent
            .iter()
            .filter_map(|c| match c {
                Command::SetMasterGain { gain } => Some(*gain),
                _ => None,
            })
            .collect()
    }

    /// Scenario "Quieter while playing" (master-volume): `[` three times is 3 dB less, the
    /// status line says "master -3 dB"; the engine glides the gain
    /// (`master_gain_changes_smoothly`).
    #[test]
    fn quieter_while_playing() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        hit(&mut app, 0);
        for _ in 0..3 {
            pads_key(&mut app, KeyCode::Char('['), M::NONE);
        }
        assert_eq!(app.master_db, -3);
        let gain = *master_gains(&app).last().unwrap();
        assert!((gain - 10f32.powf(-3.0 / 20.0)).abs() < 1e-6, "{gain}");
        let rows = pads_screen(&mut app);
        assert!(rows[0].contains("master -3 dB"), "{}", rows[0]);
    }

    fn recorded_workflow_event(app: &mut super::App, event: ratatui::crossterm::event::KeyEvent) {
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        app.dismiss_startup();
        if app.pad_naming.is_some() && app.pad_name_key(&event) {
            return;
        }
        if app.project_picker.is_some() && app.project_picker_key(&event) {
            return;
        }
        if app.screen_learn.is_some() && app.learn_key(&keymap, &event) {
            return;
        }
        if app.screen == Screen::Files && app.browser.editing && app.filter_key(&event) {
            return;
        }
        let action = keymap.resolve_in(app.screen, app.edit_keys(), &event);
        assert!(
            action.is_some() || event.kind == ratatui::crossterm::event::KeyEventKind::Release,
            "unbound recorded event: {event:?} on {:?}",
            app.screen
        );
        if let Some(action) = action {
            assert!(!app.apply(action));
        }
    }

    fn recorded_workflow_key(
        app: &mut super::App,
        code: ratatui::crossterm::event::KeyCode,
        modifiers: ratatui::crossterm::event::KeyModifiers,
    ) {
        recorded_workflow_event(
            app,
            ratatui::crossterm::event::KeyEvent::new(code, modifiers),
        );
    }

    fn recorded_workflow_text(app: &mut super::App, text: &str) {
        use ratatui::crossterm::event::{KeyCode as K, KeyModifiers as M};
        for c in text.chars() {
            recorded_workflow_key(app, K::Char(c), M::NONE);
        }
    }

    fn recorded_workflow_hit(app: &mut super::App, key: char) {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
        recorded_workflow_event(app, KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE));
        recorded_workflow_event(
            app,
            KeyEvent::new_with_kind(
                KeyCode::Char(key),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            ),
        );
    }

    fn recorded_workflow_app(
        name: &str,
    ) -> (
        super::App,
        std::path::PathBuf,
        std::sync::mpsc::Receiver<super::loader::Request>,
    ) {
        let root = tmp_dir(name);
        let samples = root.join("samples");
        std::fs::create_dir_all(&samples).unwrap();
        let wave: Vec<_> = ring().into_iter().step_by(2).collect();
        for name in ["break.wav", "snare.wav"] {
            engine::offline::write_wav(&samples.join(name), 48_000, &wave, &wave).unwrap();
        }
        let mut app = test_app();
        app.paths.projects = root.join("projects");
        app.paths.samples = samples.clone();
        app.browser = super::browser::Browser::new(Some(samples));
        app.session_path = root.join("scratch.toml");
        app.user_path = Some(root.join("config.toml"));
        let (requests, incoming) = std::sync::mpsc::channel();
        let (_, replies) = std::sync::mpsc::channel();
        app.loader = Some((requests, replies));
        (app, root, incoming)
    }

    fn recorded_workflow_loads(
        app: &mut super::App,
        incoming: &std::sync::mpsc::Receiver<super::loader::Request>,
    ) {
        while let Ok(request) = incoming.try_recv() {
            let sound = super::loader::decode(&request.path, app.rate, |_| {}).unwrap();
            app.loader_reply(super::loader::Reply::Loaded {
                pad: request.pad,
                generation: request.generation,
                path: request.path,
                sound: std::sync::Arc::new(sound),
            });
        }
    }

    fn recorded_workflow_project(app: &mut super::App, name: &str) {
        use ratatui::crossterm::event::{KeyCode as K, KeyModifiers as M};
        recorded_workflow_key(app, K::Char('p'), M::CONTROL);
        recorded_workflow_text(app, name);
        recorded_workflow_key(app, K::Enter, M::NONE);
        assert!(app.project_picker.is_none());
        assert_eq!(app.project.as_ref().unwrap().name, name);
    }

    fn recorded_workflow_file(
        app: &mut super::App,
        incoming: &std::sync::mpsc::Receiver<super::loader::Request>,
        pad: char,
        filter: &str,
    ) {
        use ratatui::crossterm::event::{KeyCode as K, KeyModifiers as M};
        recorded_workflow_key(app, K::F(2), M::NONE);
        recorded_workflow_hit(app, pad);
        let selected = app.pads.selected_abs();
        recorded_workflow_key(app, K::F(6), M::NONE);
        if !app.browser.filter.is_empty() {
            recorded_workflow_key(app, K::Esc, M::NONE);
        }
        recorded_workflow_key(app, K::Char('/'), M::NONE);
        recorded_workflow_text(app, filter);
        recorded_workflow_key(app, K::Enter, M::NONE);
        assert_eq!(app.browser.entries.len(), 1);
        recorded_workflow_key(app, K::Enter, M::CONTROL);
        recorded_workflow_loads(app, incoming);
        assert_eq!(app.screen, Screen::Pads);
        assert_eq!(app.pads.selected_abs(), selected);
        assert!(app.pads.sound[selected].is_some());
        assert!(app.pads.loading[selected].is_none());
    }

    #[test]
    fn recorded_old_workflow_and_shared_fx_save_and_reopen() {
        use super::pads::Param;
        use ratatui::crossterm::event::{KeyCode as K, KeyModifiers as M};
        let (mut app, root, incoming) = recorded_workflow_app("recorded-fx-projects");
        recorded_workflow_project(&mut app, "A");
        recorded_workflow_file(&mut app, &incoming, 'q', "break");
        let sound = app.pads.sound[4].clone().unwrap();
        recorded_workflow_key(&mut app, K::Char('n'), M::CONTROL);
        for _ in 0.."break".len() {
            recorded_workflow_key(&mut app, K::Backspace, M::NONE);
        }
        let sent = app.sent.len();
        recorded_workflow_text(&mut app, "Бочка");
        let mut view = panel_now(&mut app);
        super::preview_pad_name(&mut view, app.pads.bank, app.pad_naming.as_ref());
        assert_eq!(view.pads[4].name, "Бочка");
        assert!(view.pads[4].naming);
        assert_eq!(app.sent.len(), sent, "name input does not play pads");
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        recorded_workflow_key(&mut app, K::Char('n'), M::CONTROL);
        recorded_workflow_text(&mut app, "wrong");
        recorded_workflow_key(&mut app, K::Esc, M::NONE);
        assert_eq!(super::pad_name(&app.pads, 4), "Бочка");
        assert!(std::sync::Arc::ptr_eq(
            app.pads.sound[4].as_ref().unwrap(),
            &sound
        ));

        recorded_workflow_key(&mut app, K::F(6), M::NONE);
        recorded_workflow_key(&mut app, K::Esc, M::NONE);
        recorded_workflow_key(&mut app, K::Char('/'), M::NONE);
        let sent = app.sent.len();
        recorded_workflow_text(&mut app, "snare");
        assert_eq!(app.browser.entries[0].name, "snare.wav");
        assert!(
            !app.sent[sent..]
                .iter()
                .any(|c| matches!(c, Command::PadPress { .. }))
        );
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        assert!(!app.browser.editing);
        assert_eq!(app.browser.filter, "snare");
        recorded_workflow_key(&mut app, K::Esc, M::NONE);
        assert_eq!(app.screen, Screen::Files);
        assert_eq!(app.browser.entries.len(), 2);
        recorded_workflow_key(&mut app, K::Char('/'), M::NONE);
        recorded_workflow_text(&mut app, "break");
        recorded_workflow_key(&mut app, K::Esc, M::NONE);
        assert!(!app.browser.editing);
        assert!(app.browser.filter.is_empty());
        recorded_workflow_key(&mut app, K::Esc, M::NONE);
        assert_eq!(app.screen, Screen::Pads);
        recorded_workflow_loads(&mut app, &incoming);

        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        for (page, param) in [
            ("TONE", Param::Volume),
            ("ENV", Param::Attack),
            ("FILTER", Param::Filter),
            ("SOURCE", Param::Source),
            ("FX", Param::Drive),
        ] {
            assert_eq!(app.pads.page().0, page);
            assert_eq!(app.pads.param, param);
            assert_eq!(panel_now(&mut app).edit, Some(page));
            assert_eq!(app.help_state().scope, "pad Q");
            if page != "FX" {
                recorded_workflow_key(&mut app, K::Tab, M::NONE);
            }
        }
        page_knob(&mut app, 0, 0.4);
        page_knob(&mut app, 1, 0.2);
        page_knob(&mut app, 2, 0.5);
        assert_eq!((app.pads.drive[4], app.pads.sends[4]), (40, [20, 50]));
        assert_eq!(app.pads.sends[5], [0, 0]);
        recorded_workflow_key(&mut app, K::Char('k'), M::NONE);
        assert_eq!(app.pads.param, Param::Send1);
        let pad = app.pads.snapshot(4);
        let original_fx = app.pads.fx;
        recorded_workflow_key(&mut app, K::F(4), M::NONE);
        assert!(!app.edit);
        page_knob(&mut app, 1, 0.8);
        page_knob(&mut app, 2, 0.3);
        let first = app.pads.fx;
        assert_eq!((first[0].size, first[0].tone), (80, 30));
        recorded_workflow_key(&mut app, K::Tab, M::NONE);
        assert_eq!(app.pads.fx, first);
        page_knob(&mut app, 0, 0.1);
        page_knob(&mut app, 1, 0.6);
        page_knob(&mut app, 2, 0.2);
        assert_eq!(app.pads.fx[0], first[0]);
        assert_eq!((app.pads.fx[1].size, app.pads.fx[1].tone), (60, 20));
        recorded_workflow_key(&mut app, K::Char('z'), M::CONTROL);
        assert_eq!(app.pads.fx[1].tone, first[1].tone);
        assert_eq!(app.pads.fx[1].size, 60);
        page_knob(&mut app, 2, 0.2);
        let saved_fx = app.pads.fx;
        assert_ne!(saved_fx, original_fx);
        assert_eq!(app.pads.snapshot(4), pad);
        recorded_workflow_key(&mut app, K::F(2), M::NONE);
        assert_eq!(app.pads.param, Param::Send1);
        assert_eq!(app.pads.page().0, "FX");
        assert!(panel_now(&mut app).slot.is_none());
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        recorded_workflow_key(&mut app, K::Tab, M::NONE);
        assert_eq!(app.pads.page().0, "TONE");
        recorded_workflow_key(&mut app, K::BackTab, M::SHIFT);
        assert_eq!(app.pads.page().0, "FX");
        recorded_workflow_key(&mut app, K::Esc, M::NONE);

        recorded_workflow_key(&mut app, K::F(3), M::NONE);
        assert_eq!(app.screen, Screen::Chop);
        assert_eq!(app.chop_from, Some(4));
        for _ in 0..8 {
            recorded_workflow_key(&mut app, K::Char('l'), M::NONE);
        }
        let cut = app.chop.as_ref().unwrap().cursor;
        assert!(cut > 0);
        recorded_workflow_key(&mut app, K::Char('m'), M::NONE);
        let slices = app.chop.as_ref().unwrap().slices();
        assert_eq!(slices.len(), 2);
        assert_eq!(slices[1].start, cut);
        recorded_workflow_hit(&mut app, '1');
        assert_eq!(app.chop.as_ref().unwrap().selected, 0);
        recorded_workflow_key(&mut app, K::Char('p'), M::NONE);
        recorded_workflow_hit(&mut app, 'e');
        recorded_workflow_hit(&mut app, 'r');
        assert!(app.pads.file[6].is_none());
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        assert!(app.plan.is_none());
        for (abs, region) in [(6, slices[0]), (7, slices[1])] {
            assert_eq!(app.pads.file[abs], app.pads.file[4]);
            assert_eq!(app.pads.region[abs], region);
            assert!(app.pads.link[abs].is_some());
        }
        recorded_workflow_key(&mut app, K::F(2), M::NONE);
        recorded_workflow_key(&mut app, K::Char('s'), M::CONTROL);
        assert_eq!(flash(&app), "Session saved");
        let project_a = root.join("projects/A/project.toml");
        assert!(project_a.is_file());
        assert!(!root.join("scratch.toml").exists());
        let stored = super::session::load(&project_a).unwrap();
        assert_eq!(stored.pads["4"].name.as_deref(), Some("Бочка"));
        let saved_pads = app.pads.to_session(None, app.rate).pads;

        recorded_workflow_key(&mut app, K::Char('a'), M::CONTROL);
        recorded_workflow_key(&mut app, K::Char('x'), M::CONTROL);
        assert!(!app.pads.marked.is_empty() && app.pad_in_hand.is_some());
        recorded_workflow_project(&mut app, "B");
        recorded_workflow_loads(&mut app, &incoming);
        assert!(app.pads.marked.is_empty() && !app.pads.marking);
        assert!(app.pad_in_hand.is_none());
        assert_eq!(app.pads.fx, original_fx);
        assert!(app.pads.file.iter().all(Option::is_none));
        assert!(app.pads.sound.iter().all(Option::is_none));
        assert!(app.pads.custom_name.iter().all(Option::is_none));
        assert!(app.pads.sends.iter().all(|s| *s == [0, 0]));
        assert!(app.history.is_empty());
        assert!(app.chop.is_none() && app.chops.is_empty() && app.older.is_empty());
        assert!(app.plan.is_none() && app.ask.is_none());
        assert!(app.browser.filter.is_empty() && app.browser.in_hand.is_none());
        recorded_workflow_key(&mut app, K::Char('p'), M::CONTROL);
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        recorded_workflow_loads(&mut app, &incoming);
        assert_eq!(app.project.as_ref().unwrap().name, "A");
        assert_eq!(app.pads.fx, saved_fx);
        assert_eq!(super::pad_name(&app.pads, 4), "Бочка");
        assert_eq!(app.pads.to_session(None, app.rate).pads, saved_pads);
        let mut restarted = test_app();
        restarted.paths = app.paths.clone();
        restarted.user_path = Some(root.join("config.toml"));
        restarted.session_path = root.join("restarted-scratch.toml");
        recorded_workflow_key(&mut restarted, K::Char('p'), M::CONTROL);
        recorded_workflow_key(&mut restarted, K::Enter, M::NONE);
        wait_for(&mut restarted, |a| {
            a.pads.loading.iter().all(Option::is_none)
        });
        assert_eq!(restarted.pads.fx, saved_fx);
        assert_eq!(
            restarted.pads.to_session(None, restarted.rate).pads,
            saved_pads
        );
        assert_eq!(super::pad_name(&restarted.pads, 4), "Бочка");
        assert_eq!(restarted.pads.sends[4], [20, 50]);
        assert_eq!(restarted.pads.region[6], slices[0]);
        assert_eq!(restarted.pads.region[7], slices[1]);
        assert!(restarted.pads.sound[6].is_some() && restarted.pads.sound[7].is_some());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recorded_marked_pads_group_reset_clear_swap_and_undo_after_fx() {
        use ratatui::crossterm::event::{KeyCode as K, KeyModifiers as M};
        let (mut app, root, incoming) = recorded_workflow_app("recorded-fx-pad-edits");
        recorded_workflow_project(&mut app, "A");
        for (pad, file) in [('q', "break"), ('w', "snare")] {
            recorded_workflow_file(&mut app, &incoming, pad, file);
        }
        recorded_workflow_key(&mut app, K::Char('n'), M::CONTROL);
        recorded_workflow_text(&mut app, "!");
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        assert_eq!(super::pad_name(&app.pads, 5), "snare!");
        recorded_workflow_key(&mut app, K::F(6), M::NONE);
        recorded_workflow_key(&mut app, K::Esc, M::NONE);
        recorded_workflow_key(&mut app, K::Char('/'), M::NONE);
        recorded_workflow_text(&mut app, "break");
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        assert_eq!(app.browser.in_hand, Some(root.join("samples/break.wav")));
        recorded_workflow_hit(&mut app, 'e');
        recorded_workflow_loads(&mut app, &incoming);
        assert!(app.pads.sound[6].is_some() && app.browser.in_hand.is_none());
        recorded_workflow_key(&mut app, K::F(2), M::NONE);
        recorded_workflow_key(&mut app, K::Char('m'), M::NONE);
        for pad in ['q', 'w'] {
            recorded_workflow_hit(&mut app, pad);
        }
        recorded_workflow_key(&mut app, K::Char('m'), M::NONE);
        assert_eq!(app.pads.scope_pads(), [4, 5]);
        assert_eq!(panel_now(&mut app).scope, "2 marked");
        assert_eq!(app.pads.selected_abs(), 5);
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        recorded_workflow_key(&mut app, K::Char('k'), M::NONE);
        recorded_workflow_key(&mut app, K::Char('k'), M::NONE);
        recorded_workflow_key(&mut app, K::Char('l'), M::NONE);
        assert_eq!(app.pads.group[4..7], [1, 1, 0]);
        recorded_workflow_key(&mut app, K::Char('z'), M::CONTROL);
        assert_eq!(app.pads.group[4..7], [0, 0, 0]);
        assert_eq!(flash(&app), "undo: mute group on 2 pads");
        page_knob(&mut app, 2, 0.25);
        assert_eq!(app.pads.group[4..7], [2, 2, 0]);
        recorded_workflow_key(&mut app, K::BackTab, M::SHIFT);
        assert_eq!(app.pads.page().0, "FX");
        page_knob(&mut app, 0, 0.4);
        page_knob(&mut app, 1, 0.3);
        page_knob(&mut app, 2, 0.2);
        assert_eq!(app.pads.sends[4..7], [[30, 20], [30, 20], [0, 0]]);
        let before_fx: Vec<_> = (0..super::PADS).map(|p| app.pads.snapshot(p)).collect();
        recorded_workflow_key(&mut app, K::F(4), M::NONE);
        page_knob(&mut app, 1, 0.7);
        let fx = app.pads.fx;
        assert_eq!(
            (0..super::PADS)
                .map(|p| app.pads.snapshot(p))
                .collect::<Vec<_>>(),
            before_fx
        );
        assert_eq!(app.pads.scope_pads(), [4, 5]);
        recorded_workflow_key(&mut app, K::F(2), M::NONE);
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        let before: Vec<_> = (0..super::PADS).map(|p| app.pads.snapshot(p)).collect();
        recorded_workflow_key(&mut app, K::Backspace, M::NONE);
        assert_eq!(
            app.ask.as_ref().unwrap().lines()[0],
            "Reset 2 pads to clean?"
        );
        recorded_workflow_key(&mut app, K::Char('l'), M::NONE);
        assert_eq!(
            (0..super::PADS)
                .map(|p| app.pads.snapshot(p))
                .collect::<Vec<_>>(),
            before
        );
        recorded_workflow_key(&mut app, K::Esc, M::NONE);
        assert!(app.ask.is_none() && app.edit);
        assert_eq!(
            (0..super::PADS)
                .map(|p| app.pads.snapshot(p))
                .collect::<Vec<_>>(),
            before
        );
        recorded_workflow_key(&mut app, K::Backspace, M::NONE);
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        for abs in [4, 5] {
            assert_eq!(app.pads.group[abs], 0);
            assert_eq!(app.pads.drive[abs], 0);
            assert_eq!(app.pads.sends[abs], [0, 0]);
            assert!(app.pads.file[abs].is_some() && app.pads.sound[abs].is_some());
        }
        assert!(app.edit);
        assert_eq!(app.pads.fx, fx);
        recorded_workflow_key(&mut app, K::Char('z'), M::CONTROL);
        assert_eq!(flash(&app), "undo: reset on 2 pads");
        assert_eq!(
            (0..super::PADS)
                .map(|p| app.pads.snapshot(p))
                .collect::<Vec<_>>(),
            before
        );
        recorded_workflow_key(&mut app, K::Esc, M::NONE);
        assert!(!app.edit);
        assert_eq!(app.pads.scope_pads(), [4, 5]);
        recorded_workflow_key(&mut app, K::Backspace, M::NONE);
        assert_eq!(app.ask.as_ref().unwrap().lines()[0], "Clear 2 pads?");
        recorded_workflow_key(&mut app, K::Esc, M::NONE);
        assert!(app.ask.is_none());
        assert_eq!(
            (0..super::PADS)
                .map(|p| app.pads.snapshot(p))
                .collect::<Vec<_>>(),
            before
        );
        recorded_workflow_key(&mut app, K::Backspace, M::NONE);
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        assert!(app.pads.file[4].is_none() && app.pads.file[5].is_none());
        assert_eq!(app.pads.snapshot(6), before[6]);
        recorded_workflow_key(&mut app, K::Char('z'), M::CONTROL);
        assert_eq!(flash(&app), "undo: clear on 2 pads");
        assert_eq!(
            (0..super::PADS)
                .map(|p| app.pads.snapshot(p))
                .collect::<Vec<_>>(),
            before
        );
        recorded_workflow_key(&mut app, K::Esc, M::NONE);
        assert!(app.pads.marked.is_empty());
        assert_eq!(panel_now(&mut app).scope, "pad W");
        recorded_workflow_hit(&mut app, 'q');
        recorded_workflow_key(&mut app, K::Char('x'), M::CONTROL);
        assert_eq!(app.pad_in_hand, Some(4));
        recorded_workflow_hit(&mut app, 'w');
        assert!(app.pad_in_hand.is_none());
        assert_eq!(app.pads.snapshot(4), before[5]);
        assert_eq!(app.pads.snapshot(5), before[4]);
        recorded_workflow_key(&mut app, K::Char('z'), M::CONTROL);
        assert_eq!(flash(&app), "undo: swap on 2 pads");
        assert_eq!(
            (0..super::PADS)
                .map(|p| app.pads.snapshot(p))
                .collect::<Vec<_>>(),
            before
        );
        assert_eq!(app.pads.selected_abs(), 5);
        assert_eq!(app.pads.fx, fx);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recorded_f4_from_each_screen_preserves_selection_scope_and_overlays() {
        use input::Group;
        use ratatui::crossterm::event::{KeyCode as K, KeyModifiers as M};
        let (keymap, _) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        let (mut app, root, incoming) = recorded_workflow_app("recorded-fx-navigation");
        recorded_workflow_project(&mut app, "A");
        recorded_workflow_file(&mut app, &incoming, 'q', "break");
        recorded_workflow_key(&mut app, K::Char('a'), M::CONTROL);
        let marked = app.pads.marked.clone();
        let scope = app.pads.scope_pads();
        let selected = app.pads.selected_abs();
        recorded_workflow_key(&mut app, K::Enter, M::NONE);
        recorded_workflow_key(&mut app, K::BackTab, M::SHIFT);
        recorded_workflow_key(&mut app, K::Char('k'), M::NONE);
        let pad_param = app.pads.param;
        recorded_workflow_key(&mut app, K::F(1), M::NONE);
        recorded_workflow_key(&mut app, K::Esc, M::NONE);
        assert!(!app.help_open && app.edit);
        recorded_workflow_key(&mut app, K::F(10), M::NONE);
        assert_eq!(app.learn_group(), Group::PadsEdit);
        assert_eq!(app.screen_learn.as_ref().unwrap().page, Some("FX"));
        recorded_workflow_key(&mut app, K::Esc, M::NONE);
        assert!(app.screen_learn.is_none() && app.edit);
        let snapshots: Vec<_> = (0..super::PADS).map(|p| app.pads.snapshot(p)).collect();
        let effects = app.pads.fx;
        recorded_workflow_key(&mut app, K::F(4), M::NONE);
        assert!(!app.edit);
        assert_eq!(app.pads.param, pad_param);
        for (key, screen, group) in [
            (2, Screen::Pads, Group::Pads),
            (3, Screen::Chop, Group::Chop),
            (6, Screen::Files, Group::Files),
            (12, Screen::Settings, Group::Files),
            (4, Screen::Fx, Group::Fx),
        ] {
            recorded_workflow_key(&mut app, K::F(key), M::NONE);
            assert_eq!(app.screen, screen);
            assert!(!app.edit);
            if screen == Screen::Files && !app.browser.filter.is_empty() {
                recorded_workflow_key(&mut app, K::Esc, M::NONE);
            }
            recorded_workflow_loads(&mut app, &incoming);
            let browser_selection = app.browser.current().map(|e| e.name.clone());
            recorded_workflow_key(&mut app, K::F(1), M::NONE);
            assert!(app.help_open);
            recorded_workflow_key(&mut app, K::F(4), M::NONE);
            assert_eq!(app.screen, Screen::Fx);
            assert!(app.help_open);
            assert_eq!(app.learn_group(), Group::Fx);
            assert_eq!(
                app.help_state().page,
                if app.fx_slot == 0 { "FX 1" } else { "FX 2" }
            );
            let help = line(&super::help_groups(&keymap, app.screen, &app.help_state()));
            assert!(
                help.iter()
                    .any(|(_, text)| text.contains(&format!("FX {} selected", app.fx_slot + 1)))
            );
            assert!(
                super::full_help(&keymap, app.screen, true)
                    .iter()
                    .any(|(_, text)| text.contains("shared FX block"))
            );
            recorded_workflow_key(&mut app, K::Esc, M::NONE);
            assert!(!app.help_open);
            assert_eq!(app.screen, Screen::Fx);
            recorded_workflow_key(&mut app, K::Tab, M::NONE);
            let slot = app.fx_slot;
            assert_eq!(app.fx_view().selected, slot);
            recorded_workflow_key(&mut app, K::Char('k'), M::NONE);
            assert_eq!(app.fx_view().param, 1);
            recorded_workflow_key(&mut app, K::F(10), M::NONE);
            assert_eq!(app.learn_group(), Group::Fx);
            let sent = app.sent.len();
            recorded_workflow_hit(&mut app, 'w');
            assert_eq!(app.learn_picked().unwrap().0, "pad.6");
            recorded_workflow_key(&mut app, K::F(4), M::NONE);
            assert_eq!(app.learn_picked().unwrap().0, "screen.fx");
            assert_eq!(app.screen, Screen::Fx);
            assert_eq!(app.fx_slot, slot);
            assert_eq!(app.sent.len(), sent, "learn keys do not act");
            recorded_workflow_key(&mut app, K::F(10), M::NONE);
            assert!(app.screen_learn.is_none());
            recorded_workflow_key(&mut app, K::F(2), M::NONE);
            assert_eq!(app.screen, Screen::Pads);
            assert_eq!(app.pads.param, pad_param);
            assert_eq!(app.pads.selected_abs(), selected);
            assert_eq!(app.pads.marked, marked);
            assert_eq!(app.pads.scope_pads(), scope);
            assert_eq!(panel_now(&mut app).scope, "16 marked");
            assert_eq!(app.pads.fx, effects);
            assert_eq!(
                (0..super::PADS)
                    .map(|p| app.pads.snapshot(p))
                    .collect::<Vec<_>>(),
                snapshots
            );
            assert_eq!(
                app.browser.current().map(|e| e.name.clone()),
                browser_selection
            );
            recorded_workflow_key(&mut app, K::F(key), M::NONE);
            assert_eq!(app.screen, screen);
            if screen != Screen::Settings {
                assert_eq!(app.layer_group(), group);
                recorded_workflow_key(&mut app, K::F(10), M::NONE);
                assert_eq!(app.learn_group(), group);
                let sent = app.sent.len();
                recorded_workflow_key(&mut app, K::F(4), M::NONE);
                assert_eq!(app.screen, screen, "learn selects F4 instead of opening it");
                assert_eq!(app.learn_picked().unwrap().0, "screen.fx");
                assert_eq!(app.sent.len(), sent);
                recorded_workflow_key(&mut app, K::Esc, M::NONE);
                assert!(app.screen_learn.is_none());
            }
            recorded_workflow_key(&mut app, K::F(4), M::NONE);
            assert_eq!(app.screen, Screen::Fx);
            recorded_workflow_key(&mut app, K::F(key), M::NONE);
            assert_eq!(app.screen, screen, "F4 can return to each screen");
            assert_eq!(app.pads.scope_pads(), scope);
            assert_eq!(app.pads.selected_abs(), selected);
            recorded_workflow_key(&mut app, K::F(2), M::NONE);
        }
        assert_eq!(app.pads.marked, marked);
        std::fs::remove_dir_all(root).unwrap();
    }

    /// Scenario "Knob with soft takeover" (master-volume): at −10 dB a learned knob standing
    /// at its maximum puts the master at +6 dB at once (the jump response, 58); the engine
    /// glides there (`master_gain_changes_smoothly`).
    #[test]
    fn knob_with_soft_takeover() {
        let mut app = test_app();
        app.set_master(-10);
        app.sent.clear();
        app.midi_event(super::midi::Event::Param {
            port: "X-Session".into(),
            control: "cc 40".into(),
            param: input::midi::Param::MasterVolume,
            value: 1.0,
            takeover: true,
        });
        assert_eq!(app.master_db, 6);
        assert_eq!(master_gains(&app), [10f32.powf(6.0 / 20.0)]);
    }
}

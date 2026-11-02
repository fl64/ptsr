mod audio;
mod browser;
mod chop;
mod config;
mod learn;
mod loader;
mod meters;
mod midi;
mod pads;
mod session;
mod wav;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

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
/// How long a one-off message stays (save error, buffer size rejected).
const FLASH: Duration = Duration::from_secs(5);
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
    sess.chops = sess
        .chops
        .into_iter()
        .map(|(file, st)| {
            (
                session::to_stored_path(folder, Path::new(&file))
                    .to_string_lossy()
                    .into_owned(),
                st,
            )
        })
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
    sess.chops = sess
        .chops
        .into_iter()
        .map(|(file, st)| {
            (
                session::from_stored_path(folder, Path::new(&file))
                    .to_string_lossy()
                    .into_owned(),
                st,
            )
        })
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
            | Ask::ClearPads { lines, .. } => lines,
        }
    }
}

struct App {
    audio: Option<Audio>,
    cfg: AudioConfig,
    /// Where to save choices; `None` means the user file is broken and left alone.
    user_path: Option<PathBuf>,
    routed: Routed,
    screen: Screen,
    settings_row: usize,
    help_open: bool,
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
    /// `false` when the session file could not be read: it is then left alone.
    session_ok: bool,
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
    flash: Option<(String, Instant)>,
    /// Startup notices (config, session, keymap, terminal): the first key event of the
    /// session clears them (22), until then they hold the notice line.
    startup: Vec<String>,
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
            self.flash = Some((e, Instant::now()));
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
        self.stop_preview();
        self.edit = false;
        self.set_master(sess.master_db);
        self.pads = pads::Pads::default();
        self.chop = None;
        self.plan = None;
        self.plan_undo.clear();
        self.history.clear();
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
                first_pad.insert(path.clone(), abs);
                self.load_pad(abs, path, region);
            } else {
                log::warn!("session: {} is gone", path.display());
                self.pads.missing[abs] = true;
                self.send(Command::SetPadSound {
                    pad: abs,
                    sound: None,
                });
            }
        }
        self.session_dirty = None;
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
        self.flash = Some((
            result.map_or_else(|e| e, |_| format!("{what}: {shown}")),
            Instant::now(),
        ));
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
            self.flash = Some((e, Instant::now()));
        }
    }

    /// The keys of the open folder picker (11): Up/Down choose, Right enters a folder,
    /// Left goes up (at `/` it stays), Enter picks the folder it is in, Esc cancels;
    /// the rest do nothing.
    fn picker_key(&mut self, action: Action) {
        let Some(mut picker) = self.path_picker.take() else {
            return;
        };
        match action {
            Action::SettingsRow(step) => picker.move_cursor(step as isize),
            Action::SettingsValue(step) => {
                if step > 0 {
                    if let Some(dir) = picker.entries.get(picker.selected).cloned() {
                        picker.at = dir;
                        picker.entries = folders_of(&picker.at);
                        picker.selected = 0;
                    }
                } else if let Some(parent) = picker.at.parent().map(Path::to_path_buf) {
                    let from = picker.at.file_name().map(|n| n.to_owned());
                    picker.at = parent;
                    picker.entries = folders_of(&picker.at);
                    // the folder we came from stays under the cursor, as the files screen does
                    picker.selected = from
                        .and_then(|n| {
                            picker
                                .entries
                                .iter()
                                .position(|e| e.file_name() == Some(&n))
                        })
                        .unwrap_or(0);
                }
            }
            Action::SettingsLearn => {
                let (which, at) = (picker.which, picker.at.clone());
                self.paths_set(which, at);
                return; // picked: the picker stays closed
            }
            Action::Back => return, // cancelled: the picker stays closed
            _ => {}
        }
        self.path_picker = Some(picker);
    }

    /// (11) Opens a project: its `project.toml` read back becomes the whole app state; a
    /// file that cannot be read leaves the empty session and says so, as the start does
    /// with a broken session file.
    fn open_project(&mut self, folder: PathBuf) {
        let (sess, notice) = match session::load(&folder.join("project.toml")) {
            Ok(s) => (s, None),
            Err(e) => {
                log::error!("{e}");
                (session::Session::default(), Some(e))
            }
        };
        // a readable session may be saved back, a broken one is left alone (as at start)
        self.session_ok = notice.is_none();
        let sess = with_resolved_paths(sess, Some(&folder));
        let name = file_name(&folder);
        self.project = Some(Project {
            name: name.clone(),
            folder: folder.clone(),
        });
        self.apply_session(sess);
        self.flash = Some((
            notice.unwrap_or_else(|| format!("Project {name}")),
            Instant::now(),
        ));
        // the last opened name goes into the config; the next start reopens it (11)
        self.project_cfg.last = Some(name);
        self.save_project_cfg();
    }

    /// (11) Makes a project folder in the projects root and opens it: `project.toml` with
    /// an empty session and an empty `samples/`. A name that is already a project just
    /// opens; an empty name does nothing.
    fn create_project(&mut self, name: String) {
        if name.trim().is_empty() {
            return;
        }
        let folder = self.paths.projects.join(&name);
        if folder.join("project.toml").is_file() {
            self.open_project(folder);
            return;
        }
        let result = std::fs::create_dir_all(&folder)
            .and_then(|()| std::fs::create_dir_all(folder.join("samples")))
            .map_err(|e| format!("{}: {e}", folder.display()))
            .and_then(|()| {
                session::save(&folder.join("project.toml"), &session::Session::default())
            });
        if let Err(e) = result {
            log::error!("{e}");
            self.flash = Some((e, Instant::now()));
            return;
        }
        self.open_project(folder);
    }

    fn send(&mut self, cmd: Command) {
        // the first sound ends the splash (73): a note that can sound, or a preview
        let sounds = match &cmd {
            Command::PadPress { pad, .. } => self.pad_sounds(*pad),
            Command::Preview { sound: Some(_), .. } => true,
            _ => false,
        };
        if sounds {
            self.splash = false;
        }
        #[cfg(test)]
        self.sent.push(cmd.clone());
        if let Some(a) = &mut self.audio {
            // queue full: the command is dropped, audio never waits
            let _ = a.commands.push(cmd);
        }
    }

    /// Runs an action; returns `true` to quit.
    fn apply(&mut self, action: Action) -> bool {
        let was = self.screen;
        let cuts = self.chop.as_ref().map(|c| c.saved());
        self.note_done(action);
        // a change of the pads made on the pads or files screen is a step of the undo (61)
        let undoable =
            matches!(self.screen, Screen::Pads | Screen::Files) && action != Action::Undo;
        let what = self.step_name(action);
        let before = undoable.then(|| self.before());
        let quit = self.run(action);
        if let Some(b) = before {
            self.note_step(b, what);
        }
        self.follow_loop();
        let mut relinked = 0;
        if action == Action::Undo && self.screen == Screen::Chop && cuts.is_some() {
            relinked = self.relink_after_undo();
        }
        self.cuts_changed(cuts);
        if matches!(self.screen, Screen::Chop) || was == Screen::Chop {
            // pads linked back by the undo changed too, even when their bounds did not move
            let changed = self.sync_links() + relinked;
            self.say_changed(changed, action == Action::Undo);
        }
        // the preview plays only while the files or the chop screen is open
        if was != self.screen && matches!(was, Screen::Files | Screen::Chop) {
            self.stop_preview();
        }
        if was != self.screen && (was == Screen::Settings || self.screen == Screen::Settings) {
            self.reroute();
        }
        // the chop screen opens on silence: sounding pads fade out without a click (24)
        if was != self.screen && self.screen == Screen::Chop {
            self.send(Command::HushPads);
        }
        if self.screen != Screen::Chop {
            self.plan = None;
            if matches!(self.ask, Some(Ask::Plan { .. })) {
                self.ask = None;
            }
        }
        if was != Screen::Files && self.screen == Screen::Files {
            self.preview_selection();
        }
        quit
    }

    fn set_master(&mut self, db: i8) {
        self.master_db = db.clamp(pads::MIN_DB, pads::MAX_DB);
        let gain = 10f32.powf(self.master_db as f32 / 20.0);
        self.send(Command::SetMasterGain { gain });
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
                    let name = std::mem::take(name);
                    self.create_project(name);
                    return true; // the project is open, the picker is gone
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
                if let Some(name) = picker.entries.get(picker.selected).cloned() {
                    self.open_project(self.paths.projects.join(name));
                    return true; // the project is open, the picker is gone
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

    /// The project picker as drawn: the open project is marked (35).
    fn picker_view(&self) -> Option<tui::ProjectPickerView> {
        let open = self.project.as_ref().map(|p| p.name.as_str());
        self.project_picker
            .as_ref()
            .map(|p| tui::ProjectPickerView {
                entries: p
                    .entries
                    .iter()
                    .map(|name| match open {
                        Some(o) if o == name => format!("{name}  ● open"),
                        _ => name.clone(),
                    })
                    .collect(),
                selected: p.selected,
                naming: p.naming.clone(),
            })
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
        if self.recording.is_some() {
            return;
        }
        let Some(a) = self.audio.as_ref() else {
            self.flash = Some((
                "No sound: there is nothing to record from.".into(),
                Instant::now(),
            ));
            return;
        };
        if source == audio::SOURCE_MIC && self.routed.input.is_none() {
            self.flash = Some((
                "No microphone is connected: nothing to record from.".into(),
                Instant::now(),
            ));
            return;
        }
        if source == audio::SOURCE_OUTPUT {
            let tapped = a.tap_output(&self.routed.playback);
            if !tapped {
                self.flash = Some((
                    "The output device has no monitor to record.".into(),
                    Instant::now(),
                ));
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
            self.flash = Some((
                "The recording was silent: nothing is kept.".into(),
                Instant::now(),
            ));
            return;
        };
        if let Err(e) = std::fs::create_dir_all(root) {
            self.flash = Some((
                format!("Cannot create {}: {e}", root.display()),
                Instant::now(),
            ));
            return;
        }
        let path = free_name(root, &stamp(std::time::SystemTime::now()));
        if let Err(e) = wav::write(&path, &sample, self.rate) {
            self.flash = Some((e, Instant::now()));
            return;
        }
        let secs = sample.left.len() as f32 / self.rate as f32;
        let cap = if capped {
            ", stopped at the 30 s cap"
        } else {
            ""
        };
        self.flash = Some((
            format!("Recorded {secs:.1} s{cap} · hit a pad"),
            Instant::now(),
        ));
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

    /// The lit thing while it is still lit.
    fn lit_now(&self) -> Option<Lit> {
        self.lit
            .filter(|(_, at)| at.elapsed() < LIT_FOR)
            .map(|(l, _)| l)
    }

    /// The pad panel's marks: the edit frame with its page and knobs, the lit parameter.
    fn mark_pads(&self, mut v: tui::Pads) -> tui::Pads {
        if self.edit && self.screen == Screen::Pads {
            let (page, params) = self.pads.page();
            v.edit = Some(page);
            v.pages = pads::page_names();
            if page == "FILTER" {
                v.curve = Some(filter_curve(
                    self.pads.filter_settings(self.pads.selected_abs()),
                ));
            }
            v.knobs = params.iter().map(|&p| tui_param(p)).collect();
            // a shared slot's block stands under the FX block on its page only (56)
            if let Some(k) = params.first().and_then(|p| p.slot()) {
                v.slot = Some((
                    page,
                    format!("FX {} * shared", k + 1),
                    params
                        .iter()
                        .map(|&p| {
                            (
                                tui_param(p),
                                label8(self.pads.name(p)),
                                self.pads.fx_text(self.pads.selected_abs(), p),
                            )
                        })
                        .collect(),
                ));
                v.scope = "all pads".into();
            }
        }
        if let Some(Lit::Pad(p)) = self.lit_now() {
            v.lit = Some(tui_param(p));
        }
        // the learn mode labels the grid and the panel with their controls (47, 52)
        self.learn_grid(&mut v.pads);
        self.learn_panel(&mut v);
        v.hidden = self.ui.hide_pads;
        v
    }

    /// The chop screen's marks: the edit name on the slice line and the bound in play — a
    /// held modifier's (Ctrl the start, Alt the end), the picked one in the edit, the one
    /// that just moved.
    fn mark_chop<'a>(&self, mut v: tui::Chop<'a>, held: Option<input::Modifier>) -> tui::Chop<'a> {
        v.edit = self.edit;
        // the plan is checked on the grid: the assign mode shows it even with the pads hidden (32)
        v.pads.hidden = self.ui.hide_pads && self.plan.is_none();
        v.hot = match (held, self.lit_now()) {
            (Some(input::Modifier::Ctrl), _) => Some(false),
            (Some(input::Modifier::Alt), _) => Some(true),
            _ if self.edit => Some(self.slice_end),
            (_, Some(Lit::Bound(end))) => Some(end),
            _ => None,
        };
        v
    }

    /// What the help line says about the states of this moment.
    fn help_state(&self) -> HelpState {
        HelpState {
            hand: match (self.browser.in_hand.is_some(), self.pad_in_hand.is_some()) {
                (true, _) => Some("hit a pad to put it there"),
                (false, true) => Some("hit a pad to swap them"),
                _ => None,
            },
            preview_on: self.preview_on,
            assign: self.plan.is_some(),
            snap: self.chop.as_ref().map_or(chop::Snap::Off, |c| c.snap),
            tab: self.settings_tab,
            param_name: match (self.screen, self.slice_end) {
                (Screen::Chop, true) => "end",
                (Screen::Chop, false) => "start",
                _ => self.pads.param_name(),
            },
            page: match self.screen {
                Screen::Chop => "BOUNDS",
                _ => self.pads.page().0,
            },
            play: self.chop.as_ref().map_or("to end", |c| c.play.name()),
            looping: self.chop.as_ref().is_some_and(|c| c.looping),
            as_pad: self.chop.as_ref().is_some_and(|c| c.as_pad),
            marking: self.pads.marking,
            scope: self.pads.scope_name(),
            edit: self.edit,
        }
    }

    /// Switches the screen; the edit mode does not survive the switch.
    fn goto(&mut self, screen: Screen) {
        let from = self.screen;
        self.screen = screen;
        self.edit = false;
        self.midi_run = None;
        // a file saved in the folder while the program runs shows on the next visit (66)
        if screen == Screen::Files && from != Screen::Files {
            self.browser.reread();
        }
        if screen == Screen::Settings {
            // the MIDI tab opens folded, with the group of the screen it came from (28)
            self.midi_open = vec![match from {
                Screen::Pads => input::Group::Pads,
                Screen::Files => input::Group::Files,
                Screen::Chop => input::Group::Chop,
                Screen::Settings => input::Group::Global,
            }];
            self.midi_row = self.midi_row.min(self.midi_list().len().saturating_sub(1));
        }
    }

    fn run(&mut self, action: Action) -> bool {
        // the folder picker is open: the keys go to it before the screen (11)
        if self.path_picker.is_some() {
            self.picker_key(action);
            return false;
        }
        match action {
            Action::Quit => return true,
            // a question waits for Enter (the key of the step itself) or Esc only
            a if self.ask.is_some() => match a {
                // the settings Enter answers the reset question (59)
                Action::SettingsLearn if matches!(&self.ask, Some(Ask::ResetProfile { .. })) => {
                    if let Some(Ask::ResetProfile { name, port, .. }) = self.ask.take() {
                        self.seed_device(&port, &name, true);
                        self.flash =
                            Some((format!("{name}: bindings from its profile"), Instant::now()));
                    }
                }
                Action::ChopCommit | Action::ChopEdit | Action::BrowserHold | Action::PadEdit => {
                    match self.ask.as_ref() {
                        Some(Ask::Plan { .. }) => self.apply_plan(),
                        Some(Ask::ResetProfile { .. }) => {}
                        Some(Ask::ResetPads { pads, .. }) => {
                            let pads = pads.clone();
                            self.ask = None;
                            self.reset_pads(&pads);
                        }
                        Some(Ask::ClearPads { pads, .. }) => {
                            let pads = pads.clone();
                            self.ask = None;
                            self.clear_pads(&pads);
                        }
                        Some(Ask::PutFile { abs, path, .. }) => {
                            let (abs, path) = (*abs, path.clone());
                            self.put_file(abs, path);
                        }
                        None => {}
                    }
                }
                // the files Enter answered about the pad it loads onto: after the load the
                // list is done; the peek keeps the question visible while put_file answers it
                Action::BrowserLoad if matches!(&self.ask, Some(Ask::PutFile { .. })) => {
                    if let Some(Ask::PutFile { abs, path, .. }) = self.ask.as_ref() {
                        let (abs, path) = (*abs, path.clone());
                        self.put_file(abs, path);
                        if self.ask.is_none() {
                            self.goto(Screen::Pads);
                        }
                    }
                }
                Action::Back => self.ask = None,
                // a pad pressed before the question must still let go, or a gate pad hangs
                Action::PadRelease { .. } => {
                    for cmd in self.pads.apply(a) {
                        self.send(cmd);
                    }
                }
                _ => {}
            },
            // the assign mode is deaf: only the pads, Tab, Enter, Esc and the assign
            // keys act; the wave keys would cut, zoom or move while the owner lays
            // slices onto pads (21)
            // assigning: J L pick the slice the next pad hit plans, and play it (35)
            Action::CursorMove { right, .. }
                if self.screen == Screen::Chop && self.plan.is_some() =>
            {
                if let Some(c) = self.chop.as_mut() {
                    c.select(if right { 1 } else { -1 });
                    let slice = c.selected;
                    self.play_slice(slice);
                }
            }
            a if self.screen == Screen::Chop && self.plan.is_some() && assign_deaf(&a) => {
                self.flash = Some((
                    "assigning: Enter applies, Esc leaves".into(),
                    Instant::now(),
                ));
            }
            Action::Click => self.send(Command::Click {
                velocity: CLICK_VELOCITY,
            }),
            Action::SessionSave => {
                // the manual save speaks (22); an error flash from the save wins
                if self.save_session() {
                    self.flash = Some(("Session saved".into(), Instant::now()));
                }
            }
            Action::RecordMic(_) | Action::RecordOutputToggle => {
                match record_step(action, self.recording.map(|(s, _)| s)) {
                    Some(Rec::Start(source)) => self.record_start(source),
                    Some(Rec::Stop) => self.record_stop(),
                    None => {}
                }
            }
            Action::HelpToggle => self.help_open = !self.help_open,
            Action::LearnToggle
                if matches!(self.screen, Screen::Pads | Screen::Chop | Screen::Files) =>
            {
                self.screen_learn = match self.screen_learn {
                    Some(_) => None,
                    None => Some(self.learn_open()),
                };
            }
            Action::LearnToggle => {}
            Action::ProjectPick => {
                self.project_picker = Some(ProjectPicker::new(&self.paths.projects));
            }
            Action::PadsToggle => {
                self.ui.hide_pads = !self.ui.hide_pads;
                let result = match &self.user_path {
                    Some(p) => config::save_section(p, "ui", &self.ui),
                    None => Err("the pads choice is not saved: fix the config file first".into()),
                };
                if let Err(e) = result {
                    log::error!("{e}");
                    self.flash = Some((e, Instant::now()));
                }
            }
            Action::ScreenPads => self.goto(Screen::Pads),
            Action::ScreenFiles => self.goto(Screen::Files),
            Action::ScreenChop if self.screen == Screen::Files => self.open_file_chop(),
            Action::ScreenChop => self.open_chop(),
            Action::PadPress { pad, velocity } if self.screen == Screen::Chop => {
                self.chop_pad(pad, velocity)
            }
            Action::Back if self.screen == Screen::Chop && self.plan.is_some() => self.plan = None,
            Action::ChopPlay if self.screen == Screen::Chop => {
                // the cursor is the owner's: play starts from it every time, a stop leaves it
                if self.playhead().is_some() {
                    self.stop_preview();
                } else if let Some(c) = &self.chop {
                    let region = c.play_region();
                    let looped = c.looping.then_some(region);
                    let cmd = Command::Preview {
                        sound: Some(c.sound.clone()),
                        region,
                    };
                    let as_pad = self.as_pad_of(c.selected);
                    // the loop first: the preview starts already knowing it (49); and the
                    // pad it plays as (50)
                    self.send(Command::PreviewLoop { region: looped });
                    self.send(Command::PreviewAs { pad: as_pad });
                    self.loop_sent = looped;
                    self.send(cmd);
                }
            }
            Action::CursorJump { right } if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    let before = c.selected;
                    c.jump_cursor(right);
                    if c.selected != before {
                        let slice = c.selected;
                        self.play_slice(slice);
                    }
                }
            }
            Action::ChopLoopToggle if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.looping = !c.looping;
                }
                // a play going on loops from now, or plays out its turn
                if self.playhead().is_some() {
                    self.loop_sent = None;
                    if !self.chop.as_ref().is_some_and(|c| c.looping) {
                        self.send(Command::PreviewLoop { region: None });
                    }
                }
            }
            Action::ChopLoopToggle => {}
            Action::ChopAsPadToggle if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.as_pad = !c.as_pad;
                }
            }
            Action::ChopAsPadToggle => {}
            Action::ChopPlayMode if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.play = c.play.next();
                }
            }
            Action::ChopCut if self.screen == Screen::Chop => {
                // while it plays the cut lands where the sound is; the cursor stays
                let playhead = self.playhead();
                let Some(c) = self.chop.as_mut() else {
                    return false;
                };
                let before = c.cut_ids().to_vec();
                let placed = c.cut(playhead.unwrap_or(c.cursor));
                if placed == chop::Cut::Refused {
                    self.flash = Some(("too close to another cut".into(), Instant::now()));
                } else {
                    // a cut inside a slice on pads keeps them on the left part, the slice
                    // that still starts at their cut (36): they get shorter, and say so
                    let split = c.cut_ids().iter().position(|id| !before.contains(id));
                    let (file, epoch) = (c.file.clone(), c.epoch);
                    let start = split.map(|i| c.start_id(i));
                    let shorter: Vec<String> = (0..PADS)
                        .filter(|&a| {
                            start.is_some_and(|id| {
                                self.pads.file[a].as_ref() == Some(&file)
                                    && self.pads.link[a] == Some((id, epoch))
                            })
                        })
                        .map(bank_key)
                        .collect();
                    self.sync_links();
                    let snapped = (placed == chop::Cut::Snapped).then_some("marker snapped to hit");
                    let pads = match shorter.len() {
                        0 => None,
                        1 => Some(format!("pad {} is shorter now; Ctrl+Z returns", shorter[0])),
                        _ => Some(format!(
                            "pads {} are shorter now; Ctrl+Z returns",
                            shorter.join(", ")
                        )),
                    };
                    let text = match (snapped, pads) {
                        (Some(a), Some(b)) => Some(format!("{a}; {b}")),
                        (Some(a), None) => Some(a.to_string()),
                        (None, b) => b,
                    };
                    if let Some(t) = text {
                        self.flash = Some((t, Instant::now()));
                    }
                }
            }
            Action::CursorMove { right, fine } if self.screen == Screen::Chop => {
                let repeats = held_repeats(&mut self.bound_repeats, fine);
                if let Some(c) = self.chop.as_mut() {
                    let before = c.selected;
                    let step = c.view_step(repeats);
                    c.move_cursor(right, step);
                    // the page of slices follows the slice under the cursor
                    // the slice the cursor enters plays once (21 "The cursor selects")
                    if c.selected != before {
                        let slice = c.selected;
                        self.play_slice(slice);
                    }
                }
            }
            Action::ChopZoom { zoom_in } if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.zoom(zoom_in);
                }
            }
            Action::ChopRegionStart if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    let pos = c.cursor;
                    c.set_region_start(pos);
                }
            }
            Action::ChopRegionEnd if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    let pos = c.cursor;
                    c.set_region_end(pos);
                }
            }
            Action::ChopSnapToggle if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.snap = c.snap.next();
                }
            }
            Action::ChopRegionClear if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.clear_region();
                }
            }
            Action::ChopNormalize if self.screen == Screen::Chop => self.normalize_pad(),
            Action::ChopCommit | Action::ChopEdit if self.screen == Screen::Chop => {
                if self.plan.is_some() {
                    self.apply_plan();
                } else if action == Action::ChopEdit {
                    // listen: Enter opens the slice edit; the commit stays the device key
                    self.edit = true;
                } else if !self.edit {
                    // the edit and the assign never stand together: Esc comes first; a
                    // controller has no pads to tap, so its commit lays the slices in order
                    // from the selected pad and the next commit applies them (21)
                    self.plan = Some(chop::Plan::InOrder(Some(self.pads.selected_abs())));
                    self.plan_undo.clear();
                }
            }
            Action::ChopAssign if self.screen == Screen::Chop && !self.edit => self.start_assign(),
            Action::ChopParts | Action::ParamNext
                if self.screen == Screen::Chop && self.plan.is_some() =>
            {
                let selected = self.chop.as_ref().map_or(0, |c| c.selected);
                let next = match &self.plan {
                    Some(chop::Plan::InOrder(_)) => {
                        if let Some(c) = self.chop.as_mut() {
                            c.selected = 0;
                        }
                        chop::Plan::ByHand(Vec::new())
                    }
                    // the owner hits the start pad next
                    _ => chop::Plan::InOrder(None),
                };
                self.change_plan(next, selected);
            }
            Action::ParamNext | Action::ParamPrev if self.screen == Screen::Chop => {
                // BOUNDS has two parameters: next and previous both swap them
                self.slice_end = !self.slice_end;
            }
            // BOUNDS is the only page of a slice: the page keys keep it (54)
            Action::PageNext | Action::PagePrev if self.screen == Screen::Chop => {}
            // the value keys act on a pad or a slice; the other screens have none
            Action::ParamNext
            | Action::ParamPrev
            | Action::PageNext
            | Action::PagePrev
            | Action::EditValue { .. }
                if !matches!(self.screen, Screen::Pads | Screen::Chop) => {}
            Action::Undo if self.screen == Screen::Chop && self.plan.is_some() => {
                match self.plan_undo.pop() {
                    Some((prev, selected)) => {
                        self.plan = Some(prev);
                        if let Some(c) = self.chop.as_mut() {
                            c.select_slice(selected);
                        }
                    }
                    None => self.flash = Some(("nothing to undo".into(), Instant::now())),
                }
            }
            Action::MarkerSelect(step) if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.select(step);
                    let slice = c.selected;
                    self.play_slice(slice);
                }
            }
            Action::ChopParts
            | Action::ChopHits
            | Action::ChopEqualParts
            | Action::MarkerNudge { .. }
            | Action::EditValue { .. }
            | Action::SliceBound { .. }
            | Action::MarkerDelete
            | Action::SliceEndDelete
            | Action::Undo
                if self.screen == Screen::Chop =>
            {
                let linked = self.linked_pads();
                let end = self.slice_end;
                if let Some(c) = self.chop.as_mut() {
                    match action {
                        Action::ChopParts | Action::ChopEqualParts => {
                            c.next_parts();
                            if linked > 0 {
                                self.flash = Some((
                                    format!(
                                        "{linked} pads keep their old slices and no longer follow the cuts; Ctrl+Z links them again"
                                    ),
                                    Instant::now(),
                                ));
                            }
                        }
                        Action::ChopHits => {
                            // the cuts it replaces may be hand-made: say how many there were (68)
                            let were = c.slices().len();
                            let (db, _) = c.chop_by_hits();
                            let now = c.slices().len();
                            let mut text = format!("by hits at {db:.0} dB: {now} slices");
                            if were > 1 {
                                text.push_str(&format!(" (were {were}) · Ctrl+Z brings them back"));
                            }
                            if linked > 0 {
                                text.push_str(&format!(
                                    "; {linked} pads keep their old slices, Ctrl+Z links them again"
                                ));
                            }
                            self.flash = Some((text, Instant::now()));
                        }
                        Action::MarkerNudge { right, fine } => {
                            let ms = if fine { NUDGE_FINE_MS } else { NUDGE_MS };
                            c.nudge(false, if right { ms } else { -ms });
                            self.lit = Some((Lit::Bound(false), Instant::now()));
                        }
                        Action::EditValue { up, fine } => {
                            let step = c.view_step(held_repeats(&mut self.bound_repeats, fine));
                            c.nudge_by(end, up, step);
                            self.lit = Some((Lit::Bound(end), Instant::now()));
                        }
                        Action::SliceBound { end, right, fine } => {
                            let step = c.view_step(held_repeats(&mut self.bound_repeats, fine));
                            c.nudge_by(end, right, step);
                            // the bound just moved is the picked one: it is drawn thick and lit
                            self.slice_end = end;
                            self.lit = Some((Lit::Bound(end), Instant::now()));
                        }
                        Action::MarkerDelete => c.delete(),
                        Action::SliceEndDelete => c.delete_end(),
                        _ => {
                            if !c.undo() && !self.undo_fresh() {
                                self.flash = Some(("nothing to undo".into(), Instant::now()));
                            }
                        }
                    }
                }
            }
            Action::ScreenSettings => {
                self.goto(match self.screen {
                    Screen::Settings => Screen::Pads,
                    _ => Screen::Settings,
                });
            }
            Action::Back if self.browser.in_hand.is_some() => self.browser.in_hand = None,
            Action::Back if self.pad_in_hand.is_some() => self.pad_in_hand = None,
            // Esc on the files screen drops a filter first (66)
            Action::Back if self.screen == Screen::Files && self.browser.clear_filter() => {}
            // a pad in hand: Enter swaps with the selected pad instead of opening its edit
            Action::PadEdit if self.screen == Screen::Pads && self.pad_in_hand.is_some() => {
                self.swap_with(self.pads.selected_abs())
            }
            Action::PadEdit if self.screen == Screen::Pads && self.browser.in_hand.is_some() => {
                self.put_in_hand(self.pads.selected_abs())
            }
            Action::PadEdit if self.screen == Screen::Pads => self.edit = true,
            Action::PadTake => {
                self.browser.in_hand = None;
                self.pad_in_hand = Some(self.pads.selected_abs());
            }
            Action::PadPress { pad, .. } if self.pad_in_hand.is_some() => {
                self.swap_with(self.pads.bank * PADS_PER_BANK + pad);
                self.pads.selected = pad;
            }
            // a run waits for the next control: Esc stops it and keeps what is learned (28)
            Action::Back if self.midi_run.is_some() => self.midi_run = None,
            Action::Back if self.learning.is_some() => self.learning = None,
            // the help covers the screen: Esc closes it before the edit mode under it
            Action::Back if self.help_open => self.help_open = false,
            // the edit mode is the innermost layer of the screen: Esc closes it, the screen stays
            Action::Back if self.edit => self.edit = false,
            // Esc drops the marks first, then goes back as now
            Action::Back if self.pads.drop_marks() => {}
            Action::Back => self.goto(Screen::Pads),
            // MIDI tab: the bank keys jump between group rows
            Action::BankNext | Action::BankPrev
                if self.screen == Screen::Settings && self.settings_tab == 2 =>
            {
                let list = self.midi_list();
                if let Some(to) = group_jump(&list, self.midi_row, action == Action::BankNext) {
                    self.midi_row = to;
                }
            }
            Action::SettingsRow(step) if self.settings_tab == 2 => {
                self.midi_row = (self.midi_row as isize + step as isize)
                    .clamp(0, self.midi_list().len() as isize - 1)
                    as usize
            }
            // the Paths tab has two rows: the projects root and the samples root (11)
            Action::SettingsRow(step) if self.settings_tab == 3 => {
                self.settings_row =
                    (self.settings_row as isize + step as isize).clamp(0, 1) as usize
            }
            Action::SettingsRow(step) => {
                self.settings_row = (self.settings_row as isize + step as isize)
                    .clamp(0, SETTINGS_ROWS as isize - 1)
                    as usize
            }
            // MIDI tab: the knob response row steps its value (58)
            Action::SettingsValue(step)
                if self.settings_tab == 2
                    && matches!(
                        self.midi_list().get(self.midi_row),
                        Some(MidiListRow::Response)
                    ) =>
            {
                self.step_response(step as isize)
            }
            // MIDI tab: → opens the group of the selected row, ← closes it (28)
            Action::SettingsValue(step) if self.settings_tab == 2 => self.midi_fold(step > 0),
            // the Paths tab: Enter opens the folder picker of the selected root (11)
            Action::SettingsLearn if self.settings_tab == 3 => {
                let current = if self.settings_row == 0 {
                    self.paths.projects.clone()
                } else {
                    self.paths.samples.clone()
                };
                self.path_picker = Some(PathPicker::open(self.settings_row, &current));
            }
            Action::SettingsLearn
                if self.settings_tab == 2
                    && self.learning.as_ref().is_some_and(|l| l.pending.is_some()) =>
            {
                self.learned_save()
            }
            Action::SettingsLearn if self.settings_tab == 2 => {
                match self.midi_list().get(self.midi_row) {
                    Some(MidiListRow::Group { .. }) => self.midi_fold(true),
                    Some(MidiListRow::Response) => self.step_response(1),
                    Some(MidiListRow::Reset { name, port }) => {
                        self.ask = Some(Ask::ResetProfile {
                            lines: vec![
                                format!("Reset {name} to its profile?"),
                                "Every binding of yours for it is dropped.".into(),
                                "Enter: reset    Esc: keep".into(),
                            ],
                            name: name.clone(),
                            port: port.clone(),
                        });
                    }
                    Some(MidiListRow::Set { infos, .. }) => self.midi_run_start(infos),
                    Some(MidiListRow::Action { info }) => {
                        self.midi_run = None;
                        self.learning = Some(learn::Learning::new(info));
                    }
                    None => {}
                }
            }
            Action::SettingsClear if self.settings_tab == 2 => {
                match self.midi_list().get(self.midi_row) {
                    // the run rows clear all together (28)
                    Some(MidiListRow::Set { infos: run, .. }) => {
                        let infos: &'static [input::ActionInfo] = run;
                        let mut profiles: Vec<String> = Vec::new();
                        for info in infos {
                            for p in learn::clear(&mut self.learned, info.group, info.name) {
                                if !profiles.contains(&p) {
                                    profiles.push(p);
                                }
                            }
                        }
                        for p in profiles {
                            self.sync_learned(&p, infos[0].group);
                        }
                        self.save_midi();
                    }
                    Some(MidiListRow::Action { info }) => {
                        for profile in learn::clear(&mut self.learned, info.group, info.name) {
                            self.sync_learned(&profile, info.group);
                        }
                        self.save_midi();
                    }
                    _ => {}
                }
            }
            Action::SettingsLearn | Action::SettingsClear => {}
            Action::SettingsValue(step) => self.change(self.settings_row, step as isize),
            Action::SettingsTab => {
                self.settings_tab = (self.settings_tab + 1) % 4;
                self.learning = None;
                self.midi_run = None;
            }
            // a file in hand: the list keys choose the pad it goes to
            Action::BrowserHold
                if self.screen == Screen::Files && self.browser.in_hand.is_some() =>
            {
                self.put_in_hand(self.pads.selected_abs())
            }
            // Enter on the list: a file goes to the selected pad at once, a folder opens
            Action::BrowserLoad if self.screen == Screen::Files => {
                match self.browser.selected_file() {
                    Some(path) => {
                        self.put_file(self.pads.selected_abs(), path);
                        if self.ask.is_none() {
                            self.goto(Screen::Pads);
                        }
                    }
                    None => {
                        let dir = self.browser.dir.clone();
                        self.browser.enter();
                        if self.browser.dir != dir {
                            self.dirty();
                            self.preview_selection();
                        }
                    }
                }
            }
            Action::BrowserUp | Action::BrowserDown => {
                self.browser
                    .step(if action == Action::BrowserUp { -1 } else { 1 });
                self.preview_selection();
            }
            Action::BrowserEnter | Action::BrowserParent | Action::BrowserHold => {
                let dir = self.browser.dir.clone();
                match action {
                    Action::BrowserEnter => self.browser.enter(),
                    Action::BrowserParent => self.browser.parent(),
                    _ => self.browser.take(),
                }
                if self.browser.dir != dir {
                    self.dirty();
                    self.preview_selection();
                }
            }
            Action::MasterVolume(step) => {
                self.set_master(self.master_db + step);
                self.dirty();
            }
            Action::BrowserFilter => self.browser.editing = true,
            Action::BrowserFavorite => {
                // on a file Ctrl+B marks the folder it lies in: say so, or it reads as nothing (68)
                let of_file = self.browser.current().is_some_and(|e| !e.is_dir);
                let (name, added) = self.browser.toggle_favorite();
                let whose = if of_file {
                    " (the folder of this file)"
                } else {
                    ""
                };
                self.flash = Some((
                    if added {
                        format!("★ {name}: favorite{whose}")
                    } else {
                        format!("{name}: no longer a favorite{whose}")
                    },
                    Instant::now(),
                ));
                {
                    let section = config::BrowserConfig {
                        favorites: self.browser.favorites.clone(),
                    };
                    let result = match &self.user_path {
                        Some(p) => config::save_section(p, "browser", &section),
                        None => Err("favorites are not saved: fix the config file first".into()),
                    };
                    if let Err(e) = result {
                        log::error!("{e}");
                        self.flash = Some((e, Instant::now()));
                    }
                }
            }
            Action::BrowserPreviewToggle => {
                self.preview_on = !self.preview_on;
                self.preview_selection();
            }
            Action::PadPress { pad, .. } if self.browser.in_hand.is_some() => {
                self.put_in_hand(self.pads.bank * PADS_PER_BANK + pad);
                self.pads.selected = pad;
            }
            // the SOURCE page: the app changes it, it knows the cuts of the file (27)
            Action::EditValue { up, fine }
                if self.screen == Screen::Pads
                    && matches!(
                        self.pads.param,
                        pads::Param::Source | pads::Param::Start | pads::Param::End
                    ) =>
            {
                self.source_value(up, fine)
            }
            Action::PadParam(step)
                if self.screen == Screen::Pads
                    && matches!(
                        self.pads.param,
                        pads::Param::Source | pads::Param::Start | pads::Param::End
                    ) =>
            {
                self.source_value(step > 0, false)
            }
            Action::Undo if matches!(self.screen, Screen::Pads | Screen::Files) => self.undo_pads(),
            // in the pad edit the same key resets the settings of the scope, after a question (76)
            Action::PadClear if self.edit && self.screen == Screen::Pads => self.ask_reset(),
            // with marks the clear asks first: one key must not wipe a bank unasked (76)
            Action::PadClear if !self.pads.marked.is_empty() => self.ask_clear(),
            Action::PadClear => self.clear_pads(&[self.pads.selected_abs()]),
            pad_action => {
                if matches!(pad_action, Action::PadPress { .. }) {
                    self.meters.hit(Instant::now());
                }
                for cmd in self.pads.apply(pad_action) {
                    self.send(cmd);
                }
                let value = matches!(pad_action, Action::PadParam(_) | Action::EditValue { .. });
                if value || pad_action == Action::PadModeNext {
                    self.dirty();
                }
                if value {
                    self.lit = Some((Lit::Pad(self.pads.param), Instant::now()));
                }
                if let Some(n) = self.pads.limit_hits.take() {
                    let what = self.pads.param_name();
                    let pad = if n == 1 { "pad" } else { "pads" };
                    self.flash = Some((format!("{what}: {n} {pad} at the limit"), Instant::now()));
                } else if value {
                    // the changed value pops up, so the ear and the eye agree
                    let value = self.pads.param_value();
                    let what = self.pads.param_name();
                    self.flash = Some((format!("{what}: {value}"), Instant::now()));
                }
            }
        }
        false
    }

    /// Opens the chop screen for the selected pad's sample.
    fn open_chop(&mut self) {
        let abs = self.pads.selected_abs();
        let (Some(sound), Some(file)) = (self.pads.sound[abs].clone(), self.pads.file[abs].clone())
        else {
            let why = if self.pads.loading[abs].is_some() {
                "pad is still loading"
            } else {
                "pad is empty: put a sample on it first (F6)"
            };
            self.flash = Some((why.into(), Instant::now()));
            return;
        };
        let (region, link) = (self.pads.region[abs], self.pads.link[abs]);
        // a pad of a slice opens the chop the slice came from (45)
        let created = match link {
            Some((_, epoch)) if self.focus_epoch(file.clone(), sound.clone(), epoch) => false,
            _ => self.focus_chop(file, sound),
        };
        // a pad with the whole file means a new chop of it (42); the older one stays with
        // its pads (45), undo goes back to it
        if link.is_none()
            && region == Region::WHOLE
            && !created
            && let Some(old) = self.chop.take_if(|c| !c.is_blank())
        {
            let epoch = self.epoch_floor(&old.file).max(old.epoch + 1);
            self.fresh_undo = Some((old.file.clone(), epoch, old.epoch));
            self.chop = Some(old.fresh(epoch));
            self.older.push(old);
            self.flash = Some((
                "a new chop of this file; the older one keeps its pads, Ctrl+Z goes back to it"
                    .into(),
                Instant::now(),
            ));
        }
        self.sync_links();
        let Some(c) = self.chop.as_mut() else {
            return;
        };
        let selected = match link {
            Some((id, epoch)) if epoch == c.epoch => {
                // the pad of a slice: the picture opens on that slice (37)
                let i = c.slice_of(id);
                c.select_slice(i);
                c.view_slice(i);
                i
            }
            _ => {
                // a free slice on a new chop brings its edges as the first cuts; an existing
                // chop is left as it is, so its linked pads keep their sound
                if created && region != Region::WHOLE {
                    c.ensure_cut(region.start);
                    c.ensure_cut(region.end);
                }
                // the pad's own piece: the picture opens on it (69)
                if region != Region::WHOLE {
                    c.view_region(region);
                }
                c.slice_at(region.start)
            }
        };
        c.select_slice(selected);
        self.chop_from = Some(abs);
        self.show_bank_of_chop();
        self.goto(Screen::Chop);
    }

    /// The chop screen opens on the bank that holds slices of this chop when the bank shown
    /// holds none (36): the grid shows the real pads, where the slices lie.
    fn show_bank_of_chop(&mut self) {
        let Some(c) = &self.chop else {
            return;
        };
        let p = &self.pads;
        let ours = |a: usize| {
            p.file[a].as_ref() == Some(&c.file) && p.link[a].is_some_and(|l| l.1 == c.epoch)
        };
        let here = p.bank * PADS_PER_BANK..(p.bank + 1) * PADS_PER_BANK;
        if here.into_iter().any(ours) {
            return;
        }
        if let Some(a) = (0..PADS).find(|&a| ours(a)) {
            self.pads.bank = a / PADS_PER_BANK;
        }
    }

    /// F3 on the files screen: the selected file goes to the chop screen without a pad.
    fn open_file_chop(&mut self) {
        let Some(path) = self
            .browser
            .current()
            .filter(|e| !e.is_dir)
            .map(|e| self.browser.dir.join(&e.name))
        else {
            self.flash = Some(("select a file to chop".into(), Instant::now()));
            return;
        };
        // already in memory: on a pad or in a chop
        let sound = (0..PADS)
            .find(|&a| self.pads.file[a].as_ref() == Some(&path))
            .and_then(|a| self.pads.sound[a].clone())
            .or_else(|| {
                self.chop
                    .iter()
                    .chain(self.chops.values())
                    .find(|c| c.file == path)
                    .map(|c| c.sound.clone())
            });
        match sound {
            Some(sound) => self.show_file_chop(path, sound),
            None => {
                self.load_gen[CHOP] += 1;
                self.flash = Some((format!("Loading {}…", file_name(&path)), Instant::now()));
                self.request(CHOP, path, self.load_gen[CHOP]);
            }
        }
    }

    fn show_file_chop(&mut self, path: PathBuf, sound: Arc<engine::Sample>) {
        if self.screen == Screen::Files {
            self.stop_preview();
        }
        self.focus_chop(path, sound);
        self.plan = None;
        self.chop_from = None;
        self.show_bank_of_chop();
        self.goto(Screen::Chop);
    }

    /// Ctrl+N: the volume of the pad this chop was opened from puts the peak it plays
    /// (the pad's region) at −1 dB, rounded down. The peak comes from the sample, not
    /// the current gain, so repeating Ctrl+N lands on the same value.
    fn normalize_pad(&mut self) {
        let no_pad = "no pad to set the volume on".to_string();
        let Some(abs) = self.chop_from else {
            self.flash = Some((no_pad, Instant::now()));
            return;
        };
        let Some(sound) = self.pads.sound[abs].clone() else {
            self.flash = Some((no_pad, Instant::now()));
            return;
        };
        let (start, end) = (
            self.pads.region[abs].start.min(sound.len()),
            self.pads.region[abs].end.min(sound.len()),
        );
        let mut peak = 0.0f32;
        for ch in std::iter::once(&sound.left).chain(sound.right.as_ref()) {
            for x in &ch[start..end.max(start)] {
                peak = peak.max(x.abs());
            }
        }
        // silence: the smallest float rounds the gain up to the +6 dB pad limit
        let raw = (-1.0 - 20.0 * peak.max(f32::EPSILON).log10()).floor();
        let db = raw
            .clamp(f32::from(pads::MIN_DB), f32::from(pads::MAX_DB))
            .round() as i8;
        if db != self.pads.volume_db[abs] {
            self.pads.volume_db[abs] = db;
            let gain = 10f32.powf(db as f32 / 20.0);
            self.send(Command::SetPadGain { pad: abs, gain });
            self.dirty();
        }
        let text = if raw > f32::from(pads::MAX_DB) {
            format!("normalized: {db:+} dB, the pad volume limit stopped it")
        } else {
            format!("normalized: {db:+} dB")
        };
        self.flash = Some((text, Instant::now()));
    }

    /// Makes the chop of `file` the open one; `true` when it had to be made new.
    fn focus_chop(&mut self, file: PathBuf, sound: Arc<engine::Sample>) -> bool {
        // the newest chop of the file is open already; an older one open is put aside (45)
        if self.chop.as_ref().is_some_and(|c| c.file == file)
            && !self.chops.contains_key(&file)
            && !self.pending_chops.contains_key(&file)
        {
            return false;
        }
        if let Some(old) = self.chop.take() {
            self.park(old);
        }
        let (chop, created) = match (self.chops.remove(&file), self.pending_chops.remove(&file)) {
            (Some(c), _) => (c, false),
            (None, Some(saved)) => (chop::Chop::from_saved(file, sound, self.rate, saved), false),
            (None, None) => (chop::Chop::new(file, sound, self.rate), true),
        };
        let mut chop = chop;
        chop.reserve_epochs(self.epoch_floor(&chop.file));
        self.chop = Some(chop);
        created
    }

    /// Puts a chop aside: the newest of its file goes to `chops`, an older one to `older`.
    fn park(&mut self, c: chop::Chop) {
        let newer = self
            .chops
            .get(&c.file)
            .map(|n| n.epoch)
            .or_else(|| self.pending_chops.get(&c.file).map(|s| s.epoch));
        match newer {
            Some(e) if e > c.epoch => self.older.push(c),
            _ => {
                if let Some(s) = self.pending_chops.remove(&c.file) {
                    self.pending_older.push((c.file.clone(), s));
                }
                if let Some(prev) = self.chops.insert(c.file.clone(), c) {
                    self.older.push(prev);
                }
            }
        }
    }

    /// The open chop's place among the chops of its file, by age, and their count (45);
    /// `None` when the file has one chop.
    fn which_chop(&self) -> Option<(usize, usize)> {
        let c = self.chop.as_ref()?;
        let mut epochs: Vec<u32> = self
            .chops
            .values()
            .chain(&self.older)
            .filter(|o| o.file == c.file)
            .map(|o| o.epoch)
            .chain(self.pending_chops.get(&c.file).map(|s| s.epoch))
            .chain(
                self.pending_older
                    .iter()
                    .filter(|(f, _)| *f == c.file)
                    .map(|(_, s)| s.epoch),
            )
            .chain([c.epoch])
            .collect();
        epochs.sort_unstable();
        epochs.dedup();
        let n = epochs.len();
        (n > 1).then(|| {
            (
                epochs.iter().position(|&e| e == c.epoch).unwrap_or(0) + 1,
                n,
            )
        })
    }

    /// A chop of `file` at `epoch` is kept: put aside or waiting in the session.
    fn chop_lives(&self, file: &Path, epoch: u32) -> bool {
        self.chops
            .values()
            .chain(&self.older)
            .any(|c| c.file == file && c.epoch == epoch)
            || self
                .pending_chops
                .get(file)
                .is_some_and(|s| s.epoch == epoch)
            || self
                .pending_older
                .iter()
                .any(|(f, s)| f == file && s.epoch == epoch)
    }

    /// The chop is the newest of its file: no chop of it has a later epoch.
    fn is_newest(&self, c: &chop::Chop) -> bool {
        let later = |e: u32| e > c.epoch;
        !(self
            .chop
            .iter()
            .chain(self.chops.values())
            .chain(&self.older))
        .any(|o| o.file == c.file && later(o.epoch))
            && !self
                .pending_chops
                .get(&c.file)
                .is_some_and(|s| later(s.epoch))
    }

    /// A pad plays a slice of the chop of `file` at `epoch`.
    fn linked_to(&self, file: &Path, epoch: u32) -> bool {
        (0..PADS).any(|a| {
            self.pads.file[a].as_deref() == Some(file)
                && self.pads.link[a].is_some_and(|l| l.1 == epoch)
        })
    }

    /// The first epoch no chop of `file` and no pad of it uses (45).
    fn epoch_floor(&self, file: &Path) -> u32 {
        let chops = self
            .chop
            .iter()
            .chain(self.chops.values())
            .chain(&self.older)
            .filter(|c| c.file == file)
            .map(|c| c.epoch);
        let saved = self
            .pending_chops
            .get(file)
            .map(|s| s.epoch)
            .into_iter()
            .chain(
                self.pending_older
                    .iter()
                    .filter(|(f, _)| f == file)
                    .map(|(_, s)| s.epoch),
            );
        let pads = (0..PADS)
            .filter(|&a| self.pads.file[a].as_deref() == Some(file))
            .filter_map(|a| self.pads.link[a].map(|l| l.1));
        chops.chain(saved).chain(pads).max().unwrap_or(0) + 1
    }

    /// Opens the chop of `file` at `epoch` (45): the newest, an older one kept or waiting
    /// in the session, or one rebuilt from the pads that still play its slices; `false`
    /// when there is none.
    fn focus_epoch(&mut self, file: PathBuf, sound: Arc<engine::Sample>, epoch: u32) -> bool {
        if self
            .chop
            .as_ref()
            .is_some_and(|c| c.file == file && c.epoch == epoch)
        {
            return true;
        }
        let newest = self
            .chops
            .get(&file)
            .map(|c| c.epoch)
            .or_else(|| self.pending_chops.get(&file).map(|s| s.epoch));
        if newest == Some(epoch) {
            self.focus_chop(file, sound);
            return true;
        }
        let found = if let Some(i) = self
            .older
            .iter()
            .position(|c| c.file == file && c.epoch == epoch)
        {
            Some(self.older.remove(i))
        } else if let Some(i) = self
            .pending_older
            .iter()
            .position(|(f, s)| *f == file && s.epoch == epoch)
        {
            let (_, saved) = self.pending_older.remove(i);
            Some(chop::Chop::from_saved(
                file.clone(),
                sound,
                self.rate,
                saved,
            ))
        } else {
            self.rebuild(&file, epoch)
                .map(|saved| chop::Chop::from_saved(file.clone(), sound, self.rate, saved))
        };
        let Some(mut c) = found else {
            return false;
        };
        if let Some(old) = self.chop.take() {
            self.park(old);
        }
        c.reserve_epochs(self.epoch_floor(&file));
        self.chop = Some(c);
        true
    }

    /// A chop lost from a project saved before 45, rebuilt from the pads that still play
    /// its slices: every start and end inside the file is a cut, a start keeps its pad's id.
    fn rebuild(&self, file: &Path, epoch: u32) -> Option<chop::Saved> {
        let pads: Vec<usize> = (0..PADS)
            .filter(|&a| {
                self.pads.file[a].as_deref() == Some(file)
                    && self.pads.link[a].is_some_and(|l| l.1 == epoch)
            })
            .collect();
        let len = pads
            .iter()
            .find_map(|&a| self.pads.sound[a].as_ref().map(|s| s.len()))?;
        let mut cuts: std::collections::BTreeMap<usize, Option<u32>> = Default::default();
        for &a in &pads {
            let (r, id) = (self.pads.region[a], self.pads.link[a]?.0);
            if r.start > 0 && r.start < len {
                cuts.insert(r.start, Some(id));
            }
            if r.end > 0 && r.end < len {
                cuts.entry(r.end).or_insert(None);
            }
        }
        let mut next = pads
            .iter()
            .filter_map(|&a| self.pads.link[a].map(|l| l.0 + 1))
            .max()
            .unwrap_or(1);
        let cuts = cuts
            .into_iter()
            .map(|(at, id)| {
                let id = id.unwrap_or_else(|| {
                    next += 1;
                    next - 1
                });
                (id, at)
            })
            .collect();
        Some(chop::Saved {
            range: Region { start: 0, end: len },
            cuts,
            epoch,
            next_id: next,
        })
    }

    /// Undo past the first step of a fresh chop (45): the older chop it stood in front of
    /// is open again; the fresh one stays only if a pad plays a slice of it.
    fn undo_fresh(&mut self) -> bool {
        let Some((file, fresh, old)) = self.fresh_undo.clone() else {
            return false;
        };
        if !self
            .chop
            .as_ref()
            .is_some_and(|c| c.file == file && c.epoch == fresh)
        {
            return false;
        }
        let Some(i) = self
            .older
            .iter()
            .position(|c| c.file == file && c.epoch == old)
        else {
            return false;
        };
        let back = self.older.remove(i);
        let Some(fresh_chop) = self.chop.replace(back) else {
            return false;
        };
        if self.linked_to(&file, fresh) {
            self.park(fresh_chop);
        }
        self.fresh_undo = None;
        self.sync_links();
        true
    }

    /// Pads linked to the open chop follow its cuts: new regions go to the engine.
    /// Returns how many pads now play something else.
    fn sync_links(&mut self) -> usize {
        let Some(c) = &self.chop else {
            return 0;
        };
        let slices = c.slices();
        let (range, file) = (c.range, c.file.clone());
        let mut changed = Vec::new();
        let mut left_out = Vec::new();
        for abs in 0..PADS {
            let Some((id, epoch)) = self.pads.link[abs] else {
                continue;
            };
            if self.pads.file[abs].as_ref() != Some(&c.file) {
                continue;
            }
            // a pad of another chop of the file: it keeps its slice number while that chop
            // lives (45); else the cuts were replaced, it keeps its bounds, no number (27)
            if epoch != c.epoch {
                if !self.chop_lives(&file, epoch) {
                    self.pads.slice[abs] = None;
                }
                continue;
            }
            // the region left this pad's piece out: it keeps playing what it played (35)
            let r = self.pads.region[abs];
            if r.start < range.start || r.end > range.end {
                left_out.push((abs, self.pads.link[abs], self.pads.slice[abs]));
                continue;
            }
            let i = c.slice_of(id);
            self.pads.slice[abs] = Some(i as u16 + 1);
            if self.pads.region[abs] != slices[i] {
                self.pads.region[abs] = slices[i];
                changed.push((abs, slices[i]));
            }
        }
        for &(pad, region) in &changed {
            self.send(Command::SetPadRegion { pad, region });
        }
        if !left_out.is_empty() {
            for &(a, _, _) in &left_out {
                (self.pads.link[a], self.pads.slice[a]) = (None, None);
            }
            self.range_unlinks.push((file, left_out.clone()));
        }
        if !changed.is_empty() || !left_out.is_empty() {
            self.dirty();
        }
        changed.len() + left_out.len()
    }

    /// The status after an edit that changed laid-out pads.
    fn say_changed(&mut self, n: usize, undo: bool) {
        if n == 0 {
            return;
        }
        let pads = if n == 1 { "pad" } else { "pads" };
        let text = if undo {
            format!("{n} {pads} changed back")
        } else {
            format!("{n} {pads} changed; Ctrl+Z returns")
        };
        self.flash = Some((text, Instant::now()));
    }

    /// Slices of the open chop that no pad plays: they wear a `○` (36).
    fn slices_on_no_pad(&self) -> usize {
        let Some(c) = &self.chop else {
            return 0;
        };
        let mut on = vec![false; c.slices().len()];
        for abs in 0..PADS {
            if self.pads.file[abs].as_ref() == Some(&c.file)
                && self.pads.link[abs].is_some_and(|l| l.1 == c.epoch)
                && let Some(n) = self.pads.slice[abs]
                && let Some(o) = on.get_mut(n as usize - 1)
            {
                *o = true;
            }
        }
        on.iter().filter(|&&o| !o).count()
    }

    /// Pads that follow the open chop's cuts now.
    fn linked_pads(&self) -> usize {
        let Some(c) = &self.chop else {
            return 0;
        };
        (0..PADS)
            .filter(|&a| {
                self.pads.file[a].as_ref() == Some(&c.file)
                    && self.pads.link[a].is_some_and(|l| l.1 == c.epoch)
            })
            .count()
    }

    /// Where the chop screen playback is, in frames of the sample.
    fn playhead(&self) -> Option<usize> {
        if self.screen != Screen::Chop {
            return None;
        }
        self.preview_frame()
    }

    /// Where the preview of the files screen plays, 0..1 of the file (33); nothing while it
    /// is silent or the file has not loaded yet.
    fn preview_at(&self) -> Option<f32> {
        if self.screen != Screen::Files {
            return None;
        }
        let (secs, _) = self.preview.as_ref()?;
        let rate = self
            .audio
            .as_ref()
            .map_or(48_000, |a| a.stats.format().1.max(1));
        let frames = secs * rate as f32;
        Some((self.preview_frame()? as f32 / frames.max(1.0)).min(1.0))
    }

    /// The frame the preview voice plays, the chop playback and the file preview alike.
    fn preview_frame(&self) -> Option<usize> {
        #[cfg(test)]
        if self.fake_playhead.is_some() {
            return self.fake_playhead;
        }
        let pos = self.audio.as_ref()?.preview_pos.load(Relaxed);
        (pos != usize::MAX).then_some(pos)
    }

    /// Plays slice `slice` of the chop screen.
    fn play_slice(&mut self, slice: usize) {
        let Some(c) = &self.chop else {
            return;
        };
        if let Some(&region) = c.slices().get(slice) {
            let cmd = Command::Preview {
                sound: Some(c.sound.clone()),
                region,
            };
            let as_pad = self.as_pad_of(slice);
            self.send(Command::PreviewAs { pad: as_pad });
            self.send(cmd);
        }
    }

    /// The pad slice `slice` of the open chop plays as while "as pad" is on (50): the first
    /// pad in pad order that plays it; `None` when the switch is off or no pad plays it.
    fn as_pad_of(&self, slice: usize) -> Option<usize> {
        let c = self.chop.as_ref().filter(|c| c.as_pad)?;
        (0..PADS).find(|&a| {
            self.pads.file[a].as_ref() == Some(&c.file)
                && self.pads.link[a].is_some_and(|l| l.1 == c.epoch)
                && self.pads.slice[a] == Some(slice as u16 + 1)
        })
    }

    /// A pad hit on the chop screen: listening plays the real pad and, when it plays a
    /// slice of this chop, selects that slice (36); assigning plans the pad of the current
    /// bank and plays the slice planned there.
    fn chop_pad(&mut self, pad: usize, velocity: f32) {
        self.meters.hit(Instant::now());
        let Some(mut plan) = self.plan.clone() else {
            // no slice on a pad yet: the keys play the slices, 16 a bank page (36)
            if let Some(c) = self.chop.as_mut()
                && !(0..PADS).any(|a| {
                    self.pads.file[a].as_ref() == Some(&c.file)
                        && self.pads.link[a].is_some_and(|l| l.1 == c.epoch)
                })
            {
                let slice = slice_page(self.pads.bank, c.slices().len()) * PADS_PER_BANK + pad;
                if c.select_slice(slice) {
                    self.play_slice(slice);
                }
                return;
            }
            for cmd in self.pads.apply(Action::PadPress { pad, velocity }) {
                self.send(cmd);
            }
            let abs = self.pads.bank * PADS_PER_BANK + pad;
            if let Some(c) = self.chop.as_mut()
                && self.pads.file[abs].as_ref() == Some(&c.file)
                && let Some((id, epoch)) = self.pads.link[abs]
                && epoch == c.epoch
            {
                // the slice just heard is the one the next edit acts on
                let slice = c.slice_of(id);
                c.select_slice(slice);
            }
            return;
        };
        let Some(c) = self.chop.as_mut() else {
            return;
        };
        let abs = self.pads.bank * PADS_PER_BANK + pad;
        self.pads.selected = pad;
        let (count, before) = (c.slices().len(), c.selected);
        let by_hand = matches!(plan, chop::Plan::ByHand(_));
        let all_placed = format!("all {count} slices placed: Enter applies");
        if by_hand && plan.all_planned(count) {
            self.flash = Some((all_placed, Instant::now()));
        } else {
            plan.hit(abs, before);
            if by_hand {
                match plan.next_unplanned(count, before) {
                    Some(next) => {
                        c.select_slice(next);
                    }
                    None => self.flash = Some((all_placed, Instant::now())),
                }
            }
            self.change_plan(plan.clone(), before);
        }
        if let Some(&(slice, _)) = plan.targets(count).iter().find(|t| t.1 == abs) {
            self.play_slice(slice);
        }
    }

    /// A new plan; the old one and the slice selected before go to undo.
    fn change_plan(&mut self, plan: chop::Plan, selected: usize) {
        if let Some(old) = self.plan.replace(plan.clone())
            && old != plan
        {
            self.plan_undo.push((old, selected));
        }
    }

    /// `P`: the assign mode starts by hand from slice 1.
    fn start_assign(&mut self) {
        if self.chop.is_none() {
            return;
        }
        // tap pads one by one, from the slice just heard (04e): it stays selected
        self.plan = Some(chop::Plan::ByHand(Vec::new()));
        self.plan_undo.clear();
    }

    /// Enter in the assign mode: the planned slices go onto their pads.
    fn apply_plan(&mut self) {
        // in order with no pad hit: the selected pad is the start, the plan is shown and the
        // next Enter applies it, as the controller's commit does (67)
        if matches!(self.plan, Some(chop::Plan::InOrder(None))) {
            let start = self.pads.selected_abs();
            let slice = self.chop.as_ref().map_or(0, |c| c.selected);
            self.change_plan(chop::Plan::InOrder(Some(start)), slice);
            self.flash = Some((
                format!("from {}: Enter applies", pad_place(start)),
                Instant::now(),
            ));
            return;
        }
        let (Some(c), Some(plan)) = (&self.chop, &self.plan) else {
            return;
        };
        let (sound, file, slices) = (c.sound.clone(), c.file.clone(), c.slices());
        let link_ids: Vec<u32> = (0..slices.len()).map(|i| c.start_id(i)).collect();
        let epoch = c.epoch;
        let targets = plan.targets(slices.len());
        let taken = |p: usize| self.pads.file[p].is_some();
        let Some(summary) = chop::summary(&targets, slices.len(), taken) else {
            self.flash = Some(("nothing planned: hit a pad".into(), Instant::now()));
            return;
        };
        let lost: Vec<String> = targets
            .iter()
            .filter(|t| taken(t.1))
            .map(|&(slice, abs)| {
                format!(
                    "  {}: {} → slice {}",
                    pad_place(abs),
                    pad_name(&self.pads, abs),
                    slice + 1
                )
            })
            .collect();
        if !lost.is_empty() {
            let mut lines = vec![format!(
                "Replace what lies on {} pad{}?",
                lost.len(),
                if lost.len() == 1 { "" } else { "s" }
            )];
            lines.extend(lost);
            lines.extend(overwrite_footer());
            // what lies there is read now: a changed list is asked about again
            if !matches!(&self.ask, Some(Ask::Plan { lines: old }) if *old == lines) {
                self.ask = Some(Ask::Plan { lines });
                return;
            }
        }
        self.ask = None;
        for (slice, abs) in targets {
            let region = slices[slice];
            // a load still on its way to this pad would replace the slice
            self.load_gen[abs] += 1;
            self.pads.loading[abs] = None;
            self.pads.missing[abs] = false;
            self.pads.file[abs] = Some(file.clone());
            self.pads.set_sound(abs, sound.clone());
            self.pads.region[abs] = region;
            self.pads.slice[abs] = Some(slice as u16 + 1);
            self.pads.link[abs] = Some((link_ids[slice], epoch));
            self.send(Command::SetPadSound {
                pad: abs,
                sound: Some(sound.clone()),
            });
            self.send(Command::SetPadRegion { pad: abs, region });
        }
        self.flash = Some((format!("slices placed on {summary}"), Instant::now()));
        // the chop stays open: its cuts can still be moved and the pads follow
        self.plan = None;
        self.goto(Screen::Pads);
        self.dirty();
    }

    fn dirty(&mut self) {
        self.session_dirty.get_or_insert_with(Instant::now);
    }

    /// Where the session is written now (11): with a project open — its `project.toml`,
    /// otherwise the scratch file.
    fn session_file(&self) -> PathBuf {
        self.project.as_ref().map_or_else(
            || self.session_path.clone(),
            |p| p.folder.join("project.toml"),
        )
    }

    /// Saves the session now; `true` when the file was written. The autosave calls it
    /// the same way and stays silent (22); the manual save says "Session saved".
    fn save_session(&mut self) -> bool {
        self.session_dirty = None;
        if !self.session_ok {
            return false;
        }
        // the project folder vanished under the program (11): writing would only
        // resurrect the empty folder, so the save is refused; the pads whose files lived
        // in it are marked missing, the sound in memory keeps playing
        let gone = self
            .project
            .as_ref()
            .map(|p| p.folder.clone())
            .filter(|f| !f.is_dir());
        if let Some(folder) = gone {
            log::warn!("{} is gone; the session is not saved", folder.display());
            self.session_ok = false;
            for abs in 0..PADS {
                if self.pads.file[abs]
                    .as_ref()
                    .is_some_and(|f| f.starts_with(&folder))
                {
                    self.pads.missing[abs] = true;
                }
            }
            self.flash = Some((
                format!(
                    "{} is gone; the session is not saved",
                    tilde(&folder.display().to_string())
                ),
                Instant::now(),
            ));
            return false;
        }
        let mut s = self
            .pads
            .to_session(Some(self.browser.dir.clone()), self.rate);
        s.master_db = self.master_db;
        let open = self
            .chop
            .iter()
            .chain(self.chops.values())
            .chain(self.older.iter());
        for c in open {
            let key = c.file.to_string_lossy().into_owned();
            if self.is_newest(c) {
                s.chops.insert(key, c.saved().to_state(self.rate));
            } else if self.linked_to(&c.file, c.epoch) {
                // an older chop lives while a pad plays one of its slices (45)
                s.kept
                    .entry(key)
                    .or_default()
                    .push(c.saved().to_state(self.rate));
            }
            // a pad linked to a deleted cut is saved with the cut that starts its slice now
            for p in s.pads.values_mut() {
                if p.file.as_ref() == Some(&c.file)
                    && p.epoch == Some(c.epoch)
                    && let Some(id) = p.link
                {
                    p.link = Some(c.start_id(c.slice_of(id)));
                }
            }
        }
        for (file, saved) in &self.pending_chops {
            s.chops
                .entry(file.to_string_lossy().into_owned())
                .or_insert_with(|| saved.to_state(self.rate));
        }
        for (file, saved) in &self.pending_older {
            if self.linked_to(file, saved.epoch) {
                s.kept
                    .entry(file.to_string_lossy().into_owned())
                    .or_default()
                    .push(saved.to_state(self.rate));
            }
        }
        // with a project open the session goes into its `project.toml`, paths inside
        // the folder stored relative (11); the scratch file stays untouched
        if let Some(p) = &self.project {
            s = with_stored_paths(s, &p.folder);
        }
        if let Err(e) = session::save(&self.session_file(), &s) {
            log::error!("{e}");
            // a failed save is not retried: like a file the start could not read (11)
            self.session_ok = false;
            self.flash = Some((e, Instant::now()));
            return false;
        }
        true
    }

    fn load_pad(&mut self, abs: usize, path: PathBuf, region: Region) {
        self.load_gen[abs] += 1;
        self.pads.loading[abs] = Some(0);
        self.load_region[abs] = region;
        // the loader keeps only the newest request of a pad: pads waiting for this pad's old
        // file would never hear back, so the first of them loads it instead
        let waiting: Vec<_> = self
            .followers
            .iter()
            .filter(|f| f.0 == abs)
            .map(|f| (f.1, f.2))
            .collect();
        self.followers.retain(|f| f.0 != abs);
        if let Some(&(next, generation)) = waiting.first()
            && let Some(old) = self.pads.file[next].clone()
        {
            self.request(next, old, generation);
            self.followers
                .extend(waiting[1..].iter().map(|&(f, g)| (next, f, g)));
        }
        self.request(abs, path, self.load_gen[abs]);
    }

    fn request(&self, pad: usize, path: PathBuf, generation: u64) {
        if let Some((tx, _)) = &self.loader {
            let _ = tx.send(loader::Request {
                pad,
                path,
                generation,
            });
        }
    }

    /// `abs` gets the file `leader` is loading, without reading it again.
    fn follow(&mut self, leader: usize, abs: usize, region: Region) {
        self.load_gen[abs] += 1;
        self.pads.loading[abs] = Some(0);
        self.load_region[abs] = region;
        self.followers.push((leader, abs, self.load_gen[abs]));
    }

    /// Swaps the pad in hand with `abs`.
    fn swap_with(&mut self, abs: usize) {
        let Some(from) = self.pad_in_hand.take() else {
            return;
        };
        if from == abs {
            return;
        }
        if self.pads.loading[from].is_some() || self.pads.loading[abs].is_some() {
            // the load would land on the old place
            self.flash = Some((
                "Wait until loading finishes, then swap.".into(),
                Instant::now(),
            ));
            return;
        }
        let cmd = self.pads.swap(from, abs);
        self.send(cmd);
        self.dirty();
    }

    /// Puts the file in hand on a pad; a pad that holds something asks first.
    fn put_in_hand(&mut self, abs: usize) {
        let Some(path) = self.browser.in_hand.clone() else {
            return;
        };
        self.put_file(abs, path);
    }

    /// Loads `path` onto `abs` at once; a pad that holds something asks first. A file in
    /// hand equal to `path` empties the hand once it lies on the pad.
    fn put_file(&mut self, abs: usize, path: PathBuf) {
        if self.pads.file[abs].is_some() {
            if same_whole_sound(&self.pads, abs, &path) {
                // the pad already plays this very file, whole: nothing changes
                self.ask = None;
                if self.browser.in_hand.as_ref() == Some(&path) {
                    self.browser.in_hand = None;
                }
                self.flash = Some((
                    format!("{} already lies there", file_name(&path)),
                    Instant::now(),
                ));
                return;
            }
            let mut lines = vec![
                "Replace what lies on 1 pad?".to_string(),
                format!(
                    "  {}: {} → {}",
                    pad_place(abs),
                    pad_name(&self.pads, abs),
                    file_name(&path)
                ),
            ];
            lines.extend(overwrite_footer());
            let asked = matches!(
                &self.ask,
                Some(Ask::PutFile {
                    abs: a,
                    path: p,
                    lines: old,
                }) if *a == abs && *p == path && *old == lines
            );
            if !asked {
                self.ask = Some(Ask::PutFile {
                    abs,
                    path: path.clone(),
                    lines,
                });
                return;
            }
        }
        self.ask = None;
        if self.browser.in_hand.as_ref() == Some(&path) {
            self.browser.in_hand = None;
        }
        self.load_pad(abs, path, Region::WHOLE);
    }

    fn loader_reply(&mut self, reply: loader::Reply) {
        // pads waiting for this load get the same result, even when it is stale for its own pad
        if let loader::Reply::Loaded { pad, .. } | loader::Reply::Failed { pad, .. } = &reply {
            let pad = *pad;
            let waiting: Vec<_> = self
                .followers
                .iter()
                .filter(|f| f.0 == pad)
                .map(|f| (f.1, f.2))
                .collect();
            self.followers.retain(|f| f.0 != pad);
            for (f, generation) in waiting {
                self.loader_reply(match &reply {
                    loader::Reply::Loaded { path, sound, .. } => loader::Reply::Loaded {
                        pad: f,
                        generation,
                        path: path.clone(),
                        sound: sound.clone(),
                    },
                    loader::Reply::Failed { path, error, .. } => loader::Reply::Failed {
                        pad: f,
                        generation,
                        path: path.clone(),
                        error: error.clone(),
                    },
                    loader::Reply::Progress { .. } => continue,
                });
            }
        }
        match reply {
            loader::Reply::Loaded {
                pad: PREVIEW,
                generation,
                sound,
                ..
            } => {
                if generation == self.load_gen[PREVIEW] {
                    let rate = self
                        .audio
                        .as_ref()
                        .map_or(48_000, |a| a.stats.format().1.max(1));
                    let secs = sound.len() as f32 / rate as f32;
                    self.preview = Some((secs, loader::envelope(&sound, 1600)));
                    self.send(Command::Preview {
                        sound: Some(sound),
                        region: Region::WHOLE,
                    });
                }
            }
            loader::Reply::Failed {
                pad: PREVIEW,
                generation,
                path,
                error,
            } => {
                if generation == self.load_gen[PREVIEW] {
                    self.flash = Some((
                        format!("Cannot play {}: {error}", file_name(&path)),
                        Instant::now(),
                    ));
                }
            }
            loader::Reply::Progress {
                pad: PREVIEW | CHOP,
                ..
            } => {}
            loader::Reply::Loaded {
                pad: CHOP,
                generation,
                path,
                sound,
            } => {
                if generation == self.load_gen[CHOP] {
                    self.flash = None;
                    self.show_file_chop(path, sound);
                }
            }
            loader::Reply::Failed {
                pad: CHOP,
                generation,
                path,
                error,
            } => {
                if generation == self.load_gen[CHOP] {
                    self.flash = Some((
                        format!("Cannot chop {}: {error}", file_name(&path)),
                        Instant::now(),
                    ));
                }
            }
            loader::Reply::Progress { pad, percent } => {
                if self.pads.loading[pad].is_some() {
                    self.pads.loading[pad] = Some(percent);
                }
            }
            loader::Reply::Loaded {
                pad,
                generation,
                path,
                sound,
            } if generation == self.load_gen[pad] => {
                self.pads.loading[pad] = None;
                self.pads.missing[pad] = false;
                self.pads.file[pad] = Some(path);
                self.pads.set_sound(pad, sound.clone());
                let region = self.load_region[pad];
                self.pads.region[pad] = region;
                if region == Region::WHOLE {
                    self.pads.slice[pad] = None;
                    self.pads.link[pad] = None;
                }
                self.send(Command::SetPadSound {
                    pad,
                    sound: Some(sound),
                });
                if region != Region::WHOLE {
                    self.send(Command::SetPadRegion { pad, region });
                }
                self.dirty();
            }
            loader::Reply::Failed {
                pad,
                generation,
                path,
                error,
            } if generation == self.load_gen[pad] => {
                self.pads.loading[pad] = None;
                self.flash = Some((
                    format!("Cannot load {}: {error}", file_name(&path)),
                    Instant::now(),
                ));
                // a session file that no longer loads: the pad goes silent; a new file that
                // failed leaves the pad as it was
                if self.pads.file[pad].as_ref() == Some(&path) {
                    self.pads.missing[pad] = true;
                    self.pads.clear_sound(pad);
                    self.send(Command::SetPadSound { pad, sound: None });
                }
            }
            // a newer load for the same pad was started
            _ => {}
        }
    }

    /// The group a binding of the learn mode goes to (53): the group of the layer in sight,
    /// the edit group while the edit is open; the page knobs of the edit are learned in F12.
    fn learn_group(&self) -> input::Group {
        match &self.screen_learn {
            Some(l) => l.group,
            None => self.layer_group(),
        }
    }

    /// The group of the layer in sight (57): the edit group with the edit open.
    fn layer_group(&self) -> input::Group {
        use input::Group;
        match (self.screen, self.edit) {
            (Screen::Pads, true) => Group::PadsEdit,
            (Screen::Pads, false) => Group::Pads,
            (Screen::Chop, true) => Group::ChopEdit,
            (Screen::Chop, false) => Group::Chop,
            _ => Group::Files,
        }
    }

    /// The screen's own group, under its edit group.
    fn screen_group(&self) -> input::Group {
        use input::Group;
        match self.screen {
            Screen::Pads => Group::Pads,
            Screen::Chop => Group::Chop,
            _ => Group::Files,
        }
    }

    /// The learn mode as `F10` opens it (57): with the action done in the last 5 s picked,
    /// in its layer and page, or with nothing picked in the layer in sight.
    fn learn_open(&mut self) -> ScreenLearn {
        let last = self
            .last_done
            .take()
            .filter(|d| d.at.elapsed() <= LAST_DONE && d.group == self.layer_group());
        let mut l = ScreenLearn {
            picked: None,
            pending: None,
            clearing: false,
            learner: None,
            ignore: None,
            refused: false,
            group: self.layer_group(),
            page: (self.edit && self.screen == Screen::Pads).then(|| self.pads.page().0),
            once: false,
        };
        if let Some(d) = last {
            l.learner = Some(learn::Learning::new(action_info(d.name)));
            l.picked = Some((d.name, d.label));
            l.page = d.page;
            l.once = true;
        }
        l
    }

    /// Notes what an action did, for "do, then F10" (57): a pad hit its pad, a value step
    /// the parameter it changed, a command its entry of the learn list.
    fn note_done(&mut self, action: Action) {
        if self.screen_learn.is_some() || action == Action::LearnToggle {
            return;
        }
        let group = self.layer_group();
        let page = (self.edit && self.screen == Screen::Pads).then(|| self.pads.page().0);
        let found: Option<(&'static str, String)> = match action {
            Action::PadPress { pad, .. } => {
                let key = PAD_KEYS[pad].to_ascii_uppercase();
                Some((
                    action_info(&format!("pad.{}", pad + 1)).name,
                    format!("pad {key}"),
                ))
            }
            Action::EditValue { .. } | Action::PadParam(_) if self.screen == Screen::Pads => {
                let label = pads::name_of(self.pads.param);
                self.learn_params()
                    .iter()
                    .find(|(_, l)| *l == label)
                    .map(|&(n, l)| (n, l.to_string()))
            }
            Action::EditValue { .. } if self.screen == Screen::Chop => Some(if self.slice_end {
                ("slice.end", "slice end".into())
            } else {
                ("slice.start", "slice start".into())
            }),
            a => {
                let groups = [group, self.screen_group(), input::Group::Global];
                input::ACTIONS
                    .iter()
                    .find(|i| {
                        !i.knob && groups.contains(&i.group) && Action::parse(i.name) == Some(a)
                    })
                    .map(|i| (i.name, i.label.to_string()))
            }
        };
        if let Some((name, label)) = found {
            self.last_done = Some(LastDone {
                at: Instant::now(),
                name,
                label,
                group,
                page,
            });
        }
    }

    /// The parameters the learn mode walks with `I` `K` (52): they have no key of their own.
    fn learn_params(&self) -> &'static [(&'static str, &'static str)] {
        match self.screen {
            Screen::Pads => &[
                ("pad.volume", "volume"),
                ("pad.pitch", "pitch"),
                ("pad.attack", "attack"),
                ("pad.decay", "decay"),
                ("pad.release", "release"),
                ("pad.filter", "filter"),
                ("pad.cutoff", "cutoff"),
                ("pad.reso", "reso"),
                ("pad.start", "start"),
                ("pad.end", "end"),
            ],
            Screen::Chop => &[
                ("slice.start", "slice start"),
                ("slice.end", "slice end"),
                ("chop.cursor_pos", "cursor"),
                ("chop.zoom", "zoom"),
                ("chop.region_start_pos", "region start"),
                ("chop.region_end_pos", "region end"),
                ("chop.threshold", "threshold"),
            ],
            _ => &[],
        }
    }

    /// Picks `name` in the learn mode: it lights and waits for a control.
    fn learn_pick(&mut self, name: &'static str, label: String) {
        // a pick by a key hears every control again; the move after a binding sets `ignore`
        let (group, page, once) = match &self.screen_learn {
            Some(l) => (l.group, l.page, l.once),
            None => (self.layer_group(), None, false),
        };
        self.screen_learn = Some(ScreenLearn {
            picked: Some((name, label)),
            pending: None,
            clearing: false,
            learner: Some(learn::Learning::new(action_info(name))),
            ignore: None,
            refused: false,
            group,
            page,
            once,
        });
    }

    /// The element after `name` (53): the next pad, the next parameter, the next command of
    /// the screen's learn list; `None` past the last.
    fn learn_next(&self, name: &str) -> Option<(&'static str, String)> {
        if let Some(n) = name
            .strip_prefix("pad.")
            .and_then(|n| n.parse::<usize>().ok())
        {
            return (n < PADS_PER_BANK).then(|| {
                let key = PAD_KEYS[n].to_ascii_uppercase();
                (
                    action_info(&format!("pad.{}", n + 1)).name,
                    format!("pad {key}"),
                )
            });
        }
        let params = self.learn_params();
        if let Some(i) = params.iter().position(|(n, _)| *n == name) {
            return params.get(i + 1).map(|&(n, l)| (n, l.to_string()));
        }
        let group = self.learn_group();
        let list: Vec<_> = input::ACTIONS
            .iter()
            .filter(|i| !i.knob && (i.group == group || i.group == input::Group::Global))
            .collect();
        let at = list.iter().position(|i| i.name == name)?;
        list.get(at + 1).map(|i| (i.name, i.label.to_string()))
    }

    /// Binds the waiting or given control to the picked element and moves on (53).
    fn learn_save(&mut self, profile: String, key: input::midi::Key) {
        let group = self.learn_group();
        let Some((name, label)) = self.learn_picked() else {
            return;
        };
        learn::assign(&mut self.learned, &profile, group, key, name);
        self.sync_learned(&profile, group);
        self.save_midi();
        log::info!("MIDI learn on screen: {profile} {key} -> {name}");
        self.flash = Some((format!("{key} → {label}"), Instant::now()));
        // "do, then F10" binds one element: nothing to walk on to (57)
        if self.screen_learn.as_ref().is_some_and(|l| l.once) {
            self.screen_learn = None;
            return;
        }
        match self.learn_next(name) {
            Some((next, next_label)) => self.learn_pick(next, next_label),
            None => self.learn_pick(name, label),
        }
        if let Some(l) = self.screen_learn.as_mut() {
            l.ignore = Some(key);
        }
    }

    /// `I` `K` in the learn mode: the next or previous parameter of the screen.
    fn learn_walk(&mut self, step: isize) {
        let params = self.learn_params();
        if params.is_empty() {
            return;
        }
        let at = self
            .learn_picked()
            .and_then(|(name, _)| params.iter().position(|(n, _)| *n == name));
        let next = match at {
            Some(i) => (i as isize + step).rem_euclid(params.len() as isize) as usize,
            None if step > 0 => 0,
            None => params.len() - 1,
        };
        let (name, label) = params[next];
        self.learn_pick(name, label.into());
    }

    /// The picked element of the learn mode: (action, what the screen calls it).
    fn learn_picked(&self) -> Option<(&'static str, String)> {
        self.screen_learn.as_ref()?.picked.clone()
    }

    /// A control moved in the learn mode (53): a free one is bound at once and the pick moves
    /// on; one that drives something else waits for `Enter`; the control just bound is not
    /// heard; a knob does not drive a command.
    fn learn_bind(&mut self, profile: String, key: input::midi::Key) {
        let Some((name, _)) = self.learn_picked() else {
            return;
        };
        let renew = |l: &mut ScreenLearn| l.learner = Some(learn::Learning::new(action_info(name)));
        if self
            .screen_learn
            .as_ref()
            .is_some_and(|l| l.ignore == Some(key))
        {
            if let Some(l) = self.screen_learn.as_mut() {
                renew(l);
            }
            return;
        }
        let knob = self
            .live_profiles()
            .iter()
            .find(|p| p.name == profile)
            .is_none_or(|p| p.is_knob(key));
        if knob != action_info(name).knob {
            if let Some(l) = self.screen_learn.as_mut() {
                l.refused = true;
                renew(l);
            }
            return;
        }
        // every control waits for Enter next to the old one (57); another control moved
        // before the answer takes its place, so the learner starts over
        if let Some(l) = self.screen_learn.as_mut() {
            l.pending = Some((profile, key));
            l.clearing = false;
            l.ignore = None;
            l.refused = false;
            renew(l);
        }
    }

    /// What the label of `name` says in the learn mode (57): its control, and for the
    /// picked element with a control waiting `old → new` (`none` when there is none).
    fn learn_shown(&self, profiles: &[Profile], name: &str) -> Option<String> {
        let now = self.learn_label(profiles, name);
        let l = self.screen_learn.as_ref()?;
        let picked = l.picked.as_ref().is_some_and(|(n, _)| *n == name);
        let new = match (&l.pending, l.clearing) {
            _ if !picked => return now,
            (Some((profile, key)), _) => control_name(profiles, profile, *key),
            (None, true) => "none".into(),
            // the picked element says what drives it, "none" too (57)
            (None, false) => return now.or_else(|| Some("none".into())),
        };
        Some(format!("{} → {new}", now.as_deref().unwrap_or("none")))
    }

    /// `Enter` in the learn mode: the waiting control drives the picked element, or the
    /// asked clear is done; `false` when nothing waited.
    fn learn_confirm(&mut self) -> bool {
        let group = self.learn_group();
        let Some(l) = self.screen_learn.as_mut() else {
            return false;
        };
        let Some((name, label)) = l.picked.clone() else {
            return false;
        };
        if let Some((profile, key)) = l.pending.take() {
            self.learn_save(profile, key);
            return true;
        }
        if std::mem::take(&mut l.clearing) {
            for profile in learn::clear(&mut self.learned, group, name) {
                self.sync_learned(&profile, group);
            }
            self.save_midi();
            self.flash = Some((format!("cleared: {label}"), Instant::now()));
            return true;
        }
        false
    }

    /// A key while the learn mode is open; every key stays in the mode (52): it picks the
    /// element it drives outside the mode, `I` `K` walk the parameters, `Enter` saves,
    /// `Esc` drops, `Backspace` asks to clear, the learn key (`F10`, 57) leaves.
    fn learn_key(&mut self, keymap: &Keymap, k: &event::KeyEvent) -> bool {
        use ratatui::crossterm::event::{KeyCode, KeyEventKind};
        if k.kind == KeyEventKind::Release {
            return true;
        }
        let waiting = self
            .screen_learn
            .as_ref()
            .is_some_and(|l| l.pending.is_some() || l.clearing);
        // the key of `learn.toggle` in the keymap leaves the mode
        if keymap.resolve_in(self.screen, false, k) == Some(Action::LearnToggle) {
            self.screen_learn = None;
            return true;
        }
        match k.code {
            KeyCode::Esc if waiting => {
                if let Some(l) = self.screen_learn.as_mut() {
                    (l.pending, l.clearing) = (None, false);
                }
                return true;
            }
            KeyCode::Esc => {
                self.screen_learn = None;
                return true;
            }
            KeyCode::Enter if waiting => {
                self.learn_confirm();
                return true;
            }
            KeyCode::Backspace => {
                let bound = self.learn_picked().is_some_and(|(name, _)| {
                    self.learn_label(&self.live_profiles(), name).is_some()
                });
                if let Some(l) = self.screen_learn.as_mut().filter(|_| bound) {
                    (l.pending, l.clearing) = (None, true);
                }
                return true;
            }
            _ => {}
        }
        if waiting {
            return true;
        }
        // the parameters have no key of their own: the edit layer's I K walk them
        match keymap.resolve_in(self.screen, true, k) {
            Some(Action::ParamNext) => {
                self.learn_walk(1);
                return true;
            }
            Some(Action::ParamPrev) => {
                self.learn_walk(-1);
                return true;
            }
            _ => {}
        }
        match keymap.resolve_in(self.screen, false, k) {
            Some(Action::PadPress { pad, .. }) => {
                let key = PAD_KEYS[pad].to_ascii_uppercase();
                self.learn_pick(
                    action_info(&format!("pad.{}", pad + 1)).name,
                    format!("pad {key}"),
                );
            }
            Some(action) => {
                // the action of the learn list this key drives, in this screen's group or
                // the global one
                let groups = [
                    self.learn_group(),
                    self.screen_group(),
                    input::Group::Global,
                ];
                let found = input::ACTIONS.iter().find(|i| {
                    !i.knob && groups.contains(&i.group) && Action::parse(i.name) == Some(action)
                });
                if let Some(info) = found {
                    self.learn_pick(info.name, info.label.into());
                }
            }
            None => {}
        }
        true
    }

    /// The control that drives `name` in the layer in sight, short: `k5`, `cc 28`,
    /// `note 36`; `None` when none does.
    fn learn_label(&self, profiles: &[Profile], name: &str) -> Option<String> {
        let scope = self.learn_group().scope();
        profiles
            .iter()
            .find_map(|p| p.assignment(scope, name))
            .map(|a| a.split(" (").next().unwrap_or_default().to_string())
    }

    /// The element a waiting control drives now on this screen, which would lose it:
    /// (action, label).
    fn learn_taken(&self) -> Option<(&'static str, String)> {
        let l = self.screen_learn.as_ref()?;
        let (_, key) = l.pending.as_ref()?;
        let (picked, _) = l.picked.clone()?;
        self.taken_by(*key, picked)
    }

    /// The element other than `picked` that control `key` drives now on this screen.
    fn taken_by(&self, key: input::midi::Key, picked: &str) -> Option<(&'static str, String)> {
        self.taken_in(self.learn_group(), key, picked)
    }

    /// The action other than `picked` that control `key` drives now in `group`.
    fn taken_in(
        &self,
        group: input::Group,
        key: input::midi::Key,
        picked: &str,
    ) -> Option<(&'static str, String)> {
        let scope = group.scope();
        let profiles = self.live_profiles();
        // "cc 28" or "k5 (cc 28)": no "(learned)" mark since 59
        let (learned, named) = (key.to_string(), format!("({key})"));
        input::ACTIONS
            .iter()
            .filter(|i| i.name != picked)
            .find(|i| {
                profiles.iter().any(|p| {
                    p.assignment(scope, i.name)
                        .is_some_and(|a| a == learned || a.ends_with(&named))
                })
            })
            .map(|i| (i.name, i.label.to_string()))
    }

    /// The grid cells in the learn mode: a bound pad carries its control, the picked one
    /// stands out, the one a waiting control would leave is marked (52).
    fn learn_grid(&self, cells: &mut [tui::PadView]) {
        if self.screen_learn.is_none() {
            return;
        }
        let profiles = self.live_profiles();
        let picked = self.learn_picked().map(|(n, _)| n);
        let taken = self.learn_taken().map(|(n, _)| n);
        for (i, cell) in cells.iter_mut().enumerate().take(PADS_PER_BANK) {
            let name = format!("pad.{}", i + 1);
            cell.note = self.learn_shown(&profiles, &name).map(|c| format!("◀ {c}"));
            cell.in_hand = picked == Some(action_info(&name).name);
            cell.marked = taken == Some(action_info(&name).name);
        }
    }

    /// The panel in the learn mode: the bound parameters carry their control, the picked
    /// one is lit, the one a waiting control would leave says so.
    fn learn_panel(&self, v: &mut tui::Pads) {
        if self.screen_learn.is_none() || self.screen != Screen::Pads {
            return;
        }
        let profiles = self.live_profiles();
        let picked = self.learn_picked().map(|(n, _)| n);
        let taken = self.learn_taken().map(|(n, _)| n);
        for &(name, label) in self.learn_params() {
            let Some(p) = pads::ORDER_ALL
                .into_iter()
                .find(|&p| pads::name_of(p) == label)
            else {
                continue;
            };
            let control = self.learn_shown(&profiles, name);
            if picked == Some(name) {
                v.lit = Some(tui_param(p));
            }
            match (control, taken == Some(name)) {
                (Some(c), true) => v.learn.push((tui_param(p), format!("{c}  ✗ leaves"))),
                (Some(c), false) => v.learn.push((tui_param(p), c)),
                (None, _) => {}
            }
        }
    }

    /// The three lines of the learn mode in words (52): the top line of the screen, then
    /// what to do next in the help band.
    fn learn_lines(&self) -> Option<Vec<String>> {
        let l = self.screen_learn.as_ref()?;
        let profiles = self.live_profiles();
        let picked = l.picked.clone();
        // where the binding goes (57): the layer and the page of the edit
        let place = match l.page {
            Some(page) => format!("{} · {page}", l.group.title()),
            None => l.group.title().to_string(),
        };
        // "do, then F10": say where the pick came from and how to pick another (57)
        let from = if l.once {
            " (the last thing you did)"
        } else {
            ""
        };
        let other = " · I K pick a parameter, a pad or a command key picks another";
        let lines = match (&picked, &l.pending, l.clearing) {
            (Some((name, label)), Some((profile, key)), _) => {
                let shown = self.learn_shown(&profiles, name).unwrap_or_default();
                let first = format!(" LEARN · {label} · {place} · {shown} · Enter bind · Esc keep");
                let second = match self.learn_taken() {
                    Some((_, other)) => format!(
                        " ✗ {key} is taken on this layer: \"{other}\" is marked and loses it"
                    ),
                    None => match self.bound_elsewhere(profile, *key) {
                        Some(note) => format!(" {note} · that layer keeps it"),
                        None => " another control takes its place".to_string(),
                    },
                };
                vec![first, second]
            }
            (Some((name, label)), None, true) => {
                let shown = self.learn_shown(&profiles, name).unwrap_or_default();
                vec![
                    format!(" LEARN · {label} · {place} · {shown} · Enter clear · Esc keep"),
                    " Backspace removes the control of the picked element".into(),
                ]
            }
            (Some((name, label)), None, false) if l.refused && action_info(name).knob => vec![
                format!(" LEARN · \"{label}\"{from} needs a knob or a fader, not a button"),
                format!(
                    " it has a value to turn, a button only presses · turn a knob{other} · F10 done"
                ),
            ],
            (Some((_, label)), None, false) if l.refused => vec![
                format!(" LEARN · \"{label}\"{from} needs a button or a pad, not a knob"),
                format!(
                    " it acts once per press, a knob would repeat it on every tick · press a button{other} · F10 done"
                ),
            ],
            (Some((name, label)), None, false) => {
                let now = match self.learn_label(&profiles, name) {
                    Some(c) => format!("now {c}"),
                    None => "now none".into(),
                };
                vec![
                    format!(" LEARN · picked: {label}{from} · {place} · {now} · move a control on the controller"),
                    " a pad or a command key picks another · I K the parameters · Backspace clear · F10 done".into(),
                ]
            }
            (None, ..) => vec![
                format!(
                    " LEARN · {place} · press what to bind: a pad, a command key, PgUp PgDn; I K the parameters"
                ),
                " then move a control on the controller · F10 done".into(),
            ],
        };
        Some(lines)
    }

    /// Where else on this screen a control is bound, outside the layer of the binding (57):
    /// `k5 on PADS: volume`.
    fn bound_elsewhere(&self, profile: &str, key: input::midi::Key) -> Option<String> {
        let groups = [
            self.screen_group(),
            self.layer_group(),
            input::Group::Global,
        ];
        self.bound_in(profile, key, self.learn_group(), &groups)
            .into_iter()
            .next()
    }

    /// Each action control `key` keeps in `groups` other than `here`:
    /// `b9 on GLOBAL: bank next`.
    fn bound_in(
        &self,
        profile: &str,
        key: input::midi::Key,
        here: input::Group,
        groups: &[input::Group],
    ) -> Vec<String> {
        let named = control_name(&self.live_profiles(), profile, key);
        groups
            .iter()
            .filter(|g| **g != here)
            .filter_map(|&g| {
                let map = learn::group_map(&self.learned, profile, g);
                let action = map.get(&key)?;
                let label = input::ACTIONS
                    .iter()
                    .find(|i| i.name == *action)
                    .map_or(action.as_str(), |i| i.label);
                Some(format!("{named} on {}: {label}", g.title()))
            })
            .collect()
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
            self.flash = Some((e, Instant::now()));
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
        self.flash = Some((
            result.map_or_else(|e| e, |_| format!("knobs: {}", next.name())),
            Instant::now(),
        ));
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

    fn midi_event(&mut self, ev: midi::Event) {
        let cuts = self.chop.as_ref().map(|c| c.saved());
        // a knob on the pads screen is a step of the undo (61); a turn joins one step
        // a button runs its action through `apply`, which keeps the step itself
        let before = (self.screen == Screen::Pads && matches!(ev, midi::Event::Param { .. }))
            .then(|| self.before());
        self.midi_event_inner(ev);
        if let Some(b) = before {
            let what = match self.lit_now() {
                Some(Lit::Pad(p)) => self.pads.name(p).to_string(),
                _ => "change".into(),
            };
            self.note_step(b, what);
        }
        self.cuts_changed(cuts);
        self.follow_loop();
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

    fn midi_event_inner(&mut self, ev: midi::Event) {
        let now = Instant::now();
        // the learn mode takes the controls to bind them; nothing acts (47)
        if self.screen_learn.is_some()
            && matches!(
                ev,
                midi::Event::PadHit { .. }
                    | midi::Event::PadRelease { .. }
                    | midi::Event::Action(_)
                    | midi::Event::Param { .. }
            )
        {
            return;
        }
        match ev {
            midi::Event::Connected { port, profile } => {
                self.flash = Some((format!("MIDI connected: {port} ({profile} profile)"), now));
                self.seed_device(&port, &profile, false);
                self.midi_devices.push((port, profile));
            }
            midi::Event::Disconnected { port } => {
                self.flash = Some((format!("MIDI disconnected: {port}"), now));
                self.midi_devices.retain(|(p, _)| *p != port);
            }
            midi::Event::Message {
                port,
                profile,
                msg,
                meaning,
            } => {
                // a run takes every new control for the next pad or bank; a control
                // already taken in the run waits for another (28)
                if let Some(r) = self.midi_run.as_mut()
                    && !r.taken.contains(&msg.key())
                    && let Some((profile, key)) = r.learner.feed(&profile, msg, now)
                {
                    self.midi_run_hit(profile, key);
                }
                if let Some(l) = self.learning.as_mut()
                    && let Some((profile, key)) = l.feed(&profile, msg, now)
                {
                    self.learned(profile, key);
                }
                // a button pressed for a parameter: the knob learner never hears a press, so
                // the refusal is said here (57)
                if let Some(l) = self.screen_learn.as_mut()
                    && matches!(msg, input::midi::Msg::NoteOn { .. })
                    && l.picked.as_ref().is_some_and(|(n, _)| action_info(n).knob)
                {
                    l.refused = true;
                    l.pending = None;
                }
                if let Some(l) = self.screen_learn.as_mut()
                    && let Some((profile, key)) =
                        l.learner.as_mut().and_then(|r| r.feed(&profile, msg, now))
                {
                    self.learn_bind(profile, key);
                }
                if self.input_log.len() == INPUT_LOG {
                    self.input_log.remove(0);
                }
                self.input_log
                    .push(format!("{port}   {msg}   -> {meaning}"));
            }
            midi::Event::PadHit { .. } if self.ask.is_some() => {}
            midi::Event::PadHit { abs } if self.screen == Screen::Chop => {
                self.play_slice(abs % PADS_PER_BANK)
            }
            midi::Event::PadHit { abs } if self.browser.in_hand.is_some() => self.put_in_hand(abs),
            midi::Event::PadHit { abs } if self.pad_in_hand.is_some() => self.swap_with(abs),
            midi::Event::PadHit { abs } => {
                // the note already went to the engine from the midi reader: a pad with
                // a sound ends the splash (73)
                if self.pad_sounds(abs) {
                    self.splash = false;
                }
                self.meters.hit(Instant::now());
                self.pads.hit(abs)
            }
            midi::Event::PadRelease { abs } => self.pads.release(abs),
            midi::Event::Action(action) => {
                // quitting from a controller is not offered
                self.apply(action);
            }
            midi::Event::Param {
                port,
                control,
                param: Param::Slot(k),
                value,
                takeover,
            } => {
                if let Some(param) = self.slot_param(k) {
                    self.midi_event(midi::Event::Param {
                        port,
                        control,
                        param,
                        value,
                        takeover,
                    });
                }
            }
            // the FX pages (56): the pad's drive and sends follow the scope, a slot is one
            // for all pads
            midi::Event::Param {
                port,
                control,
                param: Param::PadEffect(i),
                value,
                takeover,
            } => {
                let Some(&p) = EFFECT_PARAMS.get(i as usize) else {
                    return;
                };
                if self.screen != Screen::Pads {
                    return;
                }
                let who = if p.slot().is_some() {
                    0xFFFF
                } else {
                    self.pads.selected_abs() as u64
                };
                let target = (5u64 << 40) | ((i as u64) << 16) | who;
                let current = self.pads.fx_pos(p);
                let value = if takeover {
                    self.takeover.apply(&port, &control, target, value, current)
                } else {
                    Ok(value)
                };
                match value {
                    Ok(v) => {
                        self.pickup = None;
                        self.lit = Some((Lit::Pad(p), now));
                        let cmds = self.pads.set_fx_pos(p, v);
                        if !cmds.is_empty() {
                            for cmd in cmds {
                                self.send(cmd);
                            }
                            self.dirty();
                        }
                    }
                    Err(m) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        let name = self.pads.name(p);
                        self.pickup = Some((format!("turn {control} {way} to {name}"), now));
                    }
                }
            }
            // a switch, not a value: no pickup; a slice has one page (54)
            midi::Event::Param {
                param: Param::Page,
                value,
                ..
            } => {
                if self.edit && self.screen == Screen::Pads {
                    self.pads.set_page_pos(value);
                    self.lit = Some((Lit::Pad(self.pads.param), now));
                }
            }
            midi::Event::Param {
                port,
                control,
                param: param @ (Param::SliceStart | Param::SliceEnd),
                value,
                takeover,
            } => {
                let end = param == Param::SliceEnd;
                let Some(c) = self.chop.as_ref().filter(|_| self.screen == Screen::Chop) else {
                    return;
                };
                // each edge of each slice is its own takeover target, so selecting another
                // slice makes the knob catch up instead of jumping the cut
                let target = (1 << 32) | ((c.selected as u64) << 1) | end as u64;
                let current = c.edge_pos(end);
                let value = if takeover {
                    self.takeover.apply(&port, &control, target, value, current)
                } else {
                    Ok(value)
                };
                match value {
                    Ok(v) => {
                        if let Some(c) = self.chop.as_mut() {
                            c.set_edge(end, v);
                        }
                        self.lit = Some((Lit::Bound(end), now));
                        let changed = self.sync_links();
                        self.say_changed(changed, false);
                    }
                    Err(m) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        self.flash = Some((format!("turn {control} {way} to the cut"), now));
                    }
                }
            }
            midi::Event::Param {
                port,
                control,
                param: param @ (Param::Zoom | Param::Cursor | Param::RegionStart | Param::RegionEnd),
                value,
                takeover,
            } => {
                let Some(c) = self.chop.as_ref().filter(|_| self.screen == Screen::Chop) else {
                    return;
                };
                // every chop knob of every file is its own takeover target, apart from the
                // pad numbers and the slice bounds: a knob switched from another parameter or
                // another file catches up instead of jumping the view or the region
                let (kind, current) = match param {
                    Param::Zoom => (1, c.zoom_pos()),
                    Param::Cursor => (2, c.cursor_pos()),
                    Param::RegionStart => (3, c.region_edge_pos(false)),
                    Param::RegionEnd => (4, c.region_edge_pos(true)),
                    _ => return,
                };
                let file = {
                    use std::hash::{Hash, Hasher};
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    c.file.hash(&mut h);
                    h.finish() & 0xFFFF_FFFF
                };
                let target = (3 << 60) | (file << 4) | kind;
                let value = if takeover {
                    self.takeover.apply(&port, &control, target, value, current)
                } else {
                    Ok(value)
                };
                match value {
                    Ok(v) => {
                        if let Some(c) = self.chop.as_mut() {
                            match param {
                                Param::Zoom => c.set_zoom(v),
                                Param::Cursor => c.set_cursor_pos(v),
                                Param::RegionStart => c.set_region_edge(false, v),
                                _ => c.set_region_edge(true, v),
                            }
                        }
                        let changed = self.sync_links();
                        self.say_changed(changed, false);
                    }
                    Err(m) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        self.flash = Some((format!("turn {control} {way}"), now));
                    }
                }
            }
            midi::Event::Param {
                port,
                control,
                param: Param::Threshold,
                value,
                takeover,
            } => {
                let Some(c) = self.chop.as_ref().filter(|_| self.screen == Screen::Chop) else {
                    return;
                };
                // the threshold knob of one file catches up on its own, as every chop knob
                let file = {
                    use std::hash::{Hash, Hasher};
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    c.file.hash(&mut h);
                    h.finish() & 0xFFFF_FFFF
                };
                let target = (5 << 60) | (file << 4);
                let value = if takeover {
                    self.takeover
                        .apply(&port, &control, target, value, c.threshold_pos())
                } else {
                    Ok(value)
                };
                match value {
                    Ok(v) => {
                        let mut said = None;
                        if let Some(c) = self.chop.as_mut()
                            && c.set_threshold(v)
                        {
                            said = Some((c.threshold_db, c.markers.len()));
                        }
                        let changed = self.sync_links();
                        if let Some((db, n)) = said {
                            let cuts = if n == 1 { "cut" } else { "cuts" };
                            let mut text = format!("{n} {cuts} at {db:.0} dB");
                            if changed > 0 {
                                text.push_str(&format!("; {changed} pads changed; Ctrl+Z returns"));
                            }
                            self.flash = Some((text, now));
                        }
                    }
                    Err(m) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        self.flash = Some((format!("turn {control} {way}"), now));
                    }
                }
            }
            midi::Event::Param {
                port,
                control,
                param: param @ (Param::PadSource | Param::PadStart | Param::PadEnd),
                value,
                takeover,
            } => {
                let abs = self.pads.selected_abs();
                let Some((start, end, len)) = self.pad_bounds(abs) else {
                    return;
                };
                let list = self.source_list(abs);
                let last = list.len().saturating_sub(1).max(1) as f32;
                let (kind, current) = match param {
                    Param::PadSource => {
                        let at = list.iter().position(|&s| s == self.pads.source(abs));
                        (1, at.unwrap_or(0) as f32 / last)
                    }
                    Param::PadStart => (2, start as f32 / len as f32),
                    _ => (3, end as f32 / len as f32),
                };
                // apart from every other knob target, per pad and per parameter
                let target = (4 << 60) | ((abs as u64) << 4) | kind;
                let value = if takeover {
                    self.takeover.apply(&port, &control, target, value, current)
                } else {
                    Ok(value)
                };
                match (value, param) {
                    (Ok(v), Param::PadSource) => {
                        let i = (v.clamp(0.0, 1.0) * last).round() as usize;
                        if let Some(&s) = list.get(i) {
                            self.set_source(abs, s);
                        }
                    }
                    (Ok(v), _) => {
                        let to = (v.clamp(0.0, 1.0) * len as f32).round() as usize;
                        self.move_bound(abs, param == Param::PadEnd, to);
                    }
                    (Err(m), _) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        self.pickup = Some((format!("turn {control} {way}"), now));
                    }
                }
            }
            midi::Event::Param {
                port,
                control,
                param,
                value,
                takeover,
            } => {
                // a knob position 0..1 spans the range of the parameter
                let span = (pads::MAX_DB - pads::MIN_DB) as f32;
                let (current, target, shown) = match param {
                    Param::PadVolume => {
                        let abs = self.pads.selected_abs();
                        (
                            self.pads.volume_pos(),
                            abs as u64,
                            format!("{:+} dB", self.pads.volume_db[abs]),
                        )
                    }
                    Param::PadPitch => {
                        let abs = self.pads.selected_abs();
                        (
                            self.pads.pitch_pos(),
                            abs as u64,
                            format!("{:+} st", self.pads.pitch[abs]),
                        )
                    }
                    Param::MasterVolume => {
                        let pos = (self.master_db - pads::MIN_DB) as f32 / span;
                        (pos, u64::MAX, format!("{:+} dB", self.master_db))
                    }
                    Param::PadAttack | Param::PadDecay | Param::PadRelease => {
                        let p = pad_env(param);
                        let abs = self.pads.selected_abs();
                        let ms = match p {
                            pads::Param::Attack => self.pads.attack_ms[abs],
                            pads::Param::Decay => self.pads.decay_ms[abs],
                            _ => self.pads.release_ms[abs],
                        };
                        // one knob slot turns volume on TONE and attack on ENV: another
                        // target per parameter, so the page change makes the knob pick up
                        let tag = (p as u64 + 1) << 40;
                        (self.pads.env_pos(p), tag | abs as u64, format!("{ms} ms"))
                    }
                    Param::PadFilter | Param::PadCutoff | Param::PadReso => {
                        let p = pad_filter(param);
                        let abs = self.pads.selected_abs();
                        // its own target per parameter, as the envelope knobs (44)
                        let tag = (p as u64 + 1) << 40;
                        let pads = &self.pads;
                        let shown = match p {
                            pads::Param::Filter => pads::kind_name(pads.filter[abs]).to_string(),
                            pads::Param::Cutoff => pads::cutoff_text(pads.cutoff_hz[abs]),
                            _ => format!("{} %", pads.reso[abs]),
                        };
                        (self.pads.filter_pos(p), tag | abs as u64, shown)
                    }
                    Param::SliceStart
                    | Param::SliceEnd
                    | Param::Zoom
                    | Param::Cursor
                    | Param::RegionStart
                    | Param::RegionEnd
                    | Param::Threshold
                    | Param::PadSource
                    | Param::PadStart
                    | Param::PadEnd
                    | Param::Slot(_)
                    | Param::Page
                    | Param::PadEffect(_) => return,
                };
                let value = if takeover {
                    self.takeover.apply(&port, &control, target, value, current)
                } else {
                    Ok(value)
                };
                match (value, param) {
                    (Ok(v), Param::PadVolume) => {
                        self.pickup = None;
                        self.lit = Some((Lit::Pad(pads::Param::Volume), now));
                        let cmds = self.pads.set_volume_pos(v);
                        if !cmds.is_empty() {
                            for cmd in cmds {
                                self.send(cmd);
                            }
                            self.dirty();
                        }
                    }
                    (Ok(v), Param::PadPitch) => {
                        self.pickup = None;
                        self.lit = Some((Lit::Pad(pads::Param::Pitch), now));
                        let cmds = self.pads.set_pitch_pos(v);
                        if !cmds.is_empty() {
                            for cmd in cmds {
                                self.send(cmd);
                            }
                            self.dirty();
                        }
                    }
                    (Ok(v), Param::PadFilter | Param::PadCutoff | Param::PadReso) => {
                        let p = pad_filter(param);
                        self.pickup = None;
                        self.lit = Some((Lit::Pad(p), now));
                        let cmds = self.pads.set_filter_pos(p, v);
                        if !cmds.is_empty() {
                            for cmd in cmds {
                                self.send(cmd);
                            }
                            self.dirty();
                        }
                    }
                    (Ok(v), Param::PadAttack | Param::PadDecay | Param::PadRelease) => {
                        let p = pad_env(param);
                        self.pickup = None;
                        self.lit = Some((Lit::Pad(p), now));
                        let cmds = self.pads.set_env_pos(p, v);
                        if !cmds.is_empty() {
                            for cmd in cmds {
                                self.send(cmd);
                            }
                            self.dirty();
                        }
                    }
                    (
                        Ok(_),
                        Param::SliceStart
                        | Param::SliceEnd
                        | Param::Zoom
                        | Param::Cursor
                        | Param::RegionStart
                        | Param::RegionEnd
                        | Param::Threshold
                        | Param::PadSource
                        | Param::PadStart
                        | Param::PadEnd
                        | Param::Slot(_)
                        | Param::Page
                        | Param::PadEffect(_),
                    ) => {}
                    (Ok(v), Param::MasterVolume) => {
                        let db = (pads::MIN_DB as f32 + v.clamp(0.0, 1.0) * span).round() as i8;
                        if db != self.master_db {
                            self.set_master(db);
                            self.dirty();
                        }
                    }
                    (Err(m), _) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        let hint = format!("turn {control} {way} to {shown}");
                        match param {
                            Param::PadVolume
                            | Param::PadPitch
                            | Param::PadAttack
                            | Param::PadDecay
                            | Param::PadRelease
                            | Param::PadFilter
                            | Param::PadCutoff
                            | Param::PadReso => self.pickup = Some((hint, now)),
                            // the master hint is needed on every screen, not only on the pads one
                            Param::MasterVolume => {
                                self.flash = Some((format!("Master: {hint}"), now))
                            }
                            Param::SliceStart
                            | Param::SliceEnd
                            | Param::Zoom
                            | Param::Cursor
                            | Param::RegionStart
                            | Param::RegionEnd
                            | Param::Threshold
                            | Param::PadSource
                            | Param::PadStart
                            | Param::PadEnd
                            | Param::Slot(_)
                            | Param::Page
                            | Param::PadEffect(_) => {}
                        }
                    }
                }
            }
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
            Action::EditValue { .. } | Action::PadParam(_) => self.pads.param_name().into(),
            Action::PadModeNext => "mode".into(),
            Action::PadClear => "clear".into(),
            Action::PadEdit if matches!(self.ask, Some(Ask::ResetPads { .. })) => "reset".into(),
            Action::PadEdit if matches!(self.ask, Some(Ask::ClearPads { .. })) => "clear".into(),
            _ if self.pad_in_hand.is_some() => "swap".into(),
            _ if self.browser.in_hand.is_some() => "file".into(),
            _ => "change".into(),
        }
    }

    /// Keeps what an action changed of the pads as a step (61); a change of the same thing
    /// on the same pads within a second joins the step before, so a knob turn is one step.
    fn note_step(&mut self, b: Before, what: String) {
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
        let now = Instant::now();
        let touched: Vec<usize> = pads.iter().map(|(a, _)| *a).collect();
        if let Some(last) = self.history.last_mut()
            && last.what == what
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
            at: now,
        });
    }

    /// `Ctrl+Z` on the pads (61): the last step comes back, every pad of it sounds as before.
    fn undo_pads(&mut self) {
        let Some(step) = self.history.pop() else {
            self.flash = Some(("nothing to undo".into(), Instant::now()));
            return;
        };
        for (abs, snap) in &step.pads {
            // a file still loading for the pad would land over the undone one
            self.load_gen[*abs] += 1;
            for cmd in self.pads.put_back(*abs, snap) {
                self.send(cmd);
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
        self.flash = Some((format!("undo: {} on {on}", step.what), Instant::now()));
    }

    fn set_pad_region(&mut self, abs: usize, region: Region) {
        self.pads.region[abs] = region;
        self.send(Command::SetPadRegion { pad: abs, region });
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
        self.flash = Some((format!("source: {}", source_word(source)), Instant::now()));
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
        self.flash = Some((
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
        ));
    }

    /// The value keys on the SOURCE page: the source steps through its list, a bound moves
    /// by the fine step (1 ms, a tap) or the coarse one (10 ms, held), as every value (27).
    fn source_value(&mut self, up: bool, fine: bool) {
        let abs = self.pads.selected_abs();
        let Some((start, stop, _)) = self.pad_bounds(abs) else {
            self.flash = Some(("empty pad: nothing to play from".into(), Instant::now()));
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
            self.flash = Some((
                result.map_or_else(|e| e, |_| format!("wave: {what} from the next start")),
                Instant::now(),
            ));
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
                Some(Err(e)) => self.flash = Some((e, Instant::now())),
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

    fn notice(&self) -> Option<String> {
        if let Some((source, at)) = &self.recording {
            return Some(format!(
                "Recording {} {}.{:01} s… {}",
                if *source == audio::SOURCE_MIC {
                    "the microphone"
                } else {
                    "the output"
                },
                at.elapsed().as_secs(),
                at.elapsed().subsec_millis() / 100,
                if *source == audio::SOURCE_MIC {
                    "release Ctrl+R to stop"
                } else {
                    "Ctrl+T to stop"
                }
            ));
        }
        if self.audio.as_ref().is_some_and(|a| a.stopped.load(Relaxed)) {
            return Some("Audio stopped: the sound server or device went away.".into());
        }
        if let Some((msg, at)) = &self.flash
            && at.elapsed() < FLASH
        {
            return Some(msg.clone());
        }
        if let Some(abs) = (0..PADS).find(|&p| self.pads.loading[p].is_some()) {
            let name = self.pads.file[abs]
                .as_deref()
                .map(file_name)
                .unwrap_or_default();
            return Some(format!(
                "Loading {name} {}%",
                self.pads.loading[abs].unwrap_or(0)
            ));
        }
        if let Some((what, want, got)) = self.missing() {
            return Some(format!(
                "{what} '{want}' not found, using '{}'",
                got.unwrap_or("none")
            ));
        }
        self.startup.first().cloned()
    }

    /// The first key event of the session clears the startup notices: the owner has read
    /// them (22).
    fn dismiss_startup(&mut self) {
        self.startup.clear();
    }

    /// The splash view while no sound went (73); `None` draws the wave in the strip.
    fn splash_view(&self) -> Option<tui::Splash> {
        self.splash.then(|| tui::Splash {
            version: env!("CARGO_PKG_VERSION"),
            loading: self.splash_line(),
        })
    }

    /// A pad that can sound now: a loaded sample, not still loading, not failed (73).
    fn pad_sounds(&self, pad: usize) -> bool {
        self.pads.file[pad].is_some() && !self.pads.missing[pad] && self.pads.loading[pad].is_none()
    }

    /// The loading line of the splash (73): what loads now, `ready` when all is loaded,
    /// or a file that did not load.
    fn splash_line(&self) -> String {
        if let Some(pad) = (0..PADS).find(|&p| self.pads.missing[p] && self.pads.file[p].is_some())
        {
            return format!(
                "cannot load {}",
                file_name(self.pads.file[pad].as_deref().unwrap_or(Path::new("")))
            );
        }
        let loading = self.pads.loading.iter().filter(|l| l.is_some()).count();
        if loading > 0 {
            format!(
                "loading the session, {loading} {}",
                if loading == 1 { "sample" } else { "samples" }
            )
        } else {
            "ready".into()
        }
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

/// Copies the old `arasaka-404` folders under these bases (12) and gives the startup
/// notice that names where the settings came from; a failed copy only speaks in the log.
fn settings_moved(bases: &[PathBuf]) -> Option<String> {
    match config::move_old_folders(bases) {
        Ok(old) => old.map(|o| format!("settings moved from {}", tilde(&o.display().to_string()))),
        Err(e) => {
            log::error!("{e}");
            None
        }
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
    // before anything reads the config, the keymap or the session (12)
    let moved = settings_moved(&[
        config::xdg_base("XDG_CONFIG_HOME", ".config"),
        config::xdg_base("XDG_DATA_HOME", ".local/share"),
    ]);
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
        audio,
        cfg,
        user_path: loaded.user_path,
        path_picker: None,
        project_picker: None,
        paths,
        routed: Routed::default(),
        screen: Screen::Pads,
        settings_row: 0,
        help_open: false,
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
        session_path: session::path(),
        project: start_project,
        project_cfg: loaded.config.project,
        session_ok: session_notice.is_none(),
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
        flash: None,
        startup: Vec::new(),
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
    // a broken project file speaks right away, as open_project does (11)
    if let Some(e) = project_notice {
        app.flash = Some((e, Instant::now()));
    }
    let (keymap, keymap_notice) =
        load_keymap(&config::xdg_dir("XDG_CONFIG_HOME", ".config").join("keymap.toml"));
    let (profiles, profile_errors) = midi::load_profiles(DEVICES);
    if let Some(msg) = audio_msg {
        app.flash = Some((msg, Instant::now()));
    }
    if let Some(e) = profile_errors.first()
        && app.flash.is_none()
    {
        app.flash = Some((
            format!("{e}; that device uses the default profile"),
            Instant::now(),
        ));
    }
    let shared = Arc::new(midi::Shared::default());
    app.profiles = profiles.clone();
    if let Some(notice) = app.drop_dead_bindings() {
        app.startup.push(notice);
    }
    let maps = midi::parse_learned(&app.learned);
    for p in app.profiles.iter_mut() {
        midi::apply_learned(p, &maps);
        p.set_seeded(app.seeded.contains(&p.name));
    }
    app.midi = Some(midi::start(
        profiles,
        maps,
        app.seeded.clone(),
        app.audio.as_mut().and_then(|a| a.midi_commands.take()),
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
    let mut shown = tui::Shown::default();
    // the startup notices live on the app: the first key event of the session clears them
    for notice in [moved, loaded.error, session_notice, keymap_notice] {
        app.startup.extend(notice);
    }
    if !kitty {
        app.startup.push(NO_RELEASES_NOTICE.into());
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
        let events: Vec<_> = app
            .midi
            .as_ref()
            .map(|m| m.events.try_iter().collect())
            .unwrap_or_default();
        for ev in events {
            app.midi_event(ev);
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
        // a knob turn changes the volume many times a second: save once things settle
        if app
            .session_dirty
            .is_some_and(|t| t.elapsed() > SESSION_SAVE)
        {
            app.save_session();
        }
        // the cap ends the recording in the audio thread: the status names it, the file goes in hand
        if app.recording.is_some()
            && app
                .audio
                .as_ref()
                .is_some_and(|a| a.recorder.stopped_by_cap())
        {
            app.record_stop();
        }
        shared.bank.store(app.pads.bank, Relaxed);
        let screen_code = match app.screen {
            Screen::Pads => 0,
            Screen::Settings => 1,
            Screen::Files => 2,
            Screen::Chop => 3,
        };
        shared.screen.store(screen_code, Relaxed);
        shared.edit.store(app.edit, Relaxed);
        shared.learn.store(app.screen_learn.is_some(), Relaxed);
        shared.hand.store(
            app.browser.in_hand.is_some()
                || app.pad_in_hand.is_some()
                || app.screen == Screen::Chop,
            Relaxed,
        );
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
        let fresh = app.window_title();
        if fresh != title {
            title = fresh;
            let _ = execute!(std::io::stdout(), SetTitle(title.as_str()));
        }

        let notice = app.notice();
        let (buffer_size, sample_rate) = app.audio.as_ref().map_or((0, 0), |a| a.stats.format());
        let status = tui::Status {
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
            notice: notice.as_deref(),
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
                    let mut view =
                        chop_view(c, app.plan.as_ref(), app.playhead(), app.rate, &app.pads);
                    // the keyboard of slices draws the slices' own waves
                    if view.slice_keys.is_none() {
                        bank_waves(&mut view.pads, &mut app.pads, app.rate);
                    }
                    view.which = app.which_chop();
                    app.learn_grid(&mut view.pads.pads);
                    tui::Screen::Chop(Box::new(app.mark_chop(view, mods.get())))
                }
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
                            hidden: app.ui.hide_pads,
                            ..pads_view(&mut app.pads, None, app.pad_in_hand, out, None, app.rate)
                        };
                        app.learn_grid(&mut v.pads);
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
                    if app.screen == Screen::Pads {
                        view.edit_key = keys(&keymap, Screen::Pads, &[Action::PadEdit]);
                    }
                    tui::Screen::Pads(app.mark_pads(view))
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
        let groups = help_groups(&keymap, app.screen, &app.help_state());
        let groups = match (&app.project_picker, mods.get()) {
            // the learn mode speaks in its own lines, the columns step aside (52)
            _ if app.screen_learn.is_some() => Vec::new(),
            // the picker takes the keys: the line says its own (35)
            (Some(p), _) => picker_help(p.naming.is_some()),
            (None, Some(m)) => modifier_groups(&keymap, app.screen, m, app.ui.hide_pads),
            (None, None) if kitty => without_modifiers(groups),
            (None, None) => as_combos(groups),
        };
        let full = app.help_open.then(|| full_help(&keymap, app.screen, kitty));
        let learn_lines = app.learn_lines();
        let help = tui::Help {
            groups: &groups,
            hint: hint_for(&app),
            full: full.as_deref(),
            question: app.ask.as_ref().map(|a| a.lines()),
            learn: learn_lines.as_deref(),
        };
        let mut want = Vec::new();
        let drawn = if graphics {
            term.draw(|f| tui::draw_gfx(f, &status, &screen, &help, &mut want))
        } else {
            term.draw(|f| tui::draw(f, &status, &screen, &help))
        };
        if let Err(e) = drawn {
            break Err(e);
        }
        // the pictures go out after the frame: transmit what changed, place where it sits
        if graphics {
            let cell_px = match terminal::window_size() {
                Ok(s) if s.columns > 0 && s.rows > 0 => (s.width / s.columns, s.height / s.rows),
                _ => (0, 0),
            };
            let cmds = shown.update(&want, cell_px);
            if !cmds.is_empty() {
                let mut out = std::io::stdout();
                let sent = out
                    .write_all(cmds.concat().as_bytes())
                    .and_then(|_| out.flush());
                if let Err(e) = sent {
                    break Err(e);
                }
            }
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
        // any key event at all means the startup notices were read
        app.dismiss_startup();
        // the project picker is open: its keys go first, raw, like the filter line (11)
        if app.project_picker.is_some() && app.project_picker_key(&k) {
            continue;
        }
        // the learn mode takes every key: it walks, clears, leaves (47)
        if app.screen_learn.is_some() && app.learn_key(&keymap, &k) {
            continue;
        }
        let typing = app.browser.editing && app.screen == Screen::Files;
        mods.update(&k, typing);
        if typing && app.filter_key(&k) {
            continue;
        }
        // a press of a key still down is its repeat; without releases every press is new
        let k = if kitty { keys_down.repeat(&k) } else { k };
        let Some(action) = keymap.resolve_in(app.screen, app.edit_keys(), &k) else {
            continue;
        };
        // without the kitty protocol there are no releases, so a held pad would never reset
        let action = if kitty {
            held.filter(action)
        } else {
            Some(action)
        };
        if let Some(action) = action
            && app.apply(action)
        {
            break Ok(());
        }
    };

    if app.session_dirty.is_some() {
        app.save_session();
    }
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
        Screen::Pads if app.pads.file.iter().all(Option::is_none) => {
            Some("Pads are empty: F6 opens files, Enter takes one, a pad hit puts it there.")
        }
        Screen::Pads => Some("Hit 1-4 Q-R A-F Z-V to play pads; Shift for full velocity."),
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

/// The shipped keymap with the owner's file `user` over it (64); the working folder does
/// not matter. A broken file is named with its line and the shipped keys stay.
fn load_keymap(user: &Path) -> (Keymap, Option<String>) {
    let (mut keymap, _) =
        Keymap::from_toml(DEFAULT_KEYMAP).expect("the shipped keymap parses (tested in input)");
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
    (keymap, notice.or(warning))
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
                glyphs: pad_glyphs(&changed),
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
        fx,
        bank: p.bank,
        pads,
        selected: p.selected,
        pickup,
        param: tui_param(p.param),
        scope: p.scope_name(),
        bank_fill: bank_fill(p),
        bank_marks: bank_marks(p),
        mark_plate: mark_plate(p),
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
    let sound = p.sound[abs].as_deref()?;
    let (wave, env, dur_ms) = pads::piece(
        sound,
        p.region[abs],
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

/// The plate on the title line of the pads screen (76): marking is on, or pads stay marked.
fn mark_plate(p: &pads::Pads) -> Option<String> {
    let n = p.marked.len();
    if p.marking {
        Some(format!("MARK · hit pads · {n} marked"))
    } else {
        (n > 0).then(|| format!("{n} MARKED · Esc drops"))
    }
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

fn chop_view<'a>(
    c: &'a chop::Chop,
    plan: Option<&chop::Plan>,
    playhead: Option<usize>,
    rate: u32,
    p: &pads::Pads,
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
        None if !laid => {
            let page = slice_page(p.bank, slices.len());
            let pads = (0..PADS_PER_BANK)
                .map(|i| {
                    let n = page * PADS_PER_BANK + i;
                    let slice = slices.get(n);
                    tui::PadView {
                        glyphs: String::new(),
                        changed: Vec::new(),
                        dim: slice.is_none(),
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
                    mark_plate: None,
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
                        glyphs: String::new(),
                        changed: Vec::new(),
                        dim: !ours(abs) && !whole(abs),
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
                    first,
                    // the keys of 21 and 36: the banks turn, P lays the slices out
                    "Ctrl+1…8: banks".into(),
                    "P: put them on pads".into(),
                ],
                Some(marks),
                tui::Pads {
                    bank: p.bank,
                    pads,
                    // the pad of the selected slice, when it is in this bank
                    selected: (0..PADS_PER_BANK)
                        .find(|&i| {
                            let abs = p.bank * PADS_PER_BANK + i;
                            ours(abs) && p.slice[abs] == Some(c.selected as u16 + 1)
                        })
                        .unwrap_or(usize::MAX),
                    pickup: None,
                    param: tui::PadParam::Volume,
                    scope: String::new(),
                    bank_fill: bank_fill(p),
                    bank_marks: bank_marks(p),
                    mark_plate: mark_plate(p),
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
                    mark_plate: mark_plate(p),
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
    if st.edit && matches!(screen, Screen::Pads | Screen::Chop) {
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
        Screen::Pads => vec![
            tui::HelpGroup {
                name: "PAD",
                // the pad panel shows the mode and the picked parameter: the help line names only the keys
                items: vec![
                    item(&[Action::PadEdit], "edit".into()),
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
                    item(&[Action::ParamNext], "in order / by hand".into()),
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
                        item(
                            &[
                                Action::ChopZoom { zoom_in: true },
                                Action::ChopZoom { zoom_in: false },
                            ],
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
                        item(&[Action::ChopEqualParts], "equal parts".into()),
                        item(&[Action::ChopHits], "by hits".into()),
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
    if m == input::Modifier::Shift {
        return vec![tui::HelpGroup {
            name: "SHIFT",
            items: vec![("1-4 Q-R A-F Z-V".into(), "pads at full velocity".into())],
        }];
    }
    let (prefix, name) = match m {
        input::Modifier::Alt => ("Alt+", "ALT"),
        _ => ("Ctrl+", "CTRL"),
    };
    let mut items: Vec<(String, String)> = Vec::new();
    for (k, a) in keymap.bindings(screen) {
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
fn ctrl_text(a: Action) -> String {
    match a {
        Action::RecordMic(_) => "mic: hold".into(),
        Action::RecordOutputToggle => "output".into(),
        Action::PadTake => "take pad".into(),
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
        Action::PadSelect(d) => format!("select pad {}", format!("{d:?}").to_lowercase()),
        Action::PadModeNext => "pad mode: one-shot / gate".into(),
        Action::PadEdit => "pads: edit the selected pad; Esc closes".into(),
        Action::PadParamNext => "pads: the next parameter".into(),
        Action::PadParam(s) if s < 0 => "pad: the picked parameter down a step".into(),
        Action::PadParam(_) => "pad: the picked parameter up a step".into(),
        Action::PadMarkBank => "pads: mark bank; again unmarks it".into(),
        Action::PadMarkToggle => "pads: marking on / off; a pad hit marks or unmarks it".into(),
        Action::PadClear => {
            "pads: take the sound off the selected pad; in the edit, reset its settings".into()
        }
        Action::ScreenPads => "pads screen".into(),
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
        Action::Undo => "chop: undo".into(),
        Action::BrowserUp => "file up".into(),
        Action::BrowserDown => "file down".into(),
        Action::BrowserEnter => "open folder".into(),
        Action::BrowserParent => "parent folder".into(),
        Action::BrowserHold => "take file in hand / open folder".into(),
        Action::BrowserLoad => "the file loads onto the selected pad; a folder opens".into(),
        Action::BrowserPreviewToggle => "play the selected file: on / off".into(),
        Action::BrowserFilter => "filter the list by name".into(),
        Action::BrowserFavorite => "the selected folder: favorite on / off".into(),
        Action::HelpToggle => "this help".into(),
        Action::LearnToggle => "bind controller knobs and buttons to what the screen shows".into(),
        Action::ProjectPick => "open the project picker".into(),
        Action::PadsToggle => "hide / show the pads".into(),
        Action::Back => "back".into(),
        Action::Quit => "quit".into(),
        Action::SessionSave => "save the session now".into(),
        Action::Click => "test click".into(),
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
        | Action::PadEdit
        | Action::PadParamNext
        | Action::PadParam(_)
        | Action::PadMarkBank
        | Action::PadMarkToggle
        | Action::PadClear
        | Action::PadTake
        | Action::LearnToggle => 0,
        Action::BankNext | Action::BankPrev | Action::BankSelect(_) => 1,
        Action::BrowserUp
        | Action::BrowserDown
        | Action::BrowserEnter
        | Action::BrowserParent
        | Action::BrowserHold
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
        | Action::SliceBound { .. }
        | Action::ParamNext
        | Action::ParamPrev
        | Action::PageNext
        | Action::PagePrev
        | Action::EditValue { .. }
        | Action::ChopCommit
        | Action::ChopEdit
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
        Action::MasterVolume(_) | Action::Click => 5,
        Action::RecordMic(_) | Action::RecordOutputToggle => 5,
        Action::ScreenPads
        | Action::ScreenSettings
        | Action::ScreenFiles
        | Action::ScreenChop
        | Action::HelpToggle
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
            ("⇥", "one-shot: plays to the end"),
            ("⇢", "gate: plays while held"),
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
    use input::Action;

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
        // an empty pad draws nothing
        p.sound[0] = None;
        assert!(big_view(&p, 48_000).is_none());
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
        assert_eq!(groups.len(), 7, "one row per group");
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
    fn test_app() -> super::App {
        use super::*;
        App {
            audio: None,
            cfg: AudioConfig::default(),
            user_path: None,
            path_picker: None,
            project_picker: None,
            paths: config::PathsConfig::default(),
            routed: Routed::default(),
            screen: Screen::Pads,
            settings_row: 0,
            help_open: false,
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
            session_path: "session.toml".into(),
            project: None,
            project_cfg: config::ProjectConfig::default(),
            session_ok: false,
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
            flash: None,
            startup: Vec::new(),
            splash: false,
            recording: None,
            #[cfg(test)]
            sent: Vec::new(),
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
        app.midi_event(super::midi::Event::PadHit { abs: empty });
        assert!(
            app.splash,
            "a controller note on an empty pad makes no sound"
        );
    }

    /// 73: a note of the controller on a pad with a sound ends the splash too.
    #[test]
    fn a_controller_note_ends_the_splash() {
        let mut app = laid_out_break();
        app.splash = true;
        assert!(app.pads.file[4].is_some(), "pad Q has a sample");
        app.midi_event(super::midi::Event::PadHit { abs: 4 });
        assert!(!app.splash, "the controller note ended the splash");
    }

    /// The config and data bases under one temp folder, with the old folders filled as
    /// the owner had them (12).
    fn old_folders(name: &str) -> (std::path::PathBuf, [std::path::PathBuf; 2]) {
        let root = tmp_dir(name);
        let bases = [root.join("config"), root.join("share")];
        let old_cfg = bases[0].join("arasaka-404");
        std::fs::create_dir_all(old_cfg.join("devices")).unwrap();
        std::fs::write(old_cfg.join("config.toml"), "[audio]\noutput = \"Buds\"\n").unwrap();
        std::fs::write(old_cfg.join("keymap.toml"), "[screens.pads]\n").unwrap();
        std::fs::write(old_cfg.join("devices/my.toml"), "name = \"My\"\n").unwrap();
        let old_data = bases[1].join("arasaka-404");
        std::fs::create_dir_all(&old_data).unwrap();
        std::fs::write(old_data.join("session.toml"), "browser_dir = \"/s\"\n").unwrap();
        (root, bases)
    }

    /// Scenario "The settings move at the first start" (12): the config and the session
    /// are copied into the `ptsr` folders, the old folders stay, the status line names
    /// the move once.
    #[test]
    fn the_settings_move_at_the_first_start() {
        let (root, bases) = old_folders("move-first");
        let notice = super::settings_moved(&bases);
        assert_eq!(
            notice.as_deref(),
            Some(
                format!(
                    "settings moved from {}",
                    bases[0].join("arasaka-404").display()
                )
                .as_str()
            )
        );
        for (base, file) in [
            (&bases[0], "config.toml"),
            (&bases[0], "keymap.toml"),
            (&bases[0], "devices/my.toml"),
            (&bases[1], "session.toml"),
        ] {
            let (old, new) = (
                base.join("arasaka-404").join(file),
                base.join("ptsr").join(file),
            );
            assert_eq!(
                std::fs::read(&new).unwrap(),
                std::fs::read(&old).unwrap(),
                "{file} is copied"
            );
        }
        let loaded = super::config::load_from(&bases[0].join("ptsr/config.toml"));
        assert_eq!(loaded.config.audio.output.as_deref(), Some("Buds"));
        assert!(!bases[0].join("ptsr.part").exists(), "no side folder stays");
        // the second start finds the new folders: nothing moves, nothing is said
        assert_eq!(super::settings_moved(&bases), None);
        std::fs::remove_dir_all(&root).ok();
    }

    /// Scenario "The new folder wins" (12): with `ptsr` there, the old folder is not copied.
    #[test]
    fn the_new_folder_wins() {
        let (root, bases) = old_folders("move-wins");
        for base in &bases {
            std::fs::create_dir_all(base.join("ptsr")).unwrap();
        }
        std::fs::write(bases[0].join("ptsr/config.toml"), "[audio]\n").unwrap();
        // the data base first: its `ptsr` folder is empty and still wins
        let data_first = [bases[1].clone(), bases[0].clone()];
        assert_eq!(super::settings_moved(&data_first), None, "nothing is said");
        assert!(
            !bases[0].join("ptsr/keymap.toml").exists(),
            "nothing is copied"
        );
        assert!(
            !bases[1].join("ptsr/session.toml").exists(),
            "nothing is copied"
        );
        assert_eq!(
            std::fs::read_to_string(bases[0].join("ptsr/config.toml")).unwrap(),
            "[audio]\n"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    /// Scenario "A first start without old folders" (12): no folder is made, nothing is said.
    #[test]
    fn a_first_start_without_old_folders() {
        let root = tmp_dir("move-none");
        let bases = [root.join("config"), root.join("share")];
        assert_eq!(super::settings_moved(&bases), None);
        assert!(!root.exists(), "no folder is made");
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
        let loaded = super::load_keymap(&path);
        std::fs::remove_dir_all(&dir).ok();
        loaded
    }

    fn pads_ctrl(k: &input::Keymap, c: char) -> Option<Action> {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
        k.resolve(Screen::Pads, &KeyEvent::new(KeyCode::Char(c), M::CONTROL))
    }

    /// 71 input-devices "The shipped bindings all act": the keymap and the device profiles
    /// name only actions the program has; Ctrl+K and knobs k6–k10 of the chop screen are free.
    #[test]
    fn the_shipped_bindings_all_act() {
        let (k, warnings) = input::Keymap::from_toml(super::DEFAULT_KEYMAP).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(pads_ctrl(&k, 'k'), None, "Ctrl+K is free");
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../config/devices");
        let (profiles, errors) = super::midi::load_profiles(dir);
        // the SMC-PAD file is a template to fill after purchase: it does not parse yet
        let errors: Vec<_> = errors.iter().filter(|e| !e.contains("smc-pad")).collect();
        assert!(errors.is_empty(), "{errors:?}");
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

    fn flash(app: &super::App) -> &str {
        app.flash.as_ref().map_or("", |(s, _)| s.as_str())
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
        app.session_ok = true;
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
        app.session_ok = true;
        app.dirty();
        app.save_session();
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
        app.session_ok = true;
        app.pads.file[4] = Some(pj.join("samples/a.wav"));
        assert!(app.save_session());
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
        app.session_ok = true;
        app.pads.file[4] = Some(pj.join("samples/a.wav"));
        assert!(app.save_session());
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
        app.session_ok = true;
        app.pads.file[4] = Some(outside.join("amen.wav"));
        assert!(app.save_session());
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
        assert!(app.session_ok);
        // a pad outside the project folder keeps its file through the deletion
        let outside_dir = tmp_dir("proj-gone-outside");
        std::fs::create_dir_all(&outside_dir).unwrap();
        let outside = outside_dir.join("keep.wav");
        std::fs::write(&outside, b"").unwrap();
        app.pads.file[1] = Some(outside.clone());

        std::fs::remove_dir_all(&pj).unwrap();
        app.dirty();
        assert!(!app.save_session(), "the save fails without the folder");
        assert!(!pj.exists(), "the save does not resurrect the folder");
        assert!(app.pads.missing[0], "the pad inside the folder is missing");
        assert!(!app.pads.missing[1], "the pad outside keeps its file");
        assert_eq!(app.window_title(), "ptsr", "the title drops the name");
        assert!(!app.session_ok);
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
            ("Ctrl+Y", "by hits"),
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
        app.create_project("beat2".into());
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
            assert_eq!(keys, ["F1", "F3", "F6", "F12"], "{screen:?}");
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
        app.flash = Some(("MIDI connected".into(), std::time::Instant::now()));
        assert!(
            super::hint_for(&app).is_some(),
            "the hint stays under the message"
        );
        app.flash = None;
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
        for want in [("R", "mic: hold"), ("T", "output"), ("1…8", "bank")] {
            assert!(
                items.contains(&(want.0.into(), want.1.into())),
                "{want:?} missing in {items:?}"
            );
        }
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
        assert_eq!(names, ["PAD", "CHANGE", "BANK", "REC", ""]);
        assert_eq!(tui::help_dropped(&groups, 140), 0);
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
            assert_eq!(names[n - 3..], ["BANK", "REC", ""], "{screen:?}: {names:?}");
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
            ("Y", "equal parts"),
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
                ("J L".to_string(), "right bound".to_string()),
                // 49: the loop of the play; 50: play as the pad
                ("O".to_string(), "loop".to_string()),
                ("P".to_string(), "as pad".to_string())
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
        assert!(items.contains(&("J L".into(), "bank".into())), "{items:?}");
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
        app.midi_event(super::midi::Event::PadHit { abs: 3 });
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

    /// 51 "Ctrl+Y cuts at the marks" and "No marks in even steps".
    #[test]
    fn ctrl_y_cuts_at_the_marks() {
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
        chop_key(&mut app, KeyCode::Char('y'), M::CONTROL); // −6 dB
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
        let status = |app: &super::App| app.flash.as_ref().map(|f| f.0.clone());
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
        app.flash = None;
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
        chop_key(&mut app, KeyCode::Char('y'), M::CONTROL);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.markers, [24_000, 48_000], "the two loudest hits at −6 dB");
        assert_eq!(c.threshold_db, -6.0);
        assert_eq!(
            app.flash.as_ref().map(|f| f.0.as_str()),
            Some("by hits at -6 dB: 3 slices")
        );
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
        chop_key(&mut app, KeyCode::Char('y'), M::CONTROL);
        assert_eq!(
            app.flash.as_ref().map(|f| f.0.as_str()),
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
        let ctrl_y = |app: &mut super::App| chop_key(app, KeyCode::Char('y'), M::CONTROL);
        let state = |app: &super::App| {
            let c = app.chop.as_ref().unwrap();
            (c.markers.clone(), app.flash.as_ref().map(|f| f.0.clone()))
        };
        ctrl_y(&mut app);
        assert_eq!(state(&app).0, [24_000, 48_000]);
        ctrl_y(&mut app);
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
        ctrl_y(&mut app);
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
        ctrl_y(&mut app);
        assert_eq!(
            state(&app).0,
            [24_000, 48_000, 72_000, 96_000],
            "−18 dB again, all four"
        );
        ctrl_y(&mut app);
        assert_eq!(
            state(&app).0,
            [24_000, 48_000, 72_000, 96_000],
            "−24 dB holds all"
        );
        ctrl_y(&mut app);
        assert_eq!(
            state(&app).0,
            [24_000, 48_000, 72_000, 96_000],
            "−30 dB holds all"
        );
        ctrl_y(&mut app);
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
        let ctrl_y = |app: &mut super::App| chop_key(app, KeyCode::Char('y'), M::CONTROL);
        ctrl_y(&mut app);
        ctrl_y(&mut app);
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
        ctrl_y(&mut app);
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
        chop_key(&mut app, KeyCode::Char('y'), KeyModifiers::CONTROL);
        assert_eq!(app.chop.as_ref().unwrap().markers, [24_000, 48_000]);
        // the knob sits far from the −6 dB (0.15 of the way): it waits, naming the way
        xsession(&mut app, knob(K11, 127));
        assert_eq!(
            app.chop.as_ref().unwrap().markers,
            [24_000, 48_000],
            "no jump"
        );
        assert_eq!(
            app.flash.as_ref().map(|f| f.0.as_str()),
            Some("turn k11 down ↓")
        );
        // down through the current value: the knob picks up at 0 dB, only the loudest keeps a cut
        xsession(&mut app, knob(K11, 0));
        assert_eq!(app.chop.as_ref().unwrap().markers, [24_000]);
        assert_eq!(
            app.flash.as_ref().map(|f| f.0.as_str()),
            Some("1 cut at 0 dB")
        );
        // up to the quiet end: every hit the detector found gets its cut
        xsession(&mut app, knob(K11, 127));
        let (markers, db) = (
            app.chop.as_ref().unwrap().markers.clone(),
            app.chop.as_ref().unwrap().threshold_db,
        );
        assert_eq!(markers, [24_000, 48_000, 72_000, 96_000]);
        assert_eq!(db, -40.0);
        assert_eq!(
            app.flash.as_ref().map(|f| f.0.as_str()),
            Some("4 cuts at -40 dB")
        );
        // mid: the −15.6 dB hit is out, the −9.5 dB one stays
        xsession(&mut app, knob(K11, 32));
        assert_eq!(app.chop.as_ref().unwrap().markers, [24_000, 48_000, 72_000]);
        assert_eq!(
            app.flash.as_ref().map(|f| f.0.as_str()),
            Some("3 cuts at -10 dB")
        );
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
        assert_eq!(
            app.flash.as_ref().map(|f| f.0.as_str()),
            Some("marker snapped to hit")
        );
        // the hits mode off: the cut stays where the sound is
        app.apply(input::Action::ChopSnapToggle);
        app.fake_playhead = Some(48_000 + 12_000);
        app.flash = None;
        app.apply(input::Action::ChopCut);
        let c = app.chop.as_ref().unwrap();
        assert_eq!(c.markers, [48_000, 60_000], "where pressed, no snap");
        assert_eq!(app.flash, None, "no snap status with the hits mode off");
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
            chop_key(&mut app, KeyCode::Char('y'), M::CONTROL); // −6, −12, −18 dB
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
        app.flash = None;
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
        for _ in 0..21 {
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
                (Param::Fx1Kind, "FX1*"),
                (Param::Fx1A, "FX1*"),
                (Param::Fx1B, "FX1*"),
                (Param::Fx2Kind, "FX2*"),
                (Param::Fx2A, "FX2*"),
                (Param::Fx2B, "FX2*"),
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
    /// SOURCE picks the drive on FX; K past FX2* comes back to the volume.
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
        while app.pads.param != Param::Fx2B {
            app.apply(input::Action::ParamNext);
        }
        app.apply(input::Action::ParamNext);
        assert_eq!(app.pads.param, Param::Volume);
        assert_eq!(
            super::pads::page_names(),
            ["TONE", "ENV", "FILTER", "SOURCE", "FX", "FX1*", "FX2*"]
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
                // M is the marker key: a pad mark here, a cut marker there
                if k.chars().all(|c| c.is_ascii_alphabetic())
                    && !"JLIKM".contains(&k)
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
        assert!(items.iter().any(|(k, _)| k == "Ctrl+R"), "{items:?}");
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
        for (glyph, word) in [("✱", "more settings"), ("⇥", "one-shot"), ("⇢", "gate")] {
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
        app.startup = vec![super::NO_RELEASES_NOTICE.into()];
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
        assert!(app.startup.is_empty(), "the whole list is cleared at once");
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
                .any(|r| r.0.contains('J') && r.1 == "previous / next bank"),
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
        assert!(rows.contains(&("Ctrl+R".into(), "record the microphone: hold".into())));
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
        assert_eq!(pads.get("note 70").map(String::as_str), Some("pad.edit"));
        assert_eq!(xs_live(&app, press(B10)), Some(Action::PadEdit));
        // the MIDI tab names it as the profile does, no "(learned)"
        let p = app.live_profiles().remove(0);
        assert_eq!(
            p.assignment(input::midi::Scope::Screen(Screen::Pads), "pad.edit")
                .as_deref(),
            Some("b10 (note 70)")
        );
        // the edit layer went into both edit groups
        assert_eq!(
            app.learned["X-Session Pro"]["chop_edit"]
                .get("note 46")
                .map(String::as_str),
            Some("edit.page_next")
        );
    }

    /// 59 input-devices: "Learned wins".
    #[test]
    fn learned_wins() {
        let app = xsession_connected(&[(input::Group::Pads, B3, "bank.next")]);
        assert_eq!(xs_live(&app, press(B3)), Some(Action::BankNext));
        assert!(
            !app.learned["X-Session Pro"]["pads"]
                .values()
                .any(|a| a == "pad.mode_next"),
            "the profile's pad mode on b3 is not added"
        );
        assert_eq!(
            xs_live(&app, press(B10)),
            Some(Action::PadEdit),
            "the rest is"
        );
    }

    /// 59 midi-learn: "Clear a profile role" and "Cleared stays cleared".
    #[test]
    fn clear_a_profile_role() {
        let mut app = xsession_connected(&[]);
        assert_eq!(xs_live(&app, press(B3)), Some(Action::PadModeNext));
        super::learn::clear(&mut app.learned, input::Group::Pads, "pad.mode_next");
        app.sync_learned("X-Session Pro", input::Group::Pads);
        assert_eq!(xs_live(&app, press(B3)), None, "no profile role under it");
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
        assert_eq!(xs_live(&app, press(B10)), None);
    }

    /// 59 midi-learn: "Reset to profile".
    #[test]
    fn reset_to_profile() {
        let mut app = xsession_connected(&[]);
        super::learn::clear(&mut app.learned, input::Group::Pads, "pad.edit");
        super::learn::assign(
            &mut app.learned,
            "X-Session Pro",
            input::Group::Pads,
            input::midi::Key::Note(B3),
            "bank.next",
        );
        app.sync_learned("X-Session Pro", input::Group::Pads);
        assert_eq!(xs_live(&app, press(B10)), None);
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
        assert_eq!(xs_live(&app, press(B10)), Some(Action::PadEdit));
        assert_eq!(xs_live(&app, press(B3)), Some(Action::PadModeNext));
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

    const B3: u8 = 45;
    const B4: u8 = 46;
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
        xsession(&mut app, press(B10));
        assert!(app.edit, "b10 opens the edit");
        xsession(&mut app, press(B4));
        assert_eq!(app.pads.page().0, "ENV", "b4 turns the page (54)");
        // k1 is the first slot of ENV: the attack, picked up at 0 ms
        xsession(&mut app, knob(K1, 0));
        xsession(&mut app, knob(K1, 64));
        assert!(app.pads.attack_ms[abs] > 0);
        assert_eq!(
            app.lit_now(),
            Some(super::Lit::Pad(P::Attack)),
            "the screen says which"
        );
        xsession(&mut app, press(B10));
        assert!(!app.edit, "b10 closes the edit");
    }

    /// 54 input-devices: "Pages from the controller".
    #[test]
    fn pages_from_the_controller() {
        use super::pads::Param as P;
        let mut app = test_app();
        let abs = app.pads.selected_abs();
        xsession(&mut app, press(B10));
        let mut pages = Vec::new();
        for b in [B4, B4, B3] {
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
        // seven pages (56), a seventh each: the middle is SOURCE, at once, no pickup
        page(&mut app, 0.5);
        assert_eq!(app.pads.page().0, "SOURCE");
        assert_eq!(app.pads.param, P::Source);
        // a turn inside the same part keeps the pick
        app.apply(input::Action::ParamNext);
        page(&mut app, 0.52);
        assert_eq!(app.pads.param, P::Start);
        page(&mut app, 1.0);
        assert_eq!(app.pads.page().0, "FX2*");
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
        xsession(&mut app, press(B10));
        xsession(&mut app, press(B4));
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
        xsession(&mut app, press(B10));
        xsession(&mut app, press(B4));
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
        xsession(&mut app, press(B10));
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
        chop_key(&mut app, KeyCode::Char('n'), M::CONTROL);
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
        chop_key(&mut app, KeyCode::Char('n'), M::CONTROL);
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
        chop_key(&mut app, KeyCode::Char('n'), M::CONTROL);
        let gains = |app: &super::App| {
            app.sent
                .iter()
                .filter(|c| matches!(c, Command::SetPadGain { .. }))
                .count()
        };
        let after_first = gains(&app);
        chop_key(&mut app, KeyCode::Char('n'), M::CONTROL);
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
        chop_key(&mut app, KeyCode::Char('n'), M::CONTROL);
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
        // the app side decides the gain: the quiet pad, Ctrl+N
        let mut app = test_app();
        quiet_pad(&mut app, 0.5);
        open_chop_from_pads(&mut app);
        chop_key(
            &mut app,
            ratatui::crossterm::event::KeyCode::Char('n'),
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

    /// 56 "A slot page": on FX2* knob 1 turns the delay into a reverb; knobs 2 and 3 turn
    /// its size and tone; the block says `FX 2 * shared`, the change line `all pads`; on
    /// TONE no FX 2 block.
    #[test]
    fn a_slot_page() {
        use engine::fx::FxKind;
        let mut app = test_app();
        app.apply(Action::PadEdit);
        while app.pads.page().0 != "FX2*" {
            app.apply(Action::PageNext);
        }
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
        let v = panel_now(&mut app);
        let (page, title, rows) = v.slot.clone().expect("the slot block on its page");
        assert_eq!((page, title.as_str()), ("FX2*", "FX 2 * shared"));
        assert_eq!(rows[1].1.trim(), "size");
        assert_eq!(v.scope, "all pads");
        // the slot keeps the delay's own values for when it comes back
        assert_eq!(app.pads.fx[1].time_ms, 250);
        while app.pads.page().0 != "TONE" {
            app.apply(Action::PageNext);
        }
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
        app.apply(Action::PageNext);
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
        for (k, word) in [("⇥", "one-shot"), ("⇢", "gate")] {
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
        app.flash.as_ref().map(|f| f.0.clone()).unwrap_or_default()
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
        app.apply(Action::PadEdit);
        while app.pads.page().0 != "FX2*" {
            app.apply(Action::PageNext);
        }
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

    /// 77 "A filter marks the pad", "A fresh pad has no mark", "The mark goes with the
    /// setting".
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
        let plate = |app: &mut super::App| panel_now(app).mark_plate;
        assert_eq!(plate(&mut app), None);
        app.apply(Action::PadMarkToggle);
        for pad in [0, 1, 2] {
            app.apply(Action::PadPress { pad, velocity: 0.8 });
            app.apply(Action::PadRelease { pad });
        }
        assert_eq!(
            plate(&mut app).as_deref(),
            Some("MARK · hit pads · 3 marked")
        );
        app.apply(Action::PadMarkToggle);
        assert_eq!(plate(&mut app).as_deref(), Some("3 MARKED · Esc drops"));
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
        app.session_ok = true;
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
        assert!(app.save_session());
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
        app.apply(Action::ChopCommit);
        assert!(
            matches!(app.plan, Some(super::chop::Plan::InOrder(Some(4)))),
            "in order from Q"
        );
        app.apply(Action::ChopCommit);
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
        assert!(app.flash.is_none() || flash(&app) != "assigning: Enter applies, Esc leaves");
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
        app.session_ok = true;
        app.screen = Screen::Chop;
        app.chop = Some(chop_with_parts(0));
        app.chop.as_mut().unwrap().set_cursor(30_000);
        assert!(app.session_dirty.is_none());
        chop_key(&mut app, KeyCode::Char('m'), M::NONE);
        assert!(app.session_dirty.is_some(), "the cut marks the session");
        app.save_session();
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
        app.midi_event(super::midi::Event::PadHit { abs: 4 });
        app.pads.hit_at[4] = Some(two_seconds_ago);
        assert!(lit(&mut app), "held from a controller");
        app.midi_event(super::midi::Event::PadRelease { abs: 4 });
        assert!(!lit(&mut app));
    }

    /// Scenario "L turns to the next bank" (pad-playback): L on bank 1 opens bank 2 with
    /// the same place selected.
    #[test]
    fn l_turns_to_the_next_bank() {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let mut app = test_app();
        app.pads.selected = 6;
        pads_key(&mut app, KeyCode::Char('l'), M::NONE);
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
        assert_eq!(xs_live(&app, press(B10)), Some(Action::PadEdit));
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
    /// switches the pad mode on the pads screen and turns the bank on the other screens.
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
        for screen in [Screen::Files, Screen::Settings] {
            assert_eq!(
                xs_on(&mut app, screen, B9),
                Some(Action::BankNext),
                "{screen:?}"
            );
        }
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
        // b3 is the pad mode from the profile
        assert_eq!(xs_on(&mut app, Screen::Pads, B3), Some(Action::PadModeNext));
        learn_row(&mut app, input::Group::Pads, "pad.mode_next");
        app.apply(Action::Back);
        app.apply(Action::SettingsClear);
        assert_eq!(xs_on(&mut app, Screen::Pads, B3), None, "no control left");
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

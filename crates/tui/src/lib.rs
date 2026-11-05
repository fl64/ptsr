//! Screens. They only draw a state snapshot and never touch the engine.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

mod kitty;
pub use kitty::{Shown, WaveImage, is_graphics_ack};

// Colors from docs/ui/STYLE.md
const BG: Color = Color::Rgb(0x07, 0x08, 0x0A);
const TEXT: Color = Color::Rgb(0xC9, 0xD1, 0xD9);
const DIM: Color = Color::Rgb(0x8A, 0x92, 0x9E);
const CYAN: Color = Color::Rgb(0x33, 0xE1, 0xFF);
const MAGENTA: Color = Color::Rgb(0xFF, 0x5F, 0xAE);
const PANEL: Color = Color::Rgb(0x11, 0x14, 0x1A);
const BORDER: Color = Color::Rgb(0x2A, 0x30, 0x3A);
const WHITE: Color = Color::Rgb(0xF2, 0xF4, 0xF7);
const BRIGHT: Color = Color::Rgb(0xE6, 0xEA, 0xF0);
/// The meters' green: the palette's bank-4 tone, a mark of level, not of state.
/// The bound in play: the one the keys move now.
const AMBER: Color = Color::Rgb(0xFF, 0xC8, 0x3D);
/// A marked pad (76): the frame and the dots of the marks in the bank row, and the fill
/// under the pad's name; brighter than the amber and olive banks, so a mark never reads
/// as a bank frame.
const MARK: Color = Color::Rgb(0xFF, 0xE0, 0x3A);
const MARK_BG: Color = Color::Rgb(0x3D, 0x33, 0x05);
const METER: Color = Color::Rgb(0x86, 0xC4, 0x7E);
/// The meters' display floor, dB.
const METER_FLOOR: f32 = -60.0;
/// The meters: L, R and the overload light.
const METER_ROWS: u16 = 3;
/// How many banks the program has.
pub const BANKS: usize = 8;

/// One soft color per bank (docs/ui/STYLE.md): coral..orchid, 1..8. A mark of belonging,
/// softer than the cyan/magenta of state.
pub const BANK_COLORS: [Color; BANKS] = [
    Color::Rgb(0xD9, 0x88, 0x80), // 1 coral
    Color::Rgb(0xD9, 0xA0, 0x66), // 2 amber
    Color::Rgb(0xC9, 0xC4, 0x6E), // 3 olive
    Color::Rgb(0x86, 0xC4, 0x7E), // 4 green
    Color::Rgb(0x6F, 0xC2, 0xB4), // 5 teal
    Color::Rgb(0x7F, 0xA3, 0xE0), // 6 blue
    Color::Rgb(0xA8, 0x8B, 0xE0), // 7 violet
    Color::Rgb(0xD0, 0x8C, 0xC4), // 8 orchid
];

/// Session state, independent of temporary notices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveStatus {
    Clean,
    Unsaved,
    Failed,
}

impl SaveStatus {
    fn spans(self) -> Vec<Span<'static>> {
        match self {
            Self::Clean => Vec::new(),
            Self::Unsaved => vec![" Unsaved ·".fg(AMBER).bg(BG)],
            Self::Failed => vec![" Save failed ·".fg(MAGENTA).bg(BG).bold()],
        }
    }
}

/// Semantic meaning of an optional title badge, independent of its label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeKind {
    PadMode,
    On,
    Attention,
}

/// Existing mode with full and compact labels, in application priority order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Badge {
    pub kind: BadgeKind,
    pub normal: String,
    pub compact: String,
}

impl Badge {
    pub fn new(kind: BadgeKind, normal: impl Into<String>, compact: impl Into<String>) -> Self {
        Self {
            kind,
            normal: normal.into(),
            compact: compact.into(),
        }
    }
}

/// Shared title and status snapshot.
pub struct Status<'a> {
    pub badges: Vec<Badge>,
    pub save: SaveStatus,
    /// Audio backend (`jack`, `cpal`); `None` means no audio.
    pub backend: Option<&'a str>,
    /// A recording is running right now.
    pub recording: bool,
    pub buffer_size: u32,
    pub sample_rate: u32,
    /// Audio thread load peak, %.
    pub load_peak: f32,
    pub xruns: u64,
    pub master_db: i8,
    /// Output meter levels in dB: bar, peak hold, per channel; with the latched clip light.
    pub meters: ([f32; 2], [f32; 2], bool),
    /// Message for the owner: why there is no audio, terminal warning.
    pub notice: Option<&'a str>,
    /// The terminal has no kitty protocol: no key releases, gate pads play as one-shot.
    pub no_releases: bool,
    /// The open project's name; `None` when no project is open.
    pub project: Option<&'a str>,
}

/// One row of the MIDI learn list: a group with its actions folded in, or an action of
/// an open group (28).
pub enum MidiRow {
    Group {
        name: &'static str,
        /// "31 actions · 2 learned".
        counts: String,
    },
    Action {
        label: String,
        /// What drives it now, per device; `old → new` while a control waits (74).
        who: String,
        /// A waiting control leaves this action: the row is in the taken colour (74).
        taken: bool,
        /// What the waiting control keeps in the other groups, a grey line each (74).
        stays: Vec<String>,
    },
}

/// Audio settings screen (F12).
pub struct Settings<'a> {
    /// Selected row: 0 output, 1 input, 2 buffer, 3 wave; on the Paths tab 0 projects,
    /// 1 samples.
    pub row: usize,
    /// The waves are drawn as kitty pictures from the next start (04b); Braille otherwise.
    pub graphics: bool,
    /// Device the audio actually goes to.
    pub output: Option<&'a str>,
    /// Saved choice that is not among the devices now.
    pub output_missing: Option<&'a str>,
    pub input: Option<&'a str>,
    pub input_missing: Option<&'a str>,
    /// Input peak, 0..1.
    pub input_level: f32,
    /// The actual buffer size the server runs.
    pub buffer_size: u32,
    /// The saved choice: 0 = auto (follow the server), else a fixed size.
    pub buffer_choice: u32,
    /// Device choice is available (JACK); not in cpal mode.
    pub routing: bool,
    /// The config asks for JACK: no routing then means JACK is down, not a wrong backend.
    pub backend_jack: bool,
    /// 0 audio, 1 input (MIDI devices and the input debug log), 2 MIDI, 3 paths (11).
    pub tab: usize,
    /// Connected MIDI devices: "port (profile)".
    pub midi_devices: &'a [String],
    /// Latest input events, newest last.
    pub input_log: &'a [String],
    /// MIDI tab: the groups with their actions under the open ones.
    pub midi_rows: &'a [MidiRow],
    pub midi_row: usize,
    /// The ready line about waiting for a control: a single learn or a run (28).
    pub learning: Option<&'a str>,
    /// Paths tab (11): the two roots as shown, `~`-shortened.
    pub paths: Option<(&'a str, &'a str)>,
    /// Paths tab (11): the open folder picker replaces the tab rows while it is open.
    pub picker: Option<PathPickerView>,
}

/// The folder picker of the Paths tab (11).
pub struct PathPickerView {
    /// " PICK THE PROJECTS ROOT " or " PICK THE SAMPLES ROOT ".
    pub title: &'static str,
    /// The folders of the open folder, by name.
    pub entries: Vec<String>,
    pub selected: usize,
}

/// The project picker of Ctrl+P (11): while it is open it replaces the whole body of
/// the screen, whatever the screen is.
pub struct ProjectPickerView {
    /// The project folders of the projects root that hold a `project.toml`, by name.
    pub entries: Vec<String>,
    pub selected: usize,
    /// The name being typed for a new project; a letter opens the line, Esc closes it.
    pub naming: Option<String>,
}

pub struct PadView {
    pub naming: bool,
    /// `✱` when the pad has more settings than its mode line shows (77), or empty.
    pub glyphs: String,
    /// The parameters that differ from a fresh pad (55): bright in the panel, the rest dim.
    pub changed: Vec<PadParam>,
    /// Keyboard key of the pad.
    pub key: char,
    pub name: String,
    pub gate: bool,
    pub volume_db: f32,
    /// Pitch in semitones, shown in the panel.
    pub pitch: i8,
    /// The mute group (70): 0 is off, 1–8.
    pub group: u8,
    /// Envelope times in ms, shown in the panel.
    pub attack_ms: u16,
    pub decay_ms: u16,
    pub release_ms: u16,
    /// The pad was hit a moment ago or is held.
    pub flash: bool,
    /// Its file is gone or broken.
    pub missing: bool,
    /// Loading progress, 0..100.
    pub loading: Option<u8>,
    /// Taken with Ctrl+X: the next pad hit swaps it there.
    pub in_hand: bool,
    /// Marked for the change scope: framed in the grid.
    pub marked: bool,
    /// Second line in place of mode and volume (the chop screen shows slice lengths).
    pub note: Option<String>,
    /// What the info panel says about the selected pad in place of its name.
    pub detail: Option<String>,
    /// What the pad plays (`all`, `slice 3`, `pad`) and its start and end in seconds,
    /// for the SOURCE page of the panel (27).
    pub source: String,
    pub bounds: (f32, f32),
    /// The FILTER page as the panel says it (44): kind, cutoff, resonance.
    pub filter: [String; 3],
    /// The wave the pad plays with its envelope (29): peaks 0..1. Empty means no wave
    /// (a missing file), one zero a flat line (an empty pad).
    pub wave: Vec<f32>,
    /// Drawn dim: on the chop screen a pad that plays no slice of the open chop (36).
    pub dim: bool,
    /// A dashed frame: on the chop screen the pad that plays the whole file (46).
    pub dashed: bool,
}

/// Chop screen (F3).
pub struct Chop<'a> {
    pub file: &'a str,
    /// "LISTEN" or "ASSIGN · in order".
    pub mode: String,
    pub seconds: f32,
    /// Which chop of the file this is and how many it has (45); `None` with one.
    pub which: Option<(usize, usize)>,
    /// The counted hits on the screen in hits mode (51), as fractions of the view.
    pub hits: Vec<f32>,
    /// What was opened (the file or a pad's slice), one channel: the waveform is drawn from it.
    pub wave: &'a [f32],
    /// Range start, cuts, range end, 0..1 over `wave`; outside the range is trimmed.
    pub bounds: Vec<f32>,
    /// The selected slice: between `bounds[selected]` and `bounds[selected + 1]`.
    pub selected: usize,
    /// The owner's cursor 0..1: only the owner moves it, playback starts from it.
    pub cursor: Option<f32>,
    /// Where the sound is 0..1, while it plays; it never moves the cursor.
    pub playhead: Option<f32>,
    /// What play covers now, from and to 0..1: the green bar above the wave.
    pub play: Option<(f32, f32)>,
    /// "slice 3 · 0.548 s · 261 ms".
    pub info: String,
    /// Lines next to the grid: what the pads do now, the plan summary.
    pub side: Vec<String>,
    /// Per bank, (something lies there, the plan writes there / slices of this chop lie
    /// there).
    pub banks: Option<[(bool, bool); BANKS]>,
    /// Banks with a pad that plays this file whole: `◆` in the bank row (37).
    pub source_banks: [bool; BANKS],
    /// Before any slice lies on a pad the grid is a keyboard of slices (36): the first and
    /// the last slice it plays, named over the grid instead of the bank row.
    pub slice_keys: Option<(usize, usize)>,
    /// The low, mid and high level of the wave shown, in even blocks over it (38); empty
    /// draws the wave in plain tones.
    pub bands: &'a [[f32; 3]],
    /// The average of each band over the file: the color shows the band above its average.
    pub band_mean: [f32; 3],
    /// Listen mode: the slices; assign mode: the real pads of the bank with the plan.
    pub pads: Pads,
    /// The slice edit mode is open: the slice line names it and its page.
    pub edit: bool,
    /// The pads that play each slice, as bank and key (`2-E`), joined by commas (27).
    pub owners: Vec<String>,
    /// The bank of the first pad that plays each slice, `None` on no pad: the color of its
    /// mark (36).
    pub owner_bank: Vec<Option<usize>>,
    /// The bound of the selected slice the keys move now (`true` the end), drawn in the
    /// bound colour: a held modifier's, the picked one in the edit, the one just moved.
    pub hot: Option<bool>,
}

/// Files screen (F6).
pub struct Files<'a> {
    pub dir: &'a str,
    /// (name, is a folder, is a favorite shortcut).
    pub entries: Vec<(&'a str, bool, bool)>,
    pub selected: usize,
    /// File name in hand, waiting for a pad hit.
    pub in_hand: Option<&'a str>,
    pub error: Option<&'a str>,
    /// The current bank, shown left of the list: where a file can go and what is there.
    pub pads: Pads,
    pub filter: &'a str,
    /// The filter line is open and takes the keys.
    pub editing: bool,
    /// Duration and peak envelope of the selected file, once the preview loaded it.
    pub preview: Option<(f32, &'a [f32])>,
    /// Where the preview plays now, 0..1 of the file, while it sounds (33).
    pub playhead: Option<f32>,
}

/// The parameter the pads panel highlights: the one the value keys change now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadParam {
    Volume,
    Pitch,
    Group,
    Attack,
    Decay,
    Release,
    Source,
    Start,
    End,
    Filter,
    Cutoff,
    Reso,
    Drive,
    Send1,
    Send2,
    Fx1Kind,
    Fx1A,
    Fx1B,
    Fx2Kind,
    Fx2A,
    Fx2B,
}

/// The filter response for the strip: `levels` are linear gains at frequencies spread
/// evenly along the octaves from 20 Hz to 20 kHz; the caption names kind, cutoff, resonance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FilterCurve {
    pub caption: String,
    pub levels: Vec<f32>,
    /// The frequencies at the left and the right end of `levels`, in Hz.
    pub span: (f32, f32),
}

/// Top and bottom of the curve in dB: the top rises over a resonance hump.
const CURVE_TOP_DB: f32 = 12.0;
const CURVE_BOTTOM_DB: f32 = -36.0;

/// Pads screen (F2).
/// A row of the pad panel: the parameter, its label and its value (56).
pub type PanelRow = (PadParam, &'static str, String);

/// Existing placement result, not a new pad action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadCorner {
    Available,
    Replace,
    Slice(u16),
}

/// Read-only feedback for the sixteen cells of the visible bank.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PadFeedback {
    pub outline: bool,
    pub corners: [Option<PadCorner>; 16],
}

pub struct Pads {
    pub feedback: PadFeedback,
    /// Any real pad voice in the current ready project, across all banks; not hit flashes or FX tails.
    pub playing: bool,
    pub bank: usize,
    pub pads: Vec<PadView>,
    pub selected: usize,
    pub pickup: Option<String>,
    /// The picked parameter, highlighted in the panel.
    pub param: PadParam,
    /// What the parameter and the mode changes act on now, e.g. `pad Q`, `3 marked`.
    pub scope: String,
    pub bank_fill: [u16; BANKS],
    /// Per bank, its marked pads (76); a bank with marks shows them in the bank row.
    pub bank_marks: [u16; BANKS],
    /// Live envelope of the master output, oldest first (the newest value is last).
    pub out_wave: Vec<f32>,
    /// The splash at start (73): while it is `Some`, the strip shows it instead of the
    /// wave, until the first sound.
    pub splash: Option<Splash>,
    /// The piece of the selected pad with its envelope, or nothing on an empty pad.
    pub big: Option<BigView>,
    /// The filter curve while the FILTER page is picked (44); it takes the strip.
    pub curve: Option<FilterCurve>,
    /// The learn mode (47): the control of each panel parameter that can take one.
    pub learn: Vec<(PadParam, String)>,
    /// The page of the open pad edit mode (TONE, ENV); `None` while it is closed.
    pub edit: Option<&'static str>,
    /// Every page of the pad edit in `Tab` order: the frame title names them all (43).
    pub pages: Vec<&'static str>,
    /// The parameters of that page in knob order: knob 1 turns the first.
    pub knobs: Vec<PadParam>,
    /// The parameter a knob or a value key changed a moment ago: it is lit.
    pub lit: Option<PadParam>,
    /// The grid is hidden (32): one pad line stands in its place, the strip takes its rows.
    pub hidden: bool,
    /// The key that opens the pad edit, named on the closed panel (70); empty: no hint.
    pub edit_key: String,
    pub fx: Vec<PanelRow>,
    /// Kept for callers that still build the old shared-slot view; not drawn on pads.
    pub slot: Option<(&'static str, String, Vec<PanelRow>)>,
}

/// The big view of the selected pad's piece, both at the same resolution.
pub struct BigView {
    /// Shape of the piece: peaks per column, 0..1.
    pub wave: Vec<f32>,
    /// The pad's envelope over it: attack from the start, decay into the end, 0..1.
    pub env: Vec<f32>,
    /// The attack, the decay and the release as fractions of the piece's length.
    pub attack: f32,
    pub decay: f32,
    pub release: f32,
}

/// One project-wide effect. Rows are type, first parameter and second parameter.
/// Labels and values come from the app; this view does not interpret effect settings.
pub struct FxBlock {
    pub rows: [(&'static str, String); 3],
}

/// Shared effects screen (F4). Block and parameter indices start at zero.
pub struct Fx {
    pub blocks: [FxBlock; 2],
    pub selected: usize,
    pub param: usize,
    /// A takeover hint for the selected block, shown above the bottom help.
    pub pickup: Option<String>,
    /// Learned controls of the selected block: (parameter index, control label).
    pub learn: Vec<(usize, String)>,
    /// The parameter that changed a moment ago, in the selected block.
    pub lit: Option<usize>,
}

pub enum Screen<'a> {
    Pads(Pads),
    Fx(Fx),
    Settings(Settings<'a>),
    Files(Files<'a>),
    // boxed: the chop view is the largest by far, and a screen is built once a frame
    Chop(Box<Chop<'a>>),
    /// The project picker (11) is open: it takes the body, the screen under it hides.
    ProjectPicker(ProjectPickerView),
}

/// A group of the help line: `PAD ↑ ↓ ← → select Enter mode: gate`, keys drawn as caps.
pub struct HelpGroup {
    pub name: &'static str,
    /// (keys, what they do; may include the current state).
    pub items: Vec<(String, String)>,
}

pub struct Help<'a> {
    pub groups: &'a [HelpGroup],
    /// Next-step hint shown when there is no notice.
    pub hint: Option<&'a str>,
    /// Full help (F1): (keys, description); `None` when closed.
    pub full: Option<&'a [(String, String)]>,
    /// A question that waits for Enter or Esc (overwrite pads?), line by line.
    pub question: Option<&'a [String]>,
    /// The learn mode in words (52): the first line takes the top of the screen, the rest
    /// the help band.
    pub learn: Option<&'a [String]>,
}

pub fn draw(frame: &mut Frame, s: &Status, screen: &Screen, help: &Help) {
    draw_inner(frame, s, screen, help, None);
}

/// The logo of the splash (73), byte-exact: 4 lines of 10 characters, trailing spaces
/// kept.
#[rustfmt::skip]
pub const SPLASH_LOGO: [&str; 4] = [
    "          ",
    "  _ _/_ __",
    " /_// _\\/ ",
    "/         ",
];

/// What the splash shows besides the logo (73).
pub struct Splash {
    /// The version of the build.
    pub version: &'static str,
    /// What loads now, `ready` when all is loaded, or a file that did not load.
    pub loading: String,
}

/// The splash in the wave strip at start (73): the logo on the left, the name, the
/// version and the loading line on its right, instead of the wave, until the first sound.
fn splash_strip(frame: &mut Frame, area: Rect, s: &Splash) {
    if area.height < 4 {
        // a very low terminal: one line says the name and what loads
        let line = Line::from(vec![
            Span::styled("ptsr", Style::new().fg(CYAN).bold()),
            Span::styled(format!(" · {}", s.loading), Style::new().fg(TEXT)),
        ]);
        let [a] = Layout::vertical([Constraint::Length(1)])
            .flex(Flex::Center)
            .areas(area);
        frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), a);
        return;
    }
    let logo_w = SPLASH_LOGO
        .iter()
        .map(|l| Line::from(*l).width())
        .max()
        .unwrap_or(0) as u16;
    let info = vec![
        Line::from("ptsr - PT's SampleR").fg(CYAN).bold(),
        Line::from(s.version).fg(DIM),
        Line::from(s.loading.as_str()).fg(TEXT),
    ];
    let info_w = info.iter().map(|l| l.width()).max().unwrap_or(0) as u16;
    let [mid] = Layout::vertical([Constraint::Length(4)])
        .flex(Flex::Center)
        .areas(area);
    let [logo, _, text] = Layout::horizontal([
        Constraint::Length(logo_w),
        Constraint::Length(2),
        Constraint::Length(info_w),
    ])
    .flex(Flex::Center)
    .areas(mid);
    frame.render_widget(
        Paragraph::new(
            SPLASH_LOGO
                .iter()
                .map(|l| Line::from(*l).fg(TEXT))
                .collect::<Vec<_>>(),
        ),
        logo,
    );
    frame.render_widget(Paragraph::new(info), text);
}

/// The frame with kitty graphics (04b): the wave areas are left blank and their
/// pictures are described in `images` for the caller to send to the terminal. The
/// drawing is otherwise the same as [`draw`].
pub fn draw_gfx(
    frame: &mut Frame,
    s: &Status,
    screen: &Screen,
    help: &Help,
    images: &mut Vec<WaveImage>,
) {
    draw_inner(frame, s, screen, help, Some(images));
}

fn draw_inner(
    frame: &mut Frame,
    s: &Status,
    screen: &Screen,
    help: &Help,
    images: Option<&mut Vec<WaveImage>>,
) {
    frame.render_widget(Block::new().bg(BG).fg(TEXT), frame.area());
    let help_rows = help_lines(help.groups, frame.area().width);
    let [top, body, notice, help_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(HELP_ROWS),
    ])
    .areas(frame.area());

    match help.learn {
        Some(lines) => {
            let mut spans = s.save.spans();
            spans.push(Span::styled(
                lines.first().cloned().unwrap_or_default(),
                Style::new().fg(BG).bg(MAGENTA).bold(),
            ));
            frame.render_widget(
                Line::from(spans).style(Style::new().fg(BG).bg(MAGENTA).bold()),
                top,
            );
        }
        None => frame.render_widget(status_line(s), top),
    }
    let pickup = match screen {
        Screen::Fx(f) => f.pickup.as_deref(),
        _ => None,
    };
    match (s.notice, pickup, help.hint) {
        (Some(n), _, _) | (None, Some(n), _) => {
            frame.render_widget(Line::from(format!(" {n}")).fg(MAGENTA), notice)
        }
        (None, None, Some(h)) => frame.render_widget(Line::from(format!(" {h}")).fg(DIM), notice),
        _ => {}
    }
    match screen {
        // the project picker (11) is modal: it takes the body instead of the screen,
        // the help line below stays as on any screen
        Screen::ProjectPicker(p) => frame.render_widget(project_picker(p, s, body.width), body),
        Screen::Pads(p) => pads(frame, body, p, s),
        Screen::Fx(f) => fx(frame, body, f, s),
        Screen::Settings(st) => settings(frame, body, st, s),
        Screen::Files(f) => files(frame, body, f, s),
        Screen::Chop(c) => chop(frame, body, c, s, images),
    }
    match help.learn {
        Some(lines) => {
            let rest: Vec<Line> = lines
                .iter()
                .skip(1)
                .map(|l| Line::from(l.clone()))
                .collect();
            frame.render_widget(Paragraph::new(rest), help_area);
        }
        None => frame.render_widget(Paragraph::new(help_rows), help_area),
    }
    if let Some(full) = help.full {
        full_help(frame, body, full);
    }
    if let Some(q) = help.question {
        question(frame, body, q);
    }
}

fn question(frame: &mut Frame, area: Rect, lines: &[String]) {
    let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16 + 4;
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(lines.len() as u16 + 2)])
        .flex(Flex::Center)
        .areas(area);
    let last = lines.len().saturating_sub(1);
    let text: Vec<Line> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let line = Line::from(format!(" {l}"));
            match i {
                0 => line.fg(MAGENTA).bold(),
                i if i == last => line.fg(CYAN),
                _ => line,
            }
        })
        .collect();
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(text).block(
            Block::bordered()
                .border_style(Style::new().fg(MAGENTA))
                .bg(PANEL),
        ),
        area,
    );
}

/// How many rows the help line takes at `width` columns.
pub fn help_rows(groups: &[HelpGroup], width: u16) -> usize {
    help_lines(groups, width).len()
}

/// How many groups drop off at `width` columns, from the end, for want of room.
pub fn help_dropped(groups: &[HelpGroup], width: u16) -> usize {
    let shown = groups.iter().filter(|g| !g.items.is_empty()).count();
    let lines = help_lines(groups, width);
    shown
        - lines.first().map_or(0, |l| {
            l.spans.iter().filter(|s| s.content == " │ ").count() + 1
        })
}

/// The help in `width` columns (43): every group is a column of up to `HELP_ROWS` rows of
/// `key text`, its name beside the first row; a longer group takes more columns under the
/// same name. Groups that do not fit drop off whole from the end (the screens, then REC).
fn help_lines(groups: &[HelpGroup], width: u16) -> Vec<Line<'static>> {
    let rows = HELP_ROWS as usize;
    let mut blocks: Vec<Vec<Vec<Span<'static>>>> = groups
        .iter()
        .filter(|g| !g.items.is_empty())
        .map(|g| help_block(g, rows))
        .collect();
    let block_width = |b: &Vec<Vec<Span>>| b[0].iter().map(|s| s.width()).sum::<usize>();
    // the leading space, then " │ " between the groups
    let total = |bs: &[Vec<Vec<Span>>]| {
        1 + bs.iter().map(block_width).sum::<usize>() + 3 * bs.len().saturating_sub(1)
    };
    while blocks.len() > 1 && total(&blocks) > width as usize {
        blocks.pop();
    }
    let used = blocks
        .iter()
        .map(|b| {
            b.iter()
                .filter(|r| r.iter().any(|s| !s.content.trim().is_empty()))
                .count()
        })
        .max()
        .unwrap_or(1)
        .max(1);
    (0..used)
        .map(|r| {
            let mut line: Vec<Span> = vec![" ".into()];
            for (i, b) in blocks.iter().enumerate() {
                if i > 0 {
                    line.push(" │ ".fg(BORDER));
                }
                line.extend(b[r].iter().cloned());
            }
            Line::from(line)
        })
        .collect()
}

/// One group as `rows` rows of equal width: the name, then columns of `key text` cells.
/// A group shorter than the band puts its name on the free first row, which saves the
/// name's width.
fn help_block(g: &HelpGroup, rows: usize) -> Vec<Vec<Span<'static>>> {
    if !g.name.is_empty() && g.items.len() < rows {
        let mut inner = help_block(
            &HelpGroup {
                name: "",
                items: g.items.clone(),
            },
            rows - 1,
        );
        let w = inner[0]
            .iter()
            .map(|s| s.width())
            .sum::<usize>()
            .max(g.name.len());
        for row in inner.iter_mut() {
            let have: usize = row.iter().map(|s| s.width()).sum();
            row.push(" ".repeat(w - have).into());
        }
        inner.insert(0, vec![format!("{:<w$}", g.name).fg(DIM)]);
        return inner;
    }
    let name = if g.name.is_empty() {
        String::new()
    } else {
        format!("{} ", g.name)
    };
    let mut out: Vec<Vec<Span>> = (0..rows)
        .map(|r| {
            let text = if r == 0 {
                name.clone()
            } else {
                " ".repeat(name.chars().count())
            };
            vec![text.fg(DIM)]
        })
        .collect();
    for (c, column) in g.items.chunks(rows).enumerate() {
        let caps = column
            .iter()
            .map(|(k, _)| k.chars().count() + 2)
            .max()
            .unwrap_or(0);
        let texts = column
            .iter()
            .map(|(_, t)| t.chars().count())
            .max()
            .unwrap_or(0);
        for (r, row) in out.iter_mut().enumerate() {
            if c > 0 {
                row.push("  ".into());
            }
            match column.get(r) {
                Some((k, t)) => {
                    let cap = key(k);
                    let pad = caps - cap.width();
                    row.push(cap);
                    // the cap's own trailing space parts it from the text
                    row.push(format!("{}{t:<texts$}", " ".repeat(pad)).into());
                }
                None => row.push(" ".repeat(caps + texts).into()),
            }
        }
    }
    out
}

fn full_help(frame: &mut Frame, area: Rect, rows: &[(String, String)]) {
    let keys_width = rows
        .iter()
        .map(|(k, _)| k.chars().count())
        .max()
        .unwrap_or(0);
    let text_width = rows
        .iter()
        .map(|(_, d)| d.chars().count())
        .max()
        .unwrap_or(0);
    // a row is " cap  text": the cap pads the key by one column on each side; plus the frame
    let width = ((keys_width + text_width + 7) as u16).min(area.width);
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(rows.len() as u16 + 2)])
        .flex(Flex::Center)
        .areas(area);
    let lines: Vec<Line> = rows
        .iter()
        .map(|(k, d)| {
            // a row without keys is a heading
            if k.is_empty() {
                return Line::from(format!(" {d}")).fg(DIM).bold();
            }
            let pad = keys_width - k.chars().count();
            Line::from(vec![
                " ".into(),
                key(k),
                format!("{:pad$}  ", "").into(),
                d.clone().into(),
            ])
        })
        .collect();
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .title(" HELP · F1 closes ")
                .border_style(Style::new().fg(CYAN))
                .bg(PANEL),
        ),
        area,
    );
}

/// Pad width bounds: narrow terminals keep the sketch size, wide ones show longer names.
const PAD_W_MIN: u16 = 13;
const PAD_W_MAX: u16 = 40;

/// Pad width that leaves `reserve` columns for what stands next to the grid.
fn pad_width(total: u16, reserve: u16) -> u16 {
    (total.saturating_sub(reserve) / 4).clamp(PAD_W_MIN, PAD_W_MAX)
}
const PAD_H: u16 = 4;
/// Rows the info panel reserves under the selected pad: the parameter lines and the
/// edit frame around them, kept even while the frame is not drawn so nothing jumps.
const PANEL_ROWS: u16 = 30;
/// Rows the help line always keeps at the bottom, so the screen above never moves.
const HELP_ROWS: u16 = 3;

/// "bank [1 ⣿⣿] 2 ⣰⣄ …" line: each bank shows its fill as two braille glyphs,
/// every bank in its own color, the open one bold in brackets.
fn banks(p: &Pads) -> Vec<Span<'static>> {
    // every bank brings its own leading space
    let mut spans = vec!["bank".fg(DIM)];
    // the pads of the open bank lit right now: a hit flashes in the row too (62)
    let hits = p
        .pads
        .iter()
        .take(16)
        .enumerate()
        .filter(|(_, pad)| pad.flash)
        .fold(0u16, |m, (i, _)| m | 1 << i);
    for (i, &color) in BANK_COLORS.iter().enumerate() {
        let b = i + 1;
        // one glyph cell has one colour, so a bank shows one thing at a time: the pads
        // held or flashing (white, 62), else its marks (yellow, 76), else its fill
        let (mask, dots) = if i == p.bank && hits != 0 {
            (hits, Some(WHITE))
        } else if p.bank_marks[i] != 0 {
            (p.bank_marks[i], Some(MARK))
        } else {
            (p.bank_fill[i], None)
        };
        let glyphs = bank_glyphs(mask);
        let dots = dots.map(|c| Style::new().fg(c).bold());
        // the closed banks keep their number and glyphs together: the row fits a narrow
        // window
        if i == p.bank {
            let style = Style::new().fg(color).bold();
            match dots {
                // the space before the bracket, as before a closed bank: no `1⣿⣿[2`
                Some(dots) => {
                    spans.push(Span::styled(format!(" [{b} "), style));
                    spans.push(Span::styled(glyphs, dots));
                    spans.push(Span::styled("]", style));
                }
                None => spans.push(Span::styled(format!(" [{b} {glyphs}]"), style)),
            }
        } else {
            let style = Style::new().fg(color);
            match dots {
                Some(dots) => {
                    spans.push(Span::styled(format!(" {b}"), style));
                    spans.push(Span::styled(glyphs, dots));
                }
                None => spans.push(Span::styled(format!(" {b}{glyphs}"), style)),
            }
        }
    }
    spans
}

/// Two braille glyphs mirroring the 4×4 pad grid of a bank: a lit dot is a pad with a sound
/// (bit `r*4+c` of `mask` is pad at row `r`, column `c`).
fn bank_glyphs(mask: u16) -> String {
    // Braille dot bits: left dot column rows 0..3 and right dot column rows 0..3.
    const LEFT: [u32; 4] = [0x01, 0x02, 0x04, 0x40];
    const RIGHT: [u32; 4] = [0x08, 0x10, 0x20, 0x80];
    let mut out = String::new();
    for half in 0..2 {
        let mut bits = 0u32;
        for row in 0..4 {
            for col in 0..2 {
                if mask & (1 << (row * 4 + half * 2 + col)) != 0 {
                    bits |= if col == 0 { LEFT[row] } else { RIGHT[row] };
                }
            }
        }
        out.push(char::from_u32(0x2800 + bits).unwrap_or('\u{2800}'));
    }
    out
}

/// Rows of the pad frame besides the strip and the pad waves: the title, the bank row, the grid.
const FRAME_ROWS: u16 = 2 + PAD_H * 4;
/// The strip keeps this many rows before the pad waves get any; below it only on a low
/// terminal, down to [`STRIP_MIN`].
const STRIP_FULL: u16 = 8;
const STRIP_MIN: u16 = 3;
/// Columns the right column of the pad frame keeps next to the grid.
const SIDE_W: u16 = 40;

/// The frame of the screens with pads (31): the same rows and columns on F2, F6 and F3.
struct PadFrame {
    title: Rect,
    /// Over the full width, all the height the grid leaves.
    strip: Rect,
    bank: Rect,
    grid: Rect,
    /// Wave rows in each pad cell (29).
    waves: u16,
    /// The mode column, from the bank row down.
    side: Rect,
}

/// The pad waves give way first, then the strip shrinks; a tall terminal gives the rest to the strip.
/// `hidden` (32): the strip stands in the grid's place, the left column from the title down
/// with the pad line under it; the right column keeps the full height, it needs its rows.
fn pad_frame(area: Rect, hidden: bool) -> PadFrame {
    if hidden {
        let [title, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        let [left, _, side] = Layout::horizontal([
            Constraint::Length(pad_width(area.width, SIDE_W) * 4),
            Constraint::Length(2),
            Constraint::Fill(1),
        ])
        .areas(body);
        let [strip, bank] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(left);
        return PadFrame {
            title,
            strip,
            bank,
            grid: Rect { height: 0, ..bank },
            waves: 0,
            side,
        };
    }
    let free = area.height.saturating_sub(FRAME_ROWS);
    let waves = (free.saturating_sub(STRIP_FULL) / PAD_H).min(3);
    let strip_h = (free - waves * PAD_H).max(STRIP_MIN);
    let [title, strip, body] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(strip_h),
        Constraint::Fill(1),
    ])
    .areas(area);
    let [left, _, side] = Layout::horizontal([
        Constraint::Length(pad_width(area.width, SIDE_W) * 4),
        Constraint::Length(2),
        Constraint::Fill(1),
    ])
    .areas(body);
    let [bank, grid] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(left);
    PadFrame {
        title,
        strip,
        bank,
        grid,
        waves,
        side,
    }
}

/// The 4×4 grid of the current bank; `waves` extra Braille rows in each pad cell (29),
/// 0 keeps the cells exactly as they were.
fn pad_grid(frame: &mut Frame, area: Rect, p: &Pads, waves: u16) {
    let bank = BANK_COLORS[p.bank % BANK_COLORS.len()];
    let rows = Layout::vertical([Constraint::Length(PAD_H + waves); 4]).split(area);
    for (r, row) in rows.iter().enumerate() {
        let cells = Layout::horizontal([Constraint::Length(area.width / 4); 4]).split(*row);
        for (c, cell) in cells.iter().enumerate() {
            let i = r * 4 + c;
            if let Some(pad) = p.pads.get(i) {
                pad_cell(
                    frame,
                    *cell,
                    pad,
                    i == p.selected,
                    bank,
                    waves,
                    (p.feedback.outline, p.feedback.corners[i]),
                );
            }
        }
    }
}

/// The line in place of a hidden grid (32): the bank row (`with_bank`, 62), then the
/// selected pad and what it plays; it flashes as the pad does on a hit. A narrow window
/// cuts the pad part at the end first.
fn pad_line(p: &Pads, mut spans: Vec<Span<'static>>, with_bank: bool) -> Line<'static> {
    if with_bank {
        // every bank, as above the grid (62); the pad part after it is cut first
        spans.extend(banks(p));
        spans.push(" · ".fg(DIM));
    }
    if let Some(sel) = p.pads.get(p.selected) {
        let style = if sel.flash {
            Style::new().fg(BG).bg(WHITE)
        } else {
            Style::new().fg(CYAN)
        };
        spans.push(Span::styled(
            format!("pad {} {}", sel.key.to_uppercase(), sel.name),
            style,
        ));
    }
    Line::from(spans)
}

/// Keep fixed slots first. Drop lower priorities before compacting the last badge.
fn title<'a>(mut fixed: Vec<Span<'a>>, badges: &[Badge], width: u16, reserve: usize) -> Line<'a> {
    let available =
        (width as usize).saturating_sub(fixed.iter().map(Span::width).sum::<usize>() + reserve);
    let mut count = badges.len().min(3);
    let normal_width = |n: usize| {
        badges[..n]
            .iter()
            .map(|b| Line::from(b.normal.as_str()).width() + 3)
            .sum::<usize>()
    };
    while count > 1 && normal_width(count) > available {
        count -= 1;
    }
    let compact = normal_width(count) > available;
    let used = badges[..count]
        .iter()
        .map(|b| {
            Line::from(if compact {
                b.compact.as_str()
            } else {
                b.normal.as_str()
            })
            .width()
                + 3
        })
        .sum::<usize>();
    if used <= available {
        for b in &badges[..count] {
            let color = match b.kind {
                BadgeKind::PadMode => MARK,
                BadgeKind::On => CYAN,
                BadgeKind::Attention => MAGENTA,
            };
            let text = if compact { &b.compact } else { &b.normal };
            fixed.push(" ".into());
            fixed.push(Span::styled(
                format!(" {text} "),
                Style::new().fg(BG).bg(color).bold(),
            ));
        }
    }
    Line::from(fixed)
}

/// Shorten title details in display cells, not UTF-8 bytes.
fn title_detail(text: &str, width: usize) -> String {
    if Line::from(text).width() <= width {
        return text.to_string();
    }
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    for ch in text.chars() {
        let mut next = out.clone();
        next.push(ch);
        if Line::from(next.as_str()).width() + 1 > width {
            break;
        }
        out = next;
    }
    out.push('…');
    out
}

fn pads(frame: &mut Frame, area: Rect, p: &Pads, s: &Status) {
    let f = pad_frame(area, p.hidden);
    let (badge, badge_style) = if p.playing {
        (" PLAY ▶   ", Style::new().fg(BG).bg(CYAN))
    } else {
        (" PLAY     ", Style::new().fg(DIM).bg(BG))
    };
    frame.render_widget(
        title(
            vec![
                Span::styled(" F2 PADS ", Style::new().fg(BG).bg(TEXT)),
                " ".into(),
                Span::styled(badge, badge_style),
            ],
            &s.badges,
            f.title.width,
            0,
        ),
        f.title,
    );
    if p.hidden {
        frame.render_widget(pad_line(p, Vec::new(), true), f.bank);
    } else {
        frame.render_widget(Line::from(banks(p)), f.bank);
    }
    // a recording takes the strip in the rec colour (25), `out_wave` then holds what goes
    // into the file; the pad edit draws the pad's envelope over its piece; else the output
    match (p.edit, &p.big) {
        _ if s.recording => envelope_strip(frame, f.strip, &p.out_wave, MAGENTA),
        (Some(_), _) if p.curve.is_some() => filter_strip(
            frame,
            f.strip,
            p.curve.as_ref().unwrap_or(&FilterCurve::default()),
        ),
        (Some(_), Some(big)) => {
            let gate = p.pads.get(p.selected).is_some_and(|v| v.gate);
            pad_envelope_strip(frame, f.strip, big, p.param, gate)
        }
        _ if p.splash.is_some() => splash_strip(frame, f.strip, p.splash.as_ref().unwrap()),
        _ => envelope_strip(frame, f.strip, &p.out_wave, DIM),
    }
    let [meters_area, _, info] = Layout::vertical([
        Constraint::Length(METER_ROWS),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(f.side);
    meters(frame, meters_area, s);
    if !p.hidden {
        pad_grid(frame, f.grid, p, f.waves);
    }

    if let Some(sel) = p.pads.get(p.selected) {
        let [panel, _] =
            Layout::vertical([Constraint::Length(PANEL_ROWS), Constraint::Fill(1)]).areas(info);
        let lines = vec![
            Line::from(vec![
                "pad ".fg(DIM),
                Span::styled(
                    sel.key.to_uppercase().to_string(),
                    Style::new().fg(CYAN).bold(),
                ),
                format!("  bank {}", p.bank + 1).fg(DIM),
                // the closed panel says how to open the edit; the edit has its frame (70)
                if p.edit.is_none() && !p.edit_key.is_empty() {
                    format!("   {}: edit", p.edit_key).fg(DIM)
                } else {
                    "".into()
                },
            ]),
            // what the next change hits, on top: under the blocks it read as a part of the
            // last one (70); the same word as the help line (68)
            Line::from(vec!["  scope   ".fg(DIM), p.scope.clone().fg(CYAN)]),
            Line::from(sel.detail.clone().unwrap_or_else(|| sel.name.clone())),
            Line::from(vec!["  mode    ".fg(DIM), mode(sel.gate).into()]),
            param_line(
                p,
                PadParam::Volume,
                "volume  ",
                format!("{:+.0} dB", sel.volume_db),
            ),
            param_line(
                p,
                PadParam::Pitch,
                "pitch   ",
                format!("{:+} st", sel.pitch),
            ),
            param_line(
                p,
                PadParam::Group,
                "mute    ",
                if sel.group == 0 {
                    "off".to_string()
                } else {
                    sel.group.to_string()
                },
            ),
        ];
        // every page at once (43), in blocks under their names (48): a page change moves
        // only the knob numbers and the lit name
        let width = panel.width.saturating_sub(2) as usize;
        let block = |name: &str, folded: bool| -> Line<'static> {
            // a block with a change carries a dot (55)
            let params: &[PadParam] = match name {
                "TONE" => &[PadParam::Volume, PadParam::Pitch, PadParam::Group],
                "ENV" => &[PadParam::Attack, PadParam::Decay, PadParam::Release],
                "FILTER" => &[PadParam::Filter, PadParam::Cutoff, PadParam::Reso],
                "FX" => &[PadParam::Drive, PadParam::Send1, PadParam::Send2],
                _ => &[PadParam::Source, PadParam::Start, PadParam::End],
            };
            let mut name = if params.iter().any(|w| sel.changed.contains(w)) {
                format!("{name} ●")
            } else {
                name.to_string()
            };
            if folded {
                name.push_str(" ▸");
            }
            let rule = "─".repeat(width.saturating_sub(name.chars().count() + 3));
            let lit = p.edit.is_some_and(|e| name.starts_with(e));
            let style = if lit {
                Style::new().fg(MAGENTA).bold()
            } else {
                Style::new().fg(DIM)
            };
            Line::from(Span::styled(format!("─ {name} {rule}"), style))
        };
        // the blocks under the pad rows: (name, its parameters, its rows)
        let mut blocks: Vec<(&str, Vec<PadParam>, Vec<Line>)> = vec![
            (
                "ENV",
                vec![PadParam::Attack, PadParam::Decay, PadParam::Release],
                vec![
                    param_line(
                        p,
                        PadParam::Attack,
                        "attack  ",
                        format!("{} ms", sel.attack_ms),
                    ),
                    param_line(
                        p,
                        PadParam::Decay,
                        "decay   ",
                        format!("{} ms", sel.decay_ms),
                    ),
                    release_line(p, sel),
                ],
            ),
            (
                "FILTER",
                vec![PadParam::Filter, PadParam::Cutoff, PadParam::Reso],
                vec![
                    param_line(p, PadParam::Filter, "filter  ", sel.filter[0].clone()),
                    param_line(p, PadParam::Cutoff, "cutoff  ", sel.filter[1].clone()),
                    param_line(p, PadParam::Reso, "reso    ", sel.filter[2].clone()),
                ],
            ),
            (
                "SOURCE",
                vec![PadParam::Source, PadParam::Start, PadParam::End],
                vec![
                    param_line(p, PadParam::Source, "source  ", sel.source.clone()),
                    param_line(
                        p,
                        PadParam::Start,
                        "start   ",
                        format!("{:.3} s", sel.bounds.0),
                    ),
                    param_line(
                        p,
                        PadParam::End,
                        "end     ",
                        format!("{:.3} s", sel.bounds.1),
                    ),
                ],
            ),
        ];
        // Only drive and sends belong to the pad. Shared slots have their own screen (82).
        if !p.fx.is_empty() {
            blocks.push((
                "FX",
                p.fx.iter().map(|r| r.0).collect(),
                p.fx.iter()
                    .map(|(w, label, value)| param_line(p, *w, label, value.clone()))
                    .collect(),
            ));
        }
        // the pad rows: TONE stands under the mode
        let mut lines = lines;
        let tone = lines.split_off(4);
        // the open page stays unfolded; with the edit closed, the block of the picked one
        let open = |name: &str, params: &[PadParam]| match p.edit {
            Some(e) => e == name,
            None => params.contains(&p.param),
        };
        let tone_params = [PadParam::Volume, PadParam::Pitch, PadParam::Group];
        // the pickup hint only while there is one, it is the last row
        let mut tail = Vec::new();
        if let Some(hint) = &p.pickup {
            tail.push(Line::from(hint.clone()).fg(MAGENTA));
        }
        let full = lines.len()
            + 1
            + tone.len()
            + blocks.iter().map(|b| 1 + b.2.len()).sum::<usize>()
            + tail.len();
        // a short window folds the blocks to their titles, the lowest first (70); the open
        // page and the pad rows stay
        let room = framed_inner_height(panel) as usize;
        let mut over = full.saturating_sub(room);
        let mut folded = vec![false; blocks.len()];
        for i in (0..blocks.len()).rev() {
            if over == 0 {
                break;
            }
            if !open(blocks[i].0, &blocks[i].1) {
                folded[i] = true;
                over = over.saturating_sub(blocks[i].2.len());
            }
        }
        let tone_folded = over > 0 && !open("TONE", &tone_params);
        // the blocks, each row with the parameter it shows: the scroll follows the pick
        let mut body: Vec<(Option<PadParam>, Line)> = vec![(None, block("TONE", tone_folded))];
        if !tone_folded {
            body.extend(tone_params.iter().map(|w| Some(*w)).zip(tone));
        }
        for ((name, params, rows), folded) in blocks.into_iter().zip(folded) {
            body.push((None, block(name, folded)));
            if !folded {
                body.extend(params.into_iter().map(Some).zip(rows));
            }
        }
        // still too tall: the blocks scroll with the picked parameter in sight, and say how
        // many rows are above and below (70)
        let space = room.saturating_sub(lines.len() + tail.len());
        if body.len() > space && space >= 3 {
            let at = body
                .iter()
                .position(|(w, _)| *w == Some(p.param))
                .unwrap_or(0);
            let start = at.saturating_sub(space / 2).min(body.len() - space);
            let end = start + space;
            let mut seen: Vec<Line> = body[start..end].iter().map(|(_, l)| l.clone()).collect();
            // a marker takes the edge row; the pick sits in the middle, away from the edges
            if start > 0 {
                seen[0] = Line::from(format!("  ▲ {} more", start + 1)).fg(DIM);
            }
            if end < body.len() {
                let n = seen.len();
                seen[n - 1] = Line::from(format!("  ▼ {} more", body.len() - end + 1)).fg(DIM);
            }
            lines.extend(seen);
        } else {
            lines.extend(body.into_iter().map(|(_, l)| l));
        }
        lines.extend(tail);
        // a narrow panel shortens the labels, so the values stay whole (70)
        if width < NARROW_PANEL {
            for line in &mut lines {
                for span in &mut line.spans {
                    if let Some(short) = short_label(&span.content) {
                        span.content = short.into();
                    }
                }
            }
        }
        // the edit mode frames the panel and names its page; closed, the frame's place
        // stays empty, so the text does not move when the mode opens
        let framed = Block::bordered();
        let text_area = framed.inner(panel);
        if let Some(page) = p.edit {
            // every page named, the current one lit: Tab and I K lead through them; a title
            // wider than the panel drops "EDIT ·", the frame colour still tells the mode (56)
            let pages_len: usize = p.pages.iter().map(|n| n.chars().count() + 1).sum::<usize>() + 1;
            let fits = pages_len + 7 <= panel.width.saturating_sub(2) as usize;
            // the space after the last page may be cut by the frame
            let all_fit = pages_len <= panel.width.saturating_sub(2) as usize + 1;
            let mut title = if fits {
                vec![Span::styled(" EDIT ·", Style::new().fg(MAGENTA).bold())]
            } else {
                Vec::new()
            };
            // too narrow for every page: the open one and its place, `TONE 1/7` (70)
            let at = p.pages.iter().position(|&n| n == page).unwrap_or(0);
            if !all_fit {
                title.push(" ".into());
                title.push(Span::styled(
                    format!("{page} {}/{}", at + 1, p.pages.len()),
                    Style::new().fg(BG).bg(MAGENTA).bold(),
                ));
            }
            for &name in p.pages.iter().filter(|_| all_fit) {
                title.push(" ".into());
                title.push(if name == page {
                    Span::styled(name, Style::new().fg(BG).bg(MAGENTA).bold())
                } else {
                    name.fg(DIM)
                });
            }
            title.push(" ".into());
            frame.render_widget(
                framed
                    .border_style(Style::new().fg(MAGENTA))
                    .title(Line::from(title)),
                panel,
            );
        }
        // no wrap: a long name is cut at the edge instead of pushing the last rows out
        frame.render_widget(Paragraph::new(lines), text_area);
    }
}

/// Below this many columns inside the frame the panel labels shorten (70): a label takes
/// 10 columns, a value up to 12.
const NARROW_PANEL: usize = 22;

/// The short label of a narrow panel (70).
fn short_label(label: &str) -> Option<&'static str> {
    Some(match label {
        "  mode    " => "  mode ",
        "  scope   " => "  scope ",
        "volume  " => "vol  ",
        "pitch   " => "pit  ",
        "mute    " => "mute ",
        "attack  " => "atk  ",
        "decay   " => "dec  ",
        "release " => "rel  ",
        "filter  " => "flt  ",
        "cutoff  " => "cut  ",
        "reso    " => "res  ",
        "source  " => "src  ",
        "start   " => "in   ",
        "end     " => "out  ",
        "drive   " => "drv  ",
        "send 1  " => "s1   ",
        "send 2  " => "s2   ",
        "type    " => "type ",
        "size    " => "size ",
        "tone    " => "tone ",
        "time    " => "time ",
        "feedback" => "fb   ",
        _ => return None,
    })
}

/// The rows inside the panel frame: the frame is drawn in the edit mode, its place is
/// kept when closed.
fn framed_inner_height(panel: Rect) -> u16 {
    Block::bordered().inner(panel).height
}

/// The release row: a one-shot pad keeps the value but plays to its end, so the release
/// acts in gate only — dimmed with "(gate)" until the pad is gate.
fn release_line(p: &Pads, sel: &PadView) -> Line<'static> {
    let mut line = param_line(
        p,
        PadParam::Release,
        "release ",
        format!("{} ms", sel.release_ms),
    );
    if !sel.gate {
        for span in line.spans.iter_mut().skip(1) {
            if span.style.fg != Some(CYAN) && span.style.bg.is_none() {
                span.style = span.style.fg(BORDER);
            }
        }
        line.spans.push(" (gate)".fg(BORDER));
    }
    line
}

/// A parameter row of the pad panel: the knob number while its page is up, the label, and
/// the value — highlighted when picked, lit for a moment after a change.
/// A learn label: the control now in magenta, a waiting one after ` → ` in amber (57).
fn learn_spans(text: &str) -> Vec<Span<'static>> {
    match text.split_once(" → ") {
        Some((now, new)) => vec![
            now.to_string().fg(MAGENTA),
            " → ".fg(DIM),
            Span::styled(new.to_string(), Style::new().fg(AMBER).bold()),
        ],
        None => vec![text.to_string().fg(MAGENTA)],
    }
}

fn param_line(p: &Pads, which: PadParam, label: &'static str, value: String) -> Line<'static> {
    let knob = match p.knobs.iter().position(|&k| k == which) {
        Some(i) => Span::styled(format!("{} ", i + 1), Style::new().fg(MAGENTA).bold()),
        None => "  ".into(),
    };
    let style = if p.lit == Some(which) {
        Style::new().fg(BG).bg(CYAN).bold()
    } else if p.param == which {
        Style::new().fg(CYAN).bold()
    } else {
        Style::new()
    };
    // a value off its default stands out, a default one steps back (55)
    let changed = p
        .pads
        .get(p.selected)
        .is_some_and(|s| s.changed.contains(&which));
    let style = match (style == Style::new(), changed) {
        (true, true) => Style::new().fg(WHITE),
        (true, false) => Style::new().fg(DIM),
        (false, _) => style,
    };
    let mut spans = vec![knob, label.fg(DIM), Span::styled(value, style)];
    if let Some((_, control)) = p.learn.iter().find(|(w, _)| *w == which) {
        spans.extend(learn_spans(&format!("  ◀ {control}")));
    }
    Line::from(spans)
}

/// The live strip above the grid: the last seconds of the master output as a wave,
/// the newest sound at the right; silence is a thin middle line.
fn envelope_strip(frame: &mut Frame, area: Rect, out_wave: &[f32], color: Color) {
    let rows = Layout::vertical(vec![Constraint::Length(1); area.height as usize]).split(area);
    let silence = [0.0f32];
    let wave = if out_wave.is_empty() {
        &silence[..]
    } else {
        out_wave
    };
    // the meters' decibel scale: -60 dB is the middle line, 0 dBFS is full height,
    // so a peak stands as tall on the strip as its bar on the meters
    let db: Vec<f32> = wave.iter().map(|&p| db_height(p)).collect();
    for (r, line) in braille_abs(&db, area.width as usize, rows.len())
        .iter()
        .enumerate()
    {
        frame.render_widget(
            Line::from(Span::styled(line.clone(), Style::new().fg(color))),
            rows[r],
        );
    }
}

/// The strip on the FILTER page (44): the response as a line of Braille dots, the
/// frequency along the octaves from the left, the level upwards from −36 to +12 dB; the
/// caption on the top row and a few frequencies on the bottom one.
fn filter_strip(frame: &mut Frame, area: Rect, curve: &FilterCurve) {
    let (cols, rows) = (area.width as usize, area.height as usize);
    if cols == 0 || rows == 0 || curve.levels.is_empty() {
        return;
    }
    let (dx, dy) = (cols * 2, rows * 4);
    const BITS: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
    let mut dots = vec![vec![0u8; cols]; rows];
    let db = |l: f32| 20.0 * l.max(1e-6).log10();
    // the hump's top stays inside: the scale rises with it
    let peak = curve.levels.iter().copied().fold(0.0, f32::max);
    let top = CURVE_TOP_DB.max(db(peak) + 3.0);
    for x in 0..dx {
        let i = (x * curve.levels.len() / dx).min(curve.levels.len() - 1);
        let t = (top - db(curve.levels[i]).clamp(CURVE_BOTTOM_DB, top)) / (top - CURVE_BOTTOM_DB);
        let y = (t * (dy - 1) as f32).round() as usize;
        dots[y / 4][x / 2] |= BITS[x % 2][y % 4];
    }
    let mut text: Vec<Vec<Span>> = dots
        .iter()
        .map(|row| {
            row.iter()
                .map(|&b| {
                    let c = char::from_u32(0x2800 + b as u32).unwrap_or(' ');
                    Span::styled(c.to_string(), Style::new().fg(CYAN))
                })
                .collect()
        })
        .collect();
    let mut put = |row: usize, at: usize, s: &str, style: Style| {
        for (k, ch) in s.chars().enumerate() {
            if let Some(cell) = text.get_mut(row).and_then(|r| r.get_mut(at + k)) {
                *cell = Span::styled(ch.to_string(), style);
            }
        }
    };
    put(0, 1, &curve.caption, Style::new().fg(TEXT).bold());
    if rows > 1 {
        let (lo, hi) = curve.span;
        let decades = (hi / lo).log10();
        for (hz, name) in [
            (20.0f32, "20"),
            (200.0, "200"),
            (2_000.0, "2k"),
            (20_000.0, "20k"),
        ] {
            let x = ((hz / lo).log10() / decades * (cols - 1) as f32).round() as usize;
            let at = x.min(cols.saturating_sub(name.len()));
            put(rows - 1, at, name, Style::new().fg(DIM));
        }
    }
    let lines: Vec<Line> = text.into_iter().map(Line::from).collect();
    frame.render_widget(Paragraph::new(lines), area);
}

/// The strip in the pad edit: the piece's wave dimmed and the envelope over it as a bright
/// line of Braille dots — the attack rises from zero, the decay falls to zero at the end of
/// the piece; the picked part is cyan. The release acts only in gate mode, from the key's
/// release, so a gate pad shows it apart from the wave as a dotted "key up" fall; a
/// one-shot pad shows none.
fn pad_envelope_strip(frame: &mut Frame, area: Rect, big: &BigView, picked: PadParam, gate: bool) {
    let (cols, rows) = (area.width as usize, area.height as usize);
    if cols == 0 || rows == 0 {
        return;
    }
    // the release takes its share of the width, at most a third, at least room for its
    // name, after a gap that keeps it apart from the wave
    const LABEL: &str = "key up";
    let tail = if gate && big.release > 0.0 {
        ((cols as f32 * (big.release / (1.0 + big.release)).min(1.0 / 3.0)).round() as usize)
            .max(LABEL.len() + 2)
            .min(cols / 2)
    } else {
        0
    };
    let piece = cols - tail;
    let gap = usize::from(tail > 0);
    let wave = braille_abs(&big.wave, piece, rows);
    // the line: one dot per dot column, at the envelope's height
    let (dx, dy) = (cols * 2, rows * 4);
    const BITS: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
    let mut line = vec![vec![0u8; cols]; rows];
    let mut part = vec![PadParam::Volume; cols];
    let piece_dots = piece * 2;
    for x in 0..dx {
        let (level, which) = if x < piece_dots {
            let t = (x as f32 + 0.5) / piece_dots.max(1) as f32;
            let i = ((t * big.env.len() as f32) as usize).min(big.env.len().saturating_sub(1));
            let which = if t < big.attack {
                PadParam::Attack
            } else if t > 1.0 - big.decay {
                PadParam::Decay
            } else {
                PadParam::Volume
            };
            (big.env.get(i).copied().unwrap_or(1.0), which)
        } else {
            // the gap column stays empty; then dotted: every other dot column, from full
            // to silence
            let from = piece_dots + gap * 2;
            if x < from || x % 2 == 1 {
                continue;
            }
            let t = (x - from) as f32 / (dx - from).max(1) as f32;
            (1.0 - t, PadParam::Release)
        };
        let y = ((1.0 - level.clamp(0.0, 1.0)) * (dy - 1) as f32).round() as usize;
        line[y / 4][x / 2] |= BITS[x % 2][y % 4];
        part[x / 2] = which;
    }
    // the release's name on the top row, at the right end where the fall is already low
    let label_from = (tail > 0).then(|| cols - LABEL.len());
    for (r, row) in line.iter().enumerate() {
        let wave_row: Vec<char> = wave[r].chars().collect();
        if r == 0
            && let Some(from) = label_from
        {
            let mut spans: Vec<Span> = Vec::new();
            for (x, &bits) in row.iter().enumerate().take(from) {
                spans.push(envelope_cell(bits, x, &wave_row, &part, picked));
            }
            let color = if picked == PadParam::Release {
                CYAN
            } else {
                DIM
            };
            spans.push(Span::styled(LABEL, Style::new().fg(color)));
            frame.render_widget(
                Line::from(spans),
                Rect {
                    y: area.y,
                    height: 1,
                    ..area
                },
            );
            continue;
        }
        let spans: Vec<Span> = row
            .iter()
            .enumerate()
            .map(|(x, &bits)| envelope_cell(bits, x, &wave_row, &part, picked))
            .collect();
        frame.render_widget(
            Line::from(spans),
            Rect {
                y: area.y + r as u16,
                height: 1,
                ..area
            },
        );
    }
}

/// One cell of the envelope strip: a dot of the line (cyan on the picked part) or the dim
/// wave under it.
fn envelope_cell(
    bits: u8,
    x: usize,
    wave_row: &[char],
    part: &[PadParam],
    picked: PadParam,
) -> Span<'static> {
    if bits != 0 {
        let ch = char::from_u32(0x2800 + u32::from(bits)).unwrap_or(' ');
        let color = if part[x] == picked { CYAN } else { WHITE };
        Span::styled(ch.to_string(), Style::new().fg(color))
    } else {
        let ch = wave_row.get(x).copied().unwrap_or(' ');
        Span::styled(ch.to_string(), Style::new().fg(BORDER))
    }
}

/// Height of a peak on the meters' decibel scale: 0 silent, 1 full scale.
fn db_height(p: f32) -> f32 {
    if p > 0.0 {
        ((20.0 * p.log10()).max(METER_FLOOR) - METER_FLOOR) / -METER_FLOOR
    } else {
        0.0
    }
}

/// L/R meters of the master output: a dB bar per channel with a peak-hold mark,
/// and the latching overload light.
fn meters(frame: &mut Frame, area: Rect, s: &Status) {
    let (level, peak, clip) = s.meters;
    let rows = Layout::vertical([Constraint::Length(1); METER_ROWS as usize]).split(area);
    // " L " + bar + " -60 dB": the number keeps its width across the range
    let bar_w = (area.width.saturating_sub(11)) as usize;
    let x_of = |db: f32| {
        (((db - METER_FLOOR) / -METER_FLOOR) * bar_w as f32)
            .round()
            .clamp(0.0, bar_w as f32) as usize
    };
    for (ch, name) in [0usize, 1].into_iter().zip(["L", "R"]) {
        let (lvl, peak) = (x_of(level[ch]), x_of(peak[ch]));
        let mut bar = "█".repeat(lvl);
        if peak > lvl {
            bar.push_str(&" ".repeat(peak - lvl - 1));
            bar.push('▎');
        }
        let bar = format!("{bar:<bar_w$}");
        let line = Line::from(vec![
            Span::styled(format!(" {name} "), Style::new().fg(DIM)),
            Span::styled(bar, Style::new().fg(METER)),
            Span::styled(format!(" {:>4.0} dB", level[ch]), Style::new().fg(BRIGHT)),
        ]);
        frame.render_widget(line, rows[ch]);
    }
    let light = if clip {
        Span::styled(" CLIP ", Style::new().fg(BG).bg(MAGENTA))
    } else {
        Span::styled(" clip ", Style::new().fg(DIM))
    };
    frame.render_widget(Line::from(light), rows[2]);
}

fn mode(gate: bool) -> &'static str {
    if gate { "gate" } else { "one-shot" }
}

fn pad_cell(
    frame: &mut Frame,
    area: Rect,
    pad: &PadView,
    selected: bool,
    bank: Color,
    waves: u16,
    (outline, corner): (bool, Option<PadCorner>),
) {
    // The bank color owns the idle frame and key letter; state colors go above it.
    // A marked pad keeps its frame in every idle state, so the group stays visible.
    let (border, bg, fg) = match (pad.flash, pad.marked, selected) {
        _ if pad.in_hand => (MAGENTA, PANEL, TEXT),
        (true, _, _) => (WHITE, WHITE, BG),
        (false, true, _) => (MARK, MARK_BG, BRIGHT),
        (false, false, true) => (CYAN, PANEL, TEXT),
        (false, false, false) if outline => (MARK, PANEL, if pad.dim { DIM } else { TEXT }),
        // the chop screen: a pad with no slice of the open chop steps back (36)
        (false, false, false) if pad.dim => (BORDER, PANEL, DIM),
        (false, false, false) => (bank, PANEL, TEXT),
    };
    let key = Span::styled(
        pad.key.to_uppercase().to_string(),
        Style::new().fg(if pad.flash {
            BG
        } else if outline && !pad.in_hand && !pad.marked && !selected {
            bank
        } else {
            border
        }),
    );
    let mut block = Block::bordered()
        .title(key)
        .border_style(Style::new().fg(border))
        .style(Style::new().bg(bg).fg(fg));
    if let Some(corner) = corner {
        let text = match corner {
            PadCorner::Available => "◎".to_string(),
            PadCorner::Replace => "!".to_string(),
            PadCorner::Slice(n) => n.to_string(),
        };
        block = block.title_top(
            Line::from(text)
                .fg(if pad.flash { BG } else { MARK })
                .right_aligned(),
        );
    }
    if pad.dashed {
        block = block.border_set(ratatui::symbols::border::Set {
            vertical_left: "┆",
            vertical_right: "┆",
            horizontal_top: "┄",
            horizontal_bottom: "┄",
            ..ratatui::symbols::border::PLAIN
        });
    }
    // the mode as a glyph, the volume, the mute group (77): `⇢ +1 mute 2`; no separator,
    // so `⇢ -24 mute 8` fits the 13 cells of the narrowest pad
    let mut second = if pad.gate { "⇥" } else { "⇢" }.to_string();
    if pad.volume_db != 0.0 {
        second = format!("{second} {:+.0}", pad.volume_db);
    }
    if pad.group > 0 {
        second = format!("{second} mute {}", pad.group);
    }
    // `✱`: the pad has more settings, the panel names them (77); the frame stays clean
    if !pad.glyphs.is_empty() {
        second = format!("{second} {}", pad.glyphs);
    }
    let second = match (pad.missing, pad.loading, &pad.note) {
        (_, Some(p), _) => Line::from(format!(" load {p}%")).fg(CYAN),
        (true, None, _) => Line::from(" missing").fg(MAGENTA),
        (_, _, Some(n)) if n.contains(" → ") => Line::from(learn_spans(&format!(" {n}"))),
        (_, _, Some(n)) => Line::from(format!(" {n}")).fg(if pad.flash { BG } else { DIM }),
        _ => Line::from(format!(" {second}")).fg(if pad.flash { BG } else { DIM }),
    };
    let width = area.width.saturating_sub(3) as usize;
    let name = if pad.naming {
        let mut chars = Vec::new();
        let mut used = usize::from(width > 0);
        for c in pad.name.chars().rev() {
            let n = Span::raw(c.to_string()).width();
            if used + n > width {
                break;
            }
            chars.push(c);
            used += n;
        }
        let mut text = chars.into_iter().rev().collect::<String>();
        if width > 0 {
            text.push('▏');
        }
        text
    } else {
        pad.name.clone()
    };
    let text = vec![Line::from(format!(" {name}")), second];
    frame.render_widget(Paragraph::new(text).block(block), area);
    // the wave sits between the mode line and the bottom frame (29); the selected pad's
    // wave wears the pad's text colour, the rest is dim
    if waves > 0 && !pad.wave.is_empty() {
        let color = if selected || pad.flash { fg } else { DIM };
        let cols = area.width.saturating_sub(2) as usize;
        let rows = braille_abs(&pad.wave, cols, waves as usize);
        for (w, row) in rows.iter().enumerate() {
            frame.render_widget(
                Line::from(Span::styled(row.clone(), Style::new().fg(color))),
                Rect {
                    x: area.x + 1,
                    y: area.y + PAD_H - 1 + w as u16,
                    width: area.width - 2,
                    height: 1,
                },
            );
        }
    }
}

/// The project picker of Ctrl+P (11), drawn over the whole body in place of the screen
/// (as [`PathPickerView`] takes the rows of the Paths tab in place of theirs).
fn fx(frame: &mut Frame, area: Rect, f: &Fx, s: &Status) {
    let [header, body] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
    let mut heading = title(
        vec![Span::styled(" F4 FX ", Style::new().fg(BG).bg(TEXT))],
        &s.badges,
        header.width,
        24,
    );
    heading.spans.push("· shared project effects".fg(DIM));
    frame.render_widget(heading, header);
    let label_width = f
        .blocks
        .iter()
        .flat_map(|block| &block.rows)
        .map(|(label, _)| Line::from(label.trim()).width())
        .max()
        .unwrap_or(0);
    let widths: [u16; 2] = std::array::from_fn(|i| {
        f.blocks[i]
            .rows
            .iter()
            .enumerate()
            .map(|(param, (label, value))| {
                let mut line = Line::from(format!(
                    " {} {:label_width$} {value}",
                    param + 1,
                    label.trim(),
                ));
                if i == f.selected.min(1)
                    && let Some((_, control)) = f.learn.iter().find(|(p, _)| *p == param)
                {
                    line.spans.push(Span::raw(format!(" ◀ {control}")));
                }
                (line.width() + 2).min(u16::MAX as usize) as u16
            })
            .max()
            .unwrap_or(0)
            .max(34)
    });
    let beside = widths[0].saturating_add(widths[1]).saturating_add(2) <= body.width;
    let framed = body.height >= if beside { 5 } else { 10 };
    let height = if framed {
        5
    } else if beside {
        body.height.min(3)
    } else {
        (body.height / 2).min(3)
    };
    let gap = u16::from(!beside && framed && body.height >= 11);
    for (i, view) in f.blocks.iter().enumerate() {
        let area = Rect::new(
            body.x + if beside && i == 1 { widths[0] + 2 } else { 0 },
            body.y + if beside { 0 } else { i as u16 * (height + gap) },
            if framed || beside {
                widths[i].min(body.width)
            } else {
                body.width.min(widths[i].saturating_add(6))
            },
            height,
        );
        let selected = i == f.selected.min(1);
        let color = if selected { MAGENTA } else { BORDER };
        let heading = format!(
            "{} FX {}{} ",
            if selected { "▶" } else { " " },
            i + 1,
            if selected { " · selected" } else { "" },
        );
        let text_area = if framed {
            let block = Block::bordered()
                .border_style(Style::new().fg(color))
                .title(Span::styled(heading, Style::new().fg(color).bold()));
            let inner = block.inner(area);
            frame.render_widget(block, area);
            inner
        } else {
            area
        };
        let mut lines = Vec::new();
        for (param, (label, value)) in view.rows.iter().enumerate() {
            let style = if selected && f.lit == Some(param) {
                Style::new().fg(BG).bg(CYAN).bold()
            } else if selected && f.param.min(2) == param {
                Style::new().fg(CYAN).bold()
            } else {
                Style::new().fg(TEXT)
            };
            let mut spans = Vec::new();
            if framed {
                spans.push(Span::raw(" "));
            } else {
                spans.push(if param == 0 {
                    Span::styled(
                        format!("{} FX {} ", if selected { "▶" } else { " " }, i + 1),
                        Style::new().fg(color).bold(),
                    )
                } else {
                    "       ".into()
                });
            }
            spans.push(Span::styled(
                format!("{} ", param + 1),
                Style::new().fg(if selected { MAGENTA } else { DIM }),
            ));
            spans.push(Span::styled(
                format!("{:label_width$} ", label.trim()),
                style,
            ));
            spans.push(Span::styled(value.clone(), style));
            if selected && let Some((_, control)) = f.learn.iter().find(|(p, _)| *p == param) {
                spans.extend(learn_spans(&format!(" ◀ {control}")));
            }
            lines.push(Line::from(spans));
        }
        frame.render_widget(Paragraph::new(lines), text_area);
    }
}

fn project_picker(p: &ProjectPickerView, s: &Status, width: u16) -> Paragraph<'static> {
    let mut lines = vec![title(
        vec![Span::styled(" PROJECTS ", Style::new().fg(BG).bg(TEXT))],
        &s.badges,
        width,
        0,
    )];
    if p.entries.is_empty() {
        lines.push("no projects yet — type a name".fg(DIM).into());
    }
    for (i, name) in p.entries.iter().enumerate() {
        let row = format!("   {name}");
        lines.push(Line::from(if i == p.selected {
            Span::styled(row, Style::new().fg(BG).bg(CYAN))
        } else {
            row.into()
        }));
    }
    lines.push(Line::default());
    match &p.naming {
        Some(name) => {
            lines.push(format!("name: {name}").into());
            lines.push(" Enter create · Esc cancel".fg(DIM).into());
        }
        None => {
            lines.push(
                " Enter open · type a name to create · Esc close"
                    .fg(DIM)
                    .into(),
            );
        }
    }
    Paragraph::new(lines)
}

fn settings(frame: &mut Frame, area: Rect, st: &Settings, s: &Status) {
    let [header, _, body] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);
    let tab = |i: usize, name: &str| {
        if st.tab == i {
            Span::styled(format!("[{name}]"), Style::new().fg(CYAN).bold())
        } else {
            format!(" {name} ").fg(DIM)
        }
    };
    let names = ["Audio", "Input", "MIDI", "Paths"];
    let mut heading = title(
        vec![Span::styled(" F12 SETTINGS ", Style::new().fg(BG).bg(TEXT))],
        &s.badges,
        header.width,
        11,
    );
    let mut tabs = vec![Span::raw("   ")];
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            tabs.push(" ".into());
        }
        tabs.push(tab(i, name));
    }
    let room = (header.width as usize).saturating_sub(heading.width());
    if tabs.iter().map(Span::width).sum::<usize>() > room {
        tabs = vec![
            "   ".into(),
            tab(st.tab, names.get(st.tab).copied().unwrap_or("Settings")),
        ];
    }
    heading.spans.extend(tabs);
    frame.render_widget(heading, header);
    frame.render_widget(settings_body(st, body.height), body);
}

fn settings_body(st: &Settings, height: u16) -> Paragraph<'static> {
    let label = |row: usize, name: &str| {
        let text = format!(" {name:<8}");
        if st.row == row {
            Span::styled(text, Style::new().fg(BG).bg(CYAN))
        } else {
            text.fg(DIM)
        }
    };
    let device = |actual: Option<&str>, missing: Option<&str>| {
        let mut spans = vec![Span::from(format!(" ◀ {} ▶", actual.unwrap_or("none")))];
        if let Some(m) = missing {
            spans.push(format!("   {m} not found").fg(MAGENTA));
        }
        spans
    };

    let mut lines = Vec::new();
    if st.tab == 3 {
        if let Some(p) = &st.picker {
            lines.push(Line::from(Span::styled(
                p.title,
                Style::new().fg(BG).bg(TEXT),
            )));
            for (i, name) in p.entries.iter().enumerate() {
                let row = format!("   {name}");
                lines.push(Line::from(if i == p.selected {
                    Span::styled(row, Style::new().fg(BG).bg(CYAN))
                } else {
                    row.into()
                }));
            }
            lines.push(Line::default());
            lines.push(" Enter pick · Esc cancel".fg(DIM).into());
            return Paragraph::new(lines);
        }
        if let Some((projects, samples)) = st.paths {
            lines.push(Line::from(vec![
                label(0, "Projects root"),
                Span::from(format!(" {projects}")),
            ]));
            lines.push(Line::from(vec![
                label(1, "Samples root"),
                Span::from(format!(" {samples}")),
            ]));
        }
        return Paragraph::new(lines);
    }
    if st.tab == 2 {
        lines.push(match st.learning {
            Some(l) => Span::styled(l.to_string(), Style::new().fg(MAGENTA)).into(),
            None => " Enter: learn a control for the action. Backspace: clear the learned one."
                .fg(DIM)
                .into(),
        });
        let width = st
            .midi_rows
            .iter()
            .map(|r| match r {
                MidiRow::Action { label, .. } => label.len(),
                MidiRow::Group { .. } => 0,
            })
            .max()
            .unwrap_or(0);
        let mut selected_line = 0;
        for (i, row) in st.midi_rows.iter().enumerate() {
            match row {
                MidiRow::Group { name, counts } => {
                    lines.push(Line::default());
                    let row = format!(" {name} · {counts}");
                    lines.push(Line::from(if i == st.midi_row {
                        selected_line = lines.len();
                        Span::styled(row, Style::new().fg(BG).bg(CYAN))
                    } else {
                        row.bold()
                    }));
                }
                MidiRow::Action {
                    label,
                    who,
                    taken,
                    stays,
                } => {
                    let label = format!("   {label:<width$} ");
                    let label = match (i == st.midi_row, taken) {
                        (true, _) => {
                            selected_line = lines.len();
                            Span::styled(label, Style::new().fg(BG).bg(CYAN))
                        }
                        (false, true) => label.fg(MAGENTA),
                        (false, false) => label.fg(DIM),
                    };
                    let who = format!("  {who}");
                    let mut spans = vec![label];
                    match (taken, who.contains(" → ")) {
                        (true, _) => spans.push(Span::styled(who, Style::new().fg(MAGENTA).bold())),
                        (false, true) => spans.extend(learn_spans(&who)),
                        (false, false) => spans.push(who.into()),
                    }
                    lines.push(Line::from(spans));
                    let pad = " ".repeat(width + 6);
                    lines.extend(
                        stays
                            .iter()
                            .map(|s| Line::from(format!("{pad}{s}").fg(DIM))),
                    );
                }
            }
        }
        // scroll only when the selected row would leave the screen
        let offset = (selected_line as u16 + 2).saturating_sub(height);
        return Paragraph::new(lines).scroll((offset, 0));
    }
    if st.tab == 1 {
        lines.push(" MIDI devices".fg(DIM).into());
        if st.midi_devices.is_empty() {
            lines.push("   none connected".into());
        }
        lines.extend(
            st.midi_devices
                .iter()
                .map(|d| Line::from(format!("   {d}"))),
        );
        lines.push(Line::default());
        lines.push(" Latest input".fg(DIM).into());
        lines.extend(st.input_log.iter().map(|l| Line::from(format!("   {l}"))));
        return Paragraph::new(lines);
    }
    if st.routing {
        let mut out = vec![label(0, "Output")];
        out.extend(device(st.output, st.output_missing));
        let mut inp = vec![label(1, "Input")];
        inp.extend(device(st.input, st.input_missing));
        lines.push(Line::from(out));
        lines.push(Line::from(inp));
        lines.push(Line::from(vec![
            " ".repeat(10).into(),
            meter(st.input_level),
        ]));
    } else {
        let msg = if st.backend_jack {
            " JACK is not available right now."
        } else {
            " Device choice needs JACK: set backend = \"jack\" in the config."
        };
        lines.push(msg.fg(MAGENTA).into());
        lines.push(Line::default());
        lines.push(Line::default());
    }
    let shown = if st.buffer_choice == 0 {
        format!("auto ({})", st.buffer_size)
    } else {
        format!("{}", st.buffer_choice)
    };
    lines.push(Line::from(vec![
        label(2, "Buffer"),
        Span::from(format!(" ◀ {shown} ▶")),
    ]));
    lines.push(
        " auto: follow the server — smaller if pads feel late, bigger if clicks"
            .fg(DIM)
            .into(),
    );
    lines.push(Line::from(vec![
        label(3, "Wave"),
        Span::from(format!(
            " ◀ {} ▶",
            if st.graphics { "picture" } else { "Braille" }
        )),
    ]));
    lines.push(
        " picture: kitty graphics, from the next start; Braille is lighter"
            .fg(DIM)
            .into(),
    );
    Paragraph::new(lines)
}

fn files(frame: &mut Frame, area: Rect, f: &Files, s: &Status) {
    let pf = pad_frame(area, f.pads.hidden);
    // the title carries what the screen says: the file in hand or the error (31)
    let note = match (f.in_hand, f.error) {
        (Some(name), _) => format!(
            "   In hand: {name}. Arrows choose a pad, Enter puts it there (or hit a pad). Esc puts it back."
        )
        .fg(MAGENTA),
        (None, Some(e)) => format!("   {e}").fg(MAGENTA),
        (None, None) => "".into(),
    };
    let reserve = note.width().min(20);
    let mut heading = title(
        vec![Span::styled(" F6 FILES ", Style::new().fg(BG).bg(TEXT))],
        &s.badges,
        pf.title.width,
        reserve,
    );
    heading.spans.push(note);
    frame.render_widget(heading, pf.title);
    // the wave of the selected file over the whole strip, its length on the first row
    if let Some((secs, wave)) = f.preview {
        // the file the wave is of, next to its length (66)
        let label = match f.entries.get(f.selected) {
            Some((name, false, _)) => format!(" {name} · {secs:.2} s "),
            _ => format!(" {secs:.2} s "),
        };
        // the wave over the whole strip, the label over the left end of its top row: a
        // long file name beside the wave pushed every row right
        let area = pf.strip;
        let cols = area.width as usize;
        if cols >= 2 && area.height > 0 {
            let rows = braille(wave, cols, area.height as usize);
            let lines: Vec<Line> = rows
                .iter()
                .map(|r| Line::from(r.clone().fg(CYAN)))
                .collect();
            frame.render_widget(Paragraph::new(lines), area);
            let width = (label.chars().count() as u16).min(area.width);
            frame.render_widget(
                Line::from(Span::styled(label, Style::new().fg(CYAN).bg(BG))),
                Rect {
                    width,
                    height: 1,
                    ..area
                },
            );
            // the thin green line of the chop playhead, over every row of the wave
            if let Some(at) = f.playhead {
                let x = area.x + ((at.clamp(0.0, 1.0) * cols as f32) as u16).min(cols as u16 - 1);
                for y in area.y..area.bottom() {
                    if let Some(cell) = frame.buffer_mut().cell_mut((x, y)) {
                        cell.set_symbol("│").set_fg(METER);
                    }
                }
            }
        }
    }

    let bank_spans = if f.in_hand.is_some() {
        vec![
            Span::styled(" hit a pad ↓ ", Style::new().fg(BG).bg(MAGENTA)),
            " ".into(),
        ]
    } else {
        Vec::new()
    };
    if f.pads.hidden {
        frame.render_widget(pad_line(&f.pads, bank_spans, true), pf.bank);
    } else {
        frame.render_widget(Line::from([bank_spans, banks(&f.pads)].concat()), pf.bank);
        pad_grid(frame, pf.grid, &f.pads, pf.waves);
    }

    // the folder and the filter stand right above the list they name
    let [path, list] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(pf.side);
    frame.render_widget(
        // the filter before the folder: a long path does not push it out of sight (66)
        Line::from(vec![
            match (f.editing, f.filter.is_empty()) {
                (true, _) => Span::styled(
                    format!(" filter: {}▌ ", f.filter),
                    Style::new().fg(BG).bg(CYAN),
                ),
                (false, false) => format!(" filter: {} · Esc clears  ", f.filter).fg(CYAN),
                (false, true) => "".into(),
            },
            format!(" {}", f.dir).fg(DIM),
        ]),
        path,
    );

    let mut lines = Vec::new();
    if f.entries.is_empty() {
        let why = if f.filter.is_empty() {
            "here"
        } else {
            "matching the filter"
        };
        lines.push(Line::from(format!(" no folders or audio files {why}")).fg(DIM));
    }
    for (i, (name, is_dir, favorite)) in f.entries.iter().enumerate() {
        let in_hand = !is_dir && f.in_hand == Some(*name);
        let text = match (*is_dir, in_hand) {
            (true, _) if *favorite => format!(" ★ {name}/ "),
            (true, _) => format!(" {name}/ "),
            (false, true) => format!(" {name}  ← in hand "),
            (false, false) => format!(" {name} "),
        };
        lines.push(match (i == f.selected, in_hand) {
            (_, true) => Line::from(Span::styled(text, Style::new().fg(BG).bg(MAGENTA))),
            (true, false) => Line::from(Span::styled(text, Style::new().fg(BG).bg(CYAN))),
            (false, false) if *is_dir => Line::from(text).fg(TEXT),
            (false, false) => Line::from(text).fg(DIM),
        });
    }
    // keep the selection on screen
    let offset = (f.selected as u16 + 1).saturating_sub(list.height);
    frame.render_widget(Paragraph::new(lines).scroll((offset, 0)), list);
}

/// Waveform in Braille characters: each cell is 2 dot columns by 4 dot rows; every dot
/// column shows the peak of its stretch of `wave`, mirrored around the middle and
/// scaled to the wave's own peak.
pub fn braille(wave: &[f32], cols: usize, rows: usize) -> Vec<String> {
    let top = wave.iter().fold(0f32, |m, x| m.max(x.abs())).max(1e-6);
    braille_scaled(wave, cols, rows, top)
}

/// Braille at a fixed full scale: a quiet sound draws low, a loud one tall.
pub fn braille_abs(wave: &[f32], cols: usize, rows: usize) -> Vec<String> {
    braille_scaled(wave, cols, rows, 1.0)
}

fn braille_scaled(wave: &[f32], cols: usize, rows: usize, top: f32) -> Vec<String> {
    let (dx, dy) = (cols * 2, rows * 4);
    let mut cells = vec![vec![0u8; cols]; rows];
    if wave.is_empty() || dx == 0 || dy == 0 {
        return cells.iter().map(|r| " ".repeat(r.len())).collect();
    }
    // dot bits of a Braille cell, by (column, row)
    const BITS: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
    for (x, &peak) in dot_peaks(wave, dx, top).iter().enumerate() {
        // at least the middle dot, so silence still reads as a line
        let half = ((peak * dy as f32 / 2.0).round() as usize).max(1);
        let mid = dy / 2;
        for y in mid.saturating_sub(half)..(mid + half).min(dy) {
            cells[y / 4][x / 2] |= BITS[x % 2][y % 4];
        }
    }
    cells
        .iter()
        .map(|r| {
            r.iter()
                .map(|&bits| char::from_u32(0x2800 + bits as u32).unwrap_or(' '))
                .collect()
        })
        .collect()
}

/// The peak of every dot column (two per cell) of `wave`, scaled by `top`: the shape the
/// Braille wave and its kitty picture both draw.
fn dot_peaks(wave: &[f32], dots: usize, top: f32) -> Vec<f32> {
    if wave.is_empty() || dots == 0 {
        return Vec::new();
    }
    (0..dots)
        .map(|x| {
            let (a, b) = (
                x * wave.len() / dots,
                ((x + 1) * wave.len() / dots).max(x * wave.len() / dots + 1),
            );
            wave[a..b.min(wave.len())]
                .iter()
                .fold(0f32, |m, v| m.max(v.abs()))
                / top
        })
        .collect()
}

fn chop(
    frame: &mut Frame,
    area: Rect,
    c: &Chop,
    s: &Status,
    mut images: Option<&mut Vec<WaveImage>>,
) {
    let pf = pad_frame(area, c.pads.hidden);
    let header = pf.title;
    // the marks over the wave are the top two rows of the strip (31)
    let [numbers, play_bar, wave] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(pf.strip);
    let slices = c.bounds.len().saturating_sub(1);
    let playing = c.mode == "LISTEN ▶";
    let badge = if playing {
        " LISTEN ▶ "
    } else {
        " LISTEN   "
    };
    let badge_style = if playing {
        Style::new().fg(BG).bg(CYAN)
    } else {
        Style::new().fg(DIM).bg(BG)
    };
    let slice_count = format!(" · {slices} slice{}", if slices == 1 { "" } else { "s" });
    let variant = c
        .which
        .map_or_else(String::new, |(i, n)| format!(" · chop {i} of {n}"));
    let suffix_width = Line::from(format!("{variant}{slice_count}")).width();
    let fixed = vec![
        Span::styled(" F3 CHOP ", Style::new().fg(BG).bg(TEXT)),
        " ".into(),
        Span::styled(badge, badge_style),
    ];
    let fixed_width = fixed.iter().map(Span::width).sum::<usize>();
    let mode_width = s
        .badges
        .first()
        .map_or(0, |b| Line::from(b.compact.as_str()).width() + 3);
    // Keep the variant and count. Reduce the filename budget before hiding the active mode.
    let file_reserve = (header.width as usize)
        .saturating_sub(fixed_width + mode_width + suffix_width + 3)
        .min(12);
    let mut heading = title(
        fixed,
        &s.badges,
        header.width,
        suffix_width + 3 + file_reserve,
    );
    let details = Line::from(vec![
        format!("   cuts of {}", c.file).into(),
        variant.clone().into(),
        format!("   {:.2} s{slice_count}", c.seconds).fg(DIM),
    ]);
    let room = (header.width as usize).saturating_sub(heading.width());
    if details.width() <= room {
        heading.spans.extend(details.spans);
    } else {
        let file_width = room.saturating_sub(suffix_width + 3);
        heading
            .spans
            .push(format!("   {}", title_detail(c.file, file_width)).into());
        heading.spans.push(variant.into());
        heading.spans.push(slice_count.fg(DIM));
    }
    frame.render_widget(heading, header);
    let width = wave.width as usize;
    let col = |at: f32| ((at * width as f32) as usize).min(width.saturating_sub(1));
    let (first, last) = (
        c.bounds.first().copied().unwrap_or(0.0),
        c.bounds.last().copied().unwrap_or(1.0),
    );

    // where every slice lies (36): a slice on a pad wears a plate in its bank color across
    // its width with the number and the pads (`3 2-Q`), a gap of one cell before the next;
    // a slice on no pad shows its number and a dim `○`; the cursor is the "!" above it all
    let mut nums: Vec<(char, Color, Option<Color>)> = vec![(' ', DIM, None); width];
    for (i, &start) in c.bounds.iter().take(slices).enumerate() {
        let (from, to) = (col(start), col(c.bounds[i + 1]));
        let room = to.saturating_sub(from);
        let number = (i + 1).to_string();
        match c.owner_bank.get(i).copied().flatten() {
            Some(b) => {
                let plate = BANK_COLORS[b % BANK_COLORS.len()];
                let pads = c.owners.get(i).cloned().unwrap_or_default();
                let text: Vec<char> = format!("{number} {pads}").chars().collect();
                let end = (from + room.saturating_sub(1)).max(from + 1).min(width);
                for (k, cell) in nums.iter_mut().take(end).skip(from).enumerate() {
                    *cell = (text.get(k).copied().unwrap_or(' '), BG, Some(plate));
                }
            }
            None => {
                let label: Vec<(char, Color, Option<Color>)> = number
                    .chars()
                    .map(|ch| (ch, DIM, None))
                    .chain((number.len() < room).then_some(('○', BORDER, None)))
                    .collect();
                for (k, cell) in label.into_iter().enumerate() {
                    if let Some(at) = nums.get_mut(from + k) {
                        *at = cell;
                    }
                }
            }
        }
    }
    if let Some(at) = c.cursor
        && let Some(cell) = nums.get_mut(((at * width as f32) as usize).min(width - 1))
    {
        cell.0 = '!';
        if cell.2.is_none() {
            cell.1 = DIM;
        }
    }
    frame.render_widget(
        Line::from(
            nums.into_iter()
                .map(|(ch, fg, bg)| {
                    let style = Style::new().fg(fg);
                    Span::styled(ch.to_string(), bg.map_or(style, |b| style.bg(b)))
                })
                .collect::<Vec<_>>(),
        ),
        numbers,
    );

    // what play covers now: a green bar right above the wave; the hits of the hits mode
    // stand on the same row as ▾ (51)
    let mut bar: Vec<Span> = (0..width).map(|_| Span::raw(" ")).collect();
    if let Some((from, to)) = c.play {
        // a span narrower than a cell still shows one
        let (first, last) = (col(from), col(to.max(from)).max(col(from)));
        for (x, cell) in bar.iter_mut().enumerate() {
            if (first..=last).contains(&x) && (x < last || last == first || to >= 1.0) {
                *cell = Span::styled("━", Style::new().fg(METER));
            }
        }
    }
    for &h in &c.hits {
        if let Some(cell) = bar.get_mut(col(h)) {
            *cell = Span::styled("▾", Style::new().fg(TEXT).bold());
        }
    }
    frame.render_widget(Line::from(bar), play_bar);

    let sel = (
        c.bounds.get(c.selected).copied().unwrap_or(0.0),
        c.bounds.get(c.selected + 1).copied().unwrap_or(1.0),
    );
    // the wave: Braille cells, or blank cells with the picture described for the
    // terminal to draw under the marks (04b)
    // the color tells what plays (38): the band above its average takes the column; every
    // slice of the region is at full color, the selection is a lighter background below
    let at_color = |x: usize| {
        let at = (x as f32 + 0.5) / width.max(1) as f32;
        let (plain, bright) = if at < first || at > last {
            (BORDER, 0.35)
        } else if at >= sel.0 && at < sel.1 {
            (TEXT, 1.0)
        } else {
            (DIM, 1.0)
        };
        if c.bands.is_empty() {
            return plain;
        }
        let b = c.bands[(x * c.bands.len() / width.max(1)).min(c.bands.len() - 1)];
        // silence has no color to tell: its flat line keeps the plain tone
        if b.iter().sum::<f32>() < 1e-4 {
            return plain;
        }
        let color = band_color(b, c.band_mean, bright);
        // the selected slice is the one in color; the rest is grey at its brightness (40)
        if at >= sel.0 && at < sel.1 && at >= first && at <= last {
            color
        } else {
            grey(color)
        }
    };
    let graphics = images.is_some();
    if let Some(out) = images.as_mut() {
        let top = c.wave.iter().fold(0f32, |m, x| m.max(x.abs())).max(1e-6);
        out.push(WaveImage {
            id: 1,
            area: wave,
            peaks: dot_peaks(c.wave, width * 2, top),
            colors: (0..width).map(at_color).collect(),
        });
    }
    let rows = if graphics {
        vec![" ".repeat(width); wave.height as usize]
    } else {
        braille(c.wave, width, wave.height as usize)
    };
    let mut lines: Vec<Vec<Span>> = rows
        .iter()
        .map(|r| {
            r.chars()
                .enumerate()
                .map(|(x, ch)| Span::from(ch.to_string()).fg(at_color(x)))
                .collect()
        })
        .collect();
    let mut mark = |at: f32, ch: &'static str, color: Color| {
        for line in lines.iter_mut() {
            if let Some(span) = line.get_mut(col(at)) {
                *span = Span::styled(ch, Style::new().fg(color).bold());
            }
        }
    };
    for (k, &b) in c.bounds.iter().enumerate() {
        let edge = k == 0 || k == slices;
        // the range edges are drawn only when trimmed
        if edge && (b <= 0.0 || b >= 1.0) && k != c.selected && k != c.selected + 1 {
            continue;
        }
        if k == c.selected || k == c.selected + 1 {
            // the bound in play differs by colour, not by weight
            let hot = c.hot == Some(k == c.selected + 1);
            mark(b, "│", if hot { AMBER } else { MAGENTA });
        } else {
            mark(b, "│", CYAN);
        }
    }
    // the playhead is a thin line of its own; the owner's cursor is drawn over it
    if let Some(at) = c.playhead {
        mark(at, "│", METER);
    }
    // the cursor on the lit bound wears its colour: the bound must not hide under it
    let hot_at = c
        .hot
        .and_then(|end| c.bounds.get(c.selected + usize::from(end)).copied());
    if let Some(at) = c.cursor {
        let on_hot = hot_at.is_some_and(|h| col(h) == col(at));
        mark(at, "┃", if on_hot { AMBER } else { WHITE });
    }
    frame.render_widget(
        Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>()),
        wave,
    );
    if let Some(marks) = c.banks {
        let mut spans = vec!["bank ".fg(DIM)];
        for (i, (taken, planned)) in marks.iter().enumerate() {
            let glyphs = bank_glyphs(c.pads.bank_fill[i]);
            // the bank of the source wears a ◆ (37)
            let src = if c.source_banks[i] { "◆" } else { "" };
            let text = if i == c.pads.bank {
                format!("[{}{src} {glyphs}]", i + 1)
            } else {
                format!(" {}{src} {glyphs} ", i + 1)
            };
            let color = match (planned, taken) {
                (true, _) => MAGENTA,
                (false, true) => TEXT,
                (false, false) => DIM,
            };
            let style = Style::new().fg(color);
            spans.push(Span::styled(
                text,
                if i == c.pads.bank {
                    style.bold()
                } else {
                    style
                },
            ));
        }
        frame.render_widget(Line::from(spans), pf.bank);
    }
    if let Some((a, b)) = c.slice_keys {
        frame.render_widget(
            Line::from(vec![
                Span::styled(format!(" slices {a}–{b} "), Style::new().fg(BG).bg(CYAN)),
                "  the keys play the slices; once one lies on a pad, the grid shows the pads"
                    .fg(DIM),
            ]),
            pf.bank,
        );
    }
    if c.pads.hidden {
        // the listen mode has no bank row: the line names the pad and its slice
        frame.render_widget(pad_line(&c.pads, Vec::new(), false), pf.bank);
    } else {
        pad_grid(frame, pf.grid, &c.pads, pf.waves);
    }

    // the right column: the slice line on top, wrapped with the side list, not cut
    let mut slice_line = Vec::new();
    if c.edit {
        slice_line.push(Span::styled(
            " EDIT · BOUNDS ",
            Style::new().fg(BG).bg(MAGENTA).bold(),
        ));
    }
    slice_line.push(format!(" {}", c.info).fg(CYAN));
    let lines: Vec<Line> = std::iter::once(Line::from(slice_line))
        .chain(c.side.iter().enumerate().map(|(i, l)| {
            let line = Line::from(l.clone());
            if i == 0 { line.fg(CYAN) } else { line.fg(DIM) }
        }))
        .collect();
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), pf.side);
}

/// A color without its hue, at its brightness: the slices that are not selected (40).
fn grey(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => {
            let l = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32).round() as u8;
            Color::Rgb(l, l, l)
        }
        c => c,
    }
}

/// The tones of the three bands (38): the low red, the mid green, the high blue — the
/// owner asked for contrast after the softer palette merged into grey.
const BAND_TONES: [(f32, f32, f32); 3] = [
    (255.0, 70.0, 60.0),
    (80.0, 225.0, 100.0),
    (80.0, 130.0, 255.0),
];

/// A wave column colored by the band that stands out against its average over the file,
/// at `bright` from the background (1.0 full, less steps back). Each band is read against
/// its mean, and the shares go to the 4th power: a break whose bands are nearly even on
/// average still shows its kicks red and its hats blue, not a grey mix.
fn band_color(b: [f32; 3], mean: [f32; 3], bright: f32) -> Color {
    let rel = [0, 1, 2].map(|i| (b[i] / mean[i].max(1e-6)).powi(4));
    let total = rel.iter().sum::<f32>().max(1e-9);
    let mut rgb = [0.0f32; 3];
    for (level, tone) in rel.iter().zip(BAND_TONES) {
        let w = level / total;
        rgb[0] += w * tone.0;
        rgb[1] += w * tone.1;
        rgb[2] += w * tone.2;
    }
    let bg = [7.0f32, 8.0, 10.0];
    let ch = |i: usize| {
        (bg[i] + (rgb[i] - bg[i]) * bright)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color::Rgb(ch(0), ch(1), ch(2))
}

/// Input level: -60..0 dB over 30 cells.
fn meter(level: f32) -> Span<'static> {
    const WIDTH: usize = 30;
    let db = 20.0 * level.max(1e-6).log10();
    let filled = (((db + 60.0) / 60.0).clamp(0.0, 1.0) * WIDTH as f32).round() as usize;
    let text = format!(
        "{}{} {db:>5.0} dB",
        "█".repeat(filled),
        "░".repeat(WIDTH - filled)
    );
    if level >= 1.0 {
        text.fg(MAGENTA)
    } else {
        text.fg(CYAN)
    }
}

/// The status marker of a terminal without key releases, in the owner's words (68).
const NO_RELEASES: &str = " · no key releases: gate pads play one-shot";

fn status_line(s: &Status) -> Line<'static> {
    let Some(backend) = s.backend else {
        let mut line = s.save.spans();
        if s.recording {
            line.push(" ● rec ".fg(MAGENTA).bold());
        }
        line.push(" ○ no audio".fg(MAGENTA));
        if s.no_releases {
            line.push(NO_RELEASES.fg(DIM));
        }
        if let Some(name) = s.project {
            line.push(Span::from(format!(" · {name}")));
        }
        return Line::from(line);
    };
    let ms = s.buffer_size as f32 / s.sample_rate.max(1) as f32 * 1000.0;
    let size = if s.buffer_size == 0 {
        "auto".to_string()
    } else {
        s.buffer_size.to_string()
    };
    let xruns = Span::from(format!("xruns {}", s.xruns));
    let mut spans = s.save.spans();
    if s.recording {
        spans.push(" ● rec ".fg(MAGENTA).bold());
    }
    spans.push(" ● ".fg(CYAN));
    spans.push(Span::from(format!(
        "{backend} {size} @ {} Hz · {ms:.1} ms · CPU {:.0} % · master {:+} dB · ",
        s.sample_rate, s.load_peak, s.master_db
    )));
    spans.push(if s.xruns > 0 {
        xruns.fg(MAGENTA).bold()
    } else {
        xruns
    });
    if s.no_releases {
        spans.push(NO_RELEASES.fg(DIM));
    }
    if let Some(name) = s.project {
        spans.push(Span::from(format!(" · {name}")));
    }
    Line::from(spans)
}

/// A key cap: bright text on the border gray, so the key stands apart from what it does
/// without shouting.
fn key(k: &str) -> Span<'static> {
    Span::styled(format!(" {k} "), Style::new().fg(BRIGHT).bg(BORDER))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Modifier;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn pads_screen_shows_grid_bank_and_selection() {
        let keys = [
            '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v',
        ];
        let pads = Pads {
            bank: 1,
            pads: keys
                .iter()
                .enumerate()
                .map(|(i, &key)| PadView {
                    naming: false,
                    glyphs: String::new(),
                    changed: Vec::new(),
                    dim: false,
                    dashed: false,
                    source: String::new(),
                    bounds: (0.0, 0.0),
                    filter: Default::default(),
                    wave: Vec::new(),
                    key,
                    name: format!("N{i}"),
                    gate: i == 5,
                    volume_db: if i == 5 { -3.0 } else { 0.0 },
                    pitch: 0,
                    attack_ms: 0,
                    decay_ms: 0,
                    release_ms: 0,
                    group: 0,
                    flash: false,
                    missing: i == 7,
                    loading: (i == 8).then_some(40),
                    in_hand: false,
                    marked: false,
                    note: None,
                    detail: None,
                })
                .collect(),
            selected: 5,
            pickup: None,
            scope: String::new(),
            param: PadParam::Volume,
            // pads 1 and W of the current bank (2) hold a sound
            playing: false,
            feedback: PadFeedback::default(),
            bank_fill: [0, 0b0000_0000_0010_0001, 0, 0, 0, 0, 0, 0],
            bank_marks: [0; BANKS],
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
        };
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: Some("jack"),
            recording: false,
            buffer_size: 128,
            sample_rate: 48_000,
            load_peak: 1.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0, -60.0], [-60.0, -60.0], false),
            no_releases: false,
            notice: None,
            project: None,
        };
        let groups = [HelpGroup {
            name: "PAD",
            items: vec![("Enter".into(), "mode: gate".into())],
        }];
        let help = Help {
            groups: &groups,
            hint: Some("hint"),
            full: None,
            question: None,
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(100, 26)).unwrap();
        term.draw(|f| draw(f, &status, &Screen::Pads(pads), &help))
            .unwrap();
        let text: String = term
            .backend()
            .buffer()
            .content()
            .chunks(100)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
            .collect();
        println!("{text}");
        for want in [
            "F2 PADS",
            "[2 ⠑",
            "bank 2",
            "N15",
            "pad W",
            "gate",
            "-3 dB",
            " Enter mode: gate",
            "missing",
            "load 40%",
        ] {
            assert!(text.contains(want), "{want} missing");
        }
    }

    /// 73 "The splash at start", the drawing: while no sound went, the wave strip shows
    /// the logo, the name, the version and the loading line instead of the wave; the rest
    /// of the screen shows as always. The logo is byte-exact, trailing spaces kept.
    #[test]
    fn the_splash_takes_the_wave_strip_until_the_first_sound() {
        let (w, h) = (100, 44);
        let f = frame_at(w, h);
        let mut p = idle_pads();
        p.splash = Some(Splash {
            version: "0.1.0",
            loading: "loading the session, 2 samples".into(),
        });
        let rows = render_pads_at(p, w, h);
        let text: Vec<String> = rows
            .iter()
            .map(|r| r.iter().map(|(s, _, _)| s.as_str()).collect())
            .collect();
        // the splash lives inside the strip, above the pad grid
        let grid = f.grid.y as usize;
        let at = text[..grid]
            .iter()
            .rposition(|r| r.contains("ptsr - PT's SampleR"))
            .expect("the name is in the strip");
        assert!(at + 1 < grid, "the name sits above the grid");
        let pad = text[at].find("ptsr").unwrap();
        // the logo stands left of the name, byte-exact, 10 characters a line; its first
        // line is spaces, the block runs down from the name's row
        let logo_x = pad - 2 - 10;
        assert!(text[at][logo_x..logo_x + 10].chars().all(|c| c == ' '));
        for (i, want) in SPLASH_LOGO.iter().enumerate().skip(1) {
            assert_eq!(&text[at + i][logo_x..logo_x + 10], *want, "logo line {i}");
        }
        assert_eq!(&text[at + 1][pad..pad + 5], "0.1.0");
        assert_eq!(
            &text[at + 2][pad..pad + "loading the session, 2 samples".len()],
            "loading the session, 2 samples"
        );
        // no wave draws under the splash: the strip rows hold no braille dots
        for (r, row) in rows
            .iter()
            .enumerate()
            .take(f.strip.bottom() as usize)
            .skip(f.strip.y as usize)
        {
            assert!(
                !row.iter().any(|(s, _, _)| s.as_str() > "\u{2800}"),
                "strip row {r} draws no wave while the splash shows"
            );
        }
        // the first sound ends the splash: with it gone the strip shows the wave again
        let mut p = idle_pads();
        for pad in &mut p.pads {
            pad.wave = vec![1.0; 1200];
        }
        let rows = render_pads_at(p, w, h);
        assert!(
            rows[f.strip.y as usize..f.strip.bottom() as usize]
                .iter()
                .any(|r| r.iter().any(|(s, _, _)| s.as_str() > "\u{2800}")),
            "the strip draws the wave once the splash is gone"
        );
    }

    #[test]
    fn the_pad_edit_takes_the_strip_from_the_splash() {
        let mut p = idle_pads();
        p.splash = Some(Splash {
            version: "0.1.0",
            loading: "ready".into(),
        });
        p.big = Some(BigView {
            wave: vec![0.5; 64],
            env: vec![1.0; 64],
            attack: 0.1,
            decay: 0.0,
            release: 0.5,
        });
        p.edit = Some("ENV");
        let rows = render_pads_at(p, 100, 44);
        let text: String = rows
            .iter()
            .flat_map(|row| row.iter().map(|(s, _, _)| s.as_str()))
            .collect();
        assert!(!text.contains("ptsr - PT's SampleR"));

        let mut p = idle_pads();
        p.splash = Some(Splash {
            version: "0.1.0",
            loading: "ready".into(),
        });
        let rows = render_pads_at(p, 100, 44);
        let text: String = rows
            .iter()
            .flat_map(|row| row.iter().map(|(s, _, _)| s.as_str()))
            .collect();
        assert!(text.contains("ptsr - PT's SampleR"));
    }

    #[test]
    fn a_long_pad_name_while_typing() {
        for width in [64, 100] {
            let mut p = idle_pads();
            p.pads[0].name = "very long 名字名字名字 tail".into();
            p.pads[0].naming = true;
            let f = frame_at(width, 44);
            let rows = render_pads_at(p, width, 44);
            let row = &rows[(f.grid.y + 1) as usize];
            let cell_width = f.grid.width as usize / 4;
            let text: String = row[..cell_width]
                .iter()
                .map(|(s, _, _)| s.as_str())
                .collect();
            assert!(text.contains("tail"), "{text}");
            assert_eq!(row[0].0, "│");
            assert_eq!(row[cell_width - 1].0, "│");
        }
    }

    #[test]
    fn the_pad_name_cursor() {
        for name in ["", "long 名字名字名字 tail"] {
            let mut p = idle_pads();
            p.pads[0].name = name.into();
            p.pads[0].naming = true;
            let f = frame_at(64, 44);
            let rows = render_pads_at(p, 64, 44);
            let row = &rows[(f.grid.y + 1) as usize];
            let width = f.grid.width as usize / 4;
            let text: String = row[..width].iter().map(|(s, _, _)| s.as_str()).collect();
            assert!(text.contains('▏'), "{text}");
            assert_eq!(row[width - 1].0, "│");
        }
        let mut p = idle_pads();
        p.pads[0].name = "kick".into();
        let rows = render_pads_at(p, 64, 44);
        let f = frame_at(64, 44);
        assert!(
            !rows[(f.grid.y + 1) as usize]
                .iter()
                .any(|(s, _, _)| s == "▏")
        );
    }

    /// Scenario "Project picker" (11): the status line names the open project on every
    /// screen, with and without audio; without a project it names none.
    #[test]
    fn the_status_line_names_the_open_project() {
        let status = |backend, project| Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend,
            recording: false,
            buffer_size: 128,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0; 2], [-60.0; 2], false),
            no_releases: false,
            notice: None,
            project,
        };
        let text = |s: &Status| {
            status_line(s)
                .spans
                .iter()
                .map(|sp| sp.content.clone())
                .collect::<String>()
        };
        let with_audio = status(Some("jack"), Some("MyBeat"));
        assert!(text(&with_audio).contains("MyBeat"));
        let without_audio = status(None, Some("MyBeat"));
        assert!(text(&without_audio).contains("MyBeat"));
        let no_project = status(Some("jack"), None);
        assert!(!text(&no_project).contains("MyBeat"));
    }

    /// Scenario "Terminal without the kitty protocol" (22) and 68 "Plain words for no key
    /// releases": the status keeps `no key releases: gate pads play one-shot` for the
    /// session; with the protocol it shows none.
    #[test]
    fn plain_words_for_no_key_releases() {
        let status = |no_releases| Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: Some("jack"),
            recording: false,
            buffer_size: 128,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0; 2], [-60.0; 2], false),
            no_releases,
            notice: None,
            project: None,
        };
        let text = |s: &Status| {
            status_line(s)
                .spans
                .iter()
                .map(|sp| sp.content.clone())
                .collect::<String>()
        };
        assert!(text(&status(true)).contains("no key releases: gate pads play one-shot"));
        assert!(!text(&status(false)).contains("no key releases"));
    }

    /// Scenario "No big view" (29): a pad is selected, nothing is drawn under the info
    /// panel, even with the piece data at hand; the panel keeps its own rows.
    #[test]
    fn no_big_view() {
        let mut p = idle_pads();
        p.big = Some(BigView {
            wave: vec![1.0; 64],
            env: vec![1.0; 64],
            attack: 0.25,
            decay: 0.25,
            release: 0.0,
        });
        let rows = render_pads_at(p, 100, 44);
        // the right column: the meters, a gap, the panel; under it, until the notice row,
        // no cell draws
        let f = frame_at(100, 44);
        let (x, panel) = (
            f.side.x as usize,
            f.side.y as usize + METER_ROWS as usize + 1,
        );
        // a low window cuts the panel at its bottom (56: the FX blocks made it taller)
        let under = (panel + PANEL_ROWS as usize).min(f.side.bottom() as usize);
        assert!(
            rows[panel..under]
                .iter()
                .flat_map(|r| r[x..].iter())
                .any(|(s, _, _)| s != " "),
            "the panel itself is drawn"
        );
        assert!(
            rows[under..f.side.bottom() as usize]
                .iter()
                .all(|r| r[x..].iter().all(|(s, _, _)| s == " ")),
            "nothing is drawn under the panel"
        );
    }

    /// Scenario "Every pad draws its wave" (29): in a terminal tall enough for three wave
    /// rows (44 since 31) every pad with a sample shows its wave in three rows under its
    /// mode line, an empty pad a flat middle line.
    #[test]
    fn every_pad_draws_its_wave() {
        let mut p = idle_pads();
        for (i, pad) in p.pads.iter_mut().enumerate() {
            // pad 2 is empty: a flat line; the rest sound at full scale
            pad.wave = if i == 1 { vec![0.0] } else { vec![1.0; 1200] };
        }
        p.selected = 5;
        let rows = render_pads_at(p, 100, 44);
        let f = frame_at(100, 44);
        assert_eq!(f.waves, 3);
        // a cell is 4+3 rows: frame, name, mode, waves, frame; 15 columns wide at 100
        let top = f.grid.y as usize;
        let has_dots = |r: usize, from: usize, to: usize| {
            rows[r][from..to]
                .iter()
                .any(|(s, _, _)| s.as_str() > "\u{2800}")
        };
        for g in 0..4 {
            let waves_at = top + g * 7 + 3;
            for r in waves_at..waves_at + 3 {
                assert!(has_dots(r, 1, 14), "wave row {r} of pad row {g} draws");
            }
            assert_eq!(
                rows[top + g * 7 + 6][0].0,
                "└",
                "the frame follows the three wave rows"
            );
        }
        // the empty pad: a flat line only in the middle of its three rows
        assert!(
            !has_dots(top + 3, 16, 29) && !has_dots(top + 5, 16, 29),
            "an empty pad draws no dots above and below the middle"
        );
        assert!(
            has_dots(top + 4, 16, 29),
            "the flat line sits in the middle row"
        );
        // the selected pad's wave wears the pad's text colour, the rest are dim
        let color_of = |r: usize, from: usize, to: usize| {
            rows[r][from..to]
                .iter()
                .find(|(s, _, _)| s.as_str() > "\u{2800}")
                .map(|(_, fg, _)| *fg)
        };
        assert_eq!(
            color_of(top + 3, 1, 14),
            Some(DIM),
            "an idle pad's wave is dim"
        );
        assert_eq!(
            color_of(top + 10, 16, 29),
            Some(TEXT),
            "the selected pad's wave is in the text colour"
        );
    }

    /// Scenario "The wave follows the terminal" (29) with "A low terminal" (31): lower the
    /// terminal step by step — the pad waves take three, then two, then one row while the
    /// strip keeps its 8, and the whole grid always fits.
    #[test]
    fn the_wave_follows_the_terminal() {
        let sounding = || {
            let mut p = idle_pads();
            for pad in &mut p.pads {
                pad.wave = vec![1.0; 1200];
            }
            p
        };
        for (h, w) in [(44, 3), (40, 2), (36, 1), (26, 0)] {
            let rows = render_pads_at(sounding(), 100, h);
            let f = frame_at(100, h);
            assert_eq!(f.waves, w as u16, "h {h}");
            if w > 0 {
                assert!(
                    f.strip.height >= STRIP_FULL,
                    "h {h}: the strip keeps its 8 rows"
                );
            }
            // the wave rows sit under the first pad's mode line, then its frame comes
            // right after them
            let top = f.grid.y as usize;
            for (r, row) in rows.iter().enumerate().skip(top + 3).take(w) {
                assert!(
                    row[1..14].iter().any(|(s, _, _)| s.as_str() > "\u{2800}"),
                    "h {h}: wave row {r} draws"
                );
            }
            assert_eq!(
                rows[top + 3 + w][0].0,
                "└",
                "h {h}: the frame follows the {w} wave rows"
            );
            // the whole grid always fits: the rows between its last frame and the
            // notice line stay empty over the grid columns
            let (bottom, end) = (top + 16 + 4 * w, (h - 4) as usize);
            assert!(
                rows[bottom..end]
                    .iter()
                    .all(|r| r[..f.grid.width as usize].iter().all(|(s, _, _)| s == " ")),
                "h {h}: nothing draws under the grid"
            );
        }
    }

    /// Scenario "A low terminal" (31): going lower row by row, the pad waves give way
    /// first, then the strip shrinks down to 3 rows, and the grid always fits.
    #[test]
    fn a_low_terminal() {
        let mut last = frame_at(100, 70);
        for h in (26..70).rev() {
            let f = frame_at(100, h);
            assert!(
                f.waves <= last.waves,
                "h {h}: the waves never come back going lower"
            );
            if f.waves < last.waves {
                assert!(
                    f.strip.height >= STRIP_FULL,
                    "h {h}: the waves go before the strip"
                );
            }
            assert!(f.strip.height >= STRIP_MIN, "h {h}");
            assert!(
                f.grid.height >= PAD_H * (4 + f.waves),
                "h {h}: the whole grid fits"
            );
            last = f;
        }
        assert_eq!(
            frame_at(100, 26).strip.height,
            3,
            "26 rows: the strip at its least"
        );
    }

    /// Scenario "A tall terminal gives the strip the rest" (31).
    #[test]
    fn a_tall_terminal_gives_the_strip_the_rest() {
        let f = frame_at(100, 60);
        assert_eq!(f.waves, 3);
        // 55 body rows: the title, the bank row, the grid of 4 cells of 7 rows, the strip the rest
        assert_eq!(f.strip.height, 55 - 2 - 28);
        assert_eq!(f.strip.width, 100, "over the full width");
    }

    /// Scenario "Short terminal" (29): at 26 rows the pads are drawn without waves, as
    /// before, and the whole grid fits.
    #[test]
    fn short_terminal() {
        let mut p = idle_pads();
        for pad in &mut p.pads {
            pad.wave = vec![1.0; 1200];
        }
        let rows = render_pads_at(p, 100, 26);
        let top = frame_at(100, 26).grid.y as usize;
        // no braille dots anywhere in the grid rows over the grid columns
        assert!(
            rows[top..top + 16]
                .iter()
                .all(|r| r[..60].iter().all(|(s, _, _)| s.as_str() <= "\u{2800}")),
            "no wave draws at 26 rows"
        );
        // and the grid is today's: the cells are 4 rows, 16 rows of grid in total
        assert_eq!(
            rows[top + 3][0].0,
            "└",
            "the first cell ends at its 4th row"
        );
        assert_eq!(
            rows[top + 15][0].0,
            "└",
            "the last cell ends the 16-row grid"
        );
    }

    #[test]
    fn files_screen_shows_pads_next_to_the_list_and_the_file_in_hand() {
        let keys = [
            '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v',
        ];
        let pads = Pads {
            bank: 2,
            pads: keys
                .iter()
                .map(|&key| PadView {
                    naming: false,
                    glyphs: String::new(),
                    changed: Vec::new(),
                    dim: false,
                    dashed: false,
                    source: String::new(),
                    bounds: (0.0, 0.0),
                    filter: Default::default(),
                    wave: Vec::new(),
                    key,
                    name: if key == 'w' {
                        "amen".into()
                    } else {
                        "A2".into()
                    },
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
                    note: None,
                    detail: None,
                })
                .collect(),
            selected: 5,
            pickup: None,
            scope: String::new(),
            param: PadParam::Volume,
            playing: false,
            feedback: PadFeedback::default(),
            bank_fill: [0, 0, 1 << 5, 0, 0, 0, 0, 0],
            bank_marks: [0; BANKS],
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
        };
        let f = Files {
            dir: "/home/user/Downloads",
            entries: vec![
                ("drums", true, true),
                ("breaks", true, false),
                ("amen-loop.wav", false, false),
            ],
            selected: 2,
            in_hand: Some("amen-loop.wav"),
            error: None,
            pads,
            filter: "",
            editing: false,
            preview: None,
            playhead: None,
        };
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: Some("jack"),
            recording: false,
            buffer_size: 256,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0, -60.0], [-60.0, -60.0], false),
            no_releases: false,
            notice: None,
            project: None,
        };
        let groups = [HelpGroup {
            name: "IN HAND",
            items: vec![("1-4 Q-R A-F Z-V".into(), "hit a pad to put it there".into())],
        }];
        let help = Help {
            groups: &groups,
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(110, 26)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Files(f), &help))
            .unwrap();
        let text: String = term
            .backend()
            .buffer()
            .content()
            .chunks(110)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
            .collect();
        println!("{text}");
        for want in [
            "F6 FILES",
            "hit a pad",
            "[3 ⠐",
            "amen",
            "breaks/",
            "★ drums/",
            "amen-loop.wav  ← in hand",
            "IN HAND",
        ] {
            assert!(text.contains(want), "{want} missing");
        }
        // the grid is left of the list on the same rows
        let row = text.lines().find(|l| l.contains("← in hand")).unwrap();
        let border = row.find(['│', '┌', '└']).expect("grid border on the row");
        assert!(border < row.find("← in hand").unwrap());
    }

    /// Scenario "The same frame on every screen" (31): F2, F6 and F3 in the same terminal
    /// put the strip and the pad grid at the same rows and columns.
    #[test]
    fn the_same_frame_on_every_screen() {
        let (status, help) = chop_status();
        let wave: Vec<f32> = (0..4000).map(|i| (i % 1000) as f32 / 1000.0).collect();
        let sounding = || {
            let mut p = idle_pads();
            for pad in &mut p.pads {
                pad.wave = vec![1.0; 1200];
            }
            p
        };
        let mut chop = chop_demo(&wave);
        chop.pads = sounding();
        let screens = [
            Screen::Pads(sounding()),
            Screen::Files(Files {
                dir: "~/Music",
                entries: vec![("amen.wav", false, false)],
                selected: 0,
                in_hand: None,
                error: None,
                pads: sounding(),
                filter: "",
                editing: false,
                preview: Some((1.0, &wave)),
                playhead: None,
            }),
            Screen::Chop(Box::new(chop)),
        ];
        let mut seen = Vec::new();
        for screen in &screens {
            let mut term = Terminal::new(TestBackend::new(100, 44)).unwrap();
            term.draw(|fr| draw(fr, &status, screen, &help)).unwrap();
            let rows: Vec<Vec<String>> = term
                .backend()
                .buffer()
                .content()
                .chunks(100)
                .map(|r| r.iter().map(|c| c.symbol().to_string()).collect())
                .collect();
            // the grid: its top-left corner, the right edge of its first row, the frame of
            // the first cell after its wave rows; the strip: the rows above the bank row
            let top = rows.iter().position(|r| r[0] == "┌").expect("the grid");
            let right = rows[top].iter().rposition(|c| c == "┐").unwrap();
            let cell_end = (top + 1..rows.len()).find(|&y| rows[y][0] == "└").unwrap();
            let strip_dots = (2..top - 1)
                .filter(|&y| rows[y].iter().any(|c| c.as_str() > "\u{2800}"))
                .count();
            seen.push((top, right, cell_end, strip_dots > 0));
        }
        let f = frame_at(100, 44);
        let want = (
            f.grid.y as usize,
            f.grid.width as usize - 1,
            f.grid.y as usize + 3 + f.waves as usize,
            true,
        );
        assert_eq!(seen, vec![want; 3], "F2, F6, F3");
    }

    /// A screen drawn at `w`×`h` as text rows and as (symbol, fg, bg) cells.
    fn draw_rows(screen: &Screen, w: u16, h: u16) -> Vec<Vec<(String, Color, Color)>> {
        draw_badge_rows(screen, w, h, Vec::new())
    }

    fn draw_badge_rows(
        screen: &Screen,
        w: u16,
        h: u16,
        badges: Vec<Badge>,
    ) -> Vec<Vec<(String, Color, Color)>> {
        let (mut status, help) = chop_status();
        status.badges = badges;
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|fr| draw(fr, &status, screen, &help)).unwrap();
        term.backend()
            .buffer()
            .content()
            .chunks(w as usize)
            .map(|r| {
                r.iter()
                    .map(|c| (c.symbol().to_string(), c.fg, c.bg))
                    .collect()
            })
            .collect()
    }

    fn shared_fx() -> Fx {
        Fx {
            blocks: [
                FxBlock {
                    rows: [
                        ("type", "reverb".into()),
                        ("size", "65 %".into()),
                        ("tone", "80 %".into()),
                    ],
                },
                FxBlock {
                    rows: [
                        ("type", "delay".into()),
                        ("time", "250 ms".into()),
                        ("feedback", "40 %".into()),
                    ],
                },
            ],
            selected: 0,
            param: 0,
            pickup: None,
            learn: Vec::new(),
            lit: None,
        }
    }

    /// 82 "Open the shared FX screen": both blocks and all knob rows stay visible.
    #[test]
    fn open_the_shared_fx_screen() {
        for (w, h) in [(80, 24), (40, 12)] {
            let rows = draw_rows(&Screen::Fx(shared_fx()), w, h);
            let text = (0..rows.len())
                .map(|y| row_text(&rows, y))
                .collect::<Vec<_>>()
                .join("\n");
            let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
            for want in [
                "F4 FX",
                "shared project",
                "▶ FX 1",
                "FX 2",
                "1 type reverb",
                "2 size 65 %",
                "3 tone 80 %",
                "1 type delay",
                "2 time 250 ms",
                "3 feedback 40 %",
            ] {
                assert!(compact.contains(want), "{w}x{h}: {want}:\n{text}");
            }
            let first = text.find("FX 1").unwrap();
            let second = text.find("FX 2").unwrap();
            assert!(first < second, "the blocks keep their order");
        }
    }

    /// 82 "Select the next shared effect": moving the pick does not change row values.
    #[test]
    fn select_the_next_shared_effect() {
        for (w, h) in [(80, 24), (40, 12)] {
            for selected in 0..2 {
                for param in 0..3 {
                    let mut f = shared_fx();
                    f.selected = selected;
                    f.param = param;
                    let rows = draw_rows(&Screen::Fx(f), w, h);
                    let text = (0..rows.len())
                        .map(|y| row_text(&rows, y))
                        .collect::<Vec<_>>()
                        .join("\n");
                    assert!(text.contains(&format!("▶ FX {}", selected + 1)));
                    assert_eq!(text.matches('▶').count(), 1, "one selected block");
                    for value in ["reverb", "65 %", "80 %", "delay", "250 ms", "40 %"] {
                        assert!(text.contains(value), "{value}:\n{text}");
                    }
                    let picked: String = rows
                        .iter()
                        .flatten()
                        .filter(|(_, fg, _)| *fg == CYAN)
                        .map(|(s, _, _)| s.as_str())
                        .collect();
                    let f = shared_fx();
                    let (label, value) = &f.blocks[selected].rows[param];
                    let compact = picked.split_whitespace().collect::<Vec<_>>().join(" ");
                    assert!(compact.contains(&format!("{label} {value}")), "{picked}");
                    let other = f.blocks[1 - selected].rows[param].0;
                    if other != *label {
                        assert!(!picked.contains(other), "only the selected row is lit");
                    }
                }
            }
        }
    }

    #[test]
    fn compact_shared_effect_cards() {
        for (w, h, beside) in [
            (160, 50, true),
            (80, 24, true),
            (70, 16, true),
            (69, 50, false),
            (40, 50, false),
            (40, 16, false),
        ] {
            let rows = draw_rows(&Screen::Fx(shared_fx()), w, h);
            let positions = |symbol: &str| {
                rows.iter()
                    .enumerate()
                    .flat_map(|(y, row)| {
                        row.iter()
                            .enumerate()
                            .filter(move |(_, (s, _, _))| s == symbol)
                            .map(move |(x, _)| (x, y))
                    })
                    .collect::<Vec<_>>()
            };
            let tops = positions("┌");
            let bottoms = positions("└");
            let rights = positions("┘");
            assert_eq!(tops.len(), 2, "{w}x{h}: two framed cards");
            assert_eq!(bottoms.len(), 2);
            assert_eq!(rights.len(), 2);
            for i in 0..2 {
                assert_eq!(bottoms[i], (tops[i].0, tops[i].1 + 4));
                assert_eq!(rights[i], (tops[i].0 + 33, tops[i].1 + 4));
                assert!(bottoms[i].1 <= 12, "{w}x{h}: compact card height");
            }
            assert_eq!(tops[0].1 == tops[1].1, beside, "{w}x{h}");
            assert_eq!(tops[0].0 == tops[1].0, !beside, "{w}x{h}");
            assert_eq!(rows[tops[0].1][tops[0].0].1, MAGENTA);
            assert_eq!(rows[tops[1].1][tops[1].0].1, BORDER);
            let values = [["reverb", "65 %", "80 %"], ["delay", "250 ms", "40 %"]];
            for (i, values) in values.iter().enumerate() {
                for (param, value) in values.iter().enumerate() {
                    let line = row_text(&rows, tops[i].1 + param + 1);
                    assert!(line.contains(value), "{w}x{h}: {value}: {line}");
                    let x = tops[i].0 + 13;
                    assert_eq!(rows[tops[i].1 + param + 1][x].0, &value[..1]);
                }
            }
            for row in &rows[bottoms[1].1 + 1..h as usize - 3] {
                assert!(row.iter().all(|(s, _, _)| s == " "));
            }
        }
        let rows = draw_rows(&Screen::Fx(shared_fx()), 40, 12);
        assert!(rows.iter().flatten().all(|(s, _, _)| s != "┌"));
    }

    #[test]
    fn compact_shared_effect_cards_keep_long_learn_labels_and_values() {
        for (w, h) in [(160, 50), (80, 24), (40, 16), (40, 12)] {
            let mut f = shared_fx();
            f.selected = 1;
            f.learn = vec![(1, "CC 12 → CC 13".into())];
            let rows = draw_rows(&Screen::Fx(f), w, h);
            let text = (0..rows.len())
                .map(|y| row_text(&rows, y))
                .collect::<Vec<_>>()
                .join("\n");
            for value in ["reverb", "65 %", "80 %", "delay", "250 ms", "40 %"] {
                assert!(text.contains(value), "{w}x{h}: {value}:\n{text}");
            }
            assert!(text.contains("CC 12 → CC 13"), "{w}x{h}:\n{text}");
        }
    }

    #[test]
    fn shared_fx_shows_takeover_learn_labels_and_recent_changes() {
        let mut f = shared_fx();
        f.selected = 1;
        f.param = 1;
        f.lit = Some(2);
        f.pickup = Some("pickup: turn to 250 ms".into());
        f.learn = vec![(1, "CC 12 → CC 13".into())];
        let rows = draw_rows(&Screen::Fx(f), 80, 24);
        let text = (0..rows.len())
            .map(|y| row_text(&rows, y))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("pickup: turn to 250 ms"));
        assert_eq!(text.matches("CC 12 → CC 13").count(), 1);
        let changed: String = rows
            .iter()
            .flatten()
            .filter(|(_, _, bg)| *bg == CYAN)
            .map(|(s, _, _)| s.as_str())
            .collect();
        assert_eq!(changed.trim(), "feedback 40 %");
        let pending: String = rows
            .iter()
            .flatten()
            .filter(|(_, fg, _)| *fg == AMBER)
            .map(|(s, _, _)| s.as_str())
            .collect();
        assert!(pending.contains("CC 13"));
    }

    #[test]
    fn shared_fx_pickup_does_not_hide_a_notice() {
        let (mut status, help) = chop_status();
        status.notice = Some("project saved");
        let mut f = shared_fx();
        f.pickup = Some("pickup: turn to 250 ms".into());
        let mut term = Terminal::new(TestBackend::new(40, 12)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Fx(f), &help))
            .unwrap();
        let text: String = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(text.contains("project saved"));
        assert!(!text.contains("pickup:"));
    }

    #[test]
    fn shared_fx_selection_and_tiny_sizes_do_not_panic() {
        let (status, help) = chop_status();
        for (w, h) in [(0, 0), (1, 1), (4, 3), (20, 8), (40, 12), (80, 24)] {
            for selected in [0, 1, usize::MAX] {
                let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
                let mut f = shared_fx();
                f.selected = selected;
                f.param = usize::MAX;
                f.lit = Some(usize::MAX);
                f.learn = vec![(usize::MAX, "unmapped".into())];
                term.draw(|fr| draw(fr, &status, &Screen::Fx(f), &help))
                    .unwrap();
            }
        }
    }

    /// Scenarios "Mode badges follow the active screen state" and "Badges retain identity without color".
    #[test]
    fn mode_badges_follow_the_active_screen_state() {
        let wave = [0.1; 400];
        for (w, h) in [(120, 40), (80, 26)] {
            let screens = [
                Screen::Pads(idle_pads()),
                Screen::Fx(shared_fx()),
                Screen::Files(Files {
                    dir: "samples",
                    entries: vec![],
                    selected: 0,
                    in_hand: None,
                    error: None,
                    pads: idle_pads(),
                    filter: "",
                    editing: false,
                    preview: None,
                    playhead: None,
                }),
                Screen::Chop(Box::new(chop_demo(&wave))),
                Screen::Settings(settings_view(0, 0)),
                Screen::ProjectPicker(ProjectPickerView {
                    entries: vec!["beat".into()],
                    selected: 0,
                    naming: None,
                }),
            ];
            for screen in &screens {
                let before = draw_rows(screen, w, h);
                for (kind, normal, compact, color) in [
                    (BadgeKind::PadMode, "COPY · hit a pad", "COPY", MARK),
                    (BadgeKind::On, "EDIT · FILTER", "EDIT FILTER", CYAN),
                    (BadgeKind::Attention, "ASK", "ASK", MAGENTA),
                    (BadgeKind::Attention, "● REC 12s", "REC", MAGENTA),
                ] {
                    let after =
                        draw_badge_rows(screen, w, h, vec![Badge::new(kind, normal, compact)]);
                    assert!(
                        row_text(&after, 1).contains(normal),
                        "{w}: {}",
                        row_text(&after, 1)
                    );
                    let text: String = after[1]
                        .iter()
                        .filter(|(_, _, bg)| *bg == color)
                        .map(|(symbol, _, _)| symbol.as_str())
                        .collect();
                    assert_eq!(text.trim(), normal);
                    for y in 0..h as usize {
                        if y != 1 {
                            assert_eq!(after[y], before[y], "{w}x{h}, row {y}");
                        }
                    }
                    assert_eq!(
                        draw_rows(screen, w, h),
                        before,
                        "leaving the mode removes only its badge"
                    );
                }
            }
        }
    }

    /// Scenario "Narrow titles retain the pad mode".
    #[test]
    fn narrow_titles_retain_the_pad_mode() {
        let wave = [0.1; 400];
        let file = "名字-long-break-name-".repeat(12);
        for (w, h, which) in [
            (80, 26, (2, 3)),
            (120, 40, (2, 3)),
            (80, 26, (128, 129)),
            (120, 40, (128, 129)),
        ] {
            let mut view = chop_demo(&wave);
            view.file = &file;
            view.which = Some(which);
            view.bounds = (0..=128).map(|n| n as f32 / 128.0).collect();
            let badges = vec![
                Badge::new(
                    BadgeKind::PadMode,
                    "IN HAND + COPY + SWAP · hit a pad",
                    "IN HAND/COPY/SWAP",
                ),
                Badge::new(BadgeKind::Attention, "ASK", "ASK"),
                Badge::new(BadgeKind::Attention, "● REC 123s", "REC"),
                Badge::new(BadgeKind::On, "EDIT · BOUNDS", "EDIT BOUNDS"),
            ];
            let rows = draw_badge_rows(&Screen::Chop(Box::new(view)), w, h, badges);
            let text = row_text(&rows, 1);
            for label in ["F3 CHOP", "LISTEN", "IN HAND", "COPY", "SWAP", "128 slices"] {
                assert!(text.contains(label), "{w}: {label}: {text}");
            }
            assert!(
                text.contains(&format!("chop {} of {}", which.0, which.1)),
                "{text}"
            );
            let name_column = rows[1]
                .iter()
                .position(|cell| cell.0 == "名")
                .expect("the filename starts with a wide glyph");
            assert_eq!(
                rows[1][name_column + 2].0,
                "字",
                "wide glyphs keep their two-cell spacing"
            );
            assert_eq!(rows[1].len(), w as usize);
            assert!(
                !text.contains("EDIT"),
                "the fourth optional badge cannot appear"
            );
            if w == 80 {
                assert!(text.contains("IN HAND/COPY/SWAP"), "{text}");
                assert!(!text.contains("ASK") && !text.contains("REC"));
            } else {
                assert!(text.contains("ASK"), "{text}");
                assert_eq!(text.contains("REC 123s"), which == (2, 3), "{text}");
            }
        }
        for (normal, compact) in [
            ("MARK · hit pads · 128 marked", "MARK 128"),
            ("128 MARKED · Esc drops", "128 MARKED"),
            ("ASSIGN · 128/128 placed", "ASSIGN 128/128"),
            ("EDIT · FILTER", "EDIT FILTER"),
            ("EDIT · BOUNDS", "EDIT BOUNDS"),
            ("EDIT · FX 2", "EDIT FX 2"),
        ] {
            let badge = Badge::new(BadgeKind::PadMode, normal, compact);
            let width = (Line::from(compact).width() + 3) as u16;
            let line = title(vec![], &[badge], width, 0);
            assert_eq!(
                line.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
                    .trim(),
                compact
            );
            assert!(line.width() <= width as usize);
        }
    }

    /// Scenario "Playback slots remain stable beside modes".
    #[test]
    fn playback_slots_remain_stable_beside_modes() {
        let wave = [0.1; 400];
        for (w, h) in [(80, 26), (120, 40)] {
            for graphics in [false, true] {
                let mut first_column = None;
                for playing in [false, true, false] {
                    let mut view = chop_demo(&wave);
                    view.mode = if playing { "LISTEN ▶" } else { "LISTEN" }.into();
                    let (mut status, help) = chop_status();
                    status.badges = vec![Badge::new(BadgeKind::On, "EDIT · BOUNDS", "EDIT BOUNDS")];
                    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
                    let mut images = Vec::new();
                    term.draw(|frame| {
                        let screen = Screen::Chop(Box::new(view));
                        if graphics {
                            draw_gfx(frame, &status, &screen, &help, &mut images);
                        } else {
                            draw(frame, &status, &screen, &help);
                        }
                    })
                    .unwrap();
                    let row = &term.backend().buffer().content()[w as usize..2 * w as usize];
                    let text: String = row.iter().map(|c| c.symbol()).collect();
                    assert_eq!(
                        row[10..20].iter().map(|c| c.symbol()).collect::<String>(),
                        if playing {
                            " LISTEN ▶ "
                        } else {
                            " LISTEN   "
                        }
                    );
                    for cell in &row[10..20] {
                        assert_eq!(
                            (cell.fg, cell.bg),
                            if playing { (BG, CYAN) } else { (DIM, BG) }
                        );
                    }
                    let column = row
                        .windows(4)
                        .position(|r| r.iter().map(|c| c.symbol()).collect::<String>() == "EDIT")
                        .unwrap();
                    if let Some(before) = first_column {
                        assert_eq!(column, before);
                    }
                    first_column = Some(column);
                    assert!(text.contains("break_lofi.wav") && text.contains("4 slices"));
                    assert_eq!(images.len(), usize::from(graphics));
                    if graphics {
                        assert_eq!(
                            images[0].area,
                            Rect {
                                y: frame_at(w, h).strip.y + 2,
                                height: frame_at(w, h).strip.height - 2,
                                ..frame_at(w, h).strip
                            }
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn settings_title_badges_stay_visible_while_midi_rows_scroll() {
        let entries: Vec<_> = (0..40)
            .map(|n| MidiRow::Action {
                label: format!("action{n:02}"),
                who: "none".into(),
                taken: false,
                stays: Vec::new(),
            })
            .collect();
        for (w, h) in [(80, 26), (120, 40)] {
            let mut st = settings_view(2, 0);
            st.midi_rows = &entries;
            st.midi_row = 39;
            let rows = draw_badge_rows(
                &Screen::Settings(st),
                w,
                h,
                vec![Badge::new(BadgeKind::PadMode, "LEARN", "LEARN")],
            );
            let header = row_text(&rows, 1);
            assert!(
                header.contains("F12 SETTINGS")
                    && header.contains("LEARN")
                    && header.contains("[MIDI]"),
                "{header}"
            );
            assert!(rows[3..h as usize - 4].iter().any(|row| {
                row.iter()
                    .map(|c| c.0.as_str())
                    .collect::<String>()
                    .contains("action39")
            }));
        }
    }

    fn hidden_pads() -> Pads {
        Pads {
            hidden: true,
            ..idle_pads()
        }
    }

    /// Scenario "Hide the pads" (32): no grid, the strip takes its rows, the pad line names
    /// the bank and the selected pad.
    #[test]
    fn hide_the_pads() {
        let rows = draw_rows(&Screen::Pads(hidden_pads()), 100, 30);
        assert!(rows.iter().all(|r| r[0].0 != "┌"), "no grid");
        let f = pad_frame(Rect::new(0, 1, 100, 30 - 2 - HELP_ROWS), true);
        assert_eq!(
            f.strip.height,
            25 - 2,
            "the strip takes all but the title and the pad line"
        );
        assert!(
            f.strip.height > frame_at(100, 30).strip.height + 15,
            "taller by the grid rows"
        );
        assert_eq!(
            f.strip.width,
            frame_at(100, 30).grid.width,
            "in the grid's place"
        );
        let line = row_text(&rows, f.bank.y as usize);
        assert!(line.contains("bank [1 ⠀⠀] 2⠀⠀"), "every bank (62): {line}");
        assert!(line.contains("8⠀⠀ · pad W N5"), "{line}");
    }

    /// 62 "Every bank in the pad line" and "A hit lights in the bank row": bank 3 shows its
    /// fill; a hit on a pad of the open bank turns its glyph half white, with the pads
    /// hidden and shown.
    #[test]
    fn a_hit_lights_in_the_bank_row() {
        for hidden in [true, false] {
            let mut p = idle_pads();
            p.hidden = hidden;
            p.bank_fill[2] = 0xFFFF;
            p.bank_fill[0] = 0x00FF; // pads 1–4 and Q–R hold sounds
            p.pads[5].flash = true; // pad W, in the left half of bank 1
            let spans = banks(&p);
            let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
            assert!(text.contains(" 3⣿⣿"), "bank 3 with its fill: {text}");
            let white: Vec<&str> = spans
                .iter()
                .filter(|s| s.style.fg == Some(WHITE))
                .map(|s| s.content.as_ref())
                .collect();
            // only the pad held now, both glyphs of the bank, no fill mixed in
            assert_eq!(white, vec![bank_glyphs(1 << 5).as_str()], "{white:?}");
        }
    }

    /// 76 "A marked pad is yellow": pad E of bank 3 has the dark yellow fill and the
    /// yellow frame, its name readable on it; the other pads keep the bank colour.
    #[test]
    fn a_marked_pad_is_yellow() {
        let mut p = idle_pads();
        p.bank = 2;
        p.pads[6].marked = true; // pad E
        let rows = draw_rows(&Screen::Pads(p), 100, 30);
        let cells: Vec<&(String, Color, Color)> = rows.iter().flatten().collect();
        assert!(
            cells
                .iter()
                .any(|(s, fg, bg)| s == "─" && *fg == MARK && *bg == MARK_BG),
            "the yellow frame"
        );
        assert!(
            cells
                .iter()
                .any(|(s, fg, bg)| s.trim() != "" && *fg == BRIGHT && *bg == MARK_BG),
            "the name on the fill"
        );
        assert!(
            cells
                .iter()
                .any(|(s, fg, _)| s == "─" && *fg == BANK_COLORS[2]),
            "the other pads keep the bank colour"
        );
    }

    /// Scenarios "Placement marks distinguish confirmation from swap" and "Assignment numbers appear on target pads".
    #[test]
    fn pad_corner_marks_keep_identity_and_state_styles() {
        for width in [13, 24, 40] {
            for corner in [
                PadCorner::Available,
                PadCorner::Replace,
                PadCorner::Slice(128),
            ] {
                for state in ["idle", "selected", "hit", "marked", "hand"] {
                    let mut pad = idle_pads().pads.remove(0);
                    pad.name = "名字".into();
                    pad.flash = state == "hit";
                    pad.marked = state == "marked";
                    pad.in_hand = state == "hand";
                    let mut term = Terminal::new(TestBackend::new(width, 4)).unwrap();
                    term.draw(|frame| {
                        pad_cell(
                            frame,
                            frame.area(),
                            &pad,
                            state == "selected",
                            BANK_COLORS[2],
                            0,
                            (true, Some(corner)),
                        )
                    })
                    .unwrap();
                    let cells = term.backend().buffer().content();
                    let text = match corner {
                        PadCorner::Available => "◎".to_string(),
                        PadCorner::Replace => "!".to_string(),
                        PadCorner::Slice(n) => n.to_string(),
                    };
                    let (fg, bg) = match state {
                        "selected" => (CYAN, PANEL),
                        "hit" => (WHITE, WHITE),
                        "marked" => (MARK, MARK_BG),
                        "hand" => (MAGENTA, PANEL),
                        _ => (MARK, PANEL),
                    };
                    assert_eq!((cells[0].fg, cells[0].bg), (fg, bg), "{width}, {state}");
                    assert_eq!(cells[1].symbol(), "1", "the key stays readable");
                    assert_eq!(
                        cells[1].fg,
                        if state == "idle" {
                            BANK_COLORS[2]
                        } else if state == "hit" {
                            BG
                        } else {
                            fg
                        }
                    );
                    let start = width as usize - 1 - Line::from(text.as_str()).width();
                    let label: String = cells[start..width as usize - 1]
                        .iter()
                        .map(|cell| cell.symbol())
                        .collect();
                    assert_eq!(label, text);
                    assert_eq!(cells[width as usize - 1].symbol(), "┐");
                    assert_eq!(cells[width as usize + 2].symbol(), "名");
                    assert_eq!(cells[width as usize + 4].symbol(), "字");
                }
            }
        }
    }

    /// Scenarios "Marking changes appearance without silencing pads" and "Hidden grids retain visible mode identity".
    #[test]
    fn yellow_grid_feedback_keeps_shared_geometry_and_hidden_layout() {
        fn pads<'a>(screen: &'a mut Screen<'_>) -> &'a mut Pads {
            match screen {
                Screen::Pads(p) => p,
                Screen::Files(f) => &mut f.pads,
                Screen::Chop(c) => &mut c.pads,
                _ => panic!("a grid screen is required"),
            }
        }
        let wave = [0.1; 400];
        for (w, h) in [(80, 26), (120, 40)] {
            let mut screens = [
                Screen::Pads(idle_pads()),
                Screen::Files(Files {
                    dir: "samples",
                    entries: vec![],
                    selected: 0,
                    in_hand: None,
                    error: None,
                    pads: idle_pads(),
                    filter: "",
                    editing: false,
                    preview: None,
                    playhead: None,
                }),
                Screen::Chop(Box::new(chop_demo(&wave))),
            ];
            for screen in &mut screens {
                let p = pads(screen);
                p.pads[1].flash = true;
                p.pads[2].marked = true;
                p.pads[3].in_hand = true;
                let before = draw_rows(screen, w, h);
                let p = pads(screen);
                p.feedback.outline = true;
                p.feedback.corners[0] = Some(PadCorner::Available);
                p.feedback.corners[1] = Some(PadCorner::Replace);
                p.feedback.corners[2] = Some(PadCorner::Slice(128));
                let after = draw_rows(screen, w, h);
                assert!(
                    after
                        .iter()
                        .flatten()
                        .any(|(text, fg, _)| text == "◎" && *fg == MARK)
                );
                assert!(
                    after
                        .iter()
                        .flatten()
                        .any(|(text, fg, bg)| text == "─" && *fg == MARK && *bg == PANEL)
                );
                // The cell edges, strip, panel and bottom rows stay at their original coordinates.
                for y in 0..h as usize {
                    for x in 0..w as usize {
                        if ["┌", "┐", "└", "┘", "│", "┆"].contains(&before[y][x].0.as_str())
                        {
                            assert_eq!(after[y][x].0, before[y][x].0, "{w}x{h}: {x},{y}");
                        }
                    }
                }
                for y in [
                    0,
                    1,
                    h as usize - 4,
                    h as usize - 3,
                    h as usize - 2,
                    h as usize - 1,
                ] {
                    assert_eq!(after[y], before[y], "shared row {y}");
                }
                pads(screen).feedback = PadFeedback::default();
                assert_eq!(
                    draw_rows(screen, w, h),
                    before,
                    "leaving the mode restores the cells"
                );
                pads(screen).hidden = true;
                let hidden = draw_rows(screen, w, h);
                pads(screen).feedback = PadFeedback {
                    outline: true,
                    corners: [Some(PadCorner::Replace); 16],
                };
                assert_eq!(
                    draw_rows(screen, w, h),
                    hidden,
                    "mode marks cannot open a hidden grid"
                );
            }
        }
    }

    /// 77 "The mode as a glyph" and "The group on the pad": `⇢ +1 mute 2` under the
    /// name of a one-shot pad at +1 dB in group 2, `⇥` for a gate pad at 0 dB; no digit on
    /// the frame.
    #[test]
    fn the_mode_as_a_glyph() {
        let mut p = idle_pads();
        p.pads[4].gate = false;
        p.pads[4].volume_db = 1.0;
        p.pads[4].group = 2;
        p.pads[5].gate = true;
        p.pads[5].volume_db = 0.0;
        p.pads[5].group = 0;
        let rows = draw_rows(&Screen::Pads(p), 100, 30);
        let all: Vec<String> = (0..rows.len()).map(|y| row_text(&rows, y)).collect();
        let text = all.join("\n");
        assert!(text.contains("│ ⇢ +1 mute 2"), "{text}");
        assert!(text.contains("│ ⇥ "), "{text}");
        assert!(!text.contains("one-shot +"), "{text}");
        assert!(
            !all.iter().any(|l| l.contains("2┐")),
            "no group digit on a frame: {text}"
        );
    }

    /// Scenario "Pads PLAY badge has fixed width".
    #[test]
    fn pads_play_badge_has_fixed_width() {
        for (width, height) in [(60, 24), (80, 30), (100, 40)] {
            for hidden in [false, true] {
                for edit in [false, true] {
                    for marked in [false, true] {
                        let mut first: Option<Vec<Vec<(String, Color, Color)>>> = None;
                        let mut mark_column = None;
                        for playing in [false, true, false] {
                            let mut p = idle_pads();
                            p.playing = playing;
                            p.hidden = hidden;
                            p.edit = edit.then_some("TONE");
                            let badges = if marked {
                                vec![Badge::new(
                                    BadgeKind::PadMode,
                                    "MARK · hit pads · 3 marked",
                                    "MARK 3",
                                )]
                            } else {
                                Vec::new()
                            };
                            let rows = render_pads_snapshot(
                                p,
                                ([-60.0; 2], [-60.0; 2], false),
                                false,
                                width,
                                height,
                                badges,
                            );
                            let y = (0..rows.len())
                                .find(|&y| row_text(&rows, y).contains("F2 PADS"))
                                .unwrap();
                            let row = &rows[y];
                            let badge = row
                                .windows(4)
                                .position(|cells| {
                                    cells.iter().map(|cell| cell.0.as_str()).collect::<String>()
                                        == "PLAY"
                                })
                                .expect("PLAY badge");
                            let text: String = row[badge - 1..badge + 9]
                                .iter()
                                .map(|cell| cell.0.as_str())
                                .collect();
                            assert_eq!(
                                text,
                                if playing {
                                    " PLAY ▶   "
                                } else {
                                    " PLAY     "
                                }
                            );
                            for cell in &row[badge - 1..badge + 9] {
                                assert_eq!(
                                    (cell.1, cell.2),
                                    if playing { (BG, CYAN) } else { (DIM, BG) }
                                );
                            }
                            if marked {
                                let at = row
                                    .windows(4)
                                    .position(|cells| {
                                        cells.iter().map(|cell| cell.0.as_str()).collect::<String>()
                                            == "MARK"
                                    })
                                    .unwrap();
                                if let Some(column) = mark_column {
                                    assert_eq!(at, column);
                                }
                                mark_column = Some(at);
                                let plate = "MARK · hit pads · 3 marked";
                                assert_eq!(
                                    row[at..at + plate.chars().count()]
                                        .iter()
                                        .map(|cell| cell.0.as_str())
                                        .collect::<String>(),
                                    plate
                                );
                                for cell in &row[at..at + plate.chars().count()] {
                                    assert_eq!((cell.1, cell.2), (BG, MARK));
                                }
                            }
                            if let Some(before) = &first {
                                for (i, (old, new)) in before.iter().zip(&rows).enumerate() {
                                    if i != y {
                                        assert_eq!(old, new, "only the title changes");
                                    }
                                }
                            } else {
                                first = Some(rows);
                            }
                        }
                    }
                }
            }
        }
    }

    /// 76 "The mark plate": the plate stands on the title line, black on yellow.
    #[test]
    fn the_mark_plate_is_yellow_on_the_title_line() {
        let rows = draw_badge_rows(
            &Screen::Pads(idle_pads()),
            100,
            30,
            vec![Badge::new(
                BadgeKind::PadMode,
                "MARK · hit pads · 3 marked",
                "MARK 3",
            )],
        );
        let y = (0..rows.len())
            .find(|&y| row_text(&rows, y).contains("F2 PADS"))
            .expect("the title line");
        let title = row_text(&rows, y);
        assert!(
            title.contains("F2 PADS   PLAY       MARK · hit pads · 3 marked "),
            "{title}"
        );
        assert!(
            rows[y]
                .iter()
                .any(|(s, fg, bg)| s == "M" && *fg == BG && *bg == MARK),
            "black on yellow"
        );
    }

    /// 76 "Marks in another bank are seen" and "No marks, the fill": bank 2 shows all its
    /// dots yellow, bank 5 the three marked pads; the other banks their fill.
    #[test]
    fn marks_in_another_bank_are_seen() {
        let mut p = idle_pads();
        p.bank = 1;
        p.bank_fill = [0x000F; BANKS];
        let spans = banks(&p);
        assert!(
            spans.iter().all(|s| s.style.fg != Some(MARK)),
            "no marks, the fill"
        );
        p.bank_marks[1] = 0xFFFF;
        p.bank_marks[4] = 0b0111;
        let spans = banks(&p);
        let yellow: Vec<&str> = spans
            .iter()
            .filter(|s| s.style.fg == Some(MARK))
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(
            yellow,
            vec![bank_glyphs(0xFFFF).as_str(), bank_glyphs(0b0111).as_str()]
        );
        let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(
            text.contains(&format!(" 3{}", bank_glyphs(0x000F))),
            "{text}"
        );
    }

    /// 76 "A hit over the marks": while pad Q of the open bank flashes, its glyphs show
    /// only pad Q in white; after, the yellow marks.
    #[test]
    fn a_hit_over_the_marks() {
        let mut p = idle_pads();
        p.bank = 1;
        p.bank_marks[1] = 0x00F0;
        p.pads[4].flash = true; // pad Q
        let colored = |p: &Pads, c| -> Vec<String> {
            banks(p)
                .iter()
                .filter(|s| s.style.fg == Some(c))
                .map(|s| s.content.to_string())
                .collect()
        };
        assert_eq!(colored(&p, WHITE), vec![bank_glyphs(1 << 4)]);
        assert!(colored(&p, MARK).is_empty(), "no mix in one cell");
        p.pads[4].flash = false;
        assert_eq!(colored(&p, MARK), vec![bank_glyphs(0x00F0)]);
    }

    /// Scenario "The right column keeps its rows" (32): with the pads hidden the right
    /// column draws all it draws with them shown, on F2, F6 and F3.
    #[test]
    fn the_right_column_keeps_its_rows() {
        let side_text = |screen: &Screen| -> String {
            let rows = draw_rows(screen, 100, 30);
            let side = pad_frame(Rect::new(0, 1, 100, 25), true).side;
            rows[side.y as usize..side.bottom() as usize]
                .iter()
                .map(|r| {
                    r[side.x as usize..]
                        .iter()
                        .map(|c| c.0.as_str())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        let f2 = side_text(&Screen::Pads(hidden_pads()));
        for want in [" L ", " R ", "clip", "pad W", "volume", "release"] {
            assert!(f2.contains(want), "F2 {want}: {f2}");
        }
        let names: Vec<String> = (0..12).map(|i| format!("file{i:02}.wav")).collect();
        let f6 = side_text(&Screen::Files(Files {
            dir: "~/Music",
            entries: names.iter().map(|n| (n.as_str(), false, false)).collect(),
            selected: 0,
            in_hand: None,
            error: None,
            pads: hidden_pads(),
            filter: "",
            editing: false,
            preview: None,
            playhead: None,
        }));
        assert!(f6.contains("~/Music"), "{f6}");
        assert!(
            f6.contains("file00.wav") && f6.contains("file11.wav"),
            "the whole list: {f6}"
        );
        let wave: Vec<f32> = (0..4000).map(|i| (i % 1000) as f32 / 1000.0).collect();
        let mut chop = chop_demo(&wave);
        chop.pads = hidden_pads();
        let side_lines = chop.side.clone();
        let f3 = side_text(&Screen::Chop(Box::new(chop)));
        for l in &side_lines {
            let first = l.split_whitespace().next().unwrap_or_default();
            assert!(f3.contains(first), "F3 {l}: {f3}");
        }
    }

    /// Scenario "The same on every screen" (32): hidden on F6 and F3 too, the same pad line
    /// row and the same strip.
    #[test]
    fn hidden_the_same_on_every_screen() {
        let wave: Vec<f32> = (0..4000).map(|i| (i % 1000) as f32 / 1000.0).collect();
        let mut chop = chop_demo(&wave);
        chop.pads = hidden_pads();
        let files = Screen::Files(Files {
            dir: "~/Music",
            entries: vec![("amen.wav", false, false)],
            selected: 0,
            in_hand: None,
            error: None,
            pads: hidden_pads(),
            filter: "",
            editing: false,
            preview: Some((1.0, &wave)),
            playhead: None,
        });
        let f = pad_frame(Rect::new(0, 1, 100, 30 - 2 - HELP_ROWS), true);
        for (name, screen) in [("F6", files), ("F3", Screen::Chop(Box::new(chop)))] {
            let rows = draw_rows(&screen, 100, 30);
            assert!(rows.iter().all(|r| r[0].0 != "┌"), "{name}: no grid");
            let line = row_text(&rows, f.bank.y as usize);
            assert!(line.contains("pad W"), "{name}: {line}");
            // the strip draws down to the row above the pad line
            assert!(
                row_text(&rows, f.strip.bottom() as usize - 1)
                    .chars()
                    .any(|c| c as u32 >= 0x2800),
                "{name}: the strip reaches the pad line"
            );
        }
    }

    /// Scenario "The pad line flashes" (32).
    #[test]
    fn the_pad_line_flashes() {
        let mut p = hidden_pads();
        p.selected = 4;
        p.pads[4].flash = true;
        let rows = draw_rows(&Screen::Pads(p), 100, 30);
        let y = pad_frame(Rect::new(0, 1, 100, 25), true).bank.y as usize;
        let line = row_text(&rows, y);
        assert!(line.contains("pad Q N4"), "{line}");
        let at = line.chars().collect::<Vec<_>>();
        let x = (0..at.len())
            .find(|&i| at[i..].starts_with(&['p', 'a', 'd', ' ', 'Q']))
            .unwrap();
        assert_eq!(rows[y][x].2, WHITE, "the line lights up like a hit pad");
    }

    /// 66, from the owner's screen: a long file name does not push the wave right; under
    /// the label row the wave starts at the left edge of the strip.
    #[test]
    fn a_long_name_leaves_the_wave_whole() {
        let wave = vec![0.5f32; 400];
        let name = "Gabriel Saban - Growing Pressure (hitmos.fm).mp3";
        let screen = Screen::Files(Files {
            dir: "~/Music",
            entries: vec![(name, false, false)],
            selected: 0,
            in_hand: None,
            error: None,
            pads: idle_pads(),
            filter: "",
            editing: false,
            preview: Some((121.6, &wave)),
            playhead: None,
        });
        let rows = draw_rows(&screen, 100, 30);
        let strip = frame_at(100, 30).strip;
        let y = strip.y as usize;
        assert!(row_text(&rows, y).contains(name));
        for row in &rows[y + 1..strip.bottom() as usize] {
            let cell = &row[strip.x as usize + 1].0;
            assert!(
                cell.chars().any(|c| (c as u32) > 0x2800),
                "the wave starts at the left edge: {cell:?}"
            );
        }
    }

    /// 66 "The filter is seen" and "The wave names its file": a filter on stands before a
    /// long folder path with how to clear it; the wave names the selected file.
    #[test]
    fn the_filter_is_seen_and_the_wave_names_its_file() {
        let wave = vec![0.5f32; 400];
        let screen = Screen::Files(Files {
            dir: "/home/user/music/samples/very/long/folder/of/breaks/and/one/shots",
            entries: vec![("kick.wav", false, false)],
            selected: 0,
            in_hand: None,
            error: None,
            pads: idle_pads(),
            filter: "sn",
            editing: false,
            preview: Some((0.33, &wave)),
            playhead: None,
        });
        let rows = draw_rows(&screen, 100, 30);
        let text: Vec<String> = rows
            .iter()
            .map(|r| r.iter().map(|c| c.0.as_str()).collect())
            .collect();
        let has = |want: &str| text.iter().any(|r| r.contains(want));
        assert!(has("filter: sn · Esc clears"), "{}", text.join("\n"));
        assert!(has("kick.wav · 0.33 s"), "the wave names its file");
    }

    /// Scenario "The line runs with the sound" (33): halfway through the file the green
    /// line stands in the middle of the wave, over every row of the strip; no playhead, no line.
    #[test]
    fn the_line_runs_with_the_sound() {
        let wave: Vec<f32> = (0..1600).map(|i| (i % 100) as f32 / 100.0).collect();
        let files = |playhead| {
            Screen::Files(Files {
                dir: "~/Music",
                entries: vec![("amen.wav", false, false)],
                selected: 0,
                in_hand: None,
                error: None,
                pads: idle_pads(),
                filter: "",
                editing: false,
                preview: Some((4.0, &wave)),
                playhead,
            })
        };
        let strip = frame_at(100, 30).strip;
        // the wave takes the whole width; the label lies over its top row (66)
        let cols = strip.width as usize;
        let rows = draw_rows(&files(Some(0.5)), 100, 30);
        let x = strip.x as usize + cols / 2;
        for (y, row) in rows
            .iter()
            .enumerate()
            .take(strip.bottom() as usize)
            .skip(strip.y as usize)
        {
            assert_eq!(row[x].0, "│", "row {y}");
            assert_eq!(row[x].1, METER, "the green of the chop playhead");
        }
        let line_at = |rows: &[Vec<(String, Color, Color)>]| {
            (strip.y as usize..strip.bottom() as usize)
                .any(|y| rows[y].iter().any(|(s, fg, _)| s == "│" && *fg == METER))
        };
        assert!(
            !line_at(&draw_rows(&files(None), 100, 30)),
            "no line when silent"
        );
    }

    /// Scenario "The folder above its list" (31): the path stands right above the first
    /// line of the list, in the list's column, not above the pads.
    #[test]
    fn the_folder_above_its_list() {
        let f = Files {
            dir: "/home/user/Downloads",
            entries: vec![("breaks", true, false), ("amen.wav", false, false)],
            selected: 1,
            in_hand: None,
            error: None,
            pads: idle_pads(),
            filter: "",
            editing: false,
            preview: None,
            playhead: None,
        };
        let (status, help) = chop_status();
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Files(f), &help))
            .unwrap();
        let rows: Vec<String> = term
            .backend()
            .buffer()
            .content()
            .chunks(100)
            .map(|row| row.iter().map(|c| c.symbol()).collect())
            .collect();
        let side = frame_at(100, 30).side;
        let at = |y: usize, what: &str| {
            rows[y]
                .chars()
                .collect::<Vec<_>>()
                .windows(what.chars().count())
                .position(|w| w.iter().collect::<String>() == what)
        };
        let y = side.y as usize;
        assert_eq!(
            at(y, "/home/user/Downloads"),
            Some(side.x as usize + 1),
            "{}",
            rows[y]
        );
        assert_eq!(
            at(y + 1, "breaks/"),
            Some(side.x as usize + 1),
            "the list starts under it"
        );
        assert!(
            !rows[1].contains("Downloads"),
            "not in the title line above the pads"
        );
    }

    #[test]
    fn braille_waveform_follows_the_loudness() {
        // loud half, then silence: full columns on the left, a thin middle line on the right
        let wave: Vec<f32> = (0..800).map(|i| if i < 400 { 1.0 } else { 0.0 }).collect();
        let rows = braille(&wave, 4, 2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].chars().next(), Some('⣿'));
        assert_eq!(rows[1].chars().next(), Some('⣿'));
        // the silent part keeps only the dots next to the middle
        assert_eq!(rows[0].chars().nth(3), Some('⣀'));
        assert_eq!(rows[1].chars().nth(3), Some('⠉'));
        assert_eq!(braille(&[], 3, 1), ["   "]);
    }

    /// 21 "Nothing jumps": the grid and the panel stay on their rows whatever the help says.
    #[test]
    fn nothing_jumps() {
        let rows_of = |groups: Vec<HelpGroup>, edit: bool| {
            let mut p = idle_pads();
            if edit {
                p.edit = Some("TONE");
            }
            let help = Help {
                groups: &groups,
                hint: None,
                full: None,
                question: None,
                learn: None,
            };
            let status = Status {
                badges: Vec::new(),
                save: SaveStatus::Clean,
                backend: None,
                recording: false,
                buffer_size: 128,
                sample_rate: 48_000,
                load_peak: 0.0,
                xruns: 0,
                master_db: 0,
                meters: ([-60.0; 2], [-60.0; 2], false),
                no_releases: false,
                notice: None,
                project: None,
            };
            let mut term = Terminal::new(TestBackend::new(100, 26)).unwrap();
            term.draw(|f| draw(f, &status, &Screen::Pads(p), &help))
                .unwrap();
            let text: Vec<String> = term
                .backend()
                .buffer()
                .content()
                .chunks(100)
                .map(|row| row.iter().map(|c| c.symbol()).collect())
                .collect();
            // where the pad names and the panel values are
            let at = |want: &str| text.iter().position(|r| r.contains(want));
            (at("N0"), at("N15"), at("volume"), at("release"))
        };
        let short = vec![HelpGroup {
            name: "PAD",
            items: vec![("Enter".into(), "edit".into())],
        }];
        let long = vec![HelpGroup {
            name: "EDIT",
            items: (0..40)
                .map(|i| (format!("K{i}"), "a long help item".into()))
                .collect(),
        }];
        let base = rows_of(short, false);
        assert!(base.1.is_some(), "the whole grid fits in 26 rows");
        assert_eq!(
            rows_of(long, true),
            base,
            "a long help and the edit frame move nothing"
        );
    }

    /// The pad frame (31) of a `w`×`h` terminal: the body between the status line on top
    /// and the notice and help rows at the bottom.
    fn frame_at(w: u16, h: u16) -> PadFrame {
        pad_frame(Rect::new(0, 1, w, h - 2 - HELP_ROWS), false)
    }

    /// The strip rows of a 100×26 terminal as the colours of their drawn cells.
    fn strip_colors(rows: &[Vec<(String, Color, Color)>]) -> Vec<Color> {
        let f = frame_at(100, 26);
        rows[f.strip.y as usize..f.strip.bottom() as usize]
            .iter()
            .flat_map(|r| r.iter())
            .filter(|(s, _, _)| s != " ")
            .map(|(_, fg, _)| *fg)
            .collect()
    }

    /// recording 25: "The voice on the strip" — while a recording runs the strip draws what
    /// goes into the file in the rec colour; after it the output is dim again. The output
    /// recording takes the same path: the app hands the recorded peaks as `out_wave`.
    #[test]
    fn the_voice_on_the_strip() {
        let loud = || {
            let mut p = idle_pads();
            p.out_wave = vec![0.8; 64];
            p
        };
        let rec = strip_colors(&render_pads_rec(
            loud(),
            ([-60.0; 2], [-60.0; 2], false),
            true,
            100,
            26,
        ));
        assert!(
            !rec.is_empty() && rec.iter().all(|c| *c == MAGENTA),
            "{rec:?}"
        );
        let after = strip_colors(&render_pads(loud()));
        assert!(after.iter().all(|c| *c == DIM), "{after:?}");
    }

    /// recording 25: "Recording in the pad edit" — the recording takes the place of the
    /// envelope, and the envelope comes back when it stops.
    #[test]
    fn recording_in_the_pad_edit() {
        let edit = || {
            let mut p = idle_pads();
            p.edit = Some("ENV");
            p.param = PadParam::Attack;
            p.out_wave = vec![0.8; 64];
            p.big = Some(BigView {
                wave: vec![0.5; 64],
                env: vec![1.0; 64],
                attack: 0.1,
                decay: 0.0,
                release: 0.5,
            });
            p
        };
        let rec = strip_colors(&render_pads_rec(
            edit(),
            ([-60.0; 2], [-60.0; 2], false),
            true,
            100,
            26,
        ));
        assert!(
            rec.iter().all(|c| *c == MAGENTA),
            "the recording, not the envelope: {rec:?}"
        );
        let back = strip_colors(&render_pads(edit()));
        assert!(
            back.iter().any(|c| *c == WHITE || *c == CYAN),
            "the envelope line is back: {back:?}"
        );
    }

    /// pad-playback 21: "The attack is a slope" and "The live output comes back".
    #[test]
    fn the_attack_is_a_slope() {
        let f = frame_at(100, 26);
        let strip = |attack: f32, picked: PadParam, edit: bool| {
            let mut p = idle_pads();
            p.edit = edit.then_some("ENV");
            p.param = picked;
            p.out_wave = vec![1.0; 64];
            let env: Vec<f32> = (0..64)
                .map(|i| ((i as f32 + 0.5) / 64.0 / attack.max(1e-6)).min(1.0))
                .collect();
            p.big = Some(BigView {
                wave: vec![0.5; 64],
                env,
                attack,
                decay: 0.0,
                release: 0.5,
            });
            let rows = render_pads(p);
            rows[f.strip.y as usize..f.strip.bottom() as usize].to_vec()
        };
        // the first column where the line reaches the top row
        let top_at = |rows: &[Vec<(String, Color, Color)>]| {
            rows[0]
                .iter()
                .position(|(s, fg, _)| (*fg == WHITE || *fg == CYAN) && s != " ")
        };
        let short = strip(0.1, PadParam::Volume, true);
        let long = strip(0.5, PadParam::Attack, true);
        let (a, b) = (top_at(&short).unwrap(), top_at(&long).unwrap());
        assert!(b > a, "a longer attack reaches the top later: {a} vs {b}");
        // the picked attack is cyan on the rising part
        assert!(
            long.iter().flatten().any(|(_, fg, _)| *fg == CYAN),
            "the picked part has its own colour"
        );
        // closed: the live output again, no envelope line
        let live = strip(0.5, PadParam::Attack, false);
        assert!(
            live.iter()
                .flatten()
                .all(|(_, fg, _)| *fg != CYAN && *fg != WHITE),
            "the live strip is dim"
        );
    }

    /// pad-playback 21: "The release follows the mode".
    #[test]
    fn the_release_follows_the_mode() {
        let text = |gate: bool| -> String {
            let mut p = idle_pads();
            p.edit = Some("ENV");
            p.param = PadParam::Release;
            let sel = p.selected;
            p.pads[sel].gate = gate;
            p.pads[sel].release_ms = 250;
            p.big = Some(BigView {
                wave: vec![0.5; 64],
                env: vec![1.0; 64],
                attack: 0.0,
                decay: 0.0,
                release: 0.5,
            });
            render_pads(p)
                .iter()
                .map(|row| row.iter().map(|c| c.0.as_str()).collect::<String>() + "\n")
                .collect()
        };
        let one_shot = text(false);
        assert!(
            !one_shot.contains("key up"),
            "no release for a one-shot pad"
        );
        assert!(one_shot.contains("release 250 ms (gate)"), "{one_shot}");
        let gate = text(true);
        assert!(gate.contains("key up"), "{gate}");
        assert!(!gate.contains("(gate)"), "a gate pad uses it");
    }

    /// 21: "The frame tells the mode" and "The knob numbers".
    #[test]
    fn the_frame_tells_the_mode() {
        let text = |p: Pads| -> String {
            render_pads(p)
                .iter()
                .map(|row| row.iter().map(|c| c.0.as_str()).collect::<String>() + "\n")
                .collect()
        };
        let idle = text(idle_pads());
        assert!(!idle.contains("EDIT"), "no frame while the mode is closed");
        let mut p = idle_pads();
        p.edit = Some("ENV");
        p.pages = vec!["TONE", "ENV", "SOURCE", "FILTER"];
        p.param = PadParam::Decay;
        p.knobs = vec![PadParam::Attack, PadParam::Decay, PadParam::Release];
        let edit = text(p);
        println!("{edit}");
        assert!(
            edit.contains("EDIT · TONE ENV SOURCE FILTER"),
            "the frame names every page"
        );
        for want in ["1 attack", "2 decay", "3 release"] {
            assert!(edit.contains(want), "{want} missing");
        }
        assert!(!edit.contains("1 volume"), "volume is not on the page");
        // the frame is magenta, apart from the bank-coloured pad frames
        let rows = {
            let mut p = idle_pads();
            p.edit = Some("ENV");
            render_pads(p)
        };
        assert!(
            rows.iter()
                .flatten()
                .any(|(s, fg, _)| s == "┐" && *fg == MAGENTA)
        );
    }

    /// 43 "Every row in sight" and "The title shows where Tab leads": on every page the
    /// panel shows all rows; only the knob numbers and the lit page in the title move.
    /// The window is tall enough for every row: a shorter one folds (70).
    #[test]
    fn every_row_in_sight() {
        let pages = [
            (
                "TONE",
                vec![PadParam::Volume, PadParam::Pitch, PadParam::Group],
            ),
            (
                "ENV",
                vec![PadParam::Attack, PadParam::Decay, PadParam::Release],
            ),
            (
                "SOURCE",
                vec![PadParam::Source, PadParam::Start, PadParam::End],
            ),
            (
                "FILTER",
                vec![PadParam::Filter, PadParam::Cutoff, PadParam::Reso],
            ),
        ];
        for (page, knobs) in pages {
            let mut p = idle_pads();
            p.edit = Some(page);
            p.pages = vec!["TONE", "ENV", "SOURCE", "FILTER"];
            p.param = knobs[0];
            p.knobs = knobs.clone();
            // a window tall enough for every row; a 40-row one folds a block (70)
            let rows = render_pads_at(p, 100, TALL);
            let text: String = rows
                .iter()
                .map(|row| row.iter().map(|c| c.0.as_str()).collect::<String>() + "\n")
                .collect();
            for row in [
                "volume", "pitch", "attack", "decay", "release", "source", "start", "end",
                "filter", "cutoff", "reso", "scope",
            ] {
                assert!(text.contains(row), "{page}: {row} missing\n{text}");
            }
            // the lit page: the title cells of its name stand on the magenta background
            let lit: String = rows
                .iter()
                .flatten()
                .filter(|(_, _, bg)| *bg == MAGENTA)
                .map(|(c, _, _)| c.as_str())
                .collect();
            assert_eq!(lit, page, "the title lights {page}");
            assert!(text.contains("1 "), "{page}: knob numbers");
        }
    }

    /// 44: the FILTER page draws the curve in the strip: its caption on the top row, the
    /// frequencies on the bottom one, the line high on the left and low on the right for a
    /// low-pass.
    #[test]
    fn the_filter_curve_takes_the_strip() {
        let mut p = idle_pads();
        p.edit = Some("FILTER");
        p.pages = vec!["TONE", "ENV", "SOURCE", "FILTER"];
        p.param = PadParam::Filter;
        p.curve = Some(FilterCurve {
            caption: "low-pass · 1.0 kHz · reso 0 %".into(),
            levels: (0..256)
                .map(|i| if i < 128 { 1.0 } else { 0.001 })
                .collect(),
            span: (14.1, 28_284.0),
        });
        let rows = render_pads_at(p, 100, 40);
        let text: Vec<String> = rows
            .iter()
            .map(|row| row.iter().map(|c| c.0.as_str()).collect())
            .collect();
        let all = text.join("\n");
        assert!(all.contains("low-pass · 1.0 kHz · reso 0 %"), "{all}");
        assert!(text.iter().any(|r| r.contains("20k")), "the frequencies");
        // the full level sits near the top, the cut part on the bottom
        let dotted = |r: &String| r.chars().any(|c| ('\u{2801}'..='\u{28ff}').contains(&c));
        assert!(text.iter().take(6).any(dotted), "{all}");
    }

    /// 44 "The hump stays on the screen", the drawing side: a hump of +24 dB still has its
    /// top dot inside the strip, the scale rose with it.
    #[test]
    fn a_tall_hump_fits_the_strip() {
        let mut p = idle_pads();
        p.edit = Some("FILTER");
        p.param = PadParam::Filter;
        let mut levels = vec![1.0; 256];
        levels[128] = 16.0; // +24 dB
        p.curve = Some(FilterCurve {
            caption: "low-pass".into(),
            levels,
            span: (14.1, 28_284.0),
        });
        let rows = render_pads_at(p, 100, 40);
        let text: Vec<String> = rows
            .iter()
            .map(|r| r.iter().map(|c| c.0.as_str()).collect())
            .collect();
        // the caption's row is the strip's first: the hump's top dot is on it
        let first = text.iter().find(|r| r.contains("low-pass")).unwrap();
        assert!(
            first
                .chars()
                .any(|c| ('\u{2801}'..='\u{28ff}').contains(&c)),
            "the top dot is on the first row: {first}"
        );
    }

    /// 46: a dashed pad draws its frame with dashes, the others with lines.
    #[test]
    fn a_dashed_frame() {
        let mut p = idle_pads();
        p.pads[0].dashed = true;
        let rows = render_pads(p);
        let text: String = rows
            .iter()
            .map(|r| r.iter().map(|c| c.0.as_str()).collect::<String>() + "\n")
            .collect();
        assert!(text.contains("┄┄┄") && text.contains('┆'), "{text}");
        assert!(text.contains("───"), "the other pads keep their lines");
    }

    /// 48 "Blocks": the panel's rows stand in blocks under the lines TONE, ENV, FILTER,
    /// SOURCE in this order; the line of the edit's page is lit.
    #[test]
    fn the_panel_in_blocks() {
        let mut p = idle_pads();
        p.edit = Some("FILTER");
        p.pages = vec!["TONE", "ENV", "FILTER", "SOURCE"];
        p.param = PadParam::Filter;
        let rows = render_pads_at(p, 100, TALL);
        let text: Vec<String> = rows
            .iter()
            .map(|r| r.iter().map(|c| c.0.as_str()).collect())
            .collect();
        let at = |name: &str| {
            text.iter()
                .position(|r| r.contains(&format!("─ {name} ─")))
                .unwrap_or_else(|| panic!("no {name} line:\n{}", text.join("\n")))
        };
        let (tone, env, filter, source) = (at("TONE"), at("ENV"), at("FILTER"), at("SOURCE"));
        assert!(tone < env && env < filter && filter < source);
        let row = |name: &str| text.iter().position(|r| r.contains(name)).unwrap();
        assert!(
            tone < row("volume") && row("volume") < env,
            "volume under TONE"
        );
        assert!(source < row("start "), "start under SOURCE");
        // the FILTER line is lit, the others dim
        let color = |line: usize, name: &str| {
            let x = text[line].find(name).unwrap();
            let x = text[line][..x].chars().count();
            rows[line][x].1
        };
        assert_eq!(color(filter, "FILTER"), MAGENTA);
        assert_eq!(color(tone, "TONE"), DIM);
    }

    /// 77 "The panel names the settings" (drawing): a changed value bright, a default one
    /// dim, the blocks with a change carry a dot; the mode line of the pad ends with `✱`.
    #[test]
    fn the_panel_shows_the_change() {
        let mut p = idle_pads();
        let sel = p.selected;
        p.pads[sel].changed = vec![PadParam::Pitch, PadParam::Filter];
        p.pads[sel].glyphs = "✱".into();
        let rows = render_pads_at(p, 100, TALL);
        let text: Vec<String> = rows
            .iter()
            .map(|r| r.iter().map(|c| c.0.as_str()).collect())
            .collect();
        let has = |want: &str| text.iter().any(|r| r.contains(want));
        assert!(has("─ TONE ● ─"), "{}", text.join("\n"));
        assert!(has("─ FILTER ● ─"));
        assert!(has("─ ENV ─") && !has("ENV ●"));
        assert!(has("─ SOURCE ─") && !has("SOURCE ●"));
        assert!(has("│ ⇢ ✱"), "the mark on the mode line");
        let color = |name: &str, value: &str| {
            let line = text.iter().position(|r| r.contains(name)).unwrap();
            let x = text[line].find(value).unwrap();
            let x = text[line][..x].chars().count();
            rows[line][x].1
        };
        assert_eq!(color("pitch", "+0 st"), WHITE, "changed: bright");
        assert_eq!(color("attack", "0 ms"), DIM, "default: dim");
    }

    /// A window tall enough for every row of the panel but its FX block (70).
    const TALL: u16 = 46;

    /// The panel of a pad with effects, open on `page` with `param` picked, as text.
    fn panel_text(w: u16, h: u16, page: &'static str, param: PadParam) -> String {
        let mut p = idle_pads();
        p.edit = Some(page);
        p.param = param;
        p.pages = vec!["TONE", "ENV", "FILTER", "SOURCE", "FX", "FX1*", "FX2*"];
        p.fx = vec![
            (PadParam::Drive, "drive   ", "0 %".into()),
            (PadParam::Send1, "send 1  ", "0 %".into()),
            (PadParam::Send2, "send 2  ", "0 %".into()),
        ];
        let rows = render_pads_at(p, w, h);
        // the panel: from its titled frame to the right edge
        let x = rows
            .iter()
            .find_map(|r| {
                (0..r.len().saturating_sub(1))
                    .rev()
                    .find(|&i| r[i].0 == "┌" && r[i + 1].0 == " ")
            })
            .unwrap_or(0);
        rows.iter()
            .map(|r| r[x..].iter().map(|c| c.0.as_str()).collect::<String>() + "\n")
            .collect()
    }

    /// 70 pad-playback "The closed panel names the edit key": the first row ends with
    /// `Enter: edit` while the edit is closed; in the edit the frame tells and the hint goes.
    #[test]
    fn the_closed_panel_names_the_edit_key() {
        let row = |edit: Option<&'static str>| {
            let mut p = idle_pads();
            p.edit_key = "Enter".into();
            p.edit = edit;
            p.pages = vec!["TONE", "ENV", "FILTER", "SOURCE"];
            render_pads_at(p, 170, 52)
                .iter()
                .map(|r| r.iter().map(|c| c.0.as_str()).collect::<String>())
                .find(|r| r.contains("pad W  bank 1"))
                .unwrap_or_default()
        };
        assert!(
            row(None).contains("pad W  bank 1   Enter: edit"),
            "{}",
            row(None)
        );
        assert!(
            !row(Some("TONE")).contains("Enter: edit"),
            "{}",
            row(Some("TONE"))
        );
    }

    /// 70 pad-playback "The scope is always named": the second row of the panel, under the
    /// pad and its bank, says `scope   bank 2`; nothing under the blocks names it.
    #[test]
    fn the_scope_is_always_named() {
        let mut p = idle_pads();
        p.scope = "bank 2".into();
        let rows: Vec<String> = render_pads_at(p, 170, 52)
            .iter()
            .map(|r| r.iter().map(|c| c.0.as_str()).collect())
            .collect();
        let pad = rows
            .iter()
            .position(|r| r.contains("pad W  bank 1"))
            .unwrap();
        assert!(
            rows[pad + 1].contains("scope   bank 2"),
            "{}",
            rows[pad + 1]
        );
        assert!(!rows.iter().any(|r| r.contains("change ")), "no change row");
    }

    /// 70: the mute group has its own row under the pitch, `off` or its number.
    #[test]
    fn the_group_under_the_pitch() {
        let text = |group| {
            let mut p = idle_pads();
            let sel = p.selected;
            p.pads[sel].group = group;
            render_pads_at(p, 170, 52)
                .iter()
                .map(|r| r.iter().map(|c| c.0.as_str()).collect::<String>() + "\n")
                .collect::<String>()
        };
        let t = text(1);
        assert!(t.contains("  mute    1"), "{t}");
        let pitch = t.find("pitch ").unwrap();
        let mute = t.find("mute    1").unwrap();
        let mode = t.find("mode    one-shot").unwrap();
        assert!(mode < pitch && pitch < mute, "under the pitch");
        assert!(text(0).contains("  mute    off"));
    }

    /// 70 pad-playback "A shorter window folds the blocks": at 100×40 on TONE the TONE,
    /// ENV and FILTER rows are seen, SOURCE and FX fold to `▸`, the scope row stays; on
    /// SOURCE its rows are seen and FILTER folds.
    #[test]
    fn a_shorter_window_folds_the_blocks() {
        let t = panel_text(100, 40, "TONE", PadParam::Volume);
        for row in [
            "volume", "pitch", "mute", "attack", "release", "filter", "reso", "scope",
        ] {
            assert!(t.contains(row), "{row}:\n{t}");
        }
        assert!(t.contains("─ SOURCE ▸") && t.contains("─ FX ▸"), "{t}");
        assert!(!t.contains("start ") && !t.contains("drive"), "{t}");
        let t = panel_text(100, 40, "SOURCE", PadParam::Start);
        assert!(t.contains("start ") && t.contains("─ FILTER ▸"), "{t}");
        assert!(t.contains("scope"));
        // a taller window unfolds them; the panel stops growing with the pad grid, so
        // the FX block may stay folded
        let t = panel_text(100, TALL, "TONE", PadParam::Volume);
        assert!(t.contains("start ") && !t.contains("SOURCE ▸"), "{t}");
    }

    /// 70 pad-playback "A very short window scrolls": at 100×30, walking every parameter,
    /// the picked one is in sight, ▼ while rows are below, ▲ while rows are above, and
    /// the scope row stays.
    #[test]
    fn a_very_short_window_scrolls() {
        use PadParam as W;
        let walk = [
            ("TONE", W::Volume, "volume "),
            ("TONE", W::Pitch, "pitch "),
            ("TONE", W::Group, "mute "),
            ("ENV", W::Attack, "attack "),
            ("ENV", W::Decay, "decay "),
            ("ENV", W::Release, "release "),
            ("FILTER", W::Filter, "filter "),
            ("FILTER", W::Cutoff, "cutoff "),
            ("FILTER", W::Reso, "reso "),
            ("SOURCE", W::Source, "source "),
            ("SOURCE", W::Start, "start "),
            ("SOURCE", W::End, "end "),
            ("FX", W::Drive, "drive "),
            ("FX", W::Send1, "send 1 "),
            ("FX", W::Send2, "send 2 "),
        ];
        for (page, param, label) in walk {
            let t = panel_text(100, 30, page, param);
            assert!(t.contains(label), "{label} in sight:\n{t}");
            assert!(t.contains("scope"), "{label}: scope\n{t}");
        }
        assert!(panel_text(100, 30, "TONE", W::Volume).contains("▼"));
        let last = panel_text(100, 30, "FX", W::Send2);
        assert!(last.contains("▲") && !last.contains("▼"), "{last}");
    }

    /// 70 pad-playback "A narrow window": at 70×40 the labels shorten, the values stay
    /// whole, and the title names the open page and its place.
    #[test]
    fn a_narrow_window() {
        let t = panel_text(70, 40, "TONE", W_PITCH);
        for row in ["vol  +0 dB", "pit  +0 st", "mute off", "TONE 1/7"] {
            assert!(t.contains(row), "{row}:\n{t}");
        }
        assert!(!t.contains("FILTER SOURCE"), "not every page in the title");
    }

    const W_PITCH: PadParam = PadParam::Pitch;

    /// 82: a caller can keep the old slot field, but pads show only drive and sends.
    #[test]
    fn shared_slots_are_not_drawn_in_the_pad_panel() {
        let mut p = idle_pads();
        p.edit = Some("FX");
        p.pages = vec!["TONE", "ENV", "FILTER", "SOURCE", "FX"];
        p.scope = "pad W".into();
        p.fx = vec![
            (PadParam::Drive, "drive   ", "0 %".into()),
            (PadParam::Send1, "send 1  ", "0 %  reverb".into()),
            (PadParam::Send2, "send 2  ", "0 %  delay".into()),
        ];
        p.slot = Some((
            "FX2*",
            "FX 2 * shared".into(),
            vec![
                (PadParam::Fx2Kind, "type    ", "delay".into()),
                (PadParam::Fx2A, "time    ", "250 ms".into()),
                (PadParam::Fx2B, "feedback", "40 %".into()),
            ],
        ));
        let rows = render_pads_at(p, 170, 52);
        let text: Vec<String> = rows
            .iter()
            .map(|r| r.iter().map(|c| c.0.as_str()).collect())
            .collect();
        let has = |want: &str| text.iter().any(|r| r.contains(want));
        assert!(!has("FX 2 * shared"), "{}", text.join("\n"));
        assert!(!has("feedback") && !has("250 ms"), "no shared slot rows");
        assert!(has("pad W"), "the pad scope stays visible");
        assert!(has("drive") && has("send 1") && has("send 2"));
        assert!(!has("FX2*"), "no shared slot page in the title");
    }

    /// 51: the hits of the hits mode stand as ▾ on the row over the wave, at their place.
    #[test]
    fn the_hits_are_marked() {
        let wave = vec![0.5f32; 200];
        let mut c = chop_demo(&wave);
        c.hits = vec![0.25, 0.75];
        let (status, help) = chop_status();
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Chop(Box::new(c)), &help))
            .unwrap();
        let buf = term.backend().buffer().clone();
        let rows: Vec<String> = buf
            .content()
            .chunks(100)
            .map(|r| r.iter().map(|c| c.symbol()).collect())
            .collect();
        let row = rows
            .iter()
            .find(|r| r.contains('▾'))
            .expect("a row of marks");
        assert_eq!(row.matches('▾').count(), 2, "{row}");
    }

    /// 21: "A knob lights what it turns" — the drawing side: the lit value is inverted.
    #[test]
    fn a_knob_lights_what_it_turns() {
        let mut p = idle_pads();
        p.lit = Some(PadParam::Pitch);
        let rows = render_pads(p);
        let lit: Vec<_> = rows
            .iter()
            .flatten()
            .filter(|(_, _, bg)| *bg == CYAN)
            .map(|(s, _, _)| s.as_str())
            .collect();
        assert_eq!(lit.concat().trim(), "+0 st", "{lit:?}");
    }

    /// The chop screen of the marker test: 4 slices, slice 2 selected, the cursor at 0.6.
    fn chop_demo(wave: &[f32]) -> Chop<'_> {
        let keys = [
            '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v',
        ];
        let pads = Pads {
            bank: 0,
            pads: keys
                .iter()
                .enumerate()
                .map(|(i, &key)| PadView {
                    naming: false,
                    glyphs: String::new(),
                    changed: Vec::new(),
                    dim: false,
                    dashed: false,
                    source: String::new(),
                    bounds: (0.0, 0.0),
                    filter: Default::default(),
                    wave: Vec::new(),
                    key,
                    name: if i < 4 {
                        format!("slice {}", i + 1)
                    } else {
                        "-".into()
                    },
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
                    note: (i < 4).then(|| "261 ms".into()),
                    detail: None,
                })
                .collect(),
            selected: 0,
            pickup: None,
            scope: String::new(),
            param: PadParam::Volume,
            bank_fill: [0; BANKS],
            bank_marks: [0; BANKS],
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
            playing: false,
            feedback: PadFeedback::default(),
        };
        Chop {
            owners: Vec::new(),
            owner_bank: Vec::new(),
            source_banks: [false; BANKS],
            slice_keys: None,
            bands: &[],
            band_mean: [1.0; 3],
            file: "break_lofi.wav",
            mode: "LISTEN".into(),
            seconds: 2.61,
            which: None,
            hits: Vec::new(),
            wave,
            bounds: vec![0.0, 0.25, 0.5, 0.75, 1.0],
            selected: 1,
            cursor: Some(0.6),
            playhead: None,
            play: None,
            info: "slice 2 · 0.653 s".into(),
            side: vec!["pads play the slices".into(), "Enter: assign".into()],
            banks: Some({
                let mut b = [(false, false); BANKS];
                b[1] = (true, true);
                b
            }),
            pads,
            edit: false,
            hot: None,
        }
    }

    /// 27 chopping: "Open from a slice pad" on the screen — the header says whose cuts
    /// these are, every slice wide enough shows its pads next to its number.
    #[test]
    fn the_cuts_belong_to_the_file() {
        let wave: Vec<f32> = (0..4000).map(|i| (i % 1000) as f32 / 1000.0).collect();
        let c = Chop {
            owners: vec!["1-Q".into(), String::new(), "1-E,2-W".into(), String::new()],
            owner_bank: vec![Some(0), None, Some(0), None],
            source_banks: [false; BANKS],
            slice_keys: None,
            bands: &[],
            band_mean: [1.0; 3],
            ..chop_demo(&wave)
        };
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: Some("jack"),
            recording: false,
            buffer_size: 128,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0; 2], [-60.0; 2], false),
            no_releases: false,
            notice: None,
            project: None,
        };
        let help = Help {
            groups: &[],
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Chop(Box::new(c)), &help))
            .unwrap();
        let rows: Vec<String> = term
            .backend()
            .buffer()
            .content()
            .chunks(100)
            .map(|row| row.iter().map(|c| c.symbol()).collect())
            .collect();
        let h = rows.iter().position(|r| r.contains("F3 CHOP")).unwrap();
        assert!(rows[h].contains("cuts of break_lofi.wav"), "{}", rows[h]);
        let numbers = &rows[h + 1];
        // 36 "Slices on pads are marked": a plate in the bank color with the pads, a dim ○
        assert!(numbers.starts_with("1 1-Q"), "{numbers}");
        assert!(numbers.contains("2○"), "{numbers}");
        assert!(numbers.contains("3 1-E,2-W"), "{numbers}");
        let cells = term
            .backend()
            .buffer()
            .content()
            .chunks(100)
            .nth(h + 1)
            .unwrap();
        assert_eq!(
            cells[0].bg, BANK_COLORS[0],
            "the plate wears the bank color"
        );
        assert_eq!(cells[3].bg, BANK_COLORS[0], "across the slice");
        let ring = cells.iter().find(|c| c.symbol() == "○").unwrap();
        assert_eq!(ring.fg, BORDER, "the ring is dim");
        assert_ne!(ring.bg, BANK_COLORS[0], "no plate on a slice on no pad");
    }

    /// 38 "Kick and hat by color" and "The selection stays visible": a low column is warm,
    /// a high one cyan; the selected slice is brighter than the same color elsewhere.
    #[test]
    fn kick_and_hat_by_color() {
        let rgb = |c: Color| match c {
            Color::Rgb(r, g, b) => (r as i32, g as i32, b as i32),
            _ => panic!("{c:?}"),
        };
        let even = [1.0; 3];
        let (r, _, b) = rgb(band_color([1.0, 0.1, 0.05], even, 1.0));
        assert!(r > b + 100, "a kick is red");
        let (r, _, b) = rgb(band_color([0.05, 0.1, 1.0], even, 1.0));
        assert!(b > r + 100, "a hat is blue");
        // a break whose bands are nearly even on average (measured 0.37 / 0.31 / 0.32): a
        // column a little richer in low than usual is clearly red, not grey
        let mean = [0.37, 0.31, 0.32];
        let (r, g, b) = rgb(band_color([0.45, 0.28, 0.27], mean, 1.0));
        assert!(
            r > g + 80 && r > b + 80,
            "the kick stands out: ({r}, {g}, {b})"
        );
        let dim = rgb(band_color([1.0, 0.1, 0.05], even, 0.35));
        let full = rgb(band_color([1.0, 0.1, 0.05], even, 1.0));
        assert!(full.0 > dim.0, "outside the region the wave is dim");
        // 40: drawn, the selected slice is the one in color, the other slices grey, no plate
        let wave: Vec<f32> = (0..4000).map(|i| ((i % 40) as f32 / 40.0) - 0.5).collect();
        let bands: Vec<[f32; 3]> = (0..16)
            .map(|i| {
                if i < 8 {
                    [1.0, 0.1, 0.05]
                } else {
                    [0.05, 0.1, 1.0]
                }
            })
            .collect();
        let mut c = chop_demo(&wave);
        let (s0, s1) = (c.bounds[c.selected], c.bounds[c.selected + 1]);
        c.bands = &bands;
        let (status, help) = chop_status();
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Chop(Box::new(c)), &help))
            .unwrap();
        let row = frame_at(100, 30).strip.y as usize + 4;
        let buf = term.backend().buffer().clone();
        let cell = |x: usize| buf.content()[row * 100 + x].clone();
        let inside = (s0 * 100.0).ceil() as usize + 1..(s1 * 100.0) as usize;
        let dot = |c: &ratatui::buffer::Cell| {
            c.symbol() > "\u{2800}" && c.symbol() != "│" && c.symbol() != "┃"
        };
        assert!(
            inside.clone().map(cell).filter(dot).any(|c| {
                let (r, g, b) = rgb(c.fg);
                r != g || g != b
            }),
            "the selected slice is in color"
        );
        assert!(
            (0..100)
                .filter(|x| !inside.contains(x))
                .map(cell)
                .filter(dot)
                .all(|c| {
                    let (r, g, b) = rgb(c.fg);
                    r == g && g == b
                }),
            "the other slices are grey"
        );
        assert!((0..100).all(|x| cell(x).bg == BG), "no background plate");
    }

    /// 21 "A held modifier lights its bound", the cursor on that bound: it wears the bound
    /// colour instead of hiding it.
    #[test]
    fn the_lit_bound_under_the_cursor() {
        let wave: Vec<f32> = (0..4000).map(|i| (i % 1000) as f32 / 1000.0).collect();
        let draw_with = |hot: Option<bool>| {
            let mut c = chop_demo(&wave);
            c.cursor = Some(c.bounds[c.selected]);
            c.hot = hot;
            let at = c.bounds[c.selected];
            let (status, help) = chop_status();
            let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
            term.draw(|fr| draw(fr, &status, &Screen::Chop(Box::new(c)), &help))
                .unwrap();
            let strip = frame_at(100, 30).strip;
            let x = ((at * 100.0) as usize).min(99);
            let row = (strip.y + 3) as usize;
            term.backend()
                .buffer()
                .content()
                .chunks(100)
                .nth(row)
                .unwrap()[x]
                .clone()
        };
        let lit = draw_with(Some(false));
        assert_eq!(lit.symbol(), "┃", "the cursor is there");
        assert_eq!(lit.fg, AMBER, "in the colour of the lit bound");
        assert_eq!(
            draw_with(None).fg,
            WHITE,
            "no modifier held: the plain cursor"
        );
    }

    /// Scenario "Chop listen badge has fixed width".
    #[test]
    fn chop_listen_badge_has_fixed_width() {
        let wave = [0.1; 4000];
        let (status, help) = chop_status();
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        let mut file_column = None;
        for mode in ["LISTEN", "LISTEN ▶", "LISTEN"] {
            let mut view = chop_demo(&wave);
            view.mode = mode.into();
            term.draw(|frame| draw(frame, &status, &Screen::Chop(Box::new(view)), &help))
                .unwrap();
            let header = term
                .backend()
                .buffer()
                .content()
                .chunks(100)
                .find(|row| {
                    row.iter()
                        .map(|cell| cell.symbol())
                        .collect::<String>()
                        .contains("F3 CHOP")
                })
                .unwrap();
            let cells: Vec<_> = header.iter().map(|cell| cell.symbol()).collect();
            let badge = cells
                .windows(6)
                .position(|w| w.concat() == "LISTEN")
                .unwrap();
            let file = cells
                .windows(7)
                .position(|w| w.concat() == "cuts of")
                .unwrap();
            assert_eq!(file - (badge - 1) - 3, 10, "fixed badge width: {mode}");
            if let Some(column) = file_column {
                assert_eq!(file, column, "file text must not jump");
            } else {
                file_column = Some(file);
            }
            let playing = mode.ends_with('▶');
            assert_eq!(header[badge].fg, if playing { BG } else { DIM });
            assert_eq!(header[badge].bg, if playing { CYAN } else { BG });
            assert_eq!(header[badge + 7].symbol(), if playing { "▶" } else { " " });
        }
        let mut view = chop_demo(&wave);
        view.mode = "ASSIGN · by hand".into();
        let mut status = status;
        status.badges = vec![Badge::new(
            BadgeKind::PadMode,
            "ASSIGN · 1/4 placed",
            "ASSIGN 1/4",
        )];
        term.draw(|frame| draw(frame, &status, &Screen::Chop(Box::new(view)), &help))
            .unwrap();
        let header = term
            .backend()
            .buffer()
            .content()
            .chunks(100)
            .find(|row| {
                row.iter()
                    .map(|cell| cell.symbol())
                    .collect::<String>()
                    .contains("F3 CHOP")
            })
            .unwrap();
        let text = header.iter().map(|cell| cell.symbol()).collect::<String>();
        assert!(
            text.contains(" LISTEN     ASSIGN · 1/4 placed    cuts of"),
            "{text}"
        );
        let start = text.find(" ASSIGN").unwrap();
        for cell in &header[start..start + " ASSIGN · 1/4 placed ".chars().count()] {
            assert_eq!(
                (cell.fg, cell.bg),
                (BG, MARK),
                "assignment is a separate PadMode badge"
            );
        }
    }

    /// Scenario "Keyboard pad shows the Chop playback line": the supplied position
    /// uses the same green overlay in both wave modes and never moves the edit cursor.
    #[test]
    fn chop_pad_playback_line_in_braille_and_picture() {
        let wave: Vec<f32> = (0..4000).map(|i| (i % 40) as f32 / 80.0).collect();
        let (status, help) = chop_status();
        for graphics in [false, true] {
            for edit in [false, true] {
                let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
                for position in [Some(0.6), Some(0.7), None] {
                    let mut view = chop_demo(&wave);
                    view.cursor = Some(0.1);
                    view.playhead = position;
                    view.edit = edit;
                    let screen = Screen::Chop(Box::new(view));
                    let mut images = Vec::new();
                    term.draw(|frame| {
                        if graphics {
                            draw_gfx(frame, &status, &screen, &help, &mut images);
                        } else {
                            draw(frame, &status, &screen, &help);
                        }
                    })
                    .unwrap();
                    let strip = frame_at(100, 30).strip;
                    for y in strip.y + 3..strip.y + strip.height {
                        let buffer = term.backend().buffer();
                        let cursor = buffer.cell((10, y)).unwrap();
                        assert_eq!((cursor.symbol(), cursor.fg), ("┃", WHITE));
                        for (at, x) in [(0.6, 60), (0.7, 70)] {
                            let cell = buffer.cell((x, y)).unwrap();
                            let is_line = cell.symbol() == "│" && cell.fg == METER;
                            assert_eq!(
                                is_line,
                                position == Some(at),
                                "graphics {graphics}, edit {edit}, row {y}"
                            );
                        }
                    }
                    if graphics {
                        assert_eq!(images.len(), 1);
                        assert_eq!(images[0].id, 1);
                    } else {
                        assert!(images.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    fn chop_screen_shows_markers_cursor_and_slice_pads() {
        let wave: Vec<f32> = (0..4000).map(|i| (i % 1000) as f32 / 1000.0).collect();
        let c = chop_demo(&wave);
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: Some("jack"),
            recording: false,
            buffer_size: 128,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0, -60.0], [-60.0, -60.0], false),
            no_releases: false,
            notice: None,
            project: None,
        };
        let help = Help {
            groups: &[],
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Chop(Box::new(c)), &help))
            .unwrap();
        let text: String = term
            .backend()
            .buffer()
            .content()
            .chunks(100)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
            .collect();
        println!("{text}");
        for want in [
            "F3 CHOP",
            "break_lofi.wav",
            "2.61 s · 4 slices",
            "slice 2 · 0.653 s",
            "LISTEN",
            "Enter: assign",
            "bank ",
            "slice 3",
            "261 ms",
        ] {
            assert!(text.contains(want), "{want} missing");
        }
        let wave_row = text.lines().nth(4).unwrap();
        let cols: Vec<char> = wave_row.chars().collect();
        assert_eq!(cols[25], '│', "the selected slice starts here");
        assert_eq!(
            cols[50], '│',
            "and ends here: thin, the start is the picked bound"
        );
        assert_eq!(cols[75], '│');
        assert_eq!(cols[60], '┃', "cursor");
        assert!(cols[10] as u32 >= 0x2800, "waveform in Braille");
        let numbers = text.lines().nth(2).unwrap();
        assert!(numbers.starts_with('1') && numbers.chars().nth(75) == Some('4'));
        // 21 "The bar shows what plays": the green bar right above the wave covers the span
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|fr| {
            draw(
                fr,
                &status,
                &Screen::Chop(Box::new(Chop {
                    play: Some((0.5, 0.75)),
                    ..chop_demo(&wave)
                })),
                &help,
            )
        })
        .unwrap();
        let bar_row: Vec<(String, Color)> = (0..100)
            .map(|x| {
                let c = term.backend().buffer().cell((x, 3)).unwrap();
                (c.symbol().to_string(), c.fg)
            })
            .collect();
        assert_eq!(bar_row[60], ("━".to_string(), METER));
        assert_eq!(bar_row[40].0, " ", "before the span");
        assert_eq!(bar_row[80].0, " ", "after the span");
        // 21 "The playhead does not take the cursor": the sound runs on its own thin line
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|fr| {
            draw(
                fr,
                &status,
                &Screen::Chop(Box::new(Chop {
                    cursor: Some(0.1),
                    playhead: Some(0.6),
                    ..chop_demo(&wave)
                })),
                &help,
            )
        })
        .unwrap();
        let buf = term.backend().buffer();
        let cell = |x: u16, y: u16| buf.cell((x, y)).unwrap().clone();
        assert_eq!(
            (cell(10, 4).symbol(), cell(10, 4).fg),
            ("┃", WHITE),
            "the cursor"
        );
        assert_eq!(
            (cell(60, 4).symbol(), cell(60, 4).fg),
            ("│", METER),
            "the playhead"
        );
        assert_eq!(
            cell(10, 2).symbol(),
            "!",
            "the ! marks the cursor, not the sound"
        );
        // 21 "The picked bound is visible": with the end picked, the end is the thick line
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|fr| {
            draw(
                fr,
                &status,
                &Screen::Chop(Box::new(Chop {
                    hot: Some(true),
                    edit: true,
                    ..chop_demo(&wave)
                })),
                &help,
            )
        })
        .unwrap();
        let rows: Vec<String> = term
            .backend()
            .buffer()
            .content()
            .chunks(100)
            .map(|row| row.iter().map(|c| c.symbol()).collect())
            .collect();
        let cols: Vec<char> = rows[4].chars().collect();
        assert_eq!((cols[25], cols[50]), ('│', '│'), "both lines thin");
        let fg = |x: u16| term.backend().buffer().cell((x, 4)).unwrap().fg;
        assert_eq!(
            (fg(25), fg(50)),
            (MAGENTA, AMBER),
            "the end takes the bound colour"
        );
        assert!(
            rows.iter().any(|r| r.contains("EDIT · BOUNDS")),
            "the slice line names the edit"
        );
    }

    #[test]
    fn question_box_over_the_screen() {
        let q = [
            "Replace what lies on 1 pad?".to_string(),
            "  bank 2  Q  kick  →  slice 1".into(),
            "Enter: replace · Esc: keep".into(),
        ];
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: None,
            recording: false,
            buffer_size: 0,
            sample_rate: 0,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0, -60.0], [-60.0, -60.0], false),
            no_releases: false,
            notice: None,
            project: None,
        };
        let pads = Pads {
            bank: 0,
            pads: Vec::new(),
            selected: 0,
            pickup: None,
            scope: String::new(),
            param: PadParam::Volume,
            bank_fill: [0; BANKS],
            bank_marks: [0; BANKS],
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
            playing: false,
            feedback: PadFeedback::default(),
        };
        let help = Help {
            groups: &[],
            hint: None,
            full: None,
            question: Some(&q),
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(80, 20)).unwrap();
        term.draw(|f| draw(f, &status, &Screen::Pads(pads), &help))
            .unwrap();
        let text: String = term
            .backend()
            .buffer()
            .content()
            .chunks(80)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
            .collect();
        for line in &q {
            assert!(text.contains(line.trim()), "{line} missing");
        }
    }

    #[test]
    fn nothing_is_cut_in_the_full_help() {
        let rows = vec![
            (String::new(), "PADS".to_string()),
            (
                "Ctrl+X".to_string(),
                "take the selected pad; the next pad hit swaps them".to_string(),
            ),
            ("1-4 Q-R A-F Z-V".to_string(), "play pads".to_string()),
        ];
        let mut term = Terminal::new(TestBackend::new(140, 34)).unwrap();
        term.draw(|f| full_help(f, f.area(), &rows)).unwrap();
        let text: String = term
            .backend()
            .buffer()
            .content()
            .chunks(140)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
            .collect();
        assert!(text.contains("the next pad hit swaps them"), "{text}");
        assert!(text.contains(" Ctrl+X "));
    }

    #[test]
    fn keys_look_like_keys() {
        let groups = [HelpGroup {
            name: "PAD",
            items: vec![("Enter".into(), "mode: gate".into())],
        }];
        let rows = help_lines(&groups, 140);
        let spans: Vec<_> = rows.iter().flat_map(|r| r.spans.iter()).collect();
        let cap = spans.iter().find(|s| s.content.contains("Enter")).unwrap();
        assert_eq!(cap.content, " Enter ");
        assert_eq!(cap.style.bg, Some(BORDER), "the key is a cap");
        let text = spans.iter().find(|s| s.content.contains("mode")).unwrap();
        assert_eq!(text.style.bg, None, "the text next to it is plain");
    }

    /// 43 "Help line fits the terminal": the groups stand in columns of up to three rows,
    /// a longer group takes another column under its name, a short one has its name above;
    /// what does not fit drops off whole from the end.
    #[test]
    fn help_stands_in_columns() {
        let groups = [
            HelpGroup {
                name: "CHOP",
                items: vec![("Enter".into(), "to pads".into()); 4],
            },
            HelpGroup {
                name: "BANK",
                items: vec![("PgUp PgDn".into(), "turn".into())],
            },
            HelpGroup {
                name: "",
                items: vec![("F1".into(), "help".into())],
            },
        ];
        let text = |w| -> Vec<String> {
            help_lines(&groups, w)
                .iter()
                .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
                .collect()
        };
        let rows = text(140);
        assert_eq!(rows.len(), 3, "{rows:#?}");
        assert!(
            rows[0].starts_with(" CHOP  Enter to pads   Enter to pads"),
            "{rows:#?}"
        );
        assert!(rows[1].starts_with("       Enter to pads"), "{rows:#?}");
        assert!(rows[0].contains("│ BANK"), "a short group: its name above");
        assert!(rows[1].contains("│  PgUp PgDn turn"), "{rows:#?}");
        let width = rows[0].chars().count();
        assert!(
            rows.iter().all(|r| r.chars().count() == width),
            "the columns line up"
        );
        // narrower than all: the last group goes whole, nothing is cut in the middle
        assert_eq!(help_dropped(&groups, width as u16 - 1), 1);
        assert!(text(width as u16 - 1).iter().all(|r| !r.contains("F1")));
        assert!(text(width as u16 - 1)[1].contains("PgUp PgDn turn"));
    }

    #[test]
    fn bank_glyphs_mirror_the_pad_grid() {
        // every pad lit: two full glyphs
        assert_eq!(bank_glyphs(0xFFFF), "⣿⣿");
        // nothing lit: two blank glyphs keep the row aligned
        assert_eq!(bank_glyphs(0), "\u{2800}\u{2800}");
        // pad 1 (row 0, col 0): top-left dot of the first glyph
        assert_eq!(bank_glyphs(1), "\u{2801}\u{2800}");
        // pad 4 (row 1, col 0): second dot of the left column
        assert_eq!(bank_glyphs(1 << 4), "\u{2802}\u{2800}");
        // pad 5 (row 1, col 1): second dot of the right column
        assert_eq!(bank_glyphs(1 << 5), "\u{2810}\u{2800}");
        // pad 8 (row 1, col 3): second dot of the right column of the second glyph
        assert_eq!(bank_glyphs(1 << 7), "\u{2800}\u{2810}");
    }

    /// 16 idle pads, pad 6 (Q) selected, bank 1.
    fn idle_pads() -> Pads {
        let keys = [
            '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v',
        ];
        Pads {
            bank: 0,
            pads: keys
                .iter()
                .enumerate()
                .map(|(i, &key)| PadView {
                    naming: false,
                    glyphs: String::new(),
                    changed: Vec::new(),
                    dim: false,
                    dashed: false,
                    source: String::new(),
                    bounds: (0.0, 0.0),
                    filter: Default::default(),
                    wave: Vec::new(),
                    key,
                    name: format!("N{i}"),
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
                    note: None,
                    detail: None,
                })
                .collect(),
            selected: 5,
            pickup: None,
            scope: String::new(),
            param: PadParam::Volume,
            bank_fill: [0; BANKS],
            bank_marks: [0; BANKS],
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
            playing: false,
            feedback: PadFeedback::default(),
        }
    }

    /// The pads screen as cells: (symbol, fg, bg) per position.
    fn render_pads(p: Pads) -> Vec<Vec<(String, Color, Color)>> {
        render_pads_with(p, ([-60.0, -60.0], [-60.0, -60.0], false))
    }

    fn render_pads_with(
        p: Pads,
        meters: ([f32; 2], [f32; 2], bool),
    ) -> Vec<Vec<(String, Color, Color)>> {
        render_pads_rec(p, meters, false, 100, 26)
    }

    /// The pads screen at any size, for the wave-row tests.
    fn render_pads_at(p: Pads, w: u16, h: u16) -> Vec<Vec<(String, Color, Color)>> {
        render_pads_rec(p, ([-60.0, -60.0], [-60.0, -60.0], false), false, w, h)
    }

    fn render_pads_rec(
        p: Pads,
        meters: ([f32; 2], [f32; 2], bool),
        recording: bool,
        w: u16,
        h: u16,
    ) -> Vec<Vec<(String, Color, Color)>> {
        render_pads_snapshot(p, meters, recording, w, h, Vec::new())
    }

    fn render_pads_snapshot(
        p: Pads,
        meters: ([f32; 2], [f32; 2], bool),
        recording: bool,
        w: u16,
        h: u16,
        badges: Vec<Badge>,
    ) -> Vec<Vec<(String, Color, Color)>> {
        let status = Status {
            badges,
            save: SaveStatus::Clean,
            backend: Some("jack"),
            recording,
            buffer_size: 128,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters,
            no_releases: false,
            notice: None,
            project: None,
        };
        let groups = [HelpGroup {
            name: "PAD",
            items: vec![("Enter".into(), "mode: gate".into())],
        }];
        let help = Help {
            groups: &groups,
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| draw(f, &status, &Screen::Pads(p), &help))
            .unwrap();
        term.backend()
            .buffer()
            .content()
            .chunks(w as usize)
            .map(|row| {
                row.iter()
                    .map(|c| (c.symbol().to_string(), c.fg, c.bg))
                    .collect()
            })
            .collect()
    }

    /// Colors of the pad frames: the box-drawing cells of the grid.
    /// The colors of the pad frames: box lines inside the grid's columns (the panel beside
    /// it draws its own block lines, 48).
    fn frame_colors(rows: &[Vec<(String, Color, Color)>]) -> Vec<(Color, Color)> {
        const BOX: &str = "─│┌┐└┘├┤┬┴┼";
        let width = rows.first().map_or(0, |r| r.len()) as u16;
        let grid = pad_width(width, SIDE_W) as usize * 4;
        rows.iter()
            .flat_map(|r| r.iter().take(grid))
            .filter(|(s, _, _)| BOX.contains(s.as_str()))
            .map(|(_, fg, bg)| (*fg, *bg))
            .collect()
    }

    #[test]
    fn bank_colors_frames_and_row_follow_the_open_bank() {
        let p = idle_pads();
        // the bank row: every bank in its own color, the open one bold in brackets
        let spans = banks(&p);
        assert_eq!(spans.len(), 9, "\"bank\" plus eight banks");
        for (i, span) in spans.iter().skip(1).enumerate() {
            assert_eq!(span.style.fg, Some(BANK_COLORS[i]), "bank {}", i + 1);
        }
        assert!(spans[1].content.starts_with(" [1 "));
        assert!(
            spans[1].content.ends_with(']'),
            "the open bank is in brackets"
        );
        assert!(spans[1].style.add_modifier.contains(Modifier::BOLD));
        assert!(
            spans[2].style.add_modifier.is_empty(),
            "a closed bank is not bold"
        );

        // idle frames and key letters take the bank color, the selected pad stays cyan
        let rows = render_pads(idle_pads());
        let frames = frame_colors(&rows);
        assert!(!frames.is_empty(), "the grid drew its frames");
        for (fg, bg) in &frames {
            assert!(
                fg == &BANK_COLORS[0] || fg == &CYAN,
                "unexpected frame color {fg:?}"
            );
            assert_eq!(*bg, PANEL);
        }
        assert!(
            frames.iter().any(|(fg, _)| fg == &BANK_COLORS[0]),
            "idle frames are in the bank color"
        );
        let key = rows
            .iter()
            .flatten()
            .find(|(s, _, _)| s == "Z")
            .expect("the Z pad title");
        assert_eq!(
            key.1, BANK_COLORS[0],
            "the key letter of an idle pad is in the bank color"
        );

        // switching to bank 2 recolors the idle frames and the key letters
        let mut p2 = idle_pads();
        p2.bank = 1;
        let rows = render_pads(p2);
        let frames = frame_colors(&rows);
        for (fg, _) in &frames {
            assert!(
                fg == &BANK_COLORS[1] || fg == &CYAN,
                "unexpected frame color {fg:?}"
            );
        }
        let key = rows
            .iter()
            .flatten()
            .find(|(s, _, _)| s == "Z")
            .expect("the Z pad title");
        assert_eq!(key.1, BANK_COLORS[1]);
    }

    #[test]
    fn meanings_stay_above_the_bank_color() {
        let mut p = idle_pads();
        p.bank = 2; // bank 3
        p.pads[7].in_hand = true; // R
        p.pads[11].flash = true; // F is being hit
        let rows = render_pads(p);
        let frames = frame_colors(&rows);
        for (fg, bg) in &frames {
            if fg == &WHITE {
                assert_eq!(*bg, WHITE, "the hit lights the whole pad cell");
            } else {
                assert_eq!(*bg, PANEL);
                assert!(
                    fg == &CYAN || fg == &MAGENTA || fg == &BANK_COLORS[2],
                    "unexpected frame color {fg:?}"
                );
            }
        }
        for want in [CYAN, MAGENTA, WHITE, BANK_COLORS[2]] {
            assert!(
                frames.iter().any(|(fg, _)| fg == &want),
                "{want:?} missing from the frames"
            );
        }
        // key letters follow their state, not the bank
        let r = rows
            .iter()
            .flatten()
            .find(|(s, _, b)| s == "R" && *b == PANEL)
            .expect("the R pad title");
        assert_eq!(r.1, MAGENTA, "the pad in hand keeps magenta");
        let f = rows
            .iter()
            .flatten()
            .find(|(s, _, b)| s == "F" && *b == WHITE)
            .expect("the hit pad title");
        assert_eq!(f.1, BG, "the hit key letter stays dark on white");
    }

    #[test]
    /// The file's wave fills the shared strip (31), over the full width.
    fn files_preview_fills_the_strip() {
        let pads = Pads {
            bank: 0,
            pads: Vec::new(),
            selected: 0,
            pickup: None,
            scope: String::new(),
            param: PadParam::Volume,
            bank_fill: [0; BANKS],
            bank_marks: [0; BANKS],
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
            playing: false,
            feedback: PadFeedback::default(),
        };
        // quiet start, loud middle: the envelope must show the difference
        let wave: Vec<f32> = (0..1600).map(|i| if i < 800 { 0.1 } else { 1.0 }).collect();
        let f = Files {
            dir: "~/Music/breaks",
            entries: vec![("amen.wav", false, false)],
            selected: 0,
            in_hand: None,
            error: None,
            pads,
            filter: "",
            editing: false,
            preview: Some((5.22, &wave)),
            playhead: None,
        };
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: Some("jack"),
            recording: false,
            buffer_size: 128,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0, -60.0], [-60.0, -60.0], false),
            no_releases: false,
            notice: None,
            project: None,
        };
        let help = Help {
            groups: &[],
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Files(f), &help))
            .unwrap();
        let text: String = term
            .backend()
            .buffer()
            .content()
            .chunks(100)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
            .collect();
        // the length label sits on the first row of the strip
        let rows: Vec<&str> = text.lines().collect();
        let strip = frame_at(100, 30).strip;
        let (y, end) = (strip.y as usize, strip.bottom() as usize);
        assert!(end - y >= 7, "the strip takes the free rows");
        assert!(rows[y].contains("5.22 s"), "{}", rows[y]);
        // every row of the strip carries the waveform
        for row in &rows[y..end] {
            assert!(
                row.chars().any(|c| (c as u32) >= 0x2800),
                "no braille in {row:?}"
            );
        }
        // on the top row the quiet left half stays dark, the loud right half lights up
        let top: Vec<bool> = rows[y].chars().map(|c| (c as u32) >= 0x2801).collect();
        assert!(
            !top[12..50].iter().any(|d| *d),
            "quiet start reaches the top: {}",
            rows[y]
        );
        assert!(
            top[55..95].iter().any(|d| *d),
            "loud middle does not reach the top: {}",
            rows[y]
        );
    }

    #[test]
    fn one_slice_is_said_in_plain_words() {
        let pads = Pads {
            bank: 0,
            pads: Vec::new(),
            selected: 0,
            pickup: None,
            scope: String::new(),
            param: PadParam::Volume,
            bank_fill: [0; BANKS],
            bank_marks: [0; BANKS],
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
            playing: false,
            feedback: PadFeedback::default(),
        };
        let wave: Vec<f32> = vec![0.5; 100];
        let c = Chop {
            owners: Vec::new(),
            owner_bank: Vec::new(),
            source_banks: [false; BANKS],
            slice_keys: None,
            bands: &[],
            band_mean: [1.0; 3],
            file: "kick.wav",
            mode: "LISTEN".into(),
            seconds: 2.61,
            which: None,
            hits: Vec::new(),
            wave: &wave,
            bounds: vec![0.0, 1.0],
            selected: 0,
            cursor: None,
            playhead: None,
            play: None,
            info: "slice 1 · start 0.000 s · length 2610 ms".into(),
            side: vec!["Enter: assign".into()],
            banks: None,
            pads,
            edit: false,
            hot: None,
        };
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: None,
            recording: false,
            buffer_size: 0,
            sample_rate: 0,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0, -60.0], [-60.0, -60.0], false),
            no_releases: false,
            notice: None,
            project: None,
        };
        let help = Help {
            groups: &[],
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Chop(Box::new(c)), &help))
            .unwrap();
        let rows: Vec<String> = term
            .backend()
            .buffer()
            .content()
            .chunks(100)
            .map(|row| row.iter().map(|c| c.symbol()).collect())
            .collect();
        let text = rows.join("\n");
        assert!(text.contains("1 slice"), "{text}");
        assert!(!text.contains("1 slices"), "{text}");
        // the slice line wraps in the right column (31): read the column as one text
        let side = frame_at(100, 30).side;
        let column: String = rows[side.y as usize..side.bottom() as usize]
            .iter()
            .map(|r| r.chars().skip(side.x as usize).collect::<String>())
            .collect::<Vec<_>>()
            .join(" ");
        let words = column.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(words.contains("start 0.000 s · length"), "{words}");
    }

    /// Scenarios "Pending work has a visible save state" and
    /// "A save failure stays visible after its notice".
    #[test]
    fn a_save_failure_stays_visible_after_its_notice() {
        let wave = [0.1; 400];
        let screens = [
            Screen::Pads(idle_pads()),
            Screen::Files(Files {
                dir: "~/Music",
                entries: vec![("kick.wav", false, false)],
                selected: 0,
                in_hand: None,
                error: None,
                pads: idle_pads(),
                filter: "",
                editing: false,
                preview: None,
                playhead: None,
            }),
            Screen::Chop(Box::new(chop_demo(&wave))),
            Screen::Fx(shared_fx()),
            Screen::Settings(settings_view(0, 0)),
            Screen::ProjectPicker(ProjectPickerView {
                entries: vec!["beat".into()],
                selected: 0,
                naming: Some("new".into()),
            }),
        ];
        let groups = [HelpGroup {
            name: "CHECK",
            items: vec![("F1".into(), "help".into())],
        }];
        let learn = ["LEARN · select an action".into(), "Esc: cancel".into()];
        let full = [("F1".into(), "close help".into())];
        let question = ["Keep the current pads?".into(), "Esc: cancel".into()];
        for (w, h) in [(60, 24), (100, 26), (252, 60)] {
            for backend in [None, Some("jack")] {
                for screen in &screens {
                    for notice in [None, Some("write refused"), Some("another notice")] {
                        for mode in 0..4 {
                            let help = Help {
                                groups: &groups,
                                hint: Some("normal hint"),
                                full: (mode == 1).then_some(full.as_slice()),
                                question: (mode == 2).then_some(question.as_slice()),
                                learn: (mode == 3).then_some(learn.as_slice()),
                            };
                            let mut status = status_at(128, 0);
                            status.backend = backend;
                            status.notice = notice;
                            status.recording = true;
                            status.project = Some("beat");
                            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
                            term.draw(|f| draw(f, &status, screen, &help)).unwrap();
                            let baseline = term.backend().buffer().clone();
                            for save in [
                                SaveStatus::Unsaved,
                                SaveStatus::Failed,
                                SaveStatus::Clean,
                                SaveStatus::Unsaved,
                            ] {
                                status.save = save;
                                term.draw(|f| draw(f, &status, screen, &help)).unwrap();
                                let buffer = term.backend().buffer();
                                let top = buffer.content()[..w as usize]
                                    .iter()
                                    .map(|c| c.symbol())
                                    .collect::<String>();
                                assert_eq!(
                                    top.contains("Unsaved"),
                                    save == SaveStatus::Unsaved,
                                    "{w}x{h}: {top}"
                                );
                                assert_eq!(
                                    top.contains("Save failed"),
                                    save == SaveStatus::Failed,
                                    "{w}x{h}: {top}"
                                );
                                assert_eq!(
                                    &buffer.content()[w as usize..],
                                    &baseline.content()[w as usize..],
                                    "body, notices and help do not move"
                                );
                                if mode == 3 {
                                    assert!(top.contains("LEARN · select an action"));
                                    assert_eq!(buffer[(w - 1, 0)].bg, MAGENTA);
                                } else {
                                    assert!(top.contains("rec"));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn status_at(buffer_size: u32, xruns: u64) -> Status<'static> {
        Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: Some("jack"),
            recording: false,
            buffer_size,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns,
            master_db: 0,
            meters: ([-60.0; 2], [-60.0; 2], false),
            no_releases: false,
            notice: None,
            project: None,
        }
    }

    /// audio-engine "Start with PipeWire running": the status names the buffer, the rate
    /// and the latency in milliseconds.
    #[test]
    fn start_with_pipewire_running() {
        let text: String = status_line(&status_at(256, 0))
            .spans
            .iter()
            .map(|s| s.content.clone())
            .collect();
        assert!(text.contains("jack 256 @ 48000 Hz · 5.3 ms"), "{text}");
    }

    /// audio-engine "Buffer underrun": the xrun counter counts up and stands out.
    #[test]
    fn buffer_underrun() {
        let span = |xruns| {
            status_line(&status_at(256, xruns))
                .spans
                .into_iter()
                .find(|s| s.content.starts_with("xruns"))
                .unwrap()
        };
        let quiet = span(0);
        assert_eq!(quiet.content, "xruns 0");
        assert_ne!(quiet.style.fg, Some(MAGENTA));
        let late = span(3);
        assert_eq!(late.content, "xruns 3");
        assert_eq!(late.style.fg, Some(MAGENTA), "highlighted");
    }

    /// audio-settings "Microphone" (26): the input meter on F12 moves with the level. That
    /// the input stays out of the output is the engine's: its process writes the voices only.
    #[test]
    fn microphone() {
        let bars = |level: f32| meter(level).content.chars().filter(|&c| c == '█').count();
        assert_eq!(bars(0.0), 0, "silence: an empty meter");
        assert!(bars(0.05) > 0, "speaking moves it");
        assert!(bars(0.5) > bars(0.05), "louder, longer");
    }

    #[test]
    fn settings_say_honestly_when_jack_is_down() {
        let st = Settings {
            row: 0,
            output: None,
            output_missing: None,
            input: None,
            input_missing: None,
            input_level: 0.0,
            buffer_size: 1024,
            buffer_choice: 0,
            graphics: false,
            routing: false,
            backend_jack: true,
            tab: 0,
            midi_devices: &[],
            input_log: &[],
            midi_rows: &[],
            midi_row: 0,
            learning: None,
            paths: None,
            picker: None,
        };
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: None,
            recording: false,
            buffer_size: 0,
            sample_rate: 0,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0, -60.0], [-60.0, -60.0], false),
            no_releases: false,
            notice: None,
            project: None,
        };
        let help = Help {
            groups: &[],
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Settings(st), &help))
            .unwrap();
        let text: String = term
            .backend()
            .buffer()
            .content()
            .chunks(80)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
            .collect();
        assert!(text.contains("JACK is not available right now"), "{text}");
        assert!(!text.contains("set backend"), "{text}");
    }

    /// 28 midi-learn: the folded MIDI list on the drawing: a closed group is one row
    /// with its counts, an open group has its actions under it. 74: a waiting control is
    /// in the pending colour, the row that loses it in the taken colour, the `stays`
    /// lines in grey.
    #[test]
    fn the_midi_tab_folds_the_groups() {
        let st = Settings {
            row: 0,
            output: None,
            output_missing: None,
            input: None,
            input_missing: None,
            input_level: 0.0,
            buffer_size: 1024,
            buffer_choice: 0,
            graphics: false,
            routing: true,
            backend_jack: false,
            tab: 2,
            midi_devices: &[],
            input_log: &[],
            midi_rows: &[
                MidiRow::Group {
                    name: "GLOBAL",
                    counts: "9 actions · 1 learned".into(),
                },
                MidiRow::Group {
                    name: "PADS",
                    counts: "8 actions · 0 learned".into(),
                },
                MidiRow::Action {
                    label: "pad volume".into(),
                    who: "xsession-pro: k1 (cc 24)".into(),
                    taken: false,
                    stays: Vec::new(),
                },
                MidiRow::Action {
                    label: "pad mode".into(),
                    who: "X-Session Pro: b9 (note 69)  ✗ loses b9".into(),
                    taken: true,
                    stays: Vec::new(),
                },
                MidiRow::Action {
                    label: "marking on / off".into(),
                    who: "none → b9".into(),
                    taken: false,
                    stays: vec!["b9 on GLOBAL: bank next · stays".into()],
                },
                MidiRow::Group {
                    name: "CHOP",
                    counts: "14 actions · 2 learned".into(),
                },
            ],
            midi_row: 1,
            learning: None,
            paths: None,
            picker: None,
        };
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: None,
            recording: false,
            buffer_size: 0,
            sample_rate: 0,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0, -60.0], [-60.0, -60.0], false),
            no_releases: false,
            notice: None,
            project: None,
        };
        let help = Help {
            groups: &[],
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Settings(st), &help))
            .unwrap();
        let text: String = term
            .backend()
            .buffer()
            .content()
            .chunks(80)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
            .collect();
        assert!(text.contains("PADS · 8 actions · 0 learned"), "{text}");
        assert!(text.contains("CHOP · 14 actions · 2 learned"), "{text}");
        assert!(text.contains("pad volume"), "{text}");
        // the colour of the first cell of `what`, `skip` cells into it
        let fg = |what: &str, skip: usize| {
            let (y, line) = text
                .lines()
                .enumerate()
                .find(|(_, l)| l.contains(what))
                .unwrap_or_else(|| panic!("{what}: {text}"));
            let x = line[..line.find(what).unwrap()].chars().count() + skip;
            term.backend().buffer()[(x as u16, y as u16)].fg
        };
        assert_eq!(fg("✗ loses b9", 0), MAGENTA);
        assert_eq!(fg("pad mode", 0), MAGENTA);
        assert_eq!(fg("none → b9", 7), AMBER);
        assert_eq!(fg("b9 on GLOBAL: bank next · stays", 0), DIM);
    }

    #[test]
    fn the_buffer_row_explains_itself() {
        let st = Settings {
            row: 2,
            output: None,
            output_missing: None,
            input: None,
            input_missing: None,
            input_level: 0.0,
            buffer_size: 1024,
            buffer_choice: 0,
            graphics: false,
            routing: true,
            backend_jack: true,
            tab: 0,
            midi_devices: &[],
            input_log: &[],
            midi_rows: &[],
            midi_row: 0,
            learning: None,
            paths: None,
            picker: None,
        };
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: Some("jack"),
            recording: false,
            buffer_size: 1024,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0, -60.0], [-60.0, -60.0], false),
            no_releases: false,
            notice: None,
            project: None,
        };
        let help = Help {
            groups: &[],
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(90, 24)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Settings(st), &help))
            .unwrap();
        let text: String = term
            .backend()
            .buffer()
            .content()
            .chunks(90)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
            .collect();
        assert!(text.contains("auto (1024)"), "{text}");
        assert!(text.contains("follow the server"), "{text}");
        assert!(text.contains("pads feel late"), "{text}");
        assert!(text.contains("if clicks"), "{text}");
    }

    /// The settings screen on the given tab, as the tests read it back.
    fn settings_view(tab: usize, row: usize) -> Settings<'static> {
        Settings {
            row,
            output: None,
            output_missing: None,
            input: None,
            input_missing: None,
            input_level: 0.0,
            buffer_size: 1024,
            buffer_choice: 0,
            graphics: false,
            routing: true,
            backend_jack: true,
            tab,
            midi_devices: &[],
            input_log: &[],
            midi_rows: &[],
            midi_row: 0,
            learning: None,
            paths: None,
            picker: None,
        }
    }

    fn settings_text(st: Settings) -> String {
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: None,
            recording: false,
            buffer_size: 0,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0, -60.0], [-60.0, -60.0], false),
            no_releases: false,
            notice: None,
            project: None,
        };
        let help = Help {
            groups: &[],
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::Settings(st), &help))
            .unwrap();
        term.backend()
            .buffer()
            .content()
            .chunks(80)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
            .collect()
    }

    /// Scenario "Pick the roots" (11): the two Paths rows show the roots as the picker
    /// left them, `~`-shortened.
    #[test]
    fn pick_the_roots_the_rows_show_the_picked_folders() {
        let st = Settings {
            paths: Some(("~/beats/projects", "~/beats/samples")),
            ..settings_view(3, 0)
        };
        let text = settings_text(st);
        assert!(text.contains("[Paths]"), "{text}");
        assert!(text.contains("Projects root ~/beats/projects"), "{text}");
        assert!(text.contains("Samples root ~/beats/samples"), "{text}");
    }

    /// The project picker (11) as the screen body shows it, as the tests read it back.
    fn project_picker_text(p: ProjectPickerView) -> String {
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: None,
            recording: false,
            buffer_size: 0,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0, -60.0], [-60.0, -60.0], false),
            no_releases: false,
            notice: None,
            project: None,
        };
        let help = Help {
            groups: &[],
            hint: None,
            full: None,
            question: None,
            learn: None,
        };
        let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
        term.draw(|fr| draw(fr, &status, &Screen::ProjectPicker(p), &help))
            .unwrap();
        term.backend()
            .buffer()
            .content()
            .chunks(80)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
            .collect()
    }

    /// Scenario "Pick the roots" (11): the open picker lists the folders, highlights the
    /// chosen one and says how to pick and cancel; the rows hide under it.
    #[test]
    fn pick_the_roots_the_picker_lists_the_folders() {
        let entries = vec!["beats".to_string(), "Field".to_string()];
        let st = Settings {
            paths: Some(("~/beats/projects", "~/beats/samples")),
            picker: Some(PathPickerView {
                title: " PICK THE PROJECTS ROOT ",
                entries,
                selected: 1,
            }),
            ..settings_view(3, 1)
        };
        let text = settings_text(st);
        assert!(text.contains("PICK THE PROJECTS ROOT"), "{text}");
        assert!(text.contains("beats"), "{text}");
        assert!(text.contains("Field"), "{text}");
        assert!(text.contains("Enter pick · Esc cancel"), "{text}");
        assert!(!text.contains("Projects root"), "{text}");
    }

    /// Scenario "Open a project" (11): the picker takes the body, lists the projects with
    /// the chosen one highlighted, and says how to open one.
    #[test]
    fn project_picker_lists_the_projects() {
        let text = project_picker_text(ProjectPickerView {
            entries: vec!["MyBeat".to_string(), "Field".to_string()],
            selected: 1,
            naming: None,
        });
        assert!(text.contains(" PROJECTS "), "{text}");
        assert!(text.contains("MyBeat"), "{text}");
        assert!(text.contains("Field"), "{text}");
        assert!(text.contains("Enter open"), "{text}");
        assert!(!text.contains("name:"), "{text}");
    }

    /// Scenario "Create a project from the picker" (11): the typed name shows on the name
    /// line with the hint about creating; an empty root says there is nothing to open yet.
    #[test]
    fn project_picker_typing_names_a_new_project() {
        let text = project_picker_text(ProjectPickerView {
            entries: vec![],
            selected: 0,
            naming: Some("NewB".to_string()),
        });
        assert!(text.contains("no projects yet"), "{text}");
        assert!(text.contains("name: NewB"), "{text}");
        assert!(text.contains("Enter create"), "{text}");
        assert!(!text.contains("Enter open"), "{text}");
    }

    #[test]
    fn air_before_the_open_bank() {
        let mut p = idle_pads();
        p.bank = 1;
        let row: String = banks(&p).iter().map(|s| s.content.as_ref()).collect();
        assert!(row.contains(" [2 "), "{row}");
        let at = row.find('[').unwrap();
        assert_eq!(&row[at - 1..at], " ", "{row}");
    }

    #[test]
    fn bank_row_with_fill_fits_narrow_terminals() {
        let p = Pads {
            bank: 0,
            pads: Vec::new(),
            selected: 0,
            pickup: None,
            scope: String::new(),
            param: PadParam::Volume,
            playing: false,
            feedback: PadFeedback::default(),
            bank_fill: [0xFFFF; BANKS],
            bank_marks: [0; BANKS],
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
        };
        let row: String = banks(&p)
            .iter()
            .map(|s| s.content.as_ref())
            .collect::<String>();
        assert!(row.chars().count() <= 60, "row is {row:?}");
        assert!(row.contains("[1 ⣿⣿]"));
    }

    #[test]
    fn pads_grow_with_the_terminal_within_bounds() {
        assert_eq!(pad_width(80, 30), PAD_W_MIN);
        assert_eq!(pad_width(140, 30), 27);
        assert_eq!(pad_width(120, 40), 20);
        assert_eq!(pad_width(200, 30), 40);
        assert_eq!(pad_width(400, 40), PAD_W_MAX);
    }

    /// A row of the render as plain text.
    fn row_text(rows: &[Vec<(String, Color, Color)>], y: usize) -> String {
        rows[y].iter().map(|(s, _, _)| s.clone()).collect()
    }

    /// A cell that draws nothing: a space or a blank Braille cell.
    fn blank(c: char) -> bool {
        c == ' ' || c == '\u{2800}'
    }

    #[test]
    fn i_see_what_i_play() {
        let mut p = idle_pads();
        // a long quiet stretch (−50 dB), then a loud hit: the newest sound is at the right
        let mut wave = vec![0.003f32; 400];
        wave.extend([0.95; 100]);
        p.out_wave = wave;
        let rows = render_pads(p);
        let y = frame_at(100, 26).strip.y as usize;
        let (top, mid) = (row_text(&rows, y), row_text(&rows, y + 1));
        let ch = |t: &str, x: usize| t.chars().nth(x).unwrap();
        assert!(
            blank(ch(&top, 5)),
            "the quiet past stays near the middle, nothing tall on the left"
        );
        assert!(
            !blank(ch(&top, 95)),
            "the loud hit just played reaches the top on the right"
        );
        assert!(
            !blank(ch(&mid, 5)) && !blank(ch(&mid, 95)),
            "the wave runs on"
        );
    }

    #[test]
    fn the_strip_agrees_with_the_meters() {
        // −6 dB: on the meters' scale this is a tall bar, and the wave must rise as high
        let mut p = idle_pads();
        p.out_wave = vec![0.5];
        let rows = render_pads(p);
        let glyph = row_text(&rows, 2).chars().next().unwrap() as u32 - 0x2800;
        assert!(
            glyph.count_ones() >= 6,
            "a −6 dB peak stands near full height, like its meter bar"
        );
        // −40 dB reads low on the meters, and the wave stays off the top
        let mut p = idle_pads();
        p.out_wave = vec![0.01];
        let rows = render_pads(p);
        let glyph = row_text(&rows, 2).chars().next().unwrap() as u32 - 0x2800;
        assert_eq!(glyph, 0, "a −40 dB peak does not touch the top row");
    }

    #[test]
    fn quiet_and_loud_do_not_merge() {
        // a −40 dB wave stays in the middle band, a loud one fills the strip
        let f = frame_at(100, 26);
        let (y0, y1) = (f.strip.y as usize, f.strip.bottom() as usize - 1);
        let mut p = idle_pads();
        p.out_wave = vec![0.01];
        let rows = render_pads(p);
        let (top, bottom) = (row_text(&rows, y0), row_text(&rows, y1));
        assert!(
            blank(top.chars().next().unwrap()),
            "quiet does not reach the top"
        );
        assert!(
            blank(bottom.chars().next().unwrap()),
            "quiet does not reach the bottom either"
        );
        let mut p = idle_pads();
        p.out_wave = vec![0.95];
        let rows = render_pads(p);
        let (top, bottom) = (row_text(&rows, y0), row_text(&rows, y1));
        assert!(!blank(top.chars().next().unwrap()), "loud fills the top");
        assert!(
            !blank(bottom.chars().next().unwrap()),
            "loud fills the bottom"
        );
    }

    #[test]
    fn silence_is_a_line() {
        let mut p = idle_pads();
        p.out_wave = vec![0.0; 512];
        let rows = render_pads(p);
        let top = row_text(&rows, 2);
        let mid = row_text(&rows, 3);
        assert!(
            top.chars().take(64).all(blank),
            "silence draws nothing above the line"
        );
        assert!(
            mid.chars().take(64).any(|c| !blank(c)),
            "the middle line is there"
        );
    }

    #[test]
    fn meters_follow_the_sound() {
        let p = idle_pads();
        // L at -6 dB with its peak at -3, R at -12 dB with the peak under the bar
        let rows = render_pads_with(p, ([-6.0, -12.0], [-3.0, -12.0], false));
        // the meters top the right column (31)
        let y = frame_at(100, 26).side.y as usize;
        let (l, r) = (row_text(&rows, y), row_text(&rows, y + 1));
        let bars = |t: &str| t.chars().filter(|&c| c == '\u{2588}').count();
        assert!(bars(&l) > bars(&r), "the louder channel has the longer bar");
        assert!(l.contains("-6 dB"), "the dB number is next to the bar");
        assert!(l.contains('\u{258e}'), "the peak mark stands past the bar");
        assert!(
            !r.contains('\u{258e}'),
            "no mark when the peak is under the bar"
        );
    }

    #[test]
    fn overload_latches() {
        let y = frame_at(100, 26).side.y as usize + 2;
        let rows = render_pads_with(idle_pads(), ([-6.0, -6.0], [-6.0, -6.0], true));
        let light = rows[y]
            .iter()
            .find(|(s, _, _)| s == "C")
            .expect("the lit CLIP light");
        assert_eq!(light.2, MAGENTA, "the light is a magenta plate");
        let rows = render_pads_with(idle_pads(), ([-60.0, -60.0], [-60.0, -60.0], false));
        assert!(
            rows[y].iter().all(|(_, fg, _)| *fg != MAGENTA),
            "a clean output leaves the light dark"
        );
    }

    fn chop_status() -> (Status<'static>, Help<'static>) {
        (
            Status {
                badges: Vec::new(),
                save: SaveStatus::Clean,
                backend: Some("jack"),
                recording: false,
                buffer_size: 128,
                sample_rate: 48_000,
                load_peak: 0.0,
                xruns: 0,
                master_db: 0,
                meters: ([-60.0, -60.0], [-60.0, -60.0], false),
                no_releases: false,
                notice: None,
                project: None,
            },
            Help {
                groups: &[],
                hint: None,
                full: None,
                question: None,
                learn: None,
            },
        )
    }

    /// Scenario "Terminal without graphics" (04b): the waveforms stay Braille and every
    /// mark is drawn as ever — the plain `draw` is exactly the screen of today.
    #[test]
    fn terminal_without_graphics_draws_braille() {
        let wave: Vec<f32> = (0..4000).map(|i| (i % 1000) as f32 / 1000.0).collect();
        let (status, help) = chop_status();
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|fr| {
            draw(
                fr,
                &status,
                &Screen::Chop(Box::new(chop_demo(&wave))),
                &help,
            )
        })
        .unwrap();
        let rows: Vec<String> = term
            .backend()
            .buffer()
            .content()
            .chunks(100)
            .map(|row| row.iter().map(|c| c.symbol()).collect())
            .collect();
        let wave_row = &rows[4];
        assert!(
            wave_row.chars().any(|c| c as u32 >= 0x2800),
            "the wave is Braille"
        );
        assert_eq!(wave_row.chars().nth(25), Some('│'), "the slice bound");
        assert_eq!(wave_row.chars().nth(60), Some('┃'), "the cursor");
    }

    /// Scenario "Graphic waveform" (04b): with graphics the wave cells are left blank
    /// for the picture under them, the marks stay, and the picture is described with its
    /// id, its area and the same colours the Braille wave has.
    #[test]
    fn graphics_leave_the_wave_cells_to_the_picture() {
        let wave: Vec<f32> = (0..4000).map(|i| (i % 1000) as f32 / 1000.0).collect();
        let (status, help) = chop_status();
        let mut images = Vec::new();
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|fr| {
            draw_gfx(
                fr,
                &status,
                &Screen::Chop(Box::new(chop_demo(&wave))),
                &help,
                &mut images,
            )
        })
        .unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].id, 1);
        // the wave is the strip under its two rows of marks
        let strip = frame_at(100, 30).strip;
        assert_eq!(
            (images[0].area.width, images[0].area.height),
            (100, strip.height - 2)
        );
        assert_eq!(images[0].peaks.len(), 2 * 100, "a peak per dot column");
        // the colours follow the Braille ones: dim off-range, bright in the picked slice
        assert_eq!(images[0].colors[10], DIM);
        assert_eq!(images[0].colors[30], TEXT);
        let rows: Vec<String> = term
            .backend()
            .buffer()
            .content()
            .chunks(100)
            .map(|row| row.iter().map(|c| c.symbol()).collect())
            .collect();
        let wave_row = &rows[4];
        assert!(
            wave_row.chars().enumerate().all(|(x, c)| {
                matches!((x, c), (25, '│') | (50, '│') | (75, '│') | (60, '┃')) | (c == ' ')
            }),
            "only the marks over blank cells: {wave_row}"
        );
        let numbers = &rows[2];
        assert!(
            numbers.contains('!'),
            "the cursor mark stays above the wave"
        );
    }
}

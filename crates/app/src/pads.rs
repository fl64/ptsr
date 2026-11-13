//! Pad state on the UI side: bank, selection, modes, volumes. Turns pad actions into engine commands.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use engine::filter::{CUTOFF_MAX, CUTOFF_MIN, FilterKind, FilterSettings};
use engine::fx::{FEEDBACK_MAX, FxKind, FxSettings};
use engine::{Command, FX_SLOTS, PADS, PADS_PER_BANK, PITCH_LIMIT, PadMode, Region, Sample};
use input::{Action, Dir};

use crate::session::{FxState, PadState, PatternState, Session};

/// Everything a pad has, as the undo keeps it (61): the sound by reference.
#[derive(Clone, Debug)]
pub struct PadSnap {
    pub(crate) file: Option<PathBuf>,
    pub(crate) name: Option<String>,
    missing: bool,
    gate: bool,
    pitch_gate: bool,
    volume_db: i8,
    pitch: i8,
    env: [u16; 3],
    filter: (FilterKind, f32, u8),
    drive: u8,
    sends: [u8; FX_SLOTS],
    group: u8,
    sound: Option<Arc<Sample>>,
    region: Region,
    pub(crate) slice: Option<u16>,
    pub(crate) link: Option<(u32, u32)>,
}

impl PartialEq for PadSnap {
    fn eq(&self, o: &Self) -> bool {
        let same_sound = match (&self.sound, &o.sound) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        // a tuple compares at most 12 fields
        same_sound
            && self.name == o.name
            && self.group == o.group
            && self.pitch_gate == o.pitch_gate
            && (
                &self.file,
                self.missing,
                self.gate,
                self.volume_db,
                self.pitch,
                self.env,
                self.filter,
                self.drive,
                self.sends,
                self.region,
                self.slice,
                self.link,
            ) == (
                &o.file,
                o.missing,
                o.gate,
                o.volume_db,
                o.pitch,
                o.env,
                o.filter,
                o.drive,
                o.sends,
                o.region,
                o.slice,
                o.link,
            )
    }
}

/// A shared effect slot of the project (56): the kind it holds and the parameters of every
/// kind, so a kind turned back finds its own as they were.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FxSlot {
    pub kind: FxKind,
    /// Reverb size and tone, in percent.
    pub size: u8,
    pub tone: u8,
    /// Delay time in ms and feedback in percent.
    pub time_ms: u16,
    pub feedback: u8,
}

impl FxSlot {
    /// A new project: a reverb in FX 1, a delay in FX 2.
    pub const DEFAULTS: [FxSlot; FX_SLOTS] =
        [FxSlot::new(FxKind::Reverb), FxSlot::new(FxKind::Delay)];

    const fn new(kind: FxKind) -> Self {
        Self {
            kind,
            size: 50,
            tone: 50,
            time_ms: 250,
            feedback: 40,
        }
    }

    pub fn settings(&self) -> FxSettings {
        match self.kind {
            FxKind::Reverb => FxSettings::Reverb {
                size: self.size as f32 / 100.0,
                tone: self.tone as f32 / 100.0,
            },
            FxKind::Delay => FxSettings::Delay {
                time_ms: self.time_ms as f32,
                feedback: self.feedback as f32 / 100.0,
            },
        }
    }

    pub fn state(&self) -> FxState {
        FxState {
            kind: kind_word(self.kind).into(),
            size: self.size,
            tone: self.tone,
            time_ms: self.time_ms,
            feedback: self.feedback,
        }
    }

    /// From the project file; an unknown kind or out-of-range values fall back safely.
    pub fn from_state(s: &FxState) -> Self {
        let max_fb = (FEEDBACK_MAX * 100.0) as u8;
        Self {
            kind: if s.kind == "delay" {
                FxKind::Delay
            } else {
                FxKind::Reverb
            },
            size: s.size.min(100),
            tone: s.tone.min(100),
            time_ms: s.time_ms.clamp(10, 2_000),
            feedback: s.feedback.min(max_fb),
        }
    }
}

/// The delay time on a knob: 10..2000 ms on a square curve, the short times get room.
fn time_of(pos: f32) -> u16 {
    (10.0 + pos * pos * 1_990.0).round() as u16
}

fn time_pos(ms: u16) -> f32 {
    ((ms.saturating_sub(10)) as f32 / 1_990.0).sqrt()
}

/// The word of a slot kind in the project file and on screen.
pub fn kind_word(kind: FxKind) -> &'static str {
    match kind {
        FxKind::Reverb => "reverb",
        FxKind::Delay => "delay",
    }
}

const BANKS: usize = PADS / PADS_PER_BANK;
/// Volume range of pads and the master, in dB.
pub const MIN_DB: i8 = -40;
pub const MAX_DB: i8 = 6;

/// Keyboard keys of the 16 pads of a bank, shown in the grid and the panel.
pub const PAD_KEYS: [char; PADS_PER_BANK] = [
    '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v',
];

/// Only keyboard hits can set the target used by a keyboard release.
#[derive(Clone, Copy)]
pub enum HitSource {
    Keyboard,
    Midi,
}

/// The parameter the pads screen changes with `Alt+←` `Alt+→` (the arrows in the edit
/// mode); picked with `Tab`. UI state, not saved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Param {
    #[default]
    Volume,
    Pitch,
    /// The mute group (70): 0 is off, 1–8.
    Group,
    Attack,
    Decay,
    Release,
    /// The SOURCE page (27): what the pad plays and its own start and end; the app
    /// changes them, it knows the cuts of the file.
    Source,
    Start,
    End,
    /// The FILTER page (44): the kind, the cutoff and the resonance.
    Filter,
    Cutoff,
    Reso,
    /// The FX page (56): the pad's drive and its sends to the two shared slots.
    Drive,
    Send1,
    Send2,
    /// The pages of the shared slots (56): the kind, then the two parameters of that kind
    /// (reverb: size, tone; delay: time, feedback).
    Fx1Kind,
    Fx1A,
    Fx1B,
    Fx2Kind,
    Fx2A,
    Fx2B,
}

impl Param {
    /// The shared slot a parameter sets (56): those pages act on every pad.
    pub fn slot(self) -> Option<usize> {
        match self {
            Param::Fx1Kind | Param::Fx1A | Param::Fx1B => Some(0),
            Param::Fx2Kind | Param::Fx2A | Param::Fx2B => Some(1),
            _ => None,
        }
    }
}

/// Envelope step of the value keys and its limits, in ms; `Ctrl` steps 1 ms. Volume and
/// pitch have no finer step than 1 dB and 1 semitone: the session keeps them whole.
pub const ENV_STEP_MS: i32 = 5;
const ENV_FINE_MS: i32 = 1;

/// The pages of the pad edit mode: knob k turns the k-th parameter of the page.
const PAGES: [(&str, &[Param]); 5] = [
    ("TONE", &[Param::Volume, Param::Pitch, Param::Group]),
    ("ENV", &[Param::Attack, Param::Decay, Param::Release]),
    ("FILTER", &[Param::Filter, Param::Cutoff, Param::Reso]),
    // the pad's own bounds after the sound: where it plays from (48)
    ("SOURCE", &[Param::Source, Param::Start, Param::End]),
    ("FX", &[Param::Drive, Param::Send1, Param::Send2]),
];

/// The name of a pad parameter in the panel and the help line.
pub fn name_of(p: Param) -> &'static str {
    match p {
        Param::Volume => "volume",
        Param::Pitch => "pitch",
        Param::Group => "mute group",
        Param::Attack => "attack",
        Param::Decay => "decay",
        Param::Release => "release",
        Param::Source => "source",
        Param::Start => "start",
        Param::End => "end",
        Param::Filter => "filter",
        Param::Cutoff => "cutoff",
        Param::Reso => "reso",
        Param::Drive => "drive",
        Param::Send1 => "send 1",
        Param::Send2 => "send 2",
        Param::Fx1Kind | Param::Fx2Kind => "type",
        // the names of a slot's parameters follow its kind: see `Pads::name`
        Param::Fx1A | Param::Fx2A => "fx a",
        Param::Fx1B | Param::Fx2B => "fx b",
    }
}

/// The page names in `Tab` order, for the edit frame title (43).
pub fn page_names() -> Vec<&'static str> {
    PAGES.iter().map(|(name, _)| *name).collect()
}

/// All parameters in `Tab` order, for the learn mode's panel (47).
pub const ORDER_ALL: [Param; 15] = ORDER;

/// All parameters in `I` `K` order.
const ORDER: [Param; 15] = [
    Param::Volume,
    Param::Pitch,
    Param::Group,
    Param::Attack,
    Param::Decay,
    Param::Release,
    Param::Filter,
    Param::Cutoff,
    Param::Reso,
    Param::Source,
    Param::Start,
    Param::End,
    Param::Drive,
    Param::Send1,
    Param::Send2,
];

/// Mute groups 1…`GROUPS` (70); 0 is off.
pub const GROUPS: u8 = 8;

/// The mute group as the panel says it: `off`, `1`…`8`.
pub fn group_text(g: u8) -> String {
    if g == 0 { "off".into() } else { g.to_string() }
}

/// The cutoff of a new pad, in Hz.
pub const CUTOFF_DEFAULT: f32 = 1_000.0;
/// The filter kinds in the order the value keys step them.
const KINDS: [FilterKind; 4] = [
    FilterKind::Off,
    FilterKind::LowPass,
    FilterKind::HighPass,
    FilterKind::Band,
];

/// The name of a filter kind in the panel and the project file.
pub fn kind_name(k: FilterKind) -> &'static str {
    match k {
        FilterKind::Off => "off",
        FilterKind::LowPass => "low-pass",
        FilterKind::HighPass => "high-pass",
        FilterKind::Band => "band",
    }
}

fn kind_of(name: &str) -> FilterKind {
    KINDS
        .into_iter()
        .find(|&k| kind_name(k) == name)
        .unwrap_or(FilterKind::Off)
}

/// A cutoff as the panel says it: `600 Hz`, `1.2 kHz`.
pub fn cutoff_text(hz: f32) -> String {
    if hz < 1_000.0 {
        format!("{hz:.0} Hz")
    } else {
        format!("{:.1} kHz", hz / 1_000.0)
    }
}
pub const ENV_MS_LIMIT: i32 = 5000;

/// What a pad with a sample plays (27), as an MPC pad layer's Slice: the whole file, a
/// slice that follows the file's cuts, or its own start and end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    All,
    Slice(u16),
    Own,
}

/// The piece a pad plays: the region's peaks and the envelope over them, `cols` values
/// each, 0..1, and the piece's length in ms. Shared by the big view and the cell waves.
pub(crate) fn piece(
    sample: &Sample,
    region: Region,
    rate: u32,
    attack_ms: u16,
    decay_ms: u16,
    cols: usize,
) -> (Vec<f32>, Vec<f32>, f32) {
    let (start, end) = (region.start, region.end.min(sample.len()));
    let wave = crate::loader::envelope_region(sample, start, end, cols);
    let dur_ms = end.saturating_sub(start) as f32 / rate.max(1) as f32 * 1000.0;
    let env = crate::env_curve(dur_ms, attack_ms, decay_ms, cols);
    (wave, env, dur_ms)
}

/// The cached cell wave of a pad (29): the sound, region, envelope and width it was
/// computed from. The entry keeps the sample alive, so no other allocation can land on
/// the same pointer while cached and the identity check stays sound.
struct PadWave {
    sound: Arc<Sample>,
    region: Region,
    attack_ms: u16,
    decay_ms: u16,
    cols: usize,
    wave: Vec<f32>,
}

pub struct Pads {
    pub bank: usize,
    /// Selected pad 0..16 of the current bank.
    pub selected: usize,
    pub gate: [bool; PADS],
    pub pitch_gate: [bool; PADS],
    /// The saved raw A1 data; playback uses a separate compiled list.
    pub pattern: Option<PatternState>,
    pub(crate) pattern_rows: Option<Vec<engine::pattern::PlaybackRow>>,
    pub volume_db: [i8; PADS],
    /// Pad pitch in semitones, −`PITCH_LIMIT`…+`PITCH_LIMIT`.
    pub pitch: [i8; PADS],
    /// Pad envelope times in ms: fade-in at the piece start, fade-to-silence at
    /// its end, fade after a gate release. Zero attack or decay means off,
    /// zero release means the engine's short cut.
    pub attack_ms: [u16; PADS],
    pub decay_ms: [u16; PADS],
    pub release_ms: [u16; PADS],
    /// The pad filter (44): kind, cutoff in Hz, resonance in percent.
    pub filter: [FilterKind; PADS],
    pub cutoff_hz: [f32; PADS],
    pub reso: [u8; PADS],
    /// The pad drive and its two sends, in percent (56).
    pub drive: [u8; PADS],
    /// The mute group of each pad (70): 0 is off, 1–`GROUPS`.
    pub group: [u8; PADS],
    pub sends: [[u8; FX_SLOTS]; PADS],
    /// The two shared effect slots of the project (56).
    pub fx: [FxSlot; FX_SLOTS],
    /// The parameter `-` `=` change right now.
    pub param: Param,
    /// Marked pads, absolute; the changes act on them instead of the selected pad (76).
    pub marked: BTreeSet<usize>,
    /// Marking is on: a pad hit marks or unmarks the pad.
    pub marking: bool,
    /// How many pads of the scope sat at a limit during the last parameter change;
    /// `None` when none did. The UI turns it into a pop-up.
    pub limit_hits: Option<usize>,
    /// Last hit of each pad, for the flash on screen.
    pub hit_at: [Option<Instant>; PADS],
    /// Pads held down right now, lit on screen.
    pub held: [bool; PADS],
    /// Keyboard presses mark pads held; off without the kitty protocol, which has no releases.
    pub track_keys: bool,
    /// The file each pad plays; `None` plays the test tone.
    pub file: [Option<PathBuf>; PADS],
    pub custom_name: [Option<String>; PADS],
    /// The pad's file is gone or broken; the pad is silent.
    pub missing: [bool; PADS],
    /// Loading progress, 0..100, while a file is being loaded for the pad.
    pub loading: [Option<u8>; PADS],
    /// The loaded sample, shared with the engine; slices of one file share one sample.
    pub sound: [Option<Arc<Sample>>; PADS],
    /// The part of the sample the pad plays, in frames at the engine rate.
    pub region: [Region; PADS],
    /// Slice number (from 1) when the pad holds a slice; shown after the file name.
    pub slice: [Option<u16>; PADS],
    /// The pad follows the cuts of its file: (cut id at the slice start, chop epoch).
    /// `None` for a whole file or a free slice that only keeps its own edges.
    pub link: [Option<(u32, u32)>; PADS],
    /// Which absolute pad each key pressed, so the release goes to the same pad
    /// even if the bank changed while the key was held.
    pressed: [usize; PADS_PER_BANK],
    /// Cell wave of each pad (29); see [`Pads::wave`].
    wave_cache: [Option<PadWave>; PADS],
}

impl Default for Pads {
    fn default() -> Self {
        Self {
            bank: 0,
            selected: 0,
            gate: [false; PADS],
            pitch_gate: [true; PADS],
            pattern: None,
            pattern_rows: None,
            volume_db: [0; PADS],
            pitch: [0; PADS],
            attack_ms: [0; PADS],
            decay_ms: [0; PADS],
            release_ms: [0; PADS],
            filter: [FilterKind::Off; PADS],
            cutoff_hz: [CUTOFF_DEFAULT; PADS],
            reso: [0; PADS],
            drive: [0; PADS],
            group: [0; PADS],
            sends: [[0; FX_SLOTS]; PADS],
            fx: FxSlot::DEFAULTS,
            param: Param::default(),
            marked: BTreeSet::new(),
            marking: false,
            limit_hits: None,
            hit_at: [None; PADS],
            held: [false; PADS],
            track_keys: true,
            file: std::array::from_fn(|_| None),
            custom_name: std::array::from_fn(|_| None),
            missing: [false; PADS],
            loading: [None; PADS],
            sound: std::array::from_fn(|_| None),
            region: [Region::WHOLE; PADS],
            slice: [None; PADS],
            link: [None; PADS],
            pressed: std::array::from_fn(|i| i),
            wave_cache: std::array::from_fn(|_| None),
        }
    }
}

impl Pads {
    /// Absolute pad index of the selected pad.
    pub fn selected_abs(&self) -> usize {
        self.bank * PADS_PER_BANK + self.selected
    }

    /// The wave a pad's cell draws (29): its region's peaks with the envelope over them,
    /// `cols` columns, 0..1. Kept per pad, computed again only when the sound, the region,
    /// the envelope or the width changes. A frame never scans the sample. An empty pad
    /// draws a flat line, a missing file nothing; either drops its cached entry.
    pub fn wave(&mut self, abs: usize, rate: u32, cols: usize) -> Vec<f32> {
        if self.missing[abs] {
            self.wave_cache[abs] = None;
            return Vec::new();
        }
        let Some(sound) = self.sound[abs].clone() else {
            self.wave_cache[abs] = None;
            return vec![0.0];
        };
        let (region, attack_ms, decay_ms) =
            (self.region[abs], self.attack_ms[abs], self.decay_ms[abs]);
        let fresh = self.wave_cache[abs].as_ref().is_some_and(|c| {
            Arc::ptr_eq(&c.sound, &sound)
                && c.region == region
                && c.attack_ms == attack_ms
                && c.decay_ms == decay_ms
                && c.cols == cols
        });
        if !fresh {
            let (peaks, env, _) = piece(&sound, region, rate, attack_ms, decay_ms, cols);
            let wave = peaks.iter().zip(&env).map(|(w, e)| w * e).collect();
            self.wave_cache[abs] = Some(PadWave {
                sound,
                region,
                attack_ms,
                decay_ms,
                cols,
                wave,
            });
        }
        self.wave_cache[abs]
            .as_ref()
            .map_or_else(Vec::new, |c| c.wave.clone())
    }

    /// A hit from a controller (the audio thread already has it): flash, and select it
    /// when it is in the current bank.
    pub fn hit(&mut self, abs: usize) {
        if abs < PADS {
            self.hit_at[abs] = Some(Instant::now());
            self.held[abs] = true;
            if abs / PADS_PER_BANK == self.bank {
                self.selected = abs % PADS_PER_BANK;
            }
            self.mark_hit(abs);
        }
    }

    /// While marking is on, a hit marks or unmarks the pad; the pad still sounds.
    fn mark_hit(&mut self, abs: usize) {
        if self.marking && !self.marked.insert(abs) {
            self.marked.remove(&abs);
        }
    }

    /// The pads a parameter or the mode change acts on right now.
    pub fn scope_pads(&self) -> Vec<usize> {
        if self.marked.is_empty() {
            vec![self.selected_abs()]
        } else {
            self.marked.iter().copied().collect()
        }
    }

    /// What the panel says about the scope: `pad Q` or `3 marked`.
    pub fn scope_name(&self) -> String {
        if self.marked.is_empty() {
            format!("pad {}", PAD_KEYS[self.selected].to_ascii_uppercase())
        } else {
            format!("{} marked", self.marked.len())
        }
    }

    /// Esc: drops the marks and turns marking off. `true` when there was something to drop
    /// and Esc is consumed.
    pub fn drop_marks(&mut self) -> bool {
        if self.marked.is_empty() && !self.marking {
            return false;
        }
        self.marked.clear();
        self.marking = false;
        true
    }

    /// Marks the 16 pads of the open bank and turns marking on; when all of them are marked
    /// already, unmarks them (76).
    fn mark_bank(&mut self) {
        let pads = self.bank * PADS_PER_BANK..(self.bank + 1) * PADS_PER_BANK;
        if pads.clone().all(|abs| self.marked.contains(&abs)) {
            for abs in pads {
                self.marked.remove(&abs);
            }
        } else {
            self.marked.extend(pads);
        }
        self.marking = true;
    }

    /// Takes the sound off a pad: file, slice, link; the mode, volume and pitch stay.
    pub fn clear(&mut self, abs: usize) -> Command {
        self.file[abs] = None;
        self.custom_name[abs] = None;
        self.missing[abs] = false;
        self.loading[abs] = None;
        self.slice[abs] = None;
        self.link[abs] = None;
        self.region[abs] = Region::WHOLE;
        self.sound[abs] = None;
        Command::SetPadSound {
            pad: abs,
            sound: None,
        }
    }

    fn compile_pattern(
        pattern: Option<&PatternState>,
        descriptors: Option<&[engine::pattern::PlaybackRow]>,
    ) -> Result<Option<Arc<engine::pattern::Pattern>>, String> {
        pattern
            .map(|state| {
                let raw = state.checked()?;
                let compiled = match descriptors {
                    Some(rows) => raw.compile_recorded(rows).map_err(|e| e.to_string())?,
                    None => raw.compile(),
                };
                Ok(Arc::new(compiled))
            })
            .transpose()
    }

    pub fn recorded_pattern_command(&self) -> Result<Command, String> {
        Ok(Command::PublishRecordedPattern {
            pattern: Self::compile_pattern(self.pattern.as_ref(), self.pattern_rows.as_deref())?
                .ok_or("Recording needs A1.")?,
        })
    }

    pub fn restore_recorded_pattern(
        &mut self,
        pattern: &PatternState,
        rows: &[engine::pattern::PlaybackRow],
    ) -> Result<Command, String> {
        let compiled =
            Self::compile_pattern(Some(pattern), Some(rows))?.ok_or("Recording needs A1.")?;
        self.pattern = Some(pattern.clone());
        self.pattern_rows = Some(rows.to_vec());
        Ok(Command::PublishRecordedPattern { pattern: compiled })
    }

    pub fn prepare_pattern_record(&mut self) -> Result<Command, String> {
        use engine::pattern::{PlaybackRow, RowId};
        let pattern = self.pattern.clone().unwrap_or_else(|| PatternState {
            slot: "A1".into(),
            bars: 2,
            quantize: "1/16".into(),
            strength: 100,
            hits: Vec::new(),
        });
        let rows = match &self.pattern_rows {
            Some(rows) => rows.clone(),
            None => (0..pattern.hits.len())
                .map(|slot| RowId::new(slot, 1).map(PlaybackRow::existing))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?,
        };
        let compiled =
            Self::compile_pattern(Some(&pattern), Some(&rows))?.ok_or("Recording needs A1.")?;
        self.pattern = Some(pattern);
        self.pattern_rows = Some(rows);
        Ok(Command::PublishRecordedPattern { pattern: compiled })
    }

    /// Sounds and all raw hit addresses move in one audio transaction.
    pub fn swap(&mut self, a: usize, b: usize) -> Result<Command, String> {
        if a >= PADS || b >= PADS {
            return Err("Cannot swap an invalid pad address.".into());
        }
        let mut pattern = self.pattern.clone();
        if let Some(state) = &mut pattern {
            for row in &mut state.hits {
                if row[1] == a as u32 {
                    row[1] = b as u32;
                } else if row[1] == b as u32 {
                    row[1] = a as u32;
                }
            }
        }
        let compiled = Self::compile_pattern(pattern.as_ref(), self.pattern_rows.as_deref())?;
        self.pattern = pattern;
        self.file.swap(a, b);
        self.custom_name.swap(a, b);
        self.missing.swap(a, b);
        self.gate.swap(a, b);
        self.pitch_gate.swap(a, b);
        self.volume_db.swap(a, b);
        self.pitch.swap(a, b);
        self.attack_ms.swap(a, b);
        self.decay_ms.swap(a, b);
        self.release_ms.swap(a, b);
        // every setting moves with the pad, as in the engine (60)
        self.filter.swap(a, b);
        self.cutoff_hz.swap(a, b);
        self.reso.swap(a, b);
        self.drive.swap(a, b);
        self.sends.swap(a, b);
        self.group.swap(a, b);
        self.sound.swap(a, b);
        self.region.swap(a, b);
        self.slice.swap(a, b);
        self.link.swap(a, b);
        self.wave_cache.swap(a, b);
        Ok(if self.pattern.is_some() {
            Command::SwapPatternPads {
                a,
                b,
                pattern: compiled,
            }
        } else {
            Command::SwapPads { a, b }
        })
    }

    /// Everything a pad has, for the undo (61).
    pub fn snapshot(&self, abs: usize) -> PadSnap {
        PadSnap {
            file: self.file[abs].clone(),
            name: self.custom_name[abs].clone(),
            missing: self.missing[abs],
            gate: self.gate[abs],
            pitch_gate: self.pitch_gate[abs],
            volume_db: self.volume_db[abs],
            pitch: self.pitch[abs],
            env: [
                self.attack_ms[abs],
                self.decay_ms[abs],
                self.release_ms[abs],
            ],
            filter: (self.filter[abs], self.cutoff_hz[abs], self.reso[abs]),
            drive: self.drive[abs],
            sends: self.sends[abs],
            group: self.group[abs],
            sound: self.sound[abs].clone(),
            region: self.region[abs],
            slice: self.slice[abs],
            link: self.link[abs],
        }
    }

    /// Resets a pad to a freshly loaded one (76): volume, mode, pitch, envelope, filter,
    /// drive, sends and mute group at their defaults; the sound and its source stay. The
    /// sound is not sent again, so a sounding voice only glides to the new values.
    pub fn reset_clean(&mut self, abs: usize) -> Vec<Command> {
        let now = self.snapshot(abs);
        let clean = PadSnap {
            file: now.file,
            name: now.name,
            missing: now.missing,
            sound: now.sound,
            region: now.region,
            slice: now.slice,
            link: now.link,
            ..Pads::default().snapshot(abs)
        };
        let loading = self.loading[abs];
        let mut cmds = self.put_back(abs, &clean);
        self.loading[abs] = loading;
        cmds.retain(|c| {
            !matches!(
                c,
                Command::SetPadSound { .. } | Command::SetPadRegion { .. }
            )
        });
        cmds
    }

    /// Puts a pad back as a snapshot has it; returns the engine commands that make it
    /// sound so again.
    pub fn put_back(&mut self, abs: usize, s: &PadSnap) -> Vec<Command> {
        self.file[abs] = s.file.clone();
        self.custom_name[abs] = s.name.clone();
        self.missing[abs] = s.missing;
        self.gate[abs] = s.gate;
        self.pitch_gate[abs] = s.pitch_gate;
        self.volume_db[abs] = s.volume_db;
        self.pitch[abs] = s.pitch;
        [
            self.attack_ms[abs],
            self.decay_ms[abs],
            self.release_ms[abs],
        ] = s.env;
        (self.filter[abs], self.cutoff_hz[abs], self.reso[abs]) = s.filter;
        self.drive[abs] = s.drive;
        self.sends[abs] = s.sends;
        self.group[abs] = s.group;
        self.sound[abs] = s.sound.clone();
        self.region[abs] = s.region;
        self.slice[abs] = s.slice;
        self.link[abs] = s.link;
        self.loading[abs] = None;
        self.wave_cache[abs] = None;
        vec![
            Command::SetPadMode {
                pad: abs,
                mode: if s.gate {
                    PadMode::Gate
                } else {
                    PadMode::OneShot
                },
            },
            Command::SetPadGain {
                pad: abs,
                gain: 10f32.powf(s.volume_db as f32 / 20.0),
            },
            Command::SetPadPitch {
                pad: abs,
                semitones: s.pitch,
            },
            self.pitch_gate_command(abs),
            self.env_command(abs),
            self.filter_command(abs),
            self.drive_command(abs),
            self.sends_command(abs),
            self.group_command(abs),
            // the sound first: a new sound resets the region in the engine
            Command::SetPadSound {
                pad: abs,
                sound: s.sound.clone(),
            },
            Command::SetPadRegion {
                pad: abs,
                region: s.region,
            },
        ]
    }

    /// Puts a loaded sound on the pad.
    pub fn set_sound(&mut self, abs: usize, sound: Arc<Sample>) {
        self.sound[abs] = Some(sound);
    }

    /// Takes the sound off the pad (a failed load).
    pub fn clear_sound(&mut self, abs: usize) {
        self.sound[abs] = None;
    }

    /// Session snapshot of every pad that is not at its defaults; `rate` turns slice frames
    /// into seconds.
    pub fn to_session(&self, browser_dir: Option<PathBuf>, rate: u32) -> Session {
        let mut s = Session {
            browser_dir,
            bank: self.bank as u8,
            fx: self.fx.iter().map(FxSlot::state).collect(),
            pattern: self.pattern.as_ref().map(|p| p.slot.clone()),
            patterns: self.pattern.iter().cloned().collect(),
            ..Default::default()
        };
        for abs in 0..PADS {
            let r = self.region[abs];
            let secs = |frames: usize| frames as f64 / rate.max(1) as f64;
            let state = PadState {
                file: self.file[abs].clone(),
                name: self.custom_name[abs].clone(),
                gate: self.gate[abs],
                pitch_gate: self.pitch_gate[abs],
                volume_db: self.volume_db[abs],
                pitch: self.pitch[abs],
                attack_ms: self.attack_ms[abs],
                decay_ms: self.decay_ms[abs],
                release_ms: self.release_ms[abs],
                filter: match self.filter[abs] {
                    FilterKind::Off => String::new(),
                    k => kind_name(k).to_string(),
                },
                cutoff_hz: (self.cutoff_hz[abs] != CUTOFF_DEFAULT).then_some(self.cutoff_hz[abs]),
                reso: self.reso[abs],
                drive: self.drive[abs],
                sends: self.sends[abs],
                group: self.group[abs],
                start: (r != Region::WHOLE).then(|| secs(r.start)),
                end: (r != Region::WHOLE && r.end != usize::MAX).then(|| secs(r.end)),
                slice: self.slice[abs],
                link: self.link[abs].map(|l| l.0),
                epoch: self.link[abs].map(|l| l.1),
            };
            if !state.is_default() {
                s.pads.insert(abs.to_string(), state);
            }
        }
        s
    }

    /// Takes modes, volumes, files and slices from a session; returns the engine commands for
    /// modes and volumes. Files and slices are only noted here, loading them is the caller's job.
    pub fn restore(&mut self, s: &Session, rate: u32) -> Vec<Command> {
        let mut cmds = Vec::new();
        self.bank = (s.bank as usize).min(BANKS - 1);
        self.pattern = s.patterns.first().cloned();
        self.pattern_rows = None;
        self.pitch_gate.fill(true);
        for (key, state) in &s.pads {
            let Some(abs) = key.parse::<usize>().ok().filter(|&a| a < PADS) else {
                log::warn!("session: bad pad {key}");
                continue;
            };
            self.gate[abs] = state.gate;
            self.pitch_gate[abs] = state.pitch_gate;
            self.volume_db[abs] = state.volume_db.clamp(MIN_DB, MAX_DB);
            self.pitch[abs] = state.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
            self.attack_ms[abs] = state.attack_ms;
            self.decay_ms[abs] = state.decay_ms;
            self.release_ms[abs] = state.release_ms;
            self.filter[abs] = kind_of(&state.filter);
            self.cutoff_hz[abs] = state
                .cutoff_hz
                .unwrap_or(CUTOFF_DEFAULT)
                .clamp(CUTOFF_MIN, CUTOFF_MAX);
            self.reso[abs] = state.reso.min(100);
            self.drive[abs] = state.drive.min(100);
            self.sends[abs] = state.sends.map(|s| s.min(100));
            self.group[abs] = state.group.min(GROUPS);
            self.file[abs] = state.file.clone();
            self.custom_name[abs] = state.name.clone();
            self.slice[abs] = state.slice;
            self.link[abs] = state.link.zip(state.epoch);
            let frames = |secs: f64| (secs.max(0.0) * rate as f64).round() as usize;
            self.region[abs] = match (state.start, state.end) {
                (None, None) => Region::WHOLE,
                (start, end) => Region {
                    start: start.map_or(0, frames),
                    end: end.map_or(usize::MAX, frames),
                },
            };
        }
        // every pad, the defaults too: the engine still holds the previous project's values
        for abs in 0..PADS {
            cmds.push(Command::SetPadMode {
                pad: abs,
                mode: if self.gate[abs] {
                    PadMode::Gate
                } else {
                    PadMode::OneShot
                },
            });
            cmds.push(Command::SetPadGain {
                pad: abs,
                gain: 10f32.powf(self.volume_db[abs] as f32 / 20.0),
            });
            cmds.push(Command::SetPadPitch {
                pad: abs,
                semitones: self.pitch[abs],
            });
            cmds.push(self.pitch_gate_command(abs));
            cmds.push(self.env_command(abs));
            cmds.push(self.filter_command(abs));
            cmds.push(self.drive_command(abs));
            cmds.push(self.sends_command(abs));
            cmds.push(self.group_command(abs));
        }
        // the slots of the project; an older one has the new-project slots (56)
        self.fx =
            std::array::from_fn(|k| s.fx.get(k).map_or(FxSlot::DEFAULTS[k], FxSlot::from_state));
        for k in 0..FX_SLOTS {
            cmds.push(self.fx_command(k));
        }
        match Self::compile_pattern(self.pattern.as_ref(), None) {
            Ok(pattern) => cmds.push(Command::SetPattern { pattern }),
            Err(error) => log::error!("Cannot restore pattern: {error}"),
        }
        cmds
    }

    pub fn pitch_gate_command(&self, abs: usize) -> Command {
        Command::SetPadPitchGate {
            pad: abs,
            enabled: self.pitch_gate[abs],
            linked_slice: matches!(self.source(abs), Source::Slice(_)),
        }
    }

    /// The drive command of a pad (56).
    pub fn drive_command(&self, abs: usize) -> Command {
        Command::SetPadDrive {
            pad: abs,
            drive: self.drive[abs] as f32 / 100.0,
        }
    }

    /// The mute group command of a pad (70).
    pub fn group_command(&self, abs: usize) -> Command {
        Command::SetPadGroup {
            pad: abs,
            group: self.group[abs],
        }
    }

    /// Sets the mute group `g` on every pad of the scope (70): a group is a name, not an
    /// amount, so the pads take the same one, as `H` gives them the same mode.
    fn set_group(&mut self, g: u8) -> Vec<Command> {
        let g = g.min(GROUPS);
        let mut cmds = Vec::new();
        for abs in self.scope_pads() {
            if self.group[abs] != g {
                self.group[abs] = g;
                cmds.push(self.group_command(abs));
            }
        }
        cmds
    }

    /// The sends command of a pad (56).
    pub fn sends_command(&self, abs: usize) -> Command {
        Command::SetPadSends {
            pad: abs,
            sends: self.sends[abs].map(|s| s as f32 / 100.0),
        }
    }

    /// The command of a shared slot (56): its kind with that kind's parameters.
    pub fn fx_command(&self, slot: usize) -> Command {
        Command::SetFx {
            slot,
            settings: self.fx[slot].settings(),
        }
    }

    /// A release from a controller.
    pub fn release(&mut self, abs: usize) {
        if abs < PADS {
            self.held[abs] = false;
        }
    }

    /// The source of a pad, derived from what it keeps: a live link with its slice number
    /// is a slice, an unlinked whole region is the file, anything else its own bounds.
    pub fn source(&self, abs: usize) -> Source {
        match (self.link[abs], self.slice[abs]) {
            (Some(_), Some(n)) => Source::Slice(n),
            _ if self.region[abs] == Region::WHOLE => Source::All,
            _ => Source::Own,
        }
    }

    /// Volume of the selected pad as a knob position 0..1 over the MIN_DB..MAX_DB range.
    pub fn volume_pos(&self) -> f32 {
        (self.volume_db[self.selected_abs()] - MIN_DB) as f32 / (MAX_DB - MIN_DB) as f32
    }

    /// Pitch of the selected pad as a knob position 0..1 over −`PITCH_LIMIT`…+`PITCH_LIMIT`.
    pub fn pitch_pos(&self) -> f32 {
        (self.pitch[self.selected_abs()] + PITCH_LIMIT) as f32 / (2 * PITCH_LIMIT) as f32
    }

    /// A knob turn: the selected pad takes the knob position (soft takeover is
    /// against it), the other pads of the scope move by the same delta, a pad at
    /// a limit stays there. Empty when nothing changed.
    pub fn set_volume_pos(&mut self, pos: f32) -> Vec<Command> {
        let abs = self.selected_abs();
        let db = (MIN_DB as f32 + pos.clamp(0.0, 1.0) * (MAX_DB - MIN_DB) as f32).round() as i8;
        let delta = db - self.volume_db[abs];
        if delta == 0 {
            return Vec::new();
        }
        let mut cmds = Vec::new();
        for pad in self.scope_pads() {
            let db = if pad == abs {
                db
            } else {
                (self.volume_db[pad] + delta).clamp(MIN_DB, MAX_DB)
            };
            if db == self.volume_db[pad] {
                continue;
            }
            self.volume_db[pad] = db;
            cmds.push(Command::SetPadGain {
                pad,
                gain: 10f32.powf(db as f32 / 20.0),
            });
        }
        cmds
    }

    /// The pitch knob, same rule: the selected pad takes the position, the scope
    /// follows the delta. Empty when nothing changed.
    pub fn set_pitch_pos(&mut self, pos: f32) -> Vec<Command> {
        let abs = self.selected_abs();
        let semis = (pos.clamp(0.0, 1.0) * (2 * PITCH_LIMIT) as f32).round() as i8 - PITCH_LIMIT;
        let delta = semis - self.pitch[abs];
        if delta == 0 {
            return Vec::new();
        }
        let mut cmds = Vec::new();
        for pad in self.scope_pads() {
            let semis = if pad == abs {
                semis
            } else {
                (self.pitch[pad] + delta).clamp(-PITCH_LIMIT, PITCH_LIMIT)
            };
            if semis == self.pitch[pad] {
                continue;
            }
            self.pitch[pad] = semis;
            cmds.push(Command::SetPadPitch {
                pad,
                semitones: semis,
            });
        }
        cmds
    }

    /// An envelope time of the selected pad as a knob position: a square curve over
    /// 0..ENV_MS_LIMIT, so the short times near zero get most of the knob's travel.
    pub fn env_pos(&self, param: Param) -> f32 {
        let ms = self.env_of(self.selected_abs(), param);
        (ms as f32 / ENV_MS_LIMIT as f32).sqrt()
    }

    /// The envelope knob, the rule of the volume knob: the selected pad takes the
    /// position, the scope follows the delta. Empty when nothing changed.
    pub fn set_env_pos(&mut self, param: Param, pos: f32) -> Vec<Command> {
        let abs = self.selected_abs();
        let ms = (pos.clamp(0.0, 1.0).powi(2) * ENV_MS_LIMIT as f32).round() as i32;
        let delta = ms - self.env_of(abs, param) as i32;
        if delta == 0 {
            return Vec::new();
        }
        let mut cmds = Vec::new();
        for pad in self.scope_pads() {
            let old = self.env_of(pad, param) as i32;
            let new = if pad == abs {
                ms
            } else {
                (old + delta).clamp(0, ENV_MS_LIMIT)
            };
            if new == old {
                continue;
            }
            self.set_env_ms(pad, param, new as u16);
            cmds.push(self.env_command(pad));
        }
        cmds
    }

    fn env_of(&self, abs: usize, param: Param) -> u16 {
        match param {
            Param::Attack => self.attack_ms[abs],
            Param::Decay => self.decay_ms[abs],
            Param::Release => self.release_ms[abs],
            _ => 0,
        }
    }

    /// Envelope time of the pad, in ms, for the parameter being changed.
    fn env_ms(&self, abs: usize) -> u16 {
        self.env_of(abs, self.param)
    }

    /// Sets one envelope time of the pad; the caller sends `env_command` after.
    fn set_env_ms(&mut self, abs: usize, param: Param, ms: u16) {
        match param {
            Param::Attack => self.attack_ms[abs] = ms,
            Param::Decay => self.decay_ms[abs] = ms,
            Param::Release => self.release_ms[abs] = ms,
            _ => {}
        }
    }

    /// The filter command for the pad: kind, cutoff and resonance in one.
    pub fn filter_command(&self, abs: usize) -> Command {
        Command::SetPadFilter {
            pad: abs,
            settings: self.filter_settings(abs),
        }
    }

    pub fn filter_settings(&self, abs: usize) -> FilterSettings {
        FilterSettings {
            kind: self.filter[abs],
            cutoff_hz: self.cutoff_hz[abs],
            resonance: self.reso[abs] as f32 / 100.0,
        }
    }

    /// A filter parameter of the selected pad as a knob position 0..1: the kind by quarters,
    /// the cutoff along the octaves, the resonance in percent.
    pub fn filter_pos(&self, param: Param) -> f32 {
        let abs = self.selected_abs();
        match param {
            Param::Filter => {
                let i = KINDS
                    .iter()
                    .position(|&k| k == self.filter[abs])
                    .unwrap_or(0);
                (i as f32 + 0.5) / KINDS.len() as f32
            }
            Param::Cutoff => {
                (self.cutoff_hz[abs] / CUTOFF_MIN).ln() / (CUTOFF_MAX / CUTOFF_MIN).ln()
            }
            _ => self.reso[abs] as f32 / 100.0,
        }
    }

    /// The filter knob, the rule of the volume knob: the selected pad takes the position,
    /// the scope follows the same change. Empty when nothing changed.
    pub fn set_filter_pos(&mut self, param: Param, pos: f32) -> Vec<Command> {
        let abs = self.selected_abs();
        let pos = pos.clamp(0.0, 1.0);
        let mut cmds = Vec::new();
        match param {
            Param::Filter => {
                let kind = KINDS[((pos * KINDS.len() as f32) as usize).min(KINDS.len() - 1)];
                for pad in self.scope_pads() {
                    if self.filter[pad] != kind {
                        self.filter[pad] = kind;
                        cmds.push(self.filter_command(pad));
                    }
                }
            }
            Param::Cutoff => {
                let hz = CUTOFF_MIN * (CUTOFF_MAX / CUTOFF_MIN).powf(pos);
                let ratio = hz / self.cutoff_hz[abs];
                if (ratio - 1.0).abs() < 1e-4 {
                    return cmds;
                }
                for pad in self.scope_pads() {
                    let new = if pad == abs {
                        hz
                    } else {
                        (self.cutoff_hz[pad] * ratio).clamp(CUTOFF_MIN, CUTOFF_MAX)
                    };
                    if new != self.cutoff_hz[pad] {
                        self.cutoff_hz[pad] = new;
                        cmds.push(self.filter_command(pad));
                    }
                }
            }
            _ => {
                let pct = (pos * 100.0).round() as i16;
                let delta = pct - self.reso[abs] as i16;
                if delta == 0 {
                    return cmds;
                }
                for pad in self.scope_pads() {
                    let new = (self.reso[pad] as i16 + delta).clamp(0, 100) as u8;
                    if new != self.reso[pad] {
                        self.reso[pad] = new;
                        cmds.push(self.filter_command(pad));
                    }
                }
            }
        }
        cmds
    }

    /// The envelope command for the pad: one command carries all three times, so a
    /// scope change stays one burst like in 19.
    fn env_command(&self, abs: usize) -> Command {
        Command::SetPadEnvelope {
            pad: abs,
            attack_ms: self.attack_ms[abs],
            decay_ms: self.decay_ms[abs],
            release_ms: self.release_ms[abs],
        }
    }

    /// The page of the picked parameter and its parameters in knob order.
    pub fn page(&self) -> (&'static str, &'static [Param]) {
        PAGES
            .into_iter()
            .find(|(_, params)| params.contains(&self.param))
            .unwrap_or(PAGES[0])
    }

    /// The page knob (54): the page under `pos` 0..1; a turn inside the current page keeps
    /// the pick, a new page picks its first parameter.
    pub fn set_page_pos(&mut self, pos: f32) {
        let at = ((pos.clamp(0.0, 1.0) * PAGES.len() as f32) as usize).min(PAGES.len() - 1);
        if !PAGES[at].1.contains(&self.param) {
            self.param = PAGES[at].1[0];
        }
    }

    /// Name of the parameter being changed, for the help line and the pop-ups.
    pub fn param_name(&self) -> &'static str {
        self.name(self.param)
    }

    /// The name of a parameter as the panel says it: a slot's two parameters follow the
    /// kind it holds (56).
    pub fn name(&self, p: Param) -> &'static str {
        match (p.slot().map(|k| self.fx[k].kind), p) {
            (Some(FxKind::Reverb), Param::Fx1A | Param::Fx2A) => "size",
            (Some(FxKind::Reverb), Param::Fx1B | Param::Fx2B) => "tone",
            (Some(FxKind::Delay), Param::Fx1A | Param::Fx2A) => "time",
            (Some(FxKind::Delay), Param::Fx1B | Param::Fx2B) => "feedback",
            _ => name_of(p),
        }
    }

    /// The value of an effect parameter as the panel says it (56): the pad's for the FX
    /// page, the slot's for a starred page.
    pub fn fx_text(&self, abs: usize, p: Param) -> String {
        let slot = |k: usize| &self.fx[k];
        match p {
            Param::Group => group_text(self.group[abs]),
            Param::Drive => format!("{} %", self.drive[abs]),
            Param::Send1 => format!("{} %", self.sends[abs][0]),
            Param::Send2 => format!("{} %", self.sends[abs][1]),
            Param::Fx1Kind | Param::Fx2Kind => kind_word(slot(p.slot().unwrap_or(0)).kind).into(),
            Param::Fx1A | Param::Fx2A => {
                let f = slot(p.slot().unwrap_or(0));
                match f.kind {
                    FxKind::Reverb => format!("{} %", f.size),
                    FxKind::Delay => format!("{} ms", f.time_ms),
                }
            }
            Param::Fx1B | Param::Fx2B => {
                let f = slot(p.slot().unwrap_or(0));
                match f.kind {
                    FxKind::Reverb => format!("{} %", f.tone),
                    FxKind::Delay => format!("{} %", f.feedback),
                }
            }
            _ => String::new(),
        }
    }

    /// An effect parameter as a knob position 0..1 (56): percent over 0..100, the kind by
    /// halves, the delay time along 10..2000 ms on a curve, the feedback over 0..90 %.
    pub fn fx_pos(&self, p: Param) -> f32 {
        let abs = self.selected_abs();
        let pct = |v: u8| v as f32 / 100.0;
        let f = p.slot().map(|k| self.fx[k]);
        match (p, f) {
            (Param::Group, _) => self.group[abs] as f32 / GROUPS as f32,
            (Param::Drive, _) => pct(self.drive[abs]),
            (Param::Send1, _) => pct(self.sends[abs][0]),
            (Param::Send2, _) => pct(self.sends[abs][1]),
            (Param::Fx1Kind | Param::Fx2Kind, Some(f)) => match f.kind {
                FxKind::Reverb => 0.25,
                FxKind::Delay => 0.75,
            },
            (Param::Fx1A | Param::Fx2A, Some(f)) => match f.kind {
                FxKind::Reverb => pct(f.size),
                FxKind::Delay => time_pos(f.time_ms),
            },
            (Param::Fx1B | Param::Fx2B, Some(f)) => match f.kind {
                FxKind::Reverb => pct(f.tone),
                FxKind::Delay => f.feedback as f32 / (FEEDBACK_MAX * 100.0),
            },
            _ => 0.0,
        }
    }

    /// A knob sets an effect parameter (56): the selected pad takes the position, the other
    /// pads of the scope move by the same amount (the FX page); a slot is one for all pads.
    pub fn set_fx_pos(&mut self, p: Param, pos: f32) -> Vec<Command> {
        let pos = pos.clamp(0.0, 1.0);
        let pct = (pos * 100.0).round() as u8;
        if let Some(k) = p.slot() {
            let f = &mut self.fx[k];
            let before = *f;
            match p {
                Param::Fx1Kind | Param::Fx2Kind => {
                    f.kind = if pos < 0.5 {
                        FxKind::Reverb
                    } else {
                        FxKind::Delay
                    }
                }
                Param::Fx1A | Param::Fx2A => match f.kind {
                    FxKind::Reverb => f.size = pct,
                    FxKind::Delay => f.time_ms = time_of(pos),
                },
                _ => match f.kind {
                    FxKind::Reverb => f.tone = pct,
                    FxKind::Delay => f.feedback = (pos * FEEDBACK_MAX * 100.0).round() as u8,
                },
            }
            return if *f != before {
                vec![self.fx_command(k)]
            } else {
                Vec::new()
            };
        }
        if p == Param::Group {
            return self.set_group((pos * GROUPS as f32).round() as u8);
        }
        let abs = self.selected_abs();
        let delta = pct as i16 - self.pad_pct(abs, p) as i16;
        let mut cmds = Vec::new();
        for pad in self.scope_pads() {
            let new = (self.pad_pct(pad, p) as i16 + delta).clamp(0, 100) as u8;
            if new != self.pad_pct(pad, p) {
                cmds.push(self.set_pad_pct(pad, p, new));
            }
        }
        cmds
    }

    /// The drive or a send of a pad, in percent.
    fn pad_pct(&self, abs: usize, p: Param) -> u8 {
        match p {
            Param::Drive => self.drive[abs],
            Param::Send1 => self.sends[abs][0],
            _ => self.sends[abs][1],
        }
    }

    /// Sets the drive or a send of a pad; returns the engine command.
    fn set_pad_pct(&mut self, abs: usize, p: Param, v: u8) -> Command {
        match p {
            Param::Drive => {
                self.drive[abs] = v;
                self.drive_command(abs)
            }
            Param::Send1 => {
                self.sends[abs][0] = v;
                self.sends_command(abs)
            }
            _ => {
                self.sends[abs][1] = v;
                self.sends_command(abs)
            }
        }
    }

    /// Value of the selected pad for the parameter being changed, for the pop-up.
    pub fn param_value(&self) -> String {
        let abs = self.selected_abs();
        match self.param {
            Param::Volume => format!("{:+} dB", self.volume_db[abs]),
            Param::Pitch => format!("{:+} st", self.pitch[abs]),
            Param::Attack => format!("{} ms", self.attack_ms[abs]),
            Param::Decay => format!("{} ms", self.decay_ms[abs]),
            Param::Release => format!("{} ms", self.release_ms[abs]),
            Param::Filter => kind_name(self.filter[abs]).into(),
            Param::Cutoff => cutoff_text(self.cutoff_hz[abs]),
            Param::Reso => format!("{} %", self.reso[abs]),
            // the app says these, it knows the file and its cuts
            Param::Source | Param::Start | Param::End => String::new(),
            p => self.fx_text(abs, p),
        }
    }

    /// Starts a UI-routed pad hit without sharing keyboard release targets with MIDI.
    pub fn press(&mut self, pad: usize, velocity: f32, source: HitSource) -> Vec<Command> {
        if pad >= PADS_PER_BANK {
            return Vec::new();
        }
        let abs = self.bank * PADS_PER_BANK + pad;
        if matches!(source, HitSource::Keyboard) {
            self.pressed[pad] = abs;
        }
        self.selected = pad;
        self.hit_at[abs] = Some(Instant::now());
        self.held[abs] = self.track_keys;
        self.mark_hit(abs);
        vec![Command::PadPress { pad: abs, velocity }]
    }

    /// Applies a keyboard pad action; returns the engine commands to send.
    /// Non-pad actions are ignored.
    pub fn apply(&mut self, action: Action) -> Vec<Command> {
        match action {
            Action::PadPress { pad, velocity } if pad < PADS_PER_BANK => {
                self.press(pad, velocity, HitSource::Keyboard)
            }
            Action::PadRelease { pad } if pad < PADS_PER_BANK => {
                self.held[self.pressed[pad]] = false;
                vec![Command::PadRelease {
                    pad: self.pressed[pad],
                }]
            }
            Action::BankNext => {
                self.bank = (self.bank + 1).min(BANKS - 1);
                Vec::new()
            }
            Action::BankPrev => {
                self.bank = self.bank.saturating_sub(1);
                Vec::new()
            }
            Action::BankSelect(bank) => {
                self.bank = bank.min(BANKS - 1);
                Vec::new()
            }
            Action::PadSelect(dir) => {
                let (row, col) = (self.selected / 4, self.selected % 4);
                let (row, col) = match dir {
                    Dir::Up => (row.saturating_sub(1), col),
                    Dir::Down => ((row + 1).min(3), col),
                    Dir::Left => (row, col.saturating_sub(1)),
                    Dir::Right => (row, (col + 1).min(3)),
                };
                self.selected = row * 4 + col;
                Vec::new()
            }
            Action::PadPitchGateToggle => {
                let enabled = !self.pitch_gate[self.selected_abs()];
                self.scope_pads()
                    .into_iter()
                    .map(|abs| {
                        self.pitch_gate[abs] = enabled;
                        self.pitch_gate_command(abs)
                    })
                    .collect()
            }
            Action::PadModeNext => {
                // every pad of the scope gets the next mode of the selected pad
                let sel = self.selected_abs();
                let gate = !self.gate[sel];
                let mode = if gate {
                    PadMode::Gate
                } else {
                    PadMode::OneShot
                };
                let mut cmds = Vec::new();
                for abs in self.scope_pads() {
                    self.gate[abs] = gate;
                    cmds.push(Command::SetPadMode { pad: abs, mode });
                }
                cmds
            }
            Action::PageNext | Action::PagePrev => {
                // a whole page: the pick lands on its first parameter, knob 1 and J L agree (54)
                let at = PAGES
                    .iter()
                    .position(|(_, params)| params.contains(&self.param))
                    .unwrap_or(0);
                let step = if action == Action::PagePrev {
                    PAGES.len() - 1
                } else {
                    1
                };
                self.param = PAGES[(at + step) % PAGES.len()].1[0];
                Vec::new()
            }
            Action::PadMarkBank => {
                self.mark_bank();
                Vec::new()
            }
            Action::PadMarkToggle => {
                self.marking = !self.marking;
                Vec::new()
            }
            Action::PadParamNext | Action::ParamNext | Action::ParamPrev => {
                let at = ORDER.iter().position(|&p| p == self.param).unwrap_or(0);
                let step = if action == Action::ParamPrev {
                    ORDER.len() - 1
                } else {
                    1
                };
                self.param = ORDER[(at + step) % ORDER.len()];
                Vec::new()
            }
            Action::PadParam(_) | Action::EditValue { .. } => {
                let (step, env_ms, fine) = match action {
                    Action::EditValue { up, fine } => (
                        if up { 1 } else { -1 },
                        if fine { ENV_FINE_MS } else { ENV_STEP_MS },
                        fine,
                    ),
                    Action::PadParam(step) => (step, ENV_STEP_MS, true),
                    _ => (0, ENV_STEP_MS, true),
                };
                // a relative change: every pad of the scope moves by the step,
                // a pad at a limit stays there and is counted for the pop-up
                let mut cmds = Vec::new();
                let mut limits = 0;
                match self.param {
                    Param::Volume => {
                        for abs in self.scope_pads() {
                            let db = (self.volume_db[abs] + step).clamp(MIN_DB, MAX_DB);
                            if db == self.volume_db[abs] {
                                limits += 1;
                                continue;
                            }
                            self.volume_db[abs] = db;
                            cmds.push(Command::SetPadGain {
                                pad: abs,
                                gain: 10f32.powf(db as f32 / 20.0),
                            });
                        }
                    }
                    Param::Pitch => {
                        for abs in self.scope_pads() {
                            let semitones =
                                (self.pitch[abs] + step).clamp(-PITCH_LIMIT, PITCH_LIMIT);
                            if semitones == self.pitch[abs] {
                                limits += 1;
                                continue;
                            }
                            self.pitch[abs] = semitones;
                            cmds.push(Command::SetPadPitch {
                                pad: abs,
                                semitones,
                            });
                        }
                    }
                    Param::Attack | Param::Decay | Param::Release => {
                        for abs in self.scope_pads() {
                            let ms = self.env_ms(abs) as i32;
                            let new = (ms + step as i32 * env_ms).clamp(0, ENV_MS_LIMIT) as u16;
                            if new as i32 == ms {
                                limits += 1;
                                continue;
                            }
                            self.set_env_ms(abs, self.param, new);
                            cmds.push(self.env_command(abs));
                        }
                    }
                    // the app changes the source page: it knows the cuts of the file
                    Param::Source | Param::Start | Param::End => {}
                    // the scope takes the selected pad's next group, off…8, no wrap (70)
                    Param::Group => {
                        let now = self.group[self.selected_abs()];
                        let g = (now as i8 + step.signum()).clamp(0, GROUPS as i8) as u8;
                        if g == now {
                            limits += 1;
                        }
                        cmds.extend(self.set_group(g));
                    }
                    // the drive and the sends follow the scope, a tap 1 %, a held key 10 % (56)
                    Param::Drive | Param::Send1 | Param::Send2 => {
                        let pct = if fine { 1 } else { 10 } * step as i16;
                        let param = self.param;
                        for abs in self.scope_pads() {
                            let now = self.pad_pct(abs, param);
                            let new = (now as i16 + pct).clamp(0, 100) as u8;
                            if new == now {
                                limits += 1;
                                continue;
                            }
                            cmds.push(self.set_pad_pct(abs, param, new));
                        }
                    }
                    // a shared slot is one for all pads: the scope does not apply (56)
                    p @ (Param::Fx1Kind
                    | Param::Fx1A
                    | Param::Fx1B
                    | Param::Fx2Kind
                    | Param::Fx2A
                    | Param::Fx2B) => {
                        let k = p.slot().unwrap_or(0);
                        let f = &mut self.fx[k];
                        let before = *f;
                        let pct = |v: u8, max: u8| {
                            (v as i16 + if fine { 1 } else { 10 } * step as i16)
                                .clamp(0, max as i16) as u8
                        };
                        match p {
                            Param::Fx1Kind | Param::Fx2Kind => {
                                f.kind = match f.kind {
                                    FxKind::Reverb => FxKind::Delay,
                                    FxKind::Delay => FxKind::Reverb,
                                }
                            }
                            Param::Fx1A | Param::Fx2A => match f.kind {
                                FxKind::Reverb => f.size = pct(f.size, 100),
                                FxKind::Delay => {
                                    let ms = if fine { 10 } else { 100 } * step as i32;
                                    f.time_ms = (f.time_ms as i32 + ms).clamp(10, 2_000) as u16;
                                }
                            },
                            _ => match f.kind {
                                FxKind::Reverb => f.tone = pct(f.tone, 100),
                                FxKind::Delay => {
                                    f.feedback = pct(f.feedback, (FEEDBACK_MAX * 100.0) as u8)
                                }
                            },
                        }
                        if *f == before {
                            limits += 1;
                        } else {
                            cmds.push(self.fx_command(k));
                        }
                    }
                    Param::Filter | Param::Cutoff | Param::Reso => {
                        for abs in self.scope_pads() {
                            let changed = match self.param {
                                Param::Filter => {
                                    let i = KINDS.iter().position(|&k| k == self.filter[abs]);
                                    let i = i.unwrap_or(0) as i32 + step as i32;
                                    let kind = KINDS[i.clamp(0, KINDS.len() as i32 - 1) as usize];
                                    std::mem::replace(&mut self.filter[abs], kind) != kind
                                }
                                // a tap a semitone, a held key an octave
                                Param::Cutoff => {
                                    let st = if fine { 1.0 } else { 12.0 } * step as f32;
                                    let hz = (self.cutoff_hz[abs] * (st / 12.0).exp2())
                                        .clamp(CUTOFF_MIN, CUTOFF_MAX);
                                    std::mem::replace(&mut self.cutoff_hz[abs], hz) != hz
                                }
                                _ => {
                                    let pct = if fine { 1 } else { 10 } * step as i16;
                                    let new = (self.reso[abs] as i16 + pct).clamp(0, 100) as u8;
                                    std::mem::replace(&mut self.reso[abs], new) != new
                                }
                            };
                            if changed {
                                cmds.push(self.filter_command(abs));
                            } else {
                                limits += 1;
                            }
                        }
                    }
                }
                self.limit_hits = (limits > 0).then_some(limits);
                cmds
            }
            _ => Vec::new(),
        }
    }
}

use crate::*;

impl App {
    pub(super) fn load_pad(&mut self, abs: usize, path: PathBuf, region: Region) {
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

    pub(super) fn request(&mut self, pad: usize, path: PathBuf, generation: u64) {
        let sent = self.loader.as_ref().is_some_and(|(tx, _)| {
            tx.send(loader::Request {
                pad,
                path: path.clone(),
                generation,
            })
            .is_ok()
        });
        if !sent {
            self.loader_reply(loader::Reply::Failed {
                pad,
                generation,
                path,
                error: "Loader unavailable".into(),
            });
        }
    }

    /// `abs` gets the file `leader` is loading, without reading it again.
    pub(super) fn follow(&mut self, leader: usize, abs: usize, region: Region) {
        self.load_gen[abs] += 1;
        self.pads.loading[abs] = Some(0);
        self.load_region[abs] = region;
        self.followers.push((leader, abs, self.load_gen[abs]));
    }

    /// 83a: Ctrl+C takes a copy of the selected loaded pad; the snapshot is taken now,
    /// so later changes of the source do not change what is placed. Empty, missing and
    /// loading sources are refused with a plain notice, nothing else changes.
    pub(super) fn take_copy(&mut self) {
        let abs = self.pads.selected_abs();
        if self.pads.sound[abs].is_none() {
            self.post_notice(
                Level::Warn,
                ("Copy what? This pad holds nothing.".into(), Instant::now()),
            );
            return;
        }
        if self.pads.missing[abs] {
            self.post_notice(
                Level::Warn,
                (
                    "Copy what? The file this pad plays is gone.".into(),
                    Instant::now(),
                ),
            );
            return;
        }
        if self.pads.loading[abs].is_some() {
            self.post_notice(
                Level::Warn,
                (
                    "Wait until loading finishes, then copy.".into(),
                    Instant::now(),
                ),
            );
            return;
        }
        let mut snap = self.pads.snapshot(abs);
        // Keep the full pickup name even if the source later changes or has no file name.
        snap.name = Some(pad_name(&self.pads, abs));
        self.copy_in_hand = Some((abs, snap));
        self.post_notice(Level::Info, (("COPY · hit a pad").into(), Instant::now()));
    }

    /// 83a: a pad hit with a copy in hand picks the target: the hit does not play, the
    /// source stays; a hit on the source cancels the copy. `false` when no copy is in
    /// hand, or the target is loading and the copy waits.
    pub(super) fn copy_target(&mut self, abs: usize) -> bool {
        let Some((source, snap)) = self.copy_in_hand.clone() else {
            return false;
        };
        if abs == source {
            // a hit on the original source cancels: no pad changes, no undo step
            self.copy_in_hand = None;
            self.post_notice(Level::Info, ("copy dropped".into(), Instant::now()));
            return true;
        }
        if self.pads.loading[abs].is_some() {
            self.post_notice(
                Level::Warn,
                (
                    "Wait until loading finishes, then place the copy.".into(),
                    Instant::now(),
                ),
            );
            return true;
        }
        if self.pads.sound[abs].is_some() && self.pads.loading[abs].is_none() {
            let mut lines = vec!["Replace what lies on 1 pad?".to_string()];
            lines.push(format!(
                "  {}: {} → {}",
                pad_place(abs),
                pad_name(&self.pads, abs),
                self.copy_name(&snap)
            ));
            lines.extend(overwrite_footer());
            // a changed target asks again: only the same question, still open, places
            if !matches!(&self.ask,
                Some(Ask::CopyPad { source: s, lines: old })
                    if *s == source && old[..2] == lines[..2])
            {
                self.ask = Some(Ask::CopyPad { source, lines });
                return true;
            }
        }
        self.place_copy(source, snap, abs);
        true
    }

    /// Enter in the copy question: the question names the source, so the target is the
    /// selected pad; a changed target asks again inside `copy_target`.
    pub(super) fn place_copy_asked(&mut self, source: usize) {
        let Some((snap_source, _)) = self.copy_in_hand.clone() else {
            self.ask = None;
            return;
        };
        debug_assert_eq!(snap_source, source, "the question names the copy in hand");
        let abs = self.pads.selected_abs();
        self.copy_target(abs);
    }

    /// Places the snapshot with an available numbered name. The source stays unchanged.
    /// One undo step restores only the target.
    pub(super) fn place_copy(&mut self, source: usize, mut snap: pads::PadSnap, abs: usize) {
        self.ask = None;
        self.copy_in_hand = None;
        snap.name = Some(self.copy_name(&snap));
        // the undo step is the apply's own: only the target changed (the snapshot was
        // taken at pickup, the source stays), so the step restores the target alone
        let _ = source;
        for cmd in self.pads.put_back(abs, &snap) {
            self.send(cmd);
        }
        self.dirty();
        self.post_notice(Level::Done, ("copy placed".into(), Instant::now()));
    }

    /// The question and placement use the same pickup base and current occupied names.
    pub(super) fn copy_name(&self, snap: &pads::PadSnap) -> String {
        let name = snap.name.as_deref().unwrap_or("copy");
        // Recognize decimal text without parsing: an existing marker can exceed any integer type.
        let base = name
            .strip_suffix(')')
            .and_then(|text| text.rsplit_once(" (copy "))
            .filter(|(_, number)| {
                !number.is_empty()
                    && !number.starts_with('0')
                    && number.bytes().all(|byte| byte.is_ascii_digit())
            })
            .map_or(name, |(base, _)| base);
        let occupied: std::collections::BTreeSet<_> =
            (0..PADS).map(|abs| pad_name(&self.pads, abs)).collect();
        let mut number = 1;
        loop {
            let candidate = format!("{base} (copy {number})");
            if !occupied.contains(&candidate) {
                return candidate;
            }
            // At most PADS names are occupied, so a candidate is free by PADS + 1.
            number += 1;
        }
    }

    /// Swaps the pad in hand with `abs`.
    pub(super) fn swap_with(&mut self, abs: usize) {
        self.flush_audio();
        let Some(from) = self.pad_in_hand.take() else {
            return;
        };
        if from == abs {
            return;
        }
        if self.pads.loading[from].is_some() || self.pads.loading[abs].is_some() {
            // the load would land on the old place
            self.post_notice(
                Level::Warn,
                (
                    "Wait until loading finishes, then swap.".into(),
                    Instant::now(),
                ),
            );
            return;
        }
        let before = vec![
            (from, self.pads.snapshot(from)),
            (abs, self.pads.snapshot(abs)),
        ];
        let cmd = match self.pads.swap(from, abs) {
            Ok(cmd) => cmd,
            Err(error) => {
                self.post_notice(Level::Error, (error, Instant::now()));
                return;
            }
        };
        if self.pads.pattern.is_some() {
            self.push_step(before, None, "swap".into());
            if let Some(step) = self.history.last_mut() {
                step.swap = Some((from, abs));
            }
        }
        self.send(cmd);
        self.dirty();
    }

    /// Puts the file in hand on a pad; a pad that holds something asks first.
    pub(super) fn put_in_hand(&mut self, abs: usize) {
        let Some(path) = self.browser.in_hand.clone() else {
            return;
        };
        self.put_file(abs, path);
    }

    /// Loads `path` onto `abs` at once; a pad that holds something asks first. A file in
    /// hand equal to `path` empties the hand once it lies on the pad.
    pub(super) fn put_file(&mut self, abs: usize, path: PathBuf) {
        if self.pads.file[abs].is_some() {
            if same_whole_sound(&self.pads, abs, &path) {
                // the pad already plays this very file, whole: nothing changes
                self.ask = None;
                if self.browser.in_hand.as_ref() == Some(&path) {
                    self.browser.in_hand = None;
                }
                self.post_notice(
                    Level::Info,
                    (
                        format!("{} already lies there", file_name(&path)),
                        Instant::now(),
                    ),
                );
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

    pub(super) fn loader_reply(&mut self, reply: loader::Reply) {
        // A stale leader reply must not consume followers of a newer load.
        if let loader::Reply::Loaded {
            pad, generation, ..
        }
        | loader::Reply::Failed {
            pad, generation, ..
        } = &reply
            && *generation == self.load_gen[*pad]
        {
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
                    self.post_system_notice(
                        Level::Error,
                        (
                            format!("Cannot play {}: {error}", file_name(&path)),
                            Instant::now(),
                        ),
                    );
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
                    self.chop_loading = None;
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
                    self.chop_loading = None;
                    self.post_system_notice(
                        Level::Error,
                        (
                            format!("Cannot chop {}: {error}", file_name(&path)),
                            Instant::now(),
                        ),
                    );
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
                let before = (!self.restoring_project).then(|| self.pads.snapshot(pad));
                self.pads.loading[pad] = None;
                self.pads.missing[pad] = false;
                if self.pads.file[pad].as_ref() != Some(&path)
                    || self.pads.region[pad] != self.load_region[pad]
                {
                    self.pads.custom_name[pad] = None;
                }
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
                self.send(self.pads.pitch_gate_command(pad));
                if let Some(before) = before
                    && before != self.pads.snapshot(pad)
                {
                    self.push_step(vec![(pad, before)], None, "file".into());
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
                self.post_system_notice(
                    Level::Error,
                    (
                        format!("Cannot load {}: {error}", file_name(&path)),
                        Instant::now(),
                    ),
                );
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
        self.finish_project_restore();
    }
    /// The sources a pad can step through: all, every slice of its file, its own bounds.
    pub(super) fn source_list(&self, abs: usize) -> Vec<pads::Source> {
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
    pub(super) fn pad_bounds(&self, abs: usize) -> Option<(usize, usize, usize)> {
        let len = self.pads.sound[abs].as_ref()?.len().max(1);
        let r = self.pads.region[abs];
        Some((r.start.min(len), r.end.min(len), len))
    }

    /// The edit layer takes the keys: the edit is open and no question waits; a question
    /// takes Enter and Esc of the screen, where the edit's Enter closes the edit (76).
    pub(super) fn edit_keys(&self) -> bool {
        self.edit && self.ask.is_none()
    }

    /// `Backspace` in the pad edit (76): asks before the pads of the scope with a sound go
    /// back to clean.
    pub(super) fn ask_reset(&mut self) {
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
    pub(super) fn ask_clear(&mut self) {
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

    pub(super) fn clear_pads(&mut self, pads: &[usize]) {
        for &abs in pads {
            // a file still loading for the pad would land after the clear
            self.load_gen[abs] += 1;
            let cmd = self.pads.clear(abs);
            self.send(cmd);
            self.send(self.pads.pitch_gate_command(abs));
        }
        self.dirty();
    }

    pub(super) fn reset_pads(&mut self, pads: &[usize]) {
        for &abs in pads {
            for cmd in self.pads.reset_clean(abs) {
                self.send(cmd);
            }
        }
        self.dirty();
    }

    /// Every pad and the slots as they are now, for the undo (61).
    pub(super) fn before(&self) -> Before {
        Before {
            pads: (0..engine::PADS).map(|a| self.pads.snapshot(a)).collect(),
            fx: self.pads.fx,
        }
    }

    /// What an action changes, as the undo says it (61).
    pub(super) fn step_name(&self, action: Action) -> String {
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
    pub(super) fn note_step(&mut self, b: Before, what: String) {
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

    pub(super) fn push_step(
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

    pub(super) fn push_pattern_step(&mut self, before: pattern_record::Undo) {
        self.push_pattern_step_named(before, "pattern pass");
    }

    pub(super) fn push_pattern_step_named(&mut self, before: pattern_record::Undo, what: &str) {
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
    pub(super) fn undo_pads(&mut self) {
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

    pub(super) fn set_pad_region(&mut self, abs: usize, region: Region) {
        self.pads.region[abs] = region;
        self.send(Command::SetPadRegion { pad: abs, region });
        self.send(self.pads.pitch_gate_command(abs));
        self.dirty();
    }

    /// Puts a pad on another source, one undo step (27).
    pub(super) fn set_source(&mut self, abs: usize, source: pads::Source) {
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
    pub(super) fn move_bound(&mut self, abs: usize, end: bool, to: usize) {
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
    pub(super) fn source_value(&mut self, up: bool, fine: bool) {
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
    pub(super) fn pad_name_key(&mut self, k: &event::KeyEvent) -> bool {
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

    /// A pad that can sound now: a loaded sample, not still loading, not failed (73).
    pub(super) fn pad_sounds(&self, pad: usize) -> bool {
        pad < PADS
            && self.pads.file[pad].is_some()
            && !self.pads.missing[pad]
            && self.pads.loading[pad].is_none()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn custom_name_survives_session_swap_reset_and_undo() {
        let mut pads = super::Pads::default();
        pads.file[0] = Some("kick.wav".into());
        pads.custom_name[0] = Some("Бочка".into());
        let saved = pads.to_session(None, 48_000);
        let text = toml::to_string(&saved).unwrap();
        let loaded: crate::session::Session = toml::from_str(&text).unwrap();
        let mut restored = super::Pads::default();
        restored.restore(&loaded, 48_000);
        assert_eq!(restored.custom_name[0].as_deref(), Some("Бочка"));
        let snapshot = restored.snapshot(0);
        restored.swap(0, 1).unwrap();
        assert_eq!(restored.custom_name[1].as_deref(), Some("Бочка"));
        restored.reset_clean(1);
        assert_eq!(restored.custom_name[1].as_deref(), Some("Бочка"));
        restored.clear(1);
        assert_eq!(restored.custom_name[1], None);
        restored.put_back(0, &snapshot);
        assert_eq!(restored.custom_name[0].as_deref(), Some("Бочка"));
        let old: crate::session::PadState = toml::from_str("file = 'old.wav'").unwrap();
        assert_eq!(old.name, None);
    }
    use super::*;

    fn press(pad: usize) -> Action {
        Action::PadPress { pad, velocity: 1.0 }
    }

    #[test]
    fn bank_offsets_pads_and_release_follows_the_press() {
        let mut p = Pads::default();
        assert!(matches!(
            p.apply(press(5))[..],
            [Command::PadPress { pad: 5, .. }]
        ));
        p.apply(Action::BankNext);
        // still held from bank 1: the release must reach pad 5, not 21
        assert!(matches!(
            p.apply(Action::PadRelease { pad: 5 })[..],
            [Command::PadRelease { pad: 5 }]
        ));
        assert!(matches!(
            p.apply(press(5))[..],
            [Command::PadPress { pad: 21, .. }]
        ));
        assert_eq!(p.selected, 5);
    }

    #[test]
    fn midi_ui_press_keeps_keyboard_target_selection_marks_and_held_display() {
        for track_keys in [false, true] {
            let mut pads = Pads {
                track_keys,
                marking: true,
                ..Pads::default()
            };
            pads.apply(press(2));
            pads.apply(Action::BankSelect(7));
            let commands = pads.press(2, 0.25, HitSource::Midi);
            assert!(matches!(
                commands[..],
                [Command::PadPress {
                    pad: 114,
                    velocity: 0.25
                }]
            ));
            assert_eq!(pads.selected, 2);
            assert_eq!(pads.marked, [2, 114].into());
            assert!(pads.hit_at[114].is_some());
            assert_eq!(pads.held[114], track_keys);
            assert!(pads.press(PADS_PER_BANK, 1.0, HitSource::Midi).is_empty());
            assert_eq!(pads.selected, 2);
            assert_eq!(pads.marked, [2, 114].into());
            assert!(matches!(
                pads.apply(Action::PadRelease { pad: 2 })[..],
                [Command::PadRelease { pad: 2 }]
            ));
            assert!(!pads.held[2]);
            assert_eq!(pads.held[114], track_keys);
        }
    }

    #[test]
    fn held_until_release_even_across_banks() {
        let mut p = Pads::default();
        p.apply(press(4));
        assert!(p.held[4]);
        p.apply(Action::BankNext);
        p.apply(Action::PadRelease { pad: 4 });
        assert!(!p.held[4] && !p.held[20]);
        // controller
        p.hit(33);
        assert!(p.held[33]);
        p.release(33);
        assert!(!p.held[33]);
        // no kitty protocol: no releases will come, so keyboard presses do not hold
        p.track_keys = false;
        p.apply(press(1));
        assert!(!p.held[17]);
    }

    /// 60 "Every setting moves": the filter, the pitch and the drive of Q go to V, and Q
    /// is left with what V had.
    #[test]
    fn every_setting_moves() {
        let mut p = Pads::default();
        let (q, v) = (4, 15);
        p.filter[q] = FilterKind::LowPass;
        p.cutoff_hz[q] = 500.0;
        p.reso[q] = 30;
        p.pitch[q] = 3;
        p.drive[q] = 40;
        p.sends[q] = [20, 10];
        p.attack_ms[q] = 15;
        let before = |p: &Pads, a: usize| {
            (
                p.filter[a],
                p.cutoff_hz[a],
                p.reso[a],
                p.pitch[a],
                p.drive[a],
                p.sends[a],
                p.attack_ms[a],
            )
        };
        let (at_q, at_v) = (before(&p, q), before(&p, v));
        p.swap(q, v).unwrap();
        assert_eq!(before(&p, v), at_q, "V has what Q had");
        assert_eq!(before(&p, q), at_v, "Q has what V had");
    }

    #[test]
    fn swap_moves_everything_together() {
        let mut p = Pads::default();
        p.file[0] = Some("/m/kick.wav".into());
        p.gate[0] = true;
        p.volume_db[0] = -5;
        p.missing[3] = true;
        p.region[0] = Region { start: 5, end: 9 };
        // the envelope goes along: the engine swaps the whole pad, the model must too
        (p.attack_ms[0], p.decay_ms[0], p.release_ms[0]) = (10, 5, 200);
        p.group[0] = 2; // and the mute group (70)
        assert!(matches!(
            p.swap(0, 3).unwrap(),
            Command::SwapPads { a: 0, b: 3 }
        ));
        assert_eq!((p.group[3], p.group[0]), (2, 0));
        assert_eq!(
            (p.attack_ms[3], p.decay_ms[3], p.release_ms[3]),
            (10, 5, 200)
        );
        assert_eq!((p.attack_ms[0], p.decay_ms[0], p.release_ms[0]), (0, 0, 0));
        assert_eq!(
            (p.region[3], p.region[0]),
            (Region { start: 5, end: 9 }, Region::WHOLE)
        );
        assert_eq!(
            p.file[3].as_deref(),
            Some(std::path::Path::new("/m/kick.wav"))
        );
        assert!(p.gate[3] && p.volume_db[3] == -5 && p.missing[0]);
        assert!(p.file[0].is_none() && !p.gate[0] && p.volume_db[0] == 0 && !p.missing[3]);
    }

    #[test]
    fn session_round_trip() {
        let mut p = Pads::default();
        p.apply(Action::BankNext);
        p.apply(Action::PadModeNext);
        p.apply(Action::PadParam(-3));
        p.file[16] = Some("/m/amen.wav".into());
        p.file[3] = Some("/m/kick.wav".into());
        p.file[40] = Some("/m/amen.wav".into());
        p.region[40] = Region {
            start: 26_304,
            end: 38_832,
        };
        p.slice[40] = Some(3);
        let s = p.to_session(Some("/m".into()), 48_000);
        assert_eq!(s.pads.len(), 3, "pads at defaults are left out");
        assert_eq!(s.pads["40"].start, Some(0.548));
        assert_eq!(s.pads["3"].start, None);

        let mut q = Pads::default();
        let cmds = q.restore(&s, 48_000);
        assert_eq!(q.region[40], p.region[40]);
        assert_eq!((q.slice[40], q.slice[3]), (Some(3), None));
        assert_eq!(q.region[3], Region::WHOLE);
        // the audio rate changed since: the slice stays at the same time
        let mut r = Pads::default();
        r.restore(&s, 44_100);
        assert_eq!(r.region[40].start, 24_167);
        assert!(q.gate[16] && q.volume_db[16] == -3);
        assert_eq!(
            q.file[3].as_deref(),
            Some(std::path::Path::new("/m/kick.wav"))
        );
        // every pad gets its mode, the defaults too: the engine may hold another project's
        assert!(cmds.iter().any(|c| matches!(
            c,
            Command::SetPadMode {
                pad: 16,
                mode: PadMode::Gate
            }
        )));
        assert!(cmds.iter().any(|c| matches!(
            c,
            Command::SetPadMode {
                pad: 3,
                mode: PadMode::OneShot
            }
        )));
        assert_eq!(
            cmds.iter()
                .filter(|c| matches!(c, Command::SetPadMode { .. }))
                .count(),
            PADS
        );
    }

    #[test]
    fn banks_and_selection_stay_in_range() {
        let mut p = Pads::default();
        p.apply(Action::BankPrev);
        assert_eq!(p.bank, 0);
        for _ in 0..20 {
            p.apply(Action::BankNext);
        }
        assert_eq!(p.bank, 7);
        p.apply(Action::BankSelect(2));
        assert_eq!(p.bank, 2);
        p.apply(Action::BankSelect(99));
        assert_eq!(p.bank, 7);
        p.apply(Action::PadSelect(Dir::Up));
        p.apply(Action::PadSelect(Dir::Left));
        assert_eq!(p.selected, 0);
        for _ in 0..5 {
            p.apply(Action::PadSelect(Dir::Down));
            p.apply(Action::PadSelect(Dir::Right));
        }
        assert_eq!(p.selected, 15);
    }

    #[test]
    fn controller_hit_and_knob_volume() {
        let mut p = Pads::default();
        p.hit(16 + 3);
        assert_eq!(
            p.selected, 0,
            "a hit in another bank does not move the selection"
        );
        p.apply(Action::BankNext);
        p.hit(16 + 3);
        assert_eq!(p.selected, 3);
        assert!((p.volume_pos() - 40.0 / 46.0).abs() < 1e-6);
        assert!(matches!(
            p.set_volume_pos(1.0)[..],
            [Command::SetPadGain { pad: 19, .. }]
        ));
        assert_eq!(p.volume_db[19], MAX_DB);
        assert!(p.set_volume_pos(1.0).is_empty());
        p.set_volume_pos(0.0);
        assert_eq!(p.volume_db[19], MIN_DB);
    }

    /// Spec: "Pitch knob on a bank".
    #[test]
    fn pitch_knob_on_a_bank() {
        let mut p = Pads {
            bank: 1,
            ..Default::default()
        };
        p.pitch[19] = 5; // a melody laid out over bank 2
        p.apply(Action::PadMarkBank);
        assert_eq!(p.scope_name(), "16 marked");
        assert!(
            (p.pitch_pos() - 24.0 / 48.0).abs() < 1e-6,
            "pitch 0 sits at the middle of the knob"
        );
        // the knob turns the selected pad from 0 to +3 semitones
        let cmds = p.set_pitch_pos((3.0 + PITCH_LIMIT as f32) / (2.0 * PITCH_LIMIT as f32));
        assert_eq!(cmds.len(), 16, "every pad of the bank got a command");
        assert!(
            cmds.iter()
                .all(|c| matches!(c, Command::SetPadPitch { .. }))
        );
        assert!(cmds.iter().any(|c| matches!(
            c,
            Command::SetPadPitch {
                pad: 19,
                semitones: 8
            }
        )));
        for pad in 16..32 {
            let want = if pad == 19 { 8 } else { 3 };
            assert_eq!(p.pitch[pad], want, "pad {pad} follows");
        }
        assert_eq!(p.pitch[19], 8, "the differences stay");
        assert_eq!(p.pitch[0], 0, "other banks do not move");
    }

    /// The volume knob moves the scope by the same delta; a pad at a limit stays.
    #[test]
    fn volume_knob_moves_the_bank_by_the_delta() {
        let mut p = Pads {
            bank: 1,
            ..Default::default()
        };
        p.volume_db[17] = -6;
        p.volume_db[18] = MAX_DB;
        p.apply(Action::PadMarkBank);
        // the selected pad is at 0 dB, the knob goes to +2 dB
        let pos = (2.0 - MIN_DB as f32) / (MAX_DB - MIN_DB) as f32;
        let cmds = p.set_volume_pos(pos);
        assert_eq!(p.volume_db[16], 2);
        assert_eq!(p.volume_db[17], -4, "the differences stay");
        assert_eq!(p.volume_db[18], MAX_DB, "a pad at the limit stays");
        assert_eq!(
            cmds.len(),
            15,
            "16 pads of the bank, one already at the top"
        );
    }

    /// A pitch knob beyond the limit leaves the pad at +24 and changes nothing.
    #[test]
    fn a_pitch_knob_past_the_limit_changes_nothing() {
        let mut p = Pads::default();
        p.pitch[0] = PITCH_LIMIT;
        assert!(p.set_pitch_pos(1.0).is_empty());
        assert_eq!(p.pitch[0], PITCH_LIMIT);
    }

    /// Spec: "The envelope follows the scope".
    #[test]
    fn an_envelope_change_follows_the_scope() {
        let mut p = Pads {
            bank: 1,
            ..Default::default()
        };
        p.release_ms[19] = 100; // a longer release laid out on one pad of bank 2
        p.param = Param::Release;
        p.apply(Action::PadMarkBank); // bank 2 marked
        assert_eq!(p.scope_name(), "16 marked");
        assert_eq!(p.param, Param::Release);
        // bank 2 is marked: raise the release by one step of 5 ms
        let cmds = p.apply(Action::PadParam(1));
        assert_eq!(cmds.len(), 16, "every pad of the bank got a command");
        assert!(cmds.iter().all(|c| matches!(
            c,
            Command::SetPadEnvelope {
                pad: _,
                attack_ms: 0,
                decay_ms: 0,
                release_ms: _
            }
        )));
        assert!(cmds.iter().any(|c| matches!(
            c,
            Command::SetPadEnvelope {
                pad: 19,
                release_ms: 105,
                ..
            }
        )));
        for pad in 16..32 {
            let want = if pad == 19 { 105 } else { 5 };
            assert_eq!(p.release_ms[pad], want, "pad {pad} follows");
        }
        assert_eq!(p.release_ms[0], 0, "other banks do not move");
        // the value of the selected pad pops up
        assert_eq!(p.param_value(), "5 ms");
        assert_eq!(p.param_name(), "release");
    }

    /// The envelope stays in 0..5000 ms; a pad at a limit counts into the pop-up.
    #[test]
    fn the_envelope_stays_in_the_limits() {
        let mut p = Pads {
            param: Param::Attack,
            ..Default::default()
        };
        assert!(
            p.apply(Action::PadParam(-1))[..].is_empty(),
            "attack is at 0"
        );
        assert_eq!(p.limit_hits, Some(1));
        p.limit_hits = None;
        p.apply(Action::PadParam(1));
        for _ in 0..(ENV_MS_LIMIT / ENV_STEP_MS) {
            p.apply(Action::PadParam(1));
        }
        assert_eq!(p.attack_ms[0], ENV_MS_LIMIT as u16);
        assert!(p.apply(Action::PadParam(1))[..].is_empty());
        assert_eq!(p.limit_hits, Some(1));
        assert_eq!(p.param_value(), "5000 ms");
    }

    #[test]
    fn mode_and_volume_of_the_selected_pad() {
        let mut p = Pads::default();
        p.apply(Action::BankNext);
        p.apply(Action::PadSelect(Dir::Right));
        assert!(matches!(
            p.apply(Action::PadModeNext)[..],
            [Command::SetPadMode {
                pad: 17,
                mode: PadMode::Gate
            }]
        ));
        for _ in 0..100 {
            p.apply(Action::PadParam(-1));
        }
        assert_eq!(p.volume_db[17], MIN_DB);
        assert!(matches!(
            p.apply(Action::PadParam(1))[..],
            [Command::SetPadGain { pad: 17, gain }]
                if (gain - 10f32.powf(-39.0 / 20.0)).abs() < 1e-6
        ));
    }

    /// Spec: "Pick the parameter".
    #[test]
    fn tab_picks_the_parameter_and_minus_equal_change_it() {
        let mut p = Pads::default();
        // volume is picked first: `=` raises the volume by 1 dB
        assert!(matches!(
            p.apply(Action::PadParam(1))[..],
            [Command::SetPadGain { pad: 0, .. }]
        ));
        assert_eq!(p.volume_db[0], 1);
        // Tab: pitch is picked, `=` raises the pitch, `-` lowers it back
        assert!(p.apply(Action::PadParamNext).is_empty());
        assert_eq!(p.param, Param::Pitch);
        assert!(matches!(
            p.apply(Action::PadParam(1))[..],
            [Command::SetPadPitch {
                pad: 0,
                semitones: 1
            }]
        ));
        assert!(matches!(
            p.apply(Action::PadParam(-1))[..],
            [Command::SetPadPitch {
                pad: 0,
                semitones: 0
            }]
        ));
        // the pick walks all parameters, the mute group (70), FILTER (44), SOURCE (48),
        // the effects (56), and comes back to volume
        for want in [
            Param::Group,
            Param::Attack,
            Param::Decay,
            Param::Release,
            Param::Filter,
            Param::Cutoff,
            Param::Reso,
            Param::Source,
            Param::Start,
            Param::End,
            Param::Drive,
            Param::Send1,
            Param::Send2,
            Param::Volume,
        ] {
            assert!(p.apply(Action::PadParamNext).is_empty());
            assert_eq!(p.param, want);
        }
        // the pitch keeps its value while the other parameters are changed
        assert!(matches!(
            p.apply(Action::PadParam(1))[..],
            [Command::SetPadGain { pad: 0, .. }]
        ));
        assert_eq!(p.pitch[0], 0);
        assert_eq!(p.attack_ms[0], 0);
    }

    #[test]
    fn pitch_stays_in_the_limits() {
        let mut p = Pads::default();
        p.apply(Action::PadParamNext);
        // every press moves until the limit; past it the pad stays and gets no command
        for _ in 0..PITCH_LIMIT {
            assert!(matches!(
                p.apply(Action::PadParam(1))[..],
                [Command::SetPadPitch { .. }]
            ));
        }
        assert_eq!(p.pitch[0], PITCH_LIMIT);
        assert!(p.apply(Action::PadParam(1)).is_empty());
        for _ in 0..2 * i16::from(PITCH_LIMIT) {
            p.apply(Action::PadParam(-1));
        }
        assert_eq!(p.pitch[0], -PITCH_LIMIT);
        assert!(p.apply(Action::PadParam(-1)).is_empty());
    }

    /// 56 "Saved and opened" and "An older project": the drive, the sends and the slots
    /// go into the project file and come back; an older project opens with the drive and
    /// the sends at 0, a reverb in FX 1 and a delay in FX 2.
    #[test]
    fn the_effects_are_saved_and_older_projects_open_without_them() {
        let mut p = Pads::default();
        p.drive[22] = 30;
        p.sends[22] = [40, 0];
        p.fx[1].kind = FxKind::Reverb;
        p.fx[1].size = 70;
        p.fx[1].time_ms = 500; // the delay of the slot is kept while it holds a reverb
        let text = toml::to_string(&p.to_session(None, 48_000)).unwrap();
        let back: Session = toml::from_str(&text).unwrap();
        let mut q = Pads::default();
        let cmds = q.restore(&back, 48_000);
        assert_eq!((q.drive[22], q.sends[22]), (30, [40, 0]));
        assert_eq!(q.fx, p.fx, "{text}");
        // the engine hears them again
        assert!(cmds.iter().any(|c| matches!(
            c,
            Command::SetPadDrive { pad: 22, drive } if (*drive - 0.3).abs() < 1e-6
        )));
        assert!(cmds.iter().any(|c| matches!(
            c,
            Command::SetFx { slot: 1, settings: FxSettings::Reverb { size, .. } }
                if (*size - 0.7).abs() < 1e-6
        )));
        // a project from before the effects
        let old: Session = toml::from_str("version = 3\n\n[pads.5]\nvolume_db = -6\n").unwrap();
        let mut r = Pads::default();
        r.drive[5] = 50;
        r.fx[0].kind = FxKind::Delay;
        r.restore(&old, 48_000);
        assert_eq!((r.drive[5], r.sends[5]), (0, [0, 0]));
        assert_eq!(r.fx, FxSlot::DEFAULTS);
        // a pad with no effects is left out of the file
        assert!(Pads::default().to_session(None, 48_000).pads.is_empty());
    }

    /// 70 pad-playback "The group is saved": pad Q of bank 1 in group 1 goes into the
    /// project file and comes back, and the engine hears it again.
    #[test]
    fn the_group_is_saved() {
        let mut p = Pads::default();
        p.group[4] = 1;
        let text = toml::to_string(&p.to_session(None, 48_000)).unwrap();
        assert!(text.contains("group = 1"), "{text}");
        let back: Session = toml::from_str(&text).unwrap();
        let mut q = Pads::default();
        let cmds = q.restore(&back, 48_000);
        assert_eq!(q.group[4], 1);
        assert!(
            cmds.iter()
                .any(|c| matches!(c, Command::SetPadGroup { pad: 4, group: 1 }))
        );
        // a pad with no group is left out of the file
        assert!(Pads::default().to_session(None, 48_000).pads.is_empty());
    }

    /// 70 pad-playback "An older project": no `group` in the file, every pad is off, and
    /// the engine is told so, as it may still hold the groups of the project before.
    #[test]
    fn an_older_project() {
        let old: Session = toml::from_str("version = 3\n\n[pads.5]\nvolume_db = -6\n").unwrap();
        let mut r = Pads::default();
        r.group[5] = 3;
        let cmds = r.restore(&old, 48_000);
        assert_eq!(r.group[5], 0);
        let told = cmds
            .iter()
            .filter(|c| matches!(c, Command::SetPadGroup { group: 0, .. }))
            .count();
        assert_eq!(told, PADS, "every pad is told it is off");
    }

    /// 44 "Saved and opened" and "An older project".
    #[test]
    fn the_filter_is_saved_and_older_projects_open_with_it_off() {
        let mut p = Pads::default();
        p.filter[5] = FilterKind::Band;
        p.cutoff_hz[5] = 2_000.0;
        p.reso[5] = 40;
        let text = toml::to_string(&p.to_session(None, 48_000)).unwrap();
        assert!(text.contains("filter = \"band\""), "{text}");
        let back: Session = toml::from_str(&text).unwrap();
        let mut q = Pads::default();
        let cmds = q.restore(&back, 48_000);
        assert_eq!(
            (q.filter[5], q.cutoff_hz[5], q.reso[5]),
            (FilterKind::Band, 2_000.0, 40)
        );
        let sent = |cmds: &[Command], pad: usize| {
            cmds.iter().find_map(|c| match c {
                Command::SetPadFilter { pad: p, settings } if *p == pad => Some(*settings),
                _ => None,
            })
        };
        assert_eq!(
            sent(&cmds, 5),
            Some(q.filter_settings(5)),
            "the engine hears it again"
        );
        // a project from before the filter: no keys, every pad off at 1 kHz
        let old: Session = toml::from_str("version = 3\n\n[pads.5]\nvolume_db = -6\n").unwrap();
        let mut r = Pads::default();
        r.filter[5] = FilterKind::LowPass;
        let cmds = r.restore(&old, 48_000);
        assert_eq!(r.filter[5], FilterKind::Off);
        assert_eq!(r.cutoff_hz[5], CUTOFF_DEFAULT);
        assert_eq!(sent(&cmds, 5), Some(r.filter_settings(5)));
        // a pad with the filter at its defaults is left out of the file
        assert!(Pads::default().to_session(None, 48_000).pads.is_empty());
    }

    /// Spec: "Restart" and "Old session".
    #[test]
    fn pitch_is_saved_restored_and_old_sessions_open_at_zero() {
        let mut p = Pads::default();
        p.pitch[0] = 7;
        p.pitch[5] = -24;
        let s = p.to_session(None, 48_000);
        assert_eq!(s.pads["0"].pitch, 7);
        assert_eq!(s.pads["5"].pitch, -24);
        // a session saved before pitch existed: no pitch keys, opens with every pad at 0
        let old: Session = toml::from_str("version = 3\n\n[pads.3]\nvolume_db = -6\n").unwrap();
        let mut q = Pads::default();
        let cmds = q.restore(&old, 48_000);
        assert_eq!(q.pitch[3], 0);
        // every pad is sent its 0, so a pitch of the previous project does not stay
        assert!(cmds.iter().all(|c| !matches!(
            c,
            Command::SetPadPitch { semitones, .. } if *semitones != 0
        )));
        // a session with pitch restores the values and sends them to the engine
        let text = toml::to_string(&s).unwrap();
        let back: Session = toml::from_str(&text).unwrap();
        let mut q = Pads::default();
        let cmds = q.restore(&back, 48_000);
        assert_eq!(q.pitch[0], 7);
        assert_eq!(q.pitch[5], -24);
        assert!(cmds.iter().any(|c| matches!(
            c,
            Command::SetPadPitch {
                pad: 0,
                semitones: 7
            }
        )));
        assert!(cmds.iter().any(|c| matches!(
            c,
            Command::SetPadPitch {
                pad: 5,
                semitones: -24
            }
        )));
    }
    /// Spec: "The scope is always named".
    #[test]
    fn ctrl_a_marks_the_bank_and_the_panel_names_it() {
        let mut p = Pads {
            bank: 1,
            ..Default::default()
        };
        assert_eq!(p.scope_name(), "pad 1");
        assert!(p.apply(Action::PadMarkBank).is_empty());
        assert_eq!(p.scope_name(), "16 marked");
        assert!(p.marking, "Ctrl+A turns marking on");
        assert_eq!(p.scope_pads(), (16..32).collect::<Vec<_>>());
    }

    /// Spec: "The bank, then a few more".
    #[test]
    fn the_bank_then_a_few_more() {
        let mut p = Pads {
            bank: 1,
            ..Default::default()
        };
        p.apply(Action::PadMarkBank);
        p.apply(press(4)); // Q
        p.apply(press(5)); // W
        p.bank = 4;
        for pad in [0, 1, 2] {
            p.apply(press(pad));
        }
        let mut want: BTreeSet<usize> = (16..32).filter(|&a| a != 20 && a != 21).collect();
        want.extend([64, 65, 66]);
        assert_eq!(p.marked, want);
        assert_eq!(p.scope_name(), "17 marked");
    }

    /// Spec: "Ctrl+A again unmarks the bank".
    #[test]
    fn ctrl_a_again_unmarks_the_bank() {
        let mut p = Pads {
            bank: 4,
            ..Default::default()
        };
        p.apply(Action::PadMarkToggle);
        p.apply(press(0));
        p.bank = 1;
        p.apply(Action::PadMarkBank);
        assert_eq!(p.marked.len(), 17);
        p.apply(Action::PadMarkBank);
        assert_eq!(p.marked, BTreeSet::from([64]));
        assert!(p.marking, "marking stays on");
    }

    /// Spec: "A bank lower" — a relative change keeps the differences.
    #[test]
    fn a_bank_lower_keeps_the_melody_shape() {
        let mut p = Pads {
            bank: 1,
            ..Default::default()
        };
        p.apply(Action::PadParamNext);
        p.pitch[16] = 0;
        p.pitch[17] = 3;
        assert!(p.apply(Action::PadMarkBank).is_empty());
        assert_eq!(p.scope_name(), "16 marked");
        p.apply(Action::PadParam(-1));
        p.apply(Action::PadParam(-1));
        assert_eq!(p.pitch[16], -2);
        assert_eq!(p.pitch[17], 1, "the melody keeps its shape");
        // the bank next door is untouched
        assert_eq!(p.pitch[0], 0);
        assert_eq!(p.pitch[32], 0);
    }

    /// Spec: "All pads quieter".
    #[test]
    fn all_pads_quieter() {
        let mut p = Pads::default();
        p.volume_db[63] = 5;
        for bank in 0..PADS / PADS_PER_BANK {
            p.bank = bank;
            assert!(p.apply(Action::PadMarkBank).is_empty());
        }
        let cmds = p.apply(Action::PadParam(-3));
        assert_eq!(cmds.len(), PADS, "every pad gets its own gain");
        for abs in 0..PADS {
            // pad 63 sat at +5: the differences stay
            assert_eq!(
                p.volume_db[abs],
                if abs == 63 { 2 } else { -3 },
                "pad {abs}"
            );
        }
    }

    /// Spec: "A limit" — a pad at the limit stays and is counted for the pop-up.
    #[test]
    fn a_limit_stays_and_the_popup_counts() {
        let mut p = Pads::default();
        p.apply(Action::PadParamNext);
        p.pitch[0] = PITCH_LIMIT;
        p.pitch[1] = 0;
        for bank in 0..PADS / PADS_PER_BANK {
            p.bank = bank;
            assert!(p.apply(Action::PadMarkBank).is_empty());
        }
        let cmds = p.apply(Action::PadParam(1));
        assert_eq!(p.pitch[0], PITCH_LIMIT, "the pad at the limit stays");
        assert_eq!(p.pitch[1], 1);
        assert_eq!(p.limit_hits, Some(1));
        // only the pads that moved get commands
        assert_eq!(cmds.len(), PADS - 1);
        assert!(
            !cmds
                .iter()
                .any(|c| matches!(c, Command::SetPadPitch { pad: 0, .. }))
        );
    }

    /// Spec: "Mode for a bank".
    #[test]
    fn mode_for_a_bank() {
        let mut p = Pads::default();
        assert!(p.apply(Action::PadMarkBank).is_empty());
        let cmds = p.apply(Action::PadModeNext);
        assert_eq!(cmds.len(), PADS_PER_BANK);
        for i in 0..PADS_PER_BANK {
            assert!(p.gate[i], "pad {i}");
        }
        assert!(!p.gate[16], "the bank next door is untouched");
        assert!(cmds.iter().all(|c| matches!(
            c,
            Command::SetPadMode {
                mode: PadMode::Gate,
                ..
            }
        )));
    }

    /// Spec: "Clear stays on one pad".
    #[test]
    fn clear_stays_on_one_pad() {
        let mut p = Pads::default();
        p.file[0] = Some("/m/kick.wav".into());
        p.file[1] = Some("/m/amen.wav".into());
        p.slice[1] = Some(2);
        let cmd = p.clear(p.selected_abs());
        assert!(matches!(
            cmd,
            Command::SetPadSound {
                pad: 0,
                sound: None
            }
        ));
        assert!(p.file[0].is_none() && p.sound[0].is_none());
        assert_eq!(
            p.file[1].as_deref(),
            Some(std::path::Path::new("/m/amen.wav"))
        );
        assert_eq!(p.slice[1], Some(2));
    }

    /// Spec: "Three kicks quieter" — marking, then a relative change on the marks.
    #[test]
    fn three_kicks_quieter() {
        let mut p = Pads::default();
        assert!(p.apply(Action::PadMarkToggle).is_empty());
        // a marked pad still sounds
        for pad in [1, 2, 3] {
            assert!(matches!(
                p.apply(press(pad))[..],
                [Command::PadPress { .. }]
            ));
        }
        assert!(p.apply(Action::PadMarkToggle).is_empty());
        assert_eq!(p.scope_name(), "3 marked");
        let cmds = p.apply(Action::PadParam(-4));
        assert_eq!(cmds.len(), 3);
        assert_eq!(p.volume_db[1], -4);
        assert_eq!(p.volume_db[2], -4);
        assert_eq!(p.volume_db[3], -4);
        assert_eq!(p.volume_db[0], 0, "the unmarked pad keeps its volume");
    }

    /// Spec: "Unmark" — a hit on a marked pad unmarks it.
    #[test]
    fn unmark_by_hitting_again() {
        let mut p = Pads::default();
        assert!(p.apply(Action::PadMarkToggle).is_empty());
        p.apply(press(1));
        assert_eq!(p.marked.len(), 1);
        p.apply(press(1));
        assert!(p.marked.is_empty());
        assert_eq!(p.scope_name(), "pad 2");
    }

    /// Spec: "Drop the marks" — Esc clears the marks, turns marking off, and the scope is
    /// the selected pad.
    #[test]
    fn esc_drops_the_marks() {
        let mut p = Pads::default();
        assert!(p.apply(Action::PadMarkToggle).is_empty());
        p.apply(press(1));
        p.apply(press(2));
        assert!(!p.marked.is_empty());
        assert!(p.drop_marks());
        assert!(p.marked.is_empty());
        assert!(!p.marking, "Esc turns marking off");
        assert_eq!(p.scope_name(), "pad 3", "the last hit stays selected");
        // Esc with no marks goes back as usual
        assert!(!p.drop_marks());
    }

    /// Marking works on controller hits too.
    #[test]
    fn controller_hits_mark_too() {
        let mut p = Pads::default();
        assert!(p.apply(Action::PadMarkToggle).is_empty());
        p.hit(20);
        p.hit(21);
        p.hit(20);
        assert_eq!(p.marked, BTreeSet::from([21]));
    }

    /// Scenario "The attack shows on the pad" (29): with the attack up, the wave rises
    /// more slowly at its start.
    #[test]
    fn the_attack_shows_on_the_pad() {
        let mut p = Pads::default();
        p.sound[0] = Some(Arc::new(Sample::mono(vec![1.0; 48_000])));
        p.attack_ms[0] = 100;
        let fast = p.wave(0, 48_000, 64);
        p.attack_ms[0] = 500;
        let slow = p.wave(0, 48_000, 64);
        assert_eq!(
            fast[8], 1.0,
            "a short attack is done by an eighth of the piece"
        );
        assert!(
            slow[0] < 0.05,
            "the long attack starts in silence: {}",
            slow[0]
        );
        assert!(
            slow[8] < slow[16],
            "still rising: {} then {}",
            slow[8],
            slow[16]
        );
        assert!(slow[8] < fast[8], "the long attack rises more slowly");
        assert_eq!(slow[63], 1.0, "the attack does not touch the end");
    }

    /// 29, task 1: the decay falls to silence at the end of the piece.
    #[test]
    fn the_decay_falls_to_the_end() {
        let mut p = Pads::default();
        p.sound[0] = Some(Arc::new(Sample::mono(vec![1.0; 48_000])));
        p.decay_ms[0] = 500;
        let w = p.wave(0, 48_000, 64);
        assert_eq!(w[0], 1.0, "the decay does not touch the start");
        assert!(w[32] > w[48], "falling: {} then {}", w[32], w[48]);
        assert!(w[63] < 0.05, "ends in silence: {}", w[63]);
    }

    /// Scenario "Every pad draws its wave" (29), the empty pad: a flat line, and a
    /// missing file draws nothing at all.
    #[test]
    fn an_empty_pad_draws_a_flat_line() {
        let mut p = Pads::default();
        assert_eq!(p.wave(0, 48_000, 64), vec![0.0]);
        p.missing[1] = true;
        p.sound[1] = Some(Arc::new(Sample::mono(vec![1.0; 48_000])));
        assert!(p.wave(1, 48_000, 64).is_empty());
    }

    /// 29, task 1: the cache — an unchanged pad is not computed again, a changed one is.
    #[test]
    fn the_cache_keeps_the_wave_of_an_unchanged_pad() {
        let mut p = Pads::default();
        p.sound[0] = Some(Arc::new(Sample::mono(vec![1.0; 48_000])));
        assert!(!p.wave(0, 48_000, 64).is_empty());
        // tamper with the cache: the same key comes back from it untouched
        p.wave_cache[0].as_mut().unwrap().wave = vec![7.0];
        assert_eq!(p.wave(0, 48_000, 64), vec![7.0]);
        // another envelope recomputes
        p.attack_ms[0] = 250;
        assert!(p.wave(0, 48_000, 64).iter().all(|&w| w < 7.0));
        // another region recomputes: an empty piece draws nothing
        p.attack_ms[0] = 0;
        p.region[0].end = 0;
        assert!(p.wave(0, 48_000, 64).is_empty());
    }

    /// 29, review: the width is part of the cache key — a wave asked with another
    /// width is computed again at that width.
    #[test]
    fn a_new_width_recomputes_the_wave() {
        let mut p = Pads::default();
        p.sound[0] = Some(Arc::new(Sample::mono(vec![1.0; 48_000])));
        assert_eq!(p.wave(0, 48_000, 64).len(), 64);
        assert_eq!(
            p.wave(0, 48_000, 32).len(),
            32,
            "the wider terminal gets a wider wave"
        );
    }

    /// 29, review: a cleared pad drops its cache entry, the sample is not held in
    /// memory; a missing file drops it too.
    #[test]
    fn a_cleared_pad_drops_its_cached_wave() {
        let mut p = Pads::default();
        let sound = Arc::new(Sample::mono(vec![1.0; 48_000]));
        p.sound[0] = Some(sound.clone());
        p.wave(0, 48_000, 64);
        assert!(p.wave_cache[0].is_some());
        p.clear(p.selected_abs());
        p.wave(0, 48_000, 64);
        assert!(p.wave_cache[0].is_none(), "the cleared pad's entry is gone");
        assert_eq!(
            Arc::strong_count(&sound),
            1,
            "only the caller holds the sample now"
        );
        // a missing file drops the entry too
        p.sound[1] = Some(Arc::new(Sample::mono(vec![1.0; 48_000])));
        p.missing[1] = true;
        p.wave(1, 48_000, 64);
        assert!(p.wave_cache[1].is_none());
    }

    /// 54 editing: "Tab turns the page".
    #[test]
    fn tab_turns_the_page() {
        let mut p = Pads {
            param: Param::Pitch,
            ..Default::default()
        };
        p.apply(Action::PageNext);
        p.apply(Action::PageNext);
        assert_eq!(p.page().0, "FILTER");
        assert_eq!(p.param, Param::Filter, "the first parameter of the page");
    }

    #[test]
    fn shift_tab_turns_back() {
        let mut p = Pads::default();
        p.apply(Action::PagePrev);
        assert_eq!(p.page().0, "FX");
        assert_eq!(p.param, Param::Drive);
    }
}

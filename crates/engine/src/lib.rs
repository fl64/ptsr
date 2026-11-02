//! Audio engine: voices and mixing. Knows nothing about the UI or the sound card:
//! runs the same way in the JACK/cpal callback and in the offline render.

pub mod checks;
pub mod filter;
pub mod fx;
pub mod offline;
pub mod rt;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

/// How many voices sound at once. Memory for them is allocated up front.
pub const MAX_VOICES: usize = 32;
/// Spare slots for voices that were cut and are fading out.
const FADING_SLOTS: usize = 16;
const SLOTS: usize = MAX_VOICES + FADING_SLOTS;
pub const PADS_PER_BANK: usize = 16;
/// How many banks.
pub const BANKS: usize = 8;
/// 8 banks of 16 pads.
pub const PADS: usize = BANKS * PADS_PER_BANK;
/// Fade-out time of a cut voice: shorter clicks, longer smears the hit.
const FADE_SECONDS: f32 = 0.005;
/// Glide time of a pitch change on a sounding voice: slow enough to hear no step.
const PITCH_GLIDE_SECONDS: f32 = 0.020;
/// Time the envelope of a sounding voice catches up with changed attack or decay.
const ENV_GLIDE_SECONDS: f32 = 0.020;
/// Lowest and highest pad pitch, in semitones.
pub const PITCH_LIMIT: i8 = 24;
/// Engine-side safety cap for the envelope times, in ms; the app keeps them at 0..5000.
const ENV_MS_MAX: u16 = 10_000;
/// The shared effect slots (56).
pub const FX_SLOTS: usize = 2;
/// Frames of the send buses: a longer block is processed in parts of this length.
const BUS_FRAMES: usize = 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PadMode {
    /// Plays to the end.
    #[default]
    OneShot,
    /// Plays while the pad is held.
    Gate,
}

/// A sound in memory at the engine sample rate: mono or stereo.
#[derive(Debug, Default)]
pub struct Sample {
    pub left: Box<[f32]>,
    /// `None` for mono: the left channel plays on both sides.
    pub right: Option<Box<[f32]>>,
}

impl Sample {
    pub fn mono(samples: Vec<f32>) -> Self {
        Self {
            left: samples.into_boxed_slice(),
            right: None,
        }
    }

    pub fn len(&self) -> usize {
        self.left.len()
    }

    pub fn is_empty(&self) -> bool {
        self.left.is_empty()
    }
}

/// The part of a sample a pad plays, in frames at the engine rate; `end` past the sample
/// length means "to the end".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub start: usize,
    pub end: usize,
}

impl Region {
    pub const WHOLE: Region = Region {
        start: 0,
        end: usize::MAX,
    };
}

impl Default for Region {
    fn default() -> Self {
        Self::WHOLE
    }
}

/// What the engine does on command from other threads or from a render scenario.
#[derive(Clone, Debug)]
pub enum Command {
    Click {
        velocity: f32,
    },
    PadPress {
        pad: usize,
        velocity: f32,
    },
    PadRelease {
        pad: usize,
    },
    SetPadMode {
        pad: usize,
        mode: PadMode,
    },
    /// Pad volume, linear (1.0 is 0 dB).
    SetPadGain {
        pad: usize,
        gain: f32,
    },
    /// Pad pitch in semitones, −`PITCH_LIMIT`…+`PITCH_LIMIT`; the next hit plays at it
    /// and a sounding voice glides to it.
    SetPadPitch {
        pad: usize,
        semitones: i8,
    },
    /// Pad envelope times in ms: attack fades in at the piece start, decay fades the
    /// tail to silence so it ends right at the piece end, release fades after a gate
    /// release. Zero attack or decay means off; zero release means the short click-free
    /// cut. A sounding voice follows smoothly.
    SetPadEnvelope {
        pad: usize,
        attack_ms: u16,
        decay_ms: u16,
        release_ms: u16,
    },
    /// The pad filter (44): a sounding voice glides to it, no click; `Off` leaves the
    /// sound untouched.
    SetPadFilter {
        pad: usize,
        settings: filter::FilterSettings,
    },
    /// The pad drive 0..1 (56): 0 leaves the sound untouched; a change glides.
    SetPadDrive {
        pad: usize,
        drive: f32,
    },
    /// What a pad sends to the shared slots, 0..1 each (56); a change glides.
    SetPadSends {
        pad: usize,
        sends: [f32; FX_SLOTS],
    },
    /// The mute group of a pad (70): 0 is off, 1–8 a group whose pads cut each other.
    SetPadGroup {
        pad: usize,
        group: u8,
    },
    /// What a shared slot holds (56): a new kind fades the old tail out and starts empty.
    SetFx {
        slot: usize,
        settings: fx::FxSettings,
    },
    /// A new sample for a pad; `None` makes it silent. Playing voices keep their old sample.
    /// The pad plays the whole sample until a `SetPadRegion` follows.
    SetPadSound {
        pad: usize,
        sound: Option<Arc<Sample>>,
    },
    /// The part of its sample a pad plays from the next hit on; playing voices keep theirs.
    SetPadRegion {
        pad: usize,
        region: Region,
    },
    /// Plays a sound apart from the pads (browser file, chop screen); the previous preview
    /// fades out. `None` only stops the preview.
    Preview {
        sound: Option<Arc<Sample>>,
        region: Region,
    },
    /// The next preview plays as `pad` does (50): its volume, pitch, envelope and filter,
    /// without being the pad; it holds for that one preview only.
    PreviewAs {
        pad: Option<usize>,
    },
    /// The preview loops `region` (49): at its end it starts again from its start; a new
    /// region is taken at the next turn; `None` plays to the end once.
    PreviewLoop {
        region: Option<Region>,
    },
    /// Volume of the whole output, linear (1.0 is 0 dB).
    SetMasterGain {
        gain: f32,
    },
    /// Two pads trade their sample, mode and volume; their sounding voices fade out.
    SwapPads {
        a: usize,
        b: usize,
    },
    /// Every sounding pad voice fades out over the short cut: the chop screen opens and
    /// the owner wants to hear the slices alone. The preview is not touched.
    HushPads,
}

#[derive(Clone)]
struct Pad {
    sound: Option<Arc<Sample>>,
    region: Region,
    mode: PadMode,
    gain: f32,
    /// Volume at the start of the block: ramps to `gain` within the block.
    gain_from: f32,
    /// How many sample frames a voice reads per output frame: `2^(pitch/12)`.
    rate: f32,
    /// Envelope windows in sample frames of the piece: fade-in at the start,
    /// fade-to-silence at the end; `0.0` means off.
    attack: f32,
    decay: f32,
    /// Frames a gate release fades over; `0` means the short `fade_len` cut.
    release_frames: u32,
    /// What the voices of the pad filter to (44).
    filter: filter::Target,
    /// The drive and the sends (56), with their values at the start of the block: they
    /// ramp within the block as the volume does.
    drive: f32,
    drive_from: f32,
    sends: [f32; FX_SLOTS],
    sends_from: [f32; FX_SLOTS],
    /// The mute group (70): 0 is off; a hit cuts the other pads of the same group.
    group: u8,
}

#[derive(Default)]
struct Voice {
    /// `None` means the slot is free. The voice owns a reference, so replacing the pad's
    /// sample does not cut it.
    sound: Option<Arc<Sample>>,
    /// Pad of the voice; `None` is the metronome click or the preview.
    pad: Option<usize>,
    /// The browser preview voice.
    preview: bool,
    /// Position in the sound.
    pos: usize,
    /// Where the voice stops; before the sample end it fades out over the last frames,
    /// so a slice cut in the middle of a sound does not click.
    end: usize,
    /// Sample of the current block where the voice starts.
    delay: usize,
    velocity: f32,
    /// Samples left until silence if the voice was cut.
    fade: Option<u32>,
    /// What `fade` counts down from: the release ramp divides by it.
    fade_total: u32,
    /// Gain at the start of a shortened project-switch fade.
    fade_gain: f32,
    /// Where the voice started in the sound: attack counts from here.
    pos0: usize,
    /// Envelope windows of the pad when the voice started, in frames.
    attack: f32,
    decay: f32,
    /// Envelope gain applied right now; moves towards the position target by
    /// `env_step` per frame, so a changed envelope does not click.
    env: f32,
    /// Fractional part of the read position, `0.0..1.0`.
    frac: f32,
    /// Read speed the voice glides to, in sample frames per output frame.
    rate: f32,
    /// Read speed right now; moves towards `rate` by `rate_step` per output frame.
    rate_now: f32,
    rate_step: f32,
    /// Output frames left until the glide is done.
    glide_left: u32,
    /// Start order: the oldest voice is given up first.
    started: u64,
    /// The pad filter of this voice: its state and the settings it glides with.
    filter: filter::VoiceFilter,
    /// Frames left of the fade-in after a loop seam (49).
    fade_in: u32,
    /// A preview that plays as this pad does (50); it is not the pad: a hit on the pad does
    /// not cut it.
    as_pad: Option<usize>,
}

pub struct Engine {
    voices: [Voice; SLOTS],
    pads: [Pad; PADS],
    click: Arc<Sample>,
    master: f32,
    /// Master gain at the start of the block: ramps to `master` within the block.
    master_from: f32,
    fade_len: u32,
    /// Length of a pitch glide, in output frames.
    glide_len: u32,
    /// Largest envelope gain change per output frame.
    env_step: f32,
    sample_rate: f32,
    /// How far a filter glides towards its target per output frame.
    filter_glide: f32,
    /// The region the preview loops (49); `None` plays it once.
    preview_loop: Option<Region>,
    /// The pad the next preview plays as (50).
    preview_as: Option<usize>,
    started: u64,
    /// Samples the audio thread lets go of are sent back here to be freed elsewhere.
    garbage: Option<rtrb::Producer<Arc<Sample>>>,
    /// Where the preview plays, in frames; `usize::MAX` when it is silent.
    preview_pos: Option<Arc<AtomicUsize>>,
    /// Where the engine reports its output peaks for the UI meters.
    stats: Option<Arc<crate::rt::Stats>>,
    /// The shared effect slots and their send buses, left and right per slot (56).
    slots: [fx::Slot; FX_SLOTS],
    bus: Vec<[f32; 2 * FX_SLOTS]>,
    switch_fade: Option<u32>,
    switch_clear: Option<(usize, usize)>,
    reset_pad: Option<usize>,
}

/// Hands a sample the engine no longer needs to the garbage queue; without a queue
/// (offline render, tests) it is simply dropped.
fn retire(garbage: &mut Option<rtrb::Producer<Arc<Sample>>>, sound: Arc<Sample>) {
    if let Some(g) = garbage {
        // The caller reserves a slot before removing the reference from its owner.
        assert!(g.push(sound).is_ok());
    }
}

fn retire_slot(
    garbage: &mut Option<rtrb::Producer<Arc<Sample>>>,
    sound: &mut Option<Arc<Sample>>,
) -> bool {
    if sound.is_none() {
        return true;
    }
    if garbage.as_ref().is_some_and(|g| g.slots() == 0) {
        return false;
    }
    retire(garbage, sound.take().unwrap());
    true
}

impl Engine {
    /// Engine with the built-in click and test pad sounds.
    pub fn new(sample_rate: u32) -> Self {
        let pads = (0..PADS).map(|p| test_tone(sample_rate, p)).collect();
        Self::with_sounds(sample_rate, click(sample_rate), pads)
    }

    /// `pads` are mono pad sounds in order; pads beyond the list play the last sound.
    pub fn with_sounds(sample_rate: u32, click: Vec<f32>, pads: Vec<Vec<f32>>) -> Self {
        let mut sounds: Vec<Arc<Sample>> = pads
            .into_iter()
            .map(|p| Arc::new(Sample::mono(p)))
            .collect();
        let last = sounds.last().cloned().unwrap_or_default();
        sounds.resize(PADS, last);
        Self {
            voices: std::array::from_fn(|_| Voice::default()),
            pads: std::array::from_fn(|i| Pad {
                sound: Some(sounds[i].clone()),
                region: Region::WHOLE,
                mode: PadMode::OneShot,
                gain: 1.0,
                gain_from: 1.0,
                rate: 1.0,
                attack: 0.0,
                decay: 0.0,
                release_frames: 0,
                filter: filter::Target::BYPASS,
                drive: 0.0,
                drive_from: 0.0,
                sends: [0.0; FX_SLOTS],
                sends_from: [0.0; FX_SLOTS],
                group: 0,
            }),
            click: Arc::new(Sample::mono(click)),
            master: 1.0,
            master_from: 1.0,
            fade_len: ((sample_rate as f32 * FADE_SECONDS) as u32).max(1),
            glide_len: ((sample_rate as f32 * PITCH_GLIDE_SECONDS) as u32).max(1),
            env_step: 1.0 / (ENV_GLIDE_SECONDS * sample_rate as f32),
            sample_rate: sample_rate as f32,
            filter_glide: filter::glide_factor(sample_rate as f32),
            preview_loop: None,
            preview_as: None,
            started: 0,
            garbage: None,
            preview_pos: None,
            stats: None,
            // a new project: a reverb in FX 1, a delay in FX 2 (56)
            slots: [fx::FxKind::Reverb, fx::FxKind::Delay].map(|k| {
                fx::Slot::new(
                    k,
                    sample_rate as f32,
                    ((sample_rate as f32 * FADE_SECONDS) as u32).max(1),
                )
            }),
            bus: vec![[0.0; 2 * FX_SLOTS]; BUS_FRAMES],
            switch_fade: None,
            switch_clear: None,
            reset_pad: None,
        }
    }

    /// Where the audio thread sends samples it lets go of; the other end drops them.
    pub fn set_garbage(&mut self, garbage: rtrb::Producer<Arc<Sample>>) {
        self.garbage = Some(garbage);
    }

    /// Where the engine reports the preview position for the UI.
    pub fn set_preview_pos(&mut self, pos: Arc<AtomicUsize>) {
        pos.store(usize::MAX, Relaxed);
        self.preview_pos = Some(pos);
    }

    /// Where the engine reports its output peaks for the UI meters.
    pub fn set_stats(&mut self, stats: Arc<crate::rt::Stats>) {
        self.stats = Some(stats);
    }

    /// Whether the garbage queue can accept the references released by a command.
    pub fn can_handle(&self, cmd: &Command) -> bool {
        let needed = match cmd {
            Command::SetPadSound { pad, sound } => {
                if *pad < PADS {
                    usize::from(self.pads[*pad].sound.is_some())
                } else {
                    usize::from(sound.is_some())
                }
            }
            Command::Preview { sound: Some(_), .. }
            | Command::Click { .. }
            | Command::PadPress { .. } => {
                usize::from(self.voices.iter().all(|v| v.sound.is_some()))
            }
            _ => 0,
        };
        self.garbage.as_ref().is_none_or(|g| g.slots() >= needed)
    }

    pub fn can_discard(&self, cmd: &Command) -> bool {
        let needed = matches!(
            cmd,
            Command::SetPadSound { sound: Some(_), .. } | Command::Preview { sound: Some(_), .. }
        );
        !needed || self.garbage.as_ref().is_none_or(|g| g.slots() > 0)
    }

    /// Begin a project transition without releasing samples on the audio thread.
    pub fn switch_project(&mut self) {
        for v in &mut self.voices {
            if v.sound.is_some() && v.fade.is_none_or(|f| f > self.fade_len) {
                if let Some(f) = v.fade {
                    v.fade_gain *= f as f32 / v.fade_total.max(1) as f32;
                }
                v.fade = Some(self.fade_len);
                v.fade_total = self.fade_len;
            }
        }
        if self.switch_clear.is_none() && self.switch_fade.is_none() {
            self.switch_fade = Some(self.fade_len);
        }
    }

    /// Advance bounded transition work; true means restore commands may run.
    pub fn project_transition_step(&mut self) -> bool {
        if self.switch_fade.is_some() {
            return false;
        }
        if let Some((slot, mut cursor)) = self.switch_clear {
            if self.slots[slot].clear_step(&mut cursor, 4096) {
                self.switch_clear = (slot + 1 < FX_SLOTS).then_some((slot + 1, 0));
            } else {
                self.switch_clear = Some((slot, cursor));
            }
        }
        if let Some(mut cursor) = self.reset_pad {
            while cursor < PADS {
                let p = &mut self.pads[cursor];
                if !retire_slot(&mut self.garbage, &mut p.sound) {
                    break;
                }
                Self::reset_pad_settings(p);
                cursor += 1;
            }
            self.reset_pad = (cursor < PADS).then_some(cursor);
        }
        self.switch_clear.is_none() && self.reset_pad.is_none()
    }

    fn reset_pad_settings(p: &mut Pad) {
        p.region = Region::WHOLE;
        p.mode = PadMode::OneShot;
        p.gain = 1.0;
        p.gain_from = 1.0;
        p.rate = 1.0;
        p.attack = 0.0;
        p.decay = 0.0;
        p.release_frames = 0;
        p.filter = filter::Target::BYPASS;
        p.drive = 0.0;
        p.drive_from = 0.0;
        p.sends = [0.0; FX_SLOTS];
        p.sends_from = [0.0; FX_SLOTS];
        p.group = 0;
    }

    /// Retire sample references in an obsolete command.
    pub fn discard(&mut self, cmd: Command) {
        match cmd {
            Command::SetPadSound { sound: Some(s), .. }
            | Command::Preview { sound: Some(s), .. } => retire(&mut self.garbage, s),
            _ => {}
        }
    }

    /// Runs the command `offset` samples after the start of the next block.
    pub fn handle(&mut self, offset: usize, cmd: Command) {
        match cmd {
            Command::Click { velocity } => {
                let click = self.click.clone();
                self.start(click, Region::WHOLE, None, offset, velocity, false);
            }
            Command::PadPress { pad, velocity } if pad < PADS => {
                // a new hit cuts the previous voice of the same pad, and the other pads of
                // its mute group with the same short fade (70)
                self.fade_pad(pad);
                let group = self.pads[pad].group;
                if group != 0 {
                    let fade = self.fade_len;
                    for v in self.voices.iter_mut() {
                        if let Some(p) = v.pad
                            && p != pad
                            && v.sound.is_some()
                            && v.fade.is_none()
                            && self.pads[p].group == group
                        {
                            v.fade = Some(fade);
                            v.fade_total = fade;
                        }
                    }
                }
                if let Some(sound) = self.pads[pad].sound.clone() {
                    let region = self.pads[pad].region;
                    self.start(sound, region, Some(pad), offset, velocity, false);
                }
            }
            Command::PadRelease { pad } if pad < PADS => {
                if self.pads[pad].mode == PadMode::Gate {
                    self.release_pad(pad);
                }
            }
            Command::SetPadMode { pad, mode } if pad < PADS => self.pads[pad].mode = mode,
            Command::SetPadGain { pad, gain } if pad < PADS => self.pads[pad].gain = gain.max(0.0),
            Command::SetPadPitch { pad, semitones } if pad < PADS => {
                let semitones = semitones.clamp(-PITCH_LIMIT, PITCH_LIMIT);
                let rate = 2f32.powf(semitones as f32 / 12.0);
                self.pads[pad].rate = rate;
                // a sounding voice glides to the new speed, so the change does not click
                let glide = self.glide_len;
                for v in self
                    .voices
                    .iter_mut()
                    .filter(|v| v.sound.is_some() && (v.pad == Some(pad) || v.as_pad == Some(pad)))
                {
                    v.rate = rate;
                    v.rate_step = (rate - v.rate_now) / glide as f32;
                    v.glide_left = glide;
                }
            }
            Command::SetPadEnvelope {
                pad,
                attack_ms,
                decay_ms,
                release_ms,
            } if pad < PADS => {
                let (attack, decay, release) =
                    self.envelope_frames(attack_ms, decay_ms, release_ms);
                let pad_state = &mut self.pads[pad];
                pad_state.attack = attack;
                pad_state.decay = decay;
                pad_state.release_frames = release;
                // the voices pick the new windows up; the envelope gain glides, no click
                for v in self
                    .voices
                    .iter_mut()
                    .filter(|v| v.sound.is_some() && (v.pad == Some(pad) || v.as_pad == Some(pad)))
                {
                    v.attack = attack;
                    v.decay = decay;
                }
            }
            Command::SetPadFilter { pad, settings } if pad < PADS => {
                self.pads[pad].filter = filter::Target::new(settings, self.sample_rate);
            }
            Command::SetPadGroup { pad, group } if pad < PADS => self.pads[pad].group = group,
            Command::SetPadDrive { pad, drive } if pad < PADS => {
                self.pads[pad].drive = drive.clamp(0.0, 1.0);
            }
            Command::SetPadSends { pad, sends } if pad < PADS => {
                self.pads[pad].sends = sends.map(|s| s.clamp(0.0, 1.0));
            }
            Command::SetFx { slot, settings } if slot < FX_SLOTS => {
                self.slots[slot].set(settings);
            }
            Command::SetPadSound { pad, sound } if pad < PADS => {
                self.pads[pad].region = Region::WHOLE;
                if let Some(old) = std::mem::replace(&mut self.pads[pad].sound, sound) {
                    retire(&mut self.garbage, old);
                }
            }
            Command::SetPadSound { sound: Some(s), .. } => retire(&mut self.garbage, s),
            Command::SetPadRegion { pad, region } if pad < PADS => self.pads[pad].region = region,
            Command::SetMasterGain { gain } => self.master = gain.max(0.0),
            Command::SwapPads { a, b } if a < PADS && b < PADS => {
                self.fade_pad(a);
                self.fade_pad(b);
                self.pads.swap(a, b);
                // playing voices follow their pad to its new index, keeping their own volume
                for v in self.voices.iter_mut() {
                    if v.pad == Some(a) {
                        v.pad = Some(b);
                    } else if v.pad == Some(b) {
                        v.pad = Some(a);
                    }
                }
            }
            Command::Preview { sound, region } => {
                let fade = self.fade_len;
                for v in self
                    .voices
                    .iter_mut()
                    .filter(|v| v.preview && v.fade.is_none())
                {
                    v.fade = Some(fade);
                    v.fade_total = fade;
                }
                // the borrowed pad holds for this preview only (50)
                let as_pad = self.preview_as.take();
                if let Some(sound) = sound {
                    self.start(sound, region, None, offset, 1.0, true);
                    if let Some(p) = as_pad {
                        self.borrow_pad(p);
                    }
                }
            }
            Command::PreviewLoop { region } => self.preview_loop = region,
            Command::PreviewAs { pad } => self.preview_as = pad.filter(|&p| p < PADS),
            Command::HushPads => {
                let fade = self.fade_len;
                for v in self.voices.iter_mut() {
                    if v.sound.is_some() && v.pad.is_some() && !v.preview && v.fade.is_none() {
                        v.fade = Some(fade);
                        v.fade_total = fade;
                    }
                }
            }
            _ => {}
        }
    }

    fn fade_pad(&mut self, pad: usize) {
        let fade = self.fade_len;
        for v in self.voices.iter_mut() {
            if v.sound.is_some() && v.pad == Some(pad) && v.fade.is_none() {
                v.fade = Some(fade);
                v.fade_total = fade;
            }
        }
    }

    /// A gate release: the voices fade over the pad's release time, not the short cut.
    fn release_pad(&mut self, pad: usize) {
        let default_fade = self.fade_len;
        let release = self.pads[pad].release_frames;
        let fade = if release == 0 { default_fade } else { release };
        for v in self.voices.iter_mut() {
            if v.sound.is_some() && v.pad == Some(pad) && v.fade.is_none() {
                v.fade = Some(fade);
                v.fade_total = fade;
            }
        }
    }

    /// Envelope windows in frames of the piece; zero attack or decay means off,
    /// zero release means the short cut.
    fn envelope_frames(&self, attack_ms: u16, decay_ms: u16, release_ms: u16) -> (f32, f32, u32) {
        let ms = |n: u16| n.min(ENV_MS_MAX) as f32 * 0.001 * self.sample_rate;
        (ms(attack_ms), ms(decay_ms), (ms(release_ms)) as u32)
    }

    /// The newest preview voice plays as pad `p` (50): its pitch, envelope and filter from
    /// the start, its volume through the mix.
    fn borrow_pad(&mut self, p: usize) {
        let pad = &self.pads[p];
        let (rate, attack, decay, target) = (pad.rate, pad.attack, pad.decay, pad.filter);
        let sr = self.sample_rate;
        let Some(v) = self
            .voices
            .iter_mut()
            .filter(|v| v.preview && v.sound.is_some() && v.fade.is_none())
            .max_by_key(|v| v.started)
        else {
            return;
        };
        v.as_pad = Some(p);
        (v.rate, v.rate_now, v.attack, v.decay) = (rate, rate, attack, decay);
        v.env = if attack > 0.0 {
            0.0
        } else if decay > 0.0 {
            ((v.end - v.pos) as f32 / decay).clamp(0.0, 1.0)
        } else {
            1.0
        };
        v.filter = filter::VoiceFilter::start(target, sr);
    }

    fn start(
        &mut self,
        sound: Arc<Sample>,
        region: Region,
        pad: Option<usize>,
        offset: usize,
        velocity: f32,
        preview: bool,
    ) {
        let playing = |v: &&mut Voice| v.sound.is_some() && v.fade.is_none();
        // at most MAX_VOICES sounding voices: the oldest one starts fading
        if self.voices.iter_mut().filter(playing).count() >= MAX_VOICES
            && let Some(oldest) = self
                .voices
                .iter_mut()
                .filter(playing)
                .min_by_key(|v| v.started)
        {
            let fade = self.fade_len;
            oldest.fade = Some(fade);
            oldest.fade_total = fade;
        }
        // no free slot: take the fading voice with the least time left
        let slot = self
            .voices
            .iter()
            .position(|v| v.sound.is_none())
            .unwrap_or_else(|| {
                (0..SLOTS)
                    .min_by_key(|&i| self.voices[i].fade.unwrap_or(u32::MAX))
                    .unwrap_or(0)
            });
        self.started += 1;
        if let Some(old) = self.voices[slot].sound.take() {
            retire(&mut self.garbage, old);
        }
        let end = region.end.min(sound.len());
        let rate = pad.map_or(1.0, |p| self.pads[p].rate);
        let (attack, decay) = pad.map_or((0.0, 0.0), |p| (self.pads[p].attack, self.pads[p].decay));
        let pos = region.start.min(end);
        // the envelope starts where the position puts it, so a voice without
        // attack and decay enters at full level, like before the envelope existed
        let env = if attack > 0.0 {
            0.0
        } else if decay > 0.0 {
            ((end - pos) as f32 / decay).clamp(0.0, 1.0)
        } else {
            1.0
        };
        self.voices[slot] = Voice {
            sound: Some(sound),
            pad,
            pos,
            end,
            delay: offset,
            velocity: velocity.clamp(0.0, 1.0),
            preview,
            fade: None,
            fade_total: 0,
            fade_gain: 1.0,
            pos0: pos,
            attack,
            decay,
            env,
            frac: 0.0,
            rate,
            rate_now: rate,
            rate_step: 0.0,
            glide_left: 0,
            started: self.started,
            fade_in: 0,
            as_pad: None,
            filter: filter::VoiceFilter::start(
                pad.map_or(filter::Target::BYPASS, |p| self.pads[p].filter),
                self.sample_rate,
            ),
        };
    }

    /// Fills a block. Any block length; left and right have the same length. A block
    /// longer than the send buses is filled in parts.
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let n = left.len().min(right.len());
        let mut at = 0;
        while at < n {
            let end = (at + BUS_FRAMES).min(n);
            self.process_part(&mut left[at..end], &mut right[at..end]);
            at = end;
        }
    }

    fn process_part(&mut self, left: &mut [f32], right: &mut [f32]) {
        let n = left.len().min(right.len());
        let (left, right) = (&mut left[..n], &mut right[..n]);
        left.fill(0.0);
        right.fill(0.0);
        let bus = &mut self.bus[..n];
        bus.fill([0.0; 2 * FX_SLOTS]);
        let fade_len = self.fade_len as f32;
        let env_step = self.env_step;
        for v in self.voices.iter_mut() {
            let Some(sound) = &v.sound else {
                continue;
            };
            if v.delay >= n {
                v.delay -= n;
                continue;
            }
            let (l, r) = (&sound.left, sound.right.as_ref().unwrap_or(&sound.left));
            // a preview playing as a pad takes its volume and filter (50)
            let src = v.pad.or(v.as_pad);
            let (g0, g1) = src.map_or((1.0, 1.0), |p| (self.pads[p].gain_from, self.pads[p].gain));
            // the drive and the sends of the pad (56), ramping within the block
            let (d0, d1) = src.map_or((0.0, 0.0), |p| {
                (self.pads[p].drive_from, self.pads[p].drive)
            });
            let fixed_drive = (d0 == d1 && d0 > 0.0).then(|| fx::Drive::new(d0));
            let (s0_, s1_) = src.map_or(([0.0; FX_SLOTS], [0.0; FX_SLOTS]), |p| {
                (self.pads[p].sends_from, self.pads[p].sends)
            });
            let sending = s0_.iter().chain(s1_.iter()).any(|&s| s > 0.0);
            let target = src.map_or(filter::Target::BYPASS, |p| self.pads[p].filter);
            let (glide, sr) = (self.filter_glide, self.sample_rate);
            // the looping preview fades out at every seam, the sample's end too (49)
            let looping = if v.preview && v.fade.is_none() {
                self.preview_loop
                    .map(|r| Region {
                        start: r.start.min(l.len()),
                        end: r.end.min(l.len()),
                    })
                    .filter(|r| r.end > r.start + 2 * self.fade_len as usize)
            } else {
                None
            };
            // only a slice that stops before the sample end needs its own fade-out
            let mut tail = if v.end < l.len() || looping.is_some() {
                v.end
            } else {
                usize::MAX
            };
            let mut i = v.delay;
            while i < n && v.fade != Some(0) {
                if v.pos >= v.end {
                    // the next turn of the loop takes the region as it is now
                    let Some(r) = looping else {
                        break;
                    };
                    (v.pos, v.pos0, v.end, v.frac) = (r.start, r.start, r.end, 0.0);
                    v.fade_in = self.fade_len;
                    tail = r.end;
                }
                let mut g = v.velocity * (g0 + (g1 - g0) * i as f32 / n as f32);
                // the pad envelope: attack rises from the piece start, decay dies
                // at its end; the gain glides to the target, a change does not click
                let mut env_target: f32 = 1.0;
                if v.attack > 0.0 {
                    env_target = env_target.min((v.pos - v.pos0) as f32 / v.attack);
                }
                if v.decay > 0.0 {
                    env_target *= ((v.end - v.pos) as f32 / v.decay).clamp(0.0, 1.0);
                }
                v.env += (env_target - v.env).clamp(-env_step, env_step);
                g *= v.env;
                if tail - v.pos < self.fade_len as usize {
                    g *= (tail - v.pos) as f32 / fade_len;
                }
                if v.fade_in > 0 {
                    g *= 1.0 - v.fade_in as f32 / fade_len;
                    v.fade_in -= 1;
                }
                if let Some(f) = v.fade.as_mut() {
                    g *= v.fade_gain * *f as f32 / v.fade_total.max(1) as f32;
                    *f -= 1;
                }
                // linear interpolation between the frames around the read position
                let f = v.frac;
                let p1 = (v.pos + 1).min(v.end - 1);
                let (s0, s1) = (l[v.pos], l[p1]);
                let (t0, t1) = (
                    r.get(v.pos).copied().unwrap_or(0.0),
                    r.get(p1).copied().unwrap_or(0.0),
                );
                let [mut sl, mut sr_] =
                    v.filter
                        .tick(&target, [s0 + (s1 - s0) * f, t0 + (t1 - t0) * f], glide, sr);
                let t = i as f32 / n as f32;
                if d0 > 0.0 || d1 > 0.0 {
                    let d = d0 + (d1 - d0) * t;
                    let drive = fixed_drive.unwrap_or_else(|| fx::Drive::new(d));
                    (sl, sr_) = (drive.apply(sl), drive.apply(sr_));
                }
                left[i] += sl * g;
                right[i] += sr_ * g;
                if sending {
                    for k in 0..FX_SLOTS {
                        let send = g * (s0_[k] + (s1_[k] - s0_[k]) * t);
                        bus[i][2 * k] += sl * send;
                        bus[i][2 * k + 1] += sr_ * send;
                    }
                }
                // the read position advances by the rate; while the rate glides,
                // it moves from `rate_now` towards `rate` one step per frame
                let rate = if v.glide_left == 0 {
                    v.rate
                } else {
                    v.rate_now += v.rate_step;
                    v.glide_left -= 1;
                    if v.glide_left == 0 {
                        v.rate_now = v.rate;
                    }
                    v.rate_now
                };
                v.frac += rate;
                let whole = v.frac as usize;
                v.frac -= whole as f32;
                v.pos += whole;
                i += 1;
            }
            v.delay = 0;
            // a looping preview ends only when it is stopped
            if (v.pos >= v.end && looping.is_none()) || v.fade == Some(0) {
                // A completed voice stays silent in its slot until garbage has room.
                v.fade = Some(0);
                retire_slot(&mut self.garbage, &mut v.sound);
            }
        }
        for p in self.pads.iter_mut() {
            p.gain_from = p.gain;
            p.drive_from = p.drive;
            p.sends_from = p.sends;
        }
        // the shared slots add their wet sound to the pads' own (56)
        if self.switch_clear.is_none() {
            for (k, slot) in self.slots.iter_mut().enumerate() {
                for (i, frame) in bus.iter().enumerate() {
                    let [wl, wr] = slot.tick([frame[2 * k], frame[2 * k + 1]]);
                    let gain = self.switch_fade.map_or(1.0, |f| {
                        f.saturating_sub(i as u32) as f32 / self.fade_len as f32
                    });
                    left[i] += wl * gain;
                    right[i] += wr * gain;
                }
            }
        }
        if let Some(f) = self.switch_fade {
            if f as usize <= n {
                self.switch_fade = None;
                self.reset_pad = Some(0);
                self.preview_loop = None;
                self.preview_as = None;
                self.switch_clear = Some((0, 0));
            } else {
                self.switch_fade = Some(f - n as u32);
            }
        }
        if let Some(at) = &self.preview_pos {
            let pos = self
                .voices
                .iter()
                .find(|v| v.preview && v.sound.is_some() && v.fade.is_none())
                .map_or(usize::MAX, |v| v.pos);
            at.store(pos, Relaxed);
        }
        let (m0, m1) = (self.master_from, self.master);
        for (i, (l, r)) in left.iter_mut().zip(right.iter_mut()).enumerate() {
            let g = m0 + (m1 - m0) * i as f32 / n as f32;
            *l *= g;
            *r *= g;
        }
        self.master_from = self.master;
        if let Some(stats) = &self.stats {
            let peak = |b: &[f32]| b.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            stats.record_output(peak(left), peak(right));
        }
    }
}

/// Metronome click: 1.5 kHz, 1 ms attack, decays to zero in 30 ms.
/// The attack does not start at zero, so the onset is the first sample.
pub fn click(sample_rate: u32) -> Vec<f32> {
    let sr = sample_rate as f32;
    let attack = (sr * 0.001) as usize;
    let len = (sr * 0.030) as usize;
    (0..len)
        .map(|i| {
            let env = if i < attack {
                (i + 1) as f32 / attack as f32
            } else {
                (1.0 - (i - attack) as f32 / (len - attack) as f32).powi(3)
            };
            0.5 * env * (std::f32::consts::TAU * 1500.0 * i as f32 / sr).cos()
        })
        .collect()
}

/// Semitone of a pad test sound: each pad is one higher, each bank 3 semitones higher.
fn test_semitone(pad: usize) -> usize {
    pad % PADS_PER_BANK + 3 * (pad / PADS_PER_BANK)
}

/// Pad test sound for pads without a sample: a 1 s decaying tone that starts and ends without a click.
pub fn test_tone(sample_rate: u32, pad: usize) -> Vec<f32> {
    let sr = sample_rate as f32;
    let freq = 110.0 * 2f32.powf(test_semitone(pad) as f32 / 12.0);
    let attack = (sr * 0.002) as usize;
    let len = sample_rate as usize;
    (0..len)
        .map(|i| {
            let t = i as f32 / len as f32;
            let env = ((i + 1) as f32 / attack as f32).min(1.0) * (1.0 - t).powi(3);
            0.3 * env * (std::f32::consts::TAU * freq * i as f32 / sr).sin()
        })
        .collect()
}

/// Note name of a pad test sound, e.g. `A2`.
pub fn test_tone_name(pad: usize) -> String {
    const NAMES: [&str; 12] = [
        "A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#",
    ];
    let s = test_semitone(pad);
    // 110 Hz is A2; the octave number changes at C
    let octave = 2 + (s + 9) / 12;
    format!("{}{octave}", NAMES[s % 12])
}

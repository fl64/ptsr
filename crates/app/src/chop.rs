//! The chop screen state: cuts over a pad's sample, slice selection and editing with undo,
//! and the plan of which slice goes to which pad. Pure logic; the app turns it into engine
//! commands.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use engine::{PADS, PADS_PER_BANK, Region, Sample};

use crate::session::ChopState;

/// Equal-parts counts, in the order Tab steps through them.
pub const PARTS: [usize; 5] = [2, 4, 8, 16, 32];
/// Undo steps kept.
/// The closest the wave view may get, frames: a dot of the picture stays a frame or two.
const MIN_VIEW: usize = 256;
const UNDO: usize = 64;
/// `turning` of the region knobs: no boundary index of a slice ever reaches this.
const REGION_TURN: usize = usize::MAX;
/// `turning` of the threshold knob: the same, one below the region's.
const THRESHOLD_TURN: usize = usize::MAX - 1;

/// What a cut in time did: refused (outside the range or too close to another cut),
/// placed where pressed, or snapped to the nearest hit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cut {
    Refused,
    Placed,
    Snapped,
}

/// The cuts of a file as the session keeps them (frames at the engine rate).
#[derive(Clone, Debug, PartialEq)]
pub struct Saved {
    pub range: Region,
    /// (id, position) of every cut, in order.
    pub cuts: Vec<(u32, usize)>,
    pub epoch: u32,
    pub next_id: u32,
}

impl Saved {
    /// In seconds for the session, so the cuts survive a change of the audio rate.
    pub fn to_state(&self, rate: u32) -> ChopState {
        let secs = |f: usize| f as f64 / rate.max(1) as f64;
        ChopState {
            start: secs(self.range.start),
            end: secs(self.range.end),
            cuts: self.cuts.iter().map(|c| secs(c.1)).collect(),
            ids: self.cuts.iter().map(|c| c.0).collect(),
            epoch: self.epoch,
            next_id: self.next_id,
        }
    }

    pub fn from_state(st: &ChopState, rate: u32) -> Self {
        let frames = |s: f64| (s.max(0.0) * rate as f64).round() as usize;
        Saved {
            range: Region {
                start: frames(st.start),
                end: frames(st.end),
            },
            cuts: st
                .ids
                .iter()
                .copied()
                .zip(st.cuts.iter().map(|&c| frames(c)))
                .collect(),
            epoch: st.epoch,
            next_id: st.next_id,
        }
    }
}

/// Repeats of a held key from the tap step to the top one.
pub const RAMP_REPEATS: u32 = 6;

/// The hit threshold before `Ctrl+Y` or the knob set one (51): the swells inside one sound
/// stay below it on the owner's recordings.
const HIT_FLOOR_DB: f32 = -12.0;

/// How the arrows walk the cursor on the chop screen; stepped with `N`, not saved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Snap {
    /// Even steps of the view.
    #[default]
    Off,
    /// Hit to hit; a cut lands on the nearest hit.
    Hits,
    /// Cut to cut (43).
    Cuts,
}

impl Snap {
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Hits,
            Self::Hits => Self::Cuts,
            Self::Cuts => Self::Off,
        }
    }

    /// The help line name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Hits => "hits",
            Self::Cuts => "cuts",
        }
    }
}

/// What play covers on the chop screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayMode {
    /// From the cursor to the end of the region.
    #[default]
    ToEnd,
    /// From the cursor to the end of the slice the cursor is in.
    ToCut,
    /// The whole slice the cursor is in.
    Slice,
}

impl PlayMode {
    pub fn next(self) -> Self {
        match self {
            Self::ToEnd => Self::ToCut,
            Self::ToCut => Self::Slice,
            Self::Slice => Self::ToEnd,
        }
    }

    /// The help line name.
    pub fn name(self) -> &'static str {
        match self {
            Self::ToEnd => "to end",
            Self::ToCut => "to cut",
            Self::Slice => "slice",
        }
    }
}

/// One undo step: the cuts with their ids, the range, the epoch and the threshold,
/// as they were before a change.
type UndoStep = (Vec<usize>, Vec<u32>, Region, u32, f32);

pub struct Chop {
    pub file: PathBuf,
    pub sound: Arc<Sample>,
    /// The whole file: the screen shows it and the range edges stay inside it.
    pub view: Region,
    /// The part of the sample being chopped; its edges can be trimmed.
    pub range: Region,
    /// Cut positions in frames, sorted, strictly inside `range`.
    pub markers: Vec<usize>,
    /// The id of each cut, so a pad stays linked to its slice when other cuts come and go.
    /// The range start has id 0.
    ids: Vec<u32>,
    /// The last position of every cut ever made: a pad linked to a cut that was deleted plays
    /// the slice that now covers that place.
    known: HashMap<u32, usize>,
    next_id: u32,
    /// Grows when all cuts are replaced (equal parts): pads linked to an older epoch keep
    /// their sound but no longer follow the cuts; undo brings the epoch and the links back.
    pub epoch: u32,
    next_epoch: u32,
    /// The selected slice.
    pub selected: usize,
    /// The owner's cursor (a frame inside the range): only the owner moves it.
    pub cursor: usize,
    /// What play covers from the cursor; stepped with `O`, not saved.
    pub play: PlayMode,
    /// `Space` loops what `play` picks (49); stepped with `Alt+O`, starts off, not saved.
    pub looping: bool,
    /// What the chop screen plays sounds as the pad of the slice (50); `Alt+P`, starts off.
    pub as_pad: bool,
    /// How the arrows walk the cursor: even steps, hit to hit, cut to cut.
    pub snap: Snap,
    /// Hit starts in the sample with their strength (the peak of their block), sorted.
    hits: Vec<(usize, f32)>,
    /// The low, mid and high level of every `BAND_BLOCK` frames (38): the wave colors.
    pub bands: Vec<[f32; 3]>,
    /// The average of each band over the sounding blocks: a column is colored by the band
    /// that stands out against it.
    pub band_mean: [f32; 3],
    /// The auto-chop threshold in dB below the loudest hit of the region; `Ctrl+Y` steps
    /// it, the threshold knob sets it.
    pub threshold_db: f32,
    /// `Ctrl+Y` or the threshold knob set the threshold (51); until then the hits use
    /// `HIT_FLOOR_DB`.
    threshold_set: bool,
    /// The last equal-parts count, so Tab goes on from it.
    pub parts: Option<usize>,
    rate: usize,
    undo: Vec<UndoStep>,
    /// The boundary a knob is turning: the whole turn is one undo step.
    turning: Option<usize>,
}

impl Chop {
    /// The whole file, no cuts.
    pub fn new(file: PathBuf, sound: Arc<Sample>, rate: u32) -> Self {
        let whole = Region {
            start: 0,
            end: sound.len(),
        };
        Self::from_saved(
            file,
            sound,
            rate,
            Saved {
                range: whole,
                cuts: Vec::new(),
                epoch: 1,
                next_id: 1,
            },
        )
    }

    pub fn from_saved(file: PathBuf, sound: Arc<Sample>, rate: u32, saved: Saved) -> Self {
        let len = sound.len();
        let range = Region {
            start: saved.range.start.min(len),
            end: saved.range.end.min(len),
        };
        let cuts: Vec<_> = saved
            .cuts
            .into_iter()
            .filter(|&(_, p)| p > range.start && p < range.end)
            .collect();
        let hits = find_hits(&sound.left, rate.max(1) as usize);
        let bands = find_bands(&sound.left, rate.max(1) as f32);
        let band_mean = band_mean(&bands);
        Self {
            file,
            sound,
            view: Region { start: 0, end: len },
            range,
            markers: cuts.iter().map(|c| c.1).collect(),
            ids: cuts.iter().map(|c| c.0).collect(),
            known: cuts.iter().copied().collect(),
            next_id: saved
                .next_id
                .max(cuts.iter().map(|c| c.0 + 1).max().unwrap_or(1)),
            epoch: saved.epoch,
            next_epoch: saved.epoch + 1,
            selected: 0,
            cursor: range.start,
            play: PlayMode::default(),
            looping: false,
            as_pad: false,
            snap: Snap::Off,
            threshold_db: 0.0,
            threshold_set: false,
            hits,
            bands,
            band_mean,
            parts: None,
            rate: rate.max(1) as usize,
            undo: Vec::new(),
            turning: None,
        }
    }

    /// No cuts and the whole file: nothing a fresh chop would replace.
    pub fn is_blank(&self) -> bool {
        self.markers.is_empty() && self.range == self.view
    }

    /// A chop of the same file at `epoch` with no cuts (45); this one stays as it is, with
    /// its pads.
    pub fn fresh(&self, epoch: u32) -> Self {
        let whole = Region {
            start: 0,
            end: self.sound.len(),
        };
        Self {
            file: self.file.clone(),
            sound: self.sound.clone(),
            view: whole,
            range: whole,
            markers: Vec::new(),
            ids: Vec::new(),
            known: HashMap::new(),
            next_id: 1,
            epoch,
            next_epoch: epoch + 1,
            selected: 0,
            cursor: 0,
            play: self.play,
            looping: self.looping,
            as_pad: self.as_pad,
            snap: self.snap,
            hits: self.hits.clone(),
            bands: self.bands.clone(),
            band_mean: self.band_mean,
            threshold_db: 0.0,
            threshold_set: false,
            parts: None,
            rate: self.rate,
            undo: Vec::new(),
            turning: None,
        }
    }

    /// The next epoch this chop gives out is at least `floor`: the chops of one file never
    /// share an epoch, pads tell them apart by it (45).
    pub fn reserve_epochs(&mut self, floor: u32) {
        self.next_epoch = self.next_epoch.max(floor);
    }

    /// The ids of the cuts, in order; a pad follows the cut at its slice start.
    pub fn cut_ids(&self) -> &[u32] {
        &self.ids
    }

    pub fn saved(&self) -> Saved {
        Saved {
            range: self.range,
            cuts: self
                .ids
                .iter()
                .copied()
                .zip(self.markers.iter().copied())
                .collect(),
            epoch: self.epoch,
            next_id: self.next_id,
        }
    }

    fn ms(&self, ms: usize) -> usize {
        self.rate * ms / 1000
    }

    fn save_undo(&mut self) {
        if self.undo.len() == UNDO {
            self.undo.remove(0);
        }
        self.undo.push((
            self.markers.clone(),
            self.ids.clone(),
            self.range,
            self.epoch,
            self.threshold_db,
        ));
    }

    /// Range start, cuts, range end.
    fn boundaries(&self) -> Vec<usize> {
        let mut b = vec![self.range.start];
        b.extend(&self.markers);
        b.push(self.range.end);
        b
    }

    pub fn slices(&self) -> Vec<Region> {
        self.boundaries()
            .windows(2)
            .map(|w| Region {
                start: w[0],
                end: w[1],
            })
            .collect()
    }

    /// The slice that covers frame `pos`.
    pub fn slice_at(&self, pos: usize) -> usize {
        self.markers.partition_point(|&m| m <= pos)
    }

    /// The id a pad keeps to stay linked to slice `i`: the id of the cut at its start.
    pub fn start_id(&self, i: usize) -> u32 {
        if i == 0 { 0 } else { self.ids[i - 1] }
    }

    /// The slice a pad linked to cut `id` plays now.
    pub fn slice_of(&self, id: u32) -> usize {
        if id == 0 {
            return 0;
        }
        match self.ids.iter().position(|&i| i == id) {
            Some(i) => i + 1,
            None => self.known.get(&id).map_or(0, |&p| self.slice_at(p)),
        }
    }

    /// A cut at `pos` without undo, when a chop is first made from a pad's slice edges.
    pub fn ensure_cut(&mut self, pos: usize) {
        if pos <= self.range.start || pos >= self.range.end || self.markers.contains(&pos) {
            return;
        }
        let i = self.markers.partition_point(|&m| m < pos);
        self.insert(i, pos);
    }

    fn insert(&mut self, i: usize, pos: usize) {
        let id = self.next_id;
        self.next_id += 1;
        self.markers.insert(i, pos);
        self.ids.insert(i, id);
        self.known.insert(id, pos);
    }

    /// Where boundary `k` may go: between its neighbours (the edges of the file for the
    /// range ends), 1 ms away from each.
    fn zone(&self, k: usize) -> (usize, usize) {
        let b = self.boundaries();
        let gap = self.ms(1).max(1);
        let low = if k == 0 {
            self.view.start
        } else {
            b[k - 1] + gap
        };
        let high = b
            .get(k + 1)
            .map_or(self.view.end, |&n| n.saturating_sub(gap));
        (low, high.max(low))
    }

    fn set_boundary(&mut self, k: usize, pos: usize) {
        let (low, high) = self.zone(k);
        let pos = pos.clamp(low, high);
        let last = self.markers.len() + 1;
        match k {
            0 => self.range.start = pos,
            k if k == last => self.range.end = pos,
            k => {
                self.markers[k - 1] = pos;
                self.known.insert(self.ids[k - 1], pos);
            }
        }
    }

    fn boundary(&self, k: usize) -> usize {
        self.boundaries()[k]
    }

    /// Replaces the cuts with the next equal-parts count over the range; returns the count.
    /// This starts a new epoch: pads linked to the old cuts let go of them.
    pub fn next_parts(&mut self) -> usize {
        // a chop from the session does not know its last count: the slices tell it
        let now = self.markers.len() + 1;
        let last = Some(now).filter(|n| PARTS.contains(n)).or(self.parts);
        let n = match last {
            Some(p) => PARTS[(PARTS.iter().position(|&x| x == p).unwrap_or(0) + 1) % PARTS.len()],
            None => PARTS[0],
        };
        self.save_undo();
        self.turning = None;
        self.epoch = self.next_epoch;
        self.next_epoch += 1;
        let (start, len) = (self.range.start, self.range.end - self.range.start);
        let mut cuts: Vec<usize> = (1..n).map(|i| start + len * i / n).collect();
        cuts.dedup();
        cuts.retain(|&m| m > start && m < self.range.end);
        self.markers.clear();
        self.ids.clear();
        for (i, pos) in cuts.into_iter().enumerate() {
            self.insert(i, pos);
        }
        self.parts = Some(n);
        self.selected = 0;
        n
    }

    /// Replaces the cuts with the starts of the hits within the threshold below the
    /// loudest hit of the region; returns the number of cuts.
    fn cut_by_hits(&mut self) -> usize {
        // the marks of the hits mode are these cuts (51)
        let cuts = self.strong_hits();
        self.markers.clear();
        self.ids.clear();
        for (i, pos) in cuts.into_iter().enumerate() {
            self.insert(i, pos);
        }
        self.selected = 0;
        self.markers.len()
    }

    /// Chops by hits, stepping the threshold −6, −12, −18, −24, −30 dB below the loudest
    /// hit of the region, then −6 again; returns (threshold dB, number of cuts). Like
    /// equal parts this replaces the cuts: a new epoch, one undo step, linked pads let go.
    pub fn chop_by_hits(&mut self) -> (f32, usize) {
        let next = [-6.0, -12.0, -18.0, -24.0, -30.0]
            .into_iter()
            .find(|&s| s < self.threshold_db - 0.01)
            .unwrap_or(-6.0);
        self.save_undo();
        self.turning = None;
        self.epoch = self.next_epoch;
        self.next_epoch += 1;
        self.threshold_db = next;
        self.threshold_set = true;
        let n = self.cut_by_hits();
        (next, n)
    }

    /// Chops by hits, stepping the threshold to the next higher listed value
    /// (−6, −12, −18, −24, −30 dB), stopping at −6 dB without wrapping (83a). Before any
    /// hit threshold is set it starts at −6 dB. A knob value at or above −6 dB stays:
    /// `None` means nothing changed and no undo step was made; `Some((dB, cuts))` stepped
    /// and replaced the cuts as `chop_by_hits` does.
    pub fn chop_by_hits_back(&mut self) -> Option<(f32, usize)> {
        let next = if !self.threshold_set {
            -6.0
        } else {
            // the next higher listed value while below −6 dB; at or above it: nothing
            [-30.0, -24.0, -18.0, -12.0, -6.0]
                .into_iter()
                .find(|&s| s > self.threshold_db + 0.01)?
        };
        self.save_undo();
        self.turning = None;
        self.epoch = self.next_epoch;
        self.next_epoch += 1;
        self.threshold_db = next;
        self.threshold_set = true;
        let n = self.cut_by_hits();
        Some((next, n))
    }

    /// The threshold as a knob position 0..1: 0 is 0 dB, 1 is −40 dB below the loudest hit.
    pub fn threshold_pos(&self) -> f32 {
        (-self.threshold_db / 40.0).clamp(0.0, 1.0)
    }

    /// Sets the threshold from a knob position 0..1 (0…−40 dB below the loudest hit) and
    /// recuts at once; `true` when the threshold moved. A whole turn of the knob, however
    /// many wiggles, is one undo step, as with the region knobs.
    pub fn set_threshold(&mut self, pos: f32) -> bool {
        let db = -40.0 * pos.clamp(0.0, 1.0);
        let db = if db == 0.0 { 0.0 } else { db }; // −0.0 would print as "-0 dB"
        if (db - self.threshold_db).abs() < 0.01 {
            return false;
        }
        if self.turning != Some(THRESHOLD_TURN) {
            self.save_undo();
            self.turning = Some(THRESHOLD_TURN);
        }
        self.threshold_db = db;
        self.threshold_set = true;
        self.cut_by_hits();
        true
    }

    /// The start of the strong hit nearest `pos` within the snap window (±90 ms), if any.
    fn nearest_hit(&self, pos: usize) -> Option<usize> {
        self.strong_hits()
            .into_iter()
            .filter(|&h| h.abs_diff(pos) <= self.ms(90))
            .min_by_key(|&h| h.abs_diff(pos))
    }

    /// The hits the threshold keeps (51), inside the region: at most the threshold below
    /// its loudest hit; −12 dB until `Ctrl+Y` or the knob set one.
    /// The cursor's hits mode, the snap of a cut, the marks and `Ctrl+Y` all use these.
    pub fn strong_hits(&self) -> Vec<usize> {
        let (start, end) = (self.range.start, self.range.end);
        let inside = |p: usize| p > start && p < end;
        let loudest = self
            .hits
            .iter()
            .filter(|&&(p, _)| inside(p))
            .fold(0.0f32, |m, &(_, s)| m.max(s));
        let floor = if self.threshold_set {
            self.threshold_db
        } else {
            HIT_FLOOR_DB
        };
        let min = loudest * 10f32.powf(floor / 20.0);
        self.hits
            .iter()
            .filter(|&&(p, s)| inside(p) && s >= min)
            .map(|&(p, _)| p)
            .collect()
    }

    /// A cut at `pos` (a playing cursor): where pressed, or moved to the nearest hit
    /// within 90 ms while the hits mode is on. `Refused` when it is outside the range
    /// or closer than 10 ms to another cut, which would make a slice too short to hear.
    pub fn cut(&mut self, pos: usize) -> Cut {
        let at = if self.snap == Snap::Hits {
            self.nearest_hit(pos).unwrap_or(pos)
        } else {
            pos
        };
        let gap = self.ms(10);
        if at <= self.range.start
            || at >= self.range.end
            || self.boundaries().iter().any(|&m| m.abs_diff(at) < gap)
        {
            return Cut::Refused;
        }
        self.save_undo();
        self.turning = None;
        let i = self.markers.partition_point(|&m| m < at);
        self.insert(i, at);
        // the new slice starts at the cut
        self.selected = i + 1;
        if at == pos { Cut::Placed } else { Cut::Snapped }
    }

    /// Selects the previous (`-1`) or next (`1`) slice; the cursor jumps to its start,
    /// so play and edit keep one position.
    pub fn select(&mut self, step: i8) {
        let last = self.markers.len();
        self.selected = if step < 0 {
            self.selected.saturating_sub(1)
        } else {
            (self.selected + 1).min(last)
        };
        self.cursor = self.boundary(self.selected);
    }

    /// Selects slice `i`; `false` when there is no such slice and the selection stays.
    /// The cursor jumps to the slice start.
    pub fn select_slice(&mut self, i: usize) -> bool {
        let ok = i <= self.markers.len();
        if ok {
            self.selected = i;
            self.cursor = self.boundary(i);
        }
        ok
    }

    /// Moves the cursor one step: 1/32 of the view, 1/256 with `fine`; in hits mode to the
    /// next hit on screen instead, when there is one. The slice under it becomes selected.
    /// The step of the cursor and the bounds: a share of the visible wave, so they move alike
    /// at any zoom. A tap (0 repeats) and the first repeat are 1/128, about a cell; then a
    /// held key grows over `RAMP_REPEATS` repeats to 1/32 per repeat.
    pub fn view_step(&self, repeats: u32) -> usize {
        let view = self.view.end - self.view.start;
        let grown = repeats.saturating_sub(1).min(RAMP_REPEATS) as usize;
        let ramp = RAMP_REPEATS as usize;
        (view * (ramp + 3 * grown) / (ramp * 128)).max(1)
    }

    /// Moves the cursor by `step` frames; in hits mode it jumps to the next hit in view,
    /// in cuts mode to the next cut.
    pub fn move_cursor(&mut self, right: bool, step: usize) {
        if self.snap == Snap::Cuts {
            return self.jump_cursor(right);
        }
        let even = if right {
            (self.cursor + step).min(self.range.end)
        } else {
            self.cursor.saturating_sub(step).max(self.range.start)
        };
        let strong = if self.snap == Snap::Hits {
            self.strong_hits()
        } else {
            Vec::new()
        };
        let hit = if right {
            let end = self.range.end.min(self.view.end);
            strong.iter().find(|&&h| h > self.cursor && h < end)
        } else {
            let start = self.range.start.max(self.view.start);
            strong
                .iter()
                .rev()
                .find(|&&h| h < self.cursor && h >= start)
        };
        self.cursor = hit.copied().unwrap_or(even);
        self.selected = self.slice_at(self.cursor);
        self.follow();
    }

    /// Puts the cursor on the next cut to the right (`right`), or the previous one to the
    /// left; past the last cut, the end of the range, before the first, its start.
    pub fn jump_cursor(&mut self, right: bool) {
        let bounds = self.boundaries();
        let to = if right {
            bounds.iter().copied().find(|&b| b > self.cursor)
        } else {
            bounds.iter().rev().copied().find(|&b| b < self.cursor)
        };
        self.cursor = to.unwrap_or(if right {
            self.range.end
        } else {
            self.range.start
        });
        self.selected = self.slice_at(self.cursor);
        self.follow();
    }

    /// Puts the cursor at `pos` (clamped into the range); the slice under it is selected.
    /// Only tests jump the cursor: in the program the owner walks it (21).
    #[cfg(test)]
    pub fn set_cursor(&mut self, pos: usize) {
        self.cursor = pos.clamp(self.range.start, self.range.end);
        self.selected = self.slice_at(self.cursor);
        self.follow();
    }

    /// The cursor as a knob position 0..1 inside the region.
    pub fn cursor_pos(&self) -> f32 {
        (self.cursor - self.range.start) as f32 / (self.range.end - self.range.start).max(1) as f32
    }

    /// Puts the cursor at a knob position 0..1 inside the region; the slice under it
    /// is selected, the view follows.
    pub fn set_cursor_pos(&mut self, pos: f32) {
        let len = self.range.end - self.range.start;
        self.cursor =
            self.range.start + ((len as f32 * pos.clamp(0.0, 1.0)).round() as usize).min(len);
        self.selected = self.slice_at(self.cursor);
        self.follow();
    }

    /// What play covers: from the cursor to the region end.
    pub fn play_region(&self) -> Region {
        let start = self.cursor.min(self.range.end.saturating_sub(1));
        let slice = self.slices()[self.slice_at(start)];
        match self.play {
            PlayMode::ToEnd => Region {
                start,
                end: self.range.end,
            },
            PlayMode::ToCut => Region {
                start,
                end: slice.end,
            },
            PlayMode::Slice => slice,
        }
    }

    /// The region starts at `pos`: equal parts and playback cover from here.
    pub fn set_region_start(&mut self, pos: usize) {
        self.set_range(pos, self.range.end);
    }

    /// The region ends at `pos`.
    pub fn set_region_end(&mut self, pos: usize) {
        self.set_range(self.range.start, pos);
    }

    /// The region is the whole sample again.
    pub fn clear_region(&mut self) {
        self.set_range(0, self.sound.len());
    }

    /// A new region with the same guarantees as a loaded one: inside the file,
    /// at least one frame long, markers only inside it, the cursor inside it.
    fn set_range(&mut self, start: usize, end: usize) {
        if (start, end) == (self.range.start, self.range.end) {
            return;
        }
        self.save_undo();
        self.apply_range(start, end);
    }

    /// The body of `set_range` without the undo step: the region knobs keep a whole
    /// turn as one undo step, like the slice edge knobs do.
    fn apply_range(&mut self, start: usize, end: usize) {
        let len = self.sound.len();
        let end = end.min(len);
        let start = start.min(end.saturating_sub(1));
        let end = end.max(start + 1).min(len.max(1));
        self.range = Region { start, end };
        // ids go with their cuts, or linked pads look up slices that are gone
        let (markers, ids) = self
            .markers
            .iter()
            .zip(&self.ids)
            .filter(|&(&m, _)| m > start && m < end)
            .unzip();
        self.markers = markers;
        self.ids = ids;
        self.cursor = self.cursor.clamp(start, end);
        self.selected = self.slice_at(self.cursor);
    }

    /// Zooms the view ×2 in or out around the cursor, with limits; the view keeps
    /// the cursor at the same place of the picture.
    pub fn zoom(&mut self, zoom_in: bool) {
        let whole = self.sound.len();
        let len = self.view.end.saturating_sub(self.view.start);
        let new_len = if zoom_in {
            (len / 2).max(MIN_VIEW.min(len))
        } else {
            (len * 2).min(whole)
        };
        if new_len == len {
            return;
        }
        let at = (self.cursor.saturating_sub(self.view.start)) as f32 / len.max(1) as f32;
        let start = (self.cursor as f32 - at * new_len as f32) as usize;
        self.view.start = start.min(whole.saturating_sub(new_len));
        self.view.end = self.view.start + new_len;
        self.follow();
    }

    /// The zoom as a knob position 0..1 on a log scale: 0 is the whole file,
    /// 1 is the closest view (`MIN_VIEW`).
    pub fn zoom_pos(&self) -> f32 {
        let whole = self.sound.len();
        if whole <= MIN_VIEW {
            return 1.0;
        }
        let len = self
            .view
            .end
            .saturating_sub(self.view.start)
            .clamp(1, whole) as f32;
        (1.0 - (len / MIN_VIEW as f32).ln() / (whole as f32 / MIN_VIEW as f32).ln()).clamp(0.0, 1.0)
    }

    /// Sets the zoom from a knob position 0..1 on a log scale (0 is the whole file,
    /// 1 the closest view), the cursor keeping its place of the picture, like `zoom` does.
    pub fn set_zoom(&mut self, pos: f32) {
        let whole = self.sound.len();
        let len = if whole <= MIN_VIEW {
            whole
        } else {
            let span = (whole as f32 / MIN_VIEW as f32).ln();
            (MIN_VIEW as f32 * (span * (1.0 - pos.clamp(0.0, 1.0))).exp()).round() as usize
        }
        .clamp(MIN_VIEW.min(whole), whole);
        let now = self.view.end.saturating_sub(self.view.start);
        if len == now {
            return;
        }
        let at = (self.cursor.saturating_sub(self.view.start)) as f32 / now.max(1) as f32;
        let start = (self.cursor as f32 - at * len as f32) as usize;
        self.view.start = start.min(whole.saturating_sub(len));
        self.view.end = self.view.start + len;
        self.follow();
    }

    /// The view on slice `i` with half of it on each side (37): the slice the owner came for
    /// fills the middle, the neighbours show at the edges; the cursor at the slice start
    /// lands on the quarter of the view, where `follow` leaves it in place.
    pub fn view_slice(&mut self, i: usize) {
        if let Some(r) = self.slices().get(i).copied() {
            self.view_region(r);
        }
    }

    /// The view on `r`: `r` in the middle, half of its length on each side.
    pub fn view_region(&mut self, r: Region) {
        let whole = self.sound.len();
        let len = (r.end - r.start).max(1);
        let span = (len * 2).max(MIN_VIEW.min(whole)).min(whole);
        let start = (r.start + len / 2)
            .saturating_sub(span / 2)
            .min(whole - span);
        self.view = Region {
            start,
            end: start + span,
        };
    }

    /// Keeps the cursor inside the middle half of the view; the view follows the cursor.
    fn follow(&mut self) {
        let len = self.view.end.saturating_sub(self.view.start);
        if len >= self.sound.len() {
            self.view.start = 0;
            self.view.end = self.sound.len();
            return;
        }
        let quarter = len / 4;
        if self.cursor < self.view.start + quarter {
            self.view.start = self.cursor.saturating_sub(quarter);
            self.view.end = self.view.start + len;
        } else if self.cursor + quarter > self.view.end {
            self.view.end = (self.cursor + quarter).min(self.sound.len());
            self.view.start = self.view.end.saturating_sub(len);
        }
    }

    /// Moves the start (`end` false) or the end of the selected slice by `ms` milliseconds
    /// (negative is left); the start of the first slice trims the start of the sample.
    pub fn nudge(&mut self, end: bool, ms: i32) {
        let frames = self.ms(ms.unsigned_abs() as usize).max(1);
        self.nudge_by(end, ms >= 0, frames);
    }

    /// Moves a bound of the selected slice by `step` frames, right or left.
    pub fn nudge_by(&mut self, end: bool, right: bool, step: usize) {
        let k = self.selected + end as usize;
        let now = self.boundary(k);
        let want = if !right {
            now.saturating_sub(step)
        } else {
            now + step
        };
        let (low, high) = self.zone(k);
        if want.clamp(low, high) != now {
            self.save_undo();
            self.turning = None;
            self.set_boundary(k, want);
        }
    }

    /// Removes the cut at the start of the selected slice: it joins the previous one.
    pub fn delete(&mut self) {
        if self.selected == 0 {
            return;
        }
        self.save_undo();
        self.turning = None;
        self.markers.remove(self.selected - 1);
        self.ids.remove(self.selected - 1);
        self.selected -= 1;
    }

    /// Deletes the cut at the end of the selected slice; the last slice ends at the region's
    /// edge and keeps it.
    pub fn delete_end(&mut self) {
        if self.selected >= self.markers.len() {
            return;
        }
        self.save_undo();
        self.turning = None;
        self.markers.remove(self.selected);
        self.ids.remove(self.selected);
    }

    /// Knob position 0..1 of the start (`end` false) or end of the selected slice in its zone.
    pub fn edge_pos(&self, end: bool) -> f32 {
        let k = self.selected + end as usize;
        let (low, high) = self.zone(k);
        (self.boundary(k) - low) as f32 / (high - low).max(1) as f32
    }

    /// Sets the start or end of the selected slice from a knob position 0..1.
    pub fn set_edge(&mut self, end: bool, pos: f32) {
        let k = self.selected + end as usize;
        let (low, high) = self.zone(k);
        let to = low + ((high - low) as f32 * pos.clamp(0.0, 1.0)).round() as usize;
        if to == self.boundary(k) {
            return;
        }
        if self.turning != Some(k) {
            self.save_undo();
            self.turning = Some(k);
        }
        self.set_boundary(k, to);
    }

    /// The region start (`end` false) or end as a knob position 0..1 of the file.
    pub fn region_edge_pos(&self, end: bool) -> f32 {
        let at = if end {
            self.range.end
        } else {
            self.range.start
        };
        at as f32 / self.sound.len().max(1) as f32
    }

    /// Sets the region start or end from a knob position 0..1 of the file; the region
    /// stays at least 10 ms long, a whole turn of the knob is one undo step.
    pub fn set_region_edge(&mut self, end: bool, pos: f32) {
        let whole = self.sound.len();
        let gap = self.ms(10).max(1);
        let to = (whole as f32 * pos.clamp(0.0, 1.0)).round() as usize;
        let (start, end_at) = if end {
            (self.range.start, to.max(self.range.start + gap).min(whole))
        } else {
            (to.min(self.range.end.saturating_sub(gap)), self.range.end)
        };
        if (start, end_at) == (self.range.start, self.range.end) {
            return;
        }
        if self.turning != Some(REGION_TURN) {
            self.save_undo();
            self.turning = Some(REGION_TURN);
        }
        self.apply_range(start, end_at);
    }

    /// Brings back the cuts, range and threshold before the last change; `false` when
    /// there is nothing.
    pub fn undo(&mut self) -> bool {
        let Some((markers, ids, range, epoch, threshold_db)) = self.undo.pop() else {
            return false;
        };
        for (&id, &pos) in ids.iter().zip(&markers) {
            self.known.insert(id, pos);
        }
        self.markers = markers;
        self.ids = ids;
        self.range = range;
        self.epoch = epoch;
        self.threshold_db = threshold_db;
        // back before any Ctrl+Y: the threshold is unset again (a knob at 0 dB undone to
        // 0 dB reads as unset too, an edge without a cut to lose)
        self.threshold_set = threshold_db != 0.0;
        self.turning = None;
        // the slice under the cursor is the one the line, the edit and the bounds act on (35)
        self.cursor = self.cursor.clamp(self.range.start, self.range.end);
        self.selected = self.slice_at(self.cursor);
        true
    }
}

/// Frames per block of the band levels (38).
pub const BAND_BLOCK: usize = 256;

/// The low (under ~200 Hz), mid and high (over ~4 kHz) RMS of every `BAND_BLOCK` frames,
/// from two one-pole filters (38): what plays in a column — a kick, a snare, a hat — reads
/// as the wave color. One pass over the sound, when a chop is made.
fn find_bands(wave: &[f32], rate: f32) -> Vec<[f32; 3]> {
    let coef = |hz: f32| 1.0 - (-std::f32::consts::TAU * hz / rate).exp();
    let (a_low, a_high) = (coef(200.0), coef(4000.0));
    let (mut low, mut below_high) = (0.0f32, 0.0f32);
    wave.chunks(BAND_BLOCK)
        .map(|block| {
            let mut sum = [0.0f32; 3];
            for &x in block {
                low += a_low * (x - low);
                below_high += a_high * (x - below_high);
                let parts = [low, below_high - low, x - below_high];
                for (s, p) in sum.iter_mut().zip(parts) {
                    *s += p * p;
                }
            }
            sum.map(|s| (s / block.len() as f32).sqrt())
        })
        .collect()
}

/// The average level of each band over the blocks that sound (not silence).
fn band_mean(bands: &[[f32; 3]]) -> [f32; 3] {
    let loud: Vec<_> = bands
        .iter()
        .filter(|b| b.iter().sum::<f32>() >= 1e-4)
        .collect();
    if loud.is_empty() {
        return [1.0; 3];
    }
    let mut sum = [0.0f32; 3];
    for b in &loud {
        for (s, v) in sum.iter_mut().zip(b.iter()) {
            *s += v;
        }
    }
    sum.map(|s| (s / loud.len() as f32).max(1e-6))
}

/// Hit starts: a 256-frame block at least 1.5× as loud as the 2 blocks before it (~10 ms)
/// and above −40 dB, 50 ms apart; a longer look back misses hits on the tail of the last one; the start is the block's first frame at half its peak.
/// 04d picked this own detector over aubio (close on breaks, no dependency); aubio can replace it here.
fn find_hits(wave: &[f32], rate: usize) -> Vec<(usize, f32)> {
    const BLOCK: usize = 256;
    let peak = |b: &[f32]| b.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    let peaks: Vec<f32> = wave.chunks(BLOCK).map(peak).collect();
    let gap = rate * 50 / 1000;
    let mut hits: Vec<(usize, f32)> = Vec::new();
    for (i, &p) in peaks.iter().enumerate() {
        let before = peaks[i.saturating_sub(2)..i]
            .iter()
            .fold(0.0f32, |m, &v| m.max(v));
        if p < 0.01 || p < 1.5 * before {
            continue;
        }
        let block = &wave[i * BLOCK..];
        let at = i * BLOCK + block.iter().position(|v| v.abs() >= p / 2.0).unwrap_or(0);
        if hits.last().is_none_or(|&(h, _)| at >= h + gap) {
            hits.push((at, p));
        }
    }
    hits
}

/// How the assign mode plans slices onto pads.
#[derive(Clone, Debug, PartialEq)]
pub enum Plan {
    /// Slice 1 on this pad, the rest after it, across banks; `None` until the owner hits the
    /// start pad.
    InOrder(Option<usize>),
    /// (slice, pad) pairs put one by one.
    ByHand(Vec<(usize, usize)>),
}

impl Plan {
    /// (slice, pad) pairs for `count` slices, in pad order.
    pub fn targets(&self, count: usize) -> Vec<(usize, usize)> {
        let mut t: Vec<_> = match self {
            Plan::InOrder(None) => Vec::new(),
            Plan::InOrder(Some(from)) => (0..count).zip(*from..PADS).collect(),
            Plan::ByHand(pairs) => pairs.iter().copied().filter(|p| p.0 < count).collect(),
        };
        t.sort_by_key(|p| p.1);
        t
    }

    /// A pad hit in the assign mode with `slice` selected: in order the chain starts there,
    /// by hand the slice goes there (replacing what the plan had on that pad).
    pub fn hit(&mut self, pad: usize, slice: usize) {
        match self {
            Plan::InOrder(from) => *from = Some(pad),
            Plan::ByHand(pairs) => {
                pairs.retain(|p| p.1 != pad);
                pairs.push((slice, pad));
            }
        }
    }

    fn planned(&self, count: usize) -> Vec<bool> {
        let mut done = vec![false; count];
        for (slice, _) in self.targets(count) {
            done[slice] = true;
        }
        done
    }

    pub fn all_planned(&self, count: usize) -> bool {
        self.planned(count).iter().all(|&d| d)
    }

    /// The next slice not planned yet: after `after`, else the first one left.
    pub fn next_unplanned(&self, count: usize, after: usize) -> Option<usize> {
        let done = self.planned(count);
        (after + 1..count).chain(0..count).find(|&i| !done[i])
    }
}

/// Where the plan goes, for the summary ("9 slices → …") and the status after applying it
/// ("slices placed on …"): "banks 1–2 · replaces 2 · 1 left out". `None` when nothing is
/// planned. `taken` tells whether a pad holds something now.
pub fn summary(
    targets: &[(usize, usize)],
    count: usize,
    taken: impl Fn(usize) -> bool,
) -> Option<String> {
    let (first, last) = targets.first().zip(targets.last())?;
    let (b0, b1) = (first.1 / PADS_PER_BANK + 1, last.1 / PADS_PER_BANK + 1);
    let mut s = if b0 == b1 {
        format!("bank {b0}")
    } else {
        format!("banks {b0}–{b1}")
    };
    let replaced = targets.iter().filter(|p| taken(p.1)).count();
    if replaced > 0 {
        s += &format!(" · replaces {replaced}");
    }
    let placed: std::collections::BTreeSet<_> = targets.iter().map(|p| p.0).collect();
    let left = count - placed.len();
    if left > 0 {
        s += &format!(" · {left} left out");
    }
    Some(s)
}

/// What the plan would overwrite, by bank: "bank 2: Q kick, W amen 3". `taken` gives the
/// label of a pad that holds something ("Q kick"), `None` for an empty pad.
pub fn overwrites(targets: &[(usize, usize)], taken: impl Fn(usize) -> Option<String>) -> String {
    let mut banks: Vec<(usize, Vec<String>)> = Vec::new();
    for &(_, pad) in targets {
        let Some(label) = taken(pad) else {
            continue;
        };
        let bank = pad / PADS_PER_BANK + 1;
        match banks.last_mut() {
            Some((b, items)) if *b == bank => items.push(label),
            _ => banks.push((bank, vec![label])),
        }
    }
    banks
        .iter()
        .map(|(b, items)| format!("bank {b}: {}", items.join(", ")))
        .collect::<Vec<_>>()
        .join("; ")
}

use crate::*;

impl App {
    /// Opens the chop screen for the selected pad's sample.
    pub(super) fn open_chop(&mut self) {
        let abs = self.pads.selected_abs();
        let (Some(sound), Some(file)) = (self.pads.sound[abs].clone(), self.pads.file[abs].clone())
        else {
            let why = if self.pads.loading[abs].is_some() {
                "pad is still loading"
            } else {
                "pad is empty: put a sample on it first (F6)"
            };
            self.post_notice(Level::Warn, (why.into(), Instant::now()));
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
            self.post_notice(
                Level::Info,
                (
                    "a new chop of this file; the older one keeps its pads, Ctrl+Z goes back to it"
                        .into(),
                    Instant::now(),
                ),
            );
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
    pub(super) fn show_bank_of_chop(&mut self) {
        if self.pattern_owns_pads() {
            return;
        }
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
    pub(super) fn open_file_chop(&mut self) {
        let Some(path) = self
            .browser
            .current()
            .filter(|e| !e.is_dir)
            .map(|e| self.browser.dir.join(&e.name))
        else {
            self.post_notice(
                Level::Warn,
                ("select a file to chop".into(), Instant::now()),
            );
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
                self.chop_loading = Some(path.clone());
                self.request(CHOP, path, self.load_gen[CHOP]);
            }
        }
    }

    pub(super) fn show_file_chop(&mut self, path: PathBuf, sound: Arc<engine::Sample>) {
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
    pub(super) fn normalize_pad(&mut self) {
        let no_pad = "no pad to set the volume on".to_string();
        let Some(abs) = self.chop_from else {
            self.post_notice(Level::Warn, (no_pad, Instant::now()));
            return;
        };
        let Some(sound) = self.pads.sound[abs].clone() else {
            self.post_notice(Level::Warn, (no_pad, Instant::now()));
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
        self.post_notice(
            if raw > f32::from(pads::MAX_DB) {
                Level::Warn
            } else {
                Level::Done
            },
            (text, Instant::now()),
        );
    }

    /// Makes the chop of `file` the open one; `true` when it had to be made new.
    pub(super) fn focus_chop(&mut self, file: PathBuf, sound: Arc<engine::Sample>) -> bool {
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
    pub(super) fn park(&mut self, c: chop::Chop) {
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
    pub(super) fn which_chop(&self) -> Option<(usize, usize)> {
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
    pub(super) fn chop_lives(&self, file: &Path, epoch: u32) -> bool {
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
    pub(super) fn is_newest(&self, c: &chop::Chop) -> bool {
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
    pub(super) fn linked_to(&self, file: &Path, epoch: u32) -> bool {
        (0..PADS).any(|a| {
            self.pads.file[a].as_deref() == Some(file)
                && self.pads.link[a].is_some_and(|l| l.1 == epoch)
        })
    }

    /// The first epoch no chop of `file` and no pad of it uses (45).
    pub(super) fn epoch_floor(&self, file: &Path) -> u32 {
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
    pub(super) fn focus_epoch(
        &mut self,
        file: PathBuf,
        sound: Arc<engine::Sample>,
        epoch: u32,
    ) -> bool {
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
    pub(super) fn rebuild(&self, file: &Path, epoch: u32) -> Option<chop::Saved> {
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
    pub(super) fn undo_fresh(&mut self) -> bool {
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
    pub(super) fn sync_links(&mut self) -> usize {
        let Some(c) = &self.chop else {
            return 0;
        };
        let slices = c.slices();
        let (range, file) = (c.range, c.file.clone());
        let mut changed = Vec::new();
        let mut left_out = Vec::new();
        let linked: [bool; PADS] =
            std::array::from_fn(|abs| matches!(self.pads.source(abs), pads::Source::Slice(_)));
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
        for (abs, was_linked) in linked.into_iter().enumerate() {
            if was_linked != matches!(self.pads.source(abs), pads::Source::Slice(_)) {
                self.send(self.pads.pitch_gate_command(abs));
            }
        }
        if !changed.is_empty() || !left_out.is_empty() {
            self.dirty();
        }
        changed.len() + left_out.len()
    }

    /// The status after an edit that changed laid-out pads.
    pub(super) fn say_changed(&mut self, n: usize, undo: bool) {
        if n == 0 {
            return;
        }
        let pads = if n == 1 { "pad" } else { "pads" };
        let text = if undo {
            format!("{n} {pads} changed back")
        } else {
            format!("{n} {pads} changed; Ctrl+Z returns")
        };
        self.post_notice(Level::Info, (text, Instant::now()));
    }

    /// Slices of the open chop that no pad plays: they wear a `○` (36).
    pub(super) fn slices_on_no_pad(&self) -> usize {
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
    pub(super) fn linked_pads(&self) -> usize {
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
    pub(super) fn playhead(&self) -> Option<usize> {
        if self.screen != Screen::Chop {
            return None;
        }
        self.preview_frame()
    }

    /// Where the preview of the files screen plays, 0..1 of the file (33); nothing while it
    /// is silent or the file has not loaded yet.
    pub(super) fn preview_at(&self) -> Option<f32> {
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
    pub(super) fn preview_frame(&self) -> Option<usize> {
        #[cfg(test)]
        if self.fake_playhead.is_some() {
            return self.fake_playhead;
        }
        let pos = self.audio.as_ref()?.preview_pos.load(Relaxed);
        (pos != usize::MAX).then_some(pos)
    }

    /// Plays slice `slice` of the chop screen.
    pub(super) fn play_slice(&mut self, slice: usize) {
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
    pub(super) fn as_pad_of(&self, slice: usize) -> Option<usize> {
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
    pub(super) fn chop_pad(
        &mut self,
        pad: usize,
        velocity: f32,
        source: pads::HitSource,
    ) -> Option<usize> {
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
                return None;
            }
            let mut queued = false;
            for cmd in self.pads.press(pad, velocity, source) {
                queued |= self.try_send(cmd);
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
            return queued.then_some(abs);
        };
        let c = self.chop.as_mut()?;
        let abs = self.pads.bank * PADS_PER_BANK + pad;
        self.pads.selected = pad;
        let (count, before) = (c.slices().len(), c.selected);
        let by_hand = matches!(plan, chop::Plan::ByHand(_));
        let all_placed = format!("all {count} slices placed: Enter applies");
        if by_hand && plan.all_planned(count) {
            self.post_notice(Level::Info, (all_placed, Instant::now()));
        } else {
            plan.hit(abs, before);
            if by_hand {
                match plan.next_unplanned(count, before) {
                    Some(next) => {
                        c.select_slice(next);
                    }
                    None => self.post_notice(Level::Info, (all_placed, Instant::now())),
                }
            }
            self.change_plan(plan.clone(), before);
        }
        if let Some(&(slice, _)) = plan.targets(count).iter().find(|t| t.1 == abs) {
            self.play_slice(slice);
        }
        None
    }

    /// A new plan; the old one and the slice selected before go to undo.
    pub(super) fn change_plan(&mut self, plan: chop::Plan, selected: usize) {
        if let Some(old) = self.plan.replace(plan.clone())
            && old != plan
        {
            self.plan_undo.push((old, selected));
        }
    }

    /// `P`: the assign mode starts by hand from slice 1.
    pub(super) fn start_assign(&mut self) {
        if self.chop.is_none() {
            return;
        }
        // tap pads one by one, from the slice just heard (04e): it stays selected
        self.plan = Some(chop::Plan::ByHand(Vec::new()));
        self.plan_undo.clear();
    }

    /// Enter in the assign mode: the planned slices go onto their pads.
    pub(super) fn apply_plan(&mut self) {
        // in order with no pad hit: the selected pad is the start, the plan is shown and the
        // next Enter applies it, as the controller's commit does (67)
        if matches!(self.plan, Some(chop::Plan::InOrder(None))) {
            let start = self.pads.selected_abs();
            let slice = self.chop.as_ref().map_or(0, |c| c.selected);
            self.change_plan(chop::Plan::InOrder(Some(start)), slice);
            self.post_notice(
                Level::Info,
                (
                    format!("from {}: Enter applies", pad_place(start)),
                    Instant::now(),
                ),
            );
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
            self.post_notice(
                Level::Warn,
                ("nothing planned: hit a pad".into(), Instant::now()),
            );
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
            self.pads.custom_name[abs] = None;
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
            self.send(self.pads.pitch_gate_command(abs));
        }
        self.post_notice(
            Level::Done,
            (format!("slices placed on {summary}"), Instant::now()),
        );
        // the chop stays open: its cuts can still be moved and the pads follow
        self.plan = None;
        self.goto(Screen::Pads);
        self.dirty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: u32 = 48_000;

    fn chop(len: usize) -> Chop {
        Chop::new(
            "break.wav".into(),
            Arc::new(Sample::mono(vec![0.0; len])),
            SR,
        )
    }

    /// 38 "Kick and hat by color", the measuring half: a 60 Hz tone is low, a 9 kHz tone
    /// is high, a 1 kHz tone is mid.
    #[test]
    fn bands_tell_low_mid_and_high() {
        let tone = |hz: f32| -> Vec<f32> {
            (0..48_000)
                .map(|i| (i as f32 * hz / 48_000.0 * std::f32::consts::TAU).sin() * 0.5)
                .collect()
        };
        let main = |hz: f32| {
            let b = find_bands(&tone(hz), 48_000.0);
            // past the first blocks, where the filters settle
            let [l, m, h] = b[50];
            if l > m && l > h {
                0
            } else if m > h {
                1
            } else {
                2
            }
        };
        assert_eq!(main(60.0), 0, "a kick is low");
        assert_eq!(main(1_000.0), 1, "a snare body is mid");
        assert_eq!(main(9_000.0), 2, "a hat is high");
        assert_eq!(
            find_bands(&tone(60.0), 48_000.0).len(),
            48_000usize.div_ceil(BAND_BLOCK)
        );
    }

    #[test]
    fn zoom_keeps_the_cursor_at_its_place_of_the_picture() {
        let mut c = chop(48_000);
        c.set_cursor(12_000);
        let (len, at) = (
            c.view.end - c.view.start,
            (c.cursor - c.view.start) as f32 / (c.view.end - c.view.start) as f32,
        );
        c.zoom(true);
        assert_eq!(c.view.end - c.view.start, len / 2, "zoomed in ×2");
        let at2 = (c.cursor - c.view.start) as f32 / (c.view.end - c.view.start) as f32;
        assert!((at - at2).abs() < 0.01, "the cursor keeps its place");
        assert_eq!(c.cursor, 12_000, "zoom does not move the cursor");
        c.zoom(false);
        assert_eq!(
            c.view,
            Region {
                start: 0,
                end: 48_000
            },
            "back to the whole file"
        );
    }

    /// Scenario "Zoom in and out" (chopping): two zooms in show a quarter of the wave, one
    /// out shows half of it; the cursor keeps its place of the picture.
    #[test]
    fn zoom_in_and_out() {
        let mut c = chop(96_000);
        c.set_cursor(30_000);
        let place =
            |c: &Chop| (c.cursor - c.view.start) as f32 / (c.view.end - c.view.start) as f32;
        let at = place(&c);
        c.zoom(true);
        c.zoom(true);
        assert_eq!(c.view.end - c.view.start, 24_000, "a quarter");
        assert!((place(&c) - at).abs() < 0.01, "{} vs {at}", place(&c));
        c.zoom(false);
        assert_eq!(c.view.end - c.view.start, 48_000, "a half");
        assert!((place(&c) - at).abs() < 0.01, "{} vs {at}", place(&c));
        assert_eq!(c.cursor, 30_000);
    }

    #[test]
    fn zoom_stops_at_the_limits() {
        let mut c = chop(48_000);
        for _ in 0..20 {
            c.zoom(false);
        }
        assert_eq!(
            c.view.end - c.view.start,
            48_000,
            "cannot zoom past the whole file"
        );
        for _ in 0..20 {
            c.zoom(true);
        }
        assert!(
            c.view.end - c.view.start >= MIN_VIEW,
            "the view stops at the closest useful zoom"
        );
        // and the view stays inside the file
        assert!(c.view.start < c.view.end && c.view.end <= 48_000);
    }

    /// 17 "Zoom knob": the knob position spans the view between the whole file and
    /// `MIN_VIEW` on a log scale, the cursor keeping its place of the picture.
    #[test]
    fn the_zoom_knob_spans_the_file_on_a_log_scale() {
        let mut c = chop(96_000);
        c.set_cursor(48_000);
        c.set_zoom(0.0);
        assert_eq!(c.view.end - c.view.start, 96_000, "0 is the whole file");
        c.set_zoom(1.0);
        assert_eq!(c.view.end - c.view.start, MIN_VIEW, "1 is the close-up");
        let at = (c.cursor - c.view.start) as f32 / MIN_VIEW as f32;
        assert!(
            (at - 0.5).abs() < 0.02,
            "the cursor stays in the middle: {at}"
        );
        // the middle of the knob is the middle of the log scale
        c.set_zoom(0.5);
        let want = (96_000f32 * MIN_VIEW as f32).sqrt() as usize;
        assert!(
            (c.view.end - c.view.start).abs_diff(want) <= 2,
            "{} vs {want}",
            c.view.end - c.view.start
        );
        // and the knob position reads back
        assert!((c.zoom_pos() - 0.5).abs() < 0.01, "{}", c.zoom_pos());
    }

    /// 17 "Region knobs": the edges follow the knob and the region never gets shorter
    /// than 10 ms; a whole turn of a knob is one undo step.
    #[test]
    fn the_region_knobs_keep_the_region_10_ms() {
        let mut c = chop(96_000);
        c.set_region_edge(false, 0.5);
        assert_eq!(c.range.start, 48_000);
        // the end cannot cross the start: 10 ms at 48 kHz is 480 frames
        c.set_region_edge(true, 0.0);
        assert_eq!(
            c.range,
            Region {
                start: 48_000,
                end: 48_480
            }
        );
        // turning further into the floor changes nothing, and the whole turn of both
        // knobs is one undo step
        c.set_region_edge(true, 0.4);
        assert_eq!(c.range.end, 48_480, "the floor holds");
        assert!(c.undo(), "one undo for the whole turn");
        assert_eq!(
            c.range,
            Region {
                start: 0,
                end: 96_000
            }
        );
    }

    /// 17 "Cursor knob": the knob walks the cursor inside the region and the slice
    /// under the cursor becomes the selected one.
    #[test]
    fn the_cursor_knob_walks_the_region() {
        let mut c = chop(96_000);
        c.next_parts(); // 2 parts, the cut at 48_000
        c.set_cursor_pos(0.0);
        assert_eq!((c.cursor, c.selected), (0, 0));
        c.set_cursor_pos(1.0);
        assert_eq!(c.cursor, 96_000, "the cursor stays inside the region");
        assert_eq!(c.selected, 1, "the slice under the cursor is selected");
        c.set_cursor_pos(0.4);
        assert_eq!((c.cursor, c.selected), (38_400, 0));
        // and the position reads back
        assert!((c.cursor_pos() - 0.4).abs() < 0.01);
    }

    #[test]
    fn the_view_follows_the_cursor() {
        let mut c = chop(48_000);
        // zoom to a quarter, put the cursor at the view start, walk it out of the view
        c.set_cursor(0);
        c.zoom(true);
        let len = c.view.end - c.view.start;
        for _ in 0..40 {
            let tap = c.view_step(0);
            c.move_cursor(true, tap);
        }
        assert!(
            c.cursor >= c.view.start && c.cursor <= c.view.end,
            "the cursor stays inside the view"
        );
        assert!(c.view.end <= 48_000 && c.view.start + len == c.view.end);
    }

    #[test]
    fn equal_parts_cycle_and_cover_the_range() {
        let mut c = chop(48_000);
        assert_eq!(c.next_parts(), 2);
        assert_eq!(c.markers, [24_000]);
        assert_eq!(c.next_parts(), 4);
        assert_eq!(c.next_parts(), 8);
        assert_eq!(c.markers.len(), 7);
        let s = c.slices();
        assert_eq!((s[0].start, s[7].end), (0, 48_000));
        assert!(s.iter().all(|r| r.end - r.start == 6_000));
        c.next_parts();
        c.next_parts();
        assert_eq!(c.next_parts(), 2, "32 wraps back to 2");
    }

    #[test]
    fn a_free_slice_opens_as_cuts_of_the_whole_file() {
        let mut c = chop(48_000);
        c.ensure_cut(12_000);
        c.ensure_cut(24_000);
        c.ensure_cut(24_000);
        assert_eq!(c.markers, [12_000, 24_000]);
        assert_eq!(
            c.slice_at(12_000),
            1,
            "the pad's slice starts at its first cut"
        );
        assert_eq!(c.slice_at(11_999), 0);
        c.ensure_cut(0);
        assert_eq!(c.markers.len(), 2, "not on the range edges");
    }

    #[test]
    fn links_follow_cuts_and_joins() {
        let mut c = chop(48_000);
        c.next_parts();
        c.next_parts();
        // pads on slices 2 and 4
        let (two, four) = (c.start_id(1), c.start_id(3));
        assert_eq!((c.slice_of(two), c.slice_of(four)), (1, 3));
        // a new cut inside slice 1: the later slices are numbered one up
        assert_ne!(c.cut(6_000), Cut::Refused);
        assert_eq!((c.slice_of(two), c.slice_of(four)), (2, 4));
        // joining slice "two" with the one before: its pad plays the joined slice
        c.select_slice(2);
        c.delete();
        assert_eq!((c.slice_of(two), c.slice_of(four)), (1, 3));
        assert_eq!(c.slice_of(0), 0);
        // undo brings the cut back and the pad follows it again
        assert!(c.undo());
        assert_eq!(c.slice_of(two), 2);
        // a moved cut is remembered for its pads
        c.select_slice(2);
        c.nudge(false, 20);
        assert_eq!(c.slices()[c.slice_of(two)].start, 12_000 + 960);
    }

    #[test]
    fn equal_parts_start_a_new_epoch_and_undo_returns_it() {
        let mut c = chop(48_000);
        c.next_parts();
        let first = c.epoch;
        c.next_parts();
        assert_ne!(c.epoch, first);
        assert!(c.undo());
        assert_eq!(c.epoch, first);
        c.next_parts();
        assert!(
            c.epoch > first + 1,
            "a new chop never reuses an undone epoch"
        );
        // cuts are not a new epoch
        let e = c.epoch;
        c.cut(5_000);
        assert_eq!(c.epoch, e);
    }

    /// Scenario "Eight slices after a restart" (chopping).
    #[test]
    fn equal_parts_go_on_after_a_restart() {
        let mut c = chop(48_000);
        c.next_parts();
        c.next_parts();
        c.next_parts();
        let back = &mut Chop::from_saved(c.file.clone(), c.sound.clone(), SR, c.saved());
        assert_eq!(back.next_parts(), 16);
        // 6 slices are not an equal-parts count: the last count goes on
        let mut d = chop(48_000);
        d.next_parts();
        d.next_parts();
        d.cut(1_000);
        d.cut(30_000);
        assert_eq!(d.next_parts(), 8);
    }

    #[test]
    fn saved_round_trip() {
        let mut c = chop(48_000);
        c.next_parts();
        c.next_parts();
        c.select_slice(0);
        c.nudge(false, 20);
        let id = c.start_id(2);
        let back = Chop::from_saved(c.file.clone(), c.sound.clone(), SR, c.saved());
        assert_eq!(back.saved(), c.saved());
        assert_eq!(back.slice_of(id), 2);
        assert_eq!(back.slices(), c.slices());
        // through seconds, as the session keeps it
        let st = c.saved().to_state(SR);
        assert_eq!(Saved::from_state(&st, SR), c.saved());
        assert_eq!(Saved::from_state(&st, 44_100).range.end, 44_100);
    }

    #[test]
    fn cut_inserts_in_order_and_selects_the_new_slice() {
        let mut c = chop(48_000);
        assert_ne!(c.cut(30_000), Cut::Refused);
        assert_ne!(c.cut(10_000), Cut::Refused);
        assert_eq!(c.markers, [10_000, 30_000]);
        assert_eq!(c.selected, 1, "the slice that starts at 10000");
        // 5 ms from an existing cut, and the very edges
        assert_eq!(c.cut(10_240), Cut::Refused);
        assert_eq!(c.cut(0), Cut::Refused);
        assert_eq!(c.cut(48_000), Cut::Refused);
        assert_eq!(c.cut(100), Cut::Refused);
        assert_eq!(c.slices().len(), 3);
    }

    #[test]
    fn nudge_moves_the_slice_start_and_stops_at_neighbours() {
        let mut c = chop(48_000);
        c.cut(10_000);
        c.cut(11_500);
        c.select(-1);
        assert_eq!(c.selected, 1);
        c.nudge(false, -20);
        assert_eq!(c.markers[0], 10_000 - 960);
        c.nudge(false, 4);
        assert_eq!(c.markers[0], 10_000 - 960 + 192);
        // pushing into the next cut stops 1 ms short of it
        for _ in 0..10 {
            c.nudge(false, 20);
        }
        assert_eq!(c.markers[0], 11_500 - 48);
    }

    #[test]
    fn select_slice_by_number() {
        let mut c = chop(48_000);
        c.next_parts();
        c.next_parts();
        assert!(c.select_slice(2));
        c.nudge(false, 20);
        assert_eq!(c.markers[1], 24_000 + 960, "the start of slice 3 moved");
        assert!(!c.select_slice(4), "only 4 slices");
        assert_eq!(c.selected, 2);
    }

    #[test]
    fn cursor_moves_steps_and_selects_the_slice_under_it() {
        let mut c = chop(48_000 * 2);
        c.next_parts(); // 2 slices over 96_000 frames, 48_000 each
        assert_eq!(c.cursor, 0);
        // one step is 1/32 of the view (the whole file: 3000 frames)
        walk(&mut c, true);
        assert_eq!(c.cursor, 3000);
        assert_eq!(c.selected, 0, "still in the first slice");
        for _ in 0..15 {
            walk(&mut c, true);
        }
        assert_eq!(c.selected, 1, "16 steps = half the file: the second slice");
        // the cursor does not leave the range
        for _ in 0..40 {
            walk(&mut c, true);
        }
        assert_eq!(c.cursor, c.range.end);
        walk(&mut c, false);
        assert!(c.cursor < c.range.end);
    }

    #[test]
    fn region_keys_set_clear_and_undo() {
        let mut c = chop(48_000 * 2);
        c.next_parts(); // markers every 24_000
        c.set_cursor(30_000);
        c.set_region_start(30_000);
        assert_eq!(
            c.range,
            Region {
                start: 30_000,
                end: 96_000
            }
        );
        assert_eq!(c.markers, [48_000], "markers before the region are dropped");
        c.set_cursor(60_000);
        c.set_region_end(60_000);
        assert_eq!(
            c.range,
            Region {
                start: 30_000,
                end: 60_000
            }
        );
        assert_eq!(c.markers, [48_000], "markers after the region are dropped");
        // equal parts cover the region, not the file
        c.next_parts(); // next in the cycle: 2 → 4 parts
        let s = c.slices();
        assert_eq!((s[0].start, s[3].end), (30_000, 60_000));
        c.clear_region();
        assert_eq!(
            c.range,
            Region {
                start: 0,
                end: 96_000
            }
        );
        assert_eq!(c.markers.len(), 3, "the parts stay inside the file");
        assert!(c.undo(), "the last region change is undone");
        assert_eq!(
            c.range,
            Region {
                start: 30_000,
                end: 60_000
            }
        );
    }

    #[test]
    fn a_region_keeps_pads_linked_to_the_slices_inside_it() {
        let mut c = chop(48_000 * 2);
        for _ in 0..4 {
            c.next_parts(); // 16 parts, cuts every 6_000
        }
        let ids: Vec<u32> = (0..16).map(|i| c.start_id(i)).collect();
        c.set_cursor(30_000);
        c.set_region_start(30_000);
        let slices = c.slices();
        for &id in &ids {
            assert!(c.slice_of(id) < slices.len(), "a pad finds its slice");
        }
        // the cut at 36_000 (slice 7) is now the start of the second slice of the region
        assert_eq!(slices[c.slice_of(ids[6])].start, 36_000);
    }

    /// Scenario "Listen from the middle" (chopping): to the end of the region when one is set.
    #[test]
    fn play_the_region() {
        let mut c = chop(48_000 * 2);
        c.set_cursor(30_000);
        c.set_region_start(30_000);
        c.set_cursor(60_000);
        c.set_region_end(60_000);
        // the cursor sits on the end edge: play still gets one frame, not an empty region
        assert_eq!(
            c.play_region(),
            Region {
                start: 59_999,
                end: 60_000
            }
        );
        c.set_cursor(40_000);
        assert_eq!(
            c.play_region(),
            Region {
                start: 40_000,
                end: 60_000
            },
            "the sound stops at the region end, not at the end of the file"
        );
    }

    #[test]
    fn a_degenerate_region_stays_one_frame_long() {
        let mut c = chop(48_000);
        c.set_cursor(10_000);
        c.set_region_end(10_000);
        // up to the cursor, not including its frame
        assert_eq!(
            c.range,
            Region {
                start: 0,
                end: 10_000
            }
        );
        // start beyond the end collapses onto it
        c.set_cursor(c.range.end - 1);
        let end = c.range.end;
        c.set_region_start(end + 5_000);
        assert_eq!(c.range.start, end - 1, "collapsed to one frame");
        assert!(c.range.end > c.range.start);
    }

    /// A break: a hit every 12_000 frames (0.25 s), each decaying over ~8_000.
    fn break_of_hits(len: usize, every: usize) -> Chop {
        let mut wave = vec![0.0f32; len];
        for h in (0..len).step_by(every) {
            for i in 0..8_000.min(len - h) {
                wave[h + i] = (-(i as f32) / 1500.0).exp();
            }
        }
        Chop::new("break.wav".into(), Arc::new(Sample::mono(wave)), SR)
    }

    #[test]
    fn even_steps_on_a_break() {
        let mut c = break_of_hits(96_000, 12_000);
        assert_eq!(c.snap, Snap::Off, "the chop screen opens in even steps");
        for n in 1..=12 {
            walk(&mut c, true);
            assert_eq!(c.cursor, n * 3000, "a peak does not hold the cursor back");
        }
    }

    #[test]
    fn hit_to_hit() {
        let mut c = break_of_hits(96_000, 12_000);
        c.snap = Snap::Hits;
        walk(&mut c, true);
        assert_eq!(c.cursor, 12_000, "the start of the next hit");
        walk(&mut c, true);
        assert_eq!(c.cursor, 24_000);
        walk(&mut c, false);
        assert_eq!(c.cursor, 12_000, "back on the start of the previous hit");
        c.set_cursor(13_000);
        walk(&mut c, false);
        assert_eq!(c.cursor, 12_000, "from inside a hit, back to its start");
    }

    #[test]
    fn no_hits_on_the_screen() {
        // hits at 0 and 48_000 only; a close view around the cursor sees none of them
        let mut c = break_of_hits(96_000, 48_000);
        c.snap = Snap::Hits;
        c.set_cursor(10_000);
        for _ in 0..6 {
            c.zoom(true);
        }
        let len = c.view.end - c.view.start;
        assert!(c.view.end < 48_000, "the next hit is off screen");
        let mut prev = c.cursor;
        while c.cursor != 48_000 {
            walk(&mut c, true);
            assert!(
                c.cursor > prev && c.cursor <= 48_000,
                "even steps, never past the hit"
            );
            if c.cursor != 48_000 {
                assert_eq!(
                    c.cursor - prev,
                    len / 32,
                    "an even step while no hit is in view"
                );
            }
            prev = c.cursor;
        }
    }

    /// The top step of a held key: 1/32 of the view, as the even step was before 21.
    fn walk(c: &mut Chop, right: bool) {
        let top = c.view_step(RAMP_REPEATS + 1);
        c.move_cursor(right, top);
    }

    /// chopping 21 "Fine step" and "Steps follow the view": a tap is 1/128 of the view, a
    /// held key grows to 1/32; zoomed in, the same tap is a smaller step.
    #[test]
    fn fine_step() {
        let mut c = chop(96_000);
        assert_eq!(c.view_step(0), 96_000 / 128, "a tap");
        assert_eq!(
            c.view_step(1),
            96_000 / 128,
            "the first repeat is a tap's step"
        );
        assert_eq!(c.view_step(2), 96_000 / 128 * 3 / 2, "then it grows");
        assert_eq!(c.view_step(RAMP_REPEATS + 1), 96_000 / 32, "the top");
        assert_eq!(c.view_step(100), 96_000 / 32, "and it stays there");
        for _ in 0..4 {
            c.zoom(true);
        }
        let view = c.view.end - c.view.start;
        assert_eq!(
            c.view_step(0),
            view / 128,
            "zoomed in: the same share, a smaller step"
        );
    }

    #[test]
    fn select_moves_the_cursor_to_the_slice_start() {
        let mut c = chop(48_000 * 2);
        c.next_parts();
        c.set_cursor(60_000);
        assert_eq!(c.selected, 1);
        c.select(-1);
        assert_eq!(c.cursor, 0, "the start of slice 1");
    }

    #[test]
    fn cut_on_pause() {
        let mut c = chop(48_000 * 2);
        c.set_cursor(30_000);
        assert_ne!(
            c.cut(c.cursor),
            Cut::Refused,
            "a cut lands under the cursor without playing"
        );
        assert_eq!(c.markers, vec![30_000]);
        assert_eq!(c.selected, 1, "the new slice is selected");
    }

    /// Scenario "Trim the sample start" (chopping): only the range moves, the file is not touched.
    #[test]
    fn first_slice_trims_the_sample_start() {
        let mut c = chop(48_000);
        c.next_parts();
        c.select(-1);
        assert_eq!(c.selected, 0);
        c.nudge(false, 20);
        assert_eq!(c.range.start, 960);
        assert_eq!(c.slices()[0].start, 960);
        c.nudge(false, -20);
        c.nudge(false, -20);
        assert_eq!(c.range.start, 0, "not before the file start");
        assert!(c.undo() && c.range.start == 960);
        // deleting on the first slice does nothing
        c.delete();
        assert_eq!(c.markers, [24_000]);
    }

    /// Scenario "Undo a delete" (chopping).
    #[test]
    fn delete_joins_with_the_previous_slice_and_undo() {
        let mut c = chop(48_000);
        c.next_parts();
        c.next_parts();
        let four = c.markers.clone();
        c.select(1);
        c.select(1);
        c.delete();
        assert_eq!(c.markers, [12_000, 36_000]);
        assert_eq!(c.selected, 1);
        assert!(c.undo());
        assert_eq!(c.markers, four);
        assert!(c.undo(), "back to 2 parts");
        assert_eq!(c.markers, [24_000]);
        assert!(c.undo() && c.markers.is_empty());
        assert!(!c.undo());
    }

    /// Scenario "Turn the start" (chopping).
    #[test]
    fn knobs_move_the_edges_of_the_selected_slice() {
        let mut c = chop(48_000);
        c.next_parts();
        c.next_parts();
        c.select(1);
        c.select(1);
        // slice 3 is 24000..36000; its start may go from 12048 to 35952
        let (low, high) = (12_048, 35_952);
        assert!((c.edge_pos(false) - (24_000 - low) as f32 / (high - low) as f32).abs() < 1e-6);
        c.set_edge(false, 0.0);
        assert_eq!(c.markers[1], low, "the end of slice 2 moved with it");
        c.set_edge(false, 0.5);
        c.set_edge(false, 1.0);
        assert_eq!(c.markers[1], high);
        // the end of the last slice trims the sample end
        c.select(1);
        c.set_edge(true, 0.0);
        assert_eq!(c.range.end, 36_000 + 48);
        // one turn of a knob is one undo step
        assert!(c.undo() && c.range.end == 48_000);
        assert!(c.undo() && c.markers[1] == 24_000);
    }

    #[test]
    fn plan_in_order_across_banks_and_out_of_pads() {
        let pads = |p: &Plan, n| p.targets(n).iter().map(|t| t.1).collect::<Vec<_>>();
        // 9 slices from pad 1 of bank 1
        assert_eq!(pads(&Plan::InOrder(Some(0)), 9), (0..9).collect::<Vec<_>>());
        // from pad 13 of bank 1: 13-16 and 1-5 of bank 2
        assert_eq!(
            pads(&Plan::InOrder(Some(12)), 9),
            (12..21).collect::<Vec<_>>()
        );
        // pad 14 of bank 8: three fit
        assert_eq!(Plan::InOrder(Some(PADS - 3)).targets(9).len(), 3);
        assert!(
            Plan::InOrder(None).targets(9).is_empty(),
            "no start pad yet"
        );
        let mut p = Plan::InOrder(None);
        p.hit(36, 4);
        assert_eq!(p, Plan::InOrder(Some(36)));
    }

    #[test]
    fn plan_by_hand() {
        let mut p = Plan::ByHand(Vec::new());
        assert_eq!(p.next_unplanned(4, 0), Some(1));
        p.hit(4, 0);
        p.hit(0, 1);
        assert_eq!(p.next_unplanned(4, 1), Some(2));
        p.hit(5, 3);
        // after the last slice the first one left comes next
        assert_eq!(p.next_unplanned(4, 3), Some(2));
        assert!(!p.all_planned(4));
        p.hit(6, 2);
        assert!(p.all_planned(4));
        assert_eq!(p.next_unplanned(4, 2), None);
        // the same pad again: the new slice replaces the planned one
        p.hit(4, 3);
        assert_eq!(p.targets(4), [(1, 0), (3, 4), (3, 5), (2, 6)]);
        assert!(!p.all_planned(4), "slice 1 lost its pad");
        assert_eq!(
            p.targets(2),
            [(1, 0)],
            "slices that no longer exist drop out"
        );
    }

    #[test]
    fn overwrites_by_bank() {
        let t = Plan::InOrder(Some(14)).targets(4);
        let taken = |p: usize| match p {
            15 => Some("R kick".to_string()),
            16 => Some("1 amen 3".to_string()),
            17 => Some("2 snare".to_string()),
            _ => None,
        };
        assert_eq!(
            overwrites(&t, taken),
            "bank 1: R kick; bank 2: 1 amen 3, 2 snare"
        );
        assert_eq!(overwrites(&t, |_| None), "");
    }

    #[test]
    fn summary_texts() {
        let none = |_| false;
        let t = Plan::InOrder(Some(0)).targets(9);
        assert_eq!(summary(&t, 9, none).unwrap(), "bank 1");
        let t = Plan::InOrder(Some(12)).targets(9);
        assert_eq!(
            summary(&t, 9, |p| p == 12 || p == 13).unwrap(),
            "banks 1–2 · replaces 2"
        );
        let t = Plan::InOrder(Some(PADS - 3)).targets(9);
        assert_eq!(summary(&t, 9, none).unwrap(), "bank 8 · 6 left out");
        assert_eq!(summary(&[], 4, none), None);
    }

    /// 21: "To the cut" and "The slice": play covers what the mode says, from the cursor.
    /// Scenario "Listen from the middle" (chopping): to the end of the sample without a region.
    #[test]
    fn play_modes_cover_their_span() {
        let mut c = chop(48_000 * 2);
        c.next_parts();
        c.next_parts(); // 4 slices of 24_000
        c.set_cursor(60_000); // the middle of slice 3
        assert_eq!(
            c.play_region(),
            Region {
                start: 60_000,
                end: 96_000
            },
            "to the end"
        );
        c.play = c.play.next();
        assert_eq!(
            c.play_region(),
            Region {
                start: 60_000,
                end: 72_000
            },
            "to the cut"
        );
        c.play = c.play.next();
        assert_eq!(
            c.play_region(),
            Region {
                start: 48_000,
                end: 72_000
            },
            "the slice"
        );
        c.play = c.play.next();
        assert_eq!(c.play, PlayMode::ToEnd, "round and round");
    }

    /// chopping 21 "Cut to cut" and "To the ends".
    #[test]
    fn cut_to_cut() {
        let mut c = chop(96_000);
        c.next_parts();
        c.next_parts(); // cuts at 24_000, 48_000, 72_000
        c.set_cursor(30_000); // inside slice 2
        c.jump_cursor(true);
        assert_eq!((c.cursor, c.selected), (48_000, 2), "the start of slice 3");
        c.jump_cursor(true);
        assert_eq!(c.cursor, 72_000, "the start of slice 4");
        c.jump_cursor(false);
        assert_eq!(c.cursor, 48_000, "back on slice 3");
        c.jump_cursor(true);
        c.jump_cursor(true);
        assert_eq!(c.cursor, 96_000, "past the last cut: the end");
        c.set_cursor(10_000);
        c.jump_cursor(false);
        assert_eq!(c.cursor, 0, "before the first cut: the start");
    }
}

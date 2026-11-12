//! Bounded manual-input capture and accepted recording state.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering::SeqCst};

use crate::PadMode;
use crate::pattern::{Grid, Hit, MAX_GATE_TICKS, MAX_HITS, Pattern, RowId};

pub const CAPACITY: usize = 512;

/// One accepted UI press shares its original physical release, not a later bank.
#[derive(Debug)]
pub struct EraseHold {
    released: Arc<AtomicBool>,
    disposition: AtomicU8,
}

impl EraseHold {
    pub fn new(released: Arc<AtomicBool>) -> Self {
        Self {
            released,
            disposition: AtomicU8::new(0),
        }
    }

    pub(crate) fn released(&self) -> bool {
        self.released.load(SeqCst)
    }

    pub(crate) fn accept(&self, erasing: bool) {
        self.disposition.store(if erasing { 2 } else { 1 }, SeqCst);
    }

    pub(crate) fn reject(&self) {
        self.disposition.store(3, SeqCst);
    }

    pub(crate) fn ordinary(&self) -> bool {
        self.disposition.load(SeqCst) == 1
    }

    /// A pending erase release must not consume a later press's sound-release obligation.
    pub fn may_release_sound(&self) -> bool {
        self.ordinary()
    }
}

impl PartialEq for EraseHold {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for EraseHold {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Idle,
    CountIn,
    Recording,
    Finishing,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Onset {
        pad: usize,
        velocity: u8,
        mode: PadMode,
        tick: u32,
    },
    Release {
        pad: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    pub epoch: u64,
    pub pass: u64,
    /// Orders capture and erasure work, including omitted feedback. Use loss counters for omissions.
    pub sequence: u64,
    pub run: u64,
    pub frame_ticks: f64,
    pub ticks: f64,
    pub kind: Kind,
    /// Only an accepted Gate onset can address retained completion data.
    pub gate: Option<usize>,
}

impl Event {
    /// Preserve the original lap even when rounding moves the saved tick across zero.
    pub fn origin(self, loop_ticks: u32) -> Option<crate::pattern::Origin> {
        if self.run == 0
            || loop_ticks == 0
            || !self.ticks.is_finite()
            || self.ticks < 0.0
            || !matches!(self.kind, Kind::Onset { .. })
        {
            return None;
        }
        Some(crate::pattern::Origin {
            run: self.run,
            // Match the sequencer's nearest-frame seam, not the saved tick rounding.
            lap: ((self.ticks + self.frame_ticks * 0.5 + 1e-9) / f64::from(loop_ticks)).floor()
                as u64,
            ticks: self.ticks.to_bits(),
            pass: self.pass,
            sequence: self.sequence,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub epoch: u64,
    pub pass: u64,
    pub phase: Phase,
    /// Accepted position, frozen at the end while Finishing.
    pub ticks: f64,
    pub loop_ticks: u32,
    /// Remaining beat number, 4 through 1. Zero outside count-in.
    pub count_in_beat: u8,
    pub sequence: u64,
    pub dropped_onsets: u64,
    pub dropped_events: u64,
    pub reserved_rows: u32,
    pub captured_onsets: u32,
    pub limit_refusals: u64,
    pub erasing: bool,
    pub erase_targets: [u64; 2],
    pub erase_intervals: u64,
    pub erased_rows: u64,
    pub frame_ticks: f64,
    pub erase_refusals: u64,
    pub erasure_sequence: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EraseInterval {
    pub epoch: u64,
    pub pass: u64,
    pub run: u64,
    pub closed_sequence: u64,
    pub sequence: u64,
    pub index: u64,
    pub slot: usize,
    pub pad: usize,
    pub start: f64,
    pub end: f64,
    pub complete: bool,
    pub loop_ticks: u32,
    pub grid: Grid,
    pub strength: u8,
}

/// Setup allocates storage once. Recovery needs an original accepted onset, not just a pad.
pub struct Recovery {
    owner: Published,
    gates: Box<[Gate]>,
    intervals: Box<[EraseSlot]>,
    removed: Box<[RemovedSlot]>,
}

impl Recovery {
    fn new() -> Self {
        Self {
            owner: Published::default(),
            gates: (0..MAX_HITS).map(|_| Gate::default()).collect(),
            intervals: (0..MAX_HITS).map(|_| EraseSlot::default()).collect(),
            removed: (0..MAX_HITS).map(|_| RemovedSlot::default()).collect(),
        }
    }

    /// Read only removals accepted within this coherent project and pass boundary.
    pub fn erased(&self, id: RowId, boundary: Snapshot) -> Option<bool> {
        let owner = self.owner.read(boundary.epoch)?;
        if owner.pass != boundary.pass
            || boundary.pass == 0
            || !matches!(boundary.phase, Phase::Recording | Phase::Finishing)
        {
            return None;
        }
        let removed = self.removed.get(id.slot)?.read()?;
        let result = removed.0 == boundary.epoch
            && removed.1 == boundary.pass
            && removed.2 == id.generation
            && removed.3 != 0
            && removed.3 <= boundary.sequence;
        (self.owner.read(boundary.epoch)? == owner).then_some(result)
    }

    /// Clamp progress to accepted audio time. No read waits for a concurrent writer.
    pub fn erase_interval(&self, slot: usize, boundary: Snapshot) -> Option<EraseInterval> {
        let owner = self.owner.read(boundary.epoch)?;
        if owner.pass != boundary.pass
            || boundary.pass == 0
            || !matches!(boundary.phase, Phase::Recording | Phase::Finishing)
            || !boundary.ticks.is_finite()
            || !boundary.frame_ticks.is_finite()
            || boundary.frame_ticks < 0.0
        {
            return None;
        }
        let mut interval = self.intervals.get(slot)?.read(slot)?;
        if interval.epoch != boundary.epoch
            || interval.pass != boundary.pass
            || interval.sequence == 0
            || interval.sequence > boundary.sequence
            || interval.index > boundary.erase_intervals
            || interval.start > boundary.ticks + 1e-9
        {
            return None;
        }
        interval.complete = interval.closed_sequence != 0
            && interval.closed_sequence <= boundary.sequence
            && interval.end <= boundary.ticks + boundary.frame_ticks * 0.5 + 1e-9;
        if !interval.complete {
            interval.end = interval.end.min(boundary.ticks).max(interval.start);
        }
        (self.owner.read(boundary.epoch)? == owner).then_some(interval)
    }

    /// One bounded attempt. Pending, obsolete or concurrently changing data returns no row.
    pub fn completed(&self, onset: Event) -> Option<Hit> {
        let owner = self.owner.read(onset.epoch)?;
        if owner.pass != onset.pass || onset.pass == 0 {
            return None;
        }
        let Kind::Onset {
            pad,
            velocity,
            mode,
            tick,
        } = onset.kind
        else {
            return None;
        };
        let duration = match mode {
            PadMode::OneShot if onset.gate.is_none() => 0,
            PadMode::Gate => self.gates.get(onset.gate?)?.read(onset)?,
            _ => return None,
        };
        let row = Hit::new(tick, pad, velocity, duration).ok()?;
        (self.owner.read(onset.epoch)? == owner).then_some(row)
    }

    /// Recover a closed capture prefix after audio loss or during final App shutdown.
    /// Supply its last coherent snapshot, not an interface timer. Never resume that prefix.
    /// This returns a raw row, not an audio acknowledgement or a phase change.
    pub fn interrupted(&self, onset: Event, boundary: Snapshot) -> Option<Hit> {
        let owner = self.owner.read(onset.epoch)?;
        if onset.pass == 0
            || owner.pass != onset.pass
            || boundary.epoch != onset.epoch
            || boundary.pass != onset.pass
            || !matches!(boundary.phase, Phase::Recording | Phase::Finishing)
            || boundary.count_in_beat != 0
            || boundary.loop_ticks == 0
            || boundary.loop_ticks != owner.loop_ticks
            || boundary.captured_onsets == 0
            || onset.sequence == 0
            || onset.sequence > boundary.sequence
            || !boundary.ticks.is_finite()
            || !onset.ticks.is_finite()
            || onset.ticks < 0.0
            || onset.ticks > boundary.ticks
        {
            return None;
        }
        let Kind::Onset {
            pad,
            velocity,
            mode,
            tick,
        } = onset.kind
        else {
            return None;
        };
        if tick >= boundary.loop_ticks {
            return None;
        }
        let duration = match mode {
            PadMode::OneShot if onset.gate.is_none() => 0,
            PadMode::Gate => {
                let slot = onset.gate?;
                if slot >= boundary.captured_onsets as usize {
                    return None;
                }
                let known = self.gates.get(slot)?.duration(onset)?;
                let limit = ((boundary.ticks - onset.ticks + 1e-9).round())
                    .clamp(1.0, f64::from(MAX_GATE_TICKS)) as u32;
                if known == 0 { limit } else { known.min(limit) }
            }
            _ => return None,
        };
        let row = Hit::new(tick, pad, velocity, duration).ok()?;
        (self.owner.read(onset.epoch)? == owner).then_some(row)
    }
}

#[derive(Default)]
struct Gate {
    revision: AtomicU64,
    epoch: AtomicU64,
    pass: AtomicU64,
    sequence: AtomicU64,
    duration: AtomicU64,
}

impl Gate {
    fn record(&self, onset: Event, duration: u32) {
        self.revision.fetch_add(1, SeqCst);
        self.epoch.store(onset.epoch, SeqCst);
        self.pass.store(onset.pass, SeqCst);
        self.sequence.store(onset.sequence, SeqCst);
        self.duration.store(u64::from(duration), SeqCst);
        self.revision.fetch_add(1, SeqCst);
    }

    fn read(&self, onset: Event) -> Option<u32> {
        self.duration(onset).filter(|&duration| duration > 0)
    }

    /// Zero identifies a stable pending Gate, not a stale or changing read.
    fn duration(&self, onset: Event) -> Option<u32> {
        let revision = self.revision.load(SeqCst);
        if revision & 1 != 0
            || self.epoch.load(SeqCst) != onset.epoch
            || self.pass.load(SeqCst) != onset.pass
            || self.sequence.load(SeqCst) != onset.sequence
        {
            return None;
        }
        let duration = self.duration.load(SeqCst) as u32;
        (self.revision.load(SeqCst) == revision).then_some(duration)
    }
}

#[derive(Default)]
struct RemovedSlot {
    revision: AtomicU64,
    epoch: AtomicU64,
    pass: AtomicU64,
    generation: AtomicU64,
    sequence: AtomicU64,
}

impl RemovedSlot {
    fn record(&self, state: Snapshot, id: RowId) {
        self.revision.fetch_add(1, SeqCst);
        self.epoch.store(state.epoch, SeqCst);
        self.pass.store(state.pass, SeqCst);
        self.generation.store(id.generation, SeqCst);
        self.sequence.store(state.sequence, SeqCst);
        self.revision.fetch_add(1, SeqCst);
    }

    fn read(&self) -> Option<(u64, u64, u64, u64)> {
        let revision = self.revision.load(SeqCst);
        if revision & 1 != 0 {
            return None;
        }
        let value = (
            self.epoch.load(SeqCst),
            self.pass.load(SeqCst),
            self.generation.load(SeqCst),
            self.sequence.load(SeqCst),
        );
        (self.revision.load(SeqCst) == revision).then_some(value)
    }
}

#[derive(Default)]
struct EraseSlot {
    revision: AtomicU64,
    epoch: AtomicU64,
    pass: AtomicU64,
    run: AtomicU64,
    closed_sequence: AtomicU64,
    sequence: AtomicU64,
    index: AtomicU64,
    pad: AtomicU64,
    start: AtomicU64,
    end: AtomicU64,
    loop_ticks: AtomicU64,
    grid: AtomicU64,
    strength: AtomicU64,
}

impl EraseSlot {
    fn record(&self, interval: EraseInterval) {
        self.revision.fetch_add(1, SeqCst);
        self.epoch.store(interval.epoch, SeqCst);
        self.pass.store(interval.pass, SeqCst);
        self.run.store(interval.run, SeqCst);
        self.closed_sequence.store(interval.closed_sequence, SeqCst);
        self.sequence.store(interval.sequence, SeqCst);
        self.index.store(interval.index, SeqCst);
        self.pad.store(interval.pad as u64, SeqCst);
        self.start.store(interval.start.to_bits(), SeqCst);
        self.end.store(interval.end.to_bits(), SeqCst);
        self.loop_ticks
            .store(u64::from(interval.loop_ticks), SeqCst);
        self.grid.store(interval.grid as u64, SeqCst);
        self.strength.store(u64::from(interval.strength), SeqCst);
        self.revision.fetch_add(1, SeqCst);
    }

    fn close(&self, ticks: f64, sequence: u64) {
        self.revision.fetch_add(1, SeqCst);
        let end = f64::from_bits(self.end.load(SeqCst)).min(ticks);
        self.end.store(end.to_bits(), SeqCst);
        self.closed_sequence.store(sequence, SeqCst);
        self.revision.fetch_add(1, SeqCst);
    }

    fn read(&self, slot: usize) -> Option<EraseInterval> {
        let revision = self.revision.load(SeqCst);
        if revision & 1 != 0 {
            return None;
        }
        let grid = match self.grid.load(SeqCst) {
            0 => Grid::Off,
            1 => Grid::Quarter,
            2 => Grid::Eighth,
            3 => Grid::Sixteenth,
            4 => Grid::ThirtySecond,
            5 => Grid::SixtyFourth,
            6 => Grid::EighthTriplet,
            7 => Grid::SixteenthTriplet,
            8 => Grid::ThirtySecondTriplet,
            _ => return None,
        };
        let interval = EraseInterval {
            epoch: self.epoch.load(SeqCst),
            pass: self.pass.load(SeqCst),
            run: self.run.load(SeqCst),
            closed_sequence: self.closed_sequence.load(SeqCst),
            sequence: self.sequence.load(SeqCst),
            index: self.index.load(SeqCst),
            slot,
            pad: self.pad.load(SeqCst) as usize,
            start: f64::from_bits(self.start.load(SeqCst)),
            end: f64::from_bits(self.end.load(SeqCst)),
            complete: false,
            loop_ticks: self.loop_ticks.load(SeqCst) as u32,
            grid,
            strength: self.strength.load(SeqCst) as u8,
        };
        (self.revision.load(SeqCst) == revision).then_some(interval)
    }
}

struct Slots {
    owned: Box<[u64]>,
    free: Box<[usize]>,
    len: usize,
}

impl Slots {
    fn new() -> Self {
        Self {
            owned: vec![0; MAX_HITS].into_boxed_slice(),
            free: (0..MAX_HITS).rev().collect(),
            len: MAX_HITS,
        }
    }

    fn reset(&mut self) {
        self.owned.fill(0);
        for (i, slot) in self.free.iter_mut().enumerate() {
            *slot = MAX_HITS - i - 1;
        }
        self.len = MAX_HITS;
    }

    fn take(&mut self, sequence: u64) -> Option<usize> {
        if self.len == 0 || sequence == 0 {
            return None;
        }
        self.len -= 1;
        let slot = self.free[self.len];
        self.owned[slot] = sequence;
        Some(slot)
    }

    fn release(&mut self, slot: usize, sequence: u64) -> bool {
        if sequence == 0 || self.owned.get(slot) != Some(&sequence) || self.len >= self.free.len() {
            return false;
        }
        self.owned[slot] = 0;
        self.free[self.len] = slot;
        self.len += 1;
        true
    }
}

#[derive(Clone, Copy, Default)]
struct Window {
    next: Option<usize>,
    sequence: u64,
    start: f64,
    end: f64,
    closed: bool,
    closed_sequence: u64,
    expired: bool,
    grid: Grid,
    strength: u8,
}

impl Window {
    fn contains(self, tick: f64, birth: Option<f64>) -> bool {
        tick + 1e-9 >= self.start
            && tick + 1e-9 < self.end
            && birth.is_none_or(|birth| tick + 1e-9 >= birth)
    }
}

struct Windows {
    slots: Slots,
    nodes: Box<[Window]>,
    heads: [Option<usize>; crate::PADS],
    earliest: f64,
    dirty: bool,
}

impl Windows {
    fn new() -> Self {
        Self {
            slots: Slots::new(),
            nodes: vec![Window::default(); MAX_HITS].into_boxed_slice(),
            heads: [None; crate::PADS],
            earliest: f64::INFINITY,
            dirty: false,
        }
    }

    fn reset(&mut self) {
        self.slots.reset();
        self.heads.fill(None);
        self.earliest = f64::INFINITY;
        self.dirty = false;
    }

    fn start(
        &mut self,
        pad: usize,
        start: f64,
        end: f64,
        sequence: u64,
        meta: (Grid, u8),
    ) -> Option<usize> {
        let slot = self.slots.take(sequence)?;
        self.nodes[slot] = Window {
            next: self.heads[pad],
            sequence,
            start,
            end,
            closed: false,
            closed_sequence: 0,
            expired: false,
            grid: meta.0,
            strength: meta.1,
        };
        self.heads[pad] = Some(slot);
        Some(slot)
    }

    fn close(&mut self, pad: usize, slot: usize, ticks: f64, sequence: u64) {
        let node = &mut self.nodes[slot];
        node.expired = node.end.is_finite() && ticks >= node.end;
        node.end = node.end.min(ticks);
        node.closed = true;
        node.closed_sequence = sequence;
        if node.end <= node.start {
            self.heads[pad] = node.next;
            self.slots.release(slot, node.sequence);
        } else {
            self.earliest = self.earliest.min(node.end);
            self.dirty = true;
        }
    }

    fn contains(
        &self,
        pad: usize,
        tick: f64,
        row: crate::pattern::PlaybackRow,
        owner: (u64, u64),
        meta: (Grid, u8),
    ) -> bool {
        let mut next = self.heads[pad];
        while let Some(slot) = next {
            let node = self.nodes[slot];
            if (node.grid, node.strength) == meta
                && (!node.closed || !row.born_after(owner.1, node.closed_sequence))
                && node.contains(tick, row.birth_in(owner.0))
            {
                return true;
            }
            next = node.next;
        }
        false
    }

    fn traversed(
        &self,
        pad: usize,
        tick: u32,
        loop_ticks: u32,
        row: crate::pattern::PlaybackRow,
        owner: (u64, u64, f64, f64),
        meta: (Grid, u8),
    ) -> bool {
        let birth = row.birth_in(owner.0);
        let mut next = self.heads[pad];
        while let Some(slot) = next {
            let node = self.nodes[slot];
            let start = node.start.max(birth.unwrap_or(0.0));
            let lap = ((start - f64::from(tick) - 1e-9) / f64::from(loop_ticks))
                .ceil()
                .max(0.0);
            let occurrence = f64::from(tick) + lap * f64::from(loop_ticks);
            let limit = owner.2
                + if node.expired {
                    owner.3 * 0.5 + 1e-9
                } else {
                    0.0
                };
            if (node.grid, node.strength) == meta
                && occurrence + 1e-9 < limit
                && (!node.closed || !row.born_after(owner.1, node.closed_sequence))
                && node.contains(occurrence, birth)
            {
                return true;
            }
            next = node.next;
        }
        false
    }

    fn prune(&mut self, due: f64) {
        if due < self.earliest {
            return;
        }
        self.earliest = f64::INFINITY;
        for pad in 0..crate::PADS {
            let (mut previous, mut next): (Option<usize>, _) = (None, self.heads[pad]);
            while let Some(slot) = next {
                let node = self.nodes[slot];
                if node.closed && node.end <= due {
                    if let Some(previous) = previous {
                        self.nodes[previous].next = node.next;
                    } else {
                        self.heads[pad] = node.next;
                    }
                    self.slots.release(slot, node.sequence);
                } else {
                    if node.closed {
                        self.earliest = self.earliest.min(node.end);
                    }
                    previous = Some(slot);
                }
                next = node.next;
            }
        }
    }
}

#[derive(Clone, Copy)]
struct HeldErase {
    owner: u64,
    slot: usize,
    window: usize,
    sequence: u64,
    start: f64,
    until: f64,
}

pub(crate) struct Recorder {
    queue: Option<rtrb::Producer<Event>>,
    recovery: Option<Arc<Recovery>>,
    pending: [Option<Event>; crate::PADS],
    ordinary: [bool; crate::PADS],
    targets: [Option<HeldErase>; crate::PADS],
    gate_slots: Slots,
    erase_slots: Slots,
    windows: Windows,
    removed: Box<[u64]>,
    present: Box<[u64]>,
    unconfirmed: usize,
    reservation_sequence: u64,
    grid: Grid,
    strength: u8,
    step: f64,
    state: Snapshot,
    run: u64,
}

impl Default for Recorder {
    fn default() -> Self {
        Self {
            queue: None,
            recovery: None,
            pending: [None; crate::PADS],
            ordinary: [false; crate::PADS],
            targets: [None; crate::PADS],
            gate_slots: Slots::new(),
            erase_slots: Slots::new(),
            windows: Windows::new(),
            removed: vec![0; MAX_HITS].into_boxed_slice(),
            present: vec![0; MAX_HITS].into_boxed_slice(),
            unconfirmed: 0,
            reservation_sequence: 0,
            grid: Grid::Off,
            strength: 0,
            step: 0.0,
            state: Snapshot::default(),
            run: 0,
        }
    }
}

impl Recorder {
    /// Setup only. The caller retains the consumer outside the audio callback.
    pub fn enable(&mut self) -> rtrb::Consumer<Event> {
        let (tx, rx) = rtrb::RingBuffer::new(CAPACITY);
        self.queue = Some(tx);
        self.recovery = Some(Arc::new(Recovery::new()));
        self.reset(self.state.epoch);
        rx
    }

    pub fn recovery(&self) -> Option<Arc<Recovery>> {
        self.recovery.clone()
    }

    pub fn reset(&mut self, epoch: u64) {
        self.pending.fill(None);
        self.ordinary.fill(false);
        self.targets.fill(None);
        self.gate_slots.reset();
        self.erase_slots.reset();
        self.windows.reset();
        self.removed.fill(0);
        self.unconfirmed = 0;
        self.reservation_sequence = 0;
        self.state = Snapshot {
            epoch,
            ..Snapshot::default()
        };
        if let Some(recovery) = &self.recovery {
            recovery.owner.record(self.state);
        }
    }

    pub fn set_run(&mut self, run: u64) {
        if run != self.run {
            self.windows.reset();
        }
        self.run = run;
    }

    pub fn set_step(&mut self, step: f64) {
        self.step = step;
    }

    pub fn accepts_publication(&self, stamp: Option<(u64, u64)>) -> bool {
        match stamp {
            Some((pass, sequence)) => {
                pass != 0
                    && pass == self.state.pass
                    && sequence >= self.state.erasure_sequence
                    && sequence <= self.state.sequence
            }
            None => self.state.phase == Phase::Idle || self.state.erasure_sequence == 0,
        }
    }

    pub fn snapshot(&self, ticks: f64) -> Snapshot {
        let mut state = self.state;
        if state.phase == Phase::Recording {
            state.frame_ticks = self.step;
            state.ticks = ticks;
            for (pad, target) in self.targets.iter().enumerate() {
                if target.is_some_and(|target| ticks + self.step * 0.5 + 1e-9 < target.until) {
                    state.erase_targets[pad / 64] |= 1 << (pad % 64);
                }
            }
        } else if state.phase != Phase::Finishing {
            state.frame_ticks = 0.0;
        }
        state
    }

    pub fn counting(&self) -> bool {
        self.state.phase == Phase::CountIn
    }

    pub fn begin_recording(&mut self) {
        if self.counting() {
            self.state.phase = Phase::Recording;
            self.state.ticks = 0.0;
            self.state.count_in_beat = 0;
        }
    }

    pub fn cancel_count_in(&mut self) {
        if self.counting() {
            self.state.phase = Phase::Idle;
            self.state.ticks = 0.0;
            self.state.count_in_beat = 0;
        }
    }

    pub fn start(
        &mut self,
        pass: u64,
        ticks: f64,
        loop_ticks: u32,
        playing: bool,
        raw_rows: usize,
    ) -> bool {
        if self.queue.is_none()
            || loop_ticks == 0
            || raw_rows > MAX_HITS
            || self.state.phase != Phase::Idle
            || pass <= self.state.pass
        {
            return false;
        }
        self.gate_slots.reset();
        self.erase_slots.reset();
        self.targets.fill(None);
        self.windows.reset();
        self.removed.fill(0);
        self.unconfirmed = 0;
        self.reservation_sequence = 0;
        self.state = Snapshot {
            epoch: self.state.epoch,
            pass,
            phase: if playing {
                Phase::Recording
            } else {
                Phase::CountIn
            },
            ticks: if playing { ticks } else { 0.0 },
            loop_ticks,
            count_in_beat: if playing { 0 } else { 4 },
            reserved_rows: raw_rows as u32,
            ..Snapshot::default()
        };
        if let Some(recovery) = &self.recovery {
            recovery.owner.record(self.state);
        }
        true
    }

    pub fn finish(&mut self, pass: u64, ticks: f64) {
        if pass != self.state.pass {
            return;
        }
        if self.counting() {
            self.cancel_count_in();
        } else if self.state.phase == Phase::Recording {
            for pad in 0..crate::PADS {
                self.close_target(pad, ticks);
                self.close(pad, ticks);
            }
            self.state.erasing = false;
            self.state.phase = Phase::Finishing;
            self.state.ticks = ticks;
            self.state.frame_ticks = self.step;
        }
    }

    pub fn stop(&mut self, ticks: f64) {
        self.finish(self.state.pass, ticks);
    }

    pub fn acknowledge(&mut self, pass: u64, sequence: u64) {
        if self.state.phase == Phase::Finishing
            && pass == self.state.pass
            && sequence == self.state.sequence
            && self.unconfirmed == 0
            && self.erase_slots.len == MAX_HITS
            && self
                .queue
                .as_ref()
                .is_some_and(|q| q.slots() == q.buffer().capacity())
        {
            self.state.phase = Phase::Idle;
            self.windows.reset();
        }
    }

    pub fn onset(&mut self, pad: usize, gain: f32, mode: PadMode, ticks: f64, frame_ticks: f64) {
        if pad >= crate::PADS {
            return;
        }
        self.ordinary[pad] = true;
        if self.state.phase != Phase::Recording || !gain.is_finite() || gain <= 0.0 {
            return;
        }
        self.close(pad, ticks);
        // The clock accumulates floating-point steps. Preserve an exact musical midpoint.
        let tick = ((ticks + 1e-9).round() % f64::from(self.state.loop_ticks)) as u32;
        // the hit keeps the quantize of its capture: the grid of the last published pattern
        let tick = self.grid.place(self.strength, tick, self.state.loop_ticks);
        let kind = Kind::Onset {
            pad,
            velocity: (gain * 127.0).round().clamp(1.0, 127.0) as u8,
            mode,
            tick,
        };
        let Some(mut event) = self.event(ticks, kind) else {
            return;
        };
        event.frame_ticks = frame_ticks;
        if self.state.reserved_rows as usize >= MAX_HITS {
            self.state.limit_refusals = self.state.limit_refusals.saturating_add(1);
            self.omit(kind);
            return;
        }
        if self.state.captured_onsets == u32::MAX {
            self.omit(kind);
            return;
        }
        if mode == PadMode::Gate {
            let Some(slot) = self.gate_slots.take(event.sequence) else {
                self.omit(kind);
                return;
            };
            event.gate = Some(slot);
            if let Some(recovery) = &self.recovery {
                recovery.gates[slot].record(event, 0);
            }
        }
        if self.push(event) {
            self.state.reserved_rows += 1;
            self.state.captured_onsets += 1;
            if mode == PadMode::Gate {
                self.pending[pad] = Some(event);
            }
        } else if let Some(slot) = event.gate {
            self.gate_slots.release(slot, event.sequence);
        }
    }

    pub fn release(&mut self, pad: usize, ticks: f64) {
        if pad >= crate::PADS {
            return;
        }
        self.ordinary[pad] = false;
        self.close_target(pad, ticks);
        if self.state.phase == Phase::Recording {
            self.close(pad, ticks);
            if let Some(event) = self.event(ticks, Kind::Release { pad }) {
                self.push(event);
            }
        }
    }

    pub fn erasing(&self) -> bool {
        self.state.phase == Phase::Recording && self.state.erasing
    }

    pub fn set_erasing(&mut self, pass: u64, enabled: bool, ticks: f64) {
        if self.state.phase != Phase::Recording || pass != self.state.pass {
            return;
        }
        if !enabled {
            for pad in 0..crate::PADS {
                self.close_target(pad, ticks);
            }
        }
        self.state.erasing = enabled;
    }

    pub fn target(&mut self, pass: u64, pad: usize, ticks: f64, one_loop: bool) {
        if !self.erasing() || pass != self.state.pass || pad >= crate::PADS {
            return;
        }
        if one_loop {
            self.close_target(pad, ticks);
        } else if self.ordinary[pad] || self.targets[pad].is_some() {
            return;
        }
        self.start_target(
            pad,
            ticks,
            if one_loop {
                ticks + f64::from(self.state.loop_ticks)
            } else {
                f64::INFINITY
            },
        );
    }

    pub(crate) fn held_target(&self, pad: usize) -> Option<u64> {
        self.targets.get(pad)?.map(|target| target.owner)
    }

    pub(crate) fn release_target(&mut self, pass: u64, pad: usize, ticks: f64) {
        if self.state.pass == pass && self.erasing() && pad < crate::PADS {
            self.close_target(pad, ticks);
        }
    }

    pub(crate) fn refuse_target(&mut self) {
        self.state.dropped_events = self.state.dropped_events.saturating_add(1);
        self.state.erase_refusals = self.state.erase_refusals.saturating_add(1);
    }

    fn start_target(&mut self, pad: usize, ticks: f64, until: f64) {
        let Some(sequence) = self.state.sequence.checked_add(1) else {
            self.refuse_target();
            return;
        };
        self.state.sequence = sequence;
        let Some(index) = self.state.erase_intervals.checked_add(1) else {
            self.refuse_target();
            return;
        };
        let Some(slot) = self.erase_slots.take(sequence) else {
            self.refuse_target();
            return;
        };
        let Some(window) =
            self.windows
                .start(pad, ticks, until, sequence, (self.grid, self.strength))
        else {
            self.erase_slots.release(slot, sequence);
            self.refuse_target();
            return;
        };
        let interval = EraseInterval {
            epoch: self.state.epoch,
            pass: self.state.pass,
            run: self.run,
            closed_sequence: 0,
            sequence,
            index,
            slot,
            pad,
            start: ticks,
            end: until,
            complete: false,
            loop_ticks: self.state.loop_ticks,
            grid: self.grid,
            strength: self.strength,
        };
        if let Some(recovery) = &self.recovery {
            recovery.intervals[slot].record(interval);
        }
        self.targets[pad] = Some(HeldErase {
            owner: sequence,
            slot,
            window,
            sequence,
            start: ticks,
            until,
        });
        self.state.erase_intervals = index;
        self.state.erasure_sequence = sequence;
    }

    fn close_target(&mut self, pad: usize, ticks: f64) {
        if let Some(target) = self.targets[pad].take() {
            self.state.sequence = self.state.sequence.saturating_add(1);
            self.state.erasure_sequence = self.state.sequence;
            let end = ticks.max(target.start);
            self.windows
                .close(pad, target.window, end, self.state.sequence);
            if let Some(recovery) = &self.recovery {
                recovery.intervals[target.slot].close(end, self.state.sequence);
            }
        }
    }

    pub fn expire_targets(&mut self, ticks: f64) {
        if !self.erasing() {
            return;
        }
        for pad in 0..crate::PADS {
            if let Some(target) =
                self.targets[pad].filter(|t| t.until <= ticks + self.step * 0.5 + 1e-9)
            {
                self.close_target(pad, target.until);
            }
        }
    }

    pub fn has_targets(&self, pad: usize) -> bool {
        self.windows.heads[pad].is_some()
    }

    pub fn targets(
        &self,
        pad: usize,
        tick: f64,
        row: crate::pattern::PlaybackRow,
        meta: (Grid, u8),
    ) -> bool {
        self.windows
            .contains(pad, tick, row, (self.run, self.state.pass), meta)
    }

    pub fn prune_targets(&mut self, due: f64) {
        self.windows.prune(due);
    }

    pub fn commit_closed(&mut self, pattern: Option<&Pattern>, ticks: f64) {
        if !self.windows.dirty {
            return;
        }
        self.windows.dirty = false;
        self.apply_windows(pattern, ticks);
    }

    fn apply_windows(&mut self, pattern: Option<&Pattern>, ticks: f64) {
        if self.windows.slots.len == MAX_HITS {
            return;
        }
        if let Some(pattern) = pattern
            && let Some(rows) = &pattern.rows
        {
            for (hit, row) in pattern.events().iter().zip(rows) {
                if self.windows.traversed(
                    hit.pad(),
                    hit.tick(),
                    pattern.loop_ticks(),
                    *row,
                    (self.run, self.state.pass, ticks, self.step),
                    (pattern.grid(), pattern.strength()),
                ) {
                    self.remove(row.id);
                }
            }
        }
    }

    pub fn removed(&self, id: RowId) -> bool {
        self.unconfirmed != 0 && self.removed[id.slot] == id.generation
    }

    pub fn remove(&mut self, id: RowId) {
        if self.removed(id) {
            return;
        }
        let Some(sequence) = self.state.sequence.checked_add(1) else {
            return;
        };
        self.state.sequence = sequence;
        self.state.erasure_sequence = sequence;
        self.removed[id.slot] = id.generation;
        self.unconfirmed += 1;
        self.state.reserved_rows = self.state.reserved_rows.saturating_sub(1);
        self.state.erased_rows = self.state.erased_rows.saturating_add(1);
        if let Some(recovery) = &self.recovery {
            recovery.removed[id.slot].record(self.state, id);
        }
    }

    pub fn published(&mut self, pattern: Option<&Pattern>, previous: Option<&Pattern>, ticks: f64) {
        let grid = pattern.map_or(Grid::Off, Pattern::grid);
        let strength = pattern.map_or(0, Pattern::strength);
        if (grid, strength) != (self.grid, self.strength) {
            let targets = self.targets;
            for pad in 0..crate::PADS {
                self.close_target(pad, ticks);
            }
            self.commit_closed(previous, ticks);
            self.grid = grid;
            self.strength = strength;
            for (pad, target) in targets.iter().enumerate() {
                if let Some(target) = target.filter(|t| ticks + self.step * 0.5 + 1e-9 < t.until) {
                    self.start_target(pad, ticks, target.until);
                    if let Some(current) = self.targets[pad].as_mut() {
                        current.owner = target.owner;
                    }
                }
            }
        }
        if self.unconfirmed == 0 {
            self.apply_windows(pattern, ticks);
            return;
        }
        self.present.fill(0);
        if let Some(rows) = pattern.and_then(|p| p.rows.as_deref()) {
            for row in rows {
                self.present[row.id.slot] = row.id.generation;
            }
        }
        for (slot, generation) in self.removed.iter_mut().enumerate() {
            if *generation != 0 && self.present[slot] != *generation {
                *generation = 0;
                self.unconfirmed -= 1;
            }
        }
        self.apply_windows(pattern, ticks);
    }

    pub fn consume_interval(&mut self, pass: u64, slot: usize, sequence: u64) {
        if pass != self.state.pass || self.erase_slots.owned.get(slot) != Some(&sequence) {
            return;
        }
        if self.targets.iter().any(|target| {
            target.is_some_and(|target| target.slot == slot && target.sequence == sequence)
        }) {
            return;
        }
        self.erase_slots.release(slot, sequence);
    }

    pub fn consume_gate(&mut self, pass: u64, slot: usize, sequence: u64) {
        if pass != self.state.pass || self.gate_slots.owned.get(slot) != Some(&sequence) {
            return;
        }
        for pending in &mut self.pending {
            if pending.is_some_and(|event| event.gate == Some(slot) && event.sequence == sequence) {
                *pending = None;
            }
        }
        self.gate_slots.release(slot, sequence);
    }

    pub fn confirm_rows(
        &mut self,
        pass: u64,
        sequence: u64,
        captured: u32,
        removed: u64,
        rows: u32,
    ) {
        if pass == 0
            || pass != self.state.pass
            || !matches!(self.state.phase, Phase::Recording | Phase::Finishing)
            || sequence < self.reservation_sequence
            || sequence > self.state.sequence
            || captured > self.state.captured_onsets
            || removed > self.state.erased_rows
            || rows as usize > MAX_HITS
        {
            return;
        }
        let total = u64::from(rows) + u64::from(self.state.captured_onsets - captured);
        let Some(reserved) = total.checked_sub(self.state.erased_rows - removed) else {
            return;
        };
        if reserved <= MAX_HITS as u64 {
            self.state.reserved_rows = reserved as u32;
            self.reservation_sequence = sequence;
        }
    }

    fn close(&mut self, pad: usize, ticks: f64) {
        let Some(onset) = self.pending[pad].take() else {
            return;
        };
        let duration =
            ((ticks - onset.ticks + 1e-9).round()).clamp(1.0, f64::from(MAX_GATE_TICKS)) as u32;
        if let Some(recovery) = &self.recovery
            && let Some(slot) = onset.gate
        {
            recovery.gates[slot].record(onset, duration);
        }
    }

    fn event(&mut self, ticks: f64, kind: Kind) -> Option<Event> {
        let Some(sequence) = self.state.sequence.checked_add(1) else {
            self.omit(kind);
            return None;
        };
        self.state.sequence = sequence;
        Some(Event {
            epoch: self.state.epoch,
            pass: self.state.pass,
            sequence,
            run: self.run,
            frame_ticks: 0.0,
            ticks,
            kind,
            gate: None,
        })
    }

    fn push(&mut self, event: Event) -> bool {
        if self.queue.as_mut().is_none_or(|q| q.push(event).is_err()) {
            self.omit(event.kind);
            false
        } else {
            true
        }
    }

    fn omit(&mut self, kind: Kind) {
        self.state.dropped_events = self.state.dropped_events.saturating_add(1);
        if matches!(kind, Kind::Onset { .. }) {
            self.state.dropped_onsets = self.state.dropped_onsets.saturating_add(1);
        }
    }
}

/// One writer and one bounded read attempt. No reader waits for an audio callback.
#[derive(Default)]
pub(crate) struct Published {
    revision: AtomicU64,
    epoch: AtomicU64,
    pass: AtomicU64,
    phase: AtomicU64,
    ticks: AtomicU64,
    loop_ticks: AtomicU64,
    count_in_beat: AtomicU64,
    sequence: AtomicU64,
    dropped_onsets: AtomicU64,
    dropped_events: AtomicU64,
    reserved_rows: AtomicU64,
    captured_onsets: AtomicU64,
    limit_refusals: AtomicU64,
    erasing: AtomicU64,
    erase_target_low: AtomicU64,
    erase_target_high: AtomicU64,
    erase_intervals: AtomicU64,
    erased_rows: AtomicU64,
    frame_ticks: AtomicU64,
    erase_refusals: AtomicU64,
    erasure_sequence: AtomicU64,
}

impl Published {
    pub fn record(&self, state: Snapshot) {
        self.revision.fetch_add(1, SeqCst);
        self.epoch.store(state.epoch, SeqCst);
        self.pass.store(state.pass, SeqCst);
        self.phase.store(state.phase as u64, SeqCst);
        self.ticks.store(state.ticks.to_bits(), SeqCst);
        self.loop_ticks.store(u64::from(state.loop_ticks), SeqCst);
        self.count_in_beat
            .store(u64::from(state.count_in_beat), SeqCst);
        self.sequence.store(state.sequence, SeqCst);
        self.dropped_onsets.store(state.dropped_onsets, SeqCst);
        self.dropped_events.store(state.dropped_events, SeqCst);
        self.reserved_rows
            .store(u64::from(state.reserved_rows), SeqCst);
        self.captured_onsets
            .store(u64::from(state.captured_onsets), SeqCst);
        self.limit_refusals.store(state.limit_refusals, SeqCst);
        self.erasing.store(u64::from(state.erasing), SeqCst);
        self.erase_target_low.store(state.erase_targets[0], SeqCst);
        self.erase_target_high.store(state.erase_targets[1], SeqCst);
        self.erase_intervals.store(state.erase_intervals, SeqCst);
        self.erased_rows.store(state.erased_rows, SeqCst);
        self.frame_ticks.store(state.frame_ticks.to_bits(), SeqCst);
        self.erase_refusals.store(state.erase_refusals, SeqCst);
        self.erasure_sequence.store(state.erasure_sequence, SeqCst);
        self.revision.fetch_add(1, SeqCst);
    }

    pub fn read(&self, epoch: u64) -> Option<Snapshot> {
        let revision = self.revision.load(SeqCst);
        if revision & 1 != 0 || self.epoch.load(SeqCst) != epoch {
            return None;
        }
        let state = Snapshot {
            epoch,
            pass: self.pass.load(SeqCst),
            phase: match self.phase.load(SeqCst) {
                1 => Phase::CountIn,
                2 => Phase::Recording,
                3 => Phase::Finishing,
                _ => Phase::Idle,
            },
            ticks: f64::from_bits(self.ticks.load(SeqCst)),
            loop_ticks: self.loop_ticks.load(SeqCst) as u32,
            count_in_beat: self.count_in_beat.load(SeqCst) as u8,
            sequence: self.sequence.load(SeqCst),
            dropped_onsets: self.dropped_onsets.load(SeqCst),
            dropped_events: self.dropped_events.load(SeqCst),
            reserved_rows: self.reserved_rows.load(SeqCst) as u32,
            captured_onsets: self.captured_onsets.load(SeqCst) as u32,
            limit_refusals: self.limit_refusals.load(SeqCst),
            erasing: self.erasing.load(SeqCst) != 0,
            erase_targets: [
                self.erase_target_low.load(SeqCst),
                self.erase_target_high.load(SeqCst),
            ],
            erase_intervals: self.erase_intervals.load(SeqCst),
            erased_rows: self.erased_rows.load(SeqCst),
            frame_ticks: f64::from_bits(self.frame_ticks.load(SeqCst)),
            erase_refusals: self.erase_refusals.load(SeqCst),
            erasure_sequence: self.erasure_sequence.load(SeqCst),
        };
        (self.revision.load(SeqCst) == revision).then_some(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn capture_counter_exhaustion_cannot_overbook_reused_capacity() {
        let mut recorder = Recorder::default();
        let mut events = recorder.enable();
        recorder.start(1, 0.0, 3840, true, 0);
        recorder.state.captured_onsets = u32::MAX;
        recorder.onset(0, 1.0, PadMode::Gate, 0.0, 0.04);
        assert_eq!(recorder.state.reserved_rows, 0);
        assert_eq!(recorder.state.captured_onsets, u32::MAX);
        assert_eq!(recorder.state.dropped_onsets, 1);
        assert!(events.pop().is_err());
    }

    #[test]
    fn erase_interval_reads_never_mix_concurrent_holds_or_wait_for_a_writer() {
        let slot = Arc::new(EraseSlot::default());
        let a = EraseInterval {
            epoch: 7,
            pass: 2,
            run: 3,
            closed_sequence: 11,
            sequence: 10,
            index: 1,
            slot: 0,
            pad: 127,
            start: 240.0,
            end: 480.0,
            complete: false,
            loop_ticks: 3840,
            grid: Grid::Sixteenth,
            strength: 100,
        };
        let b = EraseInterval {
            epoch: 8,
            pass: 9,
            run: 10,
            closed_sequence: 31,
            sequence: 30,
            index: 2,
            pad: 0,
            start: 1000.0,
            end: 2200.0,
            loop_ticks: 7680,
            grid: Grid::EighthTriplet,
            strength: 50,
            ..a
        };
        slot.record(a);
        let writer = slot.clone();
        let thread = std::thread::spawn(move || {
            for _ in 0..20_000 {
                writer.record(a);
                writer.record(b);
            }
        });
        for _ in 0..20_000 {
            if let Some(interval) = slot.read(0) {
                assert!(interval == a || interval == b, "{interval:?}");
            }
        }
        thread.join().unwrap();
        slot.revision.store(1, SeqCst);
        assert_eq!(slot.read(0), None);
    }

    #[test]
    fn erased_identity_reads_never_mix_projects_or_generations() {
        let slot = Arc::new(RemovedSlot::default());
        let a = Snapshot {
            epoch: 7,
            pass: 2,
            sequence: 10,
            ..Snapshot::default()
        };
        let b = Snapshot {
            epoch: 8,
            pass: 9,
            sequence: 30,
            ..Snapshot::default()
        };
        let first = RowId::new(0, 11).unwrap();
        let second = RowId::new(0, 22).unwrap();
        slot.record(a, first);
        let writer = slot.clone();
        let thread = std::thread::spawn(move || {
            for _ in 0..20_000 {
                writer.record(a, first);
                writer.record(b, second);
            }
        });
        for _ in 0..20_000 {
            if let Some(value) = slot.read() {
                assert!(value == (7, 2, 11, 10) || value == (8, 9, 22, 30));
            }
        }
        thread.join().unwrap();
        slot.revision.store(1, SeqCst);
        assert_eq!(slot.read(), None);
    }

    #[test]
    fn recording_snapshots_never_mix_projects_passes_or_boundaries() {
        let shared = Arc::new(Published::default());
        let a = Snapshot {
            epoch: 7,
            pass: 1,
            phase: Phase::CountIn,
            ticks: 960.0,
            loop_ticks: 3840,
            count_in_beat: 3,
            sequence: 0,
            dropped_onsets: 0,
            dropped_events: 0,
            reserved_rows: 4000,
            captured_onsets: 0,
            limit_refusals: 0,
            erasing: false,
            erase_targets: [0, 0],
            erase_intervals: 0,
            erased_rows: 0,
            frame_ticks: 0.04,
            erase_refusals: 0,
            erasure_sequence: 0,
        };
        let b = Snapshot {
            epoch: 8,
            pass: 99,
            phase: Phase::Finishing,
            ticks: 7890.5,
            loop_ticks: 7680,
            count_in_beat: 0,
            sequence: 600,
            dropped_onsets: 40,
            dropped_events: 88,
            reserved_rows: 4096,
            captured_onsets: 512,
            limit_refusals: 12,
            erasing: true,
            erase_targets: [19, 1 << 63],
            erase_intervals: 7,
            erased_rows: 90,
            frame_ticks: 0.1,
            erase_refusals: 11,
            erasure_sequence: 7,
        };
        shared.record(a);
        let writer = shared.clone();
        let thread = std::thread::spawn(move || {
            for i in 0..20_000 {
                writer.record(if i & 1 == 0 { a } else { b });
            }
        });
        for _ in 0..20_000 {
            if let Some(state) = shared.read(7) {
                assert_eq!(state, a);
            }
            if let Some(state) = shared.read(8) {
                assert_eq!(state, b);
            }
            assert_eq!(shared.read(6), None);
        }
        thread.join().unwrap();
        assert_eq!(shared.read(8), Some(b));
        shared.revision.store(1, SeqCst);
        assert_eq!(shared.read(8), None);
    }

    #[test]
    fn recovery_requires_the_original_project_pass_press_and_terminal_slot() {
        let mut recorder = Recorder::default();
        let mut events = recorder.enable();
        let recovery = recorder.recovery().unwrap();
        recorder.reset(7);
        recorder.start(8, 0.0, 3840, true, 0);
        recorder.onset(0, 1.0, PadMode::Gate, 0.0, 0.04);
        let onset = events.pop().unwrap();
        assert_eq!(recovery.completed(onset), None);
        recorder.release(0, 25.0);
        let complete = Hit::new(0, 0, 127, 25).unwrap();
        assert_eq!(recovery.completed(onset), Some(complete));
        for stale in [
            Event { epoch: 6, ..onset },
            Event { pass: 7, ..onset },
            Event {
                sequence: 99,
                ..onset
            },
            Event {
                gate: None,
                ..onset
            },
            Event {
                gate: Some(MAX_HITS),
                ..onset
            },
        ] {
            assert_eq!(recovery.completed(stale), None);
        }
        assert_eq!(
            recovery.completed(events.pop().unwrap()),
            None,
            "release is not an onset"
        );
        recorder.onset(0, 1.0, PadMode::Gate, 30.0, 0.04);
        let newer = events.pop().unwrap();
        assert_eq!(recovery.completed(newer), None);
        assert_eq!(recovery.completed(onset), Some(complete));
        recorder.reset(9);
        assert_eq!(recovery.completed(onset), None);
        assert_eq!(recovery.completed(newer), None);
    }

    #[test]
    fn interrupted_recovery_distinguishes_pending_from_unstable_or_stale_slots() {
        let mut recorder = Recorder::default();
        let mut events = recorder.enable();
        let recovery = recorder.recovery().unwrap();
        recorder.start(8, 0.0, 3840, true, 0);
        recorder.onset(0, 1.0, PadMode::Gate, 0.0, 0.04);
        let onset = events.pop().unwrap();
        let boundary = recorder.snapshot(240.0);
        assert_eq!(recovery.completed(onset), None);
        assert_eq!(
            recovery.interrupted(onset, boundary),
            Some(Hit::new(0, 0, 127, 240).unwrap())
        );
        let gate = &recovery.gates[onset.gate.unwrap()];
        let revision = gate.revision.load(SeqCst);
        gate.revision.store(revision | 1, SeqCst);
        assert_eq!(recovery.interrupted(onset, boundary), None);
        gate.revision.store(revision, SeqCst);
        let revision = recovery.owner.revision.load(SeqCst);
        recovery.owner.revision.store(revision | 1, SeqCst);
        assert_eq!(recovery.interrupted(onset, boundary), None);
        recovery.owner.revision.store(revision, SeqCst);
        gate.record(
            Event {
                sequence: onset.sequence + 1,
                ..onset
            },
            0,
        );
        assert_eq!(recovery.interrupted(onset, boundary), None);
        gate.record(onset, 17);
        assert_eq!(
            recovery.interrupted(onset, boundary).unwrap().gate_ticks(),
            17
        );
        recorder.reset(1);
        assert_eq!(recovery.interrupted(onset, boundary), None);
    }

    #[test]
    fn interrupted_recovery_never_combines_concurrent_terminal_press_data() {
        let mut recorder = Recorder::default();
        let mut events = recorder.enable();
        let recovery = recorder.recovery().unwrap();
        recorder.start(8, 0.0, 3840, true, 0);
        recorder.onset(0, 1.0, PadMode::Gate, 0.0, 0.04);
        let onset = events.pop().unwrap();
        let boundary = recorder.snapshot(240.0);
        let other = Event { pass: 9, ..onset };
        let writer = recovery.clone();
        let thread = std::thread::spawn(move || {
            for _ in 0..20_000 {
                writer.gates[0].record(onset, 0);
                writer.gates[0].record(other, 91);
                writer.gates[0].record(onset, 17);
                writer.gates[0].record(other, 91);
            }
        });
        for _ in 0..20_000 {
            if let Some(row) = recovery.interrupted(onset, boundary) {
                assert_eq!((row.tick(), row.pad(), row.velocity()), (0, 0, 127));
                assert!(row.gate_ticks() == 240 || row.gate_ticks() == 17);
            }
        }
        thread.join().unwrap();
        assert_eq!(recovery.interrupted(onset, boundary), None);
    }

    #[test]
    fn concurrent_terminal_reads_never_combine_different_press_identities() {
        let gate = Arc::new(Gate::default());
        let a = Event {
            epoch: 7,
            pass: 8,
            sequence: 11,
            run: 1,
            frame_ticks: 0.04,
            ticks: 0.0,
            kind: Kind::Onset {
                pad: 0,
                velocity: 127,
                mode: PadMode::Gate,
                tick: 0,
            },
            gate: Some(0),
        };
        let b = Event {
            epoch: 8,
            pass: 9,
            sequence: 12,
            ..a
        };
        let writer = gate.clone();
        let thread = std::thread::spawn(move || {
            for _ in 0..20_000 {
                writer.record(a, 17);
                writer.record(b, 91);
            }
        });
        for _ in 0..20_000 {
            if let Some(duration) = gate.read(a) {
                assert_eq!(duration, 17);
            }
            if let Some(duration) = gate.read(b) {
                assert_eq!(duration, 91);
            }
        }
        thread.join().unwrap();
        assert_eq!(gate.read(a), None);
        assert_eq!(gate.read(b), Some(91));
        gate.revision.store(1, SeqCst);
        assert_eq!(gate.read(b), None);
    }
}

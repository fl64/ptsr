//! Accepted manual-pad capture. Collection and compilation run on the UI thread.

use std::sync::Arc;

use engine::capture::{CAPACITY, EraseInterval, Event, Kind, Phase, Recovery, Snapshot};
use engine::pattern::{MAX_HITS, PlaybackRow, RawPattern, RowId, TICKS_PER_BAR};
use engine::{Command, Engine, PadMode};

use crate::pads::Pads;

/// Pattern controls: the three settings, then the Clear A1 command row (98b).
pub(super) const PANEL_ROWS: usize = 4;
pub(super) const CLEAR_ROW: usize = 3;

const GRIDS: [&str; 9] = [
    "off", "1/4", "1/8", "1/16", "1/32", "1/64", "1/8T", "1/16T", "1/32T",
];

#[derive(Clone, Copy)]
pub(super) enum Setting {
    Bars,
    Quantize,
    Strength,
}

impl Setting {
    pub(super) const ALL: [Self; 3] = [Self::Bars, Self::Quantize, Self::Strength];

    pub(super) fn from_param(param: input::midi::Param) -> Option<Self> {
        match param {
            input::midi::Param::PatternBars => Some(Self::Bars),
            input::midi::Param::PatternQuantize => Some(Self::Quantize),
            input::midi::Param::PatternStrength => Some(Self::Strength),
            _ => None,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Bars => "Bars",
            Self::Quantize => "Quantize",
            Self::Strength => "Strength",
        }
    }

    fn count(self) -> usize {
        match self {
            Self::Bars => 64,
            Self::Quantize => GRIDS.len(),
            Self::Strength => 11,
        }
    }

    pub(super) fn index(self, pads: &Pads) -> usize {
        let Some(pattern) = &pads.pattern else {
            return match self {
                Self::Bars => 1,
                Self::Quantize => 3,
                Self::Strength => 10,
            };
        };
        match self {
            Self::Bars => usize::from(pattern.bars.saturating_sub(1)),
            Self::Quantize => GRIDS
                .iter()
                .position(|grid| *grid == pattern.quantize)
                .unwrap_or(3),
            Self::Strength => usize::from(pattern.strength / 10),
        }
    }

    pub(super) fn position(self, pads: &Pads) -> f32 {
        self.index(pads) as f32 / (self.count() - 1) as f32
    }

    pub(super) fn text(self, pads: &Pads) -> String {
        let index = self.index(pads);
        match self {
            Self::Bars => (index + 1).to_string(),
            Self::Quantize => {
                if index == 0 {
                    "Off".into()
                } else {
                    GRIDS[index].into()
                }
            }
            Self::Strength => format!("{} %", index * 10),
        }
    }

    fn target(self) -> u64 {
        u64::MAX - 2 - self as u64
    }

    fn set(self, pattern: &mut crate::session::PatternState, index: usize) {
        match self {
            Self::Bars => pattern.bars = (index + 1) as u8,
            Self::Quantize => pattern.quantize = GRIDS[index].into(),
            Self::Strength => pattern.strength = (index * 10) as u8,
        }
    }
}

trait Feedback {
    fn erased(&self, id: RowId, boundary: Snapshot) -> Option<bool>;
    fn erase_interval(&self, slot: usize, boundary: Snapshot) -> Option<EraseInterval>;
    fn completed(&self, onset: Event) -> Option<engine::pattern::Hit>;
    fn interrupted(&self, onset: Event, boundary: Snapshot) -> Option<engine::pattern::Hit>;
}

impl Feedback for Recovery {
    fn erased(&self, id: RowId, boundary: Snapshot) -> Option<bool> {
        Recovery::erased(self, id, boundary)
    }

    fn erase_interval(&self, slot: usize, boundary: Snapshot) -> Option<EraseInterval> {
        Recovery::erase_interval(self, slot, boundary)
    }

    fn completed(&self, onset: Event) -> Option<engine::pattern::Hit> {
        Recovery::completed(self, onset)
    }

    fn interrupted(&self, onset: Event, boundary: Snapshot) -> Option<engine::pattern::Hit> {
        Recovery::interrupted(self, onset, boundary)
    }
}

pub struct Capture {
    events: rtrb::Consumer<Event>,
    recovery: Arc<Recovery>,
    pass: Option<Pass>,
    requested: Option<(u64, u64, Undo)>,
    undo_requested: bool,
    undo_pending: Option<Undo>,
    end_requested: bool,
    erase_requested: Option<bool>,
    last: Option<Snapshot>,
    unavailable: bool,
    exiting: bool,
    dirty: bool,
    generations: Box<[u64]>,
}

struct Pass {
    epoch: u64,
    id: u64,
    sequence: u64,
    pending: Vec<(RowId, Event)>,
    /// Rows this pass captured, in capture order: they replace same-step rows at its end.
    fresh: Vec<RowId>,
    replaced: usize,
    reclaim: Vec<Event>,
    captured: u32,
    consumed_intervals: Box<[u64]>,
    complete_intervals: u64,
    published_erasure: u64,
    confirmed: Option<(u64, u32, u64, u32)>,
    losses: (u64, u64, u64),
    acknowledged: bool,
    reported_interruption: bool,
    reported_result: bool,
    before: Option<Undo>,
}

pub struct Undo {
    pattern: crate::session::PatternState,
    rows: Vec<PlaybackRow>,
}

impl Undo {
    fn new(pads: &Pads) -> Result<Self, String> {
        Ok(Self {
            pattern: pads.pattern.clone().ok_or("Recording needs A1.")?,
            rows: pads
                .pattern_rows
                .clone()
                .ok_or("Recording needs row identities.")?,
        })
    }

    fn changed(&self, pads: &Pads) -> bool {
        pads.pattern.as_ref() != Some(&self.pattern)
    }

    pub fn restore(&self, pads: &mut Pads) -> Result<Command, String> {
        pads.restore_recorded_pattern(&self.pattern, &self.rows)
    }
}

#[derive(Default)]
pub struct Update {
    pub commands: Vec<Command>,
    pub notice: Option<String>,
    pub undo: Option<Undo>,
    pub changed: bool,
    /// A pass ended: its new hits and the older hits they replaced.
    pub finished: Option<(u32, usize)>,
}

impl Capture {
    pub fn new(engine: &mut Engine) -> Self {
        let events = engine.enable_capture();
        Self {
            events,
            recovery: engine.capture_recovery().expect("capture setup"),
            pass: None,
            requested: None,
            undo_requested: false,
            undo_pending: None,
            end_requested: false,
            erase_requested: None,
            last: None,
            unavailable: false,
            exiting: false,
            dirty: false,
            generations: vec![0; MAX_HITS].into_boxed_slice(),
        }
    }

    fn closed(&self) -> bool {
        self.unavailable || self.exiting
    }

    pub fn available(&self) -> bool {
        !self.closed() && self.requested.is_none() && self.undo_pending.is_none()
    }

    pub fn blocking_phase(&self) -> Option<&'static str> {
        if !self.blocks_autosave() {
            return None;
        }
        Some(if self.exiting {
            "exit pending"
        } else if self.requested.is_some() {
            "start pending"
        } else {
            match self.last.map(|s| s.phase) {
                Some(Phase::CountIn) => "count-in",
                Some(Phase::Recording) => "recording",
                _ => "finishing",
            }
        })
    }

    pub fn blocks_autosave(&self) -> bool {
        if self.requested.is_some() || self.dirty || self.undo_pending.is_some() {
            return true;
        }
        let Some(boundary) = self.last else {
            return false;
        };
        if self.closed() {
            matches!(boundary.phase, Phase::Recording | Phase::Finishing)
                && self
                    .pass
                    .as_ref()
                    .is_some_and(|pass| pass.before.is_some() || !pass.pending.is_empty())
        } else {
            boundary.phase != Phase::Idle
        }
    }

    pub fn request_undo(&mut self) -> bool {
        if self.requested.is_some()
            || self.undo_pending.is_some()
            || self.pass.as_ref().is_some_and(|pass| pass.before.is_some())
        {
            self.undo_requested = true;
            true
        } else {
            false
        }
    }

    pub fn finish_for_save(&self) -> Option<Command> {
        if self.closed() {
            return None;
        }
        let pass = self
            .requested
            .as_ref()
            .map(|(_, pass, _)| *pass)
            .or_else(|| {
                self.last
                    .filter(|s| matches!(s.phase, Phase::CountIn | Phase::Recording))
                    .map(|s| s.pass)
            })?;
        Some(Command::PatternRecordEnd { pass })
    }

    pub fn begin(&mut self, epoch: u64, pass: u64, pads: &Pads) -> Result<(), String> {
        let before = Undo::new(pads)?;
        for row in &before.rows {
            let id = row.id();
            self.generations[id.slot()] = self.generations[id.slot()].max(id.generation());
        }
        self.requested = Some((epoch, pass, before));
        self.end_requested = false;
        self.erase_requested = None;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.exiting = false;
        self.pass = None;
        self.requested = None;
        self.undo_requested = false;
        self.undo_pending = None;
        self.end_requested = false;
        self.erase_requested = None;
        self.last = None;
        self.dirty = false;
        self.generations.fill(0);
        for _ in 0..CAPACITY {
            if self.events.pop().is_err() {
                break;
            }
        }
    }

    pub fn poll_for_exit(
        &mut self,
        epoch: u64,
        snapshot: Option<Snapshot>,
        stopped: bool,
        pads: &mut Pads,
    ) -> Result<Update, String> {
        if !self.closed()
            && !stopped
            && snapshot.filter(|s| s.epoch == epoch).is_none()
            && self.blocks_autosave()
        {
            return Ok(Update::default());
        }
        if !self.closed()
            && let Some(snapshot) = snapshot.filter(|s| s.epoch == epoch)
        {
            self.last = Some(snapshot);
        }
        self.exiting = true;
        self.poll(epoch, None, stopped, pads)
    }

    pub fn poll(
        &mut self,
        epoch: u64,
        snapshot: Option<Snapshot>,
        stopped: bool,
        pads: &mut Pads,
    ) -> Result<Update, String> {
        let recovery = self.recovery.clone();
        self.poll_feedback(epoch, snapshot, stopped, pads, recovery.as_ref())
    }

    fn poll_feedback(
        &mut self,
        epoch: u64,
        snapshot: Option<Snapshot>,
        stopped: bool,
        pads: &mut Pads,
        feedback: &impl Feedback,
    ) -> Result<Update, String> {
        let mut update = Update::default();
        // A closed collector never advances its boundary or resumes an old pass.
        if !self.closed()
            && let Some(snapshot) = snapshot.filter(|s| s.epoch == epoch)
        {
            self.last = Some(snapshot);
        }
        self.unavailable |= stopped;
        let closed = self.closed();
        // The previous poll queued the complete take and its acknowledgement first.
        if let Some(before) = &self.undo_pending {
            let command = before.restore(pads)?;
            if !closed {
                update.commands.push(command);
            }
            self.undo_pending = None;
            self.undo_requested = false;
            update.changed = true;
            update.notice = Some("Pattern pass undone. No redo is available.".into());
        }
        let Some(boundary) = self.last.filter(|s| s.epoch == epoch) else {
            if closed {
                self.requested = None;
                self.undo_requested = false;
            }
            return Ok(update);
        };
        if boundary.phase == Phase::Idle || boundary.pass == 0 {
            self.pass = None;
            if self.requested.is_none() || closed {
                self.undo_requested = false;
                self.end_requested = false;
                self.erase_requested = None;
            }
            if closed
                || self
                    .requested
                    .as_ref()
                    .is_some_and(|&(e, p, _)| e != epoch || p <= boundary.pass)
            {
                self.requested = None;
                self.undo_requested = false;
            }
            for _ in 0..CAPACITY {
                let Ok(event) = self.events.peek() else { break };
                if event.epoch == epoch && event.pass > boundary.pass {
                    break;
                }
                let _ = self.events.pop();
            }
            return Ok(update);
        }
        if self
            .pass
            .as_ref()
            .is_none_or(|p| p.epoch != epoch || p.id != boundary.pass)
        {
            self.pass = Some(Pass {
                epoch,
                id: boundary.pass,
                sequence: 0,
                pending: Vec::new(),
                fresh: Vec::new(),
                replaced: 0,
                reclaim: Vec::new(),
                captured: 0,
                consumed_intervals: vec![0; MAX_HITS].into_boxed_slice(),
                complete_intervals: 0,
                published_erasure: 0,
                confirmed: None,
                losses: (0, 0, 0),
                acknowledged: false,
                reported_interruption: false,
                reported_result: false,
                before: Some(
                    self.requested
                        .take()
                        .filter(|&(e, p, _)| e == epoch && p == boundary.pass)
                        .map(|(_, _, before)| before)
                        .ok_or("Recording has no matching start snapshot.")?,
                ),
            });
        }
        let pass = self.pass.as_mut().unwrap();
        // History can already have restored rows while the audio acknowledgement is queued.
        if pass.acknowledged || (closed && pass.before.is_none()) {
            return Ok(update);
        }
        let pattern = pads.pattern.as_mut().ok_or("Recording needs A1.")?;
        let descriptors = pads
            .pattern_rows
            .as_mut()
            .ok_or("Recording needs row identities.")?;
        if descriptors.len() != pattern.hits.len() {
            return Err("Recording rows and identities do not match.".into());
        }
        let intervals: Vec<_> = if boundary.erase_intervals > pass.complete_intervals {
            (0..MAX_HITS)
                .filter_map(|slot| feedback.erase_interval(slot, boundary))
                .filter(|interval| pass.consumed_intervals[interval.slot] != interval.sequence)
                .collect()
        } else {
            Vec::new()
        };
        let mut feedback_ready = true;
        let mut removed = [false; MAX_HITS];
        if boundary.erased_rows != 0 {
            for row in descriptors.iter() {
                match feedback.erased(row.id(), boundary) {
                    Some(erased) => removed[row.id().slot()] = erased,
                    None => feedback_ready = false,
                }
            }
        }
        self.dirty |= remove_rows(pattern, descriptors, &removed);
        self.dirty |= erase_rows(pattern, descriptors, &intervals, boundary)?;
        for _ in 0..CAPACITY {
            let Ok(&event) = self.events.peek() else {
                break;
            };
            if event.epoch != epoch || event.pass < boundary.pass {
                let _ = self.events.pop();
                continue;
            }
            if event.pass != boundary.pass || event.sequence > boundary.sequence {
                break;
            }
            if event.sequence > pass.sequence
                && matches!(event.kind, Kind::Onset { .. })
                && descriptors.len() == MAX_HITS
            {
                // A concurrent interval read can leave a removed row until the next UI turn.
                feedback_ready = false;
                break;
            }
            if event.sequence <= pass.sequence {
                let _ = self.events.pop();
                continue;
            }
            let Kind::Onset {
                pad,
                velocity,
                mode,
                tick,
            } = event.kind
            else {
                let _ = self.events.pop();
                pass.sequence = event.sequence;
                continue;
            };
            let mut occupied = [false; MAX_HITS];
            for row in descriptors.iter() {
                occupied[row.id().slot()] = true;
            }
            let slot = occupied
                .iter()
                .position(|used| !used)
                .ok_or("Recording exceeds the raw row limit.")?;
            let origin = event
                .origin(boundary.loop_ticks)
                .ok_or("Invalid recording origin.")?;
            let generation = self.generations[slot]
                .max(boundary.pass - 1)
                .checked_add(1)
                .ok_or("Recording row generation is exhausted.")?;
            let id = RowId::new(slot, generation).map_err(|e| e.to_string())?;
            let _ = self.events.pop();
            pass.sequence = event.sequence;
            self.generations[slot] = generation;
            let pending = mode == PadMode::Gate;
            // the audio thread placed the tick with the quantize of its capture (98c)
            pattern
                .hits
                .push([tick, pad as u32, u32::from(velocity), u32::from(pending)]);
            descriptors.push(PlaybackRow::recorded(id, origin, pending));
            pass.fresh.push(id);
            if pending {
                pass.pending.push((id, event));
            }
            pass.captured += 1;
            self.dirty = true;
        }
        self.dirty |= erase_rows(pattern, descriptors, &intervals, boundary)?;
        pass.pending.retain(|&(id, onset)| {
            let Some(index) = descriptors.iter().position(|row| row.id() == id) else {
                pass.reclaim.push(onset);
                return false;
            };
            let hit = if closed {
                feedback.interrupted(onset, boundary)
            } else {
                feedback.completed(onset)
            };
            let Some(hit) = hit else { return true };
            pattern.hits[index][3] = hit.gate_ticks();
            descriptors[index] = PlaybackRow::recorded(
                id,
                onset
                    .origin(boundary.loop_ticks)
                    .expect("accepted onset origin"),
                false,
            );
            self.dirty = true;
            pass.reclaim.push(onset);
            false
        });
        let drained = self.events.peek().is_err()
            || self.events.peek().is_ok_and(|event| {
                event.epoch == epoch
                    && event.pass == boundary.pass
                    && event.sequence > boundary.sequence
            });
        let prefix_ready = drained
            && feedback_ready
            && pass.captured == boundary.captured_onsets
            && pass.complete_intervals + intervals.len() as u64 == boundary.erase_intervals;
        let publish = self.dirty || pass.published_erasure < boundary.erasure_sequence;
        if publish || !pass.reclaim.is_empty() {
            let command = pads.recorded_pattern_command()?;
            update.changed = self.dirty;
            if !closed {
                update.commands.push(match command {
                    Command::PublishRecordedPattern { pattern }
                        if boundary.erasure_sequence != 0 =>
                    {
                        Command::PublishCapturedPattern {
                            pattern,
                            pass: boundary.pass,
                            sequence: boundary.sequence,
                        }
                    }
                    command => command,
                });
            }
            self.dirty = false;
            pass.published_erasure = boundary.erasure_sequence;
        }
        for onset in pass.reclaim.drain(..) {
            if !closed && let Some(slot) = onset.gate {
                update.commands.push(Command::PatternGateConsumed {
                    pass: onset.pass,
                    slot,
                    sequence: onset.sequence,
                });
            }
        }
        if prefix_ready {
            for interval in &intervals {
                if interval.complete || closed {
                    if !closed {
                        update.commands.push(Command::PatternEraseConsumed {
                            pass: interval.pass,
                            slot: interval.slot,
                            sequence: interval.sequence,
                        });
                    }
                    pass.consumed_intervals[interval.slot] = interval.sequence;
                    pass.complete_intervals += 1;
                }
            }
            let confirmation = (
                boundary.sequence,
                boundary.captured_onsets,
                boundary.erased_rows,
                pads.pattern.as_ref().unwrap().hits.len() as u32,
            );
            if !closed && boundary.erasure_sequence != 0 && pass.confirmed != Some(confirmation) {
                update.commands.push(Command::PatternRecordRows {
                    pass: boundary.pass,
                    sequence: confirmation.0,
                    captured_onsets: confirmation.1,
                    erased_rows: confirmation.2,
                    raw_rows: confirmation.3,
                });
                pass.confirmed = Some(confirmation);
            }
        }
        let losses = (
            boundary.dropped_onsets,
            boundary.dropped_events,
            boundary.limit_refusals,
        );
        if losses != pass.losses {
            update.notice = Some(format!(
                "Pattern recording: {} omitted onsets, {} lost events, {} row-limit refusals.",
                losses.0, losses.1, losses.2,
            ));
            pass.losses = losses;
        }
        if self.unavailable && !pass.reported_interruption {
            update.notice = Some(format!(
                "Pattern recording interrupted. Retained accepted rows. {} omitted onsets, {} lost events, {} row-limit refusals.",
                losses.0, losses.1, losses.2,
            ));
            pass.reported_interruption = true;
        }
        let terminal =
            boundary.phase == Phase::Finishing || (closed && boundary.phase == Phase::Recording);
        if terminal
            && pass.pending.is_empty()
            && prefix_ready
            && pass.complete_intervals == boundary.erase_intervals
        {
            if !pass.fresh.is_empty() {
                let pattern = pads.pattern.as_mut().ok_or("Recording needs A1.")?;
                let rows = pads
                    .pattern_rows
                    .as_mut()
                    .ok_or("Recording needs row identities.")?;
                let removed = replaced_rows(pattern, rows, &pass.fresh)?;
                pass.fresh.clear();
                pass.replaced = removed.iter().filter(|&&gone| gone).count();
                if remove_rows(pattern, rows, &removed) {
                    let command = pads.recorded_pattern_command()?;
                    if !closed {
                        update.commands.push(match command {
                            Command::PublishRecordedPattern { pattern }
                                if boundary.erasure_sequence != 0 =>
                            {
                                Command::PublishCapturedPattern {
                                    pattern,
                                    pass: boundary.pass,
                                    sequence: boundary.sequence,
                                }
                            }
                            command => command,
                        });
                    }
                    update.changed = true;
                }
            }
            if let Some(before) = pass.before.take().filter(|before| before.changed(pads)) {
                if self.undo_requested {
                    self.undo_pending = Some(before);
                } else {
                    update.undo = Some(before);
                }
                update.changed = true;
            }
            // a pass with capture warnings or an interruption already has its notice
            if !self.undo_requested
                && !pass.reported_result
                && pass.losses == (0, 0, 0)
                && !self.unavailable
                && !self.exiting
            {
                update.finished = Some((pass.captured, pass.replaced));
            }
            pass.reported_result = true;
            self.undo_requested = false;
            if !closed && !pass.acknowledged {
                update.commands.push(Command::PatternRecordAck {
                    pass: boundary.pass,
                    sequence: boundary.sequence,
                });
                pass.acknowledged = true;
            }
        }
        Ok(update)
    }
}

impl crate::App {
    pub(super) fn pattern_panel_action(&mut self, action: input::Action) {
        use input::Action;
        if self.transport_input_blocked() {
            return;
        }
        if action == Action::PatternControls {
            if self.pattern_panel.take().is_some() {
                return;
            }
            self.flush_audio();
            if self.pads.pattern.is_none() {
                let boundary = match self.pattern_edit_boundary(Setting::Bars) {
                    Ok(boundary) => boundary,
                    Err(text) => {
                        self.pattern_notice(text);
                        return;
                    }
                };
                if let Err(error) = self.apply_pattern_setting(
                    Setting::Bars,
                    Setting::Bars.position(&self.pads),
                    boundary,
                ) {
                    self.post_notice(crate::Level::Error, (error, std::time::Instant::now()));
                    return;
                }
            }
            self.pattern_panel = Some(0);
            return;
        }
        let Some(selected) = self.pattern_panel else {
            self.pattern_notice("Open pattern controls before selecting or changing a value.");
            return;
        };
        match action {
            Action::PatternParamPrev => self.pattern_panel = Some(selected.saturating_sub(1)),
            Action::PatternParamNext => {
                self.pattern_panel = Some((selected + 1).min(PANEL_ROWS - 1));
            }
            // the change keys only change values; Enter runs the Clear A1 row
            Action::PatternValueDec | Action::PatternValueInc if selected == CLEAR_ROW => {}
            Action::PatternValueDec | Action::PatternValueInc => {
                self.flush_audio();
                let setting = Setting::ALL[selected];
                let boundary = match self.pattern_edit_boundary(setting) {
                    Ok(boundary) => boundary,
                    Err(text) => {
                        self.pattern_notice(text);
                        return;
                    }
                };
                let index = setting.index(&self.pads);
                let index = if action == Action::PatternValueInc {
                    (index + 1).min(setting.count() - 1)
                } else {
                    index.saturating_sub(1)
                };
                match self.apply_pattern_setting(
                    setting,
                    index as f32 / (setting.count() - 1) as f32,
                    boundary,
                ) {
                    Ok(true) => self.takeover.reset_target(setting.target()),
                    Ok(false) => {}
                    Err(error) => {
                        self.post_notice(crate::Level::Error, (error, std::time::Instant::now()));
                    }
                }
            }
            _ => {}
        }
    }

    /// What limits the selected control row, and whether it refuses now (98b).
    pub(super) fn pattern_controls_restriction(&self, row: usize) -> (&'static str, bool) {
        let locked = self.audio.as_ref().is_some_and(|audio| {
            audio.pattern_capture.end_requested || audio.pattern_capture.requested.is_some()
        }) || matches!(
            self.pattern_record_state().map(|s| s.phase),
            Some(Phase::CountIn | Phase::Finishing)
        );
        let recording = self
            .pattern_record_state()
            .is_some_and(|s| s.phase == Phase::Recording);
        if row == CLEAR_ROW {
            return if locked || recording {
                ("end the pass first", true)
            } else if self
                .pads
                .pattern
                .as_ref()
                .is_some_and(|p| !p.hits.is_empty())
            {
                ("Enter", false)
            } else {
                ("", false)
            };
        }
        if locked {
            return ("locked", true);
        }
        match (Setting::ALL[row.min(Setting::ALL.len() - 1)], recording) {
            (Setting::Bars, true) => ("Idle only", true),
            (Setting::Bars, false) => ("1-64", false),
            (Setting::Strength, _) => ("next hits, step 10", false),
            (Setting::Quantize, _) => ("next hits", false),
        }
    }

    pub(super) fn pattern_setting_indexes(&self) -> [usize; 3] {
        Setting::ALL.map(|setting| setting.index(&self.pads))
    }

    pub(super) fn reset_pattern_takeover(&mut self) {
        for setting in Setting::ALL {
            self.takeover.reset_target(setting.target());
        }
    }

    pub(super) fn pattern_stop_requested(&mut self) {
        if let Some(audio) = self.audio.as_mut()
            && audio.pattern_capture.finish_for_save().is_some()
        {
            audio.pattern_capture.end_requested = true;
        }
    }

    fn pattern_edit_boundary(&self, setting: Setting) -> Result<Option<Snapshot>, &'static str> {
        use std::sync::atomic::Ordering::{Acquire, Relaxed};
        let Some(audio) = &self.audio else {
            return Ok(None);
        };
        let capture = &audio.pattern_capture;
        if capture.requested.is_some() {
            return Err("Pattern setting blocked: wait for recording start to be accepted.");
        }
        if capture.undo_pending.is_some() {
            return Err("Pattern setting blocked: wait for pattern Undo to complete.");
        }
        if audio.stopped.load(Relaxed) {
            return if capture.blocks_autosave() {
                Err("Pattern setting blocked: wait for the interrupted take to finish recovery.")
            } else {
                Ok(None)
            };
        }
        if capture.end_requested {
            return Err("Pattern setting blocked while finishing: wait for the completed pass.");
        }
        let epoch = audio.project.epoch.load(Acquire);
        let Some(state) = capture.last.filter(|s| s.epoch == epoch) else {
            return if capture.blocks_autosave() {
                Err("Pattern setting blocked: wait for accepted recording feedback.")
            } else {
                Ok(None)
            };
        };
        match state.phase {
            Phase::CountIn => Err(
                "Pattern setting blocked during count-in: wait for Recording or cancel count-in.",
            ),
            Phase::Finishing => {
                Err("Pattern setting blocked while finishing: wait for the completed pass.")
            }
            Phase::Recording if matches!(setting, Setting::Bars) => {
                Err("Bars change only while Idle: finish the recording pass first.")
            }
            Phase::Recording => {
                let rows = self.pads.pattern.as_ref().map_or(0, |p| p.hits.len()) as u32;
                if capture.pass.as_ref().is_none_or(|pass| {
                    state.erasure_sequence != 0
                        && pass.confirmed
                            != Some((
                                state.sequence,
                                state.captured_onsets,
                                state.erased_rows,
                                rows,
                            ))
                }) {
                    Err("Pattern setting blocked: wait for accepted erasure feedback.")
                } else {
                    Ok(Some(state))
                }
            }
            Phase::Idle => Ok(Some(state)),
        }
    }

    fn apply_pattern_setting(
        &mut self,
        setting: Setting,
        position: f32,
        boundary: Option<Snapshot>,
    ) -> Result<bool, String> {
        let index = (position.clamp(0.0, 1.0) * (setting.count() - 1) as f32).round() as usize;
        let created = self.pads.pattern.is_none();
        if !created && setting.index(&self.pads) == index {
            return Ok(false);
        }
        let initial = if created || self.pads.pattern_rows.is_none() {
            Some(self.pads.prepare_pattern_record()?)
        } else {
            None
        };
        let mut pattern = self
            .pads
            .pattern
            .clone()
            .ok_or("Pattern setting needs A1.")?;
        let command = if setting.index(&self.pads) != index {
            setting.set(&mut pattern, index);
            let rows = self
                .pads
                .pattern_rows
                .clone()
                .ok_or("Pattern setting needs row identities.")?;
            self.pads.restore_recorded_pattern(&pattern, &rows)?
        } else {
            initial.ok_or("Pattern setting has no publication.")?
        };
        let command = match (boundary, command) {
            (Some(state), Command::PublishRecordedPattern { pattern })
                if state.phase == Phase::Recording =>
            {
                Command::PublishCapturedPattern {
                    pattern,
                    pass: state.pass,
                    sequence: state.sequence,
                }
            }
            (_, command) => command,
        };
        if self.audio_alive().is_some() {
            self.send(command);
        }
        self.dirty();
        Ok(true)
    }

    pub(super) fn pattern_knob(
        &mut self,
        setting: Setting,
        port: &str,
        control: &str,
        value: f32,
        takeover: bool,
    ) {
        if !value.is_finite() || self.quit_requested || self.transport_input_blocked() {
            return;
        }
        self.flush_audio();
        let boundary = match self.pattern_edit_boundary(setting) {
            Ok(boundary) => boundary,
            Err(text) => {
                self.pattern_notice(text);
                return;
            }
        };
        let value = value.clamp(0.0, 1.0);
        let value = if takeover {
            self.takeover.apply(
                port,
                control,
                setting.target(),
                value,
                setting.position(&self.pads),
            )
        } else {
            Ok(value)
        };
        match value {
            Ok(value) => {
                self.pickup = None;
                if let Err(error) = self.apply_pattern_setting(setting, value, boundary) {
                    self.post_notice(crate::Level::Error, (error, std::time::Instant::now()));
                }
            }
            Err(direction) => {
                let way = if direction == input::midi::Move::Up {
                    "up ↑"
                } else {
                    "down ↓"
                };
                self.post_notice(
                    crate::Level::Info,
                    (
                        format!(
                            "A1 {}: turn {control} {way} to {}",
                            setting.label(),
                            setting.text(&self.pads)
                        ),
                        std::time::Instant::now(),
                    ),
                );
            }
        }
    }

    pub(super) fn pattern_routes_pads(&self) -> bool {
        self.pattern_panel.is_some() || self.pattern_owns_pads()
    }

    pub(super) fn pattern_routes_pads_for(&self, state: Option<Snapshot>) -> bool {
        self.pattern_panel.is_some() || self.pattern_owns_pads_for(state)
    }

    pub(super) fn pattern_owns_pads(&self) -> bool {
        self.pattern_owns_pads_for(self.pattern_record_state())
    }

    fn pattern_owns_pads_for(&self, state: Option<Snapshot>) -> bool {
        self.audio.as_ref().is_some_and(|audio| {
            let capture = &audio.pattern_capture;
            state.map_or_else(
                || capture.blocks_autosave(),
                |s| {
                    s.phase != Phase::Idle
                        || capture.requested.is_some()
                        || capture.dirty
                        || capture.undo_pending.is_some()
                },
            )
        })
    }

    pub(super) fn pattern_record_state(&self) -> Option<Snapshot> {
        use std::sync::atomic::Ordering::Acquire;
        let audio = self.audio_alive()?;
        let epoch = audio.project.epoch.load(Acquire);
        if audio.project.ready.load(Acquire) != epoch {
            return None;
        }
        let snapshot = audio
            .stats
            .recording(epoch)
            .or_else(|| audio.pattern_capture.last.filter(|s| s.epoch == epoch));
        (audio.project.epoch.load(Acquire) == epoch && audio.project.ready.load(Acquire) == epoch)
            .then_some(snapshot)
            .flatten()
    }

    fn pattern_notice(&mut self, text: &str) {
        self.post_notice(crate::Level::Warn, (text.into(), std::time::Instant::now()));
    }

    pub(super) fn finish_pattern_record(&mut self) -> bool {
        if !self.pattern_owns_pads() {
            return false;
        }
        if let Some(audio) = self.audio.as_ref()
            && !audio.pattern_capture.end_requested
            && let Some(command) = audio.pattern_capture.finish_for_save()
        {
            self.send(command);
        }
        true
    }

    pub(super) fn pattern_end_requested(&mut self, pass: u64) {
        if let Some(audio) = self.audio.as_mut()
            && audio
                .pattern_capture
                .finish_for_save()
                .is_some_and(|c| matches!(c, Command::PatternRecordEnd { pass: p } if p == pass))
        {
            audio.pattern_capture.end_requested = true;
        }
    }

    pub(super) fn toggle_pattern_record(&mut self) {
        if self.transport_input_blocked() {
            if self.plan.is_some() {
                self.pattern_notice("Pattern recording blocked: finish Chop assignment first.");
            }
            return;
        }
        self.flush_audio();
        if self.finish_pattern_record() {
            return;
        }
        if let Some((source, _)) = self.recording {
            self.refuse_other_recording(source);
            return;
        }
        if self.browser.in_hand.is_some()
            || self.copy_in_hand.is_some()
            || self.pad_in_hand.is_some()
            || self.pads.marking
            || !self.pads.marked.is_empty()
            || self.plan.is_some()
        {
            self.pattern_notice(
                "Pattern recording blocked: finish placement, marking or Chop assignment first.",
            );
            return;
        }
        let Some(state) = self.pattern_record_state() else {
            self.pattern_notice(
                "Pattern recording unavailable: wait for the project and audio to be ready.",
            );
            return;
        };
        let Some(pass) = state.pass.checked_add(1) else {
            self.pattern_notice(
                "Pattern recording unavailable: reopen the project before another pass.",
            );
            return;
        };
        if self.try_send(Command::PatternRecordStart { pass }) {
            // a pass starts with ordinary recording, never with whole-pad erasure
            self.erase_pads = false;
        } else {
            self.pattern_notice(
                "Pattern recording not started: wait for pending audio work, then try again.",
            );
        }
    }

    pub(super) fn toggle_pattern_erase(&mut self) {
        if self.transport_input_blocked() {
            return;
        }
        self.flush_audio();
        if self.erase_pads {
            self.erase_pads = false;
            return;
        }
        let phase = self.pattern_record_state().map(|s| s.phase);
        if matches!(phase, None | Some(Phase::Idle)) && self.pattern_tidy_refusal().is_none() {
            self.start_whole_pad_erasure();
            return;
        }
        let Some(state) = self
            .pattern_record_state()
            .filter(|s| s.phase == Phase::Recording)
        else {
            self.pattern_notice(
                "Pattern erasure needs Recording: start A1 recording and complete count-in first.",
            );
            return;
        };
        let capture = &self.audio.as_ref().unwrap().pattern_capture;
        if capture.end_requested {
            self.pattern_notice("Pattern erasure unavailable: the pass is finishing.");
            return;
        }
        let enabled = !capture.erase_requested.unwrap_or(state.erasing);
        if self.try_send(Command::PatternErase {
            pass: state.pass,
            enabled,
        }) {
            self.audio.as_mut().unwrap().pattern_capture.erase_requested = Some(enabled);
        }
    }

    /// Held erasure of the current pass, as requested or as the audio thread reports it.
    pub(super) fn pattern_erasing(&self) -> bool {
        let Some(state) = self
            .pattern_record_state()
            .filter(|s| s.phase == Phase::Recording)
        else {
            return false;
        };
        let capture = &self.audio.as_ref().unwrap().pattern_capture;
        !capture.end_requested && capture.erase_requested.unwrap_or(state.erasing)
    }

    /// Outside a pass, erasure is a mode of Pads: the counts show only there (98b).
    fn start_whole_pad_erasure(&mut self) {
        let now = std::time::Instant::now();
        if self.screen != crate::Screen::Pads {
            self.post_notice(crate::Level::Info, ("Erase pads on Pads (F2).".into(), now));
            return;
        }
        if self
            .pads
            .pattern
            .as_ref()
            .is_none_or(|pattern| pattern.hits.is_empty())
        {
            self.post_notice(crate::Level::Info, ("A1 has no hits to erase.".into(), now));
            return;
        }
        self.pattern_panel = None;
        self.erase_pads = true;
    }

    /// The raw A1 rows of each absolute pad, hidden rows too: what a press erases.
    pub(super) fn pattern_pad_hits(&self) -> [u16; engine::PADS] {
        let mut counts = [0u16; engine::PADS];
        if let Some(pattern) = &self.pads.pattern {
            for hit in &pattern.hits {
                if let Some(count) = counts.get_mut(hit[1] as usize) {
                    *count = count.saturating_add(1);
                }
            }
        }
        counts
    }

    /// A press in whole-pad erasure: every A1 row of the pad goes, one Undo step (98b).
    pub(super) fn erase_whole_pad(&mut self, abs: usize) {
        self.flush_audio();
        if let Some(text) = self.pattern_tidy_refusal() {
            self.erase_pads = false;
            self.pattern_notice(text);
            return;
        }
        let name = crate::pad_label(&self.pads, abs);
        let step = format!(
            "erase pad {}",
            crate::pads::PAD_KEYS[abs % crate::PADS_PER_BANK].to_ascii_uppercase()
        );
        let result = self.tidy_pattern(&step, |pattern, rows| {
            let mut removed = [false; MAX_HITS];
            for (hit, row) in pattern.hits.iter().zip(rows) {
                removed[row.id().slot()] = hit[1] as usize == abs;
            }
            Ok(removed)
        });
        let count = *result.as_ref().unwrap_or(&0);
        self.tidy_notice(
            result,
            format!("A1: {name} erased, {}. Ctrl+Z restores.", hits(count)),
            &format!("A1: {name} has no hits."),
        );
    }

    pub(super) fn keyboard_erase_target(&self, pad: usize) -> Option<Command> {
        if self.pads.track_keys {
            return None;
        }
        let state = self
            .pattern_record_state()
            .filter(|s| s.phase == Phase::Recording)?;
        let capture = &self.audio.as_ref()?.pattern_capture;
        (!capture.end_requested && capture.erase_requested.unwrap_or(state.erasing)).then_some(
            Command::PatternEraseTarget {
                pass: state.pass,
                pad,
                one_loop: true,
            },
        )
    }

    fn refuse_other_recording(&mut self, source: u8) {
        let text = if source == crate::audio::SOURCE_MIC {
            "Microphone recording is active: finish it before starting another recording target."
        } else {
            "Output recording is active: finish it before starting another recording target."
        };
        self.produce_notice(
            crate::Notice::recording_refusal(self.notice_origin.clone(), text),
            std::time::Instant::now(),
        );
    }

    pub(super) fn audio_record_action(&mut self, action: input::Action) {
        match crate::record_step(action, self.recording.map(|(source, _)| source)) {
            Some(crate::Rec::Start(source)) => self.record_start(source),
            Some(crate::Rec::Stop) => self.record_stop(),
            None if matches!(
                action,
                input::Action::RecordMic(true) | input::Action::RecordOutputToggle
            ) =>
            {
                if let Some((source, _)) = self.recording {
                    self.refuse_other_recording(source);
                }
            }
            None => {}
        }
    }

    pub(super) fn refuse_audio_during_pattern(&mut self) -> bool {
        self.flush_audio();
        if !self.pattern_owns_pads() {
            return false;
        }
        let phase = self
            .audio
            .as_ref()
            .and_then(|audio| audio.pattern_capture.blocking_phase())
            .unwrap_or("finishing");
        self.pattern_notice(&format!(
            "A1 pattern {phase} is active: finish the pattern pass before audio recording."
        ));
        true
    }

    pub(super) fn request_active_pattern_undo(&mut self) -> bool {
        let Some(audio) = self.audio.as_mut() else {
            return false;
        };
        if !audio.pattern_capture.request_undo() {
            return false;
        }
        if !audio.stopped.load(std::sync::atomic::Ordering::Relaxed)
            && let Some(command) = audio.pattern_capture.finish_for_save()
        {
            self.send(command);
        }
        self.flush_audio();
        true
    }

    pub(super) fn undo_pattern(&mut self) {
        if self.request_active_pattern_undo() {
            return;
        }
        self.flush_audio();
        if self
            .history
            .last()
            .is_some_and(|step| step.pattern.is_some())
        {
            self.undo_pads();
        } else {
            let text = if self.history.is_empty() {
                "No pattern pass to undo. No redo is available."
            } else {
                "Pattern Undo blocked: undo the newer pad or FX edit with normal Undo first."
            };
            self.post_notice(crate::Level::Warn, (text.into(), std::time::Instant::now()));
        }
    }

    /// Tidy commands leave A1 alone while a pass owns it.
    pub(super) fn pattern_tidy_refusal(&self) -> Option<&'static str> {
        let refusal = "A1 edit blocked: end the recording pass first.";
        if self
            .pattern_record_state()
            .is_some_and(|state| state.phase != Phase::Idle)
        {
            return Some(refusal);
        }
        match self.pattern_edit_boundary(Setting::Quantize) {
            Err(_) => Some(refusal),
            Ok(Some(state)) if state.phase != Phase::Idle => Some(refusal),
            Ok(_) => None,
        }
    }

    /// Check the complete replacement before changing A1 or its Undo history.
    pub(super) fn remove_step_event(
        &mut self,
        event: crate::pattern_view::Event,
        expected: &crate::session::PatternState,
    ) -> Result<bool, String> {
        self.flush_audio();
        if let Some(text) = self.pattern_tidy_refusal() {
            return Err(text.into());
        }
        if self.pads.pattern.as_ref() != Some(expected) {
            return Ok(false);
        }
        let rows = match self.pads.pattern_rows.as_ref() {
            Some(rows) => rows.clone(),
            None => (0..expected.hits.len())
                .map(|slot| {
                    RowId::new(slot, 1)
                        .map(PlaybackRow::existing)
                        .map_err(|error| error.to_string())
                })
                .collect::<Result<Vec<_>, _>>()?,
        };
        if rows.get(event.source).map(|row| row.id()) != Some(event.id)
            || expected.hits.get(event.source).copied()
                != Some([
                    event.raw.tick(),
                    event.raw.pad() as u32,
                    event.raw.velocity() as u32,
                    event.raw.gate_ticks(),
                ])
        {
            return Ok(false);
        }
        let mut replacement = expected.clone();
        let mut replacement_rows = rows.clone();
        replacement.hits.remove(event.source);
        replacement_rows.remove(event.source);
        replacement
            .checked()?
            .compile_recorded(&replacement_rows)
            .map_err(|error| error.to_string())?;
        let before = Undo {
            pattern: expected.clone(),
            rows,
        };
        let command = self
            .pads
            .restore_recorded_pattern(&replacement, &replacement_rows)?;
        if self.audio_alive().is_some() {
            self.send(command);
        }
        self.push_pattern_step_named(before, "remove one A1 hit");
        self.dirty();
        Ok(true)
    }

    /// Removes the rows `pick` marks as one common-history step and publishes the rest.
    /// Returns how many rows went; none leaves history and the project unchanged.
    fn tidy_pattern(
        &mut self,
        what: &str,
        pick: impl Fn(&crate::session::PatternState, &[PlaybackRow]) -> Result<[bool; MAX_HITS], String>,
    ) -> Result<usize, String> {
        if self.pads.pattern_rows.is_none() {
            // loaded rows get identities first, as recording and settings give them
            let command = self.pads.prepare_pattern_record()?;
            if self.audio_alive().is_some() {
                self.send(command);
            }
        }
        let before = Undo::new(&self.pads)?;
        let pattern = self.pads.pattern.as_mut().ok_or("A1 edit needs A1.")?;
        let rows = self
            .pads
            .pattern_rows
            .as_mut()
            .ok_or("A1 edit needs row identities.")?;
        let removed = pick(pattern, rows)?;
        let count = rows.iter().filter(|row| removed[row.id().slot()]).count();
        if count == 0 {
            return Ok(0);
        }
        remove_rows(pattern, rows, &removed);
        let command = self.pads.recorded_pattern_command()?;
        if self.audio_alive().is_some() {
            self.send(command);
        }
        self.push_pattern_step_named(before, what);
        self.dirty();
        Ok(count)
    }

    fn tidy_notice(&mut self, result: Result<usize, String>, done: String, nothing: &str) {
        let now = std::time::Instant::now();
        match result {
            Ok(0) => self.post_notice(crate::Level::Info, (nothing.into(), now)),
            Ok(_) => self.post_notice(crate::Level::Done, (done, now)),
            Err(error) => self.post_notice(crate::Level::Error, (error, now)),
        }
    }

    /// `pattern.clear`: asks before every hit goes (98b).
    pub(super) fn ask_clear_pattern(&mut self) {
        self.flush_audio();
        if let Some(text) = self.pattern_tidy_refusal() {
            self.pattern_notice(text);
            return;
        }
        let hits_count = self
            .pads
            .pattern
            .as_ref()
            .map_or(0, |pattern| pattern.hits.len());
        if hits_count == 0 {
            self.post_notice(
                crate::Level::Info,
                ("A1 has no hits.".into(), std::time::Instant::now()),
            );
            return;
        }
        self.ask = Some(crate::Ask::ClearPattern {
            lines: vec![
                format!("Clear all {} of A1?", hits(hits_count)),
                "Bars, quantize and strength stay. Ctrl+Z restores.".into(),
            ],
        });
    }

    pub(super) fn clear_pattern_confirmed(&mut self) {
        self.flush_audio();
        if let Some(text) = self.pattern_tidy_refusal() {
            self.pattern_notice(text);
            return;
        }
        if self.pads.pattern.is_none() {
            return;
        }
        let result = self.tidy_pattern("clear", |_, _| Ok([true; MAX_HITS]));
        let count = *result.as_ref().unwrap_or(&0);
        self.tidy_notice(
            result,
            format!("A1 cleared: {}.", hits(count)),
            "A1 has no hits.",
        );
    }
}

/// `1 hit`, `3 hits`: a count of hits as the screen says it (98i).
pub(super) fn hits(count: usize) -> String {
    if count == 1 {
        "1 hit".into()
    } else {
        format!("{count} hits")
    }
}

/// The rows a completed pass replaces. Rows of one pad that compile to one playback tick
/// keep only the most recently captured fresh row. A group without a fresh row, such as
/// saved duplicates, stays; hidden rows are not compiled and stay too.
fn replaced_rows(
    pattern: &crate::session::PatternState,
    rows: &[PlaybackRow],
    fresh: &[RowId],
) -> Result<[bool; MAX_HITS], String> {
    let compiled = pattern
        .checked()?
        .compile_recorded(rows)
        .map_err(|error| error.to_string())?;
    let order = |id: RowId| fresh.iter().position(|&f| f == id);
    let mut keep: Vec<((usize, u32), RowId)> = Vec::new();
    let mut groups: Vec<((usize, u32), RowId)> = Vec::new();
    for (hit, row) in compiled
        .events()
        .iter()
        .zip(compiled.playback_rows().unwrap_or_default())
    {
        let key = (hit.pad(), hit.tick());
        groups.push((key, row.id()));
        if let Some(rank) = order(row.id()) {
            match keep.iter_mut().find(|(k, _)| *k == key) {
                Some(kept) if order(kept.1) < Some(rank) => kept.1 = row.id(),
                Some(_) => {}
                None => keep.push((key, row.id())),
            }
        }
    }
    let mut removed = [false; MAX_HITS];
    for (key, id) in groups {
        if let Some((_, kept)) = keep.iter().find(|(k, _)| *k == key)
            && *kept != id
        {
            removed[id.slot()] = true;
        }
    }
    Ok(removed)
}

fn remove_rows(
    pattern: &mut crate::session::PatternState,
    rows: &mut Vec<PlaybackRow>,
    removed: &[bool; MAX_HITS],
) -> bool {
    let before = rows.len();
    let mut index = 0;
    pattern.hits.retain(|_| {
        let keep = !removed[rows[index].id().slot()];
        index += 1;
        keep
    });
    rows.retain(|row| !removed[row.id().slot()]);
    rows.len() != before
}

fn erase_rows(
    pattern: &mut crate::session::PatternState,
    rows: &mut Vec<PlaybackRow>,
    intervals: &[EraseInterval],
    boundary: Snapshot,
) -> Result<bool, String> {
    if intervals.is_empty() || rows.is_empty() {
        return Ok(false);
    }
    let raw = pattern.checked()?;
    let mut compiled = Vec::new();
    let mut removed = [false; MAX_HITS];
    for interval in intervals {
        let key = (interval.loop_ticks, interval.grid, interval.strength);
        let index = match compiled.iter().position(|(meta, _)| *meta == key) {
            Some(index) => index,
            None => {
                // saved ticks are placed already: the interval grid must not move them again
                let raw = RawPattern::saved(
                    (interval.loop_ticks / TICKS_PER_BAR) as u8,
                    interval.grid,
                    interval.strength,
                    raw.hits().to_vec(),
                )
                .map_err(|e| e.to_string())?;
                let candidate = raw.compile_recorded(rows).map_err(|e| e.to_string())?;
                compiled.push((key, candidate));
                compiled.len() - 1
            }
        };
        let candidate = &compiled[index].1;
        for (hit, row) in candidate
            .events()
            .iter()
            .zip(candidate.playback_rows().unwrap())
        {
            if hit.pad() != interval.pad
                || (interval.complete && row.born_after(interval.pass, interval.closed_sequence))
            {
                continue;
            }
            let start = interval
                .start
                .max(row.birth_in(interval.run).unwrap_or(0.0));
            let lap = ((start - f64::from(hit.tick()) - 1e-9) / f64::from(interval.loop_ticks))
                .ceil()
                .max(0.0);
            let occurrence = f64::from(hit.tick()) + lap * f64::from(interval.loop_ticks);
            let limit = boundary.ticks
                + if interval.complete {
                    boundary.frame_ticks * 0.5 + 1e-9
                } else {
                    0.0
                };
            if occurrence + 1e-9 >= start
                && occurrence + 1e-9 < interval.end
                && occurrence + 1e-9 < limit
            {
                removed[row.id().slot()] = true;
            }
        }
    }
    Ok(remove_rows(pattern, rows, &removed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_lifecycle_exit_defers_an_unstable_live_snapshot_but_not_a_lost_unaccepted_start() {
        let mut engine = Engine::with_sounds(48_000, vec![], vec![]);
        let mut capture = Capture::new(&mut engine);
        let mut pads = Pads::default();
        pads.prepare_pattern_record().unwrap();
        capture.begin(1, 1, &pads).unwrap();
        let update = capture.poll_for_exit(1, None, false, &mut pads).unwrap();
        assert!(
            capture.blocks_autosave(),
            "a failed live snapshot read is not proof that no start was accepted"
        );
        assert!(update.commands.is_empty());
        let update = capture.poll_for_exit(1, None, true, &mut pads).unwrap();
        assert!(!capture.blocks_autosave());
        assert!(update.commands.is_empty() && update.undo.is_none());
        assert!(pads.pattern.as_ref().unwrap().hits.is_empty());
    }

    #[test]
    fn pattern_autosave_lost_unaccepted_request_with_no_snapshot_does_not_block() {
        let mut engine = Engine::with_sounds(48_000, vec![], vec![]);
        let mut capture = Capture::new(&mut engine);
        let mut pads = Pads::default();
        pads.prepare_pattern_record().unwrap();
        capture.begin(1, 1, &pads).unwrap();
        assert!(capture.blocks_autosave());
        let update = capture.poll(1, None, true, &mut pads).unwrap();
        assert!(!capture.blocks_autosave());
        assert!(!update.changed);
        assert!(update.commands.is_empty());
        assert!(update.undo.is_none());
    }

    fn erasure_fixture() -> (Engine, Capture, Pads) {
        let mut engine = Engine::with_sounds(48_000, vec![], vec![]);
        let mut capture = Capture::new(&mut engine);
        let mut pads = Pads::default();
        pads.prepare_pattern_record().unwrap();
        let pattern = pads.pattern.as_mut().unwrap();
        pattern.bars = 1;
        pattern.quantize = "off".into();
        pattern.hits = vec![[120, 0, 90, 0], [120, 1, 80, 0], [5000, 0, 70, 0]];
        pads.pattern_rows = None;
        engine.handle(0, pads.prepare_pattern_record().unwrap());
        capture.begin(0, 1, &pads).unwrap();
        engine.handle(0, Command::TransportPlay);
        engine.handle(0, Command::PatternRecordStart { pass: 1 });
        engine.handle(
            0,
            Command::PatternErase {
                pass: 1,
                enabled: true,
            },
        );
        (engine, capture, pads)
    }

    struct UnavailableFeedback<'a> {
        recovery: &'a Recovery,
        interval: bool,
        removal: bool,
    }

    impl Feedback for UnavailableFeedback<'_> {
        fn erased(&self, id: RowId, boundary: Snapshot) -> Option<bool> {
            if self.removal && id.slot() == 0 {
                None
            } else {
                self.recovery.erased(id, boundary)
            }
        }

        fn erase_interval(&self, slot: usize, boundary: Snapshot) -> Option<EraseInterval> {
            if self.interval && slot == 0 {
                None
            } else {
                self.recovery.erase_interval(slot, boundary)
            }
        }

        fn completed(&self, onset: Event) -> Option<engine::pattern::Hit> {
            self.recovery.completed(onset)
        }

        fn interrupted(&self, onset: Event, boundary: Snapshot) -> Option<engine::pattern::Hit> {
            self.recovery.interrupted(onset, boundary)
        }
    }

    #[test]
    fn app_erasure_unavailable_feedback_defers_consumption_confirmation_and_acknowledgement() {
        for interval in [false, true] {
            for stopped in [false, true] {
                let (mut engine, mut capture, mut pads) = erasure_fixture();
                let original = pads.pattern.clone();
                engine.handle(
                    0,
                    Command::PatternEraseTarget {
                        pass: 1,
                        pad: 0,
                        one_loop: false,
                    },
                );
                for _ in 0..32 {
                    engine.process(&mut [0.0; 128], &mut [0.0; 128]);
                }
                engine.handle(0, Command::PatternRecordEnd { pass: 1 });
                let boundary = engine.recording();
                assert_eq!((boundary.erase_intervals, boundary.erased_rows), (1, 1));
                let recovery = capture.recovery.clone();
                let unavailable = UnavailableFeedback {
                    recovery: &recovery,
                    interval,
                    removal: !interval,
                };
                let first = capture
                    .poll_feedback(0, Some(boundary), stopped, &mut pads, &unavailable)
                    .unwrap();
                assert!(first.undo.is_none());
                assert!(capture.blocks_autosave());
                assert!(
                    !first.commands.iter().any(|c| matches!(
                        c,
                        Command::PatternEraseConsumed { .. }
                            | Command::PatternRecordRows { .. }
                            | Command::PatternRecordAck { .. }
                    )),
                    "incomplete feedback must not release prefix ownership"
                );
                assert_eq!(capture.pass.as_ref().unwrap().complete_intervals, 0);
                let retry = capture.poll(0, Some(boundary), stopped, &mut pads).unwrap();
                assert_eq!(
                    pads.pattern.as_ref().unwrap().hits,
                    [[120, 1, 80, 0], [5000, 0, 70, 0]]
                );
                let undo = retry.undo.expect("exactly one completed pass");
                assert_eq!(Some(undo.pattern), original);
                let commands: Vec<_> = first
                    .commands
                    .iter()
                    .chain(&retry.commands)
                    .filter_map(|c| match c {
                        Command::PublishCapturedPattern { .. } => Some("publish"),
                        Command::PatternEraseConsumed { .. } => Some("consume"),
                        Command::PatternRecordRows { .. } => Some("rows"),
                        Command::PatternRecordAck { .. } => Some("ack"),
                        _ => None,
                    })
                    .collect();
                if stopped {
                    assert!(
                        commands.is_empty(),
                        "closed capture completes without audio commands"
                    );
                    assert!(!capture.blocks_autosave());
                } else {
                    assert_eq!(commands, ["publish", "consume", "rows", "ack"]);
                }
                let repeated = capture.poll(0, Some(boundary), stopped, &mut pads).unwrap();
                assert!(repeated.commands.is_empty() && repeated.undo.is_none());
            }
        }
    }

    #[test]
    fn app_erasure_concurrent_interval_closure_retains_the_complete_accepted_prefix() {
        let (mut engine, mut capture, mut pads) = erasure_fixture();
        let original = pads.pattern.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            for _ in 0..64 {
                engine.handle(
                    0,
                    Command::PatternEraseTarget {
                        pass: 1,
                        pad: 0,
                        one_loop: false,
                    },
                );
                engine.process(&mut [0.0; 128], &mut [0.0; 128]);
                sender.send(engine.recording()).unwrap();
                engine.handle(0, Command::PadRelease { pad: 0 });
            }
            engine.handle(0, Command::PatternRecordEnd { pass: 1 });
            sender.send(engine.recording()).unwrap();
            engine
        });
        let mut queued = Vec::new();
        let mut undo = None;
        let mut last = Snapshot::default();
        for boundary in receiver {
            let update = capture.poll(0, Some(boundary), false, &mut pads).unwrap();
            for command in &update.commands {
                match command {
                    Command::PatternEraseConsumed { pass, sequence, .. } => {
                        assert_eq!(*pass, boundary.pass);
                        assert!(*sequence <= boundary.sequence);
                    }
                    Command::PatternRecordAck { pass, sequence } => {
                        assert_eq!((*pass, *sequence), (boundary.pass, boundary.sequence));
                        assert_eq!(boundary.phase, Phase::Finishing);
                        assert_eq!(
                            capture.pass.as_ref().unwrap().complete_intervals,
                            boundary.erase_intervals
                        );
                    }
                    _ => {}
                }
            }
            if let Some(before) = update.undo {
                assert!(undo.replace(before).is_none(), "one history step per pass");
            }
            queued.extend(update.commands);
            last = boundary;
        }
        let mut engine = worker.join().unwrap();
        // Retry the stable terminal prefix without another audio callback.
        let retry = capture.poll(0, Some(last), false, &mut pads).unwrap();
        if let Some(before) = retry.undo {
            assert!(undo.replace(before).is_none());
        }
        queued.extend(retry.commands);
        assert_eq!((last.erase_intervals, last.erased_rows), (64, 1));
        assert_eq!(
            pads.pattern.as_ref().unwrap().hits,
            [[120, 1, 80, 0], [5000, 0, 70, 0]]
        );
        assert_eq!(Some(undo.unwrap().pattern), original);
        assert_eq!(
            queued
                .iter()
                .filter(|c| matches!(c, Command::PatternEraseConsumed { .. }))
                .count(),
            64
        );
        assert_eq!(
            queued
                .iter()
                .filter(|c| matches!(c, Command::PatternRecordAck { .. }))
                .count(),
            1
        );
        for command in queued {
            engine.handle(0, command);
        }
        assert_eq!(engine.recording().phase, Phase::Idle);
        let repeated = capture
            .poll(0, Some(engine.recording()), false, &mut pads)
            .unwrap();
        assert!(repeated.commands.is_empty() && repeated.undo.is_none());
    }

    #[test]
    fn invalid_pass_undo_compiles_before_changing_the_model() {
        let mut pads = Pads::default();
        pads.prepare_pattern_record().unwrap();
        let mut before = Undo::new(&pads).unwrap();
        before.pattern.strength = 101;
        let pattern = pads.pattern.clone();
        let rows = pads.pattern_rows.clone();
        assert_eq!(
            before.restore(&mut pads).unwrap_err(),
            "Pattern strength must be within 0..100 in steps of 10."
        );
        assert_eq!(pads.pattern, pattern);
        assert_eq!(pads.pattern_rows, rows);
    }
}

//! Bounded pattern cursor and note-owned musical deadlines.

use std::sync::Arc;

use crate::pattern::{Hit, MAX_HITS, Pattern, PlaybackRow};

const RELEASES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReleaseKind {
    Gate(u32),
    Cut,
}

#[derive(Clone, Copy)]
pub(super) struct Release {
    pub tick: f64,
    pub note: u64,
    pub kind: ReleaseKind,
}

#[derive(Clone, Copy)]
struct Consumed {
    generation: u64,
    lap: u64,
}

pub(super) struct Sequencer {
    pub pattern: Option<Arc<Pattern>>,
    index: usize,
    lap: u64,
    run: u64,
    consumed: Box<[Option<Consumed>]>,
    releases: [Option<Release>; RELEASES],
    pub dropped: u64,
}

impl Default for Sequencer {
    fn default() -> Self {
        Self {
            pattern: None,
            index: 0,
            lap: 0,
            run: 0,
            consumed: vec![None; MAX_HITS].into_boxed_slice(),
            releases: [None; RELEASES],
            dropped: 0,
        }
    }
}

impl Sequencer {
    pub fn reset(&mut self) {
        self.index = 0;
        self.lap = 0;
        self.consumed.fill(None);
        self.clear_notes();
    }

    pub fn restart(&mut self) -> u64 {
        self.reset();
        self.run = self.run.wrapping_add(1).max(1);
        self.run
    }

    fn clear_notes(&mut self) {
        self.releases.fill(None);
    }

    fn consume(history: &mut [Option<Consumed>], row: PlaybackRow, lap: u64) {
        let previous = &mut history[row.id.slot];
        if previous.is_none_or(|p| p.generation != row.id.generation || p.lap < lap) {
            *previous = Some(Consumed {
                generation: row.id.generation,
                lap,
            });
        }
    }

    fn eligible(&self, row: PlaybackRow) -> bool {
        !row.pending
            && row
                .origin
                .is_none_or(|o| o.run != self.run || self.lap > o.lap)
            && self.consumed[row.id.slot]
                .is_none_or(|p| p.generation != row.id.generation || p.lap < self.lap)
    }

    /// Exclude events assigned to frames before this replacement boundary.
    pub fn seek(&mut self, ticks: f64, step: f64) {
        self.index = 0;
        self.lap = 0;
        let Some(pattern) = &self.pattern else { return };
        let lower = ticks - step * 0.5 + 1e-9;
        let length = f64::from(pattern.loop_ticks());
        self.lap = (lower.max(0.0) / length).floor() as u64;
        let base = self.lap as f64 * length;
        if let Some(rows) = &pattern.rows {
            for (hit, row) in pattern.events().iter().zip(rows.iter()) {
                if base + f64::from(hit.tick()) <= lower {
                    Self::consume(&mut self.consumed, *row, self.lap);
                }
            }
        }
        self.index = pattern
            .events()
            .partition_point(|hit| base + f64::from(hit.tick()) <= lower);
        if self.index == pattern.events().len() {
            self.index = 0;
            self.lap += 1;
        }
    }

    fn group_end(&self, pattern: &Pattern) -> usize {
        let hit = pattern.events()[self.index];
        self.index
            + pattern.events()[self.index..].partition_point(|candidate| {
                candidate.tick() == hit.tick() && candidate.pad() == hit.pad()
            })
    }

    pub fn next_hit(&self, capture: &crate::capture::Recorder) -> Option<(f64, Option<Hit>)> {
        let pattern = self.pattern.as_ref()?;
        let first = *pattern.events().get(self.index)?;
        let hit = if let Some(rows) = &pattern.rows {
            (self.index..self.group_end(pattern))
                .find(|&i| self.eligible(rows[i]) && !capture.removed(rows[i].id))
                .map(|i| pattern.events()[i])
        } else {
            Some(first)
        };
        Some((
            self.lap as f64 * f64::from(pattern.loop_ticks()) + f64::from(first.tick()),
            hit,
        ))
    }

    pub fn advance(&mut self) {
        if let Some(pattern) = &self.pattern {
            if let Some(rows) = &pattern.rows {
                let end = self.group_end(pattern);
                for row in &rows[self.index..end] {
                    Self::consume(&mut self.consumed, *row, self.lap);
                }
                self.index = end;
            } else {
                self.index += 1;
            }
        }
        if self
            .pattern
            .as_ref()
            .is_some_and(|p| self.index == p.events().len())
        {
            self.index = 0;
            self.lap += 1;
        }
    }

    pub fn earliest(&self) -> Option<usize> {
        self.releases
            .iter()
            .enumerate()
            .filter_map(|(i, r)| r.map(|r| (i, r)))
            .min_by(|(_, a), (_, b)| a.tick.total_cmp(&b.tick).then(a.note.cmp(&b.note)))
            .map(|(i, _)| i)
    }

    pub fn next_release(&self) -> Option<Release> {
        self.releases[self.earliest()?]
    }

    pub fn take_release(&mut self) -> Option<Release> {
        let i = self.earliest()?;
        self.releases[i].take()
    }

    pub fn forget(&mut self, note: u64) {
        for entry in &mut self.releases {
            if entry.is_some_and(|r| r.note == note) {
                *entry = None;
            }
        }
    }

    pub fn retain(&mut self, mut live: impl FnMut(u64) -> bool) {
        for entry in &mut self.releases {
            if entry.is_some_and(|r| !live(r.note)) {
                *entry = None;
            }
        }
    }

    /// The caller applies a returned oldest release before inserting another deadline.
    pub fn make_room(&mut self) -> Option<Release> {
        if self.releases.iter().all(Option::is_some) {
            self.take_release()
        } else {
            None
        }
    }

    pub fn insert(&mut self, release: Release) {
        if let Some(entry) = self.releases.iter_mut().find(|r| r.is_none()) {
            *entry = Some(release);
        }
    }
}

impl crate::Engine {
    pub(super) fn tick_step(&self) -> f64 {
        self.transport.state.bpm * crate::transport::TICKS_PER_QUARTER
            / (60.0 * f64::from(self.sample_rate))
    }

    fn release_voice(voice: &mut crate::Voice, kind: ReleaseKind, short: u32) {
        match kind {
            ReleaseKind::Gate(frames) if voice.fade.is_none() => {
                let frames = if frames == 0 { short } else { frames };
                voice.fade = Some(frames);
                voice.fade_total = frames;
            }
            ReleaseKind::Cut if voice.fade.is_none_or(|f| f > short) => {
                if let Some(left) = voice.fade {
                    voice.fade_gain *= left as f32 / voice.fade_total.max(1) as f32;
                }
                voice.fade = Some(short);
                voice.fade_total = short;
            }
            _ => {}
        }
    }

    fn apply_release(&mut self, release: Release) {
        if let Some(v) = self
            .voices
            .iter_mut()
            .find(|v| v.started == release.note && v.sequenced && v.sound.is_some())
        {
            Self::release_voice(v, release.kind, self.fade_len);
        }
    }

    fn end_pattern_notes(&mut self) {
        for v in &mut self.voices {
            if !v.sequenced || v.sound.is_none() {
                continue;
            }
            let kind = if v.pitch_gate {
                Some(ReleaseKind::Cut)
            } else {
                v.gate_release.map(ReleaseKind::Gate)
            };
            if let Some(kind) = kind {
                Self::release_voice(v, kind, self.fade_len);
            }
            v.gate_release = None;
            v.pitch_gate = false;
        }
        self.sequencer.clear_notes();
    }

    pub(super) fn stop_pattern(&mut self) {
        self.end_pattern_notes();
        self.sequencer.reset();
        self.transport.stop();
    }

    pub(super) fn replace_pattern(&mut self, pattern: Option<Arc<Pattern>>, release_old: bool) {
        self.capture.published(
            pattern.as_deref(),
            self.sequencer.pattern.as_deref(),
            self.transport.state.ticks,
        );
        if release_old {
            self.end_pattern_notes();
        }
        if let Some(old) = std::mem::replace(&mut self.sequencer.pattern, pattern) {
            super::retire(&mut self.garbage, old);
        }
        self.sequencer
            .seek(self.transport.state.ticks, self.tick_step());
    }

    fn prune_releases(&mut self) {
        let voices = &self.voices;
        self.sequencer.retain(|note| {
            voices
                .iter()
                .any(|v| v.started == note && v.sequenced && v.sound.is_some() && v.fade != Some(0))
        });
    }

    fn add_release(&mut self, release: Release) {
        self.prune_releases();
        if let Some(old) = self.sequencer.make_room() {
            self.apply_release(old);
        }
        self.sequencer.insert(release);
    }

    pub(super) fn pattern_due(&mut self) {
        if !self.transport.state.playing {
            return;
        }
        let due = self.transport.state.ticks + self.tick_step() * 0.5 + 1e-9;
        self.prune_releases();
        while self.sequencer.next_release().is_some_and(|r| r.tick <= due) {
            if let Some(release) = self.sequencer.take_release() {
                self.apply_release(release);
            }
        }
        while let Some((tick, mut hit)) = self
            .sequencer
            .next_hit(&self.capture)
            .filter(|(tick, _)| *tick <= due)
        {
            if let Some(pattern) = &self.sequencer.pattern {
                let pad = pattern.events()[self.sequencer.index].pad();
                if self.capture.has_targets(pad)
                    && let Some(rows) = &pattern.rows
                {
                    for row in &rows[self.sequencer.index..self.sequencer.group_end(pattern)] {
                        if self.capture.targets(
                            pad,
                            tick,
                            *row,
                            (pattern.grid(), pattern.strength()),
                        ) {
                            self.capture.remove(row.id);
                        }
                    }
                    hit = self
                        .sequencer
                        .next_hit(&self.capture)
                        .and_then(|(_, hit)| hit);
                }
            }
            self.sequencer.advance();
            let Some(hit) = hit else { continue };
            let p = &self.pads[hit.pad()];
            if (p.mode == crate::PadMode::Gate && hit.gate_ticks() == 0)
                || p.sound
                    .as_ref()
                    .is_none_or(|s| s.len().min(p.region.end) <= p.region.start)
            {
                continue;
            }
            if !self.can_start_voice() {
                self.sequencer.dropped = self.sequencer.dropped.saturating_add(1);
                continue;
            }
            let gate = (p.mode == crate::PadMode::Gate).then_some(p.release_frames);
            let pg = p.pitch_gate && p.linked_slice && p.rate > 1.0;
            let spacing = self
                .sequencer
                .pattern
                .as_ref()
                .map_or(240, |p| p.grid().gate_spacing());
            if let Some(slot) = self.trigger_pad(hit.pad(), 0, hit.gain()) {
                let v = &mut self.voices[slot];
                v.sequenced = true;
                v.gate_release = gate;
                v.pitch_gate = pg;
                let note = v.started;
                let cut_tick = pg.then(|| {
                    (tick / f64::from(spacing)).floor() * f64::from(spacing) + f64::from(spacing)
                });
                let gate_tick = gate.map(|_| tick + f64::from(hit.gate_ticks()));
                // At a shared deadline the short cut has priority over ordinary release.
                if let Some(release) = gate_tick.filter(|g| cut_tick != Some(*g)) {
                    self.add_release(Release {
                        tick: release,
                        note,
                        kind: ReleaseKind::Gate(gate.unwrap_or(0)),
                    });
                }
                if let Some(release) = cut_tick {
                    self.add_release(Release {
                        tick: release,
                        note,
                        kind: ReleaseKind::Cut,
                    });
                }
            }
        }
        self.capture.expire_targets(self.transport.state.ticks);
        self.capture.prune_targets(due);
    }

    /// Frames before the next boundary; interpolation still uses the original block.
    pub(super) fn pattern_span(&self, remaining: usize) -> usize {
        if !self.transport.state.playing {
            return remaining;
        }
        let hit = self.sequencer.next_hit(&self.capture).map(|(tick, _)| tick);
        let release = self.sequencer.next_release().map(|r| r.tick);
        let next = match (hit, release) {
            (Some(a), Some(b)) => a.min(b),
            (Some(t), None) | (None, Some(t)) => t,
            (None, None) => return remaining,
        };
        let frames = ((next - self.transport.state.ticks - 1e-9) / self.tick_step() - 0.5).ceil();
        (frames.max(1.0) as usize).min(remaining)
    }

    pub fn dropped_pattern_hits(&self) -> u64 {
        self.sequencer.dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::{Grid, Hit, RawPattern};
    use crate::{Command, Engine, PadMode};

    /// Scenario "Pattern release capacity cannot release another owner".
    #[test]
    fn pattern_release_capacity_cannot_release_another_owner() {
        let mut e = Engine::with_sounds(48_000, vec![], vec![vec![0.001; 100_000]]);
        let mut rows = Vec::new();
        for pad in 0..48 {
            e.handle(
                0,
                Command::SetPadMode {
                    pad,
                    mode: PadMode::Gate,
                },
            );
            e.handle(0, Command::SetPadPitch { pad, semitones: 1 });
            e.handle(
                0,
                Command::SetPadPitchGate {
                    pad,
                    enabled: true,
                    linked_slice: true,
                },
            );
            e.handle(
                0,
                Command::SetPadEnvelope {
                    pad,
                    attack_ms: 0,
                    decay_ms: 0,
                    release_ms: 1000,
                },
            );
            rows.push(
                Hit::new(
                    if pad < 30 {
                        0
                    } else if pad < 47 {
                        200
                    } else {
                        201
                    },
                    pad,
                    127,
                    100,
                )
                .unwrap(),
            );
        }
        // One manual voice shares the same callback and must not receive an automatic release.
        e.handle(
            0,
            Command::PadPress {
                pad: 127,
                velocity: 1.0,
            },
        );
        e.handle(
            0,
            Command::SetPattern {
                pattern: Some(Arc::new(
                    RawPattern::new(1, Grid::Quarter, 0, rows)
                        .unwrap()
                        .compile(),
                )),
            },
        );
        e.handle(0, Command::TransportPlay);
        let mut left = vec![0.0; 5025];
        let mut right = vec![0.0; 5025];
        e.process(&mut left, &mut right);
        assert_eq!(e.sequencer.releases.iter().flatten().count(), 64);
        assert!(
            e.voices
                .iter()
                .find(|v| v.pad == Some(30))
                .unwrap()
                .fade
                .is_none()
        );
        e.process(&mut [0.0], &mut [0.0]);
        // Equal earliest deadlines choose the oldest new note, pad 30, not pad 31.
        assert_eq!(
            e.voices.iter().find(|v| v.pad == Some(30)).unwrap().fade,
            Some(47_999)
        );
        assert!(
            e.voices
                .iter()
                .find(|v| v.pad == Some(31))
                .unwrap()
                .fade
                .is_none()
        );
        assert!(
            e.voices
                .iter()
                .find(|v| v.pad == Some(127))
                .unwrap()
                .fade
                .is_none()
        );
        assert_eq!(e.sequencer.releases.iter().flatten().count(), 64);
        assert_eq!(e.dropped_pattern_hits(), 0);
        assert!(
            left.iter()
                .chain(&right)
                .all(|v| v.is_finite() && v.abs() < 1.0)
        );
    }
}

//! Audio-frame clock and bounded publication for the interface.

use std::sync::atomic::{AtomicU64, Ordering::SeqCst};

pub const TICKS_PER_QUARTER: f64 = 960.0;
pub const MIN_BPM: f64 = 40.0;
pub const MAX_BPM: f64 = 300.0;
pub const DEFAULT_BPM: f64 = 120.0;

pub fn tempo(value: f64) -> Option<f64> {
    value
        .is_finite()
        .then(|| (value.clamp(MIN_BPM, MAX_BPM) * 10.0).round() / 10.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Snapshot {
    pub bpm: f64,
    pub playing: bool,
    pub metronome: bool,
    pub ticks: f64,
    /// Zero means no accepted pattern; a present empty pattern still has a length.
    pub pattern_ticks: u32,
    pub dropped_hits: u64,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            bpm: DEFAULT_BPM,
            playing: false,
            metronome: false,
            ticks: 0.0,
            pattern_ticks: 0,
            dropped_hits: 0,
        }
    }
}

impl Snapshot {
    pub fn bar_beat(self) -> (u64, u8) {
        let beat = (self.ticks / TICKS_PER_QUARTER + 1e-9).floor() as u64;
        (beat / 4 + 1, (beat % 4 + 1) as u8)
    }
}

#[derive(Default)]
pub(crate) struct Clock {
    pub state: Snapshot,
    next_beat: u64,
}

impl Clock {
    pub fn play(&mut self) {
        if !self.state.playing {
            self.state.playing = true;
            self.state.ticks = 0.0;
            self.next_beat = 0;
        }
    }

    pub fn stop(&mut self) {
        self.state.playing = false;
        self.state.ticks = 0.0;
        self.next_beat = 0;
    }

    /// Split before this tick's nearest output frame, including a buffer-end boundary.
    pub fn frames_until(&self, ticks: f64, sample_rate: f64) -> usize {
        let step = self.state.bpm * TICKS_PER_QUARTER / (60.0 * sample_rate);
        ((ticks - self.state.ticks - 1e-9) / step - 0.5)
            .ceil()
            .max(0.0) as usize
    }

    /// Returns a beat's accent at its nearest output frame, including the first beat.
    pub fn frame(&mut self, sample_rate: f64) -> Option<f32> {
        if !self.state.playing {
            return None;
        }
        let step = self.state.bpm * TICKS_PER_QUARTER / (60.0 * sample_rate);
        let due = self.next_beat as f64 * TICKS_PER_QUARTER - self.state.ticks;
        let accent = if due <= step * 0.5 + 1e-9 {
            let accent = if self.next_beat.is_multiple_of(4) {
                1.0
            } else {
                0.6
            };
            self.next_beat += 1;
            self.state.metronome.then_some(accent)
        } else {
            None
        };
        self.state.ticks += step;
        accent
    }
}

/// One writer, one bounded read attempt. Sequential ordering covers every related field.
pub(crate) struct Published {
    revision: AtomicU64,
    epoch: AtomicU64,
    bpm: AtomicU64,
    ticks: AtomicU64,
    flags: AtomicU64,
    pattern_ticks: AtomicU64,
    dropped_hits: AtomicU64,
}

impl Default for Published {
    fn default() -> Self {
        Self {
            revision: AtomicU64::new(0),
            epoch: AtomicU64::new(0),
            bpm: AtomicU64::new(DEFAULT_BPM.to_bits()),
            ticks: AtomicU64::new(0.0_f64.to_bits()),
            flags: AtomicU64::new(0),
            pattern_ticks: AtomicU64::new(0),
            dropped_hits: AtomicU64::new(0),
        }
    }
}

impl Published {
    pub fn record(&self, state: Snapshot, epoch: u64) {
        self.revision.fetch_add(1, SeqCst);
        self.epoch.store(epoch, SeqCst);
        self.bpm.store(state.bpm.to_bits(), SeqCst);
        self.ticks.store(state.ticks.to_bits(), SeqCst);
        self.pattern_ticks
            .store(u64::from(state.pattern_ticks), SeqCst);
        self.dropped_hits.store(state.dropped_hits, SeqCst);
        self.flags.store(
            u64::from(state.playing) | (u64::from(state.metronome) << 1),
            SeqCst,
        );
        self.revision.fetch_add(1, SeqCst);
    }

    pub fn read(&self, epoch: u64) -> Option<Snapshot> {
        let revision = self.revision.load(SeqCst);
        if revision & 1 != 0 || self.epoch.load(SeqCst) != epoch {
            return None;
        }
        let bpm = f64::from_bits(self.bpm.load(SeqCst));
        let ticks = f64::from_bits(self.ticks.load(SeqCst));
        let flags = self.flags.load(SeqCst);
        let state = Snapshot {
            bpm,
            ticks,
            playing: flags & 1 != 0,
            metronome: flags & 2 != 0,
            pattern_ticks: self.pattern_ticks.load(SeqCst) as u32,
            dropped_hits: self.dropped_hits.load(SeqCst),
        };
        (self.revision.load(SeqCst) == revision).then_some(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_snapshots_never_mix_concurrent_fields() {
        let shared = std::sync::Arc::new(Published::default());
        let writer = shared.clone();
        let a = Snapshot {
            bpm: 90.0,
            ticks: 123.0,
            playing: true,
            metronome: false,
            pattern_ticks: 3840,
            dropped_hits: 7,
        };
        let b = Snapshot {
            bpm: 172.0,
            ticks: 456.0,
            playing: false,
            metronome: true,
            pattern_ticks: 0,
            dropped_hits: 19,
        };
        writer.record(a, 7);
        let thread = std::thread::spawn(move || {
            for i in 0..20_000 {
                writer.record(if i & 1 == 0 { a } else { b }, 7);
            }
        });
        for _ in 0..20_000 {
            if let Some(state) = shared.read(7) {
                assert!(state == a || state == b, "{state:?}");
            }
            assert!(shared.read(6).is_none());
        }
        thread.join().unwrap();
        assert_eq!(shared.read(7), Some(b));
        shared.revision.store(1, SeqCst);
        assert_eq!(shared.read(7), None);
    }
}

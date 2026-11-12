//! Engine wrapper for the audio thread: command queue in, counters for the UI out.

use std::sync::Arc;
use std::sync::atomic::{
    AtomicU32, AtomicU64, AtomicUsize,
    Ordering::{Relaxed, SeqCst},
};
use std::time::Instant;

pub use crate::Command;
use crate::Engine;

/// Counters written by the audio thread and read by the UI.
pub struct Stats {
    /// Peak audio thread load in tenths of a percent since the last read.
    load_peak_permille: AtomicU32,
    xruns: AtomicU64,
    /// Input peak since the last read as `f32` bits (for non-negative floats bit order equals value order).
    input_peak: AtomicU32,
    /// Peak output level per channel since the last read, as `f32` bits.
    out_peak_l: AtomicU32,
    out_peak_r: AtomicU32,
    /// Per-block output peaks for the live strip.
    hist: History,
    /// Per-block peaks of what the recording writes, zero while nothing records (25).
    rec_hist: History,
    buffer_size: AtomicU32,
    sample_rate: AtomicU32,
    transport: crate::transport::Published,
    recording: crate::capture::Published,
    pad_revision: AtomicU64,
    pad_epoch: AtomicU64,
    pad_positions: [PadPosition; crate::SLOTS],
    pattern_hits: [AtomicU64; crate::PADS],
}

#[derive(Default)]
struct PadPosition {
    // Compare sample identity only; never dereference a pointer from another thread.
    sound: AtomicUsize,
    pad: AtomicUsize,
    frame: AtomicUsize,
    started: AtomicU64,
}

/// History length: ~2.7 s of blocks at 128 samples / 48 kHz.
const HIST: usize = 1024;

/// A rolling history of per-block peaks (mono max, `f32` bits): the audio thread pushes,
/// the UI reads; atomics only.
struct History {
    vals: [AtomicU32; HIST],
    /// Write cursor; grows by one per block.
    at: AtomicUsize,
}

impl Default for History {
    fn default() -> Self {
        Self {
            vals: std::array::from_fn(|_| AtomicU32::new(0)),
            at: AtomicUsize::new(0),
        }
    }
}

impl History {
    fn push(&self, peak: f32) {
        let at = self.at.fetch_add(1, Relaxed) % HIST;
        self.vals[at].store(peak.to_bits(), Relaxed);
    }

    /// Up to `n` recent values, oldest first (the newest is last).
    fn last(&self, n: usize) -> Vec<f32> {
        let end = self.at.load(Relaxed);
        let n = n.min(HIST).min(end);
        (0..n)
            .map(|i| f32::from_bits(self.vals[(end - n + i) % HIST].load(Relaxed)))
            .collect()
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            load_peak_permille: AtomicU32::new(0),
            xruns: AtomicU64::new(0),
            input_peak: AtomicU32::new(0),
            out_peak_l: AtomicU32::new(0),
            out_peak_r: AtomicU32::new(0),
            hist: History::default(),
            rec_hist: History::default(),
            buffer_size: AtomicU32::new(0),
            sample_rate: AtomicU32::new(0),
            transport: crate::transport::Published::default(),
            recording: crate::capture::Published::default(),
            pad_revision: AtomicU64::new(0),
            pad_epoch: AtomicU64::new(0),
            pad_positions: std::array::from_fn(|_| PadPosition::default()),
            pattern_hits: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }
}

impl Stats {
    /// Accepted transport state for this project, or no snapshot during a concurrent write.
    pub fn transport(&self, epoch: u64) -> Option<crate::transport::Snapshot> {
        self.transport.read(epoch)
    }

    /// Accepted recording phase and boundary, or no snapshot during a concurrent write.
    pub fn recording(&self, epoch: u64) -> Option<crate::capture::Snapshot> {
        self.recording.read(epoch)
    }

    // One writer, bounded stores, no waiting. SeqCst orders the fields with the revision
    // so a reader cannot combine an old voice identity with a new position or epoch.
    fn record_pad_positions(&self, voices: &[crate::Voice], hits: &[u64; crate::PADS], epoch: u64) {
        self.pad_revision.fetch_add(1, SeqCst);
        self.pad_epoch.store(epoch, SeqCst);
        for (slot, voice) in self.pad_positions.iter().zip(voices) {
            if let Some((pad, sound)) = voice.pad.zip(voice.sound.as_ref())
                && voice.fade != Some(0)
                && voice.delay == 0
                && if voice.sequenced {
                    voice.audible
                } else {
                    voice.pos > voice.pos0
                }
            {
                slot.sound.store(Arc::as_ptr(sound) as usize, SeqCst);
                slot.pad.store(pad, SeqCst);
                slot.frame.store(voice.pos, SeqCst);
                slot.started.store(voice.started, SeqCst);
            } else {
                slot.started.store(0, SeqCst);
            }
        }
        for (published, hit) in self.pattern_hits.iter().zip(hits) {
            published.store(*hit, SeqCst);
        }
        self.pad_revision.fetch_add(1, SeqCst);
    }

    /// Last audible sequenced note per pad, including notes that ended within a callback.
    /// No snapshot is returned during a write or for another project.
    pub fn pattern_hits(&self, epoch: u64) -> Option<[u64; crate::PADS]> {
        let revision = self.pad_revision.load(SeqCst);
        if revision & 1 != 0 || self.pad_epoch.load(SeqCst) != epoch {
            return None;
        }
        let hits = std::array::from_fn(|pad| self.pattern_hits[pad].load(SeqCst));
        (self.pad_revision.load(SeqCst) == revision).then_some(hits)
    }

    /// Sample-frame position of the newest sounding voice of an accepted pad and sample
    /// in this project. Returns no position if a callback changes the snapshot during read.
    /// The caller keeps `sound` alive; identity is compared, never dereferenced here.
    pub fn pad_position(
        &self,
        sound: &Arc<crate::Sample>,
        epoch: u64,
        mut accepts_pad: impl FnMut(usize) -> bool,
    ) -> Option<usize> {
        let revision = self.pad_revision.load(SeqCst);
        if revision & 1 != 0 || self.pad_epoch.load(SeqCst) != epoch {
            return None;
        }
        let mut newest = 0;
        let mut position = None;
        for slot in &self.pad_positions {
            let started = slot.started.load(SeqCst);
            if started > newest
                && slot.sound.load(SeqCst) == Arc::as_ptr(sound) as usize
                && accepts_pad(slot.pad.load(SeqCst))
            {
                newest = started;
                position = Some(slot.frame.load(SeqCst));
            }
        }
        if self.pad_revision.load(SeqCst) == revision {
            position
        } else {
            None
        }
    }

    /// Whether any pad voice plays in this project, across all pads and samples.
    /// Previews and effect tails have no pad entry. A changing snapshot returns false.
    pub fn pads_playing(&self, epoch: u64) -> bool {
        let revision = self.pad_revision.load(SeqCst);
        if revision & 1 != 0 || self.pad_epoch.load(SeqCst) != epoch {
            return false;
        }
        let playing = self
            .pad_positions
            .iter()
            .any(|slot| slot.started.load(SeqCst) != 0);
        playing && self.pad_revision.load(SeqCst) == revision
    }

    pub fn add_xrun(&self) {
        self.xruns.fetch_add(1, Relaxed);
    }

    pub fn xruns(&self) -> u64 {
        self.xruns.load(Relaxed)
    }

    /// Peak load in percent since the last call; resets the peak.
    pub fn take_load_peak(&self) -> f32 {
        self.load_peak_permille.swap(0, Relaxed) as f32 / 10.0
    }

    /// From the audio thread: input peak of a block.
    pub fn record_input_peak(&self, peak: f32) {
        if peak.is_finite() {
            self.input_peak.fetch_max(peak.abs().to_bits(), Relaxed);
        }
    }

    /// Input peak since the last call; resets the peak.
    pub fn take_input_peak(&self) -> f32 {
        f32::from_bits(self.input_peak.swap(0, Relaxed))
    }

    /// From the audio thread: output peaks of one block, after the master gain.
    pub fn record_output(&self, l: f32, r: f32) {
        if l.is_finite() && r.is_finite() {
            self.hist.push(l.abs().max(r.abs()));
            self.out_peak_l.fetch_max(l.abs().to_bits(), Relaxed);
            self.out_peak_r.fetch_max(r.abs().to_bits(), Relaxed);
        }
    }

    /// Output peaks per channel since the last call; resets them.
    pub fn take_output(&self) -> (f32, f32) {
        (
            f32::from_bits(self.out_peak_l.swap(0, Relaxed)),
            f32::from_bits(self.out_peak_r.swap(0, Relaxed)),
        )
    }

    /// Up to `n` recent history values, oldest first (the newest is last).
    pub fn output_history(&self, n: usize) -> Vec<f32> {
        self.hist.last(n)
    }

    /// From the audio thread, once per block: the peak of what the recording writes, or
    /// zero while nothing records, so a new recording starts on an empty strip.
    pub fn record_rec_peak(&self, peak: f32) {
        self.rec_hist
            .push(if peak.is_finite() { peak.abs() } else { 0.0 });
    }

    /// Up to `n` recent recording peaks, oldest first (the newest is last).
    pub fn rec_history(&self, n: usize) -> Vec<f32> {
        self.rec_hist.last(n)
    }

    pub fn set_format(&self, buffer_size: u32, sample_rate: u32) {
        self.buffer_size.store(buffer_size, Relaxed);
        self.sample_rate.store(sample_rate, Relaxed);
    }

    /// Buffer size and sample rate the sound server actually gave.
    pub fn format(&self) -> (u32, u32) {
        (
            self.buffer_size.load(Relaxed),
            self.sample_rate.load(Relaxed),
        )
    }
}

/// A message tagged when its producer receives the input.
#[derive(Debug)]
pub struct ProjectCommand {
    pub epoch: u64,
    /// For pad hits, true only if ready equalled epoch at input time.
    pub ready_at_send: bool,
    pub command: ProjectMessage,
}

#[derive(Debug)]
pub enum ProjectMessage {
    Command(Command),
    Ready,
}

/// Zero is the initial, ready project. Publish switches with Release ordering.
#[derive(Default)]
pub struct ProjectState {
    pub epoch: AtomicU64,
    pub ready: AtomicU64,
}

pub struct Realtime {
    engine: Engine,
    /// One queue per writer thread (UI, MIDI): rtrb is single-producer.
    commands: Vec<rtrb::Consumer<Command>>,
    project_commands: Vec<rtrb::Consumer<ProjectCommand>>,
    held: Vec<Option<ProjectCommand>>,
    project: Option<Arc<ProjectState>>,
    epoch: u64,
    marker: bool,
    stats: Arc<Stats>,
    sample_rate: u32,
}

impl Realtime {
    pub fn new(
        mut engine: Engine,
        commands: Vec<rtrb::Consumer<Command>>,
        stats: Arc<Stats>,
        sample_rate: u32,
    ) -> Self {
        engine.set_stats(stats.clone());
        Self {
            engine,
            commands,
            project_commands: Vec::new(),
            held: Vec::new(),
            project: None,
            epoch: 0,
            marker: true,
            stats,
            sample_rate,
        }
    }

    pub fn with_project(
        engine: Engine,
        commands: Vec<rtrb::Consumer<ProjectCommand>>,
        stats: Arc<Stats>,
        sample_rate: u32,
        project: Arc<ProjectState>,
    ) -> Self {
        let mut rt = Self::new(engine, Vec::new(), stats, sample_rate);
        rt.held = (0..commands.len()).map(|_| None).collect();
        rt.project_commands = commands;
        rt.project = Some(project);
        rt
    }

    /// Called from the audio callback: no allocation, no locks, no I/O.
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let started = Instant::now();
        self.engine.poll_erasure_releases();
        for queue in self.commands.iter_mut() {
            for _ in 0..256 {
                let Ok(cmd) = queue.peek() else { break };
                if !self.engine.can_handle(cmd) {
                    break;
                }
                let Ok(cmd) = queue.pop() else { break };
                self.engine.handle(0, cmd);
            }
        }
        if let Some(project) = &self.project {
            use std::sync::atomic::Ordering::{Acquire, Release};
            let epoch = project.epoch.load(Acquire);
            if epoch != self.epoch {
                self.epoch = epoch;
                self.marker = false;
                self.engine.switch_project();
                self.engine.capture.reset(epoch);
            }
            let transition_done = self.engine.project_transition_step();
            for (index, queue) in self.project_commands.iter_mut().enumerate() {
                for _ in 0..256 {
                    let msg = self.held[index].as_ref().or_else(|| queue.peek().ok());
                    let Some(msg) = msg else { break };
                    if let ProjectMessage::Command(cmd) = &msg.command {
                        let hit = matches!(
                            cmd,
                            Command::PadPress { .. }
                                | Command::PadRelease { .. }
                                | Command::PatternPadPress { .. }
                                | Command::PatternPadRelease { .. }
                                | Command::TransportPlay
                                | Command::TransportStop
                                | Command::TransportToggle
                                | Command::PatternRecordStart { .. }
                                | Command::PatternRecordEnd { .. }
                                | Command::PatternRecordAck { .. }
                                | Command::PatternErase { .. }
                                | Command::PatternEraseTarget { .. }
                                | Command::PatternEraseConsumed { .. }
                                | Command::PatternGateConsumed { .. }
                                | Command::PatternRecordRows { .. }
                                | Command::PublishRecordedPattern { .. }
                                | Command::PublishCapturedPattern { .. }
                        );
                        let discard = msg.epoch != epoch
                            || (hit
                                && (!msg.ready_at_send
                                    || !transition_done
                                    || !self.marker
                                    || project.ready.load(Acquire) != epoch));
                        let can_run = if discard {
                            self.engine.can_discard(cmd)
                        } else {
                            self.engine.can_handle(cmd)
                        };
                        if !can_run {
                            break;
                        }
                    }
                    let Some(msg) = self.held[index].take().or_else(|| queue.pop().ok()) else {
                        break;
                    };
                    if msg.epoch != epoch {
                        if let ProjectMessage::Command(cmd) = msg.command {
                            self.engine.discard(cmd);
                        }
                        continue;
                    }
                    let hit = matches!(
                        &msg.command,
                        ProjectMessage::Command(
                            Command::PadPress { .. }
                                | Command::PadRelease { .. }
                                | Command::PatternPadPress { .. }
                                | Command::PatternPadRelease { .. }
                                | Command::TransportPlay
                                | Command::TransportStop
                                | Command::TransportToggle
                                | Command::PatternRecordStart { .. }
                                | Command::PatternRecordEnd { .. }
                                | Command::PatternRecordAck { .. }
                                | Command::PatternErase { .. }
                                | Command::PatternEraseTarget { .. }
                                | Command::PatternEraseConsumed { .. }
                                | Command::PatternGateConsumed { .. }
                                | Command::PatternRecordRows { .. }
                                | Command::PublishRecordedPattern { .. }
                                | Command::PublishCapturedPattern { .. }
                        )
                    );
                    if !transition_done && !hit {
                        self.held[index] = Some(msg);
                        break;
                    }
                    match msg.command {
                        ProjectMessage::Ready => self.marker = true,
                        ProjectMessage::Command(cmd) => {
                            if hit
                                && (!msg.ready_at_send
                                    || !transition_done
                                    || !self.marker
                                    || project.ready.load(Acquire) != epoch)
                            {
                                self.engine.discard(cmd);
                            } else {
                                self.engine.handle(0, cmd);
                            }
                        }
                    }
                }
            }
            if transition_done && self.marker && project.epoch.load(Acquire) == epoch {
                project.ready.store(epoch, Release);
            }
        }
        self.engine.process(left, right);
        self.stats
            .record_pad_positions(&self.engine.voices, &self.engine.pattern_hits, self.epoch);
        self.stats
            .transport
            .record(self.engine.transport(), self.epoch);
        self.stats.recording.record(self.engine.recording());

        let budget = left.len() as f32 / self.sample_rate as f32;
        let permille = (started.elapsed().as_secs_f32() / budget * 1000.0) as u32;
        self.stats.load_peak_permille.fetch_max(permille, Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_publication_preserves_gate_and_pitch_gate_deadlines() {
        use crate::pattern::{Grid, Hit, RawPattern};
        for pitch_gate in [false, true] {
            let render = |publish| {
                let mut engine = Engine::with_sounds(48_000, vec![], vec![vec![0.02; 96_000]]);
                engine.handle(
                    0,
                    Command::SetPadMode {
                        pad: 0,
                        mode: crate::PadMode::Gate,
                    },
                );
                engine.handle(
                    0,
                    Command::SetPadPitch {
                        pad: 0,
                        semitones: if pitch_gate { 12 } else { 0 },
                    },
                );
                engine.handle(
                    0,
                    Command::SetPadPitchGate {
                        pad: 0,
                        enabled: pitch_gate,
                        linked_slice: true,
                    },
                );
                let pattern = |grid, gate| {
                    Arc::new(
                        RawPattern::new(1, grid, 100, vec![Hit::new(0, 0, 127, gate).unwrap()])
                            .unwrap()
                            .compile(),
                    )
                };
                engine.handle(
                    0,
                    Command::SetPattern {
                        pattern: Some(pattern(Grid::Sixteenth, 960)),
                    },
                );
                engine.handle(0, Command::TransportPlay);
                let (mut tx, rx) = rtrb::RingBuffer::new(4);
                let (garbage, mut trash) = rtrb::RingBuffer::new(4);
                engine.set_garbage(garbage);
                let mut rt = Realtime::new(engine, vec![rx], Arc::new(Stats::default()), 48_000);
                let (mut left, mut right) = (vec![0.0; 30_000], vec![0.0; 30_000]);
                rt.process(&mut left[..128], &mut right[..128]);
                if publish {
                    tx.push(Command::PublishRecordedPattern {
                        pattern: pattern(Grid::Quarter, 1),
                    })
                    .unwrap();
                }
                for start in (128..30_000).step_by(128) {
                    let end = (start + 128).min(30_000);
                    rt.process(&mut left[start..end], &mut right[start..end]);
                    while trash.pop().is_ok() {}
                }
                (left, right)
            };
            let control = render(false);
            let published = render(true);
            assert_eq!(published, control, "pitch gate {pitch_gate}");
            let cut = if pitch_gate { 6000 } else { 24_000 };
            for channel in [&published.0, &published.1] {
                assert!(channel[..cut].iter().all(|&v| v == 0.02));
                assert!(channel[cut + 240..].iter().all(|&v| v == 0.0));
            }
        }
    }

    #[test]
    fn project_switch_rejects_queued_hits_and_restores_in_order() {
        use std::sync::atomic::Ordering::{Acquire, Release};
        let state = Arc::new(ProjectState::default());
        let (mut ui, ui_rx) = rtrb::RingBuffer::new(8);
        let (mut midi, midi_rx) = rtrb::RingBuffer::new(8);
        let mut engine = Engine::with_sounds(48_000, vec![], vec![vec![0.5; 48_000]]);
        let (garbage, _rx) = rtrb::RingBuffer::new(512);
        engine.set_garbage(garbage);
        let mut rt = Realtime::with_project(
            engine,
            vec![ui_rx, midi_rx],
            Arc::new(Stats::default()),
            48_000,
            state.clone(),
        );
        let msg = |epoch, ready_at_send, command| ProjectCommand {
            epoch,
            ready_at_send,
            command,
        };
        let hit = || {
            ProjectMessage::Command(Command::PadPress {
                pad: 0,
                velocity: 1.0,
            })
        };
        ui.push(msg(0, true, hit())).unwrap();
        let (mut l, mut r) = ([0.0; 128], [0.0; 128]);
        rt.process(&mut l, &mut r);
        assert!(l.iter().any(|x| *x > 0.0));
        midi.push(msg(0, true, hit())).unwrap();
        state.epoch.store(1, Release);
        ui.push(msg(1, false, ProjectMessage::Ready)).unwrap();
        midi.push(msg(1, false, hit())).unwrap();
        for _ in 0..300 {
            rt.process(&mut l, &mut r);
            if state.ready.load(Acquire) == 1 {
                break;
            }
        }
        assert_eq!(state.ready.load(Acquire), 1);
        for _ in 0..5 {
            rt.process(&mut l, &mut r);
            assert!(l.iter().all(|x| *x == 0.0));
        }
        midi.push(msg(1, true, hit())).unwrap();
        rt.process(&mut l, &mut r);
        assert!(l.iter().all(|x| *x == 0.0));
        state.epoch.store(2, Release);
        ui.push(msg(
            2,
            false,
            ProjectMessage::Command(Command::SetPadSound {
                pad: 0,
                sound: Some(Arc::new(crate::Sample::mono(vec![0.25; 4096]))),
            }),
        ))
        .unwrap();
        ui.push(msg(2, false, ProjectMessage::Ready)).unwrap();
        for _ in 0..300 {
            rt.process(&mut l, &mut r);
            if state.ready.load(Acquire) == 2 {
                break;
            }
        }
        assert_eq!(state.ready.load(Acquire), 2);
        midi.push(msg(2, true, hit())).unwrap();
        rt.process(&mut l, &mut r);
        assert!(l.iter().any(|x| *x > 0.0));
    }

    #[test]
    fn second_switch_invalidates_marker_and_held_restore() {
        use std::sync::atomic::Ordering::{Acquire, Release};
        let state = Arc::new(ProjectState::default());
        let (mut tx, rx) = rtrb::RingBuffer::new(8);
        let mut engine = Engine::new(48_000);
        let (garbage, _rx) = rtrb::RingBuffer::new(512);
        engine.set_garbage(garbage);
        let mut rt = Realtime::with_project(
            engine,
            vec![rx],
            Arc::new(Stats::default()),
            48_000,
            state.clone(),
        );
        let (mut l, mut r) = ([0.0; 128], [0.0; 128]);
        state.epoch.store(1, Release);
        tx.push(ProjectCommand {
            epoch: 1,
            ready_at_send: false,
            command: ProjectMessage::Ready,
        })
        .unwrap();
        rt.process(&mut l, &mut r);
        state.epoch.store(2, Release);
        for _ in 0..300 {
            rt.process(&mut l, &mut r);
        }
        assert_ne!(state.ready.load(Acquire), 2);
        tx.push(ProjectCommand {
            epoch: 2,
            ready_at_send: false,
            command: ProjectMessage::Ready,
        })
        .unwrap();
        rt.process(&mut l, &mut r);
        assert_eq!(state.ready.load(Acquire), 2);
    }

    /// recording 25: "The output recording on the strip" — the history holds the peaks of
    /// what is recorded, newest last, and the silence before the recording as zeros.
    #[test]
    fn the_output_recording_on_the_strip() {
        let s = Stats::default();
        s.record_rec_peak(0.0);
        s.record_rec_peak(0.5);
        s.record_rec_peak(f32::NAN);
        s.record_rec_peak(-0.7);
        assert_eq!(s.rec_history(8), [0.0, 0.5, 0.0, 0.7]);
        assert_eq!(s.rec_history(2), [0.0, 0.7]);
        assert!(
            s.output_history(8).is_empty(),
            "the output history is apart"
        );
    }

    #[test]
    fn pads_playing_reads_only_a_coherent_current_project_snapshot() {
        let stats = Stats::default();
        assert!(!stats.pads_playing(0));
        assert_eq!(stats.pattern_hits(0), Some([0; crate::PADS]));
        stats.pad_positions[crate::SLOTS - 1]
            .started
            .store(1, SeqCst);
        assert!(stats.pads_playing(0), "include every pad slot");
        assert!(!stats.pads_playing(1), "old project");
        assert!(stats.pattern_hits(1).is_none());
        stats.pad_revision.store(1, SeqCst);
        assert!(!stats.pads_playing(0), "writer is in progress");
        assert!(stats.pattern_hits(0).is_none());
        stats.pad_epoch.store(1, SeqCst);
        stats.pad_revision.store(2, SeqCst);
        assert!(!stats.pads_playing(0));
        assert!(stats.pads_playing(1));
        stats.pad_positions[crate::SLOTS - 1]
            .started
            .store(0, SeqCst);
        assert!(!stats.pads_playing(1), "last voice ended");
    }

    #[test]
    fn pattern_play_feedback_includes_the_first_frame_at_low_pitch() {
        use crate::pattern::{Grid, Hit, RawPattern};
        let mut engine = Engine::with_sounds(48_000, vec![], vec![vec![0.02; 4096]]);
        engine.handle(
            0,
            Command::SetPadPitch {
                pad: 0,
                semitones: -48,
            },
        );
        engine.handle(
            0,
            Command::SetPattern {
                pattern: Some(Arc::new(
                    RawPattern::new(1, Grid::Off, 100, vec![Hit::new(0, 0, 127, 0).unwrap()])
                        .unwrap()
                        .compile(),
                )),
            },
        );
        engine.handle(0, Command::TransportPlay);
        let stats = Arc::new(Stats::default());
        let mut rt = Realtime::new(engine, vec![], stats.clone(), 48_000);
        let (mut left, mut right) = ([0.0], [0.0]);
        rt.process(&mut left, &mut right);
        assert_eq!(left, [0.02]);
        assert_eq!(right, [0.02]);
        assert!(stats.pads_playing(0));
        assert!(stats.pattern_hits(0).unwrap()[0] > 0);
        assert_eq!(
            rt.engine.voices[0].pos, 0,
            "source position is still fractional"
        );
    }

    #[test]
    fn pad_position_snapshot_keeps_source_pad_epoch_and_start_order() {
        let sample = Arc::new(crate::Sample::mono(vec![0.1; 4096]));
        let other = Arc::new(crate::Sample::mono(vec![0.2; 4096]));
        let stats = Arc::new(Stats::default());
        let mut engine = Engine::with_sounds(48_000, vec![], vec![]);
        for (pad, sound) in [(0, sample.clone()), (1, sample.clone()), (2, other.clone())] {
            engine.handle(
                0,
                Command::SetPadSound {
                    pad,
                    sound: Some(sound),
                },
            );
        }
        engine.handle(
            0,
            Command::PadPress {
                pad: 0,
                velocity: 1.0,
            },
        );
        let mut rt = Realtime::new(engine, vec![], stats.clone(), 48_000);
        let (mut l, mut r) = ([0.0; 128], [0.0; 128]);
        rt.process(&mut l, &mut r);
        assert_eq!(stats.pad_position(&sample, 0, |_| true), Some(128));
        rt.engine.handle(
            0,
            Command::PadPress {
                pad: 1,
                velocity: 1.0,
            },
        );
        rt.engine.handle(
            0,
            Command::PadPress {
                pad: 2,
                velocity: 1.0,
            },
        );
        rt.process(&mut l, &mut r);
        assert_eq!(stats.pad_position(&sample, 0, |_| true), Some(128));
        assert_eq!(stats.pad_position(&sample, 0, |pad| pad == 0), Some(256));
        assert_eq!(stats.pad_position(&other, 0, |pad| pad == 2), Some(128));
        assert_eq!(stats.pad_position(&sample, 1, |_| true), None);
        assert_eq!(stats.pad_position(&sample, 0, |pad| pad == 2), None);
        rt.engine.handle(0, Command::HushPads);
        rt.process(&mut [0.0; 512], &mut [0.0; 512]);
        assert_eq!(stats.pad_position(&sample, 0, |_| true), None);
    }

    #[test]
    fn pad_position_snapshot_never_mixes_concurrent_voice_fields() {
        let a = Arc::new(crate::Sample::mono(vec![0.1; 4096]));
        let b = Arc::new(crate::Sample::mono(vec![0.2; 4096]));
        let stats = Arc::new(Stats::default());
        let writer = stats.clone();
        let (wa, wb) = (a.clone(), b.clone());
        let thread = std::thread::spawn(move || {
            let mut voices: [crate::Voice; crate::SLOTS] =
                std::array::from_fn(|_| Default::default());
            for turn in 0..10_000 {
                let (sample, pad, frame) = if turn & 1 == 0 {
                    (&wa, 1, 1111)
                } else {
                    (&wb, 2, 2222)
                };
                voices[0].sound = Some(sample.clone());
                voices[0].pad = Some(pad);
                voices[0].pos = frame;
                voices[0].started = turn + 1;
                writer.record_pad_positions(&voices, &[turn + 1; crate::PADS], 0);
            }
        });
        for _ in 0..10_000 {
            if let Some(frame) = stats.pad_position(&a, 0, |pad| pad == 1) {
                assert_eq!(frame, 1111);
            }
            if let Some(frame) = stats.pad_position(&b, 0, |pad| pad == 2) {
                assert_eq!(frame, 2222);
            }
            if let Some(hits) = stats.pattern_hits(0) {
                assert!(hits.iter().all(|hit| *hit == hits[0]));
            }
        }
        thread.join().unwrap();
        assert_eq!(stats.pattern_hits(0), Some([10_000; crate::PADS]));
        assert_eq!(stats.pad_position(&b, 0, |pad| pad == 2), Some(2222));
        assert_eq!(stats.pad_position(&a, 0, |_| true), None);
    }

    /// Scenario "UI hangs" (audio-engine): the UI thread stops answering for 2 seconds with
    /// its end of the queue; the callback never waits for it and the pad plays on without a
    /// gap.
    #[test]
    fn ui_hangs() {
        use std::time::Duration;
        let (mut ui, rx) = rtrb::RingBuffer::new(64);
        let stats = Arc::new(Stats::default());
        let mut rt = Realtime::new(crate::Engine::new(48_000), vec![rx], stats, 48_000);
        let long = Arc::new(crate::Sample::mono(vec![0.5; 48_000 * 3]));
        let sound = Some(long);
        ui.push(Command::SetPadSound { pad: 0, sound }).unwrap();
        ui.push(Command::PadPress {
            pad: 0,
            velocity: 1.0,
        })
        .unwrap();
        let hung = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            drop(ui);
        });
        let started = Instant::now();
        let (mut l, mut r) = (vec![0.0; 128], vec![0.0; 128]);
        let mut out = Vec::new();
        for _ in 0..2 * 48_000 / 128 {
            rt.process(&mut l, &mut r);
            out.extend_from_slice(&l);
        }
        assert!(
            started.elapsed() < Duration::from_millis(1500),
            "the callback waited for the UI: {:?}",
            started.elapsed()
        );
        assert!(
            out[480..].iter().all(|&x| (x - 0.5).abs() < 1e-3),
            "a gap in the sound"
        );
        hung.join().unwrap();
    }
}

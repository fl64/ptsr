//! Engine wrapper for the audio thread: command queue in, counters for the UI out.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering::Relaxed};
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
        }
    }
}

impl Stats {
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
            }
            let transition_done = self.engine.project_transition_step();
            for (index, queue) in self.project_commands.iter_mut().enumerate() {
                for _ in 0..256 {
                    let msg = self.held[index].as_ref().or_else(|| queue.peek().ok());
                    let Some(msg) = msg else { break };
                    if let ProjectMessage::Command(cmd) = &msg.command {
                        let hit =
                            matches!(cmd, Command::PadPress { .. } | Command::PadRelease { .. });
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
                            Command::PadPress { .. } | Command::PadRelease { .. }
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

        let budget = left.len() as f32 / self.sample_rate as f32;
        let permille = (started.elapsed().as_secs_f32() / budget * 1000.0) as u32;
        self.stats.load_peak_permille.fetch_max(permille, Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

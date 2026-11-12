//! Separate active capture workloads. Preparation and collection stay outside callback timing.
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};

use engine::capture::{EraseHold, Event, Kind as EventKind, Phase, Recovery, Snapshot};
use engine::pattern::{Grid, Hit, PlaybackRow, RawPattern, RowId};
use engine::rt::{Command, Realtime, Stats};
use engine::{Engine, MAX_VOICES, PADS, PadMode, Sample};

pub const SAMPLE_RATE: u32 = 48_000;
pub const BLOCK: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Recording,
    Erasure,
}
impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Recording => "active recording",
            Self::Erasure => "active erasure",
        }
    }
}

struct Hold {
    pad: usize,
    released: Arc<AtomicBool>,
    accepted: Arc<EraseHold>,
}
impl Hold {
    fn new(pad: usize) -> Self {
        let released = Arc::new(AtomicBool::new(false));
        Self {
            pad,
            accepted: Arc::new(EraseHold::new(released.clone())),
            released,
        }
    }
    fn press(&self) -> Command {
        Command::PatternPadPress {
            pass: 1,
            pad: self.pad,
            velocity: 1.0,
            hold: self.accepted.clone(),
        }
    }
    fn release(self) -> Command {
        self.released.store(true, SeqCst);
        Command::PatternPadRelease {
            pad: self.pad,
            hold: self.accepted,
        }
    }
}

pub struct Workload {
    kind: Kind,
    realtime: Realtime,
    commands: rtrb::Producer<Command>,
    feedback: rtrb::Consumer<Event>,
    garbage: rtrb::Consumer<engine::Retired>,
    recovery: Arc<Recovery>,
    stats: Arc<Stats>,
    sound: Arc<Sample>,
    holds: Vec<Hold>,
    pending: Vec<Event>,
    onsets: usize,
    gates: usize,
    audible: usize,
    left: [f32; BLOCK],
    right: [f32; BLOCK],
}

#[derive(Debug)]
pub struct Evidence {
    pub onsets: usize,
    pub gates: usize,
    pub erased: u64,
    pub voices: usize,
}

impl Workload {
    pub fn new(
        kind: Kind,
        blocks: usize,
        setup: impl FnOnce(&mut dyn FnMut(Command)),
    ) -> io::Result<Self> {
        if blocks == 0 {
            return Err(io::Error::other(
                "active workload requires at least one block",
            ));
        }
        let sound = Arc::new(Sample::mono(
            (0..BLOCK * blocks * 2 + 1)
                .map(|i| (i as f32 * 0.01).sin() * 0.01)
                .collect(),
        ));
        let mut engine = Engine::with_sounds(SAMPLE_RATE, engine::click(SAMPLE_RATE), vec![]);
        let (garbage_tx, garbage) = rtrb::RingBuffer::new(1024);
        engine.set_garbage(garbage_tx);
        for pad in 0..MAX_VOICES {
            engine.handle(
                0,
                Command::SetPadSound {
                    pad,
                    sound: Some(sound.clone()),
                },
            );
        }
        setup(&mut |command| engine.handle(0, command));
        engine.handle(
            0,
            Command::SetPadSound {
                pad: PADS - 1,
                sound: None,
            },
        );
        engine.handle(
            0,
            Command::SetPadMode {
                pad: PADS - 1,
                mode: PadMode::Gate,
            },
        );
        let hits: Vec<Hit> = if kind == Kind::Erasure {
            (0..128)
                .flat_map(|step| {
                    (0..MAX_VOICES).map(move |pad| Hit::new(step * 60, pad, 127, 0).unwrap())
                })
                .collect()
        } else {
            vec![]
        };
        let identities = (0..hits.len())
            .map(|slot| PlaybackRow::existing(RowId::new(slot, 1).unwrap()))
            .collect::<Vec<_>>();
        engine.handle(
            0,
            Command::SetPattern {
                pattern: Some(Arc::new(
                    RawPattern::new(2, Grid::SixtyFourth, 100, hits)
                        .unwrap()
                        .compile_recorded(&identities)
                        .unwrap(),
                )),
            },
        );
        let feedback = engine.enable_capture();
        let recovery = engine.capture_recovery().unwrap();
        let stats = Arc::new(Stats::default());
        let (mut commands, rx) = rtrb::RingBuffer::new(1024);
        commands
            .push(Command::PatternRecordStart { pass: 1 })
            .unwrap();
        let mut holds = Vec::new();
        if kind == Kind::Erasure {
            commands
                .push(Command::PatternErase {
                    pass: 1,
                    enabled: true,
                })
                .unwrap();
            for pad in 0..PADS {
                let hold = Hold::new(pad);
                commands.push(hold.press()).unwrap();
                holds.push(hold);
            }
        }
        Ok(Self {
            kind,
            realtime: Realtime::new(engine, vec![rx], stats.clone(), SAMPLE_RATE),
            commands,
            feedback,
            garbage,
            recovery,
            stats,
            sound,
            holds,
            pending: vec![],
            onsets: 0,
            gates: 0,
            audible: 0,
            left: [0.0; BLOCK],
            right: [0.0; BLOCK],
        })
    }

    pub fn prepare(&mut self, block: usize) {
        if self.kind == Kind::Recording {
            self.release();
            for pad in [
                block
                    .is_multiple_of(64)
                    .then_some((block / 64) % MAX_VOICES),
                (block % 128 == 2).then_some(PADS - 1),
            ]
            .into_iter()
            .flatten()
            {
                let hold = Hold::new(pad);
                self.commands.push(hold.press()).unwrap();
                self.holds.push(hold);
            }
        }
    }
    pub fn process(&mut self) {
        self.realtime.process(&mut self.left, &mut self.right);
    }
    pub fn snapshot(&self) -> Snapshot {
        self.stats
            .recording(0)
            .expect("accepted active capture snapshot")
    }
    pub fn collect(&mut self) -> io::Result<()> {
        while let Ok(event) = self.feedback.pop() {
            if matches!(event.kind, EventKind::Onset { .. }) {
                self.onsets += 1;
                if event.gate.is_some() {
                    self.pending.push(event);
                }
            }
        }
        let mut index = 0;
        while index < self.pending.len() {
            if self.recovery.completed(self.pending[index]).is_some() {
                self.gates += 1;
                self.pending.swap_remove(index);
            } else {
                index += 1;
            }
        }
        while self.garbage.pop().is_ok() {}
        if self
            .left
            .iter()
            .chain(&self.right)
            .any(|value| !value.is_finite())
        {
            return Err(io::Error::other(
                "active workload produced a non-finite sample",
            ));
        }
        self.audible += usize::from(
            self.left
                .iter()
                .chain(&self.right)
                .any(|value| value.abs() > 1e-12),
        );
        let state = self.snapshot();
        if state.phase != Phase::Recording
            || state.pass != 1
            || state.dropped_onsets != 0
            || state.dropped_events != 0
            || state.limit_refusals != 0
            || state.erase_refusals != 0
        {
            return Err(io::Error::other(format!(
                "{} lost its active invariant: {state:?}",
                self.kind.label()
            )));
        }
        if self.kind == Kind::Erasure && state.erase_targets != [u64::MAX; 2] {
            return Err(io::Error::other("active erasure lost a held target"));
        }
        Ok(())
    }
    pub fn release(&mut self) {
        for hold in self.holds.drain(..) {
            self.commands.push(hold.release()).unwrap();
        }
    }
    pub fn finish(&mut self) {
        self.commands
            .push(Command::PatternRecordEnd { pass: 1 })
            .unwrap();
    }
    pub fn validate(&mut self, blocks: usize) -> io::Result<Evidence> {
        while let Ok(event) = self.feedback.pop() {
            if matches!(event.kind, EventKind::Onset { .. }) {
                self.onsets += 1;
            }
        }
        while self.garbage.pop().is_ok() {}
        for event in self.pending.drain(..) {
            if self.recovery.completed(event).is_none() {
                return Err(io::Error::other(
                    "active recording retained an unfinished Gate",
                ));
            }
            self.gates += 1;
        }
        let state = self.snapshot();
        let expected_gates = if self.kind == Kind::Recording {
            blocks.saturating_sub(2).div_ceil(128)
        } else {
            0
        };
        let expected_onsets = if self.kind == Kind::Recording {
            blocks.div_ceil(64) + expected_gates
        } else {
            0
        };
        let expected_erased = if self.kind == Kind::Erasure {
            ((blocks as f64 * BLOCK as f64 * 172.0 * 960.0 / (60.0 * f64::from(SAMPLE_RATE)) / 60.0)
                .ceil() as usize)
                .min(128)
                * MAX_VOICES
        } else {
            0
        };
        let voices = (0..MAX_VOICES)
            .filter(|&pad| {
                self.stats
                    .pad_position(&self.sound, 0, |accepted| accepted == pad)
                    .is_some()
            })
            .count();
        let dropped = self.stats.transport(0).unwrap().dropped_hits;
        if state.phase != Phase::Finishing
            || state.erase_targets != [0, 0]
            || state.dropped_onsets != 0
            || state.dropped_events != 0
            || state.limit_refusals != 0
            || state.erase_refusals != 0
            || self.onsets != expected_onsets
            || state.captured_onsets as usize != expected_onsets
            || self.gates != expected_gates
            || state.erased_rows as usize != expected_erased
            || voices != MAX_VOICES
            || dropped != 0
            || self.audible != blocks
        {
            return Err(io::Error::other(format!(
                "{} invalid completion: {state:?}, onsets {}/{expected_onsets}, gates {}/{expected_gates}, erased {expected_erased}, voices {voices}, dropped {dropped}, audible {}/{}",
                self.kind.label(),
                self.onsets,
                self.gates,
                self.audible,
                blocks
            )));
        }
        Ok(Evidence {
            onsets: self.onsets,
            gates: self.gates,
            erased: state.erased_rows,
            voices,
        })
    }
}

pub fn maximum_exceeded(report: &super::Report) -> bool {
    report.wall.over_target != 0 || report.cpu.over_target != 0
}

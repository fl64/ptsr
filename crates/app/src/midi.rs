//! MIDI input: finds ports, connects and drops them on the fly, sends pad hits straight to the
//! audio thread (no UI frame delay) and everything else to the UI.

#[path = "smc_usb.rs"]
mod smc_usb;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering::Relaxed};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use crate::input_sources::{Connection, Stamp as SourceStamp};

use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
use engine::{Command, PADS_PER_BANK};
use input::midi::{
    Activity, Binding, Key, ListTarget, Msg, Param, Profile, Resolved, Scope, StepTarget,
};
use input::{Action, Focus, Group, PadRoute, route_pad};
use std::sync::atomic::Ordering::Acquire;

/// midir does not report hotplug, so ports are polled.
const SCAN: Duration = Duration::from_secs(1);
const DEFAULT_HOLD: Duration = Duration::from_millis(300);

fn virtual_mode() -> bool {
    std::env::var("PTSR_DEV_VIRTUAL").is_ok_and(|value| value == "1")
}

/// The original audio attempt must not be inferred from App's later screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadSound {
    UiUnattempted { deferred: bool },
    EditUnattempted,
    DirectAccepted,
    DirectRejected,
}

pub enum Event {
    SetupStatus {
        ready: bool,
        message: String,
    },
    Connected {
        port: String,
        profile: String,
    },
    Disconnected {
        port: String,
    },
    /// Every message, for the input debug tab.
    Message {
        port: String,
        profile: String,
        msg: Msg,
        meaning: String,
    },
    /// A controller hit with its original audio disposition.
    PadHit {
        abs: usize,
        velocity: f32,
        sound: PadSound,
        ui_press: Option<u64>,
    },
    /// A controller pad was released (or its device disappeared).
    PadRelease {
        abs: usize,
        /// Dispatch queued the audio release; App must not send it again.
        direct: bool,
    },
    UiPadRelease {
        abs: usize,
        id: u64,
    },
    PanelCell {
        port: String,
        cell: u8,
    },
    /// The PAD BANK hold of a Master connection began or ended (131).
    SmcPadBank {
        port: String,
        setting: u64,
        held: bool,
    },
    Action(Action),
    Param {
        port: String,
        control: String,
        param: Param,
        value: f32,
        takeover: bool,
    },
    Step {
        port: String,
        control: String,
        target: StepTarget,
        steps: i32,
        fine: bool,
    },
}

/// One complete input snapshot. Zero is normal performance on the first bank.
#[derive(Default)]
pub struct Shared {
    focus: AtomicU32,
    acknowledged: AtomicU64,
    pad_bank_setting: AtomicU64,
}

impl Shared {
    fn set_pad_bank_shortcut(&self, enabled: bool) {
        // The UI is the only writer. Each change invalidates replies in flight.
        let setting = self.pad_bank_setting.load(Relaxed);
        if setting & 1 != u64::from(enabled) {
            self.pad_bank_setting
                .store(setting.wrapping_add(1), Relaxed);
        }
    }

    fn pad_bank_shortcut(&self) -> Option<u64> {
        let setting = self.pad_bank_setting.load(Relaxed);
        (setting & 1 != 0).then_some(setting)
    }

    /// Publishes valid state; rejects an invalid bank without changing the snapshot.
    pub fn publish(&self, focus: Focus) -> bool {
        let Some(bits) = focus.pack() else {
            return false;
        };
        self.focus.store(bits, Relaxed);
        true
    }

    /// Reads one snapshot. Returns its packed value if the encoding is invalid.
    pub fn focus(&self) -> Result<Focus, u32> {
        let bits = self.focus.load(Relaxed);
        Focus::unpack(bits).ok_or(bits)
    }

    #[cfg(test)]
    pub fn inject_focus(&self, bits: u32) {
        self.focus.store(bits, Relaxed);
    }

    fn input(&self) -> (u64, Result<Focus, u32>) {
        // Observing consumed work must precede reading the state it published.
        let acknowledged = self.acknowledged.load(Acquire);
        (acknowledged, self.focus())
    }

    fn consumed(&self, focus: Focus, sequence: u64) {
        if self.publish(focus) {
            self.acknowledged
                .store(sequence, std::sync::atomic::Ordering::Release);
        }
    }
}

/// PAD BANK holds and the note each held pad press was routed to (131).
#[derive(Default)]
struct HoldRoute {
    held: HashSet<String>,
    notes: HashMap<(String, u8, u8), u8>,
}

impl HoldRoute {
    /// With the hold shortcut, both note ranges of a physical pad mean that pad: a hold
    /// makes the hit a panel cell, otherwise it plays. The question pads P7 and P8 (notes
    /// 62 and 63) answer without a hold. A release follows its press.
    fn route(&mut self, port: &str, msg: Msg, active: bool, question: bool) -> Msg {
        match msg {
            Msg::NoteOn {
                ch: 10,
                note: note @ 36..=67,
                velocity,
            } if active && port.contains("SMC-PAD-Master") => {
                let position = (note - 36) % 16;
                let cell = 52 + position;
                let target = if self.held.contains(port) || (question && matches!(cell, 62 | 63)) {
                    cell
                } else {
                    36 + position
                };
                self.notes.insert((port.into(), 10, note), target);
                Msg::NoteOn {
                    ch: 10,
                    note: target,
                    velocity,
                }
            }
            Msg::NoteOff { ch, note } => match self.notes.remove(&(port.into(), ch, note)) {
                Some(target) => Msg::NoteOff { ch, note: target },
                None => msg,
            },
            msg => msg,
        }
    }

    fn end(&mut self, port: &str, setting: u64, ui: &mut UiSender) {
        if self.held.remove(port) {
            ui.send(
                Event::SmcPadBank {
                    port: port.into(),
                    setting,
                    held: false,
                },
                None,
            );
        }
    }

    fn end_all(&mut self, ui: &mut UiSender) {
        for port in std::mem::take(&mut self.held) {
            ui.send(
                Event::SmcPadBank {
                    port,
                    setting: 0,
                    held: false,
                },
                None,
            );
        }
    }
}

enum Raw {
    SmcSetup(Result<(), String>),
    SmcLost,
    SmcPadBank {
        port: String,
        requested_at: Instant,
        setting: u64,
        held: bool,
    },
    Connected(String, Option<Connection>),
    Disconnected(String, Option<Connection>),
    Msg(String, Msg, Stamp),
    /// New learned assignments of a profile in a group.
    Learned(String, Scope, HashMap<Key, Binding>),
    LearnedSet(LearnedMaps),
    /// The roles of a profile are copied into the learned maps (59): only those act now.
    Seeded(String),
    HoldTime(Duration),
    #[cfg(test)]
    Tick(Instant),
    #[cfg(test)]
    Fence(Sender<()>),
}

#[derive(Clone)]
struct UiHold {
    abs: usize,
    epoch: u64,
    id: u64,
    released: Arc<AtomicBool>,
}

type DirectHolds = HashMap<(String, u8, Key), Vec<(usize, u64)>>;
type UiHolds = HashMap<(String, u8, Key), Vec<UiHold>>;

#[derive(Clone, Copy)]
struct Stamp {
    epoch: u64,
    ready: bool,
    source: Option<SourceStamp>,
}

type GestureKey = (String, u8, Key);

#[derive(Clone, Copy)]
enum GestureEdge {
    Tap(Action, Stamp),
    Down(Action, Stamp),
    Up(Action),
}

type GestureOutput = (GestureKey, GestureEdge);

struct ButtonGesture {
    tap: Option<Action>,
    hold: Action,
    at: Instant,
    threshold: Duration,
    stamp: Stamp,
    active: bool,
}

struct Gestures {
    held: HashMap<GestureKey, ButtonGesture>,
    threshold: Duration,
}

impl Gestures {
    fn new(threshold: Duration) -> Self {
        Self {
            held: HashMap::new(),
            threshold,
        }
    }

    fn press(
        &mut self,
        key: GestureKey,
        at: Instant,
        stamp: Stamp,
        tap: Option<Action>,
        hold: Action,
    ) {
        self.held.entry(key).or_insert(ButtonGesture {
            tap,
            hold,
            at,
            threshold: self.threshold,
            stamp,
            active: false,
        });
    }

    fn promote(gesture: &mut ButtonGesture) -> Option<GestureEdge> {
        if gesture.active {
            return None;
        }
        gesture.active = true;
        Some(GestureEdge::Down(gesture.hold, gesture.stamp))
    }

    fn other(&mut self, port: &str, key: &GestureKey) -> Vec<GestureOutput> {
        let mut pending: Vec<_> = self
            .held
            .iter()
            .filter(|(held, gesture)| held.0 == port && *held != key && !gesture.active)
            .map(|(held, gesture)| (held.clone(), gesture.at))
            .collect();
        pending.sort_by_key(|(held, at)| (*at, held.1, held.2));
        pending
            .into_iter()
            .filter_map(|(held, _)| Some((held.clone(), Self::promote(self.held.get_mut(&held)?)?)))
            .collect()
    }

    fn release(&mut self, key: &GestureKey, at: Instant, stamp: Stamp) -> Vec<GestureOutput> {
        let Some(mut gesture) = self.held.remove(key) else {
            return Vec::new();
        };
        if gesture.active {
            return vec![(key.clone(), GestureEdge::Up(gesture.hold))];
        }
        if at
            .checked_duration_since(gesture.at)
            .is_some_and(|d| d >= gesture.threshold)
        {
            vec![
                (key.clone(), Self::promote(&mut gesture).unwrap()),
                (key.clone(), GestureEdge::Up(gesture.hold)),
            ]
        } else {
            gesture
                .tap
                .map(|tap| (key.clone(), GestureEdge::Tap(tap, stamp)))
                .into_iter()
                .collect()
        }
    }

    fn due(&mut self, now: Instant) -> Vec<GestureOutput> {
        let mut pending: Vec<_> = self
            .held
            .iter()
            .filter(|(_, gesture)| {
                !gesture.active
                    && now
                        .checked_duration_since(gesture.at)
                        .is_some_and(|d| d >= gesture.threshold)
            })
            .map(|(key, gesture)| (key.clone(), gesture.at))
            .collect();
        pending.sort_by_key(|(key, at)| (*at, key.1, key.2));
        pending
            .into_iter()
            .filter_map(|(key, _)| Some((key.clone(), Self::promote(self.held.get_mut(&key)?)?)))
            .collect()
    }

    fn disconnect(&mut self, port: &str) -> Vec<GestureOutput> {
        let mut ended = Vec::new();
        self.held.retain(|key, gesture| {
            if key.0 == port {
                if gesture.active {
                    ended.push((
                        gesture.at,
                        key.1,
                        key.2,
                        key.clone(),
                        GestureEdge::Up(gesture.hold),
                    ));
                }
                false
            } else {
                true
            }
        });
        ended.sort_by_key(|(at, ch, key, _, _)| (*at, *ch, *key));
        ended
            .into_iter()
            .map(|(_, _, _, key, edge)| (key, edge))
            .collect()
    }

    fn reject_pending(&mut self) {
        self.held.retain(|_, gesture| gesture.active);
    }

    fn release_invalid(&mut self, key: &GestureKey) -> Vec<GestureOutput> {
        match self.held.remove(key) {
            Some(gesture) if gesture.active => vec![(key.clone(), GestureEdge::Up(gesture.hold))],
            _ => Vec::new(),
        }
    }
}

fn send_gesture_edges(ui: &mut UiSender, edges: impl IntoIterator<Item = GestureOutput>) {
    let previous = ui.origin.take();
    for (key, edge) in edges {
        let source = match edge {
            GestureEdge::Tap(_, stamp) | GestureEdge::Down(_, stamp) => stamp.source,
            GestureEdge::Up(_) => None,
        };
        ui.origin = Some(Provenance {
            port: key.0,
            source,
            from_message: true,
            activity: Activity::None,
            key: Some(key.2),
        });
        match edge {
            GestureEdge::Tap(action, stamp) | GestureEdge::Down(action, stamp) => {
                ui.send(Event::Action(action), Some(stamp));
            }
            GestureEdge::Up(Action::RecordMic(true)) => {
                ui.send(Event::Action(Action::RecordMic(false)), None);
            }
            GestureEdge::Up(_) => {}
        }
    }
    ui.origin = previous;
}

impl Stamp {
    fn capture(project: Option<&ProjectState>) -> Self {
        let Some(project) = project else {
            return Self {
                epoch: 0,
                ready: false,
                source: None,
            };
        };
        let epoch = project.epoch.load(Acquire);
        let ready = project.ready.load(Acquire) == epoch;
        Self {
            epoch,
            ready,
            source: None,
        }
    }

    fn ready(epoch: u64) -> Self {
        Self {
            epoch,
            ready: true,
            source: None,
        }
    }
}

/// Learned assignments per (profile name, group scope; 28).
pub type LearnedMaps = HashMap<(String, Scope), HashMap<Key, Binding>>;

#[derive(Clone)]
struct Provenance {
    port: String,
    source: Option<SourceStamp>,
    from_message: bool,
    activity: Activity,
    key: Option<Key>,
}

pub struct UiEvent {
    pub event: Event,
    stamp: Option<Stamp>,
    sequence: Option<u64>,
    origin: Option<Provenance>,
    owner_input: bool,
    released: Option<Arc<AtomicBool>>,
}

struct UiSender {
    tx: Sender<UiEvent>,
    last_sent: u64,
    origin: Option<Provenance>,
}

impl UiSender {
    fn send(&mut self, event: Event, stamp: Option<Stamp>) {
        self.send_input(event, stamp, false, None);
    }

    fn send_input(
        &mut self,
        event: Event,
        stamp: Option<Stamp>,
        owner_input: bool,
        released: Option<Arc<AtomicBool>>,
    ) {
        let sequence = matches!(
            &event,
            Event::Action(_)
                | Event::PanelCell { .. }
                | Event::Step {
                    target: StepTarget::List(ListTarget::Page | ListTarget::Bank)
                        | StepTarget::Param(Param::Page),
                    ..
                }
                | Event::PadHit {
                    sound: PadSound::UiUnattempted { .. } | PadSound::EditUnattempted,
                    ..
                }
        )
        .then(|| self.last_sent.wrapping_add(1));
        if self
            .tx
            .send(UiEvent {
                event,
                stamp,
                sequence,
                origin: self.origin.clone(),
                owner_input,
                released,
            })
            .is_ok()
            && let Some(sequence) = sequence
        {
            self.last_sent = sequence;
        }
    }
}

impl UiEvent {
    #[cfg(test)]
    pub fn stamped(event: Event, project: &ProjectState) -> Self {
        Self {
            event,
            stamp: Some(Stamp::capture(Some(project))),
            sequence: None,
            origin: None,
            owner_input: false,
            released: None,
        }
    }

    pub fn release_signal(&self) -> Option<Arc<AtomicBool>> {
        self.released.clone()
    }

    pub fn origin(&self) -> Option<&str> {
        let origin = self.origin.as_ref()?;
        origin.from_message.then_some(origin.port.as_str())
    }

    pub fn control_key(&self) -> Option<Key> {
        self.origin.as_ref()?.key
    }

    pub(super) fn source(&self) -> Option<SourceStamp> {
        self.origin.as_ref()?.source
    }

    pub(super) fn activity(&self) -> Activity {
        self.origin
            .as_ref()
            .map_or(Activity::None, |origin| origin.activity)
    }

    pub fn owner_input(&self) -> bool {
        self.owner_input
    }

    pub fn sequence(&self) -> Option<u64> {
        self.sequence
    }

    pub fn preview_ready(&self) -> bool {
        self.stamp.is_none_or(|stamp| stamp.ready)
    }

    pub fn current(self, project: Option<&ProjectState>) -> Option<Event> {
        let valid = self.stamp.is_none_or(|stamp| {
            project.is_none_or(|project| {
                stamp.epoch == project.epoch.load(Acquire)
                    && (stamp.ready || !matches!(self.event, Event::PadHit { .. }))
            })
        });
        valid.then_some(self.event)
    }
}

pub struct Midi {
    pub events: Receiver<UiEvent>,
    updates: Sender<Raw>,
    shared: Arc<Shared>,
    #[cfg(test)]
    recorded_connections: std::cell::RefCell<HashMap<String, Option<Connection>>>,
}

impl Midi {
    #[cfg(test)]
    pub fn recorded(
        profiles: Vec<Profile>,
        commands: rtrb::Producer<ProjectCommand>,
        project: Arc<ProjectState>,
        shared: Arc<Shared>,
    ) -> (Self, std::thread::JoinHandle<()>) {
        let (updates, raw_rx) = mpsc::channel();
        let (tx, events) = mpsc::channel();
        let dispatch_shared = shared.clone();
        let worker = std::thread::spawn(move || {
            dispatch(
                raw_rx,
                tx,
                profiles,
                LearnedMaps::new(),
                Vec::new(),
                Some(commands),
                Some(project),
                dispatch_shared,
            );
        });
        (
            Self {
                events,
                updates,
                shared,
                recorded_connections: Default::default(),
            },
            worker,
        )
    }

    #[cfg(test)]
    pub fn recorded_connect(&self, port: &str) {
        let connection = Connection::fresh();
        self.recorded_connections
            .borrow_mut()
            .insert(port.into(), connection);
        self.updates
            .send(Raw::Connected(port.into(), connection))
            .unwrap();
    }

    #[cfg(test)]
    pub fn recorded_disconnect(&self, port: &str) {
        let connection = self
            .recorded_connections
            .borrow_mut()
            .remove(port)
            .flatten();
        self.updates
            .send(Raw::Disconnected(port.into(), connection))
            .unwrap();
    }

    #[cfg(test)]
    pub fn recorded_smc_setup(&self) {
        self.updates.send(Raw::SmcSetup(Ok(()))).unwrap();
    }

    #[cfg(test)]
    pub fn recorded_smc_failure(&self) {
        self.updates
            .send(Raw::SmcSetup(Err("status read failed".into())))
            .unwrap();
    }

    #[cfg(test)]
    pub fn recorded_connection(&self, port: &str) -> Option<Connection> {
        self.recorded_connections
            .borrow()
            .get(port)
            .copied()
            .flatten()
    }

    #[cfg(test)]
    pub fn recorded_bytes(&self, port: &str, bytes: &[u8], project: &ProjectState) {
        self.recorded_bytes_at(port, bytes, project, Instant::now());
    }

    #[cfg(test)]
    pub fn recorded_bytes_at(&self, port: &str, bytes: &[u8], project: &ProjectState, at: Instant) {
        let source = self
            .recorded_connection(port)
            .map(|connection| SourceStamp { connection, at });
        self.recorded_bytes_from(port, bytes, project, source);
    }

    #[cfg(test)]
    pub fn recorded_bytes_from(
        &self,
        port: &str,
        bytes: &[u8],
        project: &ProjectState,
        source: Option<SourceStamp>,
    ) {
        receive(&self.updates, port, bytes, Some(project), source);
    }

    #[cfg(test)]
    pub fn recorded_fence(&self) {
        // The dispatch loop flushes pending releases before it reads this marker.
        // This bounds a recorded frame without depending on thread timing or sleeps.
        let (tx, rx) = mpsc::channel();
        self.updates.send(Raw::Fence(tx)).unwrap();
        rx.recv_timeout(Duration::from_secs(2))
            .expect("recorded MIDI frame did not finish");
    }

    #[cfg(test)]
    pub fn recorded_tick(&self, at: Instant) {
        self.updates.send(Raw::Tick(at)).unwrap();
        self.recorded_fence();
    }

    /// Publishes the state after one UI event, even when its old-project body was discarded.
    pub fn consumed(&self, focus: Focus, sequence: u64) {
        self.shared.consumed(focus, sequence);
    }

    /// Replaces the learned assignments of `profile` in `group` in the MIDI thread.
    pub fn set_learned(&self, profile: &str, group: Group, map: HashMap<Key, Binding>) {
        let _ = self
            .updates
            .send(Raw::Learned(profile.into(), group.scope(), map));
    }

    pub(super) fn set_learned_groups(&self, maps: LearnedMaps) {
        let _ = self.updates.send(Raw::LearnedSet(maps));
    }

    /// The profile's roles live in the learned maps from now on (59).
    pub fn set_seeded(&self, profile: &str) {
        let _ = self.updates.send(Raw::Seeded(profile.into()));
    }

    pub fn set_hold_time(&self, time: Duration) {
        let _ = self.updates.send(Raw::HoldTime(time));
    }

    pub(super) fn set_pad_bank_shortcut(&self, enabled: bool) {
        self.shared.set_pad_bank_shortcut(enabled);
    }

    pub(super) fn pad_bank_shortcut_is_current(&self, setting: u64) -> bool {
        self.shared.pad_bank_shortcut() == Some(setting)
    }

    #[cfg(test)]
    pub(super) fn recorded_pad_bank(&self, port: &str, requested_at: Instant, held: bool) {
        self.updates
            .send(Raw::SmcPadBank {
                port: port.into(),
                requested_at,
                setting: self.shared.pad_bank_shortcut().unwrap_or_default(),
                held,
            })
            .unwrap();
    }
}

/// `[midi]` of the user config to per-group maps; bad entries are logged and skipped.
/// The new group keys (`pads_edit`, `chop_edit`, 28) are additive: an old config without
/// them loads unchanged.
pub fn parse_learned(cfg: &crate::config::Learned) -> LearnedMaps {
    let mut out = LearnedMaps::new();
    for (profile, screens) in cfg {
        for (screen, controls) in screens {
            let Some(group) = Group::from_key(screen) else {
                log::warn!("config [midi.{profile}]: unknown group {screen}");
                continue;
            };
            let scope = group.scope();
            let map = out.entry((profile.clone(), scope)).or_default();
            for (key, action) in controls {
                match key.parse::<Key>() {
                    Ok(k) => {
                        // the name before 76, so the learned row shows under its new name
                        let rename = |name: &str| match name {
                            "pad.scope_next" => "pad.mark_bank".to_string(),
                            other => other.to_string(),
                        };
                        let binding = match action {
                            crate::config::MidiBinding::Press(name) => Binding::Press(rename(name)),
                            crate::config::MidiBinding::Gesture { tap, hold } => Binding::Gesture {
                                tap: tap.as_deref().map(rename),
                                hold: rename(hold),
                            },
                        };
                        map.insert(k, binding);
                    }
                    Err(e) => log::warn!("config [midi.{profile}]: {e}"),
                }
            }
        }
    }
    out
}

pub fn apply_learned(profile: &mut Profile, learned: &LearnedMaps) {
    for ((name, scope), map) in learned {
        if *name == profile.name {
            profile.set_bindings(*scope, map.clone());
        }
    }
}

pub fn start(
    mut profiles: Vec<Profile>,
    learned: LearnedMaps,
    seeded: Vec<String>,
    commands: Option<rtrb::Producer<ProjectCommand>>,
    project: Option<Arc<ProjectState>>,
    shared: Arc<Shared>,
    settings: crate::config::MidiGesturesConfig,
) -> Midi {
    for p in profiles.iter_mut() {
        apply_learned(p, &learned);
        p.set_seeded(seeded.contains(&p.name));
    }
    let (raw_tx, raw_rx) = mpsc::channel();
    let (tx, rx) = mpsc::channel();
    shared.set_pad_bank_shortcut(settings.smc_pad_bank_opens_panel);
    raw_tx
        .send(Raw::HoldTime(Duration::from_millis(
            settings.hold_time_ms.into(),
        )))
        .expect("set initial MIDI hold time");
    let scan_tx = raw_tx.clone();
    let scan_project = project.clone();
    std::thread::Builder::new()
        .name("midi-scan".into())
        .spawn(move || scan(scan_tx, scan_project))
        .expect("spawn midi-scan");
    if !virtual_mode() {
        let setup_tx = raw_tx.clone();
        let setup_shared = shared.clone();
        std::thread::Builder::new()
            .name("smc-pad-setup".into())
            .spawn(move || smc_usb::watch(setup_tx, setup_shared))
            .expect("spawn smc-pad-setup");
    }
    let dispatch_shared = shared.clone();
    std::thread::Builder::new()
        .name("midi".into())
        .spawn(move || {
            dispatch(
                raw_rx,
                tx,
                profiles,
                learned,
                seeded,
                commands,
                project,
                dispatch_shared,
            )
        })
        .expect("spawn midi");
    Midi {
        events: rx,
        updates: raw_tx,
        shared,
        #[cfg(test)]
        recorded_connections: Default::default(),
    }
}

fn receive(
    tx: &Sender<Raw>,
    port: &str,
    bytes: &[u8],
    project: Option<&ProjectState>,
    source: Option<SourceStamp>,
) {
    if let Some(msg) = Msg::parse(bytes) {
        let mut stamp = Stamp::capture(project);
        stamp.source = source;
        let _ = tx.send(Raw::Msg(port.into(), msg, stamp));
    }
}

fn scan(tx: Sender<Raw>, project: Option<Arc<ProjectState>>) {
    let dev_virtual = virtual_mode();
    let mut connections: HashMap<String, (midir::MidiInputConnection<()>, Option<Connection>)> =
        HashMap::new();
    let scanner = match midir::MidiInput::new("ptsr scan") {
        Ok(s) => s,
        Err(e) => {
            log::error!("MIDI unavailable: {e}");
            return;
        }
    };
    loop {
        let ports: Vec<_> = scanner
            .ports()
            .into_iter()
            .filter_map(|p| scanner.port_name(&p).ok().map(|n| (n, p)))
            .filter(|(name, _)| {
                if name.contains("Midi Through") || name.contains("ptsr SMC-PAD setup") {
                    return false;
                }
                if name.contains("(virtual)") {
                    return dev_virtual && name.contains("SMC-PAD-Master");
                }
                !dev_virtual || (!name.contains("SMC-PAD") && !name.contains("SINCO"))
            })
            .collect();

        connections.retain(|name, (_, connection)| {
            let alive = ports.iter().any(|(n, _)| n == name);
            if !alive {
                log::info!("MIDI disconnected: {name}");
                let _ = tx.send(Raw::Disconnected(name.clone(), *connection));
            }
            alive
        });
        for (name, port) in ports {
            if connections.contains_key(&name) {
                continue;
            }
            let input = match midir::MidiInput::new("ptsr") {
                Ok(i) => i,
                Err(e) => {
                    log::warn!("MIDI: {e}");
                    continue;
                }
            };
            let (cb_tx, cb_name) = (tx.clone(), name.clone());
            let cb_project = project.clone();
            let connection = Connection::fresh();
            let conn = input.connect(
                &port,
                "ptsr-in",
                move |_, bytes, _| {
                    let at = Instant::now();
                    let source = connection.map(|connection| SourceStamp { connection, at });
                    receive(&cb_tx, &cb_name, bytes, cb_project.as_deref(), source);
                },
                (),
            );
            match conn {
                Ok(c) => {
                    log::info!("MIDI connected: {name}");
                    let _ = tx.send(Raw::Connected(name.clone(), connection));
                    connections.insert(name, (c, connection));
                }
                Err(e) => log::warn!("MIDI: cannot connect {name}: {e}"),
            }
        }
        std::thread::sleep(SCAN);
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch(
    rx: Receiver<Raw>,
    tx: Sender<UiEvent>,
    mut profiles: Vec<Profile>,
    mut learned: LearnedMaps,
    mut seeded: Vec<String>,
    mut commands: Option<rtrb::Producer<ProjectCommand>>,
    _project: Option<Arc<ProjectState>>,
    shared: Arc<Shared>,
) {
    let mut ui = UiSender {
        tx,
        last_sent: shared.input().0,
        origin: None,
    };
    let mut positions: HashMap<(String, u8, Key), u8> = HashMap::new();
    let mut fine: HashMap<String, (u8, Key)> = HashMap::new();
    let mut held_controls: HashSet<(String, u8, Key)> = HashSet::new();
    let mut gestures = Gestures::new(DEFAULT_HOLD);
    let mut defaults: HashMap<String, Profile> = HashMap::new();
    let mut pressed = DirectHolds::new();
    let mut ui_pressed = UiHolds::new();
    let mut next_ui_press = 0u64;
    let mut pending = VecDeque::new();
    let mut closed = false;
    let mut invalid_reported = false;
    let mut smc_ready = false;
    let mut smc_status: Option<String> = None;
    let mut pad_bank_after = HashMap::<String, Instant>::new();
    let mut hold = HoldRoute::default();
    let dev_virtual = virtual_mode();
    loop {
        while let Some(command) = pending.pop_front() {
            let Some(queue) = commands.as_mut() else {
                break;
            };
            if queue.is_abandoned() {
                pending.clear();
                break;
            }
            if let Err(rtrb::PushError::Full(command)) = queue.push(command) {
                pending.push_front(command);
                break;
            }
        }
        if closed && pending.is_empty() {
            break;
        }
        let raw = match rx.recv_timeout(Duration::from_millis(2)) {
            Ok(raw) => raw,
            Err(mpsc::RecvTimeoutError::Timeout) => match rx.try_recv() {
                Ok(raw) => raw,
                Err(mpsc::TryRecvError::Empty) => {
                    if shared.focus().is_ok() {
                        send_gesture_edges(&mut ui, gestures.due(Instant::now()));
                    } else {
                        gestures.reject_pending();
                    }
                    continue;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    closed = true;
                    continue;
                }
            },
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                closed = true;
                if !pending.is_empty() {
                    std::thread::sleep(Duration::from_millis(2));
                }
                continue;
            }
        };
        // Freeze the source of this raw event into every derived UI envelope.
        // Later events from another port cannot relabel queued work.
        ui.origin = match &raw {
            Raw::Msg(port, msg, stamp) => Some(Provenance {
                port: port.clone(),
                source: stamp.source,
                from_message: true,
                activity: Activity::None,
                key: Some(msg.key()),
            }),
            Raw::Connected(port, connection) | Raw::Disconnected(port, connection) => {
                Some(Provenance {
                    port: port.clone(),
                    source: connection.map(|connection| SourceStamp {
                        connection,
                        at: Instant::now(),
                    }),
                    from_message: false,
                    activity: Activity::None,
                    key: None,
                })
            }
            _ => None,
        };
        let (port, msg, stamp) = match raw {
            Raw::SmcSetup(result) => {
                let ready = result.is_ok();
                let message = match result {
                    Ok(()) => "SMC-PAD encoders ready".into(),
                    Err(error) => format!("SMC-PAD encoder setup. {error}"),
                };
                if smc_ready != ready || smc_status.as_deref() != Some(&message) {
                    ui.send(
                        Event::SetupStatus {
                            ready,
                            message: message.clone(),
                        },
                        None,
                    );
                }
                smc_ready = ready;
                smc_status = Some(message);
                if !ready {
                    hold.end_all(&mut ui);
                }
                continue;
            }
            Raw::SmcLost => {
                smc_ready = false;
                smc_status = None;
                hold.end_all(&mut ui);
                continue;
            }
            Raw::SmcPadBank {
                port,
                requested_at,
                setting,
                held,
            } => {
                crate::diag::input(|| {
                    format!(
                        "PAD BANK {} after {} ms",
                        if held { "held" } else { "released" },
                        requested_at.elapsed().as_millis()
                    )
                });
                if !held {
                    hold.end(&port, setting, &mut ui);
                } else if smc_ready
                    && shared.focus().is_ok()
                    && shared.pad_bank_shortcut() == Some(setting)
                    && pad_bank_after
                        .get(&port)
                        .is_some_and(|&cutoff| requested_at >= cutoff)
                    && hold.held.insert(port.clone())
                {
                    ui.send(
                        Event::SmcPadBank {
                            port,
                            setting,
                            held,
                        },
                        None,
                    );
                }
                continue;
            }
            Raw::Connected(port, _) => {
                let profile = profiles
                    .iter()
                    .find(|p| p.matches(&port))
                    .map_or("default".into(), |p| p.name.clone());
                if profile == "SMC-PAD" && port.contains("SMC-PAD-Master") {
                    pad_bank_after.insert(port.clone(), Instant::now());
                }
                ui.send(Event::Connected { port, profile }, None);
                continue;
            }
            Raw::Disconnected(port, _) => {
                pad_bank_after.remove(&port);
                hold.held.remove(&port);
                if port.contains("SMC-PAD-Master") {
                    smc_ready = false;
                    smc_status = None;
                }
                send_gesture_edges(&mut ui, gestures.disconnect(&port));
                positions.retain(|(device, _, _), _| *device != port);
                fine.remove(&port);
                held_controls.retain(|(device, _, _)| *device != port);
                // a held gate pad would hang without its note off
                pressed.retain(|(p, _, _), held| {
                    if *p == port {
                        for &(abs, epoch) in held.iter() {
                            pending.push_back(release_command(abs, epoch));
                            ui.send(
                                Event::PadRelease { abs, direct: true },
                                Some(Stamp::ready(epoch)),
                            );
                        }
                    }
                    *p != port
                });
                ui_pressed.retain(|(p, _, _), held| {
                    if *p == port {
                        for hold in held.iter() {
                            hold.released
                                .store(true, std::sync::atomic::Ordering::SeqCst);
                            ui.send(
                                Event::UiPadRelease {
                                    abs: hold.abs,
                                    id: hold.id,
                                },
                                Some(Stamp::ready(hold.epoch)),
                            );
                        }
                    }
                    *p != port
                });
                ui.send(Event::Disconnected { port }, None);
                continue;
            }
            Raw::Learned(name, scope, map) => {
                for p in profiles.iter_mut().chain(defaults.values_mut()) {
                    if p.name == name {
                        p.set_bindings(scope, map.clone());
                    }
                }
                learned.insert((name, scope), map);
                continue;
            }
            Raw::LearnedSet(maps) => {
                for profile in profiles.iter_mut().chain(defaults.values_mut()) {
                    apply_learned(profile, &maps);
                }
                learned.extend(maps);
                continue;
            }
            Raw::Seeded(name) => {
                for p in profiles.iter_mut().chain(defaults.values_mut()) {
                    if p.name == name {
                        p.set_seeded(true);
                    }
                }
                seeded.push(name);
                continue;
            }
            Raw::HoldTime(time) => {
                gestures.threshold = time;
                continue;
            }
            #[cfg(test)]
            Raw::Tick(at) => {
                if shared.focus().is_ok() {
                    send_gesture_edges(&mut ui, gestures.due(at));
                } else {
                    gestures.reject_pending();
                }
                continue;
            }
            #[cfg(test)]
            Raw::Fence(done) => {
                done.send(()).unwrap();
                continue;
            }
            Raw::Msg(port, msg, stamp) => (port, msg, stamp),
        };
        let hold_routes = smc_ready && shared.pad_bank_shortcut().is_some();
        let question = shared.focus().is_ok_and(|focus| focus.mode.question);
        let msg = hold.route(&port, msg, hold_routes, question);
        let (acknowledged, snapshot) = shared.input();
        let focus = match snapshot {
            Ok(focus) => {
                invalid_reported = false;
                focus
            }
            Err(bits) => {
                if !invalid_reported {
                    log::error!("MIDI rejected new input: invalid state {bits:#010x}");
                    invalid_reported = true;
                }
                gestures.reject_pending();
                let channel = match msg {
                    Msg::NoteOn { ch, .. } | Msg::NoteOff { ch, .. } | Msg::Cc { ch, .. } => ch,
                };
                if matches!(msg, Msg::NoteOff { .. } | Msg::Cc { value: 0, .. }) {
                    if fine.get(&port) == Some(&(channel, msg.key())) {
                        fine.remove(&port);
                    }
                    let key = (port.clone(), channel, msg.key());
                    send_gesture_edges(&mut ui, gestures.release_invalid(&key));
                }
                release_tracked(
                    &port,
                    msg,
                    &mut pressed,
                    &mut ui_pressed,
                    &mut pending,
                    &mut ui,
                );
                continue;
            }
        };
        let profile = match profiles.iter().find(|p| p.matches(&port)) {
            Some(p) => p,
            None => defaults.entry(port.clone()).or_insert_with(|| {
                let mut p = Profile::default_for(&port);
                apply_learned(&mut p, &learned);
                p.set_seeded(seeded.contains(&p.name));
                p
            }),
        };
        let waiting = ui.last_sent != acknowledged;
        let modal = (focus.mode.question || focus.mode.picker)
            && !focus.mode.text
            && !focus.mode.screen_learn
            && !focus.mode.midi_learn;
        let real_smc_encoder = !dev_virtual
            && port.contains("SMC-PAD-Master")
            && matches!(msg, Msg::Cc { cc: 30..=45, .. });
        if smc_ready
            && real_smc_encoder
            && let Msg::Cc { cc, value, .. } = msg
            && !(63..=65).contains(&value)
        {
            smc_ready = false;
            let message = format!("SMC-PAD encoder CC {cc} sent unexpected value {value}");
            smc_status = Some(message.clone());
            ui.send(
                Event::SetupStatus {
                    ready: false,
                    message,
                },
                None,
            );
        }
        let guarded_encoder = !(smc_ready || (dev_virtual && port.contains("(virtual)")))
            && port.contains("SMC-PAD-Master")
            && matches!(msg, Msg::Cc { cc: 30..=45, .. });
        let (resolved, activity) = if guarded_encoder {
            (None, Activity::None)
        } else {
            profile.resolve_input_mode(focus.screen, focus.mode, modal, msg)
        };
        if let Some(origin) = &mut ui.origin {
            origin.activity = activity;
        }
        let owner_input = match (activity, msg) {
            (Activity::Press, _) => true,
            (Activity::Movement, _) if matches!(resolved, Some(Resolved::Step { .. })) => true,
            (Activity::Movement, Msg::Cc { ch, value, .. }) => {
                positions.insert((port.clone(), ch, msg.key()), value) != Some(value)
            }
            _ => false,
        };
        let meaning = match &resolved {
            None if guarded_encoder => "SMC-PAD encoder setup incomplete".into(),
            None => "not assigned".into(),
            Some(Resolved::Action(a)) => format!("{a:?}"),
            Some(Resolved::PanelCell(cell)) => format!("panel cell {cell}"),
            Some(Resolved::Gesture { tap, hold }) => format!("tap {tap:?} / hold {hold:?}"),
            Some(Resolved::Param { control, param, .. }) => format!("{control} -> {param:?}"),
            Some(Resolved::Step {
                control,
                target,
                steps,
            }) => format!("{control} -> {target:?} {steps:+}"),
            Some(Resolved::Fine(on)) => format!("knob.fine {}", if *on { "on" } else { "off" }),
        };
        ui.send_input(
            Event::Message {
                port: port.clone(),
                profile: profile.name.clone(),
                msg,
                meaning,
            },
            Some(stamp),
            owner_input,
            None,
        );
        let channel = match msg {
            Msg::NoteOn { ch, .. } | Msg::NoteOff { ch, .. } | Msg::Cc { ch, .. } => ch,
        };
        let control = (port.clone(), channel, msg.key());
        if fine.get(&port) == Some(&(channel, msg.key()))
            && matches!(msg, Msg::NoteOff { .. } | Msg::Cc { value: 0, .. })
        {
            fine.remove(&port);
        }
        let relative = matches!(resolved, Some(Resolved::Step { .. }));
        let repeated = match msg {
            _ if relative => false,
            Msg::NoteOff { .. } | Msg::Cc { value: 0, .. } => {
                held_controls.remove(&control);
                false
            }
            Msg::NoteOn { .. } | Msg::Cc { .. } => !held_controls.insert(control),
        };
        let at = stamp.source.map_or_else(Instant::now, |source| source.at);
        if !relative && matches!(msg, Msg::NoteOff { .. } | Msg::Cc { value: 0, .. }) {
            let key = (port.clone(), channel, msg.key());
            send_gesture_edges(&mut ui, gestures.release(&key, at, stamp));
        } else if owner_input && (activity == Activity::Movement || !repeated) {
            let key = (port.clone(), channel, msg.key());
            send_gesture_edges(&mut ui, gestures.other(&port, &key));
        }
        if !relative
            && release_tracked(
                &port,
                msg,
                &mut pressed,
                &mut ui_pressed,
                &mut pending,
                &mut ui,
            )
        {
            continue;
        }
        if let Some(Resolved::Fine(on)) = &resolved {
            if *on {
                fine.insert(port, (channel, msg.key()));
            }
            continue;
        }
        if (focus.mode.text || focus.mode.screen_learn || focus.mode.midi_learn)
            && matches!(resolved, Some(Resolved::Action(Action::Confirm)))
        {
            continue;
        }
        if repeated
            && matches!(
                resolved,
                Some(Resolved::Action(Action::Confirm) | Resolved::Gesture { .. })
            )
        {
            continue;
        }
        let route = route_pad(focus);
        let allowed_while_ignored = matches!(resolved, Some(Resolved::PanelCell(_)))
            || (waiting && matches!(resolved, Some(Resolved::Action(_))))
            || (focus.mode.picker
                && (matches!(
                    resolved,
                    Some(Resolved::Step {
                        target: StepTarget::List(ListTarget::Projects),
                        ..
                    })
                ) || (port.contains("SMC-PAD-Master")
                    && matches!(resolved, Some(Resolved::Step { ref control, .. }) if matches!(control.as_str(), "e1" | "cc 30")))));
        if route == PadRoute::Ignore && !allowed_while_ignored {
            continue;
        }

        match resolved {
            Some(Resolved::PanelCell(cell)) => {
                ui.send(Event::PanelCell { port, cell }, Some(stamp));
            }
            Some(Resolved::Gesture { tap, hold }) => {
                gestures.press((port, channel, msg.key()), at, stamp, tap, hold);
            }
            Some(Resolved::Action(Action::PadPress { pad, velocity }))
                if waiting
                    || route == PadRoute::Ui
                    || (port.contains("SMC-PAD-Master")
                        && matches!(msg, Msg::NoteOn { note: 62 | 63, .. })) =>
            {
                let abs = focus.bank * PADS_PER_BANK + pad;
                let ui_press = stamp.ready.then(|| {
                    let held = ui_pressed.entry((port, channel, msg.key())).or_default();
                    if let Some(hold) = held
                        .iter()
                        .find(|hold| hold.abs == abs && hold.epoch == stamp.epoch)
                    {
                        return (hold.id, hold.released.clone());
                    }
                    let id = next_ui_press;
                    next_ui_press = next_ui_press.wrapping_add(1);
                    let released = Arc::new(AtomicBool::new(false));
                    held.push(UiHold {
                        abs,
                        epoch: stamp.epoch,
                        id,
                        released: released.clone(),
                    });
                    (id, released)
                });
                ui.send_input(
                    Event::PadHit {
                        abs,
                        velocity,
                        sound: if focus.mode.step_edit {
                            PadSound::EditUnattempted
                        } else {
                            PadSound::UiUnattempted { deferred: waiting }
                        },
                        ui_press: ui_press.as_ref().map(|(id, _)| *id),
                    },
                    Some(stamp),
                    false,
                    ui_press.map(|(_, released)| released),
                );
            }
            Some(Resolved::Action(Action::PadPress { pad, velocity })) => {
                let abs = focus.bank * PADS_PER_BANK + pad;
                let mut accepted = false;
                if stamp.ready && pending.is_empty() {
                    let command = ProjectCommand {
                        epoch: stamp.epoch,
                        ready_at_send: stamp.ready,
                        command: ProjectMessage::Command(Command::PadPress { pad: abs, velocity }),
                    };
                    if commands.as_mut().is_some_and(|c| c.push(command).is_ok()) {
                        accepted = true;
                        let held = pressed.entry((port, channel, msg.key())).or_default();
                        if !held.contains(&(abs, stamp.epoch)) {
                            held.push((abs, stamp.epoch));
                        }
                    }
                }
                ui.send(
                    Event::PadHit {
                        abs,
                        velocity,
                        sound: if accepted {
                            PadSound::DirectAccepted
                        } else {
                            PadSound::DirectRejected
                        },
                        ui_press: None,
                    },
                    Some(stamp),
                );
            }
            Some(Resolved::Action(Action::PadRelease { pad })) => {
                let abs = focus.bank * PADS_PER_BANK + pad;
                ui.send(Event::PadRelease { abs, direct: false }, Some(stamp));
            }
            Some(Resolved::Action(a)) => {
                ui.send(Event::Action(a), Some(stamp));
            }
            Some(Resolved::Param {
                control,
                param,
                value,
                takeover,
            }) => {
                ui.send(
                    Event::Param {
                        port,
                        control,
                        param,
                        value,
                        takeover,
                    },
                    Some(stamp),
                );
            }
            Some(Resolved::Step {
                control,
                target,
                steps,
            }) => {
                // Each detent is one step at any speed: the owner found accelerated turns too coarse.
                let is_fine = fine.contains_key(&port);
                ui.send(
                    Event::Step {
                        port,
                        control,
                        target,
                        steps,
                        fine: is_fine,
                    },
                    Some(stamp),
                );
            }
            Some(Resolved::Fine(_)) => {}
            None => {}
        }
    }
}

fn release_tracked(
    port: &str,
    msg: Msg,
    pressed: &mut DirectHolds,
    ui_pressed: &mut UiHolds,
    pending: &mut VecDeque<ProjectCommand>,
    ui: &mut UiSender,
) -> bool {
    if !matches!(msg, Msg::NoteOff { .. } | Msg::Cc { value: 0, .. }) {
        return false;
    }
    let channel = match msg {
        Msg::NoteOn { ch, .. } | Msg::NoteOff { ch, .. } | Msg::Cc { ch, .. } => ch,
    };
    let key = (port.to_string(), channel, msg.key());
    let direct = pressed.remove(&key);
    let held_ui = ui_pressed.remove(&key);
    let tracked = direct.is_some() || held_ui.is_some();
    if let Some(held) = direct {
        for (abs, epoch) in held {
            pending.push_back(release_command(abs, epoch));
            ui.send(
                Event::PadRelease { abs, direct: true },
                Some(Stamp::ready(epoch)),
            );
        }
    }
    if let Some(held) = held_ui {
        for hold in held {
            hold.released
                .store(true, std::sync::atomic::Ordering::SeqCst);
            ui.send(
                Event::UiPadRelease {
                    abs: hold.abs,
                    id: hold.id,
                },
                Some(Stamp::ready(hold.epoch)),
            );
        }
    }
    tracked
}

fn release_command(pad: usize, epoch: u64) -> ProjectCommand {
    ProjectCommand {
        epoch,
        ready_at_send: true,
        command: ProjectMessage::Command(Command::PadRelease { pad }),
    }
}

/// Reads every `*.toml` in `dir`; broken files are reported and skipped, so their device
/// falls back to the default profile.
pub fn load_profiles(dir: &str) -> (Vec<Profile>, Vec<String>) {
    let mut profiles = Vec::new();
    let mut errors = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    files.sort();
    for path in files {
        let file = path.display().to_string();
        match std::fs::read_to_string(&path)
            .map_err(|e| format!("{file}: {e}"))
            .and_then(|t| Profile::from_toml(&file, &t))
        {
            Ok(p) => {
                for warning in p.warnings() {
                    log::warn!("{warning}");
                }
                // a role naming no action does nothing: it is named, the rest loads (71)
                profiles.push(p)
            }
            Err(e) => {
                log::warn!("{e}");
                errors.push(e);
            }
        }
    }
    (profiles, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use input::{Screen, midi::Key};

    fn relative_turns() -> Vec<(String, i32, bool, Option<u64>)> {
        let profile = Profile::from_toml(
            "relative.toml",
            "[device]\nname = 'Relative'\nmatch_port = 'Relative'\n\
             [controls.knobs]\nk1 = { cc = 30, type = 'relative', encoding = 'twos' }\n\
             [controls.buttons]\nb1 = { note = 60 }\n\
             [screens.global]\nb1 = 'knob.fine'\n\
             [screens.pads]\nk1 = 'pad.volume'\n",
        )
        .unwrap();
        let (raw_tx, raw_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        let shared = Arc::new(Shared::default());
        let worker_shared = shared.clone();
        let worker = std::thread::spawn(move || {
            dispatch(
                raw_rx,
                tx,
                vec![profile],
                LearnedMaps::new(),
                Vec::new(),
                None,
                None,
                worker_shared,
            )
        });
        let at = Instant::now();
        let connection = Connection::fresh().unwrap();
        let send = |port: &str, bytes: &[u8], ms: u64| {
            raw_tx
                .send(Raw::Msg(
                    port.into(),
                    Msg::parse(bytes).unwrap(),
                    Stamp {
                        epoch: 0,
                        ready: true,
                        source: Some(SourceStamp {
                            connection,
                            at: at + Duration::from_millis(ms),
                        }),
                    },
                ))
                .unwrap();
        };
        send("Relative A", &[0xB0, 30, 1], 0);
        send("Relative A", &[0xB0, 30, 1], 10);
        send("Relative B", &[0xB0, 30, 127], 10);
        send("Relative A", &[0x90, 60, 127], 15);
        send("Relative A", &[0xB0, 30, 1], 20);
        send("Relative A", &[0x80, 60, 0], 25);
        send("Relative A", &[0xB0, 30, 1], 30);
        raw_tx
            .send(Raw::Disconnected("Relative A".into(), None))
            .unwrap();
        send("Relative A", &[0xB0, 30, 1], 40);
        let (done, finished) = mpsc::channel();
        raw_tx.send(Raw::Fence(done)).unwrap();
        finished.recv_timeout(Duration::from_secs(2)).unwrap();
        drop(raw_tx);
        worker.join().unwrap();
        let turns: Vec<_> = rx
            .try_iter()
            .filter_map(|event| {
                let port = event.origin()?.to_string();
                match event.event {
                    Event::Step { steps, fine, .. } => Some((port, steps, fine, event.sequence())),
                    _ => None,
                }
            })
            .collect();
        turns
    }

    /// 126 input-devices: "Fast turn uses single steps".
    #[test]
    fn fast_turn_uses_single_steps() {
        let turns = relative_turns();
        assert_eq!(turns.len(), 6);
        assert_eq!(
            turns
                .iter()
                .map(|(_, steps, fine, _)| (*steps, *fine))
                .collect::<Vec<_>>(),
            [
                (1, false),
                (1, false),
                (-1, false),
                (1, true),
                (1, false),
                (1, false)
            ]
        );
        assert!(turns.iter().all(|(_, _, _, sequence)| sequence.is_none()));
    }

    /// 90 input-devices: "Two devices keep separate step state".
    #[test]
    fn two_devices_keep_separate_step_state() {
        let turns = relative_turns();
        assert_eq!(turns[0].0, "Relative A");
        assert_eq!(turns[2], ("Relative B".into(), -1, false, turns[2].3));
        assert_eq!(turns[3].0, "Relative A");
        assert_eq!(turns[5], ("Relative A".into(), 1, false, turns[5].3));
    }

    /// 90 input-devices: "Relative bank step precedes a pad hit".
    #[test]
    fn relative_bank_step_precedes_a_pad_hit() {
        let profile = Profile::from_toml(
            "bank.toml",
            "[device]\nname = 'R'\nmatch_port = 'R'\n[controls.knobs]\nk1 = { cc = 30, type = 'relative', encoding = 'offset64' }\n[controls.pads]\np1 = { note = 36 }\n[screens.pads]\nk1 = 'bank'\n",
        ).unwrap();
        let (raw_tx, raw_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            dispatch(
                raw_rx,
                tx,
                vec![profile],
                LearnedMaps::new(),
                Vec::new(),
                None,
                None,
                Arc::new(Shared::default()),
            )
        });
        for bytes in [[0xB0, 30, 65], [0x90, 36, 100]] {
            raw_tx
                .send(Raw::Msg(
                    "R".into(),
                    Msg::parse(&bytes).unwrap(),
                    Stamp {
                        epoch: 0,
                        ready: true,
                        source: None,
                    },
                ))
                .unwrap();
        }
        drop(raw_tx);
        worker.join().unwrap();
        let events: Vec<_> = rx.try_iter().collect();
        let step = events
            .iter()
            .find(|e| matches!(e.event, Event::Step { .. }))
            .unwrap();
        let hit = events
            .iter()
            .find(|e| matches!(e.event, Event::PadHit { .. }))
            .unwrap();
        assert!(step.sequence().is_some());
        assert!(matches!(
            step.event,
            Event::Step {
                target: StepTarget::List(ListTarget::Bank),
                steps: 1,
                ..
            }
        ));
        assert!(matches!(
            hit.event,
            Event::PadHit {
                sound: PadSound::UiUnattempted { deferred: true },
                ..
            }
        ));
        assert!(hit.sequence().is_some());
    }

    #[test]
    fn invalid_state_release_clears_fine_mode() {
        let profile = Profile::from_toml(
            "fine.toml",
            "[device]\nname = 'R'\nmatch_port = 'R'\n[controls.knobs]\nk1 = { cc = 30, type = 'relative', encoding = 'twos' }\n[controls.buttons]\nb1 = { note = 60 }\n[screens.global]\nb1 = 'knob.fine'\n[screens.pads]\nk1 = 'pad.volume'\n",
        ).unwrap();
        let (raw_tx, raw_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        let shared = Arc::new(Shared::default());
        let worker_shared = shared.clone();
        let worker = std::thread::spawn(move || {
            dispatch(
                raw_rx,
                tx,
                vec![profile],
                LearnedMaps::new(),
                Vec::new(),
                None,
                None,
                worker_shared,
            )
        });
        let send = |bytes: [u8; 3]| {
            raw_tx
                .send(Raw::Msg(
                    "R".into(),
                    Msg::parse(&bytes).unwrap(),
                    Stamp {
                        epoch: 0,
                        ready: true,
                        source: None,
                    },
                ))
                .unwrap()
        };
        let fence = || {
            let (done, finished) = mpsc::channel();
            raw_tx.send(Raw::Fence(done)).unwrap();
            finished.recv_timeout(Duration::from_secs(2)).unwrap();
        };
        send([0x90, 60, 127]);
        fence();
        shared.inject_focus(u32::MAX);
        send([0x80, 60, 0]);
        fence();
        assert!(shared.publish(Focus::default()));
        send([0xB0, 30, 1]);
        fence();
        drop(raw_tx);
        worker.join().unwrap();
        assert!(rx.try_iter().any(|event| matches!(
            event.event,
            Event::Step {
                steps: 1,
                fine: false,
                ..
            }
        )));
    }

    #[test]
    fn raw_relative_detent_matches_keyboard_value() {
        let profile = Profile::from_toml(
            "value.toml",
            "[device]\nname = 'R'\nmatch_port = 'R'\n[controls.knobs]\nk1 = { cc = 30, type = 'relative', encoding = 'offset64' }\n[screens.pads]\nk1 = 'pad.volume'\n",
        ).unwrap();
        let (raw_tx, raw_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            dispatch(
                raw_rx,
                tx,
                vec![profile],
                LearnedMaps::new(),
                Vec::new(),
                None,
                None,
                Arc::new(Shared::default()),
            )
        });
        raw_tx
            .send(Raw::Msg(
                "R".into(),
                Msg::parse(&[0xB0, 30, 65]).unwrap(),
                Stamp {
                    epoch: 0,
                    ready: true,
                    source: None,
                },
            ))
            .unwrap();
        drop(raw_tx);
        worker.join().unwrap();
        let mut controller = crate::tests::test_app();
        for event in rx.try_iter() {
            controller.midi_event(event.event);
        }
        let mut keyboard = crate::tests::test_app();
        keyboard.apply(Action::EditValue {
            up: true,
            fine: true,
        });
        assert_eq!(controller.pads.volume_db[0], keyboard.pads.volume_db[0]);
        assert_eq!(controller.pads.volume_db[0], 1);
    }

    #[test]
    fn shared_focus_rejects_invalid_publication_and_keeps_its_default() {
        let shared = Shared::default();
        assert_eq!(shared.focus(), Ok(Focus::default()));
        let focus = Focus {
            screen: Screen::Fx,
            bank: 7,
            ..Focus::default()
        };
        assert!(shared.publish(focus));
        assert_eq!(shared.focus(), Ok(focus));
        assert!(!shared.publish(Focus { bank: 8, ..focus }));
        assert_eq!(shared.focus(), Ok(focus));
    }

    #[test]
    fn concurrent_publication_cannot_mix_message_profile_bank_and_pad_route() {
        use std::sync::atomic::{
            AtomicBool,
            Ordering::{Acquire, Release},
        };
        let ordinary = Focus::default();
        let chop = Focus {
            screen: Screen::Chop,
            bank: 7,
            mode: input::Mode {
                edit: true,
                marking: true,
                text: true,
                picker: true,
                midi_learn: true,
                question: true,
                file_in_hand: true,
                copy_in_hand: true,
                swap_in_hand: true,
                pattern_pads: false,
                screen_learn: false,
                erase_pads: false,
                step_edit: false,
                pattern_controls: false,
            },
        };
        let shared = Arc::new(Shared::default());
        let done = Arc::new(AtomicBool::new(false));
        let (writer_shared, writer_done) = (shared.clone(), done.clone());
        let writer = std::thread::spawn(move || {
            let mut writes = 0;
            while !writer_done.load(Acquire) {
                writer_shared.publish(if writes % 2 == 0 { chop } else { ordinary });
                writes += 1;
                if writes % 256 == 0 {
                    std::thread::yield_now();
                }
            }
        });
        let mut profile = Profile::default_for("snapshot");
        profile.set_learned(
            Scope::Screen(Screen::Pads),
            [(Key::Cc(80), "pad.1".into())].into(),
        );
        profile.set_learned(
            Scope::Edit(Screen::Chop),
            [(Key::Cc(80), "pad.3".into())].into(),
        );
        let (raw_tx, raw_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        let (producer, mut consumer) = rtrb::RingBuffer::new(1024);
        let dispatch_shared = shared.clone();
        let worker = std::thread::spawn(move || {
            dispatch(
                raw_rx,
                tx,
                vec![profile],
                LearnedMaps::new(),
                Vec::new(),
                Some(producer),
                None,
                dispatch_shared,
            )
        });
        let mut events = Vec::new();
        for _ in 0..1000 {
            raw_tx
                .send(Raw::Msg("snapshot".into(), knob(80, 80), test_stamp()))
                .unwrap();
            events.push(rx.recv_timeout(Duration::from_secs(1)).unwrap().event);
            let hit = rx.recv_timeout(Duration::from_secs(1)).unwrap();
            // This unit test retires its UI fixture before the next message. The
            // concurrent writer still races every profile/bank/route snapshot.
            if let Some(sequence) = hit.sequence() {
                shared.consumed(shared.focus().unwrap(), sequence);
            }
            events.push(hit.event);
        }
        drop(raw_tx);
        worker.join().unwrap();
        done.store(true, Release);
        writer.join().unwrap();
        assert_eq!(events.len(), 2000);
        for pair in events.as_chunks::<2>().0 {
            let Event::Message { meaning, .. } = &pair[0] else {
                panic!("missing input log")
            };
            let Event::PadHit {
                abs,
                velocity,
                sound,
                ui_press,
            } = &pair[1]
            else {
                panic!("missing pad hit")
            };
            assert_eq!(*velocity, 80.0 / 127.0);
            if meaning.contains("pad: 0,") {
                assert_eq!(
                    (*abs, *sound, *ui_press),
                    (0, PadSound::DirectAccepted, None)
                );
            } else {
                assert!(meaning.contains("pad: 2,"), "{meaning}");
                assert_eq!(
                    (*abs, *sound),
                    (114, PadSound::UiUnattempted { deferred: false })
                );
                assert!(ui_press.is_some());
            }
        }
        while let Ok(command) = consumer.pop() {
            assert!(matches!(command.command,
                ProjectMessage::Command(Command::PadPress { pad: 0, velocity })
                if velocity == 80.0 / 127.0));
        }
        assert!(matches!(shared.focus(), Ok(focus) if focus == ordinary || focus == chop));
    }

    #[test]
    fn numbered_work_wraps_and_publishes_focus_before_acknowledgement() {
        let shared = Shared::default();
        assert_eq!(shared.input(), (0, Ok(Focus::default())));
        shared.acknowledged.store(u64::MAX, Relaxed);
        let (tx, rx) = mpsc::channel();
        let mut sender = UiSender {
            tx,
            last_sent: u64::MAX,
            origin: None,
        };
        sender.send(Event::Action(Action::BankNext), None);
        let action = rx.recv().unwrap();
        assert_eq!(action.sequence(), Some(0));
        assert_ne!(shared.input().0, sender.last_sent);
        let focus = Focus {
            bank: 7,
            screen: Screen::Fx,
            ..Focus::default()
        };
        shared.consumed(focus, action.sequence().unwrap());
        assert_eq!(shared.input(), (0, Ok(focus)));
        sender.send(
            Event::PadHit {
                abs: 114,
                velocity: 0.5,
                sound: PadSound::UiUnattempted { deferred: true },
                ui_press: Some(1),
            },
            None,
        );
        let hit = rx.recv().unwrap();
        assert_eq!(hit.sequence(), Some(1));
        assert_ne!(
            shared.input().0,
            sender.last_sent,
            "UI hits keep their own barrier"
        );
        shared.consumed(Focus { bank: 8, ..focus }, 1);
        assert_eq!(
            shared.input(),
            (0, Ok(focus)),
            "invalid publication cannot acknowledge work"
        );
        shared.consumed(focus, 1);
        assert_eq!(shared.input(), (sender.last_sent, Ok(focus)));
    }

    #[test]
    fn direct_notifications_parameters_logs_and_releases_do_not_create_barriers() {
        let (tx, rx) = mpsc::channel();
        let mut sender = UiSender {
            tx,
            last_sent: 9,
            origin: None,
        };
        let events = [
            Event::PadHit {
                abs: 2,
                velocity: 0.5,
                sound: PadSound::DirectAccepted,
                ui_press: None,
            },
            Event::PadHit {
                abs: 2,
                velocity: 0.5,
                sound: PadSound::DirectRejected,
                ui_press: None,
            },
            Event::PadRelease {
                abs: 2,
                direct: true,
            },
            Event::UiPadRelease { abs: 2, id: 1 },
            Event::Message {
                port: "p".into(),
                profile: "p".into(),
                msg: knob(80, 101),
                meaning: "log".into(),
            },
            Event::Param {
                port: "p".into(),
                control: "cc80".into(),
                param: Param::PadVolume,
                value: 0.5,
                takeover: false,
            },
        ];
        for event in events {
            sender.send(event, None);
            assert_eq!(rx.recv().unwrap().sequence(), None);
            assert_eq!(sender.last_sent, 9);
        }
    }

    #[test]
    fn acknowledgement_protocol_keeps_atomic_source_order() {
        // Sequential postconditions cannot catch reversed atomic operations.
        // Keep a static regression gate on the actual production methods too.
        let code: String = include_str!("midi.rs")
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let input = code
            .split("fninput(&self)")
            .nth(1)
            .unwrap()
            .split("fnconsumed(")
            .next()
            .unwrap();
        assert!(
            input.find("acknowledged.load(Acquire)").unwrap() < input.find("self.focus()").unwrap()
        );
        let consumed = code
            .split("fnconsumed(&self,focus:Focus,sequence:u64)")
            .nth(1)
            .unwrap()
            .split("structStamp")
            .next()
            .unwrap();
        assert!(
            consumed.find("self.publish(focus)").unwrap()
                < consumed
                    .find(".store(sequence,std::sync::atomic::Ordering::Release)")
                    .unwrap()
        );
    }

    #[test]
    fn failed_ui_send_does_not_advance_the_sequence() {
        let (tx, rx) = mpsc::channel();
        drop(rx);
        let mut sender = UiSender {
            tx,
            last_sent: u64::MAX,
            origin: None,
        };
        sender.send(Event::Action(Action::BankNext), None);
        assert_eq!(sender.last_sent, u64::MAX);
    }

    #[test]
    fn hits_while_the_project_opens_and_old_releases_stay_in_their_epoch() {
        use std::sync::atomic::Ordering::Release;
        let state = Arc::new(ProjectState::default());
        let (raw_tx, raw_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        let (producer, mut consumer) = rtrb::RingBuffer::new(16);
        let project = state.clone();
        let thread = std::thread::spawn(move || {
            dispatch(
                raw_rx,
                tx,
                vec![],
                LearnedMaps::new(),
                vec![],
                Some(producer),
                Some(project),
                Arc::new(Shared::default()),
            )
        });
        let port = "controller".to_string();
        let sync = |msg| {
            raw_tx
                .send(Raw::Msg(port.clone(), msg, Stamp::capture(Some(&state))))
                .unwrap();
            while !matches!(
                rx.recv().unwrap().event,
                Event::PadHit { .. } | Event::PadRelease { .. }
            ) {}
        };
        sync(note_on(36));
        let first = consumer.pop().unwrap();
        assert!(matches!(
            first.command,
            ProjectMessage::Command(Command::PadPress { .. })
        ));
        state.epoch.store(1, Release);
        sync(Msg::NoteOff { ch: 1, note: 36 });
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        let release = loop {
            if let Ok(command) = consumer.pop() {
                break command;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "release was not delivered"
            );
            std::thread::sleep(Duration::from_millis(1));
        };
        assert_eq!(release.epoch, 0);
        assert!(release.ready_at_send);
        sync(note_on(36));
        assert!(consumer.pop().is_err());
        state.ready.store(1, Release);
        sync(Msg::NoteOff { ch: 1, note: 36 });
        assert!(consumer.pop().is_err());
        sync(note_on(36));
        let hit = consumer.pop().unwrap();
        assert_eq!(hit.epoch, 1);
        assert!(hit.ready_at_send);
        drop(raw_tx);
        thread.join().unwrap();
    }

    #[test]
    fn pad_hit_reports_audio_queue_acceptance() {
        for (queue, ready, full, expected) in [
            (false, true, false, false),
            (true, false, false, false),
            (true, true, true, false),
            (true, true, false, true),
        ] {
            let (raw_tx, raw_rx) = mpsc::channel();
            let (tx, rx) = mpsc::channel();
            let (mut producer, _consumer) = rtrb::RingBuffer::new(1);
            if full {
                producer.push(release_command(0, 0)).unwrap();
            }
            raw_tx
                .send(Raw::Msg(
                    "controller".into(),
                    note_on(36),
                    Stamp {
                        epoch: 0,
                        ready,
                        source: None,
                    },
                ))
                .unwrap();
            drop(raw_tx);
            dispatch(
                raw_rx,
                tx,
                vec![],
                LearnedMaps::new(),
                vec![],
                queue.then_some(producer),
                None,
                Arc::new(Shared::default()),
            );
            let expected_sound = if expected {
                PadSound::DirectAccepted
            } else {
                PadSound::DirectRejected
            };
            assert!(rx.try_iter().any(|event| matches!(event.event, Event::PadHit { abs: 0, sound, .. } if sound == expected_sound)));
        }
    }

    #[test]
    fn ui_events_keep_the_callback_epoch_and_drop_unready_hits() {
        use std::sync::atomic::Ordering::Release;
        let state = Arc::new(ProjectState::default());
        let (raw_tx, raw_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        raw_tx
            .send(Raw::Connected("controller".into(), None))
            .unwrap();
        raw_tx
            .send(Raw::Msg(
                "controller".into(),
                note_on(36),
                Stamp::capture(Some(&state)),
            ))
            .unwrap();
        state.epoch.store(1, Release);
        raw_tx
            .send(Raw::Msg(
                "controller".into(),
                note_on(37),
                Stamp::capture(Some(&state)),
            ))
            .unwrap();
        state.ready.store(1, Release);
        raw_tx
            .send(Raw::Msg(
                "controller".into(),
                note_on(38),
                Stamp::capture(Some(&state)),
            ))
            .unwrap();
        drop(raw_tx);
        dispatch(
            raw_rx,
            tx,
            vec![],
            LearnedMaps::new(),
            vec![],
            None,
            Some(state.clone()),
            Arc::new(Shared::default()),
        );
        let events: Vec<_> = rx
            .try_iter()
            .filter_map(|e| e.current(Some(&state)))
            .collect();
        assert!(matches!(events[0], Event::Connected { .. }));
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::PadHit { .. }))
                .count(),
            1
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::PadHit { abs: 2, .. }))
        );
        assert!(!events.iter().any(|e| matches!(
            e,
            Event::Message {
                msg: Msg::NoteOn { note: 36, .. },
                ..
            }
        )));
    }

    #[test]
    fn note_off_during_learn_releases_a_previously_held_pad() {
        let (raw_tx, raw_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        let (producer, mut consumer) = rtrb::RingBuffer::new(8);
        let shared = Arc::new(Shared::default());
        let worker_shared = shared.clone();
        let worker = std::thread::spawn(move || {
            dispatch(
                raw_rx,
                tx,
                vec![],
                LearnedMaps::new(),
                vec![],
                Some(producer),
                None,
                worker_shared,
            )
        });
        raw_tx
            .send(Raw::Msg("controller".into(), note_on(36), test_stamp()))
            .unwrap();
        while !matches!(rx.recv().unwrap().event, Event::PadHit { .. }) {}
        assert!(matches!(
            consumer.pop().unwrap().command,
            ProjectMessage::Command(Command::PadPress { .. })
        ));
        let mut focus = shared.focus().unwrap();
        focus.mode.screen_learn = true;
        assert!(shared.publish(focus));
        raw_tx
            .send(Raw::Msg(
                "controller".into(),
                Msg::NoteOff { ch: 1, note: 36 },
                test_stamp(),
            ))
            .unwrap();
        while !matches!(rx.recv().unwrap().event, Event::PadRelease { .. }) {}
        drop(raw_tx);
        worker.join().unwrap();
        assert!(matches!(
            consumer.pop().unwrap().command,
            ProjectMessage::Command(Command::PadRelease { pad: 0 })
        ));
    }

    #[test]
    fn queued_messages_keep_the_callback_stamp() {
        use std::sync::atomic::Ordering::Release;
        let state = Arc::new(ProjectState::default());
        let (raw_tx, raw_rx) = mpsc::channel();
        let (tx, _rx) = mpsc::channel();
        let (producer, mut consumer) = rtrb::RingBuffer::new(8);
        raw_tx
            .send(Raw::Msg(
                "controller".into(),
                note_on(36),
                Stamp::capture(Some(&state)),
            ))
            .unwrap();
        state.epoch.store(1, Release);
        raw_tx
            .send(Raw::Msg(
                "controller".into(),
                note_on(37),
                Stamp::capture(Some(&state)),
            ))
            .unwrap();
        state.ready.store(1, Release);
        drop(raw_tx);
        dispatch(
            raw_rx,
            tx,
            vec![],
            LearnedMaps::new(),
            vec![],
            Some(producer),
            Some(state),
            Arc::new(Shared::default()),
        );
        let hit = consumer.pop().unwrap();
        assert_eq!(hit.epoch, 0);
        assert!(hit.ready_at_send);
        assert!(consumer.pop().is_err());
    }

    #[test]
    fn full_queue_retries_release_after_repeated_hit_and_disconnect() {
        for disconnect in [false, true] {
            let (raw_tx, raw_rx) = mpsc::channel();
            let (tx, rx) = mpsc::channel();
            let (producer, mut consumer) = rtrb::RingBuffer::new(1);
            let thread = std::thread::spawn(move || {
                dispatch(
                    raw_rx,
                    tx,
                    vec![],
                    LearnedMaps::new(),
                    vec![],
                    Some(producer),
                    Some(Arc::new(ProjectState::default())),
                    Arc::new(Shared::default()),
                )
            });
            let port = "controller".to_string();
            raw_tx
                .send(Raw::Msg(port.clone(), note_on(36), test_stamp()))
                .unwrap();
            raw_tx
                .send(Raw::Msg(port.clone(), note_on(36), test_stamp()))
                .unwrap();
            if disconnect {
                raw_tx.send(Raw::Disconnected(port, None)).unwrap();
            } else {
                raw_tx
                    .send(Raw::Msg(
                        port,
                        Msg::NoteOff { ch: 1, note: 36 },
                        test_stamp(),
                    ))
                    .unwrap();
            }
            while !matches!(
                rx.recv_timeout(Duration::from_secs(1)).unwrap().event,
                Event::PadRelease { .. }
            ) {}
            assert!(matches!(
                consumer.pop().unwrap().command,
                ProjectMessage::Command(Command::PadPress { pad: 0, .. })
            ));
            drop(raw_tx);
            let deadline = std::time::Instant::now() + Duration::from_secs(1);
            let release = loop {
                if let Ok(command) = consumer.pop() {
                    break command;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "release was not delivered"
                );
                std::thread::sleep(Duration::from_millis(1));
            };
            assert!(matches!(
                release.command,
                ProjectMessage::Command(Command::PadRelease { pad: 0 })
            ));
            thread.join().unwrap();
        }
    }

    /// 28 midi-learn: an old config without the edit groups loads unchanged; the new
    /// `pads_edit` and `chop_edit` keys land in the edit scopes.
    #[test]
    fn old_config_without_edit_groups_loads_unchanged() {
        let old: crate::config::Learned = toml::from_str(
            "[xsession-pro.pads]\n\"cc 24\" = \"pad.volume\"\n\"note 43\" = \"bank.next\"\n",
        )
        .unwrap();
        let maps = parse_learned(&old);
        assert_eq!(maps.len(), 1);
        let map = maps
            .get(&("xsession-pro".into(), Scope::Screen(Screen::Pads)))
            .unwrap();
        assert_eq!(
            map.get(&Key::Cc(24)).map(Binding::primary),
            Some("pad.volume")
        );
        assert_eq!(
            map.get(&Key::Note(43)).map(Binding::primary),
            Some("bank.next")
        );

        let new: crate::config::Learned = toml::from_str(
            "[xsession-pro.pads]\n\"cc 24\" = \"pad.volume\"\n\n\
             [xsession-pro.pads_edit]\n\"cc 25\" = \"edit.param_2\"\n\n\
             [xsession-pro.chop_edit]\n\"cc 26\" = \"edit.param_3\"\n",
        )
        .unwrap();
        let maps = parse_learned(&new);
        assert_eq!(maps.len(), 3);
        let pads_edit = maps
            .get(&("xsession-pro".into(), Scope::Edit(Screen::Pads)))
            .unwrap();
        assert_eq!(
            pads_edit.get(&Key::Cc(25)).map(Binding::primary),
            Some("edit.param_2")
        );
        let chop_edit = maps
            .get(&("xsession-pro".into(), Scope::Edit(Screen::Chop)))
            .unwrap();
        assert_eq!(
            chop_edit.get(&Key::Cc(26)).map(Binding::primary),
            Some("edit.param_3")
        );
    }

    fn xsession() -> Profile {
        Profile::from_toml(
            "xsession-pro.toml",
            include_str!("../../../config/devices/xsession-pro.toml"),
        )
        .unwrap()
    }

    const XS: &str = "USB X-Session MIDI 1";

    fn test_stamp() -> Stamp {
        Stamp::ready(0)
    }

    fn gesture_profile() -> Profile {
        Profile::from_toml(
            "gesture.toml",
            "[device]\nname='Gesture'\nmatch_port='Gesture'\n\
             [controls.buttons]\nplay={note=60}\nrecord={note=61}\n\
             [controls.pads]\np1={note=36}\n\
             [screens.global]\nplay={tap='transport.toggle',hold='metronome.toggle'}\n\
             record={tap='record.output',hold='record.mic'}\n",
        )
        .unwrap()
    }

    fn gesture_stamp(at: Instant) -> Stamp {
        Stamp {
            source: Some(SourceStamp {
                connection: Connection::fresh().unwrap(),
                at,
            }),
            ..test_stamp()
        }
    }

    #[test]
    fn threshold_and_repeat_use_one_gesture() {
        let start = Instant::now();
        let key = ("Gesture".into(), 1, Key::Note(60));
        let mut gestures = Gestures::new(Duration::from_millis(300));
        gestures.press(
            key.clone(),
            start,
            gesture_stamp(start),
            Some(Action::TransportToggle),
            Action::MetronomeToggle,
        );
        gestures.press(
            key.clone(),
            start + Duration::from_millis(100),
            gesture_stamp(start),
            Some(Action::TransportToggle),
            Action::MetronomeToggle,
        );
        assert!(gestures.due(start + Duration::from_millis(299)).is_empty());
        let down = gestures.due(start + Duration::from_millis(300));
        assert!(matches!(
            down.as_slice(),
            [(_, GestureEdge::Down(Action::MetronomeToggle, _))]
        ));
        assert!(gestures.due(start + Duration::from_millis(400)).is_empty());
        let up = gestures.release(&key, start + Duration::from_millis(400), test_stamp());
        assert!(matches!(
            up.as_slice(),
            [(_, GestureEdge::Up(Action::MetronomeToggle))]
        ));
        assert!(
            gestures
                .release(&key, start + Duration::from_millis(500), test_stamp())
                .is_empty()
        );
    }

    #[test]
    fn play_tap_and_hold_use_different_actions() {
        let start = Instant::now();
        let (producer, _consumer) = rtrb::RingBuffer::new(32);
        let (midi, worker) = Midi::recorded(
            vec![gesture_profile()],
            producer,
            Arc::new(ProjectState::default()),
            Arc::new(Shared::default()),
        );
        midi.recorded_connect("Gesture");
        midi.recorded_bytes_at("Gesture", &[0x90, 60, 127], &ProjectState::default(), start);
        midi.recorded_bytes_at(
            "Gesture",
            &[0x80, 60, 0],
            &ProjectState::default(),
            start + Duration::from_millis(100),
        );
        midi.recorded_bytes_at(
            "Gesture",
            &[0x90, 60, 127],
            &ProjectState::default(),
            start + Duration::from_millis(200),
        );
        midi.recorded_tick(start + Duration::from_millis(500));
        midi.recorded_bytes_at(
            "Gesture",
            &[0x80, 60, 0],
            &ProjectState::default(),
            start + Duration::from_millis(600),
        );
        midi.recorded_fence();
        let actions: Vec<_> = midi
            .events
            .try_iter()
            .filter_map(|event| match event.event {
                Event::Action(action) => Some(action),
                _ => None,
            })
            .collect();
        assert_eq!(actions, [Action::TransportToggle, Action::MetronomeToggle]);
        drop(midi);
        worker.join().unwrap();
    }

    #[test]
    fn record_hold_ends_microphone_capture() {
        let start = Instant::now();
        let raw = vec![
            Raw::Msg("Gesture".into(), note_on(61), gesture_stamp(start)),
            Raw::Tick(start + DEFAULT_HOLD),
            Raw::Msg(
                "Gesture".into(),
                Msg::NoteOff { ch: 1, note: 61 },
                gesture_stamp(start + Duration::from_millis(400)),
            ),
        ];
        let (events, _) = dispatched(raw, vec![gesture_profile()], Shared::default());
        let actions: Vec<_> = events
            .into_iter()
            .filter_map(|event| match event {
                Event::Action(action) => Some(action),
                _ => None,
            })
            .collect();
        assert_eq!(actions, [Action::RecordMic(true), Action::RecordMic(false)]);
    }

    #[test]
    fn a_pad_on_another_channel_starts_the_hold() {
        let start = Instant::now();
        let raw = vec![
            Raw::Msg("Gesture".into(), note_on(60), gesture_stamp(start)),
            Raw::Msg(
                "Gesture".into(),
                Msg::NoteOn {
                    ch: 10,
                    note: 36,
                    velocity: 100,
                },
                gesture_stamp(start + Duration::from_millis(100)),
            ),
            Raw::Msg(
                "Gesture".into(),
                Msg::NoteOff { ch: 1, note: 60 },
                gesture_stamp(start + Duration::from_millis(150)),
            ),
        ];
        let (events, commands) = dispatched(raw, vec![gesture_profile()], Shared::default());
        let relevant: Vec<_> = events
            .into_iter()
            .filter_map(|event| match event {
                Event::Action(action) => Some(format!("{action:?}")),
                Event::PadHit { .. } => Some("pad".into()),
                _ => None,
            })
            .collect();
        assert_eq!(relevant, ["MetronomeToggle", "pad"]);
        assert!(matches!(
            commands.as_slice(),
            [Command::PadPress { pad: 0, .. }]
        ));
    }

    #[test]
    fn a_moving_knob_starts_a_hold_after_earlier_movement() {
        let start = Instant::now();
        let cc = |value| Msg::Cc {
            ch: 1,
            cc: 24,
            value,
        };
        let raw = vec![
            Raw::Msg("Gesture".into(), cc(10), gesture_stamp(start)),
            Raw::Msg(
                "Gesture".into(),
                note_on(60),
                gesture_stamp(start + Duration::from_millis(10)),
            ),
            Raw::Msg(
                "Gesture".into(),
                cc(10),
                gesture_stamp(start + Duration::from_millis(20)),
            ),
            Raw::Msg(
                "Gesture".into(),
                cc(11),
                gesture_stamp(start + Duration::from_millis(30)),
            ),
            Raw::Msg(
                "Gesture".into(),
                Msg::NoteOff { ch: 1, note: 60 },
                gesture_stamp(start + Duration::from_millis(40)),
            ),
        ];
        let (events, _) = dispatched(raw, vec![gesture_profile()], Shared::default());
        let actions: Vec<_> = events
            .into_iter()
            .filter_map(|event| match event {
                Event::Action(action) => Some(action),
                _ => None,
            })
            .collect();
        assert_eq!(actions, [Action::MetronomeToggle]);
    }

    #[test]
    fn two_ports_keep_separate_holds() {
        let start = Instant::now();
        let raw = vec![
            Raw::Msg("Gesture A".into(), note_on(60), gesture_stamp(start)),
            Raw::Msg("Gesture B".into(), note_on(60), gesture_stamp(start)),
            Raw::Msg(
                "Gesture A".into(),
                Msg::NoteOn {
                    ch: 10,
                    note: 36,
                    velocity: 100,
                },
                gesture_stamp(start + Duration::from_millis(100)),
            ),
            Raw::Msg(
                "Gesture B".into(),
                Msg::NoteOff { ch: 1, note: 60 },
                gesture_stamp(start + Duration::from_millis(150)),
            ),
            Raw::Msg(
                "Gesture A".into(),
                Msg::NoteOff { ch: 1, note: 60 },
                gesture_stamp(start + Duration::from_millis(200)),
            ),
        ];
        let (events, _) = dispatched(raw, vec![gesture_profile()], Shared::default());
        let actions: Vec<_> = events
            .into_iter()
            .filter_map(|event| match event {
                Event::Action(action) => Some(action),
                _ => None,
            })
            .collect();
        assert_eq!(actions, [Action::MetronomeToggle, Action::TransportToggle]);
    }

    #[test]
    fn disconnect_ends_an_active_hold() {
        let start = Instant::now();
        let raw = vec![
            Raw::Msg("Gesture".into(), note_on(61), gesture_stamp(start)),
            Raw::Tick(start + DEFAULT_HOLD),
            Raw::Msg(
                "Gesture".into(),
                note_on(60),
                gesture_stamp(start + Duration::from_millis(350)),
            ),
            Raw::Disconnected("Gesture".into(), None),
            Raw::Connected("Gesture".into(), None),
        ];
        let (events, _) = dispatched(raw, vec![gesture_profile()], Shared::default());
        let actions: Vec<_> = events
            .into_iter()
            .filter_map(|event| match event {
                Event::Action(action) => Some(action),
                _ => None,
            })
            .collect();
        assert_eq!(actions, [Action::RecordMic(true), Action::RecordMic(false)]);
    }

    #[test]
    fn missing_release_does_not_duplicate_a_hold() {
        let start = Instant::now();
        let raw = vec![
            Raw::Msg("Gesture".into(), note_on(61), gesture_stamp(start)),
            Raw::Tick(start + DEFAULT_HOLD),
            Raw::Msg(
                "Gesture".into(),
                note_on(61),
                gesture_stamp(start + Duration::from_millis(400)),
            ),
            Raw::Disconnected("Gesture".into(), None),
        ];
        let (events, _) = dispatched(raw, vec![gesture_profile()], Shared::default());
        let actions: Vec<_> = events
            .into_iter()
            .filter_map(|event| match event {
                Event::Action(action) => Some(action),
                _ => None,
            })
            .collect();
        assert_eq!(actions, [Action::RecordMic(true), Action::RecordMic(false)]);
    }

    #[test]
    fn invalid_state_keeps_original_release() {
        let start = Instant::now();
        let shared = Arc::new(Shared::default());
        let (producer, _consumer) = rtrb::RingBuffer::new(32);
        let (midi, worker) = Midi::recorded(
            vec![gesture_profile()],
            producer,
            Arc::new(ProjectState::default()),
            shared.clone(),
        );
        midi.recorded_connect("Gesture");
        midi.recorded_bytes_at("Gesture", &[0x90, 61, 127], &ProjectState::default(), start);
        midi.recorded_tick(start + DEFAULT_HOLD);
        shared.inject_focus(5);
        midi.recorded_bytes_at(
            "Gesture",
            &[0x80, 61, 0],
            &ProjectState::default(),
            start + Duration::from_millis(400),
        );
        midi.recorded_bytes_at("Gesture", &[0x90, 60, 127], &ProjectState::default(), start);
        midi.recorded_fence();
        let actions: Vec<_> = midi
            .events
            .try_iter()
            .filter_map(|event| match event.event {
                Event::Action(action) => Some(action),
                _ => None,
            })
            .collect();
        assert_eq!(actions, [Action::RecordMic(true), Action::RecordMic(false)]);
        drop(midi);
        worker.join().unwrap();
    }

    #[test]
    fn existing_hold_keeps_its_start_threshold() {
        let start = Instant::now();
        let ms = |n| start + Duration::from_millis(n);
        let raw = vec![
            Raw::Msg("Gesture".into(), note_on(60), gesture_stamp(ms(0))),
            Raw::HoldTime(Duration::from_millis(400)),
            Raw::Tick(ms(300)),
            Raw::Msg(
                "Gesture".into(),
                Msg::NoteOff { ch: 1, note: 60 },
                gesture_stamp(ms(350)),
            ),
            Raw::Msg("Gesture".into(), note_on(60), gesture_stamp(ms(500))),
            Raw::Tick(ms(850)),
            Raw::Msg(
                "Gesture".into(),
                Msg::NoteOff { ch: 1, note: 60 },
                gesture_stamp(ms(850)),
            ),
        ];
        let (events, _) = dispatched(raw, vec![gesture_profile()], Shared::default());
        let actions: Vec<_> = events
            .into_iter()
            .filter_map(|event| match event {
                Event::Action(action) => Some(action),
                _ => None,
            })
            .collect();
        assert_eq!(actions, [Action::MetronomeToggle, Action::TransportToggle]);
    }

    #[test]
    fn old_saved_action_stays_immediate() {
        let dir = std::env::temp_dir().join(format!("ptsr-old-midi-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        std::fs::write(
            &path,
            "[midi.Gesture.global]\n'note 60'='transport.toggle'\n",
        )
        .unwrap();
        let loaded = crate::config::load_from(&path);
        let learned = parse_learned(&loaded.config.midi);
        let mut profile = gesture_profile();
        apply_learned(&mut profile, &learned);
        profile.set_seeded(true);
        let raw = vec![Raw::Msg(
            "Gesture".into(),
            note_on(60),
            gesture_stamp(Instant::now()),
        )];
        let (events, _) = dispatched(raw, vec![profile], Shared::default());
        let relevant: Vec<_> = events
            .into_iter()
            .filter_map(|event| match event {
                Event::Action(action) => Some(action),
                _ => None,
            })
            .collect();
        assert_eq!(relevant, [Action::TransportToggle]);
        crate::config::save_section(&path, "knobs", &crate::config::KnobsConfig::default())
            .unwrap();
        assert_eq!(
            crate::config::load_from(&path).config.midi["Gesture"]["global"]["note 60"],
            "transport.toggle"
        );
        let saved: toml::Value = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            saved["midi"]["Gesture"]["global"]["note 60"].as_str(),
            Some("transport.toggle")
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    fn note_on(note: u8) -> Msg {
        Msg::NoteOn {
            ch: 1,
            note,
            velocity: 127,
        }
    }

    /// Runs the MIDI thread over recorded input, the UI state as `shared` holds it; returns
    /// what went to the UI and what went straight to the audio thread.
    fn dispatched(
        raw: Vec<Raw>,
        profiles: Vec<Profile>,
        shared: Shared,
    ) -> (Vec<Event>, Vec<Command>) {
        let (raw_tx, raw_rx) = mpsc::channel();
        for r in raw {
            raw_tx.send(r).unwrap();
        }
        drop(raw_tx);
        let (tx, rx) = mpsc::channel();
        let (producer, mut consumer) = rtrb::RingBuffer::new(256);
        let shared = Arc::new(shared);
        std::thread::spawn(move || {
            dispatch(
                raw_rx,
                tx,
                profiles,
                LearnedMaps::new(),
                Vec::new(),
                Some(producer),
                Some(Arc::new(ProjectState::default())),
                shared,
            );
        })
        .join()
        .unwrap();
        let mut commands = Vec::new();
        while let Ok(c) = consumer.pop() {
            if let ProjectMessage::Command(command) = c.command {
                commands.push(command);
            }
        }
        (rx.try_iter().map(|event| event.event).collect(), commands)
    }

    /// A control learned for `action` in GLOBAL, then `msgs` from it.
    fn learned_global(key: Key, action: &str, msgs: &[Msg], shared: Shared) -> Vec<Command> {
        let mut raw = vec![Raw::Learned(
            "X-Session Pro".into(),
            Scope::Global,
            [(key, action.into())].into(),
        )];
        raw.extend(msgs.iter().map(|m| Raw::Msg(XS.into(), *m, test_stamp())));
        dispatched(raw, vec![xsession()], shared).1
    }

    fn on_screen(code: u8, bank: usize) -> Shared {
        let shared = Shared::default();
        set_screen(&shared, code, bank);
        shared
    }

    fn set_screen(shared: &Shared, code: u8, bank: usize) {
        let mut focus = shared.focus().unwrap();
        focus.screen = Focus::unpack(u32::from(code)).unwrap().screen;
        focus.bank = bank;
        assert!(shared.publish(focus));
    }

    /// Scenario "Hit on any screen" (pad-playback): the button learned for "pad 1" plays
    /// pad 1 on the pads screen and on the files screen.
    #[test]
    fn hit_on_any_screen() {
        for screen in [0, 2] {
            let sent = learned_global(Key::Note(99), "pad.1", &[note_on(99)], on_screen(screen, 0));
            assert!(
                matches!(sent[..], [Command::PadPress { pad: 0, .. }]),
                "screen {screen}"
            );
        }
    }

    /// Scenario "Gate by release" (pad-playback): the note off and the CC 0 of the learned
    /// control release pad 1; the fade is `gate_stops_on_release_without_click`.
    #[test]
    fn gate_by_release() {
        let off = Msg::NoteOff { ch: 1, note: 99 };
        let sent = learned_global(Key::Note(99), "pad.1", &[note_on(99), off], on_screen(0, 0));
        assert!(matches!(
            sent[..],
            [
                Command::PadPress { pad: 0, .. },
                Command::PadRelease { pad: 0 }
            ]
        ));
        let cc = |value| Msg::Cc {
            ch: 1,
            cc: 80,
            value,
        };
        let sent = learned_global(Key::Cc(80), "pad.1", &[cc(127), cc(0)], on_screen(0, 0));
        assert!(matches!(
            sent[..],
            [
                Command::PadPress { pad: 0, .. },
                Command::PadRelease { pad: 0 }
            ]
        ));
    }

    /// Scenario "Banks change under the button" (pad-playback): with bank 3 open the
    /// control learned for "pad 2" plays pad 2 of bank 3.
    #[test]
    fn banks_change_under_the_button() {
        let sent = learned_global(Key::Note(99), "pad.2", &[note_on(99)], on_screen(0, 2));
        assert!(matches!(sent[..], [Command::PadPress { pad: 33, .. }]));
    }

    #[test]
    fn fx_screen_code_routes_old_page_knobs_to_slots() {
        for scope in [
            None,
            Some(Scope::Edit(Screen::Pads)),
            Some(Scope::Edit(Screen::Chop)),
            Some(Scope::Screen(Screen::Pads)),
        ] {
            for seeded in [false, true] {
                if seeded && scope.is_none() {
                    continue;
                }
                for editing in [false, true] {
                    let mut raw = Vec::new();
                    if let Some(scope) = scope {
                        raw.push(Raw::Learned(
                            "X-Session Pro".into(),
                            scope,
                            [
                                (Key::Cc(24), "edit.param_1".into()),
                                (Key::Cc(25), "edit.param_2".into()),
                                (Key::Cc(26), "edit.param_3".into()),
                            ]
                            .into(),
                        ));
                    }
                    if seeded {
                        raw.push(Raw::Seeded("X-Session Pro".into()));
                    }
                    raw.extend(
                        [(24, 0), (25, 64), (26, 127)]
                            .map(|(cc, value)| Raw::Msg(XS.into(), knob(cc, value), test_stamp())),
                    );
                    let shared = on_screen(4, 2);
                    let mut focus = shared.focus().unwrap();
                    focus.mode.edit = editing;
                    assert!(shared.publish(focus));
                    let (events, sent) = dispatched(raw, vec![xsession()], shared);
                    assert_eq!(
                        params(&events),
                        [
                            (Param::Slot(0), 0.0),
                            (Param::Slot(1), 64.0 / 127.0),
                            (Param::Slot(2), 1.0),
                        ],
                        "scope {scope:?}, seeded {seeded}, editing {editing}"
                    );
                    assert_eq!(events.len(), 6);
                    for event in events {
                        if let Event::Param {
                            port,
                            control,
                            param: Param::Slot(slot),
                            takeover,
                            ..
                        } = event
                        {
                            assert_eq!(port, XS);
                            assert_eq!(
                                control,
                                if scope.is_some() {
                                    format!("cc {}", 24 + slot)
                                } else {
                                    format!("k{}", 1 + slot)
                                }
                            );
                            assert!(takeover);
                        }
                    }
                    assert!(sent.is_empty(), "knobs must not send pad commands");
                }
            }
        }
    }

    #[test]
    fn release_a_pad_after_the_screen_changes() {
        for (press, release) in [
            (note_on(46), Msg::NoteOff { ch: 1, note: 46 }),
            (note_on(46), Msg::parse(&[0x90, 46, 0]).unwrap()),
            (knob(24, 127), knob(24, 0)),
        ] {
            let foreign_release = match release {
                Msg::NoteOff { note, .. } => Msg::NoteOff { ch: 2, note },
                Msg::Cc { cc, .. } => Msg::Cc {
                    ch: 2,
                    cc,
                    value: 0,
                },
                _ => unreachable!(),
            };
            let mut profile = xsession();
            profile.set_learned(Scope::Global, [(press.key(), "pad.1".into())].into());
            assert!(
                profile
                    .resolve_in(Screen::Pads, false, foreign_release)
                    .is_none()
            );
            assert!(
                profile
                    .resolve_in(Screen::Fx, false, foreign_release)
                    .is_none()
            );
            assert!(matches!(
                profile.resolve_in(Screen::Fx, false, press),
                Some(Resolved::Action(Action::PagePrev))
                    | Some(Resolved::Param {
                        param: Param::Slot(0),
                        ..
                    })
            ));
            let (raw_tx, raw_rx) = mpsc::channel();
            let (tx, rx) = mpsc::channel();
            let (producer, mut consumer) = rtrb::RingBuffer::new(1);
            let shared = Arc::new(on_screen(0, 1));
            let worker_shared = shared.clone();
            let worker = std::thread::spawn(move || {
                dispatch(
                    raw_rx,
                    tx,
                    vec![profile],
                    LearnedMaps::new(),
                    vec![],
                    Some(producer),
                    None,
                    worker_shared,
                );
            });
            let send = |port: &str, msg, stamp| {
                raw_tx.send(Raw::Msg(port.into(), msg, stamp)).unwrap();
                assert!(matches!(
                    rx.recv_timeout(Duration::from_secs(1)).unwrap().event,
                    Event::Message { msg: seen, .. } if seen == msg
                ));
                rx.recv_timeout(Duration::from_secs(1)).unwrap()
            };
            assert!(matches!(
                send(XS, press, test_stamp()).event,
                Event::PadHit {
                    abs: 16,
                    sound: PadSound::DirectAccepted,
                    ..
                }
            ));
            set_screen(&shared, 4, 2);
            let stale = Stamp {
                epoch: 7,
                ready: false,
                source: None,
            };
            raw_tx
                .send(Raw::Msg(XS.into(), foreign_release, stale))
                .unwrap();
            raw_tx.send(Raw::Connected(XS.into(), None)).unwrap();
            assert!(matches!(
                rx.recv_timeout(Duration::from_secs(1)).unwrap().event,
                Event::Message { msg, .. } if msg == foreign_release
            ));
            assert!(matches!(
                rx.recv_timeout(Duration::from_secs(1)).unwrap().event,
                Event::Connected { .. }
            ));
            assert!(rx.try_recv().is_err());
            let released = send(XS, release, stale);
            assert!(matches!(
                released.event,
                Event::PadRelease {
                    abs: 16,
                    direct: true
                }
            ));
            let stamp = released.stamp.unwrap();
            assert_eq!(stamp.epoch, 0);
            assert!(stamp.ready);
            assert!(matches!(
                send(PADS, note_on(36), test_stamp()).event,
                Event::PadHit {
                    abs: 32,
                    sound: PadSound::DirectRejected,
                    ..
                }
            ));
            raw_tx.send(Raw::Msg(XS.into(), release, stale)).unwrap();
            raw_tx
                .send(Raw::Msg(XS.into(), knob(25, 0), test_stamp()))
                .unwrap();
            raw_tx.send(Raw::Disconnected(XS.into(), None)).unwrap();
            drop(raw_tx);
            let hit = consumer.pop().unwrap();
            assert_eq!(hit.epoch, 0);
            assert!(hit.ready_at_send);
            assert!(matches!(
                hit.command,
                ProjectMessage::Command(Command::PadPress { pad: 16, .. })
            ));
            let deadline = std::time::Instant::now() + Duration::from_secs(1);
            let command = loop {
                if let Ok(command) = consumer.pop() {
                    break command;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "release was not delivered"
                );
                std::thread::sleep(Duration::from_millis(1));
            };
            assert_eq!(command.epoch, 0);
            assert!(command.ready_at_send);
            assert!(matches!(
                command.command,
                ProjectMessage::Command(Command::PadRelease { pad: 16 })
            ));
            worker.join().unwrap();
            assert!(consumer.pop().is_err());
            let events: Vec<_> = rx.try_iter().map(|event| event.event).collect();
            assert!(
                !events
                    .iter()
                    .any(|event| matches!(event, Event::PadRelease { .. }))
            );
            let expected = if matches!(release, Msg::Cc { .. }) {
                vec![(Param::Slot(0), 0.0), (Param::Slot(1), 0.0)]
            } else {
                vec![(Param::Slot(1), 0.0)]
            };
            assert_eq!(params(&events), expected);
        }
    }

    #[test]
    fn held_controls_keep_each_source_channel_until_release_or_disconnect() {
        for note in [false, true] {
            for disconnect in [false, true] {
                let press = |ch| {
                    if note {
                        Msg::NoteOn {
                            ch,
                            note: 36,
                            velocity: 127,
                        }
                    } else {
                        Msg::Cc {
                            ch,
                            cc: 80,
                            value: 127,
                        }
                    }
                };
                let release = |ch| {
                    if note {
                        Msg::NoteOff { ch, note: 36 }
                    } else {
                        Msg::Cc {
                            ch,
                            cc: 80,
                            value: 0,
                        }
                    }
                };
                let mut profile = Profile::default_for(XS);
                profile.set_learned(Scope::Global, [(press(1).key(), "pad.1".into())].into());
                let mut raw: Vec<_> = [1, 2]
                    .map(|ch| Raw::Msg(XS.into(), press(ch), test_stamp()))
                    .into();
                if disconnect {
                    raw.push(Raw::Disconnected(XS.into(), None));
                } else {
                    raw.extend([2, 1].map(|ch| Raw::Msg(XS.into(), release(ch), test_stamp())));
                }
                let (events, sent) = dispatched(raw, vec![profile], on_screen(0, 0));
                assert!(matches!(
                    sent[..],
                    [
                        Command::PadPress { pad: 0, .. },
                        Command::PadPress { pad: 0, .. },
                        Command::PadRelease { pad: 0 },
                        Command::PadRelease { pad: 0 },
                    ]
                ));
                assert_eq!(
                    events
                        .iter()
                        .filter(|event| matches!(
                            event,
                            Event::PadRelease {
                                abs: 0,
                                direct: true
                            }
                        ))
                        .count(),
                    2
                );
            }
        }
    }

    #[test]
    fn play_pads_on_the_fx_screen_across_screen_and_bank_switches() {
        for (before, after) in [(0, 4), (4, 0), (3, 4), (4, 2)] {
            let (raw_tx, raw_rx) = mpsc::channel();
            let (tx, rx) = mpsc::channel();
            let (producer, mut consumer) = rtrb::RingBuffer::new(16);
            let shared = Arc::new(on_screen(before, 0));
            let worker_shared = shared.clone();
            let worker = std::thread::spawn(move || {
                dispatch(
                    raw_rx,
                    tx,
                    vec![xsession()],
                    LearnedMaps::new(),
                    vec![],
                    Some(producer),
                    Some(Arc::new(ProjectState::default())),
                    worker_shared,
                );
            });
            let send = |msg| {
                raw_tx
                    .send(Raw::Msg(PADS.into(), msg, test_stamp()))
                    .unwrap();
                assert!(matches!(
                    rx.recv_timeout(Duration::from_secs(1)).unwrap().event,
                    Event::Message { msg: seen, .. } if seen == msg
                ));
                let event = rx.recv_timeout(Duration::from_secs(1)).unwrap();
                if let Some(sequence) = event.sequence() {
                    shared.consumed(shared.focus().unwrap(), sequence);
                }
                event.event
            };
            let hit = |velocity| Msg::NoteOn {
                ch: 1,
                note: 36,
                velocity,
            };
            let off = Msg::NoteOff { ch: 1, note: 36 };
            let first = send(hit(80));
            assert!(
                matches!(first, Event::PadHit { abs: 0, velocity, sound, ui_press }
                if velocity == 80.0 / 127.0
                    && sound == if before == 3 { PadSound::UiUnattempted { deferred: false } } else { PadSound::DirectAccepted }
                    && ui_press.is_some() == (before == 3))
            );
            let first_ui_press = match first {
                Event::PadHit { ui_press, .. } => ui_press,
                _ => unreachable!(),
            };
            set_screen(&shared, after, 2);
            let first_release = send(off);
            if let Some(original) = first_ui_press {
                assert!(matches!(first_release, Event::UiPadRelease { abs: 0, id }
                    if id == original));
            } else {
                assert!(matches!(
                    first_release,
                    Event::PadRelease {
                        abs: 0,
                        direct: true
                    }
                ));
            }
            assert!(matches!(
                send(hit(47)),
                Event::PadHit {
                    abs: 32,
                    sound: PadSound::DirectAccepted,
                    ..
                }
            ));
            set_screen(&shared, before, 1);
            assert!(matches!(
                send(off),
                Event::PadRelease {
                    abs: 32,
                    direct: true
                }
            ));
            assert!(matches!(
                send(off),
                Event::PadRelease {
                    abs: 16,
                    direct: false
                }
            ));
            raw_tx.send(Raw::Disconnected(PADS.into(), None)).unwrap();
            assert!(matches!(
                rx.recv_timeout(Duration::from_secs(1)).unwrap().event,
                Event::Disconnected { .. }
            ));
            drop(raw_tx);
            worker.join().unwrap();
            let sent: Vec<_> = std::iter::from_fn(|| consumer.pop().ok())
                .map(|command| {
                    assert_eq!(command.epoch, 0);
                    assert!(command.ready_at_send);
                    command.command
                })
                .collect();
            if before == 3 {
                // Chop candidates go through App, not the direct queue. The paired
                // offline driver also checks the actual sound and release of this hit.
                assert!(
                    matches!(sent[..], [
                    ProjectMessage::Command(Command::PadPress { pad: 32, velocity }),
                    ProjectMessage::Command(Command::PadRelease { pad: 32 }),
                ] if velocity == 47.0 / 127.0),
                    "{sent:?}"
                );
            } else {
                assert!(
                    matches!(
                        sent[..],
                        [
                            ProjectMessage::Command(Command::PadPress { pad: 0, velocity: first }),
                            ProjectMessage::Command(Command::PadRelease { pad: 0 }),
                            ProjectMessage::Command(Command::PadPress { pad: 32, velocity: second }),
                            ProjectMessage::Command(Command::PadRelease { pad: 32 }),
                        ] if first == 80.0 / 127.0 && second == 47.0 / 127.0
                    ),
                    "screen {before} -> {after}: {sent:?}"
                );
            }
            assert!(rx.try_recv().is_err(), "no stale pad events after release");
        }
    }

    fn connected(events: &[Event]) -> Vec<&str> {
        events
            .iter()
            .filter_map(|e| match e {
                Event::Connected { profile, .. } => Some(profile.as_str()),
                _ => None,
            })
            .collect()
    }

    fn params(events: &[Event]) -> Vec<(Param, f32)> {
        events
            .iter()
            .filter_map(|e| match e {
                Event::Param { param, value, .. } => Some((*param, *value)),
                _ => None,
            })
            .collect()
    }

    fn knob(cc: u8, value: u8) -> Msg {
        Msg::Cc { ch: 1, cc, value }
    }

    const PADS: &str = "SMC-PAD MIDI 1";

    /// Scenario "Two controllers" (input-devices): the pads of one controller play pads
    /// while the X-Session knob turns the pad volume, both at once.
    #[test]
    fn two_controllers() {
        let raw = vec![
            Raw::Connected(PADS.into(), None),
            Raw::Connected(XS.into(), None),
            Raw::Msg(PADS.into(), note_on(36), test_stamp()),
            Raw::Msg(XS.into(), knob(24, 100), test_stamp()),
            Raw::Msg(PADS.into(), note_on(37), test_stamp()),
        ];
        let (events, sent) = dispatched(raw, vec![xsession()], on_screen(0, 0));
        assert!(matches!(
            sent[..],
            [
                Command::PadPress { pad: 0, .. },
                Command::PadPress { pad: 1, .. }
            ]
        ));
        assert_eq!(params(&events), [(Param::PadVolume, 100.0 / 127.0)]);
        assert_eq!(connected(&events), ["default", "X-Session Pro"]);
    }

    /// Scenario "Controller pulled out during play" (input-devices): pulling a controller
    /// out sends only the release of its held pad (a one-shot plays on) and the disconnect;
    /// plugged in again, its profile acts again.
    #[test]
    fn controller_pulled_out_during_play() {
        let raw = vec![
            Raw::Connected(XS.into(), None),
            Raw::Connected(PADS.into(), None),
            Raw::Msg(PADS.into(), note_on(36), test_stamp()),
            Raw::Disconnected(PADS.into(), None),
            Raw::Disconnected(XS.into(), None),
            Raw::Connected(XS.into(), None),
            Raw::Msg(XS.into(), knob(24, 64), test_stamp()),
        ];
        let (events, sent) = dispatched(raw, vec![xsession()], on_screen(0, 0));
        assert!(matches!(
            sent[..],
            [
                Command::PadPress { pad: 0, .. },
                Command::PadRelease { pad: 0 }
            ]
        ));
        let gone = events
            .iter()
            .filter(|e| matches!(e, Event::Disconnected { .. }))
            .count();
        assert_eq!(gone, 2);
        assert_eq!(
            connected(&events),
            ["X-Session Pro", "default", "X-Session Pro"]
        );
        assert_eq!(params(&events), [(Param::PadVolume, 64.0 / 127.0)]);
    }

    /// Scenario "New controller" (input-devices): a device without a profile plays pads
    /// 1–16 from notes 36–51, and every message reaches the input debug list with what it
    /// does.
    #[test]
    fn new_controller() {
        let port = "Some Keys MIDI 1";
        let mut raw = vec![Raw::Connected(port.into(), None)];
        raw.extend((36..=52).map(|n| Raw::Msg(port.into(), note_on(n), test_stamp())));
        let (events, sent) = dispatched(raw, vec![xsession()], on_screen(0, 0));
        let pads: Vec<usize> = sent
            .iter()
            .filter_map(|c| match c {
                Command::PadPress { pad, .. } => Some(*pad),
                _ => None,
            })
            .collect();
        assert_eq!(pads, (0..16).collect::<Vec<_>>(), "note 52 plays nothing");
        let seen: Vec<&str> = events
            .iter()
            .filter_map(|e| match e {
                Event::Message { meaning, .. } => Some(meaning.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(seen.len(), 17, "every message is listed");
        assert!(seen[0].contains("PadPress"), "{}", seen[0]);
        assert_eq!(seen[16], "not assigned");
    }

    #[test]
    fn an_example_does_not_replace_a_working_profile() {
        let dir = std::env::temp_dir().join(format!("ptsr-profile-example-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("xsession-pro.toml"),
            include_str!("../../../config/devices/xsession-pro.toml"),
        )
        .unwrap();
        std::fs::write(
            dir.join("smc-pad.toml.example"),
            include_str!("../../../config/devices/smc-pad.toml.example"),
        )
        .unwrap();
        let (profiles, errors) = load_profiles(dir.to_str().unwrap());
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "X-Session Pro");
        let mut raw = vec![
            Raw::Connected(PADS.into(), None),
            Raw::Connected(XS.into(), None),
        ];
        raw.extend((36..=52).map(|note| Raw::Msg(PADS.into(), note_on(note), test_stamp())));
        raw.push(Raw::Msg(XS.into(), knob(24, 64), test_stamp()));
        let (events, sent) = dispatched(raw, profiles, on_screen(0, 0));
        assert_eq!(connected(&events), ["default", "X-Session Pro"]);
        let pads: Vec<_> = sent
            .iter()
            .filter_map(|command| match command {
                Command::PadPress { pad, .. } => Some(*pad),
                _ => None,
            })
            .collect();
        assert_eq!(pads, (0..16).collect::<Vec<_>>());
        assert_eq!(params(&events), [(Param::PadVolume, 64.0 / 127.0)]);
    }

    /// Scenario "Invalid working profiles retain default routing" (input-devices).
    /// Scenario "Profile with an error" (input-devices): a profile with `??` from the
    /// template is named with its file and line; the device plays with the default profile.
    #[test]
    fn profile_with_an_error() {
        let dir = std::env::temp_dir().join(format!("ptsr-profile-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("smc-pad.toml"),
            "[device]\nname = \"SMC-PAD\"\nmatch_port = \"SMC-PAD\"\nchannel = ??\n",
        )
        .unwrap();
        let (profiles, errors) = load_profiles(dir.to_str().unwrap());
        std::fs::remove_dir_all(&dir).ok();
        assert!(profiles.is_empty());
        let e = errors.first().expect("the error is named");
        assert!(e.contains("smc-pad.toml:4:"), "the file and the line: {e}");
        let raw = vec![
            Raw::Connected(PADS.into(), None),
            Raw::Msg(PADS.into(), note_on(36), test_stamp()),
        ];
        let (events, sent) = dispatched(raw, profiles, on_screen(0, 0));
        assert_eq!(connected(&events), ["default"]);
        assert!(matches!(sent[..], [Command::PadPress { pad: 0, .. }]));
    }

    /// Scenario "Reading the layout" (input-devices): a turned X-Session knob reaches the
    /// input debug list with its CC number, its value and what it does, or "not assigned".
    #[test]
    fn reading_the_layout() {
        let raw = vec![
            Raw::Connected(XS.into(), None),
            Raw::Msg(XS.into(), knob(24, 64), test_stamp()),
            Raw::Msg(XS.into(), knob(99, 10), test_stamp()),
        ];
        let (events, _) = dispatched(raw, vec![xsession()], on_screen(0, 0));
        let seen: Vec<String> = events
            .iter()
            .filter_map(|e| match e {
                Event::Message { msg, meaning, .. } => Some(format!("{msg}   -> {meaning}")),
                _ => None,
            })
            .collect();
        assert_eq!(seen.len(), 2);
        assert!(seen[0].starts_with("ch 1 cc 24 = 64   -> "), "{}", seen[0]);
        assert!(seen[0].contains("PadVolume"), "{}", seen[0]);
        assert_eq!(seen[1], "ch 1 cc 99 = 10   -> not assigned");
    }

    const SMC_MASTER: &str = "SINCO SMC-PAD-Master 28:1";
    const SMC_PRIVATE: &str = "SINCO SMC-PAD-Private 28:0";

    fn smc_profiles() -> Vec<Profile> {
        [
            (
                "smc-pad.toml",
                include_str!("../../../config/devices/smc-pad.toml"),
            ),
            (
                "smc-pad-private.toml",
                include_str!("../../../config/devices/smc-pad-private.toml"),
            ),
        ]
        .into_iter()
        .map(|(file, source)| Profile::from_toml(file, source).unwrap())
        .collect()
    }

    /// Scenario "Physical pad rows match the screen" (input-devices).
    /// Scenario "Two SMC-PAD ports do not double a hit" (input-devices).
    #[test]
    fn physical_pad_rows_match_the_screen_without_mirrored_hits() {
        let mut raw = vec![
            Raw::Connected(SMC_PRIVATE.into(), None),
            Raw::Connected(SMC_MASTER.into(), None),
        ];
        for note in 36..=51 {
            let velocity = note - 20;
            let on = Msg::parse(&[0x99, note, velocity]).unwrap();
            let off = if note % 2 == 0 {
                Msg::parse(&[0x89, note, 64]).unwrap()
            } else {
                Msg::parse(&[0x99, note, 0]).unwrap()
            };
            raw.push(Raw::Msg(SMC_PRIVATE.into(), on, test_stamp()));
            raw.push(Raw::Msg(SMC_MASTER.into(), on, test_stamp()));
            raw.push(Raw::Msg(SMC_PRIVATE.into(), off, test_stamp()));
            raw.push(Raw::Msg(SMC_MASTER.into(), off, test_stamp()));
        }
        let (events, sent) = dispatched(raw, smc_profiles(), on_screen(0, 0));
        assert_eq!(connected(&events), ["SMC-PAD Private", "SMC-PAD"]);
        let hits: Vec<_> = sent
            .iter()
            .filter_map(|command| match command {
                Command::PadPress { pad, velocity } => Some((*pad, *velocity)),
                _ => None,
            })
            .collect();
        let expected: Vec<_> = (36u8..=51)
            .map(|note| {
                let row = 3 - (note - 36) / 4;
                let pad = row * 4 + (note - 36) % 4;
                (pad as usize, f32::from(note - 20) / 127.0)
            })
            .collect();
        assert_eq!(hits, expected);
        assert_eq!(sent.len(), 32, "each hit has one release");
        for pair in sent.as_chunks::<2>().0 {
            assert!(
                matches!(pair, [Command::PadPress { pad: first, .. }, Command::PadRelease { pad: second }] if first == second)
            );
        }
    }

    /// Scenario "Second pad bank stays distinct" (input-devices).
    #[test]
    fn second_pad_bank_stays_distinct() {
        let raw = (52..=67)
            .map(|note| {
                Raw::Msg(
                    SMC_MASTER.into(),
                    Msg::parse(&[0x99, note, 100]).unwrap(),
                    test_stamp(),
                )
            })
            .collect();
        let (events, sent) = dispatched(raw, smc_profiles(), on_screen(0, 0));
        assert!(sent.is_empty());
        let cells: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::PanelCell { cell, .. } => Some(*cell),
                _ => None,
            })
            .collect();
        assert_eq!(
            cells,
            [13, 14, 15, 16, 9, 10, 11, 12, 5, 6, 7, 8, 1, 2, 3, 4]
        );
    }

    /// Scenario "PLAY tap and hold have separate results" (input-devices).
    #[test]
    fn smc_play_tap_and_hold_have_separate_results() {
        let (producer, _consumer) = rtrb::RingBuffer::new(32);
        let (midi, worker) = Midi::recorded(
            smc_profiles(),
            producer,
            Arc::new(ProjectState::default()),
            Arc::new(Shared::default()),
        );
        let project = ProjectState::default();
        let start = Instant::now();
        for (millis, bytes) in [
            (0, [0xB0, 27, 127]),
            (100, [0xB0, 27, 0]),
            (200, [0xB0, 27, 127]),
            (300, [0x99, 48, 100]),
            (350, [0x89, 48, 0]),
            (400, [0xB0, 27, 0]),
        ] {
            midi.recorded_bytes_at(
                SMC_MASTER,
                &bytes,
                &project,
                start + Duration::from_millis(millis),
            );
        }
        midi.recorded_fence();
        let events = midi
            .events
            .try_iter()
            .map(|event| event.event)
            .collect::<Vec<_>>();
        let actions = events
            .iter()
            .filter_map(|event| match event {
                Event::Action(action) => Some(*action),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actions, [Action::PlayContext, Action::MetronomeToggle]);
        let hold = events
            .iter()
            .position(|event| matches!(event, Event::Action(Action::MetronomeToggle)))
            .unwrap();
        let pad = events
            .iter()
            .position(|event| matches!(event, Event::PadHit { .. }))
            .unwrap();
        assert!(
            hold < pad,
            "a pad on channel 10 promotes the PLAY hold first"
        );
        drop(midi);
        worker.join().unwrap();
    }

    /// Scenario "STOP acts on press by default" (input-devices).
    #[test]
    fn smc_stop_acts_on_press_by_default() {
        let raw = [127, 0]
            .into_iter()
            .map(|value| {
                Raw::Msg(
                    SMC_MASTER.into(),
                    Msg::Cc {
                        ch: 1,
                        cc: 28,
                        value,
                    },
                    test_stamp(),
                )
            })
            .collect();
        let (events, sent) = dispatched(raw, smc_profiles(), on_screen(0, 0));
        assert!(sent.is_empty());
        let actions = events
            .iter()
            .filter_map(|event| match event {
                Event::Action(action) => Some(*action),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actions, [Action::StopContext]);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::PanelCell { .. }))
        );
    }

    /// Scenario "Local SHIFT cannot trigger a host command" (input-devices).
    /// Physical SHIFT behavior remains for the owner's hardware check.
    #[test]
    fn local_shift_cannot_trigger_a_host_command() {
        let profile = smc_profiles()
            .into_iter()
            .find(|profile| profile.name == "SMC-PAD")
            .unwrap();
        assert!(profile.controls().iter().all(|(name, _)| name != "shift"));
        assert_eq!(
            profile.resolve(
                Screen::Pads,
                Msg::Cc {
                    ch: 1,
                    cc: 49,
                    value: 127
                }
            ),
            None
        );
        assert!(matches!(
            profile.resolve(
                Screen::Pads,
                Msg::NoteOn {
                    ch: 10,
                    note: 48,
                    velocity: 100
                }
            ),
            Some(Resolved::Action(Action::PadPress { pad: 0, .. }))
        ));
    }

    /// 92 command-panel: "P7 and P8 answer a visible question on the first hit".
    #[test]
    fn p7_and_p8_answer_a_visible_question_on_the_first_hit() {
        for (note, binding) in [(62, "pad.1"), (63, "ui.confirm")] {
            let mut profile = smc_profiles()
                .into_iter()
                .find(|profile| profile.name == "SMC-PAD")
                .unwrap();
            profile.set_bindings(
                Scope::Global,
                [(Key::Note(note), Binding::Press(binding.into()))].into(),
            );
            profile.set_seeded(true);
            let (producer, _consumer) = rtrb::RingBuffer::new(32);
            let (midi, worker) = Midi::recorded(
                vec![profile],
                producer,
                Arc::new(ProjectState::default()),
                Arc::new(Shared::default()),
            );
            midi.recorded_bytes_at(
                SMC_MASTER,
                &[0x99, note, 100],
                &ProjectState::default(),
                Instant::now(),
            );
            midi.recorded_bytes_at(
                SMC_MASTER,
                &[0x89, note, 0],
                &ProjectState::default(),
                Instant::now() + Duration::from_millis(10),
            );
            midi.recorded_fence();
            let mut app = crate::tests::test_app();
            app.ask = Some(crate::Ask::ClearPattern { lines: Vec::new() });
            for event in midi.events.try_iter() {
                app.midi_ui_event(event);
            }
            assert!(app.ask.is_none(), "note {note}");
            assert!(!app.panel.visible(), "note {note}");
            assert!(
                !app.sent
                    .iter()
                    .any(|command| matches!(command, Command::PadPress { .. }))
            );
            drop(midi);
            worker.join().unwrap();
        }
    }

    /// Scenario "PLAY and STOP do not answer a question" (command-panel).
    #[test]
    fn play_and_stop_do_not_answer_a_question() {
        for (cc, command) in [
            (27, Command::TransportPlayContext),
            (28, Command::TransportStopContext),
        ] {
            for owner_confirm in [false, true] {
                let mut profile = smc_profiles()
                    .into_iter()
                    .find(|profile| profile.name == "SMC-PAD")
                    .unwrap();
                if owner_confirm {
                    profile.set_bindings(
                        Scope::Global,
                        [(Key::Cc(cc), Binding::Press("ui.confirm".into()))].into(),
                    );
                    profile.set_seeded(true);
                }
                let (producer, _consumer) = rtrb::RingBuffer::new(32);
                let (midi, worker) = Midi::recorded(
                    vec![profile],
                    producer,
                    Arc::new(ProjectState::default()),
                    Arc::new(Shared::default()),
                );
                let now = Instant::now();
                midi.recorded_bytes_at(SMC_MASTER, &[0xB0, cc, 127], &ProjectState::default(), now);
                midi.recorded_bytes_at(
                    SMC_MASTER,
                    &[0xB0, cc, 0],
                    &ProjectState::default(),
                    now + Duration::from_millis(20),
                );
                midi.recorded_fence();
                let mut app = crate::tests::test_app();
                app.ask = Some(crate::Ask::ClearPattern { lines: Vec::new() });
                for event in midi.events.try_iter() {
                    app.midi_ui_event(event);
                }
                assert!(app.ask.is_some(), "cc {cc}, owner_confirm {owner_confirm}");
                assert_eq!(
                    app.sent.iter().any(
                        |sent| std::mem::discriminant(sent) == std::mem::discriminant(&command)
                    ),
                    !owner_confirm,
                );
                drop(midi);
                worker.join().unwrap();
            }
        }
    }

    /// Scenario "Question shows both controller answers" (screen-help).
    #[test]
    fn question_shows_both_controller_answers() {
        for visible in [false, true] {
            let (producer, _consumer) = rtrb::RingBuffer::new(32);
            let (midi, worker) = Midi::recorded(
                smc_profiles(),
                producer,
                Arc::new(ProjectState::default()),
                Arc::new(Shared::default()),
            );
            midi.recorded_connect(SMC_MASTER);
            midi.recorded_fence();
            let mut app = crate::tests::test_app();
            for event in midi.events.try_iter() {
                app.midi_ui_event(event);
            }
            if visible {
                app.panel.press(SMC_MASTER, 1);
            }
            app.ask = Some(crate::Ask::ClearPattern {
                lines: vec!["Keep pattern?".into(), "Enter to clear".into()],
            });
            let (keymap, _) = input::Keymap::from_toml(crate::DEFAULT_KEYMAP).unwrap();
            let question = app.question_view(&keymap).unwrap();
            assert!(
                question
                    .answers
                    .iter()
                    .any(|answer| answer.contains("P11: keep · P12: clear"))
            );
            assert!(question.answers.iter().any(|answer| answer.contains("KB ")));
            assert!(
                !question
                    .answers
                    .iter()
                    .any(|answer| answer.contains("PLAY") || answer.contains("STOP"))
            );
            assert_eq!(question.body, ["Keep pattern?"]);
            drop(midi);
            worker.join().unwrap();
        }
    }

    /// Scenario "Help names panel entry and current cells" (screen-help).
    #[test]
    fn help_names_panel_entry_and_current_cells() {
        let (producer, _consumer) = rtrb::RingBuffer::new(32);
        let (midi, worker) = Midi::recorded(
            smc_profiles(),
            producer,
            Arc::new(ProjectState::default()),
            Arc::new(Shared::default()),
        );
        midi.recorded_connect(SMC_MASTER);
        midi.recorded_fence();
        let mut app = crate::tests::test_app();
        for event in midi.events.try_iter() {
            app.midi_ui_event(event);
        }
        let (keymap, _) = input::Keymap::from_toml(crate::DEFAULT_KEYMAP).unwrap();
        let before = app.full_help_recording_keys(&keymap, true, None);
        assert!(
            before
                .iter()
                .any(|(_, text)| text.contains("Hold PAD BANK to show the panel"))
        );
        assert!(before.iter().any(|(_, text)| text.contains("P13 edit")));
        app.panel.press(SMC_MASTER, 1);
        app.panel.move_view(-1);
        let after = app.full_help_recording_keys(&keymap, true, None);
        assert!(after.iter().any(|(_, text)| text.contains("View: screens")));
        assert!(after.iter().any(|(_, text)| text.contains("P13 pads")));
        assert!(
            after
                .iter()
                .any(|(_, text)| text.contains("P3 redo unavailable"))
        );
        assert!(
            after
                .iter()
                .any(|(key, _)| !key.is_empty() && key != "SMC-PAD" && key != "SMC cells")
        );
        drop(midi);
        worker.join().unwrap();
    }

    #[test]
    fn controller_view_names_the_visible_panel_cells() {
        let mut app = crate::tests::test_app();
        app.profiles = smc_profiles();
        app.midi_event(Event::Connected {
            port: SMC_MASTER.into(),
            profile: "SMC-PAD".into(),
        });
        app.help_overlay.size = (120, 40);
        app.panel.press(SMC_MASTER, 1);
        let (keymap, _) = input::Keymap::from_toml(crate::DEFAULT_KEYMAP).unwrap();
        app.controller_start();
        let view = app.controller_view(&keymap);
        let tui::ControllerBody::Drawing { rows, .. } = view.body else {
            panic!("SMC-PAD drawing missing");
        };
        assert_eq!(rows[4][4].as_ref().unwrap().label, "[edit]");
        assert_eq!(rows[7][6].as_ref().unwrap().label, "[redo]");
        assert!(rows[7][6].as_ref().unwrap().free);
        assert!(rows[7][7].as_ref().unwrap().free);
    }

    /// Scenario "Two second-bank presses keep input order" (command-panel).
    /// Scenario "A destructive cell cannot confirm itself" (command-panel).
    /// Scenario "A panel action followed by P8 confirms only the question" (command-panel).
    #[test]
    fn a_panel_action_followed_by_p8_confirms_only_the_question() {
        let (producer, _consumer) = rtrb::RingBuffer::new(32);
        let (midi, worker) = Midi::recorded(
            smc_profiles(),
            producer,
            Arc::new(ProjectState::default()),
            Arc::new(Shared::default()),
        );
        let start = Instant::now();
        for (step, note) in [64, 63, 66, 63].into_iter().enumerate() {
            midi.recorded_bytes_at(
                SMC_MASTER,
                &[0x99, note, 100],
                &ProjectState::default(),
                start + Duration::from_millis(step as u64 * 20),
            );
            midi.recorded_bytes_at(
                SMC_MASTER,
                &[0x89, note, 0],
                &ProjectState::default(),
                start + Duration::from_millis(step as u64 * 20 + 10),
            );
        }
        midi.recorded_fence();
        let mut app = crate::tests::test_app();
        app.pads.file[0] = Some("first.wav".into());
        app.pads.file[1] = Some("second.wav".into());
        app.pads.marked.insert(0);
        for event in midi.events.try_iter() {
            app.midi_ui_event(event);
        }
        assert!(app.ask.is_none());
        assert!(app.pads.file[0].is_none());
        assert_eq!(
            app.pads.file[1].as_deref(),
            Some(std::path::Path::new("second.wav"))
        );
        assert!(app.panel.visible());
        assert_eq!(app.panel.page(), 1);
        drop(midi);
        worker.join().unwrap();
    }

    /// 92 command-panel: "First-bank hit leaves the panel and keeps its Gate".
    #[test]
    fn first_bank_hit_leaves_the_panel_and_keeps_its_gate() {
        let (producer, mut consumer) = rtrb::RingBuffer::new(32);
        let (midi, worker) = Midi::recorded(
            smc_profiles(),
            producer,
            Arc::new(ProjectState::default()),
            Arc::new(Shared::default()),
        );
        let start = Instant::now();
        for (step, bytes) in [
            [0x99, 64, 100],
            [0x89, 64, 0],
            [0x99, 48, 100],
            [0x89, 48, 0],
        ]
        .into_iter()
        .enumerate()
        {
            midi.recorded_bytes_at(
                SMC_MASTER,
                &bytes,
                &ProjectState::default(),
                start + Duration::from_millis(step as u64 * 10),
            );
        }
        midi.recorded_fence();
        let events: Vec<_> = midi.events.try_iter().collect();
        assert!(events.iter().any(|event| matches!(
            event.event,
            Event::PadHit {
                abs: 0,
                sound: PadSound::UiUnattempted { deferred: true },
                ..
            }
        )));
        assert!(
            events
                .iter()
                .any(|event| matches!(event.event, Event::UiPadRelease { abs: 0, .. }))
        );
        let mut app = crate::tests::test_app();
        app.pads.gate[0] = true;
        for event in events {
            app.midi_ui_event(event);
        }
        assert!(!app.panel.visible());
        let sent: Vec<_> = std::iter::from_fn(|| consumer.pop().ok())
            .map(|command| command.command)
            .collect();
        assert!(sent.is_empty(), "{sent:?}");
        assert!(matches!(
            app.sent.as_slice(),
            [Command::PadPress { pad: 0, .. }]
        ));
        drop(midi);
        worker.join().unwrap();
    }

    /// Scenario "Absolute bytes cannot become false steps" (input-devices).
    #[test]
    fn absolute_bytes_cannot_become_false_steps() {
        let start = Instant::now();
        let connection = Connection::fresh().unwrap();
        let stamp = |ms| Stamp {
            epoch: 0,
            ready: true,
            source: Some(SourceStamp {
                connection,
                at: start + Duration::from_millis(ms),
            }),
        };
        let raw = vec![
            Raw::SmcSetup(Err("read timed out".into())),
            Raw::Msg(SMC_MASTER.into(), knob(31, 127), stamp(0)),
            Raw::SmcSetup(Ok(())),
            Raw::Msg(SMC_MASTER.into(), knob(31, 65), stamp(100)),
            Raw::Msg(SMC_MASTER.into(), knob(31, 63), stamp(200)),
            Raw::Msg(SMC_MASTER.into(), knob(31, 64), stamp(300)),
        ];
        let (events, sent) = dispatched(raw, smc_profiles(), on_screen(0, 0));
        assert!(sent.is_empty());
        let steps: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Step { steps, .. } => Some(*steps),
                _ => None,
            })
            .collect();
        assert_eq!(steps, [1, -1]);
        assert!(events.iter().any(|event| matches!(event, Event::Message { meaning, .. } if meaning == "SMC-PAD encoder setup incomplete")));
    }

    /// Scenario "A wrong preset gives its number and blocks knob turns" (input-devices).
    #[test]
    fn a_wrong_preset_gives_its_number_and_blocks_knob_turns() {
        let message = "Preset 6 is active. Select preset 5 to use the knobs.";
        let raw = vec![
            Raw::SmcSetup(Ok(())),
            Raw::SmcSetup(Err(message.into())),
            Raw::Msg(SMC_MASTER.into(), knob(37, 65), test_stamp()),
            Raw::SmcSetup(Ok(())),
            Raw::Msg(SMC_MASTER.into(), knob(37, 65), test_stamp()),
        ];
        let (events, _) = dispatched(raw, smc_profiles(), on_screen(0, 0));
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::SetupStatus {
            ready: false, message: shown
        } if shown == &format!("SMC-PAD encoder setup. {message}")))
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::Step { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn unexpected_real_encoder_value_disables_steps() {
        let raw = vec![
            Raw::SmcSetup(Ok(())),
            Raw::Msg(SMC_MASTER.into(), knob(31, 2), test_stamp()),
            Raw::Msg(SMC_MASTER.into(), knob(31, 65), test_stamp()),
            Raw::SmcSetup(Ok(())),
            Raw::Msg(SMC_MASTER.into(), knob(31, 65), test_stamp()),
        ];
        let (events, _) = dispatched(raw, smc_profiles(), on_screen(0, 0));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::Step { .. }))
                .count(),
            1
        );
        assert!(events.iter().any(|event| matches!(event, Event::SetupStatus { ready: false, message } if message.contains("unexpected value 2"))));
    }

    #[test]
    fn both_encoder_banks_keep_their_cc_identity() {
        let profile = smc_profiles()
            .into_iter()
            .find(|profile| profile.name == "SMC-PAD")
            .unwrap();
        for (index, cc) in (30..=45).enumerate() {
            let name = if index < 8 {
                format!("e{}", index + 1)
            } else {
                format!("f{}", index - 7)
            };
            assert_eq!(profile.kind_of(&name), Some(input::midi::ControlKind::Knob));
            let resolved = profile.resolve(Screen::Pads, knob(cc, 65));
            if profile.control_action(Screen::Pads, false, &name).is_some() {
                assert!(
                    matches!(resolved, Some(Resolved::Step { control, steps: 1, .. }) if control == name)
                );
            } else {
                assert_eq!(resolved, None);
            }
            assert_eq!(profile.resolve(Screen::Pads, knob(cc, 64)), None);
        }
    }

    /// Scenario "X-Session remains independent" (input-devices).
    #[test]
    fn xsession_remains_independent() {
        let raw = vec![
            Raw::Connected(SMC_PRIVATE.into(), None),
            Raw::Connected(SMC_MASTER.into(), None),
            Raw::Connected(XS.into(), None),
            Raw::Msg(
                SMC_PRIVATE.into(),
                Msg::parse(&[0x99, 48, 80]).unwrap(),
                test_stamp(),
            ),
            Raw::Msg(
                SMC_MASTER.into(),
                Msg::parse(&[0x99, 48, 80]).unwrap(),
                test_stamp(),
            ),
            Raw::Msg(XS.into(), knob(24, 100), test_stamp()),
            Raw::Disconnected(SMC_MASTER.into(), None),
        ];
        let mut profiles = smc_profiles();
        profiles.push(xsession());
        let (events, sent) = dispatched(raw, profiles, on_screen(0, 0));
        assert!(matches!(
            sent[..],
            [
                Command::PadPress { pad: 0, .. },
                Command::PadRelease { pad: 0 }
            ]
        ));
        assert_eq!(params(&events), [(Param::PadVolume, 100.0 / 127.0)]);
        assert_eq!(
            connected(&events),
            ["SMC-PAD Private", "SMC-PAD", "X-Session Pro"]
        );
    }

    /// Scenario "Button press and release use the current screen" (input-devices).
    #[test]
    fn button_press_and_release_use_the_current_screen() {
        let profiles = smc_profiles();
        let profile = profiles
            .iter()
            .find(|profile| profile.name == "SMC-PAD")
            .unwrap();
        for (screen, left, right) in [
            (
                Screen::Pads,
                Action::SmcList { next: false },
                Action::SmcList { next: true },
            ),
            (
                Screen::Files,
                Action::SmcList { next: false },
                Action::SmcList { next: true },
            ),
            (
                Screen::Chop,
                Action::SmcList { next: false },
                Action::SmcList { next: true },
            ),
        ] {
            for (cc, action) in [(25, left), (26, right)] {
                assert_eq!(
                    profile.resolve(screen, knob(cc, 127)),
                    Some(Resolved::Action(action))
                );
                assert_eq!(profile.resolve(screen, knob(cc, 0)), None);
            }
            assert_eq!(
                profile.resolve(screen, knob(27, 127)),
                Some(Resolved::Gesture {
                    tap: Some(Action::PlayContext),
                    hold: Action::MetronomeToggle,
                })
            );
            assert_eq!(
                profile.resolve(screen, knob(28, 127)),
                Some(Resolved::Action(Action::StopContext))
            );
            for cc in [27, 28] {
                assert_eq!(profile.resolve(screen, knob(cc, 0)), None);
            }
        }
        assert!(matches!(
            profile.resolve(Screen::Pads, knob(29, 127)),
            Some(Resolved::Gesture {
                hold: Action::RecordMic(true),
                ..
            })
        ));
    }

    #[test]
    fn smc_record_release_and_disconnect_end_the_hold() {
        for disconnect in [false, true] {
            let start = Instant::now();
            let end = if disconnect {
                Raw::Disconnected(SMC_MASTER.into(), None)
            } else {
                Raw::Msg(
                    SMC_MASTER.into(),
                    knob(29, 0),
                    gesture_stamp(start + Duration::from_millis(400)),
                )
            };
            let raw = vec![
                Raw::Msg(SMC_MASTER.into(), knob(29, 127), gesture_stamp(start)),
                Raw::Tick(start + DEFAULT_HOLD),
                end,
            ];
            let (events, _) = dispatched(raw, smc_profiles(), on_screen(0, 0));
            let actions: Vec<_> = events
                .into_iter()
                .filter_map(|event| match event {
                    Event::Action(action) => Some(action),
                    _ => None,
                })
                .collect();
            assert_eq!(actions, [Action::RecordMic(true), Action::RecordMic(false)]);
        }
    }

    /// Scenario "Owner bindings survive profile installation" (input-devices).
    #[test]
    fn owner_bindings_survive_profile_installation() {
        let mut profile = smc_profiles()
            .into_iter()
            .find(|profile| profile.name == "SMC-PAD")
            .unwrap();
        profile.set_bindings(
            Scope::Global,
            HashMap::from([(Key::Note(48), Binding::Press("pad.8".into()))]),
        );
        profile.set_seeded(true);
        let raw = vec![Raw::Msg(
            SMC_MASTER.into(),
            Msg::parse(&[0x99, 48, 80]).unwrap(),
            test_stamp(),
        )];
        let (_, sent) = dispatched(raw, vec![profile], on_screen(0, 0));
        assert!(matches!(sent[..], [Command::PadPress { pad: 7, .. }]));
    }

    /// Scenario "Controller view matches both pad banks" (screen-layout).
    #[test]
    fn controller_view_matches_both_pad_banks() {
        let profile = smc_profiles()
            .into_iter()
            .find(|profile| profile.name == "SMC-PAD")
            .unwrap();
        let rows = profile.layout().unwrap();
        let names = |row: usize| {
            rows[row][4..8]
                .iter()
                .map(|name| name.as_deref().unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(names(0), ["p1", "p2", "p3", "p4"]);
        assert_eq!(names(3), ["p13", "p14", "p15", "p16"]);
        assert_eq!(names(4), ["c1", "c2", "c3", "c4"]);
        assert_eq!(names(7), ["c13", "c14", "c15", "c16"]);
        assert_eq!(rows[0][0].as_deref(), Some("e7"));
        assert_eq!(rows[3][0].as_deref(), Some("e1"));
    }

    /// Scenario "Encoder roles follow the current screen" (screen-layout).
    #[test]
    fn encoder_roles_follow_the_current_screen() {
        let profile = smc_profiles()
            .into_iter()
            .find(|profile| profile.name == "SMC-PAD")
            .unwrap();
        assert_eq!(
            profile.control_action(Screen::Pads, false, "e1"),
            Some("smc.position")
        );
        assert_eq!(
            profile.control_action(Screen::Files, false, "e1"),
            Some("files.scroll")
        );
        assert_eq!(
            profile.control_action(Screen::Chop, false, "e1"),
            Some("chop.cursor_pos")
        );
        assert_eq!(
            profile.control_action(Screen::Pads, false, "f8"),
            Some("master.volume")
        );
        assert_eq!(profile.control_action(Screen::Files, false, "e6"), None);
        assert_eq!(profile.control_action(Screen::Files, false, "e7"), None);
        assert_eq!(
            profile.control_action(Screen::Files, false, "e8"),
            Some("master.volume")
        );
    }

    /// Scenario "Replay covers the measured controller stream" (input-devices).
    #[test]
    fn replay_covers_the_measured_controller_stream() {
        let (sender, receiver) = mpsc::channel();
        let project = ProjectState::default();
        for bytes in [
            &[0x99, 48, 100][..],
            &[0x89, 48, 64],
            &[0x99, 64, 100],
            &[0x89, 64, 64],
            &[0xb0, 25, 127],
            &[0xb0, 25, 0],
            &[0xb0, 31, 65],
            &[0xd9, 80],
        ] {
            receive(&sender, SMC_MASTER, bytes, Some(&project), None);
        }
        drop(sender);
        let mut raw = vec![Raw::SmcSetup(Ok(()))];
        raw.extend(receiver);
        let (events, sent) = dispatched(raw, smc_profiles(), on_screen(0, 0));
        assert!(matches!(
            sent[..],
            [
                Command::PadPress { pad: 0, .. },
                Command::PadRelease { pad: 0 }
            ]
        ));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::Step { steps: 1, .. }))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::Action(Action::SmcList { next: false })))
                .count(),
            1
        );
        let mut app = crate::tests::test_app();
        for event in events {
            app.midi_event(event);
        }
        assert_eq!(app.input_log.len(), 7, "pressure did not enter App");
    }
}

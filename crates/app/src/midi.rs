//! MIDI input: finds ports, connects and drops them on the fly, sends pad hits straight to the
//! audio thread (no UI frame delay) and everything else to the UI.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
use engine::{Command, PADS_PER_BANK};
use input::midi::{Key, Msg, Param, Profile, Resolved, Scope};
use input::{Action, Focus, Group, PadRoute, Screen, route_pad};
use std::sync::atomic::Ordering::Acquire;

/// midir does not report hotplug, so ports are polled.
const SCAN: Duration = Duration::from_secs(1);

pub enum Event {
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
    /// A controller hit; `accepted` says the audio queue accepted the note.
    PadHit {
        abs: usize,
        velocity: f32,
        accepted: bool,
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
    Action(Action),
    Param {
        port: String,
        control: String,
        param: Param,
        value: f32,
        takeover: bool,
    },
}

/// One complete input snapshot. Zero is normal performance on the first bank.
#[derive(Default)]
pub struct Shared {
    focus: AtomicU32,
}

impl Shared {
    /// Publishes valid state; rejects an invalid bank without changing the snapshot.
    pub fn publish(&self, focus: Focus) -> bool {
        let Some(bits) = focus.pack() else {
            return false;
        };
        self.focus.store(bits, Relaxed);
        true
    }

    /// Reads all input state with one atomic load. Only checked encodings are published.
    pub fn focus(&self) -> Focus {
        Focus::unpack(self.focus.load(Relaxed)).unwrap_or_default()
    }
}

enum Raw {
    Connected(String),
    Disconnected(String),
    Msg(String, Msg, Stamp),
    /// New learned assignments of a profile in a group.
    Learned(String, Scope, HashMap<Key, String>),
    /// The roles of a profile are copied into the learned maps (59): only those act now.
    Seeded(String),
    #[cfg(test)]
    Fence(Sender<()>),
}

#[derive(Clone, Copy)]
struct UiHold {
    abs: usize,
    epoch: u64,
    id: u64,
}

#[derive(Clone, Copy)]
struct Stamp {
    epoch: u64,
    ready: bool,
}

impl Stamp {
    fn capture(project: Option<&ProjectState>) -> Self {
        let Some(project) = project else {
            return Self {
                epoch: 0,
                ready: false,
            };
        };
        let epoch = project.epoch.load(Acquire);
        let ready = project.ready.load(Acquire) == epoch;
        Self { epoch, ready }
    }
}

/// Learned assignments per (profile name, group scope; 28).
pub type LearnedMaps = HashMap<(String, Scope), HashMap<Key, String>>;

pub struct UiEvent {
    pub event: Event,
    stamp: Option<Stamp>,
}

impl UiEvent {
    #[cfg(test)]
    pub fn stamped(event: Event, project: &ProjectState) -> Self {
        Self {
            event,
            stamp: Some(Stamp::capture(Some(project))),
        }
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
        let worker = std::thread::spawn(move || {
            dispatch(
                raw_rx,
                tx,
                profiles,
                LearnedMaps::new(),
                Vec::new(),
                Some(commands),
                Some(project),
                shared,
            );
        });
        (Self { events, updates }, worker)
    }

    #[cfg(test)]
    pub fn recorded_connect(&self, port: &str) {
        self.updates.send(Raw::Connected(port.into())).unwrap();
    }

    #[cfg(test)]
    pub fn recorded_disconnect(&self, port: &str) {
        self.updates.send(Raw::Disconnected(port.into())).unwrap();
    }

    #[cfg(test)]
    pub fn recorded_bytes(&self, port: &str, bytes: &[u8], project: &ProjectState) {
        if let Some(msg) = Msg::parse(bytes) {
            self.updates
                .send(Raw::Msg(port.into(), msg, Stamp::capture(Some(project))))
                .unwrap();
        }
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

    /// Replaces the learned assignments of `profile` in `group` in the MIDI thread.
    pub fn set_learned(&self, profile: &str, group: Group, map: HashMap<Key, String>) {
        let _ = self
            .updates
            .send(Raw::Learned(profile.into(), group.scope(), map));
    }

    /// The profile's roles live in the learned maps from now on (59).
    pub fn set_seeded(&self, profile: &str) {
        let _ = self.updates.send(Raw::Seeded(profile.into()));
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
                        let action = match action.as_str() {
                            "pad.scope_next" => "pad.mark_bank".to_string(),
                            a => a.to_string(),
                        };
                        map.insert(k, action);
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
            profile.set_learned(*scope, map.clone());
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
) -> Midi {
    for p in profiles.iter_mut() {
        apply_learned(p, &learned);
        p.set_seeded(seeded.contains(&p.name));
    }
    let (raw_tx, raw_rx) = mpsc::channel();
    let (tx, rx) = mpsc::channel();
    let scan_tx = raw_tx.clone();
    let scan_project = project.clone();
    std::thread::Builder::new()
        .name("midi-scan".into())
        .spawn(move || scan(scan_tx, scan_project))
        .expect("spawn midi-scan");
    std::thread::Builder::new()
        .name("midi".into())
        .spawn(move || {
            dispatch(
                raw_rx, tx, profiles, learned, seeded, commands, project, shared,
            )
        })
        .expect("spawn midi");
    Midi {
        events: rx,
        updates: raw_tx,
    }
}

fn scan(tx: Sender<Raw>, project: Option<Arc<ProjectState>>) {
    let mut connections: HashMap<String, midir::MidiInputConnection<()>> = HashMap::new();
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
            .filter(|(n, _)| !n.contains("Midi Through"))
            .collect();

        connections.retain(|name, _| {
            let alive = ports.iter().any(|(n, _)| n == name);
            if !alive {
                log::info!("MIDI disconnected: {name}");
                let _ = tx.send(Raw::Disconnected(name.clone()));
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
            let conn = input.connect(
                &port,
                "ptsr-in",
                move |_, bytes, _| {
                    if let Some(msg) = Msg::parse(bytes) {
                        let stamp = Stamp::capture(cb_project.as_deref());
                        let _ = cb_tx.send(Raw::Msg(cb_name.clone(), msg, stamp));
                    }
                },
                (),
            );
            match conn {
                Ok(c) => {
                    log::info!("MIDI connected: {name}");
                    let _ = tx.send(Raw::Connected(name.clone()));
                    connections.insert(name, c);
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
    let send = |event, stamp| {
        let _ = tx.send(UiEvent { event, stamp });
    };
    let mut defaults: HashMap<String, Profile> = HashMap::new();
    let mut pressed: HashMap<(String, u8, Key), Vec<(usize, u64)>> = HashMap::new();
    let mut ui_pressed: HashMap<(String, u8, Key), Vec<UiHold>> = HashMap::new();
    let mut next_ui_press = 0u64;
    let mut pending = VecDeque::new();
    let mut closed = false;
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
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                closed = true;
                if !pending.is_empty() {
                    std::thread::sleep(Duration::from_millis(2));
                }
                continue;
            }
        };
        let (port, msg, stamp) = match raw {
            Raw::Connected(port) => {
                let profile = profiles
                    .iter()
                    .find(|p| p.matches(&port))
                    .map_or("default".into(), |p| p.name.clone());
                send(Event::Connected { port, profile }, None);
                continue;
            }
            Raw::Disconnected(port) => {
                // a held gate pad would hang without its note off
                pressed.retain(|(p, _, _), held| {
                    if *p == port {
                        for &(abs, epoch) in held.iter() {
                            pending.push_back(release_command(abs, epoch));
                            send(
                                Event::PadRelease { abs, direct: true },
                                Some(Stamp { epoch, ready: true }),
                            );
                        }
                    }
                    *p != port
                });
                ui_pressed.retain(|(p, _, _), held| {
                    if *p == port {
                        for hold in held.iter() {
                            send(
                                Event::UiPadRelease {
                                    abs: hold.abs,
                                    id: hold.id,
                                },
                                Some(Stamp {
                                    epoch: hold.epoch,
                                    ready: true,
                                }),
                            );
                        }
                    }
                    *p != port
                });
                send(Event::Disconnected { port }, None);
                continue;
            }
            Raw::Learned(name, scope, map) => {
                for p in profiles.iter_mut().chain(defaults.values_mut()) {
                    if p.name == name {
                        p.set_learned(scope, map.clone());
                    }
                }
                learned.insert((name, scope), map);
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
            #[cfg(test)]
            Raw::Fence(done) => {
                done.send(()).unwrap();
                continue;
            }
            Raw::Msg(port, msg, stamp) => (port, msg, stamp),
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
        let focus = shared.focus();
        let resolved = profile.resolve_in(focus.screen, focus.mode.edit, msg);
        let meaning = match &resolved {
            None => "not assigned".into(),
            Some(Resolved::Action(a)) => format!("{a:?}"),
            Some(Resolved::Param { control, param, .. }) => format!("{control} -> {param:?}"),
        };
        send(
            Event::Message {
                port: port.clone(),
                profile: profile.name.clone(),
                msg,
                meaning,
            },
            Some(stamp),
        );
        let channel = match msg {
            Msg::NoteOn { ch, .. } | Msg::NoteOff { ch, .. } | Msg::Cc { ch, .. } => ch,
        };
        if matches!(msg, Msg::NoteOff { .. } | Msg::Cc { value: 0, .. }) {
            let key = (port.clone(), channel, msg.key());
            let direct = pressed.remove(&key);
            let ui = ui_pressed.remove(&key);
            let tracked = direct.is_some() || ui.is_some();
            if let Some(held) = direct {
                for (abs, epoch) in held {
                    pending.push_back(release_command(abs, epoch));
                    send(
                        Event::PadRelease { abs, direct: true },
                        Some(Stamp { epoch, ready: true }),
                    );
                }
            }
            if let Some(held) = ui {
                for hold in held {
                    send(
                        Event::UiPadRelease {
                            abs: hold.abs,
                            id: hold.id,
                        },
                        Some(Stamp {
                            epoch: hold.epoch,
                            ready: true,
                        }),
                    );
                }
            }
            if tracked {
                continue;
            }
        }
        let route = route_pad(focus);
        if route == PadRoute::Ignore {
            continue;
        }

        match resolved {
            Some(Resolved::Action(Action::PadPress { pad, velocity })) if route == PadRoute::Ui => {
                let abs = focus.bank * PADS_PER_BANK + pad;
                let ui_press = (focus.screen == Screen::Chop && stamp.ready).then(|| {
                    let held = ui_pressed.entry((port, channel, msg.key())).or_default();
                    if let Some(hold) = held
                        .iter()
                        .find(|hold| hold.abs == abs && hold.epoch == stamp.epoch)
                    {
                        return hold.id;
                    }
                    let id = next_ui_press;
                    next_ui_press = next_ui_press.wrapping_add(1);
                    held.push(UiHold {
                        abs,
                        epoch: stamp.epoch,
                        id,
                    });
                    id
                });
                send(
                    Event::PadHit {
                        abs,
                        velocity,
                        accepted: false,
                        ui_press,
                    },
                    Some(stamp),
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
                send(
                    Event::PadHit {
                        abs,
                        velocity,
                        accepted,
                        ui_press: None,
                    },
                    Some(stamp),
                );
            }
            Some(Resolved::Action(Action::PadRelease { pad })) => {
                let abs = focus.bank * PADS_PER_BANK + pad;
                send(Event::PadRelease { abs, direct: false }, Some(stamp));
            }
            Some(Resolved::Action(a)) => {
                send(Event::Action(a), Some(stamp));
            }
            Some(Resolved::Param {
                control,
                param,
                value,
                takeover,
            }) => {
                send(
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
            None => {}
        }
    }
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
                // a role naming no action does nothing: it is named, the rest loads (71)
                for (_, key, name) in p.roles() {
                    if !input::midi::known(&name) {
                        let e = format!("{file}: {key} {name}: no such action");
                        log::warn!("{e}");
                        errors.push(e);
                    }
                }
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

    #[test]
    fn shared_focus_rejects_invalid_publication_and_keeps_its_default() {
        let shared = Shared::default();
        assert_eq!(shared.focus(), Focus::default());
        let focus = Focus {
            screen: Screen::Fx,
            bank: 7,
            ..Focus::default()
        };
        assert!(shared.publish(focus));
        assert_eq!(shared.focus(), focus);
        assert!(!shared.publish(Focus { bank: 8, ..focus }));
        assert_eq!(shared.focus(), focus);
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
                screen_learn: false,
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
        for _ in 0..1000 {
            raw_tx
                .send(Raw::Msg("snapshot".into(), knob(80, 80), test_stamp()))
                .unwrap();
        }
        drop(raw_tx);
        let (tx, rx) = mpsc::channel();
        let (producer, mut consumer) = rtrb::RingBuffer::new(1024);
        dispatch(
            raw_rx,
            tx,
            vec![profile],
            LearnedMaps::new(),
            Vec::new(),
            Some(producer),
            None,
            shared.clone(),
        );
        done.store(true, Release);
        writer.join().unwrap();
        let events: Vec<_> = rx.try_iter().map(|event| event.event).collect();
        assert_eq!(events.len(), 2000);
        for pair in events.chunks_exact(2) {
            let Event::Message { meaning, .. } = &pair[0] else {
                panic!("missing input log")
            };
            let Event::PadHit {
                abs,
                velocity,
                accepted,
                ui_press,
            } = &pair[1]
            else {
                panic!("missing pad hit")
            };
            assert_eq!(*velocity, 80.0 / 127.0);
            if meaning.contains("pad: 0,") {
                assert_eq!((*abs, *accepted, *ui_press), (0, true, None));
            } else {
                assert!(meaning.contains("pad: 2,"), "{meaning}");
                assert_eq!((*abs, *accepted), (114, false));
                assert!(ui_press.is_some());
            }
        }
        while let Ok(command) = consumer.pop() {
            assert!(matches!(command.command,
                ProjectMessage::Command(Command::PadPress { pad: 0, velocity })
                if velocity == 80.0 / 127.0));
        }
        assert!(matches!(shared.focus(), focus if focus == ordinary || focus == chop));
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
                    Stamp { epoch: 0, ready },
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
            assert!(rx.try_iter().any(|event| matches!(event.event, Event::PadHit { abs: 0, accepted, .. } if accepted == expected)));
        }
    }

    #[test]
    fn ui_events_keep_the_callback_epoch_and_drop_unready_hits() {
        use std::sync::atomic::Ordering::Release;
        let state = Arc::new(ProjectState::default());
        let (raw_tx, raw_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        raw_tx.send(Raw::Connected("controller".into())).unwrap();
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
        let mut focus = shared.focus();
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
                raw_tx.send(Raw::Disconnected(port)).unwrap();
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
            map.get(&Key::Cc(24)).map(String::as_str),
            Some("pad.volume")
        );
        assert_eq!(
            map.get(&Key::Note(43)).map(String::as_str),
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
            pads_edit.get(&Key::Cc(25)).map(String::as_str),
            Some("edit.param_2")
        );
        let chop_edit = maps
            .get(&("xsession-pro".into(), Scope::Edit(Screen::Chop)))
            .unwrap();
        assert_eq!(
            chop_edit.get(&Key::Cc(26)).map(String::as_str),
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
        Stamp {
            epoch: 0,
            ready: true,
        }
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
            [(key, action.to_string())].into(),
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
        let mut focus = shared.focus();
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
                    let mut focus = shared.focus();
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
                    accepted: true,
                    ..
                }
            ));
            set_screen(&shared, 4, 2);
            let stale = Stamp {
                epoch: 7,
                ready: false,
            };
            raw_tx
                .send(Raw::Msg(XS.into(), foreign_release, stale))
                .unwrap();
            raw_tx.send(Raw::Connected(XS.into())).unwrap();
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
                    accepted: false,
                    ..
                }
            ));
            raw_tx.send(Raw::Msg(XS.into(), release, stale)).unwrap();
            raw_tx
                .send(Raw::Msg(XS.into(), knob(25, 0), test_stamp()))
                .unwrap();
            raw_tx.send(Raw::Disconnected(XS.into())).unwrap();
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
                    raw.push(Raw::Disconnected(XS.into()));
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
                rx.recv_timeout(Duration::from_secs(1)).unwrap().event
            };
            let hit = |velocity| Msg::NoteOn {
                ch: 1,
                note: 36,
                velocity,
            };
            let off = Msg::NoteOff { ch: 1, note: 36 };
            let first = send(hit(80));
            assert!(
                matches!(first, Event::PadHit { abs: 0, velocity, accepted, ui_press }
                if velocity == 80.0 / 127.0
                    && accepted == (before != 3)
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
                    accepted: true,
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
            raw_tx.send(Raw::Disconnected(PADS.into())).unwrap();
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
            Raw::Connected(PADS.into()),
            Raw::Connected(XS.into()),
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
            Raw::Connected(XS.into()),
            Raw::Connected(PADS.into()),
            Raw::Msg(PADS.into(), note_on(36), test_stamp()),
            Raw::Disconnected(PADS.into()),
            Raw::Disconnected(XS.into()),
            Raw::Connected(XS.into()),
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
        let mut raw = vec![Raw::Connected(port.into())];
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
            Raw::Connected(PADS.into()),
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
            Raw::Connected(XS.into()),
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
}

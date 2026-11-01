//! MIDI input: finds ports, connects and drops them on the fly, sends pad hits straight to the
//! audio thread (no UI frame delay) and everything else to the UI.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering::Relaxed};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use engine::{Command, PADS_PER_BANK};
use input::midi::{Key, Msg, Param, Profile, Resolved, Scope};
use input::{Action, Group, Screen};

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
    /// A pad was hit from a controller; the audio thread already has it.
    PadHit {
        abs: usize,
    },
    /// A controller pad was released (or its device disappeared).
    PadRelease {
        abs: usize,
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

/// UI state the MIDI thread needs: current bank and screen.
#[derive(Default)]
pub struct Shared {
    pub bank: AtomicUsize,
    /// 0 pads, 1 settings, 2 files, 3 chop.
    pub screen: AtomicU8,
    /// Pad hits go to the UI instead of the audio thread: something is in hand and the hit
    /// puts it on the pad, or the chop screen plays slices.
    pub hand: AtomicBool,
    /// An edit mode is open: the `[edit]` layer of the profiles goes first.
    pub edit: AtomicBool,
    /// The learn mode is open (47): a control only goes to the UI to be bound, pads stay
    /// silent and nothing acts.
    pub learn: AtomicBool,
}

enum Raw {
    Connected(String),
    Disconnected(String),
    Msg(String, Msg),
    /// New learned assignments of a profile in a group.
    Learned(String, Scope, HashMap<Key, String>),
    /// The roles of a profile are copied into the learned maps (59): only those act now.
    Seeded(String),
}

/// Learned assignments per (profile name, group scope; 28).
pub type LearnedMaps = HashMap<(String, Scope), HashMap<Key, String>>;

pub struct Midi {
    pub events: Receiver<Event>,
    updates: Sender<Raw>,
}

impl Midi {
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
                        map.insert(k, action.clone());
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
    commands: Option<rtrb::Producer<Command>>,
    shared: Arc<Shared>,
) -> Midi {
    for p in profiles.iter_mut() {
        apply_learned(p, &learned);
        p.set_seeded(seeded.contains(&p.name));
    }
    let (raw_tx, raw_rx) = mpsc::channel();
    let (tx, rx) = mpsc::channel();
    let scan_tx = raw_tx.clone();
    std::thread::Builder::new()
        .name("midi-scan".into())
        .spawn(move || scan(scan_tx))
        .expect("spawn midi-scan");
    std::thread::Builder::new()
        .name("midi".into())
        .spawn(move || dispatch(raw_rx, tx, profiles, learned, seeded, commands, shared))
        .expect("spawn midi");
    Midi {
        events: rx,
        updates: raw_tx,
    }
}

fn scan(tx: Sender<Raw>) {
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
            let conn = input.connect(
                &port,
                "ptsr-in",
                move |_, bytes, _| {
                    if let Some(msg) = Msg::parse(bytes) {
                        let _ = cb_tx.send(Raw::Msg(cb_name.clone(), msg));
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

fn dispatch(
    rx: Receiver<Raw>,
    tx: Sender<Event>,
    mut profiles: Vec<Profile>,
    mut learned: LearnedMaps,
    mut seeded: Vec<String>,
    mut commands: Option<rtrb::Producer<Command>>,
    shared: Arc<Shared>,
) {
    let mut defaults: HashMap<String, Profile> = HashMap::new();
    // (port, pad key) -> absolute pad, so a release reaches the pad that was pressed
    let mut pressed: HashMap<(String, usize), usize> = HashMap::new();
    let mut send = |cmd| {
        if let Some(c) = commands.as_mut() {
            // queue full: the hit is dropped, audio never waits
            let _ = c.push(cmd);
        }
    };

    for raw in rx {
        let (port, msg) = match raw {
            Raw::Connected(port) => {
                let profile = profiles
                    .iter()
                    .find(|p| p.matches(&port))
                    .map_or("default".into(), |p| p.name.clone());
                let _ = tx.send(Event::Connected { port, profile });
                continue;
            }
            Raw::Disconnected(port) => {
                // a held gate pad would hang without its note off
                pressed.retain(|(p, _), abs| {
                    if *p == port {
                        send(Command::PadRelease { pad: *abs });
                        let _ = tx.send(Event::PadRelease { abs: *abs });
                    }
                    *p != port
                });
                let _ = tx.send(Event::Disconnected { port });
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
            Raw::Msg(port, msg) => (port, msg),
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
        let screen = match shared.screen.load(Relaxed) {
            1 => Screen::Settings,
            2 => Screen::Files,
            3 => Screen::Chop,
            _ => Screen::Pads,
        };
        let resolved = profile.resolve_in(screen, shared.edit.load(Relaxed), msg);
        let meaning = match &resolved {
            None => "not assigned".into(),
            Some(Resolved::Action(a)) => format!("{a:?}"),
            Some(Resolved::Param { control, param, .. }) => format!("{control} -> {param:?}"),
        };
        let _ = tx.send(Event::Message {
            port: port.clone(),
            profile: profile.name.clone(),
            msg,
            meaning,
        });
        if shared.learn.load(Relaxed) {
            continue;
        }

        match resolved {
            Some(Resolved::Action(Action::PadPress { pad, .. })) if shared.hand.load(Relaxed) => {
                let abs = shared.bank.load(Relaxed) * PADS_PER_BANK + pad;
                let _ = tx.send(Event::PadHit { abs });
            }
            Some(Resolved::Action(Action::PadPress { pad, velocity })) => {
                let abs = shared.bank.load(Relaxed) * PADS_PER_BANK + pad;
                pressed.insert((port, pad), abs);
                send(Command::PadPress { pad: abs, velocity });
                let _ = tx.send(Event::PadHit { abs });
            }
            Some(Resolved::Action(Action::PadRelease { pad })) => {
                let abs = pressed
                    .remove(&(port, pad))
                    .unwrap_or(shared.bank.load(Relaxed) * PADS_PER_BANK + pad);
                send(Command::PadRelease { pad: abs });
                let _ = tx.send(Event::PadRelease { abs });
            }
            Some(Resolved::Action(a)) => {
                let _ = tx.send(Event::Action(a));
            }
            Some(Resolved::Param {
                control,
                param,
                value,
                takeover,
            }) => {
                let _ = tx.send(Event::Param {
                    port,
                    control,
                    param,
                    value,
                    takeover,
                });
            }
            None => {}
        }
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
        dispatch(
            raw_rx,
            tx,
            profiles,
            LearnedMaps::new(),
            Vec::new(),
            Some(producer),
            shared,
        );
        let mut commands = Vec::new();
        while let Ok(c) = consumer.pop() {
            commands.push(c);
        }
        (rx.try_iter().collect(), commands)
    }

    /// A control learned for `action` in GLOBAL, then `msgs` from it.
    fn learned_global(key: Key, action: &str, msgs: &[Msg], shared: Shared) -> Vec<Command> {
        let mut raw = vec![Raw::Learned(
            "X-Session Pro".into(),
            Scope::Global,
            [(key, action.to_string())].into(),
        )];
        raw.extend(msgs.iter().map(|m| Raw::Msg(XS.into(), *m)));
        dispatched(raw, vec![xsession()], shared).1
    }

    fn on_screen(code: u8, bank: usize) -> Shared {
        let s = Shared::default();
        s.screen.store(code, Relaxed);
        s.bank.store(bank, Relaxed);
        s
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
            Raw::Msg(PADS.into(), note_on(36)),
            Raw::Msg(XS.into(), knob(24, 100)),
            Raw::Msg(PADS.into(), note_on(37)),
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
            Raw::Msg(PADS.into(), note_on(36)),
            Raw::Disconnected(PADS.into()),
            Raw::Disconnected(XS.into()),
            Raw::Connected(XS.into()),
            Raw::Msg(XS.into(), knob(24, 64)),
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
        raw.extend((36..=52).map(|n| Raw::Msg(port.into(), note_on(n))));
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
            Raw::Msg(PADS.into(), note_on(36)),
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
            Raw::Msg(XS.into(), knob(24, 64)),
            Raw::Msg(XS.into(), knob(99, 10)),
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

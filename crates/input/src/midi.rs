//! MIDI messages to logical actions: device profiles, the default profile and soft takeover.
//! Pure logic: tested on recorded messages without hardware.

use std::collections::HashMap;

use crate::{Action, Screen};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Msg {
    /// Channel 1..16.
    NoteOn {
        ch: u8,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        ch: u8,
        note: u8,
    },
    Cc {
        ch: u8,
        cc: u8,
        value: u8,
    },
}

impl Msg {
    /// One complete message as midir delivers it; other messages (clock, sysex, ...) give `None`.
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        let (&status, data) = bytes.split_first()?;
        let ch = (status & 0x0F) + 1;
        let (a, b) = (*data.first()?, data.get(1).copied().unwrap_or(0));
        Some(match status & 0xF0 {
            // note on with velocity 0 is a note off
            0x90 if b > 0 => Msg::NoteOn {
                ch,
                note: a,
                velocity: b,
            },
            0x80 | 0x90 => Msg::NoteOff { ch, note: a },
            0xB0 => Msg::Cc {
                ch,
                cc: a,
                value: b,
            },
            _ => return None,
        })
    }
}

impl std::fmt::Display for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Msg::NoteOn { ch, note, velocity } => write!(f, "ch {ch} note {note} on {velocity}"),
            Msg::NoteOff { ch, note } => write!(f, "ch {ch} note {note} off"),
            Msg::Cc { ch, cc, value } => write!(f, "ch {ch} cc {cc} = {value}"),
        }
    }
}

/// A control by its MIDI number; learned assignments are keyed by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Cc(u8),
    Note(u8),
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Key::Cc(n) => write!(f, "cc {n}"),
            Key::Note(n) => write!(f, "note {n}"),
        }
    }
}

impl std::str::FromStr for Key {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let (kind, num) = s
            .split_once(' ')
            .ok_or_else(|| format!("bad control {s:?}"))?;
        let num: u8 = num.parse().map_err(|_| format!("bad control {s:?}"))?;
        match kind {
            "cc" => Ok(Key::Cc(num)),
            "note" => Ok(Key::Note(num)),
            _ => Err(format!("bad control {s:?}")),
        }
    }
}

impl Msg {
    pub fn key(&self) -> Key {
        match *self {
            Msg::NoteOn { note, .. } | Msg::NoteOff { note, .. } => Key::Note(note),
            Msg::Cc { cc, .. } => Key::Cc(cc),
        }
    }
}

/// Actions a knob drives with its position rather than a press.
pub fn is_param_action(action: &str) -> bool {
    param_of(action).is_some()
}

/// The program has this action, as a command or as a knob parameter (71).
pub fn known(name: &str) -> bool {
    param_of(name).is_some() || crate::Action::parse(name).is_some()
}

/// The parameter a knob action drives.
pub fn param_of(action: &str) -> Option<Param> {
    match action {
        "pad.volume" => Some(Param::PadVolume),
        "pad.pitch" => Some(Param::PadPitch),
        // every pad parameter on a knob of its own (52)
        "pad.attack" => Some(Param::PadAttack),
        "pad.decay" => Some(Param::PadDecay),
        "pad.release" => Some(Param::PadRelease),
        "pad.filter" => Some(Param::PadFilter),
        "pad.cutoff" => Some(Param::PadCutoff),
        "pad.reso" => Some(Param::PadReso),
        "pad.start" => Some(Param::PadStart),
        "pad.end" => Some(Param::PadEnd),
        "master.volume" => Some(Param::MasterVolume),
        "transport.tempo" => Some(Param::Tempo),
        "slice.start" => Some(Param::SliceStart),
        "slice.end" => Some(Param::SliceEnd),
        "chop.zoom" => Some(Param::Zoom),
        "chop.cursor_pos" => Some(Param::Cursor),
        "chop.region_start_pos" => Some(Param::RegionStart),
        "chop.region_end_pos" => Some(Param::RegionEnd),
        "chop.threshold" => Some(Param::Threshold),
        "edit.param_1" => Some(Param::Slot(0)),
        "edit.param_2" => Some(Param::Slot(1)),
        "edit.param_3" => Some(Param::Slot(2)),
        "edit.page" => Some(Param::Page),
        // Keep the direct shared FX names from the former pad edit (82).
        "pad.fx1_type" => Some(Param::PadEffect(3)),
        "pad.fx1_size" => Some(Param::PadEffect(4)),
        "pad.fx1_tone" => Some(Param::PadEffect(5)),
        "pad.fx2_type" => Some(Param::PadEffect(6)),
        "pad.fx2_size" => Some(Param::PadEffect(7)),
        "pad.fx2_tone" => Some(Param::PadEffect(8)),
        _ => None,
    }
}

/// Parameters a knob or fader can set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Param {
    PadVolume,
    PadPitch,
    MasterVolume,
    Tempo,
    /// Chop screen: start and end of the selected slice.
    SliceStart,
    SliceEnd,
    /// Chop screen: the zoom, the cursor and the region edges, all knob positions 0..1.
    Zoom,
    Cursor,
    RegionStart,
    RegionEnd,
    /// Chop screen: the auto-chop threshold, 0..1 over 0…−40 dB below the loudest hit.
    Threshold,
    /// Pads edit: the envelope times of the selected pad.
    PadAttack,
    PadDecay,
    PadRelease,
    /// Pads edit, SOURCE page (27): the source of the selected pad, its start and end.
    PadSource,
    PadStart,
    PadEnd,
    /// Pads edit, FILTER page (44): the kind, the cutoff and the resonance.
    PadFilter,
    PadCutoff,
    PadReso,
    /// The k-th parameter of the edit page (`edit.param_1..3`); the app picks the parameter.
    Slot(u8),
    /// The page of the edit (54): the travel split into equal parts, one per page.
    Page,
    /// Pads edit, the FX pages (56): the n-th of drive, send 1, send 2, then the kind and
    /// the two parameters of FX 1 and of FX 2; the app knows which is which.
    PadEffect(u8),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Resolved {
    Action(Action),
    /// An absolute knob or fader: `value` is 0..1.
    Param {
        control: String,
        param: Param,
        value: f32,
        takeover: bool,
    },
}

/// Physical activity, independent of whether the current screen has an action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Activity {
    None,
    Press,
    Release,
    Movement,
}

#[derive(Clone, Debug)]
enum Control {
    Pad(usize),
    Knob { name: String },
    Button(String),
}

/// Where learned assignments apply (28): the global group, a screen group, or the edit
/// layer of a screen — a control can be learned in PADS and in PADS · EDIT for
/// different actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scope {
    Global,
    Screen(Screen),
    Edit(Screen),
}

#[derive(Clone)]
pub struct Profile {
    pub name: String,
    pub file: String,
    match_port: String,
    /// `None` accepts every channel.
    channel: Option<u8>,
    notes: HashMap<u8, Control>,
    ccs: HashMap<u8, Control>,
    /// Screen sections; `None` key is `[screens.global]`.
    screens: HashMap<Option<Screen>, HashMap<String, String>>,
    /// `[edit]`: control -> action while an edit mode is open, over the screens and learned.
    edit: HashMap<String, String>,
    /// Learned on the MIDI tab (user config); override `screens`, the edit groups (28)
    /// stand over the `[edit]` layer.
    learned: HashMap<Scope, HashMap<Key, String>>,
    /// The roles were copied into the user config (59): only `learned` acts, the profile
    /// still names the controls.
    seeded: bool,
}

/// Notes 36..51 play pads 1..16, as on most pad controllers.
const DEFAULT_FIRST_NOTE: u8 = 36;

impl Profile {
    /// For devices without a profile (or with a broken one).
    pub fn default_for(port: &str) -> Self {
        let notes = (0..16)
            .map(|i| (DEFAULT_FIRST_NOTE + i as u8, Control::Pad(i)))
            .collect();
        Self {
            name: "default".into(),
            file: String::new(),
            match_port: port.into(),
            channel: None,
            notes,
            ccs: HashMap::new(),
            screens: HashMap::new(),
            edit: HashMap::new(),
            learned: HashMap::new(),
            seeded: false,
        }
    }

    /// The control is a knob or fader of this device; a CC the profile does not name counts
    /// as one, a note never does (53).
    pub fn is_knob(&self, key: Key) -> bool {
        match key {
            Key::Cc(cc) => !matches!(self.ccs.get(&cc), Some(Control::Button(_))),
            Key::Note(_) => false,
        }
    }

    pub fn matches(&self, port: &str) -> bool {
        !self.match_port.is_empty() && port.contains(&self.match_port)
    }

    /// Reads a profile from `config/devices/<name>.toml`; the error names the file and line.
    pub fn from_toml(file: &str, text: &str) -> Result<Self, String> {
        let t: toml::Table = text.parse().map_err(|e: toml::de::Error| {
            let line = e.span().map(|s| text[..s.start].lines().count().max(1));
            match line {
                Some(l) => format!("{file}:{l}: {}", e.message()),
                None => format!("{file}: {}", e.message()),
            }
        })?;
        let err = |m: &str| format!("{file}: {m}");
        let device = t
            .get("device")
            .and_then(|d| d.as_table())
            .ok_or_else(|| err("no [device]"))?;
        let get_str = |k: &str| device.get(k).and_then(|v| v.as_str()).map(String::from);
        let channel = match device.get("channel").and_then(|v| v.as_integer()) {
            Some(c @ 1..=16) => Some(c as u8),
            Some(c) => return Err(err(&format!("channel {c} is not 1-16"))),
            None => None,
        };

        let mut notes = HashMap::new();
        let mut ccs = HashMap::new();
        let controls = t.get("controls").and_then(|c| c.as_table());
        for (group, items) in controls.into_iter().flatten() {
            for (name, spec) in items.as_table().into_iter().flatten() {
                let num = |k: &str| {
                    spec.get(k)
                        .and_then(|v| v.as_integer())
                        .filter(|n| (0..128).contains(n))
                        .map(|n| n as u8)
                };
                let control = match group.as_str() {
                    // pads: p1..p16
                    "pads" => match name.strip_prefix('p').and_then(|n| n.parse::<usize>().ok()) {
                        Some(n @ 1..=16) => Control::Pad(n - 1),
                        _ => return Err(err(&format!("pad name {name} is not p1-p16"))),
                    },
                    "buttons" => Control::Button(name.clone()),
                    _ => {
                        if spec.get("type").and_then(|v| v.as_str()) == Some("relative") {
                            // relative encoders (SMC-PAD) are not supported yet
                            continue;
                        }
                        // the knob response is one setting for all knobs (58): a profile's
                        // `takeover` is not read
                        Control::Knob { name: name.clone() }
                    }
                };
                match (num("note"), num("cc")) {
                    (Some(n), _) => notes.insert(n, control),
                    (None, Some(c)) => ccs.insert(c, control),
                    _ => return Err(err(&format!("{group}.{name} has no note or cc"))),
                };
            }
        }

        let mut screens = HashMap::new();
        for (key, section) in t
            .get("screens")
            .and_then(|s| s.as_table())
            .into_iter()
            .flatten()
        {
            let screen = match key.as_str() {
                "global" => None,
                key => match Screen::from_key(key) {
                    Some(screen) => Some(screen),
                    None => continue,
                },
            };
            let map = section
                .as_table()
                .into_iter()
                .flatten()
                .filter_map(|(c, a)| a.as_str().map(|a| (c.clone(), a.to_string())))
                .collect();
            screens.insert(screen, map);
        }
        let edit = t
            .get("edit")
            .and_then(|s| s.as_table())
            .into_iter()
            .flatten()
            .filter_map(|(c, a)| a.as_str().map(|a| (c.clone(), a.to_string())))
            .collect();

        Ok(Self {
            name: get_str("name").unwrap_or_else(|| file.into()),
            file: file.into(),
            match_port: get_str("match_port").ok_or_else(|| err("no device.match_port"))?,
            channel,
            notes,
            ccs,
            screens,
            edit,
            learned: HashMap::new(),
            seeded: false,
        })
    }

    /// The roles copied into the user config (59): from now on only the learned act.
    pub fn set_seeded(&mut self, seeded: bool) {
        self.seeded = seeded;
    }

    /// The starting roles of the profile as bindings (59): the screen sections in their
    /// groups, `[screens.global]` in GLOBAL, `[edit]` in both edit groups, the pads of a
    /// pad control in GLOBAL.
    pub fn roles(&self) -> Vec<(Scope, Key, String)> {
        let mut out = Vec::new();
        for (screen, map) in &self.screens {
            let scope = screen.map_or(Scope::Global, Scope::Screen);
            for (control, action) in map {
                if let Some(key) = self.key_of(control) {
                    out.push((scope, key, action.clone()));
                }
            }
        }
        for (control, action) in &self.edit {
            if let Some(key) = self.key_of(control) {
                for group in Screen::ALL
                    .into_iter()
                    .filter_map(|screen| screen.rules().ui_edit_group)
                {
                    out.push((group.scope(), key, action.clone()));
                }
            }
        }
        for (note, c) in &self.notes {
            if let Control::Pad(pad) = c {
                out.push((Scope::Global, Key::Note(*note), format!("pad.{}", pad + 1)));
            }
        }
        out
    }

    /// The MIDI key of a named control (`k1` -> `cc 24`).
    fn key_of(&self, control: &str) -> Option<Key> {
        let named = |c: &Control| match c {
            Control::Knob { name } | Control::Button(name) => name == control,
            Control::Pad(_) => false,
        };
        self.ccs
            .iter()
            .find_map(|(n, c)| named(c).then_some(Key::Cc(*n)))
            .or_else(|| {
                self.notes
                    .iter()
                    .find_map(|(n, c)| named(c).then_some(Key::Note(*n)))
            })
    }

    /// The profile's name of a key (`cc 24` -> `k1`).
    pub fn name_of(&self, key: Key) -> Option<&str> {
        let c = match key {
            Key::Cc(n) => self.ccs.get(&n),
            Key::Note(n) => self.notes.get(&n),
        }?;
        match c {
            Control::Knob { name } | Control::Button(name) => Some(name),
            Control::Pad(_) => None,
        }
    }

    /// Learned controls of a group.
    pub fn set_learned(&mut self, scope: Scope, learned: HashMap<Key, String>) {
        self.learned.insert(scope, learned);
    }

    fn learned_in(&self, scope: Scope, action: &str) -> Option<Key> {
        self.learned
            .get(&scope)?
            .iter()
            .find_map(|(k, a)| (a == action).then_some(*k))
    }

    /// A learned control of `action` that works on `screen` (its own group or global).
    fn learned_action(&self, screen: Screen, action: &str) -> Option<Key> {
        self.learned_in(Scope::Screen(screen), action)
            .or_else(|| self.learned_in(Scope::Global, action))
    }

    fn action_name(&self, screen: Screen, control: &str) -> Option<&str> {
        let action = [Some(screen), None]
            .iter()
            .find_map(|s| self.screens.get(s).and_then(|m| m.get(control)))?;
        // a learned action takes over every profile control of the same action
        if self.learned_action(screen, action).is_some() {
            return None;
        }
        Some(action)
    }

    /// The FX screen accepts its own bindings first, then the former page controls.
    /// The optional name identifies a profile control, not a learned assignment.
    fn fx_binding(&self, key: Key) -> Option<(&str, Option<&str>)> {
        if let Some(map) = self.learned.get(&Scope::Screen(Screen::Fx)) {
            return map.get(&key).map(|action| (action.as_str(), None));
        }
        let learned = |scope| self.learned.get(&scope).and_then(|m| m.get(&key));
        let name = self.name_of(key);
        let profile = |screen| {
            (!self.seeded)
                .then(|| self.screens.get(&Some(screen))?.get(name?))
                .flatten()
        };
        if let Some(action) = profile(Screen::Fx) {
            return Some((action, name));
        }
        let profile_action = |action: &str| {
            !self.seeded
                && self
                    .screens
                    .get(&Some(Screen::Fx))
                    .is_some_and(|map| map.values().any(|a| a == action))
        };
        let scopes = [
            Scope::Edit(Screen::Pads),
            Scope::Edit(Screen::Chop),
            Scope::Screen(Screen::Pads),
            Scope::Global,
        ];
        let overridden = |action: &str, before: usize| {
            scopes[..before]
                .iter()
                .any(|scope| self.learned_in(*scope, action).is_some())
                || profile_action(action)
        };
        scopes
            .into_iter()
            .enumerate()
            .find_map(|(i, scope)| {
                learned(scope)
                    .filter(|a| fx_compatible(a) && !overridden(a, i))
                    .map(|a| (a.as_str(), None))
            })
            .or_else(|| {
                (!self.seeded)
                    .then(|| self.edit.get(name?))
                    .flatten()
                    .filter(|a| fx_compatible(a) && !overridden(a, scopes.len()))
                    .map(|a| (a.as_str(), name))
            })
            .or_else(|| {
                profile(Screen::Pads)
                    .filter(|a| fx_compatible(a) && !overridden(a, scopes.len()))
                    .map(|a| (a.as_str(), name))
            })
            .or_else(|| {
                (!self.seeded)
                    .then(|| self.screens.get(&None)?.get(name?))
                    .flatten()
                    .filter(|a| fx_compatible(a) && !overridden(a, scopes.len()))
                    .map(|a| (a.as_str(), name))
            })
    }

    pub fn fx_defaults(&self) -> HashMap<Key, String> {
        let mut keys: Vec<_> = self
            .learned
            .values()
            .flat_map(|m| m.keys().copied())
            .collect();
        keys.extend(self.ccs.keys().map(|cc| Key::Cc(*cc)));
        keys.extend(self.notes.keys().map(|note| Key::Note(*note)));
        keys.into_iter()
            .filter_map(|key| {
                self.fx_binding(key)
                    .map(|(action, _)| (key, action.to_string()))
            })
            .collect()
    }

    /// Who drives `action` of a group now, for the MIDI tab: "k1 (cc 24)", or "cc 24" for a
    /// control the profile does not name.
    /// Profile assignments are looked up in the group first, then in any screen section.
    pub fn assignment(&self, scope: Scope, action: &str) -> Option<String> {
        if matches!(scope, Scope::Screen(screen) if screen.rules().fx_page_controls) {
            // Include inherited page controls, also after the profile was seeded.
            let mut keys: Vec<_> = self
                .learned
                .values()
                .flat_map(|m| m.keys().copied())
                .collect();
            keys.extend(self.ccs.keys().map(|cc| Key::Cc(*cc)));
            keys.extend(self.notes.keys().map(|note| Key::Note(*note)));
            return keys.into_iter().find_map(|key| {
                let (bound, _) = self.fx_binding(key)?;
                (bound == action).then(|| match self.name_of(key) {
                    Some(name) => format!("{name} ({key})"),
                    None => key.to_string(),
                })
            });
        }
        if let Some(key) = self.learned_in(scope, action) {
            // every binding is the owner's after the seed (59): named as the profile names it
            return Some(match self.name_of(key) {
                Some(name) => format!("{name} ({key})"),
                None => key.to_string(),
            });
        }
        if self.seeded {
            return None;
        }
        let find = |m: &HashMap<String, String>| {
            m.iter()
                .find_map(|(c, a)| (a == action).then_some(c.clone()))
        };
        let control = match scope {
            // the edit groups are served by the profile's `[edit]` layer (28)
            Scope::Edit(_) => find(&self.edit),
            Scope::Screen(s) => self
                .screens
                .get(&Some(s))
                .and_then(find)
                .or_else(|| self.screens.values().find_map(find)),
            Scope::Global => self
                .screens
                .get(&None)
                .and_then(find)
                .or_else(|| self.screens.values().find_map(find)),
        }?;
        let control = control.as_str();
        let named = |c: &Control| match c {
            Control::Knob { name, .. } | Control::Button(name) => name == control,
            Control::Pad(_) => false,
        };
        let key = self
            .ccs
            .iter()
            .find_map(|(n, c)| named(c).then_some(Key::Cc(*n)))
            .or_else(|| {
                self.notes
                    .iter()
                    .find_map(|(n, c)| named(c).then_some(Key::Note(*n)))
            });
        Some(match key {
            Some(k) => format!("{control} ({k})"),
            None => control.to_string(),
        })
    }

    /// What a message means on `screen`; `None` — not assigned.
    pub fn resolve(&self, screen: Screen, msg: Msg) -> Option<Resolved> {
        self.resolve_in(screen, false, msg)
    }

    /// Same, with an edit mode open or not: the `[edit]` layer goes first.
    pub fn resolve_in(&self, screen: Screen, edit: bool, msg: Msg) -> Option<Resolved> {
        self.resolve_input_in(screen, edit, msg).0
    }

    /// Resolve the actual message once, with activity for notice acknowledgement.
    pub fn resolve_input_in(
        &self,
        screen: Screen,
        edit: bool,
        msg: Msg,
    ) -> (Option<Resolved>, Activity) {
        let mut activity = Activity::None;
        let resolved = self.resolve_inner(screen, edit, msg, &mut activity);
        (resolved, activity)
    }

    fn activity(&self, msg: Msg, learned: Option<&str>) -> Activity {
        match msg {
            Msg::NoteOn { .. } => Activity::Press,
            Msg::NoteOff { .. } => Activity::Release,
            Msg::Cc { cc, value, .. } => {
                let continuous = match self.ccs.get(&cc) {
                    Some(Control::Knob { .. }) => true,
                    Some(Control::Button(_) | Control::Pad(_)) => false,
                    None => learned.is_none_or(is_param_action),
                };
                if continuous {
                    Activity::Movement
                } else if value > 0 {
                    Activity::Press
                } else {
                    Activity::Release
                }
            }
        }
    }

    fn resolve_inner(
        &self,
        screen: Screen,
        edit: bool,
        msg: Msg,
        activity: &mut Activity,
    ) -> Option<Resolved> {
        let ch = match msg {
            Msg::NoteOn { ch, .. } | Msg::NoteOff { ch, .. } | Msg::Cc { ch, .. } => ch,
        };
        if self.channel.is_some_and(|c| c != ch) {
            return None;
        }
        *activity = self.activity(msg, None);
        let (value, on) = match msg {
            Msg::NoteOn { velocity, .. } => (velocity, true),
            Msg::NoteOff { .. } => (0, false),
            Msg::Cc { value, .. } => (value, value > 0),
        };
        let control = match msg {
            Msg::NoteOn { note, .. } | Msg::NoteOff { note, .. } => self.notes.get(&note),
            Msg::Cc { cc, .. } => self.ccs.get(&cc),
        };
        let rules = screen.rules();
        if rules.fx_page_controls
            && let Some((action, name)) = self.fx_binding(msg.key())
        {
            *activity = self.activity(msg, Some(action));
            let mut resolved = learned_resolved(action, msg, value, on)?;
            if let (Some(name), Resolved::Param { control, .. }) = (name, &mut resolved) {
                *control = name.to_string();
            }
            return Some(resolved);
        }
        // FX has no separate edit mode; its inherited controls were resolved above.
        let edit = edit && rules.midi_edit_layer;
        // the learned edit group of the screen goes first inside the edit (28), over the
        // profile's `[edit]` layer; outside the edit it does nothing
        if edit
            && let Some(action) = self
                .learned
                .get(&Scope::Edit(screen))
                .and_then(|m| m.get(&msg.key()))
        {
            *activity = self.activity(msg, Some(action));
            return learned_resolved(action, msg, value, on);
        }
        let layered = !self.seeded
            && edit
            && control.is_some_and(|c| match c {
                Control::Knob { name, .. } | Control::Button(name) => self.edit.contains_key(name),
                Control::Pad(_) => false,
            });
        if !layered {
            let learned = [Scope::Screen(screen), Scope::Global]
                .into_iter()
                .find_map(|s| self.learned.get(&s).and_then(|m| m.get(&msg.key())));
            if let Some(action) = learned {
                if rules.fx_page_controls && fx_compatible(action) {
                    return None;
                }
                *activity = self.activity(msg, Some(action));
                return learned_resolved(action, msg, value, on);
            }
        }
        // seeded (59): the profile's roles live in the learned maps now
        if self.seeded {
            return None;
        }
        let control = control?;
        let action_name = |name: &str| {
            let action = match layered {
                true => self.edit.get(name).map(String::as_str),
                false => self.action_name(screen, name),
            }?;
            (!rules.fx_page_controls || !fx_compatible(action)).then_some(action)
        };
        match control {
            Control::Pad(pad) if on => Some(Resolved::Action(Action::PadPress {
                pad: *pad,
                velocity: value as f32 / 127.0,
            })),
            Control::Pad(pad) => Some(Resolved::Action(Action::PadRelease { pad: *pad })),
            Control::Button(name) => {
                let action = action_name(name)?;
                match Action::parse(action) {
                    Some(Action::PadPress { pad, .. }) if on => {
                        Some(Resolved::Action(Action::PadPress {
                            pad,
                            velocity: value as f32 / 127.0,
                        }))
                    }
                    Some(Action::PadPress { pad, .. }) => {
                        Some(Resolved::Action(Action::PadRelease { pad }))
                    }
                    Some(a) if on => Some(Resolved::Action(a)),
                    _ => None,
                }
            }
            Control::Knob { name } => {
                let param = param_of(action_name(name)?)?;
                Some(Resolved::Param {
                    control: name.clone(),
                    param,
                    value: value as f32 / 127.0,
                    takeover: true,
                })
            }
        }
    }
}

/// Only page controls and direct shared FX controls carry over from the pad edit.
fn fx_compatible(action: &str) -> bool {
    matches!(
        param_of(action),
        Some(Param::Slot(_) | Param::Page | Param::PadEffect(3..=8))
    ) || matches!(
        Action::parse(action),
        Some(
            Action::ParamNext
                | Action::ParamPrev
                | Action::PageNext
                | Action::PagePrev
                | Action::EditValue { .. }
        )
    )
}

/// A release of a control learned for a pad ends a gate pad; other actions ignore releases.
fn pad_release(action: &str) -> Option<Resolved> {
    match Action::parse(action)? {
        Action::PadPress { pad, .. } => Some(Resolved::Action(Action::PadRelease { pad })),
        _ => None,
    }
}

/// What a learned control's action means as a message lands: a knob position or a press.
fn learned_resolved(action: &str, msg: Msg, value: u8, on: bool) -> Option<Resolved> {
    let param = param_of(action);
    match (msg, param) {
        (Msg::Cc { .. }, Some(param)) => Some(Resolved::Param {
            control: msg.key().to_string(),
            param,
            value: value as f32 / 127.0,
            takeover: true,
        }),
        // a note cannot be a knob
        _ if param.is_some() => None,
        // a pad control's release ends a gate pad; other actions ignore releases
        _ if !on => pad_release(action),
        _ => match Action::parse(action) {
            Some(Action::PadPress { pad, .. }) => Some(Resolved::Action(Action::PadPress {
                pad,
                velocity: value as f32 / 127.0,
            })),
            parsed => parsed.map(Resolved::Action),
        },
    }
}

/// Which way to move a control to pick up the parameter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    Up,
    Down,
}

/// How an absolute control drives its parameter (58): one setting for every knob.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Response {
    /// The parameter goes to the control position at once.
    #[default]
    Jump,
    /// The parameter waits until the control reaches its value; the screen shows the way.
    Pickup,
    /// The parameter moves from the first touch and meets the control at the end stop.
    Scale,
}

impl Response {
    pub const ALL: [Response; 3] = [Response::Jump, Response::Pickup, Response::Scale];

    pub fn name(self) -> &'static str {
        match self {
            Response::Jump => "jump",
            Response::Pickup => "pickup",
            Response::Scale => "scale",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.name() == name)
    }
}

/// Soft takeover: how an absolute control catches the parameter it drives, as the
/// [`Response`] says.
#[derive(Default)]
pub struct Takeover {
    pub response: Response,
    /// (device, control) -> (target the control is bound to, picked up, last position).
    state: HashMap<(String, String), (u64, bool, Option<f32>)>,
}

/// Positions closer than this count as "at the value" (one CC step is 1/127).
const PICKUP_WINDOW: f32 = 1.5 / 127.0;

impl Takeover {
    pub fn with(response: Response) -> Self {
        Self {
            response,
            ..Default::default()
        }
    }

    /// An external edit requires every control of this target to catch the new value.
    pub fn reset_target(&mut self, target: u64) {
        self.state.retain(|_, state| state.0 != target);
    }

    /// `target` identifies what the control drives now (e.g. the selected pad); a new target
    /// starts over. Returns the value to apply, or, in pickup, which way to move.
    pub fn apply(
        &mut self,
        device: &str,
        control: &str,
        target: u64,
        value: f32,
        current: f32,
    ) -> Result<f32, Move> {
        let entry = self
            .state
            .entry((device.into(), control.into()))
            .or_insert((target, false, None));
        if entry.0 != target {
            *entry = (target, false, None);
        }
        let (_, picked, last) = entry;
        let crossed = last.is_some_and(|l| (l - current) * (value - current) <= 0.0);
        let met = *picked || crossed || (value - current).abs() <= PICKUP_WINDOW;
        let prev = last.replace(value);
        match self.response {
            Response::Jump => Ok(value),
            _ if met => {
                *picked = true;
                Ok(value)
            }
            Response::Pickup => Err(if value < current {
                Move::Up
            } else {
                Move::Down
            }),
            Response::Scale => {
                // the share of the room left that the knob covers of its own travel left,
                // so both reach the end stop together
                let Some(p0) = prev else {
                    return Ok(current);
                };
                let v = if value > p0 {
                    current + (value - p0) * (1.0 - current) / (1.0 - p0)
                } else if value < p0 {
                    current - (p0 - value) * current / p0
                } else {
                    current
                };
                Ok(v.clamp(0.0, 1.0))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const XSESSION: &str = include_str!("../../../config/devices/xsession-pro.toml");

    fn xsession() -> Profile {
        Profile::from_toml("xsession-pro.toml", XSESSION).unwrap()
    }

    #[test]
    fn registry_sections_keep_profile_roles_and_midi_edit_layers() {
        let mut text = String::from(
            "[device]\nname = \"Registry\"\nmatch_port = \"Registry\"\n\
             [controls.buttons]\nbutton = { cc = 90 }\n\
             [screens.global]\nbutton = \"bank.next\"\n\
             [edit]\nbutton = \"edit.page_next\"\n",
        );
        for key in ["pads", "settings", "files", "chop", "fx"] {
            text.push_str(&format!("[screens.{key}]\nbutton = \"help.toggle\"\n"));
        }
        text.push_str("[screens.unknown]\nbutton = \"ui.quit\"\n");
        let profile = Profile::from_toml("registry.toml", &text).unwrap();
        let roles = profile.roles();
        assert_eq!(roles.len(), 8);
        for screen in Screen::ALL {
            assert!(roles.contains(&(Scope::Screen(screen), Key::Cc(90), "help.toggle".into())));
            assert_eq!(
                roles.contains(&(Scope::Edit(screen), Key::Cc(90), "edit.page_next".into())),
                matches!(screen, Screen::Pads | Screen::Chop),
            );
            let msg = Msg::Cc {
                ch: 1,
                cc: 90,
                value: 127,
            };
            assert_eq!(
                profile.resolve_in(screen, false, msg),
                Some(Resolved::Action(Action::HelpToggle))
            );
            assert_eq!(
                profile.resolve_in(screen, true, msg),
                Some(Resolved::Action(if screen == Screen::Fx {
                    Action::HelpToggle
                } else {
                    Action::PageNext
                })),
            );
        }
        let mut seeded = profile.clone();
        for scope in roles.iter().map(|(scope, _, _)| *scope) {
            seeded.set_learned(
                scope,
                roles
                    .iter()
                    .filter(|(s, _, _)| *s == scope)
                    .map(|(_, key, action)| (*key, action.clone()))
                    .collect(),
            );
        }
        seeded.set_seeded(true);
        for screen in Screen::ALL {
            let msg = Msg::Cc {
                ch: 1,
                cc: 90,
                value: 127,
            };
            assert_eq!(
                seeded.resolve_in(screen, false, msg),
                profile.resolve_in(screen, false, msg)
            );
            assert_eq!(
                seeded.resolve_in(screen, true, msg),
                Some(Resolved::Action(
                    if matches!(screen, Screen::Pads | Screen::Chop) {
                        Action::PageNext
                    } else {
                        Action::HelpToggle
                    }
                )),
            );
        }
    }

    #[test]
    fn notice_activity_uses_effective_bindings_without_changing_actions() {
        let mut p = Profile::default_for("recorded");
        p.set_learned(
            Scope::Global,
            [
                (Key::Cc(80), "session.save".into()),
                (Key::Cc(81), "pad.volume".into()),
                (Key::Cc(82), "pad.2".into()),
            ]
            .into(),
        );
        p.set_learned(
            Scope::Edit(Screen::Pads),
            [(Key::Cc(80), "pad.pitch".into())].into(),
        );
        for seeded in [false, true] {
            p.set_seeded(seeded);
            for screen in Screen::ALL {
                for edit in [false, true] {
                    for value in [0, 1, 64, 127] {
                        for cc in [80, 81, 82, 99] {
                            let msg = Msg::Cc { ch: 1, cc, value };
                            let (action, activity) = p.resolve_input_in(screen, edit, msg);
                            assert_eq!(action, p.resolve_in(screen, edit, msg));
                            let continuous = cc == 81
                                || cc == 99
                                || (cc == 80 && edit && screen == Screen::Pads);
                            assert_eq!(
                                activity,
                                if continuous {
                                    Activity::Movement
                                } else if value > 0 {
                                    Activity::Press
                                } else {
                                    Activity::Release
                                },
                                "{screen:?} edit={edit} seeded={seeded} cc={cc} value={value}"
                            );
                        }
                    }
                }
            }
        }
        let mut p = xsession();
        p.set_learned(Scope::Global, [(Key::Cc(24), "session.save".into())].into());
        assert_eq!(
            p.resolve_input_in(
                Screen::Pads,
                false,
                Msg::Cc {
                    ch: 1,
                    cc: 24,
                    value: 0
                }
            )
            .1,
            Activity::Movement
        );
        assert_eq!(
            p.resolve_input_in(
                Screen::Pads,
                false,
                Msg::Cc {
                    ch: 2,
                    cc: 24,
                    value: 0
                }
            ),
            (None, Activity::None)
        );
        let p = Profile::from_toml("button.toml", "[device]\nname='Button'\nmatch_port='Button'\n[controls.buttons]\nsave={cc=80}\n[screens.global]\nsave='session.save'\n").unwrap();
        assert_eq!(
            p.resolve_input_in(
                Screen::Pads,
                false,
                Msg::Cc {
                    ch: 1,
                    cc: 80,
                    value: 0
                }
            ),
            (None, Activity::Release)
        );
        assert_eq!(
            p.resolve_input_in(
                Screen::Pads,
                false,
                Msg::Cc {
                    ch: 1,
                    cc: 80,
                    value: 127
                }
            ),
            (Some(Resolved::Action(Action::SessionSave)), Activity::Press)
        );
    }

    #[test]
    fn parse_raw_messages() {
        assert_eq!(
            Msg::parse(&[0x90, 56, 127]),
            Some(Msg::NoteOn {
                ch: 1,
                note: 56,
                velocity: 127
            })
        );
        assert_eq!(
            Msg::parse(&[0x90, 56, 0]),
            Some(Msg::NoteOff { ch: 1, note: 56 })
        );
        assert_eq!(
            Msg::parse(&[0x8F, 56, 64]),
            Some(Msg::NoteOff { ch: 16, note: 56 })
        );
        assert_eq!(
            Msg::parse(&[0xB0, 24, 64]),
            Some(Msg::Cc {
                ch: 1,
                cc: 24,
                value: 64
            })
        );
        assert_eq!(Msg::parse(&[0xF8]), None);
        assert_eq!(Msg::parse(&[]), None);
    }

    #[test]
    fn xsession_chop_screen_knobs_and_buttons() {
        let p = Profile::from_toml("xsession-pro.toml", XSESSION).unwrap();
        let cc = |cc| {
            p.resolve(
                Screen::Chop,
                Msg::Cc {
                    ch: 1,
                    cc,
                    value: 0,
                },
            )
        };
        assert_eq!(
            cc(24),
            Some(Resolved::Param {
                control: "k1".into(),
                param: Param::SliceStart,
                value: 0.0,
                takeover: true
            })
        );
        assert!(matches!(
            cc(25),
            Some(Resolved::Param {
                param: Param::SliceEnd,
                ..
            })
        ));
        // 17: the zoom lives on k12
        assert_eq!(
            cc(36),
            Some(Resolved::Param {
                control: "k12".into(),
                param: Param::Zoom,
                value: 0.0,
                takeover: true
            })
        );
        let b7 = Msg::NoteOn {
            ch: 1,
            note: 58,
            velocity: 127,
        };
        assert_eq!(
            p.resolve(Screen::Chop, b7),
            Some(Resolved::Action(Action::ChopCut))
        );
        // 04d: k11 is the chop threshold
        assert!(matches!(
            cc(35),
            Some(Resolved::Param {
                param: Param::Threshold,
                ..
            })
        ));
    }

    #[test]
    fn x_session_pro_only() {
        let p = Profile::from_toml("xsession-pro.toml", XSESSION).unwrap();
        let button = |note| {
            p.resolve(
                Screen::Chop,
                Msg::NoteOn {
                    ch: 1,
                    note,
                    velocity: 127,
                },
            )
        };
        for note in [70, 69] {
            assert_eq!(button(note), Some(Resolved::Action(Action::ChopPlay)));
        }
        assert_eq!(button(58), Some(Resolved::Action(Action::ChopCut)));
        assert_eq!(button(46), Some(Resolved::Action(Action::MarkerSelect(-1))));
        assert_eq!(button(43), Some(Resolved::Action(Action::MarkerSelect(1))));
        assert_eq!(button(59), Some(Resolved::Action(Action::ChopEditOnly)));
        let knob = |cc| match p.resolve(
            Screen::Chop,
            Msg::Cc {
                ch: 1,
                cc,
                value: 0,
            },
        ) {
            Some(Resolved::Param { param, .. }) => Some(param),
            _ => None,
        };
        assert_eq!(knob(26), Some(Param::Cursor), "k3 the cursor");
        assert_eq!(knob(27), Some(Param::RegionStart), "k4 the region start");
        assert_eq!(knob(28), Some(Param::RegionEnd), "k5 the region end");
    }

    /// 21 input-devices: "The buttons get their roles back" — the `[edit]` layer stands
    /// only while the mode is open, over the screen roles and over a learned knob.
    #[test]
    fn the_buttons_get_their_roles_back() {
        let mut p = Profile::from_toml("xsession-pro.toml", XSESSION).unwrap();
        let note = |note| Msg::NoteOn {
            ch: 1,
            note,
            velocity: 127,
        };
        let act = |p: &Profile, screen, edit, msg| match p.resolve_in(screen, edit, msg) {
            Some(Resolved::Action(a)) => Some(a),
            _ => None,
        };
        let (b4, b7, b8) = (note(46), note(58), note(59));
        assert_eq!(act(&p, Screen::Chop, true, b4), Some(Action::PagePrev));
        assert_eq!(
            act(&p, Screen::Chop, false, b4),
            Some(Action::MarkerSelect(-1))
        );
        assert_eq!(act(&p, Screen::Chop, false, b8), Some(Action::ChopEditOnly));
        assert_eq!(act(&p, Screen::Pads, false, b7), Some(Action::PadEditOnly));
        assert_eq!(act(&p, Screen::Pads, true, b7), Some(Action::Back));
        assert_eq!(
            act(&p, Screen::Pads, false, b4),
            Some(Action::PadSelect(crate::Dir::Left))
        );

        p.set_learned(
            Scope::Screen(Screen::Pads),
            [(Key::Cc(24), "pad.pitch".into())].into(),
        );
        let k1 = Msg::Cc {
            ch: 1,
            cc: 24,
            value: 0,
        };
        let param = |edit| match p.resolve_in(Screen::Pads, edit, k1) {
            Some(Resolved::Param { param, .. }) => Some(param),
            _ => None,
        };
        assert_eq!(
            param(false),
            Some(Param::PadPitch),
            "the learned role outside"
        );
        assert_eq!(param(true), Some(Param::Slot(0)), "the page slot inside");
    }

    /// Recorded from the X-Session Pro: knob CC 24, button note 43, crossfader CC 17 and its mirror CC 20.
    #[test]
    fn xsession_profile_resolves_recorded_messages() {
        let p = xsession();
        assert!(p.matches("USB X-Session:USB X-Session MIDI 1 28:0"));
        let knob = p.resolve(
            Screen::Pads,
            Msg::Cc {
                ch: 1,
                cc: 24,
                value: 127,
            },
        );
        assert_eq!(
            knob,
            Some(Resolved::Param {
                control: "k1".into(),
                param: Param::PadVolume,
                value: 1.0,
                takeover: true
            })
        );
        assert_eq!(
            p.resolve(
                Screen::Pads,
                Msg::NoteOn {
                    ch: 1,
                    note: 43,
                    velocity: 127
                }
            ),
            Some(Resolved::Action(Action::PadSelect(crate::Dir::Right)))
        );
        // release of a button and unassigned controls do nothing
        assert_eq!(
            p.resolve(Screen::Pads, Msg::NoteOff { ch: 1, note: 43 }),
            None
        );
        assert_eq!(
            p.resolve(
                Screen::Pads,
                Msg::Cc {
                    ch: 1,
                    cc: 20,
                    value: 5
                }
            ),
            None
        );
        // other channel
        assert_eq!(
            p.resolve(
                Screen::Pads,
                Msg::Cc {
                    ch: 2,
                    cc: 24,
                    value: 5
                }
            ),
            None
        );
    }

    #[test]
    fn x_session_physical_navigation_and_preview() {
        let p = Profile::from_toml("xsession-pro.toml", XSESSION).unwrap();
        let notes = [46, 43, 44, 45, 56, 57, 58, 59, 70, 69];
        for (screen, expected) in [
            (
                Screen::Pads,
                [
                    Some(Action::PadSelect(crate::Dir::Left)),
                    Some(Action::PadSelect(crate::Dir::Right)),
                    Some(Action::ScreenFiles),
                    Some(Action::ScreenFx),
                    Some(Action::BankPrev),
                    Some(Action::BankNext),
                    Some(Action::PadEditOnly),
                    Some(Action::Back),
                    None,
                    None,
                ],
            ),
            (
                Screen::Files,
                [
                    Some(Action::BrowserUp),
                    Some(Action::BrowserDown),
                    Some(Action::ScreenFiles),
                    Some(Action::ScreenFx),
                    Some(Action::BankPrev),
                    Some(Action::BankNext),
                    Some(Action::BrowserTakeOnly),
                    Some(Action::ScreenChop),
                    Some(Action::BrowserPreviewToggle),
                    Some(Action::BrowserPreviewToggle),
                ],
            ),
            (
                Screen::Chop,
                [
                    Some(Action::MarkerSelect(-1)),
                    Some(Action::MarkerSelect(1)),
                    Some(Action::ScreenFiles),
                    Some(Action::ScreenFx),
                    Some(Action::BankPrev),
                    Some(Action::BankNext),
                    Some(Action::ChopCut),
                    Some(Action::ChopEditOnly),
                    Some(Action::ChopPlay),
                    Some(Action::ChopPlay),
                ],
            ),
            (
                Screen::Fx,
                [
                    Some(Action::PagePrev),
                    Some(Action::PageNext),
                    Some(Action::ScreenFiles),
                    Some(Action::ScreenFx),
                    Some(Action::BankPrev),
                    Some(Action::BankNext),
                    None,
                    Some(Action::Back),
                    None,
                    None,
                ],
            ),
            (
                Screen::Settings,
                [
                    None,
                    None,
                    Some(Action::ScreenFiles),
                    Some(Action::ScreenFx),
                    Some(Action::BankPrev),
                    Some(Action::BankNext),
                    None,
                    Some(Action::Back),
                    None,
                    None,
                ],
            ),
        ] {
            for editing in [false, true] {
                for (i, note) in notes.into_iter().enumerate() {
                    let action = if editing && screen != Screen::Fx {
                        match note {
                            46 => Some(Action::PagePrev),
                            43 => Some(Action::PageNext),
                            58 | 59 => Some(Action::Back),
                            _ => expected[i],
                        }
                    } else {
                        expected[i]
                    };
                    for (bytes, expected) in [
                        ([0x90, note, 127], action.map(Resolved::Action)),
                        ([0x80, note, 64], None),
                        ([0x90, note, 0], None),
                        ([0x91, note, 127], None),
                        ([0x81, note, 64], None),
                        ([0x91, note, 0], None),
                    ] {
                        assert_eq!(
                            p.resolve_in(screen, editing, Msg::parse(&bytes).unwrap()),
                            expected,
                            "{screen:?} edit={editing} bytes={bytes:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn x_session_faders_use_physical_roles() {
        for seeded in [false, true] {
            let mut p = Profile::from_toml("xsession-pro.toml", XSESSION).unwrap();
            if seeded {
                for (scope, key, action) in p.roles() {
                    p.learned.entry(scope).or_default().insert(key, action);
                }
                p.set_seeded(true);
            }
            for screen in [
                Screen::Pads,
                Screen::Files,
                Screen::Chop,
                Screen::Fx,
                Screen::Settings,
            ] {
                for editing in [false, true] {
                    for (cc, name, param) in [
                        (11, "fader_b", Some(Param::PadVolume)),
                        (12, "fader_a", Some(Param::PadPitch)),
                        (14, "pitch_a", Some(Param::MasterVolume)),
                        (15, "pitch_b", None),
                        (17, "crossfade", None),
                        (20, "", None),
                    ] {
                        for value in [0, 64, 127] {
                            let msg = Msg::parse(&[0xB0, cc, value]).unwrap();
                            assert_eq!(
                                p.resolve_in(screen, editing, msg),
                                param.map(|param| Resolved::Param {
                                    control: if seeded {
                                        format!("cc {cc}")
                                    } else {
                                        name.into()
                                    },
                                    param,
                                    value: value as f32 / 127.0,
                                    takeover: true,
                                }),
                                "{screen:?} edit={editing} seeded={seeded} cc={cc}"
                            );
                            assert_eq!(
                                p.resolve_in(
                                    screen,
                                    editing,
                                    Msg::parse(&[0xB1, cc, value]).unwrap()
                                ),
                                None
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn x_session_safe_default_buttons() {
        let p = Profile::from_toml("xsession-pro.toml", XSESSION).unwrap();
        let unsafe_actions = [
            "marker.delete",
            "chop.commit",
            "chop.hits",
            "chop.parts",
            "pad.edit",
            "chop.edit",
            "browser.hold",
        ]
        .map(|name| Action::parse(name).unwrap());
        for screen in [
            Screen::Pads,
            Screen::Files,
            Screen::Chop,
            Screen::Fx,
            Screen::Settings,
        ] {
            for editing in [false, true] {
                for note in [43, 44, 45, 46, 56, 57, 58, 59, 69, 70] {
                    if let Some(Resolved::Action(action)) =
                        p.resolve_in(screen, editing, Msg::parse(&[0x90, note, 127]).unwrap())
                    {
                        assert!(
                            !unsafe_actions.contains(&action),
                            "{screen:?} edit={editing} note={note}: {action:?}"
                        );
                    }
                }
            }
        }
        assert!(p.roles().iter().all(|(_, _, action)| known(action)));
    }

    #[test]
    fn x_session_fx_shipped_profile_inherits_page_and_global_roles() {
        let mut p = Profile::from_toml("xsession-pro.toml", XSESSION).unwrap();
        let expected: HashMap<_, _> = [
            (Key::Cc(24), "edit.param_1"),
            (Key::Cc(25), "edit.param_2"),
            (Key::Cc(26), "edit.param_3"),
            (Key::Note(46), "edit.page_prev"),
            (Key::Note(43), "edit.page_next"),
        ]
        .map(|(key, action)| (key, action.to_string()))
        .into();
        let globals: HashMap<_, _> = [
            (Key::Note(44), "screen.files"),
            (Key::Note(45), "screen.fx"),
            (Key::Note(56), "bank.prev"),
            (Key::Note(57), "bank.next"),
            (Key::Note(59), "ui.back"),
            (Key::Cc(11), "pad.volume"),
            (Key::Cc(12), "pad.pitch"),
            (Key::Cc(14), "master.volume"),
        ]
        .map(|(key, action)| (key, action.to_string()))
        .into();
        assert_eq!(p.fx_defaults(), expected);
        let roles = p.roles();
        assert!(
            roles
                .iter()
                .all(|(scope, _, _)| *scope != Scope::Screen(Screen::Fx))
        );
        assert_eq!(
            roles
                .iter()
                .filter(|(scope, _, _)| *scope == Scope::Global)
                .map(|(_, key, action)| (*key, action.clone()))
                .collect::<HashMap<_, _>>(),
            globals
        );
        for (scope, key, action) in roles {
            p.learned.entry(scope).or_default().insert(key, action);
        }
        p.set_seeded(true);
        assert_eq!(p.fx_defaults(), expected);
        for editing in [false, true] {
            for (key, action) in expected.iter().chain(&globals) {
                let bytes = match *key {
                    Key::Cc(cc) => [0xB0, cc, 64],
                    Key::Note(note) => [0x90, note, 127],
                };
                let resolved = p.resolve_in(Screen::Fx, editing, Msg::parse(&bytes).unwrap());
                if let Some(param) = param_of(action) {
                    assert_eq!(
                        resolved,
                        Some(Resolved::Param {
                            control: key.to_string(),
                            param,
                            value: 64.0 / 127.0,
                            takeover: true,
                        })
                    );
                } else {
                    assert_eq!(resolved, Action::parse(action).map(Resolved::Action));
                }
                if let Key::Note(note) = *key {
                    for bytes in [[0x80, note, 64], [0x90, note, 0], [0x91, note, 127]] {
                        assert_eq!(
                            p.resolve_in(Screen::Fx, editing, Msg::parse(&bytes).unwrap()),
                            None
                        );
                    }
                }
            }
            for note in [58, 69, 70] {
                assert_eq!(
                    p.resolve_in(Screen::Fx, editing, Msg::parse(&[0x90, note, 127]).unwrap()),
                    None
                );
            }
        }
    }

    #[test]
    fn default_profile_plays_notes_36_to_51() {
        let p = Profile::default_for("Some Pads");
        assert_eq!(
            p.resolve(
                Screen::Settings,
                Msg::NoteOn {
                    ch: 10,
                    note: 37,
                    velocity: 127
                }
            ),
            Some(Resolved::Action(Action::PadPress {
                pad: 1,
                velocity: 1.0
            }))
        );
        assert_eq!(
            p.resolve(Screen::Pads, Msg::NoteOff { ch: 10, note: 51 }),
            Some(Resolved::Action(Action::PadRelease { pad: 15 }))
        );
        assert_eq!(
            p.resolve(
                Screen::Pads,
                Msg::NoteOn {
                    ch: 1,
                    note: 52,
                    velocity: 1
                }
            ),
            None
        );
    }

    #[test]
    fn broken_profile_names_file_and_line() {
        let smc = include_str!("../../../config/devices/smc-pad.toml.example");
        let e = Profile::from_toml("smc-pad.toml", smc).err().unwrap();
        assert!(e.starts_with("smc-pad.toml:7:"), "{e}");
    }

    #[test]
    fn learned_pad_hit_carries_velocity_and_release() {
        let mut p = xsession();
        // a learned note drives pad 7; global scope, so it works on every screen
        p.set_learned(Scope::Global, [(Key::Note(60), "pad.7".to_string())].into());
        // press with a soft velocity: the pad hears it
        assert_eq!(
            p.resolve(
                Screen::Files,
                Msg::NoteOn {
                    ch: 1,
                    note: 60,
                    velocity: 51
                }
            ),
            Some(Resolved::Action(Action::PadPress {
                pad: 6,
                velocity: 51.0 / 127.0
            }))
        );
        // the release ends a gate pad
        assert_eq!(
            p.resolve(Screen::Files, Msg::NoteOff { ch: 1, note: 60 }),
            Some(Resolved::Action(Action::PadRelease { pad: 6 }))
        );
    }

    #[test]
    fn profile_button_learned_for_a_pad_has_velocity_and_release() {
        let mut p = xsession();
        // the profile's own button now hits pad 2 (a profile mapping, velocity included)
        p.screens
            .insert(None, [("b1".to_string(), "pad.2".to_string())].into());
        assert_eq!(
            p.resolve(
                Screen::Settings,
                Msg::NoteOn {
                    ch: 1,
                    note: 43,
                    velocity: 102
                }
            ),
            Some(Resolved::Action(Action::PadPress {
                pad: 1,
                velocity: 102.0 / 127.0
            }))
        );
        assert_eq!(
            p.resolve(Screen::Settings, Msg::NoteOff { ch: 1, note: 43 }),
            Some(Resolved::Action(Action::PadRelease { pad: 1 }))
        );
        // a CC button learned for a pad: press is the value, 0 is the release
        p.screens
            .insert(None, [("b9".to_string(), "pad.2".to_string())].into());
        p.ccs.insert(9, Control::Button("b9".into()));
        assert_eq!(
            p.resolve(
                Screen::Settings,
                Msg::Cc {
                    ch: 1,
                    cc: 9,
                    value: 80
                }
            ),
            Some(Resolved::Action(Action::PadPress {
                pad: 1,
                velocity: 80.0 / 127.0
            }))
        );
        assert_eq!(
            p.resolve(
                Screen::Settings,
                Msg::Cc {
                    ch: 1,
                    cc: 9,
                    value: 0
                }
            ),
            Some(Resolved::Action(Action::PadRelease { pad: 1 }))
        );
        assert_eq!(
            p.resolve(Screen::Pads, Msg::NoteOff { ch: 1, note: 43 }),
            None
        );
    }

    #[test]
    fn learned_controls_override_the_profile() {
        let mut p = xsession();
        assert_eq!(
            p.assignment(Scope::Screen(Screen::Pads), "pad.volume")
                .as_deref(),
            Some("k1 (cc 24)")
        );
        assert_eq!(
            p.assignment(Scope::Screen(Screen::Pads), "bank.next")
                .as_deref(),
            Some("b6 (note 57)")
        );
        p.set_learned(
            Scope::Screen(Screen::Pads),
            [
                (Key::Cc(31), "pad.volume".to_string()),
                (Key::Note(60), "bank.next".into()),
            ]
            .into(),
        );
        // the old profile knob and button no longer drive these actions
        assert_eq!(
            p.resolve(
                Screen::Pads,
                Msg::Cc {
                    ch: 1,
                    cc: 24,
                    value: 64
                }
            ),
            None
        );
        assert_eq!(
            p.resolve(
                Screen::Pads,
                Msg::NoteOn {
                    ch: 1,
                    note: 57,
                    velocity: 127
                }
            ),
            None
        );
        assert_eq!(
            p.resolve(
                Screen::Pads,
                Msg::Cc {
                    ch: 1,
                    cc: 31,
                    value: 127
                }
            ),
            Some(Resolved::Param {
                control: "cc 31".into(),
                param: Param::PadVolume,
                value: 1.0,
                takeover: true
            })
        );
        assert_eq!(
            p.resolve(
                Screen::Pads,
                Msg::NoteOn {
                    ch: 1,
                    note: 60,
                    velocity: 127
                }
            ),
            Some(Resolved::Action(Action::BankNext))
        );
        assert_eq!(
            p.resolve(Screen::Pads, Msg::NoteOff { ch: 1, note: 60 }),
            None
        );
        // a note cannot be a knob
        p.set_learned(
            Scope::Screen(Screen::Pads),
            [(Key::Note(57), "pad.volume".to_string())].into(),
        );
        assert_eq!(
            p.resolve(
                Screen::Pads,
                Msg::NoteOn {
                    ch: 1,
                    note: 57,
                    velocity: 127
                }
            ),
            None
        );
        assert_eq!(
            p.assignment(Scope::Screen(Screen::Pads), "pad.volume")
                .as_deref(),
            Some("b6 (note 57)") // 59: named as the profile names it, no "(learned)"
        );
        assert_eq!("cc 31".parse::<Key>(), Ok(Key::Cc(31)));
        assert!("knob 1".parse::<Key>().is_err());
    }

    #[test]
    fn learned_master_knob() {
        let mut p = Profile::default_for("X");
        p.set_learned(
            Scope::Global,
            [(Key::Cc(40), "master.volume".to_string())].into(),
        );
        assert_eq!(
            p.resolve(
                Screen::Files,
                Msg::Cc {
                    ch: 1,
                    cc: 40,
                    value: 0
                }
            ),
            Some(Resolved::Param {
                control: "cc 40".into(),
                param: Param::MasterVolume,
                value: 0.0,
                takeover: true
            })
        );
    }

    #[test]
    fn global_learned_works_everywhere_and_screen_wins() {
        let mut p = Profile::default_for("X");
        p.set_learned(
            Scope::Global,
            [(Key::Note(60), "help.toggle".to_string())].into(),
        );
        p.set_learned(
            Scope::Screen(Screen::Pads),
            [(Key::Note(60), "pad.mode_next".to_string())].into(),
        );
        let press = Msg::NoteOn {
            ch: 1,
            note: 60,
            velocity: 127,
        };
        assert_eq!(
            p.resolve(Screen::Settings, press),
            Some(Resolved::Action(Action::HelpToggle))
        );
        assert_eq!(
            p.resolve(Screen::Pads, press),
            Some(Resolved::Action(Action::PadModeNext))
        );
        assert_eq!(
            p.assignment(Scope::Global, "help.toggle").as_deref(),
            Some("note 60") // 59: a control the profile does not name
        );
    }

    /// Recorded: the owner learned `pad.pitch` on CC 20 and turns it up to the top.
    #[test]
    fn a_learned_pitch_knob_drives_the_pitch_param() {
        let mut p = xsession();
        p.set_learned(
            Scope::Screen(Screen::Pads),
            [(Key::Cc(20), "pad.pitch".to_string())].into(),
        );
        assert_eq!(
            p.resolve(
                Screen::Pads,
                Msg::Cc {
                    ch: 1,
                    cc: 20,
                    value: 127
                }
            ),
            Some(Resolved::Param {
                control: "cc 20".into(),
                param: Param::PadPitch,
                value: 1.0,
                takeover: true
            })
        );
    }

    /// 28 midi-learn: inside the edit the learned edit group goes before the profile's
    /// `[edit]` layer, outside the edit it is silent.
    #[test]
    fn a_learned_edit_knob_beats_the_profile_edit_layer() {
        let mut p = xsession();
        // the shipped profile maps k1 (cc 24) to edit.param_1
        assert_eq!(
            p.assignment(Scope::Edit(Screen::Pads), "edit.param_1")
                .as_deref(),
            Some("k1 (cc 24)")
        );
        p.set_learned(
            Scope::Edit(Screen::Pads),
            [(Key::Cc(24), "edit.param_2".to_string())].into(),
        );
        assert_eq!(
            p.resolve_in(
                Screen::Pads,
                true,
                Msg::Cc {
                    ch: 1,
                    cc: 24,
                    value: 64
                }
            ),
            Some(Resolved::Param {
                control: "cc 24".into(),
                param: Param::Slot(1),
                value: 64.0 / 127.0,
                takeover: true
            })
        );
        // outside the edit the learned edit control does nothing: k1 is the pad volume
        assert_eq!(
            p.resolve(
                Screen::Pads,
                Msg::Cc {
                    ch: 1,
                    cc: 24,
                    value: 64
                }
            ),
            Some(Resolved::Param {
                control: "k1".into(),
                param: Param::PadVolume,
                value: 64.0 / 127.0,
                takeover: true
            })
        );
    }

    fn assert_fx_param(p: &Profile, cc: u8, param: Param, control: &str) {
        for editing in [false, true] {
            assert_eq!(
                p.resolve_in(
                    Screen::Fx,
                    editing,
                    Msg::Cc {
                        ch: 1,
                        cc,
                        value: 0
                    }
                ),
                Some(Resolved::Param {
                    control: control.into(),
                    param,
                    value: 0.0,
                    takeover: true
                })
            );
        }
    }

    #[test]
    fn the_same_knobs_control_the_selected_effect() {
        let p = xsession();
        for (cc, name, param) in [
            (24, "k1", Param::Slot(0)),
            (25, "k2", Param::Slot(1)),
            (26, "k3", Param::Slot(2)),
        ] {
            assert_fx_param(&p, cc, param, name);
            assert_eq!(
                p.assignment(
                    Scope::Screen(Screen::Fx),
                    &format!("edit.param_{}", cc - 23)
                ),
                Some(format!("{name} (cc {cc})"))
            );
        }
        for (name, expected) in [("b4", Action::PagePrev), ("b1", Action::PageNext)] {
            let key = p.key_of(name).unwrap();
            let msg = match key {
                Key::Cc(cc) => Msg::Cc {
                    ch: 1,
                    cc,
                    value: 127,
                },
                Key::Note(note) => Msg::NoteOn {
                    ch: 1,
                    note,
                    velocity: 127,
                },
            };
            assert_eq!(p.resolve(Screen::Fx, msg), Some(Resolved::Action(expected)));
        }
    }

    #[test]
    fn old_learned_edit_and_seeded_page_mappings_work_on_fx() {
        for scope in [
            Scope::Edit(Screen::Pads),
            Scope::Edit(Screen::Chop),
            Scope::Screen(Screen::Pads),
        ] {
            for seeded in [false, true] {
                let mut p = xsession();
                p.screens.remove(&Some(Screen::Fx));
                p.set_seeded(seeded);
                p.set_learned(
                    scope,
                    [
                        (Key::Cc(24), "edit.param_1".into()),
                        (Key::Cc(25), "edit.param_2".into()),
                        (Key::Cc(26), "edit.param_3".into()),
                        (Key::Cc(27), "edit.page".into()),
                    ]
                    .into(),
                );
                for (cc, param) in [
                    (24, Param::Slot(0)),
                    (25, Param::Slot(1)),
                    (26, Param::Slot(2)),
                    (27, Param::Page),
                ] {
                    assert_fx_param(&p, cc, param, &format!("cc {cc}"));
                }
                assert_eq!(
                    p.assignment(Scope::Screen(Screen::Fx), "edit.page")
                        .as_deref(),
                    Some("k4 (cc 27)")
                );
            }
        }
        let mut p = xsession();
        p.screens.remove(&Some(Screen::Fx));
        p.screens.insert(
            Some(Screen::Pads),
            [
                ("k1".into(), "edit.param_1".into()),
                ("k2".into(), "edit.param_2".into()),
                ("k3".into(), "edit.param_3".into()),
                ("k4".into(), "edit.page".into()),
            ]
            .into(),
        );
        p.edit.clear();
        for (cc, param) in [
            (24, Param::Slot(0)),
            (25, Param::Slot(1)),
            (26, Param::Slot(2)),
            (27, Param::Page),
        ] {
            assert_fx_param(&p, cc, param, &format!("k{}", cc - 23));
        }
        let roles = p.roles();
        p.set_seeded(true);
        for scope in [Scope::Screen(Screen::Pads), Scope::Global] {
            p.set_learned(
                scope,
                roles
                    .iter()
                    .filter(|(s, _, _)| *s == scope)
                    .map(|(_, k, a)| (*k, a.clone()))
                    .collect(),
            );
        }
        for (cc, param) in [
            (24, Param::Slot(0)),
            (25, Param::Slot(1)),
            (26, Param::Slot(2)),
            (27, Param::Page),
        ] {
            assert_fx_param(&p, cc, param, &format!("cc {cc}"));
        }
    }

    #[test]
    fn old_direct_shared_fx_bindings_keep_their_slot() {
        for (i, action) in [
            "pad.fx1_type",
            "pad.fx1_size",
            "pad.fx1_tone",
            "pad.fx2_type",
            "pad.fx2_size",
            "pad.fx2_tone",
        ]
        .into_iter()
        .enumerate()
        {
            assert!(known(action));
            for scope in [
                Scope::Edit(Screen::Pads),
                Scope::Screen(Screen::Pads),
                Scope::Global,
            ] {
                let mut p = xsession();
                p.screens.remove(&Some(Screen::Fx));
                for seeded in [false, true] {
                    p.set_seeded(seeded);
                    p.set_learned(scope, [(Key::Cc(24), action.into())].into());
                    assert_fx_param(&p, 24, Param::PadEffect(i as u8 + 3), "cc 24");
                }
            }
        }
    }

    #[test]
    fn new_fx_bindings_take_precedence() {
        let mut p = xsession();
        p.screens.insert(
            Some(Screen::Fx),
            [
                ("k1".into(), "edit.param_3".into()),
                ("k4".into(), "edit.page".into()),
            ]
            .into(),
        );
        p.set_learned(
            Scope::Edit(Screen::Pads),
            [(Key::Cc(24), "pad.fx1_type".into())].into(),
        );
        assert_fx_param(&p, 24, Param::Slot(2), "k1");
        assert_fx_param(&p, 27, Param::Page, "k4");
        p.set_learned(
            Scope::Screen(Screen::Fx),
            [(Key::Cc(24), "edit.param_2".into())].into(),
        );
        assert_fx_param(&p, 24, Param::Slot(1), "cc 24");
        // The former knob for this action no longer changes it.
        for editing in [false, true] {
            assert_eq!(
                p.resolve_in(
                    Screen::Fx,
                    editing,
                    Msg::Cc {
                        ch: 1,
                        cc: 25,
                        value: 127
                    }
                ),
                None
            );
        }
        assert_eq!(
            p.assignment(Scope::Screen(Screen::Fx), "edit.param_2")
                .as_deref(),
            Some("k1 (cc 24)")
        );
        assert!(
            p.roles()
                .contains(&(Scope::Screen(Screen::Fx), Key::Cc(27), "edit.page".into()))
        );
        p.set_seeded(true);
        assert_fx_param(&p, 24, Param::Slot(1), "cc 24");
    }

    #[test]
    fn unseeded_x_session_fx_inherits_replaced_user_page_knob() {
        let mut p = xsession();
        p.set_learned(
            Scope::Edit(Screen::Pads),
            [(Key::Cc(80), "edit.param_1".into())].into(),
        );
        assert_fx_param(&p, 80, Param::Slot(0), "cc 80");
        for editing in [false, true] {
            assert_eq!(
                p.resolve_in(Screen::Fx, editing, Msg::parse(&[0xB0, 24, 127]).unwrap()),
                None
            );
        }
        assert_eq!(
            p.assignment(Scope::Screen(Screen::Fx), "edit.param_1"),
            Some("cc 80".into())
        );
        let defaults = p.fx_defaults();
        assert_eq!(
            defaults.get(&Key::Cc(80)).map(String::as_str),
            Some("edit.param_1")
        );
        assert!(!defaults.contains_key(&Key::Cc(24)));
    }

    #[test]
    fn replaced_page_knobs_stay_replaced() {
        let mut p = xsession();
        p.screens.remove(&Some(Screen::Fx));
        for (scope, cc) in [
            (Scope::Global, 77),
            (Scope::Screen(Screen::Pads), 78),
            (Scope::Edit(Screen::Chop), 24),
            (Scope::Edit(Screen::Pads), 80),
        ] {
            p.set_learned(scope, [(Key::Cc(cc), "edit.param_1".into())].into());
        }
        for seeded in [false, true] {
            p.set_seeded(seeded);
            assert_fx_param(&p, 80, Param::Slot(0), "cc 80");
            for cc in [24, 77, 78] {
                assert_eq!(
                    p.resolve(
                        Screen::Fx,
                        Msg::Cc {
                            ch: 1,
                            cc,
                            value: 127
                        }
                    ),
                    None
                );
            }
            assert_eq!(
                p.assignment(Scope::Screen(Screen::Fx), "edit.param_1"),
                Some("cc 80".into())
            );
            assert_eq!(
                p.fx_defaults().get(&Key::Cc(80)).map(String::as_str),
                Some("edit.param_1")
            );
        }
    }

    #[test]
    fn empty_fx_group_blocks_shared_bindings_but_not_global_pads() {
        let mut p = xsession();
        p.set_learned(
            Scope::Global,
            [
                (Key::Cc(80), "edit.param_1".into()),
                (Key::Cc(81), "pad.fx1_size".into()),
                (Key::Note(90), "pad.1".into()),
            ]
            .into(),
        );
        p.set_learned(Scope::Screen(Screen::Fx), HashMap::new());
        for seeded in [false, true] {
            p.set_seeded(seeded);
            for cc in [24, 25, 26, 80, 81] {
                assert_eq!(
                    p.resolve(
                        Screen::Fx,
                        Msg::Cc {
                            ch: 1,
                            cc,
                            value: 127
                        }
                    ),
                    None
                );
            }
            assert_eq!(
                p.assignment(Scope::Screen(Screen::Fx), "edit.param_1"),
                None
            );
            assert!(matches!(
                p.resolve(
                    Screen::Fx,
                    Msg::NoteOn {
                        ch: 1,
                        note: 90,
                        velocity: 127
                    }
                ),
                Some(Resolved::Action(Action::PadPress { pad: 0, .. }))
            ));
        }
    }

    #[test]
    fn unrelated_pad_edit_bindings_do_not_control_fx() {
        let mut p = xsession();
        p.set_seeded(true);
        p.set_learned(
            Scope::Edit(Screen::Pads),
            [
                (Key::Cc(24), "pad.volume".into()),
                (Key::Note(60), "ui.back".into()),
            ]
            .into(),
        );
        assert_eq!(
            p.resolve(
                Screen::Fx,
                Msg::Cc {
                    ch: 1,
                    cc: 24,
                    value: 127
                }
            ),
            None
        );
        assert_eq!(
            p.resolve(
                Screen::Fx,
                Msg::NoteOn {
                    ch: 1,
                    note: 60,
                    velocity: 127
                }
            ),
            None
        );
    }

    #[test]
    fn midi_pads_on_the_fx_screen_keep_velocity_and_gate_releases() {
        for seeded in [false, true] {
            let mut p = Profile::default_for("pads");
            let roles = p.roles();
            p.set_seeded(seeded);
            if seeded {
                p.set_learned(
                    Scope::Global,
                    roles
                        .into_iter()
                        .map(|(_, key, action)| (key, action))
                        .collect(),
                );
            }
            for pad in 0..16 {
                let note = DEFAULT_FIRST_NOTE + pad as u8;
                for editing in [false, true] {
                    assert_eq!(
                        p.resolve_in(
                            Screen::Fx,
                            editing,
                            Msg::NoteOn {
                                ch: 1,
                                note,
                                velocity: 80
                            }
                        ),
                        Some(Resolved::Action(Action::PadPress {
                            pad,
                            velocity: 80.0 / 127.0
                        }))
                    );
                    assert_eq!(
                        p.resolve_in(Screen::Fx, editing, Msg::NoteOff { ch: 1, note }),
                        Some(Resolved::Action(Action::PadRelease { pad }))
                    );
                }
            }
        }
    }

    /// 58 input-devices: "Pad change" — the default response jumps, no arrow.
    #[test]
    fn pad_change() {
        let mut t = Takeover::default();
        assert_eq!(t.response, Response::Jump);
        // the pad volume is at 0.3, the knob stands at 1.0
        assert_eq!(t.apply("x", "k1", 5, 0.99, 0.3), Ok(0.99));
        assert_eq!(
            t.apply("x", "k1", 6, 0.2, 0.9),
            Ok(0.2),
            "a new target jumps too"
        );
    }

    /// 58 input-devices: "Scale meets at the stop".
    #[test]
    fn scale_meets_at_the_stop() {
        let mut t = Takeover {
            response: Response::Scale,
            ..Default::default()
        };
        // the parameter at 20 %, the knob at 60 %: the first message only notes the knob
        let mut v = t.apply("x", "k1", 5, 0.6, 0.2).unwrap();
        assert_eq!(v, 0.2);
        let mut knob = 0.6f32;
        let mut first = true;
        while knob < 1.0 {
            knob = (knob + 1.0 / 127.0).min(1.0);
            let next = t.apply("x", "k1", 5, knob, v).unwrap();
            assert!(next >= v, "never falls: {v} -> {next}");
            if first {
                assert!(next > v, "rises with the first step");
                first = false;
            }
            v = next;
        }
        assert!((v - 1.0).abs() < 1e-4, "meets the knob at the stop: {v}");
        // met: the knob drives the value directly
        assert_eq!(t.apply("x", "k1", 5, 0.5, v), Ok(0.5));
    }

    /// 58 input-devices: "Pickup in the settings".
    #[test]
    fn takeover_waits_for_the_knob_to_reach_the_value() {
        let mut t = Takeover {
            response: Response::Pickup,
            ..Default::default()
        };
        // pad volume is at 0.3, the knob is at 1.0 and goes down
        assert_eq!(t.apply("x", "k1", 5, 1.0, 0.3), Err(Move::Down));
        assert_eq!(t.apply("x", "k1", 5, 0.6, 0.3), Err(Move::Down));
        // crossing 0.3 between two messages picks it up
        assert_eq!(t.apply("x", "k1", 5, 0.25, 0.3), Ok(0.25));
        assert_eq!(t.apply("x", "k1", 5, 0.9, 0.25), Ok(0.9));
        // another pad selected: the knob must pick up again
        assert_eq!(t.apply("x", "k1", 6, 0.8, 0.0), Err(Move::Down));
        assert_eq!(t.apply("x", "k1", 6, 0.005, 0.0), Ok(0.005));
        // knob below the value asks to turn up
        assert_eq!(t.apply("y", "k1", 1, 0.1, 0.5), Err(Move::Up));
    }
}

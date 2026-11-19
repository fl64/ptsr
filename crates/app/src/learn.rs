//! MIDI learn: waits for a control and keeps learned assignments in the `[midi]` config shape.

pub(crate) mod smc_layout;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use input::midi::{Binding, Key, Msg};
use input::{ActionInfo, Group};

use crate::config::{Learned, MidiBinding};

/// One row on the MIDI tab.
pub(super) enum MidiListRow {
    Response,
    HoldTime,
    SmcPadBank,
    SmcLayoutList(usize),
    SmcLayoutApply,
    SmcLayoutSlot(smc_layout::Slot),
    Reset {
        name: String,
        port: String,
    },
    Disabled {
        profile: String,
        group: String,
        key: String,
    },
    Group {
        group: input::Group,
    },
    Action {
        info: &'static input::ActionInfo,
    },
    Set {
        name: &'static str,
        infos: &'static [input::ActionInfo],
    },
}

/// A knob is decided after this much time since its first message.
const KNOB_WINDOW: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureSlot {
    Tap,
    Hold,
}

impl GestureSlot {
    fn name(self) -> &'static str {
        match self {
            Self::Tap => "tap",
            Self::Hold => "hold",
        }
    }
}

pub struct Learning {
    pub info: &'static ActionInfo,
    pub slot: GestureSlot,
    first: Option<Instant>,
    /// (profile, CC) -> messages seen.
    moved: HashMap<(String, Key), u32>,
    /// The decided control, waiting on the row for Enter (74).
    pub pending: Option<(String, Key)>,
}

impl Learning {
    pub fn new(info: &'static ActionInfo) -> Self {
        Self {
            info,
            slot: GestureSlot::Tap,
            first: None,
            moved: HashMap::new(),
            pending: None,
        }
    }

    /// Keeps the decided control until Enter (74) and listens again: another control
    /// moved after it takes its place.
    pub fn hold(&mut self, profile: String, key: Key) {
        *self = Self {
            pending: Some((profile, key)),
            slot: self.slot,
            ..Self::new(self.info)
        };
    }

    /// Feeds a message; returns the chosen (profile, control) once decided.
    /// A button action takes the first press; a knob action only collects CCs (see `poll`).
    pub fn feed(&mut self, profile: &str, msg: Msg, now: Instant) -> Option<(String, Key)> {
        let press =
            matches!(msg, Msg::NoteOn { .. }) || matches!(msg, Msg::Cc { value, .. } if value > 0);
        if !self.info.knob {
            return press.then(|| (profile.to_string(), msg.key()));
        }
        if let Msg::Cc { .. } = msg {
            *self
                .moved
                .entry((profile.to_string(), msg.key()))
                .or_default() += 1;
            self.first.get_or_insert(now);
        }
        self.poll(now)
    }

    /// For knob actions: the CC that moved most, once the window after the first message passed;
    /// a neighbour knob touched briefly loses.
    pub fn poll(&mut self, now: Instant) -> Option<(String, Key)> {
        if self
            .first
            .is_none_or(|f| now.duration_since(f) < KNOB_WINDOW)
        {
            return None;
        }
        self.moved
            .iter()
            .max_by_key(|(_, n)| **n)
            .map(|(k, _)| k.clone())
    }
}

/// Assigns `key` of `profile` to `action` in `group`: the action loses its previous learned
/// control and the control loses its previous action in that group.
/// Drops the bindings that name no action the program has (71); returns each as
/// `device group key name` for the log.
pub fn drop_dead(learned: &mut Learned) -> Vec<String> {
    let mut gone = Vec::new();
    let valid_gesture = |name: &str| {
        input::Action::parse(name)
            .is_some_and(|action| !matches!(action, input::Action::PadPress { .. }))
    };
    for (device, groups) in learned.iter_mut() {
        for (group, keys) in groups.iter_mut() {
            keys.retain(|key, binding| match binding {
                MidiBinding::Press(name) => {
                    if input::midi::known(name)
                        || (device == "SMC-PAD"
                            && name == "none"
                            && key.parse::<Key>().is_ok_and(|key| {
                                group == input::Group::Pattern.key()
                                    || matches!(key, Key::Note(52..=67) | Key::Cc(25..=28 | 30..=45))
                            }))
                    {
                        true
                    } else {
                        gone.push(format!("{device} {group} {key} {name}"));
                        false
                    }
                }
                MidiBinding::Gesture { tap, hold } => {
                    if tap.as_ref().is_some_and(|name| !valid_gesture(name)) {
                        gone.push(format!("{device} {group} {key} {}", tap.as_ref().unwrap()));
                        *tap = None;
                    }
                    if valid_gesture(hold) {
                        true
                    } else {
                        gone.push(format!("{device} {group} {key} {hold}"));
                        match tap.take() {
                            Some(name) => {
                                *binding = MidiBinding::Press(name);
                                true
                            }
                            None => false,
                        }
                    }
                }
            });
        }
        groups.retain(|group, keys| group == Group::Fx.key() || !keys.is_empty());
    }
    learned.retain(|_, groups| !groups.is_empty());
    gone
}

pub fn assign(learned: &mut Learned, profile: &str, group: Group, key: Key, action: &str) {
    if group == Group::Fx {
        materialize_fx(
            learned,
            profile,
            &input::midi::Profile::default_for(profile),
            true,
        );
    }
    let map = learned
        .entry(profile.into())
        .or_default()
        .entry(group.key().into())
        .or_default();
    map.retain(|_, binding| without_action(binding, action));
    map.insert(key.to_string(), action.into());
}

pub fn assign_slot(
    learned: &mut Learned,
    profile: &str,
    group: Group,
    key: Key,
    slot: GestureSlot,
    action: &str,
) {
    if group == Group::Fx {
        materialize_fx(
            learned,
            profile,
            &input::midi::Profile::default_for(profile),
            true,
        );
    }
    let map = learned
        .entry(profile.into())
        .or_default()
        .entry(group.key().into())
        .or_default();
    map.retain(|_, binding| without_slot_action(binding, slot, action));
    let target = map.entry(key.to_string()).or_insert_with(|| match slot {
        GestureSlot::Tap => MidiBinding::Press(action.into()),
        GestureSlot::Hold => MidiBinding::Gesture {
            tap: None,
            hold: action.into(),
        },
    });
    match (slot, target) {
        (GestureSlot::Tap, MidiBinding::Press(name)) => *name = action.into(),
        (GestureSlot::Hold, binding @ MidiBinding::Press(_)) => {
            let MidiBinding::Press(old) =
                std::mem::replace(binding, MidiBinding::Press(String::new()))
            else {
                unreachable!();
            };
            *binding = MidiBinding::Gesture {
                tap: Some(old),
                hold: action.into(),
            };
        }
        (GestureSlot::Tap, MidiBinding::Gesture { tap, .. }) => *tap = Some(action.into()),
        (GestureSlot::Hold, MidiBinding::Gesture { hold, .. }) => *hold = action.into(),
    }
}

fn without_slot_action(binding: &mut MidiBinding, slot: GestureSlot, action: &str) -> bool {
    match (slot, binding) {
        (GestureSlot::Tap, MidiBinding::Press(name)) => name != action,
        (GestureSlot::Tap, MidiBinding::Gesture { tap, .. }) => {
            if tap.as_deref() == Some(action) {
                *tap = None;
            }
            true
        }
        (GestureSlot::Hold, binding @ MidiBinding::Gesture { .. }) => {
            let MidiBinding::Gesture { tap, hold } = binding else {
                unreachable!()
            };
            if hold != action {
                return true;
            }
            match tap.take() {
                Some(name) => {
                    *binding = MidiBinding::Press(name);
                    true
                }
                None => false,
            }
        }
        (GestureSlot::Hold, MidiBinding::Press(_)) => true,
    }
}

fn without_action(binding: &mut MidiBinding, action: &str) -> bool {
    match binding {
        MidiBinding::Press(name) => name != action,
        MidiBinding::Gesture { tap, hold } if hold == action => match tap.take() {
            Some(name) if name != action => {
                *binding = MidiBinding::Press(name);
                true
            }
            _ => false,
        },
        MidiBinding::Gesture { tap, .. } => {
            if tap.as_deref() == Some(action) {
                *tap = None;
            }
            true
        }
    }
}

/// The group of a scope (59).
pub fn group_of(scope: input::midi::Scope) -> Option<Group> {
    Group::ALL.into_iter().find(|g| g.scope() == scope)
}

/// Copies the profile roles into `profile`'s bindings (59): a binding already there wins,
/// on the same control or the same action of its group.
#[cfg(test)]
pub fn seed(learned: &mut Learned, profile: &str, roles: &[(input::midi::Scope, Key, String)]) {
    let bindings: Vec<_> = roles
        .iter()
        .map(|(scope, key, action)| (*scope, *key, Binding::Press(action.clone())))
        .collect();
    seed_bindings(learned, profile, &bindings);
}

pub fn seed_bindings(
    learned: &mut Learned,
    profile: &str,
    roles: &[(input::midi::Scope, Key, Binding)],
) {
    let groups = learned.entry(profile.into()).or_default();
    let had_fx = groups.contains_key(Group::Fx.key());
    let existing = groups.clone();
    for (scope, key, action) in roles {
        let Some(group) = group_of(*scope) else {
            continue;
        };
        if group == Group::Fx {
            continue;
        }
        let map = groups.entry(group.key().into()).or_default();
        if map.contains_key(&key.to_string())
            || existing.get(group.key()).is_some_and(|m| {
                m.values().any(|a| match action {
                    Binding::Press(name) => a.has_action(name),
                    Binding::Gesture { tap, hold } => {
                        a.has_action(hold) || tap.as_deref().is_some_and(|name| a.has_action(name))
                    }
                })
            })
        {
            continue;
        }
        map.insert(key.to_string(), action.clone().into());
    }
    materialize_fx(
        learned,
        profile,
        &input::midi::Profile::default_for(profile),
        true,
    );
    if !had_fx {
        for (scope, key, action) in roles {
            if *scope == Group::Fx.scope() {
                assign_binding(learned, profile, Group::Fx, *key, action.clone());
            }
        }
    }
}

fn assign_binding(learned: &mut Learned, profile: &str, group: Group, key: Key, binding: Binding) {
    let map = learned
        .entry(profile.into())
        .or_default()
        .entry(group.key().into())
        .or_default();
    let names: Vec<&str> = match &binding {
        Binding::Press(name) => vec![name],
        Binding::Gesture { tap, hold } => tap
            .iter()
            .map(String::as_str)
            .chain([hold.as_str()])
            .collect(),
    };
    map.retain(|_, existing| names.iter().all(|name| without_action(existing, name)));
    map.insert(key.to_string(), binding.into());
}

fn materialize_fx(
    learned: &mut Learned,
    name: &str,
    profile: &input::midi::Profile,
    seeded: bool,
) -> bool {
    if learned
        .get(name)
        .is_some_and(|groups| groups.contains_key(Group::Fx.key()))
    {
        return false;
    }
    let mut profile = profile.clone();
    profile.set_seeded(seeded);
    for group in Group::ALL {
        if learned
            .get(name)
            .is_some_and(|groups| groups.contains_key(group.key()))
        {
            profile.set_bindings(group.scope(), group_bindings(learned, name, group));
        }
    }
    let map = profile
        .fx_default_bindings()
        .into_iter()
        .map(|(key, action)| (key.to_string(), action.into()))
        .collect();
    learned
        .entry(name.into())
        .or_default()
        .insert(Group::Fx.key().into(), map);
    true
}

pub fn migrate_fx(
    learned: &mut Learned,
    profiles: &[input::midi::Profile],
    seeded: &[String],
) -> bool {
    let names: Vec<_> = learned
        .keys()
        .cloned()
        .chain(seeded.iter().cloned())
        .collect();
    let mut changed = false;
    for name in names {
        let fallback = input::midi::Profile::default_for(&name);
        let profile = profiles
            .iter()
            .find(|p| p.name == name)
            .unwrap_or(&fallback);
        changed |= materialize_fx(learned, &name, profile, seeded.contains(&name));
    }
    changed
}

/// Removes the learned controls of `action` in `group`; returns the profiles that changed.
pub fn clear(learned: &mut Learned, group: Group, action: &str) -> Vec<String> {
    if group == Group::Fx {
        migrate_fx(learned, &[], &[]);
    }
    let mut changed = Vec::new();
    for (profile, groups) in learned.iter_mut() {
        if let Some(map) = groups.get_mut(group.key()) {
            let before = map.clone();
            map.retain(|_, binding| without_action(binding, action));
            if *map != before {
                changed.push(profile.clone());
            }
        }
    }
    changed
}

/// Learned controls of a profile in a group, parsed.
#[cfg(test)]
pub fn group_map(learned: &Learned, profile: &str, group: Group) -> HashMap<Key, String> {
    group_bindings(learned, profile, group)
        .into_iter()
        .map(|(key, binding)| (key, binding.primary().into()))
        .collect()
}

pub fn group_bindings(learned: &Learned, profile: &str, group: Group) -> HashMap<Key, Binding> {
    learned
        .get(profile)
        .and_then(|s| s.get(group.key()))
        .into_iter()
        .flatten()
        .filter_map(|(k, a)| Some((k.parse().ok()?, a.clone().into())))
        .collect()
}

use crate::*;

impl App {
    /// The group a binding of the learn mode goes to (53): the group of the layer in sight,
    /// the edit group while the edit is open; the page knobs of the edit are learned in F12.
    pub(super) fn learn_group(&self) -> input::Group {
        match &self.screen_learn {
            Some(l) => l.group,
            None => self.layer_group(),
        }
    }

    /// The group of the layer in sight (57): the edit group with the edit open.
    pub(super) fn layer_group(&self) -> input::Group {
        if self.edit
            && let Some(group) = self.screen.rules().ui_edit_group
        {
            return group;
        }
        self.screen_group()
    }

    /// The screen's own group, under its edit group.
    pub(super) fn screen_group(&self) -> input::Group {
        // Keep the existing Settings helper result, not its registry identity.
        if self.screen == Screen::Settings {
            input::Group::Files
        } else {
            self.screen.group()
        }
    }

    /// The learn mode as `F10` opens it (57): with the action done in the last 5 s picked,
    /// in its layer and page, or with nothing picked in the layer in sight.
    pub(super) fn learn_open(&mut self) -> ScreenLearn {
        let last = self
            .last_done
            .take()
            .filter(|d| d.at.elapsed() <= LAST_DONE && d.group == self.layer_group());
        let mut l = ScreenLearn {
            picked: None,
            pending: None,
            clearing: false,
            learner: None,
            ignore: None,
            refused: false,
            group: self.layer_group(),
            page: (self.edit && self.screen == Screen::Pads).then(|| self.pads.page().0),
            once: false,
        };
        if let Some(d) = last {
            l.learner = Some(learn::Learning::new(action_info(d.name)));
            l.picked = Some((d.name, d.label));
            l.page = d.page;
            l.once = true;
        }
        l
    }

    /// Notes what an action did, for "do, then F10" (57): a pad hit its pad, a value step
    /// the parameter it changed, a command its entry of the learn list.
    pub(super) fn note_done(&mut self, action: Action) {
        // Moving the panel views to reach Learn is not the action to learn (127).
        if self.screen_learn.is_some()
            || action == Action::LearnToggle
            || (matches!(action, Action::SmcList { .. }) && self.panel.visible())
        {
            return;
        }
        let group = self.layer_group();
        let page = (self.edit && self.screen == Screen::Pads).then(|| self.pads.page().0);
        let found: Option<(&'static str, String)> = match action {
            Action::PadPress { pad, .. } => {
                let key = tui::pads::pad_tag(pad);
                Some((
                    action_info(&format!("pad.{}", pad + 1)).name,
                    format!("pad {key}"),
                ))
            }
            Action::EditValue { .. } | Action::PadParam(_) if self.screen == Screen::Pads => {
                let label = pads::name_of(self.pads.param);
                self.learn_params()
                    .iter()
                    .find(|(_, l)| *l == label)
                    .map(|&(n, l)| (n, l.to_string()))
            }
            Action::EditValue { .. } | Action::PadParam(_) if self.screen == Screen::Fx => {
                let (name, _) = self.learn_params()[self.fx_param];
                Some((name, self.pads.name(self.fx_selected_param()).into()))
            }
            Action::EditValue { .. } if self.screen == Screen::Chop => Some(if self.slice_end {
                ("slice.end", "slice end".into())
            } else {
                ("slice.start", "slice start".into())
            }),
            a => {
                let groups = [group, self.screen_group(), input::Group::Global];
                input::ACTIONS
                    .iter()
                    .find(|i| {
                        !i.knob && groups.contains(&i.group) && Action::parse(i.name) == Some(a)
                    })
                    .map(|i| (i.name, i.label.to_string()))
            }
        };
        if let Some((name, label)) = found {
            self.last_done = Some(LastDone {
                at: Instant::now(),
                name,
                label,
                group,
                page,
            });
        }
    }

    /// The parameters the learn mode walks with `I` `K` (52): they have no key of their own.
    pub(super) fn learn_params(&self) -> &'static [(&'static str, &'static str)] {
        match self.screen {
            Screen::Fx => &[
                ("edit.param_1", "effect type"),
                ("edit.param_2", "first parameter"),
                ("edit.param_3", "second parameter"),
                ("edit.page", "effect block"),
            ],
            Screen::Pads => &[
                ("pad.volume", "volume"),
                ("pad.pitch", "pitch"),
                ("pad.attack", "attack"),
                ("pad.decay", "decay"),
                ("pad.release", "release"),
                ("pad.filter", "filter"),
                ("pad.cutoff", "cutoff"),
                ("pad.reso", "reso"),
                ("pad.start", "start"),
                ("pad.end", "end"),
            ],
            Screen::Chop => &[
                ("slice.start", "slice start"),
                ("slice.end", "slice end"),
                ("chop.cursor_pos", "cursor"),
                ("chop.zoom", "zoom"),
                ("chop.region_start_pos", "region start"),
                ("chop.region_end_pos", "region end"),
                ("chop.threshold", "threshold"),
            ],
            _ => &[],
        }
    }

    /// Picks `name` in the learn mode: it lights and waits for a control.
    pub(super) fn learn_pick(&mut self, name: &'static str, label: String) {
        // a pick by a key hears every control again; the move after a binding sets `ignore`
        let (group, page, once) = match &self.screen_learn {
            Some(l) => (l.group, l.page, l.once),
            None => (self.layer_group(), None, false),
        };
        self.screen_learn = Some(ScreenLearn {
            picked: Some((name, label)),
            pending: None,
            clearing: false,
            learner: Some(learn::Learning::new(action_info(name))),
            ignore: None,
            refused: false,
            group,
            page,
            once,
        });
    }

    /// The element after `name` (53): the next pad, the next parameter, the next command of
    /// the screen's learn list; `None` past the last.
    pub(super) fn learn_next(&self, name: &str) -> Option<(&'static str, String)> {
        if let Some(n) = name
            .strip_prefix("pad.")
            .and_then(|n| n.parse::<usize>().ok())
        {
            return (n < PADS_PER_BANK).then(|| {
                let key = tui::pads::pad_tag(n);
                (
                    action_info(&format!("pad.{}", n + 1)).name,
                    format!("pad {key}"),
                )
            });
        }
        let params = self.learn_params();
        if let Some(i) = params.iter().position(|(n, _)| *n == name) {
            return params.get(i + 1).map(|&(n, l)| (n, l.to_string()));
        }
        let group = self.learn_group();
        let list: Vec<_> = input::ACTIONS
            .iter()
            .filter(|i| !i.knob && (i.group == group || i.group == input::Group::Global))
            .collect();
        let at = list.iter().position(|i| i.name == name)?;
        list.get(at + 1).map(|i| (i.name, i.label.to_string()))
    }

    /// Binds the waiting or given control to the picked element and moves on (53).
    pub(super) fn learn_save(&mut self, profile: String, key: input::midi::Key) {
        let group = self.learn_group();
        let Some((name, label)) = self.learn_picked() else {
            return;
        };
        learn::assign(&mut self.learned, &profile, group, key, name);
        self.sync_learned(&profile, group);
        self.save_midi();
        log::info!("MIDI learn on screen: {profile} {key} -> {name}");
        self.post_notice(Level::Done, (format!("{key} → {label}"), Instant::now()));
        // "do, then F10" binds one element: nothing to walk on to (57)
        if self.screen_learn.as_ref().is_some_and(|l| l.once) {
            self.screen_learn = None;
            return;
        }
        match self.learn_next(name) {
            Some((next, next_label)) => self.learn_pick(next, next_label),
            None => self.learn_pick(name, label),
        }
        if let Some(l) = self.screen_learn.as_mut() {
            l.ignore = Some(key);
        }
    }

    /// `I` `K` in the learn mode: the next or previous parameter of the screen.
    pub(super) fn learn_walk(&mut self, step: isize) {
        let params = self.learn_params();
        if params.is_empty() {
            return;
        }
        let at = self
            .learn_picked()
            .and_then(|(name, _)| params.iter().position(|(n, _)| *n == name));
        let next = match at {
            Some(i) => (i as isize + step).rem_euclid(params.len() as isize) as usize,
            None if step > 0 => 0,
            None => params.len() - 1,
        };
        let (name, label) = params[next];
        self.learn_pick(name, label.into());
    }

    /// The picked element of the learn mode: (action, what the screen calls it).
    pub(super) fn learn_picked(&self) -> Option<(&'static str, String)> {
        self.screen_learn.as_ref()?.picked.clone()
    }

    /// A control moved in the learn mode (53): a free one is bound at once and the pick moves
    /// on; one that drives something else waits for `Enter`; the control just bound is not
    /// heard; a knob does not drive a command.
    pub(super) fn learn_bind(&mut self, profile: String, key: input::midi::Key) {
        let Some((name, _)) = self.learn_picked() else {
            return;
        };
        let renew = |l: &mut ScreenLearn| l.learner = Some(learn::Learning::new(action_info(name)));
        if self
            .screen_learn
            .as_ref()
            .is_some_and(|l| l.ignore == Some(key))
        {
            if let Some(l) = self.screen_learn.as_mut() {
                renew(l);
            }
            return;
        }
        let knob = self
            .live_profiles()
            .iter()
            .find(|p| p.name == profile)
            .is_none_or(|p| p.is_knob(key));
        if knob != action_info(name).knob {
            if let Some(l) = self.screen_learn.as_mut() {
                l.refused = true;
                renew(l);
            }
            return;
        }
        // every control waits for Enter next to the old one (57); another control moved
        // before the answer takes its place, so the learner starts over
        if let Some(l) = self.screen_learn.as_mut() {
            l.pending = Some((profile, key));
            l.clearing = false;
            l.ignore = None;
            l.refused = false;
            renew(l);
        }
    }

    /// What the label of `name` says in the learn mode (57): its control, and for the
    /// picked element with a control waiting `old → new` (`none` when there is none).
    pub(super) fn learn_shown(&self, profiles: &[Profile], name: &str) -> Option<String> {
        let now = self.learn_label(profiles, name);
        let l = self.screen_learn.as_ref()?;
        let picked = l.picked.as_ref().is_some_and(|(n, _)| *n == name);
        let new = match (&l.pending, l.clearing) {
            _ if !picked => return now,
            (Some((profile, key)), _) => control_name(profiles, profile, *key),
            (None, true) => "none".into(),
            // the picked element says what drives it, "none" too (57)
            (None, false) => return now.or_else(|| Some("none".into())),
        };
        Some(format!("{} → {new}", now.as_deref().unwrap_or("none")))
    }

    /// `Enter` in the learn mode: the waiting control drives the picked element, or the
    /// asked clear is done; `false` when nothing waited.
    pub(super) fn learn_confirm(&mut self) -> bool {
        let group = self.learn_group();
        let Some(l) = self.screen_learn.as_mut() else {
            return false;
        };
        let Some((name, label)) = l.picked.clone() else {
            return false;
        };
        if let Some((profile, key)) = l.pending.take() {
            self.learn_save(profile, key);
            return true;
        }
        if std::mem::take(&mut l.clearing) {
            for profile in learn::clear(&mut self.learned, group, name) {
                self.sync_learned(&profile, group);
            }
            self.save_midi();
            self.post_notice(Level::Done, (format!("cleared: {label}"), Instant::now()));
            return true;
        }
        false
    }

    /// A key while the learn mode is open; every key stays in the mode (52): it picks the
    /// element it drives outside the mode, `I` `K` walk the parameters, `Enter` saves,
    /// `Esc` drops, `Backspace` asks to clear, the learn key (`F10`, 57) leaves.
    pub(super) fn learn_key(&mut self, keymap: &Keymap, k: &event::KeyEvent) -> bool {
        use ratatui::crossterm::event::{KeyCode, KeyEventKind};
        if k.kind == KeyEventKind::Release {
            return true;
        }
        let waiting = self
            .screen_learn
            .as_ref()
            .is_some_and(|l| l.pending.is_some() || l.clearing);
        // the key of `learn.toggle` in the keymap leaves the mode
        if keymap.resolve_in(self.screen, false, k) == Some(Action::LearnToggle) {
            self.screen_learn = None;
            return true;
        }
        match k.code {
            KeyCode::Esc if waiting => {
                if let Some(l) = self.screen_learn.as_mut() {
                    (l.pending, l.clearing) = (None, false);
                }
                return true;
            }
            KeyCode::Esc => {
                self.screen_learn = None;
                return true;
            }
            KeyCode::Enter if waiting => {
                self.learn_confirm();
                return true;
            }
            KeyCode::Backspace => {
                let bound = self.learn_picked().is_some_and(|(name, _)| {
                    self.learn_label(&self.live_profiles(), name).is_some()
                });
                if let Some(l) = self.screen_learn.as_mut().filter(|_| bound) {
                    (l.pending, l.clearing) = (None, true);
                }
                return true;
            }
            _ => {}
        }
        if waiting {
            return true;
        }
        // the parameters have no key of their own: the edit layer's I K walk them
        match keymap.resolve_in(self.screen, true, k) {
            Some(Action::ParamNext) => {
                self.learn_walk(1);
                return true;
            }
            Some(Action::ParamPrev) => {
                self.learn_walk(-1);
                return true;
            }
            _ => {}
        }
        match keymap.resolve_in(self.screen, self.edit_keys(), k) {
            Some(Action::PadPress { pad, .. }) => {
                let key = tui::pads::pad_tag(pad);
                self.learn_pick(
                    action_info(&format!("pad.{}", pad + 1)).name,
                    format!("pad {key}"),
                );
            }
            Some(action) => {
                // the action of the learn list this key drives, in this screen's group or
                // the global one
                let groups = [
                    self.learn_group(),
                    self.screen_group(),
                    input::Group::Global,
                ];
                let found = input::ACTIONS.iter().find(|i| {
                    !i.knob && groups.contains(&i.group) && Action::parse(i.name) == Some(action)
                });
                if let Some(info) = found {
                    self.learn_pick(info.name, info.label.into());
                }
            }
            None => {}
        }
        true
    }

    /// The control that drives `name` in the layer in sight, short: `k5`, `cc 28`,
    /// `note 36`; `None` when none does.
    pub(super) fn learn_label(&self, profiles: &[Profile], name: &str) -> Option<String> {
        let scope = self.learn_group().scope();
        profiles
            .iter()
            .find_map(|p| p.assignment(scope, name))
            .map(|a| a.split(" (").next().unwrap_or_default().to_string())
    }

    /// The element a waiting control drives now on this screen, which would lose it:
    /// (action, label).
    pub(super) fn learn_taken(&self) -> Option<(&'static str, String)> {
        let l = self.screen_learn.as_ref()?;
        let (_, key) = l.pending.as_ref()?;
        let (picked, _) = l.picked.clone()?;
        self.taken_by(*key, picked)
    }

    /// The element other than `picked` that control `key` drives now on this screen.
    pub(super) fn taken_by(
        &self,
        key: input::midi::Key,
        picked: &str,
    ) -> Option<(&'static str, String)> {
        self.taken_in(self.learn_group(), key, picked)
    }

    /// The action other than `picked` that control `key` drives now in `group`.
    pub(super) fn taken_in(
        &self,
        group: input::Group,
        key: input::midi::Key,
        picked: &str,
    ) -> Option<(&'static str, String)> {
        let scope = group.scope();
        let profiles = self.live_profiles();
        // "cc 28" or "k5 (cc 28)": no "(learned)" mark since 59
        let (learned, named) = (key.to_string(), format!("({key})"));
        input::ACTIONS
            .iter()
            .filter(|i| i.name != picked)
            .find(|i| {
                profiles.iter().any(|p| {
                    p.assignment(scope, i.name)
                        .is_some_and(|a| a == learned || a.ends_with(&named))
                })
            })
            .map(|i| (i.name, i.label.to_string()))
    }

    pub(super) fn taken_in_slot(
        &self,
        profile: &str,
        group: input::Group,
        key: input::midi::Key,
        picked: &str,
        slot: learn::GestureSlot,
    ) -> Option<(&'static str, String)> {
        let binding = self
            .learned
            .get(profile)?
            .get(group.key())?
            .get(&key.to_string())?;
        let name = match (slot, binding) {
            (learn::GestureSlot::Tap, config::MidiBinding::Press(name)) => name.as_str(),
            (
                learn::GestureSlot::Tap,
                config::MidiBinding::Gesture {
                    tap: Some(name), ..
                },
            ) => name.as_str(),
            (learn::GestureSlot::Hold, config::MidiBinding::Gesture { hold, .. }) => hold.as_str(),
            _ => return None,
        };
        if name == picked {
            return None;
        }
        input::ACTIONS
            .iter()
            .find(|i| i.name == name)
            .map(|i| (i.name, i.label.to_string()))
    }

    /// The grid cells in the learn mode: a bound pad carries its control, the picked one
    /// stands out, the one a waiting control would leave is marked (52).
    pub(super) fn learn_grid(&self, cells: &mut [tui::PadView]) {
        if self.screen_learn.is_none() {
            return;
        }
        let profiles = self.live_profiles();
        let picked = self.learn_picked().map(|(n, _)| n);
        let taken = self.learn_taken().map(|(n, _)| n);
        for (i, cell) in cells.iter_mut().enumerate().take(PADS_PER_BANK) {
            let name = format!("pad.{}", i + 1);
            cell.note = self.learn_shown(&profiles, &name).map(|c| format!("◀ {c}"));
            cell.in_hand = picked == Some(action_info(&name).name);
            cell.marked = taken == Some(action_info(&name).name);
        }
    }

    /// The panel in the learn mode: the bound parameters carry their control, the picked
    /// one is lit, the one a waiting control would leave says so.
    pub(super) fn learn_panel(&self, v: &mut tui::Pads) {
        if self.screen_learn.is_none() || self.screen != Screen::Pads {
            return;
        }
        let profiles = self.live_profiles();
        let picked = self.learn_picked().map(|(n, _)| n);
        let taken = self.learn_taken().map(|(n, _)| n);
        for &(name, label) in self.learn_params() {
            let Some(p) = pads::ORDER_ALL
                .into_iter()
                .find(|&p| pads::name_of(p) == label)
            else {
                continue;
            };
            let control = self.learn_shown(&profiles, name);
            if picked == Some(name) {
                v.lit = Some(tui_param(p));
            }
            match (control, taken == Some(name)) {
                (Some(c), true) => v.learn.push((tui_param(p), format!("{c}  ✗ leaves"))),
                (Some(c), false) => v.learn.push((tui_param(p), c)),
                (None, _) => {}
            }
        }
    }

    /// Learning messages and confirmation details before the fixed-band view arranges them.
    pub(super) fn learn_lines(&self) -> Option<Vec<String>> {
        let l = self.screen_learn.as_ref()?;
        let profiles = self.live_profiles();
        let picked = l.picked.clone();
        // where the binding goes (57): the layer and the page of the edit
        let place = match l.page {
            Some(page) => format!("{} · {page}", l.group.title()),
            None => l.group.title().to_string(),
        };
        // "do, then F10": say where the pick came from and how to pick another (57)
        let from = if l.once {
            " (the last thing you did)"
        } else {
            ""
        };
        let other = " · I K pick a parameter, a pad or a command key picks another";
        let lines = match (&picked, &l.pending, l.clearing) {
            (Some((name, label)), Some((profile, key)), _) => {
                let shown = self.learn_shown(&profiles, name).unwrap_or_default();
                let first = format!(" LEARN · {label} · {place} · {shown} · Enter bind · Esc keep");
                let second = match self.learn_taken() {
                    Some((_, other)) => format!(
                        " ✗ {key} is taken on this layer: \"{other}\" is marked and loses it"
                    ),
                    None => match self.bound_elsewhere(profile, *key) {
                        Some(note) => format!(" {note} · that layer keeps it"),
                        None => " another control takes its place".to_string(),
                    },
                };
                vec![first, second]
            }
            (Some((name, label)), None, true) => {
                let shown = self.learn_shown(&profiles, name).unwrap_or_default();
                vec![
                    format!(" LEARN · {label} · {place} · {shown} · Enter clear · Esc keep"),
                    " Backspace removes the control of the picked element".into(),
                ]
            }
            (Some((name, label)), None, false) if l.refused && action_info(name).knob => vec![
                format!(" LEARN · \"{label}\"{from} needs a knob or a fader, not a button"),
                format!(
                    " it has a value to turn, a button only presses · turn a knob{other} · F10 done"
                ),
            ],
            (Some((_, label)), None, false) if l.refused => vec![
                format!(" LEARN · \"{label}\"{from} needs a button or a pad, not a knob"),
                format!(
                    " it acts once per press, a knob would repeat it on every tick · press a button{other} · F10 done"
                ),
            ],
            (Some((name, label)), None, false) => {
                let now = match self.learn_label(&profiles, name) {
                    Some(c) => format!("now {c}"),
                    None => "now none".into(),
                };
                vec![
                    format!(" LEARN · picked: {label}{from} · {place} · {now} · move a control on the controller"),
                    " a pad or a command key picks another · I K the parameters · Backspace clear · F10 done".into(),
                ]
            }
            (None, ..) => vec![
                format!(
                    " LEARN · {place} · press what to bind: a pad, a command key, PgUp PgDn; I K the parameters"
                ),
                " then move a control on the controller · F10 done".into(),
            ],
        };
        Some(lines)
    }

    /// Where else on this screen a control is bound, outside the layer of the binding (57):
    /// `k5 on PADS: volume`.
    pub(super) fn bound_elsewhere(&self, profile: &str, key: input::midi::Key) -> Option<String> {
        let groups = [
            self.screen_group(),
            self.layer_group(),
            input::Group::Global,
        ];
        self.bound_in(profile, key, self.learn_group(), &groups)
            .into_iter()
            .next()
    }

    /// Each action control `key` keeps in `groups` other than `here`:
    /// `b9 on GLOBAL: bank next`.
    pub(super) fn bound_in(
        &self,
        profile: &str,
        key: input::midi::Key,
        here: input::Group,
        groups: &[input::Group],
    ) -> Vec<String> {
        let named = control_name(&self.live_profiles(), profile, key);
        groups
            .iter()
            .filter(|g| **g != here)
            .filter_map(|&g| {
                let map = learn::group_bindings(&self.learned, profile, g);
                let action = map.get(&key)?.primary();
                let label = input::ACTIONS
                    .iter()
                    .find(|i| i.name == action)
                    .map_or(action, |i| i.label);
                Some(format!("{named} on {}: {label}", g.title()))
            })
            .collect()
    }
    /// The profiles of the connected devices with the learned assignments over them.
    pub(super) fn live_profiles(&self) -> Vec<Profile> {
        let maps = midi::parse_learned(&self.learned);
        let mut profiles: Vec<Profile> = Vec::new();
        for (port, name) in &self.midi_devices {
            // one device often has several ports with the same profile
            if name != "default" && profiles.iter().any(|p| p.name == *name) {
                continue;
            }
            let mut p = self
                .profiles
                .iter()
                .find(|p| p.name == *name)
                .cloned()
                .unwrap_or_else(|| Profile::default_for(port));
            midi::apply_learned(&mut p, &maps);
            p.set_seeded(self.seeded.contains(&p.name));
            profiles.push(p);
        }
        profiles
    }

    /// Copies the roles of a device's profile into its bindings once (59), or again with
    /// `reset`, which drops the owner's bindings of that device first; from then on only
    /// the bindings act for it.
    pub(super) fn seed_device(&mut self, port: &str, name: &str, reset: bool) {
        if !reset && self.seeded.iter().any(|s| s == name) {
            if name == "SMC-PAD"
                && !self
                    .learned
                    .get(name)
                    .is_some_and(|groups| groups.contains_key(input::Group::Pattern.key()))
                && let Some(profile) = self.profiles.iter().find(|profile| profile.name == name)
            {
                let roles: Vec<_> = profile
                    .binding_roles()
                    .into_iter()
                    .filter(|(scope, _, _)| *scope == input::midi::Scope::Pattern)
                    .collect();
                if !roles.is_empty() {
                    learn::seed_bindings(&mut self.learned, name, &roles);
                    self.sync_learned(name, input::Group::Pattern);
                    self.save_midi();
                }
            }
            self.migrate_fx_bindings();
            return;
        }
        let profile = self
            .profiles
            .iter()
            .find(|p| p.name == name)
            .cloned()
            .unwrap_or_else(|| Profile::default_for(port));
        if reset {
            self.learned.remove(name);
        }
        learn::seed_bindings(&mut self.learned, name, &profile.binding_roles());
        if !self.seeded.iter().any(|s| s == name) {
            self.seeded.push(name.into());
        }
        for g in input::Group::ALL {
            self.sync_learned(name, g);
        }
        if let Some(m) = &self.midi {
            m.set_seeded(name);
        }
        for p in self.profiles.iter_mut().filter(|p| p.name == name) {
            p.set_seeded(true);
        }
        self.save_midi();
        let section = config::DevicesConfig {
            seeded: self.seeded.clone(),
        };
        if let Some(p) = &self.user_path
            && let Err(e) = config::save_section(p, "devices", &section)
        {
            log::error!("{e}");
        }
        log::info!("MIDI: {name} bindings from its profile");
    }

    /// Sends the learned controls of `profile` in `group` to the MIDI thread and the local copy.
    pub(super) fn sync_learned(&mut self, profile: &str, group: input::Group) {
        let map = learn::group_bindings(&self.learned, profile, group);
        if let Some(m) = &self.midi {
            m.set_learned(profile, group, map.clone());
        }
        for p in self.profiles.iter_mut().filter(|p| p.name == profile) {
            p.set_bindings(group.scope(), map.clone());
        }
    }

    pub(super) fn migrate_fx_bindings(&mut self) {
        if learn::migrate_fx(&mut self.learned, &self.profiles, &self.seeded) {
            let names: Vec<_> = self.learned.keys().cloned().collect();
            for name in names {
                self.sync_learned(&name, input::Group::Fx);
            }
            self.save_midi();
        }
    }

    /// MIDI bindings of the config that name no action are dropped and the config saved
    /// once (71); the log lists them, the notice counts them. Nothing to drop: no notice.
    pub(super) fn drop_dead_bindings(&mut self) -> Option<String> {
        let gone = learn::drop_dead(&mut self.learned);
        if gone.is_empty() {
            return None;
        }
        for g in &gone {
            log::warn!("MIDI binding to no action dropped: {g}");
        }
        self.save_midi();
        let n = gone.len();
        let s = if n == 1 { "" } else { "s" };
        Some(format!(
            "MIDI: {n} binding{s} to actions the program does not have dropped from the config"
        ))
    }

    pub(super) fn save_midi(&mut self) {
        let result = match &self.user_path {
            Some(p) => config::save_section(p, "midi", &self.learned),
            None => Err("MIDI assignments are not saved: fix the config file first".into()),
        };
        if let Err(e) = result {
            log::error!("{e}");
            self.post_notice(Level::Error, (e, Instant::now()));
        }
    }

    /// A control decided on the MIDI tab waits on its row for Enter (74).
    pub(super) fn learned(&mut self, profile: String, key: input::midi::Key) {
        if let Some(l) = self.learning.as_mut() {
            l.hold(profile, key);
        }
    }

    /// Enter on the MIDI tab with a control waiting: it is assigned and saved (74).
    pub(super) fn learned_save(&mut self) {
        let Some(l) = self.learning.take_if(|l| l.pending.is_some()) else {
            return;
        };
        let Some((profile, key)) = l.pending else {
            return;
        };
        if l.info.knob {
            learn::assign(&mut self.learned, &profile, l.info.group, key, l.info.name);
        } else {
            learn::assign_slot(
                &mut self.learned,
                &profile,
                l.info.group,
                key,
                l.slot,
                l.info.name,
            );
        }
        self.sync_learned(&profile, l.info.group);
        self.save_midi();
        log::info!("MIDI learn: {profile} {key} -> {}", l.info.name);
    }

    /// The rows of the MIDI learn list: a row per group and, under an open group, a row
    /// per its action; the pads and the banks stand in one row each (28).
    pub(super) fn midi_list(&self) -> Vec<MidiListRow> {
        let sets = midi_sets();
        // the knob response first: one setting for every knob (58)
        let mut rows = vec![MidiListRow::Response, MidiListRow::HoldTime];
        if self
            .help_overlay
            .controller
            .smc_status
            .as_ref()
            .is_some_and(|(ready, _)| *ready)
            && self.midi_devices.iter().any(|(port, name)| {
                name == "SMC-PAD" && port.contains("SMC-PAD-Master") && !port.contains("(virtual)")
            })
        {
            rows.push(MidiListRow::SmcPadBank);
        }
        let changes = self.smc_layout_slots();
        if !changes.is_empty() {
            rows.push(MidiListRow::SmcLayoutList(changes.len()));
            if self.help_overlay.controller.smc_layout_open {
                rows.extend(changes.into_iter().map(MidiListRow::SmcLayoutSlot));
                rows.push(MidiListRow::SmcLayoutApply);
            }
        }
        for (port, name) in &self.midi_devices {
            if !rows
                .iter()
                .any(|r| matches!(r, MidiListRow::Reset { name: n, .. } if n == name))
            {
                rows.push(MidiListRow::Reset {
                    name: name.clone(),
                    port: port.clone(),
                });
            }
        }
        let mut disabled = self
            .learned
            .get("SMC-PAD")
            .into_iter()
            .flat_map(|groups| {
                groups.iter().flat_map(|(group, controls)| {
                    controls.iter().filter_map(|(key, binding)| {
                        let parsed = key.parse::<Key>().ok()?;
                        (matches!(binding, MidiBinding::Press(name) if name == "none")
                            && matches!(parsed, Key::Note(52..=67) | Key::Cc(25..=28 | 30..=45)))
                        .then(|| MidiListRow::Disabled {
                            profile: "SMC-PAD".into(),
                            group: group.clone(),
                            key: key.clone(),
                        })
                    })
                })
            })
            .collect::<Vec<_>>();
        disabled.sort_by(|a, b| match (a, b) {
            (
                MidiListRow::Disabled {
                    group: ga, key: ka, ..
                },
                MidiListRow::Disabled {
                    group: gb, key: kb, ..
                },
            ) => (ga, ka).cmp(&(gb, kb)),
            _ => std::cmp::Ordering::Equal,
        });
        rows.extend(disabled);
        let mut prev: Option<input::Group> = None;
        for info in input::ACTIONS {
            if prev != Some(info.group) {
                rows.push(MidiListRow::Group { group: info.group });
                prev = Some(info.group);
            }
            if !self.midi_open.contains(&info.group) {
                continue;
            }
            // the row of a set replaces the rows of the numbered pads and banks in it
            if let Some(&(name, infos)) = sets
                .iter()
                .find(|(_, s)| s.first().is_some_and(|f| f.name == info.name))
            {
                rows.push(MidiListRow::Set { name, infos });
                continue;
            }
            let numbered = (info.name.starts_with("pad.") || info.name.starts_with("bank."))
                && info
                    .name
                    .split('.')
                    .nth(1)
                    .is_some_and(|s| s.parse::<usize>().is_ok());
            if !numbered {
                rows.push(MidiListRow::Action { info });
            }
        }
        rows
    }

    /// Any profile has a learned control for `action` in `group`.
    pub(super) fn learned_any(&self, group: input::Group, action: &str) -> bool {
        self.learned.values().any(|g| {
            g.get(group.key())
                .is_some_and(|m| m.values().any(|a| a.has_action(action)))
        })
    }

    /// How many of the run row's actions have a learned control (28).
    pub(super) fn learned_in(&self, infos: &[input::ActionInfo]) -> usize {
        infos
            .iter()
            .filter(|i| self.learned_any(i.group, i.name))
            .count()
    }

    /// How many actions of `group` have a learned control, over every profile.
    pub(super) fn learned_count(&self, group: input::Group) -> usize {
        input::ACTIONS
            .iter()
            .filter(|i| i.group == group && self.learned_any(group, i.name))
            .count()
    }

    /// The rows the MIDI tab draws, from the list with its folded groups.
    pub(super) fn midi_screen(&self) -> Vec<tui::MidiRow> {
        let profiles = self.live_profiles();
        let who = |info: &input::ActionInfo| -> String {
            let who: Vec<String> = profiles
                .iter()
                .filter_map(|p| {
                    Some(format!(
                        "{}: {}",
                        p.name,
                        p.assignment(info.group.scope(), info.name)?
                    ))
                })
                .collect();
            if who.is_empty() {
                "-".into()
            } else {
                who.join("; ")
            }
        };
        // a control waiting for Enter (74): its row shows `old → new` with what the control
        // keeps in the other groups, the row of its group that loses it is marked
        let pending = self
            .learning
            .as_ref()
            .and_then(|l| Some((l.info, l.slot, l.pending.clone()?)));
        let taken = pending
            .as_ref()
            .and_then(|(i, slot, (profile, key))| {
                self.taken_in_slot(profile, i.group, *key, i.name, *slot)
            })
            .map(|t| t.0);
        let action = |info: &input::ActionInfo| {
            let mut row = (who(info), false, Vec::new());
            let Some((p, _, (profile, key))) = &pending else {
                return row;
            };
            let named = control_name(&profiles, profile, *key);
            if p.group != info.group {
                return row;
            }
            if p.name == info.name {
                let now = if row.0 == "-" { "none" } else { row.0.as_str() };
                row.0 = format!("{now} → {named}");
                row.2 = self
                    .bound_in(profile, *key, info.group, &input::Group::ALL)
                    .into_iter()
                    .map(|b| format!("{b} · stays"))
                    .collect();
            } else if taken == Some(info.name) {
                row.0 = format!("{}  ✗ loses {named}", row.0);
                row.1 = true;
            }
            row
        };
        self.midi_list()
            .iter()
            .map(|row| match row {
                MidiListRow::Group { group, .. } => tui::MidiRow::Group {
                    name: group.title(),
                    counts: format!(
                        "{} actions · {} learned",
                        input::ACTIONS.iter().filter(|i| i.group == *group).count(),
                        self.learned_count(*group)
                    ),
                },
                MidiListRow::Action { info } => {
                    let (who, taken, stays) = action(info);
                    tui::MidiRow::Action {
                        label: info.label.to_string(),
                        who,
                        taken,
                        stays,
                    }
                }
                MidiListRow::SmcLayoutList(count) => tui::MidiRow::Action {
                    label: "SMC-PAD layout review".into(),
                    who: format!("{count} changes"),
                    taken: false,
                    stays: Vec::new(),
                },
                MidiListRow::SmcLayoutApply => tui::MidiRow::Action {
                    label: "Apply all layout changes".into(),
                    who: String::new(),
                    taken: false,
                    stays: Vec::new(),
                },
                MidiListRow::SmcLayoutSlot(slot) => tui::MidiRow::Action {
                    label: slot.label(),
                    who: slot.summary(),
                    taken: false,
                    stays: Vec::new(),
                },
                MidiListRow::Reset { name, .. } => tui::MidiRow::Action {
                    label: "reset to profile".into(),
                    who: name.clone(),
                    taken: false,
                    stays: Vec::new(),
                },
                MidiListRow::Disabled {
                    profile,
                    group,
                    key,
                } => tui::MidiRow::Action {
                    label: format!("disabled {key}"),
                    who: format!("{profile} {group}: none · clear to restore"),
                    taken: false,
                    stays: Vec::new(),
                },
                MidiListRow::Response => tui::MidiRow::Action {
                    label: "knob response".into(),
                    who: format!(
                        "{}   (jump / pickup / scale)",
                        self.takeover.response.name()
                    ),
                    taken: false,
                    stays: Vec::new(),
                },
                MidiListRow::HoldTime => tui::MidiRow::Action {
                    label: "hold time".into(),
                    who: format!("{} ms", self.midi_gestures.hold_time_ms),
                    taken: false,
                    stays: Vec::new(),
                },
                MidiListRow::SmcPadBank => tui::MidiRow::Action {
                    label: "SMC-PAD PAD BANK".into(),
                    who: if self.midi_gestures.smc_pad_bank_opens_panel {
                        "Hold panel"
                    } else {
                        "Off"
                    }
                    .into(),
                    taken: false,
                    stays: Vec::new(),
                },
                MidiListRow::Set { name, infos } => tui::MidiRow::Action {
                    label: name.to_string(),
                    who: format!("{} of {} learned", self.learned_in(infos), infos.len()),
                    taken: false,
                    stays: Vec::new(),
                },
            })
            .collect()
    }

    /// Starts the run of a pads or banks row at the first not learned one, or at 1 when
    /// all are (28).
    pub(super) fn midi_run_start(&mut self, infos: &'static [input::ActionInfo]) {
        self.learning = None;
        let at = infos
            .iter()
            .position(|i| !self.learned_any(i.group, i.name))
            .unwrap_or(0);
        self.midi_run = Some(MidiRun {
            infos,
            at,
            taken: std::collections::HashSet::new(),
            learner: learn::Learning::new(&infos[at]),
        });
    }

    /// One control of the run decided: it is learned for the current action and the run
    /// moves on; it ends after the last one (28).
    pub(super) fn midi_run_hit(&mut self, profile: String, key: input::midi::Key) {
        let Some(r) = self.midi_run.as_mut() else {
            return;
        };
        let infos = r.infos;
        let info = &infos[r.at];
        let last = r.at + 1 >= infos.len();
        r.taken.insert(key);
        if last {
            self.midi_run = None;
        } else {
            r.at += 1;
            r.learner = learn::Learning::new(&infos[r.at]);
        }
        learn::assign(&mut self.learned, &profile, info.group, key, info.name);
        self.sync_learned(&profile, info.group);
        self.save_midi();
        log::info!("MIDI learn: {profile} {key} -> {}", info.name);
    }

    /// The line of the MIDI tab about waiting for a control: a single learn or a run (28).
    pub(super) fn learning_line(&self) -> Option<String> {
        if let Some(r) = &self.midi_run {
            let infos = r.infos;
            return Some(format!(
                " {} of {}: hit a control, Esc stops",
                infos[r.at].label,
                infos.len()
            ));
        }
        let Some(l) = self.learning.as_ref() else {
            if self.screen != Screen::Settings || self.settings_tab != 2 {
                return None;
            }
            return match self.midi_list().get(self.midi_row) {
                Some(MidiListRow::SmcLayoutList(_)) => Some(" Enter: show or hide changes".into()),
                Some(MidiListRow::SmcLayoutSlot(_)) => Some(" Enter: review this change".into()),
                Some(MidiListRow::SmcLayoutApply) => Some(" Enter: review the full update".into()),
                _ => None,
            };
        };
        Some(match &l.pending {
            Some((profile, key)) => {
                let replaced = self
                    .taken_in_slot(profile, l.info.group, *key, l.info.name, l.slot)
                    .map(|(_, label)| format!(" · replaces {label}"))
                    .unwrap_or_default();
                format!(
                    " {} for \"{}\" ({}){replaced} · Enter: assign and save · Esc: keep all · another control takes its place",
                    super::controller_map::smc_printed(
                        profile,
                        &control_name(&self.live_profiles(), profile, *key)
                    ),
                    l.info.label,
                    l.slot.name(),
                )
            }
            None => format!(
                " Move a control for \"{}\" ({}). J tap · L hold · Esc cancels.",
                l.info.label,
                l.slot.name()
            ),
        })
    }

    /// The next or previous knob response (58), applied at once and saved under `[knobs]`.
    pub(super) fn step_response(&mut self, step: isize) {
        let all = input::midi::Response::ALL;
        let at = all
            .iter()
            .position(|r| *r == self.takeover.response)
            .unwrap_or(0) as isize;
        let next = all[(at + step).rem_euclid(all.len() as isize) as usize];
        self.takeover.response = next;
        let section = config::KnobsConfig {
            response: next.name().into(),
        };
        let result = match &self.user_path {
            Some(p) => config::save_section(p, "knobs", &section),
            None => Err("settings are not saved: fix the config file first".into()),
        };
        self.result_notice(result, format!("knobs: {}", next.name()));
    }

    pub(super) fn step_hold_time(&mut self, step: isize) {
        let next = (self.midi_gestures.hold_time_ms as isize + step * 50).clamp(200, 600) as u32;
        if next == self.midi_gestures.hold_time_ms {
            return;
        }
        self.midi_gestures.hold_time_ms = next;
        if let Some(midi) = &self.midi {
            midi.set_hold_time(Duration::from_millis(next.into()));
        }
        let result = match &self.user_path {
            Some(path) => config::save_section(path, "midi_gestures", &self.midi_gestures),
            None => Err("settings are not saved: fix the config file first".into()),
        };
        self.result_notice(result, format!("MIDI hold time: {next} ms"));
    }

    pub(super) fn set_smc_pad_bank(&mut self, enabled: bool) {
        if self.midi_gestures.smc_pad_bank_opens_panel == enabled {
            return;
        }
        self.midi_gestures.smc_pad_bank_opens_panel = enabled;
        if let Some(midi) = &self.midi {
            midi.set_pad_bank_shortcut(enabled);
        }
        let result = match &self.user_path {
            Some(path) => config::save_section(path, "midi_gestures", &self.midi_gestures),
            None => Err("settings are not saved: fix the config file first".into()),
        };
        self.result_notice(
            result,
            if enabled {
                "Hold PAD BANK to show the panel"
            } else {
                "PAD BANK shortcut off"
            }
            .into(),
        );
    }

    /// Opens the group of the selected row (`true`) or closes it, and selects its row.
    pub(super) fn midi_fold(&mut self, open: bool) {
        let list = self.midi_list();
        let group = match list.get(self.midi_row) {
            Some(MidiListRow::Group { group, .. }) => *group,
            Some(MidiListRow::Action { info }) => info.group,
            // the pads and the banks stand in GLOBAL: the fold covers their row
            Some(MidiListRow::Set { infos, .. }) => infos[0].group,
            Some(
                MidiListRow::Response
                | MidiListRow::HoldTime
                | MidiListRow::SmcPadBank
                | MidiListRow::SmcLayoutList(_)
                | MidiListRow::SmcLayoutApply
                | MidiListRow::SmcLayoutSlot(_)
                | MidiListRow::Reset { .. }
                | MidiListRow::Disabled { .. },
            )
            | None => return,
        };
        if open {
            if !self.midi_open.contains(&group) {
                self.midi_open.push(group);
            }
        } else {
            self.midi_open.retain(|g| *g != group);
        }
        let list = self.midi_list();
        self.midi_row = list
            .iter()
            .position(|r| matches!(r, MidiListRow::Group { group: g, .. } if *g == group))
            .unwrap_or(self.midi_row);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(name: &str) -> &'static ActionInfo {
        input::ACTIONS.iter().find(|a| a.name == name).unwrap()
    }

    fn cc(cc: u8, value: u8) -> Msg {
        Msg::Cc { ch: 1, cc, value }
    }

    #[test]
    fn knob_learn_ignores_a_touched_neighbour() {
        let t0 = Instant::now();
        let ms = |n| t0 + Duration::from_millis(n);
        let mut l = Learning::new(info("pad.volume"));
        // the neighbour sends one message first, then the wanted knob turns
        assert_eq!(l.feed("X", cc(24, 70), ms(0)), None);
        assert_eq!(
            l.feed(
                "X",
                Msg::NoteOn {
                    ch: 1,
                    note: 44,
                    velocity: 127
                },
                ms(10)
            ),
            None
        );
        for i in 0..10 {
            assert_eq!(l.feed("X", cc(31, 60 + i), ms(20 + i as u64 * 20)), None);
        }
        assert_eq!(l.poll(ms(300)), None);
        assert_eq!(l.poll(ms(600)), Some(("X".into(), Key::Cc(31))));
    }

    #[test]
    fn button_learn_takes_the_first_press() {
        let now = Instant::now();
        let mut l = Learning::new(info("bank.next"));
        assert_eq!(l.feed("X", Msg::NoteOff { ch: 1, note: 44 }, now), None);
        assert_eq!(l.feed("X", cc(64, 0), now), None);
        assert_eq!(
            l.feed(
                "X",
                Msg::NoteOn {
                    ch: 1,
                    note: 57,
                    velocity: 127
                },
                now
            ),
            Some(("X".into(), Key::Note(57)))
        );
    }

    #[test]
    fn pad_learn_takes_a_note_or_a_cc_button() {
        let now = Instant::now();
        let mut l = Learning::new(info("pad.7"));
        // value 0 is not a press
        assert_eq!(l.feed("X", cc(64, 0), now), None);
        assert_eq!(
            l.feed("X", cc(64, 40), now),
            Some(("X".into(), Key::Cc(64)))
        );
        let mut l = Learning::new(info("pad.16"));
        assert_eq!(
            l.feed(
                "X",
                Msg::NoteOn {
                    ch: 1,
                    note: 90,
                    velocity: 100
                },
                now
            ),
            Some(("X".into(), Key::Note(90)))
        );
    }

    #[test]
    fn fx_migration_preserves_precedence_direct_controls_and_empty_groups() {
        use input::Screen;
        use input::midi::{Profile, Scope};
        let mut learned = Learned::new();
        for (group, cc) in [
            (Group::Global, 70),
            (Group::Pads, 71),
            (Group::ChopEdit, 24),
            (Group::PadsEdit, 80),
        ] {
            assign(&mut learned, "X", group, Key::Cc(cc), "edit.param_1");
        }
        assign(&mut learned, "X", Group::Pads, Key::Cc(90), "pad.fx2_tone");
        assert!(migrate_fx(&mut learned, &[], &[]));
        assert_eq!(
            group_map(&learned, "X", Group::Fx),
            [
                (Key::Cc(80), "edit.param_1".into()),
                (Key::Cc(90), "pad.fx2_tone".into()),
            ]
            .into()
        );
        assert!(!migrate_fx(&mut learned, &[], &[]));
        clear(&mut learned, Group::Fx, "edit.param_1");
        clear(&mut learned, Group::Fx, "pad.fx2_tone");
        assert!(drop_dead(&mut learned).is_empty());
        assert!(learned["X"]["fx"].is_empty());
        let text = toml::to_string(&learned).unwrap();
        let mut loaded: Learned = toml::from_str(&text).unwrap();
        assert!(!migrate_fx(&mut loaded, &[], &[]));
        assert!(loaded["X"]["fx"].is_empty());
        let mut p = Profile::default_for("X");
        p.name = "X".into();
        p.set_learned(
            Scope::Edit(Screen::Pads),
            [(Key::Cc(80), "edit.param_1".into())].into(),
        );
        let mut old = Learned::new();
        assign(&mut old, "X", Group::PadsEdit, Key::Cc(80), "edit.param_1");
        assert!(migrate_fx(&mut old, &[p], &["X".into()]));
        assert_eq!(
            group_map(&old, "X", Group::Fx)
                .get(&Key::Cc(80))
                .map(String::as_str),
            Some("edit.param_1")
        );
    }

    #[test]
    fn fx_seed_uses_pad_replacements_and_explicit_fx_roles() {
        let mut learned = Learned::new();
        assign(
            &mut learned,
            "X",
            Group::PadsEdit,
            Key::Cc(80),
            "edit.param_1",
        );
        seed(
            &mut learned,
            "X",
            &[
                (Group::PadsEdit.scope(), Key::Cc(24), "edit.param_1".into()),
                (Group::ChopEdit.scope(), Key::Cc(24), "edit.param_1".into()),
                (Group::PadsEdit.scope(), Key::Cc(25), "edit.param_2".into()),
                (Group::Fx.scope(), Key::Cc(90), "edit.param_2".into()),
            ],
        );
        assert_eq!(
            group_map(&learned, "X", Group::Fx),
            [
                (Key::Cc(80), "edit.param_1".into()),
                (Key::Cc(90), "edit.param_2".into()),
            ]
            .into()
        );
        clear(&mut learned, Group::Fx, "edit.param_1");
        clear(&mut learned, Group::Fx, "edit.param_2");
        seed(
            &mut learned,
            "X",
            &[(Group::Fx.scope(), Key::Cc(90), "edit.param_2".into())],
        );
        assert!(learned["X"]["fx"].is_empty());
    }

    #[test]
    fn profile_seed_keeps_duplicate_defaults_but_not_competing_user_roles() {
        let roles = [
            (
                Group::Chop.scope(),
                Key::Note(69),
                "transport.play_stop".into(),
            ),
            (
                Group::Chop.scope(),
                Key::Note(70),
                "transport.play_stop".into(),
            ),
            (Group::ChopEdit.scope(), Key::Note(58), "ui.back".into()),
            (Group::ChopEdit.scope(), Key::Note(59), "ui.back".into()),
        ];
        for reverse in [false, true] {
            let mut roles = roles.to_vec();
            if reverse {
                roles.reverse();
            }
            let mut fresh = Learned::new();
            seed(&mut fresh, "X", &roles);
            assert_eq!(group_map(&fresh, "X", Group::Chop).len(), 2);
            assert_eq!(group_map(&fresh, "X", Group::ChopEdit).len(), 2);
            let mut existing = Learned::new();
            assign(
                &mut existing,
                "X",
                Group::Chop,
                Key::Note(99),
                "transport.play_stop",
            );
            assign(
                &mut existing,
                "X",
                Group::ChopEdit,
                Key::Note(58),
                "edit.page_next",
            );
            seed(&mut existing, "X", &roles);
            assert_eq!(
                group_map(&existing, "X", Group::Chop),
                [(Key::Note(99), "transport.play_stop".into())].into()
            );
            assert_eq!(
                group_map(&existing, "X", Group::ChopEdit)
                    .get(&Key::Note(58))
                    .map(String::as_str),
                Some("edit.page_next")
            );
            assert_eq!(
                group_map(&existing, "X", Group::ChopEdit)
                    .get(&Key::Note(59))
                    .map(String::as_str),
                Some("ui.back")
            );
        }
    }

    #[test]
    fn one_control_one_action_per_group_and_clear() {
        let mut learned = Learned::new();
        assign(&mut learned, "X", Group::Global, Key::Note(57), "bank.next");
        assign(&mut learned, "X", Group::Global, Key::Note(58), "bank.next");
        assert_eq!(
            group_map(&learned, "X", Group::Global),
            [(Key::Note(58), "bank.next".to_string())].into()
        );
        // the same button moves to another action in the same group
        assign(
            &mut learned,
            "X",
            Group::Global,
            Key::Note(58),
            "help.toggle",
        );
        assert_eq!(
            group_map(&learned, "X", Group::Global),
            [(Key::Note(58), "help.toggle".to_string())].into()
        );
        // ...but may drive something else on a screen group
        assign(
            &mut learned,
            "X",
            Group::Pads,
            Key::Note(58),
            "pad.mode_next",
        );
        assert_eq!(group_map(&learned, "X", Group::Global).len(), 1);
        assert_eq!(clear(&mut learned, Group::Global, "help.toggle"), ["X"]);
        assert!(group_map(&learned, "X", Group::Global).is_empty());
        assert_eq!(group_map(&learned, "X", Group::Pads).len(), 1);
    }

    #[test]
    fn learn_two_actions_on_one_button() {
        let mut learned = Learned::new();
        let key = Key::Note(60);
        assign_slot(
            &mut learned,
            "X",
            Group::Global,
            key,
            GestureSlot::Tap,
            "transport.toggle",
        );
        assign_slot(
            &mut learned,
            "X",
            Group::Global,
            key,
            GestureSlot::Hold,
            "metronome.toggle",
        );
        assert_eq!(
            group_bindings(&learned, "X", Group::Global).get(&key),
            Some(&Binding::Gesture {
                tap: Some("transport.toggle".into()),
                hold: "metronome.toggle".into()
            })
        );
    }

    #[test]
    fn cancel_a_pending_hold() {
        let mut learned = Learned::new();
        let key = Key::Note(60);
        assign_slot(
            &mut learned,
            "X",
            Group::Global,
            key,
            GestureSlot::Tap,
            "transport.toggle",
        );
        let before = learned.clone();
        let mut learning = Learning::new(info("metronome.toggle"));
        learning.slot = GestureSlot::Hold;
        learning.hold("X".into(), key);
        assert_eq!(learning.pending, Some(("X".into(), key)));
        drop(learning);
        assert_eq!(learned, before);
    }

    #[test]
    fn reassign_one_gesture_slot() {
        let mut learned = Learned::new();
        let key = Key::Note(60);
        assign_slot(
            &mut learned,
            "X",
            Group::Global,
            key,
            GestureSlot::Tap,
            "transport.toggle",
        );
        assign_slot(
            &mut learned,
            "X",
            Group::Global,
            key,
            GestureSlot::Hold,
            "metronome.toggle",
        );
        assign_slot(
            &mut learned,
            "X",
            Group::Global,
            key,
            GestureSlot::Hold,
            "help.toggle",
        );
        assert_eq!(
            group_bindings(&learned, "X", Group::Global).get(&key),
            Some(&Binding::Gesture {
                tap: Some("transport.toggle".into()),
                hold: "help.toggle".into()
            })
        );
        assert_eq!(
            clear(&mut learned, Group::Global, "metronome.toggle"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn clear_one_gesture_slot() {
        let mut learned = Learned::new();
        let key = Key::Note(60);
        assign_slot(
            &mut learned,
            "X",
            Group::Global,
            key,
            GestureSlot::Tap,
            "transport.toggle",
        );
        assign_slot(
            &mut learned,
            "X",
            Group::Global,
            key,
            GestureSlot::Hold,
            "metronome.toggle",
        );
        assert_eq!(
            clear(&mut learned, Group::Global, "metronome.toggle"),
            ["X"]
        );
        assert_eq!(
            group_bindings(&learned, "X", Group::Global).get(&key),
            Some(&Binding::Press("transport.toggle".into()))
        );
    }

    #[test]
    fn add_a_hold_to_an_old_binding() {
        let mut learned: Learned =
            toml::from_str("[X.global]\n'note 60'='transport.toggle'\n").unwrap();
        assign_slot(
            &mut learned,
            "X",
            Group::Global,
            Key::Note(60),
            GestureSlot::Hold,
            "metronome.toggle",
        );
        assert_eq!(
            group_bindings(&learned, "X", Group::Global).get(&Key::Note(60)),
            Some(&Binding::Gesture {
                tap: Some("transport.toggle".into()),
                hold: "metronome.toggle".into()
            })
        );
    }

    #[test]
    fn seed_a_profile_with_two_actions() {
        let mut learned = Learned::new();
        let roles = [(
            Group::Global.scope(),
            Key::Note(60),
            Binding::Gesture {
                tap: Some("transport.toggle".into()),
                hold: "metronome.toggle".into(),
            },
        )];
        seed_bindings(&mut learned, "X", &roles);
        assert_eq!(
            group_bindings(&learned, "X", Group::Global).get(&Key::Note(60)),
            Some(&roles[0].2)
        );
        let mut user = Learned::new();
        assign(
            &mut user,
            "X",
            Group::Global,
            Key::Note(61),
            "metronome.toggle",
        );
        seed_bindings(&mut user, "X", &roles);
        assert_eq!(
            group_bindings(&user, "X", Group::Global).get(&Key::Note(60)),
            None
        );
        assert_eq!(
            group_bindings(&user, "X", Group::Global).get(&Key::Note(61)),
            Some(&Binding::Press("metronome.toggle".into()))
        );
    }

    #[test]
    fn invalid_hold_keeps_a_valid_tap() {
        let mut learned = Learned::new();
        learned
            .entry("X".into())
            .or_default()
            .entry("global".into())
            .or_default()
            .insert(
                "note 60".into(),
                MidiBinding::Gesture {
                    tap: Some("transport.toggle".into()),
                    hold: "layer.panel".into(),
                },
            );
        assert_eq!(drop_dead(&mut learned), ["X global note 60 layer.panel"]);
        assert_eq!(learned["X"]["global"]["note 60"], "transport.toggle");
        learned
            .get_mut("X")
            .unwrap()
            .get_mut("global")
            .unwrap()
            .insert(
                "note 60".into(),
                MidiBinding::Gesture {
                    tap: Some("transport.toggle".into()),
                    hold: "pad.1".into(),
                },
            );
        assert_eq!(drop_dead(&mut learned), ["X global note 60 pad.1"]);
        assert_eq!(learned["X"]["global"]["note 60"], "transport.toggle");
    }

    /// Scenario "Owner unbinding keeps a second-bank cell silent" (input-devices).
    #[test]
    fn smc_none_survives_dead_binding_cleanup_only_on_new_controls() {
        let mut learned = Learned::new();
        for (device, key) in [
            ("SMC-PAD", "note 64"),
            ("SMC-PAD", "cc 27"),
            ("SMC-PAD", "cc 28"),
            ("SMC-PAD", "note 48"),
            ("X", "note 64"),
        ] {
            learned
                .entry(device.into())
                .or_default()
                .entry("global".into())
                .or_default()
                .insert(key.into(), MidiBinding::Press("none".into()));
        }
        let gone = drop_dead(&mut learned);
        assert_eq!(
            gone,
            ["SMC-PAD global note 48 none", "X global note 64 none"]
        );
        for key in ["note 64", "cc 27", "cc 28"] {
            assert_eq!(learned["SMC-PAD"]["global"][key], "none");
        }
    }

    /// Scenario "Existing SMC-PAD keeps saved roles when panel defaults arrive" (input-devices).
    #[test]
    fn existing_smc_pad_keeps_saved_roles_when_panel_defaults_arrive() {
        let mut app = crate::tests::test_app();
        app.profiles.push(
            input::midi::Profile::from_toml(
                "smc-pad.toml",
                include_str!("../../../config/devices/smc-pad.toml"),
            )
            .unwrap(),
        );
        app.seeded.push("SMC-PAD".into());
        assign(
            &mut app.learned,
            "SMC-PAD",
            input::Group::Global,
            Key::Note(66),
            "pad.1",
        );
        assign(
            &mut app.learned,
            "SMC-PAD",
            input::Group::Global,
            Key::Note(67),
            "none",
        );
        assign(
            &mut app.learned,
            "X-Session Pro",
            input::Group::Global,
            Key::Cc(25),
            "bank.next",
        );
        for device in ["SMC-PAD", "X-Session Pro"] {
            app.learned
                .get_mut(device)
                .unwrap()
                .insert("fx".into(), Default::default());
        }
        let saved = app.learned.clone();
        app.seed_device("SMC-PAD-Master", "SMC-PAD", false);
        let mut expected = saved;
        expected.get_mut("SMC-PAD").unwrap().insert(
            input::Group::Pattern.key().into(),
            [
                ("cc 30".into(), MidiBinding::Press("smc.position".into())),
                ("cc 31".into(), MidiBinding::Press("transport.tempo".into())),
                (
                    "cc 32".into(),
                    MidiBinding::Press("pattern.quantize".into()),
                ),
                (
                    "cc 33".into(),
                    MidiBinding::Press("pattern.strength".into()),
                ),
                ("cc 34".into(), MidiBinding::Press("pattern.bars".into())),
            ]
            .into(),
        );
        assert_eq!(app.learned, expected);
        let mut profile = app.profiles.remove(0);
        crate::midi::apply_learned(&mut profile, &crate::midi::parse_learned(&app.learned));
        profile.set_seeded(true);
        for (note, result) in [(66, 1), (67, 0), (60, 5)] {
            let actual = profile.resolve(
                input::Screen::Pads,
                input::midi::Msg::NoteOn {
                    ch: 1,
                    note,
                    velocity: 100,
                },
            );
            match result {
                1 => assert!(matches!(
                    actual,
                    Some(input::midi::Resolved::Action(input::Action::PadPress {
                        pad: 0,
                        ..
                    }))
                )),
                0 => assert_eq!(actual, None),
                5 => assert_eq!(actual, Some(input::midi::Resolved::PanelCell(5))),
                _ => unreachable!(),
            }
        }
    }

    /// Scenario "A1 encoder bindings survive profile seeding and owner changes" (controller-reach).
    #[test]
    fn a1_encoder_bindings_survive_profile_seeding_and_owner_changes() {
        let mut app = crate::tests::test_app();
        app.profiles.push(
            input::midi::Profile::from_toml(
                "smc-pad.toml",
                include_str!("../../../config/devices/smc-pad.toml"),
            )
            .unwrap(),
        );
        app.midi_open.push(input::Group::Pattern);
        let actions: Vec<_> = app
            .midi_list()
            .into_iter()
            .filter_map(|row| match row {
                MidiListRow::Action { info } if info.group == input::Group::Pattern => {
                    Some(info.name)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            actions,
            [
                "bank",
                "pattern.bars",
                "pattern.quantize",
                "pattern.strength"
            ]
        );
        app.seeded.push("SMC-PAD".into());
        let owner = [
            ("cc 31".into(), MidiBinding::Press("none".into())),
            (
                "cc 32".into(),
                MidiBinding::Press("pattern.strength".into()),
            ),
        ]
        .into();
        app.learned
            .entry("SMC-PAD".into())
            .or_default()
            .insert(input::Group::Pattern.key().into(), owner);
        app.learned
            .get_mut("SMC-PAD")
            .unwrap()
            .insert(input::Group::Fx.key().into(), Default::default());
        let saved = app.learned.clone();
        app.seed_device("SMC-PAD-Master", "SMC-PAD", false);
        assert_eq!(app.learned, saved);
        assert!(drop_dead(&mut app.learned).is_empty());
        assert_eq!(app.learned, saved);
        app.learned
            .get_mut("SMC-PAD")
            .unwrap()
            .insert(input::Group::Pattern.key().into(), Default::default());
        let empty = app.learned.clone();
        app.seed_device("SMC-PAD-Master", "SMC-PAD", false);
        assert_eq!(app.learned, empty);
    }

    /// Scenario "Explicit none keeps one SMC-PAD cell inactive" (input-devices).
    #[test]
    fn explicit_none_keeps_one_smc_pad_cell_inactive() {
        let mut app = crate::tests::test_app();
        app.profiles.push(
            input::midi::Profile::from_toml(
                "smc-pad.toml",
                include_str!("../../../config/devices/smc-pad.toml"),
            )
            .unwrap(),
        );
        app.profiles[0].set_seeded(true);
        app.seeded.push("SMC-PAD".into());
        assign(
            &mut app.learned,
            "SMC-PAD",
            input::Group::Global,
            Key::Note(64),
            "none",
        );
        app.sync_learned("SMC-PAD", input::Group::Global);
        let press = |note| input::midi::Msg::NoteOn {
            ch: 1,
            note,
            velocity: 100,
        };
        assert_eq!(
            app.profiles[0].resolve(input::Screen::Pads, press(64)),
            None
        );
        assert_eq!(
            app.profiles[0].resolve(input::Screen::Pads, press(65)),
            Some(input::midi::Resolved::PanelCell(2))
        );
        app.screen = input::Screen::Settings;
        app.settings_tab = 2;
        let row = app
            .midi_list()
            .iter()
            .position(|row| matches!(row, MidiListRow::Disabled { key, .. } if key == "note 64"))
            .unwrap();
        app.midi_row = row;
        assert!(
            matches!(&app.midi_screen()[row], tui::MidiRow::Action { who, .. } if who.contains("none"))
        );
        app.apply(input::Action::SettingsClear);
        assert!(!app.learned["SMC-PAD"]["global"].contains_key("note 64"));
        assert_eq!(
            app.profiles[0].resolve(input::Screen::Pads, press(64)),
            Some(input::midi::Resolved::PanelCell(1))
        );
        assert_eq!(
            app.profiles[0].resolve(input::Screen::Pads, press(65)),
            Some(input::midi::Resolved::PanelCell(2))
        );
    }
}

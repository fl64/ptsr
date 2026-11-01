//! MIDI learn: waits for a control and keeps learned assignments in the `[midi]` config shape.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use input::midi::{Key, Msg};
use input::{ActionInfo, Group};

use crate::config::Learned;

/// A knob is decided after this much time since its first message.
const KNOB_WINDOW: Duration = Duration::from_millis(500);

pub struct Learning {
    pub info: &'static ActionInfo,
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
    for (device, groups) in learned.iter_mut() {
        for (group, keys) in groups.iter_mut() {
            keys.retain(|key, name| {
                let known = input::midi::known(name);
                if !known {
                    gone.push(format!("{device} {group} {key} {name}"));
                }
                known
            });
        }
        groups.retain(|_, keys| !keys.is_empty());
    }
    learned.retain(|_, groups| !groups.is_empty());
    gone
}

pub fn assign(learned: &mut Learned, profile: &str, group: Group, key: Key, action: &str) {
    let map = learned
        .entry(profile.into())
        .or_default()
        .entry(group.key().into())
        .or_default();
    map.retain(|_, a| a != action);
    map.insert(key.to_string(), action.into());
}

/// The group of a scope (59).
pub fn group_of(scope: input::midi::Scope) -> Option<Group> {
    Group::ALL.into_iter().find(|g| g.scope() == scope)
}

/// Copies the profile roles into `profile`'s bindings (59): a binding already there wins,
/// on the same control or the same action of its group.
pub fn seed(learned: &mut Learned, profile: &str, roles: &[(input::midi::Scope, Key, String)]) {
    let groups = learned.entry(profile.into()).or_default();
    for (scope, key, action) in roles {
        let Some(group) = group_of(*scope) else {
            continue;
        };
        let map = groups.entry(group.key().into()).or_default();
        if map.contains_key(&key.to_string()) || map.values().any(|a| a == action) {
            continue;
        }
        map.insert(key.to_string(), action.clone());
    }
}

/// Removes the learned controls of `action` in `group`; returns the profiles that changed.
pub fn clear(learned: &mut Learned, group: Group, action: &str) -> Vec<String> {
    let mut changed = Vec::new();
    for (profile, groups) in learned.iter_mut() {
        if let Some(map) = groups.get_mut(group.key()) {
            let before = map.len();
            map.retain(|_, a| a != action);
            if map.len() != before {
                changed.push(profile.clone());
            }
        }
    }
    changed
}

/// Learned controls of a profile in a group, parsed.
pub fn group_map(learned: &Learned, profile: &str, group: Group) -> HashMap<Key, String> {
    learned
        .get(profile)
        .and_then(|s| s.get(group.key()))
        .into_iter()
        .flatten()
        .filter_map(|(k, a)| Some((k.parse().ok()?, a.clone())))
        .collect()
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
}

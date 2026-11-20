//! Review saved SMC-PAD roles before copying the new layout to them.

use super::{App, Ask, Group, Key, Learned, Level, MidiBinding, config, midi};
use std::time::Instant;

const DEVICE: &str = "SMC-PAD";

// Known shipped roles of each changed slot: the first complete map, then change 124.
// None means the slot had no saved role.
const PREVIOUS: &[(Group, u8, &[Option<&str>])] = &[
    (Group::Global, 30, &[Some("projects.scroll")]),
    (Group::Global, 35, &[Some("edit.page")]),
    (Group::Global, 36, &[Some("transport.tempo")]),
    (Group::Global, 37, &[Some("bank")]),
    (Group::Global, 38, &[Some("pad.volume")]),
    (Group::Global, 39, &[None, Some("pad.cutoff")]),
    (Group::Global, 40, &[Some("pad.pitch")]),
    (Group::Global, 41, &[Some("pad.cutoff"), Some("pad.decay")]),
    (Group::Global, 42, &[Some("pad.decay"), Some("pad.send_1")]),
    (Group::Global, 43, &[None, Some("pad.send_2")]),
    (Group::Pads, 25, &[Some("pad.select_left")]),
    (Group::Pads, 26, &[Some("pad.select_right")]),
    (Group::Pads, 30, &[Some("bank")]),
    (Group::Pads, 31, &[Some("pad.volume")]),
    (Group::Pads, 32, &[Some("pad.pitch")]),
    (Group::Pads, 33, &[Some("pad.cutoff")]),
    (Group::Pads, 34, &[Some("pad.decay")]),
    (Group::Pads, 35, &[None]),
    (Group::Pads, 36, &[None]),
    (Group::Files, 25, &[Some("browser.up")]),
    (Group::Files, 26, &[Some("browser.down")]),
    (Group::Chop, 25, &[Some("marker.prev")]),
    (Group::Chop, 26, &[Some("marker.next")]),
    (Group::Chop, 32, &[Some("chop.threshold")]),
    (Group::Chop, 33, &[Some("slice.start")]),
    (Group::Chop, 34, &[Some("slice.end")]),
    (Group::Chop, 35, &[None]),
    (Group::Chop, 36, &[None]),
    (Group::Pattern, 30, &[Some("bank")]),
    (Group::Pattern, 31, &[Some("pattern.bars")]),
    (Group::Pattern, 34, &[None]),
    (Group::Fx, 25, &[None]),
    (Group::Fx, 26, &[None]),
    (Group::Fx, 31, &[None]),
    (Group::Fx, 32, &[None]),
    (Group::Fx, 33, &[None]),
    (Group::Fx, 35, &[Some("edit.page")]),
    (Group::Settings, 25, &[Some("settings.row_prev")]),
    (Group::Settings, 26, &[Some("settings.row_next")]),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Slot {
    group: Group,
    cc: u8,
    control: String,
    before: Option<MidiBinding>,
    after: Option<MidiBinding>,
}

impl Slot {
    fn key(&self) -> String {
        Key::Cc(self.cc).to_string()
    }

    pub(crate) fn label(&self) -> String {
        format!("{} {} ({})", self.group.title(), self.control, self.key())
    }

    pub(super) fn summary(&self) -> String {
        let role = |binding: &Option<MidiBinding>| {
            match binding.as_ref().map(MidiBinding::as_str) {
                Some("smc.position") => "position",
                Some("smc.list_prev") => "previous item",
                Some("smc.list_next") => "next item",
                Some(name) if name.starts_with("smc.page_") => "page value",
                Some(name) => input::ACTIONS
                    .iter()
                    .find(|info| info.name == name)
                    .map_or(name, |info| info.label),
                None => "unassigned",
            }
            .to_string()
        };
        format!("{} → {}", role(&self.before), role(&self.after))
    }
}

pub(crate) struct Review {
    pub(crate) lines: Vec<String>,
    slots: Vec<Slot>,
}

fn binding(learned: &Learned, group: Group, key: &str) -> Option<MidiBinding> {
    learned.get(DEVICE)?.get(group.key())?.get(key).cloned()
}

fn matches_old(current: Option<&MidiBinding>, old: &[Option<&str>]) -> bool {
    old.iter().any(|old| match (current, old) {
        (Some(MidiBinding::Press(name)), Some(old)) => name == old,
        (None, None) => true,
        _ => false,
    })
}

impl App {
    pub(super) fn smc_layout_slots(&self) -> Vec<Slot> {
        if !self.seeded.iter().any(|name| name == DEVICE)
            || !self.midi_devices.iter().any(|(port, name)| {
                name == DEVICE && port.contains("SMC-PAD-Master") && !port.contains("(virtual)")
            })
            || !PREVIOUS.iter().any(|&(group, cc, old)| {
                let current = binding(&self.learned, group, &Key::Cc(cc).to_string());
                current.is_some() && matches_old(current.as_ref(), old)
            })
        {
            return Vec::new();
        }
        let Some(profile) = self.profiles.iter().find(|profile| profile.name == DEVICE) else {
            return Vec::new();
        };
        let roles = profile.binding_roles();
        PREVIOUS
            .iter()
            .filter_map(|&(group, cc, old)| {
                let key = Key::Cc(cc);
                let control = profile.name_of(key)?.to_string();
                let key_text = key.to_string();
                let controls = self.learned.get(DEVICE)?.get(group.key())?;
                let before = controls.get(&key_text).cloned();
                if controls.is_empty() || !matches_old(before.as_ref(), old) {
                    return None;
                }
                if before.is_none() && group != Group::Global {
                    let global = binding(&self.learned, Group::Global, &key_text);
                    let old_global = PREVIOUS.iter().find_map(|&(scope, number, name)| {
                        (scope == Group::Global && number == cc).then_some(name)
                    });
                    if old_global.is_some_and(|expected| !matches_old(global.as_ref(), expected)) {
                        return None;
                    }
                }
                let after = roles.iter().find_map(|(scope, target, role)| {
                    (*scope == group.scope() && *target == key).then(|| role.clone().into())
                });
                (before != after).then_some(Slot {
                    group,
                    cc,
                    control,
                    before,
                    after,
                })
            })
            .collect()
    }

    pub(crate) fn ask_smc_layout(&mut self, slot: Option<&Slot>) {
        let mut slots = self.smc_layout_slots();
        if let Some(slot) = slot {
            // A new screen role can depend on the old global role for this knob.
            slots.retain(|candidate| {
                candidate == slot
                    || (slot.group == Group::Global
                        && slot.after.is_none()
                        && candidate.cc == slot.cc
                        && candidate.before.is_none())
            });
        }
        if slots.is_empty() {
            return;
        }
        let mut lines = vec!["Update SMC-PAD layout?".into()];
        if slot.is_some() {
            lines.extend(
                slots
                    .iter()
                    .map(|slot| format!("{} · {}", slot.label(), slot.summary())),
            );
        } else {
            lines.push(format!(
                "Apply all {} changes from the review?",
                slots.len()
            ));
        }
        lines.push("This cannot be undone.".into());
        lines.push("Enter: apply    Esc: keep".into());
        self.ask = Some(Ask::SmcLayout(Review { lines, slots }));
    }

    pub(crate) fn apply_smc_layout(&mut self, review: Review) {
        let result = self.save_smc_layout(&review.slots);
        let (level, message) = match result {
            Ok(()) => (Level::Done, "SMC-PAD layout updated.".into()),
            Err(error) => (Level::Error, format!("SMC-PAD layout kept. {error}")),
        };
        self.post_notice(level, (message, Instant::now()));
        self.midi_row = self.midi_row.min(self.midi_list().len().saturating_sub(1));
    }

    fn save_smc_layout(&mut self, slots: &[Slot]) -> Result<(), String> {
        let current = self.smc_layout_slots();
        if slots.iter().any(|slot| !current.contains(slot)) {
            return Err("Open the review again.".into());
        }
        let path = self
            .user_path
            .as_ref()
            .ok_or("The config file is unavailable.")?;
        let mut updated = self.learned.clone();
        let groups = updated
            .get_mut(DEVICE)
            .ok_or("The saved map is unavailable.")?;
        for slot in slots {
            let controls = groups
                .get_mut(slot.group.key())
                .ok_or("The saved group is unavailable.")?;
            if let Some(after) = &slot.after {
                controls.insert(slot.key(), after.clone());
            } else {
                controls.remove(&slot.key());
            }
        }
        config::save_section(path, "midi", &updated)?;
        self.learned = updated;
        let maps = Group::ALL
            .into_iter()
            .filter(|group| slots.iter().any(|slot| slot.group == *group))
            .map(|group| {
                let bindings = super::group_bindings(&self.learned, DEVICE, group);
                ((DEVICE.into(), group.scope()), bindings)
            })
            .collect();
        for profile in self
            .profiles
            .iter_mut()
            .filter(|profile| profile.name == DEVICE)
        {
            midi::apply_learned(profile, &maps);
        }
        if let Some(midi) = &self.midi {
            midi.set_learned_groups(maps);
        }
        Ok(())
    }
}

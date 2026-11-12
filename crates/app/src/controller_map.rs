//! The F1 Controller tab (98j): the top of one connected device with the action of each
//! control on the screen under F1.

use super::{App, Keymap};
use input::Action;
use input::midi::{ControlKind, Profile};

#[derive(Default)]
pub(super) struct State {
    /// The shown device; `None` until the tab picks one.
    device: Option<String>,
    /// Its place on the device line, for the next device when it disconnects.
    index: usize,
    /// The selected control by name.
    selected: Option<String>,
    scroll: usize,
    /// The port of the last MIDI message: the tab starts on its device.
    pub last_port: Option<String>,
}

/// The full help text of an action; knob names outside the learn list have none.
fn label(action: &str) -> &'static str {
    input::ACTIONS
        .iter()
        .find(|i| i.name == action)
        .map_or("", |i| i.label)
}

/// One device: the ports of one ALSA client with one profile.
pub(super) struct Device {
    pub id: String,
    pub name: String,
    pub profile: Profile,
}

/// `USB X-Session:USB X-Session MIDI 1 20:0` -> `20`: the ALSA client of the port.
fn client(port: &str) -> Option<&str> {
    let (client, port_number) = port.rsplit_once(' ')?.1.split_once(':')?;
    (!client.is_empty()
        && client.bytes().all(|b| b.is_ascii_digit())
        && port_number.bytes().all(|b| b.is_ascii_digit()))
    .then_some(client)
}

fn device_id(port: &str, profile: &str) -> String {
    match client(port) {
        Some(client) => format!("{client}/{profile}"),
        None => format!("{port}/{profile}"),
    }
}

fn brackets(kind: ControlKind) -> (char, char) {
    match kind {
        ControlKind::Knob => ('(', ')'),
        ControlKind::Button => ('[', ']'),
        ControlKind::Pad => ('|', '|'),
        ControlKind::Fader => ('=', '='),
        ControlKind::Crossfader => ('<', '>'),
    }
}

/// The places of a drawing with a control, in reading order.
fn places(rows: &[input::midi::LayoutRow]) -> Vec<(usize, usize, &str)> {
    rows.iter()
        .enumerate()
        .flat_map(|(r, row)| {
            row.iter()
                .enumerate()
                .filter_map(move |(c, place)| place.as_deref().map(|name| (r, c, name)))
        })
        .collect()
}

/// The nearest control from `(r, c)` in a direction; `None` at the edge.
fn step(rows: &[input::midi::LayoutRow], (r, c): (usize, usize), dr: i8, dc: i8) -> Option<&str> {
    let all = places(rows);
    if dc != 0 {
        return all
            .iter()
            .filter(|(pr, pc, _)| *pr == r && if dc > 0 { *pc > c } else { *pc < c })
            .min_by_key(|(_, pc, _)| pc.abs_diff(c))
            .map(|(_, _, name)| *name);
    }
    let rows_on = |pr: usize| if dr > 0 { pr > r } else { pr < r };
    let row = all
        .iter()
        .filter(|(pr, _, _)| rows_on(*pr))
        .min_by_key(|(pr, _, _)| pr.abs_diff(r))?
        .0;
    all.iter()
        .filter(|(pr, _, _)| *pr == row)
        .min_by_key(|(_, pc, _)| pc.abs_diff(c))
        .map(|(_, _, name)| *name)
}

impl App {
    /// Connected devices in the order of connection, with the learned bindings over the
    /// profile, as the MIDI dispatch has them.
    pub(super) fn controller_devices(&self) -> Vec<Device> {
        let maps = super::midi::parse_learned(&self.learned);
        let mut out: Vec<Device> = Vec::new();
        for (port, name) in &self.midi_devices {
            let id = device_id(port, name);
            if out.iter().any(|d| d.id == id) {
                continue;
            }
            let mut profile = self
                .profiles
                .iter()
                .find(|p| p.name == *name)
                .cloned()
                .unwrap_or_else(|| Profile::default_for(port));
            super::midi::apply_learned(&mut profile, &maps);
            profile.set_seeded(self.seeded.contains(&profile.name));
            out.push(Device {
                id,
                name: name.clone(),
                profile,
            });
        }
        // two devices of one profile get numbers in the order of connection
        let names: Vec<String> = out.iter().map(|d| d.name.clone()).collect();
        for (i, device) in out.iter_mut().enumerate() {
            if names.iter().filter(|n| **n == names[i]).count() > 1 {
                let number = names[..=i].iter().filter(|n| **n == names[i]).count();
                device.name = format!("{} {number}", names[i]);
            }
        }
        out
    }

    /// The tab is selected: show the device of the last MIDI message.
    pub(super) fn controller_start(&mut self) {
        let devices = self.controller_devices();
        let state = &mut self.help_overlay.controller;
        let last = state.last_port.as_ref().and_then(|port| {
            self.midi_devices
                .iter()
                .find(|(p, _)| p == port)
                .map(|(p, name)| device_id(p, name))
        });
        let index = last
            .and_then(|id| devices.iter().position(|d| d.id == id))
            .unwrap_or(0);
        state.device = devices.get(index).map(|d| d.id.clone());
        state.index = index;
        state.selected = None;
        state.scroll = 0;
    }

    /// The shown device; a device that went away gives its place to the next one.
    fn controller_current(&mut self, devices: &[Device]) -> Option<usize> {
        let state = &mut self.help_overlay.controller;
        let found = state
            .device
            .as_ref()
            .and_then(|id| devices.iter().position(|d| d.id == *id));
        let index = match found {
            Some(index) => index,
            None if devices.is_empty() => {
                state.device = None;
                return None;
            }
            None => {
                state.selected = None;
                state.scroll = 0;
                state.index.min(devices.len() - 1)
            }
        };
        state.index = index;
        state.device = Some(devices[index].id.clone());
        Some(index)
    }

    /// The help page actions change the device on this tab.
    pub(super) fn controller_page(&mut self, next: bool) {
        let devices = self.controller_devices();
        let Some(index) = self.controller_current(&devices) else {
            return;
        };
        let len = devices.len();
        let index = if next {
            (index + 1) % len
        } else {
            (index + len - 1) % len
        };
        let state = &mut self.help_overlay.controller;
        state.device = Some(devices[index].id.clone());
        state.index = index;
        state.selected = None;
        state.scroll = 0;
    }

    fn controller_drawing<'a>(&self, profile: &'a Profile) -> Option<&'a [input::midi::LayoutRow]> {
        let rows = profile.layout()?;
        let width = rows.iter().map(Vec::len).max().unwrap_or(0);
        tui::controller_fits(
            width,
            rows.len(),
            self.help_overlay.size,
            self.help_overlay.caption.is_some(),
        )
        .then_some(rows)
    }

    /// The arrows on this tab: the nearest control in their direction.
    pub(super) fn controller_select(&mut self, dr: i8, dc: i8) {
        let devices = self.controller_devices();
        let Some(index) = self.controller_current(&devices) else {
            return;
        };
        let profile = &devices[index].profile;
        let selected = self.help_overlay.controller.selected.clone();
        if let Some(rows) = self.controller_drawing(profile) {
            let all = places(rows);
            let Some(&(_, _, first)) = all.first() else {
                return;
            };
            let at = selected
                .and_then(|s| all.iter().find(|(_, _, n)| *n == s))
                .map_or((all[0].0, all[0].1), |(r, c, _)| (*r, *c));
            if let Some(name) = step(rows, at, dr, dc).or(
                // the first arrow only shows where the selection is
                self.help_overlay
                    .controller
                    .selected
                    .is_none()
                    .then_some(first),
            ) {
                self.help_overlay.controller.selected = Some(name.to_string());
            }
            return;
        }
        if dr == 0 {
            return;
        }
        let names: Vec<&str> = profile.controls().iter().map(|(n, _)| n.as_str()).collect();
        let Some(last) = names.len().checked_sub(1) else {
            return;
        };
        let at = selected.and_then(|s| names.iter().position(|n| *n == s));
        let at = match at.unwrap_or(0) {
            at if dr > 0 => (at + 1).min(last),
            at => at.saturating_sub(1),
        };
        self.help_overlay.controller.selected = Some(names[at].to_string());
        let rows = tui::controller_list_rows(
            self.help_overlay.size.1,
            self.help_overlay.caption.is_some(),
        )
        .max(1);
        let scroll = &mut self.help_overlay.controller.scroll;
        *scroll = (*scroll).min(at).max((at + 1).saturating_sub(rows));
    }

    /// The tab as `tui` draws it.
    pub(super) fn controller_view(&mut self, keymap: &Keymap) -> tui::Controller {
        let key = |app: &App, action| {
            app.help_bindings(keymap, action)
                .into_iter()
                .next()
                .unwrap_or_else(|| "none".into())
        };
        let navigation = format!(
            "{} / {} device · arrows select",
            key(self, Action::HelpPagePrev),
            key(self, Action::HelpPageNext)
        );
        let devices = self.controller_devices();
        let caption = self.help_overlay.caption.clone();
        let Some(index) = self.controller_current(&devices) else {
            let mut profiles: Vec<String> = self.profiles.iter().map(|p| p.name.clone()).collect();
            profiles.sort();
            profiles.dedup();
            return tui::Controller {
                devices: Vec::new(),
                current: 0,
                body: tui::ControllerBody::Empty { profiles },
                detail: String::new(),
                navigation,
                caption,
            };
        };
        let profile = &devices[index].profile;
        let edit = self.edit_keys();
        let action = |name: &str| profile.control_action(self.screen, edit, name);
        let first = || match self.controller_drawing(profile) {
            Some(rows) => places(rows).first().map(|(_, _, n)| n.to_string()),
            None => profile.controls().first().map(|(n, _)| n.clone()),
        };
        let selected = self
            .help_overlay
            .controller
            .selected
            .clone()
            .filter(|s| profile.kind_of(s).is_some())
            .or_else(first);
        let detail = match &selected {
            Some(name) => {
                let kind = profile.kind_of(name).map_or("", |k| k.name());
                match action(name) {
                    Some(a) => format!("{name} · {kind} · {a} · {}", label(a)),
                    None => format!("{name} · {kind} · free"),
                }
            }
            None => String::new(),
        };
        let body = match self.controller_drawing(profile) {
            Some(rows) => {
                let mut at = (usize::MAX, usize::MAX);
                let rows = rows
                    .iter()
                    .enumerate()
                    .map(|(r, row)| {
                        row.iter()
                            .enumerate()
                            .map(|(c, place)| {
                                let name = place.as_deref()?;
                                if selected.as_deref() == Some(name) {
                                    at = (r, c);
                                }
                                let (open, close) =
                                    brackets(profile.kind_of(name).unwrap_or(ControlKind::Button));
                                let short = action(name).map_or("-", input::short_label);
                                Some(tui::ControlCell {
                                    name: name.to_string(),
                                    label: format!("{open}{short}{close}"),
                                    free: action(name).is_none(),
                                })
                            })
                            .collect()
                    })
                    .collect();
                tui::ControllerBody::Drawing { rows, selected: at }
            }
            None => {
                let rows: Vec<(String, String)> = profile
                    .controls()
                    .iter()
                    .map(|(name, kind)| {
                        let what =
                            action(name).map_or("-".to_string(), |a| format!("{a} · {}", label(a)));
                        (format!("{name} {}", kind.name()), what)
                    })
                    .collect();
                let selected = selected
                    .and_then(|s| profile.controls().iter().position(|(n, _)| *n == s))
                    .unwrap_or(0);
                tui::ControllerBody::List {
                    rows,
                    selected,
                    scroll: self.help_overlay.controller.scroll,
                }
            }
        };
        tui::Controller {
            devices: devices.iter().map(|d| d.name.clone()).collect(),
            current: index,
            body,
            detail,
            navigation,
            caption,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_of_one_client_are_one_device() {
        assert_eq!(
            client("USB X-Session:USB X-Session MIDI 1 20:0"),
            Some("20")
        );
        assert_eq!(
            client("USB X-Session:USB X-Session MIDI 2 20:1"),
            Some("20")
        );
        assert_eq!(client("X"), None);
        assert_eq!(
            device_id("USB X-Session:USB X-Session MIDI 2 20:1", "X-Session Pro"),
            device_id("USB X-Session:USB X-Session MIDI 1 20:0", "X-Session Pro")
        );
    }
}

//! The F1 Controller tab (98j): the top of one connected device with the action of each
//! control on the screen under F1.

use super::{App, Keymap, Level, input_sources, midi};
use input::Action;
use input::midi::{ControlKind, Profile};

pub(super) const SHIFT_NOTICE: &str = "SMC-PAD pads are shifted (SHIFT + PAD13–16 changes octave or transpose). Reset it on the device.";

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
    /// The latest USB setup result for the measured SMC-PAD profile.
    pub smc_status: Option<(bool, String)>,
    pub smc_layout_open: bool,
    pub smc_shift_warned: std::collections::HashMap<String, input_sources::Connection>,
    pub smc_last_knob: Option<SmcKnobInput>,
}

/// The last valid SMC-PAD knob step on one connection.
pub struct SmcKnobInput {
    pub port: String,
    pub bank: usize,
    /// Physical knob 1–8 as index 0–7.
    pub knob: usize,
    pub at: std::time::Instant,
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
    /// The owner's own bindings on SMC-PAD panel cells (134): a learned action replaces
    /// the shipped cell, so the panel names that action instead of the old label.
    pub(super) fn smc_cell_overrides(&self) -> [Option<String>; 16] {
        let device = self
            .controller_devices()
            .into_iter()
            .find(|device| device.profile.name == "SMC-PAD");
        std::array::from_fn(|index| {
            let profile = &device.as_ref()?.profile;
            let action = profile.control_action_context(
                self.screen,
                self.edit_keys(),
                self.pattern_panel.is_some(),
                &format!("c{}", index + 1),
            )?;
            (action != "panel.cell").then(|| input::short_label(action).to_string())
        })
    }

    /// The SMC-PAD panel cell that `key` runs now, by its printed number and its label (135):
    /// Learn names it before a binding takes the cell off the panel.
    pub(super) fn panel_cell_of(&self, profile: &str, key: input::midi::Key) -> Option<String> {
        let device = self
            .controller_devices()
            .into_iter()
            .find(|device| device.profile.name == profile && profile == "SMC-PAD")?;
        let name = device.profile.name_of(key)?.to_string();
        let cell: u8 = name.strip_prefix('c')?.parse().ok()?;
        let action = device.profile.control_action_context(
            self.screen,
            self.edit_keys(),
            self.pattern_panel.is_some(),
            &name,
        )?;
        if action != "panel.cell" {
            return None;
        }
        let label = self.panel.label(self.panel_view(), cell).unwrap_or("empty");
        Some(format!(
            "panel cell P{} {label}",
            tui::pads::device_number(usize::from(cell))
        ))
    }

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

    pub(super) fn observe_smc_knob(&mut self, event: &super::midi::UiEvent) {
        let super::midi::Event::Message {
            port,
            profile,
            msg:
                input::midi::Msg::Cc {
                    ch: 1,
                    cc,
                    value: 63 | 65,
                },
            ..
        } = &event.event
        else {
            return;
        };
        let Some(stamp) = event.source() else {
            return;
        };
        let state = &mut self.help_overlay.controller;
        if profile == "SMC-PAD"
            && (30..=45).contains(cc)
            && state.smc_status.as_ref().is_some_and(|(ready, _)| *ready)
            && self
                .input_sources
                .connected()
                .any(|(name, id)| name == port && id == stamp.connection)
            && state
                .smc_last_knob
                .as_ref()
                .is_none_or(|last| stamp.at >= last.at)
        {
            state.smc_last_knob = Some(SmcKnobInput {
                port: port.clone(),
                bank: usize::from(*cc >= 38),
                knob: usize::from((cc - 30) % 8),
                at: stamp.at,
            });
        }
    }

    pub(super) fn observe_smc_pad(&mut self, ev: &midi::UiEvent) {
        let midi::Event::Message {
            port,
            profile,
            msg,
            meaning,
        } = &ev.event
        else {
            return;
        };
        if profile != "SMC-PAD" || !port.contains("SMC-PAD-Master") || meaning != "not assigned" {
            return;
        }
        let input::midi::Msg::NoteOn { ch: 10, note, .. } = *msg else {
            return;
        };
        let Some(stamp) = ev.source() else {
            return;
        };
        if (36..=67).contains(&note)
            || !self
                .input_sources
                .connected()
                .any(|(name, connection)| name == port && connection == stamp.connection)
        {
            return;
        }
        let warned = &mut self.help_overlay.controller.smc_shift_warned;
        if warned.insert(port.clone(), stamp.connection) != Some(stamp.connection) {
            self.post_system_notice(
                Level::Warn,
                (SHIFT_NOTICE.into(), std::time::Instant::now()),
            );
        }
    }

    fn smc_context(&self) -> (&'static str, &'static str, &'static str) {
        use super::panel::View;
        if self.project_picker.is_some() {
            ("Projects", "Project", "select")
        } else if self.path_picker.is_some() {
            ("Paths", "Folder", "folder")
        } else if self.pattern_panel.is_some() {
            ("Transport", "Page", "param")
        } else {
            match self.panel_view() {
                View::Pads => (self.pads.page().0, "Bank", ""),
                View::PadEdit => (self.pads.page().0, "Page", "param"),
                View::Pattern => ("Pattern", "Bar", "step"),
                View::Chop | View::ChopEdit => ("Chop", "Bank", "cursor"),
                View::Files => ("Files", "Folder", "file"),
                View::Settings => ("Settings", "Tab", "row"),
                View::Fx => (["FX 1", "FX 2"][self.fx_slot], "Slot", "param"),
            }
        }
    }

    fn smc_knob_role(&self, name: &str, action: Option<&str>) -> (&'static str, bool) {
        match action {
            Some("smc.position") => match self.smc_context().2 {
                "" => ("-", false),
                "param" if self.screen == input::Screen::Pads && self.edit => {
                    let (label, param) = self.smc_page_role(1).unwrap_or(("-", None));
                    (label, param.is_some())
                }
                position => (position, true),
            },
            Some("master.volume") => ("master", true),
            Some("settings.row") => ("row", true),
            Some("settings.value_next") => ("value", true),
            Some("files.scroll") => ("file", true),
            Some("projects.scroll") => ("select", true),
            Some(action) => {
                let (label, unavailable) = self
                    .smc_value_label(action)
                    .unwrap_or_else(|| (input::short_label(action), false));
                (label, !unavailable)
            }
            None if name == "e6" && self.pattern_panel.is_some() => ("swing", false),
            None => ("-", false),
        }
    }

    pub(super) fn smc_knob_detail(&self, name: &str, action: Option<&str>) -> Option<String> {
        use input::midi::{PARAMS, Param, param_of};
        let bank = if name.starts_with('e') { 1 } else { 2 };
        let knob = name.get(1..)?.parse::<u8>().ok()?;
        let (short, available) = self.smc_knob_role(name, action);
        let purpose = match action {
            Some("smc.position") => match self.smc_context().2 {
                "" => short,
                "param" if self.screen == input::Screen::Pads && self.edit => short,
                "step" => "select a step in the row",
                "select" => "select a project",
                "param" => "select a parameter",
                position => position,
            },
            Some("chop.region_start_pos") => "region start",
            Some("chop.region_end_pos") => "region end",
            Some("chop.threshold") => "hit level",
            Some("pad.send_1") => "send to FX 1",
            Some("pad.send_2") => "send to FX 2",
            Some(action) => {
                let param = match param_of(action) {
                    Some(Param::SmcPageValue(knob)) => self.smc_page_value(knob),
                    Some(Param::Slot(slot)) => self.slot_param(slot),
                    param => param,
                };
                let effective = param
                    .and_then(|param| PARAMS.iter().find(|(_, value)| *value == param))
                    .map_or(action, |(name, _)| *name);
                let text = label(effective);
                if text.is_empty() {
                    short
                } else {
                    text.strip_suffix(" (knob)").unwrap_or(text)
                }
            }
            None if short == "-" => "unassigned",
            None => short,
        };
        let state = if available || short == "-" {
            ""
        } else {
            " · unavailable"
        };
        Some(format!("B{bank} K{knob} · {purpose}{state}"))
    }

    /// The mode line (134): the active mode, its item, the next step and the cancel
    /// control, in the words of the last input device.
    pub(super) fn mode_line(&self, now: std::time::Instant) -> Option<String> {
        let smc = self.input_sources.last_is_smc();
        let (cancel, apply) = if smc {
            ("PAD BANK + 1 cancel", "PAD BANK + 4 apply")
        } else {
            ("Esc cancel", "Enter apply")
        };
        if self.ask.is_some() {
            return Some(if smc {
                "ASK · pad 4 yes · pad 1 no".into()
            } else {
                "ASK · Enter yes · Esc no".into()
            });
        }
        if self.screen_learn.is_some() || self.learning.is_some() {
            return Some(if smc {
                "LEARN · touch a control · pad 4 save · pad 1 cancel".into()
            } else {
                "LEARN · touch a control · Esc leave".into()
            });
        }
        if let Some((source, at)) = self.recording {
            let seconds = now.saturating_duration_since(at).as_secs();
            return Some(if source == super::audio::SOURCE_OUTPUT {
                let stop = if smc {
                    "PAD BANK + 6 output"
                } else {
                    "the output key"
                };
                format!("● REC OUTPUT 0:{seconds:02} · stop: {stop}")
            } else {
                let stop = if smc {
                    "release RECORD"
                } else {
                    "release the key"
                };
                format!("● REC MIC 0:{seconds:02} · stop: {stop}")
            });
        }
        if self.screen == input::Screen::Chop
            && let (Some(plan), Some(chop)) = (&self.plan, &self.chop)
        {
            let count = chop.slices().len();
            let placed = plan.targets(count).len();
            let next = match plan {
                super::chop::Plan::InOrder(None) => "hit the start pad".to_string(),
                _ => {
                    let selected = chop.selected;
                    let next = if plan
                        .targets(count)
                        .iter()
                        .any(|(slice, _)| *slice == selected)
                    {
                        plan.next_unplanned(count, selected)
                    } else {
                        Some(selected)
                    };
                    next.map_or("all placed".into(), |slice| {
                        format!("slice {} next · hit a pad", slice + 1)
                    })
                }
            };
            return Some(format!(
                "ASSIGN {placed}/{count} · {next} · {apply} · {cancel}"
            ));
        }
        let place = |what: String| Some(format!("{what} · hit a pad · {cancel}"));
        if self.erase_pads {
            let stop = if smc { "PAD BANK + 1 stop" } else { "Esc stop" };
            return Some(format!("ERASE A1 · hit a pad to erase its hits · {stop}"));
        }
        if let Some(path) = &self.browser.in_hand {
            return place(format!("IN HAND {}", super::file_name(path)));
        }
        if let Some((source, _)) = &self.copy_in_hand {
            return place(format!("COPY pad {}", tui::pads::pad_tag(*source)));
        }
        if let Some(source) = self.pad_in_hand {
            return place(format!("SWAP pad {}", tui::pads::pad_tag(source)));
        }
        if self.edit && self.screen == input::Screen::Pads {
            let (page, _) = self.pads.page();
            let (pages, close) = if smc {
                ("LEFT RIGHT page", "PAD BANK + 1 close")
            } else {
                ("Tab page", "Esc close")
            };
            return Some(format!(
                "EDIT pad {} · {page} · {pages} · {close}",
                tui::pads::pad_tag(self.pads.selected)
            ));
        }
        if self.pads.marking {
            let drop = if smc { "PAD BANK + 1 drop" } else { "Esc drop" };
            let marked = self.pads.marked.len();
            return Some(format!("MARK · hit pads · {marked} marked · {drop}"));
        }
        None
    }

    pub(super) fn smc_knob_strip(&self, now: std::time::Instant) -> Option<tui::KnobStrip> {
        if self.screen_learn.is_some() || self.learning.is_some() || self.midi_run.is_some() {
            return None;
        }
        let connected = self
            .midi_devices
            .iter()
            .any(|(_, profile)| profile == "SMC-PAD");
        let profile = self
            .profiles
            .iter()
            .find(|profile| profile.name == "SMC-PAD")
            .filter(|_| connected)?;
        let edit = self.edit_keys();
        let action = |name| {
            profile.control_action_context(self.screen, edit, self.pattern_panel.is_some(), name)
        };
        let (page, list, _) = self.smc_context();
        // A visible panel takes LEFT and RIGHT for its views (127); the hint says so (134).
        let (page, list) = if self.panel.visible()
            && self.panel_input_available()
            && self.ask.is_none()
            && self.panel_snapshot().is_some_and(|panel| !panel.target)
        {
            ("PANEL", "View")
        } else {
            (page, list)
        };
        let ready = self
            .help_overlay
            .controller
            .smc_status
            .as_ref()
            .is_some_and(|(ready, _)| *ready);
        let names = [
            ["e1", "e2", "e3", "e4", "e5", "e6", "e7", "e8"],
            ["f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8"],
        ];
        let roles = names.map(|bank| {
            bank.map(|name| {
                let (label, available) = self.smc_knob_role(name, action(name));
                (label, available && ready)
            })
        });
        let last = self
            .help_overlay
            .controller
            .smc_last_knob
            .as_ref()
            .filter(|_| ready);
        let value = last
            .filter(|_| self.value_popup.as_ref().is_some_and(|shown| shown.knob))
            .zip(self.current_value_at(now))
            .map(|(last, value)| {
                format!(
                    "B{} {} {} · {} · {}",
                    last.bank + 1,
                    last.knob + 1,
                    value.parameter,
                    value.object,
                    value.formatted
                )
            });
        let operation = (self.screen == input::Screen::Chop
            && ready
            && ["e3", "e4", "e5", "e6"].map(action)
                == [
                    Some("slice.start"),
                    Some("slice.end"),
                    Some("chop.region_start_pos"),
                    Some("chop.region_end_pos"),
                ])
        .then_some("B1 3–4 slice · 5–6 region");
        tui::pads::set_smc_words(self.input_sources.last_is_smc());
        Some(tui::KnobStrip {
            mode: self.mode_line(now),
            page,
            list,
            operation,
            roles,
            last_bank: last.map(|last| last.bank),
            last_knob: last.map(|last| last.knob),
            value,
        })
    }

    fn smc_value_label(&self, action: &str) -> Option<(&'static str, bool)> {
        use input::midi::Param;
        let param = match input::midi::param_of(action)? {
            Param::SmcPageValue(knob) => {
                let (label, param) = self.smc_page_role(knob).unwrap_or(("-", None));
                return Some((label, param.is_none()));
            }
            Param::Slot(slot) => self.slot_param(slot),
            _ => return None,
        };
        let text = match param {
            Some(Param::PadSource) => "source",
            Some(Param::PadEffect(0)) => "drive",
            Some(Param::PadEffect(9)) => "mute",
            Some(Param::PadEffect(index @ 3..=8)) => {
                self.pads.name(super::EFFECT_PARAMS[usize::from(index)])
            }
            Some(param) => input::midi::PARAMS
                .iter()
                .find_map(|(name, value)| (*value == param).then_some(input::short_label(name)))
                .unwrap_or("-"),
            None => "-",
        };
        Some((text, text == "-"))
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
                note: None,
                navigation,
                caption,
            };
        };
        let profile = &devices[index].profile;
        let edit = self.edit_keys();
        let action = |name: &str| {
            profile.control_action_context(self.screen, edit, self.pattern_panel.is_some(), name)
        };
        let panel_role = |name: &str| {
            let cell = name.strip_prefix('c')?.parse::<u8>().ok()?;
            if profile.name != "SMC-PAD" || action(name) != Some("panel.cell") {
                return None;
            }
            // The control row's pad 4 is dim when nothing waits for a yes (134).
            let label = self
                .panel
                .label(self.panel_view(), cell)
                .unwrap_or(if cell == 16 { "-" } else { "unavailable" });
            Some((
                label,
                cell >= 15 || matches!(label, "unavailable" | "mixer"),
            ))
        };
        let value_role = |name: &str| {
            if profile.name != "SMC-PAD" {
                return None;
            }
            let knob = matches!(profile.kind_of(name), Some(ControlKind::Knob));
            if knob {
                let (label, available) = self.smc_knob_role(name, action(name));
                Some((label, !available))
            } else if matches!(action(name), Some("smc.list_prev" | "smc.list_next")) {
                Some((self.smc_context().1, false))
            } else {
                None
            }
        };
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
        let mut detail = match &selected {
            Some(name) => {
                let kind = profile.kind_of(name).map_or("", |k| k.name());
                match panel_role(name) {
                    Some((role, free)) => format!(
                        "{name} · {kind} · panel cell · {role}{}",
                        if free { " unavailable" } else { "" }
                    ),
                    None => {
                        if profile.name == "SMC-PAD"
                            && profile.kind_of(name) == Some(ControlKind::Knob)
                            && let Some(detail) = self.smc_knob_detail(name, action(name))
                        {
                            detail
                        } else if let Some((role, _)) = value_role(name) {
                            format!("{name} · {kind} · {role}")
                        } else {
                            match action(name) {
                                Some(a) => format!("{name} · {kind} · {a} · {}", label(a)),
                                None => format!("{name} · {kind} · free"),
                            }
                        }
                    }
                }
            }
            None => String::new(),
        };
        if let Some(name) = &selected
            && detail.starts_with(name.as_str())
        {
            detail = detail.replacen(name.as_str(), &smc_printed(&profile.name, name), 1);
        }
        if profile.name == "SMC-PAD" {
            let status = match &self.help_overlay.controller.smc_status {
                Some((true, _)) => "encoders ready",
                Some((false, _)) | None => "encoders unavailable",
            };
            detail = format!("{status} · {detail}");
        }
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
                                let (short, free) = match panel_role(name)
                                    .or_else(|| value_role(name))
                                {
                                    Some((role, free)) => (role.to_string(), free),
                                    None => (
                                        pad_label(action(name)).unwrap_or_else(|| {
                                            action(name).map_or("-", input::short_label).to_string()
                                        }),
                                        action(name).is_none(),
                                    ),
                                };
                                Some(tui::ControlCell {
                                    name: smc_printed(&profile.name, name),
                                    label: format!("{open}{short}{close}"),
                                    free,
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
                        let what = if let Some((role, _)) = value_role(name) {
                            role.into()
                        } else {
                            action(name).map_or("-".to_string(), |a| format!("{a} · {}", label(a)))
                        };
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
            note: (profile.name == "SMC-PAD").then_some(
                "Device Note Repeat uses its own clock. Use ptsr Note Repeat when available. PAD BANK: screen actions, LEFT: screens.",
            ),
            navigation,
            caption,
        }
    }
}

/// `pad.3` reads by the pad name on the grid: `pad3`, or the device number with an
/// SMC-PAD connection (134).
fn pad_label(action: Option<&str>) -> Option<String> {
    let index: usize = action?.strip_prefix("pad.")?.parse().ok()?;
    (1..=16)
        .contains(&index)
        .then(|| format!("pad{}", tui::pads::pad_tag(index - 1)))
}

/// The SMC-PAD pads as printed (134): `p1`, the top-left pad of the profile, is P13;
/// its PAD BANK layer `c1` is PB13. Other controls keep their profile names.
pub(super) fn smc_printed(profile: &str, name: &str) -> String {
    if profile != "SMC-PAD" {
        return name.to_string();
    }
    let printed = |prefix: &str, rest: &str| {
        rest.parse::<usize>()
            .ok()
            .filter(|n| (1..=16).contains(n))
            .map(|n| format!("{prefix}{}", tui::device_number(n)))
    };
    name.strip_prefix('p')
        .and_then(|rest| printed("P", rest))
        .or_else(|| name.strip_prefix('c').and_then(|rest| printed("PB", rest)))
        .unwrap_or_else(|| name.to_string())
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

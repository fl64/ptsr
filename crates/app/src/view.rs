//! Screen data prepared from application state.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

use engine::{PADS, PADS_PER_BANK, Region, Sample, test_tone_name};
use input::Screen;

use super::{
    App, BIG_COLS, HIT_FLASH, HelpState, LIT_FOR, Level, Lit, Notice, Origin, PAD_KEYS, audio,
    bank_fill, bank_marks, chop, chop_view_routed, file_name, filter_curve, label8, pad_name, pads,
    same_whole_sound, source_word,
};

pub(super) fn pitch_gate(pads: &pads::Pads, abs: usize) -> tui::PitchGate {
    if !pads.pitch_gate[abs] {
        tui::PitchGate::Off
    } else if !matches!(pads.source(abs), pads::Source::Slice(_)) {
        tui::PitchGate::NeedsSlice
    } else if i16::from(pads.pitch[abs]) * 100 + i16::from(pads.shape[abs].cents) <= 0 {
        tui::PitchGate::NeedsPitch
    } else {
        tui::PitchGate::A1
    }
}

impl super::Ask {
    pub(super) fn lines(&self) -> &[String] {
        use super::Ask;
        match self {
            Ask::SmcLayout(review) => &review.lines,
            Ask::Exit { lines }
            | Ask::ExitRecovery { lines, .. }
            | Ask::Plan { lines }
            | Ask::PutFile { lines, .. }
            | Ask::ResetProfile { lines, .. }
            | Ask::ResetPads { lines, .. }
            | Ask::ClearPads { lines, .. }
            | Ask::CopyPad { lines, .. }
            | Ask::ClearPattern { lines }
            | Ask::Destroy { lines, .. } => lines,
        }
    }
}

fn question_answer(ask: &super::Ask, action: input::Action) -> Option<bool> {
    use super::Ask;
    use input::Action;
    match action {
        Action::Back => Some(false),
        Action::Confirm => Some(true),
        _ if matches!(
            ask,
            Ask::Exit { .. } | Ask::ExitRecovery { .. } | Ask::SmcLayout(_)
        ) =>
        {
            None
        }
        Action::SettingsLearn if matches!(ask, Ask::ResetProfile { .. }) => Some(true),
        Action::BrowserLoad if matches!(ask, Ask::PutFile { .. }) => Some(true),
        Action::ChopCommit | Action::ChopEdit | Action::BrowserHold | Action::PadEdit
            if !matches!(ask, Ask::ResetProfile { .. }) =>
        {
            Some(true)
        }
        _ => None,
    }
}

#[derive(PartialEq, Eq)]
struct RecordingKey {
    epoch: u64,
    pass: u64,
    phase: engine::capture::Phase,
    count: u8,
    bar: u64,
    beat: u64,
    bars: u32,
    erasing: bool,
    targets: Vec<String>,
    grid: String,
    strength: u8,
}

impl RecordingKey {
    fn new(app: &App, state: engine::capture::Snapshot) -> Option<Self> {
        use engine::capture::Phase;
        if state.phase == Phase::Idle {
            return None;
        }
        let bars = (state.loop_ticks / engine::pattern::TICKS_PER_BAR).max(1);
        let beat = ((state.ticks.max(0.0) / engine::transport::TICKS_PER_QUARTER + 1e-9).floor()
            as u64)
            % (u64::from(bars) * 4);
        Some(Self {
            epoch: state.epoch,
            pass: state.pass,
            phase: state.phase,
            count: state.count_in_beat,
            bar: beat / 4 + 1,
            beat: beat % 4 + 1,
            bars,
            erasing: state.phase == Phase::Recording && state.erasing,
            targets: (0..PADS)
                .filter(|&pad| {
                    state.erasing && state.erase_targets[pad / 64] & (1 << (pad % 64)) != 0
                })
                .map(super::bank_key)
                .collect(),
            grid: app
                .pads
                .pattern
                .as_ref()
                .map_or("1/16", |p| p.quantize.as_str())
                .into(),
            strength: app.pads.pattern.as_ref().map_or(100, |p| p.strength),
        })
    }

    fn badge(&self) -> tui::Badge {
        use engine::capture::Phase;
        let (normal, compact) = match self.phase {
            Phase::CountIn => {
                let text = format!("COUNT {} A1", self.count);
                (text.clone(), text)
            }
            Phase::Recording => {
                let position = format!("{}:{}/{}", self.bar, self.beat, self.bars);
                let label = if self.erasing { "ERASE A1" } else { "REC A1" };
                let targets = self.targets.first().map_or(String::new(), |first| {
                    if self.targets.len() > 1 {
                        format!(" {first}+{}", self.targets.len() - 1)
                    } else {
                        format!(" {first}")
                    }
                });
                let compact = format!("{label} {position}{targets}");
                (
                    format!("{compact} {} {}%", self.grid, self.strength),
                    compact,
                )
            }
            Phase::Finishing => ("Finishing A1".into(), "Finishing A1".into()),
            Phase::Idle => unreachable!(),
        };
        tui::Badge::new(tui::BadgeKind::Attention, normal, compact)
    }
}

#[derive(PartialEq, Eq)]
struct BadgeKey {
    screen: Screen,
    mode: input::Mode,
    assignment: Option<(usize, usize)>,
    marked: usize,
    /// The last input came from the SMC-PAD: drops name its control, not Esc (135).
    smc: bool,
    page: &'static str,
    seconds: Option<u64>,
    looping: bool,
    as_pad: bool,
    transport: (u16, bool, bool, u64, u8),
    pattern: Option<(u64, u8)>,
    recording: Option<RecordingKey>,
}

impl BadgeKey {
    fn format(&self) -> Vec<tui::Badge> {
        use tui::{
            Badge,
            BadgeKind::{Attention, On, PadMode},
        };
        let mode = self.mode;
        let mut pad_modes = Vec::new();
        if mode.screen_learn || mode.midi_learn {
            pad_modes.push(Badge::new(PadMode, "LEARN", "LEARN"));
        }
        if let Some((placed, total)) = self.assignment {
            pad_modes.push(Badge::new(
                PadMode,
                format!("ASSIGN · {placed}/{total} placed"),
                format!("ASSIGN {placed}/{total}"),
            ));
        }
        let placements: Vec<_> = [
            (mode.file_in_hand, "IN HAND"),
            (mode.copy_in_hand, "COPY"),
            (mode.swap_in_hand, "SWAP"),
        ]
        .into_iter()
        .filter_map(|(active, name)| active.then_some(name))
        .collect();
        if !placements.is_empty() {
            pad_modes.push(Badge::new(
                PadMode,
                format!("{} · hit a pad", placements.join(" + ")),
                placements.join("/"),
            ));
        }
        if mode.erase_pads {
            pad_modes.push(Badge::new(
                PadMode,
                "ERASE · hit a pad · all its A1 hits",
                "ERASE",
            ));
        }
        let count = self.marked;
        if mode.marking {
            pad_modes.push(Badge::new(
                PadMode,
                format!("MARK · hit pads · {count} marked"),
                format!("MARK {count}"),
            ));
        } else if count > 0 {
            pad_modes.push(Badge::new(
                PadMode,
                format!(
                    "{count} MARKED · {} drops",
                    if self.smc { "PAD BANK + 1" } else { "Esc" }
                ),
                format!("{count} MARKED"),
            ));
        }
        let (tempo, playing, metro, bar, beat) = self.transport;
        let mut clock = format!(
            "{} {:.1}{}{}",
            if playing { "▶" } else { "■" },
            tempo as f64 / 10.0,
            if metro { " click" } else { "" },
            if playing {
                format!(" {bar}:{beat}")
            } else {
                String::new()
            }
        );
        if let Some((bar, beat)) = self.pattern {
            clock.push_str(&format!(" · A1 {bar}:{beat}"));
        }
        let compact_clock = clock.replace(' ', "").replace("click", "·click");
        let mut badges = Vec::new();
        let mut pad_modes = pad_modes.into_iter();
        let clock_badge = if let Some(mut mode) = pad_modes.next() {
            mode.normal.push_str(&format!(" · {clock}"));
            mode.compact.push_str(&format!(" {compact_clock}"));
            mode
        } else {
            Badge::new(On, &clock, &compact_clock)
        };
        badges.push(clock_badge);
        if mode.question {
            badges.push(Badge::new(Attention, "ASK", "ASK"));
        }
        if let Some(seconds) = self.seconds {
            badges.push(Badge::new(Attention, format!("● REC {seconds}s"), "REC"));
        }
        badges.extend(pad_modes);
        if mode.edit {
            badges.push(Badge::new(
                On,
                format!("EDIT · {}", self.page),
                format!("EDIT {}", self.page),
            ));
        }
        if self.looping {
            badges.push(Badge::new(On, "LOOP", "LOOP"));
        }
        if self.as_pad {
            badges.push(Badge::new(On, "AS PAD", "AS PAD"));
        }
        if let Some(recording) = &self.recording {
            let mut badge = recording.badge();
            if mode.question {
                if let Some(index) = badges.iter().position(|b| b.normal == "ASK") {
                    let ask = badges.remove(index);
                    badges.insert(0, ask);
                }
                badge.compact = match recording.phase {
                    engine::capture::Phase::CountIn => format!("COUNT {} A1", recording.count),
                    engine::capture::Phase::Finishing => "Finishing A1".into(),
                    _ if recording.erasing => "ERASE A1".into(),
                    _ => "REC A1".into(),
                };
            }
            badges.insert(usize::from(mode.question), badge);
        }
        badges
    }
}

struct SourceKey {
    keyboard_active: bool,
    controller: Option<(super::input_sources::Source, bool)>,
    connections: Vec<(String, super::input_sources::Connection)>,
}

/// Current formatted labels only. Timers and live screen state remain outside this cache.
#[derive(Default)]
pub(super) struct FrameText {
    badge_key: Option<BadgeKey>,
    badges: Vec<tui::Badge>,
    source_key: Option<SourceKey>,
    inputs: Arc<[tui::InputSource]>,
    #[cfg(test)]
    badge_builds: usize,
    #[cfg(test)]
    source_builds: usize,
}

impl FrameText {
    #[cfg(test)]
    pub(super) fn badges_at(&mut self, app: &App, now: Instant) -> &[tui::Badge] {
        self.badges_for(app, now, app.pattern_record_state())
    }

    pub(super) fn badges_for(
        &mut self,
        app: &App,
        now: Instant,
        recording: Option<engine::capture::Snapshot>,
    ) -> &[tui::Badge] {
        let key = app.badge_key_at(now, recording);
        if self.badge_key.as_ref() != Some(&key) {
            self.badges = key.format();
            self.badge_key = Some(key);
            #[cfg(test)]
            {
                self.badge_builds += 1;
            }
        }
        &self.badges
    }

    pub(super) fn feedback_at(&mut self, app: &App, now: Instant) -> tui::Feedback {
        let source = &app.input_sources;
        let unchanged = self.source_key.as_ref().is_some_and(|key| {
            key.keyboard_active == source.keyboard_at(now)
                && key.controller == source.controller_at(now)
                && source.connected().eq(key
                    .connections
                    .iter()
                    .map(|(name, id)| (name.as_str(), *id)))
        });
        if !unchanged {
            self.inputs = app.status_feedback_at(now).inputs;
            self.source_key = Some(SourceKey {
                keyboard_active: source.keyboard_at(now),
                controller: source.controller_at(now),
                connections: source
                    .connected()
                    .map(|(name, id)| (name.to_owned(), id))
                    .collect(),
            });
            #[cfg(test)]
            {
                self.source_builds += 1;
            }
        }
        tui::Feedback {
            inputs: Arc::clone(&self.inputs),
            knob_strip: app.smc_knob_strip(now),
            recording_seconds: app
                .recording
                .map(|(_, start)| now.saturating_duration_since(start).as_secs()),
        }
    }

    #[cfg(test)]
    pub(super) fn builds(&self) -> (usize, usize) {
        (self.badge_builds, self.source_builds)
    }
}

impl App {
    pub(super) fn status_feedback_at(&self, now: Instant) -> tui::Feedback {
        tui::Feedback {
            inputs: self.input_sources.snapshot(now).into(),
            knob_strip: None,
            recording_seconds: self
                .recording
                .map(|(_, start)| now.saturating_duration_since(start).as_secs()),
        }
    }

    pub(super) fn question_view(&self, keymap: &input::Keymap) -> Option<tui::Question> {
        use super::Ask;
        use input::midi::{Activity, Msg, Profile, Resolved};
        let ask = self.ask.as_ref()?;
        let verb = match ask {
            Ask::Exit { .. } => "exit",
            Ask::ExitRecovery { .. } => "Retry Save",
            Ask::SmcLayout(_) => "apply",
            Ask::Plan { .. } | Ask::PutFile { .. } | Ask::CopyPad { .. } => "replace",
            Ask::ResetProfile { .. } | Ask::ResetPads { .. } => "reset",
            Ask::ClearPads { .. } | Ask::ClearPattern { .. } => "clear",
            Ask::Destroy { act, .. } => match act {
                crate::destroy::Act::Forget => "forget",
                crate::destroy::Act::ClearCuts => "remove",
                crate::destroy::Act::TrimSilence => "trim",
                crate::destroy::Act::AssignOrder => "switch",
                _ => "replace",
            },
        };
        let recovering = matches!(ask, Ask::ExitRecovery { .. });
        let cancel_verb = if recovering { "Back" } else { "keep" };
        let focus = self.focus();
        let keyboard_blocked = focus.mode.text || focus.mode.picker || focus.mode.screen_learn;
        let key = |confirm| {
            let mut bindings = keymap.bindings(self.screen);
            bindings.push(("Enter".into(), input::Action::Confirm));
            bindings
                .into_iter()
                .filter(|(key, _)| {
                    if keyboard_blocked {
                        return false;
                    }
                    let effective = if key == "Enter" {
                        keymap.resolve_modal_enter(
                            self.screen,
                            self.edit_keys(),
                            &ratatui::crossterm::event::KeyEvent::new(
                                ratatui::crossterm::event::KeyCode::Enter,
                                ratatui::crossterm::event::KeyModifiers::NONE,
                            ),
                        )
                    } else {
                        keymap
                            .binding_for(self.screen, self.edit_keys(), key)
                            .flatten()
                    };
                    effective.is_some_and(|action| question_answer(ask, action) == Some(confirm))
                })
                .map(|(key, _)| key)
                .min_by_key(|key| (key != "Enter" && key != "Esc", key.clone()))
        };
        let confirm =
            key(true).map_or_else(|| "no confirm key".into(), |key| format!("{key}: {verb}"));
        let cancel = key(false).map_or_else(
            || "no cancel key".into(),
            |key| format!("{key}: {cancel_verb}"),
        );
        let mut answers = vec![format!("KB {confirm} · {cancel}")];
        // After an SMC-PAD input the answer row names its pads only (134).
        if self.input_sources.last_is_smc() && !focus.mode.midi_learn {
            answers.clear();
        }
        if !keyboard_blocked && !focus.mode.midi_learn {
            let maps = super::midi::parse_learned(&self.learned);
            for (index, (port, _)) in self.input_sources.connected().enumerate() {
                if port.contains("SMC-PAD-Master") {
                    answers.push(format!("SMC P1: {cancel_verb} · P4: {verb}"));
                    continue;
                }
                // Match the worker's port selection. Assignment labels can search another layer.
                let mut profile = self
                    .profiles
                    .iter()
                    .find(|p| p.matches(port))
                    .cloned()
                    .unwrap_or_else(|| Profile::default_for(port));
                super::midi::apply_learned(&mut profile, &maps);
                profile.set_seeded(self.seeded.contains(&profile.name));
                let mut controls: [Option<String>; 2] = [None, None];
                'channels: for ch in 1..=16 {
                    for number in 0..=127 {
                        for msg in [
                            Msg::Cc {
                                ch,
                                cc: number,
                                value: 127,
                            },
                            Msg::NoteOn {
                                ch,
                                note: number,
                                velocity: 127,
                            },
                        ] {
                            let (Some(Resolved::Action(action)), Activity::Press) = profile
                                .resolve_input_modal(self.screen, focus.mode.edit, true, msg)
                            else {
                                continue;
                            };
                            let Some(confirm) = question_answer(ask, action) else {
                                continue;
                            };
                            let slot = &mut controls[usize::from(!confirm)];
                            if slot.is_none() {
                                *slot = Some(format!(
                                    "ch{ch} {}: {}",
                                    msg.key(),
                                    if confirm { verb } else { cancel_verb }
                                ));
                            }
                            if controls.iter().all(Option::is_some) {
                                break 'channels;
                            }
                        }
                    }
                }
                let controls = controls.into_iter().flatten().collect::<Vec<_>>();
                if !controls.is_empty() {
                    answers.push(format!("M{} {}", index + 1, controls.join(" · ")));
                }
            }
        }
        // Stored lines remain the overwrite guard. Change only this drawing snapshot.
        let lines = ask.lines();
        let body = if recovering {
            lines.to_vec()
        } else {
            lines[..lines.len().saturating_sub(1)].to_vec()
        };
        Some(tui::Question { body, answers })
    }

    pub(super) fn learning_view(&self, keymap: &input::Keymap) -> Option<tui::Learning> {
        let old = self.learn_lines()?;
        let learn = self.screen_learn.as_ref()?;
        // A clear request can retain the previous refusal flag. Show the current confirmation.
        let refused = learn.refused && learn.pending.is_none() && !learn.clearing;
        let place = match learn.page {
            Some(page) => format!("{} · {page}", learn.group.title()),
            None => learn.group.title().into(),
        };
        let context = if let Some((name, label)) = &learn.picked {
            let control = self
                .learn_shown(&self.live_profiles(), name)
                .unwrap_or_else(|| "none".into());
            format!(
                "LEARN · {label}{} · {place} · {control}",
                if learn.once { " · last action" } else { "" }
            )
        } else {
            format!("LEARN · {place} · select a pad or command")
        };
        let detail = if refused {
            if self
                .learn_picked()
                .is_some_and(|(name, _)| super::action_info(name).knob)
            {
                "Use a knob or fader. A button cannot change this value.".into()
            } else {
                "Use a button or pad. A knob repeats this action.".into()
            }
        } else if learn.pending.is_some() || learn.clearing {
            old.get(1).cloned().unwrap_or_default().trim().into()
        } else if learn.picked.is_some() {
            "Move a MIDI control to prepare its binding.".into()
        } else {
            "Pad and command keys select an action. MIDI does not play.".into()
        };
        let exit = keymap
            .keys_for(self.screen, input::Action::LearnToggle)
            .into_iter()
            .find(|key| {
                keymap.binding_for(self.screen, false, key)
                    == Some(Some(input::Action::LearnToggle))
            });
        let leave = exit
            .as_ref()
            .map_or_else(String::new, |key| format!(" · {key} done"));
        let done = exit.map_or_else(|| "Esc done".into(), |key| format!("Esc/{key} done"));
        let controls = if learn.pending.is_some() {
            format!("Enter bind · Esc keep{leave}")
        } else if learn.clearing {
            format!("Enter clear · Esc keep{leave}")
        } else {
            let mut parts = Vec::new();
            if !self.learn_params().is_empty() {
                let walk = [input::Action::ParamPrev, input::Action::ParamNext]
                    .into_iter()
                    .filter_map(|action| {
                        keymap
                            .edit_keys_for(action)
                            .into_iter()
                            .chain(keymap.keys_for(self.screen, action))
                            .find(|key| {
                                keymap.binding_for(self.screen, true, key) == Some(Some(action))
                            })
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                if !walk.is_empty() {
                    parts.push(format!("{walk} select"));
                }
            }
            if self
                .learn_picked()
                .and_then(|(name, _)| self.learn_label(&self.live_profiles(), name))
                .is_some()
            {
                parts.push("Bksp clear".into());
            }
            parts.push(done);
            parts.join(" · ")
        };
        Some(tui::Learning {
            context,
            detail,
            controls,
            warning: refused || self.learn_taken().is_some(),
        })
    }

    pub(super) fn pattern_controls_view(
        &self,
        keymap: &input::Keymap,
    ) -> Option<tui::PatternControls> {
        use input::Action;
        let selected = self.pattern_panel?;
        if self.key_notice_blocked() || self.help_open || self.midi_run.is_some() {
            return None;
        }
        let keys = |actions: &[Action]| {
            let mut keys = actions
                .iter()
                .flat_map(|action| self.help_bindings(keymap, *action))
                .collect::<Vec<_>>();
            keys.sort();
            keys.dedup();
            let keys = super::letters_first(keys).join("/");
            if keys.is_empty() { "none".into() } else { keys }
        };
        let (restriction, refused) = self.pattern_controls_restriction(selected);
        let hits = self.pads.pattern.as_ref().map_or(0, |p| p.hits.len());
        Some(tui::PatternControls {
            selected,
            dim: (0..super::pattern_record::PANEL_ROWS)
                .map(|row| row == super::pattern_record::CLEAR_ROW && hits == 0)
                .collect(),
            values: super::pattern_record::Setting::ALL
                .map(|setting| (setting.label().to_lowercase(), setting.text(&self.pads)))
                .into_iter()
                .chain(std::iter::once((
                    "clear A1".into(),
                    if hits == 0 {
                        "no hits".into()
                    } else {
                        super::pattern_record::hits(hits)
                    },
                )))
                .collect(),
            restriction: restriction.into(),
            refused,
            controls: [
                format!(
                    "{} select  {} change",
                    keys(&[
                        Action::ParamPrev,
                        Action::ParamNext,
                        Action::PatternParamPrev,
                        Action::PatternParamNext,
                    ]),
                    keys(&[
                        Action::EditValue {
                            up: false,
                            fine: false
                        },
                        Action::EditValue {
                            up: true,
                            fine: false
                        },
                        Action::EditValue {
                            up: false,
                            fine: true
                        },
                        Action::EditValue {
                            up: true,
                            fine: true
                        },
                        Action::PatternValueDec,
                        Action::PatternValueInc,
                    ]),
                ),
                // Enter runs the clear row and closes from a value row (98i)
                format!(
                    "{} close · Enter run",
                    keys(&[Action::Back, Action::PatternControls])
                        .split('/')
                        .filter(|key| *key != "Enter")
                        .collect::<Vec<_>>()
                        .join("/")
                ),
            ],
        })
    }

    pub(super) fn current_value_at(&self, now: Instant) -> Option<&tui::Value> {
        if self.pattern_panel.is_some() {
            return None;
        }
        let shown = self.value_popup.as_ref()?;
        if now.saturating_duration_since(shown.at) >= LIT_FOR
            || self.key_notice_blocked()
            || self.midi_run.is_some()
        {
            return None;
        }
        shown.current(self)
    }

    pub(super) fn message_history(&self) -> Vec<tui::Message<'_>> {
        self.notices
            .history()
            .iter()
            .map(|entry| tui::Message {
                level: match entry.level {
                    Level::Info => tui::NoticeLevel::Info,
                    Level::Done => tui::NoticeLevel::Done,
                    Level::Warn => tui::NoticeLevel::Warn,
                    Level::Error => tui::NoticeLevel::Error,
                },
                origin: match &entry.origin {
                    Origin::Keyboard => "Keyboard",
                    Origin::Device(port) => port.as_str(),
                    Origin::System => "System",
                },
                text: &entry.text,
            })
            .collect()
    }

    #[cfg(test)]
    pub(super) fn full_help_keys(
        &self,
        keymap: &input::Keymap,
        kitty: bool,
    ) -> Vec<(String, String)> {
        self.full_help_recording_keys(keymap, kitty, self.pattern_record_state())
    }

    pub(super) fn full_help_recording_keys(
        &self,
        keymap: &input::Keymap,
        kitty: bool,
        recording: Option<engine::capture::Snapshot>,
    ) -> Vec<(String, String)> {
        use input::Action;
        use input::midi::{Activity, Msg, Resolved};
        let mut rows = super::full_help(keymap, self.screen, kitty);
        if self.pattern_view.overview {
            rows.push((
                "Pattern overview".into(),
                if self.pattern_view.edit {
                    "Step controls move the time column. A pad selects its row and adds or removes its hit without sound."
                } else {
                    "Rows with hits and the selected pad show by default. The row toggle shows all rows. Pads still play."
                }
                .into(),
            ));
            rows.push((
                "A1 marks".into(),
                "Square: hit. Tilde: off-grid hit. Number: several hits. Dim/normal/bright: velocity 1-42/43-84/85-127. Light plate: playback. Cyan column: time cursor. Arrow: selected pad row."
                    .into(),
            ));
            if self.pattern_view.edit {
                rows.push((
                    "Step correction".into(),
                    "Use the shown step keys or learned MIDI step controls to move the time column. A pad selects its row and adds or removes only its hit at that step. An arrow marks the selected pad row. Use the event keys when several hits share one cell. Edit pads stay silent. Undo restores."
                        .into(),
                ));
            }
            rows.push((
                "Bank scope".into(),
                if self.pattern_view.edit {
                    "Bank keys move the edit target. The selected time cell stays the same."
                } else {
                    "Bank keys move through sections. The current bank stays visible. The table does not edit A1."
                }.into(),
            ));
            rows.push((
                self.help_bindings(keymap, Action::PatternOverviewEmptyToggle)
                    .join("/"),
                format!(
                    "Empty rows: {}. Switch all rows without changing A1.",
                    if self.pattern_view.hide_empty_banks {
                        "hidden"
                    } else {
                        "shown"
                    }
                ),
            ));
        }
        for action in [
            Action::HelpPagePrev,
            Action::HelpPageNext,
            Action::PatternRecord,
            Action::RecordMic(true),
            Action::RecordOutputToggle,
        ] {
            let description = if action == Action::RecordMic(true) && !kitty {
                "record the microphone: toggle on / off (no key releases)".into()
            } else {
                super::describe(action)
            };
            rows.retain(|(_, text)| text != &description);
            let keys = self.help_bindings(keymap, action);
            rows.push((
                if keys.is_empty() {
                    "none".into()
                } else {
                    keys.join(" ")
                },
                description,
            ));
        }
        rows.push((
            "A1 save".into(),
            "Manual save finishes the pass and keeps playback running.".into(),
        ));
        rows.push((
            "A1 erase".into(),
            if kitty {
                "Hold pad targets silently. Release to stop erasing them."
            } else {
                "Keyboard: one loop per pad press. MIDI targets use press and release."
            }
            .into(),
        ));
        rows.push((
            "A1 settings".into(),
            super::pattern_record::Setting::ALL
                .into_iter()
                .map(|setting| format!("{} {}", setting.label(), setting.text(&self.pads)))
                .collect::<Vec<_>>()
                .join(" · "),
        ));
        if let Some(key) = recording.and_then(|s| RecordingKey::new(self, s)) {
            rows.push(("A1 state".into(), key.badge().normal));
            if !key.targets.is_empty() {
                rows.push(("Erase targets".into(), key.targets.join(" ")));
            }
        }
        if self
            .midi_devices
            .iter()
            .any(|(port, _)| port.contains("SMC-PAD-Master"))
        {
            let view = self.panel_view();
            rows.push((
                "SMC-PAD".into(),
                format!("{} LEFT and RIGHT move its views. View: {}. PLAY starts or continues. With the panel closed, STOP pauses or resets. With the panel open, STOP twice stops all sound.", if self.midi_gestures.smc_pad_bank_opens_panel { "Hold PAD BANK to show the panel; a pad hit during the hold runs its cell." } else { "PAD BANK shows the panel on the first second-bank hit." }, if self.panel.page() == 0 { "screens" } else { "actions" }),
            ));
            if self.panel_target().is_some() {
                rows.push((
                    "Targets".into(),
                    "P3 Menu, then P3 Redo. P1 returns to targets.".into(),
                ));
            }
            for first in [1, 5, 9, 13] {
                let cells = (first..first + 4)
                    .map(|cell| {
                        let name = self.panel.label(view, cell).unwrap_or("unavailable");
                        if cell == 16
                            || name == "unavailable"
                            || (cell == 15 && self.redo_refusal().is_some())
                        {
                            format!(
                                "P{} {name} unavailable",
                                tui::device_number(usize::from(cell))
                            )
                        } else {
                            format!("P{} {name}", tui::device_number(usize::from(cell)))
                        }
                    })
                    .collect::<Vec<_>>();
                rows.push(("SMC cells".into(), cells.join(" · ")));
            }
        }
        if self.screen != Screen::Pads {
            return rows;
        }
        rows.retain(|(_, text)| !text.starts_with("Pitch Gate:"));
        let mut keys = keymap.keys_for(self.screen, Action::PadPitchGateToggle);
        if self.edit_keys() {
            keys.extend(keymap.edit_keys_for(Action::PadPitchGateToggle));
        }
        keys.retain(|key| {
            keymap.binding_for(self.screen, self.edit_keys(), key)
                == Some(Some(Action::PadPitchGateToggle))
        });
        keys.sort();
        keys.dedup();
        rows.push((
            "A1 view".into(),
            "■ hit; ~ off grid; 2..9/+ hits in one cell; light plate: playback. No quantize."
                .into(),
        ));
        rows.push((
            "A1 rows".into(),
            "all: any pad of this bank. Underlined bank number: that bank has A1 hits.".into(),
        ));
        let other_rows = std::mem::take(&mut rows);
        rows.push((String::new(), "PITCH GATE".into()));
        let pad = self.pads.selected_abs();
        rows.push((
            keys.join(" "),
            format!(
                "pad {}: {} ({})",
                super::bank_key(pad),
                pitch_gate(&self.pads, pad).label(),
                pitch_gate(&self.pads, pad).description()
            ),
        ));
        rows.push((
            String::new(),
            "Only A1 notes: linked slices with pitch above zero.".into(),
        ));
        rows.push((
            String::new(),
            "Manual hits and previews stay unchanged.".into(),
        ));
        rows.push((
            String::new(),
            "Cut at the next grid line, even an empty step.".into(),
        ));
        rows.push((
            String::new(),
            "Quantize Off uses 1/16. Pitch Gate does not extend short slices.".into(),
        ));
        for profile in self.live_profiles() {
            let mut controls = std::collections::BTreeSet::new();
            for ch in 1..=16 {
                for number in 0..=127 {
                    for msg in [
                        Msg::Cc {
                            ch,
                            cc: number,
                            value: 127,
                        },
                        Msg::NoteOn {
                            ch,
                            note: number,
                            velocity: 127,
                        },
                    ] {
                        if matches!(
                            profile.resolve_input_in(self.screen, self.edit_keys(), msg),
                            (
                                Some(Resolved::Action(Action::PadPitchGateToggle)),
                                Activity::Press
                            )
                        ) {
                            controls.insert(msg.key().to_string());
                        }
                    }
                }
            }
            if !controls.is_empty() {
                rows.push((
                    profile.name,
                    controls.into_iter().collect::<Vec<_>>().join(", "),
                ));
            }
        }
        rows.extend(other_rows);
        rows
    }

    pub(super) fn help_recording_caption(
        &self,
        recording: Option<engine::capture::Snapshot>,
    ) -> Option<String> {
        recording
            .and_then(|state| RecordingKey::new(self, state))
            .map(|key| key.badge().normal)
    }

    pub(super) fn full_help_view<'a>(
        &'a self,
        keys: &'a [(String, String)],
        entries: &'a [tui::Message<'a>],
    ) -> Option<tui::FullHelp<'a>> {
        if !self.help_open {
            return None;
        }
        if self.help_overlay.page == super::help::Page::Controller
            && let Some(view) = self.help_overlay.controller_view.as_ref()
        {
            return Some(tui::FullHelp::Controller(view));
        }
        Some(if self.help_overlay.messages() {
            if let Some(caption) = self.help_overlay.caption.as_deref() {
                tui::FullHelp::MessagesWithCaption {
                    entries,
                    scroll: self.help_overlay.scroll,
                    caption,
                }
            } else {
                tui::FullHelp::Messages {
                    entries,
                    scroll: self.help_overlay.scroll,
                }
            }
        } else {
            tui::FullHelp::KeyPage {
                rows: keys,
                scroll: self.help_overlay.keys_scroll,
                caption: self.help_overlay.caption.as_deref(),
                navigation: &self.help_overlay.navigation,
            }
        })
    }

    pub(super) fn produce_notice(&mut self, notice: Notice, at: Instant) {
        log::debug!(
            "notice {:?} {:?}: {}",
            notice.level,
            notice.origin,
            notice.text
        );
        self.notices.push(notice, at);
    }

    pub(super) fn post_notice(&mut self, level: Level, (text, at): (String, Instant)) {
        self.produce_notice(Notice::new(level, self.notice_origin.clone(), text), at);
    }

    pub(super) fn post_system_notice(&mut self, level: Level, (text, at): (String, Instant)) {
        self.produce_notice(Notice::new(level, Origin::System, text), at);
    }

    pub(super) fn post_startup(&mut self, level: Level, text: String) {
        self.produce_notice(Notice::startup(level, text), Instant::now());
    }

    pub(super) fn add_keymap_warnings(&mut self, warnings: Vec<String>) {
        for text in warnings {
            self.notices
                .record(Notice::new(Level::Warn, Origin::System, text.clone()));
            self.keymap_warnings.push_back(text);
        }
    }

    pub(super) fn result_notice(&mut self, result: Result<(), String>, success: String) {
        let (level, text) =
            result.map_or_else(|error| (Level::Error, error), |_| (Level::Done, success));
        self.post_notice(level, (text, Instant::now()));
    }

    fn recording_warning(&self) -> bool {
        self.notices.recording_refusal_pending()
    }

    fn notice_hidden(&self) -> bool {
        self.key_notice_blocked()
            || (self.recording.is_some() && !self.recording_warning())
            || self
                .audio
                .as_ref()
                .is_some_and(|audio| audio.stopped.load(Relaxed))
    }

    pub(super) fn acknowledge_notice(&mut self) {
        if !self.notice_hidden() {
            self.notices
                .acknowledge_shown(self.shown_error.as_ref(), Instant::now());
        }
        self.shown_error = None;
    }

    fn badge_key_at(&self, now: Instant, recording: Option<engine::capture::Snapshot>) -> BadgeKey {
        let assignment = match (self.screen, &self.plan, &self.chop) {
            (Screen::Chop, Some(plan), Some(chop)) => {
                let total = chop.slices().len();
                let placed = plan
                    .targets(total)
                    .into_iter()
                    .map(|(slice, _)| slice)
                    .collect::<std::collections::BTreeSet<_>>()
                    .len();
                Some((placed, total))
            }
            _ => None,
        };
        let transport = self.transport_state();
        let (bar, beat) = transport.bar_beat();
        let pattern = (transport.pattern_ticks > 0).then(|| {
            let mut relative = transport;
            relative.ticks %= f64::from(transport.pattern_ticks);
            relative.bar_beat()
        });
        BadgeKey {
            recording: recording.and_then(|s| RecordingKey::new(self, s)),
            pattern,
            transport: (
                (transport.bpm * 10.0).round() as u16,
                transport.playing,
                transport.metronome,
                bar,
                beat,
            ),
            screen: self.screen,
            mode: self.focus().mode,
            assignment,
            marked: self.pads.marked.len(),
            smc: self.input_sources.last_is_smc(),
            page: self.help_state().page,
            seconds: self
                .recording
                .map(|(_, start)| now.saturating_duration_since(start).as_secs()),
            looping: self.screen == Screen::Chop
                && self.chop.as_ref().is_some_and(|chop| chop.looping),
            as_pad: self.screen == Screen::Chop
                && self.chop.as_ref().is_some_and(|chop| chop.as_pad),
        }
    }

    /// The uncached reference uses the same formatter as the production cache.
    #[cfg(test)]
    pub(super) fn title_badges(&self) -> Vec<tui::Badge> {
        self.title_badges_at(Instant::now())
    }

    #[cfg(test)]
    pub(super) fn title_badges_at(&self, now: Instant) -> Vec<tui::Badge> {
        self.badge_key_at(now, self.pattern_record_state()).format()
    }

    /// Existing target rules, observed without applying a placement or changing focus.
    pub(super) fn pad_feedback(&self) -> tui::PadFeedback {
        use tui::PadCorner::{Available, Replace, Slice};

        let mode = self.focus().mode;
        let mut feedback = tui::PadFeedback {
            outline: mode.marking
                || mode.file_in_hand
                || mode.copy_in_hand
                || mode.swap_in_hand
                || mode.screen_learn
                || mode.midi_learn,
            ..Default::default()
        };
        if mode.screen_learn {
            return feedback;
        }
        if self.screen == Screen::Chop && self.chop.is_some() {
            if let (Some(plan), Some(chop)) = (&self.plan, &self.chop) {
                feedback.outline = true;
                for (slice, abs) in plan.targets(chop.slices().len()) {
                    if abs / PADS_PER_BANK == self.pads.bank {
                        feedback.corners[abs % PADS_PER_BANK] = Some(Slice(slice as u16 + 1));
                    }
                }
            }
            return feedback;
        }
        for (pad, corner) in feedback.corners.iter_mut().enumerate() {
            let abs = self.pads.bank * PADS_PER_BANK + pad;
            if let Some(file) = &self.browser.in_hand
                && !same_whole_sound(&self.pads, abs, file)
            {
                *corner = Some(if self.pads.file[abs].is_some() {
                    Replace
                } else {
                    Available
                });
            }
            if let Some((source, _)) = &self.copy_in_hand
                && *source != abs
                && self.pads.loading[abs].is_none()
            {
                if self.pads.sound[abs].is_some() {
                    *corner = Some(Replace);
                } else if corner.is_none() {
                    *corner = Some(Available);
                }
            }
            if let Some(source) = self.pad_in_hand
                && source != abs
                && self.pads.loading[source].is_none()
                && self.pads.loading[abs].is_none()
                && corner.is_none()
            {
                *corner = Some(Available);
            }
        }
        feedback
    }

    /// The project picker as drawn: the open project is marked (35).
    pub(super) fn picker_view(&self) -> Option<tui::ProjectPickerView> {
        let open = self.project.as_ref().map(|p| p.name.as_str());
        self.project_picker
            .as_ref()
            .map(|p| tui::ProjectPickerView {
                entries: p
                    .entries
                    .iter()
                    .map(|name| match open {
                        Some(o) if o == name => format!("{name}  ● open"),
                        _ => name.clone(),
                    })
                    .collect(),
                selected: p.selected,
                naming: p.naming.clone(),
            })
    }

    /// The lit thing while it is still lit.
    pub(super) fn lit_now(&self) -> Option<Lit> {
        self.lit
            .filter(|(_, at)| at.elapsed() < LIT_FOR)
            .map(|(l, _)| l)
    }

    pub(super) fn erase_pad_marks(
        &self,
        pads: &mut tui::Pads,
        recording: Option<engine::capture::Snapshot>,
    ) {
        if self.erase_pads {
            // what a press erases, hidden rows too, in place of the mode line (98b)
            let counts = self.pattern_pad_hits();
            for (index, pad) in pads.pads.iter_mut().enumerate() {
                let abs = pads.bank * PADS_PER_BANK + index;
                pad.note = Some(match counts.get(abs) {
                    Some(&n) if n > 0 => super::pattern_record::hits(usize::from(n)),
                    _ => String::new(),
                });
            }
            return;
        }
        let Some(state) =
            recording.filter(|s| s.phase == engine::capture::Phase::Recording && s.erasing)
        else {
            return;
        };
        for (index, pad) in pads.pads.iter_mut().enumerate() {
            let abs = pads.bank * PADS_PER_BANK + index;
            if abs < PADS && state.erase_targets[abs / 64] & (1 << (abs % 64)) != 0 {
                pads.feedback.corners[index] = Some(tui::PadCorner::Erase);
                pad.flash = false;
            }
        }
    }

    pub(super) fn hide_keyboard_help(&self) -> bool {
        self.screen_learn.is_some()
            || (self.input_sources.last_is_smc()
                && (!self.pattern_view.overview || self.help_overlay.size.0 > 80))
    }

    fn compact_a1_help(&self) -> bool {
        let (width, height) = self.help_overlay.size;
        self.pattern_view.overview
            && (72..=80).contains(&width)
            && height >= 26
            && self.midi_devices.iter().any(|(_, name)| name == "SMC-PAD")
    }

    pub(super) fn pattern_help_groups(
        &self,
        keymap: &input::Keymap,
        mut groups: Vec<tui::HelpGroup>,
        recording: Option<engine::capture::Snapshot>,
        kitty: bool,
    ) -> Vec<tui::HelpGroup> {
        use engine::capture::Phase;
        use input::Action;
        if self.pattern_view.overview {
            let item = |actions: &[Action], label: &str| {
                let keys = actions
                    .iter()
                    .flat_map(|&action| self.help_bindings(keymap, action))
                    .collect::<Vec<_>>()
                    .join("/");
                (
                    if keys.is_empty() { "none".into() } else { keys },
                    label.into(),
                )
            };
            let mut clock = super::transport_help(keymap, self.screen);
            clock.items.truncate(1);
            clock.items.push(item(&[Action::HelpToggle], "marks"));
            let bank_keys = [Action::BankPrev, Action::BankNext]
                .into_iter()
                .filter_map(|action| {
                    self.help_bindings(keymap, action)
                        .into_iter()
                        .min_by_key(String::len)
                })
                .collect::<Vec<_>>();
            let bank = (
                if bank_keys.is_empty() {
                    "none".into()
                } else {
                    bank_keys.join("/")
                },
                "bank".into(),
            );
            if self.pattern_view.edit {
                return vec![
                    tui::HelpGroup {
                        name: "STEP EDIT",
                        items: vec![
                            item(&[Action::PatternStepEdit, Action::Back], "leave"),
                            item(
                                &[
                                    Action::EditValue {
                                        up: false,
                                        fine: false,
                                    },
                                    Action::EditValue {
                                        up: true,
                                        fine: false,
                                    },
                                ],
                                "time step",
                            ),
                        ],
                    },
                    tui::HelpGroup {
                        name: "TARGET",
                        items: vec![
                            bank,
                            ("pads".into(), "add/remove".into()),
                            item(&[Action::ParamPrev, Action::ParamNext], "event"),
                        ],
                    },
                    tui::HelpGroup {
                        name: "A1",
                        items: vec![
                            item(&[Action::Undo], "restore"),
                            item(&[Action::PatternViewPrev, Action::PatternViewNext], "view"),
                        ],
                    },
                ];
            }
            let mut pattern = tui::HelpGroup {
                name: "PATTERN",
                items: vec![
                    item(&[Action::ScreenPads, Action::Back], "Pads"),
                    item(&[Action::PatternStepEdit], "edit"),
                    item(&[Action::PatternViewPrev, Action::PatternViewNext], "view"),
                ],
            };
            let compact = self.compact_a1_help();
            if compact {
                pattern.items.push(item(&[Action::HelpToggle], "marks"));
            }
            let mut overview = vec![
                pattern,
                tui::HelpGroup {
                    name: "BANK",
                    items: vec![
                        bank,
                        item(
                            &[Action::PatternOverviewEmptyToggle],
                            if self.pattern_view.hide_empty_banks {
                                "empty rows hidden"
                            } else {
                                "all rows shown"
                            },
                        ),
                    ],
                },
            ];
            if !compact {
                overview.push(clock);
            }
            return overview;
        }
        if self.key_notice_blocked() {
            return groups;
        }
        let item = |action, text: &str| {
            let keys = super::letters_first(self.help_bindings(keymap, action))
                .into_iter()
                .map(|k| k.replace('+', " "))
                .collect::<Vec<_>>()
                .join("/");
            (
                if keys.is_empty() { "none".into() } else { keys },
                text.to_string(),
            )
        };
        let items = if self.pattern_panel.is_some() {
            let keys = |actions: &[Action]| {
                let mut keys = actions
                    .iter()
                    .flat_map(|action| self.help_bindings(keymap, *action))
                    .collect::<Vec<_>>();
                keys.sort();
                keys.dedup();
                let keys = super::letters_first(keys)
                    .into_iter()
                    .map(|k| k.replace('+', " "))
                    .collect::<Vec<_>>()
                    .join("/");
                if keys.is_empty() { "none".into() } else { keys }
            };
            vec![
                (
                    keys(&[
                        Action::ParamPrev,
                        Action::ParamNext,
                        Action::PatternParamPrev,
                        Action::PatternParamNext,
                    ]),
                    "select".into(),
                ),
                (
                    keys(&[
                        Action::EditValue {
                            up: false,
                            fine: false,
                        },
                        Action::EditValue {
                            up: true,
                            fine: false,
                        },
                        Action::PatternValueDec,
                        Action::PatternValueInc,
                    ]),
                    "change".into(),
                ),
                (
                    keys(&[Action::Back, Action::PatternControls])
                        .split('/')
                        .filter(|key| *key != "Enter")
                        .collect::<Vec<_>>()
                        .join("/"),
                    "close controls".into(),
                ),
                ("Enter".into(), "run / close".into()),
            ]
        } else {
            match recording.map(|s| (s.phase, s.erasing)) {
                _ if self.erase_pads => vec![
                    ("pads".into(), "erase all A1 hits".into()),
                    item(Action::Undo, "undo"),
                    (
                        format!(
                            "{}/{}",
                            item(Action::Back, "").0,
                            item(Action::PatternErase, "").0
                        ),
                        "exit".into(),
                    ),
                ],
                Some((Phase::CountIn, _)) => vec![
                    item(Action::Back, "cancel count"),
                    item(Action::PatternRecord, "cancel count"),
                    ("pads".into(), "not recorded yet".into()),
                ],
                Some((Phase::Recording, true)) => vec![
                    (
                        "pads".into(),
                        if kitty {
                            "erase: hold"
                        } else {
                            "erase: 1 loop"
                        }
                        .into(),
                    ),
                    item(Action::PatternErase, "play pads"),
                    item(Action::PatternRecord, "end A1"),
                ],
                Some((Phase::Recording, false)) => vec![
                    item(Action::PatternRecord, "end A1"),
                    item(Action::PatternErase, "erase pads"),
                    item(Action::PatternUndo, "pass undo"),
                ],
                Some((Phase::Finishing, _)) => vec![("A1".into(), "finishing: wait".into())],
                // idle with A1: pass Undo is the hidden next step; an unbound one is left out
                _ if self.pads.pattern.is_some() => [
                    item(Action::PatternRecord, "record A1"),
                    item(Action::PatternUndo, "pass undo"),
                    item(Action::PatternControls, "pattern keys"),
                ]
                .into_iter()
                .filter(|(keys, text)| !(text == "pass undo" && keys == "none"))
                .collect(),
                _ => vec![
                    item(Action::PatternRecord, "record A1"),
                    item(Action::PatternControls, "pattern keys"),
                ],
            }
        };
        let mut items = items;
        if self.screen == Screen::Pads
            && self.pattern_panel.is_none()
            && self.recording.is_none()
            && !self.erase_pads
            && recording.is_none_or(|state| state.phase == Phase::Idle)
            && (self.source_pin_available()
                || self.pads.sound.iter().any(Option::is_some)
                || self.source_view.view_name() == "Rhythm")
        {
            let (action, label) = if self.edit {
                (
                    Action::PadsSourcePin,
                    if self.source_view.pinned(&self.pads) {
                        "unpin sample"
                    } else {
                        "pin sample"
                    },
                )
            } else {
                (
                    Action::PadsStripToggle,
                    if self.source_view.view_name() == "Sound" {
                        "Rhythm"
                    } else {
                        "Sound"
                    },
                )
            };
            let hint = if self.input_sources.last_is_smc() {
                ("PAD BANK+P16".into(), label.into())
            } else {
                item(action, label)
            };
            items.insert(0, hint);
        }
        if self.screen == Screen::Pads
            && self.pads.pattern.is_some()
            && self.pattern_panel.is_none()
            && recording.is_none_or(|state| state.phase == Phase::Idle)
        {
            let previous = item(Action::PatternViewPrev, "view");
            let next = item(Action::PatternViewNext, "view");
            items.push((format!("{}/{}", previous.0, next.0), "view".into()));
        }
        // in an edit, its own keys come first; A1 follows them (98i)
        let at = if self.edit {
            groups
                .iter()
                .position(|group| group.name == "EDIT")
                .map_or(0, |edit| edit + 1)
        } else {
            0
        };
        groups.insert(
            at,
            tui::HelpGroup {
                name: "PATTERN",
                items: items
                    .into_iter()
                    .filter(|(key, text)| key != "none" || text != "pattern keys")
                    .collect(),
            },
        );
        groups
    }

    /// The pad panel's marks: the edit frame with its page and knobs, the lit parameter.
    #[cfg(test)]
    pub(super) fn mark_pads(&self, v: tui::Pads) -> tui::Pads {
        self.mark_pads_recording(v, self.pattern_record_state())
    }

    pub(super) fn mark_pads_recording(
        &self,
        mut v: tui::Pads,
        recording: Option<engine::capture::Snapshot>,
    ) -> tui::Pads {
        v.feedback = self.pad_feedback();
        self.erase_pad_marks(&mut v, recording);
        if let Some(audio) = &self.audio {
            use std::sync::atomic::Ordering::Acquire;
            let epoch = audio.project.epoch.load(Acquire);
            v.playing = audio.project.ready.load(Acquire) == epoch
                && audio.stats.pads_playing(epoch)
                && audio.project.epoch.load(Acquire) == epoch
                && audio.project.ready.load(Acquire) == epoch;
        }
        if self.edit && self.screen == Screen::Pads {
            let (page, params) = self.pads.page();
            v.edit = Some(page);
            v.pages = pads::page_names();
            if page == "FILTER" {
                v.curve = Some(filter_curve(
                    self.pads.filter_settings(self.pads.selected_abs()),
                ));
            }
            v.knobs = params.iter().map(|&p| tui_param(p)).collect();
        }
        if let Some(Lit::Pad(p)) = self.lit_now() {
            v.lit = Some(tui_param(p));
        }
        // the learn mode labels the grid and the panel with their controls (47, 52)
        self.learn_grid(&mut v.pads);
        self.learn_panel(&mut v);
        v.hidden = self.ui.hide_pads;
        v
    }

    /// The chop screen's marks: the edit name on the slice line and the bound in play — a
    /// held modifier's (Ctrl the start, Alt the end), the picked one in the edit, the one
    /// that just moved.
    #[cfg(test)]
    pub(super) fn mark_chop<'a>(
        &self,
        v: tui::Chop<'a>,
        held: Option<input::Modifier>,
    ) -> tui::Chop<'a> {
        self.mark_chop_recording(v, held, self.pattern_record_state())
    }

    pub(super) fn mark_chop_recording<'a>(
        &self,
        mut v: tui::Chop<'a>,
        held: Option<input::Modifier>,
        recording: Option<engine::capture::Snapshot>,
    ) -> tui::Chop<'a> {
        v.edit = self.edit;
        v.pads.feedback = self.pad_feedback();
        self.erase_pad_marks(&mut v.pads, recording);
        // the plan is checked on the grid: the assign mode shows it even with the pads hidden (32)
        v.pads.hidden = self.ui.hide_pads && self.plan.is_none();
        v.hot = match (held, self.lit_now()) {
            (Some(input::Modifier::Ctrl), _) => Some(false),
            (Some(input::Modifier::Alt), _) => Some(true),
            _ if self.edit => Some(self.slice_end),
            (_, Some(Lit::Bound(end))) => Some(end),
            _ => None,
        };
        v
    }

    pub(super) fn fx_view(&self) -> tui::Fx {
        let profiles = self.live_profiles();
        let row_keys = [0, 1, 2, 4];
        tui::Fx {
            blocks: std::array::from_fn(|k| tui::FxBlock {
                rows: std::array::from_fn(|i| {
                    let p = super::values::fx_row(k, i);
                    (
                        self.pads.name(p),
                        self.pads.fx_text(self.pads.selected_abs(), p),
                    )
                }),
            }),
            selected: self.fx_slot,
            param: self.fx_param,
            pickup: self.pickup.as_ref().map(|p| p.0.clone()),
            learn: if self.screen_learn.is_some() {
                (0..4)
                    .map(|i| {
                        let name = self.learn_params()[row_keys[i]].0;
                        (
                            i,
                            self.learn_shown(&profiles, name)
                                .unwrap_or_else(|| "none".into()),
                        )
                    })
                    .collect()
            } else {
                Vec::new()
            },
            lit: match self.lit_now() {
                _ if self.screen_learn.is_some() => row_keys.iter().position(|&i| {
                    self.learn_picked()
                        .is_some_and(|p| p.0 == self.learn_params()[i].0)
                }),
                Some(Lit::Pad(p)) if p.slot() == Some(self.fx_slot) => {
                    (0..4).find(|&row| super::values::fx_row(self.fx_slot, row) == p)
                }
                _ => None,
            },
        }
    }

    /// What the help line says about the states of this moment.
    pub(super) fn help_state(&self) -> HelpState {
        HelpState {
            hand: match (
                self.browser.in_hand.is_some(),
                self.pad_in_hand.is_some(),
                self.copy_in_hand.is_some(),
            ) {
                (true, _, _) => Some("hit a pad to put it there"),
                (false, true, _) => Some("hit a pad to swap them"),
                // 83a: a copy in hand names its next step instead of the swap hint
                (false, false, true) => Some("hit a pad to copy it there"),
                _ => None,
            },
            preview_on: self.preview_on,
            assign: self.plan.is_some(),
            snap: self.chop.as_ref().map_or(chop::Snap::Off, |c| c.snap),
            tab: self.settings_tab,
            param_name: match (self.screen, self.slice_end) {
                (Screen::Chop, true) => "end",
                (Screen::Chop, false) => "start",
                (Screen::Fx, _) => self.pads.name(self.fx_selected_param()),
                _ => self.pads.param_name(),
            },
            page: match self.screen {
                Screen::Chop => "BOUNDS",
                Screen::Fx => {
                    if self.fx_slot == 0 {
                        "FX 1"
                    } else {
                        "FX 2"
                    }
                }
                _ => self.pads.page().0,
            },
            play: self.chop.as_ref().map_or("to end", |c| c.play.name()),
            looping: self.chop.as_ref().is_some_and(|c| c.looping),
            as_pad: self.chop.as_ref().is_some_and(|c| c.as_pad),
            marking: self.pads.marking,
            scope: self.pads.edit_scope_name(),
            edit: self.edit,
        }
    }

    #[cfg(test)]
    pub(super) fn chop_display<'a>(&self, chop: &'a chop::Chop) -> tui::Chop<'a> {
        self.chop_display_recording(chop, self.pattern_record_state())
    }

    pub(super) fn chop_display_recording<'a>(
        &self,
        chop: &'a chop::Chop,
        recording: Option<engine::capture::Snapshot>,
    ) -> tui::Chop<'a> {
        let preview = self.playhead();
        let mut view = chop_view_routed(
            chop,
            self.plan.as_ref(),
            preview,
            self.rate,
            &self.pads,
            self.pattern_routes_pads_for(recording),
        );
        // Only the line and badge use pad positions. Space and M still use the
        // preview position, so observing a pad cannot change transport or edits.
        if preview.is_none()
            && self.plan.is_none()
            && self.screen == Screen::Chop
            && let Some(audio) = &self.audio
        {
            use std::sync::atomic::Ordering::Acquire;
            let epoch = audio.project.epoch.load(Acquire);
            if audio.project.ready.load(Acquire) == epoch {
                let position = audio.stats.pad_position(&chop.sound, epoch, |pad| {
                    self.pads.file[pad].as_ref() == Some(&chop.file)
                        && self.pads.link[pad].is_some_and(|(_, e)| e == chop.epoch)
                });
                if audio.project.epoch.load(Acquire) == epoch
                    && audio.project.ready.load(Acquire) == epoch
                {
                    if position.is_some() {
                        view.mode.push_str(" ▶");
                    }
                    view.playhead = position
                        .filter(|p| (chop.view.start..chop.view.end).contains(p))
                        .map(|p| {
                            (p - chop.view.start) as f32
                                / (chop.view.end - chop.view.start).max(1) as f32
                        });
                }
            }
        }
        view
    }

    #[cfg(test)]
    pub(super) fn notice(&self) -> Option<String> {
        self.notice_at(Instant::now())
    }

    pub(super) fn notice_at(&self, now: Instant) -> Option<String> {
        if let Some((source, at)) = &self.recording {
            if self.recording_warning()
                && let Some(notice) = self.notices.current_at(now)
            {
                return Some(notice.text.clone());
            }
            let elapsed = now.saturating_duration_since(*at);
            return Some(format!(
                "Recording {} {}.{:01} s… {}",
                if *source == audio::SOURCE_MIC {
                    "the microphone"
                } else {
                    "the output"
                },
                elapsed.as_secs(),
                elapsed.subsec_millis() / 100,
                match (
                    *source == audio::SOURCE_MIC,
                    self.input_sources.last_is_smc()
                ) {
                    // one stop control, the one the mode line names (F35)
                    (true, true) => "release RECORD to stop",
                    (true, false) => "release Alt+R to stop",
                    (false, smc) => self.output_stop_words(smc),
                }
            ));
        }
        if self.audio.as_ref().is_some_and(|a| a.stopped.load(Relaxed)) {
            let status = "Audio stopped: the sound server or device went away.";
            if matches!(self.save_notice, Some(super::saving::SaveNotice::RecoveredUntil(until)) if now < until)
            {
                return Some(format!("Session saved · {status}"));
            }
            return Some(status.into());
        }
        let current = self.notices.current_at(now);
        if current.is_none_or(|notice| notice.level < Level::Warn)
            && self.keymap_warnings.is_empty()
            && !self.input_sources.last_is_smc()
            && !self.key_notice_blocked()
            && let Some(cue) = self.tap_tempo_hint_at(now)
        {
            return Some(cue.into());
        }
        if let Some(notice) = current {
            return Some(notice.text.clone());
        }
        if let Some(path) = &self.chop_loading {
            return Some(format!("Loading {}…", file_name(path)));
        }
        if let Some(abs) = (0..PADS).find(|&p| self.pads.loading[p].is_some()) {
            let name = self.pads.file[abs]
                .as_deref()
                .map(file_name)
                .unwrap_or_default();
            return Some(format!(
                "Loading {name} {}%",
                self.pads.loading[abs].unwrap_or(0)
            ));
        }
        if let Some((what, want, got)) = self.missing() {
            return Some(format!(
                "{what} '{want}' not found, using '{}'",
                got.unwrap_or("none")
            ));
        }
        (!self.key_notice_blocked())
            .then(|| self.keymap_warnings.front().cloned())
            .flatten()
    }

    fn pattern_feedback(&mut self, now: Instant) {
        use std::sync::atomic::Ordering::Acquire;
        let snapshot = self.audio_alive().and_then(|audio| {
            let epoch = audio.project.epoch.load(Acquire);
            if audio.project.ready.load(Acquire) != epoch {
                return None;
            }
            let hits = audio.stats.pattern_hits(epoch)?;
            let dropped = audio.stats.transport(epoch)?.dropped_hits;
            if audio.project.epoch.load(Acquire) != epoch
                || audio.project.ready.load(Acquire) != epoch
            {
                return None;
            }
            Some(((Arc::as_ptr(&audio.stats) as usize, epoch), hits, dropped))
        });
        let Some((identity, hits, dropped)) = snapshot else {
            return;
        };
        if self.transport.feedback != Some(identity) {
            self.transport.feedback = Some(identity);
            self.transport.pattern_hits.fill(0);
            self.transport.dropped_hits = 0;
        }
        for (pad, hit) in hits.into_iter().enumerate() {
            if hit != 0 && hit != self.transport.pattern_hits[pad] {
                self.pads.hit_at[pad] = Some(now);
                self.splash = false;
            }
        }
        self.transport.pattern_hits = hits;
        if dropped > self.transport.dropped_hits {
            self.post_system_notice(
                Level::Warn,
                (
                    format!(
                        "Pattern skipped {} hits: audio resources are busy.",
                        dropped - self.transport.dropped_hits
                    ),
                    now,
                ),
            );
        }
        self.transport.dropped_hits = dropped;
    }

    pub(super) fn frame_notice(&mut self) -> Option<String> {
        self.frame_notice_at(Instant::now())
    }

    pub(super) fn frame_notice_at(&mut self, now: Instant) -> Option<String> {
        self.pattern_feedback(now);
        if self
            .value_popup
            .as_ref()
            .is_some_and(|value| now.saturating_duration_since(value.at) >= LIT_FOR)
        {
            // Release sample snapshots on the UI thread, even without another input.
            self.value_popup = None;
        }
        self.notices.advance(now);
        self.notices.set_visible(!self.notice_hidden(), now);
        let notice = self
            .pad_naming
            .as_ref()
            .map(|(_, name)| format!("Name: {name}▏  Enter saves · Esc cancels"))
            .or_else(|| self.notice_at(now));
        self.keymap_warning_shown = !self.key_notice_blocked()
            && self.keymap_warnings.front().is_some()
            && notice.as_ref() == self.keymap_warnings.front();
        self.frame_notice_level = if self.pad_naming.is_some()
            || (self.recording.is_some() && !self.recording_warning())
        {
            None
        } else if self
            .audio
            .as_ref()
            .is_some_and(|audio| audio.stopped.load(Relaxed))
        {
            Some(Level::Error)
        } else {
            self.notices
                .current()
                .filter(|entry| notice.as_deref() == Some(entry.text.as_str()))
                .map(|entry| entry.level)
                .or_else(|| self.keymap_warning_shown.then_some(Level::Warn))
        };
        self.shown_error = self.notices.error_token();
        notice
    }

    /// The splash view while no sound went (73); `None` draws the wave in the strip.
    pub(super) fn splash_view(&self) -> Option<tui::Splash> {
        self.splash.then(|| tui::Splash {
            version: env!("CARGO_PKG_VERSION"),
            loading: self.splash_line(),
        })
    }

    /// The loading line of the splash (73): what loads now, `ready` when all is loaded,
    /// or a file that did not load.
    fn splash_line(&self) -> String {
        if let Some(pad) = (0..PADS).find(|&p| self.pads.missing[p] && self.pads.file[p].is_some())
        {
            return format!(
                "cannot load {}",
                file_name(self.pads.file[pad].as_deref().unwrap_or(Path::new("")))
            );
        }
        let loading = self.pads.loading.iter().filter(|l| l.is_some()).count();
        if loading > 0 {
            format!(
                "loading the session, {loading} {}",
                if loading == 1 { "sample" } else { "samples" }
            )
        } else {
            "ready".into()
        }
    }
}

pub(super) fn pad_changes(p: &pads::Pads, abs: usize) -> Vec<tui::PadParam> {
    use tui::PadParam as W;
    if p.file[abs].is_none() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut add = |on: bool, w: W| {
        if on {
            out.push(w);
        }
    };
    add(p.volume_db[abs] != 0, W::Volume);
    let shape = p.play_shape(abs);
    add(p.pitch[abs] != 0 || shape.cents != 0, W::Pitch);
    add(shape.cents != 0, W::FineTune);
    add(shape.pan != 0, W::Pan);
    add(shape.hold_ms != 0, W::Hold);
    add(shape.exponential, W::Curve);
    add(
        p.shape[abs].velocity_db.is_some()
            && shape.velocity_db != Some(engine::pad_shape::DEFAULT_VELOCITY_DB),
        W::Velocity,
    );
    add(p.group[abs] != 0, W::Group);
    add(p.attack_ms[abs] != 0, W::Attack);
    add(p.decay_ms[abs] != 0, W::Decay);
    add(p.release_ms[abs] != 0, W::Release);
    // a filter that is off counts for nothing, whatever its cutoff and reso (55)
    let on = p.filter[abs] != engine::filter::FilterKind::Off;
    add(on, W::Filter);
    add(on && p.cutoff_hz[abs] != pads::CUTOFF_DEFAULT, W::Cutoff);
    add(on && p.reso[abs] != 0, W::Reso);
    let part = p.source(abs) != pads::Source::All;
    for w in [W::Source, W::Start, W::End] {
        add(part, w);
    }
    add(p.drive[abs] != 0, W::Drive);
    add(p.sends[abs][0] != 0, W::Send1);
    add(p.sends[abs][1] != 0, W::Send2);
    out
}

/// The mark at the end of a pad's mode line (77): `✱` when the pad has settings the line
/// does not show already: pitch, envelope, filter on, drive, sends. Not the volume, the
/// mode, the group (on the line) or the source (a slice shows in the name).
pub(super) fn pad_glyphs(changed: &[tui::PadParam]) -> String {
    use tui::PadParam as W;
    let more = [
        W::Pitch,
        W::Attack,
        W::Decay,
        W::Release,
        W::Pan,
        W::Hold,
        W::Curve,
        W::Velocity,
        W::Filter,
        W::Cutoff,
        W::Reso,
        W::Drive,
        W::Send1,
        W::Send2,
    ];
    if more.iter().any(|w| changed.contains(w)) {
        "✱".into()
    } else {
        String::new()
    }
}

pub(super) fn pads_view(
    p: &mut pads::Pads,
    pickup: Option<String>,
    in_hand: Option<usize>,
    out_wave: Vec<f32>,
    splash: Option<tui::Splash>,
    rate: u32,
) -> tui::Pads {
    let pads = (0..PADS_PER_BANK)
        .map(|i| {
            let abs = p.bank * PADS_PER_BANK + i;
            let changed = pad_changes(p, abs);
            tui::PadView {
                naming: false,
                empty: p.file[abs].is_none(),
                glyphs: pad_glyphs(&changed),
                pitch_gate: pitch_gate(p, abs),
                changed,
                dim: false,
                dashed: false,
                source: source_word(p.source(abs)),
                bounds: p.sound[abs].as_ref().map_or((0.0, 0.0), |s| {
                    let r = p.region[abs];
                    let secs = |f: usize| f.min(s.len()) as f32 / rate.max(1) as f32;
                    (secs(r.start), secs(r.end))
                }),
                wave: p.wave(abs, rate, BIG_COLS),
                key: PAD_KEYS[i],
                name: p.file[abs]
                    .as_ref()
                    .map_or("—".into(), |_| pad_name(p, abs)),
                detail: Some(match p.file[abs].as_deref() {
                    None => format!("empty · test tone {}", test_tone_name(abs)),
                    Some(f) => format!("{} · {}", file_name(f), source_word(p.source(abs))),
                }),
                missing: p.missing[abs],
                loading: p.loading[abs],
                in_hand: in_hand == Some(abs),
                marked: p.marked.contains(&abs),
                note: None,
                gate: p.gate[abs],
                volume_db: p.volume_db[abs] as f32,
                pitch: p.pitch[abs],
                shape: pad_shape_view(p, abs),
                attack_ms: p.attack_ms[abs],
                decay_ms: p.decay_ms[abs],
                release_ms: p.release_ms[abs],
                group: p.group[abs],
                filter: [
                    pads::kind_name(p.filter[abs]).into(),
                    pads::cutoff_text(p.cutoff_hz[abs]),
                    format!("{} %", p.reso[abs]),
                ],
                flash: p.held[abs] || p.hit_at[abs].is_some_and(|t| t.elapsed() < HIT_FLASH),
            }
        })
        .collect();
    // the FX block of the selected pad (56): each send names the kind of its slot
    let sel = p.bank * PADS_PER_BANK + p.selected;
    let fx = [pads::Param::Drive, pads::Param::Send1, pads::Param::Send2]
        .into_iter()
        .map(|w| {
            let mut value = p.fx_text(sel, w);
            if let Some(k) = [pads::Param::Send1, pads::Param::Send2]
                .iter()
                .position(|s| *s == w)
            {
                value = format!("{value}  {}", pads::kind_word(p.fx[k].kind));
            }
            (tui_param(w), label8(p.name(w)), value)
        })
        .collect();
    tui::Pads {
        feedback: tui::PadFeedback::default(),
        source: None,
        pattern: None,
        overview: None,
        playing: false,
        fx,
        bank: p.bank,
        pads,
        selected: p.selected,
        pickup,
        param: tui_param(p.param),
        scope: p.scope_name(),
        bank_fill: bank_fill(p),
        bank_marks: bank_marks(p),
        bank_pattern: tui::BankPattern::default(),
        out_wave,
        splash,
        big: big_view(p, rate),
        curve: None,
        learn: Vec::new(),
        edit: None,
        pages: Vec::new(),
        knobs: Vec::new(),
        lit: None,
        hidden: false,
        edit_key: String::new(),
        slot: None,
    }
}

/// The piece of the selected pad with the pad's envelope over it, or nothing when the
/// pad is empty.
pub(super) fn big_view(p: &pads::Pads, rate: u32) -> Option<tui::BigView> {
    let abs = p.bank * PADS_PER_BANK + p.selected;
    // an empty pad plays its test tone: the edit shows that piece (98h)
    let tone;
    let (sound, region) = match p.sound[abs].as_deref() {
        Some(sound) => (sound, p.region[abs]),
        None => {
            tone = Sample::mono(engine::test_tone(rate, abs));
            (&tone, Region::WHOLE)
        }
    };
    let envelope = p.display_envelope(abs);
    let (wave, env, dur_ms) = pads::piece(sound, region, rate, envelope, BIG_COLS);
    let share = |ms: u16| f32::from(ms) / dur_ms.max(1.0);
    Some(tui::BigView {
        wave,
        env,
        attack: share(p.attack_ms[abs]).min(1.0),
        decay: share(p.decay_ms[abs]).min(1.0),
        release: share(p.release_ms[abs]),
        timed: envelope
            .timed()
            .then_some((share(envelope.hold_ms), envelope.exponential)),
    })
}

/// The pad's envelope as a multiplier over the piece, one value per column: attack
/// fades in over the first `attack_ms`, decay dies into silence over the last
/// `decay_ms` — the same ramps the audio voice follows.
pub(super) fn env_curve(dur_ms: f32, attack_ms: u16, decay_ms: u16, cols: usize) -> Vec<f32> {
    (0..cols)
        .map(|c| {
            let t = (c as f32 + 0.5) / cols.max(1) as f32;
            let at = if attack_ms == 0 {
                1.0
            } else {
                (t * dur_ms / f32::from(attack_ms)).min(1.0)
            };
            let de = if decay_ms == 0 {
                1.0
            } else {
                ((1.0 - t) * dur_ms / f32::from(decay_ms)).min(1.0)
            };
            at * de
        })
        .collect()
}

pub(super) fn tui_param(p: pads::Param) -> tui::PadParam {
    match p {
        pads::Param::Volume => tui::PadParam::Volume,
        pads::Param::Pitch => tui::PadParam::Pitch,
        pads::Param::Group => tui::PadParam::Group,
        pads::Param::Attack => tui::PadParam::Attack,
        pads::Param::Decay => tui::PadParam::Decay,
        pads::Param::Release => tui::PadParam::Release,
        pads::Param::Source => tui::PadParam::Source,
        pads::Param::Start => tui::PadParam::Start,
        pads::Param::End => tui::PadParam::End,
        pads::Param::Filter => tui::PadParam::Filter,
        pads::Param::Cutoff => tui::PadParam::Cutoff,
        pads::Param::Reso => tui::PadParam::Reso,
        pads::Param::Drive => tui::PadParam::Drive,
        pads::Param::Send1 => tui::PadParam::Send1,
        pads::Param::Send2 => tui::PadParam::Send2,
        pads::Param::Fx1Kind => tui::PadParam::Fx1Kind,
        pads::Param::Fx1A => tui::PadParam::Fx1A,
        pads::Param::Fx1B => tui::PadParam::Fx1B,
        pads::Param::Fx2Kind => tui::PadParam::Fx2Kind,
        pads::Param::Fx2A => tui::PadParam::Fx2A,
        pads::Param::Fx2B => tui::PadParam::Fx2B,
        pads::Param::Fx1Return => tui::PadParam::Fx1Return,
        pads::Param::Fx2Return => tui::PadParam::Fx2Return,
        pads::Param::Pan => tui::PadParam::Pan,
        pads::Param::FineTune => tui::PadParam::FineTune,
        pads::Param::Velocity => tui::PadParam::Velocity,
        pads::Param::Curve => tui::PadParam::Curve,
        pads::Param::Hold => tui::PadParam::Hold,
    }
}

pub(super) fn pad_shape_view(pads: &pads::Pads, abs: usize) -> tui::PadShapeView {
    let shape = pads.play_shape(abs);
    tui::PadShapeView {
        pan: shape.pan,
        cents: shape.cents,
        hold_ms: shape.hold_ms,
        exponential: shape.exponential,
        velocity_db: shape.velocity_db,
    }
}

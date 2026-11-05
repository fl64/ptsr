//! Screen data prepared from application state.

use std::path::Path;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

use engine::{PADS, PADS_PER_BANK};
use input::Screen;

use super::{
    App, EFFECT_PARAMS, FLASH, HelpState, LIT_FOR, Lit, audio, chop, chop_view, file_name,
    filter_curve, pads, same_whole_sound, tui_param,
};

impl App {
    /// Existing modes in title priority order. This snapshot never changes input focus.
    pub(super) fn title_badges(&self) -> Vec<tui::Badge> {
        self.title_badges_at(Instant::now())
    }

    pub(super) fn title_badges_at(&self, now: Instant) -> Vec<tui::Badge> {
        use tui::{
            Badge,
            BadgeKind::{Attention, On, PadMode},
        };

        let mode = self.focus().mode;
        let mut pad_modes = Vec::new();
        if mode.screen_learn || mode.midi_learn {
            pad_modes.push(Badge::new(PadMode, "LEARN", "LEARN"));
        }
        if self.screen == Screen::Chop
            && let (Some(plan), Some(chop)) = (&self.plan, &self.chop)
        {
            let total = chop.slices().len();
            let placed = plan
                .targets(total)
                .into_iter()
                .map(|(slice, _)| slice)
                .collect::<std::collections::BTreeSet<_>>()
                .len();
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
        let count = self.pads.marked.len();
        if mode.marking {
            pad_modes.push(Badge::new(
                PadMode,
                format!("MARK · hit pads · {count} marked"),
                format!("MARK {count}"),
            ));
        } else if count > 0 {
            pad_modes.push(Badge::new(
                PadMode,
                format!("{count} MARKED · Esc drops"),
                format!("{count} MARKED"),
            ));
        }
        let mut badges = Vec::new();
        let mut pad_modes = pad_modes.into_iter();
        badges.extend(pad_modes.next());
        if mode.question {
            badges.push(Badge::new(Attention, "ASK", "ASK"));
        }
        if let Some((_, start)) = self.recording {
            let seconds = now.saturating_duration_since(start).as_secs();
            badges.push(Badge::new(Attention, format!("● REC {seconds}s"), "REC"));
        }
        badges.extend(pad_modes);
        if mode.edit {
            let page = self.help_state().page;
            badges.push(Badge::new(
                On,
                format!("EDIT · {page}"),
                format!("EDIT {page}"),
            ));
        }
        if self.screen == Screen::Chop
            && let Some(chop) = &self.chop
        {
            if chop.looping {
                badges.push(Badge::new(On, "LOOP", "LOOP"));
            }
            if chop.as_pad {
                badges.push(Badge::new(On, "AS PAD", "AS PAD"));
            }
        }
        badges
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

    /// The pad panel's marks: the edit frame with its page and knobs, the lit parameter.
    pub(super) fn mark_pads(&self, mut v: tui::Pads) -> tui::Pads {
        v.feedback = self.pad_feedback();
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
    pub(super) fn mark_chop<'a>(
        &self,
        mut v: tui::Chop<'a>,
        held: Option<input::Modifier>,
    ) -> tui::Chop<'a> {
        v.edit = self.edit;
        v.pads.feedback = self.pad_feedback();
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
        tui::Fx {
            blocks: std::array::from_fn(|k| tui::FxBlock {
                rows: std::array::from_fn(|i| {
                    let p = EFFECT_PARAMS[3 + k * 3 + i];
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
                (0..3)
                    .map(|i| {
                        let name = self.learn_params()[i].0;
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
                _ if self.screen_learn.is_some() => self.learn_params()[..3]
                    .iter()
                    .position(|(name, _)| self.learn_picked().is_some_and(|p| p.0 == *name)),
                Some(Lit::Pad(p)) if p.slot() == Some(self.fx_slot) => EFFECT_PARAMS
                    [3 + self.fx_slot * 3..6 + self.fx_slot * 3]
                    .iter()
                    .position(|&v| v == p),
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
            scope: self.pads.scope_name(),
            edit: self.edit,
        }
    }

    pub(super) fn chop_display<'a>(&self, chop: &'a chop::Chop) -> tui::Chop<'a> {
        let preview = self.playhead();
        let mut view = chop_view(chop, self.plan.as_ref(), preview, self.rate, &self.pads);
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

    pub(super) fn notice(&self) -> Option<String> {
        self.notice_at(Instant::now())
    }

    pub(super) fn notice_at(&self, now: Instant) -> Option<String> {
        if let Some((source, at)) = &self.recording {
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
                if *source == audio::SOURCE_MIC {
                    "release Alt+R to stop"
                } else {
                    "Ctrl+T to stop"
                }
            ));
        }
        if self.audio.as_ref().is_some_and(|a| a.stopped.load(Relaxed)) {
            return Some("Audio stopped: the sound server or device went away.".into());
        }
        if let Some((msg, at)) = &self.flash
            && now.saturating_duration_since(*at) < FLASH
        {
            return Some(msg.clone());
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
        self.startup.first().cloned().or_else(|| {
            (!self.key_notice_blocked())
                .then(|| self.keymap_warnings.front().cloned())
                .flatten()
        })
    }

    pub(super) fn frame_notice(&mut self) -> Option<String> {
        let notice = self
            .pad_naming
            .as_ref()
            .map(|(_, name)| format!("Name: {name}▏  Enter saves · Esc cancels"))
            .or_else(|| self.notice());
        self.keymap_warning_shown = !self.key_notice_blocked()
            && self.keymap_warnings.front().is_some()
            && notice.as_ref() == self.keymap_warnings.front();
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

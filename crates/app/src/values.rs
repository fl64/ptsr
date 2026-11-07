//! Applied-value feedback. Observe production setters without another input route.

use super::{
    Action, App, EFFECT_PARAMS, Instant, MidiListRow, Param, Screen, bank_key, file_name, midi,
    pads, source_word,
};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone)]
enum Target {
    Master,
    Mode {
        abs: usize,
        scope: Vec<usize>,
    },
    Pad {
        abs: usize,
        param: pads::Param,
        scope: Vec<usize>,
    },
    Chop {
        param: ChopValue,
        slice: usize,
    },
    Settings(usize),
    Response,
}

#[derive(Clone, Copy)]
enum ChopValue {
    Start,
    End,
    Cursor,
    Zoom,
    RegionStart,
    RegionEnd,
    Region,
    Threshold,
    Parts,
    Snap,
    Play,
}

pub(super) struct Capture {
    target: Target,
    view: tui::Value,
    pads: Vec<(usize, pads::PadSnap)>,
    numbers: Vec<f64>,
    generations: Vec<u64>,
    file: Option<PathBuf>,
    sound: Option<Arc<engine::Sample>>,
}

impl Capture {
    fn same(&self, other: &Self) -> bool {
        let sound = match (&self.sound, &other.sound) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        sound
            && self.file == other.file
            && self.generations == other.generations
            && self.numbers == other.numbers
            && self.pads == other.pads
            && self.view == other.view
    }
}

pub(super) struct Shown {
    capture: Capture,
    pub(super) at: Instant,
}

impl Shown {
    pub(super) fn current(&self, app: &App) -> Option<&tui::Value> {
        let actual = app.capture_value(self.capture.target.clone())?;
        self.capture.same(&actual).then_some(&self.capture.view)
    }
}

impl App {
    fn pad_value_target(&self, param: pads::Param) -> Target {
        let scope = if param.slot().is_some()
            || matches!(
                param,
                pads::Param::Source | pads::Param::Start | pads::Param::End
            ) {
            vec![self.pads.selected_abs()]
        } else {
            self.pads.scope_pads()
        };
        let selected = self.pads.selected_abs();
        let abs = if scope.contains(&selected) {
            selected
        } else {
            scope[0]
        };
        Target::Pad { abs, param, scope }
    }

    pub(super) fn value_before_action(&self, action: Action) -> Option<Capture> {
        use ChopValue as C;
        if self.ask.is_some()
            || self.path_picker.is_some()
            || (self.screen == Screen::Chop && self.plan.is_some())
        {
            return None;
        }
        let chop = |param| Target::Chop {
            param,
            slice: self.chop.as_ref().map_or(0, |c| c.selected),
        };
        let target = match action {
            Action::MasterVolume(_) => Target::Master,
            Action::PadModeNext => Target::Mode {
                abs: self.pads.selected_abs(),
                scope: self.pads.scope_pads(),
            },
            Action::ChopSnapToggle if self.screen == Screen::Chop => chop(C::Snap),
            Action::ChopPlayMode if self.screen == Screen::Chop => chop(C::Play),
            Action::PadParam(_) | Action::EditValue { .. } if self.screen == Screen::Fx => {
                self.pad_value_target(self.fx_selected_param())
            }
            Action::PadParam(_) | Action::EditValue { .. } if self.screen == Screen::Pads => {
                self.pad_value_target(self.pads.param)
            }
            Action::EditValue { .. } if self.screen == Screen::Chop => {
                chop(if self.slice_end { C::End } else { C::Start })
            }
            Action::SliceBound { end, .. } if self.screen == Screen::Chop => {
                chop(if end { C::End } else { C::Start })
            }
            Action::MarkerNudge { .. } if self.screen == Screen::Chop => chop(C::Start),
            Action::CursorMove { .. } if self.screen == Screen::Chop => chop(C::Cursor),
            Action::ChopZoom { .. } if self.screen == Screen::Chop => chop(C::Zoom),
            Action::ChopRegionStart if self.screen == Screen::Chop => chop(C::RegionStart),
            Action::ChopRegionEnd if self.screen == Screen::Chop => chop(C::RegionEnd),
            Action::ChopRegionClear if self.screen == Screen::Chop => chop(C::Region),
            Action::ChopNormalize if self.screen == Screen::Chop => {
                let abs = self.chop_from?;
                Target::Pad {
                    abs,
                    param: pads::Param::Volume,
                    scope: vec![abs],
                }
            }
            Action::PadParam(_) => self.pad_value_target(self.pads.param),
            Action::ChopHits | Action::ChopHitsBack if self.screen == Screen::Chop => {
                chop(C::Threshold)
            }
            Action::ChopParts | Action::ChopEqualParts if self.screen == Screen::Chop => {
                chop(C::Parts)
            }
            Action::SettingsValue(_) | Action::SettingsLearn
                if self.screen == Screen::Settings
                    && self.settings_tab == 2
                    && matches!(
                        self.midi_list().get(self.midi_row),
                        Some(MidiListRow::Response)
                    ) =>
            {
                Target::Response
            }
            Action::SettingsValue(_)
                if self.screen == Screen::Settings && self.settings_tab == 0 =>
            {
                Target::Settings(self.settings_row)
            }
            _ => return None,
        };
        self.capture_value(target)
    }

    pub(super) fn value_before_midi(&self, event: &midi::Event) -> Option<Capture> {
        use ChopValue as C;
        if self.screen_learn.is_some() {
            return None;
        }
        let midi::Event::Param { param, .. } = event else {
            return None;
        };
        let chop = |param| Target::Chop {
            param,
            slice: self.chop.as_ref().map_or(0, |c| c.selected),
        };
        let target = match *param {
            Param::MasterVolume => Target::Master,
            Param::PadVolume => self.pad_value_target(pads::Param::Volume),
            Param::PadPitch => self.pad_value_target(pads::Param::Pitch),
            Param::PadAttack | Param::PadDecay | Param::PadRelease => {
                self.pad_value_target(super::pad_env(*param))
            }
            Param::PadFilter | Param::PadCutoff | Param::PadReso => {
                self.pad_value_target(super::pad_filter(*param))
            }
            Param::PadSource => self.pad_value_target(pads::Param::Source),
            Param::PadStart => self.pad_value_target(pads::Param::Start),
            Param::PadEnd => self.pad_value_target(pads::Param::End),
            Param::PadEffect(i) => self.pad_value_target(*EFFECT_PARAMS.get(i as usize)?),
            Param::SliceStart => chop(C::Start),
            Param::SliceEnd => chop(C::End),
            Param::Cursor => chop(C::Cursor),
            Param::Zoom => chop(C::Zoom),
            Param::RegionStart => chop(C::RegionStart),
            Param::RegionEnd => chop(C::RegionEnd),
            Param::Threshold => chop(C::Threshold),
            // Slot resolution calls production dispatch again with the concrete parameter.
            Param::Slot(_) | Param::Page => return None,
        };
        self.capture_value(target)
    }

    pub(super) fn value_after(&mut self, before: Option<Capture>, at: Instant) {
        let Some(before) = before else {
            return;
        };
        let Some(mut after) = self.capture_value(before.target.clone()) else {
            return;
        };
        let changed = match &before.target {
            Target::Chop { .. } => before.numbers.last() != after.numbers.last(),
            _ => !before.same(&after),
        };
        if !changed {
            return;
        }
        let pad_target = match &before.target {
            Target::Pad { abs, param, scope } if param.slot().is_none() => {
                Some((*abs, scope, Some(*param)))
            }
            Target::Mode { abs, scope } => Some((*abs, scope, None)),
            _ => None,
        };
        if let Some((abs, scope, param)) = pad_target {
            // A marked pad can change while the selected pad remains at its limit.
            // Name a pad that changed, rather than claim that the whole scope agrees.
            let changed: Vec<_> = before
                .pads
                .iter()
                .zip(&after.pads)
                .filter_map(|((a, old), (_, new))| (old != new).then_some(*a))
                .collect();
            let representative = if changed.contains(&abs) {
                abs
            } else {
                *changed.first().unwrap_or(&abs)
            };
            let target = match param {
                Some(param) => Target::Pad {
                    abs: representative,
                    param,
                    scope: scope.clone(),
                },
                None => Target::Mode {
                    abs: representative,
                    scope: scope.clone(),
                },
            };
            if let Some(actual) = self.capture_value(target) {
                after = actual;
            }
        }
        self.value_popup = Some(Shown { capture: after, at });
    }

    fn capture_value(&self, target: Target) -> Option<Capture> {
        use pads::Param as P;
        let mut c = Capture {
            target: target.clone(),
            view: tui::Value {
                object: String::new(),
                parameter: String::new(),
                formatted: String::new(),
                position: None,
            },
            pads: Vec::new(),
            numbers: Vec::new(),
            generations: Vec::new(),
            file: None,
            sound: None,
        };
        match target {
            Target::Master => {
                c.view.object = "Master".into();
                c.view.parameter = "volume".into();
                c.view.formatted = format!("{:+} dB", self.master_db);
                c.view.position = Some(
                    (self.master_db - pads::MIN_DB) as f32 / (pads::MAX_DB - pads::MIN_DB) as f32,
                );
                c.numbers.push(self.master_db as f64);
            }
            Target::Mode { abs, scope } => {
                c.view.object = format!("pad {}", bank_key(abs));
                c.view.parameter = "mode".into();
                c.view.formatted = if self.pads.gate[abs] {
                    "gate"
                } else {
                    "one-shot"
                }
                .into();
                c.pads = scope.iter().map(|&a| (a, self.pads.snapshot(a))).collect();
                c.generations = scope.iter().map(|&a| self.load_gen[a]).collect();
            }
            Target::Pad {
                abs,
                param: p,
                scope,
            } => {
                c.view.parameter = self.pads.name(p).into();
                if let Some(slot) = p.slot() {
                    let f = self.pads.fx[slot];
                    c.view.object = format!("FX {}", slot + 1);
                    c.view.formatted = self.pads.fx_text(abs, p);
                    c.view.position =
                        (!matches!(p, P::Fx1Kind | P::Fx2Kind)).then(|| self.pads.fx_pos(p));
                    c.numbers = vec![
                        f.kind as u8 as f64,
                        f.size as f64,
                        f.tone as f64,
                        f.time_ms as f64,
                        f.feedback as f64,
                    ];
                } else {
                    c.view.object = format!("pad {}", bank_key(abs));
                    c.pads = scope.iter().map(|&a| (a, self.pads.snapshot(a))).collect();
                    c.generations = scope.iter().map(|&a| self.load_gen[a]).collect();
                    let (text, position) = match p {
                        P::Volume => (
                            format!("{:+} dB", self.pads.volume_db[abs]),
                            Some(
                                (self.pads.volume_db[abs] - pads::MIN_DB) as f32
                                    / (pads::MAX_DB - pads::MIN_DB) as f32,
                            ),
                        ),
                        P::Pitch => (
                            format!("{:+} st", self.pads.pitch[abs]),
                            Some(
                                (self.pads.pitch[abs] + engine::PITCH_LIMIT) as f32
                                    / (2 * engine::PITCH_LIMIT) as f32,
                            ),
                        ),
                        P::Attack | P::Decay | P::Release => {
                            let ms = match p {
                                P::Attack => self.pads.attack_ms[abs],
                                P::Decay => self.pads.decay_ms[abs],
                                _ => self.pads.release_ms[abs],
                            };
                            (
                                format!("{ms} ms"),
                                Some(ms as f32 / pads::ENV_MS_LIMIT as f32),
                            )
                        }
                        P::Filter => (pads::kind_name(self.pads.filter[abs]).into(), None),
                        P::Cutoff => {
                            use engine::filter::{CUTOFF_MAX, CUTOFF_MIN};
                            (
                                pads::cutoff_text(self.pads.cutoff_hz[abs]),
                                Some(
                                    (self.pads.cutoff_hz[abs] / CUTOFF_MIN).ln()
                                        / (CUTOFF_MAX / CUTOFF_MIN).ln(),
                                ),
                            )
                        }
                        P::Reso => (
                            format!("{} %", self.pads.reso[abs]),
                            Some(self.pads.reso[abs] as f32 / 100.0),
                        ),
                        P::Source => (source_word(self.pads.source(abs)), None),
                        P::Start | P::End => {
                            let (start, end, len) = self.pad_bounds(abs)?;
                            let at = if p == P::End { end } else { start };
                            (
                                format!("{:.3} s", at as f64 / self.rate.max(1) as f64),
                                Some(at as f32 / len as f32),
                            )
                        }
                        P::Group => (
                            self.pads.fx_text(abs, p),
                            Some(self.pads.group[abs] as f32 / pads::GROUPS as f32),
                        ),
                        P::Drive => (
                            self.pads.fx_text(abs, p),
                            Some(self.pads.drive[abs] as f32 / 100.0),
                        ),
                        P::Send1 | P::Send2 => (
                            self.pads.fx_text(abs, p),
                            Some(self.pads.sends[abs][usize::from(p == P::Send2)] as f32 / 100.0),
                        ),
                        _ => return None,
                    };
                    c.view.formatted = text;
                    c.view.position = position;
                }
            }
            Target::Chop { param, slice } => {
                use ChopValue as C;
                let chop = self.chop.as_ref()?;
                c.file = Some(chop.file.clone());
                c.sound = Some(chop.sound.clone());
                c.generations.push(u64::from(chop.epoch));
                c.view.object = format!("Chop {}", file_name(&chop.file));
                let (name, value, position) = match param {
                    C::Start | C::End => {
                        let r = *chop.slices().get(slice)?;
                        c.generations.push(u64::from(chop.start_id(slice)));
                        c.view.object = format!("slice {}", slice + 1);
                        let at = if matches!(param, C::End) {
                            r.end
                        } else {
                            r.start
                        };
                        let name = if matches!(param, C::End) {
                            "end"
                        } else {
                            "start"
                        };
                        (name, at as f64, at as f32 / chop.sound.len().max(1) as f32)
                    }
                    C::Cursor => ("cursor", chop.cursor as f64, chop.cursor_pos()),
                    C::Zoom => (
                        "view",
                        (chop.view.end - chop.view.start) as f64,
                        chop.zoom_pos(),
                    ),
                    C::RegionStart => (
                        "region start",
                        chop.range.start as f64,
                        chop.region_edge_pos(false),
                    ),
                    C::RegionEnd => (
                        "region end",
                        chop.range.end as f64,
                        chop.region_edge_pos(true),
                    ),
                    C::Region => (
                        "region",
                        (chop.range.end - chop.range.start) as f64,
                        (chop.range.end - chop.range.start) as f32 / chop.sound.len().max(1) as f32,
                    ),
                    C::Snap => ("snap", chop.snap as u8 as f64, 0.0),
                    C::Play => ("play", chop.play as u8 as f64, 0.0),
                    C::Threshold => ("threshold", chop.threshold_db as f64, chop.threshold_pos()),
                    C::Parts => (
                        "equal parts",
                        chop.parts.unwrap_or(0) as f64,
                        chop.parts.unwrap_or(0) as f32 / 32.0,
                    ),
                };
                c.numbers.push(value);
                c.view.parameter = name.into();
                c.view.formatted = match param {
                    C::Snap => chop.snap.name().into(),
                    C::Play => chop.play.name().into(),
                    C::Threshold => format!("{value:.0} dB"),
                    C::Parts => format!("{value:.0}"),
                    C::Region => format!(
                        "{:.3}..{:.3} s",
                        chop.range.start as f64 / self.rate.max(1) as f64,
                        chop.range.end as f64 / self.rate.max(1) as f64
                    ),
                    _ => format!("{:.3} s", value / self.rate.max(1) as f64),
                };
                c.view.position = (!matches!(param, C::Snap | C::Play)).then_some(position);
            }
            Target::Settings(row) => {
                c.view.object = "Settings".into();
                let (name, value) = match row {
                    0 => (
                        "output",
                        self.routed.output.clone().unwrap_or_else(|| "none".into()),
                    ),
                    1 => (
                        "input",
                        self.routed.input.clone().unwrap_or_else(|| "none".into()),
                    ),
                    2 => (
                        "buffer choice",
                        if self.cfg.buffer_size == 0 {
                            "auto".into()
                        } else {
                            format!("{} frames", self.cfg.buffer_size)
                        },
                    ),
                    3 => (
                        "wave",
                        format!(
                            "{} (next start)",
                            if self.ui.graphics {
                                "picture"
                            } else {
                                "Braille"
                            }
                        ),
                    ),
                    _ => return None,
                };
                c.view.parameter = name.into();
                c.view.formatted = value;
                if row == 2 && self.cfg.buffer_size > 0 {
                    let min = super::BUFFER_SIZES[0];
                    let max = super::BUFFER_SIZES[super::BUFFER_SIZES.len() - 1];
                    c.view.position = Some(
                        ((self.cfg.buffer_size as f32 - min as f32) / (max - min) as f32)
                            .clamp(0.0, 1.0),
                    );
                }
            }
            Target::Response => {
                c.view.object = "Settings".into();
                c.view.parameter = "knob response".into();
                c.view.formatted = self.takeover.response.name().into();
            }
        }
        Some(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_choice_ranges_do_not_claim_an_actual_server_size() {
        let mut app = crate::tests::test_app();
        for size in std::iter::once(0).chain(crate::BUFFER_SIZES) {
            app.cfg.buffer_size = size;
            let capture = app.capture_value(Target::Settings(2)).unwrap();
            assert_eq!(capture.view.parameter, "buffer choice");
            assert_eq!(
                capture.view.formatted,
                if size == 0 {
                    "auto".into()
                } else {
                    format!("{size} frames")
                }
            );
            assert_eq!(capture.view.position.is_some(), size > 0);
            if size == 64 {
                assert_eq!(capture.view.position, Some(0.0));
            }
            if size == 1024 {
                assert_eq!(capture.view.position, Some(1.0));
            }
        }
        app.cfg.output = Some("missing choice".into());
        app.routed.output = Some("actual output".into());
        assert_eq!(
            app.capture_value(Target::Settings(0))
                .unwrap()
                .view
                .formatted,
            "actual output"
        );
    }
}

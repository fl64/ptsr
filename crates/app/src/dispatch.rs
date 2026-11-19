use super::{
    Action, App, Ask, CLICK_VELOCITY, Command, EFFECT_PARAMS, HeldPads, INPUT_LOG, Instant, Keymap,
    Level, Lit, MidiListRow, Move, NUDGE_FINE_MS, NUDGE_MS, Origin, PADS, PADS_PER_BANK, Param,
    PathPicker, ProjectPicker, SETTINGS_ROWS, Screen, UiPadHold, action_info, assign_deaf,
    bank_key, chop, config, event, group_jump, held_repeats, learn, midi, pad_env, pad_filter,
    pad_name, pads,
};

pub(super) fn smc_live_pad_param(
    port: &str,
    control: &str,
    target: input::midi::StepTarget,
) -> Option<pads::Param> {
    use input::midi::StepTarget;
    if !port.contains("SMC-PAD-Master")
        || !matches!(
            control,
            "f1" | "f2"
                | "f3"
                | "f4"
                | "f5"
                | "f6"
                | "cc 38"
                | "cc 39"
                | "cc 40"
                | "cc 41"
                | "cc 42"
                | "cc 43"
        )
    {
        return None;
    }
    match target {
        StepTarget::Param(Param::PadVolume) => Some(pads::Param::Volume),
        StepTarget::Param(Param::PadCutoff) => Some(pads::Param::Cutoff),
        StepTarget::Param(Param::PadPitch) => Some(pads::Param::Pitch),
        StepTarget::Param(Param::PadDecay) => Some(pads::Param::Decay),
        StepTarget::Param(Param::PadEffect(1)) => Some(pads::Param::Send1),
        StepTarget::Param(Param::PadEffect(2)) => Some(pads::Param::Send2),
        _ => None,
    }
}

impl App {
    /// Runs an action; returns `true` to quit.
    pub(super) fn apply(&mut self, action: Action) -> bool {
        super::diag::input(|| format!("action {action:?}"));
        let Some(action) = self.pattern_controls_own(action) else {
            return false;
        };
        let value_before = self.value_before_action(action);
        let was = self.screen;
        let cuts = self.chop.as_ref().map(|c| c.saved());
        self.note_done(action);
        // a change of the pads made on the pads or files screen is a step of the undo (61)
        let undoable = matches!(self.screen, Screen::Pads | Screen::Files | Screen::Fx)
            && !matches!(action, Action::Undo | Action::PatternUndo)
            && !(action == Action::Confirm && self.project_picker.is_some());
        let what = self.step_name(action);
        let before = undoable.then(|| self.before());
        let quit = self.run(action);
        if let Some(b) = before {
            self.note_step(b, what);
        }
        self.follow_loop();
        let mut relinked = 0;
        if action == Action::Undo && self.screen == Screen::Chop && cuts.is_some() {
            relinked = self.relink_after_undo();
        }
        self.cuts_changed(cuts);
        if matches!(self.screen, Screen::Chop) || was == Screen::Chop {
            // pads linked back by the undo changed too, even when their bounds did not move
            let changed = self.sync_links() + relinked;
            self.say_changed(changed, action == Action::Undo);
        }
        // the preview plays only while the files or the chop screen is open
        if was != self.screen && matches!(was, Screen::Files | Screen::Chop) {
            self.stop_preview();
        }
        if was != self.screen && (was == Screen::Settings || self.screen == Screen::Settings) {
            self.reroute();
        }
        // the chop screen opens on silence: sounding pads fade out without a click (24)
        if was != self.screen && self.screen == Screen::Chop && !self.pattern_owns_pads() {
            self.send(Command::HushPads);
        }
        if self.screen != Screen::Chop {
            self.plan = None;
            if matches!(self.ask, Some(Ask::Plan { .. })) {
                self.ask = None;
            }
        }
        if was != Screen::Files && self.screen == Screen::Files {
            self.preview_selection();
        }
        self.value_after(value_before, Instant::now());
        self.sync_panel_target();
        if !self.transport.taps.is_empty() && !self.pattern_tempo_available() {
            self.transport.taps.clear();
        }
        quit
    }

    fn run(&mut self, action: Action) -> bool {
        // the folder picker is open: the keys go to it before the screen (11)
        if self.ask.is_none() && self.path_picker.is_some() {
            let action = match action {
                Action::SmcList { next } => Action::SettingsValue(if next { 1 } else { -1 }),
                action => action,
            };
            self.picker_key(action);
            return false;
        }
        if self.ask.is_none() && action == Action::Confirm && self.project_picker.is_some() {
            self.project_picker_confirm();
            return false;
        }
        if matches!(action, Action::PlayContext | Action::StopContext) {
            self.context_button(action);
            return false;
        }
        if self.ask.is_none()
            && self.pattern_owns_pads()
            && (matches!(
                action,
                Action::PadTake
                    | Action::PadCopy
                    | Action::PadMarkToggle
                    | Action::PadMarkBank
                    | Action::ChopAssign
                    | Action::ChopCommit
            ) || (matches!(action, Action::BrowserHold | Action::BrowserTakeOnly)
                && self.screen == Screen::Files
                && self.browser.selected_file().is_some()))
        {
            self.post_notice(
                Level::Warn,
                (
                    "A1 recording owns the pads: finish the pass before placement or marking."
                        .into(),
                    Instant::now(),
                ),
            );
            return false;
        }
        match action {
            Action::Quit => {
                self.request_quit();
                return self.try_finish_exit();
            }
            // a question waits for Enter (the key of the step itself) or Esc only
            a if self.ask.is_some() => match a {
                // the settings Enter answers the reset question (59)
                Action::SettingsLearn | Action::Confirm
                    if matches!(&self.ask, Some(Ask::ResetProfile { .. })) =>
                {
                    if let Some(Ask::ResetProfile { name, port, .. }) = self.ask.take() {
                        self.seed_device(&port, &name, true);
                        self.post_notice(
                            Level::Done,
                            (format!("{name}: bindings from its profile"), Instant::now()),
                        );
                    }
                }
                Action::Confirm if matches!(&self.ask, Some(Ask::SmcLayout(_))) => {
                    if let Some(Ask::SmcLayout(review)) = self.ask.take() {
                        self.apply_smc_layout(review);
                    }
                }
                Action::Confirm if matches!(self.ask, Some(Ask::ExitRecovery { .. })) => {
                    self.request_manual_save();
                }
                Action::Confirm
                | Action::ChopCommit
                | Action::ChopEdit
                | Action::BrowserHold
                | Action::PadEdit => {
                    match self.ask.as_ref() {
                        Some(Ask::Exit { .. }) => {
                            self.ask = None;
                            self.request_quit();
                        }
                        Some(Ask::Plan { .. }) => self.apply_plan(),
                        Some(
                            Ask::ResetProfile { .. } | Ask::SmcLayout(_) | Ask::ExitRecovery { .. },
                        ) => {}
                        Some(Ask::ResetPads { pads, .. }) => {
                            let pads = pads.clone();
                            self.ask = None;
                            self.reset_pads(&pads);
                        }
                        Some(Ask::ClearPads { pads, .. }) => {
                            let pads = pads.clone();
                            self.ask = None;
                            self.clear_pads(&pads);
                        }
                        Some(Ask::ClearPattern { .. }) => {
                            self.ask = None;
                            self.clear_pattern_confirmed();
                        }
                        Some(Ask::PutFile { abs, path, .. }) => {
                            let (abs, path) = (*abs, path.clone());
                            self.put_file(abs, path);
                        }
                        // 83a: Enter places the copy on the target the question names;
                        // if the target changed since, place_copy asks again
                        Some(Ask::CopyPad { source, .. }) => {
                            let source = *source;
                            self.place_copy_asked(source);
                        }
                        None => {}
                    }
                }
                // the files Enter answered about the pad it loads onto: after the load the
                // list is done; the peek keeps the question visible while put_file answers it
                Action::BrowserLoad if matches!(&self.ask, Some(Ask::PutFile { .. })) => {
                    if let Some(Ask::PutFile { abs, path, .. }) = self.ask.as_ref() {
                        let (abs, path) = (*abs, path.clone());
                        self.put_file(abs, path);
                        if self.ask.is_none() {
                            self.goto(Screen::Pads);
                        }
                    }
                }
                Action::Back => self.ask = None,
                // a pad pressed before the question must still let go, or a gate pad hangs
                Action::PadRelease { .. } => {
                    if let Action::PadRelease { pad } = a
                        && !std::mem::take(&mut self.silent_pad_keys[pad])
                    {
                        for cmd in self.pads.apply(a) {
                            self.send(cmd);
                        }
                    }
                }
                _ => {}
            },
            Action::PadEditOnly if self.screen == Screen::Pads => self.edit = true,
            Action::ChopEditOnly if self.screen == Screen::Chop && self.plan.is_none() => {
                self.edit = true;
            }
            Action::BrowserTakeOnly
                if self.screen == Screen::Files && self.browser.in_hand.is_none() =>
            {
                let dir = self.browser.dir.clone();
                self.browser.take();
                if self.browser.dir != dir {
                    self.dirty();
                    self.preview_selection();
                }
            }
            Action::PadEditOnly | Action::ChopEditOnly | Action::BrowserTakeOnly => {}
            // the assign mode is deaf: only the pads, Tab, Enter, Esc and the assign
            // keys act; the wave keys would cut, zoom or move while the owner lays
            // slices onto pads (21)
            // assigning: J L pick the slice the next pad hit plans, and play it (35)
            Action::CursorMove { right, .. }
                if self.screen == Screen::Chop && self.plan.is_some() =>
            {
                if let Some(c) = self.chop.as_mut() {
                    c.select(if right { 1 } else { -1 });
                    let slice = c.selected;
                    self.play_slice(slice);
                }
            }
            a if self.screen == Screen::Chop && self.plan.is_some() && assign_deaf(&a) => {
                self.post_notice(
                    Level::Info,
                    (
                        "assigning: Enter applies, Esc leaves".into(),
                        Instant::now(),
                    ),
                );
            }
            Action::PatternControls
            | Action::PatternParamPrev
            | Action::PatternParamNext
            | Action::PatternValueDec
            | Action::PatternValueInc => self.pattern_panel_action(action),
            Action::PatternViewPrev | Action::PatternViewNext => {
                self.change_pattern_view(action == Action::PatternViewNext);
            }
            Action::PatternStepEdit => self.toggle_step_edit(),
            Action::EditValue { up, .. } if self.pattern_view.edit => self.move_step_cursor(up),
            Action::ParamPrev if self.pattern_view.edit => self.choose_step_event(false),
            Action::ParamNext if self.pattern_view.edit => self.choose_step_event(true),
            Action::PatternOverviewEmptyToggle
                if self.screen == Screen::Pads
                    && self.pattern_view.overview
                    && !self.transport_input_blocked()
                    && !self.help_open =>
            {
                self.pattern_view.hide_empty_banks = !self.pattern_view.hide_empty_banks;
            }
            Action::PatternOverviewEmptyToggle => {}
            Action::PatternClear => self.ask_clear_pattern(),
            Action::PatternRecord => self.toggle_pattern_record(),
            Action::PatternErase => self.toggle_pattern_erase(),
            Action::PatternUndo if self.transport_input_blocked() => {}
            Action::PatternUndo => self.undo_pattern(),
            Action::Undo if self.screen == Screen::Pads && self.request_active_pattern_undo() => {}
            Action::TransportToggle | Action::TempoStep { .. } | Action::MetronomeToggle
                if self.transport_input_blocked() => {}
            Action::TransportToggle => self.send(Command::TransportToggle),
            Action::TapTempo => self.tap_pattern_tempo(),
            Action::TempoStep { up, fine } => {
                let step = if fine { 0.1 } else { 1.0 };
                self.set_tempo(self.transport.bpm + if up { step } else { -step }, true);
            }
            Action::MetronomeToggle => {
                self.transport.metronome = !self.transport.metronome;
                self.send(Command::SetMetronome {
                    enabled: self.transport.metronome,
                });
                self.dirty();
            }
            Action::Click => self.send(Command::Click {
                velocity: CLICK_VELOCITY,
            }),
            Action::SessionSave => self.request_manual_save(),
            Action::RecordMic(_) | Action::RecordOutputToggle => self.audio_record_action(action),
            Action::HelpToggle => self.toggle_help(),
            Action::HelpPagePrev => self.help_page(false),
            Action::HelpPageNext => self.help_page(true),
            Action::LearnToggle if self.screen.rules().screen_learn => {
                self.screen_learn = match self.screen_learn {
                    Some(_) => None,
                    None => Some(self.learn_open()),
                };
            }
            Action::LearnToggle => {}
            Action::PadRename if self.screen == Screen::Pads => {
                let abs = self.pads.selected_abs();
                if self.pads.file[abs].is_some() {
                    self.pad_naming = Some((abs, pad_name(&self.pads, abs)));
                } else {
                    self.post_notice(
                        Level::Warn,
                        (
                            "pad is empty: put a sample on it first".into(),
                            Instant::now(),
                        ),
                    );
                }
            }
            Action::PadRename => {}
            Action::PadPitchGateToggle if self.screen != Screen::Pads => {}
            Action::ProjectPick => {
                if !self.refuse_pattern_project_change() {
                    self.project_picker = Some(ProjectPicker::new(&self.paths.projects));
                }
            }
            Action::PadsToggle => {
                self.ui.hide_pads = !self.ui.hide_pads;
                let result = match &self.user_path {
                    Some(p) => config::save_section(p, "ui", &self.ui),
                    None => Err("the pads choice is not saved: fix the config file first".into()),
                };
                if let Err(e) = result {
                    log::error!("{e}");
                    self.post_notice(Level::Error, (e, Instant::now()));
                }
            }
            Action::ScreenPads => self.goto(Screen::Pads),
            Action::ScreenPattern => {
                self.goto(Screen::Pads);
                self.pattern_view.overview = true;
                self.pattern_view.browse_cell = None;
            }
            Action::ScreenFiles => self.goto(Screen::Files),
            Action::ScreenFx => self.goto(Screen::Fx),
            Action::ScreenChop if self.screen == Screen::Files => self.open_file_chop(),
            Action::ScreenChop => self.open_chop(),
            Action::PadPress { pad, velocity } => {
                self.pad_hit(
                    pad,
                    velocity,
                    pads::HitSource::Keyboard,
                    midi::PadSound::UiUnattempted { deferred: false },
                    None,
                );
            }
            Action::PadRelease { pad } if std::mem::take(&mut self.silent_pad_keys[pad]) => {}
            Action::Back if self.screen == Screen::Chop && self.plan.is_some() => self.plan = None,
            Action::ChopPlay if self.screen == Screen::Chop => {
                // the cursor is the owner's: play starts from it every time, a stop leaves it
                if self.playhead().is_some() {
                    self.stop_preview();
                } else if let Some(c) = &self.chop {
                    let region = c.play_region();
                    let looped = c.looping.then_some(region);
                    let cmd = Command::Preview {
                        sound: Some(c.sound.clone()),
                        region,
                    };
                    let as_pad = self.as_pad_of(c.selected);
                    // the loop first: the preview starts already knowing it (49); and the
                    // pad it plays as (50)
                    self.send(Command::PreviewLoop { region: looped });
                    self.send(Command::PreviewAs { pad: as_pad });
                    self.loop_sent = looped;
                    self.send(cmd);
                }
            }
            Action::CursorJump { right } if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    let before = c.selected;
                    c.jump_cursor(right);
                    if c.selected != before {
                        let slice = c.selected;
                        self.play_slice(slice);
                    }
                }
            }
            Action::ChopLoopToggle if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.looping = !c.looping;
                }
                // a play going on loops from now, or plays out its turn
                if self.playhead().is_some() {
                    self.loop_sent = None;
                    if !self.chop.as_ref().is_some_and(|c| c.looping) {
                        self.send(Command::PreviewLoop { region: None });
                    }
                }
            }
            Action::ChopLoopToggle => {}
            Action::ChopAsPadToggle if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.as_pad = !c.as_pad;
                }
            }
            Action::ChopAsPadToggle => {}
            Action::ChopPlayMode if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.play = c.play.next();
                }
            }
            Action::ChopCut if self.screen == Screen::Chop => {
                // while it plays the cut lands where the sound is; the cursor stays
                let playhead = self.playhead();
                let Some(c) = self.chop.as_mut() else {
                    return false;
                };
                let before = c.cut_ids().to_vec();
                let placed = c.cut(playhead.unwrap_or(c.cursor));
                if placed == chop::Cut::Refused {
                    self.post_notice(
                        Level::Warn,
                        ("too close to another cut".into(), Instant::now()),
                    );
                } else {
                    // a cut inside a slice on pads keeps them on the left part, the slice
                    // that still starts at their cut (36): they get shorter, and say so
                    let split = c.cut_ids().iter().position(|id| !before.contains(id));
                    let (file, epoch) = (c.file.clone(), c.epoch);
                    let start = split.map(|i| c.start_id(i));
                    let shorter: Vec<String> = (0..PADS)
                        .filter(|&a| {
                            start.is_some_and(|id| {
                                self.pads.file[a].as_ref() == Some(&file)
                                    && self.pads.link[a] == Some((id, epoch))
                            })
                        })
                        .map(bank_key)
                        .collect();
                    self.sync_links();
                    let snapped = (placed == chop::Cut::Snapped).then_some("marker snapped to hit");
                    let pads = match shorter.len() {
                        0 => None,
                        1 => Some(format!("pad {} is shorter now; Ctrl+Z returns", shorter[0])),
                        _ => Some(format!(
                            "pads {} are shorter now; Ctrl+Z returns",
                            shorter.join(", ")
                        )),
                    };
                    let text = match (snapped, pads) {
                        (Some(a), Some(b)) => Some(format!("{a}; {b}")),
                        (Some(a), None) => Some(a.to_string()),
                        (None, b) => b,
                    };
                    if let Some(t) = text {
                        self.post_notice(Level::Info, (t, Instant::now()));
                    }
                }
            }
            Action::CursorMove { right, fine } if self.screen == Screen::Chop => {
                let repeats = held_repeats(&mut self.bound_repeats, fine);
                if let Some(c) = self.chop.as_mut() {
                    let before = c.selected;
                    let step = c.view_step(repeats);
                    c.move_cursor(right, step);
                    // the page of slices follows the slice under the cursor
                    // the slice the cursor enters plays once (21 "The cursor selects")
                    if c.selected != before {
                        let slice = c.selected;
                        self.play_slice(slice);
                    }
                }
            }
            Action::ChopZoom { zoom_in } if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.zoom(zoom_in);
                }
            }
            Action::ThresholdStep(step) if self.screen == Screen::Chop => {
                self.step_threshold(i32::from(step));
            }
            Action::ChopRegionStart if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    let pos = c.cursor;
                    c.set_region_start(pos);
                }
            }
            Action::ChopRegionEnd if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    let pos = c.cursor;
                    c.set_region_end(pos);
                }
            }
            Action::ChopSnapToggle if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.snap = c.snap.next();
                }
            }
            Action::ChopRegionClear if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.clear_region();
                }
            }
            Action::ChopNormalize if self.screen == Screen::Chop => self.normalize_pad(),
            Action::ChopCommit | Action::ChopEdit if self.screen == Screen::Chop => {
                if self.plan.is_some() {
                    self.apply_plan();
                } else if action == Action::ChopEdit {
                    // listen: Enter opens the slice edit; the commit stays the device key
                    self.edit = true;
                } else if !self.edit {
                    // the edit and the assign never stand together: Esc comes first; a
                    // controller has no pads to tap, so its commit lays the slices in order
                    // from the selected pad and the next commit applies them (21)
                    self.plan = Some(chop::Plan::InOrder(Some(self.pads.selected_abs())));
                    self.plan_undo.clear();
                }
            }
            Action::ChopAssign if self.screen == Screen::Chop && !self.edit => self.start_assign(),
            Action::ChopParts | Action::ParamNext | Action::ChopAssignOrder
                if self.screen == Screen::Chop && self.plan.is_some() =>
            {
                let selected = self.chop.as_ref().map_or(0, |c| c.selected);
                let next = match &self.plan {
                    Some(chop::Plan::InOrder(_)) => {
                        if let Some(c) = self.chop.as_mut() {
                            c.selected = 0;
                        }
                        chop::Plan::ByHand(Vec::new())
                    }
                    // the owner hits the start pad next
                    _ => chop::Plan::InOrder(None),
                };
                self.change_plan(next, selected);
            }
            Action::ParamNext | Action::ParamPrev if self.screen == Screen::Chop => {
                // BOUNDS has two parameters: next and previous both swap them
                self.slice_end = !self.slice_end;
            }
            // BOUNDS is the only page of a slice: the page keys keep it (54)
            Action::PageNext | Action::PagePrev if self.screen == Screen::Chop => {}
            Action::PageNext | Action::PagePrev if self.screen == Screen::Fx => {
                self.select_fx(1 - self.fx_slot);
            }
            Action::ParamNext | Action::ParamPrev | Action::PadParamNext
                if self.screen == Screen::Fx =>
            {
                let step = if action == Action::ParamPrev { 2 } else { 1 };
                self.fx_param = (self.fx_param + step) % 3;
            }
            Action::EditValue { .. } | Action::PadParam(_) if self.screen == Screen::Fx => {
                let p = self.fx_selected_param();
                let saved = self.pads.param;
                self.pads.param = p;
                let cmds = self.pads.apply(action);
                self.pads.param = saved;
                self.pads.limit_hits = None;
                self.pickup = None;
                self.lit = Some((Lit::Pad(p), Instant::now()));
                if !cmds.is_empty() {
                    for cmd in cmds {
                        self.send(cmd);
                    }
                    self.dirty();
                }
            }
            // the value keys act on a pad, a slice or a shared effect
            Action::ParamNext
            | Action::ParamPrev
            | Action::PageNext
            | Action::PagePrev
            | Action::EditValue { .. }
                if !matches!(self.screen, Screen::Pads | Screen::Chop) => {}
            Action::Undo if self.screen == Screen::Chop && self.plan.is_some() => {
                match self.plan_undo.pop() {
                    Some((prev, selected)) => {
                        self.plan = Some(prev);
                        if let Some(c) = self.chop.as_mut() {
                            c.select_slice(selected);
                        }
                    }
                    None => {
                        self.post_notice(Level::Info, ("nothing to undo".into(), Instant::now()))
                    }
                }
            }
            Action::MarkerSelect(step) if self.screen == Screen::Chop => {
                if let Some(c) = self.chop.as_mut() {
                    c.select(step);
                    let slice = c.selected;
                    self.play_slice(slice);
                }
            }
            Action::ChopParts
            | Action::ChopHits
            | Action::ChopHitsBack
            | Action::ChopEqualParts
            | Action::MarkerNudge { .. }
            | Action::EditValue { .. }
            | Action::SliceBound { .. }
            | Action::MarkerDelete
            | Action::SliceEndDelete
            | Action::Undo
                if self.screen == Screen::Chop =>
            {
                let linked = self.linked_pads();
                let end = self.slice_end;
                if let Some(c) = self.chop.as_mut() {
                    match action {
                        Action::ChopParts | Action::ChopEqualParts => {
                            c.next_parts();
                            if linked > 0 {
                                self.post_notice(Level::Warn, (
                                    format!(
                                        "{linked} pads keep their old slices and no longer follow the cuts; Ctrl+Z links them again"
                                    ),
                                    Instant::now(),
                                ));
                            }
                        }
                        Action::ChopHits => {
                            // the cuts it replaces may be hand-made: say how many there were (68)
                            let were = c.slices().len();
                            let (db, _) = c.chop_by_hits();
                            let now = c.slices().len();
                            let mut text = format!("by hits at {db:.0} dB: {now} slices");
                            if were > 1 {
                                text.push_str(&format!(" (were {were}) · Ctrl+Z brings them back"));
                            }
                            if linked > 0 {
                                text.push_str(&format!(
                                    "; {linked} pads keep their old slices, Ctrl+Z links them again"
                                ));
                            }
                            self.post_notice(
                                if linked > 0 { Level::Warn } else { Level::Done },
                                (text, Instant::now()),
                            );
                        }
                        // 83a: Shift+H steps the threshold back; a clamped step is a no-op
                        // that still names the threshold, without claiming replaced cuts
                        Action::ChopHitsBack => {
                            let were = c.slices().len();
                            match c.chop_by_hits_back() {
                                Some((db, _)) => {
                                    let now = c.slices().len();
                                    let mut text = format!("by hits at {db:.0} dB: {now} slices");
                                    if were > 1 {
                                        text.push_str(&format!(
                                            " (were {were}) · Ctrl+Z brings them back"
                                        ));
                                    }
                                    if linked > 0 {
                                        text.push_str(&format!(
                                            "; {linked} pads keep their old slices, Ctrl+Z links them again"
                                        ));
                                    }
                                    self.post_notice(
                                        if linked > 0 { Level::Warn } else { Level::Done },
                                        (text, Instant::now()),
                                    );
                                }
                                None => {
                                    let db = c.threshold_db;
                                    self.post_notice(
                                        Level::Info,
                                        (format!("by hits stays at {db:.0} dB"), Instant::now()),
                                    );
                                }
                            }
                        }
                        Action::MarkerNudge { right, fine } => {
                            let ms = if fine { NUDGE_FINE_MS } else { NUDGE_MS };
                            c.nudge(false, if right { ms } else { -ms });
                            self.lit = Some((Lit::Bound(false), Instant::now()));
                        }
                        Action::EditValue { up, fine } => {
                            let step = c.view_step(held_repeats(&mut self.bound_repeats, fine));
                            c.nudge_by(end, up, step);
                            self.lit = Some((Lit::Bound(end), Instant::now()));
                        }
                        Action::SliceBound { end, right, fine } => {
                            let step = c.view_step(held_repeats(&mut self.bound_repeats, fine));
                            c.nudge_by(end, right, step);
                            // the bound just moved is the picked one: it is drawn thick and lit
                            self.slice_end = end;
                            self.lit = Some((Lit::Bound(end), Instant::now()));
                        }
                        Action::MarkerDelete => c.delete(),
                        Action::SliceEndDelete => c.delete_end(),
                        _ => {
                            if !c.undo() && !self.undo_fresh() {
                                self.post_notice(
                                    Level::Info,
                                    ("nothing to undo".into(), Instant::now()),
                                );
                            }
                        }
                    }
                }
            }
            Action::ScreenSettings => {
                self.goto(match self.screen {
                    Screen::Settings => Screen::Pads,
                    _ => Screen::Settings,
                });
            }
            Action::Back if self.browser.in_hand.is_some() => self.browser.in_hand = None,
            Action::Back if self.pad_in_hand.is_some() => self.pad_in_hand = None,
            // 83a: Esc drops a pending copy; in the question it keeps the target and the copy
            Action::Back if matches!(self.ask, Some(Ask::CopyPad { .. })) => self.ask = None,
            Action::Back if self.copy_in_hand.is_some() => {
                self.copy_in_hand = None;
                self.post_notice(Level::Info, ("copy dropped".into(), Instant::now()));
            }
            // Esc on the files screen drops a filter first (66)
            Action::Back if self.screen == Screen::Files && self.browser.clear_filter() => {}
            // a pad in hand: Enter swaps with the selected pad instead of opening its edit
            Action::PadEdit if self.screen == Screen::Pads && self.pad_in_hand.is_some() => {
                self.swap_with(self.pads.selected_abs())
            }
            Action::PadEdit if self.screen == Screen::Pads && self.browser.in_hand.is_some() => {
                self.put_in_hand(self.pads.selected_abs())
            }
            Action::PadEdit if self.screen == Screen::Pads => self.edit = true,
            Action::PadTake => {
                self.browser.in_hand = None;
                self.pad_in_hand = Some(self.pads.selected_abs());
            }
            // 83a: Ctrl+C copies, Ctrl+Q stays the quit; a copy and a swap never share a hand
            Action::PadCopy if self.screen == Screen::Pads => {
                self.pad_in_hand = None;
                self.take_copy();
            }
            // a run waits for the next control: Esc stops it and keeps what is learned (28)
            Action::Back if self.midi_run.is_some() => self.midi_run = None,
            Action::Back if self.learning.is_some() => self.learning = None,
            // the help covers the screen: Esc closes it before the edit mode under it
            Action::Back if self.help_open => self.help_open = false,
            Action::Back if self.pattern_view.edit => {
                self.pattern_view.edit = false;
                self.pattern_view.selected_event = None;
            }
            Action::Back if self.pattern_view.overview => self.pattern_view.overview = false,
            Action::Back if self.pattern_panel.is_some() => self.pattern_panel = None,
            // Esc leaves erasure before it finishes a pass or leaves the screen (98b)
            Action::Back if self.erase_pads => self.erase_pads = false,
            Action::Back if self.pattern_erasing() => self.toggle_pattern_erase(),
            // the edit mode is the innermost layer of the screen: Esc closes it, the screen stays
            Action::Back if self.edit => self.edit = false,
            // Esc drops the marks first, then goes back as now
            Action::Back if self.pads.drop_marks() => {}
            Action::Back if self.finish_pattern_record() => {}
            Action::Back => self.goto(Screen::Pads),
            // MIDI tab: the bank keys jump between group rows
            Action::BankNext | Action::BankPrev
                if self.screen == Screen::Settings && self.settings_tab == 2 =>
            {
                let list = self.midi_list();
                if let Some(to) = group_jump(&list, self.midi_row, action == Action::BankNext) {
                    self.midi_row = to;
                }
            }
            Action::SettingsRow(step) if self.settings_tab == 2 => {
                self.midi_row = (self.midi_row as isize + step as isize)
                    .clamp(0, self.midi_list().len() as isize - 1)
                    as usize
            }
            // the Paths tab has two rows: the projects root and the samples root (11)
            Action::SettingsRow(step) if self.settings_tab == 3 => {
                self.settings_row =
                    (self.settings_row as isize + step as isize).clamp(0, 1) as usize
            }
            Action::SettingsRow(step) => {
                self.settings_row = (self.settings_row as isize + step as isize)
                    .clamp(0, SETTINGS_ROWS as isize - 1)
                    as usize
            }
            Action::SettingsValue(step)
                if self.settings_tab == 2
                    && self
                        .learning
                        .as_ref()
                        .is_some_and(|l| !l.info.knob && l.pending.is_none()) =>
            {
                if let Some(learning) = self.learning.as_mut() {
                    learning.slot = if step < 0 {
                        crate::learn::GestureSlot::Tap
                    } else {
                        crate::learn::GestureSlot::Hold
                    };
                }
            }
            // MIDI tab: the knob response row steps its value (58)
            Action::SettingsValue(step)
                if self.settings_tab == 2
                    && matches!(
                        self.midi_list().get(self.midi_row),
                        Some(MidiListRow::Response)
                    ) =>
            {
                self.step_response(step as isize)
            }
            Action::SettingsValue(step)
                if self.settings_tab == 2
                    && matches!(
                        self.midi_list().get(self.midi_row),
                        Some(MidiListRow::HoldTime)
                    ) =>
            {
                self.step_hold_time(step as isize)
            }
            Action::SettingsValue(step)
                if self.settings_tab == 2
                    && matches!(
                        self.midi_list().get(self.midi_row),
                        Some(MidiListRow::SmcPadBank)
                    ) =>
            {
                self.set_smc_pad_bank(step > 0);
            }
            // MIDI tab: → opens the group of the selected row, ← closes it (28)
            Action::SettingsValue(step) if self.settings_tab == 2 => self.midi_fold(step > 0),
            // the Paths tab: Enter opens the folder picker of the selected root (11)
            Action::SettingsLearn if self.settings_tab == 3 => {
                let current = if self.settings_row == 0 {
                    self.paths.projects.clone()
                } else {
                    self.paths.samples.clone()
                };
                self.path_picker = Some(PathPicker::open(self.settings_row, &current));
            }
            Action::SettingsLearn
                if self.settings_tab == 2
                    && self.learning.as_ref().is_some_and(|l| l.pending.is_some()) =>
            {
                self.learned_save()
            }
            Action::SettingsLearn if self.settings_tab == 2 => {
                match self.midi_list().get(self.midi_row) {
                    Some(MidiListRow::Group { .. }) => self.midi_fold(true),
                    Some(MidiListRow::Response) => self.step_response(1),
                    Some(MidiListRow::HoldTime) => self.step_hold_time(1),
                    Some(MidiListRow::SmcPadBank) => {
                        self.set_smc_pad_bank(!self.midi_gestures.smc_pad_bank_opens_panel)
                    }
                    Some(MidiListRow::SmcLayoutList(_)) => {
                        self.help_overlay.controller.smc_layout_open ^= true;
                    }
                    Some(MidiListRow::SmcLayoutApply) => self.ask_smc_layout(None),
                    Some(MidiListRow::SmcLayoutSlot(slot)) => self.ask_smc_layout(Some(slot)),
                    Some(MidiListRow::Reset { name, port }) => {
                        self.ask = Some(Ask::ResetProfile {
                            lines: vec![
                                format!("Reset {name} to its profile?"),
                                "Every binding of yours for it is dropped.".into(),
                                "Enter: reset    Esc: keep".into(),
                            ],
                            name: name.clone(),
                            port: port.clone(),
                        });
                    }
                    Some(MidiListRow::Set { infos, .. }) => self.midi_run_start(infos),
                    Some(MidiListRow::Action { info }) => {
                        self.midi_run = None;
                        self.learning = Some(learn::Learning::new(info));
                    }
                    Some(MidiListRow::Disabled { .. }) => {}
                    None => {}
                }
            }
            Action::SettingsClear if self.settings_tab == 2 => {
                match self.midi_list().get(self.midi_row) {
                    // the run rows clear all together (28)
                    Some(MidiListRow::Set { infos: run, .. }) => {
                        let infos: &'static [input::ActionInfo] = run;
                        let mut profiles: Vec<String> = Vec::new();
                        for info in infos {
                            for p in learn::clear(&mut self.learned, info.group, info.name) {
                                if !profiles.contains(&p) {
                                    profiles.push(p);
                                }
                            }
                        }
                        for p in profiles {
                            self.sync_learned(&p, infos[0].group);
                        }
                        self.save_midi();
                    }
                    Some(MidiListRow::Action { info }) => {
                        for profile in learn::clear(&mut self.learned, info.group, info.name) {
                            self.sync_learned(&profile, info.group);
                        }
                        self.save_midi();
                    }
                    Some(MidiListRow::Disabled {
                        profile,
                        group,
                        key,
                    }) => {
                        let (profile, group, key) = (profile.clone(), group.clone(), key.clone());
                        if let Some(bindings) = self
                            .learned
                            .get_mut(&profile)
                            .and_then(|m| m.get_mut(&group))
                        {
                            bindings.remove(&key);
                        }
                        if let Some(group) = input::Group::ALL
                            .into_iter()
                            .find(|candidate| candidate.key() == group)
                        {
                            self.sync_learned(&profile, group);
                        }
                        self.save_midi();
                    }
                    _ => {}
                }
            }
            Action::Confirm | Action::SettingsLearn | Action::SettingsClear => {}
            Action::SmcList { next } => self.smc_list(next),
            Action::SettingsValue(step) => self.change(self.settings_row, step as isize),
            Action::SettingsTab => {
                self.settings_tab = (self.settings_tab + 1) % 4;
                self.learning = None;
                self.midi_run = None;
            }
            // a file in hand: the list keys choose the pad it goes to
            Action::BrowserHold
                if self.screen == Screen::Files && self.browser.in_hand.is_some() =>
            {
                self.put_in_hand(self.pads.selected_abs())
            }
            // Enter on the list: a file goes to the selected pad at once, a folder opens
            Action::BrowserLoad if self.screen == Screen::Files => {
                match self.browser.selected_file() {
                    Some(path) => {
                        self.put_file(self.pads.selected_abs(), path);
                        if self.ask.is_none() {
                            self.goto(Screen::Pads);
                        }
                    }
                    None => {
                        let dir = self.browser.dir.clone();
                        self.browser.enter();
                        if self.browser.dir != dir {
                            self.dirty();
                            self.preview_selection();
                        }
                    }
                }
            }
            Action::BrowserUp | Action::BrowserDown => {
                self.browser
                    .step(if action == Action::BrowserUp { -1 } else { 1 });
                self.preview_selection();
            }
            Action::BrowserEnter | Action::BrowserParent | Action::BrowserHold => {
                let dir = self.browser.dir.clone();
                match action {
                    Action::BrowserEnter => self.browser.enter(),
                    Action::BrowserParent => self.browser.parent(),
                    _ => self.browser.take(),
                }
                if self.browser.dir != dir {
                    self.dirty();
                    self.preview_selection();
                }
            }
            Action::MasterVolume(step) => {
                self.set_master(self.master_db + step);
                self.dirty();
            }
            Action::BrowserFilter => self.browser.editing = true,
            Action::BrowserFavorite => {
                // on a file Ctrl+B marks the folder it lies in: say so, or it reads as nothing (68)
                let of_file = self.browser.current().is_some_and(|e| !e.is_dir);
                let (name, added) = self.browser.toggle_favorite();
                let whose = if of_file {
                    " (the folder of this file)"
                } else {
                    ""
                };
                self.post_notice(
                    Level::Done,
                    (
                        if added {
                            format!("★ {name}: favorite{whose}")
                        } else {
                            format!("{name}: no longer a favorite{whose}")
                        },
                        Instant::now(),
                    ),
                );
                {
                    let section = config::BrowserConfig {
                        favorites: self.browser.favorites.clone(),
                    };
                    let result = match &self.user_path {
                        Some(p) => config::save_section(p, "browser", &section),
                        None => Err("favorites are not saved: fix the config file first".into()),
                    };
                    if let Err(e) = result {
                        log::error!("{e}");
                        self.post_notice(Level::Error, (e, Instant::now()));
                    }
                }
            }
            Action::BrowserPreviewToggle => {
                self.preview_on = !self.preview_on;
                self.preview_selection();
            }
            // the SOURCE page: the app changes it, it knows the cuts of the file (27)
            Action::EditValue { up, fine }
                if self.screen == Screen::Pads
                    && matches!(
                        self.pads.param,
                        pads::Param::Source | pads::Param::Start | pads::Param::End
                    ) =>
            {
                self.source_value(up, fine)
            }
            Action::PadParam(step)
                if self.screen == Screen::Pads
                    && matches!(
                        self.pads.param,
                        pads::Param::Source | pads::Param::Start | pads::Param::End
                    ) =>
            {
                self.source_value(step > 0, false)
            }
            Action::Undo if matches!(self.screen, Screen::Pads | Screen::Files | Screen::Fx) => {
                self.undo_pads()
            }
            // in the pad edit the same key resets the settings of the scope, after a question (76)
            Action::PadClear if self.edit && self.screen == Screen::Pads => self.ask_reset(),
            // with marks the clear asks first: one key must not wipe a bank unasked (76)
            Action::PadClear if !self.pads.marked.is_empty() => self.ask_clear(),
            Action::PadClear => self.clear_pads(&[self.pads.selected_abs()]),
            pad_action => {
                for cmd in self.pads.apply(pad_action) {
                    self.send(cmd);
                }
                let value = matches!(pad_action, Action::PadParam(_) | Action::EditValue { .. });
                if value || matches!(pad_action, Action::PadModeNext | Action::PadPitchGateToggle) {
                    self.dirty();
                }
                if value {
                    self.lit = Some((Lit::Pad(self.pads.param), Instant::now()));
                }
                if let Some(n) = self.pads.limit_hits.take() {
                    let what = self.pads.param_name();
                    let pad = if n == 1 { "pad" } else { "pads" };
                    self.post_notice(
                        Level::Warn,
                        (format!("{what}: {n} {pad} at the limit"), Instant::now()),
                    );
                } else if value {
                    // the changed value pops up, so the ear and the eye agree
                    let value = self.pads.param_value();
                    let what = self.pads.param_name();
                    self.post_notice(Level::Info, (format!("{what}: {value}"), Instant::now()));
                }
            }
        }
        false
    }

    /// Open pattern controls own the edit keys and buttons: they change A1, and commands
    /// that change a pad, a slice or an effect wait (98b). Pads still play.
    fn pattern_controls_own(&self, action: Action) -> Option<Action> {
        if self.pattern_panel.is_none() || self.transport_input_blocked() {
            return Some(action);
        }
        match action {
            Action::ParamPrev => Some(Action::PatternParamPrev),
            Action::ParamNext => Some(Action::PatternParamNext),
            Action::EditValue { up: false, .. } => Some(Action::PatternValueDec),
            Action::EditValue { up: true, .. } => Some(Action::PatternValueInc),
            Action::PadClear
            | Action::PadCopy
            | Action::PadEdit
            | Action::PadEditOnly
            | Action::PadMarkBank
            | Action::PadMarkToggle
            | Action::PadModeNext
            | Action::PadParam(_)
            | Action::PadParamNext
            | Action::PadPitchGateToggle
            | Action::PadRename
            | Action::PadTake
            | Action::PageNext
            | Action::PagePrev
            | Action::BrowserHold
            | Action::BrowserLoad
            | Action::BrowserTakeOnly
            | Action::ChopAsPadToggle
            | Action::ChopAssign
            | Action::ChopAssignOrder
            | Action::ChopCommit
            | Action::ChopCut
            | Action::ChopEdit
            | Action::ChopEditOnly
            | Action::ChopEqualParts
            | Action::ChopHits
            | Action::ChopHitsBack
            | Action::ChopNormalize
            | Action::ChopParts
            | Action::ChopPlayMode
            | Action::ChopRegionClear
            | Action::ChopRegionEnd
            | Action::ChopRegionStart
            | Action::ChopSnapToggle
            | Action::MarkerDelete
            | Action::MarkerNudge { .. }
            | Action::SliceBound { .. }
            | Action::SliceEndDelete => None,
            action => Some(action),
        }
    }

    pub(super) fn focus(&self) -> input::Focus {
        input::Focus {
            screen: self.screen,
            bank: self.pads.bank,
            mode: input::Mode {
                edit: self.edit,
                marking: self.pads.marking,
                text: self.pad_naming.is_some()
                    || (self.screen == Screen::Files && self.browser.editing)
                    || self
                        .project_picker
                        .as_ref()
                        .is_some_and(|p| p.naming.is_some()),
                picker: self.project_picker.is_some() || self.path_picker.is_some(),
                screen_learn: self.screen_learn.is_some(),
                midi_learn: self.learning.is_some() || self.midi_run.is_some(),
                question: self.ask.is_some(),
                file_in_hand: self.browser.in_hand.is_some(),
                copy_in_hand: self.copy_in_hand.is_some(),
                swap_in_hand: self.pad_in_hand.is_some(),
                pattern_pads: self.pattern_routes_pads(),
                erase_pads: self.erase_pads,
                step_edit: self.pattern_view.edit,
                pattern_controls: self.pattern_panel.is_some(),
            },
        }
    }

    pub(super) fn publish_midi(&self, shared: &midi::Shared) {
        shared.publish(self.focus());
    }

    pub(super) fn midi_ui_event(&mut self, ev: midi::UiEvent) {
        self.input_sources.observe(&ev, Instant::now());
        self.observe_smc_knob(&ev);
        self.observe_smc_pad(&ev);
        let sequence = ev.sequence();
        let received_at = ev.source().map(|source| source.at);
        let preview_ready = ev.preview_ready();
        let origin = ev
            .origin()
            .map_or(Origin::System, |port| Origin::Device(port.into()));
        let owner_input = ev.owner_input();
        let released = ev.release_signal();
        let raw_key = ev.control_key();
        let blocked_exit_retry = matches!(
            self.ask,
            Some(Ask::ExitRecovery { offered_at, .. })
                if received_at.is_some_and(|at| at <= offered_at)
        ) && match &ev.event {
            midi::Event::Action(Action::Confirm) | midi::Event::PanelCell { cell: 8, .. } => true,
            midi::Event::Message {
                port,
                msg:
                    input::midi::Msg::NoteOn { note: 63, .. }
                    | input::midi::Msg::NoteOn {
                        ch: 10, note: 47, ..
                    },
                ..
            } => port.contains("SMC-PAD-Master"),
            _ => false,
        };
        let blocked_transport_answer = self.ask.is_some()
            && ev
                .origin()
                .is_some_and(|port| port.contains("SMC-PAD-Master"))
            && matches!(ev.control_key(), Some(input::midi::Key::Cc(27 | 28)))
            && matches!(ev.event, midi::Event::Action(action) if !matches!(action, Action::PlayContext | Action::StopContext));
        let blocked_picker_value = self.path_picker.is_some()
            && ev
                .origin()
                .is_some_and(|port| port.contains("SMC-PAD-Master"))
            && matches!(ev.control_key(), Some(input::midi::Key::Cc(31)))
            && matches!(ev.event, midi::Event::Action(Action::SettingsValue(_)));
        let suppressed = ev
            .origin()
            .zip(raw_key)
            .is_some_and(|(port, key)| self.question_pads.contains(&(port.into(), key)))
            && matches!(
                ev.event,
                midi::Event::Action(_) | midi::Event::PadHit { .. } | midi::Event::PanelCell { .. }
            );
        if let Some(ev) = ev
            .current(self.audio.as_ref().map(|audio| audio.project.as_ref()))
            .filter(|_| {
                !suppressed
                    && !blocked_transport_answer
                    && !blocked_picker_value
                    && !blocked_exit_retry
            })
        {
            let old = std::mem::replace(&mut self.notice_origin, origin);
            if owner_input {
                self.acknowledge_notice();
            }
            self.midi_preview_blocked = !preview_ready;
            self.transport.tap_input_at = received_at;
            self.midi_event_with_release(ev, released, raw_key);
            self.transport.tap_input_at = None;
            self.midi_preview_blocked = false;
            self.notice_origin = old;
        }
        if let Some(sequence) = sequence
            && let Some(midi) = &self.midi
        {
            midi.consumed(self.focus(), sequence);
        }
    }

    fn pad_hit(
        &mut self,
        pad: usize,
        velocity: f32,
        source: pads::HitSource,
        sound: midi::PadSound,
        hold: Option<&(u64, std::sync::Arc<engine::capture::EraseHold>)>,
    ) -> Option<usize> {
        let from_midi = matches!(source, pads::HitSource::Midi);
        let deferred =
            from_midi && matches!(sound, midi::PadSound::UiUnattempted { deferred: true });
        // Only an explicitly deferred MIDI hit chooses its bank after earlier commands.
        let (abs, local) = if from_midi {
            let local = pad % PADS_PER_BANK;
            (
                if deferred {
                    self.pads.bank * PADS_PER_BANK + local
                } else {
                    pad
                },
                local,
            )
        } else {
            (self.pads.bank * PADS_PER_BANK + pad, pad)
        };
        if self.ask.is_some() {
            return None;
        }
        if sound == midi::PadSound::EditUnattempted && !self.pattern_view.edit {
            return None;
        }
        if self.pattern_view.edit {
            if !from_midi
                || sound == midi::PadSound::EditUnattempted
                || sound == (midi::PadSound::UiUnattempted { deferred: true })
            {
                if !from_midi {
                    self.silent_pad_keys[local] = true;
                }
                self.step_pad_hit(abs, velocity);
            }
            return None;
        }
        if self.erase_pads {
            // a silent target: the press erases the pad's hits, a late direct sound is not
            // a new erasure
            if !matches!(sound, midi::PadSound::DirectAccepted) {
                self.pads.selected = local;
                self.erase_whole_pad(abs);
            }
            return None;
        }
        let pattern_pads = self.pattern_routes_pads();
        if self.screen == Screen::Chop
            && pattern_pads
            && let Some(c) = self.chop.as_mut()
            && self.pads.file[abs].as_ref() == Some(&c.file)
            && let Some((id, epoch)) = self.pads.link[abs]
            && epoch == c.epoch
        {
            c.select_slice(c.slice_of(id));
        }
        if self.screen == Screen::Chop && !pattern_pads {
            if matches!(sound, midi::PadSound::UiUnattempted { .. }) {
                return self.chop_pad(local, velocity, source);
            }
            // A late direct notification is not a new Chop action.
        } else {
            // Existing overlap order differs: MIDI file/copy/swap; keyboard copy/swap/file.
            if from_midi && self.browser.in_hand.is_some() {
                self.put_in_hand(abs);
                self.pads.selected = local;
                return None;
            }
            if self.copy_in_hand.is_some() {
                let before = from_midi.then(|| self.before());
                self.pads.selected = local;
                self.copy_target(abs);
                if let Some(before) = before {
                    self.note_step(before, "copy".into());
                }
                return None;
            }
            if self.pad_in_hand.is_some() {
                let before = (from_midi
                    && matches!(self.screen, Screen::Pads | Screen::Files | Screen::Fx))
                .then(|| self.before());
                self.swap_with(abs);
                self.pads.selected = local;
                if let Some(before) = before {
                    self.note_step(before, "swap".into());
                }
                return None;
            }
            if self.browser.in_hand.is_some() {
                self.put_in_hand(abs);
                self.pads.selected = local;
                return None;
            }
            if deferred {
                self.meters.hit(Instant::now());
                self.pads.hit(abs);
                let command = match hold {
                    Some((pass, hold)) => Command::PatternPadPress {
                        pass: *pass,
                        pad: abs,
                        velocity,
                        hold: hold.clone(),
                    },
                    None => Command::PadPress { pad: abs, velocity },
                };
                return self.try_send(command).then_some(abs);
            }
            if !from_midi {
                self.meters.hit(Instant::now());
                if let Some(command) = self.keyboard_erase_target(abs) {
                    self.pads.selected = local;
                    self.pads.hit(abs);
                    self.try_send(command);
                    return None;
                }
                let mut accepted = None;
                for cmd in self.pads.press(local, velocity, source) {
                    if self.try_send(cmd) {
                        accepted = Some(abs);
                    }
                }
                return accepted;
            }
        }
        if sound == midi::PadSound::DirectAccepted && self.pad_sounds(abs) {
            self.splash = false;
        }
        self.meters.hit(Instant::now());
        self.pads.hit(abs);
        None
    }

    pub(super) fn midi_event(&mut self, ev: midi::Event) {
        self.midi_event_with_release(ev, None, None);
    }

    fn midi_event_with_release(
        &mut self,
        ev: midi::Event,
        released: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
        raw_key: Option<input::midi::Key>,
    ) {
        let value_before = self.value_before_midi(&ev);
        let knob = matches!(&ev, midi::Event::Step { port, .. } if port.contains("SMC-PAD-Master"));
        let cuts = self.chop.as_ref().map(|c| c.saved());
        // a knob on the pads screen is a step of the undo (61); a turn joins one step
        // a button runs its action through `apply`, which keeps the step itself
        let live_pad = match &ev {
            midi::Event::Step {
                port,
                control,
                target,
                ..
            } => smc_live_pad_param(port, control, *target),
            _ => None,
        };
        let before = (live_pad.is_some()
            || (matches!(self.screen, Screen::Pads | Screen::Fx)
                && matches!(ev, midi::Event::Param { .. } | midi::Event::Step { .. })))
        .then(|| self.before());
        self.midi_event_inner(ev, released, raw_key);
        self.sync_panel_target();
        if let Some(b) = before {
            let what = match self.lit_now() {
                Some(Lit::Pad(p)) => self.param_step_name(p),
                _ => "change".into(),
            };
            self.note_step(b, what);
        }
        self.cuts_changed(cuts);
        self.follow_loop();
        let now = Instant::now();
        self.value_after(value_before, now);
        if let Some(shown) = self.value_popup.as_mut().filter(|shown| shown.at == now) {
            shown.knob = knob;
        }
    }

    fn panel_target_cell(&mut self, cell: u8) -> bool {
        use super::panel::Target;
        let Some(target) = self.panel_target() else {
            return false;
        };
        let menu = self.panel.target_menu == Some(target);
        if cell == 15 {
            self.panel.target_menu = (!menu).then_some(target);
            self.panel.first_actions();
            return true;
        }
        if menu {
            return false;
        }
        // Targets are the 16 pads of the playing layer; a hold gives only the commands (134).
        match cell {
            4 if target == Target::Assign => {
                self.apply(Action::ChopAssignOrder);
            }
            13 => {
                self.apply(Action::Back);
            }
            14 => {
                self.apply(Action::Undo);
            }
            16 if target == Target::Assign => {
                self.apply(Action::ChopCommit);
            }
            _ => {}
        }
        true
    }

    /// Pads read by the device numbers while an SMC-PAD Master is connected (134).
    pub(super) fn sync_device_numbers(&self) {
        let smc = self
            .midi_devices
            .iter()
            .any(|(port, _)| port.contains("SMC-PAD-Master"));
        tui::pads::set_device_numbers(smc);
        tui::pads::set_row_knobs(smc.then_some(super::values::PAD_EDIT_KNOBS));
        tui::pads::set_fx_knobs(smc.then_some(super::values::FX_KNOBS));
    }

    fn panel_back(&mut self) {
        let esc = event::KeyEvent::new(event::KeyCode::Esc, event::KeyModifiers::NONE);
        if self.pad_naming.is_some() {
            self.pad_name_key(&esc);
        } else if self.project_picker.is_some() {
            self.project_picker_key(&esc);
        } else if self.path_picker.is_some() {
            self.apply(Action::Back);
        } else if self.screen == Screen::Files && self.browser.editing {
            self.filter_key(&esc);
        } else if let Some(learn) = self.screen_learn.as_mut() {
            if learn.pending.is_some() || learn.clearing {
                learn.pending = None;
                learn.clearing = false;
            } else {
                self.screen_learn = None;
            }
        } else if self.midi_run.is_some()
            || self.learning.is_some()
            || self.help_open
            || self.edit
            || self.pattern_view.overview
            || self.pattern_view.edit
            || self.pattern_panel.is_some()
            || self.plan.is_some()
            || self.browser.in_hand.is_some()
            || self.pad_in_hand.is_some()
            || self.copy_in_hand.is_some()
            || self.pads.marking
            || !self.pads.marked.is_empty()
            || self.erase_pads
            || (self.screen == Screen::Files && !self.browser.filter.is_empty())
        {
            self.apply(Action::Back);
        } else {
            self.panel.hide();
        }
    }

    fn midi_event_inner(
        &mut self,
        ev: midi::Event,
        released: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
        raw_key: Option<input::midi::Key>,
    ) {
        let now = Instant::now();
        // the learn mode takes the controls to bind them; nothing acts (47)
        if self.screen_learn.is_some()
            && matches!(
                ev,
                midi::Event::PadHit { .. }
                    | midi::Event::Action(_)
                    | midi::Event::Param { .. }
                    | midi::Event::Step { .. }
            )
        {
            return;
        }
        match ev {
            midi::Event::SetupStatus { ready, message } => {
                self.help_overlay.controller.smc_status = Some((ready, message.clone()));
                self.post_system_notice(
                    if ready { Level::Info } else { Level::Warn },
                    (message, now),
                );
            }
            midi::Event::Connected { port, profile } => {
                self.post_system_notice(
                    Level::Info,
                    (format!("MIDI connected: {port} ({profile} profile)"), now),
                );
                self.seed_device(&port, &profile, false);
                self.midi_devices.push((port, profile));
                self.sync_device_numbers();
            }
            midi::Event::Disconnected { port } => {
                self.panel.disconnect(&port);
                self.help_overlay.controller.smc_shift_warned.remove(&port);
                if port.contains("SMC-PAD-Master") {
                    self.help_overlay.controller.smc_status = None;
                    if self
                        .help_overlay
                        .controller
                        .smc_last_knob
                        .as_ref()
                        .is_some_and(|last| last.port == port)
                    {
                        self.help_overlay.controller.smc_last_knob = None;
                    }
                }
                self.post_system_notice(Level::Warn, (format!("MIDI disconnected: {port}"), now));
                self.midi_devices.retain(|(p, _)| *p != port);
                self.sync_device_numbers();
            }
            midi::Event::Message {
                port,
                profile,
                msg,
                meaning,
            } => {
                let mut answered_learning = false;
                if port.contains("SMC-PAD-Master")
                    && let input::midi::Msg::NoteOn { note, ch, .. } = msg
                    && (matches!(note, 52 | 55) || (ch == 10 && matches!(note, 36 | 39)))
                {
                    // The derived pad event keeps its original MIDI key across PAD BANK routing.
                    let key = raw_key.unwrap_or_else(|| msg.key());
                    let second_bank = matches!(note, 52 | 55);
                    let cancel = matches!(note, 36 | 52);
                    self.question_pads.remove(&(port.clone(), key));
                    if self.ask.is_some()
                        && (second_bank
                            || matches!(
                                self.ask,
                                Some(Ask::Exit { .. } | Ask::ExitRecovery { .. })
                            ))
                    {
                        self.question_pads.insert((port.clone(), key));
                        self.apply(if cancel {
                            Action::Back
                        } else {
                            Action::Confirm
                        });
                    } else if second_bank
                        && self
                            .screen_learn
                            .as_ref()
                            .is_some_and(|learn| learn.pending.is_some() || learn.clearing)
                    {
                        self.question_pads.insert((port.clone(), key));
                        if cancel {
                            if let Some(learn) = self.screen_learn.as_mut() {
                                learn.pending = None;
                                learn.clearing = false;
                            }
                        } else {
                            self.learn_confirm();
                        }
                        answered_learning = true;
                    } else if second_bank
                        && self
                            .learning
                            .as_ref()
                            .is_some_and(|learn| learn.pending.is_some())
                    {
                        self.question_pads.insert((port.clone(), key));
                        if cancel {
                            self.learning = None;
                        } else {
                            self.learned_save();
                        }
                        answered_learning = true;
                    } else if key == input::midi::Key::Note(52)
                        && (self.screen_learn.is_some() || self.learning.is_some())
                    {
                        // Device pad 1 is Back: with no choice pending it leaves learning (134).
                        self.question_pads.insert((port.clone(), key));
                        self.screen_learn = None;
                        self.learning = None;
                        answered_learning = true;
                    }
                }
                self.help_overlay.controller.last_port = Some(port.clone());
                // a run takes every new control for the next pad or bank; a control
                // already taken in the run waits for another (28)
                if let Some(r) = self.midi_run.as_mut()
                    && !r.taken.contains(&msg.key())
                    && let Some((profile, key)) = r.learner.feed(&profile, msg, now)
                {
                    self.midi_run_hit(profile, key);
                }
                if !answered_learning
                    && let Some(l) = self.learning.as_mut()
                    && let Some((profile, key)) = l.feed(&profile, msg, now)
                {
                    self.learned(profile, key);
                }
                // a button pressed for a parameter: the knob learner never hears a press, so
                // the refusal is said here (57)
                if !answered_learning
                    && let Some(l) = self.screen_learn.as_mut()
                    && matches!(msg, input::midi::Msg::NoteOn { .. })
                    && l.picked.as_ref().is_some_and(|(n, _)| action_info(n).knob)
                {
                    l.refused = true;
                    l.pending = None;
                }
                if !answered_learning
                    && let Some(l) = self.screen_learn.as_mut()
                    && let Some((profile, key)) =
                        l.learner.as_mut().and_then(|r| r.feed(&profile, msg, now))
                {
                    self.learn_bind(profile, key);
                }
                super::diag::input(|| format!("midi {port} {msg} -> {meaning}"));
                if self.input_log.len() == INPUT_LOG {
                    self.input_log.remove(0);
                }
                self.input_log
                    .push(format!("{port}   {msg}   -> {meaning}"));
            }
            // The panel shows while PAD BANK is held (131); overlays hide it in the snapshot.
            midi::Event::SmcPadBank {
                port,
                setting,
                held,
            } => {
                if !held {
                    self.panel.disconnect(&port);
                } else if self
                    .midi
                    .as_ref()
                    .is_some_and(|midi| midi.pad_bank_shortcut_is_current(setting))
                    && self.panel.owner().is_none_or(|owner| owner == port)
                {
                    self.panel.hide();
                    self.panel.reveal(&port);
                }
            }
            midi::Event::PanelCell { port, cell } => {
                if !port.contains("SMC-PAD-Master") {
                    return;
                }
                if self.ask.is_some() {
                    match cell {
                        13 => {
                            self.apply(Action::Back);
                        }
                        16 => {
                            self.apply(Action::Confirm);
                        }
                        _ => {}
                    }
                    return;
                }
                if self.project_picker.is_some() || self.path_picker.is_some() {
                    match cell {
                        13 => self.panel_back(),
                        16 => {
                            self.apply(Action::Confirm);
                        }
                        _ => {}
                    }
                    return;
                }
                if self.help_open {
                    if self.panel.owner() != Some(&port) {
                        return;
                    }
                    if cell == 13 {
                        self.panel_back();
                    } else {
                        self.help_panel_cell(cell);
                    }
                    return;
                }
                if self.pad_naming.is_some()
                    || self.browser.editing
                    || self.screen_learn.is_some()
                    || self.learning.is_some()
                    || self.midi_run.is_some()
                {
                    if cell == 13 && self.panel.owner() == Some(&port) {
                        self.panel_back();
                    }
                    return;
                }
                if self.panel.press(&port, cell) == super::panel::Press::Cell(cell) {
                    self.panel.flash = Some((cell, Instant::now()));
                    let view = self.panel_view();
                    self.panel.sync_view(view);
                    if self.panel_target_cell(cell) {
                        return;
                    }
                    let choice = if self.assign_order_cell(cell) {
                        super::panel::Choice::Action(Action::ChopAssignOrder)
                    } else {
                        self.panel.choice(view, cell)
                    };
                    match choice {
                        super::panel::Choice::Action(action) => {
                            self.apply(action);
                            // A screen hit, even on the current screen, shows its actions.
                            if matches!(
                                action,
                                Action::ScreenPads
                                    | Action::ScreenFiles
                                    | Action::ScreenChop
                                    | Action::ScreenPattern
                                    | Action::ScreenFx
                                    | Action::ScreenSettings
                            ) {
                                self.panel.first_actions();
                            }
                            if action == Action::HelpToggle && self.help_open {
                                self.help_overlay.page = super::help::Page::Controller;
                                self.controller_start();
                            }
                        }
                        super::panel::Choice::Exit => {
                            self.ask = Some(Ask::Exit {
                                lines: vec!["Exit ptsr?".into(), "P4 exit · P1 stay".into()],
                            });
                        }
                        super::panel::Choice::Back => {
                            self.panel_back();
                        }
                        super::panel::Choice::Dim => {}
                    }
                }
            }
            midi::Event::PadHit {
                abs,
                velocity,
                ui_press,
                sound,
            } => {
                if let Origin::Device(port) = &self.notice_origin
                    && self.panel.owner() == Some(port)
                {
                    self.panel.hide();
                }
                if self.pattern_view.edit
                    && let Some(id) = ui_press
                    && !self.pattern_view.midi_presses.insert((abs, id))
                {
                    return;
                }
                let tracked = released.and_then(|released| {
                    let Command::PatternRecordEnd { pass } =
                        self.audio.as_ref()?.pattern_capture.finish_for_save()?
                    else {
                        return None;
                    };
                    Some((
                        pass,
                        std::sync::Arc::new(engine::capture::EraseHold::new(released)),
                    ))
                });
                if let Some(pad) = self.pad_hit(
                    abs,
                    velocity,
                    pads::HitSource::Midi,
                    sound,
                    tracked.as_ref(),
                ) && let Some(id) = ui_press
                {
                    let deferred =
                        matches!(sound, midi::PadSound::UiUnattempted { deferred: true });
                    let pattern = tracked.map(|(_, hold)| hold);
                    let held = self.midi_ui_held.entry(id).or_default();
                    if let Some(hold) = held.iter_mut().find(|hold| hold.abs == pad) {
                        hold.deferred |= deferred;
                        hold.pattern = pattern;
                    } else {
                        held.push(UiPadHold {
                            abs: pad,
                            deferred,
                            pattern,
                        });
                    }
                }
            }
            midi::Event::PadRelease { abs, direct } => {
                self.pattern_view
                    .midi_presses
                    .retain(|(pad, _)| *pad != abs);
                if direct {
                    self.midi_ui_held.retain(|_, pads| {
                        // The older direct release may precede a deferred UI press
                        // in the callback. Keep the new UI hold's own release.
                        pads.retain(|hold| hold.abs != abs || hold.deferred);
                        !pads.is_empty()
                    });
                    if self
                        .midi_ui_held
                        .values()
                        .flatten()
                        .any(|hold| hold.abs == abs)
                    {
                        return;
                    }
                    self.audio_held[abs] = None;
                }
                self.pads.release(abs);
            }
            midi::Event::UiPadRelease {
                abs: _captured_abs,
                id,
            } => {
                self.pattern_view
                    .midi_presses
                    .retain(|(_, press)| *press != id);
                if let Some(pads) = self.midi_ui_held.remove(&id) {
                    for hold in pads {
                        if !hold.deferred || self.audio_held[hold.abs].is_some() {
                            let command = match hold.pattern {
                                Some(pattern) => Command::PatternPadRelease {
                                    pad: hold.abs,
                                    hold: pattern,
                                },
                                None => Command::PadRelease { pad: hold.abs },
                            };
                            self.send(command);
                            self.pads.release(hold.abs);
                        }
                    }
                }
            }
            midi::Event::Action(action) => {
                // quitting from a controller is not offered
                let action = if self.path_picker.is_some()
                    && matches!(&self.notice_origin, Origin::Device(port) if port.contains("SMC-PAD-Master"))
                    && let Action::SettingsRow(step) = action
                {
                    Action::SettingsValue(step)
                } else {
                    action
                };
                self.apply(action);
            }
            midi::Event::Step {
                port,
                control,
                target,
                steps,
                fine,
                ..
            } => {
                self.midi_step(&port, &control, target, steps, fine);
            }
            // open pattern controls: edit knobs 1-3 turn A1, other edit knobs wait (98b)
            midi::Event::Param {
                port,
                control,
                param,
                value,
                takeover,
            } if self.pattern_panel.is_some()
                && !self.transport_input_blocked()
                && !matches!(
                    param,
                    Param::MasterVolume
                        | Param::Tempo
                        | Param::PatternBars
                        | Param::PatternQuantize
                        | Param::PatternStrength
                ) =>
            {
                if let Param::Slot(k) = param
                    && let Some(&setting) = super::pattern_record::Setting::ALL.get(k as usize)
                {
                    self.pattern_knob(setting, &port, &control, value, takeover);
                }
            }
            midi::Event::Param {
                port,
                control,
                param:
                    param @ (Param::PatternBars | Param::PatternQuantize | Param::PatternStrength),
                value,
                takeover,
            } => {
                if let Some(setting) = super::pattern_record::Setting::from_param(param) {
                    self.pattern_knob(setting, &port, &control, value, takeover);
                }
            }
            midi::Event::Param {
                port,
                control,
                param: Param::Slot(k),
                value,
                takeover,
            } => {
                if let Some(param) = self.slot_param(k) {
                    self.midi_event(midi::Event::Param {
                        port,
                        control,
                        param,
                        value,
                        takeover,
                    });
                }
            }
            // the FX pages (56): the pad's drive and sends follow the scope, a slot is one
            // for all pads
            midi::Event::Param {
                port,
                control,
                param: Param::PadEffect(i),
                value,
                takeover,
            } => {
                let Some(&p) = EFFECT_PARAMS.get(i as usize) else {
                    return;
                };
                if self.screen != Screen::Pads && !(self.screen == Screen::Fx && p.slot().is_some())
                {
                    return;
                }
                let who = if p.slot().is_some() {
                    0xFFFF
                } else {
                    self.pads.selected_abs() as u64
                };
                // Type changes give size/time and tone/feedback distinct takeover targets.
                let kind = p.slot().map_or(0, |k| self.pads.fx[k].kind as u64);
                let target = (5u64 << 40) | (kind << 32) | ((i as u64) << 16) | who;
                let current = self.pads.fx_pos(p);
                let value = if takeover {
                    self.takeover.apply(&port, &control, target, value, current)
                } else {
                    Ok(value)
                };
                match value {
                    Ok(v) => {
                        self.pickup = None;
                        self.lit = Some((Lit::Pad(p), now));
                        let cmds = self.pads.set_fx_pos(p, v);
                        if !cmds.is_empty() {
                            for cmd in cmds {
                                self.send(cmd);
                            }
                            self.dirty();
                        }
                    }
                    Err(m) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        let name = self.pads.name(p);
                        self.pickup = Some((format!("turn {control} {way} to {name}"), now));
                    }
                }
            }
            // a switch, not a value: no pickup; a slice has one page (54)
            midi::Event::Param {
                param: Param::Page,
                value,
                ..
            } => {
                if self.screen == Screen::Fx {
                    self.select_fx(usize::from(value.clamp(0.0, 1.0) >= 0.5));
                } else if self.edit && self.screen == Screen::Pads {
                    self.pads.set_page_pos(value);
                    self.lit = Some((Lit::Pad(self.pads.param), now));
                }
            }
            midi::Event::Param {
                port,
                control,
                param: param @ (Param::SliceStart | Param::SliceEnd),
                value,
                takeover,
            } => {
                let end = param == Param::SliceEnd;
                let Some(c) = self.chop.as_ref().filter(|_| self.screen == Screen::Chop) else {
                    return;
                };
                // each edge of each slice is its own takeover target, so selecting another
                // slice makes the knob catch up instead of jumping the cut
                let target = (1 << 32) | ((c.selected as u64) << 1) | end as u64;
                let current = c.edge_pos(end);
                let value = if takeover {
                    self.takeover.apply(&port, &control, target, value, current)
                } else {
                    Ok(value)
                };
                match value {
                    Ok(v) => {
                        if let Some(c) = self.chop.as_mut() {
                            c.set_edge(end, v);
                        }
                        self.slice_end = end;
                        self.lit = Some((Lit::Bound(end), now));
                        let changed = self.sync_links();
                        self.say_changed(changed, false);
                    }
                    Err(m) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        self.post_notice(
                            Level::Info,
                            (format!("turn {control} {way} to the cut"), now),
                        );
                    }
                }
            }
            midi::Event::Param {
                port,
                control,
                param: param @ (Param::Zoom | Param::Cursor | Param::RegionStart | Param::RegionEnd),
                value,
                takeover,
            } => {
                let Some(c) = self.chop.as_ref().filter(|_| self.screen == Screen::Chop) else {
                    return;
                };
                // every chop knob of every file is its own takeover target, apart from the
                // pad numbers and the slice bounds: a knob switched from another parameter or
                // another file catches up instead of jumping the view or the region
                let (kind, current) = match param {
                    Param::Zoom => (1, c.zoom_pos()),
                    Param::Cursor => (2, c.cursor_pos()),
                    Param::RegionStart => (3, c.region_edge_pos(false)),
                    Param::RegionEnd => (4, c.region_edge_pos(true)),
                    _ => return,
                };
                let file = {
                    use std::hash::{Hash, Hasher};
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    c.file.hash(&mut h);
                    h.finish() & 0xFFFF_FFFF
                };
                let target = (3 << 60) | (file << 4) | kind;
                let value = if takeover {
                    self.takeover.apply(&port, &control, target, value, current)
                } else {
                    Ok(value)
                };
                match value {
                    Ok(v) => {
                        if let Some(c) = self.chop.as_mut() {
                            match param {
                                Param::Zoom => c.set_zoom(v),
                                Param::Cursor => c.set_cursor_pos(v),
                                Param::RegionStart => c.set_region_edge(false, v),
                                _ => c.set_region_edge(true, v),
                            }
                        }
                        let changed = self.sync_links();
                        self.say_changed(changed, false);
                    }
                    Err(m) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        self.post_notice(Level::Info, (format!("turn {control} {way}"), now));
                    }
                }
            }
            midi::Event::Param {
                port,
                control,
                param: Param::Threshold,
                value,
                takeover,
            } => {
                let Some(c) = self.chop.as_ref().filter(|_| self.screen == Screen::Chop) else {
                    return;
                };
                // the threshold knob of one file catches up on its own, as every chop knob
                let file = {
                    use std::hash::{Hash, Hasher};
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    c.file.hash(&mut h);
                    h.finish() & 0xFFFF_FFFF
                };
                let target = (5 << 60) | (file << 4);
                let value = if takeover {
                    self.takeover
                        .apply(&port, &control, target, value, c.threshold_pos())
                } else {
                    Ok(value)
                };
                match value {
                    Ok(v) => {
                        let mut said = None;
                        if let Some(c) = self.chop.as_mut()
                            && c.set_threshold(v)
                        {
                            said = Some((c.threshold_db, c.markers.len()));
                        }
                        let changed = self.sync_links();
                        if let Some((db, n)) = said {
                            let cuts = if n == 1 { "cut" } else { "cuts" };
                            let mut text = format!("{n} {cuts} at {db:.0} dB");
                            if changed > 0 {
                                text.push_str(&format!("; {changed} pads changed; Ctrl+Z returns"));
                            }
                            self.post_notice(Level::Info, (text, now));
                        }
                    }
                    Err(m) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        self.post_notice(Level::Info, (format!("turn {control} {way}"), now));
                    }
                }
            }
            midi::Event::Param {
                port,
                control,
                param: param @ (Param::PadSource | Param::PadStart | Param::PadEnd),
                value,
                takeover,
            } => {
                let abs = self.pads.selected_abs();
                let Some((start, end, len)) = self.pad_bounds(abs) else {
                    return;
                };
                let list = self.source_list(abs);
                let last = list.len().saturating_sub(1).max(1) as f32;
                let (kind, current) = match param {
                    Param::PadSource => {
                        let at = list.iter().position(|&s| s == self.pads.source(abs));
                        (1, at.unwrap_or(0) as f32 / last)
                    }
                    Param::PadStart => (2, start as f32 / len as f32),
                    _ => (3, end as f32 / len as f32),
                };
                // apart from every other knob target, per pad and per parameter
                let target = (4 << 60) | ((abs as u64) << 4) | kind;
                let value = if takeover {
                    self.takeover.apply(&port, &control, target, value, current)
                } else {
                    Ok(value)
                };
                match (value, param) {
                    (Ok(v), Param::PadSource) => {
                        let i = (v.clamp(0.0, 1.0) * last).round() as usize;
                        if let Some(&s) = list.get(i) {
                            self.set_source(abs, s);
                        }
                    }
                    (Ok(v), _) => {
                        let to = (v.clamp(0.0, 1.0) * len as f32).round() as usize;
                        self.move_bound(abs, param == Param::PadEnd, to);
                    }
                    (Err(m), _) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        self.pickup = Some((format!("turn {control} {way}"), now));
                    }
                }
            }
            midi::Event::Param {
                port,
                control,
                param,
                value,
                takeover,
            } => {
                // a knob position 0..1 spans the range of the parameter
                let span = (pads::MAX_DB - pads::MIN_DB) as f32;
                if param == Param::Tempo && self.transport_input_blocked() {
                    return;
                }
                let (current, target, shown) = match param {
                    Param::Tempo => (
                        self.tempo_position(),
                        super::transport::TEMPO_TARGET,
                        format!("{:.1} BPM", self.transport.bpm),
                    ),
                    Param::PadVolume => {
                        let abs = self.pads.selected_abs();
                        (
                            self.pads.volume_pos(),
                            abs as u64,
                            format!("{:+} dB", self.pads.volume_db[abs]),
                        )
                    }
                    Param::PadPitch => {
                        let abs = self.pads.selected_abs();
                        (
                            self.pads.pitch_pos(),
                            abs as u64,
                            format!("{:+} st", self.pads.pitch[abs]),
                        )
                    }
                    Param::MasterVolume => {
                        let pos = (self.master_db - pads::MIN_DB) as f32 / span;
                        (pos, u64::MAX, format!("{:+} dB", self.master_db))
                    }
                    Param::PadAttack | Param::PadDecay | Param::PadRelease => {
                        let p = pad_env(param);
                        let abs = self.pads.selected_abs();
                        let ms = match p {
                            pads::Param::Attack => self.pads.attack_ms[abs],
                            pads::Param::Decay => self.pads.decay_ms[abs],
                            _ => self.pads.release_ms[abs],
                        };
                        // one knob slot turns volume on TONE and attack on ENV: another
                        // target per parameter, so the page change makes the knob pick up
                        let tag = (p as u64 + 1) << 40;
                        (self.pads.env_pos(p), tag | abs as u64, format!("{ms} ms"))
                    }
                    Param::PadFilter | Param::PadCutoff | Param::PadReso => {
                        let p = pad_filter(param);
                        let abs = self.pads.selected_abs();
                        // its own target per parameter, as the envelope knobs (44)
                        let tag = (p as u64 + 1) << 40;
                        let pads = &self.pads;
                        let shown = match p {
                            pads::Param::Filter => pads::kind_name(pads.filter[abs]).to_string(),
                            pads::Param::Cutoff => pads::cutoff_text(pads.cutoff_hz[abs]),
                            _ => format!("{} %", pads.reso[abs]),
                        };
                        (self.pads.filter_pos(p), tag | abs as u64, shown)
                    }
                    Param::SliceStart
                    | Param::SliceEnd
                    | Param::Zoom
                    | Param::Cursor
                    | Param::RegionStart
                    | Param::RegionEnd
                    | Param::Threshold
                    | Param::PadSource
                    | Param::PadStart
                    | Param::PadEnd
                    | Param::Slot(_)
                    | Param::SmcPageValue(_)
                    | Param::Page
                    | Param::PatternBars
                    | Param::PatternQuantize
                    | Param::PatternStrength
                    | Param::PadEffect(_) => return,
                };
                let value = if takeover {
                    self.takeover.apply(&port, &control, target, value, current)
                } else {
                    Ok(value)
                };
                match (value, param) {
                    (Ok(v), Param::Tempo) => {
                        self.pickup = None;
                        self.set_tempo(40.0 + v.clamp(0.0, 1.0) as f64 * 260.0, false);
                    }
                    (Ok(v), Param::PadVolume) => {
                        self.pickup = None;
                        self.lit = Some((Lit::Pad(pads::Param::Volume), now));
                        let cmds = self.pads.set_volume_pos(v);
                        if !cmds.is_empty() {
                            for cmd in cmds {
                                self.send(cmd);
                            }
                            self.dirty();
                        }
                    }
                    (Ok(v), Param::PadPitch) => {
                        self.pickup = None;
                        self.lit = Some((Lit::Pad(pads::Param::Pitch), now));
                        let cmds = self.pads.set_pitch_pos(v);
                        if !cmds.is_empty() {
                            for cmd in cmds {
                                self.send(cmd);
                            }
                            self.dirty();
                        }
                    }
                    (Ok(v), Param::PadFilter | Param::PadCutoff | Param::PadReso) => {
                        let p = pad_filter(param);
                        self.pickup = None;
                        self.lit = Some((Lit::Pad(p), now));
                        let cmds = self.pads.set_filter_pos(p, v);
                        if !cmds.is_empty() {
                            for cmd in cmds {
                                self.send(cmd);
                            }
                            self.dirty();
                        }
                    }
                    (Ok(v), Param::PadAttack | Param::PadDecay | Param::PadRelease) => {
                        let p = pad_env(param);
                        self.pickup = None;
                        self.lit = Some((Lit::Pad(p), now));
                        let cmds = self.pads.set_env_pos(p, v);
                        if !cmds.is_empty() {
                            for cmd in cmds {
                                self.send(cmd);
                            }
                            self.dirty();
                        }
                    }
                    (
                        Ok(_),
                        Param::SliceStart
                        | Param::SliceEnd
                        | Param::Zoom
                        | Param::Cursor
                        | Param::RegionStart
                        | Param::RegionEnd
                        | Param::Threshold
                        | Param::PadSource
                        | Param::PadStart
                        | Param::PadEnd
                        | Param::Slot(_)
                        | Param::SmcPageValue(_)
                        | Param::Page
                        | Param::PatternBars
                        | Param::PatternQuantize
                        | Param::PatternStrength
                        | Param::PadEffect(_),
                    ) => {}
                    (Ok(v), Param::MasterVolume) => {
                        let db = (pads::MIN_DB as f32 + v.clamp(0.0, 1.0) * span).round() as i8;
                        if db != self.master_db {
                            self.set_master(db);
                            self.dirty();
                        }
                    }
                    (Err(m), _) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        let hint = format!("turn {control} {way} to {shown}");
                        match param {
                            Param::PadVolume
                            | Param::PadPitch
                            | Param::PadAttack
                            | Param::PadDecay
                            | Param::PadRelease
                            | Param::PadFilter
                            | Param::PadCutoff
                            | Param::PadReso => self.pickup = Some((hint, now)),
                            // the master hint is needed on every screen, not only on the pads one
                            Param::MasterVolume => {
                                self.post_notice(Level::Info, (format!("Master: {hint}"), now))
                            }
                            Param::Tempo => {
                                self.post_notice(Level::Info, (format!("Tempo: {hint}"), now))
                            }
                            Param::SliceStart
                            | Param::SliceEnd
                            | Param::Zoom
                            | Param::Cursor
                            | Param::RegionStart
                            | Param::RegionEnd
                            | Param::Threshold
                            | Param::PadSource
                            | Param::PadStart
                            | Param::PadEnd
                            | Param::Slot(_)
                            | Param::SmcPageValue(_)
                            | Param::Page
                            | Param::PatternBars
                            | Param::PatternQuantize
                            | Param::PatternStrength
                            | Param::PadEffect(_) => {}
                        }
                    }
                }
            }
        }
    }

    fn step_threshold(&mut self, step_db: i32) {
        if self.plan.is_some() || self.help_open || self.screen_learn.is_some() {
            return;
        }
        if let Some(chop) = self.chop.as_mut() {
            let next = (chop.threshold_db + step_db as f32).clamp(-40.0, 0.0);
            chop.set_threshold(-next / 40.0);
        }
        let changed = self.sync_links();
        self.say_changed(changed, false);
    }

    fn smc_list(&mut self, next: bool) {
        // A visible panel takes LEFT and RIGHT for its views (127); a question and the
        // target view keep their own cells.
        if self.panel.visible()
            && self.panel_input_available()
            && self.ask.is_none()
            && self.panel_snapshot().is_some_and(|panel| !panel.target)
        {
            let view = self.panel_view();
            self.panel.sync_view(view);
            self.panel.move_view(if next { 1 } else { -1 });
            return;
        }
        if self.help_open && self.ask.is_none() {
            self.help_tab(next);
            return;
        }
        if self.transport_input_blocked() || self.help_open || self.pattern_panel.is_some() {
            return;
        }
        let action = match self.screen {
            Screen::Pads if self.pattern_view.overview => {
                self.move_pattern_position(if next { 1 } else { -1 }, true);
                return;
            }
            Screen::Pads if self.edit => {
                if next {
                    Action::PageNext
                } else {
                    Action::PagePrev
                }
            }
            Screen::Pads => {
                if next {
                    Action::BankNext
                } else {
                    Action::BankPrev
                }
            }
            Screen::Files => {
                if next {
                    Action::BrowserEnter
                } else {
                    Action::BrowserParent
                }
            }
            Screen::Chop => {
                if next {
                    Action::BankNext
                } else {
                    Action::BankPrev
                }
            }
            Screen::Fx => {
                self.select_fx(usize::from(next));
                return;
            }
            Screen::Settings => {
                self.settings_tab = (self.settings_tab + if next { 1 } else { 3 }) % 4;
                self.learning = None;
                self.midi_run = None;
                return;
            }
        };
        self.run(action);
    }

    fn smc_move_position(&mut self, steps: i32) {
        if self.transport_input_blocked() || self.help_open {
            return;
        }
        if self.screen == Screen::Pads && self.pattern_view.overview && self.pattern_panel.is_none()
        {
            self.move_pattern_position(steps, false);
            return;
        }
        let right = steps > 0;
        let action = if self.pattern_panel.is_some() {
            if right {
                Action::PatternParamNext
            } else {
                Action::PatternParamPrev
            }
        } else {
            match self.screen {
                Screen::Fx => {
                    if right {
                        Action::ParamNext
                    } else {
                        Action::ParamPrev
                    }
                }
                _ => return,
            }
        };
        for _ in 0..steps.unsigned_abs() {
            self.run(action);
        }
    }

    fn midi_step(
        &mut self,
        port: &str,
        control: &str,
        target: input::midi::StepTarget,
        steps: i32,
        fine: bool,
    ) {
        use input::midi::{ListTarget, StepTarget};
        if steps == 0 || self.ask.is_some() {
            return;
        }
        if self.plan.is_some()
            && port.contains("SMC-PAD-Master")
            && matches!(
                target,
                StepTarget::Param(
                    Param::SliceStart | Param::SliceEnd | Param::RegionStart | Param::RegionEnd
                )
            )
        {
            return;
        }
        let smc_selector = port.contains("SMC-PAD-Master") && matches!(control, "e1" | "cc 30");
        if self.path_picker.is_some() {
            if smc_selector {
                for _ in 0..steps.unsigned_abs() {
                    self.picker_key(Action::SettingsRow(if steps > 0 { 1 } else { -1 }));
                }
            }
            return;
        }
        if let Some(picker) = self.project_picker.as_mut() {
            if (smc_selector || target == StepTarget::List(ListTarget::Projects))
                && picker.naming.is_none()
            {
                picker.selected = (picker.selected as i64 + i64::from(steps))
                    .clamp(0, picker.entries.len().saturating_sub(1) as i64)
                    as usize;
            }
            return;
        }
        if port.contains("SMC-PAD-Master")
            && matches!(control, "e6" | "cc 35")
            && target == StepTarget::List(ListTarget::Page)
        {
            if self.panel.owner() == Some(port)
                && !self.help_open
                && self.pad_naming.is_none()
                && !self.browser.editing
                && self.screen_learn.is_none()
                && self.learning.is_none()
                && self.midi_run.is_none()
            {
                let view = self.panel_view();
                self.panel.sync_view(view);
                self.panel.move_view(steps);
            }
            return;
        }
        if let Some(param) = smc_live_pad_param(port, control, target) {
            if !self.transport_input_blocked() && !self.help_open {
                self.pickup = None;
                let commands = self.pads.step_selected_value(param, steps);
                if !commands.is_empty() {
                    for command in commands {
                        self.send(command);
                    }
                    self.dirty();
                    self.lit = Some((Lit::Pad(param), Instant::now()));
                }
            }
            return;
        }
        if self.pattern_panel.is_some()
            && (matches!(target, StepTarget::List(ListTarget::Page))
                || matches!(target, StepTarget::Param(param) if !matches!(param,
                    Param::Slot(_) | Param::MasterVolume | Param::Tempo
                        | Param::PatternBars | Param::PatternQuantize | Param::PatternStrength)))
        {
            return;
        }
        self.pickup = None;
        let right = steps > 0;
        let action = match target {
            StepTarget::List(ListTarget::Files) if self.screen == Screen::Files => {
                self.browser.step(steps as isize);
                self.preview_selection();
                return;
            }
            StepTarget::List(ListTarget::Projects) => {
                if let Some(picker) = self.project_picker.as_mut().filter(|p| p.naming.is_none()) {
                    picker.selected = (picker.selected as i64 + i64::from(steps))
                        .clamp(0, picker.entries.len().saturating_sub(1) as i64)
                        as usize;
                }
                return;
            }
            StepTarget::List(ListTarget::Settings) if self.screen == Screen::Settings => {
                Some(Action::SettingsRow(if right { 1 } else { -1 }))
            }
            StepTarget::List(ListTarget::Page)
                if self.screen == Screen::Fx || (self.screen == Screen::Pads && self.edit) =>
            {
                Some(if right {
                    Action::PageNext
                } else {
                    Action::PagePrev
                })
            }
            StepTarget::List(ListTarget::SmcPosition)
                if self.screen == Screen::Pads && self.edit && !self.pattern_view.overview =>
            {
                // In pad edit, knob 1 is the fourth page value.
                return self.midi_step(
                    port,
                    control,
                    StepTarget::Param(Param::SmcPageValue(1)),
                    steps,
                    fine,
                );
            }
            StepTarget::List(ListTarget::SmcPosition) => {
                self.smc_move_position(steps);
                return;
            }
            StepTarget::List(ListTarget::Bank) => Some(if right {
                Action::BankNext
            } else {
                Action::BankPrev
            }),
            StepTarget::Param(Param::Page) => {
                return self.midi_step(
                    port,
                    control,
                    StepTarget::List(ListTarget::Page),
                    steps,
                    fine,
                );
            }
            StepTarget::Param(Param::SmcPageValue(knob)) => {
                if !self.transport_input_blocked()
                    && !self.help_open
                    && let Some(param) = self.smc_page_value(knob)
                {
                    self.midi_step(port, control, StepTarget::Param(param), steps, fine);
                }
                return;
            }
            StepTarget::Param(Param::Slot(k)) => {
                if self.pattern_panel.is_some() {
                    if let Some(&setting) = super::pattern_record::Setting::ALL.get(k as usize) {
                        self.pattern_step(setting, steps);
                    }
                    return;
                }
                if let Some(param) = self.slot_param(k) {
                    return self.midi_step(port, control, StepTarget::Param(param), steps, fine);
                }
                return;
            }
            StepTarget::Param(
                param @ (Param::PatternBars | Param::PatternQuantize | Param::PatternStrength),
            ) => {
                if let Some(setting) = super::pattern_record::Setting::from_param(param) {
                    self.pattern_step(setting, steps);
                }
                return;
            }
            StepTarget::Param(Param::MasterVolume) => {
                Some(Action::MasterVolume(if right { 1 } else { -1 }))
            }
            StepTarget::Param(Param::Tempo)
                if self.pattern_panel.is_some()
                    && port.contains("SMC-PAD-Master")
                    && matches!(control, "e2" | "cc 31") =>
            {
                if self.help_open {
                    return;
                }
                Some(Action::TempoStep { up: right, fine })
            }
            StepTarget::Param(Param::Tempo) => Some(Action::TempoStep { up: right, fine }),
            StepTarget::Param(Param::Zoom) if self.screen == Screen::Chop => {
                if self.plan.is_none()
                    && port.contains("SMC-PAD-Master")
                    && matches!(control, "e2" | "cc 31")
                {
                    if let Some(chop) = self.chop.as_mut() {
                        chop.zoom_quarter_steps(steps);
                    }
                    return;
                }
                Some(Action::ChopZoom { zoom_in: right })
            }
            StepTarget::Param(Param::Cursor) if self.screen == Screen::Chop => {
                Some(Action::CursorMove { right, fine: true })
            }
            StepTarget::Param(
                param @ (Param::PadVolume
                | Param::PadPitch
                | Param::PadAttack
                | Param::PadDecay
                | Param::PadRelease
                | Param::PadFilter
                | Param::PadCutoff
                | Param::PadReso
                | Param::PadSource
                | Param::PadStart
                | Param::PadEnd
                | Param::PadEffect(_)),
            ) if matches!(self.screen, Screen::Pads | Screen::Fx) => {
                let selected = match param {
                    Param::PadVolume => pads::Param::Volume,
                    Param::PadPitch => pads::Param::Pitch,
                    Param::PadAttack | Param::PadDecay | Param::PadRelease => pad_env(param),
                    Param::PadFilter | Param::PadCutoff | Param::PadReso => pad_filter(param),
                    Param::PadSource => pads::Param::Source,
                    Param::PadStart => pads::Param::Start,
                    Param::PadEnd => pads::Param::End,
                    Param::PadEffect(i) => match EFFECT_PARAMS.get(i as usize) {
                        Some(p) => *p,
                        None => return,
                    },
                    _ => return,
                };
                if self.screen == Screen::Fx && selected.slot().is_none() {
                    return;
                }
                let previous = std::mem::replace(&mut self.pads.param, selected);
                for _ in 0..steps.unsigned_abs() {
                    let action = Action::EditValue {
                        up: right,
                        fine: true,
                    };
                    if self.screen == Screen::Fx {
                        let commands = self.pads.apply(action);
                        if !commands.is_empty() {
                            for command in commands {
                                self.send(command);
                            }
                            self.dirty();
                        }
                        self.lit = Some((Lit::Pad(selected), Instant::now()));
                    } else {
                        self.run(action);
                    }
                }
                self.pads.param = previous;
                return;
            }
            StepTarget::Param(param @ (Param::SliceStart | Param::SliceEnd))
                if self.screen == Screen::Chop =>
            {
                let end = param == Param::SliceEnd;
                for _ in 0..steps.unsigned_abs() {
                    if let Some(chop) = self.chop.as_mut() {
                        let step = chop.view_step(1);
                        chop.nudge_by(end, right, step);
                    }
                }
                self.slice_end = end;
                self.lit = Some((Lit::Bound(end), Instant::now()));
                let changed = self.sync_links();
                self.say_changed(changed, false);
                return;
            }
            StepTarget::Param(Param::Threshold) if self.screen == Screen::Chop => {
                self.step_threshold(-steps);
                return;
            }
            StepTarget::Param(param @ (Param::RegionStart | Param::RegionEnd))
                if self.screen == Screen::Chop =>
            {
                let end = param == Param::RegionEnd;
                if let Some(chop) = self.chop.as_mut() {
                    let length = chop.sound.len().max(1) as f32;
                    let delta = chop.view_step(1) as f32 * steps as f32 / length;
                    chop.set_region_edge(end, chop.region_edge_pos(end) + delta);
                }
                let changed = self.sync_links();
                self.say_changed(changed, false);
                return;
            }
            _ => None,
        };
        if let Some(action) = action {
            for _ in 0..steps.unsigned_abs() {
                self.run(action);
            }
        }
    }

    pub(super) fn keyboard_event(
        &mut self,
        keymap: &Keymap,
        k: &event::KeyEvent,
        kitty: bool,
        keys_down: &mut input::HeldKeys,
        held: &mut HeldPads,
        mods: &mut input::HeldModifiers,
    ) -> bool {
        let old = std::mem::replace(&mut self.notice_origin, Origin::Keyboard);
        let quit = self.keyboard_event_inner(keymap, k, kitty, keys_down, held, mods);
        self.notice_origin = old;
        quit
    }

    fn keyboard_event_inner(
        &mut self,
        keymap: &Keymap,
        k: &event::KeyEvent,
        kitty: bool,
        keys_down: &mut input::HeldKeys,
        held: &mut HeldPads,
        mods: &mut input::HeldModifiers,
    ) -> bool {
        // Keep the old startup dismissal rule, but acknowledge a keymap warning only
        // after a frame showed it. A flash that expired between frames is not a read warning.
        let warning_shown = self.keymap_warning_shown;
        self.keymap_warning_shown = false;
        self.dismiss_startup();
        // Text, picker and learn modes take every key, so their early returns skip the
        // held-key accounting below; a Ghostty repeat after such a state change would then
        // count as a new press and dismiss a warning. Track held keys on every event.
        let tracked = if kitty { keys_down.repeat(k) } else { *k };
        if tracked.kind == event::KeyEventKind::Press
            && !matches!(tracked.code, event::KeyCode::Modifier(_))
        {
            self.input_sources.keyboard(Instant::now());
            self.acknowledge_notice();
        }
        if self.pad_naming.is_some() && self.pad_name_key(k) {
            return false;
        }
        if self.project_picker.is_some() {
            if k.code == event::KeyCode::Enter {
                if keymap.resolve_modal_enter(self.screen, self.edit_keys(), &tracked)
                    == Some(Action::Confirm)
                {
                    self.apply(Action::Confirm);
                }
                return false;
            }
            if self.project_picker_key(k) {
                return false;
            }
        }
        // Learn must not swallow the release of a key held before F10. New silent
        // learn keys are not in HeldPads and must not release another control's pad.
        if self.screen_learn.is_some()
            && kitty
            && let Some(action @ Action::PadRelease { pad }) =
                keymap.resolve_in(self.screen, self.edit_keys(), k)
            && held.is_held(pad)
        {
            held.filter(action);
            if let Action::PadRelease { pad } = action
                && !std::mem::take(&mut self.silent_pad_keys[pad])
            {
                for cmd in self.pads.apply(action) {
                    self.send(cmd);
                }
            }
            return false;
        }
        if self.screen_learn.is_some() && self.learn_key(keymap, k) {
            return false;
        }
        let typing = self.browser.editing && self.screen == Screen::Files;
        mods.update(k, typing);
        if typing && self.filter_key(k) {
            return false;
        }
        let k = tracked;
        if self.help_key(&k) {
            return false;
        }
        if self.help_open
            && self.screen == Screen::Pads
            && matches!(k.code, event::KeyCode::Left | event::KeyCode::Right)
            && k.modifiers == event::KeyModifiers::ALT
        {
            return false;
        }
        if warning_shown
            && k.kind == event::KeyEventKind::Press
            && !matches!(k.code, event::KeyCode::Modifier(_))
            && !self.key_notice_blocked()
        {
            self.keymap_warnings.pop_front();
        }
        let panel = self.pattern_panel.is_some() && !self.transport_input_blocked();
        let resolved = if k.code == event::KeyCode::Enter
            && (self.ask.is_some() || self.path_picker.is_some())
        {
            keymap.resolve_modal_enter(self.screen, self.edit_keys() || panel, &k)
        } else {
            keymap.resolve_in(self.screen, self.edit_keys() || panel, &k)
        };
        let Some(action) = resolved else {
            self.moved_key_notice(keymap, &k);
            return false;
        };
        if k.code == event::KeyCode::Enter && self.ask.is_some() && action == Action::Quit {
            return false;
        }
        let action = if panel {
            match action {
                Action::ParamPrev => Action::PatternParamPrev,
                Action::ParamNext => Action::PatternParamNext,
                Action::EditValue { up: false, .. } => Action::PatternValueDec,
                Action::EditValue { up: true, .. } => Action::PatternValueInc,
                // Enter on the Clear A1 row runs it; on a value row it closes controls (98b)
                Action::Back
                    if k.code == event::KeyCode::Enter
                        && self.pattern_panel == Some(super::pattern_record::CLEAR_ROW) =>
                {
                    Action::PatternClear
                }
                action => action,
            }
        } else {
            action
        };
        let action = if kitty {
            held.filter(action)
        } else {
            Some(action)
        };
        action.is_some_and(|action| self.apply(action))
    }
}

#[cfg(test)]
mod relative_tests {
    use super::*;
    use input::midi::{ListTarget, StepTarget};

    #[test]
    fn panel_back_closes_the_top_open_mode() {
        let port = "SMC-PAD-Master test";
        for mode in 0..5 {
            let mut app = crate::tests::test_app();
            app.panel.press(port, 1);
            match mode {
                0 => app.pad_naming = Some((0, "kick".into())),
                1 => {
                    app.project_picker = Some(ProjectPicker {
                        entries: vec![],
                        selected: 0,
                        naming: Some("project".into()),
                    });
                }
                2 => {
                    app.screen = Screen::Files;
                    app.browser.editing = true;
                    app.browser.set_filter("kick".into());
                }
                3 => app.screen_learn = Some(app.learn_open()),
                4 => app.help_open = true,
                _ => unreachable!(),
            }
            app.midi_event(midi::Event::PanelCell {
                port: port.into(),
                cell: 13,
            });
            assert!(app.panel.visible(), "mode {mode}");
            match mode {
                0 => assert!(app.pad_naming.is_none()),
                1 => assert!(app.project_picker.as_ref().unwrap().naming.is_none()),
                2 => assert!(!app.browser.editing && app.browser.filter.is_empty()),
                3 => assert!(app.screen_learn.is_none()),
                4 => assert!(!app.help_open),
                _ => unreachable!(),
            }
        }
    }

    fn turn(app: &mut App, target: StepTarget, steps: i32, fine: bool) {
        app.midi_event(midi::Event::Step {
            port: "Relative test".into(),
            control: "test".into(),
            target,
            steps,
            fine,
        });
    }

    /// 90 editing: "One detent matches one value key".
    #[test]
    fn one_detent_matches_one_value_key() {
        let mut keyboard = crate::tests::test_app();
        let mut controller = crate::tests::test_app();
        controller.pickup = Some(("turn knob up".into(), Instant::now()));
        keyboard.apply(Action::EditValue {
            up: true,
            fine: true,
        });
        turn(
            &mut controller,
            StepTarget::Param(Param::PadVolume),
            1,
            false,
        );
        assert_eq!(controller.pads.volume_db[0], keyboard.pads.volume_db[0]);
        assert_eq!(controller.pads.volume_db[0], 1);
        assert!(controller.pickup.is_none());
        turn(
            &mut controller,
            StepTarget::Param(Param::PadVolume),
            -2,
            false,
        );
        assert_eq!(controller.pads.volume_db[0], -1);
    }

    /// 90 editing: "Fast value turn is one Undo step".
    #[test]
    fn fast_value_turn_is_one_undo_step() {
        let mut app = crate::tests::test_app();
        app.pads.marked.extend([0, 1]);
        app.pads.volume_db[1] = pads::MAX_DB - 1;
        turn(&mut app, StepTarget::Param(Param::PadVolume), 4, false);
        assert_eq!(app.pads.volume_db[0], 4);
        assert_eq!(app.pads.volume_db[1], pads::MAX_DB);
        assert_eq!(app.history.len(), 1);
        app.apply(Action::Undo);
        assert_eq!(app.pads.volume_db[0], 0);
        assert_eq!(app.pads.volume_db[1], pads::MAX_DB - 1);
    }

    /// 90 editing: "Relative page matches Tab".
    #[test]
    fn relative_page_matches_tab() {
        let mut keyboard = crate::tests::test_app();
        let mut controller = crate::tests::test_app();
        keyboard.apply(Action::PadEdit);
        controller.apply(Action::PadEdit);
        keyboard.apply(Action::PageNext);
        turn(
            &mut controller,
            StepTarget::List(ListTarget::Page),
            1,
            false,
        );
        assert_eq!(controller.pads.param, keyboard.pads.param);
        turn(
            &mut controller,
            StepTarget::List(ListTarget::Bank),
            2,
            false,
        );
        assert_eq!(controller.pads.bank, 2);
    }

    /// 90 editing: "Fine tempo changes a tenth".
    #[test]
    fn fine_tempo_changes_a_tenth() {
        let mut app = crate::tests::test_app();
        let before = app.transport.bpm;
        turn(&mut app, StepTarget::Param(Param::Tempo), 1, true);
        assert!((app.transport.bpm - before - 0.1).abs() < 0.001);
    }

    /// 90 projects: "Relative project selection waits for confirmation".
    #[test]
    fn relative_project_selection_waits_for_confirmation() {
        let mut app = crate::tests::test_app();
        app.project_picker = Some(ProjectPicker {
            entries: vec!["A".into(), "B".into(), "C".into()],
            selected: 0,
            naming: None,
        });
        turn(&mut app, StepTarget::List(ListTarget::Projects), 5, false);
        assert_eq!(app.project_picker.as_ref().unwrap().selected, 2);
        assert!(app.project_picker.is_some());
    }

    /// 90 sample-loading: "Relative file selection previews the last row".
    #[test]
    fn relative_file_selection_previews_the_last_row() {
        let mut app = crate::tests::test_app();
        app.screen = Screen::Files;
        app.browser.entries = ["first.wav", "last.wav"]
            .map(|name| super::super::browser::Entry {
                name: name.into(),
                is_dir: false,
                favorite: None,
            })
            .to_vec();
        let before = app.load_gen[PADS];
        turn(&mut app, StepTarget::List(ListTarget::Files), 8, false);
        assert_eq!(app.browser.selected, 1);
        assert_eq!(app.load_gen[PADS], before + 1);
    }

    /// 90 audio-settings: "Relative knob moves through MIDI rows".
    #[test]
    fn relative_knob_moves_through_midi_rows() {
        let mut app = crate::tests::test_app();
        app.screen = Screen::Settings;
        app.settings_tab = 2;
        app.midi_devices.push(("Device".into(), "default".into()));
        assert!(app.midi_list().len() > 1);
        turn(&mut app, StepTarget::List(ListTarget::Settings), 1, false);
        assert_eq!(app.midi_row, 1);
    }

    /// 90 editing: "Relative edit slot follows its page".
    #[test]
    fn relative_edit_slot_follows_its_page() {
        let mut app = crate::tests::test_app();
        app.apply(Action::PadEdit);
        assert_eq!(app.slot_param(0), Some(Param::PadVolume));
        turn(&mut app, StepTarget::Param(Param::Slot(0)), 1, false);
        assert_eq!(app.pads.volume_db[0], 1);
        app.apply(Action::PageNext);
        assert_eq!(app.slot_param(0), Some(Param::PadAttack));
        let before = app.pads.attack_ms[0];
        turn(&mut app, StepTarget::Param(Param::Slot(0)), 1, false);
        assert_eq!(app.pads.attack_ms[0], before + 1);
        app.apply(Action::PageNext);
        assert_eq!(app.slot_param(0), Some(Param::PadFilter));
        let before_filter = app.pads.filter[0];
        turn(&mut app, StepTarget::Param(Param::Slot(0)), 1, false);
        assert_ne!(app.pads.filter[0], before_filter);
        assert_eq!(app.pads.attack_ms[0], before + 1);
    }

    #[test]
    fn relative_fx_slot_changes_its_bound_parameter() {
        let mut app = crate::tests::test_app();
        app.screen = Screen::Fx;
        let before = app.pads.fx[0].size;
        turn(&mut app, StepTarget::Param(Param::PadEffect(4)), 1, false);
        assert_eq!(app.pads.fx[0].size, before + 1);
    }

    #[test]
    fn relative_pattern_strength_uses_supported_step() {
        let mut app = crate::tests::test_app();
        turn(
            &mut app,
            StepTarget::Param(Param::PatternStrength),
            -1,
            false,
        );
        assert_eq!(app.pads.pattern.as_ref().unwrap().strength, 90);
    }
}

use super::{
    Action, App, Ask, CLICK_VELOCITY, Command, EFFECT_PARAMS, HeldPads, INPUT_LOG, Instant, Keymap,
    Lit, MidiListRow, Move, NUDGE_FINE_MS, NUDGE_MS, PADS, PADS_PER_BANK, Param, PathPicker,
    ProjectPicker, Rec, SETTINGS_ROWS, Screen, UiPadHold, action_info, assign_deaf, bank_key, chop,
    config, event, group_jump, held_repeats, learn, midi, pad_env, pad_filter, pad_name, pads,
    record_step,
};

impl App {
    /// Runs an action; returns `true` to quit.
    pub(super) fn apply(&mut self, action: Action) -> bool {
        let was = self.screen;
        let cuts = self.chop.as_ref().map(|c| c.saved());
        self.note_done(action);
        // a change of the pads made on the pads or files screen is a step of the undo (61)
        let undoable = matches!(self.screen, Screen::Pads | Screen::Files | Screen::Fx)
            && action != Action::Undo;
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
        if was != self.screen && self.screen == Screen::Chop {
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
        quit
    }

    fn run(&mut self, action: Action) -> bool {
        // the folder picker is open: the keys go to it before the screen (11)
        if self.path_picker.is_some() {
            self.picker_key(action);
            return false;
        }
        match action {
            Action::Quit => return true,
            // a question waits for Enter (the key of the step itself) or Esc only
            a if self.ask.is_some() => match a {
                // the settings Enter answers the reset question (59)
                Action::SettingsLearn if matches!(&self.ask, Some(Ask::ResetProfile { .. })) => {
                    if let Some(Ask::ResetProfile { name, port, .. }) = self.ask.take() {
                        self.seed_device(&port, &name, true);
                        self.flash =
                            Some((format!("{name}: bindings from its profile"), Instant::now()));
                    }
                }
                Action::ChopCommit | Action::ChopEdit | Action::BrowserHold | Action::PadEdit => {
                    match self.ask.as_ref() {
                        Some(Ask::Plan { .. }) => self.apply_plan(),
                        Some(Ask::ResetProfile { .. }) => {}
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
                    for cmd in self.pads.apply(a) {
                        self.send(cmd);
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
                self.flash = Some((
                    "assigning: Enter applies, Esc leaves".into(),
                    Instant::now(),
                ));
            }
            Action::Click => self.send(Command::Click {
                velocity: CLICK_VELOCITY,
            }),
            Action::SessionSave => {
                // the manual save speaks (22); an error flash from the save wins
                if self.save_session() {
                    self.flash = Some(("Session saved".into(), Instant::now()));
                }
            }
            Action::RecordMic(_) | Action::RecordOutputToggle => {
                match record_step(action, self.recording.map(|(s, _)| s)) {
                    Some(Rec::Start(source)) => self.record_start(source),
                    Some(Rec::Stop) => self.record_stop(),
                    None => {}
                }
            }
            Action::HelpToggle => self.help_open = !self.help_open,
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
                    self.flash = Some((
                        "pad is empty: put a sample on it first".into(),
                        Instant::now(),
                    ));
                }
            }
            Action::PadRename => {}
            Action::ProjectPick => {
                self.project_picker = Some(ProjectPicker::new(&self.paths.projects));
            }
            Action::PadsToggle => {
                self.ui.hide_pads = !self.ui.hide_pads;
                let result = match &self.user_path {
                    Some(p) => config::save_section(p, "ui", &self.ui),
                    None => Err("the pads choice is not saved: fix the config file first".into()),
                };
                if let Err(e) = result {
                    log::error!("{e}");
                    self.flash = Some((e, Instant::now()));
                }
            }
            Action::ScreenPads => self.goto(Screen::Pads),
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
                );
            }
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
                    self.flash = Some(("too close to another cut".into(), Instant::now()));
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
                        self.flash = Some((t, Instant::now()));
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
                    None => self.flash = Some(("nothing to undo".into(), Instant::now())),
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
                                self.flash = Some((
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
                            self.flash = Some((text, Instant::now()));
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
                                    self.flash = Some((text, Instant::now()));
                                }
                                None => {
                                    let db = c.threshold_db;
                                    self.flash = Some((
                                        format!("by hits stays at {db:.0} dB"),
                                        Instant::now(),
                                    ));
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
                                self.flash = Some(("nothing to undo".into(), Instant::now()));
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
                self.flash = Some(("copy dropped".into(), Instant::now()));
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
            // the edit mode is the innermost layer of the screen: Esc closes it, the screen stays
            Action::Back if self.edit => self.edit = false,
            // Esc drops the marks first, then goes back as now
            Action::Back if self.pads.drop_marks() => {}
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
                    _ => {}
                }
            }
            Action::SettingsLearn | Action::SettingsClear => {}
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
                self.flash = Some((
                    if added {
                        format!("★ {name}: favorite{whose}")
                    } else {
                        format!("{name}: no longer a favorite{whose}")
                    },
                    Instant::now(),
                ));
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
                        self.flash = Some((e, Instant::now()));
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
                if value || pad_action == Action::PadModeNext {
                    self.dirty();
                }
                if value {
                    self.lit = Some((Lit::Pad(self.pads.param), Instant::now()));
                }
                if let Some(n) = self.pads.limit_hits.take() {
                    let what = self.pads.param_name();
                    let pad = if n == 1 { "pad" } else { "pads" };
                    self.flash = Some((format!("{what}: {n} {pad} at the limit"), Instant::now()));
                } else if value {
                    // the changed value pops up, so the ear and the eye agree
                    let value = self.pads.param_value();
                    let what = self.pads.param_name();
                    self.flash = Some((format!("{what}: {value}"), Instant::now()));
                }
            }
        }
        false
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
            },
        }
    }

    pub(super) fn publish_midi(&self, shared: &midi::Shared) {
        shared.publish(self.focus());
    }

    pub(super) fn midi_ui_event(&mut self, ev: midi::UiEvent) {
        let sequence = ev.sequence();
        let preview_ready = ev.preview_ready();
        if let Some(ev) = ev.current(self.audio.as_ref().map(|audio| audio.project.as_ref())) {
            self.midi_preview_blocked = !preview_ready;
            self.midi_event(ev);
            self.midi_preview_blocked = false;
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
        if self.screen == Screen::Chop {
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
                return self
                    .try_send(Command::PadPress { pad: abs, velocity })
                    .then_some(abs);
            }
            if !from_midi {
                self.meters.hit(Instant::now());
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
        let cuts = self.chop.as_ref().map(|c| c.saved());
        // a knob on the pads screen is a step of the undo (61); a turn joins one step
        // a button runs its action through `apply`, which keeps the step itself
        let before = (matches!(self.screen, Screen::Pads | Screen::Fx)
            && matches!(ev, midi::Event::Param { .. }))
        .then(|| self.before());
        self.midi_event_inner(ev);
        if let Some(b) = before {
            let what = match self.lit_now() {
                Some(Lit::Pad(p)) => self.param_step_name(p),
                _ => "change".into(),
            };
            self.note_step(b, what);
        }
        self.cuts_changed(cuts);
        self.follow_loop();
    }

    fn midi_event_inner(&mut self, ev: midi::Event) {
        let now = Instant::now();
        // the learn mode takes the controls to bind them; nothing acts (47)
        if self.screen_learn.is_some()
            && matches!(
                ev,
                midi::Event::PadHit { .. } | midi::Event::Action(_) | midi::Event::Param { .. }
            )
        {
            return;
        }
        match ev {
            midi::Event::Connected { port, profile } => {
                self.flash = Some((format!("MIDI connected: {port} ({profile} profile)"), now));
                self.seed_device(&port, &profile, false);
                self.midi_devices.push((port, profile));
            }
            midi::Event::Disconnected { port } => {
                self.flash = Some((format!("MIDI disconnected: {port}"), now));
                self.midi_devices.retain(|(p, _)| *p != port);
            }
            midi::Event::Message {
                port,
                profile,
                msg,
                meaning,
            } => {
                // a run takes every new control for the next pad or bank; a control
                // already taken in the run waits for another (28)
                if let Some(r) = self.midi_run.as_mut()
                    && !r.taken.contains(&msg.key())
                    && let Some((profile, key)) = r.learner.feed(&profile, msg, now)
                {
                    self.midi_run_hit(profile, key);
                }
                if let Some(l) = self.learning.as_mut()
                    && let Some((profile, key)) = l.feed(&profile, msg, now)
                {
                    self.learned(profile, key);
                }
                // a button pressed for a parameter: the knob learner never hears a press, so
                // the refusal is said here (57)
                if let Some(l) = self.screen_learn.as_mut()
                    && matches!(msg, input::midi::Msg::NoteOn { .. })
                    && l.picked.as_ref().is_some_and(|(n, _)| action_info(n).knob)
                {
                    l.refused = true;
                    l.pending = None;
                }
                if let Some(l) = self.screen_learn.as_mut()
                    && let Some((profile, key)) =
                        l.learner.as_mut().and_then(|r| r.feed(&profile, msg, now))
                {
                    self.learn_bind(profile, key);
                }
                if self.input_log.len() == INPUT_LOG {
                    self.input_log.remove(0);
                }
                self.input_log
                    .push(format!("{port}   {msg}   -> {meaning}"));
            }
            midi::Event::PadHit {
                abs,
                velocity,
                ui_press,
                sound,
            } => {
                if let Some(pad) = self.pad_hit(abs, velocity, pads::HitSource::Midi, sound)
                    && let Some(id) = ui_press
                {
                    let deferred =
                        matches!(sound, midi::PadSound::UiUnattempted { deferred: true });
                    let held = self.midi_ui_held.entry(id).or_default();
                    if let Some(hold) = held.iter_mut().find(|hold| hold.abs == pad) {
                        hold.deferred |= deferred;
                    } else {
                        held.push(UiPadHold { abs: pad, deferred });
                    }
                }
            }
            midi::Event::PadRelease { abs, direct } => {
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
                if let Some(pads) = self.midi_ui_held.remove(&id) {
                    for hold in pads {
                        if !hold.deferred || self.audio_held[hold.abs].is_some() {
                            self.send(Command::PadRelease { pad: hold.abs });
                            self.pads.release(hold.abs);
                        }
                    }
                }
            }
            midi::Event::Action(action) => {
                // quitting from a controller is not offered
                self.apply(action);
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
                        self.lit = Some((Lit::Bound(end), now));
                        let changed = self.sync_links();
                        self.say_changed(changed, false);
                    }
                    Err(m) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        self.flash = Some((format!("turn {control} {way} to the cut"), now));
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
                        self.flash = Some((format!("turn {control} {way}"), now));
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
                            self.flash = Some((text, now));
                        }
                    }
                    Err(m) => {
                        let way = if m == Move::Up { "up ↑" } else { "down ↓" };
                        self.flash = Some((format!("turn {control} {way}"), now));
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
                let (current, target, shown) = match param {
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
                    | Param::Page
                    | Param::PadEffect(_) => return,
                };
                let value = if takeover {
                    self.takeover.apply(&port, &control, target, value, current)
                } else {
                    Ok(value)
                };
                match (value, param) {
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
                        | Param::Page
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
                                self.flash = Some((format!("Master: {hint}"), now))
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
                            | Param::Page
                            | Param::PadEffect(_) => {}
                        }
                    }
                }
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
        // Keep the old startup dismissal rule, but acknowledge a keymap warning only
        // after a frame showed it. A flash that expired between frames is not a read warning.
        let warning_shown = self.keymap_warning_shown;
        self.keymap_warning_shown = false;
        self.dismiss_startup();
        // Text, picker and learn modes take every key, so their early returns skip the
        // held-key accounting below; a Ghostty repeat after such a state change would then
        // count as a new press and dismiss a warning. Track held keys on every event.
        let tracked = if kitty { keys_down.repeat(k) } else { *k };
        if self.pad_naming.is_some() && self.pad_name_key(k) {
            return false;
        }
        if self.project_picker.is_some() && self.project_picker_key(k) {
            return false;
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
            for cmd in self.pads.apply(action) {
                self.send(cmd);
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
        if warning_shown
            && k.kind == event::KeyEventKind::Press
            && !matches!(k.code, event::KeyCode::Modifier(_))
            && !self.key_notice_blocked()
        {
            self.keymap_warnings.pop_front();
        }
        let Some(action) = keymap.resolve_in(self.screen, self.edit_keys(), &k) else {
            self.moved_key_notice(keymap, &k);
            return false;
        };
        let action = if kitty {
            held.filter(action)
        } else {
            Some(action)
        };
        action.is_some_and(|action| self.apply(action))
    }
}

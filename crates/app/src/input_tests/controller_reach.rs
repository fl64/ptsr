//! Picker and Settings routes through the shipped keyboard map and raw SMC-PAD profile.

use super::*;

const SMC: &str = "SINCO SMC-PAD-Master 28:1";

fn smc_drive() -> Drive {
    let mut drive = Drive::smc_offline();
    drive.midi().recorded_smc_setup();
    drive.frame();
    drive
}

fn turn(drive: &mut Drive, cc: u8, value: u8) {
    drive.bytes(SMC, &[0xB0, cc, value]);
    drive.frame();
}

fn cell(drive: &mut Drive, note: u8) -> Frame {
    drive.bytes(SMC, &[0x99, note, 100]);
    drive.bytes(SMC, &[0x89, note, 0]);
    drive.frame()
}

/// Scenario "Every story has a recorded controller path" (controller-reach).
#[test]
fn every_story_has_a_recorded_controller_path() {
    let stories = include_str!("../../../../docs/USER-STORIES.md");
    let paths = include_str!("../../../../docs/CONTROLLER-PATHS.md");
    let rows: Vec<Vec<&str>> = paths
        .lines()
        .filter(|line| line.starts_with("| "))
        .map(|line| line.split('|').map(str::trim).collect())
        .collect();
    for heading in stories.lines().filter_map(|line| {
        line.strip_prefix("## ").or_else(|| {
            ["### Chop the sound of a pad", "### Edit a pad"]
                .contains(&line)
                .then(|| &line[4..])
        })
    }) {
        let row = rows
            .iter()
            .find(|row| row.get(1) == Some(&heading))
            .unwrap_or_else(|| panic!("no SMC-PAD route for {heading}"));
        assert!(!row[2].is_empty(), "empty SMC-PAD route for {heading}");
        let counts: Vec<(u8, u8)> = row[2]
            .split('`')
            .filter_map(|part| {
                let (visible, hidden) = part.split_once('/')?;
                Some((visible.parse().ok()?, hidden.parse().ok()?))
            })
            .collect();
        for (visible, hidden) in &counts {
            assert!(
                *visible <= 2 && *hidden <= 4,
                "{heading}: over-budget SMC-PAD route {visible}/{hidden}"
            );
        }
        assert!(
            !counts.is_empty()
                || row[3].contains("Observation")
                || row[3].contains("text")
                || row[3].contains("Keyboard-only")
                || row[3].contains("Other-device"),
            "no count or explicit exception for {heading}"
        );
    }
}

/// Scenario "Every current keyboard action has a controller route" (controller-reach).
#[test]
fn every_current_keyboard_action_has_a_controller_route() {
    use crate::panel::{Config, View};

    let drive = smc_drive();
    let profile = drive
        .app
        .profiles
        .iter()
        .find(|profile| profile.name == "SMC-PAD")
        .unwrap();
    let panel = Config::parse(
        "config/panel.toml",
        include_str!("../../../../config/panel.toml"),
    );
    assert!(panel.warnings.is_empty(), "{:?}", panel.warnings);
    let aliases = [
        ("bank.prev", "e8", "bank"),
        ("bank.next", "e8", "bank"),
        ("master.volume_down", "f8", "master.volume"),
        ("master.volume_up", "f8", "master.volume"),
        ("browser.up", "left", "browser.up"),
        ("browser.down", "right", "browser.down"),
        ("chop.cursor_left", "e1", "chop.cursor_pos"),
        ("chop.cursor_right", "e1", "chop.cursor_pos"),
        ("chop.zoom_in", "e2", "chop.zoom"),
        ("chop.zoom_out", "e2", "chop.zoom"),
        ("slice.start_left", "e4", "slice.start"),
        ("slice.start_right", "e4", "slice.start"),
        ("slice.end_left", "e5", "slice.end"),
        ("slice.end_right", "e5", "slice.end"),
        ("play.context", "play", "play.context"),
        ("stop.context", "stop", "stop.context"),
        ("metronome.toggle", "play", "play.context"),
        ("record.mic", "record", "pattern.record"),
    ];
    for screen in input::Screen::ALL {
        let views: &[View] = match screen {
            Screen::Pads => &[View::Pads, View::PadEdit, View::Pattern],
            Screen::Chop => &[View::Chop, View::ChopEdit],
            Screen::Files => &[View::Files],
            Screen::Fx => &[View::Fx],
            Screen::Settings => &[View::Settings],
        };
        let mut panel_actions = Vec::new();
        for view in views {
            for page in 0..panel.pages(*view) {
                for cell in 1..=12 {
                    if let Some(cell) = panel.cell(*view, page, cell) {
                        panel_actions.push(cell.action);
                    }
                }
            }
        }
        for (key, action) in drive.keymap.bindings(screen) {
            if action == Action::PadCopy && screen != Screen::Pads {
                continue;
            }
            if panel_actions.contains(&action)
                || matches!(action, Action::PadRename | Action::BrowserFilter)
            {
                continue;
            }
            let named = input::ACTIONS
                .iter()
                .find(|info| Action::parse(info.name) == Some(action))
                .map(|info| info.name)
                .unwrap_or("");
            let fixed = matches!(
                named,
                "screen.pads"
                    | "screen.files"
                    | "screen.chop"
                    | "screen.pattern"
                    | "screen.fx"
                    | "screen.settings"
                    | "help.toggle"
                    | "help.page_prev"
                    | "help.page_next"
                    | "edit.undo"
                    | "ui.back"
                    | "ui.confirm"
                    | "transport.play_stop"
            );
            let bank_select = named.starts_with("bank.") && named[5..].parse::<u8>().is_ok();
            let role = aliases.iter().find(|(name, _, _)| *name == named).copied();
            let role_ok = role.is_some_and(|(_, control, expected)| {
                profile.control_action(screen, false, control) == Some(expected)
            });
            let variant_role_ok = match action {
                Action::ChopZoom { .. } => {
                    profile.control_action(screen, false, "e2") == Some("chop.zoom")
                }
                Action::CursorMove { .. } => {
                    profile.control_action(screen, false, "e1") == Some("chop.cursor_pos")
                }
                Action::SliceBound { end, .. } => {
                    profile.control_action(screen, false, if end { "e5" } else { "e4" })
                        == Some(if end { "slice.end" } else { "slice.start" })
                }
                _ => false,
            };
            let bank_ok =
                bank_select && profile.control_action(screen, false, "e8") == Some("bank");
            assert!(
                fixed || role_ok || variant_role_ok || bank_ok,
                "{screen:?} key {key}: no SMC-PAD route for {named} ({action:?})"
            );
        }
    }
}

/// Scenario "Every current controller action has a keyboard route" (controller-reach).
#[test]
fn every_current_controller_action_has_a_keyboard_route() {
    use crate::panel::{Choice, Config, Press, State, View};

    let drive = smc_drive();
    let profile = drive
        .app
        .profiles
        .iter()
        .find(|profile| profile.name == "SMC-PAD")
        .unwrap();
    let keyboard: Vec<Action> = input::Screen::ALL
        .into_iter()
        .flat_map(|screen| {
            drive
                .keymap
                .bindings(screen)
                .into_iter()
                .map(|(_, action)| action)
        })
        .collect();
    let has = |name: &str| keyboard.contains(&Action::parse(name).unwrap());
    let panel = Config::parse(
        "config/panel.toml",
        include_str!("../../../../config/panel.toml"),
    );
    assert!(panel.warnings.is_empty(), "{:?}", panel.warnings);
    for view in [
        View::Pads,
        View::PadEdit,
        View::Pattern,
        View::Files,
        View::Chop,
        View::ChopEdit,
        View::Fx,
        View::Settings,
    ] {
        for page in 0..panel.pages(view) {
            for slot in 1..=12 {
                let Some(cell) = panel.cell(view, page, slot) else {
                    continue;
                };
                let mut route = State::default();
                assert_eq!(route.press(SMC, 1), Press::Revealed);
                if page > 0 {
                    assert_eq!(route.choice(view, 8), Choice::Page);
                    if page > 1 {
                        route.select_context_page(view, page as i32 - 1);
                    }
                }
                assert_eq!(route.page(), page);
                assert_eq!(
                    route.choice(view, slot),
                    if cell.action == Action::Quit {
                        Choice::Exit
                    } else {
                        Choice::Action(cell.action)
                    },
                    "{view:?} page {page} cell {cell:?}"
                );
                let contextual = match cell.action {
                    Action::ChopCommit => has("chop.edit"),
                    Action::PatternClear => has("pad.clear"),
                    Action::PatternParamPrev => has("edit.param_prev"),
                    Action::PatternParamNext => has("edit.param_next"),
                    Action::PatternValueDec => has("edit.value_dec"),
                    Action::PatternValueInc => has("edit.value_inc"),
                    _ => false,
                };
                assert!(
                    keyboard.contains(&cell.action) || contextual,
                    "{view:?} page {page} cell {cell:?}: no keyboard route"
                );
            }
        }
    }
    let pad_keys = [
        '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v',
    ];
    for (index, key) in pad_keys.into_iter().enumerate() {
        assert!(
            matches!(
                drive.keymap.resolve(Screen::Pads, &KeyEvent::new(KeyCode::Char(key), M::NONE)),
                Some(Action::PadPress { pad, .. }) if pad == index
            ),
            "SMC-PAD pad {} has no keyboard hit",
            index + 1
        );
    }
    for (scope, key, role) in profile.roles() {
        let keyboard_route = match role.as_str() {
            r if r.starts_with("pad.") && r[4..].parse::<usize>().is_ok() => true,
            r if r.starts_with("panel.cell.") => true,
            "bank" => has("bank.prev") && has("bank.next"),
            "master.volume" => has("master.volume_down") && has("master.volume_up"),
            "transport.tempo" => has("transport.tempo_down") && has("transport.tempo_up"),
            "chop.cursor_pos" => has("chop.cursor_left") && has("chop.cursor_right"),
            "chop.zoom" => has("chop.zoom_in") && has("chop.zoom_out"),
            "slice.start" => has("slice.start_left") && has("slice.start_right"),
            "slice.end" => has("slice.end_left") && has("slice.end_right"),
            "edit.page" => has("edit.page_prev") && has("edit.page_next"),
            "pad.volume" | "pad.pitch" | "pad.cutoff" | "pad.decay" | "chop.threshold"
            | "pattern.bars" | "pattern.quantize" | "pattern.strength" => {
                has("edit.value_dec") && has("edit.value_inc")
            }
            "pad.select_left" | "pad.select_right" => true,
            "files.scroll" | "projects.scroll" | "settings.row" => {
                has("browser.up") && has("browser.down") || has("settings.row_prev")
            }
            "marker.prev" | "marker.next" => has("chop.cursor_left") && has("chop.cursor_right"),
            "play.context"
            | "stop.context"
            | "record.mic"
            | "metronome.toggle"
            | "pattern.record"
            | "browser.up"
            | "browser.down"
            | "settings.row_prev"
            | "settings.row_next"
            | "settings.value_next" => has(&role),
            _ => false,
        };
        assert!(keyboard_route, "{scope:?} {key:?} {role}: no keyboard path");
    }
}

fn project_drive(files: &TransitionFiles) -> Drive {
    let mut drive = smc_drive();
    drive.app.paths.projects = files.0.clone();
    drive.app.project = Some(Project {
        name: "A".into(),
        folder: files.0.join("A"),
    });
    drive.app.user_path = Some(files.0.join("config.toml"));
    drive.app.publish_midi(&drive.shared);
    drive
}

/// Scenario "Controller opens a selected project" (controller-reach).
#[test]
fn controller_opens_a_selected_project() {
    let files = TransitionFiles::new();
    for controller in [false, true] {
        let mut drive = project_drive(&files);
        drive.tap(KeyCode::Char('p'), M::CONTROL);
        assert_eq!(drive.app.project_picker.as_ref().unwrap().selected, 0);
        if controller {
            turn(&mut drive, 30, 65);
        } else {
            drive.tap(KeyCode::Down, M::NONE);
        }
        assert_eq!(drive.app.project_picker.as_ref().unwrap().selected, 1);
        if controller {
            cell(&mut drive, 63);
        } else {
            drive.tap(KeyCode::Enter, M::NONE);
        }
        assert_eq!(drive.app.project.as_ref().unwrap().name, "B");
        assert!(drive.app.project_picker.is_none());
    }
}

#[test]
fn refused_project_save_keeps_smc_selection() {
    let files = TransitionFiles::new();
    let mut drive = project_drive(&files);
    drive.tap(KeyCode::Char('p'), M::CONTROL);
    turn(&mut drive, 30, 65);
    drive.app.session_dirty = Some(Instant::now());
    std::fs::write(files.0.join("A/project.toml.tmp"), "occupied").unwrap();
    cell(&mut drive, 63);
    assert_eq!(drive.app.project.as_ref().unwrap().name, "A");
    assert_eq!(drive.app.project_picker.as_ref().unwrap().selected, 1);
    assert!(drive.app.session_dirty.is_some());
}

#[test]
fn disconnected_smc_keeps_the_project_picker_for_keyboard() {
    let files = TransitionFiles::new();
    let mut drive = project_drive(&files);
    cell(&mut drive, 64);
    assert!(drive.app.panel.visible());
    drive.tap(KeyCode::Char('p'), M::CONTROL);
    turn(&mut drive, 30, 65);
    assert_eq!(drive.app.project_picker.as_ref().unwrap().selected, 1);
    drive.midi().recorded_disconnect(SMC);
    drive.frame();
    assert!(!drive.app.panel.visible());
    assert_eq!(drive.app.project.as_ref().unwrap().name, "A");
    assert_eq!(drive.app.project_picker.as_ref().unwrap().selected, 1);
    drive.tap(KeyCode::Enter, M::NONE);
    assert_eq!(drive.app.project.as_ref().unwrap().name, "B");
}

fn folder_drive(files: &TransitionFiles) -> Drive {
    let mut drive = smc_drive();
    drive.app.screen = Screen::Settings;
    drive.app.settings_tab = 3;
    drive.app.settings_row = 0;
    drive.app.paths.projects = files.0.clone();
    drive.app.user_path = Some(files.0.join("config.toml"));
    drive.app.apply(Action::SettingsLearn);
    drive.app.publish_midi(&drive.shared);
    assert!(drive.app.path_picker.is_some());
    drive
}

/// Scenario "Controller selects a Paths folder" (controller-reach).
#[test]
fn controller_selects_a_paths_folder() {
    let files = TransitionFiles::new();
    for controller in [false, true] {
        let mut drive = folder_drive(&files);
        if controller {
            turn(&mut drive, 30, 65);
            drive.bytes(SMC, &[0xB0, 26, 127]);
            drive.bytes(SMC, &[0xB0, 26, 0]);
            drive.frame();
        } else {
            drive.tap(KeyCode::Down, M::NONE);
            drive.tap(KeyCode::Right, M::NONE);
        }
        assert_eq!(
            drive.app.path_picker.as_ref().unwrap().at,
            files.0.join("B")
        );
        if controller {
            cell(&mut drive, 63);
        } else {
            drive.tap(KeyCode::Enter, M::NONE);
        }
        assert!(drive.app.path_picker.is_none());
        assert_eq!(drive.app.paths.projects, files.0.join("B"));
    }
    let mut drive = folder_drive(&files);
    turn(&mut drive, 30, 65);
    cell(&mut drive, 62);
    assert!(drive.app.path_picker.is_none());
    assert_eq!(drive.app.paths.projects, files.0);
}

/// Scenario "Settings knob changes only the selected value" (controller-reach).
#[test]
fn settings_knob_changes_only_the_selected_value() {
    let mut keyboard = smc_drive();
    let mut controller = smc_drive();
    for drive in [&mut keyboard, &mut controller] {
        drive.app.screen = Screen::Settings;
        drive.app.settings_tab = 0;
        drive.app.settings_row = 3;
        drive.app.publish_midi(&drive.shared);
    }
    keyboard.tap(KeyCode::Right, M::NONE);
    turn(&mut controller, 31, 65);
    assert!(keyboard.app.ui.graphics);
    assert_eq!(controller.app.ui.graphics, keyboard.app.ui.graphics);

    let files = TransitionFiles::new();
    controller.app.settings_tab = 3;
    controller.app.settings_row = 0;
    controller.app.paths.projects = files.0.clone();
    controller.app.apply(Action::SettingsLearn);
    controller.app.publish_midi(&controller.shared);
    turn(&mut controller, 31, 63);
    assert!(controller.app.ui.graphics);
    assert_eq!(controller.app.path_picker.as_ref().unwrap().selected, 0);
    assert_eq!(controller.app.paths.projects, files.0);
}

#[test]
fn settings_knob_keeps_keyboard_hold_time_bounds() {
    let mut keyboard = smc_drive();
    let mut controller = smc_drive();
    for drive in [&mut keyboard, &mut controller] {
        drive.app.screen = Screen::Settings;
        drive.app.settings_tab = 2;
        drive.app.midi_row = drive
            .app
            .midi_list()
            .iter()
            .position(|row| matches!(row, MidiListRow::HoldTime))
            .unwrap();
        drive.app.publish_midi(&drive.shared);
    }
    for _ in 0..8 {
        keyboard.tap(KeyCode::Right, M::NONE);
        turn(&mut controller, 31, 65);
    }
    assert_eq!(keyboard.app.midi_gestures.hold_time_ms, 600);
    assert_eq!(controller.app.midi_gestures, keyboard.app.midi_gestures);
    for _ in 0..10 {
        keyboard.tap(KeyCode::Left, M::NONE);
        turn(&mut controller, 31, 63);
    }
    assert_eq!(keyboard.app.midi_gestures.hold_time_ms, 200);
    assert_eq!(controller.app.midi_gestures, keyboard.app.midi_gestures);
}

/// Scenario "Modal answer wins over panel entry" (controller-reach).
#[test]
fn modal_answer_wins_over_panel_entry() {
    let files = TransitionFiles::new();
    let mut drive = project_drive(&files);
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 256])), "gate");
    drive.bytes(SMC, &[0x99, 48, 100]);
    drive.frame();
    assert!(drive.app.pads.held[0]);
    drive.tap(KeyCode::Char('p'), M::CONTROL);
    drive.app.ask = Some(Ask::ResetPads {
        pads: vec![1],
        lines: vec!["Reset pad 2?".into()],
    });
    drive.app.publish_midi(&drive.shared);
    cell(&mut drive, 63);
    assert!(drive.app.ask.is_none());
    assert!(drive.app.project_picker.is_some());
    assert!(!drive.app.panel.visible());
    drive.bytes(SMC, &[0x89, 48, 0]);
    drive.frame();
    assert!(!drive.app.pads.held[0]);
    cell(&mut drive, 62);
    assert!(drive.app.project_picker.is_none());
}

#[test]
fn another_controller_keeps_its_global_knob_in_the_project_picker() {
    let files = TransitionFiles::new();
    let mut drive = project_drive(&files);
    drive.tap(KeyCode::Char('p'), M::CONTROL);
    turn(&mut drive, 30, 65);
    let before = drive.app.master_db;
    drive.bytes(XS_PORT, &[0xB0, 14, 0]);
    drive.frame();
    assert_ne!(drive.app.master_db, before);
    assert_eq!(drive.app.project_picker.as_ref().unwrap().selected, 1);
}

/// Scenario "Encoder selects a later context page" (controller-reach).
#[test]
fn encoder_selects_a_later_context_page() {
    let mut keyboard = smc_drive();
    let mut controller = smc_drive();
    let before = controller.app.pattern_view.division.grid();
    keyboard.tap(KeyCode::Up, M::ALT);

    let mut hidden = smc_drive();
    turn(&mut hidden, 35, 65);
    assert_eq!(hidden.app.panel.page(), 0);
    assert_eq!(hidden.app.pattern_view.division.grid(), before);
    cell(&mut controller, 64);
    cell(&mut controller, 63);
    assert_eq!(controller.app.panel.page(), 1);
    let master = controller.app.master_db;
    controller.bytes(XS_PORT, &[0xB0, 14, 0]);
    controller.frame();
    assert_ne!(controller.app.master_db, master);
    assert_eq!(controller.app.panel.page(), 1);
    turn(&mut controller, 35, 63);
    assert_eq!(
        controller.app.panel.page(),
        4,
        "{:?}",
        controller.app.input_log
    );
    assert_eq!(
        controller.app.panel_snapshot().unwrap().labels[1].as_deref(),
        Some("view +")
    );
    cell(&mut controller, 65);
    assert_ne!(controller.app.pattern_view.division.grid(), before);
    assert_eq!(
        controller.app.pattern_view.division.grid(),
        keyboard.app.pattern_view.division.grid()
    );
    assert_eq!(controller.app.pads.bank, keyboard.app.pads.bank);
}

/// Scenario "Help cells navigate without acting behind Help" (controller-reach).
#[test]
fn help_cells_navigate_without_acting_behind_help() {
    let mut controller = smc_drive();
    let mut keyboard = smc_drive();
    let selected = controller.app.pads.selected_abs();
    cell(&mut controller, 64);
    cell(&mut controller, 62);
    keyboard.tap(KeyCode::F(1), M::NONE);
    assert!(controller.app.help_open);
    assert_eq!(
        controller.app.help_overlay.page,
        keyboard.app.help_overlay.page
    );

    cell(&mut controller, 63);
    keyboard.tap(KeyCode::Tab, M::NONE);
    assert_eq!(
        controller.app.help_overlay.page,
        keyboard.app.help_overlay.page
    );
    cell(&mut controller, 63);
    keyboard.tap(KeyCode::Tab, M::NONE);
    assert_eq!(
        controller.app.help_overlay.page,
        keyboard.app.help_overlay.page
    );

    let before = controller.app.controller_view(&controller.keymap).detail;
    cell(&mut controller, 57);
    let after = controller.app.controller_view(&controller.keymap).detail;
    assert_ne!(after, before);
    cell(&mut controller, 56);
    assert_eq!(
        controller.app.controller_view(&controller.keymap).detail,
        before
    );
    cell(&mut controller, 62);
    keyboard.tap(KeyCode::BackTab, M::SHIFT);
    assert_eq!(
        controller.app.help_overlay.page,
        keyboard.app.help_overlay.page
    );
    cell(&mut controller, 52);
    assert!(!controller.app.help_open);
    assert_eq!(controller.app.screen, Screen::Pads);
    assert_eq!(controller.app.pads.selected_abs(), selected);
}

/// Scenario "Exit needs a second SMC-PAD press" (controller-reach).
#[test]
fn exit_needs_a_second_smc_press() {
    let mut drive = smc_drive();
    drive.app.session_dirty = Some(Instant::now());
    cell(&mut drive, 64);
    cell(&mut drive, 63);
    turn(&mut drive, 35, 65);
    assert_eq!(drive.app.panel.page(), 2);
    cell(&mut drive, 57);
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    assert!(!drive.app.quit_requested);
    assert!(drive.app.session_dirty.is_some());
    cell(&mut drive, 62);
    assert!(drive.app.ask.is_none());
    assert!(!drive.app.quit_requested);
    assert_eq!(drive.app.panel.page(), 2);

    cell(&mut drive, 57);
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    cell(&mut drive, 52);
    assert!(drive.app.ask.is_none());
    assert!(!drive.app.quit_requested);

    cell(&mut drive, 57);
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    cell(&mut drive, 63);
    assert!(drive.app.ask.is_none());
    assert!(drive.app.quit_requested);
    assert_eq!(drive.app.panel.page(), 2);

    let mut keyboard = smc_drive();
    assert!(keyboard.app.keyboard_event(
        &keyboard.keymap,
        &KeyEvent::new_with_kind(KeyCode::Char('q'), M::CONTROL, KeyEventKind::Press),
        true,
        &mut keyboard.down,
        &mut keyboard.held,
        &mut keyboard.mods,
    ));
    assert!(keyboard.app.quit_requested);
}

#[test]
fn exit_question_keeps_a_pending_a1_save_until_confirmation() {
    use engine::capture::Phase;

    let files = TransitionFiles::new();
    let mut drive = project_drive(&files);
    drive.app.session_read_error = None;
    let start = drive.tap(KeyCode::Char('r'), M::CONTROL);
    assert!(
        start
            .commands()
            .iter()
            .any(|command| matches!(command, Command::PatternRecordStart { .. }))
    );
    drive.render_blocks(1);
    drive.frame();
    assert_eq!(
        drive.app.pattern_record_state().unwrap().phase,
        Phase::CountIn
    );
    drive.app.session_dirty = Some(Instant::now());
    drive.app.request_manual_save();
    assert!(drive.app.pending_manual_save.is_some());

    cell(&mut drive, 64);
    cell(&mut drive, 63);
    turn(&mut drive, 35, 65);
    assert_eq!(drive.app.panel.page(), 2);
    cell(&mut drive, 57);
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    assert!(drive.app.pending_manual_save.is_some());
    assert!(!drive.app.quit_requested);
    cell(&mut drive, 62);
    assert!(drive.app.ask.is_none());
    assert!(drive.app.pending_manual_save.is_some());
    assert!(!drive.app.quit_requested);

    cell(&mut drive, 57);
    cell(&mut drive, 63);
    assert!(drive.app.quit_requested);
    assert!(drive.app.quit_ready());
    assert!(drive.app.pending_manual_save.is_none());
    drive.app.save_on_exit();
    assert!(drive.app.session_dirty.is_none());
    assert!(session::load_existing(&files.0.join("A/project.toml")).is_ok());
}

/// Scenario "First entry and repeated commands have separate counts" (controller-reach).
#[test]
fn first_entry_and_repeated_commands_have_separate_counts() {
    let mut first = smc_drive();
    let mut first_cell_hits = 0;
    for note in [64, 63, 64] {
        cell(&mut first, note);
        first_cell_hits += 1;
    }
    assert!(first.app.pads.gate[0]);
    assert_eq!(1 + first_cell_hits, 4); // PAD BANK plus reveal, More and Mode.

    let mut repeated = smc_drive();
    cell(&mut repeated, 64); // Reveal before the counted visible-main route.
    let mut repeated_cell_hits = 0;
    for note in [63, 64] {
        cell(&mut repeated, note);
        repeated_cell_hits += 1;
    }
    assert!(repeated.app.pads.gate[0]);
    assert_eq!(repeated_cell_hits, 2);

    let mut knob = smc_drive();
    turn(&mut knob, 37, 65);
    assert_eq!(knob.app.pads.bank, 1, "{:?}", knob.app.input_log);
}

/// Scenario "Added cells reach their current actions" (controller-reach).
#[test]
fn added_cells_reach_their_current_actions() {
    let mut keyboard = smc_drive();
    let mut controller = smc_drive();
    keyboard.tap(KeyCode::Up, M::CONTROL);
    turn(&mut controller, 36, 65);
    assert_eq!(controller.app.transport.bpm, keyboard.app.transport.bpm);
    keyboard.tap(KeyCode::PageDown, M::NONE);
    turn(&mut controller, 37, 65);
    assert_eq!(controller.app.pads.bank, keyboard.app.pads.bank);

    keyboard.tap(KeyCode::Char('n'), M::ALT);
    cell(&mut controller, 64);
    cell(&mut controller, 63);
    turn(&mut controller, 35, 65);
    assert_eq!(controller.app.panel.page(), 2);
    assert_eq!(
        controller.app.panel_snapshot().unwrap().labels[2].as_deref(),
        Some("take")
    );
    cell(&mut controller, 56);
    assert_eq!(controller.app.pattern_panel, keyboard.app.pattern_panel);
    assert!(controller.app.pattern_panel.is_some());
}

#[test]
fn take_cell_keeps_settings_screen_and_swap_target() {
    let mut keyboard = smc_drive();
    let mut controller = smc_drive();
    for drive in [&mut keyboard, &mut controller] {
        drive.app.screen = Screen::Settings;
        drive.app.publish_midi(&drive.shared);
    }
    keyboard.tap(KeyCode::Char('x'), M::CONTROL);
    cell(&mut controller, 64);
    cell(&mut controller, 63);
    turn(&mut controller, 35, 65);
    assert_eq!(controller.app.panel.page(), 2);
    cell(&mut controller, 66);
    assert_eq!(controller.app.pad_in_hand, keyboard.app.pad_in_hand);
    assert_eq!(controller.app.screen, keyboard.app.screen);
}

#[test]
fn tempo_cells_match_all_four_keyboard_steps() {
    for (note, code, modifiers) in [
        (64, KeyCode::Down, M::CONTROL),
        (65, KeyCode::Up, M::CONTROL),
        (66, KeyCode::Down, M::CONTROL | M::SHIFT),
        (67, KeyCode::Up, M::CONTROL | M::SHIFT),
    ] {
        let mut keyboard = smc_drive();
        let mut controller = smc_drive();
        keyboard.tap(code, modifiers);
        cell(&mut controller, 64);
        cell(&mut controller, 63);
        cell(&mut controller, 58);
        cell(&mut controller, 58);
        assert_eq!(controller.app.panel.page(), 3);
        cell(&mut controller, note);
        assert_eq!(controller.app.transport.bpm, keyboard.app.transport.bpm);
    }
}

#[test]
fn grid_cell_keeps_the_screen_and_matches_ctrl_o() {
    for screen in [Screen::Pads, Screen::Files, Screen::Chop, Screen::Settings] {
        let mut keyboard = smc_drive();
        let mut controller = smc_drive();
        for drive in [&mut keyboard, &mut controller] {
            drive.app.screen = screen;
            drive.app.publish_midi(&drive.shared);
        }
        keyboard.tap(KeyCode::Char('o'), M::CONTROL);
        cell(&mut controller, 64);
        cell(&mut controller, 63);
        turn(&mut controller, 35, 65);
        assert_eq!(controller.app.panel.page(), 2);
        cell(&mut controller, 61);
        assert_eq!(controller.app.ui.hide_pads, keyboard.app.ui.hide_pads);
        assert_eq!(controller.app.screen, screen);
    }
}

#[test]
fn pattern_record_and_erase_cells_match_keyboard_modes() {
    for (note, code, modifiers) in [
        (60, KeyCode::Char('e'), M::ALT),
        (61, KeyCode::Char('r'), M::CONTROL),
    ] {
        let mut keyboard = smc_drive();
        let mut controller = smc_drive();
        if note == 60 {
            let pattern = session::PatternState {
                slot: "A1".into(),
                bars: 1,
                quantize: "1/16".into(),
                strength: 100,
                hits: vec![[0, 0, 100, 240]],
            };
            keyboard.app.pads.pattern = Some(pattern.clone());
            controller.app.pads.pattern = Some(pattern);
        }
        let keyboard_frame = keyboard.tap(code, modifiers);
        cell(&mut controller, 64);
        cell(&mut controller, 63);
        cell(&mut controller, 58);
        cell(&mut controller, 58);
        assert_eq!(controller.app.panel.page(), 3);
        let controller_frame = cell(&mut controller, note);
        assert_eq!(controller.app.erase_pads, keyboard.app.erase_pads);
        assert_eq!(
            controller
                .app
                .pattern_record_state()
                .map(|state| state.phase),
            keyboard.app.pattern_record_state().map(|state| state.phase)
        );
        if note == 60 {
            assert!(controller.app.erase_pads);
        } else {
            for frame in [keyboard_frame, controller_frame] {
                assert!(
                    frame
                        .commands()
                        .iter()
                        .any(|command| matches!(command, Command::PatternRecordStart { .. }))
                );
            }
        }
    }
}

/// Scenario "Every reachable mode shows its badge" (controller-reach).
#[test]
fn every_reachable_mode_shows_its_badge() {
    for mode in ["mark", "edit", "copy", "swap", "learn"] {
        let mut keyboard = smc_drive();
        let mut controller = smc_drive();
        let sample = Arc::new(Sample::mono(vec![0.2; 256]));
        keyboard.load_pad(0, sample.clone(), "source");
        controller.load_pad(0, sample, "source");
        cell(&mut controller, 64);
        match mode {
            "mark" => {
                keyboard.tap(KeyCode::Char('m'), M::NONE);
                cell(&mut controller, 59);
            }
            "edit" => {
                keyboard.tap(KeyCode::Enter, M::NONE);
                cell(&mut controller, 56);
            }
            "copy" => {
                keyboard.tap(KeyCode::Char('c'), M::CONTROL);
                cell(&mut controller, 57);
            }
            "swap" => {
                keyboard.tap(KeyCode::Char('x'), M::CONTROL);
                cell(&mut controller, 58);
            }
            _ => {
                keyboard.tap(KeyCode::F(10), M::NONE);
                cell(&mut controller, 63);
                turn(&mut controller, 35, 65);
                cell(&mut controller, 62);
            }
        }
        assert_eq!(
            controller.app.focus().mode,
            keyboard.app.focus().mode,
            "{mode}"
        );
        let labels = |drive: &Drive| {
            drive
                .app
                .title_badges()
                .into_iter()
                .map(|badge| badge.normal)
                .collect::<Vec<_>>()
        };
        assert_eq!(labels(&controller), labels(&keyboard), "{mode}");
        keyboard.tap(KeyCode::Esc, M::NONE);
        cell(&mut controller, 52);
        assert_eq!(
            controller.app.focus().mode,
            keyboard.app.focus().mode,
            "{mode} exit"
        );
    }
}

/// Scenario "Learning answer keeps its chosen control" (controller-reach).
#[test]
fn learning_answer_keeps_its_chosen_control() {
    let mut drive = smc_drive();
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 256])), "source");
    cell(&mut drive, 64);
    cell(&mut drive, 57); // Copy is the action to learn.
    cell(&mut drive, 63);
    turn(&mut drive, 35, 65);
    cell(&mut drive, 62);
    assert_eq!(drive.app.learn_picked().unwrap().0, "pad.copy");
    cell(&mut drive, 63);
    assert_eq!(
        drive
            .app
            .screen_learn
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .1,
        input::midi::Key::Note(63)
    );
    cell(&mut drive, 63);
    assert!(drive.app.screen_learn.is_none());
    let bindings = learn::group_bindings(&drive.app.learned, "SMC-PAD", input::Group::Pads);
    assert_eq!(
        bindings.get(&input::midi::Key::Note(63)).unwrap().primary(),
        "pad.copy"
    );

    let mut cancelled = smc_drive();
    cancelled.app.screen = Screen::Settings;
    cancelled.app.settings_tab = 2;
    cancelled.app.learning = Some(learn::Learning::new(action_info("pad.copy")));
    cancelled.app.publish_midi(&cancelled.shared);
    cell(&mut cancelled, 63);
    assert_eq!(
        cancelled
            .app
            .learning
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .1,
        input::midi::Key::Note(63)
    );
    cell(&mut cancelled, 62);
    assert!(cancelled.app.learning.is_none());
    let bindings = learn::group_bindings(&cancelled.app.learned, "SMC-PAD", input::Group::Pads);
    assert!(
        bindings
            .get(&input::midi::Key::Note(63))
            .is_none_or(|b| b.primary() != "pad.copy")
    );

    cancelled.app.learning = Some(learn::Learning::new(action_info("pad.copy")));
    cancelled.app.publish_midi(&cancelled.shared);
    cell(&mut cancelled, 62);
    assert_eq!(
        cancelled
            .app
            .learning
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .1,
        input::midi::Key::Note(62)
    );
    cell(&mut cancelled, 63);
    assert!(cancelled.app.learning.is_none());
    let bindings = learn::group_bindings(&cancelled.app.learned, "SMC-PAD", input::Group::Pads);
    assert_eq!(
        bindings.get(&input::midi::Key::Note(62)).unwrap().primary(),
        "pad.copy"
    );
}

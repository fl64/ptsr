//! Picker and Settings routes through the shipped keyboard map and raw SMC-PAD profile.

use super::*;

#[path = "panel_targets.rs"]
mod panel_targets;

#[path = "chop_from_pad.rs"]
mod chop_from_pad;

#[path = "chop_zoom.rs"]
mod chop_zoom;

#[path = "chop_join.rs"]
mod chop_join;

#[path = "cell_looks.rs"]
mod cell_looks;

#[path = "destroy_asks.rs"]
mod destroy_asks;

#[path = "normalize.rs"]
mod normalize;

#[path = "mode_lines.rs"]
mod mode_lines;

#[path = "assign_method.rs"]
mod assign_method;

#[path = "new_project.rs"]
mod new_project;

#[path = "trim_routes.rs"]
mod trim_routes;

#[path = "binding_changes.rs"]
mod binding_changes;

const SMC: &str = "SINCO SMC-PAD-Master 28:1";

/// A verified SMC-PAD with the PAD BANK shortcut off: second-bank hits reach the panel.
fn smc_drive() -> Drive {
    let mut drive = Drive::smc_offline();
    drive.app.midi_gestures.smc_pad_bank_opens_panel = false;
    drive.midi().set_pad_bank_shortcut(false);
    drive.midi().recorded_smc_setup();
    drive.frame();
    drive
}

/// A verified SMC-PAD with the default PAD BANK hold (131).
fn hold_drive() -> Drive {
    let mut drive = Drive::smc_offline();
    drive.midi().recorded_smc_setup();
    drive.frame();
    drive
}

fn hold(drive: &mut Drive, held: bool) -> Frame {
    drive.midi().recorded_pad_bank(SMC, Instant::now(), held);
    drive.frame()
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
                *visible <= 5 && (*visible..=*visible + 2).contains(hidden),
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
        ("pad.select_left", "e1", "smc.position"),
        ("pad.select_right", "e1", "smc.position"),
        ("master.volume_down", "f8", "master.volume"),
        ("master.volume_up", "f8", "master.volume"),
        ("browser.up", "e1", "files.scroll"),
        ("browser.down", "e1", "files.scroll"),
        ("chop.cursor_left", "e1", "chop.cursor_pos"),
        ("chop.cursor_right", "e1", "chop.cursor_pos"),
        ("chop.zoom_in", "e2", "chop.zoom"),
        ("chop.zoom_out", "e2", "chop.zoom"),
        ("chop.threshold_up", "e7", "chop.threshold"),
        ("chop.threshold_down", "e7", "chop.threshold"),
        ("slice.start_left", "e3", "slice.start"),
        ("slice.start_right", "e3", "slice.start"),
        ("slice.end_left", "e4", "slice.end"),
        ("slice.end_right", "e4", "slice.end"),
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
        let pads_actions: Vec<Action> = [View::Pads, View::Pattern]
            .into_iter()
            .flat_map(|view| (1..=12).map(move |cell| (view, cell)))
            .filter_map(|(view, cell)| panel.cell(view, cell).map(|cell| cell.action))
            .collect();
        // The screens view holds the global actions (127).
        let mut panel_actions = vec![Action::SessionSave, Action::ProjectPick, Action::Quit];
        for view in views {
            for cell in 1..=12 {
                if let Some(cell) = panel.cell(*view, cell) {
                    panel_actions.push(cell.action);
                }
            }
        }
        for (key, action) in drive.keymap.bindings(screen) {
            if action == Action::PadCopy && screen != Screen::Pads {
                continue;
            }
            if panel_actions.contains(&action)
                // Bindings change from the keyboard only (138).
                || matches!(
                    action,
                    Action::PadRename | Action::BrowserFilter | Action::LearnToggle
                )
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
                    // PLAY and STOP replace the panel cell (127).
                    | "transport.toggle"
                    // Bindings change from the keyboard only (138).
                    | "learn.toggle"
                    | "settings.clear"
                    // The undo cell undoes an A1 pass (138).
                    | "pattern.undo"
            );
            // Pad commands live in the Pads actions and A1 commands in the A1 actions (127).
            let pads_screen = matches!(
                named,
                "pad.take"
                    | "record.output"
                    | "pattern.controls"
                    | "pattern.record"
                    | "pattern.erase"
                    | "pattern.undo"
                    | "ui.pads_toggle"
            ) && pads_actions.contains(&action);
            let bank_select = named.starts_with("bank.");
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
                    profile.control_action(screen, false, if end { "e4" } else { "e3" })
                        == Some(if end { "slice.end" } else { "slice.start" })
                }
                // LEFT, RIGHT and knob 1 replace the duplicate panel cells (134).
                Action::BrowserParent | Action::BrowserEnter => {
                    profile.control_action(screen, false, "right") == Some("smc.list_next")
                }
                Action::SettingsTab => {
                    profile.control_action(screen, false, "right") == Some("smc.list_next")
                }
                Action::SettingsRow(_) => {
                    profile.control_action(screen, false, "e1") == Some("settings.row")
                }
                // Knobs 5 and 6 move the Chop region; the panel cells left in 134.
                Action::ChopRegionStart => {
                    profile.control_action(screen, false, "e5") == Some("chop.region_start_pos")
                }
                Action::ChopRegionEnd => {
                    profile.control_action(screen, false, "e6") == Some("chop.region_end_pos")
                }
                // Knobs 5 and 6 turned to the ends clear the trim (138).
                Action::ChopRegionClear => {
                    profile.control_action(screen, false, "e5") == Some("chop.region_start_pos")
                        && profile.control_action(screen, false, "e6")
                            == Some("chop.region_end_pos")
                }
                _ => false,
            };
            let bank_ok = bank_select
                && profile.control_action(Screen::Pads, false, "right") == Some("smc.list_next");
            let tempo_ok = matches!(action, Action::TempoStep { .. })
                && profile.control_action_context(Screen::Pads, false, true, "e2")
                    == Some("transport.tempo");
            let direct_editor = match action {
                Action::PagePrev | Action::PageNext => match screen {
                    Screen::Pads | Screen::Fx => {
                        profile.control_action(screen, false, "right") == Some("smc.list_next")
                    }
                    Screen::Chop => true,
                    _ => false,
                },
                Action::ParamPrev | Action::ParamNext | Action::EditValue { .. } => match screen {
                    Screen::Pads => {
                        profile.control_action(screen, false, "e3") == Some("smc.page_3")
                    }
                    Screen::Fx => {
                        profile.control_action(screen, false, "e2") == Some("edit.param_1")
                    }
                    Screen::Chop => {
                        profile.control_action(screen, false, "e3") == Some("slice.start")
                    }
                    _ => false,
                },
                Action::SettingsValue(_) => {
                    profile.control_action(screen, false, "e2") == Some("settings.value_next")
                }
                _ => false,
            };
            // Assign shows order in cell 4 of the Chop actions (127).
            let assign_order = action == Action::ChopAssignOrder && screen == Screen::Chop;
            assert!(
                fixed
                    || assign_order
                    || role_ok
                    || variant_role_ok
                    || bank_ok
                    || tempo_ok
                    || direct_editor
                    || pads_screen,
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
        {
            for slot in 1..=12 {
                let Some(cell) = panel.cell(view, slot) else {
                    continue;
                };
                let mut route = State::default();
                assert_eq!(route.press(SMC, 1), Press::Revealed);
                assert_eq!(route.page(), 1);
                assert_eq!(
                    route.choice(view, slot),
                    if cell.action == Action::Quit {
                        Choice::Exit
                    } else {
                        Choice::Action(cell.action)
                    },
                    "{view:?} cell {cell:?}"
                );
                let contextual = match cell.action {
                    Action::ChopCommit => has("chop.edit"),
                    Action::PatternClear => has("pad.clear"),
                    Action::PatternParamPrev => has("edit.param_prev"),
                    Action::PatternParamNext => has("edit.param_next"),
                    Action::PatternValueDec => has("edit.value_dec"),
                    Action::PatternValueInc => has("edit.value_inc"),
                    // Tab and Shift+Tab reach every pad edit page (138).
                    Action::PadPage(_) => has("edit.page_next"),
                    _ => false,
                };
                assert!(
                    keyboard.contains(&cell.action) || contextual,
                    "{view:?} cell {cell:?}: no keyboard route"
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
            "smc.list_prev" | "smc.list_next" => {
                has("bank.prev")
                    && has("bank.next")
                    && has("edit.page_next")
                    && has("browser.parent")
                    && has("browser.enter")
                    && has("settings.tab_next")
            }
            "smc.position" => has("pad.select_left") && has("pad.select_right"),
            "smc.page_2" | "smc.page_3" | "smc.page_4" | "smc.page_5" | "smc.page_6"
            | "smc.page_7" | "edit.param_1" | "edit.param_2" | "edit.param_3" => {
                has("edit.value_dec") && has("edit.value_inc")
            }
            "none" => true,
            "master.volume" => has("master.volume_down") && has("master.volume_up"),
            "transport.tempo" => has("transport.tempo_down") && has("transport.tempo_up"),
            "chop.cursor_pos" => has("chop.cursor_left") && has("chop.cursor_right"),
            "chop.threshold" => has("chop.threshold_up") && has("chop.threshold_down"),
            "chop.zoom" => has("chop.zoom_in") && has("chop.zoom_out"),
            "slice.start" => has("slice.start_left") && has("slice.start_right"),
            "slice.end" => has("slice.end_left") && has("slice.end_right"),
            "chop.region_start_pos" => has("chop.region_start"),
            "chop.region_end_pos" => has("chop.region_end"),
            "edit.page" => has("edit.page_prev") && has("edit.page_next"),
            "pad.volume" | "pad.pitch" | "pad.cutoff" | "pad.decay" | "pad.send_1"
            | "pad.send_2" | "pattern.bars" | "pattern.quantize" | "pattern.strength" => {
                has("edit.value_dec") && has("edit.value_inc")
            }
            "pad.select_left" | "pad.select_right" => {
                has("pad.select_left") && has("pad.select_right")
            }
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

#[derive(Debug, PartialEq)]
struct ThresholdState {
    db: f32,
    cuts: Vec<(u32, usize)>,
    epoch: u32,
    region: Region,
    link: Option<(u32, u32)>,
}

fn threshold_result(drive: &Drive) -> ThresholdState {
    let chop = drive.app.chop.as_ref().unwrap();
    ThresholdState {
        db: chop.threshold_db,
        cuts: chop.saved().cuts,
        epoch: chop.epoch,
        region: drive.app.pads.region[5],
        link: drive.app.pads.link[5],
    }
}

/// Scenario "Keyboard and encoder set the same Chop threshold" (controller-reach).
#[test]
fn keyboard_and_encoder_set_the_same_chop_threshold() {
    let mut samples = vec![0.0; 32_000];
    for (start, level) in [(2_000, 0.9), (10_000, 0.6), (18_000, 0.4), (26_000, 0.2)] {
        samples[start..start + 500].fill(level);
    }
    let sound = Arc::new(Sample::mono(samples));
    for view in [panel::View::Chop, panel::View::ChopEdit] {
        let mut keyboard = reach_drive(view, sound.clone());
        let mut controller = reach_drive(view, sound.clone());
        keyboard.link_chop_pad(5, 2);
        controller.link_chop_pad(5, 2);
        let initial = threshold_result(&keyboard);
        let at = Instant::now();
        for (i, (key, midi, db)) in [
            (KeyCode::Down, 65, -1.0),
            (KeyCode::Down, 65, -2.0),
            (KeyCode::Up, 63, -1.0),
        ]
        .into_iter()
        .enumerate()
        {
            keyboard.tap(key, M::CONTROL | M::ALT);
            controller.midi().recorded_bytes_at(
                SMC,
                &[0xB0, 36, midi],
                &controller.project,
                at + std::time::Duration::from_millis(100 * i as u64),
            );
            controller.frame();
            assert_eq!(keyboard.app.chop.as_ref().unwrap().threshold_db, db);
            assert_eq!(
                threshold_result(&keyboard),
                threshold_result(&controller),
                "{view:?} {db}"
            );
        }
        keyboard.tap(KeyCode::Char('z'), M::CONTROL);
        controller.tap(KeyCode::Char('z'), M::CONTROL);
        assert_eq!(threshold_result(&keyboard), threshold_result(&controller));
        assert_eq!(threshold_result(&keyboard), initial, "{view:?} Undo");

        for bound in [-40.0, 0.0] {
            let mut keyboard = reach_drive(view, sound.clone());
            let mut controller = reach_drive(view, sound.clone());
            for drive in [&mut keyboard, &mut controller] {
                drive
                    .app
                    .chop
                    .as_mut()
                    .unwrap()
                    .set_threshold(-bound / 40.0);
            }
            let before = threshold_result(&keyboard);
            let key = if bound == -40.0 {
                KeyCode::Down
            } else {
                KeyCode::Up
            };
            let midi = if bound == -40.0 { 65 } else { 63 };
            keyboard.tap(key, M::CONTROL | M::ALT);
            turn(&mut controller, 36, midi);
            assert_eq!(
                threshold_result(&keyboard),
                before,
                "{view:?} bound {bound}"
            );
            assert_eq!(threshold_result(&keyboard), threshold_result(&controller));
            keyboard.tap(KeyCode::Char('z'), M::CONTROL);
            controller.tap(KeyCode::Char('z'), M::CONTROL);
            assert_eq!(threshold_result(&keyboard), threshold_result(&controller));
            if bound == -40.0 {
                assert_eq!(keyboard.app.chop.as_ref().unwrap().threshold_db, 0.0);
            }
        }
    }
}

/// Scenario "Fast encoder turns use single threshold steps" (controller-reach).
#[test]
fn fast_encoder_turns_use_single_threshold_steps() {
    let sound = Arc::new(Sample::mono(vec![0.2; 32_000]));
    let mut keyboard = reach_drive(panel::View::Chop, sound.clone());
    let mut controller = reach_drive(panel::View::Chop, sound);
    let at = Instant::now();
    for i in 0..2 {
        keyboard.tap(KeyCode::Down, M::CONTROL | M::ALT);
        controller.midi().recorded_bytes_at(
            SMC,
            &[0xB0, 36, 65],
            &controller.project,
            at + std::time::Duration::from_millis(10 * i),
        );
        controller.frame();
    }
    assert_eq!(keyboard.app.chop.as_ref().unwrap().threshold_db, -2.0);
    assert_eq!(controller.app.chop.as_ref().unwrap().threshold_db, -2.0);
}

/// Scenario "Threshold keys respect owner bindings and input owners" (controller-reach).
#[test]
fn threshold_keys_respect_owner_bindings_and_input_owners() {
    let sound = Arc::new(Sample::mono(vec![0.2; 32_000]));
    for binding in ["none", "metronome.toggle"] {
        let mut drive = reach_drive(panel::View::Chop, sound.clone());
        assert!(
            drive
                .keymap
                .overlay(&format!(
                    "[screens.chop]\n\"Ctrl+Alt+Down\" = \"{binding}\"\n"
                ))
                .unwrap()
                .is_empty()
        );
        let before = threshold_result(&drive);
        let metro = drive.app.transport.metronome;
        drive.tap(KeyCode::Down, M::CONTROL | M::ALT);
        assert_eq!(threshold_result(&drive), before);
        assert_eq!(
            drive.app.transport.metronome,
            if binding == "none" { metro } else { !metro }
        );
    }
    for overlay in ["assign", "help", "question", "learn"] {
        let mut drive = reach_drive(panel::View::Chop, sound.clone());
        match overlay {
            "assign" => drive.app.plan = Some(chop::Plan::InOrder(None)),
            "help" => {
                drive.tap(KeyCode::F(1), M::NONE);
            }
            "question" => {
                drive.app.ask = Some(Ask::ClearPads {
                    pads: vec![0],
                    lines: vec!["Clear?".into()],
                })
            }
            _ => {
                drive.tap(KeyCode::F(10), M::NONE);
            }
        }
        let before = threshold_result(&drive);
        drive.tap(KeyCode::Down, M::CONTROL | M::ALT);
        turn(&mut drive, 36, 65);
        assert_eq!(threshold_result(&drive), before, "{overlay}");
    }
}

/// Scenario "Exact Chop threshold keys resolve in both terminal modes" (keyboard).
#[test]
fn exact_chop_threshold_keys_resolve_in_both_terminal_modes() {
    let sound = Arc::new(Sample::mono(vec![0.2; 32_000]));
    for view in [panel::View::Chop, panel::View::ChopEdit] {
        for kitty in [false, true] {
            for (code, action, db) in [
                (KeyCode::Down, Action::ThresholdStep(-1), -1.0),
                (KeyCode::Up, Action::ThresholdStep(1), 0.0),
            ] {
                let mut drive = reach_drive(view, sound.clone());
                if code == KeyCode::Up {
                    drive.app.chop.as_mut().unwrap().set_threshold(1.0 / 40.0);
                }
                let key = KeyEvent::new(code, M::CONTROL | M::ALT);
                assert_eq!(
                    drive
                        .keymap
                        .resolve_in(Screen::Chop, view == panel::View::ChopEdit, &key),
                    Some(action)
                );
                assert!(!drive.app.keyboard_event(
                    &drive.keymap,
                    &key,
                    kitty,
                    &mut drive.down,
                    &mut drive.held,
                    &mut drive.mods,
                ));
                drive.frame();
                assert_eq!(
                    drive.app.chop.as_ref().unwrap().threshold_db,
                    db,
                    "{view:?} kitty={kitty}"
                );
            }
        }
    }
}

/// Scenario "Keyboard selects a neighboring pad without sound" (controller-reach).
#[test]
fn keyboard_selects_a_neighboring_pad_without_sound() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    for view in [panel::View::Pads, panel::View::PadEdit] {
        let mut keyboard = reach_drive(view, sound.clone());
        for (start, code, selected) in [
            (0, KeyCode::Left, 0),
            (0, KeyCode::Right, 1),
            (3, KeyCode::Right, 3),
            (4, KeyCode::Left, 4),
            (7, KeyCode::Left, 6),
            (15, KeyCode::Right, 15),
        ] {
            keyboard.app.pads.selected = start;
            keyboard.app.splash = true;
            let before = keyboard.seen();
            let key_frame = keyboard.tap(code, M::ALT);
            assert_eq!(keyboard.app.pads.selected, selected, "{view:?} {start}");
            assert_eq!(keyboard.seen().pads, before.pads, "{view:?} pad values");
            assert!(key_frame.commands().is_empty(), "{view:?} key hit");
            assert!(keyboard.app.splash, "{view:?} splash");
            keyboard.key(code, M::ALT, KeyEventKind::Repeat);
            keyboard.key(code, M::ALT, KeyEventKind::Release);
            assert!(keyboard.frame().commands().is_empty());
            assert_eq!(keyboard.app.pads.selected, selected, "{view:?} repeat");
        }
    }

    let mut keyboard = reach_drive(panel::View::Pads, sound);
    keyboard.app.pads.selected = 0;
    keyboard.app.pads.gate[0] = true;
    keyboard.key(KeyCode::Char('1'), M::NONE, KeyEventKind::Press);
    keyboard.frame();
    assert!(keyboard.app.pads.held[0]);
    assert!(keyboard.tap(KeyCode::Right, M::ALT).commands().is_empty());
    assert_eq!(keyboard.app.pads.selected, 1);
    keyboard.key(KeyCode::Char('1'), M::NONE, KeyEventKind::Release);
    let key_release = keyboard.frame();
    assert!(
        key_release
            .commands()
            .iter()
            .any(|command| matches!(command, Command::PadRelease { pad: 0 }))
    );
    assert!(!keyboard.app.pads.held[0]);
    assert_eq!(keyboard.app.pads.selected, 1);
}

/// Scenario "Silent selection respects owner bindings and overlays" (controller-reach).
#[test]
fn silent_selection_respects_owner_bindings_and_overlays() {
    for binding in ["none", "metronome.toggle"] {
        let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
        let mut drive = reach_drive(panel::View::Pads, sound);
        assert!(
            drive
                .keymap
                .overlay(&format!("[screens.pads]\n\"Alt+Right\" = \"{binding}\"\n"))
                .unwrap()
                .is_empty()
        );
        let before = drive.app.transport.metronome;
        let frame = drive.tap(KeyCode::Right, M::ALT);
        assert!(!frame.commands().iter().any(|command| matches!(
            command,
            Command::PadPress { .. } | Command::PadRelease { .. }
        )));
        assert_eq!(drive.app.pads.selected, 0, "owner {binding}");
        assert_eq!(
            drive.app.transport.metronome,
            if binding == "metronome.toggle" {
                !before
            } else {
                before
            },
            "owner {binding}"
        );
    }

    for overlay in ["text", "question", "picker", "help", "screen learn"] {
        let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
        let mut drive = reach_drive(panel::View::Pads, sound);
        match overlay {
            "text" => {
                drive.tap(KeyCode::Char('n'), M::CONTROL);
                assert!(drive.app.pad_naming.is_some());
            }
            "question" => {
                drive.app.ask = Some(Ask::ClearPads {
                    pads: vec![0],
                    lines: vec!["Clear?".into()],
                });
            }
            "picker" => {
                drive.tap(KeyCode::Char('p'), M::CONTROL);
                assert!(drive.app.project_picker.is_some());
            }
            "help" => {
                drive.tap(KeyCode::F(1), M::NONE);
                assert!(drive.app.help_open);
            }
            _ => {
                drive.tap(KeyCode::F(10), M::NONE);
                assert!(drive.app.screen_learn.is_some());
            }
        }
        drive.tap(KeyCode::Right, M::ALT);
        assert_eq!(drive.app.pads.selected, 0, "{overlay}");
    }

    let drive = smc_drive();
    for (code, right) in [(KeyCode::Left, false), (KeyCode::Right, true)] {
        assert!(matches!(
            drive
                .keymap
                .resolve_in(Screen::Chop, false, &KeyEvent::new(code, M::ALT)),
            Some(Action::SliceBound { end: true, right: resolved, fine: true }) if resolved == right
        ));
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
            cell(&mut drive, 55);
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

fn request_exit_with_save_failure(drive: &mut Drive, controller: bool) {
    if controller {
        hold(drive, true);
        list_button(drive, false);
        cell(drive, panel_note(12));
        assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
        let question = drive.app.question_view(&drive.keymap).unwrap();
        assert!(
            question
                .answers
                .iter()
                .any(|line| line.contains("P1: keep · P4: exit"))
        );
        hold(drive, false);
        cell(drive, 55); // Device pad 4 confirms (134).
        assert!(!drive.app.try_finish_exit());
        drive.app.publish_midi(&drive.shared);
    } else {
        drive.tap(KeyCode::Char('q'), M::CONTROL);
    }
    assert!(!drive.app.quit_requested);
    assert!(matches!(drive.app.ask, Some(Ask::ExitRecovery { .. })));
}

/// Scenarios "A prior save refusal stops exit", "Back leaves exit recovery with the edits",
/// "A refused retry keeps the recovery question" and "A successful retry returns to the saved session".
/// Scenario "Repeated retry input keeps one save".
#[test]
fn exit_recovery_uses_confirm_and_back_from_both_inputs() {
    for controller in [false, true] {
        let files = TransitionFiles::new();
        let mut drive = project_drive(&files);
        drive.app.session_read_error = None;
        drive.app.midi_gestures.smc_pad_bank_opens_panel = true;
        drive.midi().set_pad_bank_shortcut(true);
        drive.app.goto(Screen::Files);
        drive.app.save_session().unwrap();
        let path = drive.app.session_file();
        let before = std::fs::read(&path).unwrap();
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, b"occupied").unwrap();
        drive.app.pads.pitch[0] = 5;
        drive.app.dirty();
        let pending = drive.app.session_dirty;
        let refusal = drive.app.save_session().unwrap_err();
        drive.app.notices = Notices::new(Instant::now());
        std::fs::remove_file(&tmp).unwrap();

        request_exit_with_save_failure(&mut drive, controller);
        let question = drive.app.question_view(&drive.keymap).unwrap();
        assert_eq!(question.body[0], "Save failed · A");
        assert!(refusal.ends_with(&question.body[1]));
        // After an SMC-PAD input the answer row names only the pads (134).
        if !controller {
            assert!(question.answers[0].contains("Enter: Retry Save"));
            assert!(question.answers[0].contains("Esc: Back"));
        }
        assert!(
            question
                .answers
                .iter()
                .any(|line| line.contains("P1: Back") && line.contains("P4: Retry Save"))
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(drive.app.notices.history().is_empty());

        if controller {
            cell(&mut drive, 36);
        } else {
            drive.tap(KeyCode::Esc, M::NONE);
        }
        assert!(drive.app.ask.is_none());
        assert_eq!(drive.app.screen, Screen::Files);
        assert_eq!(drive.app.session_dirty, pending);
        assert_eq!(drive.app.pads.pitch[0], 5);

        request_exit_with_save_failure(&mut drive, controller);
        std::fs::write(&tmp, b"occupied again").unwrap();
        if controller {
            for note in [39, 55] {
                drive.bytes(SMC, &[0x99, note, 100]);
                drive.bytes(SMC, &[0x89, note, 0]);
            }
            drive.frame();
        } else {
            drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Press);
            drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Repeat);
            drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Release);
            drive.frame();
        }
        assert!(matches!(drive.app.ask, Some(Ask::ExitRecovery { .. })));
        assert_eq!(drive.app.session_dirty, pending);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(std::fs::read(&tmp).unwrap(), b"occupied again");
        assert_eq!(
            drive
                .app
                .notices
                .history()
                .iter()
                .filter(|notice| notice.level == Level::Error)
                .count(),
            1
        );

        std::fs::remove_file(&tmp).unwrap();
        if controller {
            for _ in 0..2 {
                drive.bytes(SMC, &[0x99, 39, 100]);
                drive.bytes(SMC, &[0x89, 39, 0]);
            }
            drive.frame();
        } else {
            drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Press);
            drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Repeat);
            drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Release);
            drive.frame();
        }
        assert!(drive.app.ask.is_none());
        assert_eq!(drive.app.screen, Screen::Files);
        assert!(drive.app.session_dirty.is_none());
        assert!(!drive.app.try_finish_exit());
        assert_eq!(drive.last_notice().text, "Session saved");
        assert_eq!(drive.app.frame_notice().as_deref(), Some("Session saved"));
        assert_eq!(
            drive
                .app
                .notices
                .history()
                .iter()
                .filter(|notice| notice.text == "Session saved")
                .count(),
            1
        );
        let saved = session::load_existing(&path).unwrap();
        assert_eq!(saved.pads["0"].pitch, 5);
    }
}

/// Scenario "The printed SMC-PAD numbers answer both exit questions".
#[test]
fn exit_questions_use_the_printed_pads_in_either_bank() {
    for shortcut_on in [false, true] {
        for offset in [0, 16] {
            let files = TransitionFiles::new();
            let mut drive = project_drive(&files);
            drive.app.session_read_error = None;
            drive.midi().set_pad_bank_shortcut(shortcut_on);
            if shortcut_on {
                drive.midi().recorded_smc_failure();
            }
            drive.app.save_session().unwrap();
            let path = drive.app.session_file();
            let before = std::fs::read(&path).unwrap();
            let blocker = path.with_extension("toml.tmp");
            std::fs::write(&blocker, b"occupied").unwrap();
            drive.app.pads.pitch[0] = 5;
            drive.app.dirty();
            let exit = |drive: &mut Drive| {
                drive.app.ask = Some(Ask::Exit {
                    lines: vec!["Exit ptsr?".into()],
                });
                drive.frame();
                drive.app.question_view(&drive.keymap).unwrap()
            };
            let question = exit(&mut drive);
            for note in [42, 43] {
                cell(&mut drive, note + offset);
                assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
            }
            cell(&mut drive, 36 + offset);
            assert!(drive.app.ask.is_none(), "P1 in bank offset {offset}");
            assert!(!drive.app.quit_requested);
            assert!(
                question
                    .answers
                    .iter()
                    .any(|line| line.contains("P1: keep · P4: exit"))
            );

            exit(&mut drive);
            cell(&mut drive, 39 + offset);
            assert!(drive.app.quit_requested, "P4 in bank offset {offset}");
            assert!(!drive.app.try_finish_exit());
            drive.app.publish_midi(&drive.shared);
            let question = drive.app.question_view(&drive.keymap).unwrap();
            assert!(
                question
                    .answers
                    .iter()
                    .any(|line| line.contains("P1: Back · P4: Retry Save"))
            );
            cell(&mut drive, 36 + offset);
            assert!(drive.app.ask.is_none());
            assert_eq!(std::fs::read(&path).unwrap(), before);

            exit(&mut drive);
            cell(&mut drive, 39 + offset);
            assert!(!drive.app.try_finish_exit());
            drive.app.publish_midi(&drive.shared);
            std::fs::remove_file(&blocker).unwrap();
            cell(&mut drive, 39 + offset);
            assert!(drive.app.ask.is_none());
            assert!(!drive.app.quit_requested);
            assert_eq!(session::load_existing(&path).unwrap().pads["0"].pitch, 5);
        }
    }
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
            cell(&mut drive, 55);
        } else {
            drive.tap(KeyCode::Enter, M::NONE);
        }
        assert!(drive.app.path_picker.is_none());
        assert_eq!(drive.app.paths.projects, files.0.join("B"));
    }
    let mut drive = folder_drive(&files);
    turn(&mut drive, 30, 65);
    cell(&mut drive, 52);
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

/// Scenario "Every current controller action has a keyboard route" (controller-reach).
#[test]
fn shipped_settings_value_knob_matches_keyboard_on_midi_rows() {
    for row in [0, 1] {
        let mut keyboard = smc_drive();
        let mut controller = smc_drive();
        for drive in [&mut keyboard, &mut controller] {
            drive.app.screen = Screen::Settings;
            drive.app.settings_tab = 2;
            drive.app.midi_row = drive
                .app
                .midi_list()
                .iter()
                .position(|item| match row {
                    0 => matches!(item, MidiListRow::Response),
                    _ => matches!(item, MidiListRow::Group { .. }),
                })
                .unwrap();
            drive.app.publish_midi(&drive.shared);
        }
        keyboard.tap(KeyCode::Right, M::NONE);
        turn(&mut controller, 31, 65);
        assert_eq!(
            controller.app.takeover.response,
            keyboard.app.takeover.response
        );
        assert_eq!(controller.app.midi_open, keyboard.app.midi_open);
        assert_eq!(controller.app.midi_row, keyboard.app.midi_row);
        if row == 0 {
            assert_ne!(
                controller.app.takeover.response,
                input::midi::Response::Jump
            );
        } else {
            assert!(!controller.app.midi_open.is_empty());
        }
    }
}

/// Scenario "Every current controller action has a keyboard route" (controller-reach).
#[test]
fn shipped_settings_row_knob_matches_keyboard_on_each_tab() {
    for tab in 0..4 {
        for (row, code, value) in [(0, KeyCode::Down, 65), (1, KeyCode::Up, 63)] {
            let mut keyboard = smc_drive();
            let mut controller = smc_drive();
            for drive in [&mut keyboard, &mut controller] {
                drive.app.screen = Screen::Settings;
                drive.app.settings_tab = tab;
                drive.app.settings_row = row;
                drive.app.midi_row = row;
                drive.app.publish_midi(&drive.shared);
            }
            let before = (keyboard.app.settings_row, keyboard.app.midi_row);
            keyboard.tap(code, M::NONE);
            turn(&mut controller, 30, value);
            let result = |drive: &Drive| (drive.app.settings_row, drive.app.midi_row);
            assert_ne!(result(&keyboard), before, "tab {tab} row {row}");
            assert_eq!(
                result(&controller),
                result(&keyboard),
                "tab {tab} row {row}"
            );
        }
    }
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
    cell(&mut drive, 55);
    assert!(drive.app.ask.is_none());
    assert!(drive.app.project_picker.is_some());
    assert!(!drive.app.panel.visible());
    drive.bytes(SMC, &[0x89, 48, 0]);
    drive.frame();
    assert!(!drive.app.pads.held[0]);
    cell(&mut drive, 52);
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

/// A visible panel keeps RIGHT from the bank; the A1 grid cell matches the keyboard.
#[test]
fn a1_grid_cell_matches_the_keyboard() {
    let mut keyboard = smc_drive();
    let mut controller = smc_drive();
    let before = controller.app.pattern_view.division.grid();
    keyboard.tap(KeyCode::Up, M::ALT);

    let mut hidden = smc_drive();
    list_button(&mut hidden, true);
    assert_eq!(
        hidden.app.pads.bank, 1,
        "a hidden panel leaves RIGHT to the bank"
    );
    assert_eq!(hidden.app.pattern_view.division.grid(), before);
    cell(&mut controller, 64);
    assert_eq!(controller.app.panel.page(), 1);
    let master = controller.app.master_db;
    controller.bytes(XS_PORT, &[0xB0, 14, 0]);
    controller.frame();
    assert_ne!(controller.app.master_db, master);
    assert_eq!(controller.app.panel.page(), 1);
    list_button(&mut controller, true);
    assert_eq!(controller.app.panel.page(), 1);
    assert_eq!(controller.app.pads.bank, 0, "a visible panel takes RIGHT");
    // Grid moved to the A1 actions (127): LEFT, then A1.
    list_button(&mut controller, false);
    cell(&mut controller, panel_note(4));
    let finer = slot_of(panel::View::Pattern, "steps finer");
    assert_eq!(
        controller.app.panel_snapshot().unwrap().labels[usize::from(finer - 1)].as_deref(),
        Some("steps finer")
    );
    cell(&mut controller, panel_note(finer));
    assert_ne!(controller.app.pattern_view.division.grid(), before);
    assert_eq!(
        controller.app.pattern_view.division.grid(),
        keyboard.app.pattern_view.division.grid()
    );
    assert_eq!(controller.app.pads.bank, keyboard.app.pads.bank);
}

/// Scenario "Help cells navigate without acting behind Help" (controller-reach): LEFT and RIGHT select tabs (134).
#[test]
fn help_cells_navigate_without_acting_behind_help() {
    let mut controller = smc_drive();
    let mut keyboard = smc_drive();
    let selected = controller.app.pads.selected_abs();
    cell(&mut controller, 64);
    list_button(&mut controller, false);
    cell(&mut controller, panel_note(8));
    keyboard.tap(KeyCode::F(1), M::NONE);
    keyboard.tap(KeyCode::Tab, M::NONE);
    keyboard.tap(KeyCode::Tab, M::NONE);
    assert!(controller.app.help_open);
    assert_eq!(
        controller.app.help_overlay.page,
        keyboard.app.help_overlay.page
    );

    for _ in 0..3 {
        list_button(&mut controller, true);
        keyboard.tap(KeyCode::Tab, M::NONE);
        assert_eq!(
            controller.app.help_overlay.page,
            keyboard.app.help_overlay.page
        );
    }

    let before = controller.app.controller_view(&controller.keymap).detail;
    cell(&mut controller, 57);
    let after = controller.app.controller_view(&controller.keymap).detail;
    assert_ne!(after, before);
    cell(&mut controller, 56);
    assert_eq!(
        controller.app.controller_view(&controller.keymap).detail,
        before
    );
    list_button(&mut controller, false);
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

/// The screens view opens Help and Setup; Mixer stays dim.
#[test]
fn the_screens_view_opens_help_and_setup() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    for view in ALL_VIEWS {
        let mut drive = reach_drive(view, sound.clone());
        let screen = drive.app.screen;
        cell(&mut drive, panel_note(1));
        list_button(&mut drive, false);
        let main = drive.app.panel_snapshot().unwrap();
        assert_eq!(main.labels[5].as_deref(), Some("mixer"));
        assert_eq!(main.labels[6].as_deref(), Some("setup"));
        cell(&mut drive, panel_note(6));
        assert_eq!(drive.app.screen, screen);
        assert!(!drive.app.help_open);
        assert_eq!(drive.app.panel.label(view, 8), Some("help"));
        cell(&mut drive, panel_note(8));
        assert!(drive.app.help_open, "{view:?}");
        assert_eq!(drive.app.screen, screen);
    }

    let mut drive = smc_drive();
    cell(&mut drive, panel_note(1));
    list_button(&mut drive, false);
    cell(&mut drive, panel_note(7));
    assert_eq!(drive.app.screen, Screen::Settings);
}

/// Scenario "Exit needs a second SMC-PAD press" (controller-reach).
#[test]
fn exit_needs_a_second_smc_press() {
    let mut drive = smc_drive();
    drive.app.session_dirty = Some(Instant::now());
    cell(&mut drive, 64);
    list_button(&mut drive, false);
    assert_eq!(drive.app.panel.page(), 0);
    cell(&mut drive, panel_note(12));
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    assert!(!drive.app.quit_requested);
    assert!(drive.app.session_dirty.is_some());
    cell(&mut drive, 52);
    assert!(drive.app.ask.is_none());
    assert!(!drive.app.quit_requested);
    assert_eq!(drive.app.panel.page(), 0);

    cell(&mut drive, panel_note(12));
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    cell(&mut drive, 52);
    assert!(drive.app.ask.is_none());
    assert!(!drive.app.quit_requested);

    cell(&mut drive, panel_note(12));
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    cell(&mut drive, 55);
    assert!(drive.app.ask.is_none());
    assert!(drive.app.quit_requested);
    assert_eq!(drive.app.panel.page(), 0);

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
    list_button(&mut drive, false);
    assert_eq!(drive.app.panel.page(), 0);
    cell(&mut drive, panel_note(12));
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    assert!(drive.app.pending_manual_save.is_some());
    assert!(!drive.app.quit_requested);
    cell(&mut drive, 52);
    assert!(drive.app.ask.is_none());
    assert!(drive.app.pending_manual_save.is_some());
    assert!(!drive.app.quit_requested);

    cell(&mut drive, panel_note(12));
    cell(&mut drive, 55);
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
    let mode = slot_of(panel::View::Pads, "play mode");
    for slot in [1, mode] {
        cell(&mut first, panel_note(slot));
        first_cell_hits += 1;
    }
    assert!(first.app.pads.gate[0]);
    assert_eq!(1 + first_cell_hits, 3); // PAD BANK plus reveal and Mode.

    let mut repeated = smc_drive();
    cell(&mut repeated, 64); // Reveal before the counted visible route.
    cell(&mut repeated, panel_note(mode));
    assert!(repeated.app.pads.gate[0]);

    let mut shortcut = smc_drive();
    shortcut.app.set_smc_pad_bank(true);
    pad_bank_request(&mut shortcut, SMC);
    cell(&mut shortcut, panel_note(mode));
    assert!(shortcut.app.pads.gate[0]); // PAD BANK and Mode: two presses.

    let mut screen = smc_drive();
    screen.app.set_smc_pad_bank(true);
    pad_bank_request(&mut screen, SMC);
    list_button(&mut screen, false);
    cell(&mut screen, panel_note(2));
    assert_eq!(screen.app.screen, Screen::Files); // PAD BANK, LEFT and Files.

    let mut later = smc_drive();
    cell(&mut later, panel_note(1));
    list_button(&mut later, false);
    assert_eq!(later.app.panel.page(), 0);
    assert_eq!(later.app.panel.label(panel::View::Pads, 4), Some("A1"));
    // PAD BANK, the reveal, LEFT and A1: four presses; visible: two.

    let mut knob = smc_drive();
    let db = knob.app.master_db;
    turn(&mut knob, 37, 63);
    assert_eq!(knob.app.master_db, db - 1, "{:?}", knob.app.input_log);
    assert_eq!(knob.app.pads.bank, 0);
}

/// The A1 knobs cell opens the same pattern controls as the keyboard.
#[test]
fn a1_knobs_cell_matches_the_keyboard() {
    let mut keyboard = smc_drive();
    let mut controller = smc_drive();
    keyboard.tap(KeyCode::PageDown, M::NONE);
    list_button(&mut controller, true);
    assert_eq!(controller.app.pads.bank, keyboard.app.pads.bank);

    keyboard.tap(KeyCode::Char('n'), M::ALT);
    cell(&mut controller, 64);
    list_button(&mut controller, false);
    cell(&mut controller, panel_note(4));
    assert_eq!(controller.app.panel_view(), panel::View::Pattern);
    let setup = slot_of(panel::View::Pattern, "A1 setup");
    assert_eq!(
        controller.app.panel_snapshot().unwrap().labels[usize::from(setup - 1)].as_deref(),
        Some("A1 setup")
    );
    cell(&mut controller, panel_note(setup));
    assert_eq!(controller.app.pattern_panel, keyboard.app.pattern_panel);
    assert!(controller.app.pattern_panel.is_some());
}

/// Scenario "Tempo turns act only on the Pattern transport page" (controller-reach).
#[test]
fn tempo_turns_act_only_on_the_pattern_transport_page() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    for view in ALL_VIEWS {
        let mut drive = reach_drive(view, sound.clone());
        let bpm = drive.app.transport.bpm;
        turn(&mut drive, 36, 65);
        turn(&mut drive, 31, 65);
        assert_eq!(drive.app.transport.bpm, bpm, "{view:?}");
    }

    let mut keyboard = reach_drive(panel::View::Pattern, sound.clone());
    let mut controller = reach_drive(panel::View::Pattern, sound);
    for drive in [&mut keyboard, &mut controller] {
        drive.tap(KeyCode::Char('n'), M::ALT);
        assert!(drive.app.pattern_panel.is_some());
    }
    let bpm = keyboard.app.transport.bpm;
    let at = Instant::now() + std::time::Duration::from_millis(100);
    keyboard.tap(KeyCode::Up, M::CONTROL);
    timed_knob_two(&mut controller, 65, at);
    assert_eq!(controller.app.transport.bpm, keyboard.app.transport.bpm);
    assert!((controller.app.transport.bpm - bpm - 1.0).abs() < 0.0001);
    timed_knob_two(
        &mut controller,
        65,
        at + std::time::Duration::from_millis(10),
    );
    assert!((controller.app.transport.bpm - bpm - 2.0).abs() < 0.0001);
    controller.tap(KeyCode::Char('n'), M::ALT);
    assert!(controller.app.pattern_panel.is_none());
    let bpm = controller.app.transport.bpm;
    timed_knob_two(
        &mut controller,
        65,
        at + std::time::Duration::from_millis(100),
    );
    assert_eq!(controller.app.transport.bpm, bpm);
}

/// Scenario "Shared transport key keeps previews on Files and Chop" (keyboard).
#[test]
fn shared_transport_key_keeps_previews_on_files_and_chop() {
    for screen in [Screen::Files, Screen::Chop] {
        for (letter, kitty) in [('u', true), ('г', false)] {
            let mut keyboard = smc_drive();
            let mut controller = smc_drive();
            for drive in [&mut keyboard, &mut controller] {
                drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 48_000])), "preview");
                drive.tap(
                    if screen == Screen::Files {
                        KeyCode::F(6)
                    } else {
                        KeyCode::F(3)
                    },
                    M::NONE,
                );
                assert_eq!(drive.app.screen, screen);
            }

            for kind in [
                KeyEventKind::Press,
                KeyEventKind::Repeat,
                KeyEventKind::Release,
            ] {
                keyboard.app.keyboard_event(
                    &keyboard.keymap,
                    &KeyEvent::new_with_kind(KeyCode::Char(letter), M::ALT, kind),
                    kitty,
                    &mut keyboard.down,
                    &mut keyboard.held,
                    &mut keyboard.mods,
                );
                let frame = keyboard.frame();
                assert_eq!(
                    frame
                        .commands()
                        .iter()
                        .filter(|command| matches!(command, Command::TransportToggle))
                        .count(),
                    usize::from(kind == KeyEventKind::Press),
                    "{screen:?} {letter} {kind:?}"
                );
            }

            let frame = keyboard.tap(KeyCode::Char(' '), M::NONE);
            assert!(
                !frame
                    .commands()
                    .iter()
                    .any(|command| matches!(command, Command::TransportToggle)),
                "{screen:?} Space changed transport"
            );
            if screen == Screen::Files {
                assert_ne!(keyboard.app.preview_on, controller.app.preview_on);
            } else {
                assert!(
                    frame
                        .commands()
                        .iter()
                        .any(|command| matches!(command, Command::Preview { .. }))
                );
            }
        }
    }
}

/// Scenario "Owner transport key and modal input keep priority" (keyboard).
#[test]
fn owner_transport_key_and_modal_input_keep_priority() {
    for binding in ["none", "metronome.toggle"] {
        let mut drive = smc_drive();
        drive
            .keymap
            .overlay(&format!("[global]\n\"Alt+U\" = \"{binding}\"\n"))
            .unwrap();
        for screen in Screen::ALL {
            drive.app.screen = screen;
            let before_metronome = drive.app.transport.metronome;
            let frame = drive.tap(KeyCode::Char('u'), M::ALT);
            assert!(
                !frame
                    .commands()
                    .iter()
                    .any(|command| matches!(command, Command::TransportToggle)),
                "{screen:?} owner {binding}"
            );
            assert_eq!(
                drive.app.transport.metronome,
                if binding == "metronome.toggle" {
                    !before_metronome
                } else {
                    before_metronome
                },
                "{screen:?} owner {binding}"
            );
        }
    }

    for overlay in ["text", "question", "picker", "screen learn", "midi learn"] {
        let mut drive = smc_drive();
        drive.app.screen = Screen::Files;
        match overlay {
            "text" => {
                drive.tap(KeyCode::Char('/'), M::NONE);
                assert!(drive.app.browser.editing);
            }
            "question" => {
                drive.app.ask = Some(Ask::ClearPads {
                    pads: vec![0],
                    lines: vec!["Clear?".into()],
                });
            }
            "picker" => {
                drive.tap(KeyCode::Char('p'), M::CONTROL);
                assert!(drive.app.project_picker.is_some());
            }
            "screen learn" => {
                drive.tap(KeyCode::F(10), M::NONE);
                assert!(drive.app.screen_learn.is_some());
            }
            _ => {
                drive.tap(KeyCode::F(12), M::NONE);
                drive.app.settings_tab = 2;
                drive.app.midi_open.push(input::Group::Global);
                drive.app.midi_row = drive
                    .app
                    .midi_list()
                    .iter()
                    .position(|row| matches!(row, MidiListRow::Action { info } if info.name == "transport.toggle"))
                    .unwrap();
                drive.tap(KeyCode::Enter, M::NONE);
                assert!(drive.app.learning.is_some());
            }
        }
        let frame = drive.tap(KeyCode::Char('u'), M::ALT);
        assert!(
            !frame
                .commands()
                .iter()
                .any(|command| matches!(command, Command::TransportToggle)),
            "{overlay}"
        );
    }
}

#[test]
fn grid_cell_keeps_the_screen_and_matches_ctrl_o() {
    // The grid cell is on Pads action page 1 (127).
    for screen in [Screen::Pads] {
        let mut keyboard = smc_drive();
        let mut controller = smc_drive();
        for drive in [&mut keyboard, &mut controller] {
            drive.app.screen = screen;
            drive.app.publish_midi(&drive.shared);
        }
        keyboard.tap(KeyCode::Char('o'), M::CONTROL);
        cell(&mut controller, 64);
        assert_eq!(controller.app.panel.page(), 1);
        cell(
            &mut controller,
            panel_note(slot_of(panel::View::Pads, "hide pads")),
        );
        assert_eq!(controller.app.ui.hide_pads, keyboard.app.ui.hide_pads);
        assert_eq!(controller.app.screen, screen);
    }
}

#[test]
fn pattern_record_and_erase_cells_match_keyboard_modes() {
    let erase = panel_note(slot_of(panel::View::Pattern, "erase hits"));
    let record = panel_note(slot_of(panel::View::Pattern, "record A1"));
    for (note, code, modifiers) in [
        (erase, KeyCode::Char('e'), M::ALT),
        (record, KeyCode::Char('r'), M::CONTROL),
    ] {
        let mut keyboard = smc_drive();
        let mut controller = smc_drive();
        if note == erase {
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
        list_button(&mut controller, false);
        cell(&mut controller, panel_note(4)); // A1 actions (127).
        assert_eq!(controller.app.panel_view(), panel::View::Pattern);
        let controller_frame = cell(&mut controller, note);
        assert_eq!(controller.app.erase_pads, keyboard.app.erase_pads);
        assert_eq!(
            controller
                .app
                .pattern_record_state()
                .map(|state| state.phase),
            keyboard.app.pattern_record_state().map(|state| state.phase)
        );
        if note == erase {
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

/// Scenario "A1 guides a first recording from keyboard and SMC-PAD".
/// Scenario "A1 offers playback only for the accepted state".
#[test]
fn a1_guides_a_first_recording_from_keyboard_and_smc_pad() {
    use super::patterns::app_capture_frames;

    for controller in [false, true] {
        let mut drive = if controller {
            hold_drive()
        } else {
            Drive::offline()
        };
        let button = |drive: &mut Drive, cc| {
            drive.bytes(SMC, &[0xB0, cc, 127]);
            drive.frame();
            drive.bytes(SMC, &[0xB0, cc, 0]);
            drive.frame();
        };
        let record = |drive: &mut Drive| {
            if controller {
                button(drive, 29);
            } else {
                drive.tap(KeyCode::Char('r'), M::CONTROL);
            }
        };
        let cue = |drive: &Drive, keyboard: &str, smc: &str| {
            assert_eq!(
                drive.app.pattern_overview(&drive.keymap).next.as_deref(),
                Some(if controller { smc } else { keyboard }),
            );
        };
        if controller {
            screens_cell(&mut drive, 4);
        } else {
            drive.tap(KeyCode::F(5), M::NONE);
        }
        assert!(drive.app.pattern_view.overview);
        cue(&drive, "Ctrl+R record A1", "RECORD record A1");

        record(&mut drive);
        cue(&drive, "Starting A1 · wait", "Starting A1 · wait");
        app_capture_frames(&mut drive, 0);
        cue(&drive, "Ctrl+R cancel count", "RECORD cancel count");
        record(&mut drive);
        app_capture_frames(&mut drive, 0);
        drive.app.flush_audio();
        app_capture_frames(&mut drive, 0);
        cue(&drive, "Ctrl+R record A1", "RECORD record A1");

        record(&mut drive);
        app_capture_frames(&mut drive, 96_000);
        cue(&drive, "Hit pads · Ctrl+R end", "Hit pads · RECORD end");
        if controller {
            cell(&mut drive, 48);
        } else {
            drive.tap(KeyCode::Char('1'), M::NONE);
        }
        app_capture_frames(&mut drive, 128);
        record(&mut drive);
        cue(&drive, "Finishing A1 · wait", "Finishing A1 · wait");
        app_capture_frames(&mut drive, 0);
        cue(&drive, "Finishing A1 · wait", "Finishing A1 · wait");
        drive.app.flush_audio();
        app_capture_frames(&mut drive, 128);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 1);
        cue(&drive, "Space stop A1", "STOP pause A1");

        if controller {
            button(&mut drive, 28);
        } else {
            drive.tap(KeyCode::Char(' '), M::NONE);
        }
        app_capture_frames(&mut drive, 128);
        cue(&drive, "Space play A1", "PLAY play A1");
        if controller {
            button(&mut drive, 27);
        } else {
            drive.tap(KeyCode::Char(' '), M::NONE);
        }
        app_capture_frames(&mut drive, 128);
        cue(&drive, "Space stop A1", "STOP pause A1");
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
                cell(
                    &mut controller,
                    panel_note(slot_of(panel::View::Pads, "mark pad")),
                );
            }
            "edit" => {
                keyboard.tap(KeyCode::Enter, M::NONE);
                cell(
                    &mut controller,
                    panel_note(slot_of(panel::View::Pads, "edit pad")),
                );
            }
            "copy" => {
                keyboard.tap(KeyCode::Char('c'), M::CONTROL);
                cell(
                    &mut controller,
                    panel_note(slot_of(panel::View::Pads, "copy pad")),
                );
            }
            "swap" => {
                keyboard.tap(KeyCode::Char('x'), M::CONTROL);
                cell(
                    &mut controller,
                    panel_note(slot_of(panel::View::Pads, "swap pads")),
                );
            }
            _ => {
                // Bindings change from the keyboard only (138): F10 on both.
                keyboard.tap(KeyCode::F(10), M::NONE);
                controller.tap(KeyCode::F(10), M::NONE);
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
    // Copy is the action to learn.
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pads, "copy pad")),
    );
    // Learn opens from the keyboard only (138).
    drive.tap(KeyCode::F(10), M::NONE);
    assert_eq!(drive.app.learn_picked().unwrap().0, "pad.copy");
    cell(&mut drive, 55);
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
        input::midi::Key::Note(55)
    );
    cell(&mut drive, 55);
    assert!(drive.app.screen_learn.is_none());
    let bindings = learn::group_bindings(&drive.app.learned, "SMC-PAD", input::Group::Pads);
    assert_eq!(
        bindings.get(&input::midi::Key::Note(55)).unwrap().primary(),
        "pad.copy"
    );

    let mut cancelled = smc_drive();
    cancelled.app.screen = Screen::Settings;
    cancelled.app.settings_tab = 2;
    cancelled.app.learning = Some(learn::Learning::new(action_info("pad.copy")));
    cancelled.app.publish_midi(&cancelled.shared);
    cell(&mut cancelled, 55);
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
        input::midi::Key::Note(55)
    );
    cell(&mut cancelled, 52);
    assert!(cancelled.app.learning.is_none());
    let bindings = learn::group_bindings(&cancelled.app.learned, "SMC-PAD", input::Group::Pads);
    assert!(
        bindings
            .get(&input::midi::Key::Note(55))
            .is_none_or(|b| b.primary() != "pad.copy")
    );

    cancelled.app.learning = Some(learn::Learning::new(action_info("pad.copy")));
    cancelled.app.publish_midi(&cancelled.shared);
    // Device pad 1 with no choice pending leaves learning (134).
    cell(&mut cancelled, 52);
    assert!(cancelled.app.learning.is_none());
    let bindings = learn::group_bindings(&cancelled.app.learned, "SMC-PAD", input::Group::Pads);
    assert!(!bindings.contains_key(&input::midi::Key::Note(52)));
}

/// The panel slot (1-12, top-down) of a shipped label in `view`: tests follow the layout (138).
fn slot_of(view: panel::View, label: &str) -> u8 {
    let state = panel::State::default();
    (1..=12)
        .find(|&slot| state.label(view, slot) == Some(label))
        .unwrap_or_else(|| panic!("no {label} in {view:?}"))
}

fn panel_note(slot: u8) -> u8 {
    [
        64, 65, 66, 67, 60, 61, 62, 63, 56, 57, 58, 59, 52, 53, 54, 55,
    ][usize::from(slot - 1)]
}

#[test]
fn keyboard_and_smc_pad_switch_and_pin_the_pads_strip() {
    let mut keyboard = Drive::offline();
    let mut smc = hold_drive();
    let sound = Arc::new(Sample::mono(vec![0.2; 2_048]));
    for drive in [&mut keyboard, &mut smc] {
        drive.load_pad(0, sound.clone(), "break");
        drive.app.publish_midi(&drive.shared);
        assert_eq!(drive.app.source_view.view_name(), "Sound");
    }
    for (target, key_label) in [("Rhythm", "Sound"), ("Sound", "Rhythm")] {
        keyboard.tap(KeyCode::Char('9'), M::ALT);
        hold(&mut smc, true);
        cell(&mut smc, panel_note(4));
        assert_eq!(smc.app.panel_snapshot().unwrap().states[3], Some(target));
        hold(&mut smc, false);
        for drive in [&keyboard, &smc] {
            assert_eq!(drive.app.source_view.view_name(), target);
            let groups = drive
                .app
                .pattern_help_groups(&drive.keymap, vec![], None, true);
            let key = if std::ptr::eq(drive, &keyboard) {
                "Alt 9"
            } else {
                "PAD BANK+P16"
            };
            assert!(
                groups
                    .iter()
                    .flat_map(|group| &group.items)
                    .any(|item| item == &(key.into(), key_label.into()))
            );
        }
    }
    keyboard.tap(KeyCode::Char('0'), M::ALT);
    hold(&mut smc, true);
    cell(&mut smc, panel_note(slot_of(panel::View::Pads, "edit pad")));
    assert!(smc.app.edit);
    cell(&mut smc, panel_note(4));
    assert_eq!(smc.app.panel_snapshot().unwrap().states[3], Some("on"));
    cell(&mut smc, panel_note(13));
    hold(&mut smc, false);
    assert!(!smc.app.edit);
    assert!(
        keyboard.app.source_view.pinned(&keyboard.app.pads)
            && smc.app.source_view.pinned(&smc.app.pads)
    );
}

const ALL_VIEWS: [panel::View; 8] = [
    panel::View::Pads,
    panel::View::PadEdit,
    panel::View::Pattern,
    panel::View::Files,
    panel::View::Chop,
    panel::View::ChopEdit,
    panel::View::Fx,
    panel::View::Settings,
];

fn key_chord(name: &str) -> (KeyCode, M) {
    let mut modifiers = M::NONE;
    let mut key = "";
    for part in name.split('+') {
        match part {
            "Ctrl" => modifiers |= M::CONTROL,
            "Alt" => modifiers |= M::ALT,
            "Shift" => modifiers |= M::SHIFT,
            _ => key = part,
        }
    }
    let code = match key {
        "↑" => KeyCode::Up,
        "↓" => KeyCode::Down,
        "←" => KeyCode::Left,
        "→" => KeyCode::Right,
        "PgUp" => KeyCode::PageUp,
        "PgDn" => KeyCode::PageDown,
        "Enter" => KeyCode::Enter,
        "Esc" => KeyCode::Esc,
        "Tab" if modifiers.contains(M::SHIFT) => KeyCode::BackTab,
        "Tab" => KeyCode::Tab,
        "Bksp" => KeyCode::Backspace,
        "Del" => KeyCode::Delete,
        "Space" => KeyCode::Char(' '),
        other if other.starts_with('F') && other[1..].parse::<u8>().is_ok() => {
            KeyCode::F(other[1..].parse().unwrap())
        }
        other if other.chars().count() == 1 => {
            KeyCode::Char(other.chars().next().unwrap().to_ascii_lowercase())
        }
        _ => panic!("unsupported shipped key {name}"),
    };
    (code, modifiers)
}

fn reach_drive(view: panel::View, sound: Arc<Sample>) -> Drive {
    reach_drive_from(smc_drive(), view, sound)
}

fn reach_drive_from(mut drive: Drive, view: panel::View, sound: Arc<Sample>) -> Drive {
    drive.load_pad(0, sound.clone(), "source");
    match view {
        panel::View::Pads | panel::View::PadEdit | panel::View::Pattern => {
            drive.app.screen = Screen::Pads;
            drive.app.edit = view == panel::View::PadEdit;
            drive.app.pattern_view.overview = view == panel::View::Pattern;
        }
        panel::View::Files => drive.app.screen = Screen::Files,
        panel::View::Chop | panel::View::ChopEdit => {
            drive.prepare_chop(sound, 4);
            drive.app.edit = view == panel::View::ChopEdit;
        }
        panel::View::Fx => drive.app.screen = Screen::Fx,
        panel::View::Settings => drive.app.screen = Screen::Settings,
    }
    drive.app.publish_midi(&drive.shared);
    drive
}

fn reach_result(drive: &Drive, frame: &Frame) -> (Seen, String, Vec<String>) {
    let extra = format!(
        "{:?} {:?}",
        (
            drive.app.settings_tab,
            drive.app.settings_row,
            drive.app.midi_row,
            drive.app.transport.bpm,
            drive.app.transport.metronome,
            drive.app.ui.hide_pads,
            drive.app.preview_on,
            drive.app.help_open,
            drive.app.source_view.view_name(),
            drive.app.source_view.pinned(&drive.app.pads),
        ),
        (
            drive.app.help_overlay.page,
            drive.app.pattern_view.overview,
            drive.app.pattern_view.edit,
            drive.app.pattern_panel,
            drive.app.screen_learn.is_some(),
            drive
                .app
                .project_picker
                .as_ref()
                .map(|picker| picker.selected),
            drive.app.path_picker.as_ref().map(|picker| picker.selected),
            drive.app.quit_requested,
        )
    );
    let commands = frame
        .commands()
        .iter()
        .map(|command| format!("{:?}", std::mem::discriminant(*command)))
        .collect();
    (drive.seen(), extra, commands)
}

/// Scenario "Every current controller action has a keyboard route" (controller-reach).
/// The matrix compares 53 action cells through production keyboard and MIDI input, so it is slow.
#[test]
fn shipped_panel_cells_match_keyboard_results_in_each_view() {
    use panel::{Config, View};

    let config = Config::parse(
        "config/panel.toml",
        include_str!("../../../../config/panel.toml"),
    );
    assert!(config.warnings.is_empty(), "{:?}", config.warnings);
    let mut checked = 0;
    let mut special = 0;
    for view in ALL_VIEWS {
        {
            for slot in 1..=12 {
                let Some(cell_info) = config.cell(view, slot) else {
                    continue;
                };
                let action = cell_info.action;
                if matches!(
                    action,
                    Action::Quit
                        | Action::PadPage(_)
                        | Action::ChopCommit
                        | Action::PatternClear
                        | Action::PatternParamPrev
                        | Action::PatternParamNext
                        | Action::PatternValueDec
                        | Action::PatternValueInc
                ) {
                    special += 1;
                    continue;
                }
                let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
                let mut keyboard = reach_drive(view, sound.clone());
                let mut controller = reach_drive(view, sound);
                let editing = keyboard.app.edit_keys();
                let key = keyboard
                    .keymap
                    .keys_for(keyboard.app.screen, action)
                    .into_iter()
                    .chain(keyboard.keymap.edit_keys_for(action))
                    .find_map(|name| {
                        let (code, modifiers) = key_chord(&name);
                        [KeyEventKind::Press, KeyEventKind::Repeat]
                            .into_iter()
                            .find(|kind| {
                                keyboard.keymap.resolve_in(
                                    keyboard.app.screen,
                                    editing,
                                    &KeyEvent::new_with_kind(code, modifiers, *kind),
                                ) == Some(action)
                            })
                            .map(|kind| (name, code, modifiers, kind))
                    })
                    .unwrap_or_else(|| panic!("{view:?} cell {slot}: no key for {action:?}"));
                let keyboard_frame = if key.3 == KeyEventKind::Repeat {
                    for drive in [&mut keyboard, &mut controller] {
                        drive.key(key.1, key.2, KeyEventKind::Press);
                        drive.frame();
                    }
                    controller.key(key.1, key.2, KeyEventKind::Release);
                    controller.frame();
                    keyboard.key(key.1, key.2, KeyEventKind::Repeat);
                    keyboard.key(key.1, key.2, KeyEventKind::Release);
                    keyboard.frame()
                } else {
                    keyboard.tap(key.1, key.2)
                };
                cell(&mut controller, 64);
                assert_eq!(
                    controller.app.panel.page(),
                    1,
                    "{view:?} {action:?}: {:?}",
                    controller.app.input_log
                );
                if action == Action::HelpToggle {
                    keyboard.tap(KeyCode::Tab, M::NONE);
                    keyboard.tap(KeyCode::Tab, M::NONE);
                }
                // A grey cell only says why (138); its key keeps the keyboard result.
                let grey = controller.app.panel_snapshot().unwrap().dim[usize::from(slot - 1)];
                let still = controller.frame();
                let before = reach_result(&controller, &still);
                let controller_frame = cell(&mut controller, panel_note(slot));
                let mut result = reach_result(&controller, &controller_frame);
                if grey {
                    assert!(
                        result.0.notice.is_some(),
                        "{view:?} cell {slot}: a grey press says why"
                    );
                    result.0.notice = before.0.notice.clone();
                    assert_eq!(
                        result.0, before.0,
                        "{view:?} cell {slot}: a grey press acts"
                    );
                } else {
                    // notices name the undo of the device that pressed (138)
                    result.0.notice = result.0.notice.map(|n| n.replace("P2 undo", "Ctrl+Z"));
                    for line in &mut result.0.question {
                        *line = line.replace("P2 undo", "Ctrl+Z");
                    }
                    assert_eq!(
                        result,
                        reach_result(&keyboard, &keyboard_frame),
                        "{view:?} cell {slot} {action:?} key {}",
                        key.0
                    );
                }
                checked += 1;
            }
        }
    }
    assert_eq!(checked + special, 53);

    for view in ALL_VIEWS {
        let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
        let mut keyboard = reach_drive(view, sound.clone());
        let mut controller = reach_drive(view, sound);
        assert!(keyboard.app.keyboard_event(
            &keyboard.keymap,
            &KeyEvent::new_with_kind(KeyCode::Char('q'), M::CONTROL, KeyEventKind::Press),
            true,
            &mut keyboard.down,
            &mut keyboard.held,
            &mut keyboard.mods,
        ));
        cell(&mut controller, 64);
        list_button(&mut controller, false);
        cell(&mut controller, panel_note(12));
        assert!(
            matches!(controller.app.ask, Some(Ask::Exit { .. })),
            "{view:?}"
        );
        assert!(!controller.app.quit_requested, "{view:?}");
        cell(&mut controller, 55);
        assert_eq!(
            controller.app.quit_requested, keyboard.app.quit_requested,
            "{view:?}"
        );
    }

    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut keyboard = reach_drive(View::Chop, sound.clone());
    let mut controller = reach_drive(View::Chop, sound);
    keyboard.tap(KeyCode::Char('p'), M::NONE);
    keyboard.tap(KeyCode::Tab, M::NONE);
    // In order waits for the start pad on both inputs (138): the top-left pad.
    keyboard.tap(KeyCode::Char('1'), M::NONE);
    let assign = panel_note(slot_of(View::Chop, "put on pads"));
    cell(&mut controller, 64);
    cell(&mut controller, assign); // Assign opens the target view.
    cell(&mut controller, panel_note(15)); // The menu gives the Chop actions.
    cell(&mut controller, assign); // Order replaces assign while Assign is open.
    cell(&mut controller, panel_note(15)); // Back to the target pads.
    cell(&mut controller, 48); // A first-layer hit on the top-left pad starts the order.
    assert_eq!(controller.app.plan, keyboard.app.plan);

    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut keyboard = reach_drive(View::Pattern, sound.clone());
    let mut controller = reach_drive(View::Pattern, sound);
    for drive in [&mut keyboard, &mut controller] {
        drive.app.pads.pattern = Some(session::PatternState {
            slot: "A1".into(),
            bars: 1,
            quantize: "1/16".into(),
            strength: 100,
            hits: vec![[0, 0, 100, 0]],
        });
        drive.tap(KeyCode::Char('n'), M::ALT);
    }
    for _ in 0..3 {
        keyboard.tap(KeyCode::Down, M::NONE);
    }
    keyboard.tap(KeyCode::Enter, M::NONE);
    cell(&mut controller, 64);
    cell(&mut controller, panel_note(8)); // A1 clear opens its question.
    cell(&mut controller, 67);
    assert_eq!(
        controller.app.ask.as_ref().map(Ask::lines),
        keyboard.app.ask.as_ref().map(Ask::lines)
    );
}

/// Scenario "Every current controller action has a keyboard route" (controller-reach).
#[test]
fn shipped_pad_hits_match_keyboard_results_in_each_view() {
    let keys = [
        '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v',
    ];
    let notes = [
        48, 49, 50, 51, 44, 45, 46, 47, 40, 41, 42, 43, 36, 37, 38, 39,
    ];
    for view in ALL_VIEWS {
        let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
        let mut keyboard = reach_drive(view, sound.clone());
        let mut controller = reach_drive(view, sound);
        for pad in 1..16 {
            let sample = Arc::new(Sample::mono(vec![0.2; 4_800]));
            keyboard.load_pad(pad, sample.clone(), "source");
            controller.load_pad(pad, sample, "source");
        }
        for (pad, (key, note)) in keys.into_iter().zip(notes).enumerate() {
            keyboard.key(KeyCode::Char(key), M::NONE, KeyEventKind::Press);
            keyboard.frame();
            controller.bytes(SMC, &[0x99, note, 80]);
            controller.frame();
            assert_eq!(
                controller.app.pads.held[pad],
                keyboard.app.pads.held[pad],
                "{view:?} pad {} held on press",
                pad + 1
            );
            assert!(
                controller.seen() == keyboard.seen(),
                "{view:?} pad {} press",
                pad + 1
            );

            keyboard.key(KeyCode::Char(key), M::NONE, KeyEventKind::Release);
            keyboard.frame();
            controller.bytes(SMC, &[0x89, note, 0]);
            controller.frame();
            assert_eq!(
                controller.app.pads.held[pad],
                keyboard.app.pads.held[pad],
                "{view:?} pad {} held on release",
                pad + 1
            );
            assert!(
                controller.seen() == keyboard.seen(),
                "{view:?} pad {} release",
                pad + 1
            );
        }
    }
}

/// Scenario "Every current keyboard action has a controller route" (controller-reach).
/// This test opens each screen for both inputs, so it belongs to the slow group.
#[test]
fn shipped_global_knobs_match_keyboard_results_in_each_view() {
    for view in ALL_VIEWS {
        for (cc, key, modifiers) in [
            (37, KeyCode::Char(']'), M::NONE),
            (45, KeyCode::Char(']'), M::NONE),
        ] {
            let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
            let mut keyboard = reach_drive(view, sound.clone());
            let mut controller = reach_drive(view, sound);
            if view == panel::View::Settings {
                for drive in [&mut keyboard, &mut controller] {
                    drive.app.settings_tab = 2;
                    drive.app.publish_midi(&drive.shared);
                }
            }
            let before = (
                keyboard.app.transport.bpm,
                keyboard.app.pads.bank,
                keyboard.app.master_db,
                keyboard.app.midi_row,
            );
            keyboard.tap(key, modifiers);
            turn(&mut controller, cc, 65);
            assert_eq!(
                controller.app.transport.bpm, keyboard.app.transport.bpm,
                "{view:?} cc {cc} tempo"
            );
            assert_eq!(
                controller.app.pads.bank, keyboard.app.pads.bank,
                "{view:?} cc {cc} bank"
            );
            assert_eq!(
                controller.app.master_db, keyboard.app.master_db,
                "{view:?} cc {cc} volume"
            );
            assert_eq!(
                controller.app.midi_row, keyboard.app.midi_row,
                "{view:?} cc {cc} MIDI group"
            );
            assert!(
                controller.seen() == keyboard.seen(),
                "{view:?} cc {cc} state"
            );
            assert_ne!(
                (
                    controller.app.transport.bpm,
                    controller.app.pads.bank,
                    controller.app.master_db,
                    controller.app.midi_row,
                ),
                before,
                "{view:?} cc {cc} did not act"
            );
        }
    }
}

/// Scenario "Every current controller action has a keyboard route" (controller-reach).
/// This matrix creates 48 app instances, so it belongs to the slow group.
#[test]
fn shipped_transport_buttons_match_keyboard_results_in_each_view() {
    for view in ALL_VIEWS {
        for (cc, code, modifiers) in [
            (27, KeyCode::Char('t'), M::ALT),
            (28, KeyCode::Char('y'), M::ALT),
            (29, KeyCode::Char('r'), M::CONTROL),
        ] {
            let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
            let mut keyboard = reach_drive(view, sound.clone());
            let mut controller = reach_drive(view, sound);
            let key_frame = keyboard.tap(code, modifiers);
            controller.bytes(SMC, &[0xB0, cc, 127]);
            controller.bytes(SMC, &[0xB0, cc, 0]);
            let button_frame = controller.frame();
            assert_eq!(
                reach_result(&controller, &button_frame),
                reach_result(&keyboard, &key_frame),
                "{view:?} button cc {cc}"
            );
        }
    }
}

fn list_button(drive: &mut Drive, right: bool) -> Frame {
    let cc = if right { 26 } else { 25 };
    drive.bytes(SMC, &[0xB0, cc, 127]);
    drive.bytes(SMC, &[0xB0, cc, 0]);
    drive.frame()
}

/// Scenario "SMC-PAD list buttons name and move the same list" (controller-reach).
/// Scenario "Every current controller action has a keyboard route" (controller-reach).
#[test]
fn shipped_left_and_right_buttons_match_keyboard_results_in_each_view() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut pads = reach_drive(panel::View::Pads, sound.clone());
    list_button(&mut pads, false);
    assert_eq!(pads.app.pads.bank, 0);
    pads.app.pads.gate[0] = true;
    pads.app.publish_midi(&pads.shared);
    pads.bytes(SMC, &[0x99, 48, 80]);
    pads.frame();
    assert!(pads.app.pads.held[0]);
    list_button(&mut pads, true);
    assert_eq!(pads.app.pads.bank, 1);
    pads.bytes(SMC, &[0x89, 48, 0]);
    pads.frame();
    assert!(!pads.app.pads.held[0]);

    let mut edit = reach_drive(panel::View::PadEdit, sound.clone());
    let mut keyboard = reach_drive(panel::View::PadEdit, sound.clone());
    keyboard.tap(KeyCode::Tab, M::NONE);
    list_button(&mut edit, true);
    assert_eq!(edit.app.pads.page().0, keyboard.app.pads.page().0);
    let (before, page) = (edit.app.pads.param, edit.app.pads.page().0);
    turn(&mut edit, 30, 65);
    assert_eq!(
        (edit.app.pads.param, edit.app.pads.page().0),
        (before, page)
    );
    assert_eq!(edit.app.pads.bank, 0);

    let mut pattern = reach_drive(panel::View::Pattern, sound.clone());
    pattern.app.pads.pattern = Some(session::PatternState {
        slot: "A1".into(),
        bars: 2,
        quantize: "off".into(),
        strength: 50,
        hits: vec![],
    });
    let before = pattern.app.pads.pattern.clone();
    list_button(&mut pattern, true);
    assert_eq!(pattern.app.pattern_overview(&pattern.keymap).first_cell, 16);
    turn(&mut pattern, 30, 65);
    assert_eq!(pattern.app.pattern_view.browse_cell, Some(17));
    list_button(&mut pattern, true);
    assert_eq!(pattern.app.pattern_view.browse_cell, Some(17));
    assert_eq!(pattern.app.pattern_overview(&pattern.keymap).first_cell, 16);
    list_button(&mut pattern, false);
    assert_eq!(pattern.app.pattern_overview(&pattern.keymap).first_cell, 0);
    assert_eq!(pattern.app.pads.pattern, before);

    for view in [panel::View::Chop, panel::View::ChopEdit] {
        let mut chop = reach_drive(view, sound.clone());
        let mut keyboard = reach_drive(view, sound.clone());
        for drive in [&mut chop, &mut keyboard] {
            // a slice on a pad: Chop shows the real pads, not the slice keys (132)
            drive.link_chop_pad(20, 2);
            let chop = drive.app.chop.as_mut().unwrap();
            chop.snap = chop::Snap::Cuts;
            assert!(chop.select_slice(1));
        }
        if view == panel::View::ChopEdit {
            keyboard.tap(KeyCode::Esc, M::NONE);
        }
        // LEFT and RIGHT select pad banks on Chop, as on Pads (127).
        keyboard.tap(KeyCode::PageDown, M::NONE);
        list_button(&mut chop, true);
        assert_eq!(chop.app.pads.bank, 1, "{view:?}");
        assert_eq!(chop.app.pads.bank, keyboard.app.pads.bank, "{view:?}");
        assert_eq!(chop.app.chop.as_ref().unwrap().selected, 1, "{view:?}");
        list_button(&mut chop, false);
        assert_eq!(chop.app.pads.bank, 0, "{view:?}");
    }

    let mut fx = reach_drive(panel::View::Fx, sound.clone());
    list_button(&mut fx, true);
    assert_eq!(fx.app.fx_slot, 1);
    turn(&mut fx, 30, 65);
    assert_eq!(fx.app.fx_param, 1);
    list_button(&mut fx, false);
    assert_eq!(fx.app.fx_slot, 0);
    assert_eq!(fx.app.fx_param, 0);

    let mut settings = reach_drive(panel::View::Settings, sound);
    let row = settings.app.settings_row;
    list_button(&mut settings, true);
    assert_eq!(settings.app.settings_tab, 1);
    list_button(&mut settings, false);
    assert_eq!(settings.app.settings_tab, 0);
    assert_eq!(settings.app.settings_row, row);
}

/// Scenario "File list buttons navigate folders" (controller-reach).
#[test]
fn file_list_buttons_navigate_folders() {
    let files = TransitionFiles::new();
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive(panel::View::Files, sound);
    drive.app.browser = browser::Browser::new(Some(files.0.clone()));
    assert!(drive.app.browser.entries[0].is_dir);
    list_button(&mut drive, true);
    assert_eq!(drive.app.browser.dir, files.0.join("A"));
    list_button(&mut drive, false);
    assert_eq!(drive.app.browser.dir, files.0);
    drive.app.browser.selected = drive
        .app
        .browser
        .entries
        .iter()
        .position(|file| file.name == "tone.wav")
        .unwrap();
    let selected = drive.app.browser.selected;
    assert!(list_button(&mut drive, true).commands().is_empty());
    assert_eq!(drive.app.browser.selected, selected);
    assert_eq!(drive.app.browser.dir, files.0);
}

/// Scenario "Both banks pin master to the top right knob" (controller-reach).
#[test]
fn both_banks_pin_master_to_the_top_right_knob() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    for view in ALL_VIEWS {
        let mut drive = reach_drive(view, sound.clone());
        let bank = drive.app.pads.bank;
        let tempo = drive.app.transport.bpm;
        let db = drive.app.master_db;
        turn(&mut drive, 37, 63);
        assert_eq!(drive.app.master_db, db - 1, "{view:?}");
        turn(&mut drive, 45, 63);
        assert_eq!(drive.app.master_db, db - 2, "{view:?}");
        assert_eq!(drive.app.pads.bank, bank, "{view:?}");
        assert_eq!(drive.app.transport.bpm, tempo, "{view:?}");
    }
}

/// Scenario "Every current controller action has a keyboard route" (controller-reach).
/// This matrix creates 54 app instances, so it belongs to the slow group.
#[test]
fn shipped_pad_knobs_match_keyboard_values_in_pad_views() {
    for view in [panel::View::PadEdit] {
        for (cc, param, key, value) in [
            (36, pads::Param::Volume, KeyCode::Left, 63),
            (34, pads::Param::Pitch, KeyCode::Right, 65),
            (34, pads::Param::Cutoff, KeyCode::Left, 63),
            (34, pads::Param::Decay, KeyCode::Right, 65),
            (32, pads::Param::Release, KeyCode::Right, 65),
        ] {
            let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
            let mut keyboard = reach_drive(view, sound.clone());
            let mut controller = reach_drive(view, sound);
            for drive in [&mut keyboard, &mut controller] {
                drive.app.pads.param = param;
                drive.app.publish_midi(&drive.shared);
            }
            let before = keyboard.seen();
            if view == panel::View::Pattern {
                keyboard.tap(KeyCode::F(2), M::NONE);
            }
            keyboard.tap(key, M::NONE);
            if view == panel::View::Pattern {
                keyboard.tap(KeyCode::F(5), M::NONE);
            }
            turn(&mut controller, cc, value);
            let mut controller_seen = controller.seen();
            let mut keyboard_seen = keyboard.seen();
            let popup = controller.app.value_popup.as_ref().unwrap();
            assert!(popup.current(&controller.app).is_some());
            controller_seen.notice = None;
            keyboard_seen.notice = None;
            assert!(controller_seen == keyboard_seen, "{view:?} cc {cc}");
            assert_eq!(
                controller.app.pattern_view.overview,
                keyboard.app.pattern_view.overview
            );
            assert_ne!(
                controller.seen().pads,
                before.pads,
                "{view:?} cc {cc} did not act"
            );
        }
    }
}

/// Scenario "Every current controller action has a keyboard route" (controller-reach).
#[test]
fn shipped_files_knob_matches_keyboard_selection() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut keyboard = reach_drive(panel::View::Files, sound.clone());
    let mut controller = reach_drive(panel::View::Files, sound);
    for drive in [&mut keyboard, &mut controller] {
        drive.app.browser.entries = (0..3)
            .map(|index| browser::Entry {
                name: format!("file-{index}.wav"),
                is_dir: false,
                favorite: None,
            })
            .collect();
        drive.app.browser.selected = 1;
    }
    keyboard.tap(KeyCode::Down, M::NONE);
    turn(&mut controller, 30, 65);
    assert_eq!(controller.app.browser.selected, 2);
    assert_eq!(
        controller.app.browser.selected,
        keyboard.app.browser.selected
    );
}

fn pad_value(drive: &mut Drive, param: pads::Param) -> String {
    let abs = drive.app.pads.selected_abs();
    let region = drive.app.pads.region[abs];
    match param {
        pads::Param::Start => return region.start.to_string(),
        // "To the end" and the explicit end of the 4 800-frame test sample are the same end.
        pads::Param::End => return region.end.min(4_800).to_string(),
        _ => {}
    }
    let current = drive.app.pads.param;
    drive.app.pads.param = param;
    let value = drive.app.pads.param_value();
    drive.app.pads.param = current;
    value
}

/// Scenario "Pad knobs act only in pad edit" (controller-reach).
#[test]
fn pad_knobs_act_only_in_pad_edit() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    for param in [
        pads::Param::Volume,
        pads::Param::Attack,
        pads::Param::Cutoff,
        pads::Param::Start,
        pads::Param::Send1,
    ] {
        let mut drive = reach_drive(panel::View::Pads, sound.clone());
        drive.app.pads.param = param;
        let abs = drive.app.pads.selected_abs();
        let pad = drive.app.pads.snapshot(abs);
        for cc in 31..=36 {
            for value in [65, 63] {
                turn(&mut drive, cc, value);
            }
        }
        assert!(drive.app.pads.snapshot(abs) == pad, "{param:?}");
        assert_eq!(drive.app.pads.param, param);

        let selected = drive.app.pads.selected;
        drive.bytes(SMC, &[0xB0, 30, 65]);
        let frame = drive.frame();
        assert_eq!(drive.app.pads.selected, selected);
        assert!(!frame.commands().iter().any(|command| matches!(
            command,
            Command::PadPress { .. } | Command::PadRelease { .. }
        )));
        assert!(drive.app.pads.snapshot(abs) == pad, "{param:?}");
    }
}

/// Scenario "Pad edit knobs follow the screen rows" (controller-reach).
#[test]
fn pad_edit_knobs_follow_the_screen_rows() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive(panel::View::PadEdit, sound);
    drive.app.pads.param = pads::Param::Volume;
    let abs = drive.app.pads.selected_abs();
    let mut names = Vec::new();
    for _ in 0..5 {
        let (name, params) = drive.app.pads.page();
        names.push(name);
        for (index, cc) in [36, 34, 32].into_iter().enumerate() {
            let param = params[index];
            let others = |drive: &mut Drive| {
                params
                    .iter()
                    .filter(|other| **other != param)
                    .map(|other| pad_value(drive, *other))
                    .collect::<Vec<_>>()
            };
            let (before, rest) = (pad_value(&mut drive, param), others(&mut drive));
            // The end starts at the sample end, so it can only move back.
            turn(
                &mut drive,
                cc,
                if param == pads::Param::End { 63 } else { 65 },
            );
            // One loaded sample gives the source knob no other choice.
            if param != pads::Param::Source {
                assert_ne!(pad_value(&mut drive, param), before, "{name} knob {cc}");
            }
            assert_eq!(others(&mut drive), rest, "{name} knob {cc}");
        }
        let pad = drive.app.pads.snapshot(abs);
        for cc in [31, 33, 35] {
            for value in [65, 63] {
                turn(&mut drive, cc, value);
            }
        }
        // Knob 1 is the fourth value; the current pages have three.
        turn(&mut drive, 30, 65);
        assert!(drive.app.pads.snapshot(abs) == pad, "{name}");
        list_button(&mut drive, true);
    }
    assert_eq!(names, ["TONE", "ENV", "FILTER", "SOURCE", "FX"]);
}

#[test]
fn fx_knobs_change_the_current_slot() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut fx = reach_drive(panel::View::Fx, sound);
    let first = fx.app.pads.fx[0];
    let second = fx.app.pads.fx[1];
    list_button(&mut fx, true);
    turn(&mut fx, 31, 65);
    assert_eq!(fx.app.pads.fx[0], first);
    assert_ne!(fx.app.pads.fx[1].kind, second.kind);
}

/// Pad edit pages move with LEFT and RIGHT while the panel is hidden.
#[test]
fn pad_edit_pages_move_with_the_hidden_panel() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut shown = reach_drive(panel::View::PadEdit, sound.clone());
    cell(&mut shown, 64);
    let snapshot = shown.app.panel_snapshot().unwrap();
    // The page cells are on device pads 9 to 13 (138).
    assert_eq!(snapshot.labels[4].as_deref(), Some("tone page"));
    assert_eq!(snapshot.labels[0].as_deref(), Some("fx page"));
    // The pages move with LEFT and RIGHT while the panel is hidden.
    let mut drive = reach_drive(panel::View::PadEdit, sound);
    let pad = drive.app.pads.selected_abs();
    assert_eq!(drive.app.pads.page().0, "TONE");
    list_button(&mut drive, true);
    assert_eq!(drive.app.pads.page().0, "ENV");
    list_button(&mut drive, true);
    assert_eq!(drive.app.pads.page().0, "FILTER");
    list_button(&mut drive, false);
    assert_eq!(drive.app.pads.page().0, "ENV");
    turn(&mut drive, 36, 65);
    assert_eq!(drive.app.pads.attack_ms[pad], 1);
    for name in ["edit.param_next", "edit.page_next", "edit.value_inc"] {
        assert!(input::ACTIONS.iter().any(|action| action.name == name));
    }
}

/// Scenario "The fx page cell opens FX in one press" (pad-playback).
#[test]
fn the_fx_page_cell_opens_fx_in_one_press() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive(panel::View::PadEdit, sound);
    assert_eq!(drive.app.pads.page().0, "TONE");
    cell(&mut drive, 64);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::PadEdit, "fx page")),
    );
    assert_eq!(drive.app.pads.page().0, "FX");
    assert!(drive.app.edit, "the edit stays open");
    let current = drive.app.panel_snapshot().unwrap().current;
    let fx = usize::from(slot_of(panel::View::PadEdit, "fx page") - 1);
    assert!(current[fx], "the fx page cell is brighter");
    assert_eq!(
        current.iter().filter(|&&on| on).count(),
        1,
        "one current page"
    );
    for (label, page) in [
        ("tone page", "TONE"),
        ("env page", "ENV"),
        ("filter page", "FILTER"),
        ("source page", "SOURCE"),
    ] {
        cell(&mut drive, panel_note(slot_of(panel::View::PadEdit, label)));
        assert_eq!(drive.app.pads.page().0, page);
    }
}

/// Scenario "LEFT on TONE opens FX" (pad-playback).
#[test]
fn left_on_tone_opens_fx() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive(panel::View::PadEdit, sound);
    assert!(!drive.app.panel.visible());
    assert_eq!(drive.app.pads.page().0, "TONE");
    list_button(&mut drive, false);
    assert_eq!(drive.app.pads.page().0, "FX");
    list_button(&mut drive, true);
    assert_eq!(drive.app.pads.page().0, "TONE");
}

fn check_chop_context_encoders(view: panel::View) {
    for (cc, value, code, modifiers) in [
        (30, 63, KeyCode::Left, M::NONE),
        (30, 65, KeyCode::Right, M::NONE),
        (32, 63, KeyCode::Left, M::CONTROL),
        (32, 65, KeyCode::Right, M::CONTROL),
        (33, 63, KeyCode::Left, M::ALT),
        (33, 65, KeyCode::Right, M::ALT),
    ] {
        let sound = Arc::new(Sample::mono(vec![0.2; 96_000]));
        let mut keyboard = reach_drive(view, sound.clone());
        let mut controller = reach_drive(view, sound);
        for drive in [&mut keyboard, &mut controller] {
            let chop = drive.app.chop.as_mut().unwrap();
            assert!(chop.select_slice(1));
            chop.zoom(true);
        }
        let result = |drive: &Drive| {
            let chop = drive.app.chop.as_ref().unwrap();
            (
                chop.saved(),
                chop.view,
                chop.cursor,
                chop.selected,
                drive.app.slice_end,
            )
        };
        let before = result(&keyboard);
        if view == panel::View::ChopEdit && cc == 30 {
            keyboard.tap(KeyCode::Esc, M::NONE);
        }
        keyboard.tap(code, modifiers);
        if view == panel::View::ChopEdit && cc == 30 {
            keyboard.tap(KeyCode::Enter, M::NONE);
        }
        turn(&mut controller, cc, value);
        assert_ne!(result(&keyboard), before, "{view:?} cc {cc} {value}");
        assert_eq!(
            result(&controller),
            result(&keyboard),
            "{view:?} cc {cc} {value}"
        );
    }
}

/// Scenario "Every current controller action has a keyboard route" (controller-reach).
#[test]
fn shipped_chop_encoders_match_keyboard_outside_edit() {
    check_chop_context_encoders(panel::View::Chop);
}

/// Scenario "Every current controller action has a keyboard route" (controller-reach).
#[test]
fn shipped_chop_encoders_match_keyboard_in_edit() {
    check_chop_context_encoders(panel::View::ChopEdit);
}

/// Scenario "Chop knobs move two separate bound pairs" (controller-reach).
#[test]
fn chop_knobs_move_two_separate_bound_pairs() {
    let sound = Arc::new(Sample::mono(vec![0.2; 96_000]));
    for view in [panel::View::Chop, panel::View::ChopEdit] {
        for (cc, value, linked_slice) in [(32, 65, 1), (33, 63, 1), (34, 65, 0), (35, 63, 3)] {
            let mut drive = reach_drive(view, sound.clone());
            assert!(drive.app.chop.as_mut().unwrap().select_slice(1));
            drive.link_chop_pad(5, linked_slice);
            let before = drive.app.chop.as_ref().unwrap();
            let saved = before.saved();
            let slice = before.slices()[1];
            let region = before.range;
            let pad_region = drive.app.pads.region[5];
            let pad_link = drive.app.pads.link[5];
            let step = before.view_step(1);

            turn(&mut drive, cc, value);
            let chop = drive.app.chop.as_ref().unwrap();
            let actual_slice = chop.slices()[1];
            let actual_region = chop.range;
            let expected = match cc {
                32 => (
                    Region {
                        start: slice.start + step,
                        ..slice
                    },
                    region,
                ),
                33 => (
                    Region {
                        end: slice.end - step,
                        ..slice
                    },
                    region,
                ),
                34 => (
                    slice,
                    Region {
                        start: region.start + step,
                        ..region
                    },
                ),
                _ => (
                    slice,
                    Region {
                        end: region.end - step,
                        ..region
                    },
                ),
            };
            assert_eq!((actual_slice, actual_region), expected, "{view:?} CC {cc}");
            if cc < 34 {
                assert_eq!(drive.app.pads.region[5], chop.slices()[linked_slice]);
                assert_ne!(drive.app.pads.region[5], pad_region);
                assert_eq!(drive.app.pads.link[5], pad_link);
            } else {
                assert_eq!(drive.app.pads.region[5], pad_region);
                assert_eq!(drive.app.pads.link[5], None);
            }
            drive.tap(KeyCode::Char('z'), M::CONTROL);
            assert_eq!(drive.app.chop.as_ref().unwrap().saved(), saved);
            assert_eq!(drive.app.pads.region[5], pad_region);
            assert_eq!(drive.app.pads.link[5], pad_link);
        }
    }
}

/// Scenario "Chop bounds wait during Assign and questions" (controller-reach).
#[test]
fn chop_bounds_wait_during_assign_and_questions() {
    for assign in [true, false] {
        let sound = Arc::new(Sample::mono(vec![0.2; 96_000]));
        let mut drive = reach_drive(panel::View::Chop, sound);
        assert!(drive.app.chop.as_mut().unwrap().select_slice(1));
        if assign {
            drive.tap(KeyCode::Char('p'), M::NONE);
            assert!(drive.app.plan.is_some());
        } else {
            drive.app.ask = Some(Ask::Exit { lines: vec![] });
            drive.app.publish_midi(&drive.shared);
        }
        let before = drive.app.chop.as_ref().unwrap().saved();
        let plan = drive.app.plan.clone();
        for (cc, value) in [(32, 65), (33, 63), (34, 65), (35, 63)] {
            turn(&mut drive, cc, value);
            assert_eq!(drive.app.chop.as_ref().unwrap().saved(), before);
            assert_eq!(drive.app.plan, plan);
            assert_eq!(drive.app.ask.is_some(), !assign);
        }
    }
}

/// Scenario "A Chop bound turn keeps the current panel page" (controller-reach).
#[test]
fn a_chop_bound_turn_keeps_the_current_panel_page() {
    let sound = Arc::new(Sample::mono(vec![0.2; 96_000]));
    let mut drive = reach_drive(panel::View::Chop, sound);
    cell(&mut drive, 64);
    assert_eq!(drive.app.panel.page(), 1);
    turn(&mut drive, 35, 63);
    assert_eq!(drive.app.panel.page(), 1);
    assert!(drive.app.chop.as_ref().unwrap().range.end < 96_000);
    list_button(&mut drive, false);
    turn(&mut drive, 35, 63);
    assert_eq!(drive.app.panel.page(), 0);
}

/// Scenario "Bank 2 turns change no pad value" (controller-reach).
#[test]
fn bank_2_turns_change_no_pad_value() {
    let sound = Arc::new(Sample::mono(vec![0.2; 48_000]));
    for view in ALL_VIEWS {
        let mut drive = reach_drive(view, sound.clone());
        drive.app.pads.selected = 0;
        let pad = drive.app.pads.snapshot(0);
        let (page, bank, selected) = (
            drive.app.pads.page().0,
            drive.app.pads.bank,
            drive.app.pads.selected,
        );
        for cc in 38..=44 {
            for value in [63, 65] {
                turn(&mut drive, cc, value);
            }
        }
        assert!(drive.app.pads.snapshot(0) == pad, "{view:?}");
        assert_eq!(
            (
                drive.app.pads.page().0,
                drive.app.pads.bank,
                drive.app.pads.selected
            ),
            (page, bank, selected),
            "{view:?}"
        );
        let db = drive.app.master_db;
        turn(&mut drive, 45, 63);
        assert_eq!(drive.app.master_db, db - 1, "{view:?}");
    }
}

fn timed_knob_two(drive: &mut Drive, value: u8, at: Instant) {
    drive
        .midi()
        .recorded_bytes_at(SMC, &[0xB0, 31, value], &drive.project, at);
    drive.frame();
}

fn chop_view_len(drive: &Drive) -> usize {
    let view = drive.app.chop.as_ref().unwrap().view;
    view.end - view.start
}

/// Scenario "Slow Chop zoom uses smaller input steps" (controller-reach).
#[test]
fn slow_chop_zoom_uses_smaller_input_steps() {
    let sound = Arc::new(Sample::mono(vec![0.2; 96_000]));
    for view in [panel::View::Chop, panel::View::ChopEdit] {
        for (key, value, exponent_sign) in [(KeyCode::Up, 65, -1.0), (KeyCode::Down, 63, 1.0)] {
            let mut keyboard = reach_drive(view, sound.clone());
            let mut controller = reach_drive(view, sound.clone());
            for drive in [&mut keyboard, &mut controller] {
                let chop = drive.app.chop.as_mut().unwrap();
                chop.set_cursor(32_000);
                chop.zoom(true);
            }
            let initial_len = chop_view_len(&keyboard);
            if view == panel::View::ChopEdit {
                keyboard.tap(KeyCode::Esc, M::NONE);
            }
            keyboard.tap(key, M::NONE);
            let at = Instant::now();
            let mut previous_len = initial_len;
            for step in 1..=4 {
                timed_knob_two(
                    &mut controller,
                    value,
                    at + std::time::Duration::from_millis(100 * step),
                );
                let current_len = chop_view_len(&controller);
                let factor = 2.0_f64.powf(exponent_sign * step as f64 / 4.0);
                let expected = (initial_len as f64 * factor).round() as usize;
                assert!(
                    current_len.abs_diff(expected) <= 3,
                    "{view:?} {key:?} {step}"
                );
                assert_ne!(current_len, previous_len, "each turn changes the view");
                assert_eq!(controller.app.chop.as_ref().unwrap().cursor, 32_000);
                previous_len = current_len;
            }
            let keyboard_chop = keyboard.app.chop.as_ref().unwrap();
            let controller_chop = controller.app.chop.as_ref().unwrap();
            assert!(chop_view_len(&controller).abs_diff(chop_view_len(&keyboard)) <= 3);
            assert_eq!(controller_chop.saved(), keyboard_chop.saved());
            assert_eq!(controller_chop.selected, keyboard_chop.selected);
        }
    }
}

#[test]
fn smc_chop_zoom_uses_single_steps_and_reverses_at_view_limits() {
    let sound = Arc::new(Sample::mono(vec![0.2; 96_000]));
    let mut drive = reach_drive(panel::View::Chop, sound);
    let at = Instant::now();
    timed_knob_two(&mut drive, 63, at);
    assert_eq!(chop_view_len(&drive), 96_000);

    timed_knob_two(&mut drive, 65, at + std::time::Duration::from_millis(100));
    let after_slow_turn = chop_view_len(&drive);
    timed_knob_two(&mut drive, 65, at + std::time::Duration::from_millis(110));
    let after_fast_turn = chop_view_len(&drive);
    let expected = (after_slow_turn as f64 / 2.0_f64.powf(0.25)).round() as usize;
    assert_eq!(after_fast_turn, expected);
    assert!(after_fast_turn < after_slow_turn);

    drive.app.chop.as_mut().unwrap().set_zoom(1.0);
    let closest = chop_view_len(&drive);
    timed_knob_two(&mut drive, 65, at + std::time::Duration::from_millis(200));
    assert_eq!(chop_view_len(&drive), closest);
    timed_knob_two(&mut drive, 63, at + std::time::Duration::from_millis(300));
    assert!(chop_view_len(&drive) > closest);
}

/// Scenario "SMC-PAD Chop zoom preserves Assign and question priority" (controller-reach).
#[test]
fn smc_chop_zoom_preserves_assign_and_question_priority() {
    let sound = Arc::new(Sample::mono(vec![0.2; 96_000]));
    let mut drive = reach_drive(panel::View::Chop, sound);
    let initial_len = chop_view_len(&drive);
    drive.tap(KeyCode::Char('p'), M::NONE);
    assert!(drive.app.plan.is_some());
    turn(&mut drive, 31, 65);
    assert_eq!(chop_view_len(&drive), initial_len);

    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.plan.is_none());
    drive.app.ask = Some(Ask::Exit { lines: vec![] });
    drive.app.publish_midi(&drive.shared);
    turn(&mut drive, 31, 65);
    assert_eq!(chop_view_len(&drive), initial_len);
    assert!(drive.app.ask.is_some());
}

/// Scenario "Every current controller action has a keyboard route" (controller-reach).
#[test]
fn absolute_chop_bound_knob_selects_the_bound_it_moves() {
    let sound = Arc::new(Sample::mono(vec![0.2; 96_000]));
    let mut drive = reach_drive(panel::View::Chop, sound);
    assert!(drive.app.chop.as_mut().unwrap().select_slice(1));
    drive.bytes(XS_PORT, &[0xB0, 25, 90]);
    drive.frame();
    assert!(drive.app.slice_end);
    drive.bytes(XS_PORT, &[0xB0, 24, 40]);
    drive.frame();
    assert!(!drive.app.slice_end);
}

fn check_a1_encoders_on_screen(screen: Screen) {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut keyboard = reach_drive(panel::View::Pads, sound.clone());
    let mut controller = reach_drive(panel::View::Pads, sound);
    for drive in [&mut keyboard, &mut controller] {
        drive.app.goto(screen);
        drive.app.publish_midi(&drive.shared);
        drive.tap(KeyCode::Char('n'), M::ALT);
        assert_eq!(drive.app.pattern_panel, Some(0));
        assert!(drive.app.focus().mode.pattern_controls);
    }
    let settings = |drive: &Drive| {
        let pattern = drive.app.pads.pattern.as_ref().unwrap();
        (pattern.bars, pattern.quantize.clone(), pattern.strength)
    };
    for (row, cc, key, value) in [
        (0, 34, KeyCode::Right, 65),
        (1, 32, KeyCode::Right, 65),
        (2, 33, KeyCode::Left, 63),
    ] {
        for drive in [&mut keyboard, &mut controller] {
            while drive.app.pattern_panel != Some(row) {
                drive.tap(KeyCode::Char('k'), M::NONE);
            }
        }
        let before = settings(&keyboard);
        keyboard.tap(key, M::NONE);
        turn(&mut controller, cc, value);
        assert_eq!(
            settings(&controller),
            settings(&keyboard),
            "{screen:?} row {row}"
        );
        assert_ne!(
            settings(&controller),
            before,
            "{screen:?} row {row} did not act"
        );
    }
    for drive in [&mut keyboard, &mut controller] {
        drive.tap(KeyCode::Char('n'), M::ALT);
        assert!(drive.app.pattern_panel.is_none());
        assert!(!drive.app.focus().mode.pattern_controls);
    }
}

/// Scenario "A1 encoders follow the open controls on every screen" (controller-reach).
#[test]
fn a1_encoders_follow_the_open_controls_on_pads_files_and_chop() {
    for screen in [Screen::Pads, Screen::Files, Screen::Chop] {
        check_a1_encoders_on_screen(screen);
    }
}

/// Scenario "A1 encoders follow the open controls on every screen" (controller-reach).
#[test]
fn a1_encoders_follow_the_open_controls_on_fx_and_settings() {
    for screen in [Screen::Fx, Screen::Settings] {
        check_a1_encoders_on_screen(screen);
    }
}

/// Scenario "A1 encoders follow the open controls on every screen" (controller-reach).
#[test]
fn a1_position_and_master_keep_their_routes_and_close_restores_pads() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut keyboard = reach_drive(panel::View::Pads, sound.clone());
    let mut controller = reach_drive(panel::View::Pads, sound);
    for drive in [&mut keyboard, &mut controller] {
        drive.tap(KeyCode::Char('n'), M::ALT);
    }
    keyboard.tap(KeyCode::Down, M::NONE);
    turn(&mut controller, 30, 65);
    assert_eq!(controller.app.pattern_panel, keyboard.app.pattern_panel);
    assert_eq!(controller.app.pads.bank, 0);
    keyboard.tap(KeyCode::Char(']'), M::NONE);
    turn(&mut controller, 37, 65);
    assert_eq!(controller.app.master_db, keyboard.app.master_db);
    cell(&mut controller, 64);
    assert!(controller.app.panel.visible());
    assert!(controller.app.pattern_panel.is_some());
    for drive in [&mut keyboard, &mut controller] {
        drive.tap(KeyCode::Char('n'), M::ALT);
        drive.app.pads.bank = 0;
        drive.app.pads.param = pads::Param::Volume;
        drive.app.publish_midi(&drive.shared);
    }
    // Closed A1 controls give knob 1 back to Pads, where it has no role.
    let param = controller.app.pattern_panel;
    turn(&mut controller, 30, 65);
    assert_eq!(controller.app.pattern_panel, param);
    assert_eq!(controller.app.pads.selected, 0);
    assert_eq!(controller.seen().pads, keyboard.seen().pads);
}

/// Scenario "A1 encoder context preserves input owners and held pads" (controller-reach).
#[test]
fn a1_encoder_context_preserves_input_owners_and_held_pads() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive(panel::View::Pads, sound);
    drive.app.pads.gate[0] = true;
    drive.bytes(SMC, &[0x99, 48, 80]);
    drive.frame();
    assert!(drive.app.pads.held[0]);
    drive.tap(KeyCode::Char('n'), M::ALT);
    let bars = drive.app.pads.pattern.as_ref().unwrap().bars;

    drive.app.ask = Some(Ask::ClearPads {
        pads: vec![1],
        lines: vec!["Clear pad 2?".into()],
    });
    drive.app.publish_midi(&drive.shared);
    turn(&mut drive, 34, 65);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, bars);
    drive.app.ask = None;

    drive.app.project_picker = Some(ProjectPicker {
        entries: Vec::new(),
        selected: 0,
        naming: None,
    });
    drive.app.publish_midi(&drive.shared);
    turn(&mut drive, 34, 65);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, bars);
    drive.app.project_picker = None;

    drive.app.pad_naming = Some((0, "name".into()));
    drive.app.publish_midi(&drive.shared);
    turn(&mut drive, 34, 65);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, bars);
    drive.app.pad_naming = None;

    drive.app.screen_learn = Some(drive.app.learn_open());
    drive.app.publish_midi(&drive.shared);
    turn(&mut drive, 34, 65);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, bars);
    drive.app.screen_learn = None;

    drive.tap(KeyCode::Char('n'), M::ALT);
    drive.bytes(SMC, &[0x89, 48, 0]);
    drive.frame();
    assert!(!drive.app.pads.held[0]);

    drive.tap(KeyCode::Char('n'), M::ALT);
    drive.bytes(PAD_PORT, &[0x90, 36, 80]);
    drive.frame();
    assert!(drive.app.pads.held[0]);
    drive.bytes(PAD_PORT, &[0x80, 36, 0]);
    drive.frame();
    assert!(!drive.app.pads.held[0]);
}

#[test]
fn master_knob_does_not_act_behind_a_question_or_picker() {
    let files = TransitionFiles::new();
    let mut question = project_drive(&files);
    question.app.ask = Some(Ask::ClearPads {
        pads: vec![0],
        lines: vec!["Clear?".into()],
    });
    let bank = question.app.pads.bank;
    turn(&mut question, 37, 65);
    assert_eq!(question.app.pads.bank, bank);
    assert!(question.app.ask.is_some());

    let mut project = project_drive(&files);
    project.tap(KeyCode::Char('p'), M::CONTROL);
    let selected = project.app.project_picker.as_ref().unwrap().selected;
    turn(&mut project, 37, 65);
    assert_eq!(project.app.pads.bank, 0);
    assert_eq!(
        project.app.project_picker.as_ref().unwrap().selected,
        selected
    );

    let mut folder = folder_drive(&files);
    let selected = folder.app.path_picker.as_ref().unwrap().selected;
    turn(&mut folder, 37, 65);
    assert_eq!(folder.app.midi_row, 0);
    assert_eq!(folder.app.path_picker.as_ref().unwrap().selected, selected);
}

fn pad_bank_request(drive: &mut Drive, port: &str) -> Frame {
    drive.midi().recorded_pad_bank(port, Instant::now(), true);
    drive.frame()
}

fn pad_bank_row(drive: &Drive) -> Option<usize> {
    drive
        .app
        .midi_list()
        .iter()
        .position(|row| matches!(row, MidiListRow::SmcPadBank))
}

/// Scenario "PAD BANK starts on hold and preserves a saved Off choice" (controller-reach).
/// Scenario "PAD BANK setting saves and defaults to off" (controller-reach).
#[test]
fn pad_bank_starts_on_hold_and_preserves_a_saved_off_choice() {
    let files = TransitionFiles::new();
    let path = files.0.join("pad-bank-config.toml");
    std::fs::write(&path, "[midi_gestures]\nhold_time_ms = 400\n").unwrap();
    assert!(
        config::load_from(&path)
            .config
            .midi_gestures
            .smc_pad_bank_opens_panel
    );
    assert!(pad_bank_row(&Drive::offline()).is_none());

    let mut unverified = Drive::smc_offline();
    assert!(pad_bank_row(&unverified).is_none());
    pad_bank_request(&mut unverified, SMC);
    assert!(!unverified.app.panel.visible());

    let mut keyboard = saved_layout_drive(&path);
    let mut controller = hold_drive();
    assert!(controller.app.midi_gestures.smc_pad_bank_opens_panel);
    for drive in [&mut keyboard, &mut controller] {
        drive.app.screen = Screen::Settings;
        drive.app.settings_tab = 2;
        drive.app.midi_row = pad_bank_row(drive).unwrap();
        drive.app.publish_midi(&drive.shared);
    }
    keyboard.tap(KeyCode::Left, M::NONE);
    turn(&mut controller, 31, 63);
    assert!(!keyboard.app.midi_gestures.smc_pad_bank_opens_panel);
    assert!(!controller.app.midi_gestures.smc_pad_bank_opens_panel);
    let restored = config::load_from(&path).config.midi_gestures;
    assert!(!restored.smc_pad_bank_opens_panel);
    assert_eq!(restored.hold_time_ms, 400);

    let mut restart = saved_layout_drive(&path);
    pad_bank_request(&mut restart, SMC);
    assert!(!restart.app.panel.visible());
}

/// Scenario "The panel shows only while PAD BANK is held" (controller-reach).
#[test]
fn the_panel_shows_only_while_pad_bank_is_held() {
    let mut drive = hold_drive();
    let shown = hold(&mut drive, true);
    assert!(shown.commands().is_empty());
    assert_eq!(drive.app.panel.owner(), Some(SMC));
    assert_eq!(drive.app.panel.page(), 1);
    list_button(&mut drive, false);
    assert_eq!(drive.app.panel.page(), 0);
    cell(&mut drive, panel_note(2));
    assert_eq!(drive.app.screen, Screen::Files);
    assert_eq!(drive.app.panel.page(), 1);
    const SECOND: &str = "SINCO SMC-PAD-Master 32:1";
    drive.midi().recorded_connect(SECOND);
    drive.frame();
    assert!(pad_bank_request(&mut drive, SECOND).commands().is_empty());
    assert_eq!(drive.app.panel.owner(), Some(SMC));
    let hidden = hold(&mut drive, false);
    assert!(hidden.commands().is_empty());
    assert!(!drive.app.panel.visible());
    assert_eq!(drive.app.screen, Screen::Files);
    // The next hold starts on the actions again.
    hold(&mut drive, true);
    assert_eq!(drive.app.panel.page(), 1);
}

/// Every shipped action cell gives the same result through a PAD BANK hold (131)
/// as through the second-bank route. The matrix drives 53 cells, so it is slow.
#[test]
fn shipped_panel_cells_act_the_same_during_a_hold() {
    let config = panel::Config::parse(
        "config/panel.toml",
        include_str!("../../../../config/panel.toml"),
    );
    let mut checked = 0;
    for view in ALL_VIEWS {
        for slot in 1..=12 {
            if config.cell(view, slot).is_none() {
                continue;
            }
            let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
            let mut layer = reach_drive(view, sound.clone());
            cell(&mut layer, 64);
            let layer_frame = cell(&mut layer, panel_note(slot));
            let mut held = reach_drive_from(hold_drive(), view, sound);
            hold(&mut held, true);
            // The first-layer note of the same physical pad.
            let held_frame = cell(&mut held, panel_note(slot) - 16);
            assert_eq!(
                reach_result(&held, &held_frame),
                reach_result(&layer, &layer_frame),
                "{view:?} cell {slot}"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 53);
}

/// Scenario "A hold takes LEFT and RIGHT over saved roles" (controller-reach).
#[test]
fn a_hold_takes_left_and_right_over_saved_roles() {
    let files = TransitionFiles::new();
    let path = files.0.join("owner-chop-roles.toml");
    std::fs::write(
        &path,
        "[devices]\nseeded = [\"SMC-PAD\"]\n\n[midi.\"SMC-PAD\".chop]\n\"cc 25\" = \"marker.prev\"\n\"cc 26\" = \"marker.next\"\n",
    )
    .unwrap();
    let mut drive = saved_layout_drive(&path);
    drive.prepare_chop(Arc::new(Sample::mono(vec![0.2; 48_000])), 4);
    drive.app.publish_midi(&drive.shared);
    let selected = drive.app.chop.as_ref().unwrap().selected;
    hold(&mut drive, true);
    list_button(&mut drive, false);
    assert_eq!(drive.app.panel.page(), 0);
    list_button(&mut drive, true);
    assert_eq!(drive.app.panel.page(), 1);
    assert_eq!(drive.app.chop.as_ref().unwrap().selected, selected);
    hold(&mut drive, false);
    list_button(&mut drive, true);
    assert_ne!(drive.app.chop.as_ref().unwrap().selected, selected);
}

/// Scenario "PAD BANK respects input overlays" (controller-reach).
#[test]
fn pad_bank_respects_input_overlays() {
    let mut drive = hold_drive();
    // Each overlay keeps priority: the panel shows only its own cells (134).
    let overlay = |drive: &Drive| {
        let panel = drive.app.panel_snapshot().unwrap();
        assert!(panel.target, "{}", panel.view);
        assert!(panel.labels[12].is_some(), "{}", panel.view);
    };
    drive.app.help_open = true;
    hold(&mut drive, true);
    overlay(&drive);
    drive.app.help_open = false;
    // The panel shows again while the hold continues.
    assert!(drive.app.panel_snapshot().is_some());
    hold(&mut drive, false);

    drive.app.pad_naming = Some((0, "Name".into()));
    hold(&mut drive, true);
    overlay(&drive);
    hold(&mut drive, false);
    drive.app.pad_naming = None;

    drive.app.project_picker = Some(ProjectPicker {
        entries: vec!["A".into()],
        selected: 0,
        naming: None,
    });
    hold(&mut drive, true);
    overlay(&drive);
    hold(&mut drive, false);
    assert_eq!(drive.app.project_picker.take().unwrap().selected, 0);

    drive.app.learning = Some(learn::Learning::new(action_info("transport.toggle")));
    hold(&mut drive, true);
    overlay(&drive);
    assert!(drive.app.learning.as_ref().unwrap().pending.is_none());
}

/// Scenario "A pad hit follows the hold, not the device layer" (controller-reach).
#[test]
fn a_pad_hit_follows_the_hold_not_the_device_layer() {
    let mut drive = hold_drive();
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 256])), "top left");
    // Top-left pad on the device's second layer, no hold: it plays.
    cell(&mut drive, 64);
    assert!(drive.app.pads.hit_at[0].is_some());
    assert!(!drive.app.panel.visible());
    assert!(!drive.app.edit);
    // A first-layer pad during a hold runs its cell: edit pad (138).
    drive.app.pads.hit_at[0] = None;
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pads, "edit pad")) - 16,
    );
    assert!(drive.app.edit);
    assert!(drive.app.pads.hit_at[0].is_none());
}

/// Scenario "A pad release keeps the target of its press" (controller-reach).
#[test]
fn a_pad_release_keeps_the_target_of_its_press() {
    let mut drive = hold_drive();
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 256])), "gate");
    hold(&mut drive, true);
    drive.bytes(SMC, &[0x99, 48, 100]);
    drive.frame();
    hold(&mut drive, false);
    assert!(!drive.app.panel.visible());
    drive.bytes(SMC, &[0x89, 48, 0]);
    let frame = drive.frame();
    assert!(frame.commands().is_empty());
    assert!(drive.app.pads.held.iter().all(|held| !held));
    assert!(drive.app.pads.hit_at.iter().all(Option::is_none));
}

/// Scenario "A hit before the hold reply plays the pad" (controller-reach).
#[test]
fn a_hit_before_the_hold_reply_plays_the_pad() {
    let mut drive = hold_drive();
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 256])), "top left");
    // PAD BANK moved the device to its second layer; the reply is still on its way.
    cell(&mut drive, 64);
    assert!(drive.app.pads.hit_at[0].is_some());
    assert!(!drive.app.edit);
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pads, "edit pad")),
    );
    assert!(drive.app.edit);
}

/// Scenario "A failed status read hides the panel" (controller-reach).
#[test]
fn a_failed_status_read_hides_the_panel() {
    let mut drive = hold_drive();
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 256])), "top left");
    hold(&mut drive, true);
    assert!(drive.app.panel.visible());
    drive.midi().recorded_smc_failure();
    drive.frame();
    assert!(!drive.app.panel.visible());
    // The unit is not verified now: a first-layer hit plays.
    cell(&mut drive, 48);
    assert!(drive.app.pads.hit_at[0].is_some());
    assert!(!drive.app.edit);
}

/// Scenario "PAD BANK ignores invalid or disconnected sources" (controller-reach).
#[test]
fn pad_bank_ignores_invalid_or_disconnected_sources() {
    let mut drive = hold_drive();
    pad_bank_request(&mut drive, XS_PORT);
    assert!(!drive.app.panel.visible());
    let old_request = Instant::now();
    drive.midi().recorded_disconnect(SMC);
    drive.frame();
    pad_bank_request(&mut drive, SMC);
    assert!(!drive.app.panel.visible());
    drive.midi().recorded_connect(SMC);
    drive.midi().recorded_smc_setup();
    drive.frame();
    drive.midi().recorded_pad_bank(SMC, old_request, true);
    drive.frame();
    assert!(!drive.app.panel.visible());
    pad_bank_request(&mut drive, SMC);
    assert!(drive.app.panel.visible());
}

/// Scenario "First-hit reveal works without the hold shortcut" (controller-reach).
#[test]
fn first_hit_reveal_works_without_the_hold_shortcut() {
    // Shortcut off on a verified unit, then shortcut on with an unverified unit.
    for mut drive in [smc_drive(), Drive::smc_offline()] {
        pad_bank_request(&mut drive, SMC);
        assert!(!drive.app.panel.visible());
        cell(&mut drive, panel_note(2));
        assert!(drive.app.panel.visible());
        assert_eq!(drive.app.screen, Screen::Pads);
        list_button(&mut drive, false);
        cell(&mut drive, panel_note(2));
        assert_eq!(drive.app.screen, Screen::Files);
    }
}

/// Scenario "A question takes pads 1 and 4 without a hold" (command-panel).
#[test]
fn a_question_takes_pads_1_and_4_without_a_hold() {
    let mut drive = hold_drive();
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 256])), "top left");
    drive.app.session_dirty = Some(Instant::now());
    hold(&mut drive, true);
    list_button(&mut drive, false);
    cell(&mut drive, panel_note(12));
    hold(&mut drive, false);
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    // Another pad stays silent and the question stays.
    cell(&mut drive, 48);
    assert!(drive.app.pads.hit_at[0].is_none());
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    // Device pad 1 from the first layer cancels without a hold.
    cell(&mut drive, 36);
    assert!(drive.app.ask.is_none());
    assert!(!drive.app.quit_requested);

    hold(&mut drive, true);
    list_button(&mut drive, false);
    cell(&mut drive, panel_note(12));
    hold(&mut drive, false);
    // Device pad 4 from the second layer confirms.
    cell(&mut drive, 55);
    assert!(drive.app.quit_requested);
}

/// Scenario "A panel action followed by pad 4 confirms only the question" (command-panel).
#[test]
fn a_panel_action_followed_by_pad_4_confirms_only_the_question() {
    let source = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let existing = Arc::new(Sample::mono(vec![0.4; 4_800]));
    let mut drive = smc_drive();
    drive.load_pad(0, source.clone(), "source");
    drive.load_pad(2, existing, "existing");
    cell(&mut drive, panel_note(1));
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pads, "copy pad")),
    ); // Copy pad 1.
    // The occupied target and device pad 4 reach the program before one redraw.
    drive.bytes(SMC, &[0x99, 50, 100]);
    drive.bytes(SMC, &[0x89, 50, 0]);
    drive.bytes(SMC, &[0x99, 55, 100]);
    drive.bytes(SMC, &[0x89, 55, 0]);
    drive.frame();
    assert!(drive.app.ask.is_none());
    assert!(Arc::ptr_eq(
        drive.app.pads.sound[2].as_ref().unwrap(),
        &source
    ));
    let changed = (0..16)
        .filter(|&pad| pad != 0 && drive.app.pads.sound[pad].is_some())
        .count();
    assert_eq!(changed, 1, "only the asked target changes");
}

/// Scenario "A held pad 4 release does not answer" (command-panel).
#[test]
fn a_held_pad_4_release_does_not_answer() {
    let mut drive = hold_drive();
    // Device pad 4 is note 39 on the first layer, the bottom-right pad 15.
    drive.load_pad(15, Arc::new(Sample::mono(vec![0.2; 48_000])), "gate");
    drive.app.pads.gate[15] = true;
    drive.app.session_dirty = Some(Instant::now());
    drive.app.publish_midi(&drive.shared);
    drive.bytes(SMC, &[0x99, 39, 100]);
    drive.frame();
    assert!(drive.app.pads.held[15]);
    hold(&mut drive, true);
    list_button(&mut drive, false);
    cell(&mut drive, panel_note(12));
    hold(&mut drive, false);
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    drive.bytes(SMC, &[0x89, 39, 0]);
    drive.frame();
    // The offline drive sends no engine commands; the held state shows the release.
    assert!(!drive.app.pads.held[15], "the Gate ends once");
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    assert!(!drive.app.quit_requested);
}

/// Scenario "The panel behind a question shows no and yes" (command-panel).
#[test]
fn the_panel_behind_a_question_shows_no_and_yes() {
    let mut drive = smc_drive();
    drive.app.session_dirty = Some(Instant::now());
    cell(&mut drive, panel_note(1));
    list_button(&mut drive, false);
    cell(&mut drive, panel_note(12)); // Exit asks.
    assert!(matches!(drive.app.ask, Some(Ask::Exit { .. })));
    let panel = drive.app.panel_snapshot().unwrap();
    for (index, label) in panel.labels.iter().enumerate() {
        let want = match index {
            12 => Some("no"),
            15 => Some("yes"),
            _ => None,
        };
        assert_eq!(label.as_deref(), want, "cell {}", index + 1);
    }
}

/// Scenario "The printed pad 1 reads as pad 1" (screen-layout).
#[test]
fn the_printed_pad_1_reads_as_pad_1() {
    let mut drive = hold_drive();
    assert!(tui::pads::device_numbers());
    drive.bytes(SMC, &[0x99, 36, 100]);
    drive.bytes(SMC, &[0x89, 36, 0]);
    drive.frame();
    assert_eq!(drive.app.pads.selected, 12, "the bottom-left pad");
    let rows = shared_feedback_rows(&mut drive, 120, 40, Instant::now());
    let text = rows.join("\n");
    assert!(rows.iter().any(|row| row.contains("┌1 Z")), "{text}");
    assert!(rows.iter().any(|row| row.contains("┌13 1")), "{text}");
    assert!(text.contains("pad 1  bank 1"), "{text}");
}

/// Scenario "Keyboard numbers return without an SMC-PAD" (screen-layout).
#[test]
fn keyboard_numbers_return_without_an_smc_pad() {
    let mut drive = hold_drive();
    drive.app.pads.selected = 12;
    drive.midi().recorded_disconnect(SMC);
    drive.frame();
    assert!(!tui::pads::device_numbers());
    let rows = shared_feedback_rows(&mut drive, 120, 40, Instant::now());
    let text = rows.join("\n");
    assert!(rows.iter().any(|row| row.contains("┌Z─")), "{text}");
    assert!(text.contains("pad Z  bank 1"), "{text}");
    assert_eq!(drive.app.pads.selected, 12);
}

/// Scenario "Pad edit rows show knobs 7, 5 and 3" (screen-layout).
#[test]
fn pad_edit_rows_show_knobs_7_5_and_3() {
    let mut drive = hold_drive();
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 4_800])), "tone");
    drive.app.edit = true;
    while drive.app.pads.page().0 != "TONE" {
        drive.app.apply(Action::PageNext);
    }
    let rows = shared_feedback_rows(&mut drive, 120, 40, Instant::now());
    let text = rows.join("\n");
    for row in ["7 volume", "5 pitch", "3 mute"] {
        assert!(text.contains(row), "{row}: {text}");
    }
    for (knob, word) in [(7, "volume"), (5, "pitch"), (3, "mute")] {
        assert_eq!(
            drive.app.smc_page_role(knob).unwrap().0,
            word,
            "knob {knob}"
        );
    }
}

/// Scenario "Learn on Settings explains the refusal" (controller-reach).
#[test]
fn learn_on_settings_explains_the_refusal() {
    let mut drive = hold_drive();
    drive.app.screen = Screen::Settings;
    drive.app.publish_midi(&drive.shared);
    let tab = drive.app.settings_tab;
    // Learn has no panel cell (138): F10 is its only way in.
    drive.tap(KeyCode::F(10), M::NONE);
    assert!(drive.app.screen_learn.is_none());
    assert_eq!(drive.app.screen, Screen::Settings);
    assert_eq!(drive.app.settings_tab, tab);
    assert!(
        drive.last_notice().text.contains("MIDI tab"),
        "{}",
        drive.last_notice().text
    );
}

fn file_in_hand_by_smc() -> (Drive, TransitionFiles) {
    let files = TransitionFiles::new();
    let mut drive = hold_drive();
    drive.app.screen = Screen::Files;
    drive.app.browser.dir = files.0.clone();
    drive.app.browser.reread();
    drive.app.browser.selected = drive
        .app
        .browser
        .entries
        .iter()
        .position(|entry| entry.name == "tone.wav")
        .unwrap();
    drive.app.publish_midi(&drive.shared);
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Files, "take file")),
    );
    hold(&mut drive, false);
    assert!(drive.app.browser.in_hand.is_some());
    (drive, files)
}

/// Scenario "The mode line shows an item in hand" (screen-layout).
#[test]
fn the_mode_line_shows_an_item_in_hand() {
    let (drive, _files) = file_in_hand_by_smc();
    let now = Instant::now();
    let strip = drive.app.smc_knob_strip(now).unwrap();
    assert_eq!(
        strip.mode.as_deref(),
        Some("IN HAND tone.wav · hit a pad · PAD BANK + 1 cancel")
    );
    assert!(
        !drive.app.panel.visible(),
        "the line stays after the release"
    );
}

/// Scenario "The mode line shows a running recording" (screen-layout).
#[test]
fn the_mode_line_shows_a_running_recording() {
    let mut drive = hold_drive();
    turn(&mut drive, 37, 65); // An SMC-PAD input.
    let now = Instant::now();
    drive.app.recording = Some((
        audio::SOURCE_OUTPUT,
        now - std::time::Duration::from_secs(4),
    ));
    assert_eq!(
        drive.app.mode_line(now).as_deref(),
        Some("● REC OUTPUT 0:04 · STOP end rec")
    );
}

/// Scenario "The mode line shows the next Assign slice" (screen-layout).
#[test]
fn the_mode_line_shows_the_next_assign_slice() {
    let mut drive = hold_drive();
    drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 48_000])), 9);
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Chop, "put on pads")),
    );
    hold(&mut drive, false);
    for note in [36, 37] {
        drive.bytes(SMC, &[0x99, note, 100]);
        drive.bytes(SMC, &[0x89, note, 0]);
        drive.frame();
    }
    let line = drive.app.mode_line(Instant::now()).unwrap();
    assert!(line.starts_with("ASSIGN 2/9 · slice 3 next"), "{line}");
    assert!(
        line.contains("PAD BANK + 4 apply") && line.contains("PAD BANK + 1 cancel"),
        "{line}"
    );
}

/// Scenario "SMC-PAD input switches hints to controller words" (screen-help).
#[test]
fn smc_pad_input_switches_hints_to_controller_words() {
    let (drive, _files) = file_in_hand_by_smc();
    assert!(drive.app.input_sources.last_is_smc());
    let line = drive.app.mode_line(Instant::now()).unwrap();
    assert!(
        line.contains("hit a pad") && line.contains("PAD BANK + 1"),
        "{line}"
    );
    assert!(!line.contains("Enter") && !line.contains("Esc"), "{line}");
    drive.app.smc_knob_strip(Instant::now());
    assert!(tui::pads::smc_words(), "the Files text names the pads");
}

/// Scenario "A key switches hints back" (screen-help).
#[test]
fn a_key_switches_hints_back() {
    let (mut drive, _files) = file_in_hand_by_smc();
    drive.tap(KeyCode::Down, M::NONE);
    assert!(!drive.app.input_sources.last_is_smc());
    let line = drive.app.mode_line(Instant::now()).unwrap();
    assert!(line.ends_with("Esc cancel"), "{line}");
    drive.app.smc_knob_strip(Instant::now());
    assert!(!tui::pads::smc_words());
}

/// Scenario "A question changes answer words with the device" (screen-help).
#[test]
fn a_question_changes_answer_words_with_the_device() {
    let mut drive = hold_drive();
    drive.tap(KeyCode::Down, M::NONE);
    drive.app.ask = Some(Ask::ClearPattern {
        lines: vec!["Clear A1?".into()],
    });
    let answers = drive.app.question_view(&drive.keymap).unwrap().answers;
    assert!(
        answers.iter().any(|answer| answer.starts_with("KB ")),
        "{answers:?}"
    );
    turn(&mut drive, 37, 65);
    assert!(drive.app.ask.is_some());
    let answers = drive.app.question_view(&drive.keymap).unwrap().answers;
    assert!(
        answers
            .iter()
            .any(|answer| answer.contains("P1: keep · P4:")),
        "{answers:?}"
    );
    assert!(
        !answers.iter().any(|answer| answer.starts_with("KB ")),
        "{answers:?}"
    );
}

/// Holds PAD BANK, reads the device pad 1 label, presses pad 1 and releases.
fn exit_by_hold(drive: &mut Drive) -> Option<String> {
    hold(drive, true);
    let label = drive
        .app
        .panel_snapshot()
        .and_then(|panel| panel.labels[12].clone());
    cell(drive, panel_note(13));
    hold(drive, false);
    label
}

/// Opens a cell of the screens view during a hold. A target mode first opens its menu.
fn screens_cell(drive: &mut Drive, slot: u8) {
    hold(drive, true);
    if drive.app.panel_snapshot().is_some_and(|panel| panel.target) && drive.app.ask.is_none() {
        cell(drive, panel_note(15));
    }
    list_button(drive, false);
    cell(drive, panel_note(slot));
    hold(drive, false);
}

/// Runs an action cell of the current view during a hold.
fn action_cell(drive: &mut Drive, slot: u8) {
    hold(drive, true);
    cell(drive, panel_note(slot));
    hold(drive, false);
}

/// Runs the shipped action cell `label` of `view` during a hold.
fn labelled_cell(drive: &mut Drive, view: panel::View, label: &str) {
    action_cell(drive, slot_of(view, label));
}

/// Scenario "A hold in Help shows the close cell" (command-panel).
#[test]
fn a_hold_in_help_shows_the_close_cell() {
    let mut drive = hold_drive();
    screens_cell(&mut drive, 8);
    assert!(drive.app.help_open);
    hold(&mut drive, true);
    let panel = drive.app.panel_snapshot().unwrap();
    assert_eq!(panel.labels[12].as_deref(), Some("close"));
    assert_eq!(panel.labels[8].as_deref(), Some("up"));
    assert_eq!(panel.labels[9].as_deref(), Some("down"));
    cell(&mut drive, panel_note(13));
    hold(&mut drive, false);
    assert!(!drive.app.help_open);
}

/// Scenario "No mode is a dead end for the SMC-PAD" (command-panel).
#[test]
fn no_mode_is_a_dead_end_for_the_smc_pad() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let check = |name: &str, drive: &mut Drive, open: fn(&Drive) -> bool| {
        assert!(open(drive), "{name}: the mode is open");
        let label = exit_by_hold(drive);
        assert!(label.is_some(), "{name}: pad 1 has no label");
        assert!(!open(drive), "{name}: pad 1 does not leave");
    };

    let mut drive = hold_drive();
    screens_cell(&mut drive, 8);
    check("Help", &mut drive, |d| d.app.help_open);

    let files = TransitionFiles::new();
    let mut drive = hold_drive();
    drive.app.paths.projects = files.0.clone();
    drive.app.project = Some(Project {
        name: "A".into(),
        folder: files.0.join("A"),
    });
    drive.app.user_path = Some(files.0.join("config.toml"));
    drive.app.publish_midi(&drive.shared);
    screens_cell(&mut drive, 10);
    check("project list", &mut drive, |d| {
        d.app.project_picker.is_some()
    });

    let mut drive = reach_drive_from(hold_drive(), panel::View::Pads, sound.clone());
    // Learn opens from the keyboard only (138); device pad 1 still leaves it.
    drive.tap(KeyCode::F(10), M::NONE);
    check("screen learn", &mut drive, |d| d.app.screen_learn.is_some());

    let mut drive = hold_drive();
    drive.app.screen = Screen::Settings;
    drive.app.settings_tab = 2;
    drive.app.learning = Some(learn::Learning::new(action_info("pad.copy")));
    drive.app.publish_midi(&drive.shared);
    check("MIDI learn", &mut drive, |d| d.app.learning.is_some());

    let mut drive = reach_drive_from(hold_drive(), panel::View::Pads, sound.clone());
    drive.tap(KeyCode::Char('n'), M::CONTROL);
    check("pad name", &mut drive, |d| d.app.pad_naming.is_some());

    let mut drive = reach_drive_from(hold_drive(), panel::View::Files, sound.clone());
    drive.tap(KeyCode::Char('/'), M::NONE);
    check("file filter", &mut drive, |d| d.app.browser.editing);

    let mut drive = hold_drive();
    drive.app.session_dirty = Some(Instant::now());
    screens_cell(&mut drive, 12);
    check("exit question", &mut drive, |d| d.app.ask.is_some());

    let mut drive = reach_drive_from(hold_drive(), panel::View::Pads, sound.clone());
    labelled_cell(&mut drive, panel::View::Pads, "copy pad");
    check("copy", &mut drive, |d| d.app.copy_in_hand.is_some());

    let mut drive = reach_drive_from(hold_drive(), panel::View::Pads, sound.clone());
    labelled_cell(&mut drive, panel::View::Pads, "swap pads");
    check("swap", &mut drive, |d| d.app.pad_in_hand.is_some());

    let mut drive = reach_drive_from(hold_drive(), panel::View::Pads, sound.clone());
    labelled_cell(&mut drive, panel::View::Pads, "edit pad");
    check("pad edit", &mut drive, |d| d.app.edit);

    let mut drive = reach_drive_from(hold_drive(), panel::View::Pads, sound.clone());
    labelled_cell(&mut drive, panel::View::Pads, "mark pad");
    check("marking", &mut drive, |d| d.app.pads.marking);

    let mut drive = reach_drive_from(hold_drive(), panel::View::Chop, sound.clone());
    labelled_cell(&mut drive, panel::View::Chop, "put on pads");
    check("assign", &mut drive, |d| d.app.plan.is_some());

    let mut drive = reach_drive_from(hold_drive(), panel::View::Chop, sound.clone());
    labelled_cell(&mut drive, panel::View::Chop, "edit slice");
    check("chop edit", &mut drive, |d| d.app.edit);

    let mut drive = reach_drive_from(hold_drive(), panel::View::Pattern, sound.clone());
    labelled_cell(&mut drive, panel::View::Pattern, "A1 setup");
    check("A1 knobs", &mut drive, |d| d.app.pattern_panel.is_some());
}

/// A mode the owner can enter from the SMC-PAD: name, entry, and the test that it is open.
type ModeCase = (&'static str, fn(&mut Drive), fn(&Drive) -> bool);

fn mode_cases() -> Vec<ModeCase> {
    vec![
        (
            "pad edit",
            |d| labelled_cell(d, panel::View::Pads, "edit pad"),
            |d| d.app.edit,
        ),
        (
            "copy",
            |d| labelled_cell(d, panel::View::Pads, "copy pad"),
            |d| d.app.copy_in_hand.is_some(),
        ),
        (
            "swap",
            |d| labelled_cell(d, panel::View::Pads, "swap pads"),
            |d| d.app.pad_in_hand.is_some(),
        ),
        (
            "marking",
            |d| labelled_cell(d, panel::View::Pads, "mark pad"),
            |d| d.app.pads.marking,
        ),
        (
            "assign",
            |d| {
                screens_cell(d, 3);
                labelled_cell(d, panel::View::Chop, "put on pads");
            },
            |d| d.app.plan.is_some(),
        ),
        (
            "chop edit",
            |d| {
                screens_cell(d, 3);
                labelled_cell(d, panel::View::Chop, "edit slice");
            },
            |d| d.app.edit && d.app.screen == Screen::Chop,
        ),
        (
            "file in hand",
            |d| {
                screens_cell(d, 2);
                labelled_cell(d, panel::View::Files, "take file");
            },
            |d| d.app.browser.in_hand.is_some(),
        ),
        (
            "A1 knobs",
            |d| {
                screens_cell(d, 4);
                labelled_cell(d, panel::View::Pattern, "A1 setup");
            },
            |d| d.app.pattern_panel.is_some(),
        ),
        (
            "A1 step edit",
            |d| {
                screens_cell(d, 4);
                labelled_cell(d, panel::View::Pattern, "edit steps");
            },
            |d| d.app.pattern_view.edit,
        ),
        (
            "A1 erase",
            |d| {
                screens_cell(d, 4);
                labelled_cell(d, panel::View::Pattern, "erase hits");
            },
            |d| d.app.erase_pads,
        ),
        (
            "screen learn",
            // Learn opens from the keyboard only (138).
            |d| {
                d.tap(KeyCode::F(10), M::NONE);
            },
            |d| d.app.screen_learn.is_some(),
        ),
        ("help", |d| screens_cell(d, 8), |d| d.app.help_open),
    ]
}

/// The owner's starting points: empty pads, sounds on bank 2, and marked pads.
fn state_drive(state: usize, files: &TransitionFiles) -> Drive {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = hold_drive();
    drive.app.loader = Some(loader::start(48_000));
    drive.app.browser.dir = files.0.clone();
    drive.app.browser.reread();
    drive.app.browser.selected = drive
        .app
        .browser
        .entries
        .iter()
        .position(|entry| entry.name == "tone.wav")
        .unwrap();
    drive.load_pad(0, sound.clone(), "kick");
    drive.prepare_chop(sound.clone(), 20);
    drive.app.screen = Screen::Pads;
    drive.app.pattern_view.overview = false;
    if state >= 1 {
        for pad in 16..32 {
            drive.load_pad(pad, sound.clone(), "bank two");
        }
        drive.app.pads.bank = 1;
        drive.app.pads.selected = 0;
    }
    if state >= 2 {
        drive.app.pads.marked.insert(16);
    }
    drive.app.pads.pattern = Some(session::PatternState {
        slot: "A1".into(),
        bars: 1,
        quantize: "1/16".into(),
        strength: 100,
        hits: vec![[0, 16, 100, 0]],
    });
    drive.app.session_dirty = Some(Instant::now());
    drive.app.publish_midi(&drive.shared);
    drive
}

/// Scenario "No mode is a dead end for the SMC-PAD" (command-panel): every mode, in each
/// starting state, with Help and the exit question opened over it. The overlay closes
/// with device pad 1 and leaves the mode below open; device pad 1 then leaves the mode.
#[test]
fn every_mode_and_overlay_has_a_way_back_in_each_state() {
    let files = TransitionFiles::new();
    let mut failures = Vec::new();
    for state in 0..3 {
        for (name, enter, open) in mode_cases() {
            for overlay in ["none", "help", "exit question"] {
                let mut drive = state_drive(state, &files);
                enter(&mut drive);
                let case = format!("state {state}, {name}, overlay {overlay}");
                if !open(&drive) {
                    failures.push(format!("{case}: the mode does not open"));
                    continue;
                }
                if overlay != "none" && name != "help" && name != "screen learn" {
                    screens_cell(&mut drive, if overlay == "help" { 8 } else { 12 });
                    let shown = if overlay == "help" {
                        drive.app.help_open
                    } else {
                        drive.app.ask.is_some()
                    };
                    if !shown {
                        failures.push(format!("{case}: the overlay does not open"));
                        continue;
                    }
                    if exit_by_hold(&mut drive).is_none() {
                        failures.push(format!("{case}: no label on pad 1 over the mode"));
                    }
                    if drive.app.help_open || drive.app.ask.is_some() {
                        failures.push(format!("{case}: pad 1 does not close the overlay"));
                        continue;
                    }
                    if !open(&drive) {
                        failures.push(format!("{case}: closing the overlay also left the mode"));
                        continue;
                    }
                }
                if drive.app.input_sources.last_is_smc()
                    && let Some(line) = drive.app.mode_line(Instant::now())
                    && (line.contains("Enter") || line.contains("Esc"))
                {
                    failures.push(format!("{case}: keyboard words after SMC input: {line}"));
                }
                if exit_by_hold(&mut drive).is_none() {
                    failures.push(format!("{case}: no label on pad 1"));
                }
                if open(&drive) {
                    failures.push(format!("{case}: pad 1 does not leave the mode"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Scenario "No mode is a dead end for the SMC-PAD" (command-panel): with PAD BANK `Off`,
/// text entry opened from the keyboard closes with device pad 1 of the second layer.
#[test]
fn text_entry_closes_with_pad_1_when_pad_bank_is_off() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive(panel::View::Pads, sound.clone());
    drive.tap(KeyCode::Char('n'), M::CONTROL);
    assert!(drive.app.pad_naming.is_some());
    cell(&mut drive, panel_note(13));
    assert!(drive.app.pad_naming.is_none(), "pad name closes");

    let mut drive = reach_drive(panel::View::Files, sound);
    drive.tap(KeyCode::Char('/'), M::NONE);
    assert!(drive.app.browser.editing);
    cell(&mut drive, panel_note(13));
    assert!(!drive.app.browser.editing, "file filter closes");
}

/// Scenario "A cell bound by learning shows the learned action" (command-panel).
#[test]
fn a_cell_bound_by_learning_shows_the_learned_action() {
    let mut drive = hold_drive();
    // Device pad 6 of the second layer is note 57, panel cell 10: open in the screens view.
    learn::assign(
        &mut drive.app.learned,
        "SMC-PAD",
        input::Group::Global,
        input::midi::Key::Note(57),
        "pad.mode_next",
    );
    drive.app.sync_learned("SMC-PAD", input::Group::Global);
    drive.app.publish_midi(&drive.shared);
    hold(&mut drive, true);
    list_button(&mut drive, false);
    let label = drive.app.panel_snapshot().unwrap().labels[9].clone();
    hold(&mut drive, false);
    assert_eq!(label.as_deref(), Some(input::short_label("pad.mode_next")));
    assert_eq!(crate::controller_map::smc_printed("SMC-PAD", "c10"), "PB6");
}

/// Scenario "Learn on Settings explains the refusal" (controller-reach): a dim cell with a
/// label, such as mixer, says that it is not available instead of doing nothing.
#[test]
fn a_dim_labelled_cell_says_it_is_not_available() {
    let mut drive = hold_drive();
    screens_cell(&mut drive, 6);
    assert_eq!(drive.last_notice().text, "mixer: not available yet");
}

/// Scenario "A cell hit lights the cell" (screen-layout).
#[test]
fn a_cell_hit_lights_the_cell() {
    let mut drive = hold_drive();
    hold(&mut drive, true);
    let mode = slot_of(panel::View::Pads, "play mode");
    cell(&mut drive, panel_note(mode));
    let panel = drive.app.panel_snapshot().unwrap();
    assert_eq!(panel.flash, Some(usize::from(mode - 1)));
    assert_eq!(
        panel.labels[usize::from(mode - 1)].as_deref(),
        Some("play mode")
    );
    list_button(&mut drive, false);
    let before = drive.seen();
    cell(&mut drive, panel_note(6));
    let panel = drive.app.panel_snapshot().unwrap();
    assert_eq!(panel.labels[5].as_deref(), Some("mixer"));
    assert_eq!(panel.flash, Some(5));
    // The dim mixer cell only says that it is not available (134).
    let mut after = drive.seen();
    assert_eq!(after.notice.as_deref(), Some("mixer: not available yet"));
    after.notice = before.notice.clone();
    assert_eq!(after, before, "the dim mixer cell runs no action");
    drive.app.panel.flash = Some((6, Instant::now() - std::time::Duration::from_millis(121)));
    assert_eq!(drive.app.panel_snapshot().unwrap().flash, None);
    // A grey action cell lights grey, runs no action and says why (138).
    list_button(&mut drive, true);
    let copy = slot_of(panel::View::Pads, "copy pad");
    let before = drive.seen();
    cell(&mut drive, panel_note(copy));
    let panel = drive.app.panel_snapshot().unwrap();
    assert!(panel.dim[usize::from(copy - 1)]);
    assert_eq!(panel.flash, Some(usize::from(copy - 1)));
    let mut after = drive.seen();
    assert_eq!(
        after.notice.as_deref(),
        Some("Copy what? This pad holds nothing.")
    );
    after.notice = before.notice.clone();
    assert_eq!(after, before, "the grey cell runs no action");
}

fn old_layout_file(files: &TransitionFiles) -> PathBuf {
    let path = files.0.join("old-smc.toml");
    std::fs::write(
        &path,
        r#"
[devices]
seeded = ["SMC-PAD", "Recorded pads", "X-Session Pro"]

[midi."SMC-PAD".global]
"cc 30" = "projects.scroll"
"cc 35" = "pad.pitch"
"cc 36" = "none"
"cc 37" = "bank"
"cc 38" = "pad.volume"
"cc 45" = "master.volume"

[midi."SMC-PAD".pads]
"cc 25" = "pad.select_left"
"cc 26" = "pad.select_right"
"cc 30" = "bank"
"cc 31" = "pad.volume"

[midi."SMC-PAD".chop]
"cc 34" = "slice.end"

[midi."SMC-PAD".pattern]
"cc 31" = "pattern.bars"

[midi."SMC-PAD".fx]

[midi."X-Session Pro".global]
"cc 24" = "pad.pitch"
"#,
    )
    .unwrap();
    path
}

fn saved_layout_drive(path: &Path) -> Drive {
    let mut drive = Drive::with_saved_bindings_and_smc(true, Some(path), true);
    drive.midi().recorded_smc_setup();
    drive.frame();
    drive
}

fn select_layout_row(drive: &mut Drive, find: impl Fn(&MidiListRow) -> bool) {
    let target = drive.app.midi_list().iter().position(find).unwrap();
    while drive.app.midi_row != target {
        let key = if drive.app.midi_row < target {
            KeyCode::Down
        } else {
            KeyCode::Up
        };
        let previous = drive.app.midi_row;
        drive.tap(key, M::NONE);
        assert_ne!(drive.app.midi_row, previous);
    }
    drive.tap(KeyCode::Enter, M::NONE);
}

fn open_layout_review(drive: &mut Drive) {
    drive.tap(KeyCode::F(12), M::NONE);
    drive.tap(KeyCode::Tab, M::NONE);
    drive.tap(KeyCode::Tab, M::NONE);
    assert_eq!(drive.app.settings_tab, 2);
    select_layout_row(drive, |row| matches!(row, MidiListRow::SmcLayoutList(_)));
}

fn select_layout_master(drive: &mut Drive) {
    select_layout_row(
        drive,
        |row| matches!(row, MidiListRow::SmcLayoutSlot(slot) if slot.label().contains("(cc 37)")),
    );
    assert!(matches!(drive.app.ask, Some(Ask::SmcLayout(_))));
}

/// Scenario "Saved SMC-PAD roles survive the layout review" (input-devices).
#[test]
fn saved_smc_pad_roles_survive_the_layout_review() {
    let files = TransitionFiles::new();
    let path = old_layout_file(&files);
    let mut drive = saved_layout_drive(&path);
    let before = drive.app.learned.clone();
    open_layout_review(&mut drive);

    let rows = drive.app.midi_screen();
    assert!(rows.iter().any(|row| matches!(row,
        tui::MidiRow::Action { label, who, .. }
            if label.contains("e8 (cc 37)") && who.contains("→") && who.contains("master")
    )));
    assert!(!drive.app.midi_list().iter().any(|row| matches!(row,
        MidiListRow::SmcLayoutSlot(slot)
            if ["cc 35)", "cc 36)", "cc 32)"].iter().any(|key| slot.label().contains(key))
    )));
    select_layout_master(&mut drive);
    assert_eq!(drive.app.learned, before);
    drive.tap(KeyCode::Enter, M::NONE);
    let mut expected = before.clone();
    expected
        .get_mut("SMC-PAD")
        .unwrap()
        .get_mut("global")
        .unwrap()
        .insert("cc 37".into(), "master.volume".into());
    assert_eq!(drive.app.learned, expected);
    assert_eq!(config::load_from(&path).config.midi, expected);

    let mut restarted = saved_layout_drive(&path);
    assert_eq!(restarted.app.learned, expected);
    let bank = restarted.app.pads.bank;
    let volume = restarted.app.master_db;
    turn(&mut restarted, 37, 65);
    assert!(restarted.app.master_db > volume);
    assert_eq!(restarted.app.pads.bank, bank);
}

/// Scenario "Canceled or refused layout updates are safe to retry" (input-devices).
#[test]
fn canceled_or_refused_layout_updates_are_safe_to_retry() {
    let files = TransitionFiles::new();
    let path = old_layout_file(&files);
    let mut drive = saved_layout_drive(&path);
    let before = drive.app.learned.clone();
    open_layout_review(&mut drive);
    select_layout_master(&mut drive);
    cell(&mut drive, 52);
    assert!(drive.app.ask.is_none());
    assert_eq!(drive.app.learned, before);
    assert_eq!(config::load_from(&path).config.midi, before);

    drive.app.user_path = Some(files.0.join("A"));
    select_layout_master(&mut drive);
    cell(&mut drive, 55);
    assert!(drive.app.ask.is_none());
    assert_eq!(drive.app.notices.current().unwrap().level, Level::Error);
    assert_eq!(drive.app.learned, before);
    assert_eq!(config::load_from(&path).config.midi, before);
    drive.tap(KeyCode::F(2), M::NONE);
    let volume = drive.app.master_db;
    let bank = drive.app.pads.bank;
    turn(&mut drive, 37, 65);
    assert_eq!(drive.app.master_db, volume);
    assert_ne!(drive.app.pads.bank, bank);

    drive.app.user_path = Some(path.clone());
    drive.tap(KeyCode::F(12), M::NONE);
    select_layout_row(&mut drive, |row| matches!(row, MidiListRow::SmcLayoutApply));
    cell(&mut drive, 55);
    assert!(drive.app.ask.is_none());
    assert_eq!(drive.app.learned["X-Session Pro"], before["X-Session Pro"]);
    assert_eq!(
        drive.app.learned["SMC-PAD"]["global"]["cc 37"],
        "master.volume"
    );
    assert_eq!(drive.app.learned["SMC-PAD"]["global"]["cc 35"], "pad.pitch");
    assert_eq!(drive.app.learned["SMC-PAD"]["global"]["cc 36"], "none");
    assert_eq!(config::load_from(&path).config.midi, drive.app.learned);
    assert!(
        !drive
            .app
            .midi_list()
            .iter()
            .any(|row| matches!(row, MidiListRow::SmcLayoutList(_)))
    );
    let bank = drive.app.pads.bank;
    turn(&mut drive, 37, 65);
    assert!(drive.app.master_db > volume);
    assert_eq!(drive.app.pads.bank, bank);
}

/// Scenario "Removed SMC-PAD knob stays unassigned on a saved map" (input-devices).
#[test]
fn removed_smc_pad_knob_stays_unassigned_on_a_saved_map() {
    let files = TransitionFiles::new();
    let path = old_layout_file(&files);
    let saved = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, saved.replace("\"cc 37\" = \"bank\"\n", "")).unwrap();
    let mut drive = saved_layout_drive(&path);
    let volume = drive.app.master_db;
    turn(&mut drive, 37, 65);
    assert_eq!(drive.app.master_db, volume);
    assert_eq!(drive.app.pads.bank, 0);

    open_layout_review(&mut drive);
    assert!(!drive.app.midi_list().iter().any(|row| matches!(row,
        MidiListRow::SmcLayoutSlot(slot) if slot.label().contains("cc 37)")
    )));
    select_layout_row(&mut drive, |row| matches!(row, MidiListRow::SmcLayoutApply));
    drive.tap(KeyCode::Enter, M::NONE);
    let saved = config::load_from(&path).config.midi;
    assert!(!saved["SMC-PAD"]["global"].contains_key("cc 37"));
    turn(&mut drive, 37, 65);
    assert_eq!(drive.app.master_db, volume);
}

/// Scenario "A change 124 map removes the bank 2 pad roles" (input-devices).
#[test]
fn a_change_124_map_removes_the_bank_2_pad_roles() {
    let files = TransitionFiles::new();
    let path = files.0.join("smc-124.toml");
    std::fs::write(
        &path,
        r#"
[devices]
seeded = ["SMC-PAD", "Recorded pads", "X-Session Pro"]

[midi."SMC-PAD".global]
"cc 30" = "smc.position"
"cc 37" = "master.volume"
"cc 38" = "pad.volume"
"cc 39" = "pad.cutoff"
"cc 40" = "pad.pitch"
"cc 41" = "pad.decay"
"cc 42" = "pad.send_1"
"cc 43" = "pad.send_2"
"cc 45" = "master.volume"

[midi."SMC-PAD".pads]
"cc 31" = "smc.page_2"
"cc 36" = "smc.page_7"
"#,
    )
    .unwrap();
    let mut drive = saved_layout_drive(&path);
    open_layout_review(&mut drive);
    select_layout_row(&mut drive, |row| matches!(row, MidiListRow::SmcLayoutApply));
    drive.tap(KeyCode::Enter, M::NONE);
    let saved = config::load_from(&path).config.midi;
    for cc in 38..=43 {
        assert!(
            !saved["SMC-PAD"]["global"].contains_key(&format!("cc {cc}")),
            "cc {cc}"
        );
    }
    assert_eq!(saved["SMC-PAD"]["pads"].len(), 2);

    let mut restarted = saved_layout_drive(&path);
    let abs = restarted.app.pads.selected_abs();
    let pad = restarted.app.pads.snapshot(abs);
    for cc in 38..=43 {
        turn(&mut restarted, cc, 65);
    }
    assert!(restarted.app.pads.snapshot(abs) == pad);
    let volume = restarted.app.master_db;
    turn(&mut restarted, 45, 65);
    assert!(restarted.app.master_db > volume);
}

/// Scenario "The strip shows the turned knob and its value" (screen-layout).
/// `knob_strip::tests` draws the same strip at both supported widths.
#[test]
fn the_strip_shows_the_turned_knob_and_its_value() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive(panel::View::PadEdit, sound);
    drive.app.pads.param = pads::Param::Volume;
    turn(&mut drive, 36, 65);
    let at = drive.app.value_popup.as_ref().unwrap().at;
    let strip = drive.app.smc_knob_strip(at).unwrap();
    assert_eq!((strip.last_bank, strip.last_knob), (Some(0), Some(6)));
    assert_eq!(strip.roles[0][6], ("volume", true));
    let value = strip.value.unwrap();
    assert!(value.starts_with("B1 7 volume"), "{value}");
    assert!(value.contains("+1 dB"), "{value}");

    let later = at + crate::LIT_FOR;
    assert!(drive.app.smc_knob_strip(later).unwrap().value.is_none());
    drive.tap(KeyCode::Right, M::NONE);
    let at = drive.app.value_popup.as_ref().unwrap().at;
    assert!(drive.app.smc_knob_strip(at).unwrap().value.is_none());
}

/// Scenario "The panel opens on the screen actions" (command-panel).
#[test]
fn the_panel_opens_on_the_screen_actions() {
    let sound = Arc::new(Sample::mono(vec![0.2; 48_000]));
    let mut drive = reach_drive(panel::View::Chop, sound);
    cell(&mut drive, 64);
    let panel = drive.app.panel_snapshot().unwrap();
    assert_eq!((panel.page, panel.view), (1, "CHOP"));
    assert!(!panel.target);
    assert_eq!(panel.labels[8].as_deref(), Some("cut here"));
    assert_eq!(panel.labels[4].as_deref(), Some("put on pads"));
}

/// Scenario "A screen hit opens that screen with its actions" (command-panel).
#[test]
fn a_screen_hit_opens_that_screen_with_its_actions() {
    let mut drive = smc_drive();
    cell(&mut drive, 64);
    list_button(&mut drive, false);
    assert_eq!(drive.app.panel.page(), 0);
    cell(&mut drive, panel_note(2));
    assert_eq!(drive.app.screen, Screen::Files);
    let panel = drive.app.panel_snapshot().unwrap();
    assert_eq!((panel.page, panel.view), (1, "FILES"));
    assert_eq!(panel.labels[9].as_deref(), Some("take file"));
    // A hit on the current screen also shows its actions.
    list_button(&mut drive, false);
    cell(&mut drive, panel_note(2));
    assert_eq!(drive.app.panel.page(), 1);
}

/// Scenario "Global actions live in the screens view" (command-panel).
#[test]
fn global_actions_live_in_the_screens_view() {
    for (cell, label) in [
        (9, Some("save")),
        (10, Some("open")),
        (11, None),
        (12, Some("exit")),
        (8, Some("help")),
    ] {
        let mut screens = panel::State::default();
        screens.move_view(-1);
        assert_eq!(screens.label(panel::View::Pads, cell), label);
    }
    for view in ALL_VIEWS {
        {
            let mut route = panel::State::default();
            for cell in 1..=12 {
                let choice = route.choice(view, cell);
                assert!(
                    !matches!(
                        choice,
                        panel::Choice::Exit
                            | panel::Choice::Action(
                                Action::SessionSave
                                    | Action::ProjectPick
                                    | Action::LearnToggle
                                    | Action::HelpToggle
                                    | Action::TransportToggle
                            )
                    ),
                    "{view:?} cell {cell}: {choice:?}"
                );
            }
        }
    }
}

/// Scenario "Assign shows order in the Chop actions" (command-panel).
#[test]
fn assign_shows_order_in_the_chop_actions() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut keyboard = reach_drive(panel::View::Chop, sound.clone());
    let mut controller = reach_drive(panel::View::Chop, sound);
    keyboard.tap(KeyCode::Char('p'), M::NONE);
    keyboard.tap(KeyCode::Tab, M::NONE);
    cell(&mut controller, 64);
    let assign = slot_of(panel::View::Chop, "put on pads");
    let index = usize::from(assign - 1);
    assert_eq!(
        controller.app.panel_snapshot().unwrap().labels[index].as_deref(),
        Some("put on pads")
    );
    cell(&mut controller, panel_note(assign)); // Assign opens the target view.
    cell(&mut controller, panel_note(15)); // The menu gives the Chop actions.
    assert_eq!(
        controller.app.panel_snapshot().unwrap().labels[index].as_deref(),
        Some("by hand")
    );
    cell(&mut controller, panel_note(assign));
    assert!(matches!(
        controller.app.plan,
        Some(chop::Plan::InOrder(None))
    ));
    assert_eq!(controller.app.plan, keyboard.app.plan);
}

/// Scenario "A visible panel takes LEFT and RIGHT" (controller-reach).
#[test]
fn a_visible_panel_takes_left_and_right() {
    let sound = Arc::new(Sample::mono(vec![0.2; 48_000]));
    let mut drive = reach_drive(panel::View::Chop, sound);
    let selected = drive.app.chop.as_ref().unwrap().selected;
    cell(&mut drive, 64);
    list_button(&mut drive, false);
    list_button(&mut drive, false);
    assert_eq!(drive.app.panel.page(), 0);
    list_button(&mut drive, true);
    list_button(&mut drive, true);
    assert_eq!(drive.app.panel.page(), 1);
    assert_eq!(drive.app.pads.bank, 0);
    assert_eq!(drive.app.chop.as_ref().unwrap().selected, selected);
    assert_eq!(drive.app.screen, Screen::Chop);
}

/// Scenario "Pattern taps use the beat interval" (transport).
#[test]
fn pattern_taps_use_the_beat_interval() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut controller = reach_drive(panel::View::Pattern, sound.clone());
    cell(&mut controller, panel_note(1));
    let slot = slot_of(panel::View::Pattern, "tap tempo");
    assert_eq!(
        controller.app.panel.label(panel::View::Pattern, slot),
        Some("tap tempo")
    );
    let note = panel_note(slot);
    let start = Instant::now();
    let tap = |drive: &mut Drive, offset: u64| {
        let at = start + std::time::Duration::from_millis(offset);
        drive
            .midi()
            .recorded_bytes_at(SMC, &[0x99, note, 100], &drive.project, at);
        drive
            .midi()
            .recorded_bytes_at(SMC, &[0x89, note, 0], &drive.project, at);
        drive.frame()
    };
    let initial = controller.app.transport.bpm;
    tap(&mut controller, 0);
    assert_eq!(controller.app.transport.bpm, initial);
    let frame = tap(&mut controller, 400);
    assert_eq!(controller.app.transport.bpm, 150.0);
    assert!(
        frame
            .commands()
            .into_iter()
            .any(|command| matches!(command, Command::SetTempo { bpm } if *bpm == 150.0))
    );
    tap(&mut controller, 900);
    assert_eq!(controller.app.transport.bpm, 133.3);
    tap(&mut controller, 1_400);
    assert_eq!(controller.app.transport.bpm, 128.6);
    tap(&mut controller, 1_900);
    assert_eq!(controller.app.transport.bpm, 120.0);
    tap(&mut controller, 4_100);
    assert_eq!(controller.app.transport.bpm, 120.0);
    tap(&mut controller, 4_700);
    assert_eq!(controller.app.transport.bpm, 100.0);

    let mut keyboard = reach_drive(panel::View::Pattern, sound);
    let initial = keyboard.app.transport.bpm;
    keyboard.tap(KeyCode::Char('b'), M::ALT);
    assert!(!keyboard.app.transport.taps.is_empty());
    assert_eq!(keyboard.app.transport.bpm, initial);
    keyboard.tap(KeyCode::F(2), M::NONE);
    assert!(keyboard.app.transport.taps.is_empty());
    keyboard.tap(KeyCode::Char('b'), M::ALT);
    assert!(keyboard.app.transport.taps.is_empty());
    assert_eq!(keyboard.app.transport.bpm, initial);
}

const SMC_PRIVATE: &str = "SINCO SMC-PAD-Private 28:0";

/// Scenario "A mirrored hit keeps controller words" (screen-help).
#[test]
fn a_mirrored_hit_keeps_controller_words() {
    for ports in [[SMC, SMC_PRIVATE], [SMC_PRIVATE, SMC]] {
        let mut drive = hold_drive();
        drive.midi().recorded_connect(SMC_PRIVATE);
        drive.frame();
        drive.tap(KeyCode::Down, M::NONE);
        for status in [0x99, 0x89] {
            for port in ports {
                drive.bytes(port, &[status, 48, 100]);
            }
            drive.frame();
            assert!(drive.app.input_sources.last_is_smc(), "{ports:?}");
        }
        drive.app.smc_knob_strip(Instant::now());
        assert!(tui::pads::smc_words(), "{ports:?}");
    }
}

/// Scenario "A mirrored message after a key keeps keyboard words" (screen-help).
#[test]
fn a_mirrored_message_after_a_key_keeps_keyboard_words() {
    let mut drive = hold_drive();
    drive.midi().recorded_connect(SMC_PRIVATE);
    drive.frame();
    drive.tap(KeyCode::Down, M::NONE);
    drive.bytes(SMC_PRIVATE, &[0x99, 48, 100]);
    drive.bytes(SMC_PRIVATE, &[0x89, 48, 0]);
    drive.frame();
    assert!(!drive.app.input_sources.last_is_smc());
    drive.app.smc_knob_strip(Instant::now());
    assert!(!tui::pads::smc_words());
}

/// Scenario "Marking after SMC-PAD input names PAD BANK and pad 1" (screen-help).
#[test]
fn marking_after_smc_pad_input_names_pad_bank_and_pad_1() {
    let mut drive = hold_drive();
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 4_800])), "kick");
    let mark = panel_note(slot_of(panel::View::Pads, "mark pad"));
    hold(&mut drive, true);
    cell(&mut drive, mark);
    hold(&mut drive, false);
    assert!(drive.app.pads.marking, "the mark cell turns marking on");
    let line = drive.app.mode_line(Instant::now()).unwrap();
    assert!(
        line.contains("PAD BANK + 1 drop") && !line.contains("Esc"),
        "{line}"
    );
    // a mark, marking off: the badge names the same drop control
    drive.bytes(SMC, &[0x99, 48, 100]);
    drive.bytes(SMC, &[0x89, 48, 0]);
    drive.frame();
    hold(&mut drive, true);
    cell(&mut drive, mark);
    hold(&mut drive, false);
    let badges: Vec<String> = drive
        .app
        .title_badges()
        .into_iter()
        .map(|badge| badge.normal)
        .collect();
    assert!(
        badges
            .iter()
            .any(|b| b.contains("MARKED · PAD BANK + 1 drops")),
        "{badges:?}"
    );
    // after a key the words are the keys again
    drive.tap(KeyCode::Down, M::NONE);
    let badges: Vec<String> = drive
        .app
        .title_badges()
        .into_iter()
        .map(|badge| badge.normal)
        .collect();
    assert!(
        badges.iter().any(|b| b.contains("MARKED · Esc drops")),
        "{badges:?}"
    );
}

/// The F12 MIDI tab with a verified SMC-PAD and the row `pick` selected.
fn midi_tab_drive(pick: fn(&learn::MidiListRow) -> bool) -> Drive {
    let mut drive = hold_drive();
    drive.app.screen = Screen::Settings;
    drive.app.settings_tab = 2;
    if !drive.app.midi_list().iter().any(pick) {
        drive.app.midi_open.extend(input::Group::ALL);
    }
    drive.app.midi_row = drive.app.midi_list().iter().position(pick).unwrap();
    drive.app.publish_midi(&drive.shared);
    drive
}

/// Scenario "Choose opens a group but starts no learning" (midi-learn).
#[test]
fn choose_opens_a_group_but_starts_no_learning() {
    let mut drive = midi_tab_drive(|row| matches!(row, learn::MidiListRow::Group { .. }));
    let rows = drive.app.midi_list().len();
    labelled_cell(&mut drive, panel::View::Settings, "choose");
    assert!(drive.app.midi_list().len() > rows, "the group opens");
    drive.app.midi_row = drive
        .app
        .midi_list()
        .iter()
        .position(|row| matches!(row, learn::MidiListRow::Action { .. }))
        .unwrap();
    labelled_cell(&mut drive, panel::View::Settings, "choose");
    assert!(drive.app.learning.is_none());
    assert_eq!(
        drive.last_notice().text,
        "Bindings: keyboard only (F10, Enter)"
    );
    // Enter from the keyboard still starts learning.
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(drive.app.learning.is_some());
}

/// Scenario "Choose on a binding row explains the refusal" (controller-reach).
#[test]
fn choose_on_a_binding_row_explains_the_refusal() {
    for pick in [
        (|row: &learn::MidiListRow| matches!(row, learn::MidiListRow::Reset { .. }))
            as fn(&learn::MidiListRow) -> bool,
        |row| matches!(row, learn::MidiListRow::Set { .. }),
    ] {
        let mut drive = midi_tab_drive(pick);
        labelled_cell(&mut drive, panel::View::Settings, "choose");
        assert!(drive.app.ask.is_none(), "no reset question");
        assert!(drive.app.midi_run.is_none() && drive.app.learning.is_none());
        assert_eq!(
            drive.last_notice().text,
            "Bindings: keyboard only (F10, Enter)"
        );
    }
}

/// Scenario "Learn names the replaced panel cell" (controller-reach).
#[test]
fn learn_names_the_replaced_panel_cell() {
    // With the PAD BANK hold, a cell is a hold of the playing note and learning keeps it;
    // with the hold off, the PAD BANK layer sends its own notes and a binding takes them.
    let mut drive = smc_drive();
    drive.tap(KeyCode::F(10), M::NONE);
    drive.tap(KeyCode::Char('h'), M::NONE);
    // device pad 6 on the PAD BANK layer is panel cell 10, "copy pad" on Pads
    drive.bytes(SMC, &[0x99, panel_note(10), 100]);
    drive.bytes(SMC, &[0x89, panel_note(10), 0]);
    drive.frame();
    let view = drive.app.learning_view(&drive.keymap).unwrap();
    let text = format!("{} | {} | {}", view.context, view.detail, view.controls);
    assert!(text.contains("replaces panel cell P6 copy pad"), "{text}");
}

/// Scenario "The trim knob names its edge" (screen-layout).
#[test]
fn the_trim_knob_names_its_edge() {
    let mut drive = reach_drive(panel::View::Chop, Arc::new(Sample::mono(vec![0.2; 48_000])));
    turn(&mut drive, 34, 65);
    let strip = drive.app.smc_knob_strip(Instant::now()).unwrap();
    let labels: Vec<&str> = strip.roles[0].iter().map(|(label, _)| *label).collect();
    assert!(
        labels.contains(&"trim<") && labels.contains(&"trim>"),
        "{labels:?}"
    );
    assert!(labels.contains(&"hits"), "{labels:?}");
    let detail = drive
        .app
        .smc_knob_detail("e5", Some("chop.region_start_pos"));
    assert_eq!(detail.as_deref(), Some("B1 K5 · trim start"));
}

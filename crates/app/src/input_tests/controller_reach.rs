//! Picker and Settings routes through the shipped keyboard map and raw SMC-PAD profile.

use super::*;

#[path = "panel_targets.rs"]
mod panel_targets;

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
            assert!(
                fixed || role_ok || variant_role_ok || bank_ok || tempo_ok || direct_editor,
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

/// Scenario "Fast encoder turns keep acceleration" (controller-reach).
#[test]
fn fast_encoder_turns_keep_acceleration() {
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
    assert_eq!(controller.app.chop.as_ref().unwrap().threshold_db, -3.0);
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
    let view = panel::View::Pads;
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut keyboard = reach_drive(view, sound.clone());
    let mut controller = reach_drive(view, sound);
    let at = Instant::now();
    for (index, (start, code, value, selected)) in [
        (0, KeyCode::Left, 63, 0),
        (0, KeyCode::Right, 65, 1),
        (3, KeyCode::Right, 65, 3),
        (4, KeyCode::Left, 63, 4),
        (7, KeyCode::Left, 63, 6),
        (15, KeyCode::Right, 65, 15),
    ]
    .into_iter()
    .enumerate()
    {
        for drive in [&mut keyboard, &mut controller] {
            drive.app.pads.selected = start;
            drive.app.splash = true;
            drive.app.publish_midi(&drive.shared);
        }
        let before = keyboard.seen();
        let key_frame = keyboard.tap(code, M::ALT);
        controller.midi().recorded_bytes_at(
            SMC,
            &[0xB0, 30, value],
            &controller.project,
            at + std::time::Duration::from_millis(100 * index as u64),
        );
        let button_frame = controller.frame();
        assert_eq!(keyboard.app.pads.selected, selected, "{view:?} {start}");
        assert_eq!(keyboard.seen(), controller.seen(), "{view:?} {start}");
        assert_eq!(keyboard.seen().pads, before.pads, "{view:?} pad values");
        assert!(key_frame.commands().is_empty(), "{view:?} key hit");
        assert!(button_frame.commands().is_empty(), "{view:?} button hit");
        assert!(
            keyboard.app.splash && controller.app.splash,
            "{view:?} splash"
        );
        keyboard.key(code, M::ALT, KeyEventKind::Repeat);
        keyboard.key(code, M::ALT, KeyEventKind::Release);
        assert!(keyboard.frame().commands().is_empty());
        assert_eq!(keyboard.app.pads.selected, selected, "{view:?} repeat");
    }

    for drive in [&mut keyboard, &mut controller] {
        drive.app.pads.selected = 0;
        drive.app.pads.gate[0] = true;
        drive.app.publish_midi(&drive.shared);
    }
    keyboard.key(KeyCode::Char('1'), M::NONE, KeyEventKind::Press);
    keyboard.frame();
    controller.bytes(SMC, &[0x99, 48, 80]);
    controller.frame();
    assert!(keyboard.app.pads.held[0] && controller.app.pads.held[0]);
    assert!(keyboard.tap(KeyCode::Right, M::ALT).commands().is_empty());
    controller.midi().recorded_bytes_at(
        SMC,
        &[0xB0, 30, 65],
        &controller.project,
        Instant::now() + std::time::Duration::from_millis(1_000),
    );
    assert!(controller.frame().commands().is_empty());
    assert_eq!(keyboard.app.pads.selected, 1);
    assert_eq!(keyboard.seen(), controller.seen());
    keyboard.key(KeyCode::Char('1'), M::NONE, KeyEventKind::Release);
    let key_release = keyboard.frame();
    controller.bytes(SMC, &[0x89, 48, 0]);
    controller.frame();
    assert!(
        key_release
            .commands()
            .iter()
            .any(|command| matches!(command, Command::PadRelease { pad: 0 }))
    );
    assert_eq!(keyboard.seen(), controller.seen());
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

/// Scenario "Next selects a later context page" (controller-reach).
#[test]
fn next_selects_a_later_context_page() {
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
    for _ in 1..4 {
        cell(&mut controller, panel_note(11));
    }
    assert_eq!(
        controller.app.panel.page(),
        4,
        "{:?}",
        controller.app.input_log
    );
    assert_eq!(
        controller.app.panel_snapshot().unwrap().labels[1].as_deref(),
        Some("grid +")
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
    cell(&mut controller, panel_note(8));
    cell(&mut controller, panel_note(10));
    keyboard.tap(KeyCode::F(1), M::NONE);
    keyboard.tap(KeyCode::Tab, M::NONE);
    keyboard.tap(KeyCode::Tab, M::NONE);
    assert!(controller.app.help_open);
    assert_eq!(
        controller.app.help_overlay.page,
        keyboard.app.help_overlay.page
    );

    for _ in 0..3 {
        cell(&mut controller, 63);
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

/// Scenario "The main panel gives Help one context route" (command-panel).
#[test]
fn the_main_panel_gives_help_one_context_route() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    for view in ALL_VIEWS {
        let mut drive = reach_drive(view, sound.clone());
        let screen = drive.app.screen;
        cell(&mut drive, panel_note(1));
        let main = drive.app.panel_snapshot().unwrap();
        assert_eq!(main.labels[5].as_deref(), Some("mixer"));
        assert_eq!(main.labels[6].as_deref(), Some("setup"));
        cell(&mut drive, panel_note(6));
        assert_eq!(drive.app.screen, screen);
        assert!(!drive.app.help_open);
        cell(&mut drive, panel_note(8));
        assert_eq!(drive.app.panel.label(view, 10), Some("help"));
        cell(&mut drive, panel_note(10));
        assert!(drive.app.help_open, "{view:?}");
        assert_eq!(drive.app.screen, screen);
    }

    let mut drive = smc_drive();
    cell(&mut drive, panel_note(1));
    cell(&mut drive, panel_note(7));
    assert_eq!(drive.app.screen, Screen::Settings);
}

/// Scenario "Exit needs a second SMC-PAD press" (controller-reach).
#[test]
fn exit_needs_a_second_smc_press() {
    let mut drive = smc_drive();
    drive.app.session_dirty = Some(Instant::now());
    cell(&mut drive, 64);
    cell(&mut drive, 63);
    cell(&mut drive, panel_note(11));
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
    cell(&mut drive, panel_note(11));
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

    let mut shortcut = smc_drive();
    shortcut.app.set_smc_pad_bank(true);
    pad_bank_request(&mut shortcut, SMC);
    let mut shortcut_presses = 1;
    for slot in [8, 1] {
        cell(&mut shortcut, panel_note(slot));
        shortcut_presses += 1;
    }
    assert!(shortcut.app.pads.gate[0]);
    assert_eq!(shortcut_presses, 3);

    let mut screen = smc_drive();
    screen.app.set_smc_pad_bank(true);
    pad_bank_request(&mut screen, SMC);
    cell(&mut screen, panel_note(2));
    assert_eq!(screen.app.screen, Screen::Files);

    let mut later = smc_drive();
    cell(&mut later, panel_note(1));
    let mut visible_presses = 0;
    for (slot, expected_page) in [(8, 1), (11, 2), (11, 3), (11, 4)] {
        cell(&mut later, panel_note(slot));
        visible_presses += 1;
        assert_eq!(later.app.panel.page(), expected_page);
    }
    assert_eq!(later.app.panel.label(panel::View::Pads, 4), Some("step"));
    assert_eq!(visible_presses + 1, 5); // One more hit reaches the action.
    assert_eq!(visible_presses + 3, 7); // PAD BANK, reveal and the action.

    let mut knob = smc_drive();
    let db = knob.app.master_db;
    turn(&mut knob, 37, 63);
    assert_eq!(knob.app.master_db, db - 1, "{:?}", knob.app.input_log);
    assert_eq!(knob.app.pads.bank, 0);
}

/// Scenario "Added cells reach their current actions" (controller-reach).
#[test]
fn added_cells_reach_their_current_actions() {
    let mut keyboard = smc_drive();
    let mut controller = smc_drive();
    keyboard.tap(KeyCode::PageDown, M::NONE);
    list_button(&mut controller, true);
    assert_eq!(controller.app.pads.bank, keyboard.app.pads.bank);

    keyboard.tap(KeyCode::Char('n'), M::ALT);
    cell(&mut controller, 64);
    cell(&mut controller, 63);
    cell(&mut controller, panel_note(11));
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
    cell(&mut controller, panel_note(11));
    assert_eq!(controller.app.panel.page(), 2);
    cell(&mut controller, 66);
    assert_eq!(controller.app.pad_in_hand, keyboard.app.pad_in_hand);
    assert_eq!(controller.app.screen, keyboard.app.screen);
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
    keyboard.tap(KeyCode::Up, M::CONTROL | M::SHIFT);
    timed_knob_two(&mut controller, 65, at);
    assert_eq!(controller.app.transport.bpm, keyboard.app.transport.bpm);
    assert!((controller.app.transport.bpm - bpm - 0.1).abs() < 0.0001);
    timed_knob_two(
        &mut controller,
        65,
        at + std::time::Duration::from_millis(10),
    );
    assert!((controller.app.transport.bpm - bpm - 2.1).abs() < 0.0001);
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

            cell(&mut controller, 64);
            cell(&mut controller, 63);
            cell(&mut controller, panel_note(11));
            assert_eq!(controller.app.panel.page(), 2);
            let frame = cell(&mut controller, 65);
            assert_eq!(
                frame
                    .commands()
                    .iter()
                    .filter(|command| matches!(command, Command::TransportToggle))
                    .count(),
                1,
                "{screen:?} page 2 trnspt"
            );

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
        cell(&mut controller, panel_note(11));
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
                cell(&mut controller, panel_note(11));
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
    cell(&mut drive, panel_note(15)); // Open the command pages from the target view.
    cell(&mut drive, 63);
    cell(&mut drive, panel_note(11));
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

fn panel_note(slot: u8) -> u8 {
    [
        64, 65, 66, 67, 60, 61, 62, 63, 56, 57, 58, 59, 52, 53, 54, 55,
    ][usize::from(slot - 1)]
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
    let mut drive = smc_drive();
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
/// The shipped panel has 225 cells, so this production-input matrix belongs to the slow group.
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
        for page in 0..config.pages(view) {
            for slot in 1..=12 {
                let Some(cell_info) = config.cell(view, page, slot) else {
                    continue;
                };
                let action = cell_info.action;
                if view == View::ChopEdit && action == Action::ChopAssignOrder {
                    let mut controller =
                        reach_drive(view, Arc::new(Sample::mono(vec![0.2; 4_800])));
                    cell(&mut controller, 64);
                    cell(&mut controller, 63);
                    for _ in 1..4 {
                        cell(&mut controller, panel_note(11));
                    }
                    assert_eq!(controller.app.panel.page(), 4);
                    let before = controller.seen();
                    let frame = cell(&mut controller, panel_note(slot));
                    assert_eq!(
                        controller.seen(),
                        before,
                        "Chop edit order must stay inactive"
                    );
                    assert!(frame.commands().is_empty());
                    special += 1;
                    continue;
                }
                if matches!(
                    action,
                    Action::Quit
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
                    .unwrap_or_else(|| {
                        panic!("{view:?} page {page} cell {slot}: no key for {action:?}")
                    });
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
                if page > 0 {
                    cell(&mut controller, 63);
                    for _ in 1..page {
                        cell(&mut controller, panel_note(11));
                    }
                }
                assert_eq!(
                    controller.app.panel.page(),
                    page,
                    "{view:?} {action:?}: {:?}",
                    controller.app.input_log
                );
                if action == Action::HelpToggle {
                    keyboard.tap(KeyCode::Tab, M::NONE);
                    keyboard.tap(KeyCode::Tab, M::NONE);
                }
                let controller_frame = cell(&mut controller, panel_note(slot));
                assert_eq!(
                    reach_result(&controller, &controller_frame),
                    reach_result(&keyboard, &keyboard_frame),
                    "{view:?} page {page} cell {slot} {action:?} key {}",
                    key.0
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked + special, 176);

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
        cell(&mut controller, 63);
        cell(&mut controller, panel_note(11));
        cell(&mut controller, 57);
        assert!(
            matches!(controller.app.ask, Some(Ask::Exit { .. })),
            "{view:?}"
        );
        assert!(!controller.app.quit_requested, "{view:?}");
        cell(&mut controller, 63);
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
    keyboard.tap(KeyCode::Enter, M::NONE);
    cell(&mut controller, 64);
    cell(&mut controller, 63);
    cell(&mut controller, 61);
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
    cell(&mut controller, 63);
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
    let before = edit.app.pads.param;
    turn(&mut edit, 30, 65);
    assert_ne!(edit.app.pads.param, before);
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
            let chop = drive.app.chop.as_mut().unwrap();
            chop.snap = chop::Snap::Cuts;
            assert!(chop.select_slice(1));
        }
        if view == panel::View::ChopEdit {
            keyboard.tap(KeyCode::Esc, M::NONE);
        }
        keyboard.tap(KeyCode::Char('l'), M::NONE);
        list_button(&mut chop, true);
        assert_eq!(chop.app.chop.as_ref().unwrap().selected, 2);
        assert_eq!(
            chop.app.chop.as_ref().unwrap().selected,
            keyboard.app.chop.as_ref().unwrap().selected
        );
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
    for view in [
        panel::View::Pads,
        panel::View::PadEdit,
        panel::View::Pattern,
    ] {
        for (cc, param, key, value) in [
            (32, pads::Param::Volume, KeyCode::Left, 63),
            (34, pads::Param::Pitch, KeyCode::Right, 65),
            (32, pads::Param::Cutoff, KeyCode::Left, 63),
            (33, pads::Param::Decay, KeyCode::Right, 65),
            (38, pads::Param::Volume, KeyCode::Left, 63),
            (40, pads::Param::Pitch, KeyCode::Right, 65),
            (39, pads::Param::Cutoff, KeyCode::Left, 63),
            (41, pads::Param::Decay, KeyCode::Right, 65),
        ] {
            if cc < 38 && view == panel::View::Pattern {
                continue;
            }
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

/// Scenario "Pad inspector and editor share knob positions" (controller-reach).
#[test]
fn pad_inspector_and_editor_share_knob_positions() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    for view in [panel::View::Pads, panel::View::PadEdit] {
        let mut drive = reach_drive(view, sound.clone());
        let selected = drive.app.pads.selected_abs();
        let pad = drive.app.pads.snapshot(selected);
        turn(&mut drive, 33, 65);
        assert!(
            drive.app.pads.snapshot(selected) == pad,
            "{view:?} Tone pan"
        );
        turn(&mut drive, 32, 63);
        assert_eq!(drive.app.pads.volume_db[selected], -1);
        drive.app.pads.param = pads::Param::Attack;
        let isolated = Instant::now() + std::time::Duration::from_millis(100);
        for cc in [32, 33] {
            drive
                .midi()
                .recorded_bytes_at(SMC, &[0xB0, cc, 65], &drive.project, isolated);
            drive.frame();
        }
        assert_eq!(drive.app.pads.attack_ms[selected], 1);
        assert_eq!(drive.app.pads.decay_ms[selected], 1);
        assert_eq!(drive.app.pads.param, pads::Param::Attack);
    }

    let mut fx = reach_drive(panel::View::Fx, sound);
    let first = fx.app.pads.fx[0];
    let second = fx.app.pads.fx[1];
    list_button(&mut fx, true);
    turn(&mut fx, 31, 65);
    assert_eq!(fx.app.pads.fx[0], first);
    assert_ne!(fx.app.pads.fx[1].kind, second.kind);
}

/// Scenario "Direct page controls replace panel steps" (command-panel).
#[test]
fn direct_page_controls_replace_panel_steps() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive(panel::View::PadEdit, sound);
    let pad = drive.app.pads.selected_abs();
    cell(&mut drive, 64);
    let snapshot = drive.app.panel_snapshot().unwrap();
    assert!(snapshot.labels[8..12].iter().all(Option::is_none));
    cell(&mut drive, panel_note(9));
    assert_eq!(drive.app.pads.page().0, "TONE");
    list_button(&mut drive, true);
    assert_eq!(drive.app.pads.page().0, "ENV");
    let parameter = drive.app.pads.param;
    turn(&mut drive, 30, 65);
    assert_ne!(drive.app.pads.param, parameter);
    turn(&mut drive, 32, 65);
    assert_eq!(drive.app.pads.attack_ms[pad], 1);
    for name in ["edit.param_next", "edit.page_next", "edit.value_inc"] {
        assert!(input::ACTIONS.iter().any(|action| action.name == name));
    }
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
    cell(&mut drive, 63);
    assert_eq!(drive.app.panel.page(), 1);
    turn(&mut drive, 35, 63);
    assert_eq!(drive.app.panel.page(), 1);
    assert!(drive.app.chop.as_ref().unwrap().range.end < 96_000);
    cell(&mut drive, panel_note(11));
    assert_eq!(drive.app.panel.page(), 2);
}

/// Scenario "Bank 2 supplies the same live pad controls on every screen" (controller-reach).
#[test]
fn bank_2_supplies_the_same_live_pad_controls_on_every_screen() {
    let sound = Arc::new(Sample::mono(vec![0.2; 48_000]));
    for view in ALL_VIEWS {
        let mut drive = reach_drive(view, sound.clone());
        drive.load_pad(1, sound.clone(), "other");
        drive.app.pads.selected = 0;
        drive.app.pads.marked.insert(1);
        drive.app.pads.cutoff_hz[0] = 1_200.0;
        drive.app.pads.decay_ms[0] = 50;
        drive.app.pads.sends[0] = [10, 20];
        let untouched = drive.app.pads.snapshot(1);
        let volume = drive.app.pads.volume_db[0];
        let pitch = drive.app.pads.pitch[0];
        let page = drive.app.pads.param;
        cell(&mut drive, 64);
        assert!(drive.app.panel.visible());

        turn(&mut drive, 38, 63);
        assert_eq!(drive.app.pads.volume_db[0], volume - 1, "{view:?}");
        turn(&mut drive, 39, 65);
        assert!(drive.app.pads.cutoff_hz[0] > 1_200.0, "{view:?}");
        turn(&mut drive, 40, 65);
        assert_eq!(drive.app.pads.pitch[0], pitch + 1, "{view:?}");
        turn(&mut drive, 41, 65);
        assert_eq!(drive.app.pads.decay_ms[0], 51, "{view:?}");
        turn(&mut drive, 42, 65);
        assert_eq!(drive.app.pads.sends[0], [11, 20], "{view:?}");
        turn(&mut drive, 43, 65);
        assert_eq!(drive.app.pads.sends[0], [11, 21], "{view:?}");
        let selected = drive.app.pads.snapshot(0);
        turn(&mut drive, 44, 65);
        assert!(drive.app.pads.snapshot(0) == selected, "{view:?}");
        assert!(drive.app.pads.snapshot(1) == untouched, "{view:?}");
        assert_eq!(drive.app.pads.param, page);
        assert_eq!(drive.app.panel.page(), 0);

        drive.app.ask = Some(Ask::Exit { lines: vec![] });
        drive.app.publish_midi(&drive.shared);
        turn(&mut drive, 38, 63);
        assert!(drive.app.pads.snapshot(0) == selected, "{view:?}");
        assert!(drive.app.ask.is_some());
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
fn smc_chop_zoom_retains_acceleration_and_reverses_at_view_limits() {
    let sound = Arc::new(Sample::mono(vec![0.2; 96_000]));
    let mut drive = reach_drive(panel::View::Chop, sound);
    let at = Instant::now();
    timed_knob_two(&mut drive, 63, at);
    assert_eq!(chop_view_len(&drive), 96_000);

    timed_knob_two(&mut drive, 65, at + std::time::Duration::from_millis(100));
    let after_slow_turn = chop_view_len(&drive);
    timed_knob_two(&mut drive, 65, at + std::time::Duration::from_millis(110));
    let after_fast_turn = chop_view_len(&drive);
    let expected = (after_slow_turn as f64 / 2.0_f64.sqrt()).round() as usize;
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
    keyboard.tap(KeyCode::Left, M::NONE);
    turn(&mut controller, 32, 63);
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
    drive.midi().recorded_pad_bank(port, Instant::now());
    drive.frame()
}

fn pad_bank_row(drive: &Drive) -> Option<usize> {
    drive
        .app
        .midi_list()
        .iter()
        .position(|row| matches!(row, MidiListRow::SmcPadBank))
}

/// Scenario "PAD BANK starts open and preserves a saved Off choice" (controller-reach).
/// Scenario "PAD BANK setting saves and defaults to off" (controller-reach).
#[test]
fn pad_bank_starts_open_and_preserves_a_saved_off_choice() {
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
    let mut controller = smc_drive();
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

/// Scenario "PAD BANK opens the panel once before a pad hit" (controller-reach).
#[test]
fn pad_bank_opens_the_panel_once_before_a_pad_hit() {
    let mut drive = smc_drive();
    drive.app.set_smc_pad_bank(true);
    let reveal = pad_bank_request(&mut drive, SMC);
    assert_eq!(drive.app.panel.owner(), Some(SMC));
    assert_eq!(drive.app.panel.page(), 0);
    assert!(reveal.commands().is_empty());
    assert_eq!(drive.app.screen, Screen::Pads);
    cell(&mut drive, panel_note(2));
    assert_eq!(drive.app.screen, Screen::Files);
    cell(&mut drive, panel_note(8));
    assert_eq!(drive.app.panel.page(), 1);
    const SECOND: &str = "SINCO SMC-PAD-Master 32:1";
    drive.midi().recorded_connect(SECOND);
    drive.frame();
    assert!(pad_bank_request(&mut drive, SECOND).commands().is_empty());
    assert_eq!(drive.app.panel.owner(), Some(SMC));
    assert_eq!(drive.app.panel.page(), 1);

    assert!(pad_bank_request(&mut drive, SMC).commands().is_empty());
    assert!(!drive.app.panel.visible());
    assert_eq!(drive.app.screen, Screen::Files);

    assert!(pad_bank_request(&mut drive, SMC).commands().is_empty());
    assert_eq!(drive.app.panel.owner(), Some(SMC));
    assert_eq!(drive.app.panel.page(), 0);
    assert_eq!(drive.app.screen, Screen::Files);
}

/// Scenario "PAD BANK respects input overlays" (controller-reach).
#[test]
fn pad_bank_respects_input_overlays() {
    let mut drive = smc_drive();
    drive.app.set_smc_pad_bank(true);
    drive.app.ask = Some(Ask::ResetPads {
        pads: vec![1],
        lines: vec!["Reset pad 2?".into()],
    });
    pad_bank_request(&mut drive, SMC);
    assert!(drive.app.ask.is_some());
    assert!(!drive.app.panel.visible());
    drive.app.ask = None;

    drive.app.help_open = true;
    pad_bank_request(&mut drive, SMC);
    assert!(!drive.app.panel.visible());
    drive.app.help_open = false;

    drive.app.pad_naming = Some((0, "Name".into()));
    pad_bank_request(&mut drive, SMC);
    assert!(!drive.app.panel.visible());
    drive.app.pad_naming = None;

    drive.app.project_picker = Some(ProjectPicker {
        entries: vec!["A".into()],
        selected: 0,
        naming: None,
    });
    pad_bank_request(&mut drive, SMC);
    assert!(!drive.app.panel.visible());
    assert_eq!(drive.app.project_picker.take().unwrap().selected, 0);

    drive.app.learning = Some(learn::Learning::new(action_info("transport.toggle")));
    pad_bank_request(&mut drive, SMC);
    assert!(!drive.app.panel.visible());
    assert!(drive.app.learning.as_ref().unwrap().pending.is_none());
}

/// Scenario "Late PAD BANK status keeps both pad routes" (controller-reach).
#[test]
fn late_pad_bank_status_keeps_both_pad_routes() {
    let mut drive = smc_drive();
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 256])), "gate");
    drive.app.set_smc_pad_bank(true);
    let requested_at = Instant::now();
    cell(&mut drive, panel_note(2));
    assert!(drive.app.panel.visible());
    assert_eq!(drive.app.screen, Screen::Pads);
    cell(&mut drive, panel_note(8));
    assert_eq!(drive.app.panel.page(), 1);
    drive.midi().recorded_pad_bank(SMC, requested_at);
    drive.frame();
    assert_eq!(drive.app.panel.page(), 1);
    assert_eq!(drive.app.panel.owner(), Some(SMC));

    let requested_at = Instant::now();
    drive.bytes(SMC, &[0x99, 48, 100]);
    drive.midi().recorded_pad_bank(SMC, requested_at);
    drive.frame();
    assert!(!drive.app.panel.visible());
    assert!(drive.app.pads.held[0]);
    drive.bytes(SMC, &[0x89, 48, 0]);
    drive.frame();
    assert!(!drive.app.pads.held[0]);
}

/// Scenario "PAD BANK ignores invalid or disconnected sources" (controller-reach).
#[test]
fn pad_bank_ignores_invalid_or_disconnected_sources() {
    let mut drive = smc_drive();
    drive.app.set_smc_pad_bank(true);
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
    drive.midi().recorded_pad_bank(SMC, old_request);
    drive.frame();
    assert!(!drive.app.panel.visible());
    pad_bank_request(&mut drive, SMC);
    assert!(drive.app.panel.visible());
}

/// Scenario "First-hit reveal survives a missing PAD BANK pulse" (controller-reach).
#[test]
fn first_hit_reveal_survives_a_missing_pad_bank_pulse() {
    for enabled in [false, true] {
        let mut drive = smc_drive();
        drive.app.set_smc_pad_bank(enabled);
        if !enabled {
            pad_bank_request(&mut drive, SMC);
            assert!(!drive.app.panel.visible());
        }
        cell(&mut drive, panel_note(2));
        assert!(drive.app.panel.visible());
        assert_eq!(drive.app.screen, Screen::Pads);
        cell(&mut drive, panel_note(2));
        assert_eq!(drive.app.screen, Screen::Files);
    }
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
    cell(&mut drive, 62);
    assert!(drive.app.ask.is_none());
    assert_eq!(drive.app.learned, before);
    assert_eq!(config::load_from(&path).config.midi, before);

    drive.app.user_path = Some(files.0.join("A"));
    select_layout_master(&mut drive);
    cell(&mut drive, 63);
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
    cell(&mut drive, 63);
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

/// Scenario "Pattern taps use the beat interval" (transport).
#[test]
fn pattern_taps_use_the_beat_interval() {
    let sound = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut controller = reach_drive(panel::View::Pattern, sound.clone());
    cell(&mut controller, panel_note(1));
    cell(&mut controller, panel_note(8));
    assert_eq!(
        controller.app.panel.label(panel::View::Pattern, 6),
        Some("tap")
    );
    let note = panel_note(6);
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

use super::*;

#[test]
fn existing_screen_bindings_survive_the_registry_change() {
    use input::{Group, midi::Key};
    let screens = [
        (Screen::Pads, KeyCode::F(2), "screen.pads"),
        (Screen::Settings, KeyCode::F(12), "screen.settings"),
        (Screen::Files, KeyCode::F(6), "screen.files"),
        (Screen::Chop, KeyCode::F(3), "screen.chop"),
        (Screen::Fx, KeyCode::F(4), "screen.fx"),
    ];
    let path = {
        let mut old = Drive::new();
        let maps = old.app.learned.get_mut("Recorded pads").unwrap();
        let global = maps.entry("global".into()).or_default();
        for key in [Key::Note as fn(u8) -> Key, Key::Cc] {
            for (index, (_, _, action)) in screens.iter().enumerate() {
                global.insert(key(90 + index as u8).to_string(), (*action).into());
            }
            for (number, action) in [
                (95, "bank.next"),
                (96, "bank.next"),
                (98, "help.toggle"),
                (100, "bank.prev"),
                (101, "ui.back"),
            ] {
                global.insert(key(number).to_string(), action.into());
            }
        }
        for (screen, _, _) in screens {
            let group = maps.entry(screen.key().into()).or_default();
            for key in [Key::Note as fn(u8) -> Key, Key::Cc] {
                group.insert(key(96).to_string(), "help.toggle".into());
                if matches!(screen, Screen::Pads | Screen::Chop) {
                    group.insert(
                        key(99).to_string(),
                        if screen == Screen::Pads {
                            "pad.edit"
                        } else {
                            "chop.edit"
                        }
                        .into(),
                    );
                }
            }
        }
        for group in [Group::PadsEdit, Group::ChopEdit] {
            let map = maps.entry(group.key().into()).or_default();
            for key in [Key::Note as fn(u8) -> Key, Key::Cc] {
                map.insert(key(96).to_string(), "edit.param_next".into());
            }
        }
        old.save_registry_bindings()
    };
    let saved = std::fs::read_to_string(&path).unwrap();
    let expected = config::load_from(&path).config.midi;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::with_saved_bindings(false, Some(&path));
        assert_eq!(drive.app.learned, expected);
        let mut overrides = String::from(
            "[global]\n\"Ctrl+H\" = \"bank.next\"\n\"Ctrl+J\" = \"help.toggle\"\n\
             [edit]\n\"Ctrl+H\" = \"edit.param_next\"\n",
        );
        for name in ["pads", "settings", "files", "chop", "fx"] {
            overrides.push_str(&format!(
                "[screens.{name}]\n\"Ctrl+H\" = \"help.toggle\"\n\"Ctrl+J\" = \"none\"\n"
            ));
        }
        assert!(drive.keymap.overlay(&overrides).unwrap().is_empty());
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 48_000])), 4);
        drive.link_chop_pad(0, 0);
        drive.link_chop_pad(4, 1);
        for (index, (screen, key, _)) in screens.iter().enumerate() {
            drive.registry_command(source, *key, M::NONE, 90 + index as u8);
            assert_eq!(drive.app.screen, *screen);
            assert!(!drive.app.edit);
            let bank = drive.app.pads.bank;
            drive.registry_command(source, KeyCode::Char('h'), M::CONTROL, 96);
            assert!(drive.app.help_open);
            assert_eq!(drive.app.pads.bank, bank);
            drive.registry_command(source, KeyCode::F(1), M::NONE, 98);
            assert!(!drive.app.help_open);
            // Keyboard `none` blocks its global binding. MIDI 97 stays unassigned.
            drive.registry_command(source, KeyCode::Char('j'), M::CONTROL, 97);
            assert!(!drive.app.help_open);
            if matches!(screen, Screen::Pads | Screen::Chop) {
                drive.registry_command(source, KeyCode::Enter, M::NONE, 99);
                assert!(drive.app.edit);
                let before = (drive.app.pads.param, drive.app.slice_end);
                drive.registry_command(source, KeyCode::Char('h'), M::CONTROL, 96);
                assert_ne!((drive.app.pads.param, drive.app.slice_end), before);
                assert!(!drive.app.help_open);
                drive.registry_command(source, KeyCode::Esc, M::NONE, 101);
                assert!(!drive.app.edit);
            }
            if matches!(source, Source::Button) {
                drive.bind_pad_button(4);
            }
            match source {
                Source::Keyboard => drive.key(KeyCode::Char('q'), M::NONE, KeyEventKind::Press),
                Source::Notes => drive.bytes(PAD_PORT, &[0x90, 40, 80]),
                Source::Button => drive.bytes(PAD_PORT, &[0xb0, 80, 80]),
            }
            let hit = drive.frame();
            assert!(hit.commands().into_iter().any(|command| matches!(command,
                Command::PadPress { pad: 4, velocity } if *velocity == 80.0 / 127.0)));
            drive.pad_release(source, 4);
            let hit = drive.pad_press(source, 4);
            assert_eq!(drive.app.pads.selected_abs(), 4);
            if matches!(source, Source::Keyboard) || *screen == Screen::Chop {
                assert!(hit.direct.is_empty());
            } else {
                assert!(
                    !hit.ui
                        .iter()
                        .any(|command| matches!(command, Command::PadPress { .. }))
                );
            }
            assert!(hit.commands().into_iter().any(|command| matches!(command,
                Command::PadPress { pad: 4, velocity } if *velocity == 1.0)));
            drive.registry_command(source, KeyCode::F(4), M::NONE, 94);
            drive.registry_command(source, KeyCode::PageDown, M::NONE, 95);
            assert_eq!(drive.app.pads.bank, 1);
            let release = drive.pad_release(source, 4);
            assert!(
                release
                    .commands()
                    .into_iter()
                    .any(|command| matches!(command, Command::PadRelease { pad: 4 }))
            );
            assert!(
                !release
                    .commands()
                    .into_iter()
                    .any(|command| matches!(command, Command::PadRelease { pad: 20 }))
            );
            drive.registry_command(source, KeyCode::PageUp, M::NONE, 100);
            assert_eq!(drive.app.pads.bank, 0);
        }
        assert_eq!(std::fs::read_to_string(&path).unwrap(), saved);
    }
}

#[test]
fn fx_controls_keep_their_priority_through_the_registry() {
    use input::{Group, midi::Key};
    let path = {
        let mut old = Drive::new();
        for (number, action) in [(80, "edit.param_next"), (81, "edit.value_inc_fine")] {
            learn::assign(
                &mut old.app.learned,
                "X-Session Pro",
                Group::Fx,
                Key::Cc(number),
                action,
            );
        }
        old.save_registry_bindings()
    };
    let mut results = Vec::new();
    for keyboard in [true, false] {
        let mut drive = Drive::with_saved_bindings(false, Some(&path));
        let command = |drive: &mut Drive, code, mods, bytes: [u8; 3]| {
            if keyboard {
                drive.tap(code, mods);
            } else {
                drive.bytes(XS_PORT, &bytes);
                drive.bytes(XS_PORT, &[bytes[0], bytes[1], 0]);
                drive.frame();
            }
            assert!(!drive.app.edit);
        };
        command(&mut drive, KeyCode::F(4), M::NONE, [0x90, 45, 127]);
        assert_eq!(drive.app.screen, Screen::Fx);
        command(&mut drive, KeyCode::Tab, M::NONE, [0x90, 43, 127]);
        assert_eq!((drive.app.fx_slot, drive.app.fx_param), (1, 0));
        command(&mut drive, KeyCode::Char('k'), M::NONE, [0xb0, 80, 127]);
        assert_eq!(drive.app.fx_param, 1);
        let before = drive.app.pads.fx_pos(pads::Param::Fx2A);
        command(&mut drive, KeyCode::Char('l'), M::NONE, [0xb0, 81, 127]);
        assert!(drive.app.pads.fx_pos(pads::Param::Fx2A) > before);
        results.push((
            drive.app.fx_slot,
            drive.app.fx_param,
            drive.app.pads.fx_pos(pads::Param::Fx2A),
        ));
    }
    assert_eq!(results[0], results[1]);

    // A saved pre-FX profile inherits its old page control, not a later scope.
    let inherited = {
        let mut old = Drive::new();
        old.app
            .learned
            .get_mut("X-Session Pro")
            .unwrap()
            .remove("fx");
        for (group, number) in [
            (Group::PadsEdit, 82),
            (Group::ChopEdit, 83),
            (Group::Global, 84),
        ] {
            learn::assign(
                &mut old.app.learned,
                "X-Session Pro",
                group,
                Key::Cc(number),
                "edit.param_1",
            );
        }
        old.save_registry_bindings()
    };
    let mut drive = Drive::with_saved_bindings(false, Some(&inherited));
    drive.tap(KeyCode::F(4), M::NONE);
    drive.bytes(XS_PORT, &[0xb0, 83, 127]);
    drive.bytes(XS_PORT, &[0xb0, 84, 127]);
    drive.frame();
    assert_eq!(drive.app.pads.fx_pos(pads::Param::Fx1Kind), 0.25);
    drive.bytes(XS_PORT, &[0xb0, 82, 127]);
    drive.frame();
    assert_eq!(drive.app.pads.fx_pos(pads::Param::Fx1Kind), 0.75);
    learn::assign(
        &mut drive.app.learned,
        "X-Session Pro",
        Group::Fx,
        Key::Cc(82),
        "edit.param_2",
    );
    drive.app.sync_learned("X-Session Pro", Group::Fx);
    drive.bytes(XS_PORT, &[0xb0, 82, 0]);
    drive.frame();
    assert_eq!(drive.app.pads.fx_pos(pads::Param::Fx1Kind), 0.75);
    assert_eq!(drive.app.pads.fx_pos(pads::Param::Fx1A), 0.0);

    // Global changes reach FX without an old copied assignment.
    for number in [85, 86] {
        learn::assign(
            &mut drive.app.learned,
            "X-Session Pro",
            Group::Global,
            Key::Cc(number),
            "pad.volume",
        );
        drive.app.sync_learned("X-Session Pro", Group::Global);
    }
    let before = drive.app.pads.volume_db[0];
    drive.bytes(XS_PORT, &[0xb0, 85, 0]);
    drive.frame();
    assert_eq!(drive.app.pads.volume_db[0], before);
    drive.bytes(XS_PORT, &[0xb0, 86, 0]);
    drive.frame();
    assert!(drive.app.pads.volume_db[0] < before);
    assert!(
        learn::clear(&mut drive.app.learned, Group::Global, "pad.volume")
            .contains(&"X-Session Pro".into())
    );
    drive.app.sync_learned("X-Session Pro", Group::Global);
    let before = drive.app.pads.volume_db[0];
    drive.bytes(XS_PORT, &[0xb0, 86, 127]);
    drive.frame();
    assert_eq!(drive.app.pads.volume_db[0], before);

    drive
        .app
        .learned
        .get_mut("X-Session Pro")
        .unwrap()
        .get_mut("fx")
        .unwrap()
        .clear();
    let empty = drive.save_registry_bindings();
    drop(drive);
    let mut drive = Drive::with_saved_bindings(false, Some(&empty));
    assert!(drive.app.learned["X-Session Pro"]["fx"].is_empty());
    drive.tap(KeyCode::F(4), M::NONE);
    let before = (
        drive.app.fx_slot,
        drive.app.fx_param,
        drive.app.pads.fx_pos(pads::Param::Fx1Kind),
    );
    for (status, number) in [(0xb0, 24), (0xb0, 82), (0xb0, 83), (0xb0, 84), (0x90, 43)] {
        drive.bytes(XS_PORT, &[status, number, 127]);
        drive.bytes(XS_PORT, &[status, number, 0]);
        drive.frame();
    }
    assert_eq!(
        (
            drive.app.fx_slot,
            drive.app.fx_param,
            drive.app.pads.fx_pos(pads::Param::Fx1Kind)
        ),
        before
    );
    assert!(!drive.app.edit);
    assert!(drive.app.learned["X-Session Pro"]["fx"].is_empty());
}

#[test]
fn each_registered_screen_keeps_its_effective_help() {
    use ratatui::crossterm::event::ModifierKeyCode;
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = Drive::registry_screen_checks();
        assert!(
            drive
                .keymap
                .overlay("[global]\nF8 = \"bank.next\"\nPageDown = \"none\"\n")
                .unwrap()
                .is_empty()
        );
        for (screen, key, number) in REGISTRY_SCREEN_COMMANDS {
            let (action, old_key, group_names, headings): (&str, &str, &[&str], &[&str]) =
                match screen {
                    Screen::Pads => (
                        "pad.edit",
                        "Enter",
                        &["PAD", "CHANGE", "BANK", "CLOCK", "REC", ""],
                        &[
                            "PADS",
                            "BANK",
                            "CHOP",
                            "SOUND",
                            "SCREENS",
                            "GLYPHS ON A PAD",
                        ],
                    ),
                    Screen::Settings => (
                        "settings.row_next",
                        "K",
                        &["SETTINGS", "CLOCK", "REC", ""],
                        &["PADS", "BANK", "CHOP", "SETTINGS", "SOUND", "SCREENS"],
                    ),
                    Screen::Files => (
                        "browser.hold",
                        "Enter",
                        &["FILES", "BANK", "CLOCK", "REC", ""],
                        &[
                            "PADS",
                            "BANK",
                            "FILES",
                            "CHOP",
                            "SOUND",
                            "SCREENS",
                            "GLYPHS ON A PAD",
                        ],
                    ),
                    Screen::Chop => (
                        "chop.cut",
                        "M",
                        &["MOVE", "CUT", "PADS", "BOUNDS", "BANK", "CLOCK", "REC", ""],
                        &["PADS", "BANK", "CHOP", "SOUND", "SCREENS"],
                    ),
                    Screen::Fx => (
                        "edit.page_next",
                        "Tab",
                        &["SHARED FX", "BANK", "CLOCK", "REC", ""],
                        &["PADS", "BANK", "CHOP", "SOUND", "SCREENS"],
                    ),
                };
            assert!(
                drive
                    .keymap
                    .overlay(&format!(
                        "[screens.{}]\nF9 = \"{action}\"\n{old_key} = \"none\"\n",
                        screen.key()
                    ))
                    .unwrap()
                    .is_empty()
            );
            // F3 from Files uses its selected file. This check opens the loaded pad.
            drive.registry_command(source, KeyCode::F(2), M::NONE, 90);
            drive.registry_command(source, key, M::NONE, number);
            assert_eq!(drive.app.screen, screen);
            let groups = help_groups(&drive.keymap, screen, &drive.app.help_state());
            assert_eq!(
                groups.iter().map(|group| group.name).collect::<Vec<_>>(),
                group_names
            );
            assert!(
                groups
                    .iter()
                    .flat_map(|group| &group.items)
                    .any(|(keys, _)| keys.split(' ').any(|key| key == "F9")),
                "{screen:?}: override must be visible"
            );
            let normal =
                without_modifiers(help_groups(&drive.keymap, screen, &drive.app.help_state()));
            assert!(
                normal
                    .iter()
                    .flat_map(|group| &group.items)
                    .all(|(keys, _)| !keys.contains('+'))
            );
            for width in [100, 252] {
                assert_eq!(tui::help_rows(&groups, width), 3);
                assert_eq!(tui::help_rows(&normal, width), 3);
            }
            assert_eq!(tui::help_dropped(&groups, 252), 0);
            let dropped = tui::help_dropped(&groups, 100);
            assert!(dropped < groups.len());
            assert_eq!(groups[0].name, group_names[0]);
            if screen == Screen::Files {
                let kept = &groups[..groups.len() - dropped];
                assert!(kept.iter().any(|group| group.name == "BANK"));
                assert!(!kept.iter().any(|group| group.name.is_empty()));
            }
            drive.registry_command(source, KeyCode::F(1), M::NONE, 95);
            assert!(drive.app.help_open);
            let full = full_help(&drive.keymap, screen, true);
            assert_eq!(
                full.iter()
                    .filter(|(keys, _)| keys.is_empty())
                    .map(|(_, text)| text.as_str())
                    .collect::<Vec<_>>(),
                headings
            );
            assert!(
                full.iter()
                    .any(|(keys, text)| keys.split(' ').any(|key| key == "F8")
                        && text == "previous / next bank")
            );
            assert!(
                !full
                    .iter()
                    .any(|(keys, _)| keys.split(' ').any(|key| key == "PgDn"))
            );
            assert!(
                full.iter()
                    .any(|(keys, _)| keys.split(' ').any(|key| key == "F9"))
            );
            assert!(
                full_help(&drive.keymap, screen, false)
                    .iter()
                    .any(|(_, text)| text == "NO KEY RELEASES")
            );
            for (code, flags, expected, name) in [
                (
                    ModifierKeyCode::LeftControl,
                    M::CONTROL,
                    input::Modifier::Ctrl,
                    "CTRL",
                ),
                (
                    ModifierKeyCode::LeftAlt,
                    M::ALT,
                    input::Modifier::Alt,
                    "ALT",
                ),
                (
                    ModifierKeyCode::LeftShift,
                    M::SHIFT,
                    input::Modifier::Shift,
                    "SHIFT",
                ),
            ] {
                // Modifier help follows keyboard holds, also after a MIDI screen command.
                drive.key(KeyCode::Modifier(code), flags, KeyEventKind::Press);
                assert_eq!(drive.mods.get(), Some(expected));
                let held = modifier_groups(&drive.keymap, screen, expected, drive.app.ui.hide_pads);
                assert_eq!(held.len(), 1);
                assert_eq!(held[0].name, name);
                if expected == input::Modifier::Shift {
                    assert!(
                        held[0]
                            .items
                            .iter()
                            .any(|(_, text)| text == "pads at full velocity")
                    );
                    if screen == Screen::Chop {
                        assert!(
                            held[0]
                                .items
                                .contains(&("Shift+H".into(), "fewer hit cuts".into()))
                        );
                    }
                }
                drive.key(KeyCode::Modifier(code), M::NONE, KeyEventKind::Release);
                assert_eq!(drive.mods.get(), None);
            }
            drive.registry_command(source, KeyCode::F(1), M::NONE, 95);
            assert!(!drive.app.help_open);
            let texts = |drive: &Drive| {
                help_groups(&drive.keymap, screen, &drive.app.help_state())
                    .into_iter()
                    .flat_map(|group| group.items.into_iter().map(|(_, text)| text))
                    .collect::<Vec<_>>()
            };
            match screen {
                Screen::Pads => {
                    assert!(
                        texts(&drive)
                            .iter()
                            .any(|text| text.starts_with("mark: off"))
                    );
                    drive.registry_command(source, KeyCode::Char('m'), M::NONE, 28);
                    assert!(
                        texts(&drive)
                            .iter()
                            .any(|text| text.starts_with("mark: on"))
                    );
                    drive.registry_command(source, KeyCode::Char('m'), M::NONE, 28);
                }
                Screen::Files => {
                    assert!(texts(&drive).contains(&"preview: on".into()));
                    drive.registry_command(source, KeyCode::Char(' '), M::NONE, 100);
                    assert!(texts(&drive).contains(&"preview: off".into()));
                }
                Screen::Chop => {
                    drive.registry_command(source, KeyCode::Char('o'), M::ALT, 101);
                    assert!(texts(&drive).iter().any(|text| text.contains("loop: on")));
                }
                Screen::Fx => {
                    assert!(texts(&drive).contains(&"FX 1 selected".into()));
                    assert!(texts(&drive).contains(&"parameter".into()));
                    assert!(
                        full.iter()
                            .any(|(_, text)| text == "select the next / previous shared FX block")
                    );
                    assert!(
                        full.iter()
                            .any(|(_, text)| text == "select the next / previous effect parameter")
                    );
                }
                Screen::Settings => {
                    let tabs = "audio / input / MIDI / paths";
                    for expected in [
                        vec!["row", "change", tabs, "back"],
                        vec![tabs, "back"],
                        vec![
                            "learn",
                            "clear",
                            "row",
                            "open / close",
                            "groups",
                            tabs,
                            "back",
                        ],
                        vec!["row", "open", tabs, "back"],
                    ] {
                        let groups = help_groups(&drive.keymap, screen, &drive.app.help_state());
                        assert_eq!(
                            groups[0]
                                .items
                                .iter()
                                .map(|(_, text)| text.as_str())
                                .collect::<Vec<_>>(),
                            expected
                        );
                        assert_eq!(full_help(&drive.keymap, screen, true), full);
                        drive.registry_command(source, KeyCode::Tab, M::NONE, 99);
                    }
                }
            }
            if matches!(screen, Screen::Pads | Screen::Chop) {
                assert!(
                    drive
                        .keymap
                        .overlay(
                            "[edit]\nF7 = \"edit.param_next\"\nK = \"none\"\nDown = \"none\"\n"
                        )
                        .unwrap()
                        .is_empty()
                );
                let (key, number) = if screen == Screen::Pads {
                    (KeyCode::F(9), 97)
                } else {
                    (KeyCode::Enter, 98)
                };
                drive.registry_command(source, key, M::NONE, number);
                assert!(drive.app.edit);
                let edit = help_groups(&drive.keymap, screen, &drive.app.help_state());
                assert_eq!(edit.len(), 1);
                assert_eq!(edit[0].name, "EDIT");
                assert!(
                    edit[0]
                        .items
                        .iter()
                        .any(|(keys, _)| keys.split(' ').any(|key| key == "F7"))
                );
                assert!(
                    !edit[0]
                        .items
                        .iter()
                        .any(|(keys, _)| keys.split(' ').any(|key| matches!(key, "K" | "↓")))
                );
                assert!(
                    edit[0]
                        .items
                        .iter()
                        .any(|(_, text)| text == drive.app.help_state().page)
                );
                if screen == Screen::Chop {
                    assert_eq!(edit[0].items.len(), 4);
                    assert_eq!(edit[0].items[0].1, "BOUNDS");
                    assert!(!edit[0].items.iter().any(|(keys, _)| keys.contains("Tab")));
                }
                assert_eq!(tui::help_rows(&edit, 100), 3);
            }
        }
    }
}

#[test]
fn screen_learning_keeps_its_existing_entry_points() {
    use input::Group;
    for source in [Source::Keyboard, Source::Button] {
        for (screen, key, number, editing, group) in [
            (Screen::Pads, KeyCode::F(2), 90, false, Group::Pads),
            (Screen::Pads, KeyCode::F(2), 90, true, Group::PadsEdit),
            (Screen::Files, KeyCode::F(6), 92, false, Group::Files),
            (Screen::Chop, KeyCode::F(3), 93, false, Group::Chop),
            (Screen::Chop, KeyCode::F(3), 93, true, Group::ChopEdit),
            (Screen::Fx, KeyCode::F(4), 94, false, Group::Fx),
        ] {
            let mut drive = Drive::registry_screen_checks();
            drive.registry_command(source, key, M::NONE, number);
            if editing {
                drive.registry_command(
                    source,
                    KeyCode::Enter,
                    M::NONE,
                    if screen == Screen::Pads { 97 } else { 98 },
                );
            }
            assert_eq!(drive.app.edit, editing);
            let before = drive.app.learned.clone();
            drive.registry_command(source, KeyCode::F(10), M::NONE, 96);
            let learn = drive.app.screen_learn.as_ref().unwrap();
            assert_eq!(learn.group, group);
            assert_eq!(drive.app.learn_group(), group);
            assert_eq!(
                learn.page,
                if screen == Screen::Pads && editing {
                    Some(drive.app.pads.page().0)
                } else {
                    None
                }
            );
            assert!(drive.app.learn_lines().unwrap()[0].contains(group.title()));
            if screen == Screen::Chop {
                assert_eq!(drive.app.help_state().page, "BOUNDS");
                assert!(
                    drive
                        .app
                        .learn_params()
                        .iter()
                        .all(|(name, _)| !name.starts_with("edit.page"))
                );
            }
            if screen == Screen::Fx {
                assert!(!drive.app.edit);
                assert_eq!(group, Group::Fx);
            }
            drive.tap(KeyCode::Char('q'), M::NONE);
            assert_eq!(drive.app.learn_picked().unwrap().0, "pad.5");
            drive.bytes(PAD_PORT, &[0x90, 110, 100]);
            drive.bytes(PAD_PORT, &[0x80, 110, 0]);
            let pending = drive.frame();
            assert!(pending.commands().is_empty());
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
                input::midi::Key::Note(110)
            );
            assert_eq!(drive.app.learned, before);
            // Screen learn captures MIDI as candidates. Esc cancels through its existing key path.
            drive.tap(KeyCode::Esc, M::NONE);
            assert!(drive.app.screen_learn.as_ref().unwrap().pending.is_none());
            drive.tap(KeyCode::Esc, M::NONE);
            assert!(drive.app.screen_learn.is_none());
            assert_eq!(drive.app.edit, editing);
            assert_eq!(drive.app.learned, before);
        }
        let mut drive = Drive::registry_screen_checks();
        drive.registry_command(source, KeyCode::F(12), M::NONE, 91);
        for tab in 0..4 {
            assert_eq!(drive.app.settings_tab, tab);
            let before = drive.app.learned.clone();
            drive.registry_command(source, KeyCode::F(10), M::NONE, 96);
            assert!(drive.app.screen_learn.is_none());
            assert!(drive.app.learning.is_none() && drive.app.midi_run.is_none());
            assert_eq!(drive.app.learned, before);
            assert_eq!(drive.app.screen_group(), Group::Files);
            assert_eq!(drive.app.layer_group(), Group::Files);
            drive.registry_command(source, KeyCode::Tab, M::NONE, 99);
        }
    }
}

#[test]
fn midi_list_keeps_screen_groups_and_their_order() {
    use input::{Group, midi::Scope};
    let groups = [
        Group::Global,
        Group::Pads,
        Group::PadsEdit,
        Group::Settings,
        Group::Files,
        Group::Chop,
        Group::ChopEdit,
        Group::Fx,
    ];
    let identities = [
        ("global", "GLOBAL", Scope::Global),
        ("pads", "PADS", Scope::Screen(Screen::Pads)),
        ("pads_edit", "PADS · EDIT", Scope::Edit(Screen::Pads)),
        ("settings", "SETTINGS", Scope::Screen(Screen::Settings)),
        ("files", "FILES", Scope::Screen(Screen::Files)),
        ("chop", "CHOP", Scope::Screen(Screen::Chop)),
        ("chop_edit", "CHOP · EDIT", Scope::Edit(Screen::Chop)),
        ("fx", "FX", Scope::Screen(Screen::Fx)),
    ];
    for source in [Source::Keyboard, Source::Button] {
        for (screen, key, number, group) in [
            (Screen::Pads, KeyCode::F(2), 90, Group::Pads),
            (Screen::Files, KeyCode::F(6), 92, Group::Files),
            (Screen::Chop, KeyCode::F(3), 93, Group::Chop),
            (Screen::Fx, KeyCode::F(4), 94, Group::Fx),
        ] {
            for editing in [false, true] {
                if editing && !matches!(screen, Screen::Pads | Screen::Chop) {
                    continue;
                }
                let mut drive = Drive::registry_screen_checks();
                let saved = drive.app.learned.clone();
                drive.registry_command(source, key, M::NONE, number);
                if editing {
                    drive.registry_command(
                        source,
                        KeyCode::Enter,
                        M::NONE,
                        if screen == Screen::Pads { 97 } else { 98 },
                    );
                    assert!(drive.app.edit);
                }
                drive.registry_command(source, KeyCode::F(12), M::NONE, 91);
                assert_eq!(drive.app.screen, Screen::Settings);
                assert!(!drive.app.edit);
                assert_eq!(drive.app.midi_open, [group]);
                drive.registry_command(source, KeyCode::Tab, M::NONE, 99);
                drive.registry_command(source, KeyCode::Tab, M::NONE, 99);
                assert_eq!(drive.app.settings_tab, 2);
                let rows = drive.app.midi_list();
                let listed: Vec<_> = rows
                    .iter()
                    .filter_map(|row| match row {
                        MidiListRow::Group { group } => Some(*group),
                        _ => None,
                    })
                    .collect();
                assert_eq!(listed, groups);
                assert_eq!(
                    listed
                        .iter()
                        .map(|group| (group.key(), group.title(), group.scope()))
                        .collect::<Vec<_>>(),
                    identities
                );
                let visible: Vec<_> = rows
                    .iter()
                    .filter_map(|row| match row {
                        MidiListRow::Action { info } => Some(info.group),
                        MidiListRow::Set { infos, .. } => Some(infos[0].group),
                        _ => None,
                    })
                    .collect();
                assert!(!visible.is_empty());
                assert!(visible.iter().all(|visible| *visible == group));
                assert_eq!(drive.app.learned, saved);
                assert!(drive.app.screen_learn.is_none());
            }
        }
    }
}

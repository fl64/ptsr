//! Transport scenarios through the recorded production keyboard and MIDI routes.

use super::*;
use std::sync::atomic::Ordering::Acquire;

fn settle(drive: &mut Drive) {
    for _ in 0..512 {
        drive.app.flush_audio();
        drive.render_blocks(1);
        drive.frame();
        if drive.app.pending_audio.is_empty()
            && drive.project.ready.load(Acquire) == drive.project.epoch.load(Acquire)
        {
            return;
        }
    }
    panic!("project restoration did not complete");
}

fn start(drive: &mut Drive) {
    drive.app.screen = Screen::Pads;
    drive.tap(KeyCode::Char(' '), M::NONE);
    settle(drive);
    assert!(drive.app.transport_state().playing);
}

fn project_drive(files: &TransitionFiles) -> Drive {
    let mut drive = Drive::offline();
    drive.app.paths.projects = files.0.clone();
    drive.app.session_path = files.0.join("scratch/session.toml");
    drive.app.session_read_error = None;
    drive.app.project = Some(Project {
        name: "A".into(),
        folder: files.0.join("A"),
    });
    drive.app.user_path = Some(files.0.join("config.toml"));
    drive.app.save_session().unwrap();
    drive
}

#[test]
fn space_controls_transport_without_replacing_previews() {
    let mut drive = Drive::offline();
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 48_000])), "preview");
    for screen in [Screen::Pads, Screen::Fx, Screen::Settings] {
        drive.app.screen = screen;
        let frame = drive.tap(KeyCode::Char(' '), M::NONE);
        assert!(
            frame
                .ui
                .iter()
                .any(|c| matches!(c, Command::TransportToggle))
        );
        settle(&mut drive);
        assert!(drive.app.transport_state().playing);
        drive.tap(KeyCode::Char(' '), M::NONE);
        settle(&mut drive);
        assert!(!drive.app.transport_state().playing);
    }
    drive.app.screen = Screen::Pads;
    drive.tap(KeyCode::F(3), M::NONE);
    assert!(drive.app.chop.is_some());
    let frame = drive.tap(KeyCode::Char(' '), M::NONE);
    assert!(
        frame
            .ui
            .iter()
            .any(|c| matches!(c, Command::Preview { sound: Some(_), .. }))
    );
    settle(&mut drive);
    assert!(!drive.app.transport_state().playing);
    drive.app.screen = Screen::Files;
    let frame = drive.tap(KeyCode::Char(' '), M::NONE);
    assert!(
        !frame
            .commands()
            .iter()
            .any(|c| matches!(c, Command::TransportToggle))
    );
    assert_eq!(
        drive.keymap.resolve_in(
            Screen::Files,
            false,
            &KeyEvent::new(KeyCode::Char(' '), M::NONE)
        ),
        Some(Action::BrowserPreviewToggle)
    );
}

#[test]
fn tempo_and_metronome_keys_work_in_both_layouts() {
    for kitty in [false, true] {
        for letter in ['m', 'ь'] {
            let mut drive = Drive::offline();
            let mut key = |code, modifiers| {
                for kind in [KeyEventKind::Press, KeyEventKind::Release] {
                    drive.app.keyboard_event(
                        &drive.keymap,
                        &KeyEvent::new_with_kind(code, modifiers, kind),
                        kitty,
                        &mut drive.down,
                        &mut drive.held,
                        &mut drive.mods,
                    );
                }
            };
            key(KeyCode::Up, M::CONTROL);
            key(KeyCode::Up, M::CONTROL | M::SHIFT);
            key(KeyCode::Down, M::CONTROL | M::SHIFT);
            key(KeyCode::Char(letter), M::ALT);
            assert_eq!(drive.app.transport.bpm, 121.0);
            assert!(drive.app.transport.metronome);
            settle(&mut drive);
            assert_eq!(drive.app.transport_state().bpm, 121.0);
            for (bpm, code) in [(300.0, KeyCode::Up), (40.0, KeyCode::Down)] {
                drive.app.set_tempo(bpm, true);
                drive.tap(code, M::CONTROL);
                drive.tap(code, M::CONTROL | M::SHIFT);
                assert_eq!(drive.app.transport.bpm, bpm);
            }
        }
    }
}

#[test]
fn holding_space_does_not_repeatedly_toggle_transport() {
    let mut drive = Drive::offline();
    drive.key(KeyCode::Char(' '), M::NONE, KeyEventKind::Press);
    settle(&mut drive);
    for _ in 0..10 {
        drive.key(KeyCode::Char(' '), M::NONE, KeyEventKind::Repeat);
        drive.key(KeyCode::Char(' '), M::NONE, KeyEventKind::Press);
        settle(&mut drive);
        assert!(drive.app.transport_state().playing);
    }
    drive.key(KeyCode::Char(' '), M::NONE, KeyEventKind::Release);
    settle(&mut drive);
    assert!(drive.app.transport_state().playing);
    drive.tap(KeyCode::Char(' '), M::NONE);
    settle(&mut drive);
    assert!(!drive.app.transport_state().playing);
}

#[test]
fn previous_click_and_preview_bindings_retain_their_meaning() {
    let mut drive = Drive::offline();
    drive
        .keymap
        .overlay("[global]\nSpace = \"metronome.click\"\nF9 = \"transport.play_stop\"\n")
        .unwrap();
    let frame = drive.tap(KeyCode::Char(' '), M::NONE);
    assert!(frame.ui.iter().any(|c| matches!(c, Command::Click { .. })));
    settle(&mut drive);
    assert!(!drive.app.transport_state().playing);
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 48_000])), "preview");
    drive.tap(KeyCode::F(3), M::NONE);
    let frame = drive.tap(KeyCode::F(9), M::NONE);
    assert!(
        frame
            .ui
            .iter()
            .any(|c| matches!(c, Command::Preview { sound: Some(_), .. }))
    );
    settle(&mut drive);
    assert!(!drive.app.transport_state().playing);
}

#[test]
fn owner_overrides_hide_new_transport_defaults_with_a_warning() {
    let files = TransitionFiles::new();
    let path = files.0.join("keymap.toml");
    let text =
        "[global]\nSpace = \"metronome.click\"\n\"Ctrl+Up\" = \"none\"\n\"Alt+M\" = \"none\"\n";
    std::fs::write(&path, text).unwrap();
    let (map, notice, warnings) = load_keymap(&path, DEFAULT_KEYMAP);
    assert!(notice.is_none());
    let unique: std::collections::BTreeSet<_> = warnings.iter().collect();
    assert_eq!(unique.len(), warnings.len());
    for screen in Screen::ALL {
        assert_eq!(map.binding_for(screen, false, "Ctrl+Up"), Some(None));
        assert_eq!(map.binding_for(screen, false, "Alt+M"), Some(None));
        assert!(warnings.iter().any(|w| w.contains(&format!(
            "Ctrl+Up on {} hides transport.tempo_up",
            screen.key()
        ))));
        assert!(
            warnings
                .iter()
                .any(|w| w.contains(&format!("Alt+M on {} hides metronome.toggle", screen.key())))
        );
    }
    assert_eq!(warnings.iter().filter(|w| w.contains("Space")).count(), 3);
    assert_eq!(std::fs::read_to_string(path).unwrap(), text);
}

#[test]
fn transport_input_respects_existing_overlays() {
    for overlay in ["text", "question", "picker", "screen learn", "midi learn"] {
        let mut drive = Drive::offline();
        match overlay {
            "text" => {
                drive.app.screen = Screen::Files;
                drive.tap(KeyCode::Char('/'), M::NONE);
                assert!(drive.app.browser.editing);
            }
            "question" => {
                drive.app.ask = Some(Ask::ClearPads {
                    pads: vec![0],
                    lines: vec!["Clear?".into()],
                })
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
                drive.app.midi_row = drive.app.midi_list().iter().position(|row| matches!(row, MidiListRow::Action { info } if info.name == "transport.toggle")).unwrap();
                drive.tap(KeyCode::Enter, M::NONE);
                assert!(drive.app.learning.is_some());
            }
        }
        drive.tap(KeyCode::Char(' '), M::NONE);
        drive.tap(KeyCode::Up, M::CONTROL);
        drive.tap(KeyCode::Char('m'), M::ALT);
        settle(&mut drive);
        assert_eq!(
            drive.app.transport_state(),
            engine::transport::Snapshot::default(),
            "{overlay}"
        );
    }
}

#[test]
fn help_follows_effective_transport_bindings() {
    let mut drive = Drive::new();
    drive
        .keymap
        .overlay("[global]\nSpace = \"none\"\nF9 = \"transport.toggle\"\n\"Ctrl+Up\" = \"none\"\n")
        .unwrap();
    for screen in Screen::ALL {
        drive.app.screen = screen;
        let groups = help_groups(&drive.keymap, screen, &drive.app.help_state());
        let clock = groups.iter().find(|g| g.name == "CLOCK").unwrap();
        assert!(clock.items.iter().any(|(keys, _)| keys == "F9"));
        assert!(clock.items.iter().all(|(keys, _)| !keys.contains("Ctrl+↑")));
        let full = full_help(&drive.keymap, screen, true);
        assert!(
            full.iter()
                .any(|(keys, text)| keys == "F9" && text.contains("transport"))
        );
        assert!(
            !full
                .iter()
                .any(|(keys, text)| keys.contains("Space") && text.contains("transport"))
        );
    }
    let (shipped, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
    for screen in [Screen::Chop, Screen::Files] {
        let groups = help_groups(&shipped, screen, &drive.app.help_state());
        let clock = groups.iter().find(|g| g.name == "CLOCK").unwrap();
        assert!(clock.items.iter().all(|(key, _)| key != "Space"));
    }
}

#[test]
fn raw_midi_buttons_control_transport_and_metronome() {
    let mut drive = Drive::offline();
    drive.bind_action(100, "transport.toggle");
    drive.bind_action(101, "metronome.toggle");
    for (cc, playing, metro) in [(100, true, false), (101, true, true), (100, false, true)] {
        drive.bytes(PAD_PORT, &[0xb0, cc, 127]);
        drive.frame();
        settle(&mut drive);
        let before = drive.app.transport_state();
        assert_eq!((before.playing, before.metronome), (playing, metro));
        drive.bytes(PAD_PORT, &[0xb0, cc, 0]);
        drive.frame();
        settle(&mut drive);
        let after = drive.app.transport_state();
        assert_eq!((after.playing, after.metronome), (playing, metro));
    }
}

#[test]
fn raw_midi_tempo_buttons_match_keyboard_steps() {
    for (action, code, mods, expected) in [
        ("transport.tempo_up", KeyCode::Up, M::CONTROL, 121.0),
        ("transport.tempo_down", KeyCode::Down, M::CONTROL, 119.0),
        (
            "transport.tempo_up_fine",
            KeyCode::Up,
            M::CONTROL | M::SHIFT,
            120.1,
        ),
        (
            "transport.tempo_down_fine",
            KeyCode::Down,
            M::CONTROL | M::SHIFT,
            119.9,
        ),
    ] {
        for source in [Source::Keyboard, Source::Button] {
            let mut drive = Drive::offline();
            drive.bind_action(100, action);
            drive.registry_command(source, code, mods, 100);
            settle(&mut drive);
            assert_eq!(
                drive.app.transport_state().bpm,
                expected,
                "{action}/{source:?}"
            );
            assert_eq!(shown_value(&drive).formatted, format!("{expected:.1} BPM"));
        }
    }
}

#[test]
fn transport_learn_selects_without_playing() {
    let mut drive = Drive::offline();
    drive.tap(KeyCode::F(10), M::NONE);
    drive.tap(KeyCode::Char(' '), M::NONE);
    assert_eq!(drive.app.learn_picked().unwrap().0, "transport.toggle");
    let old = drive.app.learned.clone();
    drive.bytes(XS_PORT, &[0xb0, 24, 64]);
    drive.frame();
    assert!(drive.app.screen_learn.as_ref().unwrap().refused);
    assert_eq!(drive.app.learned, old);
    drive.bytes(XS_PORT, &[0x90, 69, 127]);
    drive.bytes(XS_PORT, &[0x80, 69, 0]);
    drive.frame();
    assert!(drive.app.screen_learn.as_ref().unwrap().pending.is_some());
    assert_eq!(drive.app.learned, old);
    settle(&mut drive);
    assert!(!drive.app.transport_state().playing);
    drive.tap(KeyCode::Esc, M::NONE);
    drive.tap(KeyCode::Esc, M::NONE);
    drive.tap(KeyCode::F(12), M::NONE);
    drive.app.settings_tab = 2;
    drive.app.midi_open.push(input::Group::Global);
    for info in input::ACTIONS
        .iter()
        .filter(|i| i.name.starts_with("transport.") || i.name == "metronome.toggle")
    {
        if info.name == "transport.play_stop" {
            continue;
        }
        assert_eq!(info.group, input::Group::Global);
        assert_eq!(info.knob, info.name == "transport.tempo");
    }
    drive.app.midi_row = drive
        .app
        .midi_list()
        .iter()
        .position(
            |row| matches!(row, MidiListRow::Action { info } if info.name == "transport.toggle"),
        )
        .unwrap();
    drive.tap(KeyCode::Enter, M::NONE);
    drive.bytes(PAD_PORT, &[0xb0, 100, 127]);
    drive.frame();
    assert!(drive.app.learning.as_ref().unwrap().pending.is_some());
    assert_eq!(drive.app.learned, old);
    settle(&mut drive);
    assert!(!drive.app.transport_state().playing);
    drive.tap(KeyCode::Enter, M::NONE);
    assert_eq!(
        drive.app.learned["Recorded pads"]["global"]["cc 100"],
        "transport.toggle"
    );
}

#[test]
fn tempo_knob_maps_its_full_range() {
    let mut drive = Drive::offline();
    for (value, expected) in [(0, 40.0), (64, 171.0), (127, 300.0)] {
        value_knob(&mut drive, "transport.tempo", value);
        settle(&mut drive);
        assert_eq!(drive.app.transport_state().bpm, expected);
        let popup = shown_value(&drive);
        assert_eq!(popup.object, "Transport");
        assert_eq!(popup.parameter, "tempo");
        assert_eq!(popup.formatted, format!("{expected:.1} BPM"));
        assert!(popup.position.is_some_and(|p| (0.0..=1.0).contains(&p)));
    }
}

#[test]
fn tempo_pickup_remains_separate_per_controller() {
    let mut drive = Drive::offline();
    drive.app.takeover.response = input::midi::Response::Pickup;
    for (profile, cc) in [("X-Session Pro", 100), ("Recorded pads", 101)] {
        learn::assign(
            &mut drive.app.learned,
            profile,
            input::Group::Global,
            input::midi::Key::Cc(cc),
            "transport.tempo",
        );
        drive.app.sync_learned(profile, input::Group::Global);
    }
    let turn = |drive: &mut Drive, port, cc, value| {
        drive.bytes(port, &[0xb0, cc, value]);
        drive.frame();
        settle(drive);
    };
    turn(&mut drive, XS_PORT, 100, 0);
    turn(&mut drive, PAD_PORT, 101, 0);
    assert_eq!(drive.app.transport.bpm, 120.0);
    turn(&mut drive, XS_PORT, 100, 127);
    assert_eq!(drive.app.transport.bpm, 300.0);
    turn(&mut drive, PAD_PORT, 101, 10);
    assert_eq!(drive.app.transport.bpm, 300.0);
    drive.tap(KeyCode::Down, M::CONTROL);
    assert_eq!(drive.app.transport.bpm, 299.0);
    turn(&mut drive, XS_PORT, 100, 110);
    assert_eq!(
        drive.app.transport.bpm, 299.0,
        "keyboard edit resets previous pickup"
    );
    turn(&mut drive, XS_PORT, 100, 127);
    assert_eq!(drive.app.transport.bpm, 300.0);
    turn(&mut drive, PAD_PORT, 101, 20);
    assert_eq!(drive.app.transport.bpm, 300.0, "other device stays locked");
    turn(&mut drive, PAD_PORT, 101, 127);
    assert_eq!(drive.app.transport.bpm, 300.0);
    turn(&mut drive, PAD_PORT, 101, 0);
    assert_eq!(drive.app.transport.bpm, 40.0);
}

#[test]
fn tempo_scale_and_project_pickup_use_current_preferences() {
    let mut drive = Drive::offline();
    drive.app.takeover.response = input::midi::Response::Scale;
    value_knob(&mut drive, "transport.tempo", 0);
    settle(&mut drive);
    assert_eq!(
        drive.app.transport.bpm, 120.0,
        "scale first touch preserves tempo"
    );
    for (value, expected) in [(20, 148.3), (127, 300.0), (0, 40.0)] {
        drive.bytes(XS_PORT, &[0xb0, 100, value]);
        drive.frame();
        settle(&mut drive);
        assert_eq!(drive.app.transport_state().bpm, expected);
    }
    drive.app.takeover.response = input::midi::Response::Pickup;
    drive.bytes(XS_PORT, &[0xb0, 100, 127]);
    drive.frame();
    settle(&mut drive);
    assert_eq!(drive.app.transport.bpm, 300.0);
    drive.app.apply_session(session::Session {
        bpm: 172.0,
        ..Default::default()
    });
    settle(&mut drive);
    drive.bytes(XS_PORT, &[0xb0, 100, 100]);
    drive.frame();
    settle(&mut drive);
    assert_eq!(
        drive.app.transport_state().bpm,
        172.0,
        "new project resets old pickup"
    );
    drive.bytes(XS_PORT, &[0xb0, 100, 0]);
    drive.frame();
    settle(&mut drive);
    assert_eq!(drive.app.transport_state().bpm, 40.0);
}

#[test]
fn transport_help_keeps_arrows_and_the_existing_edit_band() {
    let (map, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
    let clock = transport_help(&map, Screen::Pads);
    for key in [
        "Space",
        "Ctrl+↑",
        "Ctrl+↓",
        "Ctrl+Shift+↑",
        "Ctrl+Shift+↓",
        "Alt+M",
    ] {
        assert!(
            clock
                .items
                .iter()
                .any(|(keys, _)| keys.split(' ').any(|k| k == key)),
            "{key}: {:?}",
            clock.items
        );
    }
    let mut drive = Drive::new();
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(drive.app.edit);
    let groups = help_groups(&map, Screen::Pads, &drive.app.help_state());
    assert_eq!(groups.iter().map(|g| g.name).collect::<Vec<_>>(), ["EDIT"]);
    assert!(
        full_help(&map, Screen::Pads, true)
            .iter()
            .any(|(keys, text)| keys == "Space" && text.contains("transport"))
    );
}

#[test]
fn new_transport_bindings_survive_restart_without_a_profile_reset() {
    let files = TransitionFiles::new();
    let path = files.0.join("config.toml");
    let mut drive = Drive::new();
    let original = drive.app.learned["X-Session Pro"].clone();
    drive.bind_action(100, "transport.toggle");
    value_knob(&mut drive, "transport.tempo", 64);
    config::save_section(&path, "midi", &drive.app.learned).unwrap();
    config::save_section(
        &path,
        "devices",
        &config::DevicesConfig {
            seeded: drive.app.seeded.clone(),
        },
    )
    .unwrap();
    let expected = drive.app.learned.clone();
    let bytes = std::fs::read(&path).unwrap();
    let mut restored = Drive::with_saved_bindings(false, Some(&path));
    assert_eq!(restored.app.learned, expected);
    // Only the explicit tempo assignment differs from the original profile.
    let mut without_tempo = restored.app.learned["X-Session Pro"].clone();
    without_tempo.get_mut("global").unwrap().remove("cc 100");
    assert_eq!(without_tempo, original);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert!(
        restored
            .registry_command(Source::Button, KeyCode::Null, M::NONE, 100)
            .commands()
            .iter()
            .any(|c| matches!(c, Command::TransportToggle))
    );
}

/// Scenarios "Project transport preferences survive a restart" and "Scratch transport preferences survive a restart".
#[test]
fn saved_transport_preferences_restore_stopped() {
    for project in [false, true] {
        let files = TransitionFiles::new();
        let mut drive = project_drive(&files);
        if !project {
            drive.app.project = None;
        }
        drive.app.set_tempo(172.3, true);
        drive.tap(KeyCode::Char('m'), M::ALT);
        start(&mut drive);
        assert!(drive.app.session_dirty.is_some());
        drive.tap(KeyCode::Char('s'), M::CONTROL);
        assert!(drive.app.session_dirty.is_none());
        let path = drive.app.session_file();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("playing") && !text.contains("ticks"));
        let saved = session::load_existing(&path).unwrap();
        assert_eq!(saved.version, 4);
        assert_eq!((saved.bpm, saved.metronome), (172.3, true));
        let mut restored = Drive::offline();
        restored.app.apply_session(saved);
        settle(&mut restored);
        let state = restored.app.transport_state();
        assert_eq!(
            (state.bpm, state.metronome, state.playing, state.bar_beat()),
            (172.3, true, false, (1, 1))
        );
    }
}

#[test]
fn older_sessions_gain_transport_defaults_without_changing_pads() {
    let files = TransitionFiles::new();
    let path = files.0.join("old.toml");
    for version in 1..=3 {
        std::fs::write(&path, format!("version = {version}\nmaster_db = -6\n\n[pads.0]\nfile = \"samples/tone.wav\"\nname = \"tone (copy 3)\"\ngroup = 7\ngate = true\npitch = 3\nstart = 0.0\nend = 0.5\nslice = 1\nvolume_db = -9\nsends = [20, 30]\n")).unwrap();
        let old = session::load_existing(&path).unwrap();
        assert_eq!((old.bpm, old.metronome), (120.0, false));
        session::save(&path, &old).unwrap();
        let new = session::load_existing(&path).unwrap();
        assert_eq!(new.version, 4);
        assert_eq!(new.pads, old.pads);
        assert_eq!(new.chops, old.chops);
        assert_eq!(new.master_db, -6);
        assert_eq!(new.pads["0"].group, 7);
        assert_eq!(new.pads["0"].name.as_deref(), Some("tone (copy 3)"));
        assert_eq!(
            new.pads["0"].file.as_deref(),
            Some(Path::new("samples/tone.wav"))
        );
    }
}

#[test]
fn invalid_transport_settings_protect_the_session_file() {
    let files = TransitionFiles::new();
    for value in [
        "bpm = nan",
        "bpm = inf",
        "bpm = -inf",
        "bpm = 39.9",
        "bpm = 300.1",
        "metronome = 1",
        "metronome = 'yes'",
        "bpm = 'fast'",
    ] {
        let path = files.0.join("B/project.toml");
        let bytes = format!("version = 4\n{value}\n");
        std::fs::write(&path, &bytes).unwrap();
        let error = session::load_existing(&path).unwrap_err();
        let mut drive = project_drive(&files);
        start(&mut drive);
        let before = drive.app.transport_state();
        assert!(!drive.app.open_project(files.0.join("B")));
        assert_eq!(drive.app.transport_state(), before);
        drive.app.project = None;
        drive.app.session_path = path.clone();
        drive.app.session_read_error = Some(error);
        drive.app.set_tempo(172.0, true);
        assert!(drive.app.save_session().is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), bytes);
    }
}

/// Scenarios "Different projects retain independent transport settings" and "Late transport commands stay in their original project".
#[test]
fn different_projects_retain_independent_transport_settings() {
    let files = TransitionFiles::new();
    let target = session::Session {
        bpm: 90.3,
        metronome: true,
        ..Default::default()
    };
    session::save_project(&files.0.join("B/project.toml"), &target).unwrap();
    let mut drive = project_drive(&files);
    drive.app.set_tempo(172.0, true);
    start(&mut drive);
    while drive
        .app
        .audio
        .as_mut()
        .unwrap()
        .commands
        .push(ProjectCommand {
            epoch: 0,
            ready_at_send: true,
            command: ProjectMessage::Command(Command::TransportToggle),
        })
        .is_ok()
    {}
    drive.tap(KeyCode::Up, M::CONTROL);
    assert!(!drive.app.pending_audio.is_empty());
    assert!(drive.app.open_project(files.0.join("B")));
    settle(&mut drive);
    let state = drive.app.transport_state();
    assert_eq!(
        (state.bpm, state.metronome, state.playing, state.ticks),
        (90.3, true, false, 0.0)
    );
    assert!(drive.app.open_project(files.0.join("A")));
    settle(&mut drive);
    assert_eq!(drive.app.transport_state().bpm, 173.0);
    assert!(!drive.app.transport_state().playing);
}

#[test]
fn refused_project_switch_preserves_running_transport() {
    for failure in ["read", "save"] {
        let files = TransitionFiles::new();
        let mut drive = project_drive(&files);
        drive.app.set_tempo(172.0, true);
        drive.tap(KeyCode::Char('m'), M::ALT);
        start(&mut drive);
        if failure == "read" {
            std::fs::write(files.0.join("B/project.toml"), "version = 99\n").unwrap();
        } else {
            std::fs::create_dir(drive.app.session_file().with_extension("toml.tmp")).unwrap();
        }
        let before = drive.app.transport_state();
        assert!(!drive.app.open_project(files.0.join("B")));
        assert_eq!(drive.app.transport_state(), before);
        settle(&mut drive);
        let after = drive.app.transport_state();
        assert_eq!(
            (after.bpm, after.playing, after.metronome),
            (172.0, true, true)
        );
        assert!(after.ticks > before.ticks);
    }
}

#[test]
fn selecting_the_current_project_preserves_its_running_clock() {
    let files = TransitionFiles::new();
    let mut drive = project_drive(&files);
    drive.app.set_tempo(172.0, true);
    start(&mut drive);
    std::os::unix::fs::symlink(files.0.join("A"), files.0.join("alias")).unwrap();
    for name in ["A", "alias"] {
        TransitionFiles::pick(&mut drive, Source::Keyboard, name, false);
        let before = drive.app.transport_state();
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.project_picker.is_none());
        assert_eq!(drive.app.transport_state(), before);
        settle(&mut drive);
        assert!(drive.app.transport_state().ticks > before.ticks);
    }
}

#[test]
fn transport_feedback_follows_accepted_audio_state() {
    let mut drive = Drive::offline();
    drive.tap(KeyCode::Up, M::CONTROL);
    drive.tap(KeyCode::Char('m'), M::ALT);
    drive.tap(KeyCode::Char(' '), M::NONE);
    let label = |d: &Drive| d.app.title_badges()[0].normal.clone();
    assert_eq!(
        label(&drive),
        "■ 120.0",
        "queued controls are not accepted state"
    );
    settle(&mut drive);
    let accepted = drive.app.transport_state();
    assert_eq!(
        (accepted.bpm, accepted.playing, accepted.metronome),
        (121.0, true, true)
    );
    for screen in Screen::ALL {
        drive.app.screen = screen;
        for (width, height) in [(80, 26), (120, 40)] {
            let rows = shared_feedback_rows(&mut drive, width, height, Instant::now());
            assert!(rows[1].contains("121.0 M 1:1"), "{screen:?}: {}", rows[1]);
            drive.app.ask = Some(Ask::ClearPads {
                pads: vec![0],
                lines: vec!["Clear?".into()],
            });
            drive.app.session_write_failed = true;
            let rows = shared_feedback_rows(&mut drive, width, height, Instant::now());
            assert!(
                rows[1].contains("121.0 M 1:1") && rows[1].contains("ASK"),
                "{screen:?}: {}",
                rows[1]
            );
            assert!(rows[0].contains("Save failed"), "{}", rows[0]);
            drive.app.ask = None;
            drive.app.session_write_failed = false;
        }
    }
    assert_eq!(
        drive.app.transport_state(),
        accepted,
        "redraws cannot advance the clock"
    );
}

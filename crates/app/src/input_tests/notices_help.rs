//! Recorded notices, questions, and F1 help input.

use super::*;

/// Scenario "Startup acknowledgement retains its exceptions".
#[test]
fn startup_warnings_enter_history_once() {
    let mut drive = Drive::new();
    drive.app.add_keymap_warnings(vec![
        "first keymap warning".into(),
        "second keymap warning".into(),
    ]);
    drive
        .app
        .post_startup(Level::Warn, NO_RELEASES_NOTICE.into());
    drive
        .app
        .post_startup(Level::Error, "protected startup".into());
    drive.frame();
    let history = drive.app.notices.history().clone();
    drive.bytes(PAD_PORT, &[0x90, 36, 127]);
    drive.bytes(PAD_PORT, &[0x80, 36, 0]);
    drive.frame();
    assert_eq!(drive.app.notice().as_deref(), Some("protected startup"));
    assert_eq!(drive.app.notices.history(), &history);
    drive.key(KeyCode::Null, M::NONE, KeyEventKind::Release);
    drive.frame();
    assert!(!drive.app.notices.has_startup());
    assert_eq!(
        drive.app.frame_notice().as_deref(),
        Some("first keymap warning")
    );
    for _ in 0..10 {
        drive.app.frame_notice();
    }
    assert_eq!(drive.app.notices.history(), &history);
    drive.key(KeyCode::Right, M::NONE, KeyEventKind::Release);
    drive.frame();
    assert_eq!(drive.app.keymap_warnings.len(), 2);
    let bank = drive.app.pads.bank;
    drive.key(KeyCode::PageDown, M::NONE, KeyEventKind::Press);
    drive.frame();
    assert_ne!(
        drive.app.pads.bank, bank,
        "acknowledgement does not consume the action"
    );
    assert_eq!(drive.app.keymap_warnings.len(), 1);
    drive.key(KeyCode::PageDown, M::NONE, KeyEventKind::Press);
    drive.frame();
    assert_eq!(
        drive.app.keymap_warnings.len(),
        1,
        "an inferred repeat is not a new press"
    );
    assert_eq!(drive.app.notices.history(), &history);
}

/// Scenario "Device notices retain the actual event source".
#[test]
fn device_notices_retain_the_actual_event_source() {
    use input::{Group, midi::Key};
    let mut drive = Drive::registry_screen_checks();
    drive.tap(KeyCode::F(2), M::NONE);
    drive.bind_action(88, "pad.copy");
    learn::assign(
        &mut drive.app.learned,
        "X-Session Pro",
        Group::Global,
        Key::Note(60),
        "pad.copy",
    );
    drive.app.sync_learned("X-Session Pro", Group::Global);
    drive.midi().recorded_fence();
    drive.app.pads.selected = 3; // Empty on both devices: both requests produce Warn.
    drive.bytes(PAD_PORT, &[0xb0, 88, 127]);
    drive.bytes(XS_PORT, &[0x90, 60, 127]);
    drive.bytes(PAD_PORT, &[0xb0, 88, 0]);
    drive.bytes(XS_PORT, &[0x80, 60, 0]);
    let queued = drive.queued_events();
    let before = drive.app.notices.history().len();
    for event in queued {
        let port = event.origin().map(str::to_owned);
        let produces = matches!(event.event, midi::Event::Action(Action::PadCopy));
        drive.app.midi_ui_event(event);
        if produces {
            assert_eq!(drive.last_notice().origin, Origin::Device(port.unwrap()));
            assert_eq!(drive.last_notice().level, Level::Warn);
        }
    }
    assert_eq!(drive.app.notices.history().len(), before + 2);
    drive.tap(KeyCode::Char('c'), M::CONTROL);
    assert_eq!(drive.last_notice().origin, Origin::Keyboard);
    assert_eq!(drive.last_notice().level, Level::Warn);

    drive.app.pads.selected = 0;
    drive.tap(KeyCode::Char('c'), M::CONTROL);
    assert_eq!(drive.last_notice().level, Level::Info);
    drive.bind_action(87, "bank.next");
    learn::assign(
        &mut drive.app.learned,
        "X-Session Pro",
        Group::Global,
        Key::Note(60),
        "pad.2",
    );
    drive.app.sync_learned("X-Session Pro", Group::Global);
    drive.midi().recorded_fence();
    drive.bytes(PAD_PORT, &[0xb0, 87, 127]);
    drive.bytes(XS_PORT, &[0x90, 60, 127]);
    let queued = drive.queued_events();
    assert!(queued.iter().any(|event| matches!(
        event.event,
        midi::Event::PadHit {
            sound: midi::PadSound::UiUnattempted { deferred: true },
            ..
        }
    ) && event.origin() == Some(XS_PORT)));
    for event in queued {
        drive.app.midi_ui_event(event);
    }
    drive.frame();
    assert_eq!(drive.app.pads.bank, 1);
    assert!(drive.app.pads.file[17].is_some());
    assert_eq!(drive.last_notice().text, "copy placed");
    assert_eq!(drive.last_notice().level, Level::Done);
    assert_eq!(drive.last_notice().origin, Origin::Device(XS_PORT.into()));
    drive.bytes(XS_PORT, &[0x80, 60, 0]);
    drive.frame();
    drive.tap(KeyCode::F(3), M::NONE);
    let before = drive.app.notices.history().len();
    drive.bytes(XS_PORT, &[0xb0, 35, 80]);
    drive.frame();
    assert!(drive.app.notices.history().len() > before);
    assert_eq!(drive.last_notice().origin, Origin::Device(XS_PORT.into()));
    assert_eq!(drive.last_notice().level, Level::Info);
}

/// Scenario "Background and release events cannot dismiss errors".
#[test]
fn owner_acknowledgement_keeps_the_original_action() {
    let mut drive = Drive::new();
    drive.bind_action(88, "bank.next");
    drive
        .app
        .post_system_notice(Level::Error, ("same error".into(), Instant::now()));
    drive.frame();
    drive.bytes(PAD_PORT, &[0xb0, 88, 0]);
    drive.bytes(PAD_PORT, &[0x80, 36, 0]);
    drive.bytes(PAD_PORT, &[0xf8]);
    drive.frame();
    assert_eq!(drive.app.notice().as_deref(), Some("same error"));
    drive.bytes(PAD_PORT, &[0xb0, 88, 127]);
    drive.frame();
    assert_eq!(drive.app.pads.bank, 1);
    assert!(drive.app.notices.current().is_none());
    drive.bytes(PAD_PORT, &[0xb0, 88, 0]);
    drive.frame();

    drive
        .app
        .post_system_notice(Level::Error, ("same error".into(), Instant::now()));
    drive.frame();
    drive
        .app
        .post_system_notice(Level::Error, ("same error".into(), Instant::now()));
    drive.bytes(PAD_PORT, &[0xb0, 88, 127]);
    drive.frame();
    assert_eq!(drive.app.pads.bank, 2);
    assert_eq!(
        drive.app.notice().as_deref(),
        Some("same error"),
        "an identical replacement was not shown before input"
    );

    drive.bytes(XS_PORT, &[0xb0, 15, 64]);
    drive.frame();
    assert!(drive.app.notices.current().is_none());
    drive
        .app
        .post_system_notice(Level::Error, ("repeat test".into(), Instant::now()));
    drive.frame();
    drive.bytes(XS_PORT, &[0xb0, 15, 64]);
    drive.frame();
    assert_eq!(drive.app.notice().as_deref(), Some("repeat test"));
    drive.bytes(XS_PORT, &[0xb0, 15, 0]);
    drive.frame();
    assert!(
        drive.app.notices.current().is_none(),
        "movement to zero is not a button release"
    );

    drive.key(KeyCode::PageDown, M::NONE, KeyEventKind::Press);
    drive.frame();
    drive
        .app
        .post_system_notice(Level::Error, ("keyboard repeat".into(), Instant::now()));
    drive.frame();
    drive.key(KeyCode::PageDown, M::NONE, KeyEventKind::Press);
    drive.frame();
    assert_eq!(drive.app.notice().as_deref(), Some("keyboard repeat"));
    drive.key(KeyCode::PageDown, M::NONE, KeyEventKind::Release);
    drive.frame();
    assert_eq!(drive.app.notice().as_deref(), Some("keyboard repeat"));
    let bank = drive.app.pads.bank;
    drive.key(KeyCode::PageDown, M::NONE, KeyEventKind::Press);
    drive.frame();
    assert_ne!(drive.app.pads.bank, bank);
    assert!(drive.app.notices.current().is_none());
}

/// Scenario "Each produced result keeps its meaning".
#[test]
fn each_produced_result_keeps_its_meaning() {
    let mut drive = Drive::new();
    let path = PathBuf::from("/recorded/chop.wav");
    drive.app.load_gen[CHOP] = 2;
    drive.app.chop_loading = Some(path.clone());
    drive
        .app
        .post_system_notice(Level::Error, ("unrelated error".into(), Instant::now()));
    let history = drive.app.notices.history().clone();
    for generation in [1, 2] {
        drive.app.loader_reply(loader::Reply::Loaded {
            pad: CHOP,
            generation,
            path: path.clone(),
            sound: Arc::new(Sample::mono(vec![0.25; 1024])),
        });
        assert_eq!(drive.app.chop_loading.is_some(), generation == 1);
        assert_eq!(drive.app.notices.history(), &history);
        assert_eq!(drive.app.notice().as_deref(), Some("unrelated error"));
    }
    assert!(drive.app.chop.is_some());
    drive
        .app
        .result_notice(Err("write refused".into()), "same success label".into());
    assert_eq!(drive.last_notice().level, Level::Error);
    drive.app.result_notice(Ok(()), "same success label".into());
    assert_eq!(drive.last_notice().level, Level::Done);
    assert_eq!(drive.app.notice().as_deref(), Some("write refused"));
}

#[test]
fn notice_frame_uses_one_clock_for_expiry_text_and_level() {
    let mut drive = Drive::new();
    drive
        .app
        .add_keymap_warnings(vec!["waiting keymap warning".into()]);
    drive
        .app
        .post_notice(Level::Info, ("timed result".into(), Instant::now()));
    let deadline = drive.app.notices.deadline().unwrap();
    assert_eq!(
        drive
            .app
            .frame_notice_at(deadline - Duration::from_nanos(1))
            .as_deref(),
        Some("timed result")
    );
    assert_eq!(drive.app.frame_notice_level, Some(Level::Info));
    assert!(!drive.app.keymap_warning_shown);
    assert_eq!(
        drive.app.frame_notice_at(deadline).as_deref(),
        Some("waiting keymap warning")
    );
    assert_eq!(drive.app.frame_notice_level, Some(Level::Warn));
    assert!(drive.app.keymap_warning_shown);
    assert_eq!(drive.app.notices.history().len(), 2);
}

#[test]
fn notice_loading_state_follows_the_project_generation() {
    let mut drive = Drive::new();
    let old = drive.app.load_gen[CHOP];
    let path = PathBuf::from("/recorded/old-project.wav");
    drive.app.chop_loading = Some(path.clone());
    drive
        .app
        .post_system_notice(Level::Error, ("retained result".into(), Instant::now()));
    let history = drive.app.notices.history().clone();
    drive.app.apply_session(session::Session::default());
    assert!(drive.app.chop_loading.is_none());
    drive.app.loader_reply(loader::Reply::Loaded {
        pad: CHOP,
        generation: old,
        path,
        sound: Arc::new(Sample::mono(vec![0.25; 1024])),
    });
    assert!(
        drive.app.chop.is_none(),
        "the old project cannot restore its Chop view"
    );
    assert!(drive.app.chop_loading.is_none());
    assert_eq!(drive.app.notice().as_deref(), Some("retained result"));
    assert_eq!(drive.app.notices.history(), &history);
}

#[test]
fn input_prompts_retain_typing_priority() {
    for context in ["name", "filter", "project"] {
        for source in [Source::Keyboard, Source::Notes, Source::Button] {
            let mut drive = Drive::new();
            drive.load_pad(0, Arc::new(Sample::mono(vec![0.25; 1024])), "source");
            match context {
                "name" => {
                    drive.tap(KeyCode::Char('n'), M::CONTROL);
                }
                "filter" => {
                    drive.tap(KeyCode::F(6), M::NONE);
                    drive.tap(KeyCode::Char('/'), M::NONE);
                }
                "project" => {
                    drive.app.paths.projects = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../target/87-no-project-root");
                    drive.tap(KeyCode::Char('p'), M::CONTROL);
                    drive.tap(KeyCode::Char('q'), M::NONE);
                }
                _ => unreachable!(),
            }
            let text = |drive: &Drive| match context {
                "name" => drive.app.pad_naming.as_ref().unwrap().1.clone(),
                "filter" => drive.app.browser.filter.clone(),
                "project" => drive
                    .app
                    .project_picker
                    .as_ref()
                    .unwrap()
                    .naming
                    .clone()
                    .unwrap(),
                _ => unreachable!(),
            };
            let before = text(&drive);
            drive.app.loader_reply(loader::Reply::Failed {
                pad: PREVIEW,
                generation: drive.app.load_gen[PREVIEW],
                path: PathBuf::from("/recorded/missing.wav"),
                error: "read refused".into(),
            });
            drive.frame();
            let error = drive.last_notice().text.clone();
            assert!(drive.app.notices.current().is_none());
            assert_eq!(drive.last_notice().origin, Origin::System);
            let hit = drive.pad_press(source, 4);
            if matches!(source, Source::Keyboard) {
                if context == "filter" {
                    assert!(matches!(
                        hit.commands()[..],
                        [Command::Preview {
                            sound: None,
                            region: Region::WHOLE
                        }]
                    ));
                } else {
                    assert!(hit.commands().is_empty(), "typing is silent");
                }
                assert_eq!(text(&drive), format!("{before}q"));
            } else {
                assert_full_hit(&hit, 4);
                assert_eq!(
                    text(&drive),
                    before,
                    "direct MIDI keeps its existing route, not a fake text path"
                );
            }
            assert_only_release(&drive.pad_release(source, 4), 4);
            assert!(
                drive.app.notices.current().is_none(),
                "the hidden Error was not acknowledged"
            );
            assert_eq!(
                drive
                    .app
                    .notices
                    .history()
                    .iter()
                    .filter(|notice| notice.text == error)
                    .count(),
                1
            );
            drive.tap(KeyCode::Esc, M::NONE);
            if context == "project" {
                drive.tap(KeyCode::Esc, M::NONE);
            }
            assert_eq!(drive.app.notices.current().unwrap().level, Level::Error);
            let selected = drive.app.pads.selected;
            let volume = drive.app.pads.volume_db[selected];
            drive.tap(KeyCode::Right, M::NONE);
            assert_ne!(
                drive
                    .app
                    .notices
                    .current()
                    .map(|notice| notice.text.as_str()),
                Some(error.as_str()),
                "{context} {source:?}: the shown Error must end"
            );
            if context == "filter" {
                assert!(!drive.app.browser.editing);
                assert!(drive.app.notices.current().is_none());
            } else {
                let result = drive.app.notices.current().unwrap();
                assert_eq!(result.level, Level::Info);
                assert_eq!(result.origin, Origin::Keyboard);
                assert_eq!(
                    drive.app.pads.volume_db[selected],
                    volume + 1,
                    "acknowledgement must not consume the value step"
                );
            }
        }
    }
}

/// Scenario "F1 navigation stays inside the overlay".
#[test]
fn f1_navigation_stays_inside_the_overlay() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        let key = if matches!(source, Source::Notes) {
            input::midi::Key::Note(95)
        } else {
            input::midi::Key::Cc(95)
        };
        learn::assign(
            &mut drive.app.learned,
            "Recorded pads",
            input::Group::Global,
            key,
            "help.toggle",
        );
        drive
            .app
            .sync_learned("Recorded pads", input::Group::Global);
        drive.app.edit = true;
        drive.frame();
        let opened = drive.registry_command(source, KeyCode::F(1), M::NONE, 95);
        assert!(drive.app.help_open);
        assert!(opened.commands().is_empty());
        let before = drive.seen();
        let dirty = drive.app.session_dirty;
        for (code, mods) in [
            (KeyCode::Tab, M::NONE),
            (KeyCode::Down, M::NONE),
            (KeyCode::PageDown, M::NONE),
            (KeyCode::PageUp, M::NONE),
            (KeyCode::Up, M::NONE),
            (KeyCode::BackTab, M::SHIFT),
        ] {
            let frame = drive.tap(code, mods);
            assert!(frame.commands().is_empty(), "{source:?}: {code:?}");
            assert_eq!(drive.seen(), before, "{source:?}: {code:?}");
            assert_eq!(drive.app.session_dirty, dirty);
            assert!(drive.app.help_open);
        }
        let closed = drive.tap(KeyCode::Esc, M::NONE);
        assert!(!drive.app.help_open);
        assert!(closed.commands().is_empty());
        assert_eq!(drive.seen(), before);
        drive.tap(KeyCode::Tab, M::NONE);
        assert_ne!(drive.app.pads.param, before.param, "Tab works outside help");
    }
}

/// Scenario "F1 navigation stays inside the overlay" on every registered screen.
#[test]
fn f1_navigation_keeps_every_underlying_screen() {
    for source in [Source::Keyboard, Source::Button] {
        for (screen, key, number) in REGISTRY_SCREEN_COMMANDS {
            for (width, height) in [(80, 26), (120, 40)] {
                let mut drive = Drive::registry_screen_checks();
                drive.registry_command(source, KeyCode::F(2), M::NONE, 90);
                if screen != Screen::Pads {
                    drive.registry_command(source, key, M::NONE, number);
                }
                assert_eq!(drive.app.screen, screen);
                for index in 0..50 {
                    drive.app.post_system_notice(
                        Level::Info,
                        (format!("history {index}"), Instant::now()),
                    );
                }
                drive.registry_command(source, KeyCode::F(1), M::NONE, 95);
                drive.app.resize_help(width, height);
                let before = drive.seen();
                let dirty = drive.app.session_dirty;
                let selections = (
                    drive.app.fx_slot,
                    drive.app.fx_param,
                    drive.app.settings_row,
                    drive.app.settings_tab,
                    drive.app.browser.selected,
                    drive.app.slice_end,
                );
                for (code, mods) in [
                    (KeyCode::Tab, M::NONE),
                    (KeyCode::Down, M::NONE),
                    (KeyCode::PageDown, M::NONE),
                    (KeyCode::PageUp, M::NONE),
                    (KeyCode::Up, M::NONE),
                    (KeyCode::BackTab, M::SHIFT),
                    (KeyCode::Esc, M::NONE),
                ] {
                    assert!(
                        drive.tap(code, mods).commands().is_empty(),
                        "{source:?} {screen:?}: {code:?}"
                    );
                    assert_eq!(drive.seen(), before, "{source:?} {screen:?}: {code:?}");
                    assert_eq!(drive.app.session_dirty, dirty);
                    assert_eq!(
                        (
                            drive.app.fx_slot,
                            drive.app.fx_param,
                            drive.app.settings_row,
                            drive.app.settings_tab,
                            drive.app.browser.selected,
                            drive.app.slice_end
                        ),
                        selections
                    );
                }
                assert!(!drive.app.help_open);
            }
        }
    }
}

pub(super) fn f1_message_screen(drive: &mut Drive, width: u16, height: u16) -> Vec<String> {
    use ratatui::{Terminal, backend::TestBackend};
    drive.app.resize_help(width, height);
    let app = &mut drive.app;
    let pads = pads_view(&mut app.pads, None, None, Vec::new(), None, app.rate);
    let screen = tui::Screen::Pads(app.mark_pads(pads));
    let status = tui::Status {
        badges: app.title_badges(),
        save: app.save_status(),
        backend: None,
        recording: false,
        buffer_size: 128,
        sample_rate: 48_000,
        load_peak: 0.0,
        xruns: 0,
        master_db: app.master_db,
        meters: ([-60.0; 2], [-60.0; 2], false),
        notice: None,
        no_releases: false,
        project: app.project.as_ref().map(|project| project.name.as_str()),
    };
    let keys = full_help(&drive.keymap, app.screen, true);
    let entries = app.message_history();
    let help = tui::Help {
        groups: &[],
        hint: None,
        full: app.full_help_view(&keys, &entries),
        question: None,
        learn: None,
    };
    let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
    term.draw(|frame| tui::draw(frame, &status, &screen, &help))
        .unwrap();
    term.backend()
        .buffer()
        .content()
        .chunks(width as usize)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect())
        .collect()
}

/// Scenarios "F1 retains recent results in order" and "F1 history does not persist or edit work".
#[test]
fn f1_retains_recent_results_in_order() {
    use input::{Group, midi::Key};
    let mut drive = Drive::new();
    drive.bind_action(88, "pad.copy");
    learn::assign(
        &mut drive.app.learned,
        "X-Session Pro",
        Group::Global,
        Key::Note(60),
        "pad.copy",
    );
    drive.app.sync_learned("X-Session Pro", Group::Global);
    drive.frame();
    drive.load_pad(1, Arc::new(Sample::mono(vec![0.2; 4096])), "existing work");
    drive.tap(KeyCode::Right, M::NONE);
    assert!(!drive.app.history.is_empty());
    drive.app.notices = Notices::new(Instant::now());
    let mut expected = Vec::new();
    for index in 0..56 {
        let origin = match index % 4 {
            0 => {
                drive.tap(KeyCode::Char('c'), M::CONTROL);
                Origin::Keyboard
            }
            1 => {
                drive.bytes(PAD_PORT, &[0xb0, 88, 127]);
                drive.bytes(PAD_PORT, &[0xb0, 88, 0]);
                drive.frame();
                Origin::Device(PAD_PORT.into())
            }
            2 => {
                drive.bytes(XS_PORT, &[0x90, 60, 127]);
                drive.bytes(XS_PORT, &[0x80, 60, 0]);
                drive.frame();
                Origin::Device(XS_PORT.into())
            }
            _ => {
                drive.app.post_system_notice(
                    Level::Info,
                    (format!("system result {index:02}"), Instant::now()),
                );
                drive.frame();
                Origin::System
            }
        };
        assert_eq!(drive.last_notice().origin, origin);
        expected.push(drive.last_notice().clone());
    }
    let expected: Vec<_> = expected.into_iter().rev().take(50).collect();
    assert_eq!(
        drive
            .app
            .notices
            .history()
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        expected
    );
    drive.app.session_dirty = Some(Instant::now());
    drive.app.project = Some(Project {
        name: "history project".into(),
        folder: "target/87-history-project".into(),
    });
    let before = drive.seen();
    let dirty = drive.app.session_dirty;
    let project = (
        drive.app.project.as_ref().unwrap().name.clone(),
        drive.app.project.as_ref().unwrap().folder.clone(),
    );
    drive.tap(KeyCode::F(1), M::NONE);
    assert!(!drive.app.help_overlay.messages());
    drive.tap(KeyCode::Tab, M::NONE);
    let entries = drive.app.message_history();
    assert_eq!(entries.len(), 50);
    for (entry, stored) in entries.iter().zip(&expected) {
        assert_eq!(entry.text, stored.text);
        assert_eq!(
            entry.origin,
            match &stored.origin {
                Origin::Keyboard => "Keyboard",
                Origin::Device(port) => port,
                Origin::System => "System",
            }
        );
        assert_eq!(
            entry.level,
            if stored.level == Level::Warn {
                tui::NoticeLevel::Warn
            } else {
                tui::NoticeLevel::Info
            }
        );
    }
    for (width, height) in [(80, 26), (120, 40)] {
        let rows = f1_message_screen(&mut drive, width, height);
        assert!(rows[1].contains("[MESSAGES]"));
        assert!(rows[3].contains("system result 55"));
        let mut visible = rows.join("\n");
        for _ in 0..10 {
            let frame = drive.tap(KeyCode::PageDown, M::NONE);
            assert!(frame.commands().is_empty());
            visible.push_str(&f1_message_screen(&mut drive, width, height).join("\n"));
        }
        for name in ["Keyboard", PAD_PORT, XS_PORT, "System"] {
            assert!(visible.contains(name));
        }
        for index in (7..56).step_by(4) {
            assert!(visible.contains(&format!("system result {index:02}")));
        }
        for _ in 0..10 {
            drive.tap(KeyCode::PageUp, M::NONE);
        }
    }
    let history = drive.app.notices.history().clone();
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(!drive.app.help_open);
    assert_eq!(drive.seen(), before);
    assert_eq!(drive.app.session_dirty, dirty);
    assert_eq!(
        (
            drive.app.project.as_ref().unwrap().name.clone(),
            drive.app.project.as_ref().unwrap().folder.clone()
        ),
        project
    );
    assert_eq!(drive.app.notices.history(), &history);
    let restart = tests::test_app();
    assert!(restart.message_history().is_empty());
}

/// Scenario "F1 history handles empty and changing content".
#[test]
fn f1_history_handles_empty_and_changing_content() {
    let mut drive = Drive::new();
    drive.tap(KeyCode::F(1), M::NONE);
    let keys = f1_message_screen(&mut drive, 80, 26).join("\n");
    assert!(keys.contains("[HELP]"));
    drive.tap(KeyCode::Tab, M::NONE);
    assert!(drive.app.help_overlay.messages());
    assert!(
        f1_message_screen(&mut drive, 80, 26)
            .join("\n")
            .contains("No messages yet")
    );
    for code in [
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::PageUp,
        KeyCode::PageDown,
    ] {
        assert!(drive.tap(code, M::NONE).commands().is_empty());
        assert_eq!(drive.app.help_overlay.scroll, 0);
    }
    for index in 0..51 {
        drive
            .app
            .post_system_notice(Level::Info, (format!("result {index:02}"), Instant::now()));
    }
    f1_message_screen(&mut drive, 80, 26);
    drive.key(KeyCode::Down, M::NONE, KeyEventKind::Press);
    drive.frame();
    assert_eq!(drive.app.help_overlay.scroll, 1);
    drive.key(KeyCode::Down, M::NONE, KeyEventKind::Repeat);
    drive.frame();
    assert_eq!(drive.app.help_overlay.scroll, 2);
    drive.key(KeyCode::Down, M::NONE, KeyEventKind::Release);
    drive.frame();
    assert_eq!(drive.app.help_overlay.scroll, 2);
    drive.tap(KeyCode::PageDown, M::NONE);
    assert_eq!(
        drive.app.help_overlay.scroll,
        2 + tui::messages_page_height(26)
    );
    let at = drive.app.help_overlay.scroll;
    // a full round of the three tabs ends on Messages again
    for code in [KeyCode::Tab, KeyCode::Tab, KeyCode::Tab] {
        drive.key(code, M::NONE, KeyEventKind::Press);
        drive.frame();
        let page = drive.app.help_overlay.messages();
        drive.key(code, M::NONE, KeyEventKind::Repeat);
        drive.frame();
        assert_eq!(drive.app.help_overlay.messages(), page);
        drive.key(code, M::NONE, KeyEventKind::Release);
        drive.frame();
        assert_eq!(drive.app.help_overlay.scroll, at);
    }
    drive.app.post_system_notice(
        Level::Done,
        ("arrived while scrolled".into(), Instant::now()),
    );
    f1_message_screen(&mut drive, 80, 26);
    assert_eq!(drive.app.help_overlay.scroll, at);
    assert_eq!(
        drive.app.message_history()[0].text,
        "arrived while scrolled"
    );
    for _ in 0..20 {
        drive.tap(KeyCode::PageDown, M::NONE);
    }
    let bottom = drive.app.help_overlay.scroll;
    drive.tap(KeyCode::Down, M::NONE);
    assert_eq!(drive.app.help_overlay.scroll, bottom);
    f1_message_screen(&mut drive, 120, 40);
    assert!(drive.app.help_overlay.scroll < bottom);
    assert_eq!(
        drive.app.help_overlay.scroll,
        tui::message_lines(&drive.app.message_history(), 118).len() - tui::messages_page_height(40)
    );
    drive.app.post_system_notice(
        Level::Error,
        ("background error under help".into(), Instant::now()),
    );
    drive.frame();
    assert!(
        drive.app.notices.current().is_none(),
        "help hides the live notice"
    );
    let history = drive.app.notices.history().clone();
    drive.key(KeyCode::F(1), M::NONE, KeyEventKind::Press);
    drive.frame();
    assert!(!drive.app.help_open);
    assert_eq!(drive.app.notices.current().unwrap().level, Level::Error);
    drive.key(KeyCode::F(1), M::NONE, KeyEventKind::Repeat);
    drive.frame();
    assert!(!drive.app.help_open);
    assert_eq!(drive.app.notices.current().unwrap().level, Level::Error);
    drive.key(KeyCode::F(1), M::NONE, KeyEventKind::Release);
    drive.tap(KeyCode::F(1), M::NONE);
    assert!(!drive.app.help_overlay.messages());
    assert_eq!(drive.app.help_overlay.scroll, 0);
    assert_eq!(drive.app.notices.history(), &history);
    drive.tap(KeyCode::Tab, M::NONE);
    for _ in 0..20 {
        drive.tap(KeyCode::PageUp, M::NONE);
    }
    assert_eq!(drive.app.help_overlay.scroll, 0);
}

/// Scenario "F1 controls retain input priority and bindings".
#[test]
fn f1_controls_retain_input_priority_and_bindings() {
    let mut drive = Drive::new();
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.1; 4096])), "name");
    drive.tap(KeyCode::F(1), M::NONE);
    drive.tap(KeyCode::Char('n'), M::CONTROL);
    assert!(drive.app.pad_naming.is_some());
    drive.tap(KeyCode::Tab, M::NONE);
    assert!(!drive.app.help_overlay.messages());
    drive.key(KeyCode::Char('q'), M::NONE, KeyEventKind::Press);
    assert!(drive.frame().commands().is_empty());
    assert!(drive.app.pad_naming.as_ref().unwrap().1.ends_with('q'));
    drive.key(KeyCode::Char('q'), M::NONE, KeyEventKind::Release);
    // The existing keyboard route forwards releases even when typing consumed the press.
    assert_only_release(&drive.frame(), 4);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.pad_naming.is_none() && drive.app.help_open);
    drive.tap(KeyCode::Char('p'), M::CONTROL);
    assert!(drive.app.project_picker.is_some());
    drive.tap(KeyCode::Tab, M::NONE);
    assert!(!drive.app.help_overlay.messages());
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.project_picker.is_none() && drive.app.help_open);
    drive.tap(KeyCode::F(6), M::NONE);
    drive.tap(KeyCode::Char('/'), M::NONE);
    assert!(drive.app.browser.editing);
    drive.tap(KeyCode::Tab, M::NONE);
    assert!(!drive.app.help_overlay.messages());
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(!drive.app.browser.editing && drive.app.help_open);
    drive.tap(KeyCode::F(2), M::NONE);
    drive.tap(KeyCode::F(10), M::NONE);
    assert!(drive.app.screen_learn.is_some());
    drive.tap(KeyCode::Tab, M::NONE);
    assert!(!drive.app.help_overlay.messages());
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.screen_learn.is_none() && drive.app.help_open);
    drive.tap(KeyCode::Char('a'), M::CONTROL);
    drive.tap(KeyCode::Backspace, M::NONE);
    assert!(drive.app.ask.is_some());
    drive.tap(KeyCode::Tab, M::NONE);
    assert!(!drive.app.help_overlay.messages());
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.ask.is_none() && drive.app.help_open);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(!drive.app.help_open);
    assert!(
        drive
            .keymap
            .overlay("[global]\nF9 = \"help.toggle\"\nF1 = \"none\"\n")
            .unwrap()
            .is_empty()
    );
    drive.tap(KeyCode::F(9), M::NONE);
    assert!(drive.app.help_open);
    drive.tap(KeyCode::Tab, M::NONE);
    drive.tap(KeyCode::F(9), M::NONE);
    assert!(!drive.app.help_open);
    drive.tap(KeyCode::F(9), M::NONE);
    assert!(!drive.app.help_overlay.messages());
}

/// Scenario "F1 controls retain input priority and bindings" for settings pickers and MIDI learning.
#[test]
fn f1_keeps_settings_picker_and_learning_priority() {
    let mut drive = Drive::new();
    drive.tap(KeyCode::F(12), M::NONE);
    for _ in 0..3 {
        drive.tap(KeyCode::Tab, M::NONE);
    }
    drive.app.paths.projects = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(drive.app.path_picker.is_some());
    // The picker already owns F1. Open help before the picker instead.
    drive.tap(KeyCode::Esc, M::NONE);
    drive.tap(KeyCode::F(1), M::NONE);
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(drive.app.path_picker.is_some() && drive.app.help_open);
    drive.tap(KeyCode::Tab, M::NONE);
    assert!(!drive.app.help_overlay.messages());
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.path_picker.is_none() && drive.app.help_open);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(!drive.app.help_open);
    for _ in 0..3 {
        drive.tap(KeyCode::Tab, M::NONE);
    }
    assert_eq!(drive.app.settings_tab, 2);
    let group = drive
        .app
        .midi_list()
        .iter()
        .position(|row| {
            matches!(
                row,
                MidiListRow::Group {
                    group: input::Group::Global
                }
            )
        })
        .unwrap();
    for _ in 0..group {
        drive.tap(KeyCode::Down, M::NONE);
    }
    drive.tap(KeyCode::Right, M::NONE);
    let action = drive
        .app
        .midi_list()
        .iter()
        .position(|row| matches!(row, MidiListRow::Action { .. }))
        .unwrap();
    for _ in group..action {
        drive.tap(KeyCode::Down, M::NONE);
    }
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(drive.app.learning.is_some());
    drive.tap(KeyCode::F(1), M::NONE);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.learning.is_none() && drive.app.help_open);
    drive.tap(KeyCode::Esc, M::NONE);
    let set = drive
        .app
        .midi_list()
        .iter()
        .position(|row| matches!(row, MidiListRow::Set { .. }))
        .unwrap();
    for _ in 0..action.abs_diff(set) {
        drive.tap(
            if set > action {
                KeyCode::Down
            } else {
                KeyCode::Up
            },
            M::NONE,
        );
    }
    assert!(matches!(
        drive.app.midi_list()[drive.app.midi_row],
        MidiListRow::Set { .. }
    ));
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(drive.app.midi_run.is_some());
    drive.tap(KeyCode::F(1), M::NONE);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.midi_run.is_none() && drive.app.help_open);
}

/// Scenario "Pad releases survive F1 navigation".
#[test]
fn pad_releases_survive_f1_navigation() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.load_pad(0, Arc::new(Sample::mono(vec![0.1; 4096])), "gate");
        drive.frame();
        assert_full_hit(&drive.pad_press(source, 0), 0);
        drive.tap(KeyCode::F(1), M::NONE);
        drive.tap(KeyCode::Tab, M::NONE);
        assert_only_release(&drive.pad_release(source, 0), 0);
        assert!(!drive.app.pads.held[0]);
        assert_full_hit(&drive.pad_press(source, 0), 0);
        drive.tap(KeyCode::BackTab, M::SHIFT);
        drive.tap(KeyCode::Esc, M::NONE);
        assert_only_release(&drive.pad_release(source, 0), 0);
        assert!(!drive.app.pads.held[0]);
        let repeated = drive.pad_release(source, 0);
        if matches!(source, Source::Keyboard) {
            // HeldPads forwards each physical release, also outside help.
            assert_only_release(&repeated, 0);
        } else {
            assert!(repeated.commands().is_empty());
        }
    }
}

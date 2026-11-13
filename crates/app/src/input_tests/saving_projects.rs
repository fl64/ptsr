use super::*;

/// Scenario "Keyboard and controller save feedback agree".
/// Scenario "Repeated manual refusals each explain the reason".
/// Scenario "Successful retry clears the save warning".
#[test]
fn keyboard_and_controller_save_feedback_agree() {
    use tui::SaveStatus;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/86b-input-saves-{}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    let _cleanup = Cleanup(root.clone());
    for project in [false, true] {
        for problem in ["none", "write", "damaged", "newer"] {
            let path = root.join("project.toml");
            let mut expected = None;
            for source in [Source::Keyboard, Source::Button] {
                let mut drive = Drive::new();
                drive.bind_action(88, "session.save");
                drive.bind_action(89, "master.volume_up");
                drive.app.session_path = path.clone();
                drive.app.session_read_error = None;
                if project {
                    drive.app.project = Some(Project {
                        name: "test".into(),
                        folder: root.clone(),
                    });
                }
                session::save(&path, &session::Session::default()).unwrap();
                if matches!(problem, "damaged" | "newer") {
                    std::fs::write(
                        &path,
                        if problem == "damaged" {
                            "version = [\n"
                        } else {
                            "version = 99\n"
                        },
                    )
                    .unwrap();
                    drive.app.session_read_error = Some(session::load(&path).unwrap_err());
                }
                let tmp = path.with_extension("toml.tmp");
                if problem == "write" {
                    std::fs::write(&tmp, "occupied").unwrap();
                }
                let saved = std::fs::read(&path).unwrap();
                assert_eq!(drive.app.save_status(), SaveStatus::Clean);
                drive.registry_command(source, KeyCode::Char(']'), M::NONE, 89);
                assert_eq!(drive.app.save_status(), SaveStatus::Unsaved);
                let pending = drive.app.session_dirty;
                let before = drive.seen();
                for _ in 0..2 {
                    drive.app.notices = Notices::new(Instant::now());
                    drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 88);
                    let notice = drive.last_notice().text.clone();
                    let at = Instant::now();
                    assert_eq!(notice == "Session saved", problem == "none");
                    if problem == "none" {
                        let deadline = drive.app.notices.deadline().unwrap();
                        assert_eq!(
                            drive.app.notice_at(deadline - Duration::from_nanos(1)),
                            Some(notice.clone())
                        );
                        assert_eq!(drive.app.notice_at(deadline), None);
                    } else {
                        assert_eq!(
                            drive.app.notice_at(at + Duration::from_secs(60)),
                            Some(notice.clone())
                        );
                        assert_eq!(drive.last_notice().level, Level::Error);
                    }
                    assert_eq!(
                        drive.app.session_dirty,
                        if problem == "none" { None } else { pending }
                    );
                    assert_eq!(drive.app.session_write_failed, problem == "write");
                    assert_eq!(
                        drive.app.save_status(),
                        match problem {
                            "none" => SaveStatus::Clean,
                            "write" => SaveStatus::Failed,
                            _ => SaveStatus::Unsaved,
                        }
                    );
                    assert_eq!(drive.app.screen, before.screen);
                    assert_eq!(drive.app.history.len(), before.history.len());
                    let bytes = std::fs::read(&path).unwrap();
                    if problem != "none" {
                        assert_eq!(bytes, saved);
                    } else {
                        assert_eq!(session::load(&path).unwrap().master_db, drive.app.master_db);
                    }
                    let result = (notice.clone(), bytes, drive.app.save_status());
                    if let Some(expected) = &expected {
                        assert_eq!(
                            &result, expected,
                            "{source:?}, {problem}, project {project}"
                        );
                    } else {
                        expected = Some(result);
                    }
                    drive
                        .app
                        .post_notice(Level::Info, ("another notice".into(), at));
                    assert_eq!(drive.app.notice_at(at), Some(notice));
                }
                if problem == "write" {
                    std::fs::remove_file(&tmp).unwrap();
                    drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 88);
                    assert_eq!(drive.last_notice().text, "Session saved");
                    assert_eq!(drive.app.save_status(), SaveStatus::Clean);
                    drive.registry_command(source, KeyCode::Char(']'), M::NONE, 89);
                    assert_eq!(drive.app.save_status(), SaveStatus::Unsaved);
                    drive.app.notices = Notices::new(Instant::now());
                    drive.app.autosave(
                        drive.app.session_dirty.unwrap() + SESSION_SAVE + Duration::from_nanos(1),
                    );
                    assert_eq!(drive.app.save_status(), SaveStatus::Clean);
                    assert!(drive.app.notices.history().is_empty());
                    assert_eq!(session::load(&path).unwrap().master_db, drive.app.master_db);
                }
            }
        }
    }
}

/// Scenarios "The manual save speaks" and "Interrupted notices retain remaining time".
#[test]
fn manual_save_success_keeps_five_visible_seconds_through_real_inputs() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for project in [false, true] {
            let files = TransitionFiles::new();
            let mut drive = files.save_feedback_drive(project, source);
            drive.registry_command(source, KeyCode::Char(']'), M::NONE, 93);
            let before = Instant::now();
            drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
            let after = Instant::now();
            let saved = drive.last_notice().clone();
            assert_eq!(saved.text, "Session saved");
            assert_eq!(saved.level, Level::Done);
            assert_eq!(
                saved.origin,
                match source {
                    Source::Keyboard => Origin::Keyboard,
                    _ => Origin::Device(PAD_PORT.into()),
                }
            );
            assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
            assert_eq!(
                session::load_existing(&drive.app.session_file())
                    .unwrap()
                    .master_db,
                drive.app.master_db
            );
            let deadline = drive.app.notices.deadline().unwrap();
            assert!(deadline >= before + Duration::from_secs(5));
            assert!(deadline <= after + Duration::from_secs(5));
            let start = deadline - Duration::from_secs(5);
            drive.app.post_system_notice(
                Level::Warn,
                ("background warning".into(), start + Duration::from_secs(1)),
            );
            assert_eq!(
                drive
                    .app
                    .frame_notice_at(start + Duration::from_secs(1))
                    .as_deref(),
                Some("background warning")
            );
            assert_eq!(drive.app.frame_notice_level, Some(Level::Warn));
            assert_eq!(
                drive
                    .app
                    .frame_notice_at(start + Duration::from_secs(7))
                    .as_deref(),
                Some("Session saved")
            );
            assert_eq!(drive.app.frame_notice_level, Some(Level::Done));
            let resumed_deadline = start + Duration::from_secs(11);
            assert_eq!(drive.app.notices.deadline(), Some(resumed_deadline));
            assert_eq!(
                drive
                    .app
                    .frame_notice_at(resumed_deadline - Duration::from_nanos(1))
                    .as_deref(),
                Some("Session saved")
            );
            assert_eq!(drive.app.frame_notice_at(resumed_deadline), None);
            assert_eq!(drive.app.frame_notice_level, None);
            assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
            assert_eq!(drive.app.notices.history().len(), 2);
            assert_eq!(drive.app.notices.history()[1], saved);
        }
    }
}

/// Scenarios "Repeated manual refusals each explain the reason",
/// "Keyboard and controller save feedback agree", "An informational result cannot hide an error",
/// "A save failure stays visible after its notice" and "Successful retry clears the save warning".
#[test]
fn save_refusals_keep_priority_and_state_until_a_real_retry_succeeds() {
    use tui::SaveStatus;

    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for project in [false, true] {
            for problem in ["write", "damaged", "newer"] {
                let files = TransitionFiles::new();
                let mut drive = files.save_feedback_drive(project, source);
                let path = drive.app.session_file();
                let valid = std::fs::read(&path).unwrap();
                let tmp = path.with_extension("toml.tmp");
                if problem == "write" {
                    std::fs::write(&tmp, b"occupied").unwrap();
                } else {
                    std::fs::write(
                        &path,
                        if problem == "damaged" {
                            "version = [\n"
                        } else {
                            "version = 99\n"
                        },
                    )
                    .unwrap();
                    drive.app.session_read_error = Some(session::load_existing(&path).unwrap_err());
                }
                drive.registry_command(source, KeyCode::Char(']'), M::NONE, 93);
                let before = TransitionFiles::state(&drive);
                let pending = drive.app.session_dirty;
                let protection = drive.app.session_read_error.clone();
                let saved = std::fs::read(&path).unwrap();
                let status = if problem == "write" {
                    SaveStatus::Failed
                } else {
                    SaveStatus::Unsaved
                };
                let origin = match source {
                    Source::Keyboard => Origin::Keyboard,
                    _ => Origin::Device(PAD_PORT.into()),
                };
                let mut produced = Vec::new();
                assert!(drive.app.notices.history().is_empty());
                for _ in 0..2 {
                    let frame = drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
                    assert!(frame.commands().is_empty());
                    let refusal = drive.last_notice().clone();
                    assert_eq!(refusal.level, Level::Error);
                    assert_eq!(refusal.origin, origin);
                    assert_ne!(refusal.text, "Session saved");
                    produced.push(refusal.clone());
                    for level in [Level::Info, Level::Done] {
                        drive.app.post_system_notice(
                            level,
                            (format!("background {level:?}"), Instant::now()),
                        );
                        produced.push(drive.last_notice().clone());
                        drive.frame();
                        assert_eq!(drive.app.notices.current(), Some(&refusal));
                        assert_eq!(drive.app.frame_notice_level, Some(Level::Error));
                    }
                    assert_eq!(TransitionFiles::state(&drive), before);
                    assert_eq!(drive.app.save_status(), status);
                    assert_eq!(drive.app.session_dirty, pending);
                    assert_eq!(drive.app.session_read_error, protection);
                    assert_eq!(std::fs::read(&path).unwrap(), saved);
                    if problem == "write" {
                        assert_eq!(std::fs::read(&tmp).unwrap(), b"occupied");
                    }
                    assert_eq!(
                        drive.app.notices.history().iter().collect::<Vec<_>>(),
                        produced.iter().rev().collect::<Vec<_>>()
                    );
                }
                let bank = drive.app.pads.bank;
                drive.registry_command(source, KeyCode::PageDown, M::NONE, 92);
                assert_eq!(
                    drive.app.pads.bank,
                    bank + 1,
                    "acknowledgement keeps the action"
                );
                assert_eq!(drive.app.notices.current().unwrap().level, Level::Done);
                assert_eq!(drive.app.save_status(), status);
                assert_eq!(drive.app.session_dirty, pending);
                assert_eq!(drive.app.session_read_error, protection);
                assert_eq!(std::fs::read(&path).unwrap(), saved);
                let history = drive.app.notices.history().clone();
                if problem == "write" {
                    std::fs::remove_file(&tmp).unwrap();
                } else {
                    // Repairing the bytes alone must not remove startup read protection.
                    std::fs::write(&path, &valid).unwrap();
                }
                drive
                    .app
                    .autosave(pending.unwrap() + SESSION_SAVE + Duration::from_secs(1));
                drive.app.save_on_exit();
                assert_eq!(drive.app.notices.history(), &history);
                assert_eq!(drive.app.session_dirty, pending);
                assert_eq!(
                    &std::fs::read(&path).unwrap(),
                    if problem == "write" { &saved } else { &valid }
                );
                drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
                assert_eq!(drive.last_notice().origin, origin);
                assert_eq!(drive.app.notices.history().len(), history.len() + 1);
                if problem == "write" {
                    assert_eq!(drive.last_notice().text, "Session saved");
                    assert_eq!(drive.last_notice().level, Level::Done);
                    assert_eq!(drive.app.save_status(), SaveStatus::Clean);
                    assert!(drive.app.session_dirty.is_none());
                    assert!(!drive.app.session_write_failed);
                    assert_eq!(
                        session::load_existing(&path).unwrap().master_db,
                        drive.app.master_db
                    );
                    drive.registry_command(source, KeyCode::Char(']'), M::NONE, 93);
                    assert_eq!(drive.app.save_status(), SaveStatus::Unsaved);
                    let history = drive.app.notices.history().clone();
                    drive.app.autosave(
                        drive.app.session_dirty.unwrap() + SESSION_SAVE + Duration::from_nanos(1),
                    );
                    assert_eq!(drive.app.save_status(), SaveStatus::Clean);
                    assert_eq!(
                        drive.app.notices.history(),
                        &history,
                        "successful autosave stays silent"
                    );
                    assert_eq!(
                        session::load_existing(&path).unwrap().master_db,
                        drive.app.master_db
                    );
                } else {
                    assert_eq!(drive.last_notice().level, Level::Error);
                    assert!(
                        drive
                            .last_notice()
                            .text
                            .contains(protection.as_ref().unwrap())
                    );
                    assert_eq!(drive.app.session_read_error, protection);
                    assert_eq!(drive.app.session_dirty, pending);
                    assert_eq!(drive.app.save_status(), SaveStatus::Unsaved);
                    assert_eq!(std::fs::read(&path).unwrap(), valid);
                    assert!(
                        drive
                            .app
                            .notices
                            .history()
                            .iter()
                            .all(|notice| notice.text != "Session saved")
                    );
                }
            }
        }
    }
}

/// Scenario "The save action appears in GLOBAL".
/// Scenario "Save learning waits for confirmation".
/// Scenario "The learned save button survives restart".
/// Scenario "The learned save button retries after write refusal".
#[test]
fn the_learned_save_button_survives_restart() {
    use input::{Group, midi::Key};
    use tui::SaveStatus;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/86b-learn-save-{}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    let _cleanup = Cleanup(root.clone());
    for source in [Source::Notes, Source::Button] {
        for project in [false, true] {
            let dir = root.join(format!("{source:?}-{project}"));
            std::fs::create_dir(&dir).unwrap();
            let config_path = dir.join("config.toml");
            let session_path = dir.join(if project {
                "project.toml"
            } else {
                "session.toml"
            });
            std::fs::write(&config_path, "").unwrap();
            session::save(&session_path, &session::Session::default()).unwrap();
            let key = match source {
                Source::Notes => Key::Note(88),
                Source::Button => Key::Cc(88),
                Source::Keyboard => unreachable!(),
            };
            let configure_session = |drive: &mut Drive| {
                drive.app.session_path = session_path.clone();
                drive.app.session_read_error = None;
                if project {
                    drive.app.project = Some(Project {
                        name: "learn save".into(),
                        folder: dir.clone(),
                    });
                }
            };
            let expected_bindings = {
                let mut drive = Drive::with_saved_bindings(false, Some(&config_path));
                configure_session(&mut drive);
                drive.tap(KeyCode::Char(']'), M::NONE);
                let pending_work = drive.app.session_dirty;
                drive.tap(KeyCode::F(12), M::NONE);
                for _ in 0..2 {
                    drive.tap(KeyCode::Tab, M::NONE);
                }
                assert_eq!(drive.app.settings_tab, 2);
                let global = drive
                    .app
                    .midi_list()
                    .iter()
                    .position(|row| {
                        matches!(
                            row,
                            MidiListRow::Group {
                                group: Group::Global
                            }
                        )
                    })
                    .unwrap();
                for _ in 0..global {
                    drive.tap(KeyCode::Down, M::NONE);
                }
                assert_eq!(drive.app.midi_row, global);
                drive.tap(KeyCode::Enter, M::NONE);
                assert!(drive.app.midi_open.contains(&Group::Global));
                let rows = drive.app.midi_list();
                let saves: Vec<_> = rows
                    .iter()
                    .enumerate()
                    .filter_map(|(at, row)| match row {
                        MidiListRow::Action { info } if info.name == "session.save" => {
                            assert_eq!(info.group, Group::Global);
                            assert_eq!(info.label, "save session");
                            assert!(!info.knob);
                            Some(at)
                        }
                        _ => None,
                    })
                    .collect();
                assert_eq!(saves.len(), 1);
                for _ in global..saves[0] {
                    drive.tap(KeyCode::Down, M::NONE);
                }
                assert_eq!(drive.app.midi_row, saves[0]);
                assert_eq!(
                    drive
                        .app
                        .midi_screen()
                        .iter()
                        .filter(|row| matches!(row,
                            tui::MidiRow::Action { label, .. } if label == "save session"
                        ))
                        .count(),
                    1
                );
                for confirm in [false, true] {
                    let old_bindings = drive.app.learned.clone();
                    let config_bytes = std::fs::read(&config_path).unwrap();
                    let session_bytes = std::fs::read(&session_path).unwrap();
                    drive.tap(KeyCode::Enter, M::NONE);
                    let learning = drive.app.learning.as_ref().unwrap();
                    assert_eq!(learning.info.name, "session.save");
                    assert!(learning.pending.is_none());
                    drive.bytes(
                        PAD_PORT,
                        &match source {
                            Source::Notes => [0x80, 88, 0],
                            Source::Button => [0xb0, 88, 0],
                            Source::Keyboard => unreachable!(),
                        },
                    );
                    drive.frame();
                    assert!(drive.app.learning.as_ref().unwrap().pending.is_none());
                    let frame = drive.registry_command(source, KeyCode::Null, M::NONE, 88);
                    assert!(frame.commands().is_empty());
                    assert_eq!(
                        drive.app.learning.as_ref().unwrap().pending,
                        Some(("Recorded pads".into(), key))
                    );
                    assert!(drive.app.midi_screen().iter().any(|row| matches!(row,
                        tui::MidiRow::Action { label, who, .. }
                            if label == "save session" && who.contains('→')
                    )));
                    assert_eq!(drive.app.learned, old_bindings);
                    assert_eq!(std::fs::read(&config_path).unwrap(), config_bytes);
                    assert_eq!(std::fs::read(&session_path).unwrap(), session_bytes);
                    assert_eq!(drive.app.session_dirty, pending_work);
                    drive.tap(
                        if confirm {
                            KeyCode::Enter
                        } else {
                            KeyCode::Esc
                        },
                        M::NONE,
                    );
                    assert!(drive.app.learning.is_none());
                    if confirm {
                        assert_eq!(
                            drive.app.learned["Recorded pads"]["global"][&key.to_string()],
                            "session.save"
                        );
                        assert_eq!(
                            config::load_from(&config_path).config.midi,
                            drive.app.learned
                        );
                    } else {
                        assert_eq!(drive.app.learned, old_bindings);
                        assert_eq!(std::fs::read(&config_path).unwrap(), config_bytes);
                    }
                    assert_eq!(std::fs::read(&session_path).unwrap(), session_bytes);
                    assert_eq!(drive.app.session_dirty, pending_work);
                }
                drive.app.learned.clone()
            };
            let mut drive = Drive::with_saved_bindings(false, Some(&config_path));
            configure_session(&mut drive);
            assert_eq!(drive.app.learned, expected_bindings);
            let config_bytes = std::fs::read(&config_path).unwrap();
            drive.load_pad(0, Arc::new(Sample::mono(vec![0.25; 1024])), "learn-save");
            drive.app.browser.dir = dir.clone();
            for (screen, code, _) in REGISTRY_SCREEN_COMMANDS {
                drive.tap(KeyCode::F(2), M::NONE);
                drive.tap(code, M::NONE);
                assert_eq!(drive.app.screen, screen);
                drive.tap(KeyCode::Char(']'), M::NONE);
                drive.registry_command(source, KeyCode::Null, M::NONE, 88);
                assert_eq!(drive.last_notice().text, "Session saved");
                assert_eq!(drive.app.save_status(), SaveStatus::Clean);
                assert_eq!(
                    session::load(&session_path).unwrap().master_db,
                    drive.app.master_db
                );
                assert_eq!(drive.app.screen, screen);
            }
            drive.tap(KeyCode::F(2), M::NONE);
            drive.tap(KeyCode::Char(']'), M::NONE);
            let pending_work = drive.app.session_dirty;
            let saved = std::fs::read(&session_path).unwrap();
            let tmp = session_path.with_extension("toml.tmp");
            std::fs::write(&tmp, "occupied").unwrap();
            for _ in 0..2 {
                drive.app.notices = Notices::new(Instant::now());
                drive.registry_command(source, KeyCode::Null, M::NONE, 88);
                assert_ne!(drive.last_notice().text, "Session saved");
                assert_eq!(drive.app.save_status(), SaveStatus::Failed);
                assert_eq!(drive.app.session_dirty, pending_work);
                assert_eq!(std::fs::read(&session_path).unwrap(), saved);
            }
            std::fs::remove_file(&tmp).unwrap();
            drive.registry_command(source, KeyCode::Null, M::NONE, 88);
            assert_eq!(drive.last_notice().text, "Session saved");
            assert_eq!(drive.app.save_status(), SaveStatus::Clean);
            drive.tap(KeyCode::Char(']'), M::NONE);
            drive.app.notices = Notices::new(Instant::now());
            drive.app.autosave(
                drive.app.session_dirty.unwrap() + SESSION_SAVE + Duration::from_nanos(1),
            );
            assert_eq!(drive.app.save_status(), SaveStatus::Clean);
            assert!(drive.app.notices.history().is_empty());
            assert_eq!(
                session::load(&session_path).unwrap().master_db,
                drive.app.master_db
            );
            assert_eq!(std::fs::read(&config_path).unwrap(), config_bytes);
        }
    }
}

/// Scenario "Project edits survive an immediate switch".
/// Scenario "Scratch edits save before opening or creating a project".
/// Scenario "Keyboard and MIDI project entry share transition safety".
#[test]
fn project_edits_survive_an_immediate_switch() {
    for source in [Source::Keyboard, Source::Button] {
        for project in [false, true] {
            for create in [false, true] {
                let files = TransitionFiles::new();
                let mut drive = files.drive(project);
                TransitionFiles::edit(&mut drive, source);
                let source_file = drive.app.session_file();
                let expected_gate = drive.app.pads.gate[0];
                let expected_chop = drive.app.chop.as_ref().unwrap().saved();
                let target = if create { "C" } else { "B" };
                TransitionFiles::pick(&mut drive, source, target, create);
                drive.tap(KeyCode::Enter, M::NONE);
                assert!(drive.app.project_picker.is_none());
                assert_eq!(drive.app.project.as_ref().unwrap().name, target);
                assert!(drive.app.session_dirty.is_none());
                assert!(drive.app.history.is_empty());
                let saved = session::load_existing(&source_file).unwrap();
                assert_eq!(saved.pads["0"].gate, expected_gate);
                assert_eq!(
                    saved.chops.values().next().unwrap(),
                    &expected_chop.to_state(drive.app.rate)
                );
                if project {
                    TransitionFiles::pick(&mut drive, source, "A", false);
                    drive.tap(KeyCode::Enter, M::NONE);
                    assert_eq!(drive.app.project.as_ref().unwrap().name, "A");
                    assert_eq!(drive.app.pads.gate[0], expected_gate);
                    assert_eq!(
                        drive.app.pending_chops[&files.0.join("tone.wav")],
                        expected_chop
                    );
                }
            }
        }
    }
}

/// Scenario "A save refusal keeps the selected target".
/// Scenario "A save refusal keeps the new project name".
/// Scenario "Pending protected work blocks a transition".
/// Scenario "A refused transition explains its reason".
#[test]
fn a_save_refusal_keeps_the_selected_target() {
    for source in [Source::Keyboard, Source::Button] {
        for project in [false, true] {
            for create in [false, true] {
                for protected in [false, true] {
                    let files = TransitionFiles::new();
                    let mut drive = files.drive(project);
                    TransitionFiles::edit(&mut drive, source);
                    let source_file = drive.app.session_file();
                    let tmp = source_file.with_extension("toml.tmp");
                    if protected {
                        std::fs::write(&source_file, "version = 99\n").unwrap();
                        drive.app.session_read_error =
                            Some(session::load(&source_file).unwrap_err());
                    } else {
                        std::fs::create_dir(&tmp).unwrap();
                    }
                    let reason = drive.app.session_read_error.clone();
                    let bytes = std::fs::read(&source_file).unwrap();
                    let config_bytes =
                        std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap();
                    let target_bytes = std::fs::read(files.0.join("B/project.toml")).unwrap();
                    let pending = drive.app.session_dirty;
                    let expected_gate = drive.app.pads.gate[0];
                    let generations = drive.app.load_gen.clone();
                    let cuts = drive.app.chop.as_ref().unwrap().saved();
                    let target = if create { "C" } else { "B" };
                    TransitionFiles::pick(&mut drive, source, target, create);
                    let before = TransitionFiles::state(&drive);
                    let picker = TransitionFiles::picker(&drive);
                    let history_len = drive.app.notices.history().len();
                    for attempt in 1..=2 {
                        drive.tap(KeyCode::Enter, M::NONE);
                        assert_eq!(drive.last_notice().level, Level::Error);
                        assert_eq!(drive.last_notice().origin, Origin::Keyboard);
                        assert_eq!(drive.app.notices.history().len(), history_len + attempt);
                        assert_eq!(TransitionFiles::state(&drive), before);
                        assert_eq!(TransitionFiles::picker(&drive), picker);
                        assert_eq!(drive.app.chop.as_ref().unwrap().saved(), cuts);
                        assert_eq!(drive.app.load_gen, generations);
                        assert_eq!(drive.app.session_dirty, pending);
                        assert_eq!(drive.app.session_read_error, reason);
                        assert_eq!(drive.app.session_write_failed, !protected);
                        assert_eq!(
                            drive.app.project.as_ref().map(|p| p.name.as_str()),
                            project.then_some("A")
                        );
                        let notice = &drive.last_notice().text;
                        assert_ne!(notice, "Session saved");
                        assert!(!notice.starts_with("Project "));
                        assert!(drive.app.ask.is_none());
                        assert_eq!(std::fs::read(&source_file).unwrap(), bytes);
                        assert_eq!(
                            std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap(),
                            config_bytes
                        );
                        assert_eq!(
                            std::fs::read(files.0.join("B/project.toml")).unwrap(),
                            target_bytes
                        );
                        assert!(!files.0.join("C").exists());
                    }
                    let refusal = drive.last_notice().clone();
                    for level in [Level::Info, Level::Done] {
                        drive.app.post_system_notice(
                            level,
                            ("background result".into(), Instant::now()),
                        );
                    }
                    assert_eq!(drive.app.notices.history().len(), history_len + 4);
                    drive.tap(KeyCode::Esc, M::NONE);
                    if create {
                        // Esc first leaves the existing name line, then closes the picker.
                        assert!(drive.app.project_picker.is_some());
                        drive.tap(KeyCode::Esc, M::NONE);
                    }
                    assert!(drive.app.project_picker.is_none());
                    assert_eq!(drive.app.notices.current(), Some(&refusal));
                    assert_eq!(drive.app.frame_notice_level, Some(Level::Error));
                    assert_eq!(
                        drive.app.save_status(),
                        if protected {
                            tui::SaveStatus::Unsaved
                        } else {
                            tui::SaveStatus::Failed
                        }
                    );
                    assert_eq!(drive.app.session_dirty, pending);
                    assert_eq!(drive.app.session_read_error, reason);
                    assert_eq!(std::fs::read(&source_file).unwrap(), bytes);
                    TransitionFiles::pick(&mut drive, source, target, create);
                    assert_eq!(TransitionFiles::picker(&drive), picker);
                    assert_eq!(TransitionFiles::state(&drive), before);
                    if !protected {
                        std::fs::remove_dir(&tmp).unwrap();
                        drive.tap(KeyCode::Enter, M::NONE);
                        assert!(drive.app.project_picker.is_none());
                        assert_eq!(drive.app.project.as_ref().unwrap().name, target);
                        let saved = session::load_existing(&source_file).unwrap();
                        assert_eq!(saved.pads["0"].gate, expected_gate);
                        assert_eq!(
                            saved.chops.values().next().unwrap(),
                            &cuts.to_state(drive.app.rate)
                        );
                    }
                }
            }
        }
    }
}

/// Scenario "A target read refusal preserves the source".
#[test]
fn a_target_read_refusal_preserves_the_source() {
    for source in [Source::Keyboard, Source::Button] {
        for project in [false, true] {
            for fault in ["damaged", "newer", "unreadable", "missing"] {
                let files = TransitionFiles::new();
                let mut drive = files.drive(project);
                TransitionFiles::edit(&mut drive, source);
                TransitionFiles::pick(&mut drive, source, "B", false);
                let path = files.0.join("B/project.toml");
                match fault {
                    "damaged" => std::fs::write(&path, "version = [\n").unwrap(),
                    "newer" => std::fs::write(&path, "version = 99\n").unwrap(),
                    _ => {
                        std::fs::remove_file(&path).unwrap();
                        if fault == "unreadable" {
                            std::fs::create_dir(&path).unwrap();
                            std::fs::write(path.join("keep"), "unrelated").unwrap();
                        }
                    }
                }
                let target_bytes = std::fs::read(&path).ok();
                // A target refusal must happen before a source write attempt.
                let source_file = drive.app.session_file();
                std::fs::create_dir(source_file.with_extension("toml.tmp")).unwrap();
                drive.app.session_read_error = Some("source remains protected".into());
                let reason = drive.app.session_read_error.clone();
                let bytes = std::fs::read(&source_file).unwrap();
                let config_bytes = std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap();
                let pending = drive.app.session_dirty;
                let before = TransitionFiles::state(&drive);
                let picker = TransitionFiles::picker(&drive);
                let cuts = drive.app.chop.as_ref().unwrap().saved();
                let generations = drive.app.load_gen.clone();
                let history_len = drive.app.notices.history().len();
                for attempt in 1..=2 {
                    drive.tap(KeyCode::Enter, M::NONE);
                    assert!(drive.last_notice().text.starts_with("Project not opened:"));
                    assert_eq!(drive.last_notice().level, Level::Error);
                    assert_eq!(drive.last_notice().origin, Origin::Keyboard);
                    assert_eq!(drive.app.notices.history().len(), history_len + attempt);
                    assert_eq!(TransitionFiles::state(&drive), before);
                    assert_eq!(TransitionFiles::picker(&drive), picker);
                    assert_eq!(drive.app.chop.as_ref().unwrap().saved(), cuts);
                    assert_eq!(drive.app.load_gen, generations);
                    assert_eq!(drive.app.session_dirty, pending);
                    assert_eq!(drive.app.session_read_error, reason);
                    assert!(!drive.app.session_write_failed);
                    assert_eq!(
                        drive.app.project_cfg.last.as_deref(),
                        project.then_some("A")
                    );
                    assert_eq!(std::fs::read(&source_file).unwrap(), bytes);
                    assert_eq!(
                        std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap(),
                        config_bytes
                    );
                    assert_eq!(std::fs::read(&path).ok(), target_bytes);
                    assert_eq!(path.exists(), fault != "missing");
                    if fault == "unreadable" {
                        assert_eq!(
                            std::fs::read_to_string(path.join("keep")).unwrap(),
                            "unrelated"
                        );
                    }
                }
            }
        }
    }
}

/// Scenario "Selecting the current project keeps pending work".
#[test]
fn selecting_the_current_project_keeps_pending_work() {
    use std::os::unix::fs::symlink;
    use std::sync::atomic::Ordering::Acquire;
    for source in [Source::Keyboard, Source::Button] {
        for alias in [false, true] {
            let files = TransitionFiles::new();
            let mut drive = files.drive(true);
            TransitionFiles::edit(&mut drive, source);
            let path = drive.app.session_file();
            std::fs::create_dir(path.with_extension("toml.tmp")).unwrap();
            drive.tap(KeyCode::Char('s'), M::CONTROL);
            assert!(drive.app.session_write_failed);
            std::fs::write(&path, "version = [\n").unwrap();
            let (audio, _rt) = audio::offline(48_000);
            let epoch = audio.project.epoch.load(Acquire);
            drive.app.audio = Some(audio);
            let target = if alias {
                symlink(files.0.join("A"), files.0.join("alias")).unwrap();
                "alias"
            } else {
                "A"
            };
            TransitionFiles::pick(&mut drive, source, target, false);
            let before = TransitionFiles::state(&drive);
            let pending = drive.app.session_dirty;
            let generations = drive.app.load_gen.clone();
            let cuts = drive.app.chop.as_ref().unwrap().saved();
            let sound = drive.app.pads.sound[0].as_ref().unwrap().clone();
            let config_bytes = std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap();
            let frame = drive.tap(KeyCode::Enter, M::NONE);
            assert!(frame.commands().is_empty());
            assert!(drive.app.project_picker.is_none());
            assert_eq!(TransitionFiles::state(&drive), before);
            assert_eq!(drive.app.session_dirty, pending);
            assert!(drive.app.session_write_failed);
            assert_eq!(drive.app.load_gen, generations);
            assert_eq!(drive.app.chop.as_ref().unwrap().saved(), cuts);
            assert!(Arc::ptr_eq(
                drive.app.pads.sound[0].as_ref().unwrap(),
                &sound
            ));
            assert_eq!(
                drive
                    .app
                    .audio
                    .as_ref()
                    .unwrap()
                    .project
                    .epoch
                    .load(Acquire),
                epoch
            );
            assert_eq!(drive.app.project.as_ref().unwrap().name, "A");
            assert_eq!(std::fs::read(&path).unwrap(), b"version = [\n");
            assert_eq!(
                std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap(),
                config_bytes
            );
        }
    }
}

/// Scenario "A target creation refusal retains saved source work".
#[test]
fn a_target_creation_refusal_retains_saved_source_work() {
    for source in [Source::Keyboard, Source::Button] {
        for project in [false, true] {
            let files = TransitionFiles::new();
            let mut drive = files.drive(project);
            TransitionFiles::edit(&mut drive, source);
            let path = drive.app.session_file();
            std::fs::write(files.0.join("C"), "occupied").unwrap();
            TransitionFiles::pick(&mut drive, source, "C", true);
            let before = TransitionFiles::state(&drive);
            let picker = TransitionFiles::picker(&drive);
            let expected_gate = drive.app.pads.gate[0];
            let config_bytes = std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap();
            drive.tap(KeyCode::Enter, M::NONE);
            assert_eq!(TransitionFiles::state(&drive), before);
            assert_eq!(TransitionFiles::picker(&drive), picker);
            assert!(drive.app.session_dirty.is_none());
            assert!(!drive.app.session_write_failed);
            assert_eq!(
                drive.app.project_cfg.last.as_deref(),
                project.then_some("A")
            );
            assert_eq!(
                session::load_existing(&path).unwrap().pads["0"].gate,
                expected_gate
            );
            assert_eq!(
                std::fs::read_to_string(files.0.join("C")).unwrap(),
                "occupied"
            );
            assert_eq!(
                std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap(),
                config_bytes
            );
            std::fs::remove_file(files.0.join("C")).unwrap();
            drive.tap(KeyCode::Enter, M::NONE);
            assert!(drive.app.project_picker.is_none());
            assert_eq!(drive.app.project.as_ref().unwrap().name, "C");
        }
    }
}

#[test]
fn a_missing_source_folder_blocks_switching_until_restored() {
    for source in [Source::Keyboard, Source::Button] {
        for create in [false, true] {
            let files = TransitionFiles::new();
            let mut drive = files.drive(true);
            TransitionFiles::edit(&mut drive, source);
            let folder = files.0.join("A");
            let moved = files.0.join("moved");
            std::fs::rename(&folder, &moved).unwrap();
            let saved = std::fs::read(moved.join("project.toml")).unwrap();
            let target = if create { "C" } else { "B" };
            TransitionFiles::pick(&mut drive, source, target, create);
            let before = TransitionFiles::state(&drive);
            let picker = TransitionFiles::picker(&drive);
            let pending = drive.app.session_dirty;
            for _ in 0..2 {
                drive.tap(KeyCode::Enter, M::NONE);
                assert!(drive.last_notice().text.contains("is gone"));
                assert_eq!(TransitionFiles::state(&drive), before);
                assert_eq!(TransitionFiles::picker(&drive), picker);
                assert_eq!(drive.app.session_dirty, pending);
                assert!(drive.app.session_write_failed);
                assert!(!folder.exists());
                assert!(!files.0.join("C").exists());
                assert_eq!(std::fs::read(moved.join("project.toml")).unwrap(), saved);
            }
            std::fs::rename(&moved, &folder).unwrap();
            drive.tap(KeyCode::Enter, M::NONE);
            assert!(drive.app.project_picker.is_none());
            assert_eq!(drive.app.project.as_ref().unwrap().name, target);
            assert!(drive.app.session_dirty.is_none());
            assert!(!drive.app.session_write_failed);
        }
    }
}

/// Scenario "Clean protected scratch can leave without a write".
#[test]
fn clean_protected_scratch_can_leave_without_a_write() {
    for source in [Source::Keyboard, Source::Button] {
        let files = TransitionFiles::new();
        let mut drive = files.drive(false);
        let path = drive.app.session_file();
        std::fs::write(&path, "version = 99\n").unwrap();
        drive.app.session_read_error = Some(session::load(&path).unwrap_err());
        assert!(drive.app.session_dirty.is_none());
        TransitionFiles::pick(&mut drive, source, "B", false);
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.project_picker.is_none());
        assert_eq!(drive.app.project.as_ref().unwrap().name, "B");
        assert!(drive.app.session_read_error.is_none());
        assert_eq!(std::fs::read(&path).unwrap(), b"version = 99\n");
    }
}

struct PortableProject {
    root: PathBuf,
    file: PathBuf,
    internal: bool,
    states: Vec<session::ChopState>,
    pads: std::collections::BTreeMap<String, session::PadState>,
}

impl PortableProject {
    fn new(internal: bool) -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/86d-portable")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Relaxed)
            ));
        for name in ["A", "B"] {
            std::fs::create_dir_all(root.join(name).join("samples")).unwrap();
        }
        let file = if internal {
            root.join("A/samples/tone.wav")
        } else {
            root.join("tone.wav")
        };
        let wave: Vec<_> = (0..48_000)
            .map(|i| 0.12 + 0.07 * (i as f32 * 0.03).sin())
            .collect();
        engine::offline::write_wav(&file, RENDER_RATE, &wave, &wave).unwrap();
        let states: Vec<_> = [
            (3, vec![0.125, 0.375, 0.625, 0.875], vec![4, 8, 12, 16]),
            (7, vec![0.2, 0.4, 0.6, 0.8], vec![5, 9, 13, 17]),
            (11, vec![0.25, 0.5, 0.75], vec![6, 10, 14]),
        ]
        .into_iter()
        .map(|(epoch, cuts, ids)| session::ChopState {
            start: 0.0,
            end: 1.0,
            cuts,
            ids,
            epoch,
            next_id: 99,
        })
        .collect();
        let pads = [
            (0.125, 0.375, 4, 3, 2, -6, 3),
            (0.2, 0.4, 5, 7, 2, -3, -5),
            (0.5, 0.75, 10, 11, 3, 0, 7),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (start, end, link, epoch, slice, volume_db, pitch))| {
            (
                i.to_string(),
                session::PadState {
                    file: Some(file.clone()),
                    name: Some(format!("variant {}", i + 1)),
                    start: Some(start),
                    end: Some(end),
                    link: Some(link),
                    epoch: Some(epoch),
                    slice: Some(slice),
                    gate: false,
                    volume_db,
                    pitch,
                    release_ms: 10,
                    filter: "low-pass".into(),
                    cutoff_hz: Some(2_000.0),
                    reso: 10,
                    drive: 15,
                    ..Default::default()
                },
            )
        })
        .collect();
        let sess = session::Session {
            browser_dir: Some(root.join("A/samples")),
            pads,
            chops: [(file.to_string_lossy().into_owned(), states[2].clone())].into(),
            kept: [(file.to_string_lossy().into_owned(), states[..2].to_vec())].into(),
            ..Default::default()
        };
        session::save_project(&root.join("A/project.toml"), &sess).unwrap();
        session::save_project(&root.join("B/project.toml"), &session::Session::default()).unwrap();
        Self {
            root,
            file,
            internal,
            states,
            pads: sess.pads,
        }
    }

    fn drive(&self, source: Source) -> Drive {
        let mut drive = Drive::offline();
        for (cc, action) in [
            (88, "project.pick"),
            (89, "session.save"),
            (90, "screen.chop"),
            (91, "slice.start_right"),
        ] {
            drive.bind_action(cc, action);
        }
        drive.app.paths.projects = self.root.clone();
        drive.app.session_path = self.root.join("scratch.toml");
        drive.app.user_path = Some(self.root.join("config.toml"));
        drive.app.session_read_error = None;
        Self::open(&mut drive, source, "A");
        drive
    }

    fn open(drive: &mut Drive, source: Source, name: &str) {
        TransitionFiles::pick(drive, source, name, false);
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.project_picker.is_none());
        assert_eq!(drive.app.project.as_ref().unwrap().name, name);
        while drive.app.pads.loading.iter().any(Option::is_some) {
            let reply = drive
                .app
                .loader
                .as_ref()
                .unwrap()
                .1
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            drive.app.loader_reply(reply);
            drive.app.flush_audio();
            drive.render_blocks(1);
        }
        let epoch = drive
            .project
            .epoch
            .load(std::sync::atomic::Ordering::Acquire);
        // The production switch clears FX buffers in bounded steps. As in the
        // realtime switch tests, advance callbacks until the actual ready marker.
        for _ in 0..300 {
            drive.app.flush_audio();
            drive.render_blocks(1);
            if drive.app.pending_audio.is_empty()
                && drive
                    .project
                    .ready
                    .load(std::sync::atomic::Ordering::Acquire)
                    == epoch
            {
                break;
            }
        }
        assert!(drive.app.pending_audio.is_empty());
        assert_eq!(
            drive
                .project
                .ready
                .load(std::sync::atomic::Ordering::Acquire),
            epoch
        );
        drive.frame();
        drive.render_blocks(32);
    }

    fn f3(drive: &mut Drive, source: Source, pad: usize) {
        drive.tap(KeyCode::F(2), M::NONE);
        drive.pad_press(source, pad);
        drive.pad_release(source, pad);
        drive.registry_command(source, KeyCode::F(3), M::NONE, 90);
        assert_eq!(drive.app.screen, Screen::Chop);
    }

    fn check_variants(&self, drive: &mut Drive, source: Source, file: &Path) {
        for (pad, expected) in self.states.iter().enumerate() {
            Self::f3(drive, source, pad);
            let chop = drive.app.chop.as_ref().unwrap();
            assert_eq!(chop.file, file);
            assert_eq!(chop.saved().to_state(RENDER_RATE), *expected);
            drive.render_blocks(32);
        }
        let pads = drive.app.pads.to_session(None, RENDER_RATE).pads;
        for (key, expected) in &self.pads {
            let mut expected = expected.clone();
            expected.file = Some(file.to_path_buf());
            assert_eq!(pads[key], expected);
        }
    }

    fn save(drive: &mut Drive, source: Source) {
        drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 89);
        assert_eq!(drive.last_notice().text, "Session saved");
        assert!(drive.app.session_dirty.is_none());
    }

    fn check_stored(&self, name: &str) {
        let sess = session::load_existing(&self.root.join(name).join("project.toml")).unwrap();
        let stored = if self.internal {
            Path::new("samples/tone.wav")
        } else {
            &self.file
        };
        let key = stored.to_string_lossy();
        assert_eq!(sess.version, 5);
        assert_eq!(sess.chops.len(), 1);
        assert_eq!(sess.kept.len(), 1);
        assert_eq!(sess.chops[key.as_ref()], self.states[2]);
        assert_eq!(sess.kept[key.as_ref()], self.states[..2]);
        assert!(
            sess.pads
                .values()
                .all(|pad| pad.file.as_deref() == Some(stored))
        );
        assert_eq!(
            std::fs::read_dir(self.root.join(name).join("samples"))
                .unwrap()
                .count(),
            usize::from(self.internal)
        );
    }

    fn render(drive: &mut Drive, source: Source) -> Stereo {
        use engine::checks::{first_loud, first_non_finite, peak};
        let mut all = (Vec::new(), Vec::new());
        drive.tap(KeyCode::F(2), M::NONE);
        drive.render_blocks(32);
        for pad in 0..3 {
            let mut wave = drive.render_blocks(16);
            drive.pad_press(source, pad);
            append_stereo(&mut wave, drive.render_blocks(128));
            drive.pad_release(source, pad);
            for channel in [&wave.0, &wave.1] {
                assert_eq!(first_non_finite(channel), None);
                assert!(peak(channel) < 1.0);
                assert_eq!(peak(&channel[..2048]), 0.0);
                let onset = first_loud(channel, 2048..2176, 1e-6).unwrap();
                assert!(
                    onset.abs_diff(2048) <= 1,
                    "{source:?}, pad {pad}, onset {onset}"
                );
                assert!(peak(&channel[2048..]) > 0.01);
                assert_eq!(peak(&channel[channel.len() - 1024..]), 0.0);
            }
            append_stereo(&mut all, wave);
        }
        all
    }
}

impl Drop for PortableProject {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

/// Scenario "Internal variant paths move with the project".
/// Scenario "External variant paths remain absolute".
/// Scenario "Save and reopen after project relocation".
/// Scenario "Moving keeps unassigned cuts of older chops".
/// Scenario "Editing a moved chop changes only its linked pads".
/// Scenario "Moved slices keep their sound".
#[test]
fn portable_chops_keep_all_variants_and_sound() {
    let mut all = (Vec::new(), Vec::new());
    for source in [Source::Keyboard, Source::Button] {
        for internal in [true, false] {
            let files = PortableProject::new(internal);
            let bytes = std::fs::read(&files.file).unwrap();
            let mut drive = files.drive(source);
            files.check_variants(&mut drive, source, &files.file);
            let before = PortableProject::render(&mut drive, source);
            PortableProject::save(&mut drive, source);
            files.check_stored("A");
            std::fs::rename(files.root.join("A"), files.root.join("Moved")).unwrap();
            let moved_file = if internal {
                files.root.join("Moved/samples/tone.wav")
            } else {
                files.file.clone()
            };
            assert!(!files.root.join("A").exists());
            PortableProject::open(&mut drive, source, "Moved");
            files.check_variants(&mut drive, source, &moved_file);
            let after = PortableProject::render(&mut drive, source);
            assert_eq!(after, before, "{source:?}, internal {internal}");
            assert_eq!(std::fs::read(&moved_file).unwrap(), bytes);
            append_stereo(&mut all, before);
            append_stereo(&mut all, after);
            PortableProject::save(&mut drive, source);
            files.check_stored("Moved");
            PortableProject::open(&mut drive, source, "B");
            PortableProject::open(&mut drive, source, "Moved");
            files.check_variants(&mut drive, source, &moved_file);
            PortableProject::f3(&mut drive, source, 0);
            let other_pads = [drive.app.pads.snapshot(1), drive.app.pads.snapshot(2)];
            let original = drive.app.pads.region[0];
            drive.registry_command(source, KeyCode::Char('l'), M::CONTROL, 91);
            assert_ne!(drive.app.pads.region[0], original);
            assert_eq!(
                [drive.app.pads.snapshot(1), drive.app.pads.snapshot(2)],
                other_pads
            );
            let edited = drive.app.chop.as_ref().unwrap().saved();
            for pad in [1, 2] {
                PortableProject::f3(&mut drive, source, pad);
                assert_eq!(
                    drive
                        .app
                        .chop
                        .as_ref()
                        .unwrap()
                        .saved()
                        .to_state(RENDER_RATE),
                    files.states[pad]
                );
            }
            PortableProject::f3(&mut drive, source, 0);
            assert_eq!(drive.app.chop.as_ref().unwrap().saved(), edited);
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/86d-portable-chops.wav");
    engine::offline::write_wav(&path, RENDER_RATE, &all.0, &all.1).unwrap();
    println!(
        "86d render: {} frames, before/after sample-identical, finite/no-clip/silence/onset +/-1 checked",
        all.0.len()
    );
}

/// Scenario "Missing kept variants use the existing fallback".
/// Scenario "Stale absolute variant paths are not guessed".
#[test]
fn portable_chops_keep_old_project_compatibility() {
    for source in [Source::Keyboard, Source::Button] {
        for stale in [false, true] {
            let files = PortableProject::new(true);
            let path = files.root.join("A/project.toml");
            let mut sess = session::load_existing(&path).unwrap();
            sess.kept.clear();
            let absent = files.root.join("already-gone/tone.wav");
            if stale {
                sess.kept.insert(
                    absent.to_string_lossy().into_owned(),
                    files.states[..2].to_vec(),
                );
            }
            session::save_project(&path, &sess).unwrap();
            let mut drive = files.drive(source);
            if stale {
                assert!(
                    drive
                        .app
                        .pending_older
                        .iter()
                        .all(|(file, _)| file == &absent)
                );
                assert_eq!(drive.app.pending_older.len(), 2);
            } else {
                assert!(drive.app.pending_older.is_empty());
            }
            PortableProject::f3(&mut drive, source, 0);
            let saved = drive.app.chop.as_ref().unwrap().saved();
            assert_eq!(saved.epoch, 3);
            let cuts: Vec<_> = saved.cuts.iter().map(|(_, at)| *at).collect();
            assert_eq!(cuts, [6000, 18000]);
            assert_eq!(drive.app.pads.link[0], Some((4, 3)));
            assert_eq!(drive.app.pads.pitch[0], 3);
            assert_eq!(drive.app.pads.volume_db[0], -6);
            assert_eq!(session::load_existing(&path).unwrap().version, 5);
        }
    }
}

use super::*;

fn numbered_copy_drive() -> Drive {
    let mut drive = Drive::new();
    for (cc, action) in [
        (85, "pad.edit"),
        (86, "session.save"),
        (87, "edit.undo"),
        (88, "pad.clear"),
        (89, "bank.prev"),
    ] {
        drive.bind_action(cc, action);
    }
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.25; 1024])), "kick");
    drive
}

impl Drive {
    // Controller commands use the profile's CC buttons; pad targets also test raw notes.
    fn copy_command(&mut self, source: Source, key: KeyCode, mods: M, cc: u8) {
        let route = if matches!(source, Source::Keyboard) {
            Source::Keyboard
        } else {
            Source::Button
        };
        self.registry_command(route, key, mods, cc);
    }

    fn numbered_copy_bank(&mut self, source: Source, abs: usize) {
        let bank = abs / 16;
        if matches!(source, Source::Keyboard) {
            self.tap(KeyCode::Char(char::from(b'1' + bank as u8)), M::CONTROL);
        } else {
            for _ in 0..engine::BANKS {
                if self.app.pads.bank == bank {
                    break;
                }
                let (key, cc) = if self.app.pads.bank > bank {
                    (KeyCode::PageUp, 89)
                } else {
                    (KeyCode::PageDown, 27)
                };
                self.copy_command(source, key, M::NONE, cc);
            }
        }
        assert_eq!(self.app.pads.bank, bank, "{source:?}");
    }

    fn numbered_copy_start(&mut self, source: Source, abs: usize) {
        self.numbered_copy_bank(source, abs);
        self.pad_press(source, abs % 16);
        self.pad_release(source, abs % 16);
        self.copy_command(source, KeyCode::Char('c'), M::CONTROL, 25);
        assert!(self.app.copy_in_hand.is_some());
    }

    fn numbered_copy_target(&mut self, source: Source, abs: usize) {
        self.numbered_copy_bank(source, abs);
        assert_silent_target(&self.pad_press(source, abs % 16));
        self.pad_release(source, abs % 16);
    }

    fn numbered_copy_confirm(&mut self, source: Source) {
        self.copy_command(source, KeyCode::Enter, M::NONE, 85);
    }
}

/// 87a Scenario "Repeated source copies use numbered names".
#[test]
fn repeated_source_copies_use_numbered_names() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = numbered_copy_drive();
        let original = drive.app.pads.snapshot(0);
        drive.app.pads.marked.insert(0);
        for (number, target) in [1, 2, 18].into_iter().enumerate() {
            drive.numbered_copy_start(source, 0);
            drive.numbered_copy_target(source, target);
            let mut expected = original.clone();
            expected.name = Some(format!("kick (copy {})", number + 1));
            assert_eq!(drive.app.pads.snapshot(target), expected, "{source:?}");
            assert_eq!(drive.app.pads.snapshot(0), original);
            assert_eq!(drive.app.pads.marked, [0].into());
            assert!(drive.app.copy_in_hand.is_none());
        }
        assert_eq!(drive.app.history.len(), 3);
    }
}

/// 87a Scenario "A copy of a copy replaces its marker".
#[test]
fn a_copy_of_a_copy_replaces_its_marker() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = numbered_copy_drive();
        drive.numbered_copy_start(source, 0);
        drive.numbered_copy_target(source, 1);
        drive.numbered_copy_start(source, 1);
        drive.numbered_copy_target(source, 18);
        assert_eq!(pad_name(&drive.app.pads, 0), "kick");
        assert_eq!(pad_name(&drive.app.pads, 1), "kick (copy 1)");
        assert_eq!(pad_name(&drive.app.pads, 18), "kick (copy 2)");
    }
}

/// 87a Scenario "Slice numbers remain separate from copy numbers".
#[test]
fn slice_numbers_remain_separate_from_copy_numbers() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = numbered_copy_drive();
        drive.app.pads.custom_name[0] = None;
        drive.app.pads.file[0] = Some("break.wav".into());
        drive.app.pads.slice[0] = Some(2);
        drive.app.pads.link[0] = Some((7, 3));
        drive.app.pads.region[0] = Region {
            start: 100,
            end: 500,
        };
        drive.numbered_copy_start(source, 0);
        drive.numbered_copy_target(source, 1);
        assert_eq!(pad_name(&drive.app.pads, 1), "break 2 (copy 1)");
        assert_eq!(drive.app.pads.link[1], Some((7, 3)));
        assert_eq!(drive.app.pads.region[1], drive.app.pads.region[0]);
        assert_eq!(drive.app.pads.custom_name[0], None);
    }
}

/// 87a Scenario "Only a canonical final copy marker identifies the base".
#[test]
fn only_a_canonical_final_copy_marker_identifies_the_base() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for (name, expected) in [
            ("kick (copy 01)", "kick (copy 01) (copy 1)"),
            ("kick (Copy 1)", "kick (Copy 1) (copy 1)"),
            ("kick (copy 0)", "kick (copy 0) (copy 1)"),
            ("kick (copy )", "kick (copy ) (copy 1)"),
            ("kick (copy -1)", "kick (copy -1) (copy 1)"),
            ("kick (copy １)", "kick (copy １) (copy 1)"),
            ("kick (copy 1) tail", "kick (copy 1) tail (copy 1)"),
            (
                "kick (copy 999999999999999999999999999999)",
                "kick (copy 1)",
            ),
        ] {
            let mut drive = numbered_copy_drive();
            drive.app.pads.custom_name[0] = Some(name.into());
            drive.numbered_copy_start(source, 0);
            drive.numbered_copy_target(source, 1);
            assert_eq!(pad_name(&drive.app.pads, 1), expected, "{source:?}: {name}");
            assert_eq!(pad_name(&drive.app.pads, 0), name);
        }
    }
}

/// 87a Scenario "A name in another bank reserves its number".
#[test]
fn a_name_in_another_bank_reserves_its_number() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for state in ["loaded", "missing", "loading"] {
            let mut drive = numbered_copy_drive();
            drive.load_pad(
                64,
                drive.app.pads.sound[0].clone().unwrap(),
                "kick (copy 1)",
            );
            drive.app.pads.missing[64] = state == "missing";
            drive.app.pads.loading[64] = (state == "loading").then_some(0);
            if state == "missing" {
                drive.app.pads.sound[64] = None;
            }
            drive.numbered_copy_start(source, 0);
            drive.numbered_copy_target(source, 1);
            assert_eq!(pad_name(&drive.app.pads, 1), "kick (copy 2)");
            assert_eq!(pad_name(&drive.app.pads, 64), "kick (copy 1)");
        }
    }
}

/// 87a Scenario "Copy numbers reuse the smallest gap".
#[test]
fn copy_numbers_reuse_the_smallest_gap() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = numbered_copy_drive();
        let sound = drive.app.pads.sound[0].clone().unwrap();
        drive.load_pad(1, sound.clone(), "kick (copy 1)");
        drive.load_pad(2, sound, "kick (copy 3)");
        for _ in 0..2 {
            drive.numbered_copy_start(source, 2);
            drive.numbered_copy_target(source, 18);
            assert_eq!(pad_name(&drive.app.pads, 18), "kick (copy 2)");
            drive.copy_command(source, KeyCode::Char('z'), M::CONTROL, 87);
            assert!(drive.app.pads.sound[18].is_none());
        }
        drive.numbered_copy_start(source, 0);
        drive.numbered_copy_target(source, 18);
        drive.copy_command(source, KeyCode::Backspace, M::NONE, 88);
        assert!(drive.app.pads.sound[18].is_none());
        drive.numbered_copy_start(source, 0);
        drive.numbered_copy_target(source, 18);
        assert_eq!(pad_name(&drive.app.pads, 18), "kick (copy 2)");
    }
}

/// A full set of 128 names still leaves candidate 129 available without a counter.
#[test]
fn copy_names_are_bounded_by_current_pad_count() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = numbered_copy_drive();
        let sound = drive.app.pads.sound[0].clone().unwrap();
        for abs in 0..PADS {
            drive.load_pad(abs, sound.clone(), &format!("kick (copy {})", abs + 1));
        }
        drive.numbered_copy_start(source, 0);
        drive.numbered_copy_target(source, 1);
        assert!(drive.app.ask.as_ref().unwrap().lines()[1].ends_with("kick (copy 129)"));
        drive.numbered_copy_confirm(source);
        assert_eq!(pad_name(&drive.app.pads, 1), "kick (copy 129)");
    }
}

/// 87a Scenario "Replacement includes the current target name".
#[test]
fn replacement_includes_the_current_target_name() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = numbered_copy_drive();
        drive.load_pad(1, drive.app.pads.sound[0].clone().unwrap(), "kick (copy 1)");
        let before = drive.app.pads.snapshot(1);
        drive.numbered_copy_start(source, 0);
        drive.numbered_copy_target(source, 1);
        assert_eq!(drive.app.pads.snapshot(1), before);
        assert!(drive.app.ask.as_ref().unwrap().lines()[1].ends_with("kick (copy 2)"));
        drive.numbered_copy_confirm(source);
        assert_eq!(pad_name(&drive.app.pads, 1), "kick (copy 2)");
        assert_eq!(pad_name(&drive.app.pads, 0), "kick");
        drive.copy_command(source, KeyCode::Char('z'), M::CONTROL, 87);
        assert_eq!(drive.app.pads.snapshot(1), before);
    }
}

/// 87a Scenario "Other projects do not reserve copy numbers".
#[test]
fn other_projects_do_not_reserve_copy_numbers() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let files = TransitionFiles::new();
        let mut other = session::Session::default();
        other.pads.insert(
            "0".into(),
            session::PadState {
                name: Some("kick (copy 1)".into()),
                ..Default::default()
            },
        );
        let path = files.0.join("B/project.toml");
        session::save_project(&path, &other).unwrap();
        let before = std::fs::read(&path).unwrap();
        let mut drive = numbered_copy_drive();
        drive.app.paths.projects = files.0.clone();
        drive.numbered_copy_start(source, 0);
        drive.numbered_copy_target(source, 1);
        assert_eq!(pad_name(&drive.app.pads, 1), "kick (copy 1)");
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}

/// 87a Scenario "Unicode copy names use full text rather than clipped cells".
#[test]
fn unicode_copy_names_use_full_text_rather_than_clipped_cells() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = numbered_copy_drive();
        let base = format!("{}Бочка e\u{301}", "界".repeat(40));
        let different = format!("{base}! (copy 1)");
        drive.app.pads.custom_name[0] = Some(base.clone());
        drive.load_pad(64, drive.app.pads.sound[0].clone().unwrap(), &different);
        drive.numbered_copy_start(source, 0);
        drive.numbered_copy_target(source, 1);
        assert_eq!(pad_name(&drive.app.pads, 1), format!("{base} (copy 1)"));
        drive.numbered_copy_start(source, 1);
        drive.numbered_copy_target(source, 2);
        assert_eq!(pad_name(&drive.app.pads, 2), format!("{base} (copy 2)"));
        assert_eq!(pad_name(&drive.app.pads, 0), base);
        assert_eq!(pad_name(&drive.app.pads, 64), different);
    }
}

/// 87a Scenario "Copy naming keeps the pickup name".
#[test]
fn copy_naming_keeps_the_pickup_name() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for named in [false, true] {
            let mut drive = numbered_copy_drive();
            if !named {
                drive.app.pads.custom_name[0] = None;
            }
            drive.numbered_copy_start(source, 0);
            drive.app.pads.custom_name[0] = Some("snare".into());
            drive.app.pads.pitch[0] = -7;
            drive.numbered_copy_target(source, 1);
            assert_eq!(pad_name(&drive.app.pads, 1), "kick (copy 1)");
            assert_eq!(drive.app.pads.pitch[1], 3);
            assert_eq!(pad_name(&drive.app.pads, 0), "snare");
            assert_eq!(drive.app.pads.pitch[0], -7);
        }
    }
}

/// 87a Scenario "A changed proposed copy name asks again".
#[test]
fn a_changed_proposed_copy_name_asks_again() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = numbered_copy_drive();
        drive.load_pad(18, drive.app.pads.sound[0].clone().unwrap(), "old target");
        let before = drive.app.pads.snapshot(18);
        drive.numbered_copy_start(source, 0);
        drive.numbered_copy_target(source, 18);
        assert!(drive.app.ask.as_ref().unwrap().lines()[1].ends_with("kick (copy 1)"));
        drive.app.pads.custom_name[64] = Some("kick (copy 1)".into());
        drive.numbered_copy_confirm(source);
        assert_eq!(drive.app.pads.snapshot(18), before);
        assert!(drive.app.ask.as_ref().unwrap().lines()[1].ends_with("kick (copy 2)"));
        assert!(drive.app.history.is_empty());
        drive.app.pads.custom_name[64] = None;
        drive.numbered_copy_confirm(source);
        assert_eq!(drive.app.pads.snapshot(18), before);
        assert!(drive.app.ask.as_ref().unwrap().lines()[1].ends_with("kick (copy 1)"));
        drive.numbered_copy_confirm(source);
        assert_eq!(pad_name(&drive.app.pads, 18), "kick (copy 1)");
        assert_eq!(drive.app.history.len(), 1);
    }
}

/// 87a Scenario "Cancelled and refused copies do not consume numbers".
#[test]
fn cancelled_and_refused_copies_do_not_consume_numbers() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = numbered_copy_drive();
        drive.numbered_copy_start(source, 0);
        drive.app.pads.loading[1] = Some(0);
        drive.numbered_copy_target(source, 1);
        assert!(drive.app.copy_in_hand.is_some());
        assert!(drive.app.pads.sound[1].is_none());
        assert!(drive.app.history.is_empty());
        drive.copy_command(source, KeyCode::Esc, M::NONE, 26);
        assert!(drive.app.copy_in_hand.is_none());
        drive.app.pads.loading[1] = None;
        drive.load_pad(1, drive.app.pads.sound[0].clone().unwrap(), "old target");
        let before = drive.app.pads.snapshot(1);
        drive.numbered_copy_start(source, 0);
        drive.numbered_copy_target(source, 1);
        drive.copy_command(source, KeyCode::Esc, M::NONE, 26);
        assert_eq!(drive.app.pads.snapshot(1), before);
        assert!(drive.app.copy_in_hand.is_some());
        drive.numbered_copy_target(source, 2);
        assert_eq!(pad_name(&drive.app.pads, 2), "kick (copy 1)");
        drive.copy_command(source, KeyCode::Char('z'), M::CONTROL, 87);
        assert!(drive.app.pads.sound[2].is_none());
        assert_eq!(drive.app.pads.snapshot(1), before);
        drive.numbered_copy_start(source, 0);
        drive.numbered_copy_target(source, 2);
        assert_eq!(pad_name(&drive.app.pads, 2), "kick (copy 1)");
    }
}

/// 87a Scenario "A canonical manual source name stays unchanged".
#[test]
fn a_canonical_manual_source_name_stays_unchanged() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = numbered_copy_drive();
        drive.app.pads.custom_name[0] = Some("kick (copy 7)".into());
        let before = drive.app.pads.snapshot(0);
        drive.numbered_copy_start(source, 0);
        drive.numbered_copy_target(source, 1);
        assert_eq!(pad_name(&drive.app.pads, 1), "kick (copy 1)");
        assert_eq!(drive.app.pads.snapshot(0), before);
    }
}

/// 87a Scenarios "Old numeric suffixes stay literal" and
/// "Restored copy names determine the next available number".
#[test]
fn restored_copy_names_determine_the_next_available_number() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for project in [false, true] {
            for base in ["kick", "rec 2 2 2"] {
                let files = TransitionFiles::new();
                let mut drive = files.drive(project);
                drive.bind_action(86, "session.save");
                drive.app.pads.custom_name[0] = Some(base.into());
                drive.numbered_copy_start(source, 0);
                drive.numbered_copy_target(source, 1);
                let sample = files.0.join("tone.wav");
                let sample_bytes = std::fs::read(&sample).unwrap();
                drive.copy_command(source, KeyCode::Char('s'), M::CONTROL, 86);
                let saved = if project {
                    files.0.join("A/project.toml")
                } else {
                    drive.app.session_path.clone()
                };
                let stored = session::load(&saved).unwrap();
                assert_eq!(stored.version, 5);
                assert_eq!(stored.pads["0"].name.as_deref(), Some(base));
                assert_eq!(stored.pads["1"].name, Some(format!("{base} (copy 1)")));
                let mut again = numbered_copy_drive();
                again.app.session_path = drive.app.session_path.clone();
                again.app.session_read_error = None;
                again.app.user_path = Some(files.0.join("reopened-config.toml"));
                if project {
                    assert!(again.app.open_project(files.0.join("A")));
                } else {
                    again.app.apply_session(stored);
                }
                while again.app.pads.loading.iter().any(Option::is_some) {
                    let reply = again
                        .app
                        .loader
                        .as_ref()
                        .unwrap()
                        .1
                        .recv_timeout(Duration::from_secs(5))
                        .unwrap();
                    again.app.loader_reply(reply);
                }
                again.frame();
                assert_eq!(pad_name(&again.app.pads, 0), base);
                assert_eq!(pad_name(&again.app.pads, 1), format!("{base} (copy 1)"));
                let restored = again.app.pads.snapshot(1);
                again.numbered_copy_start(source, 1);
                again.numbered_copy_target(source, 18);
                let mut expected = restored;
                expected.name = Some(format!("{base} (copy 2)"));
                assert_eq!(again.app.pads.snapshot(18), expected);
                assert_eq!(pad_name(&again.app.pads, 0), base);
                again.copy_command(source, KeyCode::Char('s'), M::CONTROL, 86);
                assert_eq!(
                    session::load(&saved).unwrap().pads["18"].name,
                    Some(format!("{base} (copy 2)"))
                );
                assert_eq!(std::fs::read(&sample).unwrap(), sample_bytes);
            }
        }
    }
}

/// Scenario "Copy parity from raw MIDI" and 83a's controller target contract.
#[test]
fn copy_parity_from_raw_midi() {
    let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
    let mut expected = None;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.load_pad(0, sound.clone(), "source");
        drive.tap(KeyCode::Char('c'), M::CONTROL);
        drive.tap(KeyCode::PageDown, M::NONE);
        let hit = drive.pad_press(source, 2);
        assert_silent_target(&hit);
        assert_eq!(drive.app.pads.selected_abs(), 18, "{source:?}");
        assert_eq!(
            drive.app.pads.custom_name[18].as_deref(),
            Some("source (copy 1)")
        );
        assert_eq!(drive.app.pads.custom_name[0].as_deref(), Some("source"));
        assert!(drive.app.copy_in_hand.is_none());
        assert_eq!(drive.app.history.len(), 1, "the target has one undo step");
        let seen = drive.seen();
        if let Some(expected) = &expected {
            assert_eq!(&seen, expected, "{source:?}");
        } else {
            expected = Some(seen);
        }
        drive.pad_release(source, 2);
        drive.tap(KeyCode::Char('z'), M::CONTROL);
        assert!(drive.app.pads.sound[18].is_none());
        assert!(drive.app.pads.sound[0].is_some());
    }
}

/// Scenario "Copy replacement parity from raw MIDI".
#[test]
fn copy_replacement_parity_from_raw_midi() {
    let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
    let mut expected = None;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.load_pad(0, sound.clone(), "source");
        drive.load_pad(18, sound.clone(), "old target");
        let old = drive.app.pads.snapshot(18);
        drive.tap(KeyCode::Char('c'), M::CONTROL);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_silent_target(&drive.pad_press(source, 2));
        assert_eq!(drive.app.pads.selected_abs(), 18, "{source:?}");
        assert_eq!(drive.app.pads.snapshot(18), old);
        assert!(
            drive
                .app
                .ask
                .as_ref()
                .unwrap()
                .lines()
                .iter()
                .any(|line| line.contains("old target"))
        );
        let question = drive.seen();
        drive.pad_release(source, 2);
        assert_silent_target(&drive.pad_press(source, 3));
        assert_eq!(
            drive.seen(),
            question,
            "a further hit must not answer or change the target"
        );
        drive.pad_release(source, 3);
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.ask.is_none());
        assert!(drive.app.copy_in_hand.is_some());
        assert_silent_target(&drive.pad_press(source, 2));
        drive.pad_release(source, 2);
        drive.tap(KeyCode::Enter, M::NONE);
        assert_eq!(
            drive.app.pads.custom_name[18].as_deref(),
            Some("source (copy 1)")
        );
        assert!(drive.app.copy_in_hand.is_none());
        assert!(drive.app.ask.is_none());
        assert_eq!(drive.app.history.len(), 1);
        let seen = drive.seen();
        if let Some(expected) = &expected {
            assert_eq!(&seen, expected, "{source:?}");
        } else {
            expected = Some(seen);
        }
    }
}

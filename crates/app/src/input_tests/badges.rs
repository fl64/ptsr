//! Recorded mode badges, pad marks, and playback indicators.

use super::*;

/// Scenario "New pad": keyboard and raw MIDI keep an empty cell quiet.
#[test]
fn new_pad_stays_quiet_with_keyboard_and_raw_midi() {
    let mut states = Vec::new();
    for source in [Source::Keyboard, Source::Notes] {
        let mut drive = Drive::new();
        for frame in [drive.pad_press(source, 0), drive.pad_release(source, 0)] {
            assert_eq!(frame.commands().len(), 1);
            let view = pads_view(&mut drive.app.pads, None, None, Vec::new(), None, 48_000);
            assert!(view.pads[0].empty);
            assert_eq!(view.pads[0].name, "—");
            assert!(view.pads[0].wave.is_empty());
            assert_eq!(view.pads[0].detail.as_deref(), Some("empty · test tone A2"));
        }
        states.push(drive.seen());
    }
    assert_eq!(states[0], states[1]);
}

/// Scenario "Scope in the help line": keyboard and raw MIDI keep the same edit target.
#[test]
fn scope_in_the_help_line_follows_marks_from_keyboard_and_midi() {
    for source in [Source::Keyboard, Source::Notes] {
        let mut drive = Drive::new();
        drive.app.goto(Screen::Pads);
        drive.bind_action(102, "pad.mark_toggle");
        drive.bind_action(103, "pad.mark_bank");
        let shown = |drive: &Drive| {
            let groups = crate::help_groups(&drive.keymap, Screen::Pads, &drive.app.help_state());
            groups.iter().find(|g| g.name == "CHANGE").unwrap().items[0]
                .1
                .clone()
        };
        assert!(shown(&drive).contains("edit: this pad"));
        match source {
            Source::Keyboard => {
                drive.tap(KeyCode::Char('m'), M::NONE);
            }
            Source::Notes => {
                drive.bytes(PAD_PORT, &[0xb0, 102, 127]);
                drive.bytes(PAD_PORT, &[0xb0, 102, 0]);
                drive.frame();
            }
            Source::Button => unreachable!(),
        }
        for pad in 0..3 {
            drive.pad_press(source, pad);
            drive.pad_release(source, pad);
        }
        assert!(
            shown(&drive).contains("edit: marked (3)"),
            "{source:?}: {}",
            shown(&drive)
        );
        match source {
            Source::Keyboard => {
                drive.tap(KeyCode::Char('a'), M::CONTROL);
            }
            Source::Notes => {
                drive.bytes(PAD_PORT, &[0xb0, 103, 127]);
                drive.bytes(PAD_PORT, &[0xb0, 103, 0]);
                drive.frame();
            }
            Source::Button => unreachable!(),
        }
        assert!(
            shown(&drive).contains("edit: bank"),
            "{source:?}: {}",
            shown(&drive)
        );
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(shown(&drive).contains("edit: this pad"));
    }
}

fn badge_labels(drive: &Drive) -> Vec<String> {
    let before = drive.seen();
    let badges = drive.app.title_badges();
    assert_eq!(
        drive.seen(),
        before,
        "preparing badges cannot change application state"
    );
    assert!(
        badges
            .first()
            .is_some_and(|badge| badge.normal.ends_with("■ 120.0"))
    );
    badges
        .into_iter()
        .filter_map(|badge| {
            if badge.normal == "■ 120.0" {
                None
            } else {
                Some(
                    badge
                        .normal
                        .strip_suffix(" · ■ 120.0")
                        .unwrap_or(&badge.normal)
                        .to_string(),
                )
            }
        })
        .collect()
}

fn pad_feedback(drive: &Drive) -> tui::PadFeedback {
    let before = (
        drive.seen(),
        drive.app.session_dirty,
        drive.app.pending_audio.len(),
        drive.app.sent.len(),
    );
    let feedback = drive.app.pad_feedback();
    assert_eq!(
        (
            drive.seen(),
            drive.app.session_dirty,
            drive.app.pending_audio.len(),
            drive.app.sent.len()
        ),
        before,
        "drawing pad feedback cannot change state or send audio commands"
    );
    feedback
}

/// Scenario "Placement marks distinguish confirmation from swap".
#[test]
fn placement_marks_distinguish_confirmation_from_swap() {
    use tui::PadCorner::{Available, Replace};
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for operation in ["file", "copy", "swap"] {
            for target in [4, 5] {
                let mut drive = Drive::new();
                let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
                drive.load_pad(0, sound.clone(), "source");
                drive.load_pad(4, sound, "target");
                drive.bind_action(102, "pad.copy");
                drive.bind_action(103, "pad.take");
                drive.bind_action(104, "screen.pads");
                let controls = if matches!(source, Source::Keyboard) {
                    source
                } else {
                    Source::Button
                };
                match operation {
                    "file" => {
                        drive.ordered_file();
                        drive.registry_command(controls, KeyCode::Enter, M::NONE, 82);
                        drive.registry_command(controls, KeyCode::F(2), M::NONE, 104);
                    }
                    "copy" => {
                        drive.registry_command(controls, KeyCode::Char('c'), M::CONTROL, 102);
                    }
                    _ => {
                        drive.registry_command(controls, KeyCode::Char('x'), M::CONTROL, 103);
                    }
                }
                let feedback = pad_feedback(&drive);
                assert!(feedback.outline);
                let asks = operation != "swap" && target == 4;
                assert_eq!(
                    feedback.corners[target],
                    Some(if asks { Replace } else { Available })
                );
                let before = drive.seen().pads;
                assert_silent_target(&drive.pad_press(source, target));
                if asks {
                    assert_eq!(drive.seen().pads, before);
                    assert_eq!(
                        matches!(drive.app.ask, Some(Ask::PutFile { .. })),
                        operation == "file"
                    );
                    assert_eq!(
                        matches!(drive.app.ask, Some(Ask::CopyPad { .. })),
                        operation == "copy"
                    );
                } else {
                    assert!(
                        drive.app.ask.is_none(),
                        "an available target adds no question"
                    );
                    if operation == "file" {
                        let deadline = Instant::now() + Duration::from_secs(5);
                        while drive.app.pads.loading[target].is_some() {
                            let reply = drive
                                .app
                                .loader
                                .as_ref()
                                .unwrap()
                                .1
                                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                                .unwrap();
                            drive.app.loader_reply(reply);
                        }
                        assert!(drive.app.pads.sound[target].is_some());
                        assert!(drive.app.browser.in_hand.is_none());
                    }
                    assert!(
                        drive.seen().pads != before,
                        "{source:?}, {operation}, target {target}"
                    );
                }
                drive.pad_release(source, target);
            }
        }
    }
}

#[test]
fn pad_corner_marks_respect_loading_cancellation_and_whole_file_identity() {
    use tui::PadCorner::{Available, Replace};
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
        drive.load_pad(0, sound.clone(), "source");
        drive.load_pad(4, sound, "target");
        drive.bind_action(102, "pad.copy");
        drive.bind_action(103, "pad.take");
        drive.bind_action(104, "ui.back");
        let controls = if matches!(source, Source::Keyboard) {
            source
        } else {
            Source::Button
        };
        drive.registry_command(controls, KeyCode::Char('c'), M::CONTROL, 102);
        assert_eq!(
            pad_feedback(&drive).corners[0],
            None,
            "the source cancels copying"
        );
        drive.app.pads.loading[4] = Some(33);
        assert_eq!(pad_feedback(&drive).corners[4], None);
        let before = drive.seen().pads;
        assert_silent_target(&drive.pad_press(source, 4));
        drive.pad_release(source, 4);
        assert!(drive.app.copy_in_hand.is_some());
        assert!(drive.app.ask.is_none());
        assert_eq!(drive.seen().pads, before);
        drive.app.pads.loading[4] = None;
        assert_eq!(pad_feedback(&drive).corners[4], Some(Replace));
        drive.pad_press(source, 0);
        drive.pad_release(source, 0);
        assert!(drive.app.copy_in_hand.is_none());
        assert_eq!(pad_feedback(&drive), tui::PadFeedback::default());

        drive.app.pads.selected = 0;
        drive.registry_command(controls, KeyCode::Char('x'), M::CONTROL, 103);
        assert_eq!(pad_feedback(&drive).corners[4], Some(Available));
        drive.app.pads.loading[0] = Some(1);
        assert!(pad_feedback(&drive).corners.iter().all(Option::is_none));
        drive.app.pads.loading[0] = None;
        drive.app.pads.loading[4] = Some(2);
        assert_eq!(pad_feedback(&drive).corners[4], None);
        assert_silent_target(&drive.pad_press(source, 4));
        drive.pad_release(source, 4);
        assert!(
            drive.app.pad_in_hand.is_none(),
            "the existing refused swap drops its hand"
        );
        drive.app.pads.loading[4] = None;
        drive.registry_command(controls, KeyCode::Esc, M::NONE, 104);

        // Use the real browser hold route, with the same lexical file identity as pad 1.
        drive.tap(KeyCode::F(6), M::NONE);
        drive.app.browser.dir = PathBuf::new();
        drive.app.browser.entries = vec![browser::Entry {
            name: "source.wav".into(),
            is_dir: false,
            favorite: None,
        }];
        drive.app.browser.selected = 0;
        drive.bind_action(82, "browser.hold");
        drive.registry_command(controls, KeyCode::Enter, M::NONE, 82);
        assert_eq!(pad_feedback(&drive).corners[0], None);
        drive.app.pads.region[0] = Region {
            start: 1,
            end: 1024,
        };
        assert_eq!(
            pad_feedback(&drive).corners[0],
            Some(Replace),
            "a slice is not the whole file"
        );
        drive.app.pads.loading[4] = Some(50);
        assert_eq!(
            pad_feedback(&drive).corners[4],
            Some(Replace),
            "file replacement still asks during loading"
        );
    }
}

/// Scenario "Marking changes appearance without silencing pads".
#[test]
fn marking_changes_appearance_without_silencing_pads() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.load_pad(0, Arc::new(Sample::mono(vec![0.25; 1024])), "source");
        drive.bind_action(102, "pad.mark_toggle");
        drive.bind_action(103, "ui.back");
        let controls = if matches!(source, Source::Keyboard) {
            source
        } else {
            Source::Button
        };
        drive.registry_command(controls, KeyCode::Char('m'), M::NONE, 102);
        assert!(pad_feedback(&drive).outline);
        assert_full_hit(&drive.pad_press(source, 0), 0);
        assert!(drive.app.pads.marked.contains(&0));
        assert_only_release(&drive.pad_release(source, 0), 0);
        drive.registry_command(controls, KeyCode::Char('m'), M::NONE, 102);
        assert!(
            !pad_feedback(&drive).outline,
            "retained marks alone do not outline the grid"
        );
        assert_eq!(badge_labels(&drive), ["1 MARKED · Esc drops"]);
        drive.registry_command(controls, KeyCode::Esc, M::NONE, 103);
        assert!(drive.app.pads.marked.is_empty());
        assert_eq!(pad_feedback(&drive), tui::PadFeedback::default());
    }
}

/// Scenario "Assignment numbers appear on target pads".
#[test]
fn assignment_numbers_appear_on_target_pads() {
    use tui::PadCorner::Slice;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::registry_screen_checks();
        drive.bind_action(102, "chop.assign");
        drive.bind_action(103, "chop.assign_order");
        drive.bind_action(104, "bank.next");
        drive.bind_action(105, "ui.back");
        let controls = if matches!(source, Source::Keyboard) {
            source
        } else {
            Source::Button
        };
        let before = drive.seen().pads;
        let chop = drive.app.chop.as_ref().unwrap();
        let sound = chop.sound.clone();
        let slices = chop.slices();
        drive.registry_command(controls, KeyCode::Char('p'), M::NONE, 102);
        assert!(pad_feedback(&drive).outline);
        assert_slice_preview(&drive.pad_press(source, 5), &sound, slices[0]);
        drive.pad_release(source, 5);
        assert_eq!(pad_feedback(&drive).corners[5], Some(Slice(1)));
        assert_slice_preview(&drive.pad_press(source, 6), &sound, slices[1]);
        drive.pad_release(source, 6);
        assert_eq!(pad_feedback(&drive).corners[6], Some(Slice(2)));
        assert_eq!(drive.seen().pads, before, "a plan has not applied anything");
        drive.registry_command(controls, KeyCode::PageDown, M::NONE, 104);
        assert!(pad_feedback(&drive).corners.iter().all(Option::is_none));
        drive.registry_command(controls, KeyCode::Tab, M::NONE, 103);
        assert_slice_preview(&drive.pad_press(source, 14), &sound, slices[0]);
        drive.pad_release(source, 14);
        assert_eq!(
            pad_feedback(&drive).corners[14..],
            [Some(Slice(1)), Some(Slice(2))]
        );
        drive.registry_command(controls, KeyCode::PageDown, M::NONE, 104);
        assert_eq!(
            pad_feedback(&drive).corners[..2],
            [Some(Slice(3)), Some(Slice(4))]
        );
        drive.registry_command(controls, KeyCode::Esc, M::NONE, 105);
        assert_eq!(pad_feedback(&drive), tui::PadFeedback::default());
        assert_eq!(drive.seen().pads, before);
    }
}

/// Scenarios "Learning and placement keep existing sound rules" and "Hidden grids retain visible mode identity".
#[test]
fn learning_and_hidden_grids_keep_existing_sound_rules() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = Drive::registry_screen_checks();
        drive.bind_action(102, "ui.pads_toggle");
        drive.bind_action(103, "chop.assign");
        drive.bind_action(104, "ui.back");
        drive.bind_action(105, "pad.copy");
        drive.registry_command(source, KeyCode::Char('o'), M::CONTROL, 102);
        assert!(drive.app.ui.hide_pads);
        drive.registry_command(source, KeyCode::Char('p'), M::NONE, 103);
        let view = drive.app.mark_chop(
            drive.app.chop_display(drive.app.chop.as_ref().unwrap()),
            None,
        );
        assert!(!view.pads.hidden && view.pads.feedback.outline);
        drive.registry_command(source, KeyCode::Esc, M::NONE, 104);
        let view = drive.app.mark_chop(
            drive.app.chop_display(drive.app.chop.as_ref().unwrap()),
            None,
        );
        assert!(view.pads.hidden && !view.pads.feedback.outline);
        for (screen, key, cc) in [
            (Screen::Pads, KeyCode::F(2), 90),
            (Screen::Files, KeyCode::F(6), 92),
            (Screen::Chop, KeyCode::F(3), 93),
        ] {
            drive.registry_command(source, KeyCode::F(2), M::NONE, 90);
            if screen == Screen::Chop {
                // F3 on Pads opens the selected sample; on Files it opens a browser entry.
                drive.pad_press(source, 0);
                drive.pad_release(source, 0);
            }
            drive.registry_command(source, key, M::NONE, cc);
            assert_eq!(
                drive.app.screen,
                screen,
                "{source:?}: selected pad {}, notice {:?}",
                drive.app.pads.selected_abs(),
                drive.app.notice()
            );
            drive.registry_command(source, KeyCode::F(10), M::NONE, 96);
            assert!(pad_feedback(&drive).outline);
            assert!(pad_feedback(&drive).corners.iter().all(Option::is_none));
            assert_silent_target(&drive.pad_press(source, 4));
            drive.pad_release(source, 4);
            drive.tap(KeyCode::F(10), M::NONE);
            assert_eq!(pad_feedback(&drive), tui::PadFeedback::default());
        }
        drive.registry_command(source, KeyCode::F(2), M::NONE, 90);
        drive.registry_command(source, KeyCode::Char('c'), M::CONTROL, 105);
        let view = pads_view(&mut drive.app.pads, None, None, Vec::new(), None, 48_000);
        let view = drive.app.mark_pads(view);
        assert!(view.hidden && view.feedback.outline);
        assert!(badge_labels(&drive)[0].contains("COPY"));
    }
}

/// Scenario "Mode badges follow the active screen state".
#[test]
fn mode_badges_follow_keyboard_and_raw_midi() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = Drive::registry_screen_checks();
        drive.bind_action(102, "pad.copy");
        drive.bind_action(103, "ui.back");
        drive.app.browser.dir = "target/87-badge-files".into();
        for (screen, key, number) in REGISTRY_SCREEN_COMMANDS {
            drive.registry_command(source, KeyCode::F(2), M::NONE, 90);
            assert!(badge_labels(&drive).is_empty());
            drive.registry_command(source, KeyCode::Char('c'), M::CONTROL, 102);
            drive.registry_command(source, key, M::NONE, number);
            assert_eq!(drive.app.screen, screen);
            assert_eq!(
                badge_labels(&drive),
                ["COPY · hit a pad"],
                "{source:?}, {screen:?}"
            );
            drive.registry_command(source, KeyCode::Esc, M::NONE, 103);
            assert!(badge_labels(&drive).is_empty());
            if screen != Screen::Settings {
                drive.registry_command(source, KeyCode::F(10), M::NONE, 96);
                assert_eq!(badge_labels(&drive), ["LEARN"]);
                // MIDI becomes a learning candidate. F10 keeps its existing keyboard exit.
                drive.tap(KeyCode::F(10), M::NONE);
                assert!(badge_labels(&drive).is_empty());
            }
        }
    }
}

/// Scenario "Every existing mode has its badge".
#[test]
fn every_existing_mode_has_its_badge() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = Drive::new();
        drive.load_pad(0, Arc::new(Sample::mono(vec![0.25; 1024])), "source");
        for (cc, action) in [
            (102, "pad.mark_toggle"),
            (103, "pad.edit"),
            (104, "ui.back"),
            (105, "pad.take"),
            (106, "pad.copy"),
            (107, "edit.page_next"),
        ] {
            drive.bind_action(cc, action);
        }
        assert!(badge_labels(&drive).is_empty());
        drive.registry_command(source, KeyCode::Char('m'), M::NONE, 102);
        assert_eq!(badge_labels(&drive), ["MARK · hit pads · 0 marked"]);
        drive.pad_press(source, 0);
        drive.pad_release(source, 0);
        assert_eq!(badge_labels(&drive), ["MARK · hit pads · 1 marked"]);
        drive.registry_command(source, KeyCode::Char('m'), M::NONE, 102);
        assert_eq!(badge_labels(&drive), ["1 MARKED · Esc drops"]);
        drive.registry_command(source, KeyCode::Enter, M::NONE, 103);
        assert_eq!(
            badge_labels(&drive),
            ["1 MARKED · Esc drops", "EDIT · TONE"]
        );
        drive.registry_command(source, KeyCode::Tab, M::NONE, 107);
        assert!(badge_labels(&drive).contains(&"EDIT · ENV".into()));
        drive.registry_command(source, KeyCode::Esc, M::NONE, 104);
        assert_eq!(badge_labels(&drive), ["1 MARKED · Esc drops"]);
        drive.registry_command(source, KeyCode::Esc, M::NONE, 104);
        assert!(badge_labels(&drive).is_empty());
        drive.registry_command(source, KeyCode::Char('x'), M::CONTROL, 105);
        assert_eq!(badge_labels(&drive), ["SWAP · hit a pad"]);
        drive.registry_command(source, KeyCode::Esc, M::NONE, 104);
        drive.registry_command(source, KeyCode::Char('c'), M::CONTROL, 106);
        assert_eq!(badge_labels(&drive), ["COPY · hit a pad"]);
        drive.pad_press(source, 0);
        drive.pad_release(source, 0);
        drive.registry_command(source, KeyCode::Esc, M::NONE, 104);

        drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 48_000])), 4);
        for (cc, action) in [
            (108, "chop.assign"),
            (109, "chop.edit"),
            (110, "chop.loop_toggle"),
            (111, "chop.as_pad_toggle"),
        ] {
            drive.bind_action(cc, action);
        }
        drive.registry_command(source, KeyCode::Enter, M::NONE, 109);
        assert_eq!(badge_labels(&drive), ["EDIT · BOUNDS"]);
        drive.registry_command(source, KeyCode::Esc, M::NONE, 104);
        drive.registry_command(source, KeyCode::Char('o'), M::ALT, 110);
        drive.registry_command(source, KeyCode::Char('p'), M::ALT, 111);
        assert_eq!(badge_labels(&drive), ["LOOP", "AS PAD"]);
        drive.registry_command(source, KeyCode::Char('p'), M::NONE, 108);
        assert_eq!(badge_labels(&drive)[0], "ASSIGN · 0/4 placed");
        drive.pad_press(source, 4);
        drive.pad_release(source, 4);
        assert_eq!(badge_labels(&drive)[0], "ASSIGN · 1/4 placed");
        drive.registry_command(source, KeyCode::Esc, M::NONE, 104);
        assert_eq!(badge_labels(&drive), ["LOOP", "AS PAD"]);
        drive.registry_command(source, KeyCode::Char('o'), M::ALT, 110);
        drive.registry_command(source, KeyCode::Char('p'), M::ALT, 111);
        assert!(badge_labels(&drive).is_empty());
    }
}

/// Scenario "Overlapping placements remain visible".
#[test]
fn overlapping_placements_remain_visible_without_changing_priority() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for mask in 1..8 {
            let mut drive = Drive::new();
            let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
            drive.load_pad(0, sound.clone(), "source");
            drive.load_pad(4, sound, "target");
            for (cc, action) in [
                (102, "pad.copy"),
                (103, "pad.take"),
                (104, "screen.files"),
                (105, "browser.hold"),
                (106, "screen.pads"),
            ] {
                drive.bind_action(cc, action);
            }
            let controls = if matches!(source, Source::Keyboard) {
                Source::Keyboard
            } else {
                Source::Button
            };
            if mask & 2 != 0 {
                drive.registry_command(controls, KeyCode::Char('c'), M::CONTROL, 102);
            }
            if mask & 4 != 0 {
                drive.registry_command(controls, KeyCode::Char('x'), M::CONTROL, 103);
            }
            if mask & 1 != 0 {
                drive.app.browser.dir = "target/87-badge-files".into();
                drive.registry_command(controls, KeyCode::F(6), M::NONE, 104);
                drive.app.browser.entries = vec![browser::Entry {
                    name: "new.wav".into(),
                    is_dir: false,
                    favorite: None,
                }];
                drive.app.browser.selected = 0;
                drive.registry_command(controls, KeyCode::Enter, M::NONE, 105);
                drive.registry_command(controls, KeyCode::F(2), M::NONE, 106);
            }
            let labels = badge_labels(&drive);
            assert_eq!(labels.len(), 1, "one compound badge");
            for (bit, name) in [(1, "IN HAND"), (2, "COPY"), (4, "SWAP")] {
                assert_eq!(
                    labels[0].contains(name),
                    mask & bit != 0,
                    "{mask}: {labels:?}"
                );
            }
            assert_eq!(
                pad_feedback(&drive).corners[4],
                Some(if mask & 3 != 0 {
                    tui::PadCorner::Replace
                } else {
                    tui::PadCorner::Available
                }),
                "all seven overlapping operations retain their target marks"
            );
            let before = drive.seen().pads;
            assert_silent_target(&drive.pad_press(source, 4));
            drive.pad_release(source, 4);
            let swap_wins = mask & 4 != 0
                && mask & 2 == 0
                && (matches!(source, Source::Keyboard) || mask & 1 == 0);
            if swap_wins {
                assert!(drive.app.ask.is_none(), "an occupied swap remains direct");
                assert_ne!(drive.seen().pads, before);
                assert_eq!(
                    badge_labels(&drive),
                    if mask & 1 != 0 {
                        vec!["IN HAND · hit a pad".to_string()]
                    } else {
                        Vec::new()
                    }
                );
            } else {
                assert_eq!(drive.seen().pads, before);
                let file_wins =
                    mask & 1 != 0 && (!matches!(source, Source::Keyboard) || mask & 6 == 0);
                assert_eq!(
                    matches!(drive.app.ask, Some(Ask::PutFile { .. })),
                    file_wins
                );
                assert_eq!(
                    matches!(drive.app.ask, Some(Ask::CopyPad { .. })),
                    !file_wins && mask & 2 != 0
                );
                assert_eq!(badge_labels(&drive)[1], "ASK");
            }
        }
    }
}

#[test]
fn midi_list_learning_has_a_badge_without_muting_pads() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = Drive::registry_screen_checks();
        drive.bind_action(108, "settings.learn");
        drive.registry_command(source, KeyCode::F(12), M::NONE, 91);
        drive.registry_command(source, KeyCode::Tab, M::NONE, 99);
        drive.registry_command(source, KeyCode::Tab, M::NONE, 99);
        drive.app.midi_open = vec![input::Group::Pads];
        drive.app.midi_row = drive
            .app
            .midi_list()
            .iter()
            .position(|row| matches!(row, MidiListRow::Action { info } if info.name == "pad.copy"))
            .unwrap();
        drive.registry_command(source, KeyCode::Enter, M::NONE, 108);
        assert_eq!(badge_labels(&drive), ["LEARN"]);
        assert_full_hit(&drive.pad_press(source, 0), 0);
        assert_eq!(badge_labels(&drive), ["LEARN"]);
        assert_only_release(&drive.pad_release(source, 0), 0);
        // F12 keeps its existing keyboard cancellation of captured MIDI candidates.
        drive.tap(KeyCode::Esc, M::NONE);
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(badge_labels(&drive).is_empty());
    }
}

#[test]
fn recording_badges_follow_real_input_and_controlled_elapsed_time() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = Drive::offline();
        drive.app.routed.input = Some("offline microphone".into());
        drive.app.paths.samples = format!(
            "target/87-silent-recording-{}-{source:?}",
            std::process::id()
        )
        .into();
        drive.bind_action(109, "record.mic");
        if matches!(source, Source::Keyboard) {
            drive.key(KeyCode::Char('r'), M::ALT, KeyEventKind::Press);
        } else {
            drive.bytes(PAD_PORT, &[0xb0, 109, 127]);
        }
        drive.frame();
        let (_, start) = drive
            .app
            .recording
            .expect("the production route started recording");
        let mut text = view::FrameText::default();
        for seconds in [0, 1, 12, 123] {
            let badges = drive
                .app
                .title_badges_at(start + Duration::from_secs(seconds));
            assert_eq!(badges[0].normal, "■ 120.0");
            assert_eq!(badges[1].kind, tui::BadgeKind::Attention);
            assert_eq!(badges[1].normal, format!("● REC {seconds}s"));
            assert_eq!(badges[1].compact, "REC");
            let now = start + Duration::from_secs(seconds);
            assert_eq!(text.badges_at(&drive.app, now), badges);
            let builds = text.builds().0;
            assert_eq!(
                text.badges_at(&drive.app, now + Duration::from_nanos(1)),
                badges
            );
            assert_eq!(text.builds().0, builds);
            assert_eq!(
                text.feedback_at(&drive.app, now).recording_seconds,
                Some(seconds)
            );
            assert_eq!(
                drive.app.status_feedback_at(now).recording_seconds,
                Some(seconds)
            );
            for (width, height) in [(80, 26), (120, 40)] {
                let row = &shared_feedback_rows(&mut drive, width, height, now)[0];
                assert!(row.contains(&format!("rec {seconds}s")), "{row}");
            }
        }
        if matches!(source, Source::Keyboard) {
            drive.key(KeyCode::Char('r'), M::ALT, KeyEventKind::Release);
        } else {
            drive.bytes(PAD_PORT, &[0xb0, 109, 0]);
            drive.frame();
            assert!(
                drive.app.recording.is_some(),
                "the existing MIDI button ignores its release"
            );
            drive.bytes(PAD_PORT, &[0xb0, 109, 127]);
        }
        drive.frame();
        assert!(drive.app.recording.is_none());
        assert!(badge_labels(&drive).is_empty());
        assert!(
            !drive.app.paths.samples.exists(),
            "a recording with no frames writes no files"
        );
    }
}

fn pads_play_badge(drive: &mut Drive) -> bool {
    use ratatui::{Terminal, backend::TestBackend};
    let app = &mut drive.app;
    let view = pads_view(&mut app.pads, None, None, Vec::new(), None, app.rate);
    let screen = tui::Screen::Pads(app.mark_pads(view));
    let status = tui::Status {
        badges: app.title_badges(),
        save: app.save_status(),
        backend: None,
        recording: false,
        buffer_size: 128,
        sample_rate: 48_000,
        load_peak: 0.0,
        xruns: 0,
        master_db: 0,
        meters: ([-60.0; 2], [-60.0; 2], None),
        notice: None,
        no_releases: false,
        project: None,
    };
    let help = tui::Help {
        groups: &[],
        hint: None,
        full: None,
        question: None,
        learn: None,
    };
    let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
    term.draw(|frame| tui::draw(frame, &status, &screen, &help))
        .unwrap();
    let header = term
        .backend()
        .buffer()
        .content()
        .chunks(100)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .find(|row| row.contains("F2 PADS"))
        .unwrap();
    let slot = header.chars().skip(10).take(10).collect::<String>();
    assert!(
        matches!(slot.as_str(), " PLAY ▶   " | "          "),
        "{header}"
    );
    slot.contains("PLAY ▶")
}

/// Scenario "Pads PLAY badge follows keyboard and MIDI playback".
#[test]
fn pads_play_badge_follows_keyboard_and_midi_playback() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for release in [false, true] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            drive.app.pads.gate[18] = release;
            let snap = drive.app.pads.snapshot(18);
            for cmd in drive.app.pads.put_back(18, &snap) {
                drive.app.send(cmd);
            }
            drive.tap(KeyCode::F(2), M::NONE);
            drive.render_blocks(4);
            assert!(!pads_play_badge(&mut drive));
            drive.velocity_hit(source, 127);
            assert!(!pads_play_badge(&mut drive), "a queued hit is not playback");
            let hit = drive.render_blocks(1);
            assert!(pads_play_badge(&mut drive), "{source:?}, release {release}");
            assert!(engine::checks::peak(&hit.0) > 0.01);
            if release {
                drive.pad_release(source, 2);
                drive.render_blocks(1);
                assert!(pads_play_badge(&mut drive), "release fade still plays");
            }
            let end = drive.render_blocks(400);
            assert!(!pads_play_badge(&mut drive));
            if !release {
                assert!(drive.app.pads.held[18], "held does not mean sounding");
            }
            for channel in [&end.0, &end.1] {
                assert_eq!(engine::checks::first_non_finite(channel), None);
                assert!(engine::checks::peak(channel) < 1.0);
                assert_eq!(engine::checks::peak(&channel[49_000..]), 0.0);
            }
        }
    }
}

/// Scenario "Pads PLAY badge follows all sounding pads".
#[test]
fn pads_play_badge_follows_all_sounding_pads() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.app.pads.gate[18] = false;
        drive.load_pad(0, Arc::new(Sample::mono(vec![0.1; 96_000])), "second");
        for pad in [0, 18] {
            let snap = drive.app.pads.snapshot(pad);
            for cmd in drive.app.pads.put_back(pad, &snap) {
                drive.app.send(cmd);
            }
        }
        drive.tap(KeyCode::F(2), M::NONE);
        drive.render_blocks(4);
        drive.velocity_hit(source, 127);
        drive.render_blocks(1);
        drive.pad_release(source, 2);
        drive.tap(KeyCode::Char('1'), M::CONTROL);
        drive.pad_press(source, 0);
        drive.render_blocks(1);
        assert!(pads_play_badge(&mut drive));
        drive.pad_release(source, 0);
        drive.render_blocks(16);
        assert!(
            pads_play_badge(&mut drive),
            "the older pad in bank 2 still plays"
        );
        drive.tap(KeyCode::Char('3'), M::CONTROL);
        assert_eq!(drive.app.pads.selected_abs(), 32);
        assert!(
            pads_play_badge(&mut drive),
            "selected empty pad is not the sounding one"
        );
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.edit);
        assert!(pads_play_badge(&mut drive));
        drive.tap(KeyCode::Char('o'), M::CONTROL);
        assert!(drive.app.ui.hide_pads);
        assert!(pads_play_badge(&mut drive));
        drive.tap(KeyCode::F(4), M::NONE);
        drive.tap(KeyCode::F(2), M::NONE);
        assert!(
            pads_play_badge(&mut drive),
            "returning to F2 sees real pad activity"
        );
        drive.render_blocks(400);
        assert!(!pads_play_badge(&mut drive), "all pad voices ended");
    }
}

/// Scenarios "Pads PLAY badge follows all sounding pads" and "Pads PLAY badge ignores silent requests and old projects".
#[test]
fn pads_play_badge_keeps_voice_identity_beside_pad_feedback() {
    use std::sync::atomic::Ordering::Acquire;
    let tone = Arc::new(Sample::mono(
        (0..8192)
            .map(|frame| (std::f32::consts::TAU * 220.0 * frame as f32 / 48_000.0).sin() * 0.2)
            .collect(),
    ));

    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for mode in ["normal", "mark", "copy", "swap", "file", "learn"] {
            for hidden in [false, true] {
                for switch in [false, true] {
                    let mut drive = Drive::offline();
                    drive.load_pad(18, tone.clone(), "badge");
                    drive.tap(KeyCode::Char('2'), M::CONTROL);
                    drive.app.pads.gate[18] = false;
                    let snap = drive.app.pads.snapshot(18);
                    for command in drive.app.pads.put_back(18, &snap) {
                        drive.app.send(command);
                    }
                    for (cc, action) in [
                        (102, "pad.mark_toggle"),
                        (103, "pad.copy"),
                        (104, "pad.take"),
                        (105, "bank.next"),
                        (106, "ui.pads_toggle"),
                        (107, "learn.toggle"),
                        (108, "pad.edit"),
                        (109, "screen.pads"),
                    ] {
                        drive.bind_action(cc, action);
                    }
                    let controls = if matches!(source, Source::Keyboard) {
                        source
                    } else {
                        Source::Button
                    };
                    drive.registry_command(controls, KeyCode::F(2), M::NONE, 109);
                    drive.render_blocks(4);
                    assert!(!pads_play_badge(&mut drive));
                    // Offline MIDI writes directly to the callback queue. The rendered hit
                    // below proves acceptance without capturing or replaying that route.
                    drive.velocity_hit(source, 127);
                    assert!(!pads_play_badge(&mut drive), "a queued hit is not a voice");
                    let hit = drive.render_blocks(1);
                    assert!(engine::checks::peak(&hit.0) > 0.01);
                    drive.pad_release(source, 2);
                    assert!(pads_play_badge(&mut drive));
                    if hidden {
                        drive.registry_command(controls, KeyCode::Enter, M::NONE, 108);
                        assert!(drive.app.edit);
                    }
                    match mode {
                        "mark" => {
                            drive.registry_command(controls, KeyCode::Char('m'), M::NONE, 102);
                        }
                        "copy" => {
                            drive.registry_command(controls, KeyCode::Char('c'), M::CONTROL, 103);
                        }
                        "swap" => {
                            drive.registry_command(controls, KeyCode::Char('x'), M::CONTROL, 104);
                        }
                        "file" => {
                            drive.ordered_file();
                            drive.registry_command(controls, KeyCode::Enter, M::NONE, 82);
                            drive.registry_command(controls, KeyCode::F(2), M::NONE, 109);
                        }
                        _ => {}
                    }
                    drive.registry_command(controls, KeyCode::PageDown, M::NONE, 105);
                    if hidden {
                        drive.registry_command(controls, KeyCode::Char('o'), M::CONTROL, 106);
                    }
                    if mode == "learn" {
                        drive.registry_command(controls, KeyCode::F(10), M::NONE, 107);
                    }
                    assert_ne!(drive.app.pads.bank, 1, "the voice stays on another bank");
                    assert_eq!(drive.app.ui.hide_pads, hidden);
                    let feedback = pad_feedback(&drive);
                    assert_eq!(feedback.outline, mode != "normal");
                    if matches!(mode, "copy" | "swap" | "file") {
                        assert_eq!(feedback.corners[0], Some(tui::PadCorner::Available));
                    }
                    assert!(
                        pads_play_badge(&mut drive),
                        "{source:?}, {mode}, hidden {hidden}"
                    );
                    if switch {
                        let old_epoch = drive.project.epoch.load(Acquire);
                        drive.app.apply_session(session::Session::default());
                        // The old snapshot still reports a voice before the next callback.
                        assert!(
                            drive
                                .app
                                .audio
                                .as_ref()
                                .unwrap()
                                .stats
                                .pads_playing(old_epoch)
                        );
                        drive.frame();
                        if !pad_feedback(&drive).outline {
                            drive.registry_command(controls, KeyCode::Char('m'), M::NONE, 102);
                        }
                        assert!(pad_feedback(&drive).outline);
                        assert!(
                            !pads_play_badge(&mut drive),
                            "a mode cannot expose an old-project voice"
                        );
                        let new_epoch = drive.project.epoch.load(Acquire);
                        for _ in 0..256 {
                            drive.app.flush_audio();
                            drive.render_blocks(1);
                            assert!(!pads_play_badge(&mut drive));
                            if drive.project.ready.load(Acquire) == new_epoch
                                && drive.app.pending_audio.is_empty()
                            {
                                break;
                            }
                        }
                        assert_eq!(drive.project.ready.load(Acquire), new_epoch);
                        assert!(drive.app.pending_audio.is_empty());
                    } else {
                        let end = drive.render_blocks(80);
                        assert!(
                            !pads_play_badge(&mut drive),
                            "the last voice ended, not the mode"
                        );
                        assert_eq!(pad_feedback(&drive), feedback);
                        for channel in [&end.0, &end.1] {
                            assert_eq!(engine::checks::first_non_finite(channel), None);
                            assert!(engine::checks::peak(channel) < 1.0);
                            assert_eq!(engine::checks::peak(&channel[9_000..]), 0.0);
                        }
                    }
                }
            }
        }
    }
}

/// Scenario "Pads PLAY badge ignores silent requests and old projects".
#[test]
fn pads_play_badge_ignores_silent_requests_and_old_projects() {
    use std::sync::atomic::Ordering::{Acquire, Release};
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for guard in ["question", "learn", "queue", "not-ready", "no-audio"] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            drive.tap(KeyCode::F(2), M::NONE);
            drive.app.send(Command::SetPadSound {
                pad: 17,
                sound: None,
            });
            drive.render_blocks(4);
            match guard {
                "question" => {
                    drive.app.pads.marked.insert(18);
                    drive.app.ask_clear();
                    drive.frame();
                }
                "learn" => {
                    drive.tap(KeyCode::F(10), M::NONE);
                }
                "queue" if matches!(source, Source::Keyboard) => {
                    let audio = drive.app.audio.as_mut().unwrap();
                    while audio.commands.slots() > 0 {
                        audio
                            .commands
                            .push(ProjectCommand {
                                epoch: 0,
                                ready_at_send: true,
                                command: ProjectMessage::Command(Command::SetPadGain {
                                    pad: 0,
                                    gain: 1.0,
                                }),
                            })
                            .unwrap();
                    }
                }
                "queue" => {
                    // Fill the actual MIDI queue with hits of a pad with no sound.
                    for _ in 0..1024 {
                        drive.bytes(PAD_PORT, &[0x90, 37, 127]);
                    }
                    drive.frame();
                }
                "not-ready" => drive.project.ready.store(1, Release),
                "no-audio" => {
                    drive.app.audio.take();
                }
                _ => unreachable!(),
            }
            drive.velocity_hit(source, 127);
            assert!(
                !pads_play_badge(&mut drive),
                "{source:?}, {guard}, before callback"
            );
            if guard != "no-audio" {
                drive.project.ready.store(0, Release);
                let silent = drive.render_blocks(8);
                assert!(!pads_play_badge(&mut drive), "{source:?}, {guard}");
                assert_eq!(engine::checks::peak(&silent.0), 0.0);
            }
        }
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.render_blocks(4);
        drive.velocity_hit(source, 127);
        drive.render_blocks(1);
        assert!(pads_play_badge(&mut drive));
        drive.project.ready.store(1, Release);
        assert!(
            !pads_play_badge(&mut drive),
            "not ready hides an older snapshot"
        );
        drive.project.ready.store(0, Release);
        assert!(pads_play_badge(&mut drive));
        drive.app.apply_session(session::Session::default());
        assert!(
            !pads_play_badge(&mut drive),
            "old project is hidden before its fade ends"
        );
        for _ in 0..256 {
            drive.app.flush_audio();
            drive.render_blocks(1);
            assert!(!pads_play_badge(&mut drive));
            if drive.project.ready.load(Acquire) == 1 && drive.app.pending_audio.is_empty() {
                break;
            }
        }
        assert_eq!(drive.project.ready.load(Acquire), 1);
        // A new keyboard hit needs a release of the original physical press first.
        drive.pad_release(source, 2);
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.render_blocks(8);
        assert!(
            !drive.held.is_held(2),
            "fixture still holds the original keyboard pad"
        );
        drive.velocity_hit(source, 127);
        drive.render_blocks(1);
        assert!(
            pads_play_badge(&mut drive),
            "a valid new-project voice lights PLAY: {source:?}"
        );
        drive.app.audio.take();
        assert!(
            !pads_play_badge(&mut drive),
            "missing audio cannot reuse the snapshot"
        );
    }
}

/// Scenario "Pads PLAY badge ignores previews and effect tails".
#[test]
fn pads_play_badge_ignores_previews_and_effect_tails() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.app.send(Command::SetPadSends {
            pad: 18,
            sends: [0.0, 0.8],
        });
        drive.app.send(Command::SetFx {
            slot: 1,
            settings: engine::fx::FxSettings::Delay {
                time_ms: 20.0,
                feedback: 0.8,
            },
        });
        drive.render_blocks(4);
        for as_pad in [None, Some(18)] {
            let chop = drive.app.chop.as_ref().unwrap();
            let sound = chop.sound.clone();
            let region = chop.slices()[3];
            drive.app.send(Command::PreviewAs { pad: as_pad });
            drive.app.send(Command::Preview {
                sound: Some(sound),
                region,
            });
            let wave = drive.render_blocks(32);
            assert!(engine::checks::peak(&wave.0) > 0.01);
            assert!(
                !pads_play_badge(&mut drive),
                "preview, including borrowed pad settings, is not a pad hit"
            );
        }
        drive.app.send(Command::Preview {
            sound: None,
            region: Region::WHOLE,
        });
        drive.render_blocks(4);
        drive.velocity_hit(source, 127);
        drive.render_blocks(64);
        assert!(pads_play_badge(&mut drive));
        drive.pad_release(source, 2);
        let tail = drive.render_blocks(16);
        assert!(!pads_play_badge(&mut drive));
        assert!(
            engine::checks::peak(&tail.0[1024..]) > 1e-5,
            "effect tail still sounds"
        );
        for channel in [&tail.0, &tail.1] {
            assert_eq!(engine::checks::first_non_finite(channel), None);
            assert!(engine::checks::peak(channel) < 1.0);
        }
    }
}

fn displayed_pad_frame(drive: &Drive) -> Option<usize> {
    let chop = drive.app.chop.as_ref()?;
    let line = drive.app.chop_display(chop).playhead?;
    Some(chop.view.start + (line * (chop.view.end - chop.view.start) as f32).round() as usize)
}

fn check_pad_playback_line(source: Source) {
    for edit in [false, true] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.render_blocks(4);
        if edit {
            drive.tap(KeyCode::Enter, M::NONE);
            assert!(drive.app.edit);
        }
        assert_eq!(displayed_pad_frame(&drive), None);
        assert_full_hit(&drive.velocity_hit(source, 127), 18);
        let chop = drive.app.chop.as_ref().unwrap();
        let start = chop.slices()[3].start;
        let cursor = chop.cursor;
        assert_eq!(
            drive.app.chop_display(chop).mode,
            "LISTEN",
            "not lit before audio starts"
        );
        let mut wave = drive.render_blocks(1);
        assert_eq!(displayed_pad_frame(&drive), Some(start + RENDER_BLOCK));
        append_stereo(&mut wave, drive.render_blocks(3));
        assert_eq!(displayed_pad_frame(&drive), Some(start + 4 * RENDER_BLOCK));
        let chop = drive.app.chop.as_ref().unwrap();
        assert_eq!(
            chop.cursor, cursor,
            "progress must not drag the owner's cursor"
        );
        assert_eq!(
            drive.app.chop_display(chop).mode,
            "LISTEN ▶",
            "the owner-approved badge follows actual pad playback"
        );
        assert_eq!(drive.app.playhead(), None, "not a transport preview");
        assert!(
            drive.frame().commands().is_empty(),
            "drawing must not start another voice"
        );
        for samples in [&wave.0, &wave.1] {
            assert_eq!(engine::checks::first_non_finite(samples), None);
            assert!(engine::checks::peak(samples) < 1.0);
        }
    }
}

/// Scenario "Keyboard pad shows the Chop playback line".
#[test]
fn keyboard_pad_shows_the_chop_playback_line() {
    check_pad_playback_line(Source::Keyboard);
}

/// Scenario "MIDI pad shows the same Chop playback line".
#[test]
fn midi_pad_shows_the_same_chop_playback_line() {
    check_pad_playback_line(Source::Notes);
    check_pad_playback_line(Source::Button);
}

/// Scenario "Chop pad line follows pitch and retrigger".
#[test]
fn chop_pad_line_follows_pitch_and_retrigger() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for (pitch, step) in [(-12, 64), (0, 128), (12, 256)] {
            let mut drive = Drive::offline();
            drive.parity_pad(pitch, false);
            drive.render_blocks(4);
            drive.velocity_hit(source, 127);
            drive.render_blocks(3);
            assert_eq!(displayed_pad_frame(&drive), Some(144_000 + 3 * step));
            drive.pad_release(source, 2);
            assert_full_hit(&drive.velocity_hit(source, 127), 18);
            drive.render_blocks(1);
            assert_eq!(
                displayed_pad_frame(&drive),
                Some(144_000 + step),
                "{source:?}, pitch {pitch}"
            );
        }
    }
}

/// Scenario "Chop pad line ends with its voice".
#[test]
fn chop_pad_line_ends_with_its_voice() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for ending in ["end", "release", "hush"] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            drive.render_blocks(4);
            drive.velocity_hit(source, 127);
            drive.render_blocks(1);
            assert!(displayed_pad_frame(&drive).is_some());
            match ending {
                "release" => {
                    drive.pad_release(source, 2);
                    drive.render_blocks(1);
                    assert!(
                        displayed_pad_frame(&drive).is_some(),
                        "release still sounds"
                    );
                }
                "hush" => drive.app.send(Command::HushPads),
                _ => {}
            }
            let wave = drive.render_blocks(400);
            assert_eq!(displayed_pad_frame(&drive), None, "{source:?}, {ending}");
            assert_eq!(engine::checks::peak(&wave.0[49_000..]), 0.0);
        }
    }
}

/// Scenario "Chop pad line keeps transport and edits unchanged".
#[test]
fn chop_pad_line_keeps_transport_and_edits_unchanged() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.render_blocks(4);
        drive.velocity_hit(source, 127);
        drive.render_blocks(1);
        assert!(displayed_pad_frame(&drive).is_some());
        // Move within the same slice: the owner's cursor moves, no slice pick preview.
        drive.tap(KeyCode::Right, M::NONE);
        let cursor = drive.app.chop.as_ref().unwrap().cursor;
        drive.render_blocks(1);
        assert_eq!(drive.app.playhead(), None);
        let sounding = displayed_pad_frame(&drive).unwrap();
        assert_ne!(cursor, sounding);
        drive.tap(KeyCode::Char('m'), M::NONE);
        assert!(drive.app.chop.as_ref().unwrap().markers.contains(&cursor));
        assert!(!drive.app.chop.as_ref().unwrap().markers.contains(&sounding));
        assert_eq!(drive.app.chop.as_ref().unwrap().cursor, cursor);
        let play = drive.tap(KeyCode::Char(' '), M::NONE);
        assert!(
            play.ui
                .iter()
                .any(|cmd| matches!(cmd, Command::Preview { sound: Some(_), .. }))
        );
        assert!(
            !play
                .ui
                .iter()
                .any(|cmd| matches!(cmd, Command::PadPress { .. }))
        );
        // Existing cuts must be at least 10 ms apart; let the preview pass that gap.
        drive.render_blocks(8);
        assert_eq!(
            displayed_pad_frame(&drive),
            drive.app.playhead(),
            "Space line keeps priority"
        );
        let preview_at = drive.app.playhead().unwrap();
        drive.tap(KeyCode::Char('m'), M::NONE);
        assert!(
            drive
                .app
                .chop
                .as_ref()
                .unwrap()
                .markers
                .contains(&preview_at),
            "Space cut keeps its old rule"
        );
        let stop = drive.tap(KeyCode::Char(' '), M::NONE);
        assert!(
            stop.ui
                .iter()
                .any(|cmd| matches!(cmd, Command::Preview { sound: None, .. }))
        );
        drive.render_blocks(4);
        assert_eq!(drive.app.playhead(), None);
        assert!(
            displayed_pad_frame(&drive).is_some(),
            "stopping preview must not stop the pad"
        );
        drive.tap(KeyCode::Char('o'), M::ALT);
        drive.tap(KeyCode::Char(' '), M::NONE);
        drive.render_blocks(900);
        assert!(
            drive.app.playhead().is_some(),
            "existing Space loop still plays"
        );
        assert_eq!(displayed_pad_frame(&drive), drive.app.playhead());
    }
}

/// Scenario "Chop pad line rejects unrelated and silent hits".
#[test]
fn chop_pad_line_rejects_unrelated_and_silent_hits() {
    use std::sync::atomic::Ordering::{Acquire, Release};
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for guard in ["question", "learn", "queue"] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            drive.link_chop_pad(19, 2);
            let target = drive.app.pads.snapshot(19);
            for cmd in drive.app.pads.put_back(19, &target) {
                drive.app.send(cmd);
            }
            drive.render_blocks(4);
            drive.velocity_hit(Source::Keyboard, 127);
            drive.render_blocks(1);
            assert_eq!(displayed_pad_frame(&drive), Some(144_128));
            if guard == "question" {
                drive.app.pads.marked.insert(18);
                drive.app.ask_clear();
                drive.frame();
            } else if guard == "learn" {
                drive.tap(KeyCode::F(10), M::NONE);
            } else {
                let audio = drive.app.audio.as_mut().unwrap();
                while audio.commands.slots() > 0 {
                    audio
                        .commands
                        .push(ProjectCommand {
                            epoch: 0,
                            ready_at_send: true,
                            command: ProjectMessage::Command(Command::SetPadGain {
                                pad: 0,
                                gain: 1.0,
                            }),
                        })
                        .unwrap();
                }
            }
            drive.pad_press(source, 3);
            assert!(
                drive.app.audio_held[19].is_none(),
                "blocked hit has no audio hold"
            );
            drive.render_blocks(1);
            assert_eq!(
                displayed_pad_frame(&drive),
                Some(144_256),
                "older voice survives {guard}"
            );
        }
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        let sound = drive.app.chop.as_ref().unwrap().sound.clone();
        drive.load_pad(19, Arc::new(Sample::mono(vec![0.1; 96_000])), "other");
        let snap = drive.app.pads.snapshot(19);
        for cmd in drive.app.pads.put_back(19, &snap) {
            drive.app.send(cmd);
        }
        drive.render_blocks(4);
        drive.pad_press(source, 3);
        drive.render_blocks(1);
        assert_eq!(
            displayed_pad_frame(&drive),
            None,
            "another sample is not this waveform"
        );
        let chop = drive.app.chop.as_ref().unwrap();
        assert_eq!(drive.app.chop_display(chop).mode, "LISTEN");
        drive.velocity_hit(Source::Keyboard, 127);
        drive.render_blocks(1);
        assert!(displayed_pad_frame(&drive).is_some());
        drive.app.apply_session(session::Session::default());
        // Reuse the very same Sample allocation to test epoch, not just pointer filtering.
        drive.prepare_chop(sound, 4);
        drive.link_chop_pad(18, 3);
        assert_eq!(
            displayed_pad_frame(&drive),
            None,
            "old project position is hidden immediately"
        );
        let chop = drive.app.chop.as_ref().unwrap();
        assert_eq!(drive.app.chop_display(chop).mode, "LISTEN");
        for _ in 0..256 {
            drive.app.flush_audio();
            drive.render_blocks(1);
            assert_eq!(displayed_pad_frame(&drive), None);
            if drive.project.ready.load(Acquire) == 1 && drive.app.pending_audio.is_empty() {
                break;
            }
        }
        assert_eq!(drive.project.ready.load(Acquire), 1);
        drive.project.ready.store(0, Release);
        drive.velocity_hit(source, 127);
        drive.project.ready.store(1, Release);
        drive.render_blocks(4);
        assert_eq!(
            displayed_pad_frame(&drive),
            None,
            "not-ready hits cannot create a line"
        );
        let chop = drive.app.chop.as_ref().unwrap();
        assert_eq!(drive.app.chop_display(chop).mode, "LISTEN");
    }
}

/// Scenario "Chop listen badge stays lit beyond the waveform view".
#[test]
fn chop_pad_line_uses_the_visible_wave_and_current_chop() {
    let mut drive = Drive::offline();
    drive.parity_pad(0, false);
    drive.render_blocks(4);
    drive.velocity_hit(Source::Keyboard, 127);
    drive.render_blocks(4);
    for _ in 0..4 {
        drive.tap(KeyCode::Char('i'), M::NONE);
    }
    let chop = drive.app.chop.as_ref().unwrap();
    let view = drive.app.chop_display(chop);
    let expected = (144_512 - chop.view.start) as f32 / (chop.view.end - chop.view.start) as f32;
    assert_eq!(view.playhead, Some(expected));
    assert_eq!(displayed_pad_frame(&drive), Some(144_512));
    for _ in 0..4 {
        drive.tap(KeyCode::Char('i'), M::NONE);
    }
    assert_eq!(
        displayed_pad_frame(&drive),
        None,
        "do not clamp offscreen progress to an edge"
    );
    let chop = drive.app.chop.as_ref().unwrap();
    assert_eq!(
        drive.app.chop_display(chop).mode,
        "LISTEN ▶",
        "offscreen voice still plays"
    );
    for _ in 0..8 {
        drive.tap(KeyCode::Char('k'), M::NONE);
    }
    assert_eq!(displayed_pad_frame(&drive), Some(144_512));
    drive.app.pads.link[18].as_mut().unwrap().1 += 1;
    assert_eq!(
        displayed_pad_frame(&drive),
        None,
        "another chop of the same Sample is not this chop"
    );
    let chop = drive.app.chop.as_ref().unwrap();
    assert_eq!(drive.app.chop_display(chop).mode, "LISTEN");
}

/// Scenario "Chop listen badge lights only while playback runs".
#[test]
fn chop_listen_badge_lights_only_while_playback_runs() {
    use engine::checks::peak;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.app.send(Command::SetPadSends {
            pad: 18,
            sends: [0.0, 0.8],
        });
        drive.app.send(Command::SetFx {
            slot: 1,
            settings: engine::fx::FxSettings::Delay {
                time_ms: 20.0,
                feedback: 0.8,
            },
        });
        drive.render_blocks(4);
        let mode = |d: &Drive| d.app.chop_display(d.app.chop.as_ref().unwrap()).mode;
        assert_eq!(mode(&drive), "LISTEN");
        drive.velocity_hit(source, 127);
        assert_eq!(mode(&drive), "LISTEN", "queued input alone is not playback");
        drive.render_blocks(64);
        assert_eq!(mode(&drive), "LISTEN ▶");
        drive.pad_release(source, 2);
        let tail = drive.render_blocks(16);
        assert_eq!(mode(&drive), "LISTEN");
        assert!(
            peak(&tail.0[1_024..]) > 1e-5,
            "effect tail alone must not keep the badge lit"
        );
        drive.tap(KeyCode::Char(' '), M::NONE);
        drive.render_blocks(1);
        assert_eq!(mode(&drive), "LISTEN ▶");
        drive.tap(KeyCode::Char(' '), M::NONE);
        drive.render_blocks(4);
        assert_eq!(mode(&drive), "LISTEN");
    }
}

/// Scenario "The pad edit of an empty pad shows its test tone".
#[test]
fn the_pad_edit_of_an_empty_pad_shows_its_test_tone() {
    let mut drive = Drive::new();
    drive.app.goto(Screen::Pads);
    drive.app.pads.selected = 8;
    assert!(drive.app.pads.sound[8].is_none(), "pad A is empty");
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(drive.app.edit);
    let view = crate::big_view(&drive.app.pads, 48_000).expect("the test tone piece");
    // the decaying tone: loud at the start, quiet at the end
    let head = view.wave[..4].iter().copied().fold(0.0f32, f32::max);
    let tail = view.wave[view.wave.len() - 4..]
        .iter()
        .copied()
        .fold(0.0f32, f32::max);
    assert!(head > 0.1 && tail < head / 4.0, "{head} {tail}");
    let short = view.attack;
    drive.app.pads.attack_ms[8] = 300;
    let longer = crate::big_view(&drive.app.pads, 48_000).unwrap().attack;
    assert!(longer > short, "{short} -> {longer}");
    // the view hands that piece to the strip, which draws it in the edit in place of the
    // live output
    let pads = pads_view(&mut drive.app.pads, None, None, vec![0.9; 64], None, 48_000);
    assert!(pads.big.is_some());
}

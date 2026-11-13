//! Recorded feedback, status, questions, and value overlays.

use super::*;

pub(super) fn shared_feedback_rows(
    drive: &mut Drive,
    width: u16,
    height: u16,
    now: Instant,
) -> Vec<String> {
    use ratatui::{Terminal, backend::TestBackend};
    let app = &mut drive.app;
    app.input_sources.advance(now);
    let feedback = app.status_feedback_at(now);
    let learning = app.learning_view(&drive.keymap);
    let question = app.question_view(&drive.keymap);
    let pads = pads_view(&mut app.pads, None, None, Vec::new(), None, app.rate);
    let screen = tui::Screen::Pads(app.mark_pads(pads));
    let status = tui::Status {
        badges: app.title_badges_at(now),
        save: app.save_status(),
        backend: app.audio_alive().map(|audio| audio.backend),
        recording: app.recording.is_some(),
        buffer_size: 128,
        sample_rate: 48_000,
        load_peak: 0.0,
        xruns: 0,
        master_db: app.master_db,
        meters: ([-60.0; 2], [-60.0; 2], None),
        notice: None,
        no_releases: false,
        project: app.project.as_ref().map(|project| project.name.as_str()),
    };
    let help = tui::Help {
        groups: &[],
        hint: None,
        full: None,
        question: question.as_ref(),
        learn: learning.as_ref(),
    };
    let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
    term.draw(|frame| tui::draw_feedback(frame, &status, &screen, &help, &feedback, None))
        .unwrap();
    term.backend()
        .buffer()
        .content()
        .chunks(width as usize)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect())
        .collect()
}

struct ImageFrame {
    rows: Vec<String>,
    before: Vec<String>,
    after: Vec<String>,
    blockers: Vec<ratatui::layout::Rect>,
    images: Vec<ratatui::layout::Rect>,
    visible_images: Vec<ratatui::layout::Rect>,
}

fn image_frame_at(
    drive: &mut Drive,
    state: &mut graphics::State,
    enabled: bool,
    size: (u16, u16),
    now: Instant,
) -> ImageFrame {
    use ratatui::{Terminal, backend::TestBackend};
    let app = &mut drive.app;
    app.frame_notice_at(now);
    app.input_sources.advance(now);
    let value = app.current_value_at(now).cloned();
    let question = app.question_view(&drive.keymap);
    let feedback = app.status_feedback_at(now);
    let mut view = app.chop_display(app.chop.as_ref().unwrap());
    if view.slice_keys.is_none() {
        bank_waves(&mut view.pads, &mut app.pads, app.rate);
    }
    view.which = app.which_chop();
    app.learn_grid(&mut view.pads.pads);
    let screen = tui::Screen::Chop(Box::new(app.mark_chop(view, None)));
    let status = tui::Status {
        badges: app.title_badges_at(now),
        save: app.save_status(),
        backend: Some("offline"),
        recording: false,
        buffer_size: 128,
        sample_rate: 48_000,
        load_peak: 0.0,
        xruns: 0,
        master_db: app.master_db,
        meters: ([-60.0; 2], [-60.0; 2], None),
        notice: None,
        no_releases: false,
        project: None,
    };
    let keys = full_help(&drive.keymap, app.screen, true);
    let messages = app.message_history();
    let help = tui::Help {
        groups: &[],
        hint: hint_for(app),
        full: app
            .help_open
            .then(|| app.full_help_view(&keys, &messages))
            .flatten(),
        question: question.as_ref(),
        learn: None,
    };
    let mut term = Terminal::new(TestBackend::new(size.0, size.1)).unwrap();
    let mut before = Vec::new();
    let mut want = Vec::new();
    let mut blockers = Vec::new();
    term.draw(|frame| {
        blockers = tui::overlay_areas(frame.area(), &screen, &help, value.as_ref());
        before = state.before(enabled, app.screen, frame.area(), (2, 4), &blockers);
        tui::draw_feedback(
            frame,
            &status,
            &screen,
            &help,
            &feedback,
            enabled.then_some(&mut want),
        );
        if let Some(value) = &value {
            tui::value_popup(frame, value);
        }
    })
    .unwrap();
    let images = want.iter().map(|image| image.area).collect();
    let after = state.after(&mut want, &blockers);
    let visible_images = want.iter().map(|image| image.area).collect();
    let rows = term
        .backend()
        .buffer()
        .content()
        .chunks(size.0 as usize)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect())
        .collect();
    ImageFrame {
        rows,
        before,
        after,
        blockers,
        images,
        visible_images,
    }
}

/// Scenario "A value overlay restores its wave" with real keyboard/raw-MIDI setters.
#[test]
fn a_value_overlay_restores_its_wave() {
    for enabled in [false, true] {
        for source in [Source::Keyboard, Source::Button] {
            for size in [(80, 26), (120, 40)] {
                let mut drive = Drive::registry_screen_checks();
                let mut state = graphics::State::default();
                let baseline =
                    image_frame_at(&mut drive, &mut state, enabled, size, Instant::now());
                assert_eq!(
                    baseline
                        .after
                        .iter()
                        .any(|command| command.contains("a=t,f=32")),
                    enabled
                );
                match source {
                    Source::Keyboard => {
                        drive.tap(KeyCode::Char(']'), M::NONE);
                    }
                    _ => {
                        value_knob(&mut drive, "master.volume", 100);
                    }
                }
                let at = drive.app.value_popup.as_ref().unwrap().at;
                let popup = image_frame_at(&mut drive, &mut state, enabled, size, at);
                assert!(
                    popup.rows.iter().any(|row| row.contains("volume · Master")),
                    "{}",
                    popup.rows.join("\n")
                );
                let intersects = popup
                    .images
                    .iter()
                    .any(|image| popup.blockers.iter().any(|area| image.intersects(*area)));
                assert_eq!(
                    popup
                        .before
                        .iter()
                        .any(|command| command.contains("a=d,d=i")),
                    intersects
                );
                assert!(
                    popup.after.is_empty(),
                    "an unchanged unobstructed wave does not blink"
                );
                let retained = image_frame_at(
                    &mut drive,
                    &mut state,
                    enabled,
                    size,
                    at + LIT_FOR - Duration::from_nanos(1),
                );
                assert!(
                    retained
                        .rows
                        .iter()
                        .any(|row| row.contains("volume · Master"))
                );
                assert!(retained.before.is_empty() && retained.after.is_empty());
                let restored = image_frame_at(&mut drive, &mut state, enabled, size, at + LIT_FOR);
                assert!(drive.app.current_value_at(at + LIT_FOR).is_none());
                assert!(
                    restored
                        .rows
                        .iter()
                        .all(|row| !row.contains("volume · Master"))
                );
                assert!(restored.before.is_empty());
                assert_eq!(
                    restored
                        .after
                        .iter()
                        .any(|command| command.contains("a=t,f=32")),
                    intersects
                );
                assert_eq!(
                    restored
                        .after
                        .iter()
                        .any(|command| command.contains("a=p,")),
                    intersects
                );
                if intersects {
                    assert_eq!(restored.after, baseline.after);
                }
                if enabled {
                    // Current Chop waves start below this popup. Exercise a covering image fixture
                    // with the real popup rectangle and the same production expiry snapshots.
                    let area = popup.blockers[0];
                    let image = || {
                        vec![tui::WaveImage {
                            id: 1,
                            area,
                            peaks: vec![0.7; area.width as usize * 2],
                            colors: vec![ratatui::style::Color::Rgb(1, 2, 3); area.width as usize],
                        }]
                    };
                    let terminal = ratatui::layout::Rect::new(0, 0, size.0, size.1);
                    let mut covered = graphics::State::default();
                    covered.before(true, Screen::Chop, terminal, (2, 4), &[]);
                    let original = covered.after(&mut image(), &[]);
                    let hidden =
                        covered.before(true, Screen::Chop, terminal, (2, 4), &popup.blockers);
                    assert!(hidden.iter().any(|command| command.contains("a=d,d=i")));
                    assert!(covered.after(&mut image(), &popup.blockers).is_empty());
                    assert!(
                        covered
                            .before(true, Screen::Chop, terminal, (2, 4), &retained.blockers)
                            .is_empty()
                    );
                    assert!(covered.after(&mut image(), &retained.blockers).is_empty());
                    assert!(restored.blockers.is_empty());
                    covered.before(true, Screen::Chop, terminal, (2, 4), &restored.blockers);
                    assert_eq!(
                        covered.after(&mut image(), &restored.blockers),
                        original,
                        "a covering wave restores at the exact deadline without another input"
                    );
                }
            }
        }
    }
}

/// Scenario "Resize and questions keep overlays readable", also with F1 dismissal.
#[test]
fn resize_and_questions_keep_overlays_readable() {
    for enabled in [false, true] {
        for overlay in ["question", "help"] {
            let mut drive = Drive::registry_screen_checks();
            let mut state = graphics::State::default();
            image_frame_at(&mut drive, &mut state, enabled, (80, 26), Instant::now());
            drive.tap(KeyCode::Char(']'), M::NONE);
            let at = drive.app.value_popup.as_ref().unwrap().at;
            image_frame_at(&mut drive, &mut state, enabled, (80, 26), at);
            if overlay == "question" {
                drive.load_pad(18, Arc::new(Sample::mono(vec![0.25; 48_000])), "old target");
                drive.tap(KeyCode::PageDown, M::NONE);
                drive.tap(KeyCode::Char('p'), M::NONE);
                drive.pad_press(Source::Keyboard, 2);
                drive.pad_release(Source::Keyboard, 2);
            }
            drive.tap(
                if overlay == "question" {
                    KeyCode::Enter
                } else {
                    KeyCode::F(1)
                },
                M::NONE,
            );
            for (step, size) in [(80, 26), (120, 40), (80, 26)].into_iter().enumerate() {
                let shown = image_frame_at(
                    &mut drive,
                    &mut state,
                    enabled,
                    size,
                    at + Duration::from_millis(100 + step as u64),
                );
                assert!(
                    shown
                        .rows
                        .iter()
                        .all(|row| !row.contains("volume · Master"))
                );
                if overlay == "question" {
                    assert!(
                        shown
                            .rows
                            .iter()
                            .any(|row| row.contains("KB Enter: replace · Esc: keep")),
                        "{}",
                        shown.rows.join("\n")
                    );
                    assert!(
                        shown
                            .rows
                            .iter()
                            .any(|row| row.contains("This cannot be undone."))
                    );
                } else {
                    assert!(shown.rows.iter().any(|row| row.contains("HELP")));
                }
                assert!(
                    shown
                        .visible_images
                        .iter()
                        .all(|image| shown.blockers.iter().all(|area| !image.intersects(*area))),
                    "no image can cover the text"
                );
                if shown
                    .images
                    .iter()
                    .all(|image| shown.blockers.iter().any(|area| image.intersects(*area)))
                {
                    assert!(shown.after.is_empty(), "a fully covered wave stays absent");
                }
            }
            let expired = image_frame_at(&mut drive, &mut state, enabled, (80, 26), at + LIT_FOR);
            let was_hidden = enabled && expired.visible_images.is_empty();
            assert!(
                drive.app.current_value_at(at + LIT_FOR).is_none(),
                "hidden feedback still expires"
            );
            drive.tap(KeyCode::Esc, M::NONE);
            assert!(drive.app.ask.is_none() && !drive.app.help_open);
            let restored = image_frame_at(&mut drive, &mut state, enabled, (80, 26), at + LIT_FOR);
            assert_eq!(
                restored
                    .after
                    .iter()
                    .any(|command| command.contains("a=p,")),
                was_hidden
            );
            assert_eq!(
                !restored.visible_images.is_empty(),
                enabled,
                "an unobstructed wave must remain available"
            );
            assert!(
                restored
                    .rows
                    .iter()
                    .all(|row| !row.contains("volume · Master"))
            );
        }
    }
}

fn working_question_drive(kind: &str) -> Drive {
    let mut drive = Drive::new();
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    drive.load_pad(0, sound.clone(), "source");
    drive.load_pad(18, sound.clone(), "old target");
    match kind {
        "copy" => {
            drive.tap(KeyCode::Char('c'), M::CONTROL);
            drive.tap(KeyCode::PageDown, M::NONE);
            drive.pad_press(Source::Keyboard, 2);
            drive.pad_release(Source::Keyboard, 2);
            assert!(matches!(drive.app.ask, Some(Ask::CopyPad { .. })));
        }
        "file" => {
            drive.ordered_file();
            drive.tap(KeyCode::Enter, M::NONE);
            drive.tap(KeyCode::F(2), M::NONE);
            drive.tap(KeyCode::PageDown, M::NONE);
            drive.pad_press(Source::Keyboard, 2);
            drive.pad_release(Source::Keyboard, 2);
            assert!(matches!(drive.app.ask, Some(Ask::PutFile { .. })));
        }
        "plan" => {
            drive.prepare_chop(sound.clone(), 4);
            drive.load_pad(18, sound, "old target");
            drive.tap(KeyCode::PageDown, M::NONE);
            drive.tap(KeyCode::Char('p'), M::NONE);
            drive.pad_press(Source::Keyboard, 2);
            drive.pad_release(Source::Keyboard, 2);
            drive.tap(KeyCode::Enter, M::NONE);
            assert!(matches!(drive.app.ask, Some(Ask::Plan { .. })));
        }
        "reset" | "clear" => {
            drive.app.pads.volume_db[18] = -6;
            drive.tap(KeyCode::PageDown, M::NONE);
            drive.app.pads.selected = 2;
            if kind == "reset" {
                drive.tap(KeyCode::Enter, M::NONE);
            } else {
                drive.app.pads.marked.insert(18);
            }
            drive.tap(KeyCode::Backspace, M::NONE);
            assert!(if kind == "reset" {
                matches!(drive.app.ask, Some(Ask::ResetPads { .. }))
            } else {
                matches!(drive.app.ask, Some(Ask::ClearPads { .. }))
            });
        }
        "profile" => {
            drive.tap(KeyCode::F(12), M::NONE);
            drive.tap(KeyCode::Tab, M::NONE);
            drive.tap(KeyCode::Tab, M::NONE);
            drive.app.midi_row = drive
                .app
                .midi_list()
                .iter()
                .position(
                    |row| matches!(row, MidiListRow::Reset { name, .. } if name == "Recorded pads"),
                )
                .unwrap();
            drive.tap(KeyCode::Enter, M::NONE);
            assert!(matches!(drive.app.ask, Some(Ask::ResetProfile { .. })));
        }
        _ => panic!("unknown question fixture: {kind}"),
    }
    drive
}

/// Scenario "Questions show effective working answers" for every existing question.
#[test]
fn questions_show_effective_working_answers() {
    for kind in ["copy", "file", "plan", "reset", "clear", "profile"] {
        for route in ["default", "override", "midi"] {
            for confirm in [false, true] {
                let mut drive = working_question_drive(kind);
                let action = match kind {
                    "profile" => "settings.learn",
                    "plan" => "chop.edit",
                    "file" => "browser.hold",
                    _ => "pad.edit",
                };
                drive.bind_action(5, action);
                drive.bind_action(6, "ui.back");
                if route == "override" {
                    let warnings = drive.keymap.overlay(&format!("[global]\nEsc = \"none\"\nF8 = \"ui.back\"\n[screens.{}]\nEnter = \"none\"\nF9 = \"{action}\"\n", drive.app.screen.key())).unwrap();
                    assert!(warnings.is_empty(), "{warnings:?}");
                }
                let stored = drive.app.ask.as_ref().unwrap().lines().to_vec();
                let old = drive.app.pads.snapshot(18);
                let history = drive.app.history.len();
                let snapshot = drive.app.question_view(&drive.keymap).unwrap();
                let verb = match kind {
                    "reset" | "profile" => "reset",
                    "clear" => "clear",
                    _ => "replace",
                };
                let keys = if route == "override" {
                    format!("KB F9: {verb} · F8: keep")
                } else {
                    format!("KB Enter: {verb} · Esc: keep")
                };
                assert_eq!(snapshot.answers[0], keys, "{kind}/{route}");
                assert!(
                    snapshot
                        .answers
                        .iter()
                        .any(|row| row == &format!("M1 ch1 cc 5: {verb} · ch1 cc 6: keep")),
                    "{kind}: {:?}",
                    snapshot.answers
                );
                for (width, height) in [(80, 26), (120, 40)] {
                    let rows = shared_feedback_rows(&mut drive, width, height, Instant::now());
                    assert!(rows[1].contains("ASK"), "{kind}/{route}: {}", rows[1]);
                    assert!(
                        rows.iter().any(|row| row.contains(&keys)),
                        "{kind}/{route}: {}",
                        rows.join("\n")
                    );
                }
                assert_eq!(drive.app.ask.as_ref().unwrap().lines(), stored);
                assert_eq!(drive.app.pads.snapshot(18), old);
                let hit = drive.pad_press(Source::Button, 4);
                assert!(hit.commands().is_empty());
                drive.pad_release(Source::Button, 4);
                assert_eq!(
                    drive.app.ask.as_ref().unwrap().lines(),
                    stored,
                    "a pad cannot answer"
                );
                if route == "override" {
                    drive.tap(KeyCode::Enter, M::NONE);
                    drive.tap(KeyCode::Esc, M::NONE);
                    assert_eq!(
                        drive.app.ask.as_ref().unwrap().lines(),
                        stored,
                        "old key labels cannot answer"
                    );
                }
                match route {
                    "midi" => {
                        drive.registry_command(
                            Source::Button,
                            KeyCode::Enter,
                            M::NONE,
                            if confirm { 5 } else { 6 },
                        );
                    }
                    "override" => {
                        drive.tap(KeyCode::F(if confirm { 9 } else { 8 }), M::NONE);
                    }
                    _ => {
                        drive.tap(
                            if confirm {
                                KeyCode::Enter
                            } else {
                                KeyCode::Esc
                            },
                            M::NONE,
                        );
                    }
                }
                assert!(drive.app.ask.is_none(), "{kind}/{route}/{confirm}");
                if !confirm {
                    assert_eq!(drive.app.pads.snapshot(18), old);
                    assert_eq!(drive.app.history.len(), history);
                    continue;
                }
                match kind {
                    "copy" => {
                        assert_eq!(
                            drive.app.pads.custom_name[18].as_deref(),
                            Some("source (copy 1)")
                        )
                    }
                    "file" => assert!(drive.app.browser.in_hand.is_none()),
                    "plan" => assert!(drive.app.pads.link[18].is_some()),
                    "reset" => assert_eq!(drive.app.pads.volume_db[18], 0),
                    "clear" => assert!(drive.app.pads.sound[18].is_none()),
                    "profile" => {
                        assert!(!drive.app.learned["Recorded pads"]["global"].contains_key("cc 5"))
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}

/// Scenario "Missing controller confirmation stays honest".
#[test]
fn missing_controller_confirmation_stays_honest() {
    use input::{Group, midi::Key};
    let mut drive = working_question_drive("profile");
    let view = drive.app.question_view(&drive.keymap).unwrap();
    assert_eq!(view.answers[0], "KB Enter: reset · Esc: keep");
    assert!(
        view.answers
            .iter()
            .skip(1)
            .all(|row| !row.contains(": reset")),
        "{:?}",
        view.answers
    );
    drive.bind_action(5, "settings.learn");
    assert!(
        drive
            .app
            .question_view(&drive.keymap)
            .unwrap()
            .answers
            .iter()
            .any(|row| row.contains("cc 5: reset"))
    );
    learn::assign(
        &mut drive.app.learned,
        "Recorded pads",
        Group::Settings,
        Key::Cc(5),
        "settings.tab_next",
    );
    drive.app.sync_learned("Recorded pads", Group::Settings);
    drive.midi().recorded_fence();
    assert!(
        drive
            .app
            .question_view(&drive.keymap)
            .unwrap()
            .answers
            .iter()
            .all(|row| !row.contains("cc 5:")),
        "a shadowed global binding cannot answer"
    );
    drive.registry_command(Source::Button, KeyCode::Enter, M::NONE, 5);
    assert!(drive.app.ask.is_some());
    drive.midi().recorded_disconnect(PAD_PORT);
    drive.midi().recorded_connect("Unprofiled MIDI input");
    drive.frame();
    let view = drive.app.question_view(&drive.keymap).unwrap();
    assert!(
        view.answers
            .iter()
            .all(|row| !row.contains(": reset") || row.starts_with("KB "))
    );
    assert!(
        view.answers
            .iter()
            .all(|row| !row.contains("STOP") && !row.contains("Apply"))
    );
    let warnings = drive
        .keymap
        .overlay("[screens.settings]\nEnter = \"none\"\n")
        .unwrap();
    assert!(warnings.is_empty());
    assert!(
        drive.app.question_view(&drive.keymap).unwrap().answers[0].starts_with("KB no confirm key")
    );
    assert!(drive.app.ask.is_some());
}

#[test]
fn question_answers_keep_midi_edit_precedence_and_target_rechecks() {
    use input::{Group, midi::Key};
    let mut drive = working_question_drive("reset");
    drive.bind_action(5, "pad.edit");
    learn::assign(
        &mut drive.app.learned,
        "Recorded pads",
        Group::PadsEdit,
        Key::Cc(5),
        "pad.volume",
    );
    drive.app.sync_learned("Recorded pads", Group::PadsEdit);
    drive.midi().recorded_fence();
    let view = drive.app.question_view(&drive.keymap).unwrap();
    assert!(drive.app.edit);
    assert!(
        view.answers[0].contains("Enter: reset"),
        "keyboard keeps the normal question layer"
    );
    assert!(
        view.answers.iter().all(|row| !row.contains("cc 5:")),
        "MIDI keeps the actual edit layer"
    );
    drive.registry_command(Source::Button, KeyCode::Enter, M::NONE, 5);
    assert!(drive.app.ask.is_some());
    drive.registry_command(Source::Button, KeyCode::Enter, M::NONE, 29);
    assert!(drive.app.ask.is_none());
    assert_eq!(drive.app.pads.volume_db[18], 0);
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = working_question_drive("copy");
        drive.bind_action(5, "pad.edit");
        let stored = drive.app.ask.as_ref().unwrap().lines().to_vec();
        drive.app.pads.custom_name[18] = Some("changed after the question".into());
        drive.app.question_view(&drive.keymap);
        assert_eq!(drive.app.ask.as_ref().unwrap().lines(), stored);
        drive.registry_command(source, KeyCode::Enter, M::NONE, 5);
        let updated = drive
            .app
            .ask
            .as_ref()
            .expect("a changed target asks again")
            .lines();
        assert_ne!(updated, stored);
        assert!(
            updated
                .iter()
                .any(|line| line.contains("changed after the question"))
        );
        assert!(drive.app.copy_in_hand.is_some());
        assert!(drive.app.history.is_empty());
        drive.registry_command(source, KeyCode::Enter, M::NONE, 5);
        assert!(drive.app.ask.is_none());
        assert_eq!(
            drive.app.pads.custom_name[18].as_deref(),
            Some("source (copy 1)")
        );
        assert_eq!(drive.app.history.len(), 1);
    }
}

#[test]
fn feedback_cache_tracks_source_identity_and_activity() {
    let mut drive = Drive::new();
    let mut text = view::FrameText::default();
    let at = Instant::now();
    let first = text.feedback_at(&drive.app, at).inputs;
    let again = text
        .feedback_at(&drive.app, at + Duration::from_millis(20))
        .inputs;
    assert!(Arc::ptr_eq(&first, &again));
    assert_eq!(text.builds().1, 1);
    drive.bytes(XS_PORT, &[0xb0, 15, 64]);
    drive.frame();
    let receipt = drive.app.input_sources.latest.as_ref().unwrap().1;
    let active = text.feedback_at(&drive.app, receipt).inputs;
    assert!(!Arc::ptr_eq(&first, &active));
    assert!(
        active
            .iter()
            .any(|input| input.compact == "M2" && input.active)
    );
    assert!(Arc::ptr_eq(
        &active,
        &text.feedback_at(&drive.app, receipt).inputs
    ));
    drive.app.input_sources.advance(receipt + LIT_FOR);
    let expired = text.feedback_at(&drive.app, receipt + LIT_FOR).inputs;
    assert!(expired.iter().all(|input| !input.active));
    assert!(!Arc::ptr_eq(&active, &expired));
    drive.app.apply_session(session::Session::default());
    assert!(Arc::ptr_eq(
        &expired,
        &text.feedback_at(&drive.app, receipt + LIT_FOR).inputs
    ));
    drive.midi().recorded_disconnect(XS_PORT);
    drive.frame();
    let removed = text.feedback_at(&drive.app, Instant::now()).inputs;
    assert_eq!(removed.len(), 2);
    assert_eq!(removed[0].compact, "KB");
    assert!(removed[1].compact.trim().is_empty());
    drive.midi().recorded_connect(XS_PORT);
    drive.frame();
    let connected = text.feedback_at(&drive.app, Instant::now()).inputs;
    assert_eq!(connected.len(), 2);
    assert_eq!(connected[0].compact, "KB");
    assert!(connected[1].compact.trim().is_empty());
    assert!(connected.iter().all(|input| !input.active));
    assert!(
        !Arc::ptr_eq(&expired, &connected),
        "equal names do not reuse old connection data"
    );
    assert_eq!(text.builds().1, 5);
}

#[test]
fn badge_cache_tracks_real_screen_mode_and_page_changes() {
    for source in [Source::Keyboard, Source::Button] {
        for (screen, key, number) in REGISTRY_SCREEN_COMMANDS {
            // F3 on Files opens its file. Each screen starts from the same pad fixture.
            let mut drive = Drive::registry_screen_checks();
            drive.bind_action(111, "edit.page_next");
            let mut text = view::FrameText::default();
            text.badges_at(&drive.app, Instant::now());
            drive.registry_command(source, key, M::NONE, number);
            assert_eq!(drive.app.screen, screen);
            let now = Instant::now();
            assert_eq!(
                text.badges_at(&drive.app, now),
                drive.app.title_badges_at(now)
            );
            let builds = text.builds().0;
            text.badges_at(&drive.app, now);
            assert_eq!(text.builds().0, builds);
            if matches!(screen, Screen::Pads | Screen::Chop) {
                drive.registry_command(
                    source,
                    KeyCode::Enter,
                    M::NONE,
                    if screen == Screen::Pads { 97 } else { 98 },
                );
                let now = Instant::now();
                assert_eq!(
                    text.badges_at(&drive.app, now),
                    drive.app.title_badges_at(now)
                );
                assert!(text.builds().0 > builds);
                if screen == Screen::Pads {
                    let page = drive.app.help_state().page;
                    drive.registry_command(source, KeyCode::Tab, M::NONE, 111);
                    assert_ne!(drive.app.help_state().page, page);
                    let now = Instant::now();
                    assert_eq!(
                        text.badges_at(&drive.app, now),
                        drive.app.title_badges_at(now)
                    );
                    assert!(
                        text.badges_at(&drive.app, now)
                            .iter()
                            .any(|badge| badge.normal.contains(drive.app.help_state().page))
                    );
                }
            }
        }
    }
}

/// Scenario "Unchanged views do not freeze timed feedback" through real keyboard and MIDI setters.
#[test]
fn unchanged_views_do_not_freeze_timed_feedback() {
    for midi in [false, true] {
        let mut drive = value_drive();
        if midi {
            value_knob(&mut drive, "pad.volume", 100);
        } else {
            drive.tap(KeyCode::Right, M::NONE);
        }
        let at = drive.app.value_popup.as_ref().unwrap().at;
        drive.app.notices = Notices::new(at);
        drive
            .app
            .post_system_notice(Level::Info, ("cache timer".into(), at));
        assert_eq!(
            drive.app.frame_notice_at(at).as_deref(),
            Some("cache timer")
        );
        let mut text = view::FrameText::default();
        let expected = drive.app.title_badges_at(at);
        for now in [
            at,
            at + LIT_FOR - Duration::from_nanos(1),
            at + LIT_FOR,
            at + Duration::from_secs(3),
        ] {
            drive.app.input_sources.advance(now);
            let notice = drive.app.frame_notice_at(now);
            assert_eq!(notice.is_some(), now < at + Duration::from_secs(3));
            assert_eq!(
                drive.app.current_value_at(now).is_some(),
                now < at + LIT_FOR
            );
            assert_eq!(text.badges_at(&drive.app, now), expected);
            text.feedback_at(&drive.app, now);
            assert_eq!(
                text.builds().0,
                1,
                "timers do not reconstruct unchanged badge text"
            );
        }
        assert!(drive.app.value_popup.is_none());
        assert!(drive.app.input_sources.active.is_none());
    }
}

pub(super) fn input_device(drive: &Drive, port: &str) -> input_sources::Source {
    input_sources::Source::Device {
        port: port.into(),
        connection: drive.midi().recorded_connection(port).unwrap(),
    }
}

/// Scenario "Keyboard stays visible without controllers".
#[test]
fn keyboard_stays_visible_without_controllers() {
    let mut drive = Drive::offline();
    let ports: Vec<_> = drive
        .app
        .input_sources
        .connected()
        .map(|(port, _)| port.to_owned())
        .collect();
    for port in ports {
        drive.midi().recorded_disconnect(&port);
    }
    drive.frame();
    let mut cache = view::FrameText::default();
    let initial = cache.feedback_at(&drive.app, Instant::now()).inputs;
    assert_eq!(initial.len(), 1);
    assert_eq!(initial[0].compact, "KB");
    assert!(!initial[0].active);
    drive.key(KeyCode::Char('?'), M::NONE, KeyEventKind::Press);
    let receipt = drive.app.input_sources.latest.clone().unwrap();
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        drive.key(KeyCode::Char('?'), M::NONE, kind);
        assert_eq!(drive.app.input_sources.latest.as_ref(), Some(&receipt));
    }
    for (width, height) in [(80, 26), (120, 40)] {
        let before = shared_feedback_rows(
            &mut drive,
            width,
            height,
            receipt.1 - Duration::from_nanos(1),
        )[0]
        .clone();
        let mut column = None;
        assert!(before.contains("KB") && !before.contains('▸'), "{before}");
        for now in [
            receipt.1,
            receipt.1 + LIT_FOR - Duration::from_nanos(1),
            receipt.1 + LIT_FOR,
        ] {
            let feedback = cache.feedback_at(&drive.app, now);
            assert_eq!(feedback.inputs[0].active, now < receipt.1 + LIT_FOR);
            assert!(Arc::ptr_eq(
                &feedback.inputs,
                &cache.feedback_at(&drive.app, now).inputs
            ));
            let row = &shared_feedback_rows(&mut drive, width, height, now)[0];
            assert!(row.contains("KB"), "{row}");
            assert_eq!(row.contains("▸KB"), now < receipt.1 + LIT_FOR, "{row}");
            let current = row
                .find("offline")
                .map(|index| ratatui::text::Line::from(&row[..index]).width())
                .unwrap();
            assert_eq!(
                current,
                ratatui::text::Line::from(&before[..before.find("offline").unwrap()]).width()
            );
            if let Some(previous) = column {
                assert_eq!(current, previous);
            }
            column = Some(current);
        }
    }
}

/// Scenario "Status follows keyboard and connected inputs" through production input and shared drawing.
#[test]
fn status_follows_keyboard_and_connected_inputs() {
    for (_, key, _) in REGISTRY_SCREEN_COMMANDS {
        let mut drive = Drive::registry_screen_checks();
        drive.bind_action(88, "master.volume_up");
        drive.tap(key, M::NONE);
        let pads = drive.seen().pads;
        let keyboard = drive.tap(KeyCode::Char(']'), M::NONE);
        assert!(matches!(
            keyboard.ui.as_slice(),
            [Command::SetMasterGain { .. }]
        ));
        assert_eq!(
            drive.app.input_sources.active,
            Some(input_sources::Source::Keyboard)
        );
        for (width, height) in [(80, 26), (120, 40)] {
            let row = &shared_feedback_rows(&mut drive, width, height, Instant::now())[0];
            assert!(
                row.contains("▸KB") && !row.contains("M1") && !row.contains("M2"),
                "{row}"
            );
            if width == 120 {
                assert!(!row.contains(PAD_PORT) && !row.contains(XS_PORT), "{row}");
            }
        }
        drive.bytes(PAD_PORT, &[0xb0, 88, 127]);
        drive.bytes(PAD_PORT, &[0xb0, 88, 0]);
        let button = drive.frame();
        assert!(matches!(
            button.ui.as_slice(),
            [Command::SetMasterGain { .. }]
        ));
        assert_eq!(
            drive.app.input_sources.active,
            Some(input_device(&drive, PAD_PORT))
        );
        let row = &shared_feedback_rows(&mut drive, 80, 26, Instant::now())[0];
        assert!(row.contains("▸M1") && !row.contains("▸KB"), "{row}");
        let knob = value_knob(&mut drive, "master.volume", 100);
        assert!(matches!(
            knob.ui.as_slice(),
            [Command::SetMasterGain { .. }]
        ));
        assert_eq!(
            drive.app.input_sources.active,
            Some(input_device(&drive, XS_PORT))
        );
        assert_eq!(
            drive
                .app
                .input_sources
                .connected()
                .map(|(name, _)| name)
                .collect::<Vec<_>>(),
            [PAD_PORT, XS_PORT]
        );
        let row = &shared_feedback_rows(&mut drive, 80, 26, Instant::now())[0];
        assert!(row.contains("▸M2") && !row.contains("▸M1"), "{row}");
        assert_eq!(drive.seen().pads, pads);
    }
}

/// Scenario "Only the last used controller number stays visible".
/// Scenario "Controller activity keeps status columns fixed".
#[test]
fn last_controller_keeps_fixed_columns_through_input_and_expiry() {
    let mut drive = Drive::offline();
    for index in 0..8 {
        drive.midi().recorded_connect(&format!("Aux {index:02}"));
    }
    drive.frame();
    drive.app.project = Some(Project {
        name: "column check".into(),
        folder: drive.app.session_path.parent().unwrap().into(),
    });
    let ports: Vec<_> = drive
        .app
        .input_sources
        .connected()
        .map(|(port, _)| port.to_owned())
        .collect();
    assert_eq!(ports.len(), 10);
    let mut columns = [None, None];
    for index in [0, 7, 9] {
        let at = Instant::now();
        drive
            .midi()
            .recorded_bytes_at(&ports[index], &[0xb0, 15, 64], &drive.project, at);
        drive.frame();
        let label = format!("M{}", index + 1);
        for now in [at, at + LIT_FOR - Duration::from_nanos(1), at + LIT_FOR] {
            for (size, (width, height)) in [(80, 26), (120, 40)].into_iter().enumerate() {
                let row = &shared_feedback_rows(&mut drive, width, height, now)[0];
                assert!(row.contains(&label), "{row}");
                assert_eq!(row.contains('▸'), now < at + LIT_FOR, "{row}");
                assert!(
                    row.contains("KB") && !row.contains("Aux") && !row.contains(" +8"),
                    "{row}"
                );
                let column = |label| {
                    row.find(label)
                        .map(|index| ratatui::text::Line::from(&row[..index]).width())
                };
                let positions = (column("offline"), column("column check"));
                assert!(positions.0.is_some() && positions.1.is_some(), "{row}");
                if let Some(expected) = columns[size] {
                    assert_eq!(positions, expected, "{row}");
                }
                columns[size] = Some(positions);
            }
        }
        let mut cache = view::FrameText::default();
        let expired = cache.feedback_at(&drive.app, at + LIT_FOR).inputs;
        drive.tap(KeyCode::Char('?'), M::NONE);
        let keyboard_at = drive.app.input_sources.latest.as_ref().unwrap().1;
        // A release, equal CC and older input cannot select another controller.
        drive.bytes(&ports[1], &[0x80, 36, 0]);
        drive.bytes(&ports[index], &[0xb0, 15, 64]);
        drive.midi().recorded_bytes_at(
            &ports[2],
            &[0x90, 36, 80],
            &drive.project,
            at - Duration::from_nanos(1),
        );
        drive.frame();
        let retained = cache.feedback_at(&drive.app, keyboard_at).inputs;
        assert!(!Arc::ptr_eq(&expired, &retained));
        assert_eq!(retained.len(), 2);
        assert_eq!(retained[0].compact, "KB");
        assert!(retained[0].active);
        assert_eq!(retained[1].compact.trim(), label);
        assert!(!retained[1].active);
        assert!(Arc::ptr_eq(
            &retained,
            &cache.feedback_at(&drive.app, keyboard_at).inputs
        ));
        let expired_keyboard = cache.feedback_at(&drive.app, keyboard_at + LIT_FOR).inputs;
        assert!(expired_keyboard.iter().all(|input| !input.active));
        assert_eq!(expired_keyboard[1].compact.trim(), label);
        for now in [keyboard_at, keyboard_at + LIT_FOR] {
            for (size, (width, height)) in [(80, 26), (120, 40)].into_iter().enumerate() {
                let row = &shared_feedback_rows(&mut drive, width, height, now)[0];
                let column = |label| {
                    row.find(label)
                        .map(|index| ratatui::text::Line::from(&row[..index]).width())
                };
                assert_eq!(
                    Some((column("offline"), column("column check"))),
                    columns[size],
                    "{row}"
                );
            }
        }
    }
}

#[test]
fn source_labels_follow_identity_when_port_numbers_change() {
    let mut drive = Drive::new();
    drive.bytes(XS_PORT, &[0xb0, 15, 64]);
    drive.frame();
    let at = drive.app.input_sources.latest.as_ref().unwrap().1;
    assert!(shared_feedback_rows(&mut drive, 120, 40, at)[0].contains("▸M2"));
    drive.midi().recorded_disconnect(PAD_PORT);
    drive.frame();
    let row = &shared_feedback_rows(&mut drive, 120, 40, at)[0];
    assert!(
        !row.contains(PAD_PORT) && !row.contains(XS_PORT) && row.contains("▸M1"),
        "{row}"
    );
    drive.midi().recorded_connect(PAD_PORT);
    drive.frame();
    let row = &shared_feedback_rows(&mut drive, 120, 40, at)[0];
    assert!(
        !row.contains(PAD_PORT) && !row.contains(XS_PORT) && row.contains("▸M2"),
        "{row}"
    );
    let last = at + LIT_FOR - Duration::from_nanos(1);
    assert!(shared_feedback_rows(&mut drive, 120, 40, last)[0].contains("▸M2"));
    assert!(!shared_feedback_rows(&mut drive, 120, 40, at + LIT_FOR)[0].contains('▸'));
}

/// Scenario "Learning keeps safety and next steps visible" through keyboard and raw MIDI.
#[test]
fn learning_keeps_safety_and_next_steps_visible() {
    for source in [Source::Keyboard, Source::Button] {
        for (screen, key, number) in REGISTRY_SCREEN_COMMANDS {
            if screen == Screen::Settings {
                continue;
            }
            let mut drive = Drive::registry_screen_checks();
            drive.registry_command(source, key, M::NONE, number);
            drive.app.session_write_failed = true;
            drive
                .app
                .audio
                .as_ref()
                .unwrap()
                .stopped
                .store(true, Relaxed);
            drive.registry_command(source, KeyCode::F(10), M::NONE, 96);
            assert!(drive.app.screen_learn.is_some());
            let picked = drive.tap(KeyCode::Char('q'), M::NONE);
            assert!(picked.commands().is_empty(), "screen learn stays silent");
            drive.bytes(PAD_PORT, &[0x90, 110, 100]);
            drive.bytes(PAD_PORT, &[0x80, 110, 0]);
            let pending = drive.frame();
            assert!(pending.commands().is_empty());
            let before = drive.app.learned.clone();
            for (width, height) in [(80, 26), (120, 40)] {
                let rows = shared_feedback_rows(&mut drive, width, height, Instant::now());
                assert!(
                    rows[0].contains("Save failed") && rows[0].contains("no audio"),
                    "{}",
                    rows[0]
                );
                assert!(!rows[0].contains("LEARN"));
                assert!(
                    rows[height as usize - 3].contains("LEARN")
                        && rows[height as usize - 3].contains("pad Q"),
                    "{}",
                    rows[height as usize - 3]
                );
                assert!(
                    rows[height as usize - 1].contains("Enter bind · Esc keep · F10 done"),
                    "{}",
                    rows[height as usize - 1]
                );
            }
            assert_eq!(
                drive.app.learned, before,
                "drawing does not commit pending input"
            );
            drive.tap(KeyCode::Esc, M::NONE);
            let view = drive.app.learning_view(&drive.keymap).unwrap();
            assert!(!view.controls.contains("Enter bind"));
            drive.tap(KeyCode::Enter, M::NONE);
            assert_eq!(
                drive.app.learned, before,
                "a canceled control stays unbound"
            );
            drive.tap(KeyCode::F(10), M::NONE);
            assert!(drive.app.screen_learn.is_none());
        }
    }
}

#[test]
fn learning_footer_follows_real_controls() {
    let mut drive = Drive::registry_screen_checks();
    drive.tap(KeyCode::F(2), M::NONE);
    let warnings = drive.keymap.overlay(
        "[global]\nF10 = \"none\"\nF9 = \"learn.toggle\"\n\
         [edit]\nI = \"none\"\nUp = \"none\"\nK = \"none\"\nDown = \"none\"\nH = \"edit.param_prev\"\nL = \"edit.param_next\"\n"
    ).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    drive.tap(KeyCode::F(9), M::NONE);
    let view = drive.app.learning_view(&drive.keymap).unwrap();
    assert!(
        view.controls.contains("H L select") && view.controls.contains("Esc/F9 done"),
        "{}",
        view.controls
    );
    assert!(!view.controls.contains("F10") && !view.detail.contains("F10"));
    drive.tap(KeyCode::Char('l'), M::NONE);
    assert_eq!(drive.app.learn_picked().unwrap().0, "pad.volume");
    drive.bytes(PAD_PORT, &[0x90, 110, 100]);
    drive.bytes(PAD_PORT, &[0x80, 110, 0]);
    drive.frame();
    let view = drive.app.learning_view(&drive.keymap).unwrap();
    assert!(
        view.warning && view.detail.contains("knob or fader"),
        "{}",
        view.detail
    );
    assert!(view.controls.contains("Bksp clear"));
    let saved = drive.app.learned.clone();
    drive.tap(KeyCode::Backspace, M::NONE);
    let clear = drive.app.learning_view(&drive.keymap).unwrap();
    assert!(
        clear.controls.starts_with("Enter clear · Esc keep"),
        "{}",
        clear.controls
    );
    assert!(
        !clear.warning && !clear.detail.contains("knob or fader"),
        "{}",
        clear.detail
    );
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(!drive.app.screen_learn.as_ref().unwrap().clearing);
    assert_eq!(drive.app.learned, saved);
    drive.tap(KeyCode::F(10), M::NONE);
    assert!(
        drive.app.screen_learn.is_some(),
        "an overridden key cannot be advertised as exit"
    );
    drive.tap(KeyCode::F(9), M::NONE);
    assert!(drive.app.screen_learn.is_none());
    drive.tap(KeyCode::F(6), M::NONE);
    drive.tap(KeyCode::F(9), M::NONE);
    let view = drive.app.learning_view(&drive.keymap).unwrap();
    assert!(
        !view.controls.contains("select"),
        "Files has no parameter walk: {}",
        view.controls
    );
}

/// Scenario "Input highlights retain original receipt time".
#[test]
fn input_highlights_retain_original_receipt_time() {
    for deferred in [false, true] {
        let mut drive = Drive::registry_screen_checks();
        drive.tap(KeyCode::F(2), M::NONE);
        drive.load_pad(16, Arc::new(Sample::mono(vec![0.2; 4096])), "deferred");
        drive.bind_action(88, "bank.next");
        learn::assign(
            &mut drive.app.learned,
            "X-Session Pro",
            input::Group::Global,
            input::midi::Key::Note(60),
            "pad.1",
        );
        drive
            .app
            .sync_learned("X-Session Pro", input::Group::Global);
        drive.midi().recorded_fence();
        let at = Instant::now();
        if deferred {
            drive
                .midi()
                .recorded_bytes_at(PAD_PORT, &[0xb0, 88, 127], &drive.project, at);
        }
        drive
            .midi()
            .recorded_bytes_at(XS_PORT, &[0x90, 60, 81], &drive.project, at);
        let queued = drive.queued_events();
        let hit = queued
            .iter()
            .find(|event| matches!(event.event, midi::Event::PadHit { .. }))
            .unwrap();
        assert_eq!(hit.source().unwrap().at, at);
        assert_eq!(hit.origin(), Some(XS_PORT));
        assert!(
            matches!(
                hit.event,
                midi::Event::PadHit {
                    sound: midi::PadSound::UiUnattempted { deferred: true },
                    ..
                }
            ) == deferred
        );
        let keyboard = drive.tap(KeyCode::Char(']'), M::NONE);
        let newer = drive.app.input_sources.latest.clone();
        if !deferred {
            assert!(
                matches!(keyboard.direct.as_slice(), [ProjectCommand { command: ProjectMessage::Command(Command::PadPress { pad: 0, velocity }), .. }] if *velocity == 81.0 / 127.0)
            );
        }
        for event in queued {
            drive.app.midi_ui_event(event);
        }
        assert_eq!(drive.app.input_sources.latest, newer);
        assert_eq!(
            drive.app.input_sources.active,
            Some(input_sources::Source::Keyboard)
        );
        let applied = drive.frame();
        if deferred {
            assert!(applied.ui.iter().any(|command| matches!(command, Command::PadPress { pad: 16, velocity } if *velocity == 81.0 / 127.0)));
        } else {
            assert!(
                !applied
                    .ui
                    .iter()
                    .any(|command| matches!(command, Command::PadPress { .. }))
            );
        }
        drive.bytes(XS_PORT, &[0x80, 60, 0]);
        let release = drive.frame();
        let pad = if deferred { 16 } else { 0 };
        assert!(release.commands().iter().any(
            |command| matches!(command, Command::PadRelease { pad: actual } if *actual == pad)
        ));
        assert_eq!(drive.app.input_sources.latest, newer);
        let deadline = newer.unwrap().1 + LIT_FOR;
        drive
            .app
            .input_sources
            .advance(deadline - Duration::from_nanos(1));
        assert_eq!(
            drive.app.input_sources.active,
            Some(input_sources::Source::Keyboard)
        );
        drive.app.input_sources.advance(deadline);
        assert_eq!(drive.app.input_sources.active, None);
    }
}

/// Scenario "Disconnect cannot leave a false active device".
/// Scenario "Last controller number requires current connection input".
#[test]
fn disconnect_cannot_leave_a_false_active_device() {
    let mut drive = Drive::new();
    drive.bytes(PAD_PORT, &[0x90, 36, 74]);
    let queued = drive.queued_events();
    let old = drive.midi().recorded_connection(PAD_PORT).unwrap();
    assert!(
        drive.app.status_feedback_at(Instant::now()).inputs[1]
            .compact
            .trim()
            .is_empty()
    );
    drive.midi().recorded_disconnect(PAD_PORT);
    let disconnected = drive.queued_events();
    for event in disconnected {
        drive.app.midi_ui_event(event);
    }
    for event in queued {
        drive.app.midi_ui_event(event);
    }
    assert!(
        drive
            .app
            .input_sources
            .connected()
            .all(|(name, _)| name != PAD_PORT)
    );
    assert_eq!(drive.app.input_sources.active, None);
    drive.midi().recorded_connect(PAD_PORT);
    drive.frame();
    let new = drive.midi().recorded_connection(PAD_PORT).unwrap();
    assert_ne!(new, old);
    assert_eq!(drive.app.input_sources.active, None);
    assert!(
        drive.app.status_feedback_at(Instant::now()).inputs[1]
            .compact
            .trim()
            .is_empty()
    );
    drive.bytes(PAD_PORT, &[0x90, 37, 63]);
    drive.frame();
    let current = drive.app.input_sources.latest.clone();
    assert_eq!(
        drive.app.input_sources.active,
        Some(input_device(&drive, PAD_PORT))
    );
    // Deliver an old callback after the new connection. Routing is deliberately not gated.
    drive.midi().recorded_bytes_from(
        PAD_PORT,
        &[0x90, 38, 92],
        &drive.project,
        Some(input_sources::Stamp {
            connection: old,
            at: Instant::now(),
        }),
    );
    let late = drive.frame();
    assert!(late.commands().iter().any(|command| matches!(command, Command::PadPress { pad: 2, velocity } if *velocity == 92.0 / 127.0)));
    assert_eq!(drive.app.input_sources.latest, current);
    assert_eq!(
        drive.app.status_feedback_at(Instant::now()).inputs[1]
            .compact
            .trim(),
        "M1"
    );
    drive.midi().recorded_disconnect(PAD_PORT);
    drive.frame();
    assert!(
        drive.app.status_feedback_at(Instant::now()).inputs[1]
            .compact
            .trim()
            .is_empty()
    );
    drive.midi().recorded_connect(PAD_PORT);
    drive.frame();
    assert!(
        drive.app.status_feedback_at(Instant::now()).inputs[1]
            .compact
            .trim()
            .is_empty()
    );
    drive.bytes(PAD_PORT, &[0x90, 39, 80]);
    drive.frame();
    let new = drive.midi().recorded_connection(PAD_PORT).unwrap();
    assert_eq!(
        drive.app.input_sources.active,
        Some(input_device(&drive, PAD_PORT))
    );
    assert_eq!(
        drive
            .app
            .input_sources
            .connected()
            .find(|(name, _)| *name == PAD_PORT)
            .unwrap()
            .1,
        new
    );
}

/// Scenario "First input can precede the connection notice".
#[test]
fn first_input_can_precede_the_connection_notice() {
    for expired in [false, true] {
        let mut drive = Drive::new();
        const PORT: &str = "New recorded device 音";
        drive.midi().recorded_connect(PORT);
        let connected = drive.queued_events();
        let at = Instant::now()
            - if expired {
                Duration::from_secs(2)
            } else {
                Duration::ZERO
            };
        drive
            .midi()
            .recorded_bytes_at(PORT, &[0x90, 36, 80], &drive.project, at);
        let message = drive.queued_events();
        for event in message {
            drive.app.midi_ui_event(event);
        }
        assert!(
            drive
                .app
                .input_sources
                .connected()
                .all(|(name, _)| name != PORT)
        );
        assert_eq!(drive.app.input_sources.active, None);
        for event in connected {
            assert_eq!(
                event.origin(),
                None,
                "connection notices retain system origin"
            );
            drive.app.midi_ui_event(event);
        }
        assert!(
            drive
                .app
                .input_sources
                .connected()
                .any(|(name, _)| name == PORT)
        );
        assert_eq!(
            drive.app.input_sources.active,
            (!expired).then(|| input_device(&drive, PORT))
        );
        assert_eq!(drive.app.input_sources.latest.as_ref().unwrap().1, at);
    }
}

/// Scenario "Missing source metadata stays unknown".
#[test]
fn missing_source_metadata_stays_unknown() {
    let mut drive = Drive::new();
    drive.bind_action(88, "master.volume_up");
    drive
        .midi()
        .recorded_bytes_from(PAD_PORT, &[0xb0, 88, 127], &drive.project, None);
    let applied = drive.frame();
    assert!(matches!(
        applied.ui.as_slice(),
        [Command::SetMasterGain { .. }]
    ));
    assert_eq!(drive.app.input_sources.active, None);
    assert_eq!(drive.app.input_sources.latest, None);
    drive.midi().recorded_bytes_from(
        "Unknown callback port",
        &[0x90, 36, 70],
        &drive.project,
        None,
    );
    let unknown = drive.frame();
    assert!(unknown.commands().iter().any(|command| matches!(command, Command::PadPress { pad: 0, velocity } if *velocity == 70.0 / 127.0)));
    assert_eq!(drive.app.input_sources.latest, None);
    assert!(
        drive
            .app
            .input_sources
            .connected()
            .all(|(name, _)| name != "Unknown callback port")
    );
    let connected: Vec<_> = drive
        .app
        .input_sources
        .connected()
        .map(|(name, id)| (name.to_owned(), id))
        .collect();
    drive.app.midi_ui_event(midi::UiEvent::stamped(
        midi::Event::Action(Action::MasterVolume(1)),
        &drive.project,
    ));
    assert_eq!(drive.app.master_db, 2);
    assert_eq!(drive.app.input_sources.active, None);
    assert_eq!(
        drive
            .app
            .input_sources
            .connected()
            .map(|(name, id)| (name.to_owned(), id))
            .collect::<Vec<_>>(),
        connected
    );
}

#[test]
fn source_activity_keeps_classification_deadlines_and_project_lifetime() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        let pressed = drive.pad_press(source, 0);
        assert!(pressed.commands().iter().any(
            |command| matches!(command, Command::PadPress { pad: 0, velocity } if *velocity == 1.0)
        ));
        let original = drive.app.input_sources.latest.clone();
        let expected = if matches!(source, Source::Keyboard) {
            input_sources::Source::Keyboard
        } else {
            input_device(&drive, PAD_PORT)
        };
        assert_eq!(drive.app.input_sources.active, Some(expected.clone()));
        if matches!(source, Source::Keyboard) {
            drive.key(KeyCode::Char('1'), M::SHIFT, KeyEventKind::Repeat);
            drive.key(KeyCode::Char('1'), M::SHIFT, KeyEventKind::Press);
            drive.key(
                KeyCode::Modifier(event::ModifierKeyCode::LeftShift),
                M::SHIFT,
                KeyEventKind::Press,
            );
            drive.frame();
            assert_eq!(drive.app.input_sources.latest, original);
        }
        let released = drive.pad_release(source, 0);
        assert!(
            released
                .commands()
                .iter()
                .any(|command| matches!(command, Command::PadRelease { pad: 0 }))
        );
        assert_eq!(drive.app.input_sources.latest, original);
        drive.app.apply_session(session::Session::default());
        assert_eq!(drive.app.input_sources.latest, original);
        assert_eq!(drive.app.input_sources.connected().count(), 2);
        let deadline = original.unwrap().1 + LIT_FOR;
        drive
            .app
            .input_sources
            .advance(deadline - Duration::from_nanos(1));
        assert_eq!(drive.app.input_sources.active, Some(expected));
        drive.app.input_sources.advance(deadline);
        assert_eq!(drive.app.input_sources.active, None);
    }
    let restarted = tests::test_app();
    assert_eq!(restarted.input_sources.active, None);
    assert_eq!(restarted.input_sources.connected().count(), 0);
}

#[test]
fn source_activity_uses_real_knob_movement_not_equal_values() {
    let mut drive = Drive::new();
    drive.bytes(XS_PORT, &[0xb0, 15, 64]);
    drive.frame();
    let first = drive.app.input_sources.latest.clone();
    drive.bytes(XS_PORT, &[0xb0, 15, 64]);
    drive.frame();
    assert_eq!(drive.app.input_sources.latest, first);
    drive.bytes(XS_PORT, &[0xb0, 15, 0]);
    drive.frame();
    assert_eq!(
        drive.app.input_sources.active,
        Some(input_device(&drive, XS_PORT))
    );
    assert!(drive.app.input_sources.latest.as_ref().unwrap().1 > first.unwrap().1);
    let at = drive.app.input_sources.latest.as_ref().unwrap().1;
    drive.app.input_sources.advance(at + LIT_FOR);
    assert_eq!(drive.app.input_sources.active, None);
}

#[test]
fn source_knob_comparison_does_not_inherit_an_old_connection() {
    let mut drive = Drive::new();
    let old = drive.midi().recorded_connection(XS_PORT).unwrap();
    drive.midi().recorded_disconnect(XS_PORT);
    drive.frame();
    drive.midi().recorded_connect(XS_PORT);
    drive.frame();
    drive.midi().recorded_bytes_from(
        XS_PORT,
        &[0xb0, 15, 64],
        &drive.project,
        Some(input_sources::Stamp {
            connection: old,
            at: Instant::now(),
        }),
    );
    drive.frame();
    assert_eq!(drive.app.input_sources.active, None);
    drive.bytes(XS_PORT, &[0xb0, 15, 64]);
    drive.frame();
    assert_eq!(
        drive.app.input_sources.active,
        Some(input_device(&drive, XS_PORT))
    );
}

#[test]
fn source_knob_comparison_rejects_older_values_per_channel_and_control() {
    let mut drive = Drive::new();
    let at = Instant::now();
    for (value, age) in [(10, 30), (20, 20), (10, 30), (20, 10)] {
        drive.midi().recorded_bytes_at(
            PAD_PORT,
            &[0xb0, 100, value],
            &drive.project,
            at - Duration::from_nanos(age),
        );
        drive.frame();
    }
    assert_eq!(
        drive.app.input_sources.latest.as_ref().unwrap().1,
        at - Duration::from_nanos(20)
    );
    for bytes in [[0xb1, 100, 20], [0xb0, 101, 20]] {
        drive.bytes(PAD_PORT, &bytes);
        drive.frame();
        assert!(drive.app.input_sources.latest.as_ref().unwrap().1 > at);
    }
}

#[test]
fn source_metadata_survives_the_project_filter_without_acknowledging() {
    use std::sync::atomic::Ordering::Release;
    let mut drive = Drive::offline();
    drive
        .app
        .post_system_notice(Level::Error, ("A shown error".into(), Instant::now()));
    drive.frame();
    drive.bytes(PAD_PORT, &[0x90, 36, 80]);
    let queued = drive.queued_events();
    let at = queued
        .iter()
        .find(|event| event.owner_input())
        .unwrap()
        .source()
        .unwrap()
        .at;
    drive.project.epoch.store(1, Release);
    drive.project.ready.store(1, Release);
    let before = drive.seen();
    for event in queued {
        drive.app.midi_ui_event(event);
    }
    assert_eq!(drive.seen(), before, "old-project bodies remain discarded");
    assert_eq!(drive.app.notice().as_deref(), Some("A shown error"));
    assert_eq!(
        drive.app.input_sources.active,
        Some(input_device(&drive, PAD_PORT))
    );
    assert_eq!(drive.app.input_sources.latest.as_ref().unwrap().1, at);
    drive.app.input_sources.advance(at + LIT_FOR);
    assert_eq!(drive.app.input_sources.active, None);
}

fn value_drive() -> Drive {
    let mut drive = Drive::new();
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 48_000])), "value");
    drive
}

pub(super) fn value_knob(drive: &mut Drive, action: &str, value: u8) -> Frame {
    assert!(input::midi::is_param_action(action), "{action}");
    let group = if drive.app.screen == Screen::Fx && action.starts_with("edit.param_") {
        input::Group::Fx
    } else {
        input::Group::Global
    };
    learn::assign(
        &mut drive.app.learned,
        "X-Session Pro",
        group,
        input::midi::Key::Cc(100),
        action,
    );
    drive.app.sync_learned("X-Session Pro", group);
    drive.bytes(XS_PORT, &[0xb0, 100, value]);
    drive.frame()
}

pub(super) fn shown_value(drive: &Drive) -> tui::Value {
    drive
        .app
        .current_value_at(drive.app.value_popup.as_ref().expect("an applied value").at)
        .expect("a current value")
        .clone()
}

/// Scenario "Values appear on every screen".
#[test]
fn values_appear_on_every_screen() {
    for (screen, key) in [
        (Screen::Pads, 2),
        (Screen::Files, 6),
        (Screen::Chop, 3),
        (Screen::Fx, 4),
        (Screen::Settings, 12),
    ] {
        let mut drive = value_drive();
        drive.tap(KeyCode::F(key), M::NONE);
        assert_eq!(drive.app.screen, screen);
        let keyboard = drive.tap(KeyCode::Char(']'), M::NONE);
        assert_eq!(keyboard.ui.len(), 1);
        assert!(matches!(keyboard.ui[0], Command::SetMasterGain { .. }));
        assert_eq!(shown_value(&drive).object, "Master");
        assert_eq!(shown_value(&drive).formatted, "+1 dB");
        let raw = value_knob(&mut drive, "master.volume", 100);
        assert_eq!(raw.ui.len(), 1);
        assert!(raw.direct.is_empty());
        assert!(matches!(raw.ui[0], Command::SetMasterGain { .. }));
        let value = shown_value(&drive);
        assert_eq!(value.parameter, "volume");
        assert_eq!(value.formatted, format!("{:+} dB", drive.app.master_db));
        assert!(value.position.is_some_and(|p| (0.0..=1.0).contains(&p)));
        let at = drive.app.value_popup.as_ref().unwrap().at;
        let before = drive.seen();
        assert!(
            drive
                .app
                .current_value_at(at + LIT_FOR - Duration::from_nanos(1))
                .is_some()
        );
        assert!(drive.app.current_value_at(at + LIT_FOR).is_none());
        assert_eq!(drive.seen(), before);
        assert!(drive.frame().commands().is_empty());
    }
}

#[test]
fn value_popup_covers_pad_parameters_through_real_controls() {
    use pads::Param as P;
    for (p, action) in [
        (P::Volume, "pad.volume"),
        (P::Pitch, "pad.pitch"),
        (P::Group, "edit.param_3"),
        (P::Attack, "pad.attack"),
        (P::Decay, "pad.decay"),
        (P::Release, "pad.release"),
        (P::Filter, "pad.filter"),
        (P::Cutoff, "pad.cutoff"),
        (P::Reso, "pad.reso"),
        (P::Source, "edit.param_1"),
        (P::Start, "pad.start"),
        (P::End, "pad.end"),
        (P::Drive, "edit.param_1"),
        (P::Send1, "edit.param_2"),
        (P::Send2, "edit.param_3"),
    ] {
        for raw in [false, true] {
            let mut drive = value_drive();
            drive.tap(KeyCode::Enter, M::NONE);
            drive.app.pads.param = p;
            let frame = if raw {
                value_knob(&mut drive, action, if p == P::Source { 127 } else { 64 })
            } else {
                drive.tap(
                    if p == P::End {
                        KeyCode::Left
                    } else {
                        KeyCode::Right
                    },
                    M::NONE,
                )
            };
            assert!(!frame.ui.is_empty(), "{p:?}, raw {raw}");
            assert!(frame.direct.is_empty());
            assert!(
                !frame
                    .ui
                    .iter()
                    .any(|c| matches!(c, Command::PadPress { .. } | Command::Preview { .. }))
            );
            let value = shown_value(&drive);
            assert_eq!(value.object, "pad 1-1");
            assert_eq!(value.parameter, drive.app.pads.name(p), "{p:?}, raw {raw}");
            assert_eq!(value.position.is_none(), matches!(p, P::Filter | P::Source));
            if matches!(p, P::Source | P::Start | P::End) {
                assert!(!value.formatted.is_empty());
            } else {
                assert_eq!(value.formatted, drive.app.pads.param_value());
                assert!(matches!(drive.app.lit_now(), Some(Lit::Pad(actual)) if actual == p));
            }
        }
    }
}

#[test]
fn value_popup_names_shared_fx_slots_not_selected_pads() {
    for slot in 0..2 {
        for row in 0..3 {
            for raw in [false, true] {
                let mut drive = value_drive();
                drive.tap(KeyCode::F(4), M::NONE);
                drive.app.fx_slot = slot;
                drive.app.fx_param = row;
                let type_at_upper_limit = slot == 1 && row == 0;
                let frame = if raw {
                    value_knob(
                        &mut drive,
                        ["edit.param_1", "edit.param_2", "edit.param_3"][row],
                        if type_at_upper_limit { 0 } else { 100 },
                    )
                } else {
                    drive.tap(
                        if type_at_upper_limit {
                            KeyCode::Left
                        } else {
                            KeyCode::Right
                        },
                        M::NONE,
                    )
                };
                assert_eq!(
                    frame.ui.len(),
                    1,
                    "slot {slot}, row {row}, raw {raw}: {:?}",
                    drive.app.input_log
                );
                assert!(
                    matches!(frame.ui[0], Command::SetFx { slot: actual, .. } if actual == slot)
                );
                let value = shown_value(&drive);
                let p = drive.app.fx_selected_param();
                assert_eq!(value.object, format!("FX {}", slot + 1));
                assert_eq!(value.parameter, drive.app.pads.name(p));
                assert_eq!(value.formatted, drive.app.pads.fx_text(0, p));
                assert_eq!(value.position.is_some(), row > 0);
                drive.app.pads.bank = 7;
                drive.app.pads.selected = 15;
                assert_eq!(shown_value(&drive), value);
            }
        }
    }
}

#[test]
fn value_popup_covers_chop_controls_without_changing_their_results() {
    for (name, key, mods, knob) in [
        ("start", KeyCode::Right, M::CONTROL, "slice.start"),
        ("end", KeyCode::Left, M::ALT, "slice.end"),
        ("cursor", KeyCode::Right, M::NONE, "chop.cursor_pos"),
        ("view", KeyCode::Up, M::NONE, "chop.zoom"),
        (
            "region start",
            KeyCode::Char(','),
            M::NONE,
            "chop.region_start_pos",
        ),
        (
            "region end",
            KeyCode::Char('.'),
            M::NONE,
            "chop.region_end_pos",
        ),
        ("threshold", KeyCode::Char('h'), M::NONE, "chop.threshold"),
    ] {
        for raw in [false, true] {
            let mut drive = value_drive();
            drive.tap(KeyCode::F(3), M::NONE);
            drive.app.chop.as_mut().unwrap().cursor = 24_000;
            drive.frame();
            if raw {
                value_knob(&mut drive, knob, 64);
            } else {
                drive.tap(key, mods);
            }
            let value = shown_value(&drive);
            assert_eq!(value.parameter, name, "{name}, raw {raw}");
            assert!(value.position.is_some_and(|p| (0.0..=1.0).contains(&p)));
            assert!(!value.formatted.is_empty());
            if matches!(name, "start" | "end") {
                assert_eq!(value.object, "slice 1");
            }
        }
    }
    let mut drive = value_drive();
    drive.tap(KeyCode::F(3), M::NONE);
    drive.tap(KeyCode::Char('y'), M::NONE);
    assert_eq!(shown_value(&drive).parameter, "equal parts");
    assert_eq!(shown_value(&drive).formatted, "2");
    drive.app.chop.as_mut().unwrap().cursor = 12_000;
    drive.tap(KeyCode::Char(','), M::NONE);
    drive.tap(KeyCode::Char(','), M::CONTROL);
    assert_eq!(shown_value(&drive).parameter, "region");
    assert_eq!(shown_value(&drive).formatted, "0.000..1.000 s");
    drive.tap(KeyCode::Char('g'), M::CONTROL);
    assert_eq!(shown_value(&drive).object, "pad 1-1");
    assert_eq!(shown_value(&drive).parameter, "volume");
}

/// Scenario "Refused and limited values remain truthful".
#[test]
fn refused_and_limited_values_remain_truthful() {
    let mut drive = value_drive();
    drive.app.takeover.response = input::midi::Response::Pickup;
    value_knob(&mut drive, "pad.volume", 0);
    assert!(drive.app.value_popup.is_none());
    assert_eq!(drive.app.pads.volume_db[0], 0);
    assert!(drive.app.pickup.is_some());
    value_knob(&mut drive, "pad.volume", 127);
    assert_eq!(shown_value(&drive).formatted, "+6 dB");
    drive.app.pads.marked.extend([0, 1]);
    drive.app.value_popup = None;
    let frame = drive.tap(KeyCode::Right, M::NONE);
    assert_eq!(frame.ui.len(), 1);
    assert!(matches!(frame.ui[0], Command::SetPadGain { pad: 1, .. }));
    assert_eq!(drive.app.pads.volume_db[0], 6);
    assert_eq!(drive.app.pads.volume_db[1], 1);
    let value = shown_value(&drive);
    assert_eq!(
        (value.object.as_str(), value.formatted.as_str()),
        ("pad 1-2", "+1 dB")
    );
    assert!(drive.last_notice().text.contains("1 pad at the limit"));
    assert_eq!(drive.last_notice().level, Level::Warn);
    drive.app.pads.volume_db[1] = 6;
    drive.app.value_popup = None;
    assert!(drive.tap(KeyCode::Right, M::NONE).ui.is_empty());
    assert!(drive.app.value_popup.is_none());
    assert!(drive.last_notice().text.contains("2 pads at the limit"));

    let mut refused = Drive::offline();
    refused.tap(KeyCode::F(12), M::NONE);
    refused.app.settings_row = 2;
    refused.app.cfg.buffer_size = 128;
    refused.tap(KeyCode::Right, M::NONE);
    assert_eq!(refused.app.cfg.buffer_size, 128);
    assert!(refused.app.value_popup.is_none());
    assert_eq!(
        refused.last_notice().text,
        "offline audio has no buffer size"
    );
    assert_eq!(refused.last_notice().level, Level::Error);
}

/// Scenario "Value deadlines follow actual changes".
#[test]
fn value_deadlines_follow_actual_changes() {
    for raw in [false, true] {
        let mut drive = value_drive();
        if raw {
            value_knob(&mut drive, "pad.volume", 127);
        } else {
            drive.app.pads.volume_db[0] = 5;
            drive.tap(KeyCode::Right, M::NONE);
        }
        let at = Instant::now() - Duration::from_millis(600);
        drive.app.value_popup.as_mut().unwrap().at = at;
        let seen = drive.seen();
        let equal = if raw {
            value_knob(&mut drive, "pad.volume", 127)
        } else {
            drive.tap(KeyCode::Right, M::NONE)
        };
        assert!(equal.ui.is_empty());
        assert_eq!(drive.app.value_popup.as_ref().unwrap().at, at);
        assert!(drive.tap(KeyCode::Right, M::NONE).ui.is_empty());
        assert_eq!(drive.app.value_popup.as_ref().unwrap().at, at);
        assert!(drive.app.current_value_at(at + LIT_FOR).is_none());
        assert_eq!(drive.app.pads.volume_db[0], 6);
        assert_eq!(drive.app.history.len(), seen.history.len());
        if raw {
            value_knob(&mut drive, "pad.volume", 64);
        } else {
            drive.tap(KeyCode::Left, M::NONE);
        }
        let later = drive.app.value_popup.as_ref().unwrap().at;
        assert!(later > at);
        assert!(
            drive
                .app
                .current_value_at(later + LIT_FOR - Duration::from_nanos(1))
                .is_some()
        );
        assert!(drive.app.current_value_at(later + LIT_FOR).is_none());
    }
}

/// Scenario "Value snapshots retain their original object".
#[test]
fn value_snapshots_retain_their_original_object() {
    for raw in [false, true] {
        let mut drive = value_drive();
        let source = if raw {
            Source::Button
        } else {
            Source::Keyboard
        };
        if raw {
            value_knob(&mut drive, "pad.volume", 100);
        } else {
            drive.tap(KeyCode::Right, M::NONE);
        }
        let original = shown_value(&drive);
        drive.bind_action(103, "edit.undo");
        drive.bind_action(104, "bank.next");
        drive.bind_action(105, "screen.files");
        drive.registry_command(source, KeyCode::Char('o'), M::NONE, 104);
        drive.app.pads.param = pads::Param::Pitch;
        drive.registry_command(source, KeyCode::F(6), M::NONE, 105);
        assert_eq!(shown_value(&drive), original);
        drive.registry_command(source, KeyCode::Char('z'), M::CONTROL, 103);
        assert!(drive.app.current_value_at(Instant::now()).is_none());
        drive.tap(KeyCode::F(2), M::NONE);
        drive.app.pads.bank = 0;
        drive.app.pads.param = pads::Param::Volume;
        if raw {
            value_knob(&mut drive, "pad.volume", 64);
        } else {
            drive.tap(KeyCode::Right, M::NONE);
        }
        drive.app.pads.sound[0] = Some(Arc::new(Sample::mono(vec![0.2; 48_000])));
        assert!(drive.app.current_value_at(Instant::now()).is_none());
        if raw {
            value_knob(&mut drive, "pad.volume", 32);
        } else {
            drive.tap(KeyCode::Right, M::NONE);
        }
        assert!(drive.app.current_value_at(Instant::now()).is_some());
        drive.app.apply_session(session::Session::default());
        assert!(drive.app.value_popup.is_none());
    }
}

/// Scenario "Editor prompts take priority over values".
#[test]
fn editor_prompts_take_priority_over_values() {
    for raw in [false, true] {
        for overlay in 0..9 {
            let mut drive = value_drive();
            if raw {
                value_knob(&mut drive, "master.volume", 100);
            } else {
                drive.tap(KeyCode::Char(']'), M::NONE);
            }
            let at = drive.app.value_popup.as_ref().unwrap().at;
            match overlay {
                0 => {
                    drive.tap(KeyCode::F(1), M::NONE);
                }
                1 => {
                    drive.tap(KeyCode::F(6), M::NONE);
                    drive.tap(KeyCode::Char('/'), M::NONE);
                    assert!(drive.app.browser.editing);
                }
                2 => {
                    drive.tap(KeyCode::Char('n'), M::CONTROL);
                    assert!(drive.app.pad_naming.is_some());
                }
                3 => {
                    drive.tap(KeyCode::Char('p'), M::CONTROL);
                    assert!(drive.app.project_picker.is_some());
                }
                4 => {
                    drive.tap(KeyCode::F(10), M::NONE);
                    assert!(drive.app.screen_learn.is_some());
                }
                5 => {
                    drive.tap(KeyCode::Enter, M::NONE);
                    drive.tap(KeyCode::Backspace, M::NONE);
                    assert!(drive.app.ask.is_some());
                }
                6 => {
                    drive.app.path_picker =
                        Some(PathPicker::open(0, Path::new("target/87-value-picker")));
                }
                7 => {
                    drive.app.learning = Some(learn::Learning::new(action_info("pad.copy")));
                }
                _ => {
                    drive.tap(KeyCode::F(12), M::NONE);
                    drive.app.settings_tab = 2;
                    drive.app.midi_row = drive
                        .app
                        .midi_list()
                        .iter()
                        .position(|row| {
                            matches!(
                                row,
                                MidiListRow::Group {
                                    group: input::Group::Global,
                                    ..
                                }
                            )
                        })
                        .unwrap();
                    drive.tap(KeyCode::Right, M::NONE);
                    drive.app.midi_row = drive
                        .app
                        .midi_list()
                        .iter()
                        .position(|row| matches!(row, MidiListRow::Set { .. }))
                        .unwrap();
                    drive.tap(KeyCode::Enter, M::NONE);
                    assert!(drive.app.midi_run.is_some());
                }
            }
            assert!(
                drive
                    .app
                    .current_value_at(at + Duration::from_millis(1))
                    .is_none(),
                "overlay {overlay}"
            );
            let before = drive.seen();
            assert!(drive.app.current_value_at(at + LIT_FOR).is_none());
            assert_eq!(drive.seen(), before);
            drive.app.help_open = false;
            drive.app.browser.editing = false;
            drive.app.pad_naming = None;
            drive.app.project_picker = None;
            drive.app.path_picker = None;
            drive.app.screen_learn = None;
            drive.app.learning = None;
            drive.app.midi_run = None;
            drive.app.ask = None;
            assert!(drive.app.current_value_at(at + LIT_FOR).is_none());
            assert_eq!(drive.app.value_popup.as_ref().unwrap().at, at);
        }
    }
}

#[test]
fn value_popup_labels_settings_choices_without_claiming_server_changes() {
    for raw in [false, true] {
        let mut drive = value_drive();
        drive.bind_action(101, "settings.value_next");
        drive.tap(KeyCode::F(12), M::NONE);
        drive.app.settings_row = 3;
        let before = drive.app.ui.graphics;
        if raw {
            drive.bytes(PAD_PORT, &[0xb0, 101, 127]);
            drive.bytes(PAD_PORT, &[0xb0, 101, 0]);
            drive.frame();
        } else {
            drive.tap(KeyCode::Right, M::NONE);
        }
        assert_ne!(drive.app.ui.graphics, before);
        let value = shown_value(&drive);
        assert_eq!(value.object, "Settings");
        assert_eq!(value.parameter, "wave");
        assert!(value.formatted.contains("next start"));
        assert!(value.position.is_none());
        drive.app.settings_tab = 2;
        drive.app.midi_row = drive
            .app
            .midi_list()
            .iter()
            .position(|row| matches!(row, MidiListRow::Response))
            .unwrap();
        if raw {
            drive.bytes(PAD_PORT, &[0xb0, 101, 127]);
            drive.bytes(PAD_PORT, &[0xb0, 101, 0]);
            drive.frame();
        } else {
            drive.tap(KeyCode::Right, M::NONE);
        }
        assert_eq!(shown_value(&drive).parameter, "knob response");
        assert_eq!(
            shown_value(&drive).formatted,
            drive.app.takeover.response.name()
        );
    }
}

#[test]
fn value_popup_expiry_releases_sample_snapshots_on_the_ui_frame() {
    let mut drive = value_drive();
    drive.tap(KeyCode::Right, M::NONE);
    let sample = drive.app.pads.sound[0].as_ref().unwrap().clone();
    let at = drive.app.value_popup.as_ref().unwrap().at;
    let owners = Arc::strong_count(&sample);
    drive
        .app
        .frame_notice_at(at + LIT_FOR - Duration::from_nanos(1));
    assert!(drive.app.value_popup.is_some());
    assert_eq!(Arc::strong_count(&sample), owners);
    drive.app.frame_notice_at(at + LIT_FOR);
    assert!(drive.app.value_popup.is_none());
    assert_eq!(Arc::strong_count(&sample), owners - 1);
}

#[test]
fn value_popup_expiry_restores_text_without_another_input() {
    use ratatui::{Terminal, backend::TestBackend};
    for (width, height) in [(80, 26), (120, 40)] {
        let mut drive = value_drive();
        drive.tap(KeyCode::Char(']'), M::NONE);
        let app = &mut drive.app;
        let at = app.value_popup.as_ref().unwrap().at;
        let p = pads_view(&mut app.pads, None, None, Vec::new(), None, app.rate);
        let screen = tui::Screen::Pads(app.mark_pads(p));
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
        let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
        term.draw(|f| tui::draw(f, &status, &screen, &help))
            .unwrap();
        let baseline = term.backend().buffer().clone();
        for elapsed in [
            Duration::ZERO,
            LIT_FOR - Duration::from_nanos(1),
            LIT_FOR,
            LIT_FOR + Duration::from_secs(3),
        ] {
            term.draw(|f| {
                tui::draw(f, &status, &screen, &help);
                if let Some(value) = app.current_value_at(at + elapsed) {
                    tui::value_popup(f, value);
                }
            })
            .unwrap();
            assert_eq!(term.backend().buffer() == &baseline, elapsed >= LIT_FOR);
        }
        assert!(drive.frame().commands().is_empty());
    }
}

#[test]
fn value_popup_keeps_categorical_values_and_scoped_button_limits() {
    for raw in [false, true] {
        let mut drive = value_drive();
        let source = if raw {
            Source::Button
        } else {
            Source::Keyboard
        };
        for (screen, key, action, name) in [
            (Screen::Pads, KeyCode::Char('h'), "pad.mode_next", "mode"),
            (Screen::Chop, KeyCode::Char('n'), "chop.snap_toggle", "snap"),
            (Screen::Chop, KeyCode::Char('o'), "chop.play_mode", "play"),
        ] {
            if screen == Screen::Chop && drive.app.screen != screen {
                drive.tap(KeyCode::F(3), M::NONE);
            }
            drive.bind_action(102, action);
            drive.registry_command(source, key, M::NONE, 102);
            assert_eq!(shown_value(&drive).parameter, name);
            assert!(shown_value(&drive).position.is_none());
        }
        drive.tap(KeyCode::F(2), M::NONE);
        drive.app.pads.marked.extend([0, 1]);
        drive.app.pads.volume_db[0] = 6;
        drive.app.pads.volume_db[1] = 0;
        drive.bind_action(102, "edit.value_inc");
        let frame = drive.registry_command(source, KeyCode::Right, M::NONE, 102);
        assert_eq!(frame.ui.len(), 1);
        assert!(matches!(frame.ui[0], Command::SetPadGain { pad: 1, .. }));
        assert_eq!(shown_value(&drive).object, "pad 1-2");
        assert_eq!(shown_value(&drive).formatted, "+1 dB");
        assert!(drive.last_notice().text.contains("1 pad at the limit"));
        drive.app.pads.selected = 7;
        drive.app.pads.volume_db[7] = 2;
        drive.registry_command(source, KeyCode::Right, M::NONE, 102);
        assert_eq!(shown_value(&drive).object, "pad 1-2");
        assert_eq!(shown_value(&drive).formatted, "+2 dB");
        assert_eq!(drive.app.pads.volume_db[7], 2);
    }
}

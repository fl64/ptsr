//! Recorded input through the production keyboard and MIDI routes. No device or sound card.

mod transport;

use super::*;
use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers as M};

const PAD_PORT: &str = "Recorded pads MIDI 1";
const XS_PORT: &str = "USB X-Session MIDI 1";

fn shared_feedback_rows(drive: &mut Drive, width: u16, height: u16, now: Instant) -> Vec<String> {
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
        meters: ([-60.0; 2], [-60.0; 2], false),
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
        meters: ([-60.0; 2], [-60.0; 2], false),
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

fn input_device(drive: &Drive, port: &str) -> input_sources::Source {
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

fn value_knob(drive: &mut Drive, action: &str, value: u8) -> Frame {
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

fn shown_value(drive: &Drive) -> tui::Value {
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
            meters: ([-60.0; 2], [-60.0; 2], false),
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

struct Frame {
    direct: Vec<ProjectCommand>,
    ui: Vec<Command>,
}

impl Frame {
    fn commands(&self) -> Vec<&Command> {
        self.direct
            .iter()
            .map(|message| {
                assert!(message.ready_at_send);
                let ProjectMessage::Command(command) = &message.command else {
                    panic!("a recorded pad frame must not contain a project marker");
                };
                command
            })
            .chain(self.ui.iter())
            .collect()
    }
}

struct Drive {
    app: App,
    keymap: Keymap,
    down: input::HeldKeys,
    held: HeldPads,
    mods: input::HeldModifiers,
    shared: Arc<midi::Shared>,
    project: Arc<ProjectState>,
    direct: Option<rtrb::Consumer<ProjectCommand>>,
    worker: Option<std::thread::JoinHandle<()>>,
    audio_rt: Option<engine::rt::Realtime>,
}

fn pad_profile() -> Profile {
    let mut text = String::from(
        "[device]\nname = \"Recorded pads\"\nmatch_port = \"Recorded pads\"\n\
         [controls.pads]\n",
    );
    for pad in 1..=16 {
        text.push_str(&format!("p{pad} = {{ note = {} }}\n", 35 + pad));
    }
    text.push_str(
        "[controls.buttons]\ncopy = { cc = 25 }\nback = { cc = 26 }\n\
         bank = { cc = 27 }\nmark = { cc = 28 }\nedit = { cc = 29 }\n\
         [screens.global]\ncopy = \"pad.copy\"\nback = \"ui.back\"\nbank = \"bank.next\"\n\
         [screens.pads]\nmark = \"pad.mark_toggle\"\nedit = \"pad.edit\"\n",
    );
    Profile::from_toml("recorded-pad-profile.toml", &text).unwrap()
}

impl Drive {
    fn new() -> Self {
        Self::with_audio(false)
    }

    fn offline() -> Self {
        Self::with_audio(true)
    }

    fn with_audio(offline: bool) -> Self {
        Self::with_saved_bindings(offline, None)
    }

    fn with_saved_bindings(offline: bool, path: Option<&Path>) -> Self {
        let mut app = tests::test_app();
        app.pads.track_keys = true;
        app.profiles = vec![
            pad_profile(),
            Profile::from_toml(
                "xsession-pro.toml",
                include_str!("../../../config/devices/xsession-pro.toml"),
            )
            .unwrap(),
        ];
        if let Some(path) = path {
            let loaded = config::load_from(path);
            assert_eq!(loaded.user_path.as_deref(), Some(path));
            app.user_path = loaded.user_path;
            app.learned = loaded.config.midi;
            assert!(learn::drop_dead(&mut app.learned).is_empty());
            app.seeded = loaded.config.devices.seeded;
            app.migrate_fx_bindings();
            let learned = midi::parse_learned(&app.learned);
            for profile in &mut app.profiles {
                midi::apply_learned(profile, &learned);
                profile.set_seeded(app.seeded.contains(&profile.name));
            }
        }
        let shared = Arc::new(midi::Shared::default());
        app.publish_midi(&shared);
        // In render tests the MIDI writer owns the callback's real queue. Do not
        // capture and replay direct commands: that would hide UI-dependent timing.
        let (project, producer, direct, audio_rt) = if offline {
            let (mut audio, rt) = audio::offline(48_000);
            let project = audio.project.clone();
            let producer = audio.midi_commands.take().unwrap();
            app.audio = Some(audio);
            (project, producer, None, Some(rt))
        } else {
            let (producer, consumer) = rtrb::RingBuffer::new(1024);
            (
                Arc::new(ProjectState::default()),
                producer,
                Some(consumer),
                None,
            )
        };
        let (midi, worker) = midi::Midi::recorded(
            app.profiles.clone(),
            producer,
            project.clone(),
            shared.clone(),
        );
        app.midi = Some(midi);
        let (keymap, warnings) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        let mut drive = Self {
            app,
            keymap,
            down: Default::default(),
            held: Default::default(),
            mods: Default::default(),
            shared,
            project,
            direct,
            worker: Some(worker),
            audio_rt,
        };
        drive.midi().recorded_connect(PAD_PORT);
        drive.midi().recorded_connect(XS_PORT);
        drive.frame(); // Connected -> App::seed_device -> real learned-map updates.
        drive.midi().recorded_fence();
        assert!(drive.app.seeded.iter().any(|name| name == "Recorded pads"));
        assert!(drive.app.seeded.iter().any(|name| name == "X-Session Pro"));
        // A fixture with no user path reports that it cannot save the seeded map.
        // Neither input pass writes the owner's config or treats this as a user notice.
        drive.app.notices = Notices::new(Instant::now());
        drive
    }

    fn last_notice(&self) -> &Notice {
        self.app
            .notices
            .history()
            .front()
            .expect("a produced notice")
    }

    fn midi(&self) -> &midi::Midi {
        self.app.midi.as_ref().unwrap()
    }

    fn key(&mut self, code: KeyCode, modifiers: M, kind: KeyEventKind) {
        assert!(!self.app.keyboard_event(
            &self.keymap,
            &KeyEvent::new_with_kind(code, modifiers, kind),
            true,
            &mut self.down,
            &mut self.held,
            &mut self.mods,
        ));
    }

    fn tap(&mut self, code: KeyCode, modifiers: M) -> Frame {
        self.key(code, modifiers, KeyEventKind::Press);
        self.key(code, modifiers, KeyEventKind::Release);
        self.frame()
    }

    fn bytes(&self, port: &str, bytes: &[u8]) {
        self.midi().recorded_bytes(port, bytes, &self.project);
    }

    fn frame(&mut self) -> Frame {
        self.midi().recorded_fence();
        let direct = std::iter::from_fn(|| self.direct.as_mut()?.pop().ok()).collect();
        let events: Vec<_> = self.midi().events.try_iter().collect();
        for event in events {
            self.app.midi_ui_event(event);
        }
        self.app.publish_midi(&self.shared);
        self.app.input_sources.advance(Instant::now());
        self.app.frame_notice();
        Frame {
            direct,
            ui: std::mem::take(&mut self.app.sent),
        }
    }
}

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

fn f1_message_screen(drive: &mut Drive, width: u16, height: u16) -> Vec<String> {
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
    assert!(!drive.app.help_overlay.messages);
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
    drive.tap(KeyCode::BackTab, M::SHIFT);
    assert!(drive.app.help_overlay.messages);
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
    for code in [KeyCode::Tab, KeyCode::Tab] {
        drive.key(code, M::NONE, KeyEventKind::Press);
        drive.frame();
        let page = drive.app.help_overlay.messages;
        drive.key(code, M::NONE, KeyEventKind::Repeat);
        drive.frame();
        assert_eq!(drive.app.help_overlay.messages, page);
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
    assert!(!drive.app.help_overlay.messages);
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
    assert!(!drive.app.help_overlay.messages);
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
    assert!(!drive.app.help_overlay.messages);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.project_picker.is_none() && drive.app.help_open);
    drive.tap(KeyCode::F(6), M::NONE);
    drive.tap(KeyCode::Char('/'), M::NONE);
    assert!(drive.app.browser.editing);
    drive.tap(KeyCode::Tab, M::NONE);
    assert!(!drive.app.help_overlay.messages);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(!drive.app.browser.editing && drive.app.help_open);
    drive.tap(KeyCode::F(2), M::NONE);
    drive.tap(KeyCode::F(10), M::NONE);
    assert!(drive.app.screen_learn.is_some());
    drive.tap(KeyCode::Tab, M::NONE);
    assert!(!drive.app.help_overlay.messages);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.screen_learn.is_none() && drive.app.help_open);
    drive.tap(KeyCode::Char('a'), M::CONTROL);
    drive.tap(KeyCode::Backspace, M::NONE);
    assert!(drive.app.ask.is_some());
    drive.tap(KeyCode::Tab, M::NONE);
    assert!(!drive.app.help_overlay.messages);
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
    assert!(!drive.app.help_overlay.messages);
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
    assert!(!drive.app.help_overlay.messages);
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

#[derive(Clone, Copy, Debug)]
enum Source {
    Keyboard,
    Notes,
    Button,
}

#[derive(Debug, PartialEq)]
struct Seen {
    screen: Screen,
    edit: bool,
    param: pads::Param,
    held: Vec<bool>,
    bank: usize,
    selected: usize,
    slice: Option<usize>,
    chop_cursor: Option<usize>,
    plan: Option<chop::Plan>,
    file_hand: Option<PathBuf>,
    swap_hand: Option<usize>,
    copy_hand: Option<usize>,
    marks: Vec<usize>,
    marking: bool,
    question: Vec<String>,
    notice: Option<String>,
    history: Vec<(String, Vec<usize>)>,
    pads: Vec<pads::PadSnap>,
}

impl Drive {
    fn seen(&self) -> Seen {
        Seen {
            screen: self.app.screen,
            edit: self.app.edit,
            param: self.app.pads.param,
            held: self.app.pads.held.to_vec(),
            bank: self.app.pads.bank,
            selected: self.app.pads.selected,
            slice: self.app.chop.as_ref().map(|chop| chop.selected),
            chop_cursor: self.app.chop.as_ref().map(|chop| chop.cursor),
            plan: self.app.plan.clone(),
            file_hand: self.app.browser.in_hand.clone(),
            swap_hand: self.app.pad_in_hand,
            copy_hand: self.app.copy_in_hand.as_ref().map(|(source, _)| *source),
            marks: self.app.pads.marked.iter().copied().collect(),
            marking: self.app.pads.marking,
            question: self
                .app
                .ask
                .as_ref()
                .map_or_else(Vec::new, |ask| ask.lines().to_vec()),
            notice: self
                .app
                .notices
                .history()
                .front()
                .map(|notice| notice.text.clone()),
            history: self
                .app
                .history
                .iter()
                .map(|step| {
                    (
                        step.what.clone(),
                        step.pads.iter().map(|(pad, _)| *pad).collect(),
                    )
                })
                .collect(),
            pads: (0..PADS).map(|pad| self.app.pads.snapshot(pad)).collect(),
        }
    }

    fn queued_events(&self) -> Vec<midi::UiEvent> {
        self.midi().recorded_fence();
        self.midi().events.try_iter().collect()
    }

    fn raw_ordered_hit(&self, source: Source, velocity: u8) {
        self.bytes(
            PAD_PORT,
            &match source {
                Source::Notes => [0x90, 38, velocity],
                Source::Button => [0xb0, 80, velocity],
                Source::Keyboard => panic!("raw MIDI scenario"),
            },
        );
    }

    fn raw_ordered_release(&self, source: Source, release: RecordedRelease) {
        if matches!(release, RecordedRelease::Disconnect) {
            self.midi().recorded_disconnect(PAD_PORT);
        } else {
            self.bytes(
                PAD_PORT,
                &match source {
                    Source::Notes if matches!(release, RecordedRelease::Off) => [0x80, 38, 0],
                    Source::Notes => [0x90, 38, 0],
                    Source::Button => [0xb0, 80, 0],
                    Source::Keyboard => panic!("raw MIDI scenario"),
                },
            );
        }
    }

    fn bind_action(&mut self, cc: u8, action: &str) {
        learn::assign(
            &mut self.app.learned,
            "Recorded pads",
            input::Group::Global,
            input::midi::Key::Cc(cc),
            action,
        );
        self.app.sync_learned("Recorded pads", input::Group::Global);
        self.midi().recorded_fence();
    }

    fn ordered_file(&mut self) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/85-ordered-files")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Relaxed)
            ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("new.wav");
        let tone = vec![0.1; 4096];
        engine::offline::write_wav(&path, RENDER_RATE, &tone, &tone).unwrap();
        self.tap(KeyCode::F(6), M::NONE);
        self.app.browser.dir = dir;
        self.app.browser.entries = vec![browser::Entry {
            name: "new.wav".into(),
            is_dir: false,
            favorite: None,
        }];
        self.app.browser.selected = 0;
        self.bind_action(82, "browser.hold");
        self.frame();
        path
    }

    fn bind_pad_button(&mut self, pad: usize) {
        learn::assign(
            &mut self.app.learned,
            "Recorded pads",
            input::Group::Global,
            input::midi::Key::Cc(80),
            &format!("pad.{}", pad + 1),
        );
        self.app.sync_learned("Recorded pads", input::Group::Global);
        self.midi().recorded_fence();
    }

    fn pad_press(&mut self, source: Source, pad: usize) -> Frame {
        match source {
            Source::Keyboard => self.key(
                KeyCode::Char("1234qwerasdfzxcv".chars().nth(pad).unwrap()),
                M::SHIFT,
                KeyEventKind::Press,
            ),
            Source::Notes => self.bytes(PAD_PORT, &[0x90, 36 + pad as u8, 127]),
            Source::Button => {
                self.bind_pad_button(pad);
                self.bytes(PAD_PORT, &[0xb0, 80, 127]);
            }
        }
        self.frame()
    }

    fn pad_release(&mut self, source: Source, pad: usize) -> Frame {
        match source {
            Source::Keyboard => self.key(
                KeyCode::Char("1234qwerasdfzxcv".chars().nth(pad).unwrap()),
                M::SHIFT,
                KeyEventKind::Release,
            ),
            Source::Notes => self.bytes(PAD_PORT, &[0x80, 36 + pad as u8, 0]),
            Source::Button => self.bytes(PAD_PORT, &[0xb0, 80, 0]),
        }
        self.frame()
    }

    fn prepare_chop(&mut self, sound: Arc<Sample>, slices: usize) {
        if self.app.audio.is_none() {
            let (audio, rt) = audio::offline(48_000);
            self.project = audio.project.clone();
            self.app.audio = Some(audio);
            self.audio_rt = Some(rt);
        }
        let step = sound.len() / slices;
        self.app.chop = Some(chop::Chop::from_saved(
            "break.wav".into(),
            sound.clone(),
            48_000,
            chop::Saved {
                range: Region {
                    start: 0,
                    end: sound.len(),
                },
                cuts: (1..slices)
                    .map(|slice| (slice as u32, slice * step))
                    .collect(),
                epoch: 1,
                next_id: slices as u32,
            },
        ));
        self.app.screen = Screen::Chop;
        self.frame();
    }

    fn link_chop_pad(&mut self, abs: usize, slice: usize) {
        let chop = self.app.chop.as_ref().unwrap();
        let (sound, region, link) = (
            chop.sound.clone(),
            chop.slices()[slice],
            (chop.start_id(slice), chop.epoch),
        );
        self.load_pad(abs, sound, "break");
        self.app.pads.region[abs] = region;
        self.app.pads.slice[abs] = Some(slice as u16 + 1);
        self.app.pads.link[abs] = Some(link);
        self.app.pads.filter[abs] = engine::filter::FilterKind::LowPass;
        self.app.pads.cutoff_hz[abs] = 1_200.0;
    }

    fn load_pad(&mut self, pad: usize, sound: Arc<Sample>, name: &str) {
        self.app.pads.sound[pad] = Some(sound);
        self.app.pads.file[pad] = Some(format!("{name}.wav").into());
        self.app.pads.custom_name[pad] = Some(name.into());
        self.app.pads.gate[pad] = true;
        self.app.pads.pitch[pad] = 3;
    }
}

fn assert_silent_target(frame: &Frame) {
    assert!(
        frame.direct.is_empty(),
        "a target must not enter the direct audio queue: {:?}",
        frame.direct
    );
    assert!(
        !frame.ui.iter().any(|command| matches!(
            command,
            Command::PadPress { .. } | Command::Preview { sound: Some(_), .. }
        )),
        "a target must not sound from the UI: {:?}",
        frame.ui
    );
}

impl Drop for Drive {
    fn drop(&mut self) {
        // Abandon the audio queue before joining, even if a failing test left it full.
        self.direct.take();
        self.audio_rt.take();
        self.app.midi.take();
        self.worker.take().unwrap().join().unwrap();
    }
}

const REGISTRY_SCREEN_COMMANDS: [(Screen, KeyCode, u8); 5] = [
    (Screen::Pads, KeyCode::F(2), 90),
    (Screen::Settings, KeyCode::F(12), 91),
    (Screen::Files, KeyCode::F(6), 92),
    (Screen::Chop, KeyCode::F(3), 93),
    (Screen::Fx, KeyCode::F(4), 94),
];

impl Drive {
    fn registry_screen_checks() -> Self {
        let mut drive = Self::new();
        for (number, action) in [
            (90, "screen.pads"),
            (91, "screen.settings"),
            (92, "screen.files"),
            (93, "screen.chop"),
            (94, "screen.fx"),
            (95, "help.toggle"),
            (96, "learn.toggle"),
            (97, "pad.edit"),
            (98, "chop.edit"),
            (99, "settings.tab_next"),
            (100, "browser.preview_toggle"),
            (101, "chop.loop_toggle"),
        ] {
            drive.bind_action(number, action);
        }
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 48_000])), 4);
        drive.link_chop_pad(0, 0);
        drive
    }

    fn save_registry_bindings(&self) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/86-registry-config")
            .join(format!(
                "{}-{}.toml",
                std::process::id(),
                NEXT.fetch_add(1, Relaxed)
            ));
        config::save_section(&path, "midi", &self.app.learned).unwrap();
        config::save_section(
            &path,
            "devices",
            &config::DevicesConfig {
                seeded: self.app.seeded.clone(),
            },
        )
        .unwrap();
        path
    }

    fn registry_command(&mut self, source: Source, code: KeyCode, mods: M, number: u8) -> Frame {
        match source {
            Source::Keyboard => return self.tap(code, mods),
            Source::Notes => {
                self.bytes(PAD_PORT, &[0x90, number, 127]);
                self.bytes(PAD_PORT, &[0x80, number, 0]);
            }
            Source::Button => {
                self.bytes(PAD_PORT, &[0xb0, number, 127]);
                self.bytes(PAD_PORT, &[0xb0, number, 0]);
            }
        }
        self.frame()
    }
}

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

struct MidiStateLog(std::sync::Mutex<Vec<(std::thread::ThreadId, String)>>);

impl log::Log for MidiStateLog {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() == log::Level::Error && metadata.target().ends_with("::midi")
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let text = record.args().to_string();
            if text.starts_with("MIDI rejected new input:") {
                self.0
                    .lock()
                    .unwrap()
                    .push((std::thread::current().id(), text));
            }
        }
    }

    fn flush(&self) {}
}

fn midi_state_log() -> &'static MidiStateLog {
    static LOG: MidiStateLog = MidiStateLog(std::sync::Mutex::new(Vec::new()));
    static START: std::sync::Once = std::sync::Once::new();
    START.call_once(|| {
        log::set_logger(&LOG).unwrap();
        log::set_max_level(log::LevelFilter::Error);
    });
    &LOG
}

impl Drive {
    fn invalid_state_checks() -> Self {
        midi_state_log();
        let mut drive = Self::new();
        drive.bind_action(84, "bank.next");
        drive.bind_pad_button(4);
        drive.app.screen = Screen::Fx;
        drive.app.pads.bank = 2;
        drive.app.pads.selected = 7;
        drive.frame();
        assert_full_hit(&drive.pad_press(Source::Notes, 2), 34);
        assert_only_release(&drive.pad_release(Source::Notes, 2), 34);
        assert_full_hit(&drive.pad_press(Source::Button, 4), 36);
        assert_only_release(&drive.pad_release(Source::Button, 4), 36);
        drive.bytes(XS_PORT, &[0xb0, 24, 32]);
        let knob = drive.queued_events();
        assert!(
            knob.iter()
                .any(|event| matches!(event.event, midi::Event::Param { .. }))
        );
        for event in knob {
            drive.app.midi_ui_event(event);
        }
        drive.frame();
        drive
    }

    fn invalid_errors(&self) -> Vec<String> {
        let id = self.worker.as_ref().unwrap().thread().id();
        midi_state_log()
            .0
            .lock()
            .unwrap()
            .iter()
            .filter(|(thread, _)| *thread == id)
            .map(|(_, text)| text.clone())
            .collect()
    }

    fn rejected_input(&self) {
        for _ in 0..2 {
            for bytes in [
                [0x90, 38, 101],
                [0xb0, 80, 127],
                [0xb0, 84, 127],
                [0x80, 39, 0],
                [0x90, 38, 0],
                [0xb0, 80, 0],
            ] {
                self.bytes(PAD_PORT, &bytes);
            }
            self.bytes(XS_PORT, &[0xb0, 24, 127]);
        }
    }

    fn invalid_frame(&mut self) -> Frame {
        self.midi().recorded_fence();
        let direct = std::iter::from_fn(|| self.direct.as_mut()?.pop().ok()).collect();
        for event in self.midi().events.try_iter().collect::<Vec<_>>() {
            assert_eq!(
                event.sequence(),
                None,
                "invalid input cannot number UI work"
            );
            assert!(!matches!(
                event.event,
                midi::Event::PadHit { .. }
                    | midi::Event::Action(_)
                    | midi::Event::Param { .. }
                    | midi::Event::PadRelease { direct: false, .. }
            ));
            self.app.midi_ui_event(event);
        }
        // Unlike a normal UI frame, this observation must not publish valid state.
        Frame {
            direct,
            ui: std::mem::take(&mut self.app.sent),
        }
    }

    fn assert_invalid_input_rejected(&mut self, bits: u32) {
        let before = self.seen();
        self.shared.inject_focus(bits);
        assert_eq!(self.shared.focus(), Err(bits));
        self.rejected_input();
        assert!(self.invalid_frame().commands().is_empty());
        assert_eq!(self.seen(), before);
        assert!(self.app.midi_ui_held.is_empty());
        assert!(self.app.audio_held.iter().all(Option::is_none));
        assert!(self.app.pending_audio.is_empty());
        assert_eq!(
            self.invalid_errors(),
            [format!(
                "MIDI rejected new input: invalid state {bits:#010x}"
            )]
        );
        // Rejected hits must not create holds which later disconnect could release.
        self.midi().recorded_disconnect(PAD_PORT);
        assert!(self.invalid_frame().commands().is_empty());
        assert_eq!(self.shared.focus(), Err(bits));
        assert_eq!(self.invalid_errors().len(), 1);
    }
}

#[test]
fn unknown_midi_screen_rejects_new_input() {
    for code in 5..=7 {
        let mut drive = Drive::invalid_state_checks();
        let bits = (drive.app.focus().pack().unwrap() & !7) | code;
        drive.assert_invalid_input_rejected(bits);
    }
}

#[test]
fn other_invalid_midi_state_has_no_fallback() {
    for bank in 8..=15 {
        let mut drive = Drive::invalid_state_checks();
        let bits = (drive.app.focus().pack().unwrap() & !(15 << 3)) | (bank << 3);
        drive.assert_invalid_input_rejected(bits);
    }
    for bit in 17..32 {
        let mut drive = Drive::invalid_state_checks();
        let bits = drive.app.focus().pack().unwrap() | (1 << bit);
        drive.assert_invalid_input_rejected(bits);
    }
}

#[test]
fn valid_midi_state_resumes_without_rejected_input() {
    use engine::checks::peak;
    midi_state_log();
    let mut drive = Drive::offline();
    drive.parity_pad(0, false);
    drive.tap(KeyCode::F(2), M::NONE);
    drive.tap(KeyCode::Char('1'), M::CONTROL);
    drive.bind_action(84, "bank.next");
    drive.bind_pad_button(4);
    drive.render_blocks(32);
    drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
    drive.bytes(PAD_PORT, &[0xb0, 84, 0]);
    let earlier = drive.queued_events();
    let sequence = earlier.iter().find_map(|event| event.sequence()).unwrap();
    assert_eq!(drive.app.pads.bank, 0);
    drive.shared.inject_focus(5);
    drive.rejected_input();
    assert!(drive.invalid_frame().commands().is_empty());
    assert_eq!(drive.app.pads.bank, 0);
    assert_eq!(drive.invalid_errors().len(), 1);
    drive.app.publish_midi(&drive.shared);
    drive.bytes(PAD_PORT, &[0x90, 38, 101]);
    drive.bytes(PAD_PORT, &[0x80, 38, 0]);
    let next = drive.queued_events();
    assert_eq!(peak(&drive.render_blocks(2).0), 0.0);
    assert!(
        next.iter()
            .any(|event| event.sequence() == Some(sequence.wrapping_add(1))
                && matches!(
                    event.event,
                    midi::Event::PadHit {
                        sound: midi::PadSound::UiUnattempted { deferred: true },
                        ..
                    }
                ))
    );
    assert_eq!(
        next.iter()
            .filter(|event| event.sequence().is_some())
            .count(),
        1
    );
    for event in earlier.into_iter().chain(next) {
        drive.app.midi_ui_event(event);
    }
    let frame = drive.frame();
    assert_eq!(drive.app.pads.bank, 1);
    assert!(
        matches!(frame.commands()[..], [Command::PadPress {pad: 18, velocity},
        Command::PadRelease {pad: 18}] if *velocity == 101.0 / 127.0),
        "commands: {:?}",
        frame.commands()
    );
    assert!(drive.app.midi_ui_held.is_empty());
    drive.render_blocks(32);
    drive.bytes(PAD_PORT, &[0x90, 38, 77]);
    let resumed = drive.queued_events();
    assert!(resumed.iter().any(|event| event.sequence().is_none()
        && matches!(event.event,
        midi::Event::PadHit {abs: 18, velocity, sound: midi::PadSound::DirectAccepted, ..}
        if velocity == 77.0 / 127.0)));
    for event in resumed {
        drive.app.midi_ui_event(event);
    }
    assert!(drive.frame().ui.is_empty());
    assert!(peak(&drive.render_blocks(8).0) > 0.01);
    drive.bytes(PAD_PORT, &[0x80, 38, 0]);
    assert!(drive.frame().ui.is_empty());
    assert_eq!(drive.app.audio_held[18], None);
    assert_eq!(peak(&drive.render_blocks(64).0[4_096..]), 0.0);
    drive.shared.inject_focus(6);
    drive.rejected_input();
    assert!(drive.invalid_frame().commands().is_empty());
    drive.shared.inject_focus(7);
    drive.rejected_input();
    assert!(drive.invalid_frame().commands().is_empty());
    assert_eq!(
        drive.invalid_errors(),
        [
            "MIDI rejected new input: invalid state 0x00000005",
            "MIDI rejected new input: invalid state 0x00000006",
        ]
    );
    drive.app.publish_midi(&drive.shared);
    drive.bytes(PAD_PORT, &[0x90, 38, 90]);
    let hit = drive.queued_events();
    assert!(hit.iter().any(|event| event.sequence().is_none()
        && matches!(event.event,
        midi::Event::PadHit {abs: 18, velocity, sound: midi::PadSound::DirectAccepted, ..}
        if velocity == 90.0 / 127.0)));
}

#[test]
fn invalid_midi_state_preserves_original_releases() {
    midi_state_log();
    for source in [Source::Notes, Source::Button] {
        for release in [
            RecordedRelease::Off,
            RecordedRelease::Zero,
            RecordedRelease::Disconnect,
        ] {
            for full_ui_queue in [false, true] {
                let mut drive = Drive::new();
                drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 48_000])), 4);
                drive.link_chop_pad(18, 3);
                drive.link_chop_pad(34, 3);
                drive.tap(KeyCode::F(2), M::NONE);
                drive.tap(KeyCode::Char('2'), M::CONTROL);
                assert_full_hit(&drive.pad_press(source, 2), 18);
                drive.tap(KeyCode::F(3), M::NONE);
                drive.tap(KeyCode::Char('3'), M::CONTROL);
                let hit = drive.pad_press(source, 2);
                assert_full_hit(&hit, 34);
                assert!(hit.direct.is_empty());
                assert_eq!(drive.app.midi_ui_held.len(), 1);
                assert!(drive.app.pads.held[18] && drive.app.pads.held[34]);
                if full_ui_queue {
                    let mut filled = 0;
                    while drive
                        .app
                        .audio
                        .as_mut()
                        .unwrap()
                        .commands
                        .push(ProjectCommand {
                            epoch: 0,
                            ready_at_send: true,
                            command: ProjectMessage::Command(Command::SetPadGain {
                                pad: 127,
                                gain: 1.0,
                            }),
                        })
                        .is_ok()
                    {
                        filled += 1;
                    }
                    assert!(filled > 0);
                }
                drive.shared.inject_focus(5);
                drive.bytes(PAD_PORT, &[0x80, 40, 0]);
                assert!(drive.invalid_frame().commands().is_empty());
                drive.raw_ordered_release(source, release);
                let frame = drive.invalid_frame();
                assert!(matches!(
                    frame.direct[..],
                    [ProjectCommand {
                        epoch: 0,
                        command: ProjectMessage::Command(Command::PadRelease { pad: 18 }),
                        ..
                    }]
                ));
                assert!(matches!(frame.ui[..], [Command::PadRelease { pad: 34 }]));
                assert!(drive.app.midi_ui_held.is_empty());
                assert!(!drive.app.pads.held[18] && !drive.app.pads.held[34]);
                assert_eq!(drive.shared.focus(), Err(5));
                assert_eq!(drive.invalid_errors().len(), 1);
                if full_ui_queue {
                    assert_eq!(drive.app.pending_audio.len(), 1);
                    drive.render_blocks(1);
                    drive.app.flush_audio();
                    assert!(drive.app.pending_audio.is_empty());
                    assert_eq!(drive.shared.focus(), Err(5));
                }
            }
        }
    }
}

#[test]
fn invalid_midi_state_keeps_release_sources_separate() {
    midi_state_log();
    const OTHER: &str = "Recorded pads MIDI 2";
    let mut drive = Drive::new();
    drive.midi().recorded_connect(OTHER);
    drive.frame();
    for (bank, port, channel, note, abs) in [
        (1, PAD_PORT, 0, 38, 18),
        (2, PAD_PORT, 1, 38, 34),
        (3, OTHER, 0, 38, 50),
        (4, PAD_PORT, 0, 39, 67),
    ] {
        drive.tap(KeyCode::Char(char::from(b'1' + bank)), M::CONTROL);
        drive.bytes(port, &[0x90 | channel, note, 127]);
        assert_full_hit(&drive.frame(), abs);
    }
    drive.shared.inject_focus(5);
    for (port, status, note) in [
        (PAD_PORT, 0x82, 38),
        (OTHER, 0x81, 38),
        (PAD_PORT, 0x80, 40),
    ] {
        drive.bytes(port, &[status, note, 0]);
    }
    assert!(drive.invalid_frame().commands().is_empty());
    drive.bytes(PAD_PORT, &[0x80, 38, 0]);
    assert_only_release(&drive.invalid_frame(), 18);
    for abs in [34, 50, 67] {
        assert!(drive.app.pads.held[abs]);
    }
    drive.midi().recorded_disconnect(PAD_PORT);
    let frame = drive.invalid_frame();
    let mut released: Vec<_> = frame
        .commands()
        .into_iter()
        .map(|command| match command {
            Command::PadRelease { pad } => *pad,
            other => panic!("not a release: {other:?}"),
        })
        .collect();
    released.sort_unstable();
    assert_eq!(released, [34, 67]);
    assert!(drive.app.pads.held[50]);
    drive.midi().recorded_disconnect(OTHER);
    assert_only_release(&drive.invalid_frame(), 50);
    assert!(drive.app.pads.held.iter().all(|held| !held));
    assert_eq!(drive.invalid_errors().len(), 1);
}

#[test]
fn invalid_midi_state_retries_original_direct_releases() {
    midi_state_log();
    for release in [RecordedRelease::Off, RecordedRelease::Disconnect] {
        let mut drive = Drive::new();
        drive.tap(KeyCode::Char('2'), M::CONTROL);
        drive.bytes(PAD_PORT, &[0x90, 38, 127]);
        assert_full_hit(&drive.frame(), 18);
        for _ in 0..1024 {
            drive.bytes(PAD_PORT, &[0x90, 39, 90]);
        }
        for event in drive.queued_events() {
            assert_eq!(event.sequence(), None);
            drive.app.midi_ui_event(event);
        }
        drive.shared.inject_focus(5);
        drive.bytes(PAD_PORT, &[0x80, 40, 0]);
        drive.raw_ordered_release(Source::Notes, release);
        for event in drive.queued_events() {
            assert_eq!(event.sequence(), None);
            drive.app.midi_ui_event(event);
        }
        for _ in 0..1024 {
            let command = drive.direct.as_mut().unwrap().pop().unwrap();
            assert_eq!(command.epoch, 0);
            assert!(matches!(
                command.command,
                ProjectMessage::Command(Command::PadPress { pad: 19, .. })
            ));
        }
        drive.midi().recorded_fence();
        let frame = drive.invalid_frame();
        assert!(frame.ui.is_empty());
        assert!(
            frame
                .direct
                .iter()
                .all(|command| command.epoch == 0 && command.ready_at_send)
        );
        let mut released: Vec<_> = frame
            .commands()
            .into_iter()
            .map(|command| match command {
                Command::PadRelease { pad } => *pad,
                other => panic!("not a release: {other:?}"),
            })
            .collect();
        released.sort_unstable();
        assert_eq!(
            released,
            if matches!(release, RecordedRelease::Disconnect) {
                vec![18, 19]
            } else {
                vec![18]
            }
        );
        assert_eq!(drive.shared.focus(), Err(5));
        assert_eq!(drive.invalid_errors().len(), 1);
    }
}

#[test]
fn invalid_midi_state_cannot_release_a_new_project() {
    use engine::checks::{first_non_finite, peak};
    use std::sync::atomic::Ordering::Acquire;
    midi_state_log();
    for source in [Source::Notes, Source::Button] {
        for ui_route in [false, true] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            if !ui_route {
                drive.tap(KeyCode::F(2), M::NONE);
            }
            drive.velocity_hit(source, 127);
            assert!(peak(&drive.render_blocks(8).0) > 0.01);
            drive.shared.inject_focus(5);
            drive.app.apply_session(session::Session::default());
            for _ in 0..256 {
                drive.app.flush_audio();
                drive.render_blocks(1);
                if drive.project.ready.load(Acquire) == 1 && drive.app.pending_audio.is_empty() {
                    break;
                }
            }
            assert_eq!(drive.project.ready.load(Acquire), 1);
            assert!(drive.app.pending_audio.is_empty());
            drive.frame();
            drive.app.pads.track_keys = true;
            drive.parity_pad(0, false);
            drive.tap(KeyCode::F(2), M::NONE);
            drive.render_blocks(32);
            drive.velocity_hit(Source::Keyboard, 127);
            assert!(peak(&drive.render_blocks(32).0) > 0.01);
            assert_eq!(drive.app.audio_held[18], Some(1));
            drive.shared.inject_focus(5);
            drive.raw_ordered_release(source, RecordedRelease::Off);
            assert!(drive.invalid_frame().commands().is_empty());
            assert_eq!(drive.app.audio_held[18], Some(1));
            let wave = drive.render_blocks(32);
            assert_eq!(first_non_finite(&wave.0), None);
            assert!(peak(&wave.0) > 0.01 && peak(&wave.0) < 1.0);
            drive.key(KeyCode::Char('3'), M::SHIFT, KeyEventKind::Release);
            assert_only_release(&drive.invalid_frame(), 18);
            assert_eq!(peak(&drive.render_blocks(64).0[4_096..]), 0.0);
            assert_eq!(drive.shared.focus(), Err(5));
            assert_eq!(drive.invalid_errors().len(), 1);
        }
    }
}

#[test]
fn invalid_midi_state_keeps_audio_and_unrelated_holds_running() {
    use engine::checks::{first_non_finite, peak};
    midi_state_log();
    for full_midi_queue in [false, true] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        let pad = drive.app.pads.snapshot(18);
        for command in drive.app.pads.put_back(19, &pad) {
            drive.app.send(command);
        }
        drive.app.send(Command::SetPadSound {
            pad: 31,
            sound: None,
        });
        drive.tap(KeyCode::F(2), M::NONE);
        drive.render_blocks(32);
        drive.velocity_hit(Source::Notes, 127);
        drive.pad_press(Source::Keyboard, 3);
        assert!(peak(&drive.render_blocks(8).0) > 0.01);
        if full_midi_queue {
            // Fill the callback's MIDI queue with hits on an unloaded pad.
            for _ in 0..1024 {
                drive.bytes(PAD_PORT, &[0x90, 51, 127]);
            }
            let hits = drive.queued_events();
            assert_eq!(
                hits.iter()
                    .filter(|event| matches!(
                        event.event,
                        midi::Event::PadHit {
                            abs: 31,
                            sound: midi::PadSound::DirectAccepted,
                            ..
                        }
                    ))
                    .count(),
                1024
            );
            for event in hits {
                drive.app.midi_ui_event(event);
            }
        }
        drive.shared.inject_focus(5);
        drive.bytes(PAD_PORT, &[0x80, 38, 0]);
        assert!(drive.invalid_frame().commands().is_empty());
        assert!(!drive.app.pads.held[18] && drive.app.pads.held[19]);
        assert_eq!(drive.app.audio_held[19], Some(0));
        if full_midi_queue {
            drive.render_blocks(1);
            drive.midi().recorded_fence();
        }
        let wave = drive.render_blocks(32);
        assert_eq!(first_non_finite(&wave.0), None);
        assert!(
            peak(&wave.0) > 0.01 && peak(&wave.0) < 1.0,
            "full MIDI queue: {full_midi_queue}, peak: {}",
            peak(&wave.0)
        );
        drive.key(KeyCode::Char('4'), M::SHIFT, KeyEventKind::Release);
        assert_only_release(&drive.invalid_frame(), 19);
        assert_eq!(peak(&drive.render_blocks(64).0[4_096..]), 0.0);
        assert_eq!(drive.shared.focus(), Err(5));
        assert_eq!(drive.invalid_errors().len(), 1);
    }
}

#[test]
fn recorded_driver_uses_both_production_pad_routes() {
    let mut keys = Drive::new();
    keys.key(KeyCode::Char('q'), M::SHIFT, KeyEventKind::Press);
    let keyboard = keys.frame();
    assert!(keyboard.direct.is_empty());
    assert!(matches!(
        keyboard.ui[..],
        [Command::PadPress {
            pad: 4,
            velocity: 1.0
        }]
    ));
    assert_eq!(keys.app.pads.selected, 4);
    keys.key(KeyCode::Char('q'), M::SHIFT, KeyEventKind::Release);
    assert!(matches!(
        keys.frame().ui[..],
        [Command::PadRelease { pad: 4 }]
    ));

    let mut notes = Drive::new();
    notes.bytes(PAD_PORT, &[0x90, 40, 127]);
    let controller = notes.frame();
    assert!(
        controller.ui.is_empty(),
        "a direct hit must not sound twice"
    );
    assert!(matches!(
        controller.commands()[..],
        [Command::PadPress {
            pad: 4,
            velocity: 1.0
        }]
    ));
    assert_eq!(notes.app.pads.selected, 4);
    notes.bytes(PAD_PORT, &[0x80, 40, 0]);
    let release = notes.frame();
    assert!(release.ui.is_empty());
    assert!(matches!(
        release.commands()[..],
        [Command::PadRelease { pad: 4 }]
    ));
}

#[test]
fn recorded_driver_applies_seeded_buttons_and_explicit_frames() {
    let mut drive = Drive::new();
    drive.bytes(PAD_PORT, &[0xb0, 27, 127]);
    drive.frame();
    assert_eq!(drive.app.pads.bank, 1);
    drive.bytes(PAD_PORT, &[0xb0, 27, 0]);
    drive.frame();
    assert_eq!(
        drive.app.pads.bank, 1,
        "button release is not another bank turn"
    );
    drive.bytes(PAD_PORT, &[0x90, 36, 127]);
    let hit = drive.frame();
    assert!(matches!(
        hit.commands()[..],
        [Command::PadPress {
            pad: 16,
            velocity: 1.0
        }]
    ));
    drive.bytes(PAD_PORT, &[0x90, 36, 0]);
    drive.frame();
    drive.tap(KeyCode::PageUp, M::NONE);
    assert_eq!(drive.app.pads.bank, 0);
    drive.bytes(XS_PORT, &[0x90, 45, 127]);
    drive.frame();
    assert_eq!(
        drive.app.screen,
        Screen::Fx,
        "the shipped X-Session profile acts after App seeding"
    );
}

fn assert_full_hit(frame: &Frame, abs: usize) {
    assert!(
        matches!(frame.commands()[..], [Command::PadPress { pad, velocity }]
            if *pad == abs && *velocity == 1.0),
        "expected one full hit on pad {abs}: {:?}",
        frame.commands()
    );
}

fn paired_baseline(mut run: impl FnMut(&mut Drive, Source) -> Vec<Seen>) {
    let mut expected = None;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let states = run(&mut Drive::new(), source);
        if let Some(expected) = &expected {
            assert_eq!(&states, expected, "{source:?}");
        } else {
            expected = Some(states);
        }
    }
}

#[test]
fn baseline_pads_parity_from_raw_midi() {
    paired_baseline(|drive, source| {
        let mut states = Vec::new();
        for bank in 0..2 {
            if bank > 0 {
                drive.tap(KeyCode::PageDown, M::NONE);
            }
            assert_full_hit(&drive.pad_press(source, 5), bank * PADS_PER_BANK + 5);
            assert_eq!(drive.app.pads.selected, 5);
            assert!(drive.app.pads.held[bank * PADS_PER_BANK + 5]);
            states.push(drive.seen());
            let release = drive.pad_release(source, 5);
            assert!(
                matches!(release.commands()[..], [Command::PadRelease { pad }]
                if *pad == bank * PADS_PER_BANK + 5)
            );
            assert!(!drive.app.pads.held[bank * PADS_PER_BANK + 5]);
            states.push(drive.seen());
        }
        states
    });
}

#[test]
fn baseline_pad_edit_parity_from_raw_midi() {
    paired_baseline(|drive, source| {
        match source {
            Source::Keyboard => {
                drive.tap(KeyCode::Enter, M::NONE);
            }
            Source::Notes | Source::Button => {
                drive.bytes(PAD_PORT, &[0xb0, 29, 127]);
                drive.frame();
                drive.bytes(PAD_PORT, &[0xb0, 29, 0]);
                drive.frame();
            }
        }
        assert!(drive.app.edit);
        drive.tap(KeyCode::Tab, M::NONE);
        let mut states = vec![drive.seen()];
        assert_full_hit(&drive.pad_press(source, 4), 4);
        assert!(drive.app.edit, "a pad hit keeps edit open");
        states.push(drive.seen());
        drive.pad_release(source, 4);
        states.push(drive.seen());
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(!drive.app.edit);
        states.push(drive.seen());
        states
    });
}

#[test]
fn baseline_marking_parity_from_raw_midi() {
    paired_baseline(|drive, source| {
        match source {
            Source::Keyboard => {
                drive.tap(KeyCode::Char('m'), M::NONE);
            }
            Source::Notes | Source::Button => {
                drive.bytes(PAD_PORT, &[0xb0, 28, 127]);
                drive.frame();
                drive.bytes(PAD_PORT, &[0xb0, 28, 0]);
                drive.frame();
            }
        }
        assert!(drive.app.pads.marking);
        let mut states = Vec::new();
        for (bank, local, marked) in [(0, 4, vec![4]), (1, 2, vec![4, 18]), (0, 4, vec![18])] {
            drive.tap(KeyCode::Char(char::from(b'1' + bank)), M::CONTROL);
            assert_full_hit(
                &drive.pad_press(source, local),
                usize::from(bank) * PADS_PER_BANK + local,
            );
            assert_eq!(
                drive.app.pads.marked.iter().copied().collect::<Vec<_>>(),
                marked
            );
            states.push(drive.seen());
            drive.pad_release(source, local);
            states.push(drive.seen());
        }
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.pads.marked.is_empty());
        assert!(!drive.app.pads.marking);
        states.push(drive.seen());
        states
    });
}

#[test]
fn baseline_learn_keeps_pads_silent_from_raw_midi() {
    paired_baseline(|drive, source| {
        drive.tap(KeyCode::F(10), M::NONE);
        assert!(drive.app.screen_learn.is_some());
        let before = drive.seen();
        let hit = drive.pad_press(source, 4);
        assert!(hit.commands().is_empty(), "learn must not play a pad");
        assert_eq!(
            drive.seen(),
            before,
            "learning does not change the pad state"
        );
        drive.pad_release(source, 4);
        assert!(drive.app.screen_learn.is_some());
        // Keyboard picks an action; MIDI supplies a control. Those learner states
        // intentionally differ, but neither input may play or select the pad.
        vec![drive.seen()]
    });
}

/// Scenario "Swap target and undo parity from raw MIDI".
fn baseline_swap_parity(source: Source) {
    let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
    for occupied in [true, false] {
        let mut expected = None;
        for source in [Source::Keyboard, source] {
            let mut drive = Drive::new();
            drive.load_pad(0, sound.clone(), "source");
            if occupied {
                drive.load_pad(18, sound.clone(), "target");
            }
            let source_pad = drive.app.pads.snapshot(0);
            let target_pad = drive.app.pads.snapshot(18);
            drive.tap(KeyCode::Char('x'), M::CONTROL);
            drive.tap(KeyCode::PageDown, M::NONE);
            let hit = drive.pad_press(source, 2);
            assert_silent_target(&hit);
            assert!(matches!(
                hit.commands()[..],
                [Command::SwapPads { a: 0, b: 18 }]
            ));
            assert_eq!(drive.app.pads.snapshot(0), target_pad);
            assert_eq!(drive.app.pads.snapshot(18), source_pad);
            assert!(drive.app.pad_in_hand.is_none());
            assert_eq!(
                (drive.app.pads.selected_abs(), drive.app.history.len()),
                (18, 1),
                "{source:?}: a swap selects its target and makes one undo step"
            );
            let seen = drive.seen();
            if let Some(expected) = &expected {
                assert_eq!(&seen, expected, "{source:?}");
            } else {
                expected = Some(seen);
            }
            drive.pad_release(source, 2);
            assert_eq!(drive.app.history[0].what, "swap");
            assert_eq!(
                drive.app.history[0]
                    .pads
                    .iter()
                    .map(|(pad, _)| *pad)
                    .collect::<Vec<_>>(),
                vec![0, 18]
            );
            drive.tap(KeyCode::Char('z'), M::CONTROL);
            assert_eq!(drive.app.pads.snapshot(0), source_pad);
            assert_eq!(drive.app.pads.snapshot(18), target_pad);
            assert!(drive.app.history.is_empty());
        }
    }
}

#[test]
fn swap_cancellation_and_loading_refusal_parity_from_raw_midi() {
    let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
    paired_baseline(|drive, source| {
        drive.load_pad(0, sound.clone(), "source");
        drive.load_pad(18, sound.clone(), "target");
        let original: Vec<_> = (0..PADS).map(|pad| drive.app.pads.snapshot(pad)).collect();
        drive.tap(KeyCode::Char('x'), M::CONTROL);
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.pad_in_hand.is_none());
        let mut states = vec![drive.seen()];
        drive.tap(KeyCode::Char('x'), M::CONTROL);
        let same_source = drive.pad_press(source, 0);
        assert_silent_target(&same_source);
        assert!(same_source.commands().is_empty());
        assert!(drive.app.pad_in_hand.is_none());
        assert!(drive.app.history.is_empty());
        states.push(drive.seen());
        drive.pad_release(source, 0);
        drive.tap(KeyCode::Char('x'), M::CONTROL);
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.app.pads.loading[18] = Some(1);
        let refused = drive.pad_press(source, 2);
        assert_silent_target(&refused);
        assert!(refused.commands().is_empty());
        assert!(drive.app.pad_in_hand.is_none());
        assert!(drive.app.history.is_empty());
        assert_eq!(
            drive.last_notice().text,
            "Wait until loading finishes, then swap."
        );
        assert_eq!(
            (0..PADS)
                .map(|pad| drive.app.pads.snapshot(pad))
                .collect::<Vec<_>>(),
            original
        );
        states.push(drive.seen());
        states
    });
}

#[test]
fn swap_target_and_undo_parity_from_raw_midi_notes() {
    baseline_swap_parity(Source::Notes);
}

#[test]
fn swap_target_and_undo_parity_from_raw_midi_learned_button() {
    baseline_swap_parity(Source::Button);
}

/// Scenario "File target parity from raw MIDI".
fn baseline_file_hand_parity(source: Source) {
    let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
    let mut expected = None;
    for source in [Source::Keyboard, source] {
        let mut drive = Drive::new();
        drive.load_pad(18, sound.clone(), "old target");
        // An occupied target tests routing and the question without decoding a file.
        drive.app.browser.in_hand = Some("new file.wav".into());
        drive.tap(KeyCode::PageDown, M::NONE);
        let hit = drive.pad_press(source, 2);
        assert_silent_target(&hit);
        assert!(matches!(&drive.app.ask, Some(Ask::PutFile { abs: 18, .. })));
        assert_eq!(
            drive.app.pads.selected_abs(),
            18,
            "{source:?}: file target selection"
        );
        let seen = drive.seen();
        if let Some(expected) = &expected {
            assert_eq!(&seen, expected, "{source:?}");
        } else {
            expected = Some(seen);
        }
        drive.pad_release(source, 2);
        let question = drive.seen();
        assert_silent_target(&drive.pad_press(source, 3));
        assert_eq!(drive.seen(), question);
        drive.pad_release(source, 3);
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.ask.is_none());
        assert!(drive.app.browser.in_hand.is_some());
        assert!(drive.app.history.is_empty());
        assert_eq!(
            drive.app.pads.custom_name[18].as_deref(),
            Some("old target")
        );
        assert_silent_target(&drive.pad_press(source, 2));
        assert!(matches!(&drive.app.ask, Some(Ask::PutFile { abs: 18, .. })));
        drive.pad_release(source, 2);
    }
}

#[test]
fn file_target_parity_from_raw_midi_notes() {
    baseline_file_hand_parity(Source::Notes);
}

#[test]
fn file_target_parity_from_raw_midi_learned_button() {
    baseline_file_hand_parity(Source::Button);
}

/// Scenario "Question pad silence from raw MIDI".
fn baseline_question_keeps_pads_silent(source: Source) {
    let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
    for source in [Source::Keyboard, source] {
        let mut drive = Drive::new();
        for pad in [0, 1, 18] {
            drive.load_pad(pad, sound.clone(), "loaded");
        }
        drive.app.pads.marked.extend([0, 1]);
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::Backspace, M::NONE);
        assert!(matches!(&drive.app.ask, Some(Ask::ClearPads { .. })));
        let before = drive.seen();
        assert_silent_target(&drive.pad_press(source, 2));
        assert_eq!(
            drive.seen(),
            before,
            "{source:?}: a question keeps its state"
        );
        drive.pad_release(source, 2);
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.ask.is_none());
        assert_eq!(
            drive.app.pads.marked.iter().copied().collect::<Vec<_>>(),
            vec![0, 1]
        );
        let hit = drive.pad_press(source, 2);
        assert_full_hit(&hit, 18);
        match source {
            Source::Keyboard => assert!(hit.direct.is_empty()),
            Source::Notes | Source::Button => assert!(hit.ui.is_empty()),
        }
        drive.pad_release(source, 2);
    }
}

/// Scenario "Question route preserves held releases".
#[test]
fn question_route_preserves_held_releases() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
        drive.load_pad(0, sound.clone(), "held");
        drive.load_pad(16, sound, "target");
        assert_full_hit(&drive.pad_press(source, 0), 0);
        drive.tap(KeyCode::F(6), M::NONE);
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.app.browser.in_hand = Some("new file.wav".into());
        drive.frame();
        drive.tap(KeyCode::Enter, M::NONE);
        assert_eq!(drive.app.screen, Screen::Files);
        assert!(matches!(&drive.app.ask, Some(Ask::PutFile { abs: 16, .. })));
        assert!(drive.app.pads.held[0]);
        let question = drive.app.ask.as_ref().unwrap().lines().to_vec();
        let release = drive.pad_release(source, 0);
        assert!(
            matches!(release.commands()[..], [Command::PadRelease { pad: 0 }]),
            "{source:?}: {:?}",
            release.commands()
        );
        assert!(!drive.app.pads.held[0]);
        assert!(!drive.app.pads.held[16]);
        assert_eq!(drive.app.ask.as_ref().unwrap().lines(), question);
    }
}

#[test]
fn question_pad_silence_from_raw_midi_notes() {
    baseline_question_keeps_pads_silent(Source::Notes);
}

#[test]
fn question_pad_silence_from_raw_midi_learned_button() {
    baseline_question_keeps_pads_silent(Source::Button);
}

fn assert_slice_preview(frame: &Frame, sound: &Arc<Sample>, expected: Region) {
    assert!(frame.direct.is_empty(), "Chop must stay on the UI route");
    assert!(
        matches!(&frame.ui[..], [Command::PreviewAs { pad: None }, Command::Preview { sound: Some(actual), region }]
        if Arc::ptr_eq(actual, sound) && *region == expected),
        "one slice preview: {:?}",
        frame.ui
    );
}

/// Scenario "Chop parity before pad assignment".
#[test]
fn chop_parity_before_pad_assignment() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    paired_baseline(|drive, source| {
        drive.prepare_chop(sound.clone(), 20);
        let slices = drive.app.chop.as_ref().unwrap().slices();
        let mut states = Vec::new();
        for (bank, local, slice) in [(0, 5, 5), (1, 2, 18), (7, 3, 19)] {
            drive.tap(KeyCode::Char(char::from(b'1' + bank)), M::CONTROL);
            let hit = drive.pad_press(source, local);
            assert_slice_preview(&hit, &sound, slices[slice]);
            assert_eq!(
                drive.app.chop.as_ref().unwrap().selected,
                slice,
                "{source:?}"
            );
            states.push(drive.seen());
            drive.pad_release(source, local);
        }
        states
    });
}

/// Scenario "Chop parity with linked pads". Sound settings are checked by the render in 2.2.
#[test]
fn chop_parity_with_linked_pads() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    paired_baseline(|drive, source| {
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(5, 2);
        drive.link_chop_pad(18, 3);
        let original = drive.seen().pads;
        let mut states = Vec::new();
        for (bank, local, abs, slice) in [(0, 5, 5, 2), (1, 2, 18, 3)] {
            drive.tap(KeyCode::Char(char::from(b'1' + bank)), M::CONTROL);
            let hit = drive.pad_press(source, local);
            assert!(hit.direct.is_empty());
            assert_full_hit(&hit, abs);
            assert_eq!(drive.app.chop.as_ref().unwrap().selected, slice);
            assert_eq!(drive.app.pads.selected_abs(), abs);
            assert_eq!(drive.seen().pads, original);
            states.push(drive.seen());
            drive.pad_release(source, local);
        }
        states
    });
}

/// Scenario "Chop MIDI linked pad keeps velocity". The level check belongs to render 2.2.
#[test]
fn chop_midi_linked_pad_keeps_velocity() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        for velocity in [23, 101] {
            match source {
                Source::Notes => drive.bytes(PAD_PORT, &[0x90, 38, velocity]),
                Source::Button => drive.bytes(PAD_PORT, &[0xb0, 80, velocity]),
                Source::Keyboard => unreachable!(),
            }
            let hit = drive.frame();
            assert!(hit.direct.is_empty());
            assert!(
                matches!(hit.commands()[..], [Command::PadPress { pad: 18, velocity: actual }]
                if *actual == f32::from(velocity) / 127.0),
                "{source:?}: {:?}",
                hit.commands()
            );
            assert_eq!(drive.app.chop.as_ref().unwrap().selected, 3);
            drive.pad_release(source, 2);
        }
    }
}

/// Scenario "Chop parity in by-hand assignment".
#[test]
fn chop_parity_in_by_hand_assignment() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    paired_baseline(|drive, source| {
        drive.prepare_chop(sound.clone(), 4);
        drive.load_pad(18, sound.clone(), "occupied");
        let original = drive.seen().pads;
        let slices = drive.app.chop.as_ref().unwrap().slices();
        drive.tap(KeyCode::Char('p'), M::NONE);
        assert_eq!(drive.app.plan, Some(chop::Plan::ByHand(Vec::new())));
        let mut states = Vec::new();
        let mut targets = Vec::new();
        for (slice, (bank, local)) in [(0, 5), (1, 2), (1, 4), (0, 6)].into_iter().enumerate() {
            drive.tap(KeyCode::Char(char::from(b'1' + bank)), M::CONTROL);
            let hit = drive.pad_press(source, local);
            assert_slice_preview(&hit, &sound, slices[slice]);
            targets.push((slice, usize::from(bank) * PADS_PER_BANK + local));
            let mut expected = targets.clone();
            expected.sort_by_key(|target| target.1);
            assert_eq!(
                drive.app.plan.as_ref().unwrap().targets(4),
                expected,
                "{source:?}"
            );
            assert_eq!(
                drive.app.chop.as_ref().unwrap().selected,
                (slice + 1).min(3)
            );
            assert!(drive.app.ask.is_none());
            assert_eq!(
                drive.seen().pads,
                original,
                "a hit plans; it does not apply"
            );
            states.push(drive.seen());
            drive.pad_release(source, local);
        }
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(matches!(&drive.app.ask, Some(Ask::Plan { .. })));
        assert_eq!(
            drive.seen().pads,
            original,
            "only confirmation may replace the occupied target"
        );
        states.push(drive.seen());
        states
    });
}

/// Scenario "Chop parity while a question is open".
#[test]
fn chop_parity_while_a_question_is_open() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    paired_baseline(|drive, source| {
        drive.prepare_chop(sound.clone(), 4);
        drive.load_pad(18, sound.clone(), "occupied");
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::Char('p'), M::NONE);
        drive.pad_press(Source::Keyboard, 2);
        drive.pad_release(Source::Keyboard, 2);
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(matches!(&drive.app.ask, Some(Ask::Plan { .. })));
        let before = drive.seen();
        let hit = drive.pad_press(source, 4);
        assert!(hit.commands().is_empty());
        assert_eq!(drive.seen(), before);
        vec![drive.seen()]
    });
}

/// Scenario "Chop parity while learning".
#[test]
fn chop_parity_while_learning() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    paired_baseline(|drive, source| {
        drive.prepare_chop(sound.clone(), 4);
        drive.tap(KeyCode::Char('p'), M::NONE);
        drive.tap(KeyCode::F(10), M::NONE);
        assert!(drive.app.screen_learn.is_some());
        let before = drive.seen();
        let hit = drive.pad_press(source, 2);
        assert!(hit.commands().is_empty());
        assert_eq!(drive.seen(), before);
        vec![drive.seen()]
    });
}

fn check_chop_linked_gate_release(change_context: bool, source: Source) {
    for source in [Source::Keyboard, source] {
        let mut drive = Drive::new();
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 32_000])), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        let hit = drive.pad_press(source, 2);
        assert!(hit.direct.is_empty());
        assert_full_hit(&hit, 18);
        assert!(drive.app.pads.held[18]);
        if change_context {
            drive.tap(KeyCode::PageDown, M::NONE);
            drive.tap(KeyCode::F(4), M::NONE);
        }
        let release = drive.pad_release(source, 2);
        assert!(
            matches!(release.commands()[..], [Command::PadRelease { pad: 18 }]),
            "{source:?}, context changed: {change_context}, original held: {}: {:?}",
            drive.app.pads.held[18],
            release.commands()
        );
        assert!(!drive.app.pads.held[18]);
    }
}

/// Scenario "Chop linked gate release reaches audio".
#[test]
fn chop_linked_gate_release_reaches_audio() {
    check_chop_linked_gate_release(false, Source::Notes);
}

#[test]
fn chop_linked_button_gate_release_reaches_audio() {
    check_chop_linked_gate_release(false, Source::Button);
}

/// Scenario "Chop linked gate release keeps original pad".
#[test]
fn chop_linked_gate_release_keeps_original_pad() {
    check_chop_linked_gate_release(true, Source::Notes);
}

#[test]
fn chop_linked_button_gate_release_keeps_original_pad() {
    check_chop_linked_gate_release(true, Source::Button);
}

fn assert_only_release(frame: &Frame, pad: usize) {
    assert!(
        matches!(frame.commands()[..], [Command::PadRelease { pad: actual }] if *actual == pad),
        "release {pad}: {:?}",
        frame.commands()
    );
}

#[test]
fn chop_linked_gate_release_passes_question_and_learn_guards() {
    for source in [Source::Notes, Source::Button] {
        for learn in [false, true] {
            let mut drive = Drive::new();
            let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
            drive.prepare_chop(sound.clone(), 4);
            drive.link_chop_pad(18, 3);
            drive.load_pad(34, sound, "occupied");
            drive.tap(KeyCode::PageDown, M::NONE);
            assert_full_hit(&drive.pad_press(source, 2), 18);
            if learn {
                drive.tap(KeyCode::F(10), M::NONE);
                assert!(drive.app.screen_learn.is_some());
            } else {
                drive.tap(KeyCode::F(6), M::NONE);
                drive.tap(KeyCode::PageDown, M::NONE);
                drive.app.browser.in_hand = Some("new file.wav".into());
                drive.frame();
                drive.tap(KeyCode::Enter, M::NONE);
                assert!(matches!(&drive.app.ask, Some(Ask::PutFile { abs: 34, .. })));
            }
            assert_only_release(&drive.pad_release(source, 2), 18);
            assert!(drive.app.midi_ui_held.is_empty());
            assert!(!drive.app.pads.held[18]);
            assert!(if learn {
                drive.app.screen_learn.is_some()
            } else {
                drive.app.ask.is_some()
            });
        }
    }
}

#[test]
fn chop_linked_gate_disconnect_keeps_original_pad() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 32_000])), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_full_hit(&drive.pad_press(source, 2), 18);
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::F(4), M::NONE);
        drive.midi().recorded_disconnect(PAD_PORT);
        assert_only_release(&drive.frame(), 18);
        assert!(drive.app.midi_ui_held.is_empty());
        assert!(!drive.app.pads.held[18]);
    }
}

#[test]
fn chop_ui_holds_keep_port_channel_and_control_separate() {
    let other = "Recorded pads MIDI 2";
    let mut drive = Drive::new();
    drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 32_000])), 4);
    drive.link_chop_pad(18, 3);
    drive.link_chop_pad(19, 2);
    drive.midi().recorded_connect(other);
    drive.frame();
    drive.tap(KeyCode::PageDown, M::NONE);
    for (port, bytes, pad) in [
        (PAD_PORT, [0x90, 38, 127], 18),
        (PAD_PORT, [0x91, 38, 127], 18),
        (other, [0x90, 38, 127], 18),
        (PAD_PORT, [0x90, 39, 127], 19),
    ] {
        drive.bytes(port, &bytes);
        assert_full_hit(&drive.frame(), pad);
    }
    assert_eq!(drive.app.midi_ui_held.len(), 4);
    drive.tap(KeyCode::PageDown, M::NONE);
    drive.tap(KeyCode::F(4), M::NONE);
    drive.bytes(PAD_PORT, &[0x81, 38, 0]);
    assert_only_release(&drive.frame(), 18);
    assert_eq!(drive.app.midi_ui_held.len(), 3);
    drive.midi().recorded_disconnect(other);
    assert_only_release(&drive.frame(), 18);
    assert_eq!(drive.app.midi_ui_held.len(), 2);
    drive.bytes(PAD_PORT, &[0x90, 38, 0]); // note-on zero is the original note-off
    assert_only_release(&drive.frame(), 18);
    assert_eq!(drive.app.midi_ui_held.len(), 1);
    drive.bytes(PAD_PORT, &[0x80, 39, 0]);
    assert_only_release(&drive.frame(), 19);
    assert!(drive.app.midi_ui_held.is_empty());
    drive.bytes(PAD_PORT, &[0x80, 38, 0]);
    assert!(
        drive.frame().commands().is_empty(),
        "an untracked release does not sound"
    );
}

/// Scenario "Chop silent hits have no pad release".
#[test]
fn chop_silent_hits_have_no_pad_release() {
    for source in [Source::Notes, Source::Button] {
        for mode in ["preview", "plan", "question", "learn"] {
            let mut drive = Drive::new();
            let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
            drive.prepare_chop(sound.clone(), 4);
            drive.tap(KeyCode::F(2), M::NONE);
            drive.load_pad(18, sound.clone(), "held");
            drive.tap(KeyCode::PageDown, M::NONE);
            assert_full_hit(&drive.pad_press(Source::Keyboard, 2), 18);
            drive.prepare_chop(sound.clone(), 4);
            match mode {
                "plan" => {
                    drive.tap(KeyCode::Char('p'), M::NONE);
                }
                "question" => {
                    drive.load_pad(16, sound.clone(), "occupied");
                    drive.tap(KeyCode::Char('p'), M::NONE);
                    drive.pad_press(Source::Keyboard, 0);
                    drive.pad_release(Source::Keyboard, 0);
                    drive.tap(KeyCode::Enter, M::NONE);
                    assert!(matches!(&drive.app.ask, Some(Ask::Plan { .. })));
                }
                "learn" => {
                    drive.tap(KeyCode::F(10), M::NONE);
                }
                _ => {}
            }
            let hit = drive.pad_press(source, 2);
            assert!(
                !hit.commands().iter().any(|command| matches!(
                    command,
                    Command::PadPress { .. } | Command::PadRelease { .. }
                )),
                "{mode}: {:?}",
                hit.commands()
            );
            assert!(drive.app.midi_ui_held.is_empty());
            assert!(
                drive.pad_release(source, 2).commands().is_empty(),
                "{mode}, {source:?}"
            );
            assert!(
                drive.app.pads.held[18],
                "the older keyboard hold survives {mode}"
            );
            let release = drive.pad_release(Source::Keyboard, 2);
            assert!(
                !release.commands().is_empty(),
                "older keyboard hold: {mode}, {source:?}"
            );
            assert_only_release(&release, 18);
        }
    }
}

/// Scenario "Held pad release passes screen learn".
#[test]
fn held_pad_release_passes_screen_learn() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for linked in [false, true] {
            for change_context in [false, true] {
                let mut drive = Drive::new();
                let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
                drive.prepare_chop(sound.clone(), 4);
                drive.link_chop_pad(18, 3);
                if !linked {
                    drive.tap(KeyCode::F(2), M::NONE);
                }
                drive.tap(KeyCode::PageDown, M::NONE);
                assert_full_hit(&drive.pad_press(source, 2), 18);
                if change_context {
                    drive.tap(KeyCode::PageDown, M::NONE);
                    drive.tap(KeyCode::F(6), M::NONE);
                }
                drive.tap(KeyCode::F(10), M::NONE);
                assert!(drive.app.screen_learn.is_some());
                assert!(drive.tap(KeyCode::Char('q'), M::NONE).commands().is_empty());
                drive.bytes(PAD_PORT, &[0xb0, 25, 127]);
                assert!(drive.frame().commands().is_empty());
                let learn = drive.app.screen_learn.as_ref().unwrap();
                let (picked, pending) = (learn.picked.clone(), learn.pending.clone());
                assert_eq!(picked.as_ref().unwrap().0, "pad.5");
                assert!(pending.is_some());
                let bindings = drive.app.learned.clone();
                assert_only_release(&drive.pad_release(source, 2), 18);
                assert!(!drive.app.pads.held[18]);
                assert!(!drive.app.pads.held[34]);
                let learn = drive.app.screen_learn.as_ref().unwrap();
                assert_eq!(learn.picked, picked, "{source:?}, linked: {linked}");
                assert_eq!(learn.pending, pending);
                assert_eq!(drive.app.learned, bindings);
                assert!(drive.pad_release(source, 2).commands().is_empty());
                // The release also clears HeldPads, so the next real hit is not a repeat.
                drive.tap(KeyCode::F(10), M::NONE);
                drive.tap(KeyCode::F(2), M::NONE);
                assert_full_hit(
                    &drive.pad_press(source, 2),
                    if change_context { 34 } else { 18 },
                );
                drive.pad_release(source, 2);
            }
        }
    }
}

/// Scenario "New learn keys do not release an older pad".
#[test]
fn new_learn_keys_do_not_release_an_older_pad() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 32_000])), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_full_hit(&drive.pad_press(source, 2), 18);
        drive.tap(KeyCode::F(10), M::NONE);
        let bindings = drive.app.learned.clone();
        for local in [4, 2] {
            // A MIDI-held local pad key has never been pressed on the keyboard.
            // Do not press an already held keyboard key as if it were a new key.
            if matches!(source, Source::Keyboard) && local == 2 {
                continue;
            }
            assert!(
                drive
                    .pad_press(Source::Keyboard, local)
                    .commands()
                    .is_empty()
            );
            assert!(
                drive
                    .pad_release(Source::Keyboard, local)
                    .commands()
                    .is_empty()
            );
            assert!(drive.app.pads.held[18], "{source:?}, silent key {local}");
            assert_eq!(drive.app.audio_held[18], Some(0));
            assert_eq!(drive.app.learned, bindings);
        }
        assert_only_release(&drive.pad_release(source, 2), 18);
        assert!(!drive.app.pads.held[18]);
        assert!(drive.app.screen_learn.is_some());
        assert_eq!(drive.app.learned, bindings);
    }
}

#[test]
fn chop_blocked_button_does_not_steal_a_note_hold() {
    for learn in [false, true] {
        let mut drive = Drive::new();
        let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_full_hit(&drive.pad_press(Source::Notes, 2), 18);
        if learn {
            drive.tap(KeyCode::F(10), M::NONE);
        } else {
            drive.load_pad(16, sound, "occupied");
            drive.tap(KeyCode::Char('p'), M::NONE);
            drive.pad_press(Source::Keyboard, 0);
            drive.pad_release(Source::Keyboard, 0);
            drive.tap(KeyCode::Enter, M::NONE);
            assert!(drive.app.ask.is_some());
        }
        assert!(drive.pad_press(Source::Button, 2).commands().is_empty());
        assert!(drive.pad_release(Source::Button, 2).commands().is_empty());
        assert_eq!(drive.app.midi_ui_held.len(), 1);
        assert!(drive.app.pads.held[18]);
        assert_only_release(&drive.pad_release(Source::Notes, 2), 18);
        assert!(drive.app.midi_ui_held.is_empty());
    }
}

/// Scenario "Chop release keeps direct route and epoch" (route half).
#[test]
fn chop_release_keeps_direct_route_without_a_duplicate() {
    for source in [Source::Notes, Source::Button] {
        for same_pad in [false, true] {
            let mut drive = Drive::new();
            let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
            drive.load_pad(18, sound.clone(), "break");
            drive.tap(KeyCode::PageDown, M::NONE);
            let direct = drive.pad_press(source, 2);
            assert_full_hit(&direct, 18);
            assert!(direct.ui.is_empty());
            drive.prepare_chop(sound.clone(), 4);
            let target = if same_pad { 18 } else { 34 };
            drive.link_chop_pad(target, 3);
            if !same_pad {
                drive.tap(KeyCode::PageDown, M::NONE);
            }
            let ui = drive.pad_press(source, 2);
            assert_full_hit(&ui, target);
            assert!(ui.direct.is_empty());
            let release = drive.pad_release(source, 2);
            assert_eq!(release.direct.len(), 1);
            assert!(matches!(
                release.direct[0].command,
                ProjectMessage::Command(Command::PadRelease { pad: 18 })
            ));
            if same_pad {
                assert!(release.ui.is_empty());
            } else {
                assert!(matches!(release.ui[..], [Command::PadRelease { pad: 34 }]));
            }
            assert!(drive.app.midi_ui_held.is_empty());
        }
    }
}

/// Scenario "Chop release keeps direct route and epoch" (project half).
#[test]
fn chop_old_project_hit_and_release_cannot_touch_a_new_hold() {
    use std::sync::atomic::Ordering::{Acquire, Release};
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_full_hit(&drive.pad_press(source, 2), 18);
        let release: &[u8] = match source {
            Source::Notes => &[0x80, 38, 0],
            Source::Button => &[0xb0, 80, 0],
            Source::Keyboard => unreachable!(),
        };
        drive.bytes(PAD_PORT, release);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(3);
        }
        drive.bytes(
            PAD_PORT,
            match source {
                Source::Notes => &[0x90, 39, 127],
                Source::Button => &[0xb0, 80, 127],
                Source::Keyboard => unreachable!(),
            },
        );
        drive.midi().recorded_fence();
        let old: Vec<_> = drive.midi().events.try_iter().collect();
        assert!(
            old.iter()
                .any(|event| matches!(event.event, midi::Event::UiPadRelease { .. }))
        );
        assert!(
            old.iter()
                .any(|event| matches!(event.event, midi::Event::PadHit { .. }))
        );
        drive.app.apply_session(session::Session::default());
        assert!(drive.app.midi_ui_held.is_empty());
        assert_eq!(drive.project.epoch.load(Acquire), 1);
        let mut left = [0.0; 128];
        let mut right = [0.0; 128];
        for _ in 0..256 {
            drive.app.flush_audio();
            drive
                .audio_rt
                .as_mut()
                .unwrap()
                .process(&mut left, &mut right);
            if drive.project.ready.load(Acquire) == 1 && drive.app.pending_audio.is_empty() {
                break;
            }
        }
        assert_eq!(drive.project.ready.load(Acquire), 1);
        assert!(drive.app.pending_audio.is_empty());
        drive.app.pads.track_keys = true;
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_full_hit(&drive.pad_press(source, 2), 18);
        assert_eq!(drive.app.midi_ui_held.len(), 1);
        for event in old {
            drive.app.midi_ui_event(event);
        }
        assert!(drive.frame().commands().is_empty());
        assert_eq!(drive.app.midi_ui_held.len(), 1);
        assert_eq!(drive.app.audio_held[18], Some(1));
        assert!(drive.app.pads.held[18]);
        assert_only_release(&drive.pad_release(source, 2), 18);
        assert!(drive.app.midi_ui_held.is_empty());
        drive.project.ready.store(0, Release);
        assert!(
            drive.pad_press(source, 2).commands().is_empty(),
            "not ready at input time"
        );
        drive.project.ready.store(1, Release);
        assert!(
            drive.pad_release(source, 2).commands().is_empty(),
            "rejected hit has no release"
        );
    }
}

/// Scenario "Chop rejected hits preserve accepted gate release".
#[test]
fn chop_rejected_hits_preserve_accepted_gate_release() {
    let mut drive = Drive::new();
    drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 32_000])), 4);
    drive.link_chop_pad(18, 3);
    drive.tap(KeyCode::PageDown, M::NONE);
    let (producer, mut consumer) = rtrb::RingBuffer::new(1);
    drive.app.audio.as_mut().unwrap().commands = producer;
    assert_full_hit(&drive.pad_press(Source::Notes, 2), 18);
    assert!(matches!(
        consumer.pop().unwrap().command,
        ProjectMessage::Command(Command::PadPress { pad: 18, .. })
    ));
    let original = drive.app.midi_ui_held.clone();
    drive
        .app
        .audio
        .as_mut()
        .unwrap()
        .commands
        .push(ProjectCommand {
            epoch: 0,
            ready_at_send: true,
            command: ProjectMessage::Command(Command::SetPadGain { pad: 0, gain: 1.0 }),
        })
        .unwrap();
    drive.pad_press(Source::Notes, 2); // same hold, rejected retrigger
    assert_eq!(
        drive.app.audio_held[18],
        Some(0),
        "the accepted hold must survive a rejected retrigger"
    );
    drive.pad_press(Source::Button, 2); // another control, also rejected
    assert_eq!(
        drive.app.midi_ui_held, original,
        "a rejected control does not own the older voice"
    );
    assert!(drive.pad_release(Source::Button, 2).commands().is_empty());
    assert_only_release(&drive.pad_release(Source::Notes, 2), 18);
    assert_eq!(drive.app.pending_audio.len(), 1);
    assert!(drive.app.midi_ui_held.is_empty());
    assert!(matches!(
        consumer.pop().unwrap().command,
        ProjectMessage::Command(Command::SetPadGain { .. })
    ));
    drive.app.flush_audio();
    assert!(matches!(
        consumer.pop().unwrap().command,
        ProjectMessage::Command(Command::PadRelease { pad: 18 })
    ));
    assert!(consumer.pop().is_err());
    assert!(drive.app.pending_audio.is_empty());
}

#[test]
fn chop_linked_gate_stops_in_offline_audio() {
    use engine::checks::{first_non_finite, peak};
    for (source, learn) in [Source::Keyboard, Source::Notes, Source::Button]
        .into_iter()
        .flat_map(|source| [(source, false), (source, true)])
    {
        let mut drive = Drive::new();
        let sound = Arc::new(Sample::mono(vec![0.25; 96_000]));
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(18, 3);
        let region = drive.app.pads.region[18];
        for command in [
            Command::SetPadSound {
                pad: 18,
                sound: Some(sound),
            },
            Command::SetPadRegion { pad: 18, region },
            Command::SetPadMode {
                pad: 18,
                mode: engine::PadMode::Gate,
            },
            Command::SetPadEnvelope {
                pad: 18,
                attack_ms: 0,
                decay_ms: 0,
                release_ms: 20,
            },
        ] {
            drive.app.send(command);
        }
        drive.frame();
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_full_hit(&drive.pad_press(source, 2), 18);
        let mut left = [0.0; 128];
        let mut right = [0.0; 128];
        let mut rendered = Vec::new();
        for _ in 0..8 {
            drive
                .audio_rt
                .as_mut()
                .unwrap()
                .process(&mut left, &mut right);
            rendered.extend_from_slice(&left);
            rendered.extend_from_slice(&right);
        }
        assert!(
            peak(&left) > 0.01,
            "{source:?}: accepted gate sounds before release"
        );
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::F(if learn { 6 } else { 4 }), M::NONE);
        if learn {
            drive.tap(KeyCode::F(10), M::NONE);
            assert!(drive.app.screen_learn.is_some());
        }
        assert_only_release(&drive.pad_release(source, 2), 18);
        for _ in 0..64 {
            drive
                .audio_rt
                .as_mut()
                .unwrap()
                .process(&mut left, &mut right);
            rendered.extend_from_slice(&left);
            rendered.extend_from_slice(&right);
        }
        assert_eq!(first_non_finite(&rendered), None);
        assert!(peak(&rendered) < 1.0);
        assert_eq!(
            peak(&left),
            0.0,
            "{source:?}: gate must stop, not play to the sample end"
        );
        assert_eq!(peak(&right), 0.0);
    }
}

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
                assert_eq!(stored.version, 4);
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

#[test]
fn modes_85_text_and_project_picker_keep_source_boundaries() {
    for context in ["name", "filter", "project"] {
        for source in [Source::Keyboard, Source::Notes, Source::Button] {
            let mut drive = Drive::new();
            drive.load_pad(0, Arc::new(Sample::mono(vec![0.25; 1024])), "source");
            match context {
                "name" => {
                    drive.tap(KeyCode::Char('n'), M::CONTROL);
                    assert!(drive.app.pad_naming.is_some());
                }
                "filter" => {
                    drive.tap(KeyCode::F(6), M::NONE);
                    drive.tap(KeyCode::Char('/'), M::NONE);
                    assert!(drive.app.browser.editing);
                }
                "project" => {
                    drive.app.paths.projects = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../target/85-no-project-root");
                    drive.tap(KeyCode::Char('p'), M::CONTROL);
                    assert!(drive.app.project_picker.is_some());
                }
                _ => unreachable!(),
            }
            let hit = drive.pad_press(source, 4);
            if matches!(source, Source::Keyboard) {
                if context == "filter" {
                    assert!(matches!(
                        hit.commands()[..],
                        [Command::Preview {
                            sound: None,
                            region: Region::WHOLE,
                        }]
                    ));
                } else {
                    assert!(hit.commands().is_empty(), "{context}: typing is silent");
                }
                assert_eq!(drive.app.pads.selected, 0);
                match context {
                    "name" => assert_eq!(drive.app.pad_naming.as_ref().unwrap().1, "sourceq"),
                    "filter" => assert_eq!(drive.app.browser.filter, "q"),
                    "project" => assert_eq!(
                        drive.app.project_picker.as_ref().unwrap().naming.as_deref(),
                        Some("q")
                    ),
                    _ => unreachable!(),
                }
            } else {
                assert_full_hit(&hit, 4);
                assert!(hit.ui.is_empty(), "{context}: existing direct MIDI route");
                assert_eq!(drive.app.pads.selected, 4);
                assert!(drive.app.browser.filter.is_empty());
                if context == "name" {
                    assert_eq!(drive.app.pad_naming.as_ref().unwrap().1, "source");
                }
                if context == "project" {
                    assert!(drive.app.project_picker.as_ref().unwrap().naming.is_none());
                }
            }
            // Keyboard releases pass the text guards too. `sent` records this
            // attempt even without an accepted sound; keep that existing boundary.
            assert_only_release(&drive.pad_release(source, 4), 4);
        }
    }
}

#[test]
fn modes_85_folder_picker_consumes_actions_not_direct_midi_hits() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.tap(KeyCode::F(12), M::NONE);
        drive.app.settings_tab = 3;
        // A fixed empty listing avoids reading the owner's folders. The production
        // picker handler must still consume resolved keyboard and controller actions.
        drive.app.path_picker = Some(PathPicker {
            which: 0,
            at: "target/85-picker".into(),
            entries: Vec::new(),
            selected: 0,
        });
        drive.frame();
        let hit = drive.pad_press(source, 4);
        if matches!(source, Source::Keyboard) {
            assert!(hit.commands().is_empty());
            assert_eq!(drive.app.pads.selected, 0);
        } else {
            assert_full_hit(&hit, 4);
            assert!(hit.ui.is_empty());
            assert_eq!(drive.app.pads.selected, 4);
        }
        assert!(drive.app.path_picker.is_some());
        let release = drive.pad_release(source, 4);
        if matches!(source, Source::Keyboard) {
            assert!(release.commands().is_empty());
        } else {
            assert_only_release(&release, 4);
        }
        drive.bytes(PAD_PORT, &[0xb0, 27, 127]);
        assert!(drive.frame().commands().is_empty());
        assert_eq!(drive.app.pads.bank, 0, "the picker consumes MIDI actions");
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.path_picker.is_none());
    }
}

#[test]
fn modes_85_f12_learning_does_not_acquire_f10_silence() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for run in [false, true] {
            let mut drive = Drive::new();
            drive.tap(KeyCode::F(12), M::NONE);
            drive.app.settings_tab = 2;
            drive.app.midi_open = vec![input::Group::Pads, input::Group::Global];
            drive.app.midi_row = drive
                .app
                .midi_list()
                .iter()
                .position(|row| {
                    if run {
                        matches!(
                            row,
                            MidiListRow::Set {
                                name: "pads 1–16",
                                ..
                            }
                        )
                    } else {
                        matches!(row, MidiListRow::Action { info } if info.name == "pad.copy")
                    }
                })
                .unwrap();
            drive.tap(KeyCode::Enter, M::NONE);
            assert_eq!(drive.app.learning.is_some(), !run);
            assert_eq!(drive.app.midi_run.is_some(), run);
            assert!(drive.app.screen_learn.is_none());
            let hit = drive.pad_press(source, 4);
            assert_full_hit(&hit, 4);
            if matches!(source, Source::Keyboard) {
                assert!(hit.direct.is_empty());
                if !run {
                    assert!(drive.app.learning.as_ref().unwrap().pending.is_none());
                }
            } else {
                assert!(hit.ui.is_empty());
                if !run {
                    assert!(drive.app.learning.as_ref().unwrap().pending.is_some());
                }
            }
            assert_only_release(&drive.pad_release(source, 4), 4);
        }
    }
}

#[test]
fn modes_85_question_keeps_keyboard_and_midi_edit_resolution() {
    for keyboard in [true, false] {
        let mut drive = Drive::new();
        drive.load_pad(0, Arc::new(Sample::mono(vec![0.25; 1024])), "source");
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.edit);
        for (group, action) in [
            (input::Group::Pads, "pad.edit"),
            (input::Group::PadsEdit, "ui.back"),
        ] {
            learn::assign(
                &mut drive.app.learned,
                "Recorded pads",
                group,
                input::midi::Key::Cc(81),
                action,
            );
            drive.app.sync_learned("Recorded pads", group);
        }
        drive.app.pads.marked.insert(0);
        drive.app.ask_clear();
        drive.frame();
        assert!(!drive.app.edit_keys(), "a question disables keyboard edit");
        assert!(drive.app.ask.is_some());
        if keyboard {
            drive.tap(KeyCode::Enter, M::NONE);
            assert!(drive.app.pads.file[0].is_none(), "screen Enter confirms");
        } else {
            drive.bytes(PAD_PORT, &[0xb0, 81, 127]);
            drive.frame();
            assert!(drive.app.input_log.last().unwrap().ends_with("-> Back"));
            assert!(
                drive.app.pads.file[0].is_some(),
                "MIDI retains its edit layer"
            );
        }
        assert!(drive.app.ask.is_none());
        assert!(drive.app.edit);
    }
}

#[test]
fn modes_85_chop_precedes_hand_targets_and_keeps_edit() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    paired_baseline(|drive, source| {
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(4, 2);
        drive.app.browser.in_hand = Some("new file.wav".into());
        drive.app.pad_in_hand = Some(0);
        drive.app.copy_in_hand = Some((0, drive.app.pads.snapshot(4)));
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.edit);
        let before = drive.seen().pads;
        let hit = drive.pad_press(source, 4);
        assert_full_hit(&hit, 4);
        assert!(hit.direct.is_empty());
        assert_eq!(drive.app.chop.as_ref().unwrap().selected, 2);
        assert_eq!(drive.seen().pads, before);
        assert!(drive.app.ask.is_none());
        assert!(drive.app.browser.in_hand.is_some());
        assert!(drive.app.copy_in_hand.is_some());
        assert_eq!(drive.app.pad_in_hand, Some(0));
        assert!(drive.app.edit);
        assert_only_release(&drive.pad_release(source, 4), 4);
        vec![drive.seen()]
    });
}

#[test]
fn modes_85_overlapping_hands_keep_source_specific_priority() {
    for with_file in [false, true] {
        for source in [Source::Keyboard, Source::Notes, Source::Button] {
            let mut drive = Drive::new();
            let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
            drive.load_pad(0, sound.clone(), "source");
            drive.load_pad(4, sound, "target");
            drive.tap(KeyCode::Char('c'), M::CONTROL);
            drive.tap(KeyCode::Char('x'), M::CONTROL);
            assert!(drive.app.copy_in_hand.is_some());
            assert_eq!(drive.app.pad_in_hand, Some(0));
            if with_file {
                drive.tap(KeyCode::F(6), M::NONE);
                drive.app.browser.dir = "target/85-files".into();
                drive.app.browser.entries = vec![browser::Entry {
                    name: "new file.wav".into(),
                    is_dir: false,
                    favorite: None,
                }];
                drive.app.browser.selected = 0;
                drive.tap(KeyCode::Enter, M::NONE);
                drive.tap(KeyCode::F(2), M::NONE);
                assert!(drive.app.browser.in_hand.is_some());
            }
            let before = drive.seen().pads;
            assert_silent_target(&drive.pad_press(source, 4));
            if with_file && !matches!(source, Source::Keyboard) {
                assert!(matches!(drive.app.ask, Some(Ask::PutFile { abs: 4, .. })));
            } else {
                assert!(matches!(
                    drive.app.ask,
                    Some(Ask::CopyPad { source: 0, .. })
                ));
            }
            assert_eq!(drive.seen().pads, before);
            assert_eq!(drive.app.pads.selected_abs(), 4);
            assert!(drive.app.history.is_empty());
            assert!(drive.app.copy_in_hand.is_some());
            assert_eq!(drive.app.pad_in_hand, Some(0));
            let release = drive.pad_release(source, 4);
            if matches!(source, Source::Keyboard) {
                // A keyboard target keeps its existing release attempt. It did
                // not start an accepted audio hold and adds no new pad hit.
                assert_only_release(&release, 4);
            } else {
                assert!(release.commands().is_empty());
            }
        }
    }
}

#[test]
fn modes_85_marking_and_edit_still_coexist_and_sound() {
    paired_baseline(|drive, source| {
        drive.tap(KeyCode::Char('m'), M::NONE);
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.pads.marking);
        assert!(drive.app.edit);
        assert_full_hit(&drive.pad_press(source, 4), 4);
        assert!(drive.app.pads.marked.contains(&4));
        assert!(drive.app.edit);
        assert_only_release(&drive.pad_release(source, 4), 4);
        vec![drive.seen()]
    });
}

const RENDER_RATE: u32 = 48_000;
const RENDER_BLOCK: usize = 128;
const PARITY_HIT: usize = 32 * RENDER_BLOCK;
const PARITY_RELEASE: usize = PARITY_HIT + 128 * RENDER_BLOCK;
const PARITY_END: usize = PARITY_RELEASE + 128 * RENDER_BLOCK;
type Stereo = (Vec<f32>, Vec<f32>);

impl Drive {
    fn render_blocks(&mut self, blocks: usize) -> Stereo {
        let mut out = (Vec::new(), Vec::new());
        let (mut left, mut right) = ([0.0; RENDER_BLOCK], [0.0; RENDER_BLOCK]);
        for _ in 0..blocks {
            self.audio_rt
                .as_mut()
                .unwrap()
                .process(&mut left, &mut right);
            out.0.extend_from_slice(&left);
            out.1.extend_from_slice(&right);
            self.app.audio.as_mut().unwrap().free_garbage();
        }
        out
    }

    fn parity_pad(&mut self, pitch: i8, filtered: bool) -> Vec<Command> {
        use std::f64::consts::TAU;
        let sound = Arc::new(Sample::mono(
            (0..4 * RENDER_RATE as usize)
                .map(|i| {
                    let t = i as f64 / f64::from(RENDER_RATE);
                    // Each one-second slice starts at a zero crossing, also in
                    // the filter-off control. No hard sample edge hides routing checks.
                    (0.2 * (TAU * 220.0 * t).sin() + 0.08 * (TAU * 3_600.0 * t).sin()) as f32
                })
                .collect(),
        ));
        self.prepare_chop(sound, 4);
        self.link_chop_pad(18, 3);
        self.app.pads.pitch[18] = pitch;
        self.app.pads.volume_db[18] = -3;
        self.app.pads.attack_ms[18] = 0;
        self.app.pads.decay_ms[18] = 0;
        self.app.pads.release_ms[18] = 20;
        if !filtered {
            self.app.pads.filter[18] = engine::filter::FilterKind::Off;
        }
        let pad = self.app.pads.snapshot(18);
        let commands = self.app.pads.put_back(18, &pad);
        for command in &commands {
            self.app.send(command.clone());
        }
        self.tap(KeyCode::Char('2'), M::CONTROL);
        commands
    }

    fn velocity_hit(&mut self, source: Source, velocity: u8) -> Frame {
        match source {
            Source::Keyboard => {
                assert!(matches!(velocity, 80 | 127));
                self.key(
                    KeyCode::Char('3'),
                    if velocity == 127 { M::SHIFT } else { M::NONE },
                    KeyEventKind::Press,
                );
            }
            Source::Notes => self.bytes(PAD_PORT, &[0x90, 38, velocity]),
            Source::Button => {
                self.bind_pad_button(2);
                self.bytes(PAD_PORT, &[0xb0, 80, velocity]);
            }
        }
        self.frame()
    }
}

fn append_stereo(to: &mut Stereo, from: Stereo) {
    to.0.extend(from.0);
    to.1.extend(from.1);
}

fn check_parity_wave(wave: &Stereo) {
    use engine::checks::{first_loud, first_non_finite, max_step, peak};
    for channel in [&wave.0, &wave.1] {
        assert_eq!(first_non_finite(channel), None);
        assert!(peak(channel) < 1.0, "clipping");
        assert_eq!(peak(&channel[..PARITY_HIT]), 0.0);
        let onset = first_loud(channel, PARITY_HIT..PARITY_HIT + RENDER_BLOCK, 1e-6).unwrap();
        assert!(
            onset.abs_diff(PARITY_HIT) <= 1,
            "nearest-block onset: {onset}, head {:?}",
            &channel[PARITY_HIT..PARITY_HIT + 5]
        );
        assert!(peak(&channel[PARITY_RELEASE - RENDER_BLOCK..PARITY_RELEASE]) > 0.01);
        assert_eq!(
            peak(&channel[PARITY_RELEASE + 2_048..]),
            0.0,
            "gate tail ends"
        );
        assert!(max_step(channel) < 0.08, "unexpected sample jump");
    }
}

#[derive(Clone, Copy, Debug)]
enum RecordedRelease {
    Off,
    Zero,
    Disconnect,
}

/// Raw input -> production dispatch/App -> real queues -> callback. The separate
/// timestamped engine render is only an oracle; it does not feed this route.
fn render_chop_route(
    source: Source,
    velocity: u8,
    pitch: i8,
    filtered: bool,
    release: RecordedRelease,
) -> Stereo {
    let mut drive = Drive::offline();
    let settings = drive.parity_pad(pitch, filtered);
    let mut wave = drive.render_blocks(PARITY_HIT / RENDER_BLOCK);
    let hit = drive.velocity_hit(source, velocity);
    assert!(
        matches!(hit.ui[..], [Command::PadPress { pad: 18, velocity: v }]
        if v == f32::from(velocity) / 127.0)
    );
    assert_eq!(drive.app.chop.as_ref().unwrap().selected, 3);
    append_stereo(
        &mut wave,
        drive.render_blocks((PARITY_RELEASE - PARITY_HIT) / RENDER_BLOCK),
    );
    drive.tap(KeyCode::PageDown, M::NONE);
    drive.tap(KeyCode::F(6), M::NONE);
    drive.tap(KeyCode::F(10), M::NONE);
    let frame = match release {
        RecordedRelease::Off => drive.pad_release(source, 2),
        RecordedRelease::Zero => {
            assert!(matches!(source, Source::Notes));
            drive.bytes(PAD_PORT, &[0x90, 38, 0]);
            drive.frame()
        }
        RecordedRelease::Disconnect => {
            assert!(!matches!(source, Source::Keyboard));
            drive.midi().recorded_disconnect(PAD_PORT);
            drive.frame()
        }
    };
    assert_only_release(&frame, 18);
    assert!(!drive.app.pads.held[18]);
    assert!(!drive.app.pads.held[34]);
    assert!(drive.app.screen_learn.is_some());
    append_stereo(
        &mut wave,
        drive.render_blocks((PARITY_END - PARITY_RELEASE) / RENDER_BLOCK),
    );
    check_parity_wave(&wave);

    let mut engine = engine::Engine::new(RENDER_RATE);
    for setting in settings {
        engine.handle(0, setting);
    }
    let expected = engine::offline::render(
        &mut engine,
        &engine::offline::Scenario {
            sample_rate: RENDER_RATE,
            frames: PARITY_END,
            events: vec![
                engine::offline::Event {
                    frame: PARITY_HIT,
                    cmd: Command::PadPress {
                        pad: 18,
                        velocity: f32::from(velocity) / 127.0,
                    },
                },
                engine::offline::Event {
                    frame: PARITY_RELEASE,
                    cmd: Command::PadRelease { pad: 18 },
                },
            ],
        },
        RENDER_BLOCK,
    );
    assert_eq!(
        wave, expected,
        "one actual pad hit with all settings, {source:?}, {release:?}"
    );
    wave
}

/// Scenarios "Chop parity with linked pads", "Chop MIDI linked pad keeps velocity",
/// "Chop linked gate release keeps original pad", "Held pad release passes screen learn".
#[test]
fn render_84_input_parity() {
    use engine::checks::peak;
    let mut all = (Vec::new(), Vec::new());
    let keys = render_chop_route(Source::Keyboard, 127, 12, true, RecordedRelease::Off);
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let wave = render_chop_route(source, 127, 12, true, RecordedRelease::Off);
        assert_eq!(wave, keys, "matched full velocity: {source:?}");
        append_stereo(&mut all, wave);
    }
    let plain = render_chop_route(Source::Keyboard, 80, 12, true, RecordedRelease::Off);
    for source in [Source::Notes, Source::Button] {
        assert_eq!(
            render_chop_route(source, 80, 12, true, RecordedRelease::Off),
            plain
        );
        let low = render_chop_route(source, 23, 12, true, RecordedRelease::Off);
        let high = render_chop_route(source, 101, 12, true, RecordedRelease::Off);
        let ratio = peak(&low.0) / peak(&high.0);
        assert!(
            (ratio - 23.0 / 101.0).abs() < 1e-5,
            "velocity ratio {ratio}"
        );
        println!(
            "84 {source:?}: low peak {:.6}, high peak {:.6}, ratio {ratio:.6}",
            peak(&low.0),
            peak(&high.0)
        );
        append_stereo(&mut all, low);
        append_stereo(&mut all, high);
    }
    // Sensitivity controls: equality must not pass with ignored pitch or filter.
    let no_pitch = render_chop_route(Source::Notes, 127, 0, true, RecordedRelease::Zero);
    let no_filter = render_chop_route(Source::Notes, 127, 12, false, RecordedRelease::Off);
    let difference = |a: &[f32], b: &[f32]| {
        a.iter()
            .zip(b)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max)
    };
    assert!(difference(&keys.0, &no_pitch.0) > 0.05);
    assert!(difference(&keys.0, &no_filter.0) > 0.02);
    for source in [Source::Notes, Source::Button] {
        assert_eq!(
            render_chop_route(source, 127, 12, true, RecordedRelease::Disconnect),
            keys
        );
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/84-input-parity.wav");
    engine::offline::write_wav(&path, RENDER_RATE, &all.0, &all.1).unwrap();
    println!(
        "84 render: {} frames, peak {:.6}, matched routes sample-identical, onset {} (+/-1), finite/no-clip/tail-silence checked",
        all.0.len(),
        peak(&all.0).max(peak(&all.1)),
        PARITY_HIT
    );
}

#[derive(Clone, Copy, Debug)]
enum MixedHoldGuard {
    Chop,
    Fx,
    Question,
    Learn,
}

fn render_mixed_chop_holds(
    source: Source,
    midi_first: bool,
    keyboard_release_first: bool,
    guard: MixedHoldGuard,
    reject_midi: bool,
) -> Stereo {
    use engine::checks::{first_loud, first_non_finite, peak};
    assert!(!matches!(source, Source::Keyboard));
    assert!(!reject_midi || !midi_first);
    let mut drive = Drive::offline();
    let mut settings = drive.parity_pad(0, true);
    let snap = drive.app.pads.snapshot(18);
    let other = drive.app.pads.put_back(34, &snap);
    for command in &other {
        drive.app.send(command.clone());
    }
    settings.extend(other);
    drive.frame();
    let mut wave = drive.render_blocks(32);
    let first_source = if midi_first { source } else { Source::Keyboard };
    let first = drive.pad_press(first_source, 2);
    assert!(first.direct.is_empty());
    assert!(matches!(
        first.ui[..],
        [Command::PadPress {
            pad: 18,
            velocity: 1.0
        }]
    ));
    append_stereo(&mut wave, drive.render_blocks(32));
    drive.tap(KeyCode::PageDown, M::NONE);
    assert_eq!(drive.app.pads.bank, 2);
    if reject_midi {
        let mut filled = 0;
        while drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch: 0,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .is_ok()
        {
            filled += 1;
        }
        assert_eq!(filled, 1024, "reject on the actual callback queue");
    }
    let second_source = if midi_first { Source::Keyboard } else { source };
    let second = drive.pad_press(second_source, 2);
    assert!(second.direct.is_empty());
    assert!(matches!(
        second.ui[..],
        [Command::PadPress {
            pad: 34,
            velocity: 1.0
        }]
    ));
    assert_eq!(drive.app.audio_held[18], Some(0));
    assert_eq!(drive.app.audio_held[34], (!reject_midi).then_some(0));
    if reject_midi {
        assert!(drive.app.midi_ui_held.is_empty());
    } else {
        assert_eq!(drive.app.midi_ui_held.len(), 1);
    }
    append_stereo(&mut wave, drive.render_blocks(32));
    match guard {
        MixedHoldGuard::Chop => {}
        MixedHoldGuard::Fx => {
            drive.tap(KeyCode::F(4), M::NONE);
        }
        MixedHoldGuard::Question => {
            drive.app.ask = Some(Ask::Plan { lines: Vec::new() });
            drive.frame();
        }
        MixedHoldGuard::Learn => {
            drive.tap(KeyCode::F(10), M::NONE);
        }
    }
    let (keyboard_pad, midi_pad) = if midi_first { (34, 18) } else { (18, 34) };
    for release_keyboard in [keyboard_release_first, !keyboard_release_first] {
        let release_source = if release_keyboard {
            Source::Keyboard
        } else {
            source
        };
        let release = drive.pad_release(release_source, 2);
        if reject_midi && !release_keyboard {
            assert!(
                release.commands().is_empty(),
                "a rejected hit has no audio release"
            );
        } else {
            let original = if release_keyboard {
                keyboard_pad
            } else {
                midi_pad
            };
            assert_only_release(&release, original);
            assert_eq!(drive.app.audio_held[original], None);
            assert!(!drive.app.pads.held[original]);
        }
        append_stereo(&mut wave, drive.render_blocks(64));
    }
    assert!(drive.app.midi_ui_held.is_empty());
    assert_eq!(drive.app.audio_held[18], None);
    assert_eq!(drive.app.audio_held[34], None);
    assert_eq!(
        drive.app.ask.is_some(),
        matches!(guard, MixedHoldGuard::Question)
    );
    assert_eq!(
        drive.app.screen_learn.is_some(),
        matches!(guard, MixedHoldGuard::Learn)
    );

    let hit_at = 32 * RENDER_BLOCK;
    let second_at = 64 * RENDER_BLOCK;
    let release_at = 96 * RENDER_BLOCK;
    let last_release_at = 160 * RENDER_BLOCK;
    let mut expected_events = vec![engine::offline::Event {
        frame: hit_at,
        cmd: Command::PadPress {
            pad: 18,
            velocity: 1.0,
        },
    }];
    if !reject_midi {
        expected_events.push(engine::offline::Event {
            frame: second_at,
            cmd: Command::PadPress {
                pad: 34,
                velocity: 1.0,
            },
        });
    }
    for (frame, release_keyboard) in [
        (release_at, keyboard_release_first),
        (last_release_at, !keyboard_release_first),
    ] {
        if !reject_midi || release_keyboard {
            expected_events.push(engine::offline::Event {
                frame,
                cmd: Command::PadRelease {
                    pad: if release_keyboard {
                        keyboard_pad
                    } else {
                        midi_pad
                    },
                },
            });
        }
    }
    let mut expected_engine = engine::Engine::new(RENDER_RATE);
    for command in settings {
        expected_engine.handle(0, command);
    }
    let expected = engine::offline::render(
        &mut expected_engine,
        &engine::offline::Scenario {
            sample_rate: RENDER_RATE,
            frames: wave.0.len(),
            events: expected_events,
        },
        RENDER_BLOCK,
    );
    assert_eq!(
        wave, expected,
        "{source:?}, {guard:?}, MIDI first {midi_first}, keyboard release first {keyboard_release_first}, rejected {reject_midi}"
    );
    for channel in [&wave.0, &wave.1] {
        assert_eq!(first_non_finite(channel), None);
        assert!(peak(channel) < 1.0);
        assert_eq!(peak(&channel[..hit_at]), 0.0);
        assert!(
            first_loud(channel, hit_at..hit_at + RENDER_BLOCK, 1e-6)
                .unwrap()
                .abs_diff(hit_at)
                <= 1
        );
        let sounding = peak(&channel[release_at + 2048..last_release_at]);
        if reject_midi && keyboard_release_first {
            assert_eq!(
                sounding, 0.0,
                "rejected MIDI must not start when space returns"
            );
        } else {
            assert!(
                sounding > 0.01,
                "the other accepted hold must keep sounding"
            );
        }
        assert_eq!(
            peak(&channel[last_release_at + 2048..]),
            0.0,
            "both gates release"
        );
    }
    wave
}

/// Scenario "Chop MIDI hit keeps held keyboard release target".
#[test]
fn chop_midi_hit_keeps_held_keyboard_release_target() {
    for source in [Source::Notes, Source::Button] {
        for guard in [
            MixedHoldGuard::Chop,
            MixedHoldGuard::Fx,
            MixedHoldGuard::Question,
            MixedHoldGuard::Learn,
        ] {
            for keyboard_first in [true, false] {
                render_mixed_chop_holds(source, false, keyboard_first, guard, false);
            }
        }
    }
}

/// Scenario "Chop rejected MIDI hit keeps held keyboard release target".
#[test]
fn chop_rejected_midi_hit_keeps_held_keyboard_release_target() {
    for source in [Source::Notes, Source::Button] {
        for guard in [
            MixedHoldGuard::Chop,
            MixedHoldGuard::Fx,
            MixedHoldGuard::Question,
            MixedHoldGuard::Learn,
        ] {
            for keyboard_first in [true, false] {
                render_mixed_chop_holds(source, false, keyboard_first, guard, true);
            }
        }
    }
}

/// Scenario "Chop keyboard hit keeps held MIDI release target".
#[test]
fn chop_keyboard_hit_keeps_held_midi_release_target() {
    for source in [Source::Notes, Source::Button] {
        for guard in [
            MixedHoldGuard::Chop,
            MixedHoldGuard::Fx,
            MixedHoldGuard::Question,
            MixedHoldGuard::Learn,
        ] {
            for keyboard_first in [true, false] {
                render_mixed_chop_holds(source, true, keyboard_first, guard, false);
            }
        }
    }
}

#[test]
fn render_85_mixed_holds() {
    let mut wave = (Vec::new(), Vec::new());
    for source in [Source::Notes, Source::Button] {
        for (midi_first, reject_midi) in [(false, false), (true, false), (false, true)] {
            append_stereo(
                &mut wave,
                render_mixed_chop_holds(
                    source,
                    midi_first,
                    true,
                    MixedHoldGuard::Chop,
                    reject_midi,
                ),
            );
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/85-mixed-holds.wav");
    engine::offline::write_wav(&path, RENDER_RATE, &wave.0, &wave.1).unwrap();
    println!(
        "85 mixed holds: {} frames, finite/no-clip/silence/onset/independent-release checked, sample-identical to timestamped oracle",
        wave.0.len()
    );
}

#[test]
fn offline_chop_to_fx_keeps_routes_velocity_banks_and_original_releases() {
    use engine::checks::{first_loud, first_non_finite, peak};
    let mut expected = None;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(12, true);
        let snap = drive.app.pads.snapshot(18);
        for command in drive.app.pads.put_back(34, &snap) {
            drive.app.send(command);
        }
        drive.frame();
        drive.render_blocks(32);
        let hit = drive.velocity_hit(source, 80);
        assert!(
            matches!(hit.ui[..], [Command::PadPress { pad: 18, velocity }]
            if velocity == 80.0 / 127.0)
        );
        let mut wave = drive.render_blocks(8);
        assert!(peak(&wave.0) > 0.01);
        assert!(first_loud(&wave.0, 0..RENDER_BLOCK, 1e-6).unwrap() <= 1);
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::F(4), M::NONE);
        assert_eq!(drive.app.pads.bank, 2);
        assert_eq!(drive.app.screen, Screen::Fx);
        assert_only_release(&drive.pad_release(source, 2), 18);
        let tail = drive.render_blocks(64);
        assert_eq!(peak(&tail.0[2048..]), 0.0);
        append_stereo(&mut wave, tail);

        // No frame handles this hit before the next callback. Keyboard writes
        // through App immediately; normal MIDI must use its real direct queue.
        match source {
            Source::Keyboard => drive.key(KeyCode::Char('3'), M::NONE, KeyEventKind::Press),
            Source::Notes => drive.bytes(PAD_PORT, &[0x90, 38, 80]),
            Source::Button => drive.bytes(PAD_PORT, &[0xb0, 80, 80]),
        }
        drive.midi().recorded_fence();
        let direct = drive.render_blocks(1);
        assert!(peak(&direct.0) > 0.01, "{source:?}: next block before UI");
        assert!(first_loud(&direct.0, 0..RENDER_BLOCK, 1e-6).unwrap() <= 1);
        append_stereo(&mut wave, direct);
        let notification = drive.frame();
        if matches!(source, Source::Keyboard) {
            assert!(
                matches!(notification.ui[..], [Command::PadPress { pad: 34, velocity }]
                if velocity == 80.0 / 127.0)
            );
        } else {
            assert!(notification.ui.is_empty(), "no second FX hit");
        }
        assert_eq!(drive.app.pads.selected_abs(), 34);
        assert!(!drive.app.pads.held[18]);
        assert!(drive.app.pads.held[34]);
        let release = drive.pad_release(source, 2);
        if matches!(source, Source::Keyboard) {
            assert_only_release(&release, 34);
        } else {
            assert!(release.ui.is_empty(), "no second direct release");
        }
        assert!(!drive.app.pads.held[34]);
        let tail = drive.render_blocks(64);
        assert_eq!(peak(&tail.0[2048..]), 0.0, "FX gate releases its own pad");
        append_stereo(&mut wave, tail);
        for channel in [&wave.0, &wave.1] {
            assert_eq!(first_non_finite(channel), None);
            assert!(peak(channel) < 1.0);
        }
        if let Some(expected) = &expected {
            assert_eq!(
                &wave, expected,
                "{source:?}: same sound across the route change"
            );
        } else {
            expected = Some(wave);
        }
    }
}

/// Scenario "Take a file and hit a target in one update".
#[test]
fn take_a_file_and_hit_a_target_in_one_update() {
    use engine::checks::peak;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        let path = drive.ordered_file();
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        let before = drive.app.pads.snapshot(18);
        match source {
            Source::Keyboard => {
                drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Press);
                drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Release);
                drive.key(KeyCode::Char('3'), M::NONE, KeyEventKind::Press);
                drive.key(KeyCode::Char('3'), M::NONE, KeyEventKind::Release);
            }
            Source::Notes | Source::Button => {
                drive.bytes(PAD_PORT, &[0xb0, 82, 127]);
                drive.bytes(
                    PAD_PORT,
                    if matches!(source, Source::Notes) {
                        &[0x90, 38, 80]
                    } else {
                        &[0xb0, 80, 80]
                    },
                );
                drive.midi().recorded_fence();
            }
        }
        let before_ui = drive.render_blocks(2);
        assert_eq!(
            peak(&before_ui.0),
            0.0,
            "{source:?}: target must not play old sound before UI"
        );
        let hit = drive.frame();
        assert_silent_target(&hit);
        assert!(
            matches!(drive.app.ask, Some(Ask::PutFile { abs: 18, .. })),
            "{source:?}: bank {}, selected {}, hand {:?}, question {:?}, ui {:?}",
            drive.app.pads.bank,
            drive.app.pads.selected,
            drive.app.browser.in_hand,
            drive.app.ask.as_ref().map(|ask| ask.lines()),
            hit.ui
        );
        assert_eq!(drive.app.pads.snapshot(18), before);
        assert_eq!(drive.app.browser.in_hand.as_ref(), Some(&path));
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.ask.is_none());
        assert_eq!(drive.app.pads.snapshot(18), before);
        assert_eq!(drive.app.browser.in_hand.as_ref(), Some(&path));
        let after = drive.render_blocks(8);
        assert_eq!(peak(&after.0), 0.0);
        assert!(drive.app.midi_ui_held.is_empty());
    }
}

fn render_mode_direct(source: Source, velocity: u8, edit_marking: bool) -> Stereo {
    const HIT: usize = 32 * RENDER_BLOCK;
    const OFF: usize = 34 * RENDER_BLOCK;
    const END: usize = 66 * RENDER_BLOCK;
    let mut drive = Drive::offline();
    let settings = drive.parity_pad(0, false);
    drive.tap(KeyCode::F(2), M::NONE);
    if edit_marking {
        drive.tap(KeyCode::Char('m'), M::NONE);
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.edit && drive.app.pads.marking);
    }
    if matches!(source, Source::Button) {
        drive.bind_pad_button(2);
    }
    let mut wave = drive.render_blocks(32);
    let notifications = if matches!(source, Source::Keyboard) {
        let hit = drive.velocity_hit(source, velocity);
        assert!(
            matches!(hit.ui[..], [Command::PadPress { pad: 18, velocity: v }] if v == f32::from(velocity)/127.0)
        );
        Vec::new()
    } else {
        drive.raw_ordered_hit(source, velocity);
        let events = drive.queued_events();
        assert!(events.iter().any(|e| matches!(e.event,
            midi::Event::PadHit { abs: 18, sound: midi::PadSound::DirectAccepted, velocity: v, .. }
            if v == f32::from(velocity)/127.0)));
        assert!(events.iter().all(|e| e.sequence().is_none()));
        events
    };
    let before_ui = drive.render_blocks(2);
    assert!(
        engine::checks::peak(&before_ui.0) > 0.005,
        "direct hit must not wait for App"
    );
    append_stereo(&mut wave, before_ui);
    for event in notifications {
        drive.app.midi_ui_event(event);
    }
    assert!(
        drive.frame().ui.is_empty(),
        "direct notification cannot start another sound"
    );
    if matches!(source, Source::Keyboard) {
        assert_only_release(&drive.pad_release(source, 2), 18);
    } else {
        drive.raw_ordered_release(source, RecordedRelease::Off);
        assert!(drive.frame().ui.is_empty());
    }
    append_stereo(&mut wave, drive.render_blocks(32));
    let mut engine = engine::Engine::new(RENDER_RATE);
    for cmd in settings {
        engine.handle(0, cmd);
    }
    let expected = engine::offline::render(
        &mut engine,
        &engine::offline::Scenario {
            sample_rate: RENDER_RATE,
            frames: END,
            events: vec![
                engine::offline::Event {
                    frame: HIT,
                    cmd: Command::PadPress {
                        pad: 18,
                        velocity: f32::from(velocity) / 127.0,
                    },
                },
                engine::offline::Event {
                    frame: OFF,
                    cmd: Command::PadRelease { pad: 18 },
                },
            ],
        },
        RENDER_BLOCK,
    );
    assert_eq!(
        wave, expected,
        "one nearest-block direct hit, original release and velocity"
    );
    for channel in [&wave.0, &wave.1] {
        assert_eq!(engine::checks::first_non_finite(channel), None);
        assert!(engine::checks::peak(channel) < 1.0);
        assert_eq!(engine::checks::peak(&channel[..HIT]), 0.0);
        assert!(
            engine::checks::first_loud(channel, HIT..HIT + RENDER_BLOCK, 1e-6)
                .unwrap()
                .abs_diff(HIT)
                <= 1
        );
        assert_eq!(engine::checks::peak(&channel[OFF + 2048..]), 0.0);
    }
    wave
}

fn render_mode_target(source: Source) -> Stereo {
    let mut drive = Drive::offline();
    drive.parity_pad(0, false);
    drive.ordered_file();
    if matches!(source, Source::Button) {
        drive.bind_pad_button(2);
    }
    let before = drive.app.pads.snapshot(18);
    let mut wave = drive.render_blocks(32);
    drive.bytes(PAD_PORT, &[0xb0, 82, 127]);
    drive.raw_ordered_hit(source, 80);
    drive.midi().recorded_fence();
    append_stereo(&mut wave, drive.render_blocks(2));
    assert_silent_target(&drive.frame());
    assert!(matches!(drive.app.ask, Some(Ask::PutFile { abs: 18, .. })));
    assert_eq!(drive.app.pads.snapshot(18), before);
    append_stereo(&mut wave, drive.render_blocks(32));
    for channel in [&wave.0, &wave.1] {
        assert_eq!(engine::checks::first_non_finite(channel), None);
        assert_eq!(
            engine::checks::peak(channel),
            0.0,
            "silent take-file target"
        );
    }
    wave
}

#[test]
fn render_85_input_modes() {
    let keyboard = render_mode_direct(Source::Keyboard, 80, false);
    let mut wave = keyboard.clone();
    for source in [Source::Notes, Source::Button] {
        append_stereo(&mut wave, render_mode_target(source));
        let direct = render_mode_direct(source, 80, true);
        assert_eq!(
            direct, keyboard,
            "matching velocity across keyboard, notes and learned button"
        );
        append_stereo(&mut wave, direct);
        append_stereo(
            &mut wave,
            render_ordered_gate(source, 80, RecordedRelease::Off, false, MixedHoldGuard::Fx),
        );
    }
    for source in [Source::Notes, Source::Button] {
        for velocity in [23, 101] {
            append_stereo(&mut wave, render_mode_direct(source, velocity, false));
        }
    }
    assert_eq!(wave.0.len(), 99_072);
    assert_eq!(wave.1.len(), wave.0.len());
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/85-input-modes.wav");
    engine::offline::write_wav(&path, RENDER_RATE, &wave.0, &wave.1).unwrap();
    println!(
        "85 input modes: 99072 frames, 2.064 s; silent targets, one direct/deferred onset, original releases, finite/no-clip and sample-identical timestamped oracles checked"
    );
}

/// Scenario "Hit from a controller": unchanged direct path, also with edit and marks.
#[test]
fn hit_from_a_controller() {
    for source in [Source::Notes, Source::Button] {
        for velocity in [23, 101] {
            for edit_marking in [false, true] {
                render_mode_direct(source, velocity, edit_marking);
            }
        }
    }
}

/// Scenario "Direct MIDI resumes after pending input".
#[test]
fn direct_midi_resumes_after_pending_input() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.tap(KeyCode::Char('1'), M::CONTROL);
        drive.bind_action(84, "bank.next");
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
        drive.raw_ordered_hit(source, 101);
        let mut events = drive.queued_events().into_iter();
        for event in events.by_ref() {
            let command = matches!(event.event, midi::Event::Action(Action::BankNext));
            drive.app.midi_ui_event(event);
            if command {
                break;
            }
        }
        // Retiring the command is not enough: the earlier UI hit is still pending.
        drive.raw_ordered_hit(source, 23);
        let following = drive.queued_events();
        assert!(following.iter().any(|event| matches!(
            event.event,
            midi::Event::PadHit {
                sound: midi::PadSound::UiUnattempted { deferred: true },
                ..
            }
        )));
        assert_eq!(engine::checks::peak(&drive.render_blocks(2).0), 0.0);
        for event in events.chain(following) {
            drive.app.midi_ui_event(event);
        }
        let ui = drive.frame();
        assert_eq!(
            ui.ui
                .iter()
                .filter(|c| matches!(c, Command::PadPress { pad: 18, .. }))
                .count(),
            2
        );
        assert!(!drive.app.midi_ui_held.is_empty());
        assert!(
            drive
                .app
                .midi_ui_held
                .values()
                .all(|pads| pads.iter().all(|hold| hold.abs == 18 && hold.deferred))
        );
        drive.render_blocks(4);
        drive.raw_ordered_hit(source, 101);
        let direct = drive.queued_events();
        assert!(direct.iter().any(|event| matches!(
            event.event,
            midi::Event::PadHit {
                sound: midi::PadSound::DirectAccepted,
                abs: 18,
                ..
            }
        )));
        // Stable direct sound arrives without a further UI update.
        assert!(engine::checks::peak(&drive.render_blocks(2).0) > 0.01);
        for event in direct {
            drive.app.midi_ui_event(event);
        }
        assert!(drive.frame().ui.is_empty());
        drive.raw_ordered_release(source, RecordedRelease::Off);
        let releases = drive.queued_events();
        assert_eq!(
            releases
                .iter()
                .filter(|e| matches!(
                    e.event,
                    midi::Event::PadRelease {
                        abs: 18,
                        direct: true
                    }
                ))
                .count(),
            1
        );
        for event in releases {
            drive.app.midi_ui_event(event);
        }
        let release = drive.frame();
        assert!(
            matches!(release.ui[..], [Command::PadRelease { pad: 18 }]),
            "accepted deferred hold retains one UI release, separate from the direct hold"
        );
        assert!(drive.app.midi_ui_held.is_empty());
    }
}

/// Scenario "Pending pad uses preceding commands in order".
#[test]
fn pending_pad_uses_preceding_commands_in_order() {
    for source in [Source::Notes, Source::Button] {
        for velocity in [23, 101] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            let pad = drive.app.pads.snapshot(18);
            for cmd in drive.app.pads.put_back(34, &pad) {
                drive.app.send(cmd);
            }
            drive.tap(KeyCode::F(2), M::NONE);
            drive.tap(KeyCode::Char('1'), M::CONTROL);
            drive.bind_action(84, "bank.next");
            if matches!(source, Source::Button) {
                drive.bind_pad_button(2);
            }
            drive.render_blocks(32);
            for _ in 0..2 {
                drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
                drive.raw_ordered_hit(source, velocity);
            }
            drive.midi().recorded_fence();
            assert_eq!(engine::checks::peak(&drive.render_blocks(2).0), 0.0);
            let hit = drive.frame();
            assert!(hit.direct.is_empty());
            let pads: Vec<_> = hit
                .ui
                .iter()
                .filter_map(|c| match c {
                    Command::PadPress { pad, velocity: v } => {
                        assert_eq!(*v, f32::from(velocity) / 127.0);
                        Some(*pad)
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(pads, [18, 34]);
            assert_eq!(
                drive.app.midi_ui_held.len(),
                1,
                "same unreleased control, stable candidate"
            );
            assert_eq!(
                drive
                    .app
                    .midi_ui_held
                    .values()
                    .next()
                    .unwrap()
                    .iter()
                    .map(|hold| hold.abs)
                    .collect::<Vec<_>>(),
                [18, 34]
            );
            drive.raw_ordered_release(source, RecordedRelease::Off);
            let off = drive.frame();
            assert!(off.direct.is_empty());
            assert!(matches!(
                off.ui[..],
                [
                    Command::PadRelease { pad: 18 },
                    Command::PadRelease { pad: 34 }
                ]
            ));
            assert!(drive.app.midi_ui_held.is_empty());
            let stopped = drive.render_blocks(32);
            assert_eq!(engine::checks::peak(&stopped.0[2048..]), 0.0);
        }
    }
    // A blocked/no-op command and F10 transitions must retire too.
    let mut drive = Drive::new();
    drive.bind_action(85, "ui.back");
    drive.bind_action(86, "learn.toggle");
    drive.bind_action(87, "screen.chop");
    drive.bytes(PAD_PORT, &[0xb0, 85, 127]);
    drive.bytes(PAD_PORT, &[0xb0, 87, 127]); // no file: remains Pads
    drive.raw_ordered_hit(Source::Notes, 23);
    assert!(matches!(
        drive.frame().ui[..],
        [Command::PadPress { pad: 2, .. }]
    ));
    drive.bytes(PAD_PORT, &[0xb0, 86, 127]);
    drive.raw_ordered_hit(Source::Notes, 101);
    assert_silent_target(&drive.frame());
    assert!(drive.app.screen_learn.is_some());
    drive.tap(KeyCode::F(10), M::NONE);
    drive.raw_ordered_hit(Source::Notes, 101);
    assert_eq!(
        drive.frame().direct.len(),
        1,
        "normal direct route resumes after the F10 guard"
    );
}

/// Scenario "A target question blocks a following pending hit".
#[test]
fn a_target_question_blocks_a_following_pending_hit() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        let pad = drive.app.pads.snapshot(18);
        for cmd in drive.app.pads.put_back(19, &pad) {
            drive.app.send(cmd);
        }
        let path = drive.ordered_file();
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
            drive.bind_action(81, "pad.4");
        }
        drive.render_blocks(32);
        let before = [drive.app.pads.snapshot(18), drive.app.pads.snapshot(19)];
        if matches!(source, Source::Keyboard) {
            for key in [KeyCode::Enter, KeyCode::Char('3'), KeyCode::Char('4')] {
                drive.key(key, M::NONE, KeyEventKind::Press);
                drive.key(key, M::NONE, KeyEventKind::Release);
            }
        } else {
            drive.bytes(PAD_PORT, &[0xb0, 82, 127]);
            drive.raw_ordered_hit(source, 23);
            drive.bytes(
                PAD_PORT,
                if matches!(source, Source::Notes) {
                    &[0x90, 39, 101]
                } else {
                    &[0xb0, 81, 101]
                },
            );
            let mut events = drive.queued_events().into_iter();
            for event in events.by_ref() {
                let take = matches!(event.event, midi::Event::Action(Action::BrowserHold));
                drive.app.midi_ui_event(event);
                if take {
                    break;
                }
            }
            drive.raw_ordered_hit(source, 101);
            let extra = drive.queued_events();
            assert!(extra.iter().any(|event| matches!(
                event.event,
                midi::Event::PadHit {
                    sound: midi::PadSound::UiUnattempted { deferred: true },
                    ..
                }
            )));
            for event in events.chain(extra) {
                drive.app.midi_ui_event(event);
            }
        }
        assert_silent_target(&drive.frame());
        assert!(matches!(drive.app.ask, Some(Ask::PutFile { abs: 18, .. })));
        assert_eq!(drive.app.pads.selected_abs(), 18);
        assert_eq!(
            [drive.app.pads.snapshot(18), drive.app.pads.snapshot(19)],
            before
        );
        assert_eq!(drive.app.browser.in_hand.as_ref(), Some(&path));
        assert!(drive.app.midi_ui_held.is_empty());
        drive.tap(KeyCode::Esc, M::NONE);
        assert_eq!(
            [drive.app.pads.snapshot(18), drive.app.pads.snapshot(19)],
            before
        );
        assert_eq!(drive.app.browser.in_hand.as_ref(), Some(&path));
        let wave = drive.render_blocks(32);
        assert_eq!(engine::checks::peak(&wave.0), 0.0);
        assert_eq!(engine::checks::first_non_finite(&wave.0), None);
    }
}

/// Scenario "Pending command order spans controllers".
#[test]
fn pending_command_order_spans_controllers() {
    const OTHER: &str = "Recorded pads B";
    let mut drive = Drive::offline();
    drive.midi().recorded_connect(OTHER);
    drive.frame();
    drive.bind_action(84, "bank.next");
    drive.bytes(PAD_PORT, &[0x90, 37, 101]);
    drive.bytes(OTHER, &[0x92, 38, 23]);
    let initial = drive.queued_events();
    assert_eq!(
        initial
            .iter()
            .filter(|e| matches!(
                e.event,
                midi::Event::PadHit {
                    sound: midi::PadSound::DirectAccepted,
                    ..
                }
            ))
            .count(),
        2
    );
    for event in initial {
        drive.app.midi_ui_event(event);
    }
    drive.frame();
    drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
    drive.bytes(OTHER, &[0x92, 38, 101]);
    drive.bytes(PAD_PORT, &[0x80, 37, 0]);
    drive.bytes(OTHER, &[0x82, 38, 0]);
    let events = drive.queued_events();
    let direct: Vec<_> = events
        .iter()
        .filter_map(|e| match e.event {
            midi::Event::PadRelease { abs, direct: true } => Some(abs),
            _ => None,
        })
        .collect();
    assert_eq!(direct, [1, 2]);
    for event in events {
        drive.app.midi_ui_event(event);
    }
    let frame = drive.frame();
    assert!(
        matches!(frame.ui[..], [Command::PadPress { pad: 18, velocity }, Command::PadRelease { pad: 18 }]
        if velocity == 101.0/127.0),
        "cross-port UI: {:?}, bank {}",
        frame.ui,
        drive.app.pads.bank
    );
    assert!(drive.app.midi_ui_held.is_empty());
}

/// Scenario "Pending command order spans controllers": take from A, target from B,
/// and original direct releases from both ports before any UI consumption.
#[test]
fn pending_file_target_spans_controllers_and_keeps_direct_releases() {
    const OTHER: &str = "Recorded pads B";
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        let pad = drive.app.pads.snapshot(18);
        for abs in [17, 19] {
            for cmd in drive.app.pads.put_back(abs, &pad) {
                drive.app.send(cmd);
            }
        }
        let path = drive.ordered_file();
        drive.midi().recorded_connect(OTHER);
        drive.frame();
        if matches!(source, Source::Button) {
            drive.bind_pad_button(3);
        }
        drive.render_blocks(32);
        drive.bytes(PAD_PORT, &[0x90, 37, 101]);
        drive.bytes(OTHER, &[0x92, 38, 23]);
        drive.frame();
        assert!(engine::checks::peak(&drive.render_blocks(4).0) > 0.01);
        drive.bytes(PAD_PORT, &[0xb0, 82, 127]);
        drive.bytes(
            OTHER,
            if matches!(source, Source::Notes) {
                &[0x92, 39, 101]
            } else {
                &[0xb2, 80, 101]
            },
        );
        drive.bytes(PAD_PORT, &[0x80, 37, 0]);
        drive.bytes(OTHER, &[0x82, 38, 0]);
        let events = drive.queued_events();
        let releases: Vec<_> = events
            .iter()
            .filter_map(|e| match e.event {
                midi::Event::PadRelease { abs, direct: true } => Some(abs),
                _ => None,
            })
            .collect();
        assert_eq!(releases, [17, 18]);
        // Both original direct gates stop while the command and target still wait.
        assert_eq!(
            engine::checks::peak(&drive.render_blocks(32).0[2048..]),
            0.0
        );
        let before = drive.app.pads.snapshot(19);
        for event in events {
            drive.app.midi_ui_event(event);
        }
        assert_silent_target(&drive.frame());
        assert!(matches!(drive.app.ask, Some(Ask::PutFile { abs: 19, .. })));
        assert_eq!(drive.app.pads.snapshot(19), before);
        assert_eq!(drive.app.browser.in_hand.as_ref(), Some(&path));
        assert!(drive.app.midi_ui_held.is_empty());
        let quiet = drive.render_blocks(32);
        assert_eq!(engine::checks::peak(&quiet.0), 0.0);
        assert_eq!(engine::checks::first_non_finite(&quiet.0), None);
    }
}

/// Scenario "Pending pad uses preceding commands in order": paired keyboard/MIDI
/// bank, cancelled hand and no-op commands leave the same ordinary pad hit.
#[test]
fn pending_cancelled_hand_matches_keyboard_command_order() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.ordered_file();
        drive.tap(KeyCode::Char('1'), M::CONTROL);
        drive.bind_action(84, "bank.next");
        drive.bind_action(85, "ui.back");
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        if matches!(source, Source::Keyboard) {
            for key in [KeyCode::Enter, KeyCode::Esc, KeyCode::PageDown] {
                drive.key(key, M::NONE, KeyEventKind::Press);
                drive.key(key, M::NONE, KeyEventKind::Release);
            }
            drive.key(KeyCode::Char('3'), M::NONE, KeyEventKind::Press);
        } else {
            for cc in [82, 85, 84] {
                drive.bytes(PAD_PORT, &[0xb0, cc, 127]);
            }
            drive.raw_ordered_hit(source, 80);
        }
        let frame = drive.frame();
        assert!(frame.direct.is_empty());
        assert!(
            matches!(frame.ui.iter().filter(|c| matches!(c, Command::PadPress { .. })).collect::<Vec<_>>()[..],
            [Command::PadPress { pad: 18, velocity }] if *velocity == 80.0/127.0)
        );
        assert!(drive.app.browser.in_hand.is_none());
        assert_eq!(drive.app.pads.bank, 1);
        assert!(engine::checks::peak(&drive.render_blocks(4).0) > 0.01);
        if matches!(source, Source::Keyboard) {
            drive.pad_release(source, 2);
        } else {
            drive.raw_ordered_release(source, RecordedRelease::Off);
            drive.frame();
        }
        assert_eq!(
            engine::checks::peak(&drive.render_blocks(32).0[2048..]),
            0.0
        );
    }
}

fn render_ordered_gate(
    source: Source,
    velocity: u8,
    release: RecordedRelease,
    early: bool,
    guard: MixedHoldGuard,
) -> Stereo {
    const HIT: usize = 34 * RENDER_BLOCK;
    const OFF: usize = 50 * RENDER_BLOCK;
    const END: usize = 90 * RENDER_BLOCK;
    let mut drive = Drive::offline();
    let settings = drive.parity_pad(0, false);
    drive.tap(KeyCode::F(2), M::NONE);
    drive.tap(KeyCode::Char('1'), M::CONTROL);
    drive.bind_action(84, "bank.next");
    if matches!(source, Source::Button) {
        drive.bind_pad_button(2);
    }
    let mut wave = drive.render_blocks(32);
    drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
    drive.raw_ordered_hit(source, velocity);
    if early {
        drive.raw_ordered_release(source, release);
    }
    drive.midi().recorded_fence();
    append_stereo(&mut wave, drive.render_blocks(2));
    assert_eq!(engine::checks::peak(&wave.0), 0.0, "no old-bank direct hit");
    let hit = drive.frame();
    assert!(hit.direct.is_empty());
    assert!(
        matches!(hit.ui.first(), Some(Command::PadPress { pad: 18, velocity: v }) if *v == f32::from(velocity)/127.0)
    );
    if early {
        assert!(matches!(
            hit.ui[..],
            [
                Command::PadPress { pad: 18, .. },
                Command::PadRelease { pad: 18 }
            ]
        ));
    } else {
        assert_eq!(drive.app.audio_held[18], Some(0));
        assert_eq!(
            drive
                .app
                .midi_ui_held
                .values()
                .next()
                .unwrap()
                .iter()
                .map(|hold| hold.abs)
                .collect::<Vec<_>>(),
            [18]
        );
    }
    append_stereo(&mut wave, drive.render_blocks(16));
    if !early {
        if !matches!(guard, MixedHoldGuard::Chop) {
            drive.tap(KeyCode::Char('3'), M::CONTROL);
        }
        match guard {
            MixedHoldGuard::Chop => {
                drive.tap(KeyCode::F(3), M::NONE);
            }
            MixedHoldGuard::Fx => {
                drive.tap(KeyCode::F(4), M::NONE);
            }
            MixedHoldGuard::Question => {
                drive.app.ask = Some(Ask::Plan { lines: vec![] });
                drive.frame();
            }
            MixedHoldGuard::Learn => {
                drive.tap(KeyCode::F(10), M::NONE);
            }
        }
        drive.raw_ordered_release(source, release);
        let off = drive.frame();
        assert_only_release(&off, 18);
    }
    assert!(drive.app.midi_ui_held.is_empty());
    append_stereo(&mut wave, drive.render_blocks((END - OFF) / RENDER_BLOCK));
    let mut engine = engine::Engine::new(RENDER_RATE);
    for cmd in settings {
        engine.handle(0, cmd);
    }
    let mut events = vec![engine::offline::Event {
        frame: HIT,
        cmd: Command::PadPress {
            pad: 18,
            velocity: f32::from(velocity) / 127.0,
        },
    }];
    if !early && matches!(guard, MixedHoldGuard::Chop) {
        events.push(engine::offline::Event {
            frame: OFF,
            cmd: Command::HushPads,
        });
    }
    events.push(engine::offline::Event {
        frame: if early { HIT } else { OFF },
        cmd: Command::PadRelease { pad: 18 },
    });
    let expected = engine::offline::render(
        &mut engine,
        &engine::offline::Scenario {
            sample_rate: RENDER_RATE,
            frames: END,
            events,
        },
        RENDER_BLOCK,
    );
    assert_eq!(
        wave, expected,
        "actual target, velocity, onset and release: {source:?}, {velocity}, {release:?}, early {early}, {guard:?}"
    );
    for channel in [&wave.0, &wave.1] {
        assert_eq!(engine::checks::first_non_finite(channel), None);
        assert!(engine::checks::peak(channel) < 1.0);
        assert_eq!(engine::checks::peak(&channel[..HIT]), 0.0);
        assert_eq!(engine::checks::peak(&channel[OFF + 2048..]), 0.0);
        if !early {
            let onset = engine::checks::first_loud(channel, HIT..HIT + RENDER_BLOCK, 1e-6).unwrap();
            assert!(onset.abs_diff(HIT) <= 1);
        }
    }
    wave
}

/// Scenario "Deferred gate receives an early release or disconnect".
#[test]
fn deferred_gate_receives_an_early_release_or_disconnect() {
    for source in [Source::Notes, Source::Button] {
        for velocity in [23, 101] {
            for release in [
                RecordedRelease::Off,
                RecordedRelease::Zero,
                RecordedRelease::Disconnect,
            ] {
                render_ordered_gate(source, velocity, release, true, MixedHoldGuard::Fx);
            }
        }
    }
}

/// Scenario "Deferred release follows its actual pad across modes".
#[test]
fn deferred_release_follows_its_actual_pad_across_modes() {
    for source in [Source::Notes, Source::Button] {
        for velocity in [23, 101] {
            for release in [
                RecordedRelease::Off,
                RecordedRelease::Zero,
                RecordedRelease::Disconnect,
            ] {
                for guard in [
                    MixedHoldGuard::Chop,
                    MixedHoldGuard::Fx,
                    MixedHoldGuard::Question,
                    MixedHoldGuard::Learn,
                ] {
                    render_ordered_gate(source, velocity, release, false, guard);
                }
            }
        }
    }
}

/// Scenario "Deferred silent and rejected hits preserve accepted holds".
#[test]
fn deferred_silent_and_rejected_hits_preserve_accepted_holds() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.tap(KeyCode::Char('1'), M::CONTROL);
        drive.bind_action(84, "bank.next");
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
        drive.raw_ordered_hit(source, 101);
        drive.frame();
        drive.render_blocks(4);
        let held = drive.app.midi_ui_held.clone();
        let mut filled = 0;
        while drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch: 0,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .is_ok()
        {
            filled += 1;
        }
        assert_eq!(filled, 1024);
        drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
        drive.raw_ordered_hit(source, 23);
        drive.frame();
        assert_eq!(
            drive.app.midi_ui_held, held,
            "rejected target must not replace accepted identity"
        );
        assert_eq!(drive.app.audio_held[18], Some(0));
        assert_eq!(drive.app.audio_held[34], None);
        drive.raw_ordered_release(source, RecordedRelease::Off);
        let off = drive.frame();
        assert_only_release(&off, 18);
        assert!(drive.app.midi_ui_held.is_empty());
        assert!(!drive.app.pending_audio.is_empty());
        let continuing = drive.render_blocks(1);
        assert!(
            engine::checks::peak(&continuing.0) > 0.01,
            "accepted gate survives rejected retrigger"
        );
        drive.app.flush_audio();
        let tail = drive.render_blocks(32);
        assert_eq!(engine::checks::first_non_finite(&tail.0), None);
        assert!(engine::checks::peak(&tail.0) < 1.0);
        assert_eq!(
            engine::checks::peak(&tail.0[2048..]),
            0.0,
            "pending release retries"
        );
    }
}

/// Scenario "Deferred gate receives an early release or disconnect": callback
/// already consumed an older direct hold's release before App starts the retrigger.
#[test]
fn early_direct_release_cannot_steal_a_later_deferred_hold() {
    for source in [Source::Notes, Source::Button] {
        for release in [
            RecordedRelease::Off,
            RecordedRelease::Zero,
            RecordedRelease::Disconnect,
        ] {
            let mut drive = Drive::offline();
            let settings = drive.parity_pad(0, false);
            drive.tap(KeyCode::F(2), M::NONE);
            drive.bind_action(85, "ui.back");
            if matches!(source, Source::Button) {
                drive.bind_pad_button(2);
            }
            let mut wave = drive.render_blocks(32);
            drive.raw_ordered_hit(source, 101);
            drive.midi().recorded_fence();
            append_stereo(&mut wave, drive.render_blocks(4));
            drive.frame();
            drive.bytes(PAD_PORT, &[0xb0, 85, 127]);
            drive.raw_ordered_hit(source, 23);
            drive.raw_ordered_release(source, release);
            drive.midi().recorded_fence();
            let old_release = drive.render_blocks(32);
            assert_eq!(engine::checks::peak(&old_release.0[2048..]), 0.0);
            append_stereo(&mut wave, old_release);
            let ui = drive.frame();
            let tail = drive.render_blocks(64);
            assert_eq!(
                engine::checks::peak(&tail.0[2048..]),
                0.0,
                "{source:?}, {release:?}: older direct release cannot retire a later UI gate"
            );
            append_stereo(&mut wave, tail);
            assert!(matches!(
                ui.ui[..],
                [
                    Command::PadPress { pad: 18, .. },
                    Command::PadRelease { pad: 18 }
                ]
            ));
            assert!(drive.app.midi_ui_held.is_empty());
            let mut engine = engine::Engine::new(RENDER_RATE);
            for cmd in settings {
                engine.handle(0, cmd);
            }
            let expected = engine::offline::render(
                &mut engine,
                &engine::offline::Scenario {
                    sample_rate: RENDER_RATE,
                    frames: 132 * RENDER_BLOCK,
                    events: vec![
                        engine::offline::Event {
                            frame: 32 * RENDER_BLOCK,
                            cmd: Command::PadPress {
                                pad: 18,
                                velocity: 101.0 / 127.0,
                            },
                        },
                        engine::offline::Event {
                            frame: 36 * RENDER_BLOCK,
                            cmd: Command::PadRelease { pad: 18 },
                        },
                        engine::offline::Event {
                            frame: 68 * RENDER_BLOCK,
                            cmd: Command::PadPress {
                                pad: 18,
                                velocity: 23.0 / 127.0,
                            },
                        },
                        engine::offline::Event {
                            frame: 68 * RENDER_BLOCK,
                            cmd: Command::PadRelease { pad: 18 },
                        },
                    ],
                },
                RENDER_BLOCK,
            );
            assert_eq!(
                wave, expected,
                "independent callback/UI progress and both accepted holds"
            );
            for channel in [&wave.0, &wave.1] {
                assert_eq!(engine::checks::first_non_finite(channel), None);
                assert!(engine::checks::peak(channel) < 1.0);
            }
        }
    }
}

/// Scenarios "Deferred silent and rejected hits preserve accepted holds" and
/// "Deferred release follows its actual pad across modes": both direct/UI orders.
#[test]
fn deferred_retriggers_keep_both_release_obligations_for_both_press_orders() {
    for source in [Source::Notes, Source::Button] {
        for ui_first in [false, true] {
            for same_pad in [false, true] {
                for disconnect in [false, true] {
                    let mut drive = Drive::offline();
                    drive.parity_pad(0, false);
                    let pad = drive.app.pads.snapshot(18);
                    for cmd in drive.app.pads.put_back(34, &pad) {
                        drive.app.send(cmd);
                    }
                    drive.tap(KeyCode::F(2), M::NONE);
                    drive.bind_action(84, "bank.next");
                    drive.bind_action(85, "ui.back");
                    if matches!(source, Source::Button) {
                        drive.bind_pad_button(2);
                    }
                    drive.render_blocks(32);
                    if ui_first {
                        drive.bytes(PAD_PORT, &[0xb0, 85, 127]);
                    }
                    drive.raw_ordered_hit(source, 101);
                    drive.frame();
                    drive.render_blocks(4);
                    if ui_first {
                        if !same_pad {
                            drive.tap(KeyCode::Char('3'), M::CONTROL);
                        }
                    } else {
                        drive.bytes(PAD_PORT, &[0xb0, if same_pad { 85 } else { 84 }, 127]);
                    }
                    drive.raw_ordered_hit(source, 23);
                    drive.frame();
                    drive.render_blocks(4);
                    let direct_abs = if ui_first && !same_pad { 34 } else { 18 };
                    let ui_abs = if !ui_first && !same_pad { 34 } else { 18 };
                    assert_eq!(
                        drive
                            .app
                            .midi_ui_held
                            .values()
                            .next()
                            .unwrap()
                            .iter()
                            .map(|hold| hold.abs)
                            .collect::<Vec<_>>(),
                        [ui_abs]
                    );
                    drive.raw_ordered_release(
                        source,
                        if disconnect {
                            RecordedRelease::Disconnect
                        } else {
                            RecordedRelease::Off
                        },
                    );
                    let events = drive.queued_events();
                    let direct: Vec<_> = events
                        .iter()
                        .filter_map(|e| match e.event {
                            midi::Event::PadRelease { abs, direct: true } => Some(abs),
                            _ => None,
                        })
                        .collect();
                    assert_eq!(direct, [direct_abs]);
                    for event in events {
                        drive.app.midi_ui_event(event);
                    }
                    let off = drive.frame();
                    assert!(
                        matches!(off.ui[..], [Command::PadRelease { pad }] if pad == ui_abs),
                        "each accepted deferred hold keeps its own release"
                    );
                    assert!(drive.app.midi_ui_held.is_empty());
                    let stopped = drive.render_blocks(32);
                    for channel in [&stopped.0, &stopped.1] {
                        assert_eq!(engine::checks::first_non_finite(channel), None);
                        assert!(engine::checks::peak(channel) < 1.0);
                        assert_eq!(engine::checks::peak(&channel[2048..]), 0.0);
                    }
                }
            }
        }
    }
}

/// Scenario "Deferred silent and rejected hits preserve accepted holds": silent,
/// preview and assignment retriggers, including a different physical control.
#[test]
fn silent_deferred_candidates_do_not_steal_an_accepted_hold() {
    for source in [Source::Notes, Source::Button] {
        for context in ["question", "learn", "preview", "assign"] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            drive.tap(KeyCode::F(2), M::NONE);
            drive.tap(KeyCode::Char('1'), M::CONTROL);
            drive.bind_action(84, "bank.next");
            drive.bind_action(81, "pad.4");
            if matches!(source, Source::Button) {
                drive.bind_pad_button(2);
            }
            drive.render_blocks(32);
            drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
            drive.raw_ordered_hit(source, 101);
            drive.frame();
            drive.render_blocks(4);
            let held = drive.app.midi_ui_held.clone();
            drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
            let command = drive.queued_events();
            match context {
                "question" => {
                    drive.app.pads.marked.insert(18);
                    drive.tap(KeyCode::Backspace, M::NONE);
                    assert!(matches!(drive.app.ask, Some(Ask::ClearPads { .. })));
                }
                "learn" => {
                    drive.tap(KeyCode::F(10), M::NONE);
                }
                "preview" | "assign" => {
                    let selected = drive.app.pads.selected_abs();
                    drive.tap(KeyCode::F(3), M::NONE);
                    assert_eq!(
                        drive.app.screen,
                        Screen::Chop,
                        "{source:?}, {context}, selected {selected}, file {:?}",
                        drive.app.pads.file[selected]
                    );
                    if context == "assign" {
                        drive.tap(KeyCode::Char('p'), M::NONE);
                    } else {
                        // Bare-slice listen requires no linked pads anywhere, not
                        // merely an empty target on the newly selected bank.
                        drive.app.pads.link.fill(None);
                    }
                }
                _ => unreachable!(),
            }
            let learned = drive.app.learned.clone();
            drive.raw_ordered_hit(source, 23);
            drive.bytes(PAD_PORT, &[0xb0, 81, 101]);
            let hits = drive.queued_events();
            for event in command.into_iter().chain(hits) {
                drive.app.midi_ui_event(event);
            }
            assert_eq!(drive.app.midi_ui_held, held, "{source:?}, {context}");
            assert_eq!(
                drive.app.learned, learned,
                "deferred hit is not a learn confirmation"
            );
            drive.bytes(PAD_PORT, &[0xb0, 81, 0]);
            drive.frame();
            assert_eq!(
                drive.app.midi_ui_held, held,
                "another control cannot release the accepted hold"
            );
            drive.raw_ordered_release(source, RecordedRelease::Off);
            assert_only_release(&drive.frame(), 18);
            assert!(drive.app.midi_ui_held.is_empty());
            drive.app.stop_preview();
            let stopped = drive.render_blocks(64);
            assert_eq!(engine::checks::first_non_finite(&stopped.0), None);
            assert!(engine::checks::peak(&stopped.0) < 1.0);
            assert_eq!(engine::checks::peak(&stopped.0[4096..]), 0.0);
        }
    }
}

/// Scenario "Old deferred input cannot affect a new project".
#[test]
fn old_deferred_input_cannot_affect_a_new_project() {
    use std::sync::atomic::Ordering::Acquire;
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.bind_action(84, "bank.next");
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
        drive.raw_ordered_hit(source, 23);
        drive.raw_ordered_release(source, RecordedRelease::Off);
        let old = drive.queued_events();
        assert!(old.iter().any(|e| e.sequence().is_some()));
        drive.app.apply_session(session::Session::default());
        for _ in 0..256 {
            drive.app.flush_audio();
            drive.render_blocks(1);
            if drive.project.ready.load(Acquire) == 1 && drive.app.pending_audio.is_empty() {
                break;
            }
        }
        assert_eq!(drive.project.ready.load(Acquire), 1);
        assert!(drive.app.pending_audio.is_empty());
        let before = drive.app.pads.snapshot(18);
        for event in old {
            drive.app.midi_ui_event(event);
        }
        assert_eq!(drive.app.pads.snapshot(18), before);
        assert!(drive.app.midi_ui_held.is_empty());
        assert!(
            !drive
                .app
                .sent
                .iter()
                .any(|c| matches!(c, Command::PadPress { .. } | Command::PadRelease { .. }))
        );
        drive.frame();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        drive.raw_ordered_hit(source, 101);
        let next = drive.queued_events();
        assert!(
            next.iter().any(|e| matches!(
                e.event,
                midi::Event::PadHit {
                    abs: 18,
                    sound: midi::PadSound::DirectAccepted,
                    ..
                }
            )),
            "{source:?}: stale numbered work retires; focus {:?}, ready {}, epoch {}, events {:?}",
            drive.app.focus(),
            drive.project.ready.load(Acquire),
            drive.project.epoch.load(Acquire),
            next.iter()
                .map(|e| match &e.event {
                    midi::Event::PadHit { abs, sound, .. } => format!(
                        "hit {abs} {sound:?}, ready {}, seq {:?}",
                        e.preview_ready(),
                        e.sequence()
                    ),
                    midi::Event::Message { meaning, .. } => meaning.clone(),
                    _ => "other".into(),
                })
                .collect::<Vec<_>>()
        );
        assert!(engine::checks::peak(&drive.render_blocks(2).0) > 0.01);
        for event in next {
            drive.app.midi_ui_event(event);
        }
        assert!(drive.frame().ui.is_empty());
    }
}

/// Scenario "Take a file and hit a target in one update": empty targets finish loading.
#[test]
fn pending_file_on_an_empty_target_finishes_without_sound() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.app.clear_pads(&[18]);
        let path = drive.ordered_file();
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        if matches!(source, Source::Keyboard) {
            drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Press);
            drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Release);
            drive.key(KeyCode::Char('3'), M::NONE, KeyEventKind::Press);
            drive.key(KeyCode::Char('3'), M::NONE, KeyEventKind::Release);
        } else {
            drive.bytes(PAD_PORT, &[0xb0, 82, 127]);
            drive.raw_ordered_hit(source, 101);
            drive.midi().recorded_fence();
        }
        assert_eq!(engine::checks::peak(&drive.render_blocks(2).0), 0.0);
        assert_silent_target(&drive.frame());
        assert!(drive.app.ask.is_none());
        assert!(drive.app.pads.loading[18].is_some());
        assert!(drive.app.browser.in_hand.is_none());
        while drive.app.pads.loading[18].is_some() {
            let reply = drive
                .app
                .loader
                .as_ref()
                .unwrap()
                .1
                .recv_timeout(std::time::Duration::from_secs(1))
                .unwrap();
            drive.app.loader_reply(reply);
        }
        assert_eq!(drive.app.pads.file[18].as_ref(), Some(&path));
        assert!(drive.app.pads.sound[18].is_some());
        assert_silent_target(&drive.frame());
        assert!(drive.app.midi_ui_held.is_empty());
        let quiet = drive.render_blocks(32);
        for channel in [&quiet.0, &quiet.1] {
            assert_eq!(engine::checks::peak(channel), 0.0);
            assert_eq!(engine::checks::first_non_finite(channel), None);
        }
    }
}

/// Scenario "Pending pad uses preceding commands in order": keep real Chop entry/exit.
#[test]
fn pending_chop_entry_and_exit_keep_fifo_routes() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.app.pads.selected = 2;
        drive.bind_action(87, "screen.chop");
        drive.bind_action(88, "screen.pads");
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        drive.bytes(PAD_PORT, &[0xb0, 87, 127]);
        drive.raw_ordered_hit(source, 23);
        drive.bytes(PAD_PORT, &[0xb0, 88, 127]);
        drive.raw_ordered_hit(source, 101);
        drive.midi().recorded_fence();
        assert_eq!(engine::checks::peak(&drive.render_blocks(2).0), 0.0);
        let frame = drive.frame();
        assert!(frame.direct.is_empty());
        let hits: Vec<_> = frame
            .ui
            .iter()
            .filter_map(|c| match c {
                Command::PadPress { pad, velocity } => Some((*pad, *velocity)),
                _ => None,
            })
            .collect();
        assert_eq!(hits, [(18, 23.0 / 127.0), (18, 101.0 / 127.0)]);
        assert!(frame.ui.iter().any(|c| matches!(c, Command::HushPads)));
        assert_eq!(drive.app.screen, Screen::Pads);
        drive.raw_ordered_release(source, RecordedRelease::Off);
        assert_only_release(&drive.frame(), 18);
        assert!(drive.app.midi_ui_held.is_empty());
        assert_eq!(
            engine::checks::peak(&drive.render_blocks(32).0[2048..]),
            0.0
        );
    }
}

/// Scenario "Pending pad uses preceding commands in order": old Ignore must not drop pending work.
#[test]
fn pending_hit_retires_under_an_old_f10_snapshot() {
    let mut drive = Drive::offline();
    drive.bind_action(84, "bank.next");
    drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
    let command = drive.queued_events();
    drive.key(KeyCode::F(10), M::NONE, KeyEventKind::Press);
    drive.key(KeyCode::F(10), M::NONE, KeyEventKind::Release);
    drive.app.publish_midi(&drive.shared);
    assert_eq!(
        input::route_pad(drive.shared.focus().unwrap()),
        input::PadRoute::Ignore
    );
    drive.raw_ordered_hit(Source::Notes, 101);
    let hit = drive.queued_events();
    assert!(hit.iter().any(|e| matches!(
        e.event,
        midi::Event::PadHit {
            sound: midi::PadSound::UiUnattempted { deferred: true },
            ..
        }
    )));
    for event in command.into_iter().chain(hit) {
        drive.app.midi_ui_event(event);
    }
    assert_silent_target(&drive.frame());
    assert_eq!(
        drive.app.pads.bank, 0,
        "F10 blocks the command body but still retires its sequence"
    );
    assert!(drive.app.midi_ui_held.is_empty());
    drive.tap(KeyCode::F(10), M::NONE);
    drive.raw_ordered_hit(Source::Notes, 101);
    let resumed = drive.queued_events();
    assert!(resumed.iter().any(|e| matches!(
        e.event,
        midi::Event::PadHit {
            sound: midi::PadSound::DirectAccepted,
            ..
        }
    )));
    for event in resumed {
        drive.app.midi_ui_event(event);
    }
    assert!(drive.frame().ui.is_empty());
}

#[derive(Clone, Copy, Debug)]
enum LateChop {
    Linked,
    Bare,
    Assign,
    AssignOrder,
}

fn render_late_direct_notification(
    source: Source,
    rejected: bool,
    mode: LateChop,
    change_bank: bool,
    velocity: u8,
) -> Stereo {
    use engine::checks::{first_loud, first_non_finite, peak};
    let mut drive = Drive::offline();
    let mut settings = drive.parity_pad(0, true);
    // Offline Engine starts with built-in test tones, unlike the empty UI fixture.
    let clear_filler = Command::SetPadSound {
        pad: 31,
        sound: None,
    };
    drive.app.send(clear_filler.clone());
    settings.push(clear_filler);
    drive.tap(KeyCode::F(2), M::NONE);
    if matches!(source, Source::Button) {
        drive.bind_pad_button(2);
    }
    let mut wave = drive.render_blocks(32);
    if rejected {
        // Fill the production MIDI queue, not the UI queue. The filler pad is empty.
        for _ in 0..1024 {
            drive.bytes(PAD_PORT, &[0x91, 51, 127]);
        }
        drive.frame();
    }
    // Select the fixture's loaded pad before the delayed hit. Filler notifications
    // were consumed without a callback, so the real rejection queue is still full.
    drive.app.pads.selected = 2;
    let hit = if matches!(source, Source::Notes) {
        [0x90, 38, velocity]
    } else {
        [0xb0, 80, velocity]
    };
    drive.bytes(PAD_PORT, &hit);
    drive.midi().recorded_fence();
    let delayed: Vec<_> = drive.midi().events.try_iter().collect();
    assert!(delayed.iter().any(|event| matches!(event.event,
        midi::Event::PadHit { abs: 18, sound, velocity: v, ui_press: None }
        if sound == if rejected { midi::PadSound::DirectRejected } else { midi::PadSound::DirectAccepted }
            && v == f32::from(velocity) / 127.0)));
    append_stereo(&mut wave, drive.render_blocks(8));
    assert_eq!(peak(&wave.0[32 * RENDER_BLOCK..]) == 0.0, rejected);
    for kind in [KeyEventKind::Press, KeyEventKind::Release] {
        drive.key(KeyCode::F(3), M::NONE, kind);
    }
    assert_eq!(drive.app.screen, Screen::Chop);
    match mode {
        LateChop::Linked => {}
        LateChop::Bare => drive.app.pads.link[18] = None,
        LateChop::Assign | LateChop::AssignOrder => {
            for kind in [KeyEventKind::Press, KeyEventKind::Release] {
                drive.key(KeyCode::Char('p'), M::NONE, kind);
            }
            assert_eq!(drive.app.plan, Some(chop::Plan::ByHand(Vec::new())));
            if matches!(mode, LateChop::AssignOrder) {
                for kind in [KeyEventKind::Press, KeyEventKind::Release] {
                    drive.key(KeyCode::Tab, M::NONE, kind);
                }
                assert_eq!(drive.app.plan, Some(chop::Plan::InOrder(None)));
            }
        }
    }
    if change_bank {
        for kind in [KeyEventKind::Press, KeyEventKind::Release] {
            drive.key(KeyCode::Char('3'), M::CONTROL, kind);
        }
        assert_eq!(drive.app.pads.bank, 2);
    }
    drive.app.chop.as_mut().unwrap().select_slice(1);
    let (slice, cursor, plan) = (
        drive.app.chop.as_ref().unwrap().selected,
        drive.app.chop.as_ref().unwrap().cursor,
        drive.app.plan.clone(),
    );
    let entry = std::mem::take(&mut drive.app.sent);
    assert_eq!(
        entry
            .iter()
            .filter(|c| matches!(c, Command::HushPads))
            .count(),
        1
    );
    assert!(!entry.iter().any(|c| matches!(
        c,
        Command::PadPress { .. } | Command::Preview { sound: Some(_), .. }
    )));
    for event in delayed {
        drive.app.midi_ui_event(event);
    }
    let notification = drive.frame();
    assert!(
        notification.ui.is_empty(),
        "late {source:?}, rejected {rejected}, {mode:?}: {:?}",
        notification.ui
    );
    assert_eq!(drive.app.chop.as_ref().unwrap().selected, slice);
    assert_eq!(drive.app.chop.as_ref().unwrap().cursor, cursor);
    assert_eq!(drive.app.plan, plan);
    assert!(drive.app.midi_ui_held.is_empty());
    append_stereo(&mut wave, drive.render_blocks(64));
    drive.bytes(
        PAD_PORT,
        if matches!(source, Source::Notes) {
            &[0x80, 38, 0]
        } else {
            &[0xb0, 80, 0]
        },
    );
    drive.midi().recorded_fence();
    let releases: Vec<_> = drive.midi().events.try_iter().collect();
    if !rejected {
        assert!(releases.iter().any(|event| matches!(
            event.event,
            midi::Event::PadRelease {
                abs: 18,
                direct: true
            }
        )));
    }
    for event in releases {
        drive.app.midi_ui_event(event);
    }
    assert!(drive.frame().ui.is_empty(), "no extra UI audio release");
    let release_at = wave.0.len();
    append_stereo(&mut wave, drive.render_blocks(32));

    let mut expected_engine = engine::Engine::new(RENDER_RATE);
    for setting in settings {
        expected_engine.handle(0, setting);
    }
    let mut events = Vec::new();
    if !rejected {
        events.push(engine::offline::Event {
            frame: 32 * RENDER_BLOCK,
            cmd: Command::PadPress {
                pad: 18,
                velocity: f32::from(velocity) / 127.0,
            },
        });
    }
    events.extend(entry.into_iter().map(|cmd| engine::offline::Event {
        frame: 40 * RENDER_BLOCK,
        cmd,
    }));
    if !rejected {
        events.push(engine::offline::Event {
            frame: release_at,
            cmd: Command::PadRelease { pad: 18 },
        });
    }
    let expected = engine::offline::render(
        &mut expected_engine,
        &engine::offline::Scenario {
            sample_rate: RENDER_RATE,
            frames: wave.0.len(),
            events,
        },
        RENDER_BLOCK,
    );
    assert_eq!(
        wave, expected,
        "{source:?}, rejected {rejected}, {mode:?}, bank change {change_bank}, velocity {velocity}"
    );
    for channel in [&wave.0, &wave.1] {
        assert_eq!(first_non_finite(channel), None);
        assert!(peak(channel) < 1.0);
        assert_eq!(peak(&channel[..32 * RENDER_BLOCK]), 0.0);
        if rejected {
            assert_eq!(peak(channel), 0.0, "a rejected hit stays silent");
        } else {
            assert!(
                first_loud(channel, 32 * RENDER_BLOCK..33 * RENDER_BLOCK, 1e-6)
                    .unwrap()
                    .abs_diff(32 * RENDER_BLOCK)
                    <= 1
            );
        }
        assert_eq!(
            peak(&channel[40 * RENDER_BLOCK + 2048..]),
            0.0,
            "Chop hush without restart"
        );
    }
    wave
}

/// Scenario "Late accepted direct MIDI notification does not restart Chop".
#[test]
fn late_accepted_direct_midi_notification_does_not_restart_chop() {
    for source in [Source::Notes, Source::Button] {
        for mode in [
            LateChop::Linked,
            LateChop::Bare,
            LateChop::Assign,
            LateChop::AssignOrder,
        ] {
            for change_bank in [false, true] {
                for velocity in [23, 101] {
                    render_late_direct_notification(source, false, mode, change_bank, velocity);
                }
            }
        }
    }
}

/// Scenario "Late rejected direct MIDI notification stays silent on Chop".
#[test]
fn late_rejected_direct_midi_notification_stays_silent_on_chop() {
    for source in [Source::Notes, Source::Button] {
        for mode in [
            LateChop::Linked,
            LateChop::Bare,
            LateChop::Assign,
            LateChop::AssignOrder,
        ] {
            for change_bank in [false, true] {
                for velocity in [23, 101] {
                    render_late_direct_notification(source, true, mode, change_bank, velocity);
                }
            }
        }
    }
}

#[test]
fn render_85_late_midi() {
    let mut wave = (Vec::new(), Vec::new());
    for source in [Source::Notes, Source::Button] {
        for rejected in [false, true] {
            append_stereo(
                &mut wave,
                render_late_direct_notification(source, rejected, LateChop::Linked, false, 101),
            );
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/85-late-midi.wav");
    engine::offline::write_wav(&path, RENDER_RATE, &wave.0, &wave.1).unwrap();
    println!(
        "85 late MIDI: {} frames, finite/no-clip/silence/onset/hush checked, sample-identical to timestamped oracle",
        wave.0.len()
    );
}

/// Scenario "Hit from a controller": the real direct queue sounds before App sees
/// the event. Processing App later must not add another hit or release.
#[test]
fn offline_direct_midi_uses_the_next_block_without_ui_or_duplicate() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(12, true);
        drive.tap(KeyCode::F(2), M::NONE);
        let mut actual = drive.render_blocks(PARITY_HIT / RENDER_BLOCK);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.bytes(
            PAD_PORT,
            if matches!(source, Source::Notes) {
                &[0x90, 38, 127]
            } else {
                &[0xb0, 80, 127]
            },
        );
        drive.midi().recorded_fence();
        // No frame(): UI has not observed this hit.
        append_stereo(&mut actual, drive.render_blocks(1));
        assert!(engine::checks::peak(&actual.0[PARITY_HIT..]) > 0.01);
        assert!(
            drive.frame().ui.is_empty(),
            "UI must not repeat the direct hit"
        );
        append_stereo(
            &mut actual,
            drive.render_blocks((PARITY_RELEASE - PARITY_HIT) / RENDER_BLOCK - 1),
        );
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::F(6), M::NONE);
        drive.tap(KeyCode::F(10), M::NONE);
        drive.bytes(
            PAD_PORT,
            if matches!(source, Source::Notes) {
                &[0x90, 38, 0]
            } else {
                &[0xb0, 80, 0]
            },
        );
        drive.midi().recorded_fence();
        append_stereo(&mut actual, drive.render_blocks(1));
        assert!(
            drive.frame().ui.is_empty(),
            "UI must not repeat the direct release"
        );
        append_stereo(
            &mut actual,
            drive.render_blocks((PARITY_END - PARITY_RELEASE) / RENDER_BLOCK - 1),
        );
        let expected = render_chop_route(Source::Keyboard, 127, 12, true, RecordedRelease::Off);
        assert_eq!(
            actual, expected,
            "direct audio must match one UI hit, {source:?}"
        );
        check_parity_wave(&actual);
    }
}

/// Scenario "Chop rejected hits preserve accepted gate release": fill the actual
/// callback queue, not a replacement capture queue. The old voice must keep its phase.
#[test]
fn offline_full_queue_keeps_the_accepted_gate_and_delivers_pending_release() {
    use engine::checks::{first_non_finite, peak};
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(12, true);
        drive.render_blocks(32);
        drive.velocity_hit(source, 127);
        let first = drive.render_blocks(8);
        let original = drive.app.midi_ui_held.clone();
        let mut filled = 0;
        while drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch: 0,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .is_ok()
        {
            filled += 1;
        }
        assert_eq!(filled, 1024);
        drive.velocity_hit(source, 127);
        assert_eq!(drive.app.midi_ui_held, original);
        assert_eq!(drive.app.audio_held[18], Some(0));
        assert_only_release(&drive.pad_release(source, 2), 18);
        assert_eq!(drive.app.pending_audio.len(), 1);
        let after_full = drive.render_blocks(1);
        assert_eq!(drive.app.pending_audio.len(), 1);
        drive.app.flush_audio();
        assert!(drive.app.pending_audio.is_empty());
        let tail = drive.render_blocks(128);
        assert_eq!(first_non_finite(&tail.0), None);
        assert!(peak(&tail.0) < 1.0);
        assert_eq!(peak(&tail.0[4_096..]), 0.0, "pending release reaches audio");
        let mut reference = Drive::offline();
        reference.parity_pad(12, true);
        reference.render_blocks(32);
        reference.velocity_hit(Source::Keyboard, 127);
        assert_eq!(first, reference.render_blocks(8));
        assert_eq!(
            after_full,
            reference.render_blocks(1),
            "rejected hit must not retrigger"
        );
    }
    // A rejected first hit must not become a voice after queue space returns.
    let mut drive = Drive::offline();
    drive.parity_pad(12, true);
    drive.render_blocks(32);
    while drive
        .app
        .audio
        .as_mut()
        .unwrap()
        .commands
        .push(ProjectCommand {
            epoch: 0,
            ready_at_send: true,
            command: ProjectMessage::Command(Command::SetPadGain {
                pad: 127,
                gain: 1.0,
            }),
        })
        .is_ok()
    {}
    drive.velocity_hit(Source::Notes, 127);
    assert!(drive.app.midi_ui_held.is_empty());
    assert!(drive.pad_release(Source::Notes, 2).commands().is_empty());
    assert_eq!(peak(&drive.render_blocks(16).0), 0.0);
}

/// Scenario "Chop release keeps direct route and epoch": stale commands stay in
/// the real callback queues; stale UI events are delivered after the new gate starts.
#[test]
fn offline_project_switch_filters_both_routes_and_stale_releases() {
    use engine::checks::peak;
    use std::sync::atomic::Ordering::{Acquire, Release};
    for source in [Source::Notes, Source::Button] {
        for linked in [false, true] {
            let mut drive = Drive::offline();
            drive.parity_pad(12, true);
            if !linked {
                drive.tap(KeyCode::F(2), M::NONE);
            }
            drive.render_blocks(32);
            drive.velocity_hit(source, 127);
            assert!(peak(&drive.render_blocks(8).0) > 0.01);
            // Leave accepted old-epoch hit/release commands in either real queue.
            drive.pad_release(source, 2);
            drive.velocity_hit(source, 127);
            drive.bytes(
                PAD_PORT,
                if matches!(source, Source::Notes) {
                    &[0x80, 38, 0]
                } else {
                    &[0xb0, 80, 0]
                },
            );
            drive.bytes(
                PAD_PORT,
                if matches!(source, Source::Notes) {
                    &[0x90, 38, 127]
                } else {
                    &[0xb0, 80, 127]
                },
            );
            drive.midi().recorded_fence();
            let old: Vec<_> = drive.midi().events.try_iter().collect();
            assert!(
                old.iter()
                    .any(|event| matches!(event.event, midi::Event::PadHit { .. }))
            );
            assert!(old.iter().any(|event| matches!(
                event.event,
                midi::Event::UiPadRelease { .. } | midi::Event::PadRelease { .. }
            )));
            drive.app.apply_session(session::Session::default());
            for _ in 0..256 {
                drive.app.flush_audio();
                drive.render_blocks(1);
                if drive.project.ready.load(Acquire) == 1 && drive.app.pending_audio.is_empty() {
                    break;
                }
            }
            assert_eq!(drive.project.ready.load(Acquire), 1);
            assert!(drive.app.pending_audio.is_empty());
            drive.frame();
            drive.app.pads.track_keys = true;
            drive.parity_pad(12, true);
            assert_eq!(
                peak(&drive.render_blocks(32).0),
                0.0,
                "no queued old-project hit"
            );
            drive.velocity_hit(Source::Keyboard, 127);
            drive.render_blocks(8);
            for event in old {
                drive.app.midi_ui_event(event);
            }
            assert!(drive.frame().commands().is_empty());
            assert!(
                peak(&drive.render_blocks(32).0) > 0.01,
                "old release cannot stop new gate"
            );
            assert_eq!(drive.app.audio_held[18], Some(1));
            assert_only_release(&drive.pad_release(Source::Keyboard, 2), 18);
            assert_eq!(peak(&drive.render_blocks(64).0[4_096..]), 0.0);
            drive.project.ready.store(0, Release);
            drive.velocity_hit(source, 127);
            drive.project.ready.store(1, Release);
            drive.frame();
            assert!(drive.pad_release(source, 2).commands().is_empty());
            assert_eq!(
                peak(&drive.render_blocks(32).0),
                0.0,
                "not-ready hit cannot sound later"
            );
        }
    }
}

/// Scenarios "Chop parity before pad assignment", "Chop parity in by-hand assignment",
/// "Chop parity while a question is open" and "Chop parity while learning".
#[test]
fn offline_preview_assignment_and_guards_match_through_all_inputs() {
    use engine::checks::{first_non_finite, peak};
    let sound = Arc::new(Sample::mono(
        (0..96_000)
            .map(|i| (std::f32::consts::TAU * 220.0 * i as f32 / 48_000.0).sin() * 0.2)
            .collect(),
    ));
    for mode in ["preview", "plan", "question", "learn"] {
        let mut expected = None;
        for source in [Source::Keyboard, Source::Notes, Source::Button] {
            let mut drive = Drive::offline();
            drive.prepare_chop(sound.clone(), 4);
            drive.tap(KeyCode::PageDown, M::NONE);
            match mode {
                "plan" => {
                    drive.tap(KeyCode::Char('p'), M::NONE);
                }
                "question" => {
                    drive.load_pad(18, sound.clone(), "occupied");
                    drive.tap(KeyCode::Char('p'), M::NONE);
                    drive.pad_press(Source::Keyboard, 2);
                    drive.pad_release(Source::Keyboard, 2);
                    drive.tap(KeyCode::Enter, M::NONE);
                    assert!(drive.app.ask.is_some());
                }
                "learn" => {
                    drive.tap(KeyCode::F(10), M::NONE);
                }
                _ => {}
            }
            // Let the setup preview end before a guard is tested.
            assert_eq!(peak(&drive.render_blocks(512).0[60_000..]), 0.0);
            drive.pad_press(source, 2);
            let mut wave = drive.render_blocks(32);
            assert!(drive.app.audio_held.iter().all(Option::is_none));
            let release = drive.pad_release(source, 2);
            if matches!(source, Source::Keyboard) {
                // Pads::apply can return a keyboard release attempt even for a
                // preview. App::try_send rejects it without an accepted pad hold.
                assert!(
                    release
                        .commands()
                        .iter()
                        .all(|command| matches!(command, Command::PadRelease { .. }))
                );
            } else {
                assert!(release.commands().is_empty());
            }
            assert!(drive.app.pending_audio.is_empty());
            append_stereo(&mut wave, drive.render_blocks(224));
            for channel in [&wave.0, &wave.1] {
                assert_eq!(first_non_finite(channel), None);
                assert!(peak(channel) < 1.0);
                if matches!(mode, "question" | "learn") {
                    assert_eq!(peak(channel), 0.0, "guarded {source:?}, {mode}");
                } else {
                    assert!(peak(channel) > 0.01, "preview {source:?}, {mode}");
                    assert_eq!(peak(&channel[26_000..]), 0.0);
                }
            }
            if let Some(expected) = &expected {
                assert_eq!(&wave, expected, "{mode}, {source:?}");
            } else {
                expected = Some(wave);
            }
        }
    }
}

/// Scenario "Copy route preserves held MIDI releases".
#[test]
fn copy_route_preserves_held_midi_releases() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.load_pad(0, Arc::new(Sample::mono(vec![0.25; 1024])), "source");
        let press = drive.pad_press(source, 0);
        assert!(matches!(
            press.commands()[..],
            [Command::PadPress { pad: 0, .. }]
        ));
        drive.tap(KeyCode::Char('c'), M::CONTROL);
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::F(4), M::NONE);
        let release = drive.pad_release(source, 0);
        assert!(
            matches!(release.commands()[..], [Command::PadRelease { pad: 0 }]),
            "{source:?}: {:?}",
            release.commands()
        );
        assert!(!drive.app.pads.held[0]);
    }
}

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

struct TransitionFiles(PathBuf);

impl TransitionFiles {
    fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/86c-transitions")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Relaxed)
            ));
        for name in ["A", "B"] {
            let folder = root.join(name);
            std::fs::create_dir_all(folder.join("samples")).unwrap();
            session::save_project(&folder.join("project.toml"), &session::Session::default())
                .unwrap();
        }
        let wave = vec![0.25; 4096];
        engine::offline::write_wav(&root.join("tone.wav"), 48_000, &wave, &wave).unwrap();
        Self(root)
    }

    fn drive(&self, project: bool) -> Drive {
        let mut drive = Drive::new();
        drive.bind_action(88, "project.pick");
        drive.bind_action(89, "pad.mode_next");
        drive.bind_action(90, "chop.cut");
        let file = self.0.join("tone.wav");
        let sound = Arc::new(Sample::mono(vec![0.25; 4096]));
        drive.load_pad(0, sound.clone(), "tone");
        drive.app.pads.file[0] = Some(file.clone());
        drive.app.chop = Some(chop::Chop::new(file, sound, 48_000));
        drive.app.paths.projects = self.0.clone();
        drive.app.browser = browser::Browser::new(Some(self.0.clone()));
        drive.app.preview_on = false;
        drive.app.session_path = self.0.join("scratch/session.toml");
        drive.app.session_read_error = None;
        if project {
            drive.app.project = Some(Project {
                name: "A".into(),
                folder: self.0.join("A"),
            });
            drive.app.project_cfg.last = Some("A".into());
        }
        drive.app.save_session().unwrap();
        let config_path = self.0.join("config.toml");
        config::save_section(&config_path, "project", &drive.app.project_cfg).unwrap();
        drive.app.user_path = Some(config_path);
        drive.frame();
        drive
    }

    fn save_feedback_drive(&self, project: bool, source: Source) -> Drive {
        let mut drive = self.drive(project);
        for (number, action) in [
            (91, "session.save"),
            (92, "bank.next"),
            (93, "master.volume_up"),
        ] {
            // Learning keeps one control per action in a group.
            let key = match source {
                Source::Notes => input::midi::Key::Note(number),
                _ => input::midi::Key::Cc(number),
            };
            learn::assign(
                &mut drive.app.learned,
                "Recorded pads",
                input::Group::Global,
                key,
                action,
            );
        }
        drive
            .app
            .sync_learned("Recorded pads", input::Group::Global);
        drive.midi().recorded_fence();
        drive
    }

    fn edit(drive: &mut Drive, source: Source) {
        drive.registry_command(source, KeyCode::Char('h'), M::NONE, 89);
        drive.tap(KeyCode::F(3), M::NONE);
        assert_eq!(drive.app.screen, Screen::Chop);
        for _ in 0..16 {
            drive.tap(KeyCode::Right, M::NONE);
        }
        drive.registry_command(source, KeyCode::Char('m'), M::NONE, 90);
        assert_eq!(drive.app.chop.as_ref().unwrap().saved().cuts.len(), 1);
        drive.tap(KeyCode::F(2), M::NONE);
        assert!(drive.app.session_dirty.is_some());
        assert!(!drive.app.history.is_empty());
    }

    fn pick(drive: &mut Drive, source: Source, name: &str, create: bool) {
        drive.registry_command(source, KeyCode::Char('p'), M::CONTROL, 88);
        assert!(drive.app.project_picker.is_some());
        // The current project picker has keyboard selection and confirmation only.
        // MIDI still enters through the production action and dispatch above.
        if create {
            for c in name.chars() {
                drive.tap(KeyCode::Char(c), M::NONE);
            }
        } else {
            let at = drive
                .app
                .project_picker
                .as_ref()
                .unwrap()
                .entries
                .iter()
                .position(|entry| entry == name)
                .unwrap();
            for _ in 0..at {
                drive.tap(KeyCode::Down, M::NONE);
            }
        }
    }

    fn picker(drive: &Drive) -> (Vec<String>, usize, Option<String>) {
        let picker = drive.app.project_picker.as_ref().unwrap();
        (
            picker.entries.clone(),
            picker.selected,
            picker.naming.clone(),
        )
    }

    fn state(drive: &Drive) -> Seen {
        let mut seen = drive.seen();
        seen.notice = None;
        seen
    }
}

impl Drop for TransitionFiles {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
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
        assert_eq!(sess.version, 4);
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
            assert_eq!(session::load_existing(&path).unwrap().version, 4);
        }
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
        meters: ([-60.0; 2], [-60.0; 2], false),
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
    assert!(header.contains("PLAY"), "missing PLAY badge: {header}");
    header.contains("PLAY ▶")
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

    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for mode in ["normal", "mark", "copy", "swap", "file", "learn"] {
            for hidden in [false, true] {
                for switch in [false, true] {
                    let mut drive = Drive::offline();
                    drive.parity_pad(0, false);
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
                        let end = drive.render_blocks(400);
                        assert!(
                            !pads_play_badge(&mut drive),
                            "the last voice ended, not the mode"
                        );
                        assert_eq!(pad_feedback(&drive), feedback);
                        for channel in [&end.0, &end.1] {
                            assert_eq!(engine::checks::first_non_finite(channel), None);
                            assert!(engine::checks::peak(channel) < 1.0);
                            assert_eq!(engine::checks::peak(&channel[49_000..]), 0.0);
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

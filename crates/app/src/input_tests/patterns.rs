//! Pattern controls through the production keyboard and recorded MIDI routes.

use super::*;

const PG: &str = "pad.pitch_gate_toggle";

struct F1Paint {
    buffer: ratatui::buffer::Buffer,
    lines: Vec<ratatui::text::Line<'static>>,
    start: usize,
    caption: bool,
}

fn f1_pages_key(
    drive: &mut Drive,
    code: KeyCode,
    modifiers: M,
    kind: KeyEventKind,
    kitty: bool,
) -> Frame {
    assert!(!drive.app.keyboard_event(
        &drive.keymap,
        &KeyEvent::new_with_kind(code, modifiers, kind),
        kitty,
        &mut drive.down,
        &mut drive.held,
        &mut drive.mods
    ));
    drive.frame()
}

fn f1_pages_frame(
    drive: &mut Drive,
    size: (u16, u16),
    kitty: bool,
    state: Option<engine::capture::Snapshot>,
) -> F1Paint {
    use ratatui::{Terminal, backend::TestBackend};
    let app = &mut drive.app;
    app.resize_help(size.0, size.1);
    let rows = app.full_help_recording_keys(&drive.keymap, kitty, state);
    app.prepare_key_help(&drive.keymap, &rows, state);
    let start = app.help_overlay.keys_scroll;
    let caption = app.help_overlay.caption.is_some();
    let lines = tui::help_key_lines(&rows, size.0 - 2);
    // F1 covers the body. Input still uses the actual App screen and ownership.
    let pads = pads_view(&mut app.pads, None, None, vec![], None, app.rate);
    let screen = tui::Screen::Pads(app.mark_pads_recording(pads, state));
    let status = tui::Status {
        badges: view::FrameText::default()
            .badges_for(app, Instant::now(), state)
            .to_vec(),
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
        no_releases: !kitty,
        project: None,
    };
    let entries = app.message_history();
    let help = tui::Help {
        groups: &[],
        hint: None,
        full: app.full_help_view(&rows, &entries),
        question: None,
        learn: None,
    };
    let mut terminal = Terminal::new(TestBackend::new(size.0, size.1)).unwrap();
    terminal
        .draw(|frame| tui::draw(frame, &status, &screen, &help))
        .unwrap();
    F1Paint {
        buffer: terminal.backend().buffer().clone(),
        lines,
        start,
        caption,
    }
}

fn f1_pages_live(drive: &mut Drive, size: (u16, u16), kitty: bool) -> F1Paint {
    let state = drive.app.pattern_record_state();
    f1_pages_frame(drive, size, kitty, state)
}

fn f1_pages_row(frame: &F1Paint, y: u16) -> String {
    (0..frame.buffer.area.width)
        .map(|x| frame.buffer[(x, y)].symbol())
        .collect()
}

fn f1_pages_visit(drive: &mut Drive, size: (u16, u16), kitty: bool, next: (KeyCode, M)) {
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        widgets::{Paragraph, Widget},
    };
    let first = f1_pages_live(drive, size, kitty);
    assert_eq!(first.start, 0);
    let mut seen = vec![false; first.lines.len()];
    let mut frame = first;
    for _ in 0..100 {
        let height = tui::help_page_height(size.1, frame.caption);
        for row in 0..height {
            let index = frame.start + row;
            if index >= frame.lines.len() {
                break;
            }
            let area = Rect::new(0, 0, size.0 - 2, 1);
            let mut expected = Buffer::empty(area);
            Paragraph::new(frame.lines[index].clone()).render(area, &mut expected);
            let y = 2 + u16::from(frame.caption) + row as u16;
            for x in 0..size.0 - 2 {
                assert_eq!(
                    frame.buffer[(x + 1, y)].symbol(),
                    expected[(x, 0)].symbol(),
                    "{:?} {size:?} line {index}, cell {x}",
                    drive.app.screen
                );
            }
            seen[index] = true;
        }
        let old = frame.start;
        assert!(
            f1_pages_key(drive, next.0, next.1, KeyEventKind::Press, kitty)
                .commands()
                .is_empty()
        );
        f1_pages_key(drive, next.0, next.1, KeyEventKind::Release, kitty);
        frame = f1_pages_live(drive, size, kitty);
        if frame.start == old {
            break;
        }
    }
    assert!(
        seen.iter().all(|seen| *seen),
        "unreachable help lines: {seen:?}"
    );
    assert_eq!(
        frame.start,
        frame
            .lines
            .len()
            .saturating_sub(tui::help_page_height(size.1, frame.caption))
    );
}

/// Scenario "Every full help row is reachable".
/// Scenario "Full help distinguishes the three recording targets".
/// Scenario "Save and erase fallbacks are stated before use".
#[test]
fn f1_pages_all_key_table_rows_are_visible_with_owner_keys_and_fallbacks() {
    for screen in Screen::ALL {
        for size in [(80, 26), (120, 40)] {
            for kitty in [false, true] {
                for owner in 0..3 {
                    let mut drive = app_capture_drive(vec![]);
                    drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48000])), 4);
                    app_capture_start(&mut drive, 1);
                    drive.app.send(Command::PatternErase {
                        pass: 1,
                        enabled: true,
                    });
                    for pad in 0..128 {
                        drive.app.send(Command::PatternEraseTarget {
                            pass: 1,
                            pad,
                            one_loop: false,
                        });
                    }
                    app_capture_frames(&mut drive, 0);
                    assert_eq!(app_capture_snapshot(&drive).erase_targets, [u64::MAX; 2]);
                    drive.app.goto(screen);
                    let (a1, mic, output, next) = match owner {
                        0 => ("Ctrl+R", "Alt+R", "Ctrl+T", (KeyCode::PageDown, M::ALT)),
                        1 => {
                            assert!(drive.keymap.overlay("[global]\n'Ctrl+R' = 'record.mic'\n'Alt+R' = 'none'\n'Ctrl+T' = 'none'\nF8 = 'pattern.record'\nF9 = 'record.output'\n'Alt+PageDown' = 'none'\nF11 = 'help.page_next'\n").unwrap().is_empty());
                            ("F8", "Ctrl+R", "F9", (KeyCode::F(11), M::NONE))
                        }
                        _ => {
                            assert!(drive.keymap.overlay("[global]\n'Ctrl+R' = 'none'\n'Alt+R' = 'none'\n'Ctrl+T' = 'none'\n").unwrap().is_empty());
                            ("none", "none", "none", (KeyCode::PageDown, M::ALT))
                        }
                    };
                    let state = drive.app.pattern_record_state();
                    let rows = drive
                        .app
                        .full_help_recording_keys(&drive.keymap, kitty, state);
                    for (key, description) in [
                        (a1, "record A1:"),
                        (mic, "record the microphone:"),
                        (output, "record the output:"),
                    ] {
                        let row = rows
                            .iter()
                            .find(|(_, text)| text.starts_with(description))
                            .unwrap();
                        assert_eq!(row.0, key, "{screen:?} {rows:?}");
                    }
                    assert!(
                        rows.iter()
                            .any(|(_, text)| text.contains("finishes the pass")
                                && text.contains("playback running"))
                    );
                    assert!(rows.iter().any(|(_, text)| text.contains(if kitty {
                        "Hold pad targets silently"
                    } else {
                        "one loop per pad press"
                    })));
                    assert!(
                        rows.iter()
                            .any(|(_, text)| text.starts_with("record the microphone:")
                                && text.contains(if kitty { "hold" } else { "toggle on / off" }))
                    );
                    let targets = &rows
                        .iter()
                        .find(|(label, _)| label == "Erase targets")
                        .unwrap()
                        .1;
                    for pad in 0..128 {
                        assert!(targets.split_whitespace().any(|id| id == bank_key(pad)));
                    }
                    f1_pages_key(
                        &mut drive,
                        KeyCode::F(1),
                        M::NONE,
                        KeyEventKind::Press,
                        kitty,
                    );
                    f1_pages_key(
                        &mut drive,
                        KeyCode::F(1),
                        M::NONE,
                        KeyEventKind::Release,
                        kitty,
                    );
                    let before = drive.seen();
                    f1_pages_visit(&mut drive, size, kitty, next);
                    assert_eq!(drive.seen(), before);
                    assert_eq!(
                        drive.app.pattern_record_state().unwrap().erase_targets,
                        [u64::MAX; 2]
                    );
                    let frame = f1_pages_live(&mut drive, size, kitty);
                    let footer = f1_pages_row(&frame, size.1 - 5);
                    assert!(
                        footer.contains(if owner == 1 { "F11" } else { "Alt+PgDn" }),
                        "{footer}"
                    );
                    if owner == 1 {
                        assert!(!footer.contains("Alt+PgDn"));
                    }
                    drive.tap(KeyCode::Char('e'), M::ALT);
                    app_capture_frames(&mut drive, 0);
                    assert!(!app_capture_snapshot(&drive).erasing);
                    let changed = f1_pages_live(&mut drive, size, kitty);
                    assert!(changed.lines.len() < frame.lines.len());
                    assert!(changed.start < frame.start);
                    assert_eq!(
                        changed.start,
                        changed
                            .lines
                            .len()
                            .saturating_sub(tui::help_page_height(size.1, changed.caption))
                    );
                }
            }
        }
    }
}

/// Scenario "Help page keys preserve ordinary roles".
#[test]
fn f1_pages_keyboard_repeats_removals_and_edit_keys_keep_banks_and_pad_releases() {
    for screen in Screen::ALL {
        for kitty in [false, true] {
            for edit in [false, true] {
                let mut drive = app_capture_drive(vec![]);
                drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48000])), 4);
                app_capture_sound(
                    &mut drive,
                    0,
                    Arc::new(Sample::mono(vec![0.03; 48000])),
                    true,
                );
                app_capture_start(&mut drive, 1);
                drive.app.goto(screen);
                drive.app.edit = edit;
                drive.app.pads.track_keys = kitty;
                for modifier in [
                    input::Modifier::Ctrl,
                    input::Modifier::Alt,
                    input::Modifier::Shift,
                ] {
                    assert!(
                        modifier_groups(&drive.keymap, screen, modifier, false)
                            .iter()
                            .flat_map(|group| &group.items)
                            .all(|(_, text)| !text.starts_with("F1: "))
                    );
                }
                let before = drive.seen();
                f1_pages_key(
                    &mut drive,
                    KeyCode::PageDown,
                    M::ALT,
                    KeyEventKind::Press,
                    kitty,
                );
                assert_eq!(drive.seen(), before);
                f1_pages_key(
                    &mut drive,
                    KeyCode::PageDown,
                    M::ALT,
                    KeyEventKind::Release,
                    kitty,
                );
                drive.tap(KeyCode::F(1), M::NONE);
                let frame = f1_pages_live(&mut drive, (80, 26), kitty);
                assert_eq!(frame.start, 0);
                let maximum = frame.lines.len().saturating_sub(18);
                for (kind, expected) in [
                    (KeyEventKind::Press, 18.min(maximum)),
                    (KeyEventKind::Repeat, 36.min(maximum)),
                    (KeyEventKind::Release, 36.min(maximum)),
                ] {
                    assert!(
                        f1_pages_key(&mut drive, KeyCode::PageDown, M::ALT, kind, kitty)
                            .commands()
                            .is_empty()
                    );
                    assert_eq!(
                        drive.app.help_overlay.keys_scroll, expected,
                        "{screen:?} {kitty} {edit}"
                    );
                }
                assert_eq!(drive.seen(), before);
                for key in ['h', 'р'] {
                    assert!(drive.keymap.overlay("[global]\n'Alt+PageDown' = 'none'\n'Alt+H' = 'help.page_next'\n[edit]\n'Alt+PageUp' = 'none'\n'Alt+H' = 'help.page_prev'\n[screens.pads]\n'Alt+H' = 'help.page_next'\n").unwrap().is_empty());
                    let frame = f1_pages_live(&mut drive, (80, 26), kitty);
                    let maximum = frame.lines.len().saturating_sub(18);
                    let old = drive.app.help_overlay.keys_scroll;
                    f1_pages_key(
                        &mut drive,
                        KeyCode::PageDown,
                        M::ALT,
                        KeyEventKind::Press,
                        kitty,
                    );
                    f1_pages_key(
                        &mut drive,
                        KeyCode::PageDown,
                        M::ALT,
                        KeyEventKind::Release,
                        kitty,
                    );
                    assert_eq!(drive.app.help_overlay.keys_scroll, old);
                    f1_pages_key(
                        &mut drive,
                        KeyCode::Char(key),
                        M::ALT,
                        KeyEventKind::Press,
                        kitty,
                    );
                    f1_pages_key(
                        &mut drive,
                        KeyCode::Char(key),
                        M::ALT,
                        KeyEventKind::Release,
                        kitty,
                    );
                    assert_eq!(
                        drive.app.help_overlay.keys_scroll,
                        if edit {
                            old.saturating_sub(18)
                        } else {
                            (old + 18).min(maximum)
                        }
                    );
                    assert!(!drive.app.help_overlay.navigation.contains("Alt+PgDn"));
                    assert!(
                        modifier_groups(&drive.keymap, screen, input::Modifier::Alt, false)
                            .iter()
                            .flat_map(|group| &group.items)
                            .all(|(_, text)| !text.starts_with("F1: "))
                    );
                    let rows = drive.app.full_help_keys(&drive.keymap, kitty);
                    assert!(
                        !rows
                            .iter()
                            .any(|(keys, text)| text.starts_with("F1: ")
                                && keys.contains("Alt+PgDn"))
                    );
                }
                if kitty {
                    drive.pad_press(Source::Keyboard, 0);
                    app_capture_frames(&mut drive, 128);
                    drive.app.flush_audio();
                    let onset = drive.frame();
                    assert!(onset.commands().iter().any(|command| matches!(
                        command,
                        Command::PublishRecordedPattern { .. }
                            | Command::PublishCapturedPattern { .. }
                    )));
                    app_capture_frames(&mut drive, 0);
                    drive.tap(KeyCode::PageDown, M::NONE);
                    assert_eq!(drive.app.pads.bank, 1);
                    let release = drive.pad_release(Source::Keyboard, 0);
                    assert_only_release(&release, 0);
                    app_capture_frames(&mut drive, 0);
                    assert_eq!(app_capture_snapshot(&drive).captured_onsets, 1);
                } else {
                    f1_pages_key(
                        &mut drive,
                        KeyCode::PageDown,
                        M::NONE,
                        KeyEventKind::Press,
                        false,
                    );
                    assert_eq!(drive.app.pads.bank, 1);
                }
                let position = drive.app.help_overlay.keys_scroll;
                drive.tap(KeyCode::Tab, M::NONE);
                assert!(drive.app.help_overlay.messages());
                f1_pages_key(
                    &mut drive,
                    KeyCode::Char('h'),
                    M::ALT,
                    KeyEventKind::Press,
                    kitty,
                );
                assert_eq!(drive.app.help_overlay.keys_scroll, position);
                assert_eq!(drive.app.pads.bank, 1);
            }
        }
    }
}

/// Scenario "Help page commands keep first-frame order".
#[test]
fn f1_pages_first_frame_keeps_order_tabs_and_close_without_stale_requests() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for screen in Screen::ALL {
            let mut drive = app_capture_drive(vec![]);
            drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48000])), 4);
            f1_pages_routes(&mut drive, source);
            drive.app.goto(screen);
            let before = drive.seen();
            drive.registry_command(source, KeyCode::F(1), M::NONE, 92);
            for (code, number) in [
                (KeyCode::PageDown, 91),
                (KeyCode::PageDown, 91),
                (KeyCode::PageUp, 90),
            ] {
                assert!(
                    drive
                        .registry_command(source, code, M::ALT, number)
                        .commands()
                        .is_empty()
                );
            }
            let first = f1_pages_live(&mut drive, (80, 26), true);
            let expected = 38
                .min(first.lines.len().saturating_sub(19))
                .saturating_sub(19);
            assert_eq!(first.start, expected, "{source:?} {screen:?}");
            assert_eq!(drive.seen(), before);
            drive.registry_command(source, KeyCode::F(1), M::NONE, 92);
            drive.registry_command(source, KeyCode::F(1), M::NONE, 92);
            drive.registry_command(source, KeyCode::PageDown, M::ALT, 91);
            drive.tap(KeyCode::Tab, M::NONE);
            let frame = f1_pages_live(&mut drive, (80, 26), true);
            assert!(drive.app.help_overlay.messages());
            assert_eq!(frame.start, 19.min(frame.lines.len().saturating_sub(19)));
            drive.tap(KeyCode::Tab, M::NONE);
            assert_eq!(drive.app.help_overlay.keys_scroll, frame.start);
            drive.registry_command(source, KeyCode::F(1), M::NONE, 92);
            drive.registry_command(source, KeyCode::F(1), M::NONE, 92);
            drive.registry_command(source, KeyCode::PageDown, M::ALT, 91);
            drive.registry_command(source, KeyCode::F(1), M::NONE, 92);
            assert!(!drive.app.help_open);
            drive.registry_command(source, KeyCode::F(1), M::NONE, 92);
            let reopened = f1_pages_live(&mut drive, (80, 26), true);
            assert_eq!(reopened.start, 0);
            assert!(!drive.app.help_overlay.messages());
            assert_eq!(drive.seen(), before);
        }
    }
}

fn f1_pages_routes(drive: &mut Drive, source: Source) {
    for (number, name) in [
        (90, "help.page_prev"),
        (91, "help.page_next"),
        (92, "help.toggle"),
    ] {
        learn::assign(
            &mut drive.app.learned,
            "Recorded pads",
            input::Group::Global,
            if matches!(source, Source::Notes) {
                input::midi::Key::Note(number)
            } else {
                input::midi::Key::Cc(number)
            },
            name,
        );
    }
    drive
        .app
        .sync_learned("Recorded pads", input::Group::Global);
    drive.frame();
}

/// Scenario "Learned help pages retain input ownership".
#[test]
fn f1_pages_learned_notes_and_cc_keep_blockers_and_original_releases() {
    for source in [Source::Notes, Source::Button] {
        for screen in Screen::ALL {
            let mut drive = app_capture_drive(vec![]);
            drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48000])), 4);
            f1_pages_routes(&mut drive, source);
            drive.app.goto(screen);
            let before = drive.seen();
            assert!(
                drive
                    .registry_command(source, KeyCode::Null, M::NONE, 91)
                    .commands()
                    .is_empty()
            );
            assert_eq!(drive.seen(), before);
            drive.registry_command(source, KeyCode::Null, M::NONE, 92);
            assert!(drive.app.help_open);
            let frame = f1_pages_live(&mut drive, (80, 26), true);
            let maximum = frame.lines.len().saturating_sub(19);
            let previous = 38.min(maximum).saturating_sub(19);
            for (number, expected) in [(91, 19.min(maximum)), (91, 38.min(maximum)), (90, previous)]
            {
                assert!(
                    drive
                        .registry_command(source, KeyCode::Null, M::NONE, number)
                        .commands()
                        .is_empty()
                );
                assert_eq!(
                    drive.app.help_overlay.keys_scroll, expected,
                    "{source:?} {screen:?}"
                );
            }
            assert_eq!(drive.seen(), before);
            drive.tap(KeyCode::Tab, M::NONE);
            drive.registry_command(source, KeyCode::Null, M::NONE, 91);
            assert!(drive.app.help_overlay.messages());
            assert_eq!(drive.app.help_overlay.keys_scroll, previous);
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::Idle
            );
        }
        for blocker in [
            "name",
            "filter",
            "picker",
            "path",
            "question",
            "screen-learn",
            "midi-learn",
        ] {
            let mut drive = app_capture_drive(vec![]);
            f1_pages_routes(&mut drive, source);
            app_capture_sound(
                &mut drive,
                0,
                Arc::new(Sample::mono(vec![0.04; 48000])),
                true,
            );
            drive.pad_press(source, 0);
            drive.tap(KeyCode::F(1), M::NONE);
            f1_pages_live(&mut drive, (80, 26), true);
            drive.registry_command(source, KeyCode::Null, M::NONE, 91);
            assert_eq!(drive.app.help_overlay.keys_scroll, 19);
            match blocker {
                "name" => drive.app.pad_naming = Some((0, "name".into())),
                "filter" => {
                    drive.app.goto(Screen::Files);
                    drive.app.browser.editing = true;
                }
                "picker" => {
                    drive.app.project_picker = Some(ProjectPicker {
                        entries: vec![],
                        selected: 0,
                        naming: None,
                    })
                }
                "path" => {
                    drive.app.path_picker =
                        Some(PathPicker::open(0, Path::new("target/97-f1-pages")))
                }
                "question" => {
                    drive.app.ask = Some(Ask::ClearPads {
                        pads: vec![0],
                        lines: vec!["question".into()],
                    })
                }
                "screen-learn" => drive.app.screen_learn = Some(drive.app.learn_open()),
                "midi-learn" => {
                    drive.app.learning = Some(learn::Learning::new(action_info("help.page_next")))
                }
                _ => unreachable!(),
            }
            drive.frame();
            let learned = drive.app.learned.clone();
            let pattern = drive.app.pads.pattern.as_ref().unwrap().clone();
            for number in [91, 90] {
                drive.registry_command(source, KeyCode::Null, M::NONE, number);
                assert_eq!(
                    drive.app.help_overlay.keys_scroll, 19,
                    "{source:?} {blocker}"
                );
            }
            f1_pages_key(
                &mut drive,
                KeyCode::PageDown,
                M::ALT,
                KeyEventKind::Press,
                true,
            );
            f1_pages_key(
                &mut drive,
                KeyCode::PageDown,
                M::ALT,
                KeyEventKind::Release,
                true,
            );
            assert_eq!(
                drive.app.help_overlay.keys_scroll, 19,
                "{source:?} {blocker}"
            );
            assert_eq!(drive.app.learned, learned);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap(), &pattern);
            let sounding = app_capture_frames(&mut drive, 4096);
            assert!(
                sounding.0[3072..]
                    .iter()
                    .chain(&sounding.1[3072..])
                    .all(|sample| sample.abs() > 0.01),
                "{source:?} {blocker}: held Gate must still sound"
            );
            let release = drive.pad_release(source, 0);
            assert!(
                release.commands().is_empty(),
                "{source:?} {blocker}: direct release must not repeat on the UI"
            );
            let stopped = app_capture_frames(&mut drive, 24000);
            assert!(
                stopped.0[23000..]
                    .iter()
                    .chain(&stopped.1[23000..])
                    .all(|sample| sample.abs() < 0.00001),
                "{source:?} {blocker}: original Gate must stop"
            );
            assert_eq!(drive.app.help_overlay.keys_scroll, 19);
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::Idle
            );
        }
    }
}

/// Scenario "Full help retains accepted recording feedback".
#[test]
fn f1_pages_captions_follow_one_accepted_snapshot_on_both_tabs() {
    for screen in Screen::ALL {
        let mut drive = app_capture_drive(vec![]);
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48000])), 4);
        record_controls_routes(&mut drive, Source::Keyboard);
        drive.app.goto(screen);
        drive.tap(KeyCode::F(1), M::NONE);
        record_controls_record(&mut drive, Source::Keyboard);
        for size in [(80, 26), (120, 40)] {
            let frame = f1_pages_live(&mut drive, size, true);
            assert!(!frame.caption, "queued start is not accepted");
        }
        app_capture_frames(&mut drive, 0);
        let first = drive.app.pattern_record_state();
        for count in (1..=4).rev() {
            for messages in [false, true] {
                if drive.app.help_overlay.messages() != messages {
                    drive.tap(KeyCode::Tab, M::NONE);
                }
                for size in [(80, 26), (120, 40)] {
                    let frame = f1_pages_live(&mut drive, size, true);
                    assert!(f1_pages_row(&frame, 2).contains(&format!("COUNT {count} A1")));
                    assert!(
                        f1_pages_row(&frame, 0).contains("offline"),
                        "{}",
                        f1_pages_row(&frame, 0)
                    );
                    assert!(f1_pages_row(&frame, size.1 - 4).trim().is_empty());
                    assert!(f1_pages_row(&frame, size.1 - 5).contains("F1/Esc"));
                }
            }
            app_capture_frames(&mut drive, 24000);
        }
        for messages in [false, true] {
            if drive.app.help_overlay.messages() != messages {
                drive.tap(KeyCode::Tab, M::NONE);
            }
            for size in [(80, 26), (120, 40)] {
                let frame = f1_pages_live(&mut drive, size, true);
                assert!(f1_pages_row(&frame, 2).contains("REC A1 1:1/1"));
                let earlier = f1_pages_frame(&mut drive, size, true, first);
                assert!(f1_pages_row(&earlier, 2).contains("COUNT 4 A1"));
            }
        }
        app_capture_frames(&mut drive, 24000);
        let frame = f1_pages_live(&mut drive, (80, 26), true);
        assert!(f1_pages_row(&frame, 2).contains("REC A1 1:2/1"));
        record_controls_erase(&mut drive, Source::Keyboard);
        let frame = f1_pages_live(&mut drive, (80, 26), true);
        assert!(!f1_pages_row(&frame, 2).contains("ERASE"));
        let pass = app_capture_snapshot(&drive).pass;
        drive.app.send(Command::PatternEraseTarget {
            pass,
            pad: 127,
            one_loop: false,
        });
        app_capture_frames(&mut drive, 0);
        for messages in [false, true] {
            if drive.app.help_overlay.messages() != messages {
                drive.tap(KeyCode::Tab, M::NONE);
            }
            let frame = f1_pages_live(&mut drive, (80, 26), true);
            assert!(f1_pages_row(&frame, 2).contains("ERASE A1 1:2/1 8-V"));
        }
        record_controls_record(&mut drive, Source::Keyboard);
        let frame = f1_pages_live(&mut drive, (80, 26), true);
        assert!(f1_pages_row(&frame, 2).contains("ERASE A1"));
        app_capture_frames(&mut drive, 0);
        for messages in [false, true] {
            if drive.app.help_overlay.messages() != messages {
                drive.tap(KeyCode::Tab, M::NONE);
            }
            let frame = f1_pages_live(&mut drive, (80, 26), true);
            assert!(f1_pages_row(&frame, 2).contains("Finishing A1"));
        }
        record_controls_idle(&mut drive);
        assert!(!f1_pages_live(&mut drive, (80, 26), true).caption);
        drive.app.apply_session(session::Session::default());
        assert!(!f1_pages_live(&mut drive, (120, 40), true).caption);
    }
}

/// Scenario "Help page offsets follow local lifecycle".
#[test]
fn f1_pages_resize_content_and_tabs_clamp_without_editing_work() {
    let mut drive = app_capture_drive(vec![]);
    drive.tap(KeyCode::F(1), M::NONE);
    for index in 0..50 {
        drive.app.post_system_notice(
            Level::Info,
            (
                format!("Result {index:02}: {}", "long result ".repeat(18)),
                Instant::now(),
            ),
        );
    }
    let before = drive.seen();
    f1_pages_visit(&mut drive, (80, 26), true, (KeyCode::PageDown, M::ALT));
    let narrow = drive.app.help_overlay.keys_scroll;
    let frame = f1_pages_live(&mut drive, (120, 40), true);
    assert!(frame.start < narrow);
    assert_eq!(frame.start, frame.lines.len() - 33);
    let wide = frame.start;
    drive.tap(KeyCode::Tab, M::NONE);
    let frame = f1_pages_live(&mut drive, (80, 26), true);
    assert!(f1_pages_row(&frame, 1).contains("[MESSAGES]"));
    drive.tap(KeyCode::PageDown, M::NONE);
    let messages = drive.app.help_overlay.scroll;
    assert_eq!(messages, 19);
    assert_eq!(drive.app.help_overlay.keys_scroll, wide);
    // back to Help: Tab goes on to Controller now
    drive.tap(KeyCode::BackTab, M::SHIFT);
    assert_eq!(drive.app.help_overlay.page, crate::help::Page::Help);
    assert_eq!(drive.app.help_overlay.keys_scroll, wide);
    assert_eq!(drive.app.help_overlay.scroll, messages);
    assert!(drive.keymap.overlay("[global]\n'Alt+PageUp' = 'none'\n'Alt+PageDown' = 'none'\nF8 = 'help.page_prev'\nF9 = 'help.page_next'\n[edit]\nUp = 'help.page_prev'\nDown = 'help.page_next'\n").unwrap().is_empty());
    drive.app.edit = true;
    let frame = f1_pages_live(&mut drive, (80, 26), true);
    let footer = f1_pages_row(&frame, 21);
    assert!(footer.contains("F8") || footer.contains('↑'), "{footer}");
    assert!(!footer.contains("Alt+Pg"));
    assert!(
        drive
            .app
            .help_bindings(&drive.keymap, Action::HelpPageNext)
            .contains(&"↓".into())
    );
    drive.tap(KeyCode::Down, M::NONE);
    assert!(drive.app.help_overlay.keys_scroll > wide);
    drive.tap(KeyCode::Up, M::NONE);
    assert_eq!(drive.app.help_overlay.keys_scroll, wide);
    drive.app.edit = false;
    let rows = vec![("short".into(), "Only one current row.".into())];
    drive.app.prepare_key_help(&drive.keymap, &rows, None);
    assert_eq!(drive.app.help_overlay.keys_scroll, 0);
    assert_eq!(drive.app.help_overlay.scroll, messages);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(!drive.app.help_open);
    drive.tap(KeyCode::F(1), M::NONE);
    assert!(!drive.app.help_overlay.messages());
    assert_eq!(
        (
            drive.app.help_overlay.keys_scroll,
            drive.app.help_overlay.scroll
        ),
        (0, 0)
    );
    assert_eq!(drive.seen(), before);
}

fn pattern_panel_routes(drive: &mut Drive, source: Source) {
    for (number, name) in [
        (90, "pattern.controls"),
        (91, "pattern.param_prev"),
        (92, "pattern.param_next"),
        (93, "pattern.value_dec"),
        (94, "pattern.value_inc"),
    ] {
        let info = input::ACTIONS
            .iter()
            .find(|info| info.name == name)
            .unwrap();
        assert_eq!(info.group, input::Group::Global);
        assert!(!info.knob);
        learn::assign(
            &mut drive.app.learned,
            "Recorded pads",
            input::Group::Global,
            if matches!(source, Source::Notes) {
                input::midi::Key::Note(number)
            } else {
                input::midi::Key::Cc(number)
            },
            name,
        );
    }
    drive
        .app
        .sync_learned("Recorded pads", input::Group::Global);
    drive.frame();
}

/// Scenario "Adjust a pattern without opening a step grid".
#[test]
fn pattern_panel_keyboard_edits_only_the_shown_setting_on_every_screen() {
    let raw = vec![[121, 0, 90, 0], [245759, 127, 80, 960]];
    for screen in Screen::ALL {
        for close in [
            (KeyCode::Enter, M::NONE),
            (KeyCode::Esc, M::NONE),
            (KeyCode::Char('n'), M::ALT),
        ] {
            let mut drive = app_capture_drive(raw.clone());
            drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48000])), 4);
            drive.app.goto(screen);
            drive.app.edit = true;
            drive.app.pads.bank = 7;
            drive.app.pads.selected = 3;
            let before = drive.app.before();
            let chop = drive.app.chop.as_ref().unwrap().saved();
            let page = drive.app.pads.page().0;
            let settings = (drive.app.settings_row, drive.app.settings_tab);
            drive.tap(KeyCode::Char('n'), M::ALT);
            assert_eq!(drive.app.pattern_panel, Some(0));
            drive.tap(KeyCode::Right, M::NONE);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, 2);
            drive.tap(KeyCode::Char('k'), M::NONE);
            drive.tap(KeyCode::Char('l'), M::NONE);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().quantize, "1/32");
            drive.tap(KeyCode::Down, M::NONE);
            drive.tap(KeyCode::Left, M::NONE);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 90);
            let view = drive.app.pattern_controls_view(&drive.keymap).unwrap();
            assert_eq!(view.selected, 2);
            let values: Vec<&str> = view
                .values
                .iter()
                .map(|(_, value)| value.as_str())
                .collect();
            assert_eq!(values[..3], ["2", "1/32", "90 %"]);
            assert!(
                view.controls[0].contains("I/K select"),
                "{}",
                view.controls[0]
            );
            assert!(
                view.controls[0].contains("J/L change"),
                "{}",
                view.controls[0]
            );
            assert!(view.controls[1].contains("Enter"));
            assert!(drive.app.current_value_at(Instant::now()).is_none());
            drive.tap(close.0, close.1);
            assert!(drive.app.pattern_panel.is_none());
            assert_eq!(drive.app.screen, screen);
            assert_eq!((drive.app.pads.bank, drive.app.pads.selected), (7, 3));
            assert!(drive.app.edit);
            assert_eq!(drive.app.pads.page().0, page);
            assert_eq!((drive.app.settings_row, drive.app.settings_tab), settings);
            assert_eq!(drive.app.chop.as_ref().unwrap().saved(), chop);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, raw);
            for (pad, snapshot) in before.pads.iter().enumerate() {
                assert_eq!(&drive.app.pads.snapshot(pad), snapshot);
            }
            assert_eq!(drive.app.pads.fx, before.fx);
        }
    }
}

#[test]
fn pattern_panel_raw_buttons_require_controls_and_keep_releases_silent() {
    for source in [Source::Notes, Source::Button] {
        for screen in Screen::ALL {
            let mut drive = Drive::offline();
            pattern_panel_routes(&mut drive, source);
            drive.app.goto(screen);
            for number in 91..=94 {
                let frame = drive.registry_command(source, KeyCode::Null, M::NONE, number);
                assert!(drive.app.pads.pattern.is_none());
                assert!(drive.app.pattern_panel.is_none());
                assert!(frame.commands().is_empty());
                assert!(drive.last_notice().text.contains("Open pattern controls"));
            }
            drive.registry_command(source, KeyCode::Null, M::NONE, 90);
            assert_eq!(drive.app.pattern_panel, Some(0));
            let pattern = drive.app.pads.pattern.as_ref().unwrap();
            assert_eq!(
                (pattern.bars, pattern.quantize.as_str(), pattern.strength),
                (2, "1/16", 100)
            );
            for (number, row) in [(92, 1), (92, 2), (91, 1), (91, 0)] {
                drive.registry_command(source, KeyCode::Null, M::NONE, number);
                assert_eq!(drive.app.pattern_panel, Some(row));
            }
            drive.registry_command(source, KeyCode::Null, M::NONE, 94);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, 3);
            drive.registry_command(source, KeyCode::Null, M::NONE, 93);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, 2);
            drive.registry_command(source, KeyCode::Null, M::NONE, 90);
            assert!(drive.app.pattern_panel.is_none());
            assert_eq!(drive.app.screen, screen);
            assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
            assert!(drive.app.history.is_empty());
        }
    }
}

/// Scenario "Set loop length before recording".
#[test]
fn pattern_panel_creates_defaults_clamps_length_and_preserves_hidden_rows() {
    let mut drive = Drive::offline();
    drive.tap(KeyCode::Char('n'), M::ALT);
    let pattern = drive.app.pads.pattern.as_ref().unwrap();
    assert_eq!(
        (pattern.bars, pattern.quantize.as_str(), pattern.strength),
        (2, "1/16", 100)
    );
    assert!(pattern.hits.is_empty());
    for _ in 0..70 {
        drive.tap(KeyCode::Left, M::NONE);
    }
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, 1);
    for _ in 0..70 {
        drive.tap(KeyCode::Right, M::NONE);
    }
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, 64);
    drive.app.apply_session(session::Session {
        pattern: Some("A1".into()),
        patterns: vec![session::PatternState {
            slot: "A1".into(),
            bars: 64,
            quantize: "1/16".into(),
            strength: 100,
            hits: vec![[245759, 127, 80, 960]],
        }],
        ..Default::default()
    });
    settle_pattern(&mut drive);
    drive.tap(KeyCode::Char('n'), M::ALT);
    drive.tap(KeyCode::Left, M::NONE);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[245759, 127, 80, 960]]
    );
    drive.tap(KeyCode::Esc, M::NONE);
    settle_pattern(&mut drive);
    for playing in [false, true] {
        if playing {
            drive.app.send(Command::TransportPlay);
            app_capture_frames(&mut drive, 0);
        }
        drive.tap(KeyCode::Char('r'), M::CONTROL);
        app_capture_frames(&mut drive, 0);
        drive.app.flush_audio();
        drive.tap(KeyCode::Char('n'), M::ALT);
        let before = drive.app.pads.pattern.clone();
        drive.tap(KeyCode::Left, M::NONE);
        assert_eq!(drive.app.pads.pattern, before);
        assert!(
            drive
                .last_notice()
                .text
                .contains(if playing { "Idle" } else { "count-in" }),
            "{}",
            drive.last_notice().text
        );
        drive.tap(KeyCode::Esc, M::NONE);
        drive.tap(KeyCode::Char('r'), M::CONTROL);
        record_controls_idle(&mut drive);
    }
}

#[test]
fn pattern_panel_escape_closes_before_recording_and_keeps_playable_gates() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![]);
        app_capture_sound(
            &mut drive,
            0,
            Arc::new(Sample::mono(vec![0.02; 48000])),
            true,
        );
        app_capture_start(&mut drive, 1);
        drive.tap(KeyCode::Char('n'), M::ALT);
        drive.pad_press(source, 0);
        let sound = app_capture_frames(&mut drive, 128);
        assert!(sound.0.iter().any(|sample| sample.abs() > 0.001));
        assert!(sound.1.iter().any(|sample| sample.abs() > 0.001));
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.pattern_panel.is_none());
        app_capture_frames(&mut drive, 128);
        assert_eq!(
            app_capture_snapshot(&drive).phase,
            engine::capture::Phase::Recording
        );
        drive.pad_release(source, 0);
        app_capture_frames(&mut drive, 128);
        drive.tap(KeyCode::Esc, M::NONE);
        record_controls_idle(&mut drive);
        let hits = &drive.app.pads.pattern.as_ref().unwrap().hits;
        assert_eq!(hits.len(), 1);
        assert!(hits[0][3] > 0);
    }
}

/// Scenario "Pattern takeover follows the current value and device".
#[test]
fn pattern_panel_keyboard_edits_reset_two_devices_pickup_and_keep_scale_independent() {
    use input::midi::Response;
    for response in [Response::Pickup, Response::Scale] {
        let mut drive = app_capture_drive(vec![]);
        pattern_parameter_routes(&mut drive);
        pattern_parameter_knob(&mut drive, PAD_PORT, 102, 64);
        drive.app.takeover = Takeover::with(response);
        for port in [PAD_PORT, XS_PORT] {
            pattern_parameter_knob(&mut drive, port, 102, 64);
        }
        drive.tap(KeyCode::Char('n'), M::ALT);
        drive.tap(KeyCode::Down, M::NONE);
        drive.tap(KeyCode::Down, M::NONE);
        drive.tap(KeyCode::Left, M::NONE);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 40);
        for port in [PAD_PORT, XS_PORT] {
            pattern_parameter_knob(&mut drive, port, 102, 127);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 40);
        }
        if response == Response::Pickup {
            pattern_parameter_knob(&mut drive, PAD_PORT, 102, 51);
        }
        pattern_parameter_knob(&mut drive, PAD_PORT, 102, 0);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 0);
        pattern_parameter_knob(&mut drive, XS_PORT, 102, 64);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 0);
    }
}

#[test]
fn pattern_panel_effective_keys_removals_and_blocking_owners_keep_priority() {
    let mut drive = Drive::offline();
    drive.keymap.overlay("[edit]\nI = 'none'\nUp = 'none'\nJ = 'none'\nLeft = 'none'\n[global]\n'Alt+N' = 'none'\n'Alt+U' = 'pattern.controls'\n").unwrap();
    drive.tap(KeyCode::Char('n'), M::ALT);
    assert!(drive.app.pads.pattern.is_none());
    drive.tap(KeyCode::Char('u'), M::ALT);
    let view = drive.app.pattern_controls_view(&drive.keymap).unwrap();
    assert!(!view.controls[1].contains("Alt+N"));
    assert!(view.controls[1].contains("Alt+U"));
    drive
        .keymap
        .overlay("[global]\n'Alt+I' = 'pattern.param_prev'\n'Alt+J' = 'pattern.value_dec'\n")
        .unwrap();
    let view = drive.app.pattern_controls_view(&drive.keymap).unwrap();
    assert!(view.controls[0].contains("Alt+I"));
    assert!(view.controls[0].contains("Alt+J"));
    drive.tap(KeyCode::Char('i'), M::NONE);
    assert_eq!(drive.app.pattern_panel, Some(0));
    drive.tap(KeyCode::Char('j'), M::NONE);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, 2);
    drive.app.ask = Some(Ask::ClearPads {
        pads: vec![],
        lines: vec!["Clear?".into()],
    });
    assert!(drive.app.pattern_controls_view(&drive.keymap).is_none());
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.ask.is_none());
    assert_eq!(drive.app.pattern_panel, Some(0));
    drive.app.learning = Some(learn::Learning::new(action_info("pattern.controls")));
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.learning.is_none());
    assert_eq!(drive.app.pattern_panel, Some(0));
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.pattern_panel.is_none());
    for blocked in ["name", "filter", "picker", "learn", "restore"] {
        let mut drive = Drive::offline();
        pattern_panel_routes(&mut drive, Source::Button);
        match blocked {
            "name" => drive.app.pad_naming = Some((0, String::new())),
            "filter" => {
                drive.app.goto(Screen::Files);
                drive.app.browser.editing = true;
            }
            "picker" => {
                drive.app.project_picker = Some(ProjectPicker {
                    entries: vec![],
                    selected: 0,
                    naming: None,
                })
            }
            "learn" => drive.app.screen_learn = Some(drive.app.learn_open()),
            "restore" => drive.app.restoring_project = true,
            _ => unreachable!(),
        }
        drive.frame();
        drive.registry_command(Source::Button, KeyCode::Null, M::NONE, 90);
        drive.tap(KeyCode::Char('n'), M::ALT);
        assert!(drive.app.pads.pattern.is_none(), "{blocked}");
        assert!(drive.app.pattern_panel.is_none(), "{blocked}");
    }
}

#[test]
fn pattern_panel_layouts_and_protocols_toggle_once_and_keep_shift_pads() {
    for kitty in [false, true] {
        for letter in ['n', 'N', 'т', 'Т'] {
            let mut drive = Drive::offline();
            for kind in [
                KeyEventKind::Press,
                KeyEventKind::Repeat,
                KeyEventKind::Release,
            ] {
                drive.app.keyboard_event(
                    &drive.keymap,
                    &KeyEvent::new_with_kind(KeyCode::Char(letter), M::ALT, kind),
                    kitty,
                    &mut drive.down,
                    &mut drive.held,
                    &mut drive.mods,
                );
                assert_eq!(drive.app.pattern_panel, Some(0));
            }
            for (code, mods) in [
                (KeyCode::Char('r'), M::NONE),
                (KeyCode::Char('к'), M::SHIFT),
            ] {
                let frame = drive.tap(code, mods);
                assert_eq!(drive.app.pattern_panel, Some(0));
                assert!(
                    frame
                        .commands()
                        .iter()
                        .any(|c| matches!(c, Command::PadPress { pad: 7, .. }))
                );
                assert!(
                    !frame
                        .commands()
                        .iter()
                        .any(|c| matches!(c, Command::PatternRecordStart { .. }))
                );
            }
        }
    }
}

/// Scenario "Learn and retain recording command buttons".
/// Scenario "Pattern recording buttons work on every screen".
#[test]
fn pattern_panel_all_commands_confirm_clear_restart_and_execute_on_every_screen() {
    use input::Group;
    let names = [
        "pattern.controls",
        "pattern.param_prev",
        "pattern.param_next",
        "pattern.value_dec",
        "pattern.value_inc",
        "pattern.record",
        "pattern.erase",
        "pattern.undo",
        "help.page_prev",
        "help.page_next",
    ];
    for source in [Source::Notes, Source::Button] {
        let files = TransitionFiles::new();
        let config_path = files.0.join("pattern-controls.toml");
        std::fs::write(&config_path, "").unwrap();
        let expected = {
            let mut drive = Drive::with_saved_bindings(true, Some(&config_path));
            let other = drive.app.learned["X-Session Pro"].clone();
            drive.tap(KeyCode::F(12), M::NONE);
            drive.tap(KeyCode::Tab, M::NONE);
            drive.tap(KeyCode::Tab, M::NONE);
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
            while drive.app.midi_row < global {
                drive.tap(KeyCode::Down, M::NONE);
            }
            drive.tap(KeyCode::Enter, M::NONE);
            for (index, name) in names.iter().enumerate() {
                let row = drive
                    .app
                    .midi_list()
                    .iter()
                    .position(
                        |row| matches!(row, MidiListRow::Action { info } if info.name == *name),
                    )
                    .unwrap();
                while drive.app.midi_row != row {
                    drive.tap(
                        if drive.app.midi_row < row {
                            KeyCode::Down
                        } else {
                            KeyCode::Up
                        },
                        M::NONE,
                    );
                }
                let number = 90 + index as u8;
                for confirm in [false, true] {
                    let old = drive.app.learned.clone();
                    let bytes = std::fs::read(&config_path).unwrap();
                    drive.tap(KeyCode::Enter, M::NONE);
                    assert_eq!(drive.app.learning.as_ref().unwrap().info.name, *name);
                    drive.registry_command(source, KeyCode::Null, M::NONE, number);
                    assert!(drive.app.learning.as_ref().unwrap().pending.is_some());
                    assert_eq!(drive.app.learned, old);
                    assert_eq!(std::fs::read(&config_path).unwrap(), bytes);
                    assert!(drive.app.pattern_panel.is_none());
                    assert!(drive.app.pads.pattern.is_none());
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
                            config::load_from(&config_path).config.midi,
                            drive.app.learned
                        );
                    } else {
                        assert_eq!(drive.app.learned, old);
                        assert_eq!(std::fs::read(&config_path).unwrap(), bytes);
                    }
                }
                drive.tap(KeyCode::Backspace, M::NONE);
                let key = if matches!(source, Source::Notes) {
                    input::midi::Key::Note(number)
                } else {
                    input::midi::Key::Cc(number)
                };
                assert!(
                    !drive.app.learned["Recorded pads"]["global"].contains_key(&key.to_string())
                );
                assert_eq!(
                    config::load_from(&config_path).config.midi,
                    drive.app.learned
                );
                drive.tap(KeyCode::Enter, M::NONE);
                drive.registry_command(source, KeyCode::Null, M::NONE, number);
                drive.tap(KeyCode::Enter, M::NONE);
                assert_eq!(
                    drive.app.learned["Recorded pads"]["global"][&key.to_string()],
                    *name
                );
            }
            assert_eq!(drive.app.learned["X-Session Pro"], other);
            drive.app.learned.clone()
        };
        let mut drive = Drive::with_saved_bindings(true, Some(&config_path));
        assert_eq!(drive.app.learned, expected);
        for screen in Screen::ALL {
            drive.app.apply_session(session::Session::default());
            settle_pattern(&mut drive);
            drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48000])), 4);
            drive.app.goto(screen);
            drive.app.send(Command::TransportPlay);
            app_capture_frames(&mut drive, 0);
            drive.registry_command(source, KeyCode::Null, M::NONE, 95);
            app_capture_frames(&mut drive, 0);
            drive.app.flush_audio();
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::Recording
            );
            drive.registry_command(source, KeyCode::Null, M::NONE, 90);
            assert_eq!(drive.app.pattern_panel, Some(0));
            drive.registry_command(source, KeyCode::Null, M::NONE, 92);
            drive.registry_command(source, KeyCode::Null, M::NONE, 92);
            drive.registry_command(source, KeyCode::Null, M::NONE, 93);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 90);
            drive.registry_command(source, KeyCode::Null, M::NONE, 91);
            assert_eq!(drive.app.pattern_panel, Some(1));
            drive.registry_command(source, KeyCode::Null, M::NONE, 94);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().quantize, "1/32");
            for erasing in [true, false] {
                drive.registry_command(source, KeyCode::Null, M::NONE, 96);
                app_capture_frames(&mut drive, 0);
                assert_eq!(app_capture_snapshot(&drive).erasing, erasing);
            }
            drive.registry_command(source, KeyCode::Null, M::NONE, 90);
            assert!(drive.app.pattern_panel.is_none());
            drive.registry_command(source, KeyCode::Null, M::NONE, 97);
            record_controls_idle(&mut drive);
            let p = drive.app.pads.pattern.as_ref().unwrap();
            assert_eq!((p.quantize.as_str(), p.strength), ("1/16", 100));
            assert!(p.hits.is_empty());
            assert!(drive.app.history.is_empty());
            assert_eq!(drive.app.screen, screen);
            assert!(drive.app.recording.is_none());
            assert_eq!(drive.app.learned, expected);
            drive.tap(KeyCode::F(1), M::NONE);
            let frame = f1_pages_live(&mut drive, (80, 26), true);
            assert!(drive.app.help_open);
            drive.registry_command(source, KeyCode::Null, M::NONE, 99);
            assert_eq!(
                drive.app.help_overlay.keys_scroll,
                19.min(frame.lines.len().saturating_sub(19))
            );
            drive.registry_command(source, KeyCode::Null, M::NONE, 98);
            assert_eq!(drive.app.help_overlay.keys_scroll, 0);
            drive.tap(KeyCode::F(1), M::NONE);
            assert!(!drive.app.help_open);
        }
    }
}

/// Scenario "Record real pads while an unassigned chop is open".
#[test]
fn pattern_panel_chop_uses_real_pads_and_retains_release_after_controls_close() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for phase in ["controls", "count-in", "recording"] {
            let mut drive = app_capture_drive(vec![]);
            drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48000])), 4);
            app_capture_sound(&mut drive, 0, app_capture_tone(90.0), true);
            app_capture_frames(&mut drive, 0);
            assert!(
                drive
                    .app
                    .chop_display(drive.app.chop.as_ref().unwrap())
                    .slice_keys
                    .is_some()
            );
            if phase != "controls" {
                if phase == "recording" {
                    drive.app.send(Command::TransportPlay);
                    app_capture_frames(&mut drive, 0);
                }
                drive.tap(KeyCode::Char('r'), M::CONTROL);
                app_capture_frames(&mut drive, 0);
                drive.app.flush_audio();
            }
            drive.tap(KeyCode::Char('n'), M::ALT);
            assert!(
                drive
                    .app
                    .chop_display(drive.app.chop.as_ref().unwrap())
                    .slice_keys
                    .is_none()
            );
            drive.pad_press(source, 0);
            let sound = app_capture_frames(&mut drive, 6000);
            assert!(sound.0.iter().any(|v| v.abs() > 0.001));
            drive.tap(KeyCode::Char('8'), M::CONTROL);
            drive.tap(KeyCode::Esc, M::NONE);
            assert!(drive.app.pattern_panel.is_none());
            assert_eq!(
                drive
                    .app
                    .chop_display(drive.app.chop.as_ref().unwrap())
                    .slice_keys
                    .is_some(),
                phase == "controls"
            );
            drive.pad_release(source, 0);
            let tail = app_capture_frames(&mut drive, 4096);
            assert!(
                tail.0[tail.0.len() - 128..]
                    .iter()
                    .all(|v| v.abs() < 0.00001),
                "{source:?}/{phase}: original Gate must release"
            );
            if phase != "controls" {
                drive.tap(KeyCode::Char('r'), M::CONTROL);
                record_controls_idle(&mut drive);
            } else {
                drive.tap(KeyCode::Char('n'), M::ALT);
                drive.tap(KeyCode::Char('r'), M::CONTROL);
                app_capture_frames(&mut drive, 0);
                assert_eq!(
                    app_capture_snapshot(&drive).phase,
                    engine::capture::Phase::CountIn
                );
                assert!(drive.app.pattern_panel.is_some());
                drive.tap(KeyCode::Char('r'), M::CONTROL);
                record_controls_idle(&mut drive);
                drive.tap(KeyCode::Esc, M::NONE);
            }
            let hits = &drive.app.pads.pattern.as_ref().unwrap().hits;
            assert_eq!(hits.len(), usize::from(phase == "recording"));
            if phase == "recording" {
                assert_eq!(hits[0][1], 0);
                assert!(hits[0][3] > 0);
            }
            assert!(
                drive
                    .app
                    .chop_display(drive.app.chop.as_ref().unwrap())
                    .slice_keys
                    .is_some()
            );
        }
    }
}

fn pattern_parameter_routes(drive: &mut Drive) {
    for profile in ["Recorded pads", "X-Session Pro"] {
        for (cc, name) in [
            (100, "pattern.bars"),
            (101, "pattern.quantize"),
            (102, "pattern.strength"),
        ] {
            learn::assign(
                &mut drive.app.learned,
                profile,
                input::Group::Global,
                input::midi::Key::Cc(cc),
                name,
            );
        }
        drive.app.sync_learned(profile, input::Group::Global);
    }
    drive.frame();
}

fn pattern_parameter_knob(drive: &mut Drive, port: &str, cc: u8, value: u8) -> Frame {
    drive.bytes(port, &[0xb0, cc, value]);
    drive.frame()
}

/// Scenario "Absolute pattern knobs use their complete ranges".
#[test]
fn pattern_parameters_raw_ranges_show_applied_values_on_every_screen_without_rewriting_rows() {
    let raw = vec![
        [121, 0, 90, 0],
        [92000, 127, 80, 960],
        [245759, 3, 70, 3840],
    ];
    for screen in Screen::ALL {
        let mut drive = app_capture_drive(raw.clone());
        pattern_parameter_routes(&mut drive);
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48000])), 4);
        drive.app.goto(screen);
        drive.app.pads.bank = 7;
        drive.app.pads.selected = 3;
        let pads: Vec<_> = (0..engine::PADS)
            .map(|p| drive.app.pads.snapshot(p))
            .collect();
        for (name, cc, label, values) in [
            ("pattern.bars", 100, "Bars", ["1", "33", "64"]),
            (
                "pattern.quantize",
                101,
                "Quantize",
                ["Off", "1/32", "1/32T"],
            ),
            (
                "pattern.strength",
                102,
                "Strength",
                ["0 %", "50 %", "100 %"],
            ),
        ] {
            let info = input::ACTIONS.iter().find(|a| a.name == name).unwrap();
            assert_eq!(info.group, input::Group::Global);
            assert!(info.knob);
            pattern_parameter_knob(&mut drive, XS_PORT, cc, 127);
            for (value, expected) in [0, 64, 127].into_iter().zip(values) {
                pattern_parameter_knob(&mut drive, XS_PORT, cc, value);
                app_capture_frames(&mut drive, 0);
                let shown = shown_value(&drive);
                assert_eq!(
                    (
                        shown.object.as_str(),
                        shown.parameter.as_str(),
                        shown.formatted.as_str()
                    ),
                    ("A1", label, expected)
                );
                assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, raw);
                assert_eq!(drive.app.screen, screen);
                assert_eq!((drive.app.pads.bank, drive.app.pads.selected), (7, 3));
                assert!(!drive.app.transport_state().playing);
                assert_eq!(drive.app.transport_state().ticks, 0.0);
                assert!(drive.app.history.is_empty());
            }
        }
        assert_eq!(
            (0..engine::PADS)
                .map(|p| drive.app.pads.snapshot(p))
                .collect::<Vec<_>>(),
            pads
        );
    }
}

#[test]
fn pattern_parameters_every_raw_position_matches_the_discrete_lists() {
    let mut drive = app_capture_drive(vec![[121, 0, 90, 0], [5000, 1, 100, 400]]);
    pattern_parameter_routes(&mut drive);
    let raw = drive.app.pads.pattern.as_ref().unwrap().hits.clone();
    let grids = [
        "off", "1/4", "1/8", "1/16", "1/32", "1/64", "1/8T", "1/16T", "1/32T",
    ];
    for (cc, count) in [(100, 64), (101, 9), (102, 11)] {
        for value in 0..=127u8 {
            pattern_parameter_knob(&mut drive, PAD_PORT, cc, value);
            let index = (usize::from(value) * (count - 1) * 2 + 127) / 254;
            let p = drive.app.pads.pattern.as_ref().unwrap();
            match cc {
                100 => assert_eq!(usize::from(p.bars), index + 1),
                101 => assert_eq!(p.quantize, grids[index]),
                102 => assert_eq!(usize::from(p.strength), index * 10),
                _ => unreachable!(),
            }
            assert_eq!(p.hits, raw);
            app_capture_frames(&mut drive, 0);
        }
    }
}

#[test]
fn pattern_parameters_create_a1_only_after_acceptance_and_do_not_refresh_unchanged_values() {
    use input::midi::Response;
    let mut drive = Drive::offline();
    pattern_parameter_routes(&mut drive);
    drive.app.takeover.response = Response::Pickup;
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 0);
    assert!(drive.app.pads.pattern.is_none());
    assert!(drive.app.value_popup.is_none());
    assert!(drive.app.session_dirty.is_none());
    assert!(drive.last_notice().text.contains("turn"));
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 127);
    let p = drive.app.pads.pattern.as_ref().unwrap();
    assert_eq!(
        (p.slot.as_str(), p.bars, p.quantize.as_str(), p.strength),
        ("A1", 2, "1/16", 100)
    );
    assert!(p.hits.is_empty());
    assert!(drive.app.session_dirty.is_some());
    let at = drive.app.value_popup.as_ref().unwrap().at;
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 127);
    assert_eq!(drive.app.value_popup.as_ref().unwrap().at, at);
    assert!(drive.app.history.is_empty());
}

#[test]
fn pattern_parameters_pickup_is_per_device_and_resets_after_completed_and_active_undo() {
    use input::midi::Response;
    let mut drive = app_capture_drive(vec![]);
    pattern_parameter_routes(&mut drive);
    drive.app.takeover.response = Response::Pickup;
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 0);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 100);
    pattern_parameter_knob(&mut drive, XS_PORT, 102, 127);
    drive.app.pads.pattern.as_mut().unwrap().strength = 50;
    drive.app.reset_pattern_takeover();
    for port in [PAD_PORT, XS_PORT] {
        pattern_parameter_knob(&mut drive, port, 102, 127);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 50);
    }
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 64);
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 0);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 0);
    pattern_parameter_knob(&mut drive, XS_PORT, 102, 64);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().strength,
        0,
        "the other device has not caught zero"
    );
    pattern_parameter_knob(&mut drive, XS_PORT, 102, 0);
    pattern_parameter_knob(&mut drive, XS_PORT, 102, 127);
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 64);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 50);
    app_capture_start(&mut drive, 1);
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 127);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 100);
    app_capture_finish(&mut drive, 1);
    drive.tap(KeyCode::Char('z'), M::ALT);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 50);
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 127);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().strength,
        50,
        "pass Undo resets pickup ownership"
    );
    app_capture_start(&mut drive, 2);
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 0);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 0);
    drive.tap(KeyCode::Char('z'), M::ALT);
    record_controls_idle(&mut drive);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 50);
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 0);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().strength,
        50,
        "active Undo resets pickup ownership"
    );
    assert!(drive.app.history.is_empty());
}

#[test]
fn pattern_parameters_scale_keeps_two_controller_positions_separate() {
    use input::midi::{Response, Takeover};
    let mut drive = app_capture_drive(vec![]);
    pattern_parameter_routes(&mut drive);
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 64);
    drive.app.takeover = Takeover::with(Response::Scale);
    for (port, value, expected) in [
        (PAD_PORT, 0, 50),
        (PAD_PORT, 32, 60),
        (XS_PORT, 127, 60),
        (XS_PORT, 96, 50),
        (PAD_PORT, 48, 60),
        (PAD_PORT, 127, 100),
        (XS_PORT, 0, 0),
    ] {
        pattern_parameter_knob(&mut drive, port, 102, value);
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().strength,
            expected,
            "{port}: {value}"
        );
    }
}

#[test]
fn pattern_parameters_nonfinite_values_are_ignored_and_compilation_failure_keeps_the_model() {
    use input::midi::Param;
    let mut app = tests::test_app();
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        app.midi_event(midi::Event::Param {
            port: "invalid".into(),
            control: "value".into(),
            param: Param::PatternBars,
            value,
            takeover: false,
        });
        assert!(app.pads.pattern.is_none());
        assert!(app.session_dirty.is_none());
        assert!(app.value_popup.is_none());
    }
    for (param, value, expected) in [
        (Param::PatternBars, -1.0, 1),
        (Param::PatternBars, 2.0, 64),
        (Param::PatternBars, 0.5, 33),
    ] {
        app.midi_event(midi::Event::Param {
            port: "boundary".into(),
            control: "value".into(),
            param,
            value,
            takeover: false,
        });
        assert_eq!(app.pads.pattern.as_ref().unwrap().bars, expected);
    }
    app.pads
        .pattern
        .as_mut()
        .unwrap()
        .hits
        .push([0, 128, 80, 0]);
    app.pads.pattern_rows = None;
    let before = app.pads.pattern.clone();
    app.midi_event(midi::Event::Param {
        port: "invalid".into(),
        control: "value".into(),
        param: Param::PatternStrength,
        value: 0.0,
        takeover: false,
    });
    assert_eq!(app.pads.pattern, before);
    assert!(app.pads.pattern_rows.is_none());
    assert_eq!(app.notices.current().unwrap().level, Level::Error);
}

#[test]
fn pattern_parameters_refuse_pending_count_in_finishing_and_stop_before_callback_feedback() {
    for phase in ["pending", "count-in", "end", "stop", "finishing"] {
        let mut drive = app_capture_drive(vec![[121, 0, 90, 0]]);
        pattern_parameter_routes(&mut drive);
        if matches!(phase, "pending" | "count-in") {
            assert!(drive.app.try_send(Command::PatternRecordStart { pass: 1 }));
            if phase == "count-in" {
                app_capture_frames(&mut drive, 0);
            }
        } else {
            app_capture_start(&mut drive, 1);
            drive.app.send(if phase == "stop" {
                Command::TransportStop
            } else {
                Command::PatternRecordEnd { pass: 1 }
            });
            if phase == "finishing" {
                app_capture_frames(&mut drive, 0);
            }
        }
        drive.frame();
        let before = drive.app.pads.pattern.clone();
        for (cc, value) in [(100, 127), (101, 0), (102, 0)] {
            let commands = pattern_parameter_knob(&mut drive, PAD_PORT, cc, value);
            assert_eq!(drive.app.pads.pattern, before, "{phase}/{cc}");
            assert!(drive.app.value_popup.is_none());
            assert!(commands.commands().iter().all(|c| !matches!(
                c,
                Command::PublishRecordedPattern { .. } | Command::PublishCapturedPattern { .. }
            )));
            assert_eq!(drive.last_notice().level, Level::Warn);
            let reason = match phase {
                "pending" => "start",
                "count-in" => "count-in",
                _ => "finishing",
            };
            assert!(
                drive.last_notice().text.contains(reason),
                "{phase}: {}",
                drive.last_notice().text
            );
        }
    }
}

#[test]
fn pattern_parameters_full_queue_retains_grid_publications_before_end_and_refuses_length_in_recording()
 {
    let mut drive = app_capture_drive(vec![[121, 0, 90, 0], [5000, 1, 80, 0]]);
    pattern_parameter_routes(&mut drive);
    app_capture_start(&mut drive, 1);
    let epoch = app_capture_snapshot(&drive).epoch;
    for _ in 0..1024 {
        drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .unwrap();
    }
    pattern_parameter_knob(&mut drive, PAD_PORT, 100, 127);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, 1);
    assert!(drive.app.value_popup.is_none());
    let mut commands = pattern_parameter_knob(&mut drive, PAD_PORT, 101, 0).ui;
    commands.extend(pattern_parameter_knob(&mut drive, PAD_PORT, 102, 64).ui);
    drive.app.send(Command::PatternRecordEnd { pass: 1 });
    commands.extend(std::mem::take(&mut drive.app.sent));
    let settings: Vec<_> = commands
        .iter()
        .filter_map(|c| match c {
            Command::PublishCapturedPattern { pattern, .. } => {
                Some((pattern.grid().to_string(), pattern.strength()))
            }
            Command::PatternRecordEnd { .. } => Some(("end".into(), 0)),
            _ => None,
        })
        .collect();
    assert_eq!(
        settings,
        [("off".into(), 100), ("off".into(), 50), ("end".into(), 0)]
    );
    record_controls_idle(&mut drive);
    let p = drive.app.pads.pattern.as_ref().unwrap();
    assert_eq!((p.quantize.as_str(), p.strength), ("off", 50));
    assert_eq!(p.hits, [[121, 0, 90, 0], [5000, 1, 80, 0]]);
    assert_eq!(drive.app.history.len(), 1);
    drive.tap(KeyCode::Char('z'), M::ALT);
    let p = drive.app.pads.pattern.as_ref().unwrap();
    assert_eq!((p.quantize.as_str(), p.strength), ("1/16", 100));
    pattern_parameter_knob(&mut drive, PAD_PORT, 100, 127);
    app_capture_frames(&mut drive, 0);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, 64);
    assert_eq!(
        drive.app.transport_state().pattern_ticks,
        64 * engine::pattern::TICKS_PER_BAR
    );
}

#[test]
fn pattern_parameters_raw_grid_change_during_a_silent_erase_hold_applies_in_audio_and_keeps_untraversed_rows()
 {
    use engine::pattern::Grid;
    let mut drive = app_capture_drive(vec![[121, 0, 90, 0], [550, 0, 80, 0], [5000, 1, 70, 0]]);
    record_controls_routes(&mut drive, Source::Notes);
    pattern_parameter_routes(&mut drive);
    app_capture_sound(&mut drive, 0, app_capture_tone(90.0), false);
    app_capture_start(&mut drive, 1);
    record_controls_erase(&mut drive, Source::Notes);
    drive.pad_press(Source::Notes, 0);
    let before = app_capture_frames(&mut drive, 7500);
    assert!(before.0.iter().chain(&before.1).all(|v| *v == 0.0));
    assert_eq!(app_capture_snapshot(&drive).erased_rows, 1);
    // since 98c a grid change moves no saved row: playback keeps quantize Off, and the hold
    // erases by saved ticks, so it reaches tick 550
    let changed = pattern_parameter_knob(&mut drive, XS_PORT, 101, 16);
    // the new grid travels to the audio thread for the next captured hits
    assert!(changed.commands().iter().any(|c| matches!(c, Command::PublishCapturedPattern { pattern, .. } if pattern.grid() == Grid::Quarter)));
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().quantize, "1/4");
    let after = app_capture_frames(&mut drive, 12500);
    assert!(after.0.iter().chain(&after.1).all(|v| *v == 0.0));
    assert_eq!(
        app_capture_snapshot(&drive).erased_rows,
        2,
        "the hold traverses the saved tick 550"
    );
    drive.pad_release(Source::Notes, 0);
    record_controls_erase(&mut drive, Source::Notes);
    record_controls_record(&mut drive, Source::Notes);
    record_controls_idle(&mut drive);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[5000, 1, 70, 0]]
    );
    let replay = app_capture_frames(&mut drive, 96_000);
    assert!(
        replay.0.iter().chain(&replay.1).all(|v| *v == 0.0),
        "no erased pad 0 row sounds in the next loop"
    );
    drive.tap(KeyCode::Char('z'), M::ALT);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[121, 0, 90, 0], [550, 0, 80, 0], [5000, 1, 70, 0]]
    );
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().quantize, "1/16");
}

#[test]
fn pattern_parameters_stale_grid_publication_retries_after_erasure_advances_behind_a_full_queue() {
    let mut drive = app_capture_drive(vec![[121, 0, 90, 0], [550, 0, 80, 0], [1500, 1, 70, 0]]);
    record_controls_routes(&mut drive, Source::Notes);
    pattern_parameter_routes(&mut drive);
    app_capture_sound(&mut drive, 1, app_capture_tone(180.0), false);
    app_capture_start(&mut drive, 1);
    record_controls_erase(&mut drive, Source::Notes);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 7500);
    let boundary = app_capture_snapshot(&drive);
    assert_eq!(boundary.erased_rows, 1);
    for _ in 0..1024 {
        drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch: boundary.epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .unwrap();
    }
    // since 98c the grid change moves no saved row: the hold ends near tick 500, before 550
    pattern_parameter_knob(&mut drive, XS_PORT, 101, 16);
    let held = app_capture_frames(&mut drive, 5000);
    assert!(held.0.iter().chain(&held.1).all(|v| *v == 0.0));
    let advanced = app_capture_snapshot(&drive);
    assert_eq!(advanced.erased_rows, 1);
    drive.app.flush_audio();
    app_capture_frames(&mut drive, 0);
    drive.app.flush_audio();
    app_capture_frames(&mut drive, 0);
    drive.pad_release(Source::Notes, 0);
    record_controls_erase(&mut drive, Source::Notes);
    record_controls_record(&mut drive, Source::Notes);
    record_controls_idle(&mut drive);
    let pattern = drive.app.pads.pattern.as_ref().unwrap();
    assert_eq!(pattern.hits, [[550, 0, 80, 0], [1500, 1, 70, 0]]);
    assert_eq!(pattern.quantize, "1/4");
    // pad 1 sounds at its saved tick 1500, not on a 1/4 line at 1440 or 1920
    let loop_ticks = f64::from(pattern.bars) * f64::from(engine::pattern::TICKS_PER_BAR);
    let now = drive.app.transport_state().ticks % loop_ticks;
    let to = |tick: f64| (((tick - now).rem_euclid(loop_ticks)) * 25.0) as usize;
    let before = app_capture_frames(&mut drive, to(1490.0));
    assert!(
        before.0.iter().chain(&before.1).all(|v| *v == 0.0),
        "nothing sounds before the saved tick, also at 1440"
    );
    let onset = app_capture_frames(&mut drive, 500);
    assert!(
        onset.0.iter().chain(&onset.1).any(|v| v.abs() > 0.0001),
        "the row sounds at its saved tick 1500"
    );
    assert!(
        onset
            .0
            .iter()
            .chain(&onset.1)
            .all(|v| v.is_finite() && v.abs() <= 1.0)
    );
}

#[test]
fn pattern_parameters_blocking_modes_keep_knobs_from_creating_or_editing_a1() {
    for mode in [
        "name",
        "filter",
        "picker",
        "path",
        "question",
        "screen-learn",
        "midi-learn",
        "restoring",
    ] {
        let mut drive = Drive::offline();
        pattern_parameter_routes(&mut drive);
        match mode {
            "name" => drive.app.pad_naming = Some((0, "name".into())),
            "filter" => {
                drive.app.screen = Screen::Files;
                drive.app.browser.editing = true;
            }
            "picker" => {
                drive.app.project_picker = Some(crate::ProjectPicker {
                    entries: vec![],
                    selected: 0,
                    naming: None,
                })
            }
            "path" => {
                drive.app.path_picker = Some(PathPicker::open(
                    0,
                    Path::new("target/97-pattern-parameters"),
                ))
            }
            "question" => {
                drive.app.ask = Some(Ask::ClearPads {
                    pads: vec![0],
                    lines: vec!["question".into()],
                })
            }
            "screen-learn" => drive.app.screen_learn = Some(drive.app.learn_open()),
            "midi-learn" => {
                drive.app.learning = Some(learn::Learning::new(action_info("pattern.record")))
            }
            "restoring" => drive.app.restoring_project = true,
            _ => unreachable!(),
        }
        drive.frame();
        for (cc, value) in [(100, 127), (101, 0), (102, 0)] {
            pattern_parameter_knob(&mut drive, XS_PORT, cc, value);
            assert!(drive.app.pads.pattern.is_none(), "{mode}/{cc}");
            assert!(drive.app.value_popup.is_none());
            assert!(drive.app.history.is_empty());
        }
    }
}

#[test]
fn pattern_parameters_project_replacement_resets_pickup_and_audio_loss_keeps_recovered_rows_editable()
 {
    use input::midi::Response;
    use std::sync::atomic::Ordering::Relaxed;
    let mut drive = app_capture_drive(vec![]);
    pattern_parameter_routes(&mut drive);
    drive.app.takeover.response = Response::Pickup;
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 127);
    drive.app.apply_session(session::Session {
        pattern: Some("A1".into()),
        patterns: vec![session::PatternState {
            slot: "A1".into(),
            bars: 1,
            quantize: "1/16".into(),
            strength: 50,
            hits: vec![],
        }],
        ..Default::default()
    });
    settle_pattern(&mut drive);
    pattern_parameter_knob(&mut drive, PAD_PORT, 102, 127);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 50);
    app_capture_sound(&mut drive, 0, app_capture_tone(90.0), true);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Keyboard, 0);
    app_capture_frames(&mut drive, 6000);
    drive
        .app
        .audio
        .as_ref()
        .unwrap()
        .stopped
        .store(true, Relaxed);
    drive.app.takeover.response = Response::Jump;
    let result = pattern_parameter_knob(&mut drive, PAD_PORT, 102, 0);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[0, 0, 127, 240]]
    );
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 0);
    assert!(result.commands().iter().all(|c| !matches!(
        c,
        Command::PublishRecordedPattern { .. } | Command::PublishCapturedPattern { .. }
    )));
    assert!(drive.app.pending_audio.is_empty());
    assert!(
        !drive
            .app
            .audio
            .as_ref()
            .unwrap()
            .pattern_capture
            .blocks_autosave()
    );
}

fn recording_labels(drive: &Drive, state: Option<engine::capture::Snapshot>) -> String {
    view::FrameText::default()
        .badges_for(&drive.app, Instant::now(), state)
        .iter()
        .map(|b| b.normal.clone())
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Scenario "Recording indicators follow accepted audio state".
/// Scenario "Count in becomes recording at tick zero".
#[test]
fn recording_feedback_waits_for_audio_and_advances_without_ui_input() {
    use engine::capture::Phase;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![]);
        record_controls_routes(&mut drive, source);
        record_controls_record(&mut drive, source);
        let labels = recording_labels(&drive, drive.app.pattern_record_state());
        assert!(
            !labels.contains("COUNT") && !labels.contains("REC A1"),
            "{labels}"
        );
        app_capture_frames(&mut drive, 0);
        let first = drive.app.pattern_record_state().unwrap();
        for count in (1..=4).rev() {
            let state = drive.app.pattern_record_state().unwrap();
            assert_eq!((state.phase, state.count_in_beat), (Phase::CountIn, count));
            for size in [(80, 26), (120, 40)] {
                let rows = shared_feedback_rows(&mut drive, size.0, size.1, Instant::now());
                assert!(
                    rows[1].contains(&format!("COUNT {count} A1")),
                    "{}",
                    rows.join("\n")
                );
            }
            app_capture_frames(&mut drive, 24_000);
        }
        let recording = drive.app.pattern_record_state().unwrap();
        assert_eq!(recording.phase, Phase::Recording);
        assert!(recording.ticks.abs() < 0.001);
        let labels = recording_labels(&drive, Some(recording));
        assert!(labels.contains("REC A1 1:1/1 1/16 100%"), "{labels}");
        app_capture_frames(&mut drive, 24_000);
        let next = drive.app.pattern_record_state().unwrap();
        assert!(recording_labels(&drive, Some(next)).contains("REC A1 1:2/1"));
        assert!(
            recording_labels(&drive, Some(first)).contains("COUNT 4 A1"),
            "a frame uses its own coherent snapshot"
        );
        record_controls_record(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        assert!(
            recording_labels(&drive, drive.app.pattern_record_state()).contains("Finishing A1")
        );
        record_controls_idle(&mut drive);
        assert!(!recording_labels(&drive, drive.app.pattern_record_state()).contains("REC A1"));
        drive.app.apply_session(session::Session::default());
        assert!(
            drive.app.pattern_record_state().is_none(),
            "old-project feedback cannot survive replacement"
        );
        assert!(!recording_labels(&drive, drive.app.pattern_record_state()).contains("COUNT"));
        settle_pattern(&mut drive);
        assert_eq!(drive.app.pattern_record_state().unwrap().phase, Phase::Idle);
    }
}

/// Scenario "Erasure remains distinct from sounding pads".
#[test]
fn recording_feedback_erase_targets_follow_banks_and_clear_without_hits() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![]);
        record_controls_routes(&mut drive, source);
        if matches!(source, Source::Button) {
            for (cc, action) in [(80, "pad.1"), (81, "pad.3")] {
                learn::assign(
                    &mut drive.app.learned,
                    "Recorded pads",
                    input::Group::Global,
                    input::midi::Key::Cc(cc),
                    action,
                );
            }
            drive
                .app
                .sync_learned("Recorded pads", input::Group::Global);
            drive.frame();
        }
        let pad_event = |drive: &mut Drive, pad, down| {
            if matches!(source, Source::Button) {
                drive.bytes(
                    PAD_PORT,
                    &[
                        0xb0,
                        if pad == 0 { 80 } else { 81 },
                        if down { 127 } else { 0 },
                    ],
                );
                drive.frame();
            } else if down {
                drive.pad_press(source, pad);
            } else {
                drive.pad_release(source, pad);
            }
        };
        for pad in [0, 18] {
            app_capture_sound(
                &mut drive,
                pad,
                Arc::new(Sample::mono(vec![0.05; 48_000])),
                false,
            );
        }
        drive.app.send(Command::TransportPlay);
        app_capture_frames(&mut drive, 0);
        record_controls_record(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        record_controls_erase(&mut drive, source);
        pad_event(&mut drive, 0, true);
        app_capture_frames(&mut drive, 0);
        drive.tap(KeyCode::PageDown, M::NONE);
        pad_event(&mut drive, 2, true);
        let silent = app_capture_frames(&mut drive, 128);
        assert!(silent.0.iter().chain(&silent.1).all(|x| *x == 0.0));
        let state = drive.app.pattern_record_state().unwrap();
        assert_eq!(state.erase_targets, [(1 << 0) | (1 << 18), 0]);
        assert_eq!(state.captured_onsets, 0);
        for hidden in [false, true] {
            drive.app.ui.hide_pads = hidden;
            let mut pads = pads_view(&mut drive.app.pads, None, None, vec![], None, 48_000);
            pads.pads[2].flash = true;
            let pads = drive.app.mark_pads_recording(pads, Some(state));
            assert_eq!(pads.feedback.corners[2], Some(tui::PadCorner::Erase));
            assert!(!pads.pads[2].flash);
            assert_eq!(pads.hidden, hidden);
            let full = drive
                .app
                .full_help_recording_keys(&drive.keymap, true, Some(state));
            assert!(
                full.iter().any(|(label, text)| label == "Erase targets"
                    && text.contains("1-1")
                    && text.contains("2-3")),
                "{full:?}"
            );
            let labels = recording_labels(&drive, Some(state));
            assert!(
                labels.contains("ERASE A1") && labels.contains("1-1+1"),
                "{labels}"
            );
        }
        pad_event(&mut drive, 0, false);
        app_capture_frames(&mut drive, 0);
        assert_eq!(
            drive.app.pattern_record_state().unwrap().erase_targets,
            [1 << 18, 0],
            "{source:?} release must keep the independent held target"
        );
        record_controls_erase(&mut drive, source);
        let silent = app_capture_frames(&mut drive, 128);
        assert!(silent.0.iter().chain(&silent.1).all(|x| *x == 0.0));
        let state = drive.app.pattern_record_state().unwrap();
        assert!(!state.erasing && state.erase_targets == [0, 0]);
        let pads = pads_view(&mut drive.app.pads, None, None, vec![], None, 48_000);
        assert!(
            drive
                .app
                .mark_pads_recording(pads, Some(state))
                .feedback
                .corners
                .iter()
                .all(Option::is_none)
        );
        pad_event(&mut drive, 2, false);
        pad_event(&mut drive, 2, true);
        let audible = app_capture_frames(&mut drive, 128);
        assert!(audible.0.iter().chain(&audible.1).any(|x| x.abs() > 0.001));
        assert_eq!(drive.app.pattern_record_state().unwrap().captured_onsets, 1);
    }
}

/// Scenario "Recording help changes with the accepted phase".
#[test]
fn recording_feedback_phase_help_and_f1_keep_effective_keys_and_fallbacks() {
    let mut drive = app_capture_drive(vec![]);
    record_controls_routes(&mut drive, Source::Keyboard);
    for screen in Screen::ALL {
        drive.app.screen = screen;
        for kitty in [false, true] {
            let full = drive.app.full_help_keys(&drive.keymap, kitty);
            for (key, text) in [
                ("Ctrl+R", "record A1"),
                ("Alt+R", "record the microphone"),
                ("Ctrl+T", "record the output"),
            ] {
                assert!(
                    full.iter()
                        .any(|(keys, label)| keys.split_whitespace().any(|k| k == key)
                            && label.contains(text)),
                    "{screen:?}: {full:?}"
                );
            }
            assert!(
                full.iter()
                    .any(|(_, text)| text.contains("finishes the pass")
                        && text.contains("playback running"))
            );
            assert!(full.iter().any(|(_, text)| if kitty {
                text.contains("Hold pad targets silently")
            } else {
                text.contains("one loop per pad press")
            }));
            assert!(full.iter().any(|(keys, text)| keys.contains("Alt+R")
                && text.contains(if kitty { "hold" } else { "toggle" })));
        }
    }
    drive.app.screen = Screen::Pads;
    assert!(drive.keymap.overlay("[global]\n'Ctrl+R' = 'none'\n'Alt+R' = 'none'\n'Ctrl+T' = 'none'\nF8 = 'pattern.record'\nF9 = 'record.mic'\nF11 = 'record.output'\n").unwrap().is_empty());
    for kitty in [false, true] {
        let full = drive.app.full_help_keys(&drive.keymap, kitty);
        for (key, text) in [
            ("F8", "record A1"),
            ("F9", "record the microphone"),
            ("F11", "record the output"),
        ] {
            assert!(
                full.iter()
                    .any(|(keys, label)| keys.split_whitespace().any(|k| k == key)
                        && label.contains(text)),
                "{full:?}"
            );
        }
        assert!(!full.iter().any(|(keys, _)| {
            keys.split_whitespace()
                .any(|k| ["Ctrl+R", "Alt+R", "Ctrl+T"].contains(&k))
        }));
    }
    drive.tap(KeyCode::F(8), M::NONE);
    app_capture_frames(&mut drive, 0);
    let help = |drive: &Drive| {
        drive.app.pattern_help_groups(
            &drive.keymap,
            vec![],
            drive.app.pattern_record_state(),
            true,
        )
    };
    assert!(
        help(&drive)[0]
            .items
            .iter()
            .any(|(_, text)| text == "cancel count")
    );
    app_capture_frames(&mut drive, 96_000);
    assert!(
        help(&drive)[0]
            .items
            .iter()
            .any(|(key, text)| key == "F8" && text == "end A1")
    );
    record_controls_erase(&mut drive, Source::Keyboard);
    app_capture_frames(&mut drive, 0);
    assert!(
        help(&drive)[0]
            .items
            .iter()
            .any(|(_, text)| text == "erase: hold")
    );
    drive.tap(KeyCode::Char('n'), M::ALT);
    assert!(
        help(&drive)[0]
            .items
            .iter()
            .any(|(_, text)| text == "close controls")
    );
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(
        help(&drive)[0]
            .items
            .iter()
            .any(|(_, text)| text == "erase: hold")
    );
}

/// Scenario "Erase one loop without keyboard releases".
#[test]
fn recording_feedback_legacy_erase_hint_expiry_and_return_to_one_shot() {
    let mut drive = app_capture_drive(vec![[0, 0, 90, 0], [0, 1, 70, 0]]);
    record_controls_routes(&mut drive, Source::Keyboard);
    app_capture_sound(
        &mut drive,
        0,
        Arc::new(Sample::mono(vec![0.05; 48_000])),
        false,
    );
    drive.app.pads.track_keys = false;
    drive.app.send(Command::TransportPlay);
    app_capture_frames(&mut drive, 0);
    let legacy = |drive: &mut Drive, code, modifiers| {
        let key = KeyEvent::new(code, modifiers);
        assert!(!drive.app.keyboard_event(
            &drive.keymap,
            &key,
            false,
            &mut drive.down,
            &mut drive.held,
            &mut drive.mods
        ));
        drive.frame();
    };
    legacy(&mut drive, KeyCode::Char('r'), M::CONTROL);
    app_capture_frames(&mut drive, 0);
    legacy(&mut drive, KeyCode::Char('e'), M::ALT);
    legacy(&mut drive, KeyCode::Char('1'), M::NONE);
    app_capture_frames(&mut drive, 0);
    let state = drive.app.pattern_record_state().unwrap();
    assert_eq!(state.erase_targets, [1, 0]);
    let help = drive
        .app
        .pattern_help_groups(&drive.keymap, vec![], Some(state), false);
    assert!(
        help[0]
            .items
            .iter()
            .any(|(_, text)| text == "erase: 1 loop")
    );
    let full = drive.app.full_help_keys(&drive.keymap, false);
    assert!(
        full.iter()
            .any(|(_, text)| text.contains("one loop per pad press"))
    );
    let silent = app_capture_frames(&mut drive, 96_000);
    assert!(silent.0.iter().chain(&silent.1).all(|x| *x == 0.0));
    let state = drive.app.pattern_record_state().unwrap();
    assert_eq!(state.erase_targets, [0, 0]);
    assert_eq!(state.captured_onsets, 0);
    let pads = pads_view(&mut drive.app.pads, None, None, vec![], None, 48_000);
    assert!(
        drive
            .app
            .mark_pads_recording(pads, Some(state))
            .feedback
            .corners
            .iter()
            .all(Option::is_none)
    );
    legacy(&mut drive, KeyCode::Char('e'), M::ALT);
    app_capture_frames(&mut drive, 0);
    assert!(!drive.app.pattern_record_state().unwrap().erasing);
    legacy(&mut drive, KeyCode::Char('1'), M::NONE);
    let audible = app_capture_frames(&mut drive, 128);
    assert!(audible.0.iter().chain(&audible.1).any(|x| x.abs() > 0.001));
    assert_eq!(drive.app.pattern_record_state().unwrap().captured_onsets, 1);
    legacy(&mut drive, KeyCode::Char('r'), M::CONTROL);
    record_controls_idle(&mut drive);
    let hits = &drive.app.pads.pattern.as_ref().unwrap().hits;
    assert!(hits.contains(&[0, 1, 70, 0]));
    assert_eq!(hits.iter().filter(|hit| hit[1] == 0).count(), 1);
    assert!(
        hits.iter().all(|hit| hit[3] == 0),
        "ordinary legacy input remains one-shot"
    );
}

fn record_controls_routes(drive: &mut Drive, source: Source) {
    pattern_undo_routes(drive, source);
    for (number, name) in [(108, "pattern.record"), (107, "pattern.erase")] {
        learn::assign(
            &mut drive.app.learned,
            "Recorded pads",
            input::Group::Global,
            if matches!(source, Source::Notes) {
                input::midi::Key::Note(number)
            } else {
                input::midi::Key::Cc(number)
            },
            name,
        );
    }
    drive
        .app
        .sync_learned("Recorded pads", input::Group::Global);
    drive.frame();
}

fn record_controls_record(drive: &mut Drive, source: Source) -> Frame {
    drive.registry_command(source, KeyCode::Char('r'), M::CONTROL, 108)
}

fn record_controls_erase(drive: &mut Drive, source: Source) -> Frame {
    drive.registry_command(source, KeyCode::Char('e'), M::ALT, 107)
}

fn record_controls_idle(drive: &mut Drive) {
    for _ in 0..16 {
        drive.app.flush_audio();
        app_capture_frames(drive, 0);
        drive.frame();
        if app_capture_snapshot(drive).phase == engine::capture::Phase::Idle {
            drive.app.flush_audio();
            if !drive
                .app
                .audio
                .as_ref()
                .unwrap()
                .pattern_capture
                .blocks_autosave()
            {
                return;
            }
        }
    }
    panic!(
        "recording did not finish: {:?}",
        app_capture_snapshot(drive)
    );
}

/// Scenario "Ctrl R records the pattern on every screen".
/// Scenario "Learned pattern recording never changes into microphone capture".
/// Scenario "Pattern recording does not start microphone capture".
#[test]
fn record_controls_start_and_end_a1_on_every_screen_without_audio_capture() {
    use engine::capture::Phase;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for screen in Screen::ALL {
            for playing in [false, true] {
                let mut drive = app_capture_drive(vec![]);
                drive.app.apply_session(session::Session::default());
                settle_pattern(&mut drive);
                record_controls_routes(&mut drive, source);
                drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48_000])), 4);
                if screen != Screen::Chop {
                    let key = match screen {
                        Screen::Pads => 2,
                        Screen::Files => 6,
                        Screen::Fx => 4,
                        Screen::Settings => 12,
                        Screen::Chop => unreachable!(),
                    };
                    drive.tap(KeyCode::F(key), M::NONE);
                }
                drive.app.pads.bank = 7;
                drive.app.pads.selected = 3;
                if playing {
                    drive.app.send(Command::TransportPlay);
                    app_capture_frames(&mut drive, 6000);
                }
                let clock = drive.app.transport_state();
                let hand = drive.app.browser.in_hand.clone();
                let start = record_controls_record(&mut drive, source);
                assert_eq!(
                    start
                        .commands()
                        .iter()
                        .filter(|c| matches!(c, Command::PatternRecordStart { .. }))
                        .count(),
                    1
                );
                app_capture_frames(&mut drive, 0);
                assert_eq!(
                    app_capture_snapshot(&drive).phase,
                    if playing {
                        Phase::Recording
                    } else {
                        Phase::CountIn
                    }
                );
                let pattern = drive.app.pads.pattern.as_ref().unwrap();
                assert_eq!(
                    (
                        pattern.slot.as_str(),
                        pattern.bars,
                        pattern.quantize.as_str(),
                        pattern.strength
                    ),
                    ("A1", 2, "1/16", 100)
                );
                assert!(pattern.hits.is_empty());
                assert_eq!(drive.app.screen, screen);
                assert_eq!((drive.app.pads.bank, drive.app.pads.selected), (7, 3));
                assert!(drive.app.recording.is_none());
                assert!(
                    !drive
                        .app
                        .audio
                        .as_ref()
                        .unwrap()
                        .recorder
                        .recording
                        .load(std::sync::atomic::Ordering::Relaxed)
                );
                assert_eq!(drive.app.browser.in_hand, hand);
                if playing {
                    assert_eq!(drive.app.transport_state().ticks, clock.ticks);
                    drive.pad_press(source, 0);
                    app_capture_frames(&mut drive, 128);
                    drive.pad_release(source, 0);
                }
                record_controls_record(&mut drive, source);
                record_controls_idle(&mut drive);
                assert_eq!(drive.app.screen, screen);
                assert_eq!(drive.app.transport_state().playing, playing);
                assert_eq!(
                    drive.app.pads.pattern.as_ref().unwrap().hits.len(),
                    usize::from(playing)
                );
                if playing {
                    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits[0][1], 112);
                }
                assert_eq!(drive.app.history.len(), usize::from(playing));
            }
        }
    }
}

#[test]
fn record_controls_pending_start_ends_once_and_full_queue_retains_its_order() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for full in [false, true] {
            let mut drive = app_capture_drive(vec![]);
            record_controls_routes(&mut drive, source);
            if full {
                let epoch = app_capture_snapshot(&drive).epoch;
                for _ in 0..1024 {
                    drive
                        .app
                        .audio
                        .as_mut()
                        .unwrap()
                        .commands
                        .push(ProjectCommand {
                            epoch,
                            ready_at_send: true,
                            command: ProjectMessage::Command(Command::SetPadGain {
                                pad: 127,
                                gain: 1.0,
                            }),
                        })
                        .unwrap();
                }
            }
            let mut commands = record_controls_record(&mut drive, source).ui;
            commands.extend(record_controls_record(&mut drive, source).ui);
            commands.extend(record_controls_record(&mut drive, source).ui);
            let ordered: Vec<_> = commands
                .iter()
                .filter_map(|c| match c {
                    Command::PatternRecordStart { pass } => Some(("start", *pass)),
                    Command::PatternRecordEnd { pass } => Some(("end", *pass)),
                    _ => None,
                })
                .collect();
            assert_eq!(ordered, [("start", 1), ("end", 1)], "{source:?}/{full}");
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::Idle
            );
            record_controls_idle(&mut drive);
            assert!(!drive.app.pattern_owns_pads());
            assert!(drive.app.history.is_empty());
            record_controls_record(&mut drive, source);
            app_capture_frames(&mut drive, 0);
            assert_eq!(app_capture_snapshot(&drive).pass, 2);
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::CountIn
            );
        }
    }
}

/// Scenario "Audio capture refuses a pattern start without changing targets".
#[test]
fn record_controls_audio_capture_refuses_start_before_creating_a1() {
    use std::sync::atomic::Ordering::Relaxed;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for audio_source in [audio::SOURCE_MIC, audio::SOURCE_OUTPUT] {
            let mut drive = app_capture_drive(vec![]);
            drive.app.apply_session(session::Session::default());
            settle_pattern(&mut drive);
            record_controls_routes(&mut drive, source);
            drive
                .app
                .audio
                .as_ref()
                .unwrap()
                .recorder
                .start(audio_source);
            let recording = (audio_source, Instant::now());
            drive.app.recording = Some(recording);
            let before = drive.app.pads.pattern_rows.clone();
            let commands = record_controls_record(&mut drive, source);
            assert!(
                !commands
                    .commands()
                    .iter()
                    .any(|c| matches!(c, Command::PatternRecordStart { .. }))
            );
            assert!(drive.app.pads.pattern.is_none());
            assert_eq!(drive.app.recording, Some(recording));
            assert!(
                drive
                    .app
                    .audio
                    .as_ref()
                    .unwrap()
                    .recorder
                    .recording
                    .load(Relaxed)
            );
            assert!(
                drive
                    .last_notice()
                    .text
                    .contains(if audio_source == audio::SOURCE_MIC {
                        "Microphone"
                    } else {
                        "Output"
                    })
            );
            let refusal = drive.last_notice().text.clone();
            assert_eq!(
                drive.app.notice().as_deref(),
                Some(refusal.as_str()),
                "the refusal must be visible above the recording duration"
            );
            assert_eq!(drive.app.pads.pattern_rows, before);
            if audio_source == audio::SOURCE_OUTPUT {
                drive.tap(KeyCode::Char('r'), M::ALT);
                assert_eq!(
                    drive.app.recording,
                    Some(recording),
                    "a rejected mic release cannot stop output"
                );
            } else {
                drive.tap(KeyCode::Char('t'), M::CONTROL);
                assert_eq!(drive.app.recording, Some(recording));
            }
            assert!(drive.app.pads.pattern.is_none());
            assert!(drive.app.history.is_empty());
        }
    }
}

/// Scenario "Pattern recording refuses microphone and output starts".
#[test]
fn record_controls_pattern_phases_refuse_audio_capture_and_keep_the_pass() {
    use std::sync::atomic::Ordering::Relaxed;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for phase in ["start-pending", "count-in", "recording", "finishing"] {
            let mut drive = app_capture_drive(vec![]);
            record_controls_routes(&mut drive, source);
            if matches!(phase, "recording" | "finishing") {
                drive.app.send(Command::TransportPlay);
                app_capture_frames(&mut drive, 0);
            }
            record_controls_record(&mut drive, source);
            if phase != "start-pending" {
                app_capture_frames(&mut drive, 0);
            }
            if phase == "finishing" {
                record_controls_record(&mut drive, source);
                app_capture_frames(&mut drive, 0);
            }
            let accepted = app_capture_snapshot(&drive);
            let mic = drive.tap(KeyCode::Char('r'), M::ALT);
            assert!(
                mic.commands()
                    .iter()
                    .all(|c| !matches!(c, Command::PatternRecordEnd { .. }))
            );
            assert!(
                drive.last_notice().text.contains("finish the pattern pass"),
                "{phase}"
            );
            drive.tap(KeyCode::Char('t'), M::CONTROL);
            assert_eq!(app_capture_snapshot(&drive), accepted);
            assert!(drive.app.recording.is_none());
            assert!(
                !drive
                    .app
                    .audio
                    .as_ref()
                    .unwrap()
                    .recorder
                    .recording
                    .load(Relaxed)
            );
            assert!(drive.app.pattern_owns_pads());
        }
    }
}

/// Scenario "Raw recording command precedes its following hit".
#[test]
fn record_controls_raw_start_and_end_order_pad_input_before_a_redraw() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![]);
        record_controls_routes(&mut drive, source);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(0);
        }
        drive.app.send(Command::TransportPlay);
        app_capture_frames(&mut drive, 0);
        for (number, value) in [
            (108, 127),
            (108, 0),
            (
                if matches!(source, Source::Notes) {
                    36
                } else {
                    80
                },
                83,
            ),
            (
                if matches!(source, Source::Notes) {
                    36
                } else {
                    80
                },
                0,
            ),
            (108, 127),
            (108, 0),
            (
                if matches!(source, Source::Notes) {
                    37
                } else {
                    80
                },
                99,
            ),
            (
                if matches!(source, Source::Notes) {
                    37
                } else {
                    80
                },
                0,
            ),
        ] {
            drive.bytes(
                PAD_PORT,
                &[
                    if matches!(source, Source::Notes) {
                        0x90
                    } else {
                        0xb0
                    },
                    number,
                    value,
                ],
            );
        }
        let queued = drive.frame();
        app_capture_frames(&mut drive, 128);
        record_controls_idle(&mut drive);
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[0, 0, 83, 0]],
            "{source:?}, commands {:?}, snapshot {:?}, input {:?}",
            queued.commands(),
            app_capture_snapshot(&drive),
            drive.app.input_log
        );
        assert_eq!(drive.app.history.len(), 1);
        assert!(drive.app.recording.is_none());
    }
}

#[test]
fn raw_erasure_release_boundary_does_not_depend_on_ui_deferral() {
    let mut failures = Vec::new();
    for source in [Source::Notes, Source::Button] {
        for (batch, off, explicit_off) in [
            (true, true, false),
            (true, false, false),
            (false, true, false),
            (true, true, true),
        ] {
            if explicit_off && matches!(source, Source::Button) {
                continue;
            }
            let mut drive = app_capture_drive(vec![[0, 0, 90, 0], [240, 0, 70, 0]]);
            record_controls_routes(&mut drive, source);
            app_capture_start(&mut drive, 1);
            let status = if matches!(source, Source::Notes) {
                0x90
            } else {
                0xb0
            };
            let pad = if matches!(source, Source::Notes) {
                36
            } else {
                80
            };
            if matches!(source, Source::Button) {
                drive.bind_pad_button(0);
            }
            for value in [127, 0] {
                drive.bytes(PAD_PORT, &[status, 107, value]);
            }
            if !batch {
                drive.frame();
                app_capture_frames(&mut drive, 0);
            }
            drive.bytes(PAD_PORT, &[status, pad, 127]);
            drive.frame();
            app_capture_frames(&mut drive, 6000);
            assert_eq!(app_capture_snapshot(&drive).erase_targets, [1, 0]);
            drive.bytes(
                PAD_PORT,
                &[
                    if explicit_off && matches!(source, Source::Notes) {
                        0x80
                    } else {
                        status
                    },
                    pad,
                    0,
                ],
            );
            if off {
                for value in [127, 0] {
                    drive.bytes(PAD_PORT, &[status, 107, value]);
                }
            }
            drive.midi().recorded_fence();
            app_capture_frames(&mut drive, 0);
            let targets = app_capture_snapshot(&drive).erase_targets;
            app_capture_frames(&mut drive, 128);
            let erased = app_capture_snapshot(&drive).erased_rows;
            let events: Vec<_> = drive.midi().events.try_iter().collect();
            let releases: Vec<_> = events
                .iter()
                .filter_map(|event| match &event.event {
                    midi::Event::PadRelease { abs, direct } => {
                        Some(format!("audio pad={abs} direct={direct}"))
                    }
                    midi::Event::UiPadRelease { abs, id } => Some(format!("ui pad={abs} id={id}")),
                    _ => None,
                })
                .collect();
            if targets != [0, 0] || erased != 1 {
                failures.push(format!(
                    "{source:?} batch={batch} queued_off={off} explicit_off={explicit_off} targets={targets:?} erased={erased} releases={releases:?}"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "release crossed tick 240: {failures:?}"
    );
}

/// Scenario "Raw erasure command precedes its target hold".
/// Scenario "Delayed UI cannot extend an erase hold".
#[test]
fn raw_erasure_keeps_its_audio_interval_through_delayed_ui_and_pass_undo() {
    let sound = app_capture_tone(55.0);
    let mut reference_engine = engine::Engine::with_sounds(48_000, vec![], vec![]);
    reference_engine.handle(
        0,
        Command::SetPadSound {
            pad: 0,
            sound: Some(sound.clone()),
        },
    );
    let mut reference = (vec![0.0; 288_000], vec![0.0; 288_000]);
    for lap in 0..3 {
        let start = lap * 96_000;
        reference_engine.handle(
            0,
            Command::PadPress {
                pad: 0,
                velocity: 70.0 / 127.0,
            },
        );
        reference_engine.process(
            &mut reference.0[start..start + 96_000],
            &mut reference.1[start..start + 96_000],
        );
    }
    assert!(reference.0.iter().any(|sample| sample.abs() > 0.001));
    assert_ne!(reference.0, reference.1);
    for source in [Source::Notes, Source::Button] {
        for screen in Screen::ALL {
            let original = vec![
                [0, 0, 90, 0],
                [239, 0, 80, 0],
                [240, 0, 70, 0],
                [239, 1, 100, 0],
                [5000, 0, 100, 0],
            ];
            let mut drive = app_capture_drive(original.clone());
            drive.prepare_chop(sound.clone(), 4);
            drive.app.goto(screen);
            record_controls_routes(&mut drive, source);
            pattern_parameter_routes(&mut drive);
            pattern_parameter_knob(&mut drive, PAD_PORT, 101, 0);
            app_capture_sound(&mut drive, 0, sound.clone(), false);
            if matches!(source, Source::Button) {
                drive.bind_pad_button(0);
            }
            let before = (
                drive.app.screen,
                drive.app.pads.bank,
                drive.app.pads.selected,
            );
            drive.app.send(Command::TransportPlay);
            app_capture_frames(&mut drive, 0);
            record_controls_record(&mut drive, source);
            app_capture_frames(&mut drive, 0);
            let status = if matches!(source, Source::Notes) {
                0x90
            } else {
                0xb0
            };
            let pad = if matches!(source, Source::Notes) {
                36
            } else {
                80
            };
            // The target follows its command before any UI event is consumed.
            for (number, value) in [(107, 127), (107, 0), (pad, 127)] {
                drive.bytes(PAD_PORT, &[status, number, value]);
            }
            let batch = drive.frame();
            assert_eq!(
                batch
                    .commands()
                    .iter()
                    .filter(|command| matches!(
                        command,
                        Command::PatternErase { enabled: true, .. }
                    ))
                    .count(),
                1,
                "{source:?} {screen:?}"
            );
            let silent = app_capture_frames(&mut drive, 6000);
            assert!(
                silent
                    .0
                    .iter()
                    .chain(&silent.1)
                    .all(|sample| *sample == 0.0)
            );
            let held = app_capture_snapshot(&drive);
            assert!(held.erasing);
            assert_eq!(held.erase_targets, [1, 0]);
            assert_eq!((held.captured_onsets, held.erased_rows), (0, 2));
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, original);

            for (number, value) in [(pad, 0), (107, 127), (107, 0)] {
                drive.bytes(PAD_PORT, &[status, number, value]);
            }
            drive.midi().recorded_fence();
            // Only the callback runs. Do not collect or replay the direct release.
            app_capture_frames(&mut drive, 0);
            let released = app_capture_snapshot(&drive);
            assert!(released.erasing, "erasure-off still waits for the UI");
            assert_eq!(released.erase_targets, [0, 0], "{source:?} {screen:?}");
            let delayed = app_capture_frames(&mut drive, 288_000);
            let compare = |actual: &Stereo, frames: usize| {
                for (channel, (actual, expected)) in
                    [(&actual.0, &reference.0), (&actual.1, &reference.1)]
                        .into_iter()
                        .enumerate()
                {
                    assert_eq!(actual.len(), frames);
                    for (frame, (actual, expected)) in actual.iter().zip(expected).enumerate() {
                        assert!(actual.is_finite() && actual.abs() <= 1.0);
                        assert!(
                            (actual - expected).abs() <= 0.000001,
                            "{source:?} {screen:?} channel {channel} frame {frame}: {actual} != {expected}"
                        );
                    }
                }
            };
            compare(&delayed, 288_000);
            let delayed_state = app_capture_snapshot(&drive);
            assert_eq!(
                (delayed_state.captured_onsets, delayed_state.erased_rows),
                (0, 2)
            );
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, original);
            drive.frame();
            drive.app.flush_audio();
            app_capture_frames(&mut drive, 0);
            drive.app.flush_audio();
            let expected = vec![original[2], original[3], original[4]];
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, expected);
            assert!(!app_capture_snapshot(&drive).erasing);
            compare(&app_capture_frames(&mut drive, 96_000), 96_000);
            assert_eq!(app_capture_snapshot(&drive).captured_onsets, 0);
            record_controls_record(&mut drive, source);
            record_controls_idle(&mut drive);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, expected);
            assert_eq!(drive.app.history.len(), 1);
            global_pattern_undo(&mut drive, source);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, original);
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().quantize, "off");
            assert_eq!(
                (
                    drive.app.screen,
                    drive.app.pads.bank,
                    drive.app.pads.selected
                ),
                before
            );
            assert!(drive.app.history.is_empty());
        }
    }
}

/// Scenario "Stale controller work cannot enter another recording pass".
#[test]
fn raw_erasure_old_project_envelopes_and_releases_cannot_change_a_new_pass() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![[0, 0, 90, 0]]);
        record_controls_routes(&mut drive, source);
        app_capture_start(&mut drive, 1);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(0);
            drive.bind_action(81, "pad.2");
        }
        let status = if matches!(source, Source::Notes) {
            0x90
        } else {
            0xb0
        };
        let pad = if matches!(source, Source::Notes) {
            36
        } else {
            80
        };
        for (key, value) in [(107, 127), (107, 0), (pad, 127)] {
            drive.bytes(PAD_PORT, &[status, key, value]);
        }
        let old = drive.queued_events();
        assert!(old.iter().any(|event| matches!(
            event.event,
            midi::Event::PadHit {
                sound: midi::PadSound::UiUnattempted { deferred: true },
                ..
            }
        )));
        let epoch = app_capture_snapshot(&drive).epoch;
        drive.app.apply_session(session::Session::default());
        settle_pattern(&mut drive);
        app_capture_start(&mut drive, 1);
        assert!(app_capture_snapshot(&drive).epoch > epoch);
        record_controls_erase(&mut drive, source);
        drive.bytes(PAD_PORT, &[status, pad + 1, 127]);
        drive.frame();
        app_capture_frames(&mut drive, 0);
        let before = app_capture_snapshot(&drive);
        assert_eq!(before.erase_targets, [2, 0]);
        let rows = drive.app.pads.pattern.clone();
        for event in old {
            drive.app.midi_ui_event(event);
        }
        drive.bytes(PAD_PORT, &[status, pad, 0]);
        drive.frame();
        app_capture_frames(&mut drive, 128);
        let after = app_capture_snapshot(&drive);
        assert_eq!(
            (
                after.pass,
                after.phase,
                after.erasing,
                after.erase_targets,
                after.captured_onsets
            ),
            (before.pass, before.phase, true, [2, 0], 0),
            "{source:?}: stale envelopes and release"
        );
        assert_eq!(drive.app.pads.pattern, rows);
        drive.bytes(PAD_PORT, &[status, pad + 1, 0]);
        drive.frame();
        app_capture_frames(&mut drive, 0);
        assert_eq!(app_capture_snapshot(&drive).erase_targets, [0, 0]);
        app_capture_finish(&mut drive, 1);
        app_capture_start(&mut drive, 2);
        drive.pad_press(source, 2);
        app_capture_frames(&mut drive, 128);
        drive.pad_release(source, 2);
        app_capture_finish(&mut drive, 2);
        // the block at tick 5 lands on the 1/16 line at 0 (98c)
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[0, 2, 127, 0]]
        );
    }
}

#[test]
fn raw_erasure_old_pass_release_cannot_close_a_new_pass_target_on_the_same_pad() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![[240, 0, 90, 0]]);
        record_controls_routes(&mut drive, source);
        app_capture_start(&mut drive, 1);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(0);
        }
        let status = if matches!(source, Source::Notes) {
            0x90
        } else {
            0xb0
        };
        let pad = if matches!(source, Source::Notes) {
            36
        } else {
            80
        };
        for (key, value) in [(107, 127), (107, 0), (pad, 127)] {
            drive.bytes(PAD_PORT, &[status, key, value]);
        }
        drive.frame();
        app_capture_frames(&mut drive, 0);
        assert_eq!(app_capture_snapshot(&drive).erase_targets, [1, 0]);
        app_capture_finish(&mut drive, 1);
        app_capture_start(&mut drive, 2);
        record_controls_erase(&mut drive, Source::Keyboard);
        drive.pad_press(Source::Keyboard, 0);
        app_capture_frames(&mut drive, 0);
        assert_eq!(app_capture_snapshot(&drive).erase_targets, [1, 0]);
        drive.bytes(PAD_PORT, &[status, pad, 0]);
        drive.midi().recorded_fence();
        app_capture_frames(&mut drive, 0);
        drive.frame();
        app_capture_frames(&mut drive, 0);
        assert_eq!(
            app_capture_snapshot(&drive).erase_targets,
            [1, 0],
            "{source:?}: the old signal and notification retain their pass"
        );
        drive.pad_release(Source::Keyboard, 0);
        app_capture_frames(&mut drive, 0);
        assert_eq!(app_capture_snapshot(&drive).erase_targets, [0, 0]);
        assert_eq!(app_capture_snapshot(&drive).captured_onsets, 0);
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[240, 0, 90, 0]]
        );
    }
}

#[test]
fn raw_erasure_early_release_and_disconnect_cannot_create_a_nonzero_interval() {
    for source in [Source::Notes, Source::Button] {
        for after_ui in [false, true] {
            for release in [
                RecordedRelease::Off,
                RecordedRelease::Zero,
                RecordedRelease::Disconnect,
            ] {
                let original = vec![[0, 0, 90, 0], [240, 0, 70, 0]];
                let mut drive = app_capture_drive(original.clone());
                record_controls_routes(&mut drive, source);
                app_capture_start(&mut drive, 1);
                if matches!(source, Source::Button) {
                    drive.bind_pad_button(0);
                }
                let status = if matches!(source, Source::Notes) {
                    0x90
                } else {
                    0xb0
                };
                let pad = if matches!(source, Source::Notes) {
                    36
                } else {
                    80
                };
                for (key, value) in [(107, 127), (107, 0), (pad, 127)] {
                    drive.bytes(PAD_PORT, &[status, key, value]);
                }
                if after_ui {
                    drive.frame();
                }
                if matches!(release, RecordedRelease::Disconnect) {
                    drive.midi().recorded_disconnect(PAD_PORT);
                } else {
                    drive.bytes(
                        PAD_PORT,
                        &[
                            if matches!((source, release), (Source::Notes, RecordedRelease::Off)) {
                                0x80
                            } else {
                                status
                            },
                            pad,
                            0,
                        ],
                    );
                }
                drive.frame();
                app_capture_frames(&mut drive, 128);
                let state = app_capture_snapshot(&drive);
                assert_eq!(
                    (
                        state.erase_targets,
                        state.erased_rows,
                        state.captured_onsets
                    ),
                    ([0, 0], 0, 0),
                    "{source:?}/{after_ui}/{release:?}"
                );
                drive.app.flush_audio();
                assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, original);
                assert!(drive.app.midi_ui_held.is_empty());
                drive.tap(KeyCode::Char('r'), M::CONTROL);
                record_controls_idle(&mut drive);
                assert!(drive.app.history.is_empty());
            }
        }
    }
}

#[test]
fn raw_erasure_release_tracks_resolved_banks_independent_ports_channels_and_grid_edits() {
    const SECOND: &str = "Recorded pads MIDI 2";
    for source in [Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![
            [0, 16, 90, 0],
            [239, 16, 80, 0],
            [240, 16, 70, 0],
            [239, 17, 100, 0],
            [5000, 16, 100, 0],
        ]);
        record_controls_routes(&mut drive, source);
        pattern_parameter_routes(&mut drive);
        drive.bind_action(106, "bank.next");
        drive.midi().recorded_connect(SECOND);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(0);
            drive.bind_action(81, "pad.2");
        }
        drive.frame();
        app_capture_start(&mut drive, 1);
        let status = if matches!(source, Source::Notes) {
            0x90
        } else {
            0xb0
        };
        let pad = if matches!(source, Source::Notes) {
            36
        } else {
            80
        };
        drive.bytes(PAD_PORT, &[0xb0, 106, 127]);
        for (key, value) in [(107, 127), (107, 0), (pad, 127), (pad, 80)] {
            drive.bytes(PAD_PORT, &[status, key, value]);
        }
        drive.bytes(SECOND, &[status | 1, pad + 1, 127]);
        drive.frame();
        assert_eq!(drive.app.pads.bank, 1);
        app_capture_frames(&mut drive, 6000);
        assert_eq!(app_capture_snapshot(&drive).erase_targets, [3 << 16, 0]);
        // Neither another port nor another channel owns the second target's release.
        drive.bytes(PAD_PORT, &[status | 1, pad + 1, 0]);
        drive.bytes(SECOND, &[status, pad + 1, 0]);
        drive.midi().recorded_fence();
        app_capture_frames(&mut drive, 0);
        assert_eq!(app_capture_snapshot(&drive).erase_targets, [3 << 16, 0]);
        pattern_parameter_knob(&mut drive, PAD_PORT, 102, 0);
        app_capture_frames(&mut drive, 0);
        pattern_parameter_knob(&mut drive, PAD_PORT, 101, 16);
        app_capture_frames(&mut drive, 0);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().quantize, "1/4");
        assert_eq!(
            app_capture_snapshot(&drive).erase_targets,
            [3 << 16, 0],
            "grid publication retains each original hold"
        );
        drive.tap(KeyCode::F(4), M::NONE);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_eq!(drive.app.pads.bank, 2);
        drive.bytes(PAD_PORT, &[status, pad, 0]);
        drive.midi().recorded_fence();
        app_capture_frames(&mut drive, 0);
        assert_eq!(app_capture_snapshot(&drive).erase_targets, [1 << 17, 0]);
        drive.midi().recorded_disconnect(SECOND);
        drive.midi().recorded_fence();
        app_capture_frames(&mut drive, 0);
        assert_eq!(app_capture_snapshot(&drive).erase_targets, [0, 0]);
        drive.frame();
        app_capture_frames(&mut drive, 128);
        assert_eq!(app_capture_snapshot(&drive).captured_onsets, 0);
        assert!(drive.app.midi_ui_held.is_empty());
        assert_eq!((drive.app.screen, drive.app.pads.bank), (Screen::Fx, 2));
    }
}

#[test]
fn raw_erasure_pending_old_release_cannot_steal_a_fresh_gate_release_before_one_callback() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![]);
        record_controls_routes(&mut drive, source);
        app_capture_sound(
            &mut drive,
            0,
            Arc::new(Sample::mono(vec![0.03; 48000])),
            true,
        );
        app_capture_start(&mut drive, 1);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(0);
        }
        let status = if matches!(source, Source::Notes) {
            0x90
        } else {
            0xb0
        };
        let pad = if matches!(source, Source::Notes) {
            36
        } else {
            80
        };
        for (key, value) in [(107, 127), (107, 0), (pad, 127)] {
            drive.bytes(PAD_PORT, &[status, key, value]);
        }
        drive.frame();
        drive.tap(KeyCode::Char('e'), M::ALT);
        drive.pad_press(Source::Keyboard, 0);
        drive.bytes(PAD_PORT, &[status, pad, 0]);
        drive.frame();
        let first = app_capture_frames(&mut drive, 128);
        assert!(first.0.iter().all(|sample| *sample == 0.03));
        assert_eq!(app_capture_snapshot(&drive).captured_onsets, 1);
        drive.pad_release(Source::Keyboard, 0);
        let tail = app_capture_frames(&mut drive, 512);
        assert!(
            tail.0[240..]
                .iter()
                .chain(&tail.1[240..])
                .all(|sample| *sample == 0.0),
            "{source:?}: a pending old erase release cannot consume the fresh ordinary release obligation"
        );
    }
}

/// Scenario "Leave erasure without a stuck or unintended note".
#[test]
fn raw_erasure_old_release_cannot_cut_a_fresh_gate_after_erasure_ends() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![]);
        record_controls_routes(&mut drive, source);
        for pad in 0..3 {
            app_capture_sound(
                &mut drive,
                pad,
                Arc::new(Sample::mono(vec![
                    if pad == 1 { 0.02 } else { 0.03 };
                    48000
                ])),
                true,
            );
        }
        app_capture_start(&mut drive, 1);
        drive.pad_press(Source::Keyboard, 1);
        let ordinary = app_capture_frames(&mut drive, 128);
        assert!(
            ordinary
                .0
                .iter()
                .all(|sample| (*sample - 0.02).abs() < 1e-6)
        );
        if matches!(source, Source::Keyboard) {
            record_controls_erase(&mut drive, source);
            drive.pad_press(source, 0);
        } else {
            if matches!(source, Source::Button) {
                drive.bind_pad_button(0);
            }
            let status = if matches!(source, Source::Notes) {
                0x90
            } else {
                0xb0
            };
            let pad = if matches!(source, Source::Notes) {
                36
            } else {
                80
            };
            for (key, value) in [(107, 127), (107, 0), (pad, 127)] {
                drive.bytes(PAD_PORT, &[status, key, value]);
            }
            drive.frame();
        }
        let erasing = app_capture_frames(&mut drive, 128);
        assert!(
            erasing.0.iter().all(|sample| (*sample - 0.02).abs() < 1e-6),
            "the earlier ordinary Gate stays audible, the target stays silent"
        );
        assert_eq!(app_capture_snapshot(&drive).erase_targets, [1, 0]);
        drive.tap(KeyCode::Char('e'), M::ALT);
        app_capture_frames(&mut drive, 0);
        let fresh = if matches!(source, Source::Keyboard) {
            2
        } else {
            0
        };
        drive.pad_press(Source::Keyboard, fresh);
        let sounding = app_capture_frames(&mut drive, 128);
        assert!(
            sounding
                .0
                .iter()
                .all(|sample| (*sample - 0.05).abs() < 1e-6)
        );
        drive.pad_release(source, 0);
        let released = app_capture_frames(&mut drive, 128);
        assert!(
            released
                .0
                .iter()
                .all(|sample| (*sample - 0.05).abs() < 1e-6),
            "{source:?}: an old erase release cannot cut the fresh Gate"
        );
        drive.pad_release(Source::Keyboard, 1);
        let first_tail = app_capture_frames(&mut drive, 512);
        assert!(
            first_tail.0[240..]
                .iter()
                .all(|sample| (*sample - 0.03).abs() < 1e-6)
        );
        drive.pad_release(Source::Keyboard, fresh);
        let tail = app_capture_frames(&mut drive, 512);
        assert!(
            tail.0[240..]
                .iter()
                .chain(&tail.1[240..])
                .all(|sample| *sample == 0.0)
        );
        drive.app.flush_audio();
        let rows = &drive.app.pads.pattern.as_ref().unwrap().hits;
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0][1], rows[1][1]), (1, fresh as u32));
        assert!(rows.iter().all(|row| row[3] > 0));
        assert_eq!(app_capture_snapshot(&drive).captured_onsets, 2);
        assert!(drive.app.midi_ui_held.is_empty());
    }
}

#[test]
fn record_controls_erasure_orders_targets_and_rapid_toggles_without_redraws() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let original = vec![[0, 0, 90, 0], [0, 1, 70, 0]];
        let mut drive = app_capture_drive(original.clone());
        record_controls_routes(&mut drive, source);
        app_capture_sound(&mut drive, 0, app_capture_tone(60.0), false);
        drive.app.send(Command::TransportPlay);
        app_capture_frames(&mut drive, 0);
        record_controls_record(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        record_controls_erase(&mut drive, source);
        record_controls_erase(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        assert!(
            !app_capture_snapshot(&drive).erasing,
            "two commands before a callback must toggle On then Off"
        );
        record_controls_erase(&mut drive, source);
        drive.pad_press(source, 0);
        let sound = app_capture_frames(&mut drive, 6000);
        assert!(sound.0.iter().chain(&sound.1).all(|v| *v == 0.0));
        drive.pad_release(source, 0);
        record_controls_erase(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        assert_eq!(app_capture_snapshot(&drive).captured_onsets, 0);
        assert_eq!(app_capture_snapshot(&drive).erased_rows, 1);
        record_controls_record(&mut drive, source);
        record_controls_idle(&mut drive);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, [original[1]]);
        assert_eq!(drive.app.history.len(), 1);
        global_pattern_undo(&mut drive, source);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, original);
    }
}

/// Scenario "Chop recording uses absolute pad banks".
/// Scenario "A recording pad release keeps its original bank and screen".
#[test]
fn record_controls_chop_plays_real_banks_and_releases_the_original_gate() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![]);
        record_controls_routes(&mut drive, source);
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48_000])), 4);
        app_capture_sound(&mut drive, 0, app_capture_tone(60.0), true);
        app_capture_sound(&mut drive, 112, app_capture_tone(900.0), true);
        app_capture_frames(&mut drive, 0);
        assert!(
            drive
                .app
                .chop_display(drive.app.chop.as_ref().unwrap())
                .slice_keys
                .is_some()
        );
        drive.app.send(Command::TransportPlay);
        app_capture_frames(&mut drive, 0);
        record_controls_record(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        drive.frame();
        let view = drive.app.chop_display(drive.app.chop.as_ref().unwrap());
        assert_eq!(view.slice_keys, None);
        assert_eq!(view.pads.bank, 0);
        assert!(view.pads.pads.iter().all(|p| !p.dim));
        drive.pad_press(source, 0);
        let wave = app_capture_frames(&mut drive, 6000);
        assert!(wave.0.iter().any(|v| v.abs() > 0.001));
        drive.tap(KeyCode::Char('8'), M::CONTROL);
        drive.tap(KeyCode::F(4), M::NONE);
        let released = drive.pad_release(source, 0);
        assert!(
            released
                .commands()
                .iter()
                .any(|c| matches!(c, Command::PadRelease { pad: 0 }))
                || !matches!(source, Source::Keyboard),
            "keyboard release must be queued; direct MIDI release is checked by Gate duration"
        );
        assert!(
            !released
                .commands()
                .iter()
                .any(|c| matches!(c, Command::PadRelease { pad: 112 }))
        );
        app_capture_frames(&mut drive, 6000);
        drive.tap(KeyCode::F(3), M::NONE);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 6000);
        record_controls_record(&mut drive, source);
        record_controls_idle(&mut drive);
        drive.pad_release(source, 0);
        app_capture_frames(&mut drive, 0);
        let hits = &drive.app.pads.pattern.as_ref().unwrap().hits;
        assert_eq!(hits.len(), 2);
        assert_eq!(
            (hits[0][1], hits[0][3], hits[1][1], hits[1][3]),
            (0, 240, 112, 240)
        );
        assert_eq!(drive.app.screen, Screen::Chop);
        assert!(
            drive
                .app
                .chop_display(drive.app.chop.as_ref().unwrap())
                .slice_keys
                .is_some()
        );
    }
}

#[test]
fn record_controls_legacy_keyboard_erase_target_expires_after_one_loop() {
    let mut drive = app_capture_drive(vec![[0, 0, 90, 0], [0, 1, 70, 0]]);
    record_controls_routes(&mut drive, Source::Keyboard);
    drive.app.pads.track_keys = false;
    drive.app.send(Command::TransportPlay);
    app_capture_frames(&mut drive, 0);
    record_controls_record(&mut drive, Source::Keyboard);
    app_capture_frames(&mut drive, 0);
    record_controls_erase(&mut drive, Source::Keyboard);
    drive.pad_press(Source::Keyboard, 0);
    app_capture_frames(&mut drive, 96_128);
    assert_eq!(app_capture_snapshot(&drive).erase_targets, [0; 2]);
    assert_eq!(app_capture_snapshot(&drive).captured_onsets, 0);
    record_controls_record(&mut drive, Source::Keyboard);
    record_controls_idle(&mut drive);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[0, 1, 70, 0]]
    );
}

/// Scenario "Placement and recording cannot consume the same pad".
#[test]
fn record_controls_placement_and_marking_cannot_take_recording_pads() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for mode in ["swap", "copy", "mark", "marked", "assign", "file"] {
            let mut drive = app_capture_drive(vec![]);
            record_controls_routes(&mut drive, source);
            match mode {
                "swap" => drive.app.pad_in_hand = Some(0),
                "copy" => drive.app.copy_in_hand = Some((0, drive.app.pads.snapshot(0))),
                "mark" => drive.app.pads.marking = true,
                "marked" => {
                    drive.app.pads.marked.insert(0);
                }
                "assign" => {
                    drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48_000])), 4);
                    drive.app.plan = Some(chop::Plan::ByHand(vec![]));
                }
                "file" => drive.app.browser.in_hand = Some("unplaced.wav".into()),
                _ => unreachable!(),
            }
            let before = drive.app.pads.pattern.clone();
            let assignments = drive.app.before();
            record_controls_record(&mut drive, source);
            app_capture_frames(&mut drive, 0);
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::Idle
            );
            assert_eq!(drive.app.pads.pattern, before);
            assert_eq!(drive.app.before().pads, assignments.pads);
            assert_eq!(drive.app.before().fx, assignments.fx);
            assert_eq!(app_capture_snapshot(&drive).captured_onsets, 0);
            assert_eq!(drive.last_notice().level, Level::Warn);
            assert!(drive.last_notice().text.contains("first"), "{mode}");
        }
        for (name, code, modifiers) in [
            ("pad.take", KeyCode::Char('x'), M::CONTROL),
            ("pad.copy", KeyCode::Char('c'), M::CONTROL),
            ("pad.mark_toggle", KeyCode::Char('m'), M::NONE),
            ("pad.mark_bank", KeyCode::Char('a'), M::CONTROL),
            ("chop.assign", KeyCode::Char('p'), M::NONE),
            ("browser.hold", KeyCode::Enter, M::NONE),
        ] {
            let mut drive = app_capture_drive(vec![]);
            record_controls_routes(&mut drive, source);
            if name == "chop.assign" {
                drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48_000])), 4);
            } else if name == "browser.hold" {
                drive.ordered_file();
            }
            learn::assign(
                &mut drive.app.learned,
                "Recorded pads",
                input::Group::Global,
                if matches!(source, Source::Notes) {
                    input::midi::Key::Note(105)
                } else {
                    input::midi::Key::Cc(105)
                },
                name,
            );
            drive
                .app
                .sync_learned("Recorded pads", input::Group::Global);
            drive.frame();
            app_capture_start(&mut drive, 1);
            let assignments = drive.app.before();
            drive.registry_command(source, code, modifiers, 105);
            assert_eq!(drive.app.before().pads, assignments.pads);
            assert_eq!(drive.app.before().fx, assignments.fx);
            assert!(drive.app.pad_in_hand.is_none() && drive.app.copy_in_hand.is_none());
            assert!(drive.app.browser.in_hand.is_none() && drive.app.plan.is_none());
            assert!(!drive.app.pads.marking && drive.app.pads.marked.is_empty());
            assert_eq!(app_capture_snapshot(&drive).captured_onsets, 0);
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::Recording
            );
            assert!(
                drive.last_notice().text.contains("finish the pass"),
                "{name}/{source:?}"
            );
        }
    }
}

/// Scenario "Pattern commands cannot escape a blocking input mode".
#[test]
fn record_controls_blocking_modes_keep_commands_silent_and_finish_earlier_releases() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for mode in [
            "name",
            "filter",
            "picker",
            "question",
            "screen-learn",
            "midi-learn",
            "restoring",
        ] {
            let mut drive = app_capture_drive(vec![]);
            record_controls_routes(&mut drive, source);
            pattern_panel_routes(&mut drive, source);
            app_capture_sound(
                &mut drive,
                0,
                Arc::new(Sample::mono(vec![0.02; 48000])),
                true,
            );
            app_capture_start(&mut drive, 1);
            drive.pad_press(source, 0);
            let sounding = app_capture_frames(&mut drive, 6000);
            assert!(sounding.0.iter().all(|sample| *sample == 0.02));
            match mode {
                "name" => drive.app.pad_naming = Some((0, "name".into())),
                "filter" => {
                    drive.app.screen = Screen::Files;
                    drive.app.browser.editing = true;
                }
                "picker" => {
                    drive.app.project_picker = Some(crate::ProjectPicker {
                        entries: vec![],
                        selected: 0,
                        naming: None,
                    })
                }
                "question" => {
                    drive.app.ask = Some(Ask::ClearPads {
                        pads: vec![0],
                        lines: vec![],
                    })
                }
                "screen-learn" => drive.app.screen_learn = Some(drive.app.learn_open()),
                "midi-learn" => {
                    drive.app.learning = Some(learn::Learning::new(action_info("pattern.record")))
                }
                "restoring" => drive.app.restoring_project = true,
                _ => unreachable!(),
            }
            drive.frame();
            let before = app_capture_snapshot(&drive);
            record_controls_record(&mut drive, source);
            record_controls_erase(&mut drive, source);
            global_pattern_undo(&mut drive, source);
            drive.registry_command(source, KeyCode::Char('n'), M::ALT, 90);
            assert!(drive.app.pattern_panel.is_none());
            drive.pad_release(source, 0);
            let tail = app_capture_frames(&mut drive, 512);
            assert!(
                tail.0[240..]
                    .iter()
                    .chain(&tail.1[240..])
                    .all(|sample| *sample == 0.0),
                "{mode}/{source:?}: the original Gate releases through its owner"
            );
            drive.app.flush_audio();
            let after = app_capture_snapshot(&drive);
            assert_eq!(
                (
                    after.pass,
                    after.phase,
                    after.erasing,
                    after.captured_onsets
                ),
                (before.pass, before.phase, false, 1),
                "{mode}/{source:?}"
            );
            assert!(drive.app.history.is_empty());
            assert_eq!(
                drive.app.pads.pattern.as_ref().unwrap().hits,
                [[0, 0, 127, 240]],
                "{mode}/{source:?}"
            );
        }
    }
}

/// Scenario "Owner recording overrides retain their old meaning".
#[test]
fn record_controls_owner_keys_keep_mic_holds_removals_and_hidden_default_warnings() {
    for (binding, expected) in [
        ("record.mic", Some(Action::RecordMic(true))),
        ("none", None),
    ] {
        let owner = format!("[global]\n\"Ctrl+R\" = \"{binding}\"\n\"Alt+E\" = \"none\"\n");
        let files = TransitionFiles::new();
        let path = files.0.join("keymap.toml");
        std::fs::write(&path, &owner).unwrap();
        let (_, notice, warnings) = load_keymap(&path, DEFAULT_KEYMAP);
        assert!(notice.is_none());
        let expected_warnings: Vec<_> = ["pads", "chop", "files", "fx", "settings"]
            .into_iter()
            .flat_map(|screen| {
                [
                    format!("keymap.toml: Ctrl+R on {screen} hides pattern.record"),
                    format!("keymap.toml: Alt+E on {screen} hides pattern.erase"),
                ]
            })
            .collect();
        assert_eq!(warnings, expected_warnings);
        for kitty in [false, true] {
            let mut drive = app_capture_drive(vec![]);
            let (keymap, notice, shadows) = load_keymap(&path, DEFAULT_KEYMAP);
            assert!(notice.is_none());
            assert_eq!(shadows, expected_warnings);
            drive.keymap = keymap;
            assert_eq!(
                drive
                    .keymap
                    .resolve(Screen::Pads, &KeyEvent::new(KeyCode::Char('к'), M::CONTROL)),
                expected
            );
            drive.app.notices = Notices::new(Instant::now());
            for kind in [KeyEventKind::Press, KeyEventKind::Release] {
                assert!(!drive.app.keyboard_event(
                    &drive.keymap,
                    &KeyEvent::new_with_kind(KeyCode::Char('к'), M::CONTROL, kind),
                    kitty,
                    &mut drive.down,
                    &mut drive.held,
                    &mut drive.mods
                ));
            }
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::Idle
            );
            assert!(
                !drive
                    .app
                    .notices
                    .history()
                    .iter()
                    .any(|n| n.text.contains("moved"))
            );
            assert!(
                !drive
                    .app
                    .sent
                    .iter()
                    .any(|c| matches!(c, Command::PatternRecordStart { .. }))
            );
            assert_eq!(std::fs::read(&path).unwrap(), owner.as_bytes());
        }
    }
}

/// Scenario "Enter Chop during recording without interrupting the rhythm".
#[test]
fn record_controls_chop_entry_keeps_voices_clock_and_each_capture_phase() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for phase in ["count-in", "recording", "finishing"] {
            let mut drive = app_capture_drive(vec![]);
            record_controls_routes(&mut drive, source);
            app_capture_sound(
                &mut drive,
                0,
                Arc::new(Sample::mono(vec![0.02; 48_000])),
                true,
            );
            if phase != "count-in" {
                drive.app.send(Command::TransportPlay);
                app_capture_frames(&mut drive, 0);
            }
            record_controls_record(&mut drive, source);
            app_capture_frames(&mut drive, 0);
            drive.pad_press(source, 0);
            app_capture_frames(&mut drive, 128);
            if phase == "finishing" {
                record_controls_record(&mut drive, source);
                app_capture_frames(&mut drive, 0);
            }
            let before = app_capture_snapshot(&drive);
            let clock = drive.app.transport_state();
            let enter = drive.tap(KeyCode::F(3), M::NONE);
            assert_eq!(drive.app.screen, Screen::Chop);
            assert!(
                !enter
                    .commands()
                    .iter()
                    .any(|c| matches!(c, Command::HushPads))
            );
            assert_eq!(drive.app.transport_state(), clock);
            assert_eq!(app_capture_snapshot(&drive), before);
            let wave = app_capture_frames(&mut drive, 128);
            assert!(wave.0.iter().any(|v| v.abs() > 0.001));
            assert_eq!(app_capture_snapshot(&drive).phase, before.phase);
            drive.pad_release(source, 0);
        }
    }
}

/// Scenario "Preview a slice without recording a pattern hit".
#[test]
fn record_controls_chop_preview_stays_out_of_a1_and_linked_hits_select_their_slice() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![]);
        record_controls_routes(&mut drive, source);
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48_000])), 4);
        drive.link_chop_pad(18, 3);
        drive.app.pads.bank = 1;
        drive.app.chop.as_mut().unwrap().select_slice(0);
        app_capture_start(&mut drive, 1);
        drive.frame();
        let clock = drive.app.transport_state();
        let space = drive.tap(KeyCode::Char(' '), M::NONE);
        assert!(
            space
                .commands()
                .iter()
                .any(|c| matches!(c, Command::Preview { sound: Some(_), .. }))
        );
        assert_eq!(drive.app.transport_state(), clock);
        app_capture_frames(&mut drive, 128);
        assert_eq!(app_capture_snapshot(&drive).captured_onsets, 0);
        drive.app.chop.as_mut().unwrap().cursor = 11_999;
        let cursor = drive.tap(KeyCode::Char('l'), M::NONE);
        assert!(
            cursor
                .commands()
                .iter()
                .any(|c| matches!(c, Command::Preview { sound: Some(_), .. }))
        );
        app_capture_frames(&mut drive, 128);
        assert_eq!(app_capture_snapshot(&drive).captured_onsets, 0);
        drive.pad_press(source, 2);
        app_capture_frames(&mut drive, 128);
        assert_eq!(drive.app.chop.as_ref().unwrap().selected, 3);
        drive.pad_release(source, 2);
        record_controls_record(&mut drive, source);
        record_controls_idle(&mut drive);
        let hits = &drive.app.pads.pattern.as_ref().unwrap().hits;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0][1], 18);
    }
}

#[test]
fn record_controls_question_answers_precede_the_placement_guard() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![]);
        record_controls_routes(&mut drive, source);
        drive.ordered_file();
        learn::assign(
            &mut drive.app.learned,
            "Recorded pads",
            input::Group::Global,
            if matches!(source, Source::Notes) {
                input::midi::Key::Note(105)
            } else {
                input::midi::Key::Cc(105)
            },
            "browser.hold",
        );
        drive
            .app
            .sync_learned("Recorded pads", input::Group::Global);
        drive.frame();
        app_capture_start(&mut drive, 1);
        drive.app.ask = Some(Ask::ClearPads {
            pads: vec![0],
            lines: vec![],
        });
        drive.registry_command(source, KeyCode::Enter, M::NONE, 105);
        assert!(
            drive.app.ask.is_none(),
            "{source:?}: Enter answers the existing question"
        );
        assert!(drive.app.browser.in_hand.is_none());
        assert_eq!(
            app_capture_snapshot(&drive).phase,
            engine::capture::Phase::Recording
        );
    }
}

#[test]
fn record_controls_visible_refusals_keep_notice_time_and_error_acknowledgement() {
    for source in [audio::SOURCE_MIC, audio::SOURCE_OUTPUT] {
        let now = Instant::now();
        let mut app = tests::test_app();
        app.notices = Notices::new(now);
        app.recording = Some((source, now));
        app.post_notice(Level::Info, ("ordinary info".into(), now));
        assert!(app.frame_notice_at(now).unwrap().starts_with("Recording "));
        let warn_at = now + Duration::from_secs(1);
        app.produce_notice(
            Notice::recording_refusal(Origin::Keyboard, "Recording target refused"),
            warn_at,
        );
        assert_eq!(
            app.frame_notice_at(warn_at).as_deref(),
            Some("Recording target refused")
        );
        assert_eq!(app.frame_notice_level, Some(Level::Warn));
        let expired = now + Duration::from_secs(7);
        assert!(
            app.frame_notice_at(expired)
                .unwrap()
                .starts_with("Recording ")
        );
        assert_eq!(app.frame_notice_level, None);
        let later = now + Duration::from_secs(10);
        assert!(
            app.frame_notice_at(later)
                .unwrap()
                .starts_with("Recording ")
        );
        app.recording = None;
        assert_eq!(app.frame_notice_at(later).as_deref(), Some("ordinary info"));
        assert_eq!(app.notices.deadline(), Some(later + Duration::from_secs(3)));
        app.recording = Some((source, now));
        app.post_notice(Level::Error, ("Earlier recording error".into(), later));
        assert!(
            app.frame_notice_at(later)
                .unwrap()
                .starts_with("Recording "),
            "unrelated notices keep their previous recording precedence"
        );
        app.produce_notice(
            Notice::recording_refusal(Origin::Keyboard, "Another target refused"),
            later,
        );
        assert_eq!(
            app.frame_notice_at(later).as_deref(),
            Some("Earlier recording error")
        );
        assert_eq!(app.frame_notice_level, Some(Level::Error));
        app.acknowledge_notice();
        assert_eq!(
            app.frame_notice_at(later).as_deref(),
            Some("Another target refused")
        );
        assert_eq!(app.frame_notice_level, Some(Level::Warn));
        assert!(app.shown_error.is_none());
        assert!(
            app.frame_notice_at(later + Duration::from_secs(6))
                .unwrap()
                .starts_with("Recording ")
        );
        app.recording = None;
        assert_eq!(
            app.frame_notice_at(later + Duration::from_secs(6))
                .as_deref(),
            Some("ordinary info")
        );
    }
}

/// Scenario "Erase only one pad under the playhead".
#[test]
fn app_erasure_removes_stored_rows_and_pass_undo_restores_them() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let original = vec![
            [480, 0, 80, 0],
            [480, 0, 127, 0],
            [480, 1, 90, 0],
            [5000, 0, 70, 0],
        ];
        let mut drive = app_capture_drive(original.clone());
        swap_routes(&mut drive, source);
        app_capture_start(&mut drive, 1);
        drive.app.send(Command::PatternErase {
            pass: 1,
            enabled: true,
        });
        app_capture_frames(&mut drive, 6000);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 12_000);
        drive.pad_release(source, 0);
        drive.app.send(Command::PatternErase {
            pass: 1,
            enabled: false,
        });
        app_capture_frames(&mut drive, 0);
        assert_eq!(app_capture_snapshot(&drive).erased_rows, 2);
        drive.app.flush_audio();
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [original[2], original[3]]
        );
        app_capture_finish(&mut drive, 1);
        assert_eq!(drive.app.history.len(), 1);
        completed_pass_undo(&mut drive, source);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, original);
        assert!(drive.app.history.is_empty());
    }
}

/// Scenario "A later accepted onset survives an earlier erase interval".
#[test]
fn app_erasure_later_same_frame_onset_survives_delayed_collection() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive_raw(vec![[240, 0, 90, 0]]);
        swap_routes(&mut drive, source);
        app_capture_start(&mut drive, 1);
        drive.app.send(Command::PatternErase {
            pass: 1,
            enabled: true,
        });
        app_capture_frames(&mut drive, 0);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 7000);
        drive.pad_release(source, 0);
        drive.app.send(Command::PatternErase {
            pass: 1,
            enabled: false,
        });
        app_capture_frames(&mut drive, 0);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 0);
        let accepted = app_capture_snapshot(&drive);
        assert_eq!((accepted.erased_rows, accepted.captured_onsets), (1, 1));
        drive.app.flush_audio();
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[280, 0, 127, 0]]
        );
        drive.pad_release(source, 0);
        app_capture_finish(&mut drive, 1);
        for _ in 0..4 {
            drive.app.flush_audio();
            app_capture_frames(&mut drive, 0);
        }
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[280, 0, 127, 0]]
        );
        completed_pass_undo(&mut drive, source);
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[240, 0, 90, 0]]
        );
    }
}

#[test]
fn app_erasure_full_raw_model_frees_slots_before_delayed_onsets_append() {
    let mut original = vec![[240, 0, 90, 0]; 4095];
    original.push([5000, 1, 70, 0]);
    let mut drive = app_capture_drive_raw(original.clone());
    app_capture_start(&mut drive, 1);
    let old = drive.app.pads.pattern_rows.clone().unwrap();
    drive.app.send(Command::PatternErase {
        pass: 1,
        enabled: true,
    });
    app_capture_frames(&mut drive, 0);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 7000);
    drive.pad_release(Source::Notes, 0);
    drive.app.send(Command::PatternErase {
        pass: 1,
        enabled: false,
    });
    app_capture_frames(&mut drive, 0);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 0);
    assert_eq!(app_capture_snapshot(&drive).captured_onsets, 1);
    drive.app.flush_audio();
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [original[4095], [280, 0, 127, 0]]
    );
    let new = drive.app.pads.pattern_rows.as_ref().unwrap()[1].id();
    assert_eq!(new.slot(), 0);
    assert!(new.generation() > old[0].id().generation());
    app_capture_frames(&mut drive, 0);
    assert_eq!(app_capture_snapshot(&drive).reserved_rows, 2);
    drive.pad_release(Source::Notes, 0);
    app_capture_finish(&mut drive, 1);
    completed_pass_undo(&mut drive, Source::Keyboard);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, original);
}

#[test]
fn app_erasure_pending_gate_follows_identity_after_an_earlier_row_disappears() {
    let mut drive = app_capture_drive(vec![[240, 0, 90, 0]]);
    drive.app.send(Command::SetPadMode {
        pad: 1,
        mode: engine::PadMode::Gate,
    });
    app_capture_frames(&mut drive, 0);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 1);
    app_capture_frames(&mut drive, 0);
    drive.app.flush_audio();
    let id = drive.app.pads.pattern_rows.as_ref().unwrap()[1].id();
    drive.app.send(Command::PatternErase {
        pass: 1,
        enabled: true,
    });
    app_capture_frames(&mut drive, 0);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 7000);
    drive.pad_release(Source::Notes, 0);
    drive.app.send(Command::PatternErase {
        pass: 1,
        enabled: false,
    });
    app_capture_frames(&mut drive, 0);
    drive.app.flush_audio();
    assert_eq!(drive.app.pads.pattern_rows.as_ref().unwrap()[0].id(), id);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[0, 1, 127, 1]]
    );
    app_capture_frames(&mut drive, 1000);
    drive.pad_release(Source::Notes, 1);
    app_capture_finish(&mut drive, 1);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[0, 1, 127, 320]]
    );
    assert_eq!(drive.app.pads.pattern_rows.as_ref().unwrap()[0].id(), id);
}

#[test]
fn app_erasure_early_expiry_cannot_remove_a_later_onset_in_the_same_frame() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![]);
        let pattern = drive.app.pads.pattern.as_mut().unwrap();
        pattern.quantize = "off".into();
        pattern.strength = 0;
        drive.app.send(Command::SetTempo { bpm: 122.0 });
        app_capture_start(&mut drive, 1);
        app_capture_frames(&mut drive, 123);
        drive.app.send(Command::PatternErase {
            pass: 1,
            enabled: true,
        });
        drive.app.send(Command::PatternEraseTarget {
            pass: 1,
            pad: 0,
            one_loop: true,
        });
        app_capture_frames(&mut drive, 94_426);
        let expired = app_capture_snapshot(&drive);
        assert_eq!(expired.erase_targets, [0; 2]);
        assert!(expired.ticks < 3845.002);
        drive.app.send(Command::PatternErase {
            pass: 1,
            enabled: false,
        });
        app_capture_frames(&mut drive, 0);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 0);
        drive.app.flush_audio();
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[5, 0, 127, 0]]
        );
        drive.pad_release(source, 0);
        app_capture_finish(&mut drive, 1);
    }
}

/// Scenario "Reused recording capacity keeps earlier Gates separate".
#[test]
fn app_erasure_reuses_gate_and_row_slots_after_4096_onsets_in_one_pass() {
    let mut drive = app_capture_drive(vec![]);
    drive.app.pads.pattern.as_mut().unwrap().quantize = "off".into();
    drive.app.send(Command::SetPadMode {
        pad: 0,
        mode: engine::PadMode::Gate,
    });
    drive.app.send(Command::SetTempo { bpm: 300.0 });
    app_capture_frames(&mut drive, 0);
    app_capture_start(&mut drive, 1);
    let mut stale = None;
    for _ in 0..64 {
        for _ in 0..64 {
            drive.app.send(Command::PadPress {
                pad: 0,
                velocity: 1.0,
            });
            drive.app.send(Command::PadRelease { pad: 0 });
        }
        app_capture_frames(&mut drive, 0);
        drive.app.flush_audio();
        stale = drive
            .app
            .sent
            .iter()
            .rev()
            .find_map(|command| match command {
                Command::PatternGateConsumed { slot, sequence, .. } => Some((*slot, *sequence)),
                _ => None,
            })
            .or(stale);
        app_capture_frames(&mut drive, 0);
    }
    assert_eq!(app_capture_snapshot(&drive).captured_onsets, 4096);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 4096);
    let old = drive.app.pads.pattern_rows.as_ref().unwrap()[0].id();
    drive.app.send(Command::PatternErase {
        pass: 1,
        enabled: true,
    });
    drive.app.send(Command::PatternEraseTarget {
        pass: 1,
        pad: 0,
        one_loop: false,
    });
    app_capture_frames(&mut drive, 128);
    drive.app.send(Command::PatternErase {
        pass: 1,
        enabled: false,
    });
    app_capture_frames(&mut drive, 0);
    drive.app.flush_audio();
    app_capture_frames(&mut drive, 0);
    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
    drive.app.send(Command::PadPress {
        pad: 0,
        velocity: 1.0,
    });
    app_capture_frames(&mut drive, 0);
    drive.app.flush_audio();
    let new = drive.app.pads.pattern_rows.as_ref().unwrap()[0].id();
    assert_eq!(new.slot(), old.slot());
    assert!(new.generation() > old.generation());
    let (slot, sequence) = stale.unwrap();
    drive.app.send(Command::PatternGateConsumed {
        pass: 1,
        slot,
        sequence,
    });
    app_capture_frames(&mut drive, 6000);
    app_capture_finish(&mut drive, 1);
    let state = app_capture_snapshot(&drive);
    assert_eq!(
        (
            state.captured_onsets,
            state.limit_refusals,
            state.dropped_onsets
        ),
        (4097, 0, 0)
    );
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[13, 0, 127, 600]]
    );
    let consumed = drive
        .app
        .sent
        .iter()
        .find_map(|command| match command {
            Command::PatternGateConsumed {
                slot,
                sequence: new_sequence,
                ..
            } if *new_sequence > sequence => Some(*slot),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        consumed, slot,
        "the stale command addresses the actually reused slot"
    );
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    app_capture_frames(&mut drive, 0);
    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
    app_capture_start(&mut drive, 2);
    drive.app.send(Command::PadPress {
        pad: 0,
        velocity: 1.0,
    });
    app_capture_frames(&mut drive, 0);
    app_capture_finish(&mut drive, 2);
    let after_undo = drive.app.pads.pattern_rows.as_ref().unwrap()[0].id();
    assert_eq!(after_undo.slot(), new.slot());
    assert!(
        after_undo.generation() > new.generation(),
        "Undo must not rewind reused generations"
    );
}

#[test]
fn app_erasure_unpublished_onsets_use_each_retained_grid_and_strength() {
    for grid in [
        "off", "1/4", "1/8", "1/16", "1/32", "1/64", "1/8T", "1/16T", "1/32T",
    ] {
        for strength in [0, 50, 100] {
            for source in [Source::Keyboard, Source::Notes, Source::Button] {
                let mut drive = app_capture_drive(vec![[5000, 0, 70, 0]]);
                set_quantize(&mut drive, grid, strength);
                app_capture_start(&mut drive, 1);
                app_capture_frames(&mut drive, 3025);
                drive.pad_press(source, 0);
                drive.pad_release(source, 0);
                app_capture_frames(&mut drive, 0);
                let accepted = app_capture_snapshot(&drive);
                assert_eq!(accepted.captured_onsets, 1);
                for command in [
                    Command::PatternErase {
                        pass: 1,
                        enabled: true,
                    },
                    Command::PatternEraseTarget {
                        pass: 1,
                        pad: 0,
                        one_loop: false,
                    },
                ] {
                    drive
                        .app
                        .audio
                        .as_mut()
                        .unwrap()
                        .commands
                        .push(ProjectCommand {
                            epoch: accepted.epoch,
                            ready_at_send: true,
                            command: ProjectMessage::Command(command),
                        })
                        .unwrap();
                }
                app_capture_frames(&mut drive, 14_475);
                assert_eq!(
                    app_capture_snapshot(&drive).erased_rows,
                    0,
                    "the onset has not been published"
                );
                assert_eq!(app_capture_snapshot(&drive).erase_intervals, 1);
                assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 1);
                let state = drive.app.pads.pattern.as_mut().unwrap();
                state.quantize = "1/4".into();
                state.strength = 100;
                drive.app.send(Command::PatternErase {
                    pass: 1,
                    enabled: false,
                });
                app_capture_frames(&mut drive, 0);
                drive.app.flush_audio();
                let spacing = match grid {
                    "off" => None,
                    "1/4" => Some(960.0),
                    "1/8" => Some(480.0),
                    "1/16" => Some(240.0),
                    "1/32" => Some(120.0),
                    "1/64" => Some(60.0),
                    "1/8T" => Some(320.0),
                    "1/16T" => Some(160.0),
                    "1/32T" => Some(80.0),
                    _ => unreachable!(),
                };
                let tick = spacing.map_or(121.0, |spacing: f64| {
                    (121.0
                        + ((121.0 / spacing).round() * spacing - 121.0) * f64::from(strength)
                            / 100.0)
                        .round()
                });
                // the row saves the tick placed with the grid of its capture (98c); a later
                // settings change does not move it
                let mut expected = vec![[5000, 0, 70, 0]];
                if !(121.0..700.0).contains(&tick) {
                    expected.push([tick as u32, 0, 127, 0]);
                }
                assert_eq!(
                    drive.app.pads.pattern.as_ref().unwrap().hits,
                    expected,
                    "{source:?}, {grid}, {strength}, compiled {tick}"
                );
                drive.pad_release(source, 0);
                app_capture_finish(&mut drive, 1);
            }
        }
    }
}

#[test]
fn app_erasure_save_quit_and_audio_loss_keep_only_the_completed_owned_prefix() {
    use std::sync::atomic::Ordering::Relaxed;
    for end in ["save", "quit", "loss"] {
        for project in [false, true] {
            let (_files, mut drive) = pattern_autosave_drive(project);
            drive.app.pads.pattern.as_mut().unwrap().hits = vec![[240, 0, 80, 0], [240, 1, 90, 0]];
            let original = drive.app.pads.pattern.clone().unwrap();
            app_capture_start(&mut drive, 1);
            drive.app.send(Command::PatternErase {
                pass: 1,
                enabled: true,
            });
            drive.app.send(Command::PatternEraseTarget {
                pass: 1,
                pad: 0,
                one_loop: false,
            });
            app_capture_frames(&mut drive, 7000);
            drive.app.sent.clear();
            if end == "save" {
                drive.tap(KeyCode::Char('s'), M::CONTROL);
                pattern_manual_save_settle(&mut drive);
                assert_eq!(pattern_manual_save_successes(&drive), 1);
            } else {
                if end == "loss" {
                    drive.audio_rt = None;
                    drive
                        .app
                        .audio
                        .as_ref()
                        .unwrap()
                        .stopped
                        .store(true, Relaxed);
                    drive.app.flush_audio();
                    drive.tap(KeyCode::Char('s'), M::CONTROL);
                } else {
                    assert!(pattern_lifecycle_quit_key(&mut drive));
                    assert!(drive.app.quit_ready());
                    drive.app.save_on_exit();
                }
                assert!(
                    drive.app.sent.is_empty(),
                    "a closed collector must not need audio commands"
                );
            }
            let saved = session::load_existing(&drive.app.session_file()).unwrap();
            assert_eq!(saved.version, 5);
            assert_eq!(saved.patterns[0].hits, [[240, 1, 90, 0]], "{end}");
            assert_eq!(drive.app.history.len(), 1);
            if end != "quit" {
                drive.tap(KeyCode::Char('z'), M::CONTROL);
                assert_eq!(drive.app.pads.pattern.as_ref(), Some(&original));
            }
        }
    }
}

#[test]
fn app_erasure_queued_ack_and_undo_cannot_reapply_old_intervals() {
    let original = vec![[240, 0, 90, 0]];
    let mut drive = app_capture_drive(original.clone());
    app_capture_start(&mut drive, 1);
    drive.app.send(Command::PatternErase {
        pass: 1,
        enabled: true,
    });
    drive.app.send(Command::PatternEraseTarget {
        pass: 1,
        pad: 0,
        one_loop: false,
    });
    app_capture_frames(&mut drive, 7000);
    drive.app.send(Command::PatternRecordEnd { pass: 1 });
    app_capture_frames(&mut drive, 0);
    let epoch = app_capture_snapshot(&drive).epoch;
    for _ in 0..1024 {
        drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .unwrap();
    }
    drive.app.undo_pads();
    assert!(!drive.app.pending_audio.is_empty());
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, original);
    assert!(drive.app.history.is_empty());
    for _ in 0..8 {
        drive.app.flush_audio();
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, original);
    }
    for _ in 0..16 {
        drive.app.flush_audio();
        app_capture_frames(&mut drive, 0);
    }
    assert_eq!(
        app_capture_snapshot(&drive).phase,
        engine::capture::Phase::Idle
    );
    app_capture_start(&mut drive, 2);
    drive.pad_press(Source::Notes, 1);
    app_capture_frames(&mut drive, 0);
    app_capture_finish(&mut drive, 2);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 2);
    assert_eq!(drive.app.history.len(), 1);
}

#[test]
fn app_erasure_render_keeps_the_missing_beat_until_pass_undo() {
    let sounds = [app_capture_tone(60.0), app_capture_tone(900.0)];
    let original: Vec<_> = (0..4)
        .flat_map(|beat| [[beat * 960, 0, 127, 0], [beat * 960 + 480, 1, 127, 0]])
        .collect();
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(original.clone());
        swap_routes(&mut drive, source);
        for (pad, sound) in sounds.iter().enumerate() {
            app_capture_sound(&mut drive, pad, sound.clone(), false);
        }
        app_capture_frames(&mut drive, 0);
        app_capture_start(&mut drive, 1);
        let mut wave = (Vec::new(), Vec::new());
        app_capture_to(&mut drive, &mut wave, 12_000);
        drive.app.send(Command::PatternErase {
            pass: 1,
            enabled: true,
        });
        app_capture_frames(&mut drive, 0);
        drive.pad_press(source, 0);
        app_capture_to(&mut drive, &mut wave, 36_000);
        drive.pad_release(source, 0);
        drive.app.send(Command::PatternErase {
            pass: 1,
            enabled: false,
        });
        app_capture_finish(&mut drive, 1);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 7);
        app_capture_to(&mut drive, &mut wave, 160_000);
        completed_pass_undo(&mut drive, source);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, original);
        app_capture_to(&mut drive, &mut wave, 288_000);
        let mut reference = (vec![0.0f32; 288_000], vec![0.0f32; 288_000]);
        let mut onsets = Vec::new();
        for beat in 0..12 {
            for (offset, pad) in [(0, 0), (12_000, 1)] {
                let frame = beat * 24_000 + offset;
                if pad == 0 && [24_000, 120_000].contains(&frame) {
                    continue;
                }
                onsets.push(frame);
                for (actual, samples) in [
                    (&mut reference.0, &sounds[pad].left),
                    (&mut reference.1, sounds[pad].right.as_ref().unwrap()),
                ] {
                    for (target, &sample) in actual[frame..].iter_mut().zip(samples.iter()) {
                        *target += sample;
                    }
                }
            }
        }
        for (channel, reference) in [(&wave.0, &reference.0), (&wave.1, &reference.1)] {
            for (frame, (&actual, &expected)) in channel.iter().zip(reference).enumerate() {
                assert!(actual.is_finite() && actual.abs() < 1.0);
                assert!(
                    (actual - expected).abs() <= 0.000001,
                    "{source:?}, frame {frame}: {actual} != {expected}"
                );
            }
            let peak = channel.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
            assert!((0.01..0.06).contains(&peak));
            assert!(channel.windows(2).all(|w| (w[1] - w[0]).abs() < 0.01));
            for &frame in &onsets {
                let first = channel[frame..frame + 16]
                    .iter()
                    .position(|v| v.abs() > 1e-12)
                    .unwrap();
                assert!(first <= 1);
            }
            for start in [24_000, 120_000, 160_000, 283_200] {
                assert!(channel[start..start + 2400].iter().all(|&v| v == 0.0));
            }
        }
        if matches!(source, Source::Keyboard) {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../renders/97-pattern-app-erasure.wav");
            let mut writer = hound::WavWriter::create(
                path,
                hound::WavSpec {
                    channels: 2,
                    sample_rate: 48_000,
                    bits_per_sample: 32,
                    sample_format: hound::SampleFormat::Float,
                },
            )
            .unwrap();
            for (&left, &right) in wave.0.iter().zip(&wave.1) {
                writer.write_sample(left).unwrap();
                writer.write_sample(right).unwrap();
            }
            writer.finalize().unwrap();
            println!(
                "App erasure WAV: 288000 frames, stereo float32, 48000 Hz; 22 attacks within one frame, both channels match an independent sample-sum reference within 0.000001."
            );
        }
    }
}

#[test]
fn app_erasure_compile_failure_retains_work_and_reclaims_only_after_publication() {
    let mut drive = app_capture_drive(vec![[240, 0, 90, 0]]);
    app_capture_start(&mut drive, 1);
    drive.app.send(Command::PatternErase {
        pass: 1,
        enabled: true,
    });
    drive.app.send(Command::PatternEraseTarget {
        pass: 1,
        pad: 0,
        one_loop: false,
    });
    app_capture_frames(&mut drive, 7000);
    let epoch = app_capture_snapshot(&drive).epoch;
    drive
        .app
        .audio
        .as_mut()
        .unwrap()
        .commands
        .push(ProjectCommand {
            epoch,
            ready_at_send: true,
            command: ProjectMessage::Command(Command::PatternRecordEnd { pass: 1 }),
        })
        .unwrap();
    app_capture_frames(&mut drive, 0);
    drive.app.pads.pattern.as_mut().unwrap().strength = 101;
    drive.app.sent.clear();
    drive.app.flush_audio();
    assert_eq!(
        drive.last_notice().text,
        "Pattern strength must be within 0..100 in steps of 10."
    );
    assert!(drive.app.sent.is_empty());
    assert!(drive.app.history.is_empty());
    assert!(
        drive
            .app
            .audio
            .as_ref()
            .unwrap()
            .pattern_capture
            .blocks_autosave()
    );
    drive.app.pads.pattern.as_mut().unwrap().strength = 100;
    drive.app.flush_audio();
    let commands: Vec<_> = drive
        .app
        .sent
        .iter()
        .filter_map(|command| match command {
            Command::PublishCapturedPattern { .. } => Some("publish"),
            Command::PatternEraseConsumed { .. } => Some("consume"),
            Command::PatternRecordRows { .. } => Some("rows"),
            Command::PatternRecordAck { .. } => Some("ack"),
            _ => None,
        })
        .collect();
    assert_eq!(commands, ["publish", "consume", "rows", "ack"]);
    assert_eq!(drive.app.history.len(), 1);
    app_capture_frames(&mut drive, 0);
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[240, 0, 90, 0]]
    );
}

#[test]
fn app_erasure_sparse_reused_interval_slots_do_not_finish_or_confirm_twice() {
    let mut drive = app_capture_drive(vec![]);
    app_capture_start(&mut drive, 1);
    drive.app.send(Command::PatternErase {
        pass: 1,
        enabled: true,
    });
    for _ in 0..128 {
        drive.app.send(Command::PatternEraseTarget {
            pass: 1,
            pad: 0,
            one_loop: false,
        });
        app_capture_frames(&mut drive, 0);
        // Direct targets have no ordinary input hold, so release through the owned queue.
        let epoch = app_capture_snapshot(&drive).epoch;
        drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::PadRelease { pad: 0 }),
            })
            .unwrap();
        app_capture_frames(&mut drive, 0);
        drive.app.flush_audio();
        app_capture_frames(&mut drive, 0);
    }
    assert_eq!(app_capture_snapshot(&drive).erase_intervals, 128);
    drive.app.sent.clear();
    for _ in 0..8 {
        drive.app.flush_audio();
    }
    assert!(
        drive.app.sent.is_empty(),
        "an unchanged accepted prefix emits no duplicate confirmation"
    );
    app_capture_finish(&mut drive, 1);
    assert!(drive.app.history.is_empty());
    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
}

fn app_capture_drive(hits: Vec<[u32; 4]>) -> Drive {
    let mut drive = Drive::offline();
    drive.app.apply_session(session::Session {
        pattern: Some("A1".into()),
        patterns: vec![session::PatternState {
            slot: "A1".into(),
            bars: 1,
            quantize: "1/16".into(),
            strength: 100,
            hits,
        }],
        ..Default::default()
    });
    settle_pattern(&mut drive);
    drive
}

/// Quantize Off: a captured row keeps the tick of its accepted audio block (98c), so a test
/// can see the block timing.
fn app_capture_drive_raw(hits: Vec<[u32; 4]>) -> Drive {
    let mut drive = app_capture_drive(hits);
    drive.app.pads.pattern.as_mut().unwrap().quantize = "off".into();
    drive
}

fn app_capture_snapshot(drive: &Drive) -> engine::capture::Snapshot {
    let epoch = drive
        .project
        .epoch
        .load(std::sync::atomic::Ordering::Acquire);
    drive
        .app
        .audio
        .as_ref()
        .unwrap()
        .stats
        .recording(epoch)
        .unwrap()
}

fn app_capture_frames(drive: &mut Drive, frames: usize) -> Stereo {
    let (mut left, mut right) = (vec![0.0; frames], vec![0.0; frames]);
    for start in (0..frames).step_by(128) {
        let end = (start + 128).min(frames);
        drive
            .audio_rt
            .as_mut()
            .unwrap()
            .process(&mut left[start..end], &mut right[start..end]);
        drive.app.audio.as_mut().unwrap().free_garbage();
    }
    if frames == 0 {
        drive.audio_rt.as_mut().unwrap().process(&mut [], &mut []);
    }
    (left, right)
}

fn app_capture_start(drive: &mut Drive, pass: u64) {
    drive.app.send(Command::TransportPlay);
    assert!(drive.app.try_send(Command::PatternRecordStart { pass }));
    app_capture_frames(drive, 0);
    assert_eq!(
        app_capture_snapshot(drive).phase,
        engine::capture::Phase::Recording
    );
}

fn app_capture_finish(drive: &mut Drive, pass: u64) {
    drive.app.send(Command::PatternRecordEnd { pass });
    app_capture_frames(drive, 0);
    assert_eq!(
        app_capture_snapshot(drive).phase,
        engine::capture::Phase::Finishing
    );
    drive.app.flush_audio();
    app_capture_frames(drive, 0);
    assert_eq!(
        app_capture_snapshot(drive).phase,
        engine::capture::Phase::Idle
    );
}

fn app_capture_sound(drive: &mut Drive, pad: usize, sound: Arc<Sample>, gate: bool) {
    drive.load_pad(pad, sound.clone(), "capture");
    drive.app.pads.gate[pad] = gate;
    drive.app.pads.pitch[pad] = 0;
    for command in [
        Command::SetPadSound {
            pad,
            sound: Some(sound),
        },
        Command::SetPadMode {
            pad,
            mode: if gate {
                engine::PadMode::Gate
            } else {
                engine::PadMode::OneShot
            },
        },
        Command::SetPadPitch { pad, semitones: 0 },
    ] {
        drive.app.send(command);
    }
}

#[test]
fn app_capture_records_accepted_empty_pads_once_in_audio_order() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        let old = vec![[5000, 127, 90, 0], [960, 18, 70, 0]];
        let mut drive = app_capture_drive_raw(old.clone());
        app_capture_start(&mut drive, 1);
        app_capture_frames(&mut drive, 128);
        drive.pad_press(source, 2);
        let silent = app_capture_frames(&mut drive, 128);
        drive.pad_release(source, 2);
        app_capture_frames(&mut drive, 128);
        drive.app.flush_audio();
        let rows = &drive.app.pads.pattern.as_ref().unwrap().hits;
        assert_eq!(&rows[..old.len()], old);
        assert_eq!(rows.len(), old.len() + 1);
        assert_eq!(rows[old.len()], [5, 2, 127, 0]);
        assert!(silent.0.iter().chain(&silent.1).all(|&v| v == 0.0));
        let accepted = rows.clone();
        for _ in 0..5 {
            drive.app.flush_audio();
            app_capture_frames(&mut drive, 128);
        }
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, accepted);
        app_capture_finish(&mut drive, 1);
    }
}

#[test]
fn app_capture_pending_gate_finishes_the_original_row_once() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        let mut drive = app_capture_drive(vec![]);
        app_capture_sound(
            &mut drive,
            0,
            Arc::new(Sample::mono(vec![0.02; 240_000])),
            true,
        );
        app_capture_frames(&mut drive, 0);
        app_capture_start(&mut drive, 1);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 128);
        drive.app.flush_audio();
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[0, 0, 127, 1]]
        );
        app_capture_frames(&mut drive, 100_000);
        drive.app.flush_audio();
        assert_eq!(
            drive
                .app
                .audio
                .as_ref()
                .unwrap()
                .stats
                .pattern_hits(app_capture_snapshot(&drive).epoch)
                .unwrap()[0],
            0
        );
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 1);
        let end = app_capture_snapshot(&drive).ticks.round() as u32;
        app_capture_finish(&mut drive, 1);
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[0, 0, 127, end]]
        );
        let acks = drive
            .app
            .sent
            .iter()
            .filter(|c| matches!(c, Command::PatternRecordAck { pass: 1, .. }))
            .count();
        assert_eq!(acks, 1);
        let rows = drive.app.pads.pattern.clone();
        drive.pad_release(source, 0);
        app_capture_frames(&mut drive, 128);
        for _ in 0..5 {
            drive.app.flush_audio();
        }
        assert_eq!(drive.app.pads.pattern, rows);
        assert!(
            !drive
                .app
                .sent
                .iter()
                .any(|c| matches!(c, Command::PatternRecordAck { .. }))
        );
    }
}

#[test]
fn app_capture_interruption_freezes_the_last_accepted_gate_without_ack() {
    use std::sync::atomic::Ordering::Relaxed;
    let mut drive = app_capture_drive(vec![]);
    app_capture_sound(
        &mut drive,
        0,
        Arc::new(Sample::mono(vec![0.02; 48_000])),
        true,
    );
    app_capture_frames(&mut drive, 0);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 6000);
    drive.app.send(Command::PatternRecordEnd { pass: 1 });
    assert_eq!(
        app_capture_snapshot(&drive).phase,
        engine::capture::Phase::Recording
    );
    drive.audio_rt = None;
    drive
        .app
        .audio
        .as_ref()
        .unwrap()
        .stopped
        .store(true, Relaxed);
    drive.app.sent.clear();
    for _ in 0..20 {
        drive.app.flush_audio();
    }
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[0, 0, 127, 240]]
    );
    assert!(
        drive.app.sent.is_empty(),
        "no publication or fabricated acknowledgement after loss"
    );
    assert!(
        drive
            .app
            .notices
            .history()
            .iter()
            .any(|n| n.text.contains("Pattern recording interrupted"))
    );
    drive
        .app
        .audio
        .as_ref()
        .unwrap()
        .stopped
        .store(false, Relaxed);
    assert!(!drive.app.try_send(Command::PatternRecordStart { pass: 2 }));
    drive.app.flush_audio();
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[0, 0, 127, 240]]
    );
}

/// Scenario "Capture overflow preserves sound and complete recoverable rows".
#[test]
fn app_capture_overflow_keeps_only_accepted_onsets_and_recovers_lost_releases() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let old = [0, 127, 64, 0];
        let mut drive = app_capture_drive(vec![old]);
        let mut reference = app_capture_drive(vec![old]);
        for rig in [&mut drive, &mut reference] {
            rig.app.goto(Screen::Pads);
            app_capture_sound(rig, 0, Arc::new(Sample::mono(vec![0.02; 48000])), true);
            app_capture_sound(rig, 1, Arc::new(Sample::mono(vec![0.01; 48000])), false);
            app_capture_sound(rig, 127, Arc::new(Sample::mono(vec![0.005; 48000])), false);
            app_capture_frames(rig, 0);
        }
        app_capture_start(&mut drive, 1);
        reference.app.send(Command::TransportPlay);
        app_capture_frames(&mut reference, 0);
        for rig in [&mut drive, &mut reference] {
            rig.pad_press(source, 0);
        }
        assert_eq!(
            app_capture_frames(&mut drive, 128),
            app_capture_frames(&mut reference, 128)
        );
        for start in (0..639).step_by(128) {
            for i in start..(start + 128).min(639) {
                let pad = 1 + i % 15;
                for rig in [&mut drive, &mut reference] {
                    assert!(rig.app.try_send(Command::PadPress { pad, velocity: 1.0 }));
                }
            }
            let actual = app_capture_frames(&mut drive, 128);
            let expected = app_capture_frames(&mut reference, 128);
            assert!(
                actual == expected,
                "{source:?}: burst {start}, capture {:?}, free UI slots {}, reference free slots {}",
                app_capture_snapshot(&drive),
                drive.app.audio.as_mut().unwrap().commands.slots(),
                reference.app.audio.as_mut().unwrap().commands.slots()
            );
            assert!(
                actual
                    .0
                    .iter()
                    .chain(&actual.1)
                    .all(|sample| sample.is_finite() && sample.abs() < 1.0)
            );
            assert!(actual.0.iter().any(|sample| sample.abs() > 0.001));
        }
        for rig in [&mut drive, &mut reference] {
            if matches!(source, Source::Notes) {
                rig.bytes(PAD_PORT, &[0x80, 36, 0]);
                rig.midi().recorded_fence();
            } else {
                rig.pad_release(source, 0);
            }
        }
        assert_eq!(
            app_capture_frames(&mut drive, 128),
            app_capture_frames(&mut reference, 128)
        );
        let snapshot = app_capture_snapshot(&drive);
        assert_eq!(
            (
                snapshot.captured_onsets,
                snapshot.dropped_onsets,
                snapshot.dropped_events
            ),
            (
                512,
                128,
                if matches!(source, Source::Keyboard) {
                    128
                } else {
                    129
                }
            ),
            "{source:?}: source-owned release and complete feedback pressure"
        );
        drive.app.flush_audio();
        let rows = &drive.app.pads.pattern.as_ref().unwrap().hits;
        assert_eq!(rows.len(), 513);
        assert_eq!(rows[0], old);
        for (index, row) in rows[1..].iter().enumerate() {
            assert_eq!(
                row[1],
                if index == 0 {
                    0
                } else {
                    (1 + (index - 1) % 15) as u32
                }
            );
            assert_eq!(row[3], if index == 0 { 31 } else { 0 });
        }
        assert_eq!(drive.last_notice().level, Level::Warn);
        assert_eq!(
            drive.last_notice().text,
            format!(
                "Pattern recording: 128 omitted onsets, {} lost events, 0 row-limit refusals.",
                if matches!(source, Source::Keyboard) {
                    128
                } else {
                    129
                }
            )
        );
        app_capture_finish(&mut drive, 1);
        // the flood shares one step per pad: the completed pass keeps the newest row of
        // each pad and the old row of pad 128
        let rows = &drive.app.pads.pattern.as_ref().unwrap().hits;
        assert_eq!(rows.len(), 17);
        assert_eq!(rows[0], old);
        let mut pads: Vec<u32> = rows[1..].iter().map(|row| row[1]).collect();
        pads.sort_unstable();
        assert_eq!(pads, (0..16).collect::<Vec<u32>>());
        assert_eq!(
            app_capture_frames(&mut drive, 1024),
            app_capture_frames(&mut reference, 1024),
            "feedback recovery cannot add a late manual burst"
        );
        assert_eq!(drive.app.history.len(), 1);
    }
}

#[test]
fn app_capture_refuses_unavailable_or_unready_start_without_creating_a1() {
    use std::sync::atomic::Ordering::Release;
    let mut absent = Drive::new();
    assert!(!absent.app.try_send(Command::PatternRecordStart { pass: 1 }));
    assert!(absent.app.pads.pattern.is_none());
    let mut drive = Drive::offline();
    drive.project.epoch.store(1, Release);
    assert!(!drive.app.try_send(Command::PatternRecordStart { pass: 1 }));
    assert!(drive.app.pads.pattern.is_none());
    drive
        .app
        .audio
        .as_ref()
        .unwrap()
        .stopped
        .store(true, Release);
    drive.project.ready.store(1, Release);
    assert!(!drive.app.try_send(Command::PatternRecordStart { pass: 1 }));
    assert!(drive.app.pads.pattern.is_none());
}

#[test]
fn app_capture_restore_rejects_old_feedback_and_pass_identities() {
    let mut drive = app_capture_drive(vec![]);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    let old_epoch = app_capture_snapshot(&drive).epoch;
    drive.app.apply_session(session::Session::default());
    settle_pattern(&mut drive);
    assert!(drive.app.pads.pattern.is_none());
    assert!(drive.app.pads.pattern_rows.is_none());
    assert!(app_capture_snapshot(&drive).epoch > old_epoch);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 1);
    app_capture_frames(&mut drive, 128);
    drive.app.flush_audio();
    app_capture_finish(&mut drive, 1);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[0, 1, 127, 0]]
    );
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, 2);
}

#[test]
fn app_capture_initial_publication_and_updates_preserve_old_gate_tails() {
    for pitch_gate in [false, true] {
        let mut drive = app_capture_drive(vec![[0, 0, 127, 960]]);
        app_capture_sound(
            &mut drive,
            0,
            Arc::new(Sample::mono(vec![0.02; 96_000])),
            true,
        );
        drive.app.send(Command::SetPadPitch {
            pad: 0,
            semitones: if pitch_gate { 12 } else { 0 },
        });
        drive.app.send(Command::SetPadPitchGate {
            pad: 0,
            enabled: pitch_gate,
            linked_slice: true,
        });
        drive.app.send(Command::TransportPlay);
        let first = app_capture_frames(&mut drive, 128);
        app_capture_start(&mut drive, 1);
        drive.pad_press(Source::Notes, 1);
        let second = app_capture_frames(&mut drive, 128);
        drive.app.flush_audio();
        let third = app_capture_frames(&mut drive, 30_000 - 256);
        let cut = if pitch_gate { 6000 } else { 24_000 };
        for (a, b, c) in [
            (&first.0, &second.0, &third.0),
            (&first.1, &second.1, &third.1),
        ] {
            let wave: Vec<_> = a.iter().chain(b).chain(c).copied().collect();
            assert!(wave[..cut].iter().all(|&v| v == 0.02));
            assert!(wave[cut + 240..].iter().all(|&v| v == 0.0));
        }
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 2);
    }
}

#[test]
fn app_capture_pre_edit_identities_prevent_grid_replay() {
    let mut drive = app_capture_drive(vec![[121, 0, 127, 0]]);
    drive.app.pads.pattern.as_mut().unwrap().quantize = "off".into();
    let raw = drive
        .app
        .pads
        .pattern
        .as_ref()
        .unwrap()
        .checked()
        .unwrap()
        .compile();
    drive.app.send(Command::SetPattern {
        pattern: Some(Arc::new(raw)),
    });
    app_capture_sound(
        &mut drive,
        0,
        Arc::new(Sample::mono(vec![0.02; 128])),
        false,
    );
    drive.app.send(Command::TransportPlay);
    let before = app_capture_frames(&mut drive, 3200);
    assert_eq!(before.0[3025], 0.02);
    app_capture_start(&mut drive, 1);
    drive.app.pads.pattern.as_mut().unwrap().quantize = "1/16".into();
    drive
        .app
        .send(drive.app.pads.recorded_pattern_command().unwrap());
    let after = app_capture_frames(&mut drive, 4000);
    assert!(after.0.iter().chain(&after.1).all(|&v| v == 0.0));
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[121, 0, 127, 0]]
    );
}

/// Scenario "Record at the raw hit limit".
#[test]
fn app_capture_raw_limit_counts_hidden_rows_and_reserves_one_last_hit() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let old = vec![[5000, 127, 1, 0]; 4095];
        let mut drive = app_capture_drive_raw(old.clone());
        record_controls_routes(&mut drive, source);
        app_capture_sound(
            &mut drive,
            1,
            Arc::new(Sample::mono(vec![0.02; 48000])),
            false,
        );
        app_capture_start(&mut drive, 1);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 128);
        drive.pad_release(source, 0);
        drive.pad_press(source, 1);
        let refused = app_capture_frames(&mut drive, 128);
        assert!(
            refused.0.iter().all(|sample| *sample == 0.02),
            "a full pattern does not mute manual sound"
        );
        drive.pad_release(source, 1);
        drive.app.flush_audio();
        let rows = &drive.app.pads.pattern.as_ref().unwrap().hits;
        assert_eq!(&rows[..4095], old);
        assert_eq!(rows.len(), 4096);
        assert_eq!(app_capture_snapshot(&drive).limit_refusals, 1);
        assert_eq!(drive.last_notice().level, Level::Warn);
        assert_eq!(
            drive.last_notice().text,
            "Pattern recording: 1 omitted onsets, 1 lost events, 1 row-limit refusals."
        );
        record_controls_erase(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 96000);
        drive.pad_release(source, 0);
        app_capture_frames(&mut drive, 0);
        record_controls_erase(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        drive.app.flush_audio();
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, old);
        drive.pad_press(source, 2);
        app_capture_frames(&mut drive, 128);
        drive.pad_release(source, 2);
        app_capture_finish(&mut drive, 1);
        let rows = &drive.app.pads.pattern.as_ref().unwrap().hits;
        assert_eq!(&rows[..4095], old);
        assert_eq!(rows.len(), 4096);
        assert_eq!(rows[4095], [10, 2, 127, 0]);
        assert_eq!(app_capture_snapshot(&drive).limit_refusals, 1);
        assert_eq!(drive.app.history.len(), 1);
    }
}

/// Scenario "Cancel count in with a pad held".
#[test]
fn app_capture_count_in_cancellation_keeps_default_a1_without_rows() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for escape in [false, true] {
            let mut drive = Drive::offline();
            record_controls_routes(&mut drive, source);
            app_capture_sound(
                &mut drive,
                0,
                Arc::new(Sample::mono(vec![0.02; 48000])),
                true,
            );
            app_capture_frames(&mut drive, 0);
            record_controls_record(&mut drive, source);
            app_capture_frames(&mut drive, 6000);
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::CountIn
            );
            drive.pad_press(source, 0);
            let warm = app_capture_frames(&mut drive, 6000);
            assert!(warm.0.iter().all(|sample| *sample == 0.02));
            assert!(!drive.app.try_send(Command::PatternRecordStart { pass: 2 }));
            if escape {
                drive.tap(KeyCode::Esc, M::NONE);
            } else {
                record_controls_record(&mut drive, source);
            }
            record_controls_idle(&mut drive);
            let held = app_capture_frames(&mut drive, 128);
            assert!(held.0.iter().all(|sample| *sample == 0.02));
            drive.pad_release(source, 0);
            let tail = app_capture_frames(&mut drive, 512);
            assert!(
                tail.0[240..]
                    .iter()
                    .chain(&tail.1[240..])
                    .all(|sample| *sample == 0.0)
            );
            drive.app.flush_audio();
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::Idle
            );
            assert!(!drive.app.transport_state().playing);
            let raw = drive.app.pads.pattern.as_ref().unwrap();
            assert_eq!(
                (raw.bars, raw.quantize.as_str(), raw.strength),
                (2, "1/16", 100)
            );
            assert!(raw.hits.is_empty());
            assert!(drive.app.history.is_empty());
        }
    }
}

#[test]
fn app_capture_full_command_queue_refuses_onsets_but_retains_final_publication() {
    let mut drive = app_capture_drive(vec![]);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    let epoch = app_capture_snapshot(&drive).epoch;
    let audio = drive.app.audio.as_mut().unwrap();
    for _ in 0..1024 {
        audio
            .commands
            .push(ProjectCommand {
                epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .unwrap();
    }
    assert!(!drive.app.try_send(Command::PadPress {
        pad: 1,
        velocity: 1.0
    }));
    drive.app.send(Command::PatternRecordEnd { pass: 1 });
    assert!(!drive.app.pending_audio.is_empty());
    let mut completed = false;
    for _ in 0..64 {
        drive.app.flush_audio();
        app_capture_frames(&mut drive, 128);
        if app_capture_snapshot(&drive).phase == engine::capture::Phase::Idle {
            completed = true;
            break;
        }
    }
    assert!(completed);
    assert!(drive.app.pending_audio.is_empty());
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[0, 0, 127, 0]]
    );
    assert_eq!(
        drive
            .app
            .sent
            .iter()
            .filter(|c| matches!(c, Command::PatternRecordAck { pass: 1, .. }))
            .count(),
        1
    );
}

#[test]
fn app_capture_raw_midi_keeps_velocity_for_muted_pads_and_rejects_unready_onsets() {
    use std::sync::atomic::Ordering::Release;
    let mut drive = app_capture_drive_raw(vec![]);
    app_capture_sound(
        &mut drive,
        0,
        Arc::new(Sample::mono(vec![0.02; 48_000])),
        false,
    );
    drive.app.send(Command::SetPadGain { pad: 0, gain: 0.0 });
    app_capture_frames(&mut drive, 128);
    app_capture_start(&mut drive, 1);
    for velocity in [1, 63, 127] {
        drive.bytes(PAD_PORT, &[0x90, 36, velocity]);
        drive.bytes(PAD_PORT, &[0x80, 36, 0]);
        drive.midi().recorded_fence();
        let wave = app_capture_frames(&mut drive, 128);
        assert!(wave.0.iter().chain(&wave.1).all(|&v| v == 0.0));
    }
    drive.app.flush_audio();
    let raw = &drive.app.pads.pattern.as_ref().unwrap().hits;
    assert_eq!(raw, &[[0, 0, 1, 0], [5, 0, 63, 0], [10, 0, 127, 0]]);
    let previous = raw.clone();
    let epoch = app_capture_snapshot(&drive).epoch;
    drive.project.ready.store(epoch + 1, Release);
    assert!(!drive.app.try_send(Command::PadPress {
        pad: 1,
        velocity: 1.0
    }));
    drive.bytes(PAD_PORT, &[0x90, 37, 127]);
    drive.bytes(PAD_PORT, &[0x80, 37, 0]);
    drive.midi().recorded_fence();
    app_capture_frames(&mut drive, 128);
    drive.frame();
    drive.app.flush_audio();
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, previous);
}

/// Scenario "Stop closes the take before resetting the clock".
#[test]
fn app_capture_stop_uses_the_pre_reset_boundary_and_next_pass_appends_once() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![]);
        record_controls_routes(&mut drive, source);
        learn::assign(
            &mut drive.app.learned,
            "Recorded pads",
            input::Group::Global,
            if matches!(source, Source::Notes) {
                input::midi::Key::Note(106)
            } else {
                input::midi::Key::Cc(106)
            },
            "transport.toggle",
        );
        drive
            .app
            .sync_learned("Recorded pads", input::Group::Global);
        drive.frame();
        drive.app.goto(Screen::Pads);
        app_capture_sound(
            &mut drive,
            0,
            Arc::new(Sample::mono(vec![0.02; 48000])),
            true,
        );
        app_capture_frames(&mut drive, 0);
        app_capture_start(&mut drive, 1);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 6000);
        drive.registry_command(source, KeyCode::Char(' '), M::NONE, 106);
        app_capture_frames(&mut drive, 0);
        assert_eq!(drive.app.transport_state().ticks, 0.0);
        assert!((app_capture_snapshot(&drive).ticks - 240.0).abs() < 1e-8);
        record_controls_idle(&mut drive);
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[0, 0, 127, 240]]
        );
        assert_eq!(drive.app.history.len(), 1);
        assert!(!drive.app.try_send(Command::PatternRecordStart { pass: 1 }));
        app_capture_start(&mut drive, 2);
        drive.pad_release(source, 0);
        drive.pad_press(source, 1);
        app_capture_frames(&mut drive, 128);
        drive.pad_release(source, 1);
        app_capture_finish(&mut drive, 2);
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[0, 0, 127, 240], [0, 1, 127, 0]]
        );
        assert_eq!(drive.app.history.len(), 2);
    }
}

/// Scenario "Undo an overdub without losing earlier passes".
#[test]
fn completed_pass_undo_keeps_earlier_passes_without_time_merging() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![[960, 18, 70, 0]]);
        swap_routes(&mut drive, source);
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48_000])), 2);
        drive.app.chop.as_mut().unwrap().cut(12_000);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.app.edit = true;
        drive.app.pads.param = pads::Param::Volume;
        drive.tap(KeyCode::Right, M::NONE);
        drive.tap(KeyCode::F(4), M::NONE);
        drive.app.fx_param = 1;
        drive.tap(KeyCode::Right, M::NONE);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.app.edit = false;
        assert_eq!(drive.app.history.len(), 2);
        let unrelated = drive.app.before();
        let chop = drive.app.chop.as_ref().unwrap().saved();
        for pass in 1..=2 {
            if let Some(last) = drive.app.history.last_mut() {
                last.at = Instant::now() + Duration::from_secs(3600);
            }
            app_capture_start(&mut drive, pass);
            drive.pad_press(source, pass as usize - 1);
            app_capture_frames(&mut drive, 128);
            drive.pad_release(source, pass as usize - 1);
            app_capture_finish(&mut drive, pass);
            assert_eq!(drive.app.history.len(), pass as usize + 2);
        }
        let mut raw = drive.app.pads.pattern.clone().unwrap();
        raw.hits.truncate(2);
        let descriptors = drive.app.pads.pattern_rows.clone().unwrap();
        drive.registry_command(source, KeyCode::Char('z'), M::CONTROL, 111);
        app_capture_frames(&mut drive, 0);
        assert_eq!(drive.app.history.len(), 3);
        assert_eq!(drive.app.pads.pattern.as_ref(), Some(&raw));
        assert_eq!(
            drive.app.pads.pattern_rows.as_deref(),
            Some(&descriptors[..2])
        );
        assert!(drive.app.transport_state().playing);
        drive.registry_command(source, KeyCode::Char('z'), M::CONTROL, 111);
        app_capture_frames(&mut drive, 0);
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[960, 18, 70, 0]]
        );
        assert_eq!(drive.app.history.len(), 2);
        for (pad, snapshot) in unrelated.pads.iter().enumerate() {
            assert_eq!(&drive.app.pads.snapshot(pad), snapshot);
        }
        assert_eq!(drive.app.pads.fx, unrelated.fx);
        assert_eq!(drive.app.chop.as_ref().unwrap().saved(), chop);
    }
}

fn pattern_autosave_drive(project: bool) -> (TransitionFiles, Drive) {
    let files = TransitionFiles::new();
    let mut drive = app_capture_drive(vec![[960, 18, 70, 0]]);
    drive.app.session_path = files.0.join("scratch/session.toml");
    drive.app.session_read_error = None;
    if project {
        drive.app.project = Some(Project {
            name: "A".into(),
            folder: files.0.join("A"),
        });
    }
    drive.app.save_session().unwrap();
    (files, drive)
}

/// Scenario "Autosave cannot write a partial gate take".
#[test]
fn pattern_autosave_cannot_write_a_partial_gate_take() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for project in [false, true] {
            let (_files, mut drive) = pattern_autosave_drive(project);
            app_capture_sound(
                &mut drive,
                0,
                Arc::new(Sample::mono(vec![0.02; 48_000])),
                true,
            );
            app_capture_frames(&mut drive, 0);
            let file = drive.app.session_file();
            let stored = std::fs::read(&file).unwrap();
            assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
            app_capture_start(&mut drive, 1);
            drive.pad_press(source, 0);
            app_capture_frames(&mut drive, 6000);
            drive.app.flush_audio();
            assert_eq!(drive.app.save_status(), tui::SaveStatus::Unsaved);
            let pending = drive.app.session_dirty.unwrap();
            drive
                .app
                .autosave(pending + SESSION_SAVE + Duration::from_secs(1));
            assert_eq!(std::fs::read(&file).unwrap(), stored);
            assert_eq!(drive.app.session_dirty, Some(pending));
            assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits[1][3], 1);
            app_capture_finish(&mut drive, 1);
            drive
                .app
                .autosave(pending + SESSION_SAVE + Duration::from_secs(1));
            let saved = session::load_existing(&file).unwrap();
            assert_eq!(saved.version, 5);
            assert_eq!(saved.patterns, [drive.app.pads.pattern.clone().unwrap()]);
            assert_eq!(saved.patterns[0].hits[1], [0, 0, 127, 240]);
            assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
            assert_eq!(drive.app.history.len(), 1);
        }
    }
}

/// Scenario "Autosave cannot write a partial gate take".
#[test]
fn pattern_autosave_waits_for_count_in_and_saves_canceled_default_a1() {
    for project in [false, true] {
        let (_files, mut drive) = pattern_autosave_drive(project);
        drive.app.apply_session(session::Session::default());
        settle_pattern(&mut drive);
        drive.app.save_session().unwrap();
        let file = drive.app.session_file();
        let stored = std::fs::read(&file).unwrap();
        assert!(drive.app.try_send(Command::PatternRecordStart { pass: 1 }));
        let pending = drive.app.session_dirty.unwrap();
        let due = pending + SESSION_SAVE + Duration::from_secs(1);
        drive.app.autosave(due);
        assert_eq!(std::fs::read(&file).unwrap(), stored);
        app_capture_frames(&mut drive, 6000);
        assert_eq!(
            app_capture_snapshot(&drive).phase,
            engine::capture::Phase::CountIn
        );
        drive.pad_press(Source::Notes, 0);
        app_capture_frames(&mut drive, 128);
        drive.app.autosave(due);
        assert_eq!(std::fs::read(&file).unwrap(), stored);
        assert_eq!(drive.app.save_status(), tui::SaveStatus::Unsaved);
        drive.app.send(Command::PatternRecordEnd { pass: 1 });
        app_capture_frames(&mut drive, 0);
        drive.app.autosave(due);
        let saved = session::load_existing(&file).unwrap();
        assert_eq!(saved.pattern.as_deref(), Some("A1"));
        assert_eq!(
            saved.patterns[0],
            session::PatternState {
                slot: "A1".into(),
                bars: 2,
                quantize: "1/16".into(),
                strength: 100,
                hits: vec![],
            }
        );
        assert!(!drive.app.transport_state().playing);
        assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
        assert!(drive.app.history.is_empty());
        assert!(
            !drive
                .app
                .notices
                .history()
                .iter()
                .any(|n| n.text == "Session saved")
        );
    }
}

#[test]
fn pattern_autosave_waits_for_live_finishing_and_a_queued_acknowledgement() {
    let (_files, mut drive) = pattern_autosave_drive(false);
    app_capture_sound(
        &mut drive,
        0,
        Arc::new(Sample::mono(vec![0.02; 48_000])),
        true,
    );
    app_capture_frames(&mut drive, 0);
    drive.app.save_session().unwrap();
    let file = drive.app.session_file();
    let stored = std::fs::read(&file).unwrap();
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 6000);
    drive.app.send(Command::PatternRecordEnd { pass: 1 });
    app_capture_frames(&mut drive, 0);
    let pending = drive.app.session_dirty.unwrap();
    let due = pending + SESSION_SAVE + Duration::from_secs(1);
    let epoch = app_capture_snapshot(&drive).epoch;
    assert_eq!(
        app_capture_snapshot(&drive).phase,
        engine::capture::Phase::Finishing
    );
    for _ in 0..1024 {
        drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .unwrap();
    }
    drive.app.autosave(due);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits[1][3], 240);
    assert_eq!(drive.app.history.len(), 1);
    assert!(!drive.app.pending_audio.is_empty());
    assert_eq!(std::fs::read(&file).unwrap(), stored);
    assert_eq!(drive.app.session_dirty, Some(pending));
    let mut idle = false;
    for _ in 0..32 {
        app_capture_frames(&mut drive, 0);
        let phase = app_capture_snapshot(&drive).phase;
        drive.app.autosave(due);
        if phase == engine::capture::Phase::Idle {
            idle = true;
            break;
        }
        assert_eq!(phase, engine::capture::Phase::Finishing);
        assert_eq!(std::fs::read(&file).unwrap(), stored);
        assert_eq!(drive.app.save_status(), tui::SaveStatus::Unsaved);
    }
    assert!(idle);
    assert_eq!(
        session::load_existing(&file).unwrap().patterns[0].hits[1][3],
        240
    );
    assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
    assert_eq!(drive.app.history.len(), 1);
}

#[test]
fn pattern_autosave_collects_accepted_feedback_before_its_save_decision() {
    let (_files, mut drive) = pattern_autosave_drive(false);
    let file = drive.app.session_file();
    let stored = std::fs::read(&file).unwrap();
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 1);
    assert!(drive.app.session_dirty.is_none());
    drive
        .app
        .autosave(Instant::now() + SESSION_SAVE + Duration::from_secs(1));
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 2);
    assert_eq!(drive.app.save_status(), tui::SaveStatus::Unsaved);
    assert_eq!(std::fs::read(&file).unwrap(), stored);
}

/// Scenario "Audio loss during finishing retains a saveable take".
#[test]
fn pattern_autosave_saves_recovered_gates_without_a_stopped_callback_ack() {
    use std::sync::atomic::Ordering::Relaxed;
    for (project, source) in [false, true].into_iter().flat_map(|project| {
        [Source::Keyboard, Source::Notes, Source::Button].map(move |source| (project, source))
    }) {
        for finishing in [false, true] {
            let (_files, mut drive) = pattern_autosave_drive(project);
            app_capture_sound(
                &mut drive,
                0,
                Arc::new(Sample::mono(vec![0.02; 48_000])),
                true,
            );
            app_capture_frames(&mut drive, 0);
            drive.app.save_session().unwrap();
            app_capture_start(&mut drive, 1);
            drive.pad_press(source, 0);
            app_capture_frames(&mut drive, 6000);
            drive.app.send(Command::PatternRecordEnd { pass: 1 });
            if finishing {
                app_capture_frames(&mut drive, 0);
            }
            drive.audio_rt = None;
            drive
                .app
                .audio
                .as_ref()
                .unwrap()
                .stopped
                .store(true, Relaxed);
            drive.app.sent.clear();
            let due = Instant::now() + SESSION_SAVE + Duration::from_secs(1);
            drive.app.autosave(due);
            let file = drive.app.session_file();
            let saved = session::load_existing(&file).unwrap();
            assert_eq!(saved.patterns[0].hits, [[960, 18, 70, 0], [0, 0, 127, 240]]);
            assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
            assert_eq!(drive.app.history.len(), 1);
            assert!(drive.app.sent.is_empty());
            let stored = std::fs::read(&file).unwrap();
            for _ in 0..5 {
                drive.app.autosave(due);
            }
            assert_eq!(std::fs::read(&file).unwrap(), stored);
            assert_eq!(drive.app.history.len(), 1);
            drive.tap(KeyCode::Char('z'), M::CONTROL);
            drive.app.autosave(due);
            assert_eq!(
                session::load_existing(&file).unwrap().patterns[0].hits,
                [[960, 18, 70, 0]]
            );
            assert!(drive.app.history.is_empty());
            assert!(drive.app.sent.is_empty());
            assert!(
                drive
                    .app
                    .notices
                    .history()
                    .iter()
                    .any(|n| n.text.contains("Pattern recording interrupted"))
            );
        }
    }
}

#[test]
fn pattern_autosave_empty_existing_pass_and_runtime_descriptors_stay_clean() {
    let (_files, mut drive) = pattern_autosave_drive(false);
    let file = drive.app.session_file();
    let stored = std::fs::read(&file).unwrap();
    assert!(drive.app.pads.pattern_rows.is_none());
    app_capture_start(&mut drive, 1);
    drive
        .app
        .autosave(Instant::now() + SESSION_SAVE + Duration::from_secs(1));
    assert!(drive.app.session_dirty.is_none());
    assert!(drive.app.pads.pattern_rows.is_some());
    app_capture_finish(&mut drive, 1);
    drive
        .app
        .autosave(Instant::now() + SESSION_SAVE + Duration::from_secs(1));
    assert_eq!(std::fs::read(&file).unwrap(), stored);
    assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
    assert!(drive.app.history.is_empty());
}

#[test]
fn pattern_autosave_completed_setting_only_pass_marks_unsaved_work() {
    let (_files, mut drive) = pattern_autosave_drive(false);
    app_capture_start(&mut drive, 1);
    let pattern = drive.app.pads.pattern.as_mut().unwrap();
    pattern.quantize = "1/8".into();
    pattern.strength = 50;
    drive
        .app
        .send(drive.app.pads.recorded_pattern_command().unwrap());
    app_capture_finish(&mut drive, 1);
    assert_eq!(drive.app.save_status(), tui::SaveStatus::Unsaved);
    let due = drive.app.session_dirty.unwrap() + SESSION_SAVE + Duration::from_secs(1);
    drive.app.autosave(due);
    let saved = session::load_existing(&drive.app.session_file()).unwrap();
    assert_eq!(
        (
            saved.patterns[0].quantize.as_str(),
            saved.patterns[0].strength
        ),
        ("1/8", 50)
    );
    assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
    assert_eq!(drive.app.history.len(), 1);
}

#[test]
fn pattern_autosave_keeps_the_original_pending_time_through_updates_next_pass_and_undo() {
    let (_files, mut drive) = pattern_autosave_drive(false);
    app_capture_sound(
        &mut drive,
        0,
        Arc::new(Sample::mono(vec![0.02; 48_000])),
        true,
    );
    app_capture_frames(&mut drive, 0);
    drive.app.save_session().unwrap();
    let file = drive.app.session_file();
    let stored = std::fs::read(&file).unwrap();
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    drive.app.flush_audio();
    let pending = drive.app.session_dirty.unwrap();
    drive.pad_press(Source::Notes, 1);
    app_capture_frames(&mut drive, 128);
    drive.pad_release(Source::Notes, 0);
    app_capture_frames(&mut drive, 0);
    drive.app.flush_audio();
    assert_eq!(drive.app.session_dirty, Some(pending));
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits[1][3], 10);
    app_capture_finish(&mut drive, 1);
    app_capture_start(&mut drive, 2);
    let due = pending + SESSION_SAVE + Duration::from_secs(1);
    drive.app.autosave(due);
    assert_eq!(drive.app.session_dirty, Some(pending));
    assert_eq!(std::fs::read(&file).unwrap(), stored);
    app_capture_finish(&mut drive, 2);
    drive.app.autosave(due);
    assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
    assert_eq!(drive.app.history.len(), 1);
    assert_eq!(
        session::load_existing(&file).unwrap().patterns[0]
            .hits
            .len(),
        3
    );
    completed_pass_undo(&mut drive, Source::Keyboard);
    assert_eq!(drive.app.save_status(), tui::SaveStatus::Unsaved);
    drive.app.autosave(due);
    assert_eq!(
        session::load_existing(&file).unwrap().patterns[0].hits,
        [[960, 18, 70, 0]]
    );
    assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
}

#[test]
fn pattern_autosave_preserves_read_write_and_missing_folder_protection() {
    for problem in ["write", "damaged", "newer", "missing"] {
        let (files, mut drive) = pattern_autosave_drive(true);
        let file = drive.app.session_file();
        let valid = std::fs::read(&file).unwrap();
        let tmp = file.with_extension("toml.tmp");
        if problem == "write" {
            std::fs::write(&tmp, b"occupied").unwrap();
        } else if problem == "missing" {
            std::fs::remove_dir_all(files.0.join("A")).unwrap();
        } else {
            std::fs::write(
                &file,
                if problem == "damaged" {
                    "version = [\n"
                } else {
                    "version = 99\n"
                },
            )
            .unwrap();
            drive.app.session_read_error = Some(session::load_existing(&file).unwrap_err());
        }
        app_capture_start(&mut drive, 1);
        drive.pad_press(Source::Notes, 0);
        app_capture_frames(&mut drive, 128);
        app_capture_finish(&mut drive, 1);
        let pending = drive.app.session_dirty.unwrap();
        let due = pending + SESSION_SAVE + Duration::from_secs(1);
        let protection = drive.app.session_read_error.clone();
        drive.app.autosave(due);
        assert_eq!(drive.app.history.len(), 1);
        assert_eq!(drive.app.session_dirty, Some(pending));
        assert_eq!(drive.app.session_read_error, protection);
        assert!(
            !drive
                .app
                .notices
                .history()
                .iter()
                .any(|n| n.text == "Session saved")
        );
        if problem == "missing" {
            assert!(!files.0.join("A").exists());
        } else if problem == "write" {
            assert_eq!(std::fs::read(&file).unwrap(), valid);
            assert_eq!(std::fs::read(&tmp).unwrap(), b"occupied");
        } else {
            assert_eq!(
                std::fs::read_to_string(&file).unwrap(),
                if problem == "damaged" {
                    "version = [\n"
                } else {
                    "version = 99\n"
                }
            );
        }
        assert_eq!(
            drive.app.save_status(),
            if protection.is_some() {
                tui::SaveStatus::Unsaved
            } else {
                tui::SaveStatus::Failed
            }
        );
        if problem == "write" {
            std::fs::remove_file(&tmp).unwrap();
        }
        if problem == "missing" {
            std::fs::create_dir_all(files.0.join("A")).unwrap();
        }
        std::fs::write(&file, &valid).unwrap();
        drive.app.autosave(due);
        assert_eq!(
            std::fs::read(&file).unwrap(),
            valid,
            "no automatic retry or removed read protection"
        );
        drive.tap(KeyCode::Char('s'), M::CONTROL);
        assert_eq!(drive.app.history.len(), 1);
        if protection.is_none() {
            assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
            assert_eq!(drive.last_notice().text, "Session saved");
            assert_eq!(
                session::load_existing(&file).unwrap().patterns[0]
                    .hits
                    .len(),
                2
            );
        } else {
            assert_eq!(drive.app.save_status(), tui::SaveStatus::Unsaved);
            assert_ne!(drive.last_notice().text, "Session saved");
            assert_eq!(std::fs::read(&file).unwrap(), valid);
        }
    }
}

#[test]
fn pattern_autosave_project_reset_removes_old_capture_deferral_and_pending_state() {
    let (files, mut drive) = pattern_autosave_drive(true);
    let source = drive.app.session_file();
    let stored = std::fs::read(&source).unwrap();
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    drive
        .app
        .autosave(Instant::now() + SESSION_SAVE + Duration::from_secs(1));
    assert_eq!(drive.app.save_status(), tui::SaveStatus::Unsaved);
    let old_epoch = app_capture_snapshot(&drive).epoch;
    drive.app.apply_session(session::Session::default());
    drive.app.project = Some(Project {
        name: "B".into(),
        folder: files.0.join("B"),
    });
    settle_pattern(&mut drive);
    assert!(app_capture_snapshot(&drive).epoch > old_epoch);
    assert!(drive.app.session_dirty.is_none());
    assert!(drive.app.history.is_empty());
    drive.tap(KeyCode::Char(']'), M::NONE);
    let due = drive.app.session_dirty.unwrap() + SESSION_SAVE + Duration::from_secs(1);
    drive.app.autosave(due);
    let saved = session::load_existing(&drive.app.session_file()).unwrap();
    assert_eq!(saved.master_db, drive.app.master_db);
    assert!(saved.patterns.is_empty());
    assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
    assert_eq!(std::fs::read(&source).unwrap(), stored);
}

#[test]
fn pattern_autosave_defers_a_requested_start_before_audio_acceptance() {
    let (_files, mut drive) = pattern_autosave_drive(false);
    drive.tap(KeyCode::Char(']'), M::NONE);
    app_capture_frames(&mut drive, 0);
    let file = drive.app.session_file();
    let stored = std::fs::read(&file).unwrap();
    let pending = drive.app.session_dirty.unwrap();
    let due = pending + SESSION_SAVE + Duration::from_secs(1);
    let epoch = app_capture_snapshot(&drive).epoch;
    for _ in 0..1024 {
        drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .unwrap();
    }
    assert!(drive.app.try_send(Command::PatternRecordStart { pass: 1 }));
    assert_eq!(
        app_capture_snapshot(&drive).phase,
        engine::capture::Phase::Idle
    );
    assert!(!drive.app.try_send(Command::PatternRecordStart { pass: 2 }));
    drive.app.autosave(due);
    assert_eq!(std::fs::read(&file).unwrap(), stored);
    let mut count_in = false;
    for _ in 0..32 {
        app_capture_frames(&mut drive, 0);
        drive.app.autosave(due);
        assert_eq!(std::fs::read(&file).unwrap(), stored);
        if app_capture_snapshot(&drive).phase == engine::capture::Phase::CountIn {
            count_in = true;
            break;
        }
    }
    assert!(count_in);
    drive.app.send(Command::PatternRecordEnd { pass: 1 });
    app_capture_frames(&mut drive, 0);
    drive.app.autosave(due);
    assert_eq!(
        session::load_existing(&file).unwrap().master_db,
        drive.app.master_db
    );
    assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
    assert_eq!(drive.app.session_dirty, None);
}

#[test]
fn pattern_autosave_lost_start_or_count_in_keeps_only_default_a1() {
    use std::sync::atomic::Ordering::Relaxed;
    for accepted in [false, true] {
        let (_files, mut drive) = pattern_autosave_drive(false);
        drive.app.apply_session(session::Session::default());
        settle_pattern(&mut drive);
        drive.app.save_session().unwrap();
        assert!(drive.app.try_send(Command::PatternRecordStart { pass: 1 }));
        if accepted {
            app_capture_frames(&mut drive, 128);
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::CountIn
            );
        } else {
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::Idle
            );
        }
        drive.audio_rt = None;
        drive
            .app
            .audio
            .as_ref()
            .unwrap()
            .stopped
            .store(true, Relaxed);
        drive.app.sent.clear();
        drive
            .app
            .autosave(Instant::now() + SESSION_SAVE + Duration::from_secs(1));
        let saved = session::load_existing(&drive.app.session_file()).unwrap();
        assert_eq!(saved.pattern.as_deref(), Some("A1"));
        assert!(saved.patterns[0].hits.is_empty());
        assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
        assert!(drive.app.history.is_empty());
        assert!(drive.app.sent.is_empty());
    }
}

fn pattern_manual_save_source(drive: &mut Drive, source: Source) {
    learn::assign(
        &mut drive.app.learned,
        "Recorded pads",
        input::Group::Global,
        match source {
            Source::Notes => input::midi::Key::Note(91),
            _ => input::midi::Key::Cc(91),
        },
        "session.save",
    );
    drive
        .app
        .sync_learned("Recorded pads", input::Group::Global);
    drive.midi().recorded_fence();
}

fn pattern_manual_save_successes(drive: &Drive) -> usize {
    drive
        .app
        .notices
        .history()
        .iter()
        .filter(|n| n.text == "Session saved")
        .count()
}

/// Scenario "Manual save waits for a complete recording boundary".
#[test]
fn pattern_manual_save_waits_for_complete_gate_and_reports_only_after_write() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for project in [false, true] {
            let (_files, mut drive) = pattern_autosave_drive(project);
            pattern_manual_save_source(&mut drive, source);
            app_capture_sound(
                &mut drive,
                0,
                Arc::new(Sample::mono(vec![0.02; 48_000])),
                true,
            );
            app_capture_frames(&mut drive, 0);
            drive.app.save_session().unwrap();
            let file = drive.app.session_file();
            let stored = std::fs::read(&file).unwrap();
            app_capture_start(&mut drive, 1);
            drive.pad_press(source, 0);
            app_capture_frames(&mut drive, 6000);
            drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
            assert_eq!(pattern_manual_save_successes(&drive), 0);
            assert_eq!(std::fs::read(&file).unwrap(), stored);
            for _ in 0..32 {
                app_capture_frames(&mut drive, 0);
                drive.app.flush_audio();
                if pattern_manual_save_successes(&drive) != 0 {
                    break;
                }
            }
            assert_eq!(pattern_manual_save_successes(&drive), 1);
            let saved = session::load_existing(&file).unwrap();
            assert_eq!(saved.version, 5);
            assert_eq!(saved.patterns[0].hits, [[960, 18, 70, 0], [0, 0, 127, 240]]);
            assert_eq!(saved.patterns, [drive.app.pads.pattern.clone().unwrap()]);
            assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
            assert_eq!(drive.app.history.len(), 1);
            assert!(drive.app.transport_state().playing);
            let notice = drive
                .app
                .notices
                .history()
                .iter()
                .find(|n| n.text == "Session saved")
                .unwrap();
            assert_eq!(
                notice.origin,
                match source {
                    Source::Keyboard => Origin::Keyboard,
                    _ => Origin::Device(PAD_PORT.into()),
                }
            );
            assert_eq!(notice.level, Level::Done);
        }
    }
}

fn pattern_manual_save_settle(drive: &mut Drive) {
    for _ in 0..64 {
        app_capture_frames(drive, 0);
        drive.app.flush_audio();
        if drive.app.pending_manual_save.is_none() {
            return;
        }
    }
    panic!("manual save did not complete");
}

#[test]
fn pattern_manual_save_cancels_count_in_or_a_queued_start_and_keeps_default_a1() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for accepted in [false, true] {
            let (_files, mut drive) = pattern_autosave_drive(false);
            pattern_manual_save_source(&mut drive, source);
            drive.app.apply_session(session::Session::default());
            settle_pattern(&mut drive);
            drive.app.save_session().unwrap();
            let file = drive.app.session_file();
            let stored = std::fs::read(&file).unwrap();
            assert!(drive.app.try_send(Command::PatternRecordStart { pass: 1 }));
            if accepted {
                app_capture_frames(&mut drive, 6000);
                assert_eq!(
                    app_capture_snapshot(&drive).phase,
                    engine::capture::Phase::CountIn
                );
                drive.pad_press(source, 0);
                app_capture_frames(&mut drive, 128);
            }
            drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
            assert_eq!(pattern_manual_save_successes(&drive), 0);
            assert_eq!(std::fs::read(&file).unwrap(), stored);
            pattern_manual_save_settle(&mut drive);
            let saved = session::load_existing(&file).unwrap();
            assert_eq!(saved.version, 5);
            assert_eq!(saved.pattern.as_deref(), Some("A1"));
            assert_eq!(
                saved.patterns[0],
                session::PatternState {
                    slot: "A1".into(),
                    bars: 2,
                    quantize: "1/16".into(),
                    strength: 100,
                    hits: vec![]
                }
            );
            assert!(!drive.app.transport_state().playing);
            assert_eq!(
                app_capture_snapshot(&drive).phase,
                engine::capture::Phase::Idle
            );
            assert!(drive.app.history.is_empty());
            assert_eq!(pattern_manual_save_successes(&drive), 1);
            assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
        }
    }
}

/// Scenario "Repeated save waits for the same recording boundary".
#[test]
fn pattern_manual_save_repeats_join_and_preserve_origin_while_pads_and_navigation_continue() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let (_files, mut drive) = pattern_autosave_drive(false);
        pattern_manual_save_source(&mut drive, source);
        app_capture_sound(
            &mut drive,
            0,
            Arc::new(Sample::mono(vec![0.02; 48_000])),
            true,
        );
        app_capture_sound(
            &mut drive,
            1,
            Arc::new(Sample::mono(vec![0.03; 48_000])),
            true,
        );
        app_capture_frames(&mut drive, 0);
        drive.app.save_session().unwrap();
        let file = drive.app.session_file();
        let stored = std::fs::read(&file).unwrap();
        app_capture_start(&mut drive, 1);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 6000);
        let first = drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
        assert_eq!(
            first
                .ui
                .iter()
                .filter(|c| matches!(c, Command::PatternRecordEnd { pass: 1 }))
                .count(),
            1
        );
        let other = if matches!(source, Source::Keyboard) {
            Source::Button
        } else {
            Source::Keyboard
        };
        for _ in 0..3 {
            let repeated = drive.registry_command(other, KeyCode::Char('s'), M::CONTROL, 91);
            assert!(
                !repeated
                    .ui
                    .iter()
                    .any(|c| matches!(c, Command::PatternRecordEnd { .. }))
            );
        }
        assert!(!drive.app.try_send(Command::PatternRecordStart { pass: 2 }));
        assert_eq!(std::fs::read(&file).unwrap(), stored);
        drive.tap(KeyCode::F(4), M::NONE);
        assert_eq!(drive.app.screen, Screen::Fx);
        app_capture_frames(&mut drive, 0);
        assert_eq!(
            app_capture_snapshot(&drive).phase,
            engine::capture::Phase::Finishing
        );
        drive.pad_release(source, 0);
        let hit = drive.pad_press(source, 1);
        assert!(
            hit.ui
                .iter()
                .any(|c| matches!(c, Command::PadPress { pad: 1, .. }))
                || !matches!(source, Source::Keyboard)
        );
        let audio = app_capture_frames(&mut drive, 128);
        assert!(audio.0.iter().any(|&sample| sample > 0.025));
        assert!(audio.1.iter().any(|&sample| sample > 0.025));
        drive.pad_release(source, 1);
        pattern_manual_save_settle(&mut drive);
        assert_eq!(pattern_manual_save_successes(&drive), 1);
        assert_eq!(drive.app.history.len(), 1);
        assert_eq!(
            session::load_existing(&file).unwrap().patterns[0].hits,
            [[960, 18, 70, 0], [0, 0, 127, 240]]
        );
        let notice = drive
            .app
            .notices
            .history()
            .iter()
            .find(|n| n.text == "Session saved")
            .unwrap();
        assert_eq!(
            notice.origin,
            match source {
                Source::Keyboard => Origin::Keyboard,
                _ => Origin::Device(PAD_PORT.into()),
            }
        );
        let until = drive.app.notices.deadline().unwrap();
        assert!(
            drive
                .app
                .frame_notice_at(until - Duration::from_nanos(1))
                .is_some()
        );
        assert_eq!(drive.app.frame_notice_at(until), None);
    }
}

/// Scenario "Save completion removes hidden progress without hiding other notices".
#[test]
fn pattern_manual_save_hidden_progress_never_resumes_and_keeps_warning_error_and_history() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for level in [Level::Warn, Level::Error] {
            let (_files, mut drive) = pattern_autosave_drive(false);
            pattern_manual_save_source(&mut drive, source);
            app_capture_start(&mut drive, 1);
            drive.pad_press(Source::Notes, 0);
            app_capture_frames(&mut drive, 128);
            drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
            assert!(drive.app.pending_manual_save.is_some());
            drive.app.produce_notice(
                Notice::new(level, Origin::System, "Other notice"),
                Instant::now(),
            );
            let error = drive.app.notices.error_token();
            let warning_deadline = drive.app.notices.deadline();
            pattern_manual_save_settle(&mut drive);
            assert_eq!(pattern_manual_save_successes(&drive), 1);
            assert_eq!(drive.app.notices.current().unwrap().text, "Other notice");
            assert_eq!(drive.app.notices.deadline(), warning_deadline);
            assert!(
                drive
                    .app
                    .notices
                    .history()
                    .iter()
                    .any(|n| n.text == "Finishing A1 before saving.")
            );
            assert!(
                drive
                    .app
                    .notices
                    .history()
                    .iter()
                    .any(|n| n.text == "Other notice")
            );
            if level == Level::Error {
                drive
                    .app
                    .notices
                    .acknowledge_shown(error.as_ref(), Instant::now());
            } else {
                assert_eq!(
                    drive.app.frame_notice_at(warning_deadline.unwrap()),
                    Some("Session saved".into())
                );
            }
            assert_eq!(drive.app.notices.current().unwrap().text, "Session saved");
            let until = drive.app.notices.deadline().unwrap();
            assert_eq!(drive.app.frame_notice_at(until), None);
        }
    }
}

#[test]
fn pattern_manual_save_retains_one_end_and_complete_rows_under_command_queue_pressure() {
    let (_files, mut drive) = pattern_autosave_drive(true);
    pattern_manual_save_source(&mut drive, Source::Notes);
    app_capture_sound(
        &mut drive,
        0,
        Arc::new(Sample::mono(vec![0.02; 48_000])),
        true,
    );
    app_capture_frames(&mut drive, 0);
    drive.app.save_session().unwrap();
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 6000);
    let file = drive.app.session_file();
    let stored = std::fs::read(&file).unwrap();
    let epoch = app_capture_snapshot(&drive).epoch;
    for _ in 0..1024 {
        drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .unwrap();
    }
    let request = drive.registry_command(Source::Notes, KeyCode::Char('s'), M::CONTROL, 91);
    assert_eq!(
        request
            .ui
            .iter()
            .filter(|c| matches!(c, Command::PatternRecordEnd { .. }))
            .count(),
        1
    );
    assert!(drive.app.pending_manual_save.is_some());
    assert!(!drive.app.pending_audio.is_empty());
    for _ in 0..3 {
        let repeat = drive.registry_command(Source::Keyboard, KeyCode::Char('s'), M::CONTROL, 91);
        assert!(
            !repeat
                .ui
                .iter()
                .any(|c| matches!(c, Command::PatternRecordEnd { .. }))
        );
        drive
            .app
            .autosave(Instant::now() + SESSION_SAVE + Duration::from_secs(1));
        assert_eq!(std::fs::read(&file).unwrap(), stored);
        assert_eq!(pattern_manual_save_successes(&drive), 0);
    }
    for _ in 0..64 {
        app_capture_frames(&mut drive, 0);
        drive
            .app
            .autosave(Instant::now() + SESSION_SAVE + Duration::from_secs(1));
        if drive.app.pending_manual_save.is_none() {
            break;
        }
        assert_eq!(std::fs::read(&file).unwrap(), stored);
    }
    assert!(drive.app.pending_manual_save.is_none());
    assert_eq!(pattern_manual_save_successes(&drive), 1);
    assert_eq!(drive.app.history.len(), 1);
    assert_eq!(
        session::load_existing(&file).unwrap().patterns[0].hits[1][3],
        240
    );
    assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
}

#[test]
fn pattern_manual_save_joins_an_already_finishing_pass_without_another_end() {
    let (_files, mut drive) = pattern_autosave_drive(false);
    pattern_manual_save_source(&mut drive, Source::Button);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    drive.app.send(Command::PatternRecordEnd { pass: 1 });
    app_capture_frames(&mut drive, 0);
    drive.frame();
    let request = drive.registry_command(Source::Button, KeyCode::Char('s'), M::CONTROL, 91);
    assert!(
        !request
            .ui
            .iter()
            .any(|c| matches!(c, Command::PatternRecordEnd { .. }))
    );
    assert_eq!(pattern_manual_save_successes(&drive), 0);
    pattern_manual_save_settle(&mut drive);
    assert_eq!(pattern_manual_save_successes(&drive), 1);
    assert_eq!(drive.app.history.len(), 1);
    assert_eq!(
        session::load_existing(&drive.app.session_file())
            .unwrap()
            .patterns[0]
            .hits
            .len(),
        2
    );
}

#[test]
fn pattern_manual_save_lost_audio_recovers_before_or_after_the_request_without_ack() {
    use std::sync::atomic::Ordering::Relaxed;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for stage in [0, 1, 2] {
            let (_files, mut drive) = pattern_autosave_drive(false);
            pattern_manual_save_source(&mut drive, source);
            app_capture_sound(
                &mut drive,
                0,
                Arc::new(Sample::mono(vec![0.02; 48_000])),
                true,
            );
            app_capture_frames(&mut drive, 0);
            drive.app.save_session().unwrap();
            app_capture_start(&mut drive, 1);
            drive.pad_press(source, 0);
            app_capture_frames(&mut drive, 6000);
            if stage != 0 {
                drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
                if stage == 2 {
                    app_capture_frames(&mut drive, 0);
                }
                assert_eq!(pattern_manual_save_successes(&drive), 0);
            }
            drive.audio_rt = None;
            drive
                .app
                .audio
                .as_ref()
                .unwrap()
                .stopped
                .store(true, Relaxed);
            drive.app.sent.clear();
            if stage == 0 {
                let frame = drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
                assert!(frame.ui.is_empty());
            } else {
                drive.app.flush_audio();
            }
            assert!(drive.app.pending_manual_save.is_none());
            assert!(drive.app.sent.is_empty());
            assert_eq!(pattern_manual_save_successes(&drive), 1);
            assert_eq!(drive.app.history.len(), 1);
            assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
            assert_eq!(
                session::load_existing(&drive.app.session_file())
                    .unwrap()
                    .patterns[0]
                    .hits[1][3],
                240
            );
            assert!(
                drive
                    .app
                    .notices
                    .history()
                    .iter()
                    .any(|n| n.text.contains("Pattern recording interrupted"))
            );
        }
    }
}

/// Scenario "Save refusal retains a completed recording for retry".
#[test]
fn pattern_manual_save_refusal_retains_complete_take_undo_status_and_original_origin() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for problem in ["write", "damaged", "newer", "missing"] {
            let (files, mut drive) = pattern_autosave_drive(true);
            pattern_manual_save_source(&mut drive, source);
            app_capture_sound(
                &mut drive,
                0,
                Arc::new(Sample::mono(vec![0.02; 48_000])),
                true,
            );
            app_capture_frames(&mut drive, 0);
            drive.app.save_session().unwrap();
            let file = drive.app.session_file();
            let valid = std::fs::read(&file).unwrap();
            let tmp = file.with_extension("toml.tmp");
            if problem == "write" {
                std::fs::write(&tmp, b"occupied").unwrap();
            } else if problem == "missing" {
                std::fs::remove_dir_all(files.0.join("A")).unwrap();
            } else {
                std::fs::write(
                    &file,
                    if problem == "damaged" {
                        "version = [\n"
                    } else {
                        "version = 99\n"
                    },
                )
                .unwrap();
                drive.app.session_read_error = Some(session::load_existing(&file).unwrap_err());
            }
            app_capture_start(&mut drive, 1);
            drive.pad_press(source, 0);
            app_capture_frames(&mut drive, 6000);
            drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
            let pending = drive.app.session_dirty;
            pattern_manual_save_settle(&mut drive);
            let take = drive.app.pads.pattern.clone();
            assert_eq!(take.as_ref().unwrap().hits[1][3], 240);
            assert_eq!(pattern_manual_save_successes(&drive), 0);
            assert_eq!(drive.app.history.len(), 1);
            assert_eq!(drive.app.session_dirty, pending);
            let refusal = drive.last_notice().clone();
            assert_eq!(refusal.level, Level::Error);
            assert_eq!(
                refusal.origin,
                match source {
                    Source::Keyboard => Origin::Keyboard,
                    _ => Origin::Device(PAD_PORT.into()),
                }
            );
            drive.app.notices.acknowledge(Instant::now());
            assert!(
                drive
                    .app
                    .notices
                    .current()
                    .is_none_or(|n| n.text != "Finishing A1 before saving.")
            );
            drive
                .app
                .autosave(Instant::now() + SESSION_SAVE + Duration::from_secs(1));
            assert_eq!(drive.app.session_dirty, pending);
            assert_eq!(drive.app.pads.pattern, take);
            assert_eq!(pattern_manual_save_successes(&drive), 0);
            if problem == "missing" {
                assert!(!files.0.join("A").exists());
                drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
                assert!(!files.0.join("A").exists());
                std::fs::create_dir_all(files.0.join("A")).unwrap();
            } else if problem == "write" {
                assert_eq!(std::fs::read(&file).unwrap(), valid);
                assert_eq!(std::fs::read(&tmp).unwrap(), b"occupied");
                std::fs::remove_file(&tmp).unwrap();
            }
            std::fs::write(&file, &valid).unwrap();
            drive
                .app
                .autosave(Instant::now() + SESSION_SAVE + Duration::from_secs(1));
            assert_eq!(std::fs::read(&file).unwrap(), valid);
            drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
            if matches!(problem, "write" | "missing") {
                assert_eq!(pattern_manual_save_successes(&drive), 1);
                assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
                assert_eq!(
                    session::load_existing(&file).unwrap().patterns,
                    [take.unwrap()]
                );
            } else {
                assert_eq!(pattern_manual_save_successes(&drive), 0);
                assert_eq!(std::fs::read(&file).unwrap(), valid);
                assert_eq!(drive.app.save_status(), tui::SaveStatus::Unsaved);
            }
            assert_eq!(drive.app.history.len(), 1);
        }
    }
}

/// Scenario "A delayed manual save cannot write another project".
#[test]
fn pattern_manual_save_project_restore_discards_the_old_request_and_success() {
    let (files, mut drive) = pattern_autosave_drive(true);
    pattern_manual_save_source(&mut drive, Source::Button);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    let source = drive.app.session_file();
    let stored = std::fs::read(&source).unwrap();
    drive.registry_command(Source::Button, KeyCode::Char('s'), M::CONTROL, 91);
    assert!(drive.app.pending_manual_save.is_some());
    drive.app.produce_notice(
        Notice::new(Level::Warn, Origin::System, "Keep project warning"),
        Instant::now(),
    );
    let warning_deadline = drive.app.notices.deadline().unwrap();
    drive.app.project = Some(Project {
        name: "B".into(),
        folder: files.0.join("B"),
    });
    let target = drive.app.session_file();
    let target_before = std::fs::read(&target).unwrap();
    drive.app.apply_session(session::Session::default());
    settle_pattern(&mut drive);
    assert!(drive.app.pending_manual_save.is_none());
    assert_eq!(pattern_manual_save_successes(&drive), 0);
    assert_eq!(std::fs::read(&source).unwrap(), stored);
    assert_eq!(std::fs::read(&target).unwrap(), target_before);
    drive.tap(KeyCode::Char(']'), M::NONE);
    drive.registry_command(Source::Keyboard, KeyCode::Char('s'), M::CONTROL, 91);
    assert_eq!(pattern_manual_save_successes(&drive), 1);
    let saved = session::load_existing(&target).unwrap();
    assert!(saved.patterns.is_empty());
    assert_eq!(saved.master_db, drive.app.master_db);
    assert!(drive.app.history.is_empty());
    assert_eq!(
        drive.app.frame_notice_at(warning_deadline),
        Some("Session saved".into())
    );
    let until = drive.app.notices.deadline().unwrap();
    assert_eq!(drive.app.frame_notice_at(until), None);
}

/// Scenario "Save and reopen a recorded rhythm without recording state".
#[test]
fn pattern_manual_save_reopens_raw_rows_length_grid_and_strength_without_runtime_state() {
    for project in [false, true] {
        let (_files, mut drive) = pattern_autosave_drive(project);
        pattern_manual_save_source(&mut drive, Source::Notes);
        let pattern = drive.app.pads.pattern.as_mut().unwrap();
        pattern.bars = 2;
        pattern.quantize = "1/8".into();
        pattern.strength = 50;
        app_capture_start(&mut drive, 1);
        drive.pad_press(Source::Notes, 0);
        app_capture_frames(&mut drive, 6000);
        drive.registry_command(Source::Notes, KeyCode::Char('s'), M::CONTROL, 91);
        pattern_manual_save_settle(&mut drive);
        let expected = drive.app.pads.pattern.clone().unwrap();
        let saved = session::load_existing(&drive.app.session_file()).unwrap();
        assert_eq!(saved.version, 5);
        assert_eq!(saved.patterns.as_slice(), std::slice::from_ref(&expected));
        assert_eq!(
            (expected.bars, expected.quantize.as_str(), expected.strength),
            (2, "1/8", 50)
        );
        let old_epoch = app_capture_snapshot(&drive).epoch;
        drive.app.apply_session(saved);
        settle_pattern(&mut drive);
        assert_eq!(drive.app.pads.pattern.as_ref(), Some(&expected));
        assert!(!drive.app.transport_state().playing);
        let state = app_capture_snapshot(&drive);
        assert!(state.epoch > old_epoch);
        assert_eq!(state.phase, engine::capture::Phase::Idle);
        assert_eq!(state.pass, 0);
        assert!(drive.app.pads.pattern_rows.is_none());
        assert!(drive.app.pending_manual_save.is_none());
        assert!(drive.app.history.is_empty());
    }
}

fn pattern_lifecycle_quit_key(drive: &mut Drive) -> bool {
    drive.app.keyboard_event(
        &drive.keymap,
        &KeyEvent::new_with_kind(KeyCode::Char('q'), M::CONTROL, KeyEventKind::Press),
        true,
        &mut drive.down,
        &mut drive.held,
        &mut drive.mods,
    )
}

/// Scenario "Quit finalizes the recoverable pass before exit saving".
#[test]
fn pattern_lifecycle_quit_completes_gate_without_a_final_callback() {
    use std::sync::atomic::Ordering::Relaxed;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for project in [false, true] {
            for stage in ["recording", "finishing", "lost"] {
                let (_files, mut drive) = pattern_autosave_drive(project);
                app_capture_sound(
                    &mut drive,
                    0,
                    Arc::new(Sample::mono(vec![0.02; 48_000])),
                    true,
                );
                app_capture_frames(&mut drive, 0);
                drive.app.save_session().unwrap();
                app_capture_start(&mut drive, 1);
                drive.pad_press(source, 0);
                app_capture_frames(&mut drive, 6000);
                if stage == "finishing" {
                    drive.app.send(Command::PatternRecordEnd { pass: 1 });
                    app_capture_frames(&mut drive, 0);
                }
                if stage == "lost" {
                    drive.audio_rt = None;
                    drive
                        .app
                        .audio
                        .as_ref()
                        .unwrap()
                        .stopped
                        .store(true, Relaxed);
                }
                let boundary = app_capture_snapshot(&drive);
                drive.app.sent.clear();
                assert!(pattern_lifecycle_quit_key(&mut drive));
                assert!(drive.app.quit_ready());
                assert_eq!(app_capture_snapshot(&drive), boundary);
                assert_eq!(
                    drive.app.audio.as_ref().unwrap().stopped.load(Relaxed),
                    stage == "lost"
                );
                assert!(
                    drive.app.sent.is_empty(),
                    "quit cannot require publication or acknowledgement"
                );
                assert_eq!(drive.app.history.len(), 1);
                drive.app.save_on_exit();
                let saved = session::load_existing(&drive.app.session_file()).unwrap();
                assert_eq!(saved.version, 5);
                assert_eq!(saved.patterns[0].hits, [[960, 18, 70, 0], [0, 0, 127, 240]]);
                assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
                assert_eq!(pattern_manual_save_successes(&drive), 0);
                assert_eq!(
                    drive
                        .app
                        .notices
                        .history()
                        .iter()
                        .any(|n| n.text.contains("Pattern recording interrupted")),
                    stage == "lost"
                );
                let take = drive.app.pads.pattern.clone();
                if stage == "recording" {
                    drive.pad_press(Source::Notes, 1);
                    app_capture_frames(&mut drive, 6000);
                    drive.app.flush_audio();
                    assert_eq!(
                        drive.app.pads.pattern, take,
                        "later callback cannot extend a closed prefix"
                    );
                    assert_eq!(drive.app.history.len(), 1);
                    assert!(!drive.app.try_send(Command::PatternRecordStart { pass: 2 }));
                }
            }
        }
    }
}

#[test]
fn pattern_lifecycle_quit_retries_concurrent_gate_completion_without_extending_a_known_gate() {
    for _ in 0..24 {
        let (_files, mut drive) = pattern_autosave_drive(false);
        app_capture_sound(
            &mut drive,
            0,
            Arc::new(Sample::mono(vec![0.02; 48_000])),
            true,
        );
        app_capture_frames(&mut drive, 0);
        app_capture_start(&mut drive, 1);
        drive.pad_press(Source::Notes, 0);
        app_capture_frames(&mut drive, 6000);
        drive.pad_release(Source::Notes, 0);
        let mut rt = drive.audio_rt.take().unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let thread_barrier = barrier.clone();
        let worker = std::thread::spawn(move || {
            thread_barrier.wait();
            for _ in 0..16 {
                rt.process(&mut [0.0; 128], &mut [0.0; 128]);
            }
        });
        barrier.wait();
        let _ = pattern_lifecycle_quit_key(&mut drive);
        worker.join().unwrap();
        // Retry through the same UI collection path, without another audio call.
        drive.app.flush_audio();
        assert!(drive.app.quit_ready());
        drive.app.save_on_exit();
        assert_eq!(
            session::load_existing(&drive.app.session_file())
                .unwrap()
                .patterns[0]
                .hits[1][3],
            240
        );
        assert_eq!(drive.app.history.len(), 1);
    }
}

#[test]
fn pattern_lifecycle_restore_cancels_closed_capture_before_a_new_project_epoch() {
    let (_files, mut drive) = pattern_autosave_drive(false);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    let epoch = app_capture_snapshot(&drive).epoch;
    assert!(pattern_lifecycle_quit_key(&mut drive));
    drive.app.apply_session(session::Session::default());
    assert!(!drive.app.quit_requested);
    assert!(!drive.app.quit_ready());
    settle_pattern(&mut drive);
    assert!(app_capture_snapshot(&drive).epoch > epoch);
    assert!(drive.app.try_send(Command::PatternRecordStart { pass: 2 }));
    app_capture_frames(&mut drive, 128);
    assert_eq!(
        app_capture_snapshot(&drive).phase,
        engine::capture::Phase::CountIn
    );
    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
    assert!(drive.app.history.is_empty());
}

#[test]
fn pattern_lifecycle_quit_keeps_default_a1_from_count_in_or_unaccepted_start() {
    for project in [false, true] {
        for accepted in [false, true] {
            let (_files, mut drive) = pattern_autosave_drive(project);
            drive.app.apply_session(session::Session::default());
            settle_pattern(&mut drive);
            drive.app.save_session().unwrap();
            assert!(drive.app.try_send(Command::PatternRecordStart { pass: 1 }));
            if accepted {
                app_capture_frames(&mut drive, 6000);
                assert_eq!(
                    app_capture_snapshot(&drive).phase,
                    engine::capture::Phase::CountIn
                );
                drive.pad_press(Source::Notes, 0);
                app_capture_frames(&mut drive, 128);
            }
            let boundary = app_capture_snapshot(&drive);
            assert!(pattern_lifecycle_quit_key(&mut drive));
            drive.app.save_on_exit();
            assert_eq!(app_capture_snapshot(&drive), boundary);
            let saved = session::load_existing(&drive.app.session_file()).unwrap();
            assert_eq!(saved.pattern.as_deref(), Some("A1"));
            assert_eq!(
                saved.patterns[0],
                session::PatternState {
                    slot: "A1".into(),
                    bars: 2,
                    quantize: "1/16".into(),
                    strength: 100,
                    hits: vec![]
                }
            );
            assert!(drive.app.history.is_empty());
            assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
        }
    }
}

#[test]
fn pattern_lifecycle_quit_does_not_wait_for_full_command_or_feedback_queues() {
    for full in ["commands", "feedback"] {
        let (_files, mut drive) = pattern_autosave_drive(false);
        app_capture_start(&mut drive, 1);
        if full == "feedback" {
            for _ in 0..640 {
                drive.pad_press(Source::Notes, 0);
                drive.pad_release(Source::Notes, 0);
                app_capture_frames(&mut drive, 0);
            }
            assert!(app_capture_snapshot(&drive).dropped_onsets > 0);
        } else {
            drive.pad_press(Source::Notes, 0);
            app_capture_frames(&mut drive, 6000);
            let epoch = app_capture_snapshot(&drive).epoch;
            for _ in 0..1024 {
                drive
                    .app
                    .audio
                    .as_mut()
                    .unwrap()
                    .commands
                    .push(ProjectCommand {
                        epoch,
                        ready_at_send: true,
                        command: ProjectMessage::Command(Command::SetPadGain {
                            pad: 127,
                            gain: 1.0,
                        }),
                    })
                    .unwrap();
            }
        }
        let boundary = app_capture_snapshot(&drive);
        assert!(pattern_lifecycle_quit_key(&mut drive));
        drive.app.save_on_exit();
        assert_eq!(app_capture_snapshot(&drive), boundary);
        assert_eq!(drive.app.history.len(), 1);
        let saved = session::load_existing(&drive.app.session_file()).unwrap();
        assert_eq!(
            saved.patterns,
            drive
                .app
                .pads
                .pattern
                .clone()
                .into_iter()
                .collect::<Vec<_>>()
        );
        // an interrupted pass ends too: the flood on one step keeps its newest row
        assert_eq!(saved.patterns[0].hits.len(), 2);
        assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
    }
}

#[test]
fn pattern_lifecycle_quit_finishes_one_prior_manual_save_with_its_original_source() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for failed in [false, true] {
            let (_files, mut drive) = pattern_autosave_drive(false);
            pattern_manual_save_source(&mut drive, source);
            app_capture_start(&mut drive, 1);
            drive.pad_press(source, 0);
            app_capture_frames(&mut drive, 6000);
            let file = drive.app.session_file();
            let original = std::fs::read(&file).unwrap();
            let tmp = file.with_extension("toml.tmp");
            if failed {
                std::fs::write(&tmp, b"occupied").unwrap();
            }
            drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 91);
            assert!(drive.app.pending_manual_save.is_some());
            assert!(pattern_lifecycle_quit_key(&mut drive));
            assert!(drive.app.pending_manual_save.is_none());
            assert_eq!(
                drive.last_notice().origin,
                if matches!(source, Source::Keyboard) {
                    Origin::Keyboard
                } else {
                    Origin::Device(PAD_PORT.into())
                }
            );
            assert_eq!(pattern_manual_save_successes(&drive), usize::from(!failed));
            if failed {
                std::fs::remove_file(&tmp).unwrap();
            }
            let history = drive.app.notices.history().clone();
            drive.app.save_on_exit();
            assert_eq!(drive.app.notices.history(), &history);
            assert_eq!(drive.app.history.len(), 1);
            if failed {
                assert_eq!(std::fs::read(&file).unwrap(), original);
                assert_eq!(drive.app.save_status(), tui::SaveStatus::Failed);
            } else {
                assert_eq!(
                    session::load_existing(&file).unwrap().patterns[0]
                        .hits
                        .len(),
                    2
                );
                assert_eq!(drive.app.save_status(), tui::SaveStatus::Clean);
            }
        }
    }
}

#[test]
fn pattern_lifecycle_quit_keeps_existing_file_protection_and_no_retry_policy() {
    for problem in ["prior-write", "write", "damaged", "newer", "missing"] {
        let (files, mut drive) = pattern_autosave_drive(true);
        let file = drive.app.session_file();
        let valid = std::fs::read(&file).unwrap();
        let tmp = file.with_extension("toml.tmp");
        if matches!(problem, "prior-write" | "write") {
            std::fs::write(&tmp, b"occupied").unwrap();
        }
        if problem == "prior-write" {
            drive.app.dirty();
            assert!(drive.app.save_session().is_err());
            std::fs::remove_file(&tmp).unwrap();
        } else if problem == "missing" {
            std::fs::remove_dir_all(files.0.join("A")).unwrap();
        } else if matches!(problem, "damaged" | "newer") {
            std::fs::write(
                &file,
                if problem == "damaged" {
                    "version = [\n"
                } else {
                    "version = 99\n"
                },
            )
            .unwrap();
            drive.app.session_read_error = Some(session::load_existing(&file).unwrap_err());
        }
        app_capture_start(&mut drive, 1);
        drive.pad_press(Source::Notes, 0);
        app_capture_frames(&mut drive, 6000);
        assert!(pattern_lifecycle_quit_key(&mut drive));
        drive.app.save_on_exit();
        assert_eq!(drive.app.history.len(), 1);
        assert!(drive.app.session_dirty.is_some());
        assert_eq!(pattern_manual_save_successes(&drive), 0);
        if problem == "missing" {
            assert!(!files.0.join("A").exists());
        } else if matches!(problem, "damaged" | "newer") {
            assert!(drive.app.session_read_error.is_some());
        } else {
            assert_eq!(std::fs::read(&file).unwrap(), valid);
        }
        if problem == "write" {
            assert_eq!(std::fs::read(&tmp).unwrap(), b"occupied");
            std::fs::remove_file(&tmp).unwrap();
            drive.app.save_on_exit();
            assert_eq!(std::fs::read(&file).unwrap(), valid);
        }
    }
}

/// Scenario "A project request cannot interrupt an active take".
#[test]
fn pattern_lifecycle_project_requests_preserve_take_clock_and_files_in_all_phases() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for phase in ["requested", "count-in", "recording", "finishing"] {
            let (files, mut drive) = pattern_autosave_drive(true);
            learn::assign(
                &mut drive.app.learned,
                "Recorded pads",
                input::Group::Global,
                if matches!(source, Source::Notes) {
                    input::midi::Key::Note(94)
                } else {
                    input::midi::Key::Cc(94)
                },
                "project.pick",
            );
            drive
                .app
                .sync_learned("Recorded pads", input::Group::Global);
            drive.midi().recorded_fence();
            if matches!(phase, "recording" | "finishing") {
                app_capture_start(&mut drive, 1);
            } else {
                assert!(drive.app.try_send(Command::PatternRecordStart { pass: 1 }));
                if phase == "count-in" {
                    app_capture_frames(&mut drive, 6000);
                }
            }
            if phase == "finishing" {
                drive.app.send(Command::PatternRecordEnd { pass: 1 });
                app_capture_frames(&mut drive, 0);
            }
            drive.app.flush_audio();
            let take = drive.app.pads.pattern.clone();
            let clock = app_capture_snapshot(&drive);
            let source_file = drive.app.session_file();
            let original = std::fs::read(&source_file).unwrap();
            let target = files.0.join("B/project.toml");
            let before = std::fs::read(&target).unwrap();
            let screen = drive.app.screen;
            drive.registry_command(source, KeyCode::Char('p'), M::CONTROL, 94);
            assert!(drive.app.project_picker.is_none());
            assert_eq!(drive.app.screen, screen);
            assert_eq!(drive.last_notice().level, Level::Warn);
            assert!(drive.last_notice().text.contains(if phase == "requested" {
                "start pending"
            } else {
                phase
            }));
            assert!(!drive.app.open_project(files.0.join("B")));
            assert!(!drive.app.open_project(files.0.join("A")));
            assert!(!drive.app.open_project(files.0.join("absent")));
            drive.app.paths.projects = files.0.clone();
            assert!(!drive.app.create_project("must-not-exist".into()));
            assert!(!files.0.join("must-not-exist").exists());
            assert_eq!(std::fs::read(&source_file).unwrap(), original);
            assert_eq!(std::fs::read(&target).unwrap(), before);
            assert_eq!(drive.app.pads.pattern, take);
            assert_eq!(app_capture_snapshot(&drive), clock);
        }
    }
}

#[test]
fn pattern_lifecycle_idle_picker_retains_same_folder_and_target_validation() {
    let (files, mut drive) = pattern_autosave_drive(true);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    app_capture_finish(&mut drive, 1);
    let file = drive.app.session_file();
    let before = std::fs::read(&file).unwrap();
    let take = drive.app.pads.pattern.clone();
    assert!(drive.app.open_project(files.0.join("A")));
    assert_eq!(std::fs::read(&file).unwrap(), before);
    assert!(!drive.app.open_project(files.0.join("absent")));
    assert_eq!(std::fs::read(&file).unwrap(), before);
    assert_eq!(drive.app.pads.pattern, take);
    drive.tap(KeyCode::Char('p'), M::CONTROL);
    assert!(drive.app.project_picker.is_some());
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.open_project(files.0.join("B")));
    assert_eq!(
        session::load_existing(&file).unwrap().patterns,
        take.into_iter().collect::<Vec<_>>()
    );
    assert!(drive.app.pads.pattern.is_none());
    let audio = drive.app.audio.as_ref().unwrap();
    let epoch = audio
        .project
        .epoch
        .load(std::sync::atomic::Ordering::Acquire);
    assert!(
        audio.stats.recording(epoch).is_none(),
        "old snapshot is not current-project feedback"
    );
    settle_pattern(&mut drive);
    assert_eq!(
        app_capture_snapshot(&drive).phase,
        engine::capture::Phase::Idle
    );
}

fn pattern_undo_routes(drive: &mut Drive, source: Source) {
    swap_routes(drive, source);
    learn::assign(
        &mut drive.app.learned,
        "Recorded pads",
        input::Group::Global,
        if matches!(source, Source::Notes) {
            input::midi::Key::Note(109)
        } else {
            input::midi::Key::Cc(109)
        },
        "pattern.undo",
    );
    drive
        .app
        .sync_learned("Recorded pads", input::Group::Global);
    drive.frame();
}

fn global_pattern_undo(drive: &mut Drive, source: Source) -> Frame {
    drive.registry_command(source, KeyCode::Char('z'), M::ALT, 109)
}

/// Scenario "Undo a pass from another screen without crossing a swap".
#[test]
fn global_pattern_undo_works_on_every_screen_and_preserves_local_chop_edits() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for (screen, key) in [
            (Screen::Pads, 2),
            (Screen::Files, 6),
            (Screen::Chop, 3),
            (Screen::Fx, 4),
            (Screen::Settings, 12),
        ] {
            let mut drive = app_capture_drive(vec![[960, 18, 70, 0]]);
            pattern_undo_routes(&mut drive, source);
            let original = drive.app.pads.pattern.clone();
            app_capture_start(&mut drive, 1);
            drive.pad_press(Source::Notes, 0);
            app_capture_frames(&mut drive, 128);
            drive.pad_release(Source::Notes, 0);
            app_capture_finish(&mut drive, 1);
            drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48_000])), 2);
            drive.app.chop.as_mut().unwrap().cut(12_000);
            if screen != Screen::Chop {
                drive.tap(KeyCode::F(key), M::NONE);
            }
            assert_eq!(drive.app.screen, screen);
            assert_eq!(drive.app.history.len(), 1);
            assert!(drive.app.history[0].pattern.is_some());
            drive.app.pads.bank = 7;
            drive.app.pads.selected = 3;
            let cuts = drive.app.chop.as_ref().unwrap().saved();
            let before = drive.app.before();
            global_pattern_undo(&mut drive, source);
            app_capture_frames(&mut drive, 0);
            assert_eq!(
                drive.app.pads.pattern,
                original,
                "{source:?}/{screen:?}: history={:?}, notice={:?}",
                drive
                    .app
                    .history
                    .iter()
                    .map(|step| (&step.what, step.pattern.is_some()))
                    .collect::<Vec<_>>(),
                drive
                    .app
                    .notices
                    .history()
                    .front()
                    .map(|notice| &notice.text)
            );
            assert_eq!(drive.app.screen, screen);
            assert_eq!((drive.app.pads.bank, drive.app.pads.selected), (7, 3));
            assert_eq!(drive.app.chop.as_ref().unwrap().saved(), cuts);
            assert_eq!(drive.app.pads.fx, before.fx);
            for (pad, snapshot) in before.pads.iter().enumerate() {
                assert_eq!(&drive.app.pads.snapshot(pad), snapshot);
            }
            assert!(drive.app.history.is_empty());
            if screen == Screen::Chop {
                drive.tap(KeyCode::Char('z'), M::CONTROL);
                assert_ne!(drive.app.chop.as_ref().unwrap().saved(), cuts);
                assert_eq!(drive.app.pads.pattern, original);
                assert!(drive.app.history.is_empty());
            }
        }
    }
}

#[test]
fn global_pattern_undo_refuses_a_newer_swap_until_normal_undo_restores_addresses() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![[960, 0, 70, 0]]);
        pattern_undo_routes(&mut drive, source);
        let original = drive.app.pads.pattern.clone();
        app_capture_start(&mut drive, 1);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 128);
        drive.pad_release(source, 0);
        app_capture_finish(&mut drive, 1);
        drive.registry_command(source, KeyCode::Char('x'), M::CONTROL, 110);
        drive.pad_press(source, 15);
        drive.pad_release(source, 15);
        assert_eq!(drive.app.history.len(), 2);
        let swapped = drive.app.pads.pattern.clone();
        global_pattern_undo(&mut drive, source);
        assert_eq!(drive.app.pads.pattern, swapped);
        assert_eq!(drive.app.history.len(), 2);
        assert!(
            drive
                .app
                .notices
                .history()
                .front()
                .unwrap()
                .text
                .contains("normal Undo first")
        );
        completed_pass_undo(&mut drive, source);
        assert_eq!(drive.app.history.len(), 1);
        global_pattern_undo(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        assert_eq!(drive.app.pads.pattern, original);
        assert!(drive.app.history.is_empty());
    }
}

#[test]
fn global_pattern_undo_finishes_pending_gates_and_never_undoes_an_older_empty_pass() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for kind in [
            "recording",
            "finishing",
            "loss",
            "empty",
            "count-in",
            "start-pending",
            "chop",
            "settings",
            "files",
            "fx",
            "erase",
        ] {
            let mut drive = app_capture_drive(vec![]);
            pattern_undo_routes(&mut drive, source);
            app_capture_start(&mut drive, 1);
            drive.pad_press(source, 1);
            app_capture_frames(&mut drive, 128);
            drive.pad_release(source, 1);
            app_capture_finish(&mut drive, 1);
            let original = drive.app.pads.pattern.clone();
            assert_eq!(drive.app.history.len(), 1);
            if matches!(kind, "count-in" | "start-pending") {
                drive.app.send(Command::TransportStop);
                app_capture_frames(&mut drive, 0);
                assert!(drive.app.try_send(Command::PatternRecordStart { pass: 2 }));
                if kind == "count-in" {
                    app_capture_frames(&mut drive, 128);
                    assert_eq!(
                        app_capture_snapshot(&drive).phase,
                        engine::capture::Phase::CountIn
                    );
                }
            } else {
                app_capture_sound(
                    &mut drive,
                    0,
                    Arc::new(Sample::mono(vec![0.02; 48_000])),
                    true,
                );
                app_capture_start(&mut drive, 2);
                if kind != "empty" {
                    drive.pad_press(source, 0);
                    app_capture_frames(&mut drive, 6000);
                }
                if kind == "erase" {
                    drive.app.send(Command::PatternErase {
                        pass: 2,
                        enabled: true,
                    });
                    app_capture_frames(&mut drive, 0);
                    drive.pad_press(source, 1);
                    app_capture_frames(&mut drive, 96_128);
                    assert!(app_capture_snapshot(&drive).erased_rows > 0);
                } else if kind == "chop" {
                    drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48_000])), 2);
                } else if let Some(key) = match kind {
                    "settings" => Some(12),
                    "files" => Some(6),
                    "fx" => Some(4),
                    _ => None,
                } {
                    drive.tap(KeyCode::F(key), M::NONE);
                }
                if kind == "finishing" {
                    drive.app.send(Command::PatternRecordEnd { pass: 2 });
                    app_capture_frames(&mut drive, 0);
                } else if kind == "loss" {
                    drive.audio_rt = None;
                    drive
                        .app
                        .audio
                        .as_ref()
                        .unwrap()
                        .stopped
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                }
            }
            let request = global_pattern_undo(&mut drive, source);
            if kind == "loss" {
                assert!(
                    !request.commands().iter().any(|command| matches!(
                        command,
                        Command::PatternRecordEnd { .. }
                            | Command::PatternRecordAck { .. }
                            | Command::PublishRecordedPattern { .. }
                            | Command::PublishCapturedPattern { .. }
                    )),
                    "a stopped backend must not receive completion commands"
                );
            }
            for _ in 0..8 {
                drive.app.flush_audio();
                if drive.audio_rt.is_some() {
                    app_capture_frames(&mut drive, 0);
                }
            }
            assert_eq!(drive.app.pads.pattern, original, "{source:?}/{kind}");
            assert_eq!(drive.app.history.len(), 1, "{source:?}/{kind}");
            assert!(drive.app.history[0].pattern.is_some());
            assert!(
                !drive
                    .app
                    .audio
                    .as_ref()
                    .unwrap()
                    .pattern_capture
                    .blocks_autosave()
            );
            if kind != "loss" {
                assert_eq!(
                    app_capture_snapshot(&drive).phase,
                    engine::capture::Phase::Idle
                );
                drive.tap(KeyCode::F(2), M::NONE);
                app_capture_start(&mut drive, 3);
                drive.pad_press(source, 2);
                app_capture_frames(&mut drive, 128);
                drive.pad_release(source, 2);
                app_capture_finish(&mut drive, 3);
                assert_eq!(
                    drive.app.history.len(),
                    2,
                    "the next pass must not inherit Undo"
                );
                assert_eq!(
                    drive.app.pads.pattern.as_ref().unwrap().hits.len(),
                    original.as_ref().unwrap().hits.len() + 1
                );
            }
        }
    }
}

#[test]
fn global_pattern_undo_preserves_publication_order_under_full_command_queue() {
    let mut drive = app_capture_drive(vec![[960, 18, 70, 0]]);
    pattern_undo_routes(&mut drive, Source::Keyboard);
    let original = drive.app.pads.pattern.clone();
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    drive.app.flush_audio();
    app_capture_frames(&mut drive, 0);
    let epoch = app_capture_snapshot(&drive).epoch;
    for _ in 0..1024 {
        drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .unwrap();
    }
    drive.app.sent.clear();
    let request = global_pattern_undo(&mut drive, Source::Keyboard);
    assert!(!drive.app.pending_audio.is_empty());
    for _ in 0..512 {
        app_capture_frames(&mut drive, 128);
        drive.app.flush_audio();
        if drive.app.pending_audio.is_empty()
            && !drive
                .app
                .audio
                .as_ref()
                .unwrap()
                .pattern_capture
                .blocks_autosave()
        {
            break;
        }
    }
    app_capture_frames(&mut drive, 0);
    assert_eq!(drive.app.pads.pattern, original);
    assert!(drive.app.history.is_empty());
    assert!(drive.app.pending_audio.is_empty());
    assert_eq!(
        app_capture_snapshot(&drive).phase,
        engine::capture::Phase::Idle
    );
    let order: Vec<_> = request
        .ui
        .iter()
        .chain(&drive.app.sent)
        .filter_map(|command| match command {
            Command::PatternRecordEnd { .. } => Some("end"),
            Command::PatternRecordAck { .. } => Some("ack"),
            Command::PublishRecordedPattern { .. } => Some("publish"),
            _ => None,
        })
        .collect();
    assert_eq!(order, ["end", "ack", "publish"]);
}

#[test]
fn global_pattern_undo_respects_text_and_project_restoration() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for text in [false, true] {
            let mut drive = app_capture_drive(vec![]);
            pattern_undo_routes(&mut drive, source);
            app_capture_start(&mut drive, 1);
            drive.pad_press(source, 0);
            app_capture_frames(&mut drive, 128);
            drive.pad_release(source, 0);
            app_capture_finish(&mut drive, 1);
            if text {
                drive.app.pad_naming = Some((0, "name".into()));
            } else {
                drive.app.restoring_project = true;
            }
            let recorded = drive.app.pads.pattern.clone();
            global_pattern_undo(&mut drive, source);
            assert_eq!(drive.app.pads.pattern, recorded);
            assert_eq!(drive.app.history.len(), 1);
        }
    }
}

#[test]
fn global_pattern_undo_pads_ctrl_z_cancels_only_the_active_pass() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![[960, 18, 70, 0]]);
        pattern_undo_routes(&mut drive, source);
        let original = drive.app.pads.pattern.clone();
        app_capture_start(&mut drive, 1);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 128);
        drive.pad_release(source, 0);
        completed_pass_undo(&mut drive, source);
        for _ in 0..4 {
            drive.app.flush_audio();
            app_capture_frames(&mut drive, 0);
        }
        assert_eq!(drive.app.pads.pattern, original);
        assert!(drive.app.history.is_empty());
        assert_eq!(
            app_capture_snapshot(&drive).phase,
            engine::capture::Phase::Idle
        );
    }
}

fn completed_pass_undo(drive: &mut Drive, source: Source) {
    drive.registry_command(source, KeyCode::Char('z'), M::CONTROL, 111);
    app_capture_frames(drive, 0);
}

#[test]
fn completed_pass_undo_retains_default_a1_after_cancel_and_first_pass() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        swap_routes(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        assert!(drive.app.try_send(Command::PatternRecordStart { pass: 1 }));
        assert!(!drive.app.try_send(Command::PatternRecordStart { pass: 2 }));
        app_capture_frames(&mut drive, 128);
        drive.app.send(Command::PatternRecordEnd { pass: 1 });
        app_capture_frames(&mut drive, 0);
        drive.app.flush_audio();
        assert!(drive.app.history.is_empty());
        let default = drive.app.pads.pattern.clone().unwrap();
        assert!(default.hits.is_empty());
        app_capture_start(&mut drive, 2);
        drive.pad_press(source, 0);
        app_capture_frames(&mut drive, 128);
        drive.pad_release(source, 0);
        app_capture_finish(&mut drive, 2);
        assert_eq!(drive.app.history.len(), 1);
        completed_pass_undo(&mut drive, source);
        assert_eq!(drive.app.pads.pattern, Some(default));
        assert_eq!(drive.app.pads.pattern_rows, Some(vec![]));
    }
}

#[test]
fn completed_pass_undo_ignores_empty_reverted_and_refused_passes() {
    for kind in ["empty", "reverted", "full", "settings"] {
        let old = if kind == "full" {
            vec![[5000, 127, 1, 0]; 4096]
        } else {
            vec![[960, 18, 70, 0], [5000, 127, 1, 0]]
        };
        let mut drive = app_capture_drive(old);
        let before = drive.app.pads.pattern.clone();
        app_capture_start(&mut drive, 1);
        if kind == "full" {
            drive.pad_press(Source::Notes, 0);
            app_capture_frames(&mut drive, 128);
            drive.pad_release(Source::Notes, 0);
            assert_eq!(app_capture_snapshot(&drive).limit_refusals, 1);
        } else if kind != "empty" {
            let pattern = drive.app.pads.pattern.as_mut().unwrap();
            pattern.quantize = "1/8".into();
            pattern.strength = 50;
            drive
                .app
                .send(drive.app.pads.recorded_pattern_command().unwrap());
            app_capture_frames(&mut drive, 0);
            if kind == "reverted" {
                drive.app.pads.pattern = before.clone();
                drive
                    .app
                    .send(drive.app.pads.recorded_pattern_command().unwrap());
                app_capture_frames(&mut drive, 0);
            }
        }
        app_capture_finish(&mut drive, 1);
        assert_eq!(
            drive.app.history.len(),
            usize::from(kind == "settings"),
            "{kind}"
        );
        if kind == "settings" {
            drive.tap(KeyCode::Char('z'), M::CONTROL);
            app_capture_frames(&mut drive, 0);
        }
        assert_eq!(drive.app.pads.pattern, before, "{kind}");
    }
}

#[test]
fn completed_pass_undo_preserves_pad_fx_chop_and_selection_edits() {
    let mut drive = app_capture_drive(vec![[960, 18, 70, 0]]);
    drive.prepare_chop(Arc::new(Sample::mono(vec![0.02; 48_000])), 2);
    drive.tap(KeyCode::F(2), M::NONE);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Keyboard, 0);
    app_capture_frames(&mut drive, 128);
    drive.pad_release(Source::Keyboard, 0);
    drive.app.edit = true;
    drive.app.pads.param = pads::Param::Volume;
    drive.tap(KeyCode::Right, M::NONE);
    drive.tap(KeyCode::F(4), M::NONE);
    drive.app.fx_param = 1;
    drive.tap(KeyCode::Right, M::NONE);
    drive.app.chop.as_mut().unwrap().cut(12_000);
    app_capture_finish(&mut drive, 1);
    assert_eq!(drive.app.history.len(), 3);
    drive.app.pads.bank = 7;
    drive.app.pads.selected = 3;
    let before = drive.app.before();
    let chop = drive.app.chop.as_ref().unwrap().saved();
    let transport = drive.app.transport_state();
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    app_capture_frames(&mut drive, 0);
    assert_eq!(drive.app.history.len(), 2);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        [[960, 18, 70, 0]]
    );
    for (pad, snapshot) in before.pads.iter().enumerate() {
        assert_eq!(&drive.app.pads.snapshot(pad), snapshot);
    }
    assert_eq!(drive.app.pads.fx, before.fx);
    assert_eq!(drive.app.chop.as_ref().unwrap().saved(), chop);
    assert_eq!((drive.app.pads.bank, drive.app.pads.selected), (7, 3));
    assert_eq!(drive.app.screen, Screen::Fx);
    assert_eq!(drive.app.transport_state(), transport);
}

#[test]
fn completed_pass_undo_orders_delayed_completion_before_later_edits_and_swaps() {
    for swap in [false, true] {
        let original = vec![[960, 18, 70, 0]];
        let mut drive = app_capture_drive(original.clone());
        app_capture_start(&mut drive, 1);
        drive.pad_press(Source::Keyboard, 0);
        app_capture_frames(&mut drive, 128);
        drive.pad_release(Source::Keyboard, 0);
        drive.app.send(Command::PatternRecordEnd { pass: 1 });
        app_capture_frames(&mut drive, 0);
        assert!(drive.app.history.is_empty());
        if swap {
            // Exercise the production transaction before any later frame polls feedback.
            drive.app.pad_in_hand = Some(0);
            drive.app.swap_with(15);
        } else {
            drive.app.edit = true;
            drive.app.pads.param = pads::Param::Volume;
            drive.app.apply(input::Action::EditValue {
                up: true,
                fine: false,
            });
        }
        assert_eq!(drive.app.history.len(), 2);
        assert!(drive.app.history[0].pattern.is_some());
        assert!(drive.app.history[1].pattern.is_none());
        let captured = drive.app.pads.pattern.as_ref().unwrap().hits.clone();
        assert_eq!(captured.len(), 2);
        assert_eq!(captured[1][1], if swap { 15 } else { 0 });
        drive.tap(KeyCode::Char('z'), M::CONTROL);
        app_capture_frames(&mut drive, 0);
        let rows = &drive.app.pads.pattern.as_ref().unwrap().hits;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1][1], 0);
        if !swap {
            assert_eq!(drive.app.pads.volume_db[0], 0);
        }
        drive.tap(KeyCode::Char('z'), M::CONTROL);
        app_capture_frames(&mut drive, 0);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, original);
    }
}

#[test]
fn completed_pass_undo_drains_delayed_finish_before_undo_and_preserves_ack_order() {
    let mut drive = app_capture_drive(vec![]);
    app_capture_sound(
        &mut drive,
        0,
        Arc::new(Sample::mono(vec![0.02; 48_000])),
        true,
    );
    app_capture_frames(&mut drive, 0);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    drive.app.send(Command::PatternRecordEnd { pass: 1 });
    app_capture_frames(&mut drive, 0);
    assert!(drive.app.history.is_empty());
    drive.app.sent.clear();
    drive.app.undo_pads();
    app_capture_frames(&mut drive, 0);
    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
    assert!(drive.app.history.is_empty());
    assert_eq!(
        app_capture_snapshot(&drive).phase,
        engine::capture::Phase::Idle
    );
    let relevant: Vec<_> = drive
        .app
        .sent
        .iter()
        .filter_map(|command| match command {
            Command::PublishRecordedPattern { .. } => Some("publish"),
            Command::PatternRecordAck { pass: 1, .. } => Some("ack"),
            _ => None,
        })
        .collect();
    assert_eq!(relevant, ["publish", "ack", "publish"]);
    for _ in 0..10 {
        drive.app.flush_audio();
        app_capture_frames(&mut drive, 0);
    }
    assert!(drive.app.history.is_empty());
}

#[test]
fn completed_pass_undo_finishes_held_gates_once_on_end_stop_and_audio_loss() {
    use std::sync::atomic::Ordering::Relaxed;
    for end in ["end", "stop", "loss"] {
        let mut drive = app_capture_drive(vec![]);
        app_capture_sound(
            &mut drive,
            0,
            Arc::new(Sample::mono(vec![0.02; 48_000])),
            true,
        );
        app_capture_frames(&mut drive, 0);
        app_capture_start(&mut drive, 1);
        drive.pad_press(Source::Notes, 0);
        app_capture_frames(&mut drive, 6000);
        drive.app.flush_audio();
        assert!(drive.app.history.is_empty());
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits[0][3], 1);
        if end == "stop" {
            drive.app.send(Command::TransportStop);
            app_capture_frames(&mut drive, 0);
        } else {
            drive.app.send(Command::PatternRecordEnd { pass: 1 });
            if end == "loss" {
                drive.audio_rt = None;
                drive
                    .app
                    .audio
                    .as_ref()
                    .unwrap()
                    .stopped
                    .store(true, Relaxed);
                drive.app.sent.clear();
            } else {
                app_capture_frames(&mut drive, 0);
            }
        }
        for _ in 0..10 {
            drive.app.flush_audio();
        }
        assert_eq!(drive.app.history.len(), 1, "{end}");
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[0, 0, 127, 240]]
        );
        if end == "loss" {
            assert!(drive.app.sent.is_empty());
            drive.tap(KeyCode::Char('z'), M::CONTROL);
            assert!(drive.app.sent.is_empty());
        } else {
            app_capture_frames(&mut drive, 0);
            completed_pass_undo(&mut drive, Source::Keyboard);
        }
        for _ in 0..10 {
            drive.app.flush_audio();
        }
        assert!(drive.app.history.is_empty());
        assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
        assert!(drive.app.pads.pattern_rows.as_ref().unwrap().is_empty());
    }
}

#[test]
fn completed_pass_undo_reuses_slots_with_fresh_identities_and_keeps_history_bound() {
    let mut drive = app_capture_drive(vec![]);
    for step in 0..UNDO_STEPS {
        drive.app.push_step(
            vec![(0, drive.app.pads.snapshot(0))],
            None,
            format!("prior {step}"),
        );
    }
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    drive.pad_release(Source::Notes, 0);
    app_capture_finish(&mut drive, 1);
    assert_eq!(drive.app.history.len(), UNDO_STEPS);
    assert_eq!(drive.app.history[0].what, "prior 1");
    let removed = drive.app.pads.pattern_rows.clone().unwrap();
    completed_pass_undo(&mut drive, Source::Keyboard);
    assert!(!drive.app.try_send(Command::PatternRecordStart { pass: 1 }));
    app_capture_start(&mut drive, 2);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    drive.pad_release(Source::Notes, 0);
    app_capture_finish(&mut drive, 2);
    assert_ne!(drive.app.pads.pattern_rows.as_ref().unwrap(), &removed);
    assert_eq!(drive.app.pads.pattern_rows.as_ref().unwrap().len(), 1);
    assert_eq!(drive.app.history.len(), UNDO_STEPS);
    let replacement = session::Session::default();
    drive.app.apply_session(replacement);
    settle_pattern(&mut drive);
    assert!(drive.app.history.is_empty());
    assert!(drive.app.pads.pattern.is_none());
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 1);
    app_capture_frames(&mut drive, 128);
    app_capture_finish(&mut drive, 1);
    assert_eq!(drive.app.history.len(), 1);
    completed_pass_undo(&mut drive, Source::Keyboard);
    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
}

#[test]
fn completed_pass_undo_retains_publication_under_command_queue_pressure() {
    let mut drive = app_capture_drive(vec![]);
    app_capture_start(&mut drive, 1);
    drive.pad_press(Source::Notes, 0);
    app_capture_frames(&mut drive, 128);
    app_capture_finish(&mut drive, 1);
    let epoch = app_capture_snapshot(&drive).epoch;
    for _ in 0..1024 {
        drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .unwrap();
    }
    drive.app.undo_pads();
    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
    assert!(drive.app.history.is_empty());
    assert_eq!(drive.app.pending_audio.len(), 1);
    for _ in 0..512 {
        app_capture_frames(&mut drive, 128);
        drive.app.flush_audio();
        if drive.app.pending_audio.is_empty() {
            break;
        }
    }
    assert!(drive.app.pending_audio.is_empty());
    for _ in 0..8 {
        app_capture_frames(&mut drive, 0);
    }
    assert_eq!(drive.app.audio.as_ref().unwrap().commands.slots(), 1024);
    assert_eq!(app_capture_snapshot(&drive).reserved_rows, 1);
    // The next accepted start counts the restored model, not the deleted pass.
    app_capture_start(&mut drive, 2);
    assert_eq!(app_capture_snapshot(&drive).reserved_rows, 0);
}

#[test]
fn completed_pass_undo_preserves_sounding_gate_and_pitch_gate_deadlines() {
    for pitch_gate in [false, true] {
        let mut rendered = Vec::new();
        for undo in [false, true] {
            let mut drive = app_capture_drive(vec![[0, 0, 127, 960]]);
            app_capture_sound(
                &mut drive,
                0,
                Arc::new(Sample::mono(vec![0.02; 96_000])),
                true,
            );
            drive.app.send(Command::SetPadPitch {
                pad: 0,
                semitones: if pitch_gate { 12 } else { 0 },
            });
            drive.app.send(Command::SetPadPitchGate {
                pad: 0,
                enabled: pitch_gate,
                linked_slice: true,
            });
            app_capture_start(&mut drive, 1);
            let first = app_capture_frames(&mut drive, 128);
            drive.pad_press(Source::Notes, 1);
            let second = app_capture_frames(&mut drive, 128);
            app_capture_finish(&mut drive, 1);
            if undo {
                completed_pass_undo(&mut drive, Source::Keyboard);
            }
            let third = app_capture_frames(&mut drive, 30_000 - 256);
            let wave = (
                first
                    .0
                    .into_iter()
                    .chain(second.0)
                    .chain(third.0)
                    .collect::<Vec<_>>(),
                first
                    .1
                    .into_iter()
                    .chain(second.1)
                    .chain(third.1)
                    .collect::<Vec<_>>(),
            );
            let cut = if pitch_gate { 6000 } else { 24_000 };
            for channel in [&wave.0, &wave.1] {
                assert!(channel[..cut].iter().all(|&v| v == 0.02));
                assert!(channel[cut + 240..].iter().all(|&v| v == 0.0));
            }
            rendered.push(wave);
        }
        assert_eq!(rendered[0], rendered[1], "Pitch Gate {pitch_gate}");
    }
}

/// Scenario "Pattern keys work in both layouts and protocols".
#[test]
fn complete_recording_keys_use_real_layout_events_and_do_not_retoggle_on_repeat_or_release() {
    use engine::capture::Phase;
    for screen in [
        Screen::Pads,
        Screen::Files,
        Screen::Chop,
        Screen::Fx,
        Screen::Settings,
    ] {
        for kitty in [false, true] {
            for russian in [false, true] {
                for upper in [false, true] {
                    let mut drive = Drive::offline();
                    drive.app.goto(screen);
                    let letter = |en, ru| {
                        let c: char = if russian { ru } else { en };
                        KeyCode::Char(if upper {
                            c.to_uppercase().next().unwrap()
                        } else {
                            c
                        })
                    };
                    let rec = letter('r', 'к');
                    let panel = letter('n', 'т');
                    let erase = letter('e', 'у');
                    let undo = letter('z', 'я');
                    f1_pages_key(&mut drive, rec, M::CONTROL, KeyEventKind::Press, kitty);
                    app_capture_frames(&mut drive, 0);
                    assert_eq!(app_capture_snapshot(&drive).phase, Phase::CountIn);
                    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
                        f1_pages_key(&mut drive, rec, M::CONTROL, kind, kitty);
                        app_capture_frames(&mut drive, 0);
                        assert_eq!(app_capture_snapshot(&drive).phase, Phase::CountIn);
                    }
                    f1_pages_key(&mut drive, panel, M::ALT, KeyEventKind::Press, kitty);
                    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
                        f1_pages_key(&mut drive, panel, M::ALT, kind, kitty);
                        assert_eq!(drive.app.pattern_panel, Some(0));
                    }
                    f1_pages_key(&mut drive, panel, M::ALT, KeyEventKind::Press, kitty);
                    assert!(drive.app.pattern_panel.is_none());
                    f1_pages_key(&mut drive, panel, M::ALT, KeyEventKind::Release, kitty);
                    f1_pages_key(&mut drive, rec, M::CONTROL, KeyEventKind::Press, kitty);
                    f1_pages_key(&mut drive, rec, M::CONTROL, KeyEventKind::Release, kitty);
                    record_controls_idle(&mut drive);
                    drive.app.send(Command::TransportPlay);
                    app_capture_frames(&mut drive, 0);
                    f1_pages_key(&mut drive, rec, M::CONTROL, KeyEventKind::Press, kitty);
                    app_capture_frames(&mut drive, 0);
                    assert_eq!(app_capture_snapshot(&drive).phase, Phase::Recording);
                    f1_pages_key(&mut drive, rec, M::CONTROL, KeyEventKind::Release, kitty);
                    f1_pages_key(&mut drive, erase, M::ALT, KeyEventKind::Press, kitty);
                    app_capture_frames(&mut drive, 0);
                    assert!(app_capture_snapshot(&drive).erasing);
                    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
                        f1_pages_key(&mut drive, erase, M::ALT, kind, kitty);
                        app_capture_frames(&mut drive, 0);
                        assert!(app_capture_snapshot(&drive).erasing);
                    }
                    f1_pages_key(&mut drive, erase, M::ALT, KeyEventKind::Press, kitty);
                    f1_pages_key(&mut drive, erase, M::ALT, KeyEventKind::Release, kitty);
                    app_capture_frames(&mut drive, 0);
                    for modifiers in [M::NONE, M::SHIFT] {
                        for (en, ru) in "1234qwerasdfzxcv".chars().zip("1234йцукфываячсм".chars())
                        {
                            let code = letter(en, ru);
                            f1_pages_key(&mut drive, code, modifiers, KeyEventKind::Press, kitty);
                            app_capture_frames(&mut drive, 128);
                            f1_pages_key(&mut drive, code, modifiers, KeyEventKind::Release, kitty);
                            app_capture_frames(&mut drive, 0);
                            assert_eq!(app_capture_snapshot(&drive).phase, Phase::Recording);
                            assert!(!app_capture_snapshot(&drive).erasing);
                            assert!(drive.app.pattern_panel.is_none());
                        }
                    }
                    assert_eq!(app_capture_snapshot(&drive).captured_onsets, 32);
                    f1_pages_key(&mut drive, undo, M::ALT, KeyEventKind::Press, kitty);
                    record_controls_idle(&mut drive);
                    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
                    assert!(drive.app.history.is_empty());
                    assert_eq!(drive.app.screen, screen);
                    assert!(drive.app.recording.is_none());
                }
            }
        }
    }
}

/// Scenario "Change the recording grid without changing the raw take".
#[test]
fn complete_recording_every_grid_and_strength_changes_future_sound_without_rewriting_raw_rows() {
    let sounds = [700.0, 1100.0].map(|frequency| {
        let channel = |frequency: f32| {
            (0..64)
                .map(|i| {
                    0.03 * (std::f32::consts::PI * i as f32 / 63.0).sin().powi(2)
                        * (std::f32::consts::TAU * frequency * i as f32 / 48000.0).sin()
                })
                .collect::<Vec<_>>()
                .into_boxed_slice()
        };
        Arc::new(Sample {
            left: channel(frequency),
            right: Some(channel(frequency * 1.5)),
        })
    });
    let grids = [
        ("off", 0),
        ("1/4", 960),
        ("1/8", 480),
        ("1/16", 240),
        ("1/32", 120),
        ("1/64", 60),
        ("1/8T", 320),
        ("1/16T", 160),
        ("1/32T", 80),
    ];
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for (index, (grid, step)) in grids.into_iter().enumerate() {
            for strength in [0u32, 50, 100] {
                let mut drive = app_capture_drive(vec![[121, 0, 127, 0]]);
                pattern_parameter_routes(&mut drive);
                for (pad, sound) in sounds.iter().enumerate() {
                    app_capture_sound(&mut drive, pad, sound.clone(), false);
                }
                app_capture_start(&mut drive, 1);
                // the settings change during the pass first; the next hit takes them (98c)
                pattern_parameter_knob(&mut drive, XS_PORT, 101, ((index * 127 + 4) / 8) as u8);
                app_capture_frames(&mut drive, 0);
                pattern_parameter_knob(
                    &mut drive,
                    XS_PORT,
                    102,
                    ((strength * 127 + 50) / 100) as u8,
                );
                app_capture_frames(&mut drive, 128);
                drive.pad_press(source, 1);
                app_capture_frames(&mut drive, 0);
                drive.pad_release(source, 1);
                app_capture_frames(&mut drive, 0);
                drive.app.flush_audio();
                app_capture_frames(&mut drive, 0);
                assert_eq!(drive.app.pads.pattern.as_ref().unwrap().quantize, grid);
                assert_eq!(
                    u32::from(drive.app.pads.pattern.as_ref().unwrap().strength),
                    strength
                );
                app_capture_frames(&mut drive, 95872);
                let actual = app_capture_frames(&mut drive, 96000);
                let quantize = |tick: u32| {
                    if step == 0 {
                        tick
                    } else {
                        let snapped = ((tick + step / 2) / step) * step;
                        (tick * (100 - strength) + snapped * strength + 50) / 100
                    }
                };
                // the earlier row keeps its saved tick; the new row has the new settings
                assert_eq!(
                    drive.app.pads.pattern.as_ref().unwrap().hits,
                    [[121, 0, 127, 0], [quantize(5), 1, 127, 0]]
                );
                let mut events = [(121 * 25, 0), (quantize(5) as usize * 25, 1)];
                events.sort_by_key(|event| event.0);
                let expected = completed_pass_reference(&sounds, &events, 96000);
                for (a, b) in [(&actual.0, &expected.0), (&actual.1, &expected.1)] {
                    for (frame, (&got, &want)) in a.iter().zip(b).enumerate() {
                        assert!(
                            (got - want).abs() < 1e-6,
                            "{source:?}/{grid}/{strength}/{frame}: {got} != {want}"
                        );
                    }
                }
                assert_eq!(
                    drive.app.pads.pattern.as_ref().unwrap().hits,
                    [[121, 0, 127, 0], [quantize(5), 1, 127, 0]],
                    "playback does not move saved rows"
                );
                drive.app.send(Command::PatternRecordEnd { pass: 1 });
                app_capture_frames(&mut drive, 0);
                assert_eq!(
                    app_capture_snapshot(&drive).phase,
                    engine::capture::Phase::Finishing
                );
                let before = drive.app.pads.pattern.clone();
                pattern_parameter_knob(&mut drive, XS_PORT, 101, if index == 0 { 127 } else { 0 });
                assert_eq!(drive.app.pads.pattern, before);
                assert_eq!(drive.last_notice().level, Level::Warn);
                assert!(
                    drive
                        .last_notice()
                        .text
                        .to_lowercase()
                        .contains("finishing")
                );
            }
        }
        let mut count = Drive::offline();
        record_controls_routes(&mut count, source);
        pattern_parameter_routes(&mut count);
        record_controls_record(&mut count, source);
        app_capture_frames(&mut count, 0);
        let before = count.app.pads.pattern.clone();
        for (cc, value) in [(101, 0), (102, 0)] {
            pattern_parameter_knob(&mut count, XS_PORT, cc, value);
            assert_eq!(count.app.pads.pattern, before);
            assert_eq!(count.last_notice().level, Level::Warn);
            assert!(count.last_notice().text.to_lowercase().contains("count"));
        }
    }
}

/// Scenario "Record manual pads without recording the sequencer".
#[test]
fn complete_recording_excludes_repeats_clicks_previews_and_notifications_from_manual_rows() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive_raw(vec![[0, 1, 127, 0]]);
        record_controls_routes(&mut drive, source);
        for (pad, frequency) in [(0, 55.0), (1, 110.0)] {
            app_capture_sound(&mut drive, pad, app_capture_tone(frequency), false);
        }
        app_capture_start(&mut drive, 1);
        app_capture_frames(&mut drive, 128);
        if matches!(source, Source::Notes) {
            drive.bytes(PAD_PORT, &[0x90, 36, 64]);
            drive.frame();
        } else {
            drive.pad_press(source, 0);
        }
        app_capture_frames(&mut drive, 128);
        drive.pad_release(source, 0);
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.03; 48000])), 4);
        let preview = drive.tap(KeyCode::Char(' '), M::NONE);
        assert!(
            preview
                .commands()
                .iter()
                .any(|command| matches!(command, Command::Preview { sound: Some(_), .. }))
        );
        drive.app.send(Command::SetMetronome { enabled: true });
        let sounding = app_capture_frames(&mut drive, 192000);
        assert!(sounding.0.iter().any(|sample| sample.abs() > 0.001));
        let snapshot = app_capture_snapshot(&drive);
        assert_eq!(snapshot.captured_onsets, 1);
        assert!(
            drive
                .app
                .audio
                .as_ref()
                .unwrap()
                .stats
                .pattern_hits(snapshot.epoch)
                .unwrap()[1]
                >= 3
        );
        drive.app.flush_audio();
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [
                [0, 1, 127, 0],
                [
                    5,
                    0,
                    if matches!(source, Source::Notes) {
                        64
                    } else {
                        127
                    },
                    0
                ]
            ]
        );
        record_controls_record(&mut drive, source);
        record_controls_idle(&mut drive);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 2);
    }
}

/// Scenario "Refused sound input cannot create a recorded onset".
#[test]
fn complete_recording_refused_onsets_stay_silent_without_late_rows_or_empty_pad_play_feedback() {
    use std::sync::atomic::Ordering::Release;
    for (source, full) in [
        (Source::Keyboard, false),
        (Source::Notes, false),
        (Source::Button, false),
        (Source::Keyboard, true),
    ] {
        let mut drive = app_capture_drive(vec![]);
        record_controls_routes(&mut drive, source);
        app_capture_sound(
            &mut drive,
            1,
            Arc::new(Sample::mono(vec![0.02; 48000])),
            false,
        );
        app_capture_start(&mut drive, 1);
        drive.pad_press(source, 0);
        let empty = app_capture_frames(&mut drive, 128);
        assert!(empty.0.iter().chain(&empty.1).all(|sample| *sample == 0.0));
        drive.pad_release(source, 0);
        drive.app.flush_audio();
        app_capture_frames(&mut drive, 0);
        let epoch = app_capture_snapshot(&drive).epoch;
        assert!(!drive.app.audio.as_ref().unwrap().stats.pads_playing(epoch));
        if full {
            for _ in 0..1024 {
                drive
                    .app
                    .audio
                    .as_mut()
                    .unwrap()
                    .commands
                    .push(ProjectCommand {
                        epoch,
                        ready_at_send: true,
                        command: ProjectMessage::Command(Command::SetPadGain {
                            pad: 127,
                            gain: 1.0,
                        }),
                    })
                    .unwrap();
            }
        } else {
            drive.project.ready.store(epoch + 1, Release);
        }
        drive.pad_press(source, 1);
        drive.pad_release(source, 1);
        drive.project.ready.store(epoch, Release);
        for _ in 0..8 {
            let wave = app_capture_frames(&mut drive, 128);
            assert!(wave.0.iter().chain(&wave.1).all(|sample| *sample == 0.0));
            drive.frame();
            drive.app.flush_audio();
            assert!(!drive.app.audio.as_ref().unwrap().stats.pads_playing(epoch));
        }
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[0, 0, 127, 0]]
        );
        assert_eq!(app_capture_snapshot(&drive).captured_onsets, 1);
        app_capture_finish(&mut drive, 1);
    }
}

/// Scenario "Recorded positions follow audio blocks rather than redraws".
#[test]
fn complete_recording_audio_boundaries_and_half_ticks_survive_different_collection_delays() {
    let sounds = [700.0, 1100.0].map(|frequency| {
        let channel = |frequency: f32| {
            (0..64)
                .map(|i| {
                    0.03 * (std::f32::consts::PI * i as f32 / 63.0).sin().powi(2)
                        * (std::f32::consts::TAU * frequency * i as f32 / 48000.0).sin()
                })
                .collect::<Vec<_>>()
                .into_boxed_slice()
        };
        Arc::new(Sample {
            left: channel(frequency),
            right: Some(channel(frequency * 1.5)),
        })
    });
    let expected =
        completed_pass_reference(&sounds, &[(130, 0), (390, 1), (450, 0), (700, 1)], 38400);
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for collect in [false, true] {
            let mut drive = app_capture_drive(vec![]);
            drive.app.pads.pattern.as_mut().unwrap().quantize = "off".into();
            drive
                .app
                .send(drive.app.pads.recorded_pattern_command().unwrap());
            drive.app.send(Command::SetTempo { bpm: 300.0 });
            for (pad, sound) in sounds.iter().enumerate() {
                app_capture_sound(&mut drive, pad, sound.clone(), false);
            }
            app_capture_start(&mut drive, 1);
            for (delta, pad) in [(128, 0), (257, 1), (64, 0), (251, 1)] {
                app_capture_frames(&mut drive, delta);
                drive.pad_press(source, pad);
                app_capture_frames(&mut drive, 0);
                drive.pad_release(source, pad);
                app_capture_frames(&mut drive, 0);
                if collect {
                    drive.app.flush_audio();
                    app_capture_frames(&mut drive, 0);
                }
            }
            app_capture_finish(&mut drive, 1);
            assert_eq!(
                drive.app.pads.pattern.as_ref().unwrap().hits,
                [
                    [13, 0, 127, 0],
                    [39, 1, 127, 0],
                    [45, 0, 127, 0],
                    [70, 1, 127, 0]
                ]
            );
            app_capture_frames(&mut drive, 5000);
            drive.app.send(Command::TransportStop);
            drive.app.send(Command::TransportPlay);
            let actual = app_capture_frames(&mut drive, 38400);
            for (got, want) in [(&actual.0, &expected.0), (&actual.1, &expected.1)] {
                for (frame, (&a, &b)) in got.iter().zip(want).enumerate() {
                    assert!(
                        (a - b).abs() < 1e-6,
                        "{source:?}/{collect}/{frame}: {a} != {b}"
                    );
                }
                for frame in [130, 390, 450, 700] {
                    assert!(
                        got[frame..frame + 16]
                            .iter()
                            .position(|v| v.abs() > 1e-12)
                            .unwrap()
                            <= 1
                    );
                }
            }
        }
    }
}

/// Scenario "Retrigger and clamp a recorded gate".
#[test]
fn complete_recording_cross_source_retrigger_clamps_long_gates_without_changing_manual_sound() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let second = if matches!(source, Source::Keyboard) {
            Source::Notes
        } else {
            Source::Keyboard
        };
        let mut drive = app_capture_drive(vec![]);
        let mut reference = app_capture_drive(vec![]);
        for rig in [&mut drive, &mut reference] {
            app_capture_sound(rig, 0, Arc::new(Sample::mono(vec![0.02; 48000])), true);
            rig.app.send(Command::SetTempo { bpm: 300.0 });
            rig.app.send(Command::TransportPlay);
            app_capture_frames(rig, 0);
        }
        app_capture_start(&mut drive, 1);
        for rig in [&mut drive, &mut reference] {
            rig.pad_press(source, 0);
        }
        assert_eq!(
            app_capture_frames(&mut drive, 960),
            app_capture_frames(&mut reference, 960)
        );
        for rig in [&mut drive, &mut reference] {
            rig.pad_press(second, 0);
        }
        let frames = engine::pattern::MAX_GATE_TICKS as usize * 10 + 20;
        let actual = app_capture_frames(&mut drive, frames);
        let expected = app_capture_frames(&mut reference, frames);
        assert_eq!(actual, expected);
        for rig in [&mut drive, &mut reference] {
            rig.pad_release(second, 0);
            rig.pad_release(source, 0);
        }
        assert_eq!(
            app_capture_frames(&mut drive, 512),
            app_capture_frames(&mut reference, 512)
        );
        // while the pass records, the retrigger closes the first gate and the second clamps;
        // both onsets land on the 1/16 line at 0, the Gate lengths stay as played (98c)
        drive.app.flush_audio();
        app_capture_frames(&mut drive, 0);
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[0, 0, 127, 96], [0, 0, 127, 245760]]
        );
        app_capture_finish(&mut drive, 1);
        // one pad on one saved tick: the completed pass keeps the newest
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[0, 0, 127, 245760]]
        );
    }
}

/// Scenario "Erase a seam and all rows behind a collision".
#[test]
fn complete_recording_seam_erasure_removes_all_collision_rows_but_excludes_release_and_hidden_rows()
{
    let original = vec![
        [0, 0, 80, 0],
        [0, 0, 127, 0],
        [3839, 0, 100, 0],
        [3839, 0, 70, 0],
        [1, 0, 90, 0],
        [0, 1, 127, 0],
        [5000, 0, 110, 0],
    ];
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(original.clone());
        record_controls_routes(&mut drive, source);
        drive.app.pads.pattern.as_mut().unwrap().quantize = "off".into();
        drive
            .app
            .send(drive.app.pads.recorded_pattern_command().unwrap());
        let right = app_capture_tone(110.0);
        app_capture_sound(&mut drive, 0, right, false);
        app_capture_sound(
            &mut drive,
            1,
            Arc::new(Sample {
                left: vec![0.02; 64].into_boxed_slice(),
                right: Some(vec![0.0; 64].into_boxed_slice()),
            }),
            false,
        );
        app_capture_start(&mut drive, 1);
        app_capture_frames(&mut drive, 95950);
        record_controls_erase(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        drive.pad_press(source, 0);
        let seam = app_capture_frames(&mut drive, 75);
        assert!(seam.1.iter().all(|sample| *sample == 0.0));
        assert!(seam.0.iter().any(|sample| *sample > 0.0));
        drive.pad_release(source, 0);
        let boundary = app_capture_frames(&mut drive, 100);
        assert!(boundary.1.iter().any(|sample| sample.abs() > 1e-8));
        drive.app.flush_audio();
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [original[4], original[5], original[6]]
        );
        assert_eq!(app_capture_snapshot(&drive).captured_onsets, 0);
        record_controls_record(&mut drive, source);
        record_controls_idle(&mut drive);
        global_pattern_undo(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, original);
    }
}

/// Scenario "Record a first rhythm without a project file edit".
/// Scenario "Overdub an existing rhythm without restarting it".
/// Scenario "Undo erasure and grid changes as one pass".
#[test]
fn complete_recording_workflow_renders_first_take_overdub_erasure_grid_and_pass_undo() {
    use engine::capture::Phase;
    let sounds = [app_capture_tone(55.0), app_capture_tone(110.0)];
    let events = [
        (5040, 0),
        (36000, 1),
        (198000, 0),
        (228000, 1),
        (390000, 0),
        (408000, 1),
        (420000, 1),
        (582000, 0),
        (600000, 1),
        (612000, 1),
        (792000, 1),
        (800000, 0),
        (804000, 1),
        (984000, 1),
        (994000, 0),
        (996000, 1),
        (1158000, 0),
        (1176000, 1),
        (1188000, 1),
    ];
    let expected = completed_pass_reference(&sounds, &events, 1_344_000);
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.app.goto(Screen::Pads);
        record_controls_routes(&mut drive, source);
        pattern_panel_routes(&mut drive, source);
        for (pad, sound) in sounds.iter().enumerate() {
            app_capture_sound(&mut drive, pad, sound.clone(), false);
        }
        app_capture_frames(&mut drive, 0);
        let assignments = drive.app.before();
        assert!(drive.app.pads.pattern.is_none());
        let location = (
            drive.app.screen,
            drive.app.pads.bank,
            drive.app.pads.selected,
        );
        record_controls_record(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        assert_eq!(
            (
                drive.app.screen,
                drive.app.pads.bank,
                drive.app.pads.selected
            ),
            location
        );
        assert_eq!(app_capture_snapshot(&drive).phase, Phase::CountIn);
        app_capture_frames(&mut drive, 96000);
        app_capture_frames(&mut drive, 0);
        assert_eq!(app_capture_snapshot(&drive).phase, Phase::Recording);
        assert_eq!(drive.app.transport_state().ticks, 0.0);
        let mut wave = (Vec::new(), Vec::new());
        for (frame, pad) in [(5040, 0), (36000, 1)] {
            app_capture_to(&mut drive, &mut wave, frame);
            drive.pad_press(source, pad);
            app_capture_to(&mut drive, &mut wave, frame + 128);
            drive.pad_release(source, pad);
        }
        record_controls_record(&mut drive, source);
        record_controls_idle(&mut drive);
        let first = drive.app.pads.pattern.clone().unwrap();
        assert_eq!(
            (first.bars, first.quantize.as_str(), first.strength),
            (2, "1/16", 100)
        );
        assert_eq!(first.hits, [[240, 0, 127, 0], [1440, 1, 127, 0]]);
        assert_eq!(drive.app.history.len(), 1);
        assert!(drive.app.transport_state().playing);
        app_capture_to(&mut drive, &mut wave, 384000);
        let clock = drive.app.transport_state().ticks;
        record_controls_record(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        assert_eq!(drive.app.transport_state().ticks, clock);
        assert_eq!(app_capture_snapshot(&drive).pass, 2);
        app_capture_to(&mut drive, &mut wave, 408000);
        drive.pad_press(source, 1);
        app_capture_to(&mut drive, &mut wave, 408128);
        drive.pad_release(source, 1);
        record_controls_record(&mut drive, source);
        record_controls_idle(&mut drive);
        let overdub = drive.app.pads.pattern.clone().unwrap();
        assert_eq!(
            overdub.hits,
            [[240, 0, 127, 0], [1440, 1, 127, 0], [960, 1, 127, 0]]
        );
        assert_eq!(drive.app.history.len(), 2);
        app_capture_to(&mut drive, &mut wave, 768000);
        record_controls_record(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        assert_eq!(app_capture_snapshot(&drive).pass, 3);
        record_controls_erase(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        drive.pad_press(source, 0);
        app_capture_to(&mut drive, &mut wave, 774025);
        drive.pad_release(source, 0);
        app_capture_frames(&mut drive, 0);
        record_controls_erase(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        drive.registry_command(source, KeyCode::Char('n'), M::ALT, 90);
        drive.registry_command(source, KeyCode::Down, M::NONE, 92);
        drive.registry_command(source, KeyCode::Left, M::NONE, 93);
        app_capture_frames(&mut drive, 0);
        drive.registry_command(source, KeyCode::Down, M::NONE, 92);
        for _ in 0..5 {
            drive.registry_command(source, KeyCode::Left, M::NONE, 93);
            app_capture_frames(&mut drive, 0);
        }
        drive.registry_command(source, KeyCode::Char('n'), M::ALT, 90);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().quantize, "1/8");
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().strength, 50);
        app_capture_to(&mut drive, &mut wave, 800000);
        drive.pad_press(source, 0);
        app_capture_to(&mut drive, &mut wave, 800128);
        drive.pad_release(source, 0);
        record_controls_record(&mut drive, source);
        record_controls_idle(&mut drive);
        // 1/8 at 50 percent places tick 1280 halfway to its line at 1440 (98c)
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[1440, 1, 127, 0], [960, 1, 127, 0], [1360, 0, 127, 0]]
        );
        assert_eq!(drive.app.history.len(), 3);
        app_capture_to(&mut drive, &mut wave, 1008000);
        global_pattern_undo(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        assert_eq!(drive.app.pads.pattern, Some(overdub.clone()));
        assert_eq!(drive.app.history.len(), 2);
        record_controls_record(&mut drive, source);
        app_capture_frames(&mut drive, 0);
        assert_eq!(app_capture_snapshot(&drive).pass, 4);
        record_controls_record(&mut drive, source);
        record_controls_idle(&mut drive);
        assert_eq!(drive.app.pads.pattern, Some(overdub));
        assert_eq!(drive.app.history.len(), 2);
        app_capture_to(&mut drive, &mut wave, 1_344_000);
        assert_eq!(drive.app.before().pads, assignments.pads);
        assert_eq!(drive.app.before().fx, assignments.fx);
        assert_eq!((drive.app.screen, drive.app.pads.bank), (Screen::Pads, 0));
        assert!(drive.app.recording.is_none());
        for (actual, want) in [(&wave.0, &expected.0), (&wave.1, &expected.1)] {
            for (frame, (&got, &reference)) in actual.iter().zip(want).enumerate() {
                assert!(got.is_finite() && got.abs() < 1.0);
                assert!(
                    (got - reference).abs() <= 1e-6,
                    "{source:?}: frame {frame}, {got} != {reference}"
                );
            }
            for &(frame, _) in &events {
                let first = actual[frame..frame + 16]
                    .iter()
                    .position(|sample| sample.abs() > 1e-12)
                    .unwrap();
                assert!(first <= 1, "onset {frame}+{first}");
                let rms = |samples: &[f32]| {
                    (samples.iter().map(|&v| f64::from(v).powi(2)).sum::<f64>()
                        / samples.len() as f64)
                        .sqrt()
                };
                let level = rms(&actual[frame..frame + 2400]);
                let reference = rms(&want[frame..frame + 2400]);
                assert!(level > 0.0 && (20.0 * (level / reference).log10()).abs() <= 0.01);
            }
            for (start, end) in [(0, 5040), (768000, 792000), (1190400, 1344000)] {
                assert!(actual[start..end].iter().all(|v| *v == 0.0));
            }
            let step = |samples: &[f32]| {
                samples
                    .windows(2)
                    .map(|w| (w[1] - w[0]).abs())
                    .fold(0.0f32, f32::max)
            };
            assert!(step(actual) <= step(want) + 1e-6);
        }
        assert_ne!(wave.0, wave.1);
        if matches!(source, Source::Keyboard) {
            let path =
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/97-pattern-record.wav");
            let mut writer = hound::WavWriter::create(
                &path,
                hound::WavSpec {
                    channels: 2,
                    sample_rate: 48000,
                    bits_per_sample: 32,
                    sample_format: hound::SampleFormat::Float,
                },
            )
            .unwrap();
            for (&left, &right) in wave.0.iter().zip(&wave.1) {
                writer.write_sample(left).unwrap();
                writer.write_sample(right).unwrap();
            }
            writer.finalize().unwrap();
            let mut reader = hound::WavReader::open(path).unwrap();
            assert_eq!(
                (
                    reader.spec().channels,
                    reader.spec().sample_rate,
                    reader.duration()
                ),
                (2, 48000, 1344000)
            );
            let samples = reader
                .samples::<f32>()
                .map(Result::unwrap)
                .collect::<Vec<_>>();
            for (frame, pair) in samples.chunks_exact(2).enumerate() {
                assert_eq!(pair, [wave.0[frame], wave.1[frame]]);
            }
            println!(
                "Recording WAV: 1344000 frames, 48000 Hz, stereo float32; all 19 onsets within one frame, independent stereo error <=0.000001, onset RMS within 0.01 dB, finite samples, no clipping, required silence and reference-bounded cuts."
            );
        }
    }
}

fn completed_pass_reference(
    sounds: &[Arc<Sample>; 2],
    events: &[(usize, usize)],
    frames: usize,
) -> Stereo {
    let mut reference = engine::Engine::with_sounds(48_000, vec![], vec![]);
    for (pad, sound) in sounds.iter().enumerate() {
        reference.handle(
            0,
            Command::SetPadSound {
                pad,
                sound: Some(sound.clone()),
            },
        );
    }
    let mut expected = (vec![0.0; frames], vec![0.0; frames]);
    let mut from = 0;
    for &(frame, pad) in events {
        reference.process(&mut expected.0[from..frame], &mut expected.1[from..frame]);
        reference.handle(0, Command::PadPress { pad, velocity: 1.0 });
        from = frame;
    }
    reference.process(&mut expected.0[from..], &mut expected.1[from..]);
    expected
}

#[test]
fn completed_pass_undo_new_recording_reuses_slots_without_fresh_or_missing_repeats() {
    let sounds = [app_capture_tone(55.0), app_capture_tone(110.0)];
    let events = [
        (5040, 0),
        (12_000, 1),
        (102_000, 0),
        (108_000, 1),
        (197_040, 0),
        (204_000, 1),
        (294_000, 0),
        (300_000, 1),
    ];
    let expected = completed_pass_reference(&sounds, &events, 336_000);
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![[480, 1, 127, 0]]);
        swap_routes(&mut drive, source);
        for (pad, sound) in sounds.iter().enumerate() {
            app_capture_sound(&mut drive, pad, sound.clone(), false);
        }
        app_capture_frames(&mut drive, 0);
        app_capture_start(&mut drive, 1);
        let mut wave = (Vec::new(), Vec::new());
        app_capture_to(&mut drive, &mut wave, 5040);
        drive.pad_press(source, 0);
        app_capture_to(&mut drive, &mut wave, 5168);
        drive.pad_release(source, 0);
        app_capture_finish(&mut drive, 1);
        app_capture_to(&mut drive, &mut wave, 150_000);
        completed_pass_undo(&mut drive, source);
        app_capture_to(&mut drive, &mut wave, 196_912);
        app_capture_start(&mut drive, 2);
        app_capture_to(&mut drive, &mut wave, 197_040);
        drive.pad_press(source, 0);
        app_capture_to(&mut drive, &mut wave, 197_168);
        drive.pad_release(source, 0);
        app_capture_finish(&mut drive, 2);
        app_capture_to(&mut drive, &mut wave, 336_000);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 2);
        assert_eq!(drive.app.history.len(), 1);
        for (channel, reference) in [(&wave.0, &expected.0), (&wave.1, &expected.1)] {
            for (frame, (&actual, &want)) in channel.iter().zip(reference).enumerate() {
                assert!(actual.is_finite() && actual.abs() < 1.0);
                assert!(
                    (actual - want).abs() <= 0.000001,
                    "{source:?}, frame {frame}: {actual} != {want}"
                );
            }
        }
    }
}

#[test]
fn completed_pass_undo_render_keeps_current_tail_and_only_the_original_future_rhythm() {
    let sounds = [app_capture_tone(55.0), app_capture_tone(110.0)];
    let events = [
        (12_000, 1),
        (101_040, 0),
        (108_000, 1),
        (198_000, 0),
        (204_000, 1),
        (300_000, 1),
        (396_000, 1),
    ];
    let expected = completed_pass_reference(&sounds, &events, 432_000);
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for undo_at in [198_128, 210_000] {
            let mut drive = app_capture_drive(vec![[480, 1, 127, 0]]);
            swap_routes(&mut drive, source);
            for (pad, sound) in sounds.iter().enumerate() {
                app_capture_sound(&mut drive, pad, sound.clone(), false);
            }
            app_capture_frames(&mut drive, 0);
            drive.app.send(Command::TransportPlay);
            let mut wave = (Vec::new(), Vec::new());
            app_capture_to(&mut drive, &mut wave, 100_800);
            app_capture_start(&mut drive, 1);
            app_capture_to(&mut drive, &mut wave, 101_040);
            drive.pad_press(source, 0);
            app_capture_to(&mut drive, &mut wave, 101_168);
            drive.pad_release(source, 0);
            app_capture_finish(&mut drive, 1);
            app_capture_to(&mut drive, &mut wave, undo_at);
            completed_pass_undo(&mut drive, source);
            app_capture_to(&mut drive, &mut wave, 432_000);
            assert_eq!(
                drive.app.pads.pattern.as_ref().unwrap().hits,
                [[480, 1, 127, 0]]
            );
            assert!(drive.app.history.is_empty());
            for (channel, reference) in [(&wave.0, &expected.0), (&wave.1, &expected.1)] {
                for (frame, (&actual, &want)) in channel.iter().zip(reference).enumerate() {
                    assert!(actual.is_finite() && actual.abs() < 1.0);
                    assert!(
                        (actual - want).abs() <= 0.000001,
                        "{source:?}, Undo {undo_at}, frame {frame}: {actual} != {want}"
                    );
                }
                let step = |values: &[f32]| {
                    values
                        .windows(2)
                        .map(|w| (w[1] - w[0]).abs())
                        .fold(0.0f32, f32::max)
                };
                assert!(step(channel) <= step(reference) + 0.000001);
                assert!(channel[..12_000].iter().all(|&v| v == 0.0));
                assert!(channel[427_200..].iter().all(|&v| v == 0.0));
                for (frame, _) in events {
                    let first = channel[frame..frame + 16]
                        .iter()
                        .position(|v| v.abs() > 1e-12)
                        .unwrap();
                    assert!(first <= 1, "onset {frame} + {first}");
                }
            }
            if matches!(source, Source::Keyboard) && undo_at == 210_000 {
                let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../renders/97-pattern-pass-undo.wav");
                let mut writer = hound::WavWriter::create(
                    path,
                    hound::WavSpec {
                        channels: 2,
                        sample_rate: 48_000,
                        bits_per_sample: 32,
                        sample_format: hound::SampleFormat::Float,
                    },
                )
                .unwrap();
                for (&left, &right) in wave.0.iter().zip(&wave.1) {
                    writer.write_sample(left).unwrap();
                    writer.write_sample(right).unwrap();
                }
                writer.finalize().unwrap();
                println!(
                    "Pass Undo WAV: 432000 frames, 48000 Hz, stereo float32; both channels match independent manual reference within 0.000001."
                );
            }
        }
    }
}

fn app_capture_tone(frequency: f32) -> Arc<Sample> {
    let channel = |frequency: f32| {
        (0..2400)
            .map(|i| {
                let envelope = (std::f32::consts::PI * i as f32 / 2399.0).sin().powi(2);
                0.05 * envelope * (std::f32::consts::TAU * frequency * i as f32 / 48_000.0).sin()
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    };
    Arc::new(Sample {
        left: channel(frequency),
        right: Some(channel(frequency * 1.5)),
    })
}

fn app_capture_reference(late: bool, sounds: &[Arc<Sample>; 2]) -> Stereo {
    let mut engine = engine::Engine::with_sounds(48_000, vec![], vec![]);
    for (pad, sound) in [(0, sounds[0].clone()), (1, sounds[1].clone())] {
        engine.handle(
            0,
            Command::SetPadSound {
                pad,
                sound: Some(sound),
            },
        );
    }
    // The second pass ends between its manual hit and that hit's quantized step: its row
    // replaces the first pass's row, so no old hit follows at 198_000 and the new row
    // first plays at 294_000.
    let mut events = vec![(5040, 0), (197_040, 0), (294_000, 0)];
    if !late {
        events.push((102_000, 0));
    }
    events.extend((0..4).map(|lap| (12_000 + lap * 96_000, 1)));
    events.sort_unstable();
    let (mut left, mut right) = (vec![0.0; 336_000], vec![0.0; 336_000]);
    let mut at = 0;
    for (frame, pad) in events {
        engine.process(&mut left[at..frame], &mut right[at..frame]);
        engine.handle(0, Command::PadPress { pad, velocity: 1.0 });
        at = frame;
    }
    engine.process(&mut left[at..], &mut right[at..]);
    (left, right)
}

fn app_capture_to(drive: &mut Drive, wave: &mut Stereo, frame: usize) {
    assert!(frame >= wave.0.len());
    let rendered = app_capture_frames(drive, frame - wave.0.len());
    wave.0.extend(rendered.0);
    wave.1.extend(rendered.1);
}

#[test]
fn app_capture_render_retains_rhythm_without_fresh_or_delayed_double_hits() {
    let sounds = [app_capture_tone(55.0), app_capture_tone(110.0)];
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        for late in [false, true] {
            for swap in [false, true] {
                let mut drive = app_capture_drive(vec![[480, 1, 127, 0]]);
                swap_routes(&mut drive, source);
                app_capture_sound(&mut drive, 0, sounds[0].clone(), false);
                app_capture_sound(&mut drive, 1, sounds[1].clone(), false);
                app_capture_frames(&mut drive, 0);
                app_capture_start(&mut drive, 1);
                let mut wave = (Vec::new(), Vec::new());
                app_capture_to(&mut drive, &mut wave, 5040);
                drive.pad_press(source, 0);
                app_capture_to(&mut drive, &mut wave, 5168);
                if !late {
                    drive.pad_release(source, 0);
                    drive.app.flush_audio();
                    app_capture_frames(&mut drive, 0);
                }
                assert_eq!(
                    drive.app.pads.pattern.as_ref().unwrap().hits.len(),
                    if late { 1 } else { 2 },
                    "publication state before the UI resumes: {source:?}, late {late}"
                );
                app_capture_to(&mut drive, &mut wave, 150_000);
                if late {
                    drive.pad_release(source, 0);
                }
                drive.app.flush_audio();
                app_capture_frames(&mut drive, 0);
                app_capture_finish(&mut drive, 1);
                app_capture_to(&mut drive, &mut wave, 196_912);
                app_capture_start(&mut drive, 2);
                app_capture_to(&mut drive, &mut wave, 197_040);
                drive.pad_press(source, 0);
                app_capture_to(&mut drive, &mut wave, 197_168);
                drive.pad_release(source, 0);
                app_capture_finish(&mut drive, 2);
                app_capture_to(&mut drive, &mut wave, 220_000);
                let descriptors = drive.app.pads.pattern_rows.clone();
                if swap {
                    drive.registry_command(source, KeyCode::Char('x'), M::CONTROL, 110);
                    drive.pad_press(source, 15);
                    drive.pad_release(source, 15);
                    assert_eq!(drive.app.pads.pattern_rows, descriptors);
                }
                app_capture_to(&mut drive, &mut wave, 336_000);
                let expected = app_capture_reference(late, &sounds);
                for (channel, reference) in [(&wave.0, &expected.0), (&wave.1, &expected.1)] {
                    for (frame, (&actual, &want)) in channel.iter().zip(reference).enumerate() {
                        assert!(actual.is_finite() && actual.abs() < 1.0);
                        assert!(
                            (actual - want).abs() <= 0.000001,
                            "{source:?}, late {late}, swap {swap}, frame {frame}: {actual} != {want}"
                        );
                    }
                    assert!(channel[..5040].iter().all(|&v| v == 0.0));
                    assert!(channel[331_200..].iter().all(|&v| v == 0.0));
                    let actual_step = channel
                        .windows(2)
                        .map(|w| (w[1] - w[0]).abs())
                        .fold(0.0f32, f32::max);
                    let reference_step = reference
                        .windows(2)
                        .map(|w| (w[1] - w[0]).abs())
                        .fold(0.0f32, f32::max);
                    assert!(actual_step <= reference_step + 0.000001);
                }
                assert_eq!(
                    drive.app.pads.pattern.as_ref().unwrap().hits,
                    [[480, 1, 127, 0], [240, if swap { 15 } else { 0 }, 127, 0],]
                );
                if matches!(source, Source::Keyboard) && !late && !swap {
                    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../renders/97-pattern-app-capture.wav");
                    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                    let mut writer = hound::WavWriter::create(
                        path,
                        hound::WavSpec {
                            channels: 2,
                            sample_rate: 48_000,
                            bits_per_sample: 32,
                            sample_format: hound::SampleFormat::Float,
                        },
                    )
                    .unwrap();
                    for (&left, &right) in wave.0.iter().zip(&wave.1) {
                        writer.write_sample(left).unwrap();
                        writer.write_sample(right).unwrap();
                    }
                    writer.finalize().unwrap();
                    println!(
                        "App capture WAV: 336000 frames, 48000 Hz, stereo float32; both channels match independent manual reference within 0.000001."
                    );
                }
            }
        }
    }
}

fn settle_pattern(drive: &mut Drive) {
    use std::sync::atomic::Ordering::Acquire;
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
    panic!("pattern restoration did not complete");
}

fn feedback_pattern(sound: Option<Sample>, gate: bool, gate_ticks: u32) -> Drive {
    let mut drive = Drive::offline();
    drive.app.splash = true;
    let raw = session::PatternState {
        slot: "A1".into(),
        bars: 1,
        quantize: "off".into(),
        strength: 100,
        hits: vec![[0, 18, 127, gate_ticks]],
    };
    drive.app.apply_session(session::Session {
        pattern: Some("A1".into()),
        patterns: vec![raw],
        ..Default::default()
    });
    if let Some(sound) = sound {
        let sound = Arc::new(sound);
        drive.load_pad(18, sound.clone(), "feedback");
        drive.app.pads.gate[18] = gate;
        drive.app.pads.pitch[18] = 0;
        drive.app.send(Command::SetPadSound {
            pad: 18,
            sound: Some(sound),
        });
        drive.app.send(Command::SetPadMode {
            pad: 18,
            mode: if gate {
                engine::PadMode::Gate
            } else {
                engine::PadMode::OneShot
            },
        });
    }
    settle_pattern(&mut drive);
    assert!(drive.app.splash);
    drive
}

fn feedback_view(drive: &mut Drive) -> tui::Pads {
    let view = pads_view(&mut drive.app.pads, None, None, Vec::new(), None, 48_000);
    drive.app.mark_pads(view)
}

/// Scenario "Sounding pattern notes end the splash without selecting a pad".
#[test]
fn sounding_pattern_notes_end_the_splash_without_selecting_a_pad() {
    for silent in [
        "missing",
        "zero gate",
        "empty",
        "zero samples",
        "pad gain",
        "master gain",
    ] {
        let mut drive = feedback_pattern(
            (silent != "missing").then(|| {
                Sample::mono(vec![
                    if silent == "zero samples" { 0.0 } else { 0.02 };
                    1024
                ])
            }),
            silent == "zero gate",
            0,
        );
        match silent {
            "empty" => {
                let raw = drive.app.pads.pattern.as_mut().unwrap();
                raw.hits.clear();
                let pattern = Arc::new(raw.checked().unwrap().compile());
                drive.app.send(Command::SetPattern {
                    pattern: Some(pattern),
                });
            }
            "pad gain" => drive.app.send(Command::SetPadGain { pad: 18, gain: 0.0 }),
            "master gain" => drive.app.send(Command::SetMasterGain { gain: 0.0 }),
            _ => {}
        }
        settle_pattern(&mut drive);
        drive.tap(KeyCode::Char(' '), M::NONE);
        assert!(drive.app.splash, "queued event: {silent}");
        let wave = drive.render_blocks(12);
        drive.frame();
        assert!(wave.0.iter().chain(&wave.1).all(|&s| s == 0.0));
        assert!(drive.app.splash, "silent event: {silent}");
        assert!(drive.app.pads.hit_at.iter().all(Option::is_none));
        assert!(!feedback_view(&mut drive).playing);
    }
    // A note shorter than one callback must still close the splash after actual output.
    let mut drive = feedback_pattern(Some(Sample::mono(vec![0.02])), false, 0);
    drive.tap(KeyCode::Char(' '), M::NONE);
    let selection = drive.app.pads.selected;
    let input = drive.app.input_sources.latest.clone();
    drive.audio_rt.as_mut().unwrap().process(&mut [], &mut []);
    drive.frame();
    assert!(drive.app.splash, "empty callback is not a sound");
    let wave = drive.render_blocks(1);
    assert_eq!(wave.0[0], 0.02);
    assert!(wave.0[1..].iter().all(|&s| s == 0.0));
    assert!(drive.app.splash, "UI has not observed the callback yet");
    drive.frame();
    assert!(!drive.app.splash);
    assert!(drive.app.pads.hit_at[18].is_some());
    assert_eq!(drive.app.pads.selected, selection);
    assert_eq!(drive.app.input_sources.latest, input);
    assert!(
        !feedback_view(&mut drive).playing,
        "the short voice already ended"
    );
    let flash = drive.app.pads.hit_at;
    drive.render_blocks(10);
    drive.frame();
    assert_eq!(
        drive.app.pads.hit_at, flash,
        "no repeated start notification"
    );
    assert!(!drive.app.splash, "silence cannot reopen the splash");
    // Leading silence must not report a start before the first nonzero output frame.
    let mut samples = vec![0.0; 256];
    samples.extend(vec![0.02; 1024]);
    let mut drive = feedback_pattern(Some(Sample::mono(samples)), false, 0);
    drive.tap(KeyCode::Char(' '), M::NONE);
    drive.render_blocks(2);
    drive.frame();
    assert!(drive.app.splash);
    assert!(!feedback_view(&mut drive).playing);
    drive.render_blocks(1);
    drive.frame();
    assert!(!drive.app.splash);
    assert!(feedback_view(&mut drive).playing);
    // Old accepted starts cannot leak across a project transition or its ready barrier.
    let mut drive = feedback_pattern(Some(Sample::mono(vec![0.02; 1024])), false, 0);
    drive.tap(KeyCode::Char(' '), M::NONE);
    drive.render_blocks(1);
    assert!(drive.app.splash);
    drive.app.apply_session(session::Session::default());
    drive.frame();
    assert!(drive.app.splash);
    assert!(drive.app.pads.hit_at.iter().all(Option::is_none));
    finish_pattern_load(&mut drive);
    assert!(drive.app.splash);
    assert!(!feedback_view(&mut drive).playing);
}

/// Scenario "Pattern voices light PLAY without pretending to be owner input".
#[test]
fn pattern_voices_light_play_without_pretending_to_be_owner_input() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        for hidden in [false, true] {
            for editing in [false, true] {
                for bank in [0, 1] {
                    let mut drive =
                        feedback_pattern(Some(Sample::mono(vec![0.02; 8192])), true, 240);
                    swap_routes(&mut drive, source);
                    bind_pg(
                        &mut drive,
                        input::Group::Pads,
                        if matches!(source, Source::Notes) {
                            input::midi::Key::Note(100)
                        } else {
                            input::midi::Key::Cc(100)
                        },
                    );
                    drive.app.ui.hide_pads = hidden;
                    drive.app.edit = editing;
                    drive.app.pads.bank = bank;
                    drive.app.pads.selected = 3;
                    drive.app.pads.marked.extend([2, 30]);
                    drive.app.pads.held[5] = true;
                    drive.tap(KeyCode::Char(' '), M::NONE);
                    drive.registry_command(source, KeyCode::Char('g'), M::ALT, 100);
                    let input = drive.app.input_sources.latest.clone();
                    let active = drive.app.input_sources.active.clone();
                    assert_eq!(
                        active,
                        Some(if matches!(source, Source::Keyboard) {
                            input_sources::Source::Keyboard
                        } else {
                            input_device(&drive, PAD_PORT)
                        })
                    );
                    let held = drive.app.pads.held;
                    let before = drive.app.before();
                    let raw = drive.app.pads.pattern.clone();
                    assert!(
                        !feedback_view(&mut drive).playing,
                        "queued Play is not a voice"
                    );
                    drive.render_blocks(1);
                    drive.frame();
                    let view = feedback_view(&mut drive);
                    assert!(view.playing);
                    assert_eq!(view.hidden, hidden);
                    assert_eq!(view.edit.is_some(), editing);
                    assert!(drive.app.pads.hit_at[18].is_some());
                    if bank == 1 {
                        assert!(view.pads[2].flash);
                    } else {
                        assert!(!view.pads[2].flash, "another bank's pad is not this cell");
                    }
                    assert_eq!(drive.app.pads.selected, 3);
                    assert_eq!(drive.app.pads.bank, bank);
                    assert_eq!(drive.app.pads.marked, [2, 30].into());
                    assert_eq!(drive.app.pads.held, held);
                    assert_eq!(drive.app.input_sources.latest, input);
                    assert_eq!(drive.app.input_sources.active, active);
                    assert_eq!(drive.app.pads.pattern, raw);
                    for (pad, snapshot) in before.pads.iter().enumerate() {
                        assert_eq!(&drive.app.pads.snapshot(pad), snapshot);
                    }
                    let flash = drive.app.pads.hit_at;
                    drive.render_blocks(50);
                    drive.frame();
                    assert!(
                        !feedback_view(&mut drive).playing,
                        "gate deadline and fade ended"
                    );
                    assert_eq!(drive.app.pads.hit_at, flash);
                    assert!(drive.app.transport_state().playing, "clock is not PLAY");
                    assert_eq!(drive.app.input_sources.latest, input);
                }
            }
        }
    }
}

#[test]
fn pattern_drop_warning_waits_for_actual_callback_and_does_not_repeat() {
    let mut drive = feedback_pattern(Some(Sample::mono(vec![0.001; 48_000])), false, 0);
    let raw = drive.app.pads.pattern.as_mut().unwrap();
    raw.hits = (0..100).map(|tick| [tick, 18, 127, 0]).collect();
    let pattern = Arc::new(raw.checked().unwrap().compile());
    drive.app.send(Command::SetPattern {
        pattern: Some(pattern),
    });
    drive.app.send(Command::SetTempo { bpm: 300.0 });
    let dummy = Arc::new(Sample::mono(vec![0.001]));
    drive.app.pads.sound[127] = Some(dummy.clone());
    drive.app.send(Command::SetPadSound {
        pad: 127,
        sound: Some(dummy),
    });
    settle_pattern(&mut drive);
    let mut wave = (Vec::new(), Vec::new());
    fill_pattern_queue(&mut drive, true, &mut wave);
    drive.tap(KeyCode::Char(' '), M::NONE);
    assert!(
        !drive
            .app
            .notices
            .history()
            .iter()
            .any(|n| n.text.starts_with("Pattern skipped"))
    );
    for _ in 0..12 {
        pattern_block(&mut drive, &mut wave, false);
    }
    let dropped = drive.app.transport_state().dropped_hits;
    assert!(dropped > 0);
    let epoch = drive
        .project
        .epoch
        .load(std::sync::atomic::Ordering::Acquire);
    let starts = drive
        .app
        .audio
        .as_ref()
        .unwrap()
        .stats
        .pattern_hits(epoch)
        .unwrap();
    drive.frame();
    assert_eq!(
        drive.last_notice().text,
        format!("Pattern skipped {dropped} hits: audio resources are busy.")
    );
    assert_eq!(drive.last_notice().level, Level::Warn);
    let history = drive.app.notices.history().clone();
    let flash = drive.app.pads.hit_at;
    drive.app.audio.as_mut().unwrap().free_garbage();
    drive.render_blocks(30);
    drive.frame();
    assert_eq!(drive.app.notices.history(), &history);
    assert_eq!(drive.app.pads.hit_at, flash);
    assert_eq!(drive.app.transport_state().dropped_hits, dropped);
    assert_eq!(
        drive
            .app
            .audio
            .as_ref()
            .unwrap()
            .stats
            .pattern_hits(epoch)
            .unwrap(),
        starts
    );
    assert!(
        wave.0
            .iter()
            .chain(&wave.1)
            .all(|s| s.is_finite() && s.abs() < 1.0)
    );
}

fn playing_project(files: &TransitionFiles, source: Source) -> Drive {
    let mut drive = Drive::offline();
    for (number, action) in [(88, "project.pick"), (100, PG)] {
        learn::assign(
            &mut drive.app.learned,
            "Recorded pads",
            input::Group::Global,
            if matches!(source, Source::Notes) {
                input::midi::Key::Note(number)
            } else {
                input::midi::Key::Cc(number)
            },
            action,
        );
    }
    drive
        .app
        .sync_learned("Recorded pads", input::Group::Global);
    let raw = session::PatternState {
        slot: "A1".into(),
        bars: 1,
        quantize: "off".into(),
        strength: 100,
        hits: (0..16).map(|step| [step * 240, 0, 127, 0]).collect(),
    };
    session::save_project(
        &files.0.join("A/project.toml"),
        &session::Session {
            pattern: Some("A1".into()),
            patterns: vec![raw],
            pads: [(
                "0".into(),
                session::PadState {
                    file: Some(files.0.join("tone.wav")),
                    ..Default::default()
                },
            )]
            .into(),
            ..Default::default()
        },
    )
    .unwrap();
    drive.app.paths.projects = files.0.clone();
    drive.app.session_path = files.0.join("scratch.toml");
    drive.app.session_read_error = None;
    drive.app.user_path = Some(files.0.join("config.toml"));
    drive.frame();
    TransitionFiles::pick(&mut drive, source, "A", false);
    drive.tap(KeyCode::Enter, M::NONE);
    finish_pattern_load(&mut drive);
    assert!(
        drive.app.history.is_empty(),
        "project restore is not an owner edit"
    );
    drive.tap(KeyCode::Char(' '), M::NONE);
    let first = drive.render_blocks(1);
    assert!(first.0.iter().all(|&sample| sample == 0.25));
    drive
}

fn finish_pattern_load(drive: &mut Drive) {
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
    settle_pattern(drive);
}

/// Scenario "Pattern project switches reject stale work under queue pressure".
#[test]
fn pattern_project_switches_reject_stale_work_under_queue_pressure() {
    use std::sync::atomic::Ordering::Acquire;
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        for retirement in [false, true] {
            for empty in [false, true] {
                let files = TransitionFiles::new();
                let mut drive = playing_project(&files, source);
                if !empty {
                    let tone = vec![0.07; 48_000];
                    let file = files.0.join("B/samples/target.wav");
                    engine::offline::write_wav(&file, 48_000, &tone, &tone).unwrap();
                    session::save_project(
                        &files.0.join("B/project.toml"),
                        &session::Session {
                            pattern: Some("A1".into()),
                            patterns: vec![session::PatternState {
                                slot: "A1".into(),
                                bars: 2,
                                quantize: "off".into(),
                                strength: 100,
                                hits: vec![[0, 0, 127, 960]],
                            }],
                            pads: [(
                                "0".into(),
                                session::PadState {
                                    file: Some(file),
                                    gate: true,
                                    ..Default::default()
                                },
                            )]
                            .into(),
                            ..Default::default()
                        },
                    )
                    .unwrap();
                }
                drive.tap(KeyCode::Char(' '), M::NONE);
                drive.render_blocks(40);
                let old_raw = session::PatternState {
                    slot: "A1".into(),
                    bars: 1,
                    quantize: "off".into(),
                    strength: 100,
                    hits: vec![[0, 0, 127, 240], [240, 0, 127, 240]],
                };
                let old = Arc::new(old_raw.checked().unwrap().compile());
                drive.app.pads.pattern = Some(old_raw);
                drive.app.pads.gate[0] = true;
                drive.app.pads.sound[0] = Some(Arc::new(Sample::mono(vec![0.25; 48_000])));
                drive.app.pads.sound[127] = Some(Arc::new(Sample::mono(vec![0.001])));
                for pad in [0, 127] {
                    drive.app.send(Command::SetPadSound {
                        pad,
                        sound: drive.app.pads.sound[pad].clone(),
                    });
                }
                drive.app.send(Command::SetPadMode {
                    pad: 0,
                    mode: engine::PadMode::Gate,
                });
                drive.app.send(Command::SetPattern {
                    pattern: Some(old.clone()),
                });
                drive.app.dirty();
                settle_pattern(&mut drive);
                drive.tap(KeyCode::Char(' '), M::NONE);
                assert!(
                    drive
                        .render_blocks(1)
                        .0
                        .iter()
                        .all(|&sample| sample == 0.25)
                );
                let epoch = drive.project.epoch.load(Acquire);
                let mut ignored = (Vec::new(), Vec::new());
                fill_pattern_queue(&mut drive, retirement, &mut ignored);
                let late = || {
                    [
                        Command::SetPattern {
                            pattern: Some(old.clone()),
                        },
                        Command::SwapPatternPads {
                            a: 0,
                            b: 1,
                            pattern: Some(old.clone()),
                        },
                        Command::PadRelease { pad: 0 },
                        Command::TransportStop,
                    ]
                };
                for command in late() {
                    drive.app.send(command);
                }
                // Retain real old-epoch MIDI messages and their delayed UI observations.
                drive.bytes(PAD_PORT, &[0x90, 36, 127]);
                drive.bytes(PAD_PORT, &[0x80, 36, 0]);
                drive.midi().recorded_fence();
                let delayed: Vec<_> = drive.midi().events.try_iter().collect();
                TransitionFiles::pick(&mut drive, source, "B", false);
                drive.tap(KeyCode::Enter, M::NONE);
                assert_eq!(drive.app.project.as_ref().unwrap().name, "B");
                let target_epoch = drive.project.epoch.load(Acquire);
                assert_eq!(target_epoch, epoch + 1);
                assert_ne!(drive.project.ready.load(Acquire), target_epoch);
                let mut fade = (Vec::new(), Vec::new());
                for _ in 0..16 {
                    drive.app.flush_audio();
                    pattern_block(&mut drive, &mut fade, false);
                    assert_ne!(drive.project.ready.load(Acquire), target_epoch);
                    assert!(!drive.app.transport_state().playing);
                }
                assert!(fade.0[..240].iter().any(|&sample| sample > 0.0));
                assert!(
                    fade.0[240..]
                        .iter()
                        .chain(&fade.1[240..])
                        .all(|&sample| sample == 0.0)
                );
                drive.app.audio.as_mut().unwrap().free_garbage();
                finish_pattern_load(&mut drive);
                assert_eq!(drive.project.ready.load(Acquire), target_epoch);
                assert!(drive.app.history.is_empty());
                let accepted = drive.app.transport_state();
                assert!(!accepted.playing);
                assert_eq!(accepted.ticks, 0.0);
                assert_eq!(accepted.pattern_ticks, if empty { 0 } else { 7680 });
                assert!(drive.render_blocks(8).0.iter().all(|&sample| sample == 0.0));
                drive.tap(KeyCode::Char(' '), M::NONE);
                let first = drive.render_blocks(1);
                let expected = if empty { 0.0 } else { 0.07 };
                assert!(
                    first
                        .0
                        .iter()
                        .all(|&sample| (sample - expected).abs() <= 0.000001)
                );
                // Deliver obsolete pattern/swap/release/Stop after the new voice has started.
                for command in late() {
                    drive
                        .app
                        .audio
                        .as_mut()
                        .unwrap()
                        .commands
                        .push(ProjectCommand {
                            epoch,
                            ready_at_send: true,
                            command: ProjectMessage::Command(command),
                        })
                        .unwrap();
                }
                let state = drive.app.before();
                let bank = drive.app.pads.bank;
                let selected = drive.app.pads.selected;
                for event in delayed {
                    drive.app.midi_ui_event(event);
                }
                drive.frame();
                assert_eq!(drive.app.pads.bank, bank);
                assert_eq!(drive.app.pads.selected, selected);
                for (pad, snapshot) in state.pads.iter().enumerate() {
                    assert_eq!(&drive.app.pads.snapshot(pad), snapshot);
                }
                let tail = drive.render_blocks(160);
                assert!(
                    tail.0
                        .iter()
                        .chain(&tail.1)
                        .all(|&sample| sample.is_finite()
                            && sample.abs() < 1.0
                            && (sample - expected).abs() <= 0.000001)
                );
                let after = drive.app.transport_state();
                assert!(after.playing && after.ticks > 800.0);
                assert_eq!(after.pattern_ticks, accepted.pattern_ticks);
                assert_eq!(after.dropped_hits, 0);
            }
        }
    }
}

/// Scenario "Invalid pattern targets do not replace a playing project".
#[test]
fn invalid_pattern_targets_do_not_replace_a_playing_project() {
    let valid = "version = 5\npattern = \"A1\"\n[[patterns]]\nslot = \"A1\"\nbars = 1\nhits = [[0, 0, 127, 0]]\n";
    let invalid = [
        valid.replace("bars = 1", "bars = 0"),
        valid.replace("127, 0]", "127, 0, 0]"),
        valid.replace("127, 0]", "0, 0]"),
        valid.replace("pattern = \"A1\"", "pattern = \"B1\""),
        format!("{valid}[[patterns]]\nslot = \"A1\"\nbars = 1\nhits = []\n"),
        format!("{valid}[pads.0]\npitch_gate = \"on\"\n"),
        valid.replace("bars = 1", "bars = 1\nquantize = \"1/7\""),
    ];
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        for bytes in &invalid {
            let files = TransitionFiles::new();
            let mut drive = playing_project(&files, source);
            let source_bytes = std::fs::read(drive.app.session_file()).unwrap();
            std::fs::write(files.0.join("B/project.toml"), bytes).unwrap();
            let mut unused = (Vec::new(), Vec::new());
            fill_pattern_queue(&mut drive, false, &mut unused);
            drive.registry_command(source, KeyCode::Char('g'), M::ALT, 100);
            assert_eq!(drive.app.pending_audio.len(), 1);
            let raw = drive.app.pads.pattern.clone();
            let snapshot = drive.app.pads.snapshot(0);
            let dirty = drive.app.session_dirty;
            let clock = drive.app.transport_state();
            TransitionFiles::pick(&mut drive, source, "B", false);
            let picker = TransitionFiles::picker(&drive);
            drive.tap(KeyCode::Enter, M::NONE);
            assert_eq!(TransitionFiles::picker(&drive), picker);
            assert_eq!(drive.app.project.as_ref().unwrap().name, "A");
            assert_eq!(drive.app.pads.pattern, raw);
            assert_eq!(drive.app.pads.snapshot(0), snapshot);
            assert_eq!(drive.app.history.len(), 1);
            assert_eq!(drive.app.pending_audio.len(), 1);
            assert_eq!(drive.app.session_dirty, dirty);
            assert_eq!(drive.app.transport_state(), clock);
            assert_eq!(
                std::fs::read(drive.app.session_file()).unwrap(),
                source_bytes
            );
            assert_eq!(
                std::fs::read_to_string(files.0.join("B/project.toml")).unwrap(),
                *bytes
            );
            assert!(drive.last_notice().text.starts_with("Project not opened:"));
            for _ in 0..8 {
                drive.app.flush_audio();
                let wave = drive.render_blocks(1);
                assert!(wave.0.iter().chain(&wave.1).all(|&sample| sample == 0.25));
            }
            assert!(drive.app.transport_state().ticks > clock.ticks);
            assert!(drive.app.transport_state().playing);
            assert!(drive.app.pending_audio.is_empty());
        }
    }
}

/// Scenario "Refused or same-folder selection preserves the pattern".
#[test]
fn refused_or_same_folder_selection_preserves_the_pattern() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        for cause in ["target read", "source save", "canonical alias"] {
            let files = TransitionFiles::new();
            let mut drive = playing_project(&files, source);
            let source_bytes = std::fs::read(drive.app.session_file()).unwrap();
            drive.registry_command(source, KeyCode::Char('g'), M::ALT, 100);
            let target = match cause {
                "target read" => "B",
                "source save" => {
                    std::fs::create_dir(drive.app.session_file().with_extension("toml.tmp"))
                        .unwrap();
                    "B"
                }
                _ => {
                    std::os::unix::fs::symlink(files.0.join("A"), files.0.join("Alias")).unwrap();
                    "Alias"
                }
            };
            let raw = drive.app.pads.pattern.clone();
            let snapshot = drive.app.pads.snapshot(0);
            let dirty = drive.app.session_dirty;
            let clock = drive.app.transport_state();
            let epoch = drive
                .project
                .epoch
                .load(std::sync::atomic::Ordering::Acquire);
            TransitionFiles::pick(&mut drive, source, target, false);
            let picker = TransitionFiles::picker(&drive);
            if cause == "target read" {
                // The file disappears after listing and before the owner confirms it.
                std::fs::remove_file(files.0.join("B/project.toml")).unwrap();
            }
            drive.tap(KeyCode::Enter, M::NONE);
            if cause == "canonical alias" {
                assert!(drive.app.project_picker.is_none());
            } else {
                assert_eq!(TransitionFiles::picker(&drive), picker);
            }
            assert_eq!(drive.app.project.as_ref().unwrap().name, "A");
            assert_eq!(
                drive
                    .project
                    .epoch
                    .load(std::sync::atomic::Ordering::Acquire),
                epoch
            );
            assert_eq!(drive.app.pads.pattern, raw);
            assert_eq!(drive.app.pads.snapshot(0), snapshot);
            assert_eq!(drive.app.history.len(), 1);
            assert_eq!(drive.app.session_dirty, dirty);
            assert_eq!(drive.app.transport_state(), clock);
            assert_eq!(
                std::fs::read(drive.app.session_file()).unwrap(),
                source_bytes
            );
            let wave = drive.render_blocks(80);
            for channel in [&wave.0, &wave.1] {
                for (frame, &sample) in channel.iter().enumerate() {
                    let local = (frame + 128) % 6000;
                    assert_eq!(
                        sample,
                        if local < 4096 { 0.25 } else { 0.0 },
                        "{source:?}, {cause}, {frame}"
                    );
                }
            }
            let after = drive.app.transport_state();
            assert!(after.playing && after.ticks > clock.ticks);
            assert_eq!(after.pattern_ticks, clock.pattern_ticks);
        }
    }
}

fn pattern_tone(drive: &mut Drive, raw: &session::PatternState, level: f32, length: usize) {
    assert_eq!(drive.app.pads.pattern.as_ref(), Some(raw));
    if drive.app.transport_state().playing {
        drive.tap(KeyCode::Char(' '), M::NONE);
    }
    drive.app.flush_audio();
    drive.render_blocks(40);
    drive.tap(KeyCode::Char(' '), M::NONE);
    let wave = drive.render_blocks(64);
    for channel in [&wave.0, &wave.1] {
        for (frame, &sample) in channel.iter().enumerate() {
            let expected = if frame % 6000 < length { level } else { 0.0 };
            assert!(sample.is_finite() && sample.abs() < 1.0);
            assert!(
                (sample - expected).abs() <= 0.000001,
                "frame {frame}: {sample} != {expected}"
            );
        }
    }
    assert_eq!(drive.app.pads.pattern.as_ref(), Some(raw));
}

/// Scenario "Pattern addresses stay during copy clear and file replacement".
#[test]
fn pattern_addresses_stay_during_copy_clear_and_file_replacement() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        let files = TransitionFiles::new();
        let mut drive = Drive::offline();
        for (number, action) in [
            (100, PG),
            (101, "pad.copy"),
            (102, "pad.clear"),
            (103, "edit.undo"),
            (104, "browser.hold"),
            (105, "screen.pads"),
            (106, "pad.edit"),
        ] {
            learn::assign(
                &mut drive.app.learned,
                "Recorded pads",
                input::Group::Global,
                if matches!(source, Source::Notes) {
                    input::midi::Key::Note(number)
                } else {
                    input::midi::Key::Cc(number)
                },
                action,
            );
        }
        drive
            .app
            .sync_learned("Recorded pads", input::Group::Global);
        let raw = session::PatternState {
            slot: "A1".into(),
            bars: 1,
            quantize: "off".into(),
            strength: 100,
            hits: vec![[0, 1, 127, 0], [240, 1, 127, 0], [5000, 1, 80, 0]],
        };
        drive.app.apply_session(session::Session {
            pattern: Some("A1".into()),
            patterns: vec![raw.clone()],
            ..Default::default()
        });
        let sound = Arc::new(Sample::mono(vec![0.02; 64]));
        drive.load_pad(0, sound.clone(), "copy-source");
        drive.app.pads.gate[0] = false;
        drive.app.pads.pitch[0] = 0;
        drive.app.pads.pitch_gate[0] = false;
        drive.app.send(Command::SetPadSound {
            pad: 0,
            sound: Some(sound),
        });
        settle_pattern(&mut drive);
        pattern_tone(&mut drive, &raw, 0.0, 0);
        drive.registry_command(source, KeyCode::Char('c'), M::CONTROL, 101);
        // Copy must use the pickup snapshot, not this later source edit.
        drive.registry_command(source, KeyCode::Char('g'), M::ALT, 100);
        drive.pad_press(source, 1);
        drive.pad_release(source, 1);
        assert!(!drive.app.pads.pitch_gate[1]);
        assert!(drive.app.pads.pitch_gate[0]);
        pattern_tone(&mut drive, &raw, 0.02, 64);
        drive.registry_command(source, KeyCode::Char('z'), M::CONTROL, 103);
        assert!(drive.app.pads.file[1].is_none());
        pattern_tone(&mut drive, &raw, 0.0, 0);
        drive.app.pads.selected = 0;
        drive.registry_command(source, KeyCode::Char('c'), M::CONTROL, 101);
        drive.pad_press(source, 1);
        drive.pad_release(source, 1);
        pattern_tone(&mut drive, &raw, 0.02, 64);
        // An ordinary owner hit selects the target before Clear.
        drive.pad_press(source, 1);
        drive.pad_release(source, 1);
        assert_eq!(drive.app.pads.selected, 1);
        drive.registry_command(source, KeyCode::Backspace, M::NONE, 102);
        assert!(drive.app.pads.file[1].is_none());
        pattern_tone(&mut drive, &raw, 0.0, 0);
        drive.registry_command(source, KeyCode::Char('z'), M::CONTROL, 103);
        pattern_tone(&mut drive, &raw, 0.02, 64);
        drive.tap(KeyCode::F(6), M::NONE);
        drive.app.preview_on = false;
        drive.app.browser = browser::Browser::new(Some(files.0.clone()));
        drive.app.browser.selected = drive
            .app
            .browser
            .entries
            .iter()
            .position(|entry| entry.name == "tone.wav")
            .unwrap();
        drive.frame();
        drive.registry_command(source, KeyCode::Enter, M::NONE, 104);
        assert_eq!(
            drive.app.browser.in_hand.as_ref(),
            Some(&files.0.join("tone.wav"))
        );
        drive.registry_command(source, KeyCode::F(2), M::NONE, 105);
        assert_silent_target(&drive.pad_press(source, 1));
        drive.pad_release(source, 1);
        assert!(matches!(drive.app.ask, Some(Ask::PutFile { .. })));
        drive.registry_command(source, KeyCode::Enter, M::NONE, 106);
        finish_pattern_load(&mut drive);
        assert_eq!(
            drive.app.pads.file[1].as_ref(),
            Some(&files.0.join("tone.wav"))
        );
        pattern_tone(&mut drive, &raw, 0.25, 4096);
        drive.registry_command(source, KeyCode::Char('z'), M::CONTROL, 103);
        pattern_tone(&mut drive, &raw, 0.02, 64);
    }
}

/// Scenario "Pattern addresses do not follow the open bank".
#[test]
fn pattern_addresses_do_not_follow_the_open_bank() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        let mut drive = Drive::offline();
        swap_routes(&mut drive, source);
        let raw = session::PatternState {
            slot: "A1".into(),
            bars: 1,
            quantize: "off".into(),
            strength: 100,
            hits: vec![[0, 0, 127, 0], [240, 127, 64, 0]],
        };
        drive.app.apply_session(session::Session {
            pattern: Some("A1".into()),
            patterns: vec![raw.clone()],
            ..Default::default()
        });
        for (pad, value) in [(0, 0.03), (127, 0.07)] {
            drive.app.send(Command::SetPadSound {
                pad,
                sound: Some(Arc::new(Sample::mono(vec![value; 1024]))),
            });
        }
        settle_pattern(&mut drive);
        drive.tap(KeyCode::Char(' '), M::NONE);
        let mut wave = drive.render_blocks(20).0;
        drive.registry_command(source, KeyCode::Char('8'), M::CONTROL, 112);
        assert_eq!(drive.app.pads.bank, 7);
        wave.extend(drive.render_blocks(40).0);
        drive.registry_command(source, KeyCode::Char('1'), M::CONTROL, 114);
        assert_eq!(drive.app.pads.bank, 0);
        wave.extend(drive.render_blocks(720).0);
        assert_eq!(drive.app.pads.pattern.as_ref(), Some(&raw));
        assert!(
            wave.iter()
                .all(|sample| sample.is_finite() && sample.abs() < 1.0)
        );
        for (frame, sample) in wave.iter().enumerate() {
            let local = frame % 96_000;
            let expected = if local < 1024 {
                0.03
            } else if (6000..7024).contains(&local) {
                0.07 * 64.0 / 127.0
            } else {
                0.0
            };
            assert!(
                (sample - expected).abs() <= 0.000001,
                "{source:?}, frame {frame}: {sample} != {expected}"
            );
        }
    }
}

#[test]
fn chop_entry_hush_does_not_stop_future_pattern_events() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        let mut drive = Drive::offline();
        for (number, action) in [
            (106, "screen.chop"),
            (107, "screen.pads"),
            (108, "transport.toggle"),
        ] {
            learn::assign(
                &mut drive.app.learned,
                "Recorded pads",
                input::Group::Global,
                if matches!(source, Source::Notes) {
                    input::midi::Key::Note(number)
                } else {
                    input::midi::Key::Cc(number)
                },
                action,
            );
        }
        drive
            .app
            .sync_learned("Recorded pads", input::Group::Global);
        let raw = session::PatternState {
            slot: "A1".into(),
            bars: 1,
            quantize: "off".into(),
            strength: 100,
            hits: vec![[0, 0, 127, 0], [240, 0, 127, 0]],
        };
        drive.app.apply_session(session::Session {
            pattern: Some("A1".into()),
            patterns: vec![raw],
            ..Default::default()
        });
        let sound = Arc::new(Sample::mono(vec![0.02; 48_000]));
        drive.load_pad(0, sound.clone(), "chop");
        drive.app.pads.gate[0] = false;
        drive.app.pads.pitch[0] = 0;
        for command in [
            Command::SetPadSound {
                pad: 0,
                sound: Some(sound),
            },
            Command::SetPadMode {
                pad: 0,
                mode: engine::PadMode::OneShot,
            },
            Command::SetPadPitch {
                pad: 0,
                semitones: 0,
            },
        ] {
            drive.app.send(command);
        }
        settle_pattern(&mut drive);
        drive.registry_command(source, KeyCode::Char(' '), M::NONE, 108);
        assert_eq!(drive.render_blocks(1).0[0], 0.02);
        let frame = drive.registry_command(source, KeyCode::F(3), M::NONE, 106);
        assert_eq!(drive.app.screen, Screen::Chop);
        assert!(
            frame
                .commands()
                .iter()
                .any(|c| matches!(c, Command::HushPads))
        );
        let wave = drive.render_blocks(60);
        for channel in [&wave.0, &wave.1] {
            assert_eq!(channel[0], 0.02);
            assert!(channel[240..6000 - RENDER_BLOCK].iter().all(|&v| v == 0.0));
            assert_eq!(channel[6000 - RENDER_BLOCK], 0.02);
        }
        assert!(drive.app.transport_state().playing);
        let frame = drive.tap(KeyCode::Char(' '), M::NONE);
        assert!(
            frame
                .commands()
                .iter()
                .any(|c| matches!(c, Command::Preview { sound: Some(_), .. }))
        );
        assert!(
            !frame
                .commands()
                .iter()
                .any(|c| matches!(c, Command::TransportToggle))
        );
        drive.render_blocks(1);
        assert!(drive.app.transport_state().playing);
        drive.registry_command(source, KeyCode::F(2), M::NONE, 107);
        assert_eq!(drive.app.screen, Screen::Pads);
        drive.registry_command(source, KeyCode::Char(' '), M::NONE, 108);
        drive.render_blocks(1);
        assert!(!drive.app.transport_state().playing);
    }
}

/// Scenario "Pattern title follows the accepted loop across screens".
#[test]
fn pattern_title_follows_the_accepted_loop_across_screens() {
    use std::sync::atomic::Ordering::{Acquire, Release};
    let mut drive = Drive::offline();
    let mut raw = raw_pattern();
    raw.hits.clear();
    drive.app.apply_session(session::Session {
        pattern: Some("A1".into()),
        patterns: vec![raw.clone()],
        ..Default::default()
    });
    let label = |drive: &Drive| drive.app.title_badges()[0].normal.clone();
    assert!(!label(&drive).contains("A1"), "restore is not yet accepted");
    settle_pattern(&mut drive);
    assert!(label(&drive).contains("A1 1:1"), "empty A1 is present");
    drive.tap(KeyCode::Char(' '), M::NONE);
    settle_pattern(&mut drive);
    drive.render_blocks(1600);
    let accepted = drive.app.transport_state();
    assert!(accepted.ticks > 2.0 * 3840.0);
    let expected = {
        let mut relative = accepted;
        relative.ticks %= f64::from(accepted.pattern_ticks);
        let (bar, beat) = relative.bar_beat();
        format!("A1 {bar}:{beat}")
    };
    let absolute = accepted.bar_beat();
    for screen in Screen::ALL {
        drive.app.screen = screen;
        let title = label(&drive);
        assert!(title.contains(&expected), "{screen:?}: {title}");
        assert!(
            title.contains(&format!("{}:{}", absolute.0, absolute.1)),
            "{title}"
        );
        assert_eq!(drive.app.transport_state(), accepted, "no UI clock");
    }
    drive.app.screen = Screen::Pads;
    raw.bars = 4;
    drive.app.pads.pattern = Some(raw.clone());
    drive.app.send(Command::SetPattern {
        pattern: Some(Arc::new(raw.checked().unwrap().compile())),
    });
    drive.tap(KeyCode::Up, M::CONTROL);
    assert_eq!(
        drive.app.transport_state(),
        accepted,
        "pending length and BPM stay hidden"
    );
    assert!(label(&drive).contains(&expected));
    settle_pattern(&mut drive);
    let replacement = drive.app.transport_state();
    assert_eq!(replacement.pattern_ticks, 4 * 3840);
    assert_eq!(replacement.bpm, 121.0);
    assert!(replacement.ticks > accepted.ticks);
    assert!(label(&drive).contains("A1 3:"));
    drive.tap(KeyCode::Char(' '), M::NONE);
    settle_pattern(&mut drive);
    assert!(label(&drive).contains("A1 1:1"));
    let epoch = drive.project.epoch.load(Acquire);
    drive.project.epoch.store(epoch + 1, Release);
    assert!(
        !label(&drive).contains("A1"),
        "old project snapshot is not a pattern"
    );
    drive.project.ready.store(epoch + 1, Release);
    assert!(
        !label(&drive).contains("A1"),
        "cached old epoch remains excluded"
    );
}

/// Scenario "Pitch Gate feedback preserves parameter pages".
#[test]
fn pitch_gate_feedback_preserves_parameter_pages() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        for hidden in [false, true] {
            for editing in [false, true] {
                for (linked, pitch) in [(true, 12), (true, 0), (true, -12), (false, 12)] {
                    let mut drive = Drive::new();
                    drive.app.ui.hide_pads = hidden;
                    drive.app.edit = editing;
                    drive.load_pad(0, Arc::new(Sample::mono(vec![0.02; 4096])), "slice");
                    drive.app.pads.pitch[0] = pitch;
                    if linked {
                        drive.app.pads.link[0] = Some((0, 0));
                        drive.app.pads.slice[0] = Some(1);
                    }
                    bind_pg(
                        &mut drive,
                        input::Group::Pads,
                        if matches!(source, Source::Notes) {
                            input::midi::Key::Note(100)
                        } else {
                            input::midi::Key::Cc(100)
                        },
                    );
                    drive.keymap.overlay("[screens.pads]\n\"Alt+G\" = \"none\"\n\"Alt+H\" = \"pad.pitch_gate_toggle\"\n").unwrap();
                    for page in 0..pads::page_names().len() {
                        drive
                            .app
                            .pads
                            .set_page_pos((page as f32 + 0.5) / pads::page_names().len() as f32);
                        let knobs = drive.app.pads.page().1.to_vec();
                        let param = drive.app.pads.param;
                        drive.registry_command(source, KeyCode::Char('h'), M::ALT, 100);
                        let enabled = drive.app.pads.pitch_gate[0];
                        let (state, word, description, explanation) = if !enabled {
                            (
                                tui::PitchGate::Off,
                                "PG off",
                                "PG off",
                                "PG off, manual unchanged",
                            )
                        } else if !linked {
                            (
                                tui::PitchGate::NeedsSlice,
                                "PG n/a",
                                "PG on: needs linked slice",
                                "PG on: needs linked slice, manual unchanged",
                            )
                        } else if pitch <= 0 {
                            (
                                tui::PitchGate::NeedsPitch,
                                "PG n/a",
                                "PG on: needs Pitch > 0",
                                "PG on: needs Pitch > 0, manual unchanged",
                            )
                        } else {
                            (
                                tui::PitchGate::A1,
                                "PG A1",
                                "PG on: A1 only",
                                "PG on: A1 only, manual unchanged",
                            )
                        };
                        assert_eq!(shown_value(&drive).formatted, description);
                        assert_eq!(shown_value(&drive).object, "pad 1-1");
                        assert_eq!(drive.app.pads.page().1, knobs);
                        assert_eq!(drive.app.pads.param, param);
                        let view =
                            pads_view(&mut drive.app.pads, None, None, Vec::new(), None, 48_000);
                        assert_eq!(view.pads[0].pitch_gate, state);
                        let view = drive.app.mark_pads(view);
                        assert_eq!(view.hidden, hidden);
                        if editing {
                            assert_eq!(view.edit, Some(pads::page_names()[page]));
                            assert_eq!(
                                view.knobs,
                                knobs.iter().map(|&p| tui_param(p)).collect::<Vec<_>>()
                            );
                        } else {
                            assert!(view.edit.is_none());
                        }
                        for (width, height) in [(80, 26), (120, 40)] {
                            let rows =
                                shared_feedback_rows(&mut drive, width, height, Instant::now());
                            let mode = rows
                                .iter()
                                .find(|row| row.contains("mode "))
                                .expect("mode row");
                            assert!(mode.contains(word), "{width}x{height}: {mode}");
                            // the help row explains only a linked slice with Pitch Gate on
                            let applies = explanation.starts_with("PG on: A1 only")
                                || explanation.starts_with("PG on: needs Pitch");
                            assert_eq!(
                                rows[height as usize - 1].contains(explanation),
                                applies,
                                "{rows:?}"
                            );
                        }
                        let help = drive.app.full_help_keys(&drive.keymap, true);
                        assert!(
                            help.iter()
                                .any(|(keys, text)| keys == "Alt+H" && text.contains(description))
                        );
                        assert!(
                            !help
                                .iter()
                                .any(|(keys, text)| keys == "Alt+G" && text.contains("PG"))
                        );
                        assert!(
                            help.iter()
                                .any(|(_, text)| text
                                    .contains("linked slices with pitch above zero"))
                        );
                        assert!(help.iter().any(|(keys, text)| keys == "Recorded pads" && text.contains("100")));
                        assert_eq!(drive.app.pads.page().0, pads::page_names()[page]);
                    }
                }
            }
        }
    }
}

/// Scenario "Pitch Gate reports source and pitch restrictions".
#[test]
fn pitch_gate_reports_source_and_pitch_restrictions() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        let mut drive = Drive::new();
        bind_pg(
            &mut drive,
            input::Group::Pads,
            if matches!(source, Source::Notes) {
                input::midi::Key::Note(100)
            } else {
                input::midi::Key::Cc(100)
            },
        );
        for kind in ["empty", "whole", "detached", "linked"] {
            drive.app.pads = pads::Pads::default();
            if kind != "empty" {
                drive.load_pad(0, Arc::new(Sample::mono(vec![0.02; 4096])), "restriction");
            }
            if kind == "detached" {
                drive.app.pads.region[0] = engine::Region {
                    start: 16,
                    end: 2048,
                };
                drive.app.pads.slice[0] = Some(1);
            } else if kind == "linked" {
                drive.app.pads.link[0] = Some((0, 0));
                drive.app.pads.slice[0] = Some(1);
            }
            for pitch in [-1, 0, 1] {
                drive.app.pads.pitch[0] = pitch;
                for enabled in [false, true] {
                    if drive.app.pads.pitch_gate[0] != enabled {
                        drive.registry_command(source, KeyCode::Char('g'), M::ALT, 100);
                    }
                    let (state, reason) = if !enabled {
                        ("PG off", "PG off, manual unchanged")
                    } else if kind != "linked" {
                        ("PG n/a", "PG on: needs linked slice, manual unchanged")
                    } else if pitch <= 0 {
                        ("PG n/a", "PG on: needs Pitch > 0, manual unchanged")
                    } else {
                        ("PG A1", "PG on: A1 only, manual unchanged")
                    };
                    let rows = shared_feedback_rows(&mut drive, 80, 26, Instant::now());
                    assert!(
                        rows.iter()
                            .any(|row| row.contains("mode ") && row.contains(state)),
                        "{kind}, {pitch}: {rows:?}"
                    );
                    // the help row explains only a linked slice with Pitch Gate on; F1 gives
                    // the restriction for every pad
                    let applies = reason.starts_with("PG on: A1 only")
                        || reason.starts_with("PG on: needs Pitch");
                    assert_eq!(
                        rows[25].contains(reason),
                        applies,
                        "{kind}, {pitch}: {}",
                        rows[25]
                    );
                    let text = reason.trim_end_matches(", manual unchanged");
                    let help = drive.app.full_help_keys(&drive.keymap, true);
                    assert!(
                        help.iter().any(|(_, line)| line.contains(text)),
                        "{kind}, {pitch}: {help:?}"
                    );
                    assert_eq!(drive.app.pads.pitch_gate[0], enabled);
                }
            }
        }
        for pitch in [-1, 0, 1] {
            drive.app.pads.pitch[0] = pitch;
            let rows = shared_feedback_rows(&mut drive, 80, 26, Instant::now());
            assert!(rows[25].contains(if pitch > 0 {
                "PG on: A1 only, manual unchanged"
            } else {
                "PG on: needs Pitch > 0, manual unchanged"
            }));
            assert!(
                drive.app.pads.pitch_gate[0],
                "a pitch change cannot disable the switch"
            );
        }
        drive.app.pads.pitch[1] = 0;
        drive.load_pad(1, Arc::new(Sample::mono(vec![0.02; 4096])), "other");
        drive.pad_press(source, 1);
        drive.pad_release(source, 1);
        let rows = shared_feedback_rows(&mut drive, 80, 26, Instant::now());
        // a whole file: the help row leaves Pitch Gate out, F1 keeps the reason
        assert!(!rows[25].contains("needs linked slice"), "{}", rows[25]);
        let help = drive.app.full_help_keys(&drive.keymap, true);
        assert!(
            help.iter()
                .any(|(_, line)| line.contains("PG on: needs linked slice"))
        );
        assert!(drive.app.pads.pitch_gate[0] && drive.app.pads.pitch_gate[1]);
        drive.pad_press(source, 0);
        drive.pad_release(source, 0);
        let rows = shared_feedback_rows(&mut drive, 80, 26, Instant::now());
        assert!(rows[25].contains("PG on: A1 only, manual unchanged"));
        drive.app.pads.link[0] = None;
        // a detached piece: the help row drops Pitch Gate at once, F1 gives the reason
        let rows = shared_feedback_rows(&mut drive, 80, 26, Instant::now());
        assert!(!rows[25].contains("PG "), "{}", rows[25]);
        let help = drive.app.full_help_keys(&drive.keymap, true);
        assert!(
            help.iter()
                .any(|(_, line)| line.contains("PG on: needs linked slice"))
        );
        assert!(drive.app.pads.pitch_gate[0]);
    }
}

/// Scenario "Pitch Gate scope stays explicit during manual play".
#[test]
fn pitch_gate_scope_stays_explicit_during_manual_play() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        for phase in ["absent", "stopped", "mixed"] {
            let mut drive = Drive::offline();
            let sound = Arc::new(Sample::mono(vec![0.02; 48_000]));
            drive.load_pad(0, sound.clone(), "manual and A1");
            drive.app.pads.link[0] = Some((0, 0));
            drive.app.pads.slice[0] = Some(1);
            drive.app.pads.pitch[0] = 12;
            drive.app.pads.gate[0] = false;
            for command in [
                Command::SetPadSound {
                    pad: 0,
                    sound: Some(sound),
                },
                Command::SetPadPitch {
                    pad: 0,
                    semitones: 12,
                },
                Command::SetPadMode {
                    pad: 0,
                    mode: engine::PadMode::OneShot,
                },
                drive.app.pads.pitch_gate_command(0),
            ] {
                drive.app.send(command);
            }
            if phase != "absent" {
                let raw = session::PatternState {
                    slot: "A1".into(),
                    bars: 1,
                    quantize: "1/16".into(),
                    strength: 100,
                    hits: vec![[0, 0, 127, 0]],
                };
                drive.app.send(Command::SetPattern {
                    pattern: Some(Arc::new(raw.checked().unwrap().compile())),
                });
                drive.app.pads.pattern = Some(raw);
            }
            settle_pattern(&mut drive);
            if phase == "mixed" {
                drive.tap(KeyCode::Char(' '), M::NONE);
                assert!(drive.render_blocks(1).0.iter().any(|&s| s > 0.0));
            }
            let before = shared_feedback_rows(&mut drive, 80, 26, Instant::now());
            assert!(before[25].contains("PG on: A1 only, manual unchanged"));
            drive.pad_press(source, 0);
            let sound = drive.render_blocks(64);
            drive.pad_release(source, 0);
            assert!(
                sound.0[7000..]
                    .iter()
                    .chain(&sound.1[7000..])
                    .all(|&s| s > 0.0 && s.is_finite() && s < 1.0),
                "manual tail stays after the 1/16 deadline: {phase}"
            );
            let after = shared_feedback_rows(&mut drive, 80, 26, Instant::now());
            assert!(after[25].contains("PG on: A1 only, manual unchanged"));
            for rows in [&before, &after] {
                assert!(
                    rows.iter()
                        .any(|row| row.contains("mode ") && row.contains("PG A1"))
                );
                assert!(!rows[1].contains("PG"), "no top badge");
                assert!(!rows[25].contains("active"));
            }
            assert_eq!(drive.app.transport_state().playing, phase == "mixed");
        }
    }
}

/// Scenario "Pitch Gate help explains an inaudible manual toggle".
#[test]
fn pitch_gate_help_explains_an_inaudible_manual_toggle() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        for (editing, linked, pitch) in [false, true]
            .into_iter()
            .flat_map(|edit| [(edit, false, 12), (edit, true, 0), (edit, true, 12)])
        {
            let mut drive = Drive::offline();
            drive.load_pad(0, Arc::new(Sample::mono(vec![0.02; 48_000])), "manual");
            drive.app.edit = editing;
            drive.app.pads.pitch[0] = pitch;
            if linked {
                drive.app.pads.link[0] = Some((0, 0));
                drive.app.pads.slice[0] = Some(1);
            }
            bind_pg(
                &mut drive,
                input::Group::Pads,
                if matches!(source, Source::Notes) {
                    input::midi::Key::Note(100)
                } else {
                    input::midi::Key::Cc(100)
                },
            );
            drive
                .keymap
                .overlay(
                    "[screens.pads]\n\"Alt+G\" = \"none\"\n\"Alt+H\" = \"pad.pitch_gate_toggle\"\n",
                )
                .unwrap();
            let sound = drive.app.pads.sound[0].clone();
            drive.app.send(Command::SetPadSound { pad: 0, sound });
            drive.app.send(Command::SetPadPitch {
                pad: 0,
                semitones: pitch,
            });
            drive.app.send(drive.app.pads.pitch_gate_command(0));
            drive.app.pads.gate[0] = false;
            drive.app.send(Command::SetPadMode {
                pad: 0,
                mode: engine::PadMode::OneShot,
            });
            settle_pattern(&mut drive);
            drive.pad_press(source, 0);
            let mut waves = Vec::new();
            for on in [false, true] {
                let frame = drive.registry_command(source, KeyCode::Char('h'), M::ALT, 100);
                assert!(frame.commands().iter().any(|cmd| matches!(cmd, Command::SetPadPitchGate { pad: 0, enabled, linked_slice } if *enabled == on && *linked_slice == linked)));
                let value = shown_value(&drive);
                assert_eq!(value.object, "pad 1-1");
                let description = if !on {
                    "PG off"
                } else if !linked {
                    "PG on: needs linked slice"
                } else if pitch <= 0 {
                    "PG on: needs Pitch > 0"
                } else {
                    "PG on: A1 only"
                };
                assert_eq!(value.formatted, description);
                let wave = drive.render_blocks(1);
                assert!(
                    wave.0
                        .iter()
                        .chain(&wave.1)
                        .all(|&s| s > 0.0 && s.is_finite() && s < 1.0)
                );
                waves.push(wave);
                let rows = shared_feedback_rows(&mut drive, 80, 26, Instant::now());
                let applies = description.starts_with("PG on: A1 only")
                    || description.starts_with("PG on: needs Pitch");
                assert_eq!(
                    rows[25].contains(&format!("{description}, manual unchanged")),
                    applies
                );
                let help = drive.app.full_help_keys(&drive.keymap, true);
                assert!(
                    help.iter()
                        .any(|(key, text)| key == "Alt+H" && text.contains(description))
                );
                assert!(
                    !help
                        .iter()
                        .any(|(key, text)| key == "Alt+G" && text.contains("PG"))
                );
                for text in [
                    "Only A1 notes: linked slices with pitch above zero.",
                    "Manual hits and previews stay unchanged.",
                    "Cut at the next grid line, even an empty step.",
                    "Quantize Off uses 1/16.",
                ] {
                    assert!(help.iter().any(|(_, row)| row.contains(text)), "{help:?}");
                }
                assert!(
                    help.iter()
                        .any(|(key, text)| key == "Recorded pads" && text.contains("100"))
                );
                drive.tap(KeyCode::F(1), M::NONE);
                assert!(drive.app.help_open);
                drive.tap(KeyCode::F(1), M::NONE);
                assert!(!drive.app.help_open);
            }
            assert_eq!(
                waves[0], waves[1],
                "toggle leaves the existing manual voice unchanged"
            );
            drive.pad_release(source, 0);
        }
    }
}

fn raw_pattern() -> session::PatternState {
    session::PatternState {
        slot: "A1".into(),
        bars: 1,
        quantize: "1/16".into(),
        strength: 50,
        hits: vec![
            [0, 0, 100, 960],
            [121, 127, 80, 240],
            [5000, 0, 120, 0],
            [1900, 18, 90, 480],
            [6000, 127, 90, 240],
        ],
    }
}

fn swap_routes(drive: &mut Drive, source: Source) {
    for (number, action) in [
        (110, "pad.take"),
        (111, "edit.undo"),
        (112, "bank.8"),
        (113, "ui.back"),
        (114, "bank.1"),
    ] {
        learn::assign(
            &mut drive.app.learned,
            "Recorded pads",
            input::Group::Global,
            if matches!(source, Source::Notes) {
                input::midi::Key::Note(number)
            } else {
                input::midi::Key::Cc(number)
            },
            action,
        );
    }
    drive
        .app
        .sync_learned("Recorded pads", input::Group::Global);
    drive.frame();
}

fn pattern_block(drive: &mut Drive, wave: &mut Stereo, drain: bool) {
    let (mut left, mut right) = ([0.0; RENDER_BLOCK], [0.0; RENDER_BLOCK]);
    drive
        .audio_rt
        .as_mut()
        .unwrap()
        .process(&mut left, &mut right);
    wave.0.extend(left);
    wave.1.extend(right);
    if drain {
        drive.app.audio.as_mut().unwrap().free_garbage();
    }
}

fn fill_pattern_queue(drive: &mut Drive, retirement: bool, wave: &mut Stereo) {
    let epoch = drive
        .project
        .epoch
        .load(std::sync::atomic::Ordering::Acquire);
    let dummy = drive.app.pads.sound[127].clone();
    let audio = drive.app.audio.as_mut().unwrap();
    assert_eq!(audio.commands.slots(), 1024);
    for _ in 0..1024 {
        audio
            .commands
            .push(ProjectCommand {
                epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(if retirement {
                    Command::SetPadSound {
                        pad: 127,
                        sound: Some(dummy.as_ref().unwrap().clone()),
                    }
                } else {
                    Command::SetPadGain {
                        pad: 127,
                        gain: 1.0,
                    }
                }),
            })
            .unwrap();
    }
    assert_eq!(audio.commands.slots(), 0);
    if retirement {
        // Drain commands but retain all 1024 retired references outside the callback.
        for _ in 0..4 {
            pattern_block(drive, wave, false);
        }
        assert_eq!(drive.app.audio.as_mut().unwrap().commands.slots(), 1024);
    }
}

/// Scenario "Playing pattern swap and undo cannot split sound from hits".
#[test]
fn playing_pattern_swap_and_undo_cannot_split_sound_from_hits() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        for retirement in [false, true] {
            let mut drive = Drive::offline();
            swap_routes(&mut drive, source);
            let raw = session::PatternState {
                slot: "A1".into(),
                bars: 1,
                quantize: "off".into(),
                strength: 100,
                hits: vec![
                    [0, 0, 127, 100],
                    [60, 0, 127, 240],
                    [160, 1, 127, 0],
                    [240, 0, 127, 120],
                    [0, 18, 127, 480],
                    [5000, 0, 90, 100],
                ],
            };
            drive.app.apply_session(session::Session {
                pattern: Some("A1".into()),
                patterns: vec![raw.clone()],
                ..Default::default()
            });
            for (pad, value, length, gate) in [
                (0, 0.02, 48_000, true),
                (1, 0.04, 64, false),
                (18, 0.005, 48_000, true),
                (127, 0.001, 1, false),
            ] {
                let sound = Arc::new(Sample::mono(vec![value; length]));
                drive.load_pad(pad, sound.clone(), "pattern-pressure");
                drive.app.pads.gate[pad] = gate;
                drive.app.pads.pitch[pad] = 0;
                drive.app.send(Command::SetPadSound {
                    pad,
                    sound: Some(sound),
                });
                drive.app.send(Command::SetPadMode {
                    pad,
                    mode: if gate {
                        engine::PadMode::Gate
                    } else {
                        engine::PadMode::OneShot
                    },
                });
            }
            settle_pattern(&mut drive);
            let before = drive.app.before();
            let sound = drive.app.pads.sound[0].as_ref().unwrap().clone();
            let other = drive.app.pads.sound[18].as_ref().unwrap().clone();
            let epoch = drive
                .project
                .epoch
                .load(std::sync::atomic::Ordering::Acquire);
            drive.tap(KeyCode::Char(' '), M::NONE);
            let mut wave = (Vec::new(), Vec::new());
            pattern_block(&mut drive, &mut wave, true);
            let mut boundaries = Vec::new();
            for undo in [false, true] {
                fill_pattern_queue(&mut drive, retirement, &mut wave);
                if undo {
                    drive.registry_command(source, KeyCode::Char('z'), M::CONTROL, 111);
                    assert!(drive.app.history.is_empty());
                    assert_eq!(drive.app.pads.pattern.as_ref(), Some(&raw));
                    for (pad, snapshot) in before.pads.iter().enumerate() {
                        assert_eq!(&drive.app.pads.snapshot(pad), snapshot);
                    }
                } else {
                    drive.registry_command(source, KeyCode::Char('x'), M::CONTROL, 110);
                    drive.pad_press(source, 1);
                    drive.pad_release(source, 1);
                    assert_eq!(drive.app.history.len(), 1);
                    let mut remapped = raw.clone();
                    for row in &mut remapped.hits {
                        row[1] = match row[1] {
                            0 => 1,
                            1 => 0,
                            pad => pad,
                        };
                    }
                    assert_eq!(drive.app.pads.pattern.as_ref(), Some(&remapped));
                }
                // A full command queue must retain the complete transaction in UI pending work.
                assert_eq!(drive.app.pending_audio.is_empty(), retirement);
                let old_pad = usize::from(undo);
                for _ in 0..4 {
                    drive.app.flush_audio();
                    pattern_block(&mut drive, &mut wave, false);
                    let audio = drive.app.audio.as_ref().unwrap();
                    assert!(
                        audio
                            .stats
                            .pad_position(&sound, epoch, |pad| pad == old_pad)
                            .is_some()
                    );
                    assert!(
                        audio
                            .stats
                            .pad_position(&sound, epoch, |pad| pad == 1 - old_pad)
                            .is_none()
                    );
                    assert_eq!(
                        audio.stats.pad_position(&other, epoch, |pad| pad == 18),
                        Some(wave.0.len())
                    );
                    assert_eq!(drive.app.transport_state().dropped_hits, 0);
                }
                drive.app.audio.as_mut().unwrap().free_garbage();
                drive.app.flush_audio();
                let boundary = wave.0.len();
                boundaries.push(boundary);
                pattern_block(&mut drive, &mut wave, true);
                let audio = drive.app.audio.as_ref().unwrap();
                assert!(
                    audio
                        .stats
                        .pad_position(&sound, epoch, |pad| pad == 1 - old_pad)
                        .is_some()
                );
                assert!(
                    audio
                        .stats
                        .pad_position(&sound, epoch, |pad| pad == old_pad)
                        .is_none()
                );
                assert_eq!(
                    audio.stats.pad_position(&other, epoch, |pad| pad == 18),
                    Some(wave.0.len())
                );
                if !undo {
                    while wave.0.len() < 2816 {
                        pattern_block(&mut drive, &mut wave, true);
                    }
                }
            }
            while wave.0.len() < 14_080 {
                pattern_block(&mut drive, &mut wave, true);
            }
            assert!(drive.app.pending_audio.is_empty());
            assert_eq!(drive.app.transport_state().dropped_hits, 0);
            let envelope = |frame: usize, onset: usize, cut: usize| -> f32 {
                if frame < onset {
                    0.0
                } else if frame < cut {
                    1.0
                } else {
                    (240usize.saturating_sub(frame - cut)) as f32 / 240.0
                }
            };
            for channel in [&wave.0, &wave.1] {
                for (frame, &sample) in channel.iter().enumerate() {
                    let expected = 0.02
                        * (envelope(frame, 0, boundaries[0])
                            + envelope(frame, 1500, boundaries[1])
                            + envelope(frame, 6000, 9000))
                        + 0.005 * envelope(frame, 0, 12_000)
                        + if (4000..4064).contains(&frame) {
                            0.04
                        } else {
                            0.0
                        };
                    assert!(sample.is_finite() && sample.abs() < 1.0);
                    assert!(
                        (sample - expected).abs() <= 0.000001,
                        "{source:?}, retirement={retirement}, frame {frame}: {sample} != {expected}, accepted {boundaries:?}"
                    );
                }
            }
        }
    }
}

/// Scenario "Pattern hits follow swapped sounds across banks".
#[test]
fn pattern_hits_follow_swapped_sounds_across_banks() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        let mut drive = Drive::new();
        swap_routes(&mut drive, source);
        drive.load_pad(0, Arc::new(Sample::mono(vec![0.05; 4096])), "first");
        drive.load_pad(127, Arc::new(Sample::mono(vec![0.02; 8000])), "last");
        drive.app.pads.pitch_gate[127] = false;
        drive.app.pads.pattern = Some(raw_pattern());
        let before = drive.app.before();
        let raw = drive.app.pads.pattern.clone().unwrap();
        drive.registry_command(source, KeyCode::Char('x'), M::CONTROL, 110);
        drive.registry_command(source, KeyCode::Char('8'), M::CONTROL, 112);
        let frame = drive.pad_press(source, 15);
        drive.pad_release(source, 15);
        let mut expected = raw.clone();
        for row in &mut expected.hits {
            row[1] = match row[1] {
                0 => 127,
                127 => 0,
                other => other,
            };
        }
        assert_eq!(drive.app.pads.pattern.as_ref(), Some(&expected));
        assert_eq!(drive.app.pads.snapshot(0), before.pads[127]);
        assert_eq!(drive.app.pads.snapshot(127), before.pads[0]);
        assert_eq!(drive.app.pads.snapshot(18), before.pads[18]);
        assert_eq!(drive.app.history.len(), 1);
        assert!(
            matches!(frame.commands().as_slice(), [Command::SwapPatternPads { a: 0, b: 127, pattern: Some(pattern) }] if **pattern == expected.checked().unwrap().compile())
        );
        let undo = drive.registry_command(source, KeyCode::Char('z'), M::CONTROL, 111);
        assert!(
            matches!(undo.commands().as_slice(), [Command::SwapPatternPads { a: 0, b: 127, pattern: Some(pattern) }] if **pattern == raw.checked().unwrap().compile())
        );
        assert_eq!(drive.app.pads.pattern.as_ref(), Some(&raw));
        for (pad, snapshot) in before.pads.iter().enumerate() {
            assert_eq!(&drive.app.pads.snapshot(pad), snapshot);
        }
        assert!(drive.app.history.is_empty());
    }
}

/// Scenario "Cancelled pattern swaps leave raw hits unchanged".
#[test]
fn cancelled_pattern_swaps_leave_raw_hits_unchanged() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        for refusal in ["cancel", "self", "loading source", "loading target"] {
            let mut drive = Drive::new();
            swap_routes(&mut drive, source);
            drive.load_pad(0, Arc::new(Sample::mono(vec![0.05; 4096])), "first");
            drive.app.pads.pattern = Some(raw_pattern());
            if refusal == "loading source" {
                drive.app.pads.loading[0] = Some(50);
            }
            if refusal == "loading target" {
                drive.app.pads.loading[1] = Some(50);
            }
            let before = drive.app.before();
            let raw = drive.app.pads.pattern.clone();
            let pending = drive.app.pending_audio.len();
            let dirty = drive.app.session_dirty;
            drive.registry_command(source, KeyCode::Char('x'), M::CONTROL, 110);
            let frame = if refusal == "cancel" {
                drive.registry_command(source, KeyCode::Esc, M::NONE, 113)
            } else {
                let pad = usize::from(refusal != "self");
                let frame = drive.pad_press(source, pad);
                drive.pad_release(source, pad);
                frame
            };
            assert!(!frame.commands().iter().any(|command| matches!(
                command,
                Command::SwapPatternPads { .. } | Command::SwapPads { .. }
            )));
            assert_eq!(drive.app.pads.pattern, raw);
            for (pad, snapshot) in before.pads.iter().enumerate() {
                assert_eq!(&drive.app.pads.snapshot(pad), snapshot);
            }
            assert_eq!(drive.app.pending_audio.len(), pending);
            assert_eq!(drive.app.session_dirty, dirty);
            assert!(drive.app.history.is_empty());
        }
    }
}

/// Scenario "Pattern swap remains after save and reopen".
#[test]
fn pattern_swap_remains_after_save_and_reopen() {
    for project in [false, true] {
        let files = TransitionFiles::new();
        let mut drive = files.drive(project);
        drive.app.pads.pattern = Some(raw_pattern());
        drive.app.pads.pitch_gate[0] = false;
        drive.tap(KeyCode::Char('x'), M::CONTROL);
        drive.tap(KeyCode::Char('8'), M::CONTROL);
        drive.pad_press(Source::Keyboard, 15);
        drive.pad_release(Source::Keyboard, 15);
        let expected = drive.app.pads.to_session(None, drive.app.rate);
        drive.app.save_session().unwrap();
        let stored = if project {
            with_resolved_paths(
                session::load_existing(&files.0.join("A/project.toml")).unwrap(),
                Some(&files.0.join("A")),
            )
        } else {
            session::load(&drive.app.session_path).unwrap()
        };
        let mut reopened = pads::Pads::default();
        reopened.restore(&stored, drive.app.rate);
        assert_eq!(reopened.pattern, drive.app.pads.pattern);
        assert_eq!(reopened.pitch_gate, drive.app.pads.pitch_gate);
        assert_eq!(
            reopened.to_session(None, drive.app.rate).pads,
            expected.pads
        );
    }
}

fn toggle_key(drive: &mut Drive, letter: char, kitty: bool, kind: KeyEventKind) {
    drive.app.keyboard_event(
        &drive.keymap,
        &KeyEvent::new_with_kind(KeyCode::Char(letter), M::ALT, kind),
        kitty,
        &mut drive.down,
        &mut drive.held,
        &mut drive.mods,
    );
}

fn bind_pg(drive: &mut Drive, group: input::Group, key: input::midi::Key) {
    learn::assign(&mut drive.app.learned, "Recorded pads", group, key, PG);
    drive.app.sync_learned("Recorded pads", group);
    drive.frame();
}

/// Scenario "Pitch Gate keyboard toggle works in both layouts".
#[test]
fn pitch_gate_keyboard_toggle_works_in_both_layouts() {
    for kitty in [false, true] {
        for editing in [false, true] {
            for letter in ['g', 'п'] {
                let mut drive = Drive::new();
                drive.app.edit = editing;
                drive.app.pads.marked.extend([0, 17, 127]);
                drive.app.pads.pitch_gate[17] = false;
                let before = drive.app.before();
                toggle_key(&mut drive, letter, kitty, KeyEventKind::Press);
                let frame = drive.frame();
                assert_eq!(frame.ui.len(), 3);
                assert!(frame.ui.iter().all(|command| matches!(
                    command,
                    Command::SetPadPitchGate { enabled: false, .. }
                )));
                for pad in 0..PADS {
                    assert_eq!(drive.app.pads.pitch_gate[pad], ![0, 17, 127].contains(&pad));
                }
                assert_eq!(drive.app.history.len(), 1);
                assert_eq!(shown_value(&drive).parameter, "Pitch Gate");
                assert_eq!(shown_value(&drive).formatted, "PG off");
                assert!(shown_value(&drive).position.is_none());
                for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
                    toggle_key(&mut drive, letter, kitty, kind);
                    assert!(drive.frame().commands().is_empty());
                    assert_eq!(drive.app.history.len(), 1);
                }
                toggle_key(&mut drive, letter, kitty, KeyEventKind::Press);
                toggle_key(&mut drive, letter, kitty, KeyEventKind::Release);
                drive.frame();
                assert_eq!(drive.app.history.len(), 2, "distinct presses do not join");
                drive.tap(KeyCode::Char('z'), M::CONTROL);
                assert!(
                    [0, 17, 127]
                        .into_iter()
                        .all(|pad| !drive.app.pads.pitch_gate[pad])
                );
                drive.tap(KeyCode::Char('z'), M::CONTROL);
                for (pad, snapshot) in before.pads.iter().enumerate() {
                    assert_eq!(&drive.app.pads.snapshot(pad), snapshot);
                }
                assert_eq!(drive.app.edit, editing);
                assert_eq!(drive.app.pads.marked, [0, 17, 127].into());
            }
        }
    }
}

/// Scenario "Raw MIDI Pitch Gate button matches the keyboard".
#[test]
fn raw_midi_pitch_gate_button_matches_the_keyboard() {
    for editing in [false, true] {
        for key in [input::midi::Key::Cc(100), input::midi::Key::Note(100)] {
            let mut keyboard = Drive::new();
            let mut controller = Drive::new();
            for drive in [&mut keyboard, &mut controller] {
                drive.app.edit = editing;
                drive.app.pads.marked.extend([0, 18, 127]);
                drive.app.pads.pitch_gate[18] = false;
            }
            bind_pg(&mut controller, input::Group::Pads, key);
            let before = controller.app.before();
            keyboard.tap(KeyCode::Char('g'), M::ALT);
            let (press, release) = match key {
                input::midi::Key::Cc(cc) => ([0xb0, cc, 127], [0xb0, cc, 0]),
                input::midi::Key::Note(note) => ([0x90, note, 100], [0x80, note, 0]),
            };
            controller.bytes(PAD_PORT, &press);
            let frame = controller.frame();
            assert!(frame.direct.is_empty());
            assert_eq!(frame.ui.len(), 3);
            controller.bytes(PAD_PORT, &release);
            assert!(controller.frame().commands().is_empty());
            assert_eq!(controller.app.pads.pitch_gate, keyboard.app.pads.pitch_gate);
            assert_eq!(controller.app.history.len(), 1);
            assert_eq!(shown_value(&controller), shown_value(&keyboard));
            assert_eq!(controller.app.pads.bank, 0);
            assert_eq!(controller.app.pads.selected, 0);
            assert!(controller.app.pads.held.iter().all(|held| !held));
            assert!(controller.app.pads.hit_at.iter().all(Option::is_none));
            controller.tap(KeyCode::Char('z'), M::CONTROL);
            for (pad, snapshot) in before.pads.iter().enumerate() {
                assert_eq!(&controller.app.pads.snapshot(pad), snapshot);
            }
        }
    }
}

/// Scenario "Pitch Gate binding survives restart without a controller reset".
#[test]
fn pitch_gate_binding_survives_restart_without_a_controller_reset() {
    let mut drive = Drive::new();
    bind_pg(&mut drive, input::Group::Pads, input::midi::Key::Cc(100));
    bind_pg(
        &mut drive,
        input::Group::PadsEdit,
        input::midi::Key::Note(101),
    );
    let original = drive.app.learned.clone();
    let seeded = drive.app.seeded.clone();
    let path = drive.save_registry_bindings();
    let mut reopened = Drive::with_saved_bindings(false, Some(&path));
    assert_eq!(reopened.app.learned, original);
    assert_eq!(reopened.app.seeded, seeded);
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        std::fs::read_to_string(drive.save_registry_bindings()).unwrap()
    );
    for (editing, bytes, release) in [
        (false, [0xb0, 100, 127], [0xb0, 100, 0]),
        (true, [0x90, 101, 100], [0x80, 101, 0]),
    ] {
        reopened.app.edit = editing;
        reopened.frame();
        let before = reopened.app.pads.pitch_gate[0];
        reopened.bytes(PAD_PORT, &bytes);
        reopened.bytes(PAD_PORT, &release);
        reopened.frame();
        assert_eq!(reopened.app.pads.pitch_gate[0], !before);
    }
    reopened.app.edit = false;
    reopened.frame();
    let before = reopened.app.pads.pitch_gate;
    reopened.bytes(PAD_PORT, &[0x90, 101, 100]);
    reopened.bytes(PAD_PORT, &[0x80, 101, 0]);
    reopened.frame();
    assert_eq!(
        reopened.app.pads.pitch_gate, before,
        "edit binding stays in its layer"
    );
    assert_eq!(reopened.app.learned, original);
}

/// Scenario "Pitch Gate scope reset copy and undo preserve pad state".
#[test]
fn pitch_gate_scope_reset_copy_and_undo_preserve_pad_state() {
    for source in [Source::Keyboard, Source::Button, Source::Notes] {
        let mut drive = Drive::new();
        for (number, action) in [
            (100, PG),
            (101, "pad.copy"),
            (102, "pad.clear"),
            (103, "pad.edit"),
            (104, "edit.undo"),
        ] {
            learn::assign(
                &mut drive.app.learned,
                "Recorded pads",
                input::Group::Global,
                if matches!(source, Source::Notes) {
                    input::midi::Key::Note(number)
                } else {
                    input::midi::Key::Cc(number)
                },
                action,
            );
        }
        drive
            .app
            .sync_learned("Recorded pads", input::Group::Global);
        drive.load_pad(0, Arc::new(Sample::mono(vec![0.05; 4096])), "source");
        let source_state = drive.app.pads.snapshot(0);
        drive.registry_command(source, KeyCode::Char('g'), M::ALT, 100);
        assert!(!drive.app.pads.pitch_gate[0]);
        let copied = drive.app.pads.snapshot(0);
        drive.registry_command(source, KeyCode::Char('c'), M::CONTROL, 101);
        drive.pad_press(source, 1);
        drive.pad_release(source, 1);
        assert!(!drive.app.pads.pitch_gate[1]);
        assert!(Arc::ptr_eq(
            drive.app.pads.sound[0].as_ref().unwrap(),
            drive.app.pads.sound[1].as_ref().unwrap()
        ));
        drive.registry_command(source, KeyCode::Char('z'), M::CONTROL, 104);
        assert!(drive.app.pads.pitch_gate[1]);
        assert!(drive.app.pads.file[1].is_none());
        assert_eq!(drive.app.pads.snapshot(0), copied);
        drive.app.pads.selected = 0;
        drive.registry_command(source, KeyCode::Enter, M::NONE, 103);
        drive.registry_command(source, KeyCode::Backspace, M::NONE, 102);
        assert!(matches!(drive.app.ask, Some(Ask::ResetPads { .. })));
        assert!(!drive.app.pads.pitch_gate[0]);
        drive.registry_command(source, KeyCode::Enter, M::NONE, 103);
        assert!(drive.app.pads.pitch_gate[0]);
        assert_eq!(drive.app.pads.file[0], source_state.file);
        drive.registry_command(source, KeyCode::Char('z'), M::CONTROL, 104);
        assert_eq!(drive.app.pads.snapshot(0), copied);
        drive.app.edit = false;
        drive.registry_command(source, KeyCode::Backspace, M::NONE, 102);
        assert!(drive.app.pads.file[0].is_none());
        assert!(!drive.app.pads.pitch_gate[0]);
        drive.registry_command(source, KeyCode::Char('z'), M::CONTROL, 104);
        assert_eq!(drive.app.pads.snapshot(0), copied);
    }
}

/// Scenario "Pitch Gate input cannot bypass overlays or owner keys".
#[test]
fn pitch_gate_input_cannot_bypass_overlays_or_owner_keys() {
    for overlay in [
        "name",
        "question",
        "project picker",
        "folder picker",
        "screen learn",
        "filter",
    ] {
        let mut drive = Drive::new();
        match overlay {
            "name" => drive.app.pad_naming = Some((0, String::new())),
            "question" => {
                drive.app.ask = Some(Ask::ClearPads {
                    pads: vec![0],
                    lines: vec!["Clear?".into()],
                })
            }
            "project picker" => {
                drive.app.project_picker = Some(ProjectPicker::new(Path::new(
                    "target/absent-pattern-projects",
                )))
            }
            "folder picker" => {
                drive.app.path_picker = Some(PathPicker::open(0, Path::new("target")))
            }
            "screen learn" => {
                drive.tap(KeyCode::F(10), M::NONE);
            }
            _ => {
                drive.app.screen = Screen::Files;
                drive.app.browser.editing = true;
            }
        }
        let before = drive.app.pads.pitch_gate;
        drive.tap(KeyCode::Char('g'), M::ALT);
        assert_eq!(drive.app.pads.pitch_gate, before, "{overlay}");
        assert!(drive.app.history.is_empty(), "{overlay}");
    }
    let files = TransitionFiles::new();
    let path = files.0.join("keymap.toml");
    for section in ["screens.pads", "edit"] {
        for action in ["none", "pad.mode_next"] {
            let text = format!("[{section}]\n\"Alt+G\" = \"{action}\"\n");
            std::fs::write(&path, &text).unwrap();
            let (map, notice, warnings) = load_keymap(&path, DEFAULT_KEYMAP);
            assert!(notice.is_none());
            assert!(
                warnings
                    .iter()
                    .any(|warning| warning
                        == "keymap.toml: Alt+G on pads hides pad.pitch_gate_toggle")
            );
            assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
            let mut drive = Drive::new();
            drive.keymap = map;
            drive.app.edit = section == "edit";
            drive.tap(KeyCode::Char('g'), M::ALT);
            assert!(drive.app.pads.pitch_gate.iter().all(|enabled| *enabled));
            assert_eq!(drive.app.pads.gate[0], action == "pad.mode_next");
        }
    }
}

/// Scenario "Pitch Gate help and screen learn follow effective keys".
#[test]
fn pitch_gate_help_and_screen_learn_follow_effective_keys() {
    for editing in [false, true] {
        let mut drive = Drive::new();
        drive.app.edit = editing;
        drive
            .keymap
            .overlay(
                "[screens.pads]\n\"Alt+G\" = \"none\"\n\"Alt+H\" = \"pad.pitch_gate_toggle\"\n",
            )
            .unwrap();
        let full = full_help(&drive.keymap, Screen::Pads, editing);
        assert!(
            full.iter()
                .any(|(keys, text)| keys == "Alt+H" && text.contains("Pitch Gate"))
        );
        assert!(
            !full
                .iter()
                .any(|(keys, text)| keys.contains("Alt+G") && text.contains("Pitch Gate"))
        );
        if editing {
            let groups = help_groups(&drive.keymap, Screen::Pads, &drive.app.help_state());
            assert_eq!(
                groups.iter().map(|group| group.name).collect::<Vec<_>>(),
                ["EDIT"]
            );
        }
        drive.tap(KeyCode::F(10), M::NONE);
        drive.tap(KeyCode::Char('g'), M::ALT);
        assert!(drive.app.learn_picked().is_none());
        drive.tap(KeyCode::Char('h'), M::ALT);
        assert_eq!(drive.app.learn_picked().unwrap().0, PG);
        assert!(drive.app.pads.pitch_gate.iter().all(|enabled| *enabled));
        assert!(drive.app.history.is_empty());
    }
    let mut drive = Drive::new();
    drive.app.edit = true;
    drive
        .keymap
        .overlay("[edit]\n\"Alt+G\" = \"none\"\n")
        .unwrap();
    drive.tap(KeyCode::F(10), M::NONE);
    drive.tap(KeyCode::Char('g'), M::ALT);
    assert!(
        drive.app.learn_picked().is_none(),
        "learn must not select a hidden screen default"
    );
}

/// Scenario "Pitch Gate learning retains confirmation and control types".
#[test]
fn pitch_gate_learning_retains_confirmation_and_control_types() {
    let mut drive = Drive::new();
    drive.tap(KeyCode::F(10), M::NONE);
    drive.tap(KeyCode::Char('g'), M::ALT);
    let original = drive.app.learned.clone();
    drive.bytes(XS_PORT, &[0xb0, 24, 64]);
    drive.frame();
    assert!(drive.app.screen_learn.as_ref().unwrap().refused);
    assert_eq!(drive.app.learned, original);
    for key in [100, 101] {
        drive.bytes(PAD_PORT, &[0x90, key, 100]);
        drive.bytes(PAD_PORT, &[0x80, key, 0]);
        drive.frame();
        assert_eq!(
            drive.app.screen_learn.as_ref().unwrap().pending,
            Some(("Recorded pads".into(), input::midi::Key::Note(key)))
        );
        assert_eq!(drive.app.learned, original);
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.screen_learn.as_ref().unwrap().pending.is_none());
    }
    drive.bytes(PAD_PORT, &[0x90, 100, 100]);
    drive.bytes(PAD_PORT, &[0x80, 100, 0]);
    drive.frame();
    drive.tap(KeyCode::Enter, M::NONE);
    assert_eq!(drive.app.learned["Recorded pads"]["pads"]["note 100"], PG);
    drive.tap(KeyCode::Char('g'), M::ALT);
    drive.tap(KeyCode::Backspace, M::NONE);
    assert!(drive.app.screen_learn.as_ref().unwrap().clearing);
    assert_eq!(drive.app.learned["Recorded pads"]["pads"]["note 100"], PG);
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(
        !drive.app.learned["Recorded pads"]["pads"]
            .values()
            .any(|name| name == PG)
    );
    drive.tap(KeyCode::F(10), M::NONE);
    drive.tap(KeyCode::F(12), M::NONE);
    drive.app.settings_tab = 2;
    drive.app.midi_open.push(input::Group::Pads);
    drive.app.midi_row = drive
        .app
        .midi_list()
        .iter()
        .position(|row| matches!(row, MidiListRow::Action { info } if info.name == PG))
        .unwrap();
    drive.tap(KeyCode::Enter, M::NONE);
    drive.bytes(PAD_PORT, &[0xb0, 100, 127]);
    drive.bytes(PAD_PORT, &[0xb0, 100, 0]);
    drive.frame();
    assert!(drive.app.learning.as_ref().unwrap().pending.is_some());
    assert!(
        !drive.app.learned["Recorded pads"]["pads"]
            .values()
            .any(|name| name == PG)
    );
    drive.tap(KeyCode::Enter, M::NONE);
    assert_eq!(drive.app.learned["Recorded pads"]["pads"]["cc 100"], PG);
    assert!(drive.app.pads.pitch_gate.iter().all(|enabled| *enabled));
    assert!(drive.app.history.is_empty());
}

/// Hits pad 0 at tick 60 of a running pass: 1500 frames at 120 BPM, inside the 1/16 step
/// at zero. Returns the frames rendered since transport start.
fn replay_step_zero(drive: &mut Drive, source: Source) -> usize {
    app_capture_start(drive, 1);
    app_capture_frames(drive, 1500);
    drive.pad_press(source, 0);
    app_capture_frames(drive, 128);
    drive.pad_release(source, 0);
    app_capture_frames(drive, 128);
    drive.app.flush_audio();
    app_capture_frames(drive, 0);
    1500 + 256
}

fn replacement_drive(hits: Vec<[u32; 4]>, quantize: &str) -> Drive {
    let mut drive = Drive::offline();
    drive.app.apply_session(session::Session {
        pattern: Some("A1".into()),
        patterns: vec![session::PatternState {
            slot: "A1".into(),
            bars: 1,
            quantize: quantize.into(),
            strength: 100,
            hits,
        }],
        ..Default::default()
    });
    settle_pattern(&mut drive);
    drive
}

/// Scenario "A replayed step keeps only the newest hit".
#[test]
fn a_replayed_step_keeps_only_the_newest_hit() {
    let tone = app_capture_tone(55.0);
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = app_capture_drive(vec![[0, 0, 64, 0]]);
        app_capture_sound(&mut drive, 0, tone.clone(), false);
        app_capture_frames(&mut drive, 0);
        let elapsed = replay_step_zero(&mut drive, source);
        let rows = drive.app.pads.pattern.as_ref().unwrap().hits.clone();
        assert_eq!(
            rows.len(),
            2,
            "{source:?}: no replacement while the pass records"
        );
        let fresh = rows[1];
        assert_eq!((fresh[1], fresh[2]), (0, 127), "{source:?}");
        app_capture_finish(&mut drive, 1);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits, [fresh]);
        assert_eq!(drive.app.pads.pattern_rows.as_ref().unwrap().len(), 1);
        // the next loop plays the new row once, as a pattern of only that row does
        app_capture_frames(&mut drive, 96_000 - elapsed);
        let lap = app_capture_frames(&mut drive, 96_000);
        let mut reference = app_capture_drive(vec![fresh]);
        app_capture_sound(&mut reference, 0, tone.clone(), false);
        app_capture_frames(&mut reference, 0);
        reference.app.send(Command::TransportPlay);
        let expected = app_capture_frames(&mut reference, 96_000);
        assert!(
            expected.0.iter().any(|v| v.abs() > 0.001),
            "the reference plays"
        );
        for (got, want) in [(&lap.0, &expected.0), (&lap.1, &expected.1)] {
            for (frame, (a, b)) in got.iter().zip(want).enumerate() {
                assert!(
                    (a - b).abs() <= 1e-6,
                    "{source:?} frame {frame}: {a} != {b}"
                );
            }
        }
    }
}

/// Scenario "Replacement keeps other steps, pads and hidden rows".
#[test]
fn replacement_keeps_other_steps_pads_and_hidden_rows() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for quantize in ["1/16", "off"] {
            // pad 1 at the same step, pad 0 one step later, a hidden pad 0 row beyond the
            // one-bar loop, two saved pad 2 rows on one tick and a pad 0 row at zero
            let old = vec![
                [0, 1, 64, 0],
                [480, 0, 64, 0],
                [4000, 0, 64, 0],
                [0, 2, 64, 0],
                [0, 2, 90, 0],
                [0, 0, 64, 0],
            ];
            let mut drive = replacement_drive(old.clone(), quantize);
            assert_eq!(
                drive.app.pads.pattern.as_ref().unwrap().hits,
                old,
                "load keeps saved same-tick rows"
            );
            replay_step_zero(&mut drive, source);
            let fresh = *drive
                .app
                .pads
                .pattern
                .as_ref()
                .unwrap()
                .hits
                .last()
                .unwrap();
            app_capture_finish(&mut drive, 1);
            let mut expected = old.clone();
            if quantize == "1/16" {
                // only the pad 0 row on the replayed step goes
                expected.retain(|row| *row != [0, 0, 64, 0]);
            }
            expected.push(fresh);
            assert_eq!(
                drive.app.pads.pattern.as_ref().unwrap().hits,
                expected,
                "{source:?} {quantize}"
            );
        }
    }
}

/// Scenario "Undo restores replaced hits".
#[test]
fn undo_restores_replaced_hits() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        // two saved pad 0 rows on one saved tick: a step holds saved ticks since 98c
        let old = vec![[0, 0, 64, 0], [0, 0, 90, 0], [480, 1, 70, 0]];
        let mut drive = app_capture_drive(old.clone());
        pattern_undo_routes(&mut drive, source);
        replay_step_zero(&mut drive, source);
        let fresh = *drive
            .app
            .pads
            .pattern
            .as_ref()
            .unwrap()
            .hits
            .last()
            .unwrap();
        app_capture_finish(&mut drive, 1);
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            [[480, 1, 70, 0], fresh],
            "{source:?}: both saved pad 0 rows on the replayed step go"
        );
        global_pattern_undo(&mut drive, source);
        drive.app.flush_audio();
        app_capture_frames(&mut drive, 0);
        assert_eq!(
            drive.app.pads.pattern.as_ref().unwrap().hits,
            old,
            "{source:?}"
        );
        assert_eq!(
            drive.app.pads.pattern_rows.as_ref().unwrap().len(),
            old.len()
        );
    }
}

fn pattern_items(drive: &Drive) -> Vec<(String, String)> {
    drive
        .app
        .pattern_help_groups(
            &drive.keymap,
            help_groups(&drive.keymap, drive.app.screen, &drive.app.help_state()),
            drive.app.pattern_record_state(),
            true,
        )
        .into_iter()
        .flat_map(|group| group.items)
        .collect()
}

/// Scenario "Count-in and idle help name hidden rules".
#[test]
fn count_in_and_idle_help_name_hidden_rules() {
    for unbound in [false, true] {
        let mut drive = app_capture_drive(vec![[0, 0, 64, 0]]);
        drive.app.goto(Screen::Pads);
        if unbound {
            drive
                .keymap
                .overlay("[global]\n\"Alt+Z\" = \"none\"\n")
                .unwrap();
        }
        let items = pattern_items(&drive);
        let undo = items.iter().find(|(_, text)| text == "pass undo");
        if unbound {
            assert!(undo.is_none(), "a none binding has no fallback: {items:?}");
        } else {
            assert_eq!(undo.unwrap().0, "Alt Z", "{items:?}");
        }
        assert!(items.iter().any(|(_, text)| text == "record A1"));
        drive.app.apply(Action::PatternRecord);
        app_capture_frames(&mut drive, 0);
        assert_eq!(
            app_capture_snapshot(&drive).phase,
            engine::capture::Phase::CountIn
        );
        let items = pattern_items(&drive);
        assert!(
            items
                .iter()
                .any(|(keys, text)| keys == "pads" && text == "not recorded yet"),
            "{items:?}"
        );
    }
}

/// Scenario "A completed pass reports new and replaced hits".
#[test]
fn a_completed_pass_reports_new_and_replaced_hits() {
    for source in [Source::Keyboard, Source::Notes] {
        let mut drive = app_capture_drive(vec![[0, 0, 64, 0]]);
        drive.app.session_write_failed = true;
        app_capture_start(&mut drive, 1);
        // pad 1 on the step of the saved row, then pads 2 and 3 on later steps
        app_capture_frames(&mut drive, 1500);
        for pad in [0, 1, 2] {
            drive.pad_press(source, pad);
            app_capture_frames(&mut drive, 3000);
            drive.pad_release(source, pad);
            app_capture_frames(&mut drive, 3000);
        }
        drive.app.flush_audio();
        app_capture_frames(&mut drive, 0);
        app_capture_finish(&mut drive, 1);
        let done = drive
            .app
            .notices
            .history()
            .iter()
            .find(|notice| notice.level == Level::Done)
            .expect("a Done notice")
            .clone();
        assert_eq!(done.text, "A1 pass: 3 new hits, 1 replaced.", "{source:?}");
        // the save warning keeps its place
        assert_eq!(drive.app.save_status(), tui::SaveStatus::Failed);
        app_capture_start(&mut drive, 2);
        app_capture_frames(&mut drive, 3000);
        app_capture_finish(&mut drive, 2);
        let done = drive
            .app
            .notices
            .history()
            .iter()
            .find(|notice| notice.level == Level::Done)
            .unwrap()
            .clone();
        assert_eq!(done.text, "A1 pass: no new hits.", "{source:?}");
        assert_eq!(drive.app.save_status(), tui::SaveStatus::Failed);
    }
}

fn tidy_routes(drive: &mut Drive, source: Source) {
    swap_routes(drive, source);
    for (number, action) in [(115, "pattern.erase"), (116, "pattern.clear")] {
        learn::assign(
            &mut drive.app.learned,
            "Recorded pads",
            input::Group::Global,
            if matches!(source, Source::Notes) {
                input::midi::Key::Note(number)
            } else {
                input::midi::Key::Cc(number)
            },
            action,
        );
    }
    drive
        .app
        .sync_learned("Recorded pads", input::Group::Global);
    drive.frame();
}

fn tidy_hits(drive: &Drive) -> Vec<[u32; 4]> {
    drive.app.pads.pattern.as_ref().unwrap().hits.clone()
}

/// Alt+E or the learned erase button.
fn tidy_erase(drive: &mut Drive, source: Source) -> Frame {
    drive.registry_command(source, KeyCode::Char('e'), M::ALT, 115)
}

/// A pad press and release through the source.
fn tidy_pad(drive: &mut Drive, source: Source, pad: usize) -> Frame {
    let mut frame = drive.pad_press(source, pad);
    let release = drive.pad_release(source, pad);
    frame.direct.extend(release.direct);
    frame.ui.extend(release.ui);
    frame
}

/// Clear A1 by its pattern-control row and Enter (keyboard) or the learned button.
fn tidy_clear(drive: &mut Drive, source: Source) -> Frame {
    if matches!(source, Source::Keyboard) {
        drive.tap(KeyCode::Char('n'), M::ALT);
        for _ in 0..3 {
            drive.tap(KeyCode::Down, M::NONE);
        }
        assert_eq!(
            drive.app.pattern_panel,
            Some(crate::pattern_record::CLEAR_ROW)
        );
        drive.tap(KeyCode::Enter, M::NONE)
    } else {
        drive.registry_command(source, KeyCode::Null, M::NONE, 116)
    }
}

/// A pad press through the source, heard in the offline output: direct MIDI hits reach the
/// engine without a UI command, so only the sound proves them.
fn tidy_audible(drive: &mut Drive, source: Source, pad: usize) -> bool {
    // an earlier hit of the 9600-frame sound ends first
    app_capture_frames(drive, 12_288);
    tidy_pad(drive, source, pad);
    let (left, _) = app_capture_frames(drive, 4_096);
    engine::checks::peak(&left) > 0.01
}

fn tidy_sound(drive: &mut Drive, pad: usize) {
    app_capture_sound(drive, pad, Arc::new(Sample::mono(vec![0.3; 9_600])), false);
}

fn sounds(frame: &Frame, pad: usize) -> bool {
    frame.commands().iter().any(|command| {
        matches!(command, Command::PadPress { pad: p, .. } | Command::PatternPadPress { pad: p, .. } if *p == pad)
    })
}

fn published_pads(frame: &Frame) -> Option<Vec<usize>> {
    frame
        .commands()
        .iter()
        .rev()
        .find_map(|command| match command {
            Command::PublishRecordedPattern { pattern } => {
                Some(pattern.events().iter().map(|hit| hit.pad()).collect())
            }
            _ => None,
        })
}

const SOURCES: [Source; 3] = [Source::Keyboard, Source::Notes, Source::Button];

/// Scenario "Erase one whole pad while stopped".
#[test]
fn erase_one_whole_pad_while_stopped() {
    for source in SOURCES {
        // pad Q (abs 4) twice in the loop and once beyond it; pad W (abs 5) once
        let rows = vec![
            [0, 4, 90, 0],
            [960, 4, 90, 0],
            [4000, 4, 90, 0],
            [240, 5, 80, 0],
        ];
        let mut drive = app_capture_drive(rows.clone());
        drive.app.goto(Screen::Pads);
        tidy_sound(&mut drive, 4);
        tidy_routes(&mut drive, source);
        assert!(tidy_audible(&mut drive, source, 4), "{source:?}: control");
        tidy_erase(&mut drive, source);
        assert!(drive.app.erase_pads, "{source:?}");
        assert!(drive.app.focus().mode.erase_pads);
        let history = drive.app.history.len();
        assert!(
            !tidy_audible(&mut drive, source, 4),
            "{source:?}: a silent target"
        );
        assert_eq!(tidy_hits(&drive), [[240, 5, 80, 0]], "{source:?}");
        assert_eq!(drive.app.history.len(), history + 1);
        drive.tap(KeyCode::Char('z'), M::CONTROL);
        assert_eq!(tidy_hits(&drive), rows, "{source:?}: one Ctrl+Z restores");
        // a pad without rows changes nothing
        let history = drive.app.history.len();
        tidy_pad(&mut drive, source, 2);
        assert_eq!(tidy_hits(&drive), rows);
        assert_eq!(drive.app.history.len(), history);
        assert_eq!(drive.last_notice().level, Level::Info);
        assert!(
            drive.app.erase_pads,
            "the mode stays until the owner leaves it"
        );
    }
}

/// Scenario "Erasure needs hits in A1".
#[test]
fn erasure_needs_hits_in_a1() {
    for source in SOURCES {
        for absent in [false, true] {
            let mut drive = if absent {
                Drive::offline()
            } else {
                app_capture_drive(Vec::new())
            };
            drive.app.goto(Screen::Pads);
            tidy_routes(&mut drive, source);
            tidy_erase(&mut drive, source);
            assert!(!drive.app.erase_pads, "{source:?} absent {absent}");
            assert_eq!(drive.last_notice().level, Level::Info);
            assert_eq!(drive.last_notice().text, "A1 has no hits to erase.");
        }
    }
}

/// Scenario "Erase one whole pad during playback".
#[test]
fn erase_one_whole_pad_during_playback() {
    use engine::checks::{first_non_finite, max_step, peak};
    // tones with a soft end: their own release makes no click
    let tone = |hz: f32| {
        let len = 24_000;
        let wave: Vec<f32> = (0..len)
            .map(|i| {
                let fade = ((len - i) as f32 / 2400.0).min(1.0);
                0.4 * fade * (std::f32::consts::TAU * hz * i as f32 / 48_000.0).sin()
            })
            .collect();
        Arc::new(Sample::mono(wave))
    };
    let half = engine::pattern::TICKS_PER_BAR / 2;
    // pad Q low on beat 1, pad W high on beat 3
    let mut drive = app_capture_drive(vec![[0, 4, 127, 0], [half, 5, 127, 0]]);
    drive.app.goto(Screen::Pads);
    app_capture_sound(&mut drive, 4, tone(220.0), false);
    app_capture_sound(&mut drive, 5, tone(660.0), false);
    drive.app.send(Command::TransportPlay);
    let bar = 96_000;
    // one whole loop, then the erasure while the second Q voice sounds
    let (mut left, mut right) = app_capture_frames(&mut drive, bar + 4_096);
    drive.tap(KeyCode::Char('e'), M::ALT);
    let frame = tidy_pad(&mut drive, Source::Keyboard, 4);
    assert_eq!(published_pads(&frame), Some(vec![5]));
    let (l, r) = app_capture_frames(&mut drive, 2 * bar - 4_096);
    left.extend(l);
    right.extend(r);
    for channel in [&left, &right] {
        assert_eq!(first_non_finite(channel), None);
        assert!(peak(channel) < 1.0, "no clipping");
        assert!(max_step(channel) < 0.05, "no click: {}", max_step(channel));
    }
    let q = |start: usize| peak(&left[start..start + 4_000]);
    let w = |start: usize| peak(&left[start + bar / 2..start + bar / 2 + 4_000]);
    assert!(q(0) > 0.3 && w(0) > 0.3, "the first loop plays Q and W");
    assert!(q(bar) > 0.3, "the sounding Q voice keeps its release");
    assert!(q(2 * bar) < 1e-4, "the next loop plays no Q");
    assert!(w(bar) > 0.3 && w(2 * bar) > 0.3, "W keeps playing");
    let epoch = drive
        .project
        .epoch
        .load(std::sync::atomic::Ordering::Acquire);
    let transport = drive
        .app
        .audio
        .as_ref()
        .unwrap()
        .stats
        .transport(epoch)
        .unwrap();
    assert!(transport.playing, "erasure keeps transport");
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/98b-erase-pad-playback.wav");
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for (&l, &r) in left.iter().zip(&right) {
        writer.write_sample(l).unwrap();
        writer.write_sample(r).unwrap();
    }
    writer.finalize().unwrap();
}

/// Scenario "Starting a pass leaves whole-pad erasure".
#[test]
fn starting_a_pass_leaves_whole_pad_erasure() {
    let rows = vec![[0, 4, 90, 0]];
    let mut drive = app_capture_drive(rows.clone());
    drive.app.goto(Screen::Pads);
    drive.tap(KeyCode::Char('e'), M::ALT);
    assert!(drive.app.erase_pads);
    drive.tap(KeyCode::Char('r'), M::CONTROL);
    app_capture_frames(&mut drive, 0);
    assert!(!drive.app.erase_pads);
    assert!(!drive.app.focus().mode.erase_pads);
    assert_eq!(
        app_capture_snapshot(&drive).phase,
        engine::capture::Phase::CountIn
    );
    assert_eq!(tidy_hits(&drive), rows, "the start erases no row");
    let frame = tidy_pad(&mut drive, Source::Keyboard, 5);
    assert!(sounds(&frame, 5), "pads in the pass record hits");
}

/// Scenario "A pass end does not leave whole-pad erasure on".
#[test]
fn a_pass_end_does_not_leave_whole_pad_erasure_on() {
    let mut drive = app_capture_drive(vec![[0, 4, 90, 0]]);
    drive.app.goto(Screen::Pads);
    app_capture_start(&mut drive, 1);
    drive.tap(KeyCode::Char('e'), M::ALT);
    app_capture_frames(&mut drive, 0);
    assert!(drive.app.pattern_erasing(), "held erasure in the pass");
    assert!(!drive.app.erase_pads);
    app_capture_finish(&mut drive, 1);
    assert!(!drive.app.pattern_erasing());
    assert!(!drive.app.erase_pads);
    let rows = tidy_hits(&drive);
    let frame = tidy_pad(&mut drive, Source::Keyboard, 4);
    assert!(sounds(&frame, 4), "the next press sounds");
    assert_eq!(tidy_hits(&drive), rows, "and erases no row");
}

/// Scenario "Leaving Pads leaves whole-pad erasure".
#[test]
fn leaving_pads_leaves_whole_pad_erasure() {
    let rows = vec![[0, 4, 90, 0]];
    let mut drive = app_capture_drive(rows.clone());
    drive.load_pad(4, Arc::new(Sample::mono(vec![0.1; 4800])), "q");
    drive.app.goto(Screen::Pads);
    drive.tap(KeyCode::Char('e'), M::ALT);
    assert!(drive.app.erase_pads);
    drive.tap(KeyCode::F(4), M::NONE);
    assert_eq!(drive.app.screen, Screen::Fx);
    assert!(!drive.app.erase_pads);
    drive.tap(KeyCode::F(2), M::NONE);
    assert!(!drive.app.erase_pads);
    let frame = tidy_pad(&mut drive, Source::Keyboard, 4);
    assert!(sounds(&frame, 4));
    assert_eq!(tidy_hits(&drive), rows);
}

/// Scenario "Clear A1 asks first".
#[test]
fn clear_a1_asks_first() {
    for source in SOURCES {
        let rows = vec![[0, 4, 90, 0], [240, 5, 80, 0]];
        let mut drive = app_capture_drive(rows.clone());
        drive.app.goto(Screen::Pads);
        tidy_routes(&mut drive, source);
        let settings = |drive: &Drive| {
            let p = drive.app.pads.pattern.as_ref().unwrap();
            (p.bars, p.quantize.clone(), p.strength)
        };
        let before = settings(&drive);
        tidy_clear(&mut drive, source);
        assert!(
            matches!(drive.app.ask, Some(Ask::ClearPattern { .. })),
            "{source:?}"
        );
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.ask.is_none());
        assert_eq!(tidy_hits(&drive), rows, "{source:?}: keep changes nothing");
        drive.app.pattern_panel = None;
        tidy_clear(&mut drive, source);
        let frame = drive.tap(KeyCode::Enter, M::NONE);
        assert!(tidy_hits(&drive).is_empty(), "{source:?}");
        assert_eq!(settings(&drive), before);
        assert_eq!(published_pads(&frame), Some(vec![]));
        assert_eq!(drive.last_notice().text, "A1 cleared: 2 hits.");
        drive.app.pattern_panel = None;
        drive.tap(KeyCode::Char('z'), M::CONTROL);
        assert_eq!(tidy_hits(&drive), rows, "{source:?}: one Ctrl+Z restores");
    }
}

/// Scenario "Clearing during a pass refuses".
#[test]
fn clearing_during_a_pass_refuses() {
    for source in SOURCES {
        let rows = vec![[0, 4, 90, 0]];
        let mut drive = app_capture_drive(rows.clone());
        drive.app.goto(Screen::Pads);
        tidy_routes(&mut drive, source);
        app_capture_start(&mut drive, 1);
        let history = drive.app.history.len();
        tidy_clear(&mut drive, source);
        assert!(drive.app.ask.is_none(), "{source:?}");
        assert_eq!(tidy_hits(&drive), rows, "{source:?}");
        assert_eq!(drive.app.history.len(), history);
        assert_eq!(
            drive.last_notice().text,
            "A1 edit blocked: end the recording pass first."
        );
        drive.app.pattern_panel = None;
        app_capture_finish(&mut drive, 1);
    }
}

/// Scenario "Enter runs Clear A1 from pattern controls".
#[test]
fn enter_runs_clear_a1_from_pattern_controls() {
    let rows = vec![[0, 4, 90, 0]];
    let mut drive = app_capture_drive(rows.clone());
    drive.app.goto(Screen::Pads);
    drive.tap(KeyCode::Char('n'), M::ALT);
    for _ in 0..3 {
        drive.tap(KeyCode::Down, M::NONE);
    }
    let bars = drive.app.pads.pattern.as_ref().unwrap().bars;
    drive.tap(KeyCode::Char('l'), M::NONE);
    assert!(drive.app.ask.is_none(), "L changes nothing on Clear A1");
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, bars);
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(matches!(drive.app.ask, Some(Ask::ClearPattern { .. })));
    assert_eq!(tidy_hits(&drive), rows, "the question does not clear");
    drive.tap(KeyCode::Esc, M::NONE);
    drive.app.pattern_panel = Some(0);
    drive.tap(KeyCode::Enter, M::NONE);
    assert_eq!(
        drive.app.pattern_panel, None,
        "Enter on Bars closes controls"
    );
    assert!(drive.app.ask.is_none());
}

/// Scenario "Esc leaves erasure first".
#[test]
fn esc_leaves_erasure_first() {
    let mut drive = app_capture_drive(vec![[0, 4, 90, 0]]);
    drive.app.goto(Screen::Pads);
    app_capture_start(&mut drive, 1);
    drive.tap(KeyCode::Char('e'), M::ALT);
    app_capture_frames(&mut drive, 0);
    assert!(drive.app.pattern_erasing());
    drive.tap(KeyCode::Esc, M::NONE);
    app_capture_frames(&mut drive, 0);
    assert!(!drive.app.pattern_erasing(), "the first Esc leaves erasure");
    assert_eq!(
        app_capture_snapshot(&drive).phase,
        engine::capture::Phase::Recording
    );
    drive.tap(KeyCode::Esc, M::NONE);
    app_capture_frames(&mut drive, 0);
    assert_ne!(
        app_capture_snapshot(&drive).phase,
        engine::capture::Phase::Recording,
        "the second Esc finishes Recording"
    );
    drive.app.flush_audio();
    app_capture_frames(&mut drive, 0);
    // whole-pad erasure: Esc leaves it and the screen stays
    drive.tap(KeyCode::Char('e'), M::ALT);
    assert!(drive.app.erase_pads);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(!drive.app.erase_pads);
    assert_eq!(drive.app.screen, Screen::Pads);
}

/// Scenario "Learned erase and clear buttons match the keys".
#[test]
fn learned_erase_and_clear_buttons_match_the_keys() {
    let run = |source: Source| {
        let rows = vec![
            [0, 4, 90, 0],
            [0, 4, 70, 0],
            [240, 5, 80, 0],
            [480, 6, 60, 0],
        ];
        let mut drive = app_capture_drive(rows);
        drive.app.goto(Screen::Pads);
        tidy_sound(&mut drive, 4);
        tidy_routes(&mut drive, source);
        tidy_erase(&mut drive, source);
        let silent = !tidy_audible(&mut drive, source, 4);
        let erased = tidy_hits(&drive);
        tidy_erase(&mut drive, source);
        tidy_clear(&mut drive, source);
        let asked = matches!(drive.app.ask, Some(Ask::ClearPattern { .. }));
        drive.tap(KeyCode::Enter, M::NONE);
        drive.app.pattern_panel = None;
        (
            silent,
            erased,
            asked,
            tidy_hits(&drive),
            drive.app.history.len(),
        )
    };
    let keyboard = run(Source::Keyboard);
    assert!(keyboard.0 && keyboard.2, "{keyboard:?}");
    assert_eq!(keyboard.1, [[240, 5, 80, 0], [480, 6, 60, 0]]);
    assert!(keyboard.3.is_empty());
    assert_eq!(keyboard.4, 2, "each result is one Undo step");
    for source in [Source::Notes, Source::Button] {
        assert_eq!(run(source), keyboard, "{source:?}");
    }
}

/// Scenario "Pattern tidy results name their counts".
#[test]
fn pattern_tidy_results_name_their_counts() {
    let mut drive = app_capture_drive(vec![[0, 4, 90, 0], [0, 4, 70, 0], [240, 5, 80, 0]]);
    drive.load_pad(5, Arc::new(Sample::mono(vec![0.1; 4800])), "snare");
    drive.app.goto(Screen::Pads);
    drive.tap(KeyCode::Char('e'), M::ALT);
    tidy_pad(&mut drive, Source::Keyboard, 5);
    assert_eq!(drive.last_notice().level, Level::Done);
    assert_eq!(
        drive.last_notice().text,
        "A1: pad W snare erased, 1 hit. Ctrl+Z restores."
    );
    tidy_pad(&mut drive, Source::Keyboard, 5);
    assert_eq!(drive.last_notice().level, Level::Info);
    assert_eq!(drive.last_notice().text, "A1: pad W snare has no hits.");
    drive.tap(KeyCode::Esc, M::NONE);
    drive.tap(KeyCode::Char('n'), M::ALT);
    let view = drive.app.pattern_controls_view(&drive.keymap).unwrap();
    assert_eq!(view.values[3], ("clear A1".into(), "2 hits".into()));
    assert!(!view.dim[3]);
    drive.app.pattern_panel = None;
    tidy_clear(&mut drive, Source::Keyboard);
    drive.tap(KeyCode::Enter, M::NONE);
    assert_eq!(drive.last_notice().level, Level::Done);
    assert_eq!(drive.last_notice().text, "A1 cleared: 2 hits.");
    drive.app.pattern_panel = Some(0);
    let view = drive.app.pattern_controls_view(&drive.keymap).unwrap();
    assert_eq!(view.values[3], ("clear A1".into(), "no hits".into()));
    assert!(view.dim[3], "an empty A1 shows the row dim");
}

/// Scenario "Erasure help names both forms".
#[test]
fn erasure_help_names_both_forms() {
    let mut drive = app_capture_drive(vec![[0, 4, 90, 0]]);
    drive.app.goto(Screen::Pads);
    drive.tap(KeyCode::Char('e'), M::ALT);
    let items = pattern_items(&drive);
    for item in [
        ("pads", "erase all A1 hits"),
        ("Ctrl Z", "undo"),
        ("Esc/Alt E", "exit"),
    ] {
        assert!(
            items.iter().any(|(k, t)| k == item.0 && t == item.1),
            "{item:?} in {items:?}"
        );
    }
    let f1 = crate::full_help(&drive.keymap, Screen::Pads, true);
    let erase = f1
        .iter()
        .find(|(keys, _)| keys == "Alt+E")
        .expect("F1 names Alt+E");
    assert!(erase.1.contains("under the playhead"), "{erase:?}");
    assert!(erase.1.contains("all its hits"), "{erase:?}");
}

/// Scenario "Erasure outside Pads names Pads".
#[test]
fn erasure_outside_pads_names_pads() {
    let mut drive = app_capture_drive(vec![[0, 4, 90, 0]]);
    drive.app.goto(Screen::Chop);
    drive.tap(KeyCode::Char('e'), M::ALT);
    assert!(!drive.app.erase_pads);
    assert_eq!(drive.last_notice().level, Level::Info);
    assert_eq!(drive.last_notice().text, "Erase pads on Pads (F2).");
}

/// Scenario "Erasure shows what a press erases".
#[test]
fn erasure_shows_what_a_press_erases() {
    for (width, height) in [(80, 26), (120, 40)] {
        // pad Q 4 hits, one hidden beyond the loop; pad W 2 hits; pad E none
        let mut drive = app_capture_drive(vec![
            [0, 4, 90, 0],
            [240, 4, 90, 0],
            [480, 4, 90, 0],
            [4000, 4, 90, 0],
            [0, 5, 90, 0],
            [960, 5, 90, 0],
        ]);
        drive.app.goto(Screen::Pads);
        drive.tap(KeyCode::Char('e'), M::ALT);
        let rows = shared_feedback_rows(&mut drive, width, height, Instant::now());
        let text = rows.join("\n");
        assert!(
            rows[..2].iter().any(|row| row.contains("ERASE")),
            "{width}x{height}: {rows:?}"
        );
        assert!(text.contains("4 hits"), "{width}x{height}:\n{text}");
        assert!(text.contains("2 hits"), "{width}x{height}:\n{text}");
        let counts: Vec<usize> = (0..10)
            .filter(|n| text.contains(&format!(" {n} hits")))
            .collect();
        assert_eq!(counts, [2, 4], "pad E shows no count:\n{text}");
        tidy_pad(&mut drive, Source::Keyboard, 4);
        let text = shared_feedback_rows(&mut drive, width, height, Instant::now()).join("\n");
        assert!(!text.contains("4 hits"), "{width}x{height}:\n{text}");
        drive.tap(KeyCode::Esc, M::NONE);
        let text = shared_feedback_rows(&mut drive, width, height, Instant::now()).join("\n");
        assert!(!text.contains("2 hits"), "{width}x{height}:\n{text}");
        assert!(!text.contains("ERASE"));
    }
}

/// Scenario "Pad commands wait while pattern controls are open".
#[test]
fn pad_commands_wait_while_pattern_controls_are_open() {
    let mut drive = app_capture_drive(vec![[0, 4, 90, 0]]);
    drive.load_pad(4, Arc::new(Sample::mono(vec![0.1; 4800])), "q");
    drive.app.goto(Screen::Pads);
    drive.app.pads.selected = 4;
    let gate = drive.app.pads.gate[4];
    let pitch_gate = drive.app.pads.pitch_gate[4];
    let page = drive.app.pads.param;
    drive.tap(KeyCode::Char('n'), M::ALT);
    drive.tap(KeyCode::Char('h'), M::NONE);
    drive.tap(KeyCode::Char('g'), M::ALT);
    drive.tap(KeyCode::Backspace, M::NONE);
    drive.tap(KeyCode::Tab, M::NONE);
    assert_eq!(drive.app.pads.gate[4], gate, "the pad mode stays");
    assert_eq!(drive.app.pads.pitch_gate[4], pitch_gate);
    assert!(drive.app.pads.file[4].is_some(), "the sound stays");
    assert_eq!(drive.app.pads.param, page, "the panel page stays");
    assert!(drive.app.pattern_panel.is_some());
    assert!(drive.app.ask.is_none());
    let frame = tidy_pad(&mut drive, Source::Keyboard, 4);
    assert!(sounds(&frame, 4), "the pad key plays its pad");
}

/// Scenario "Knobs turn A1 while pattern controls are open".
#[test]
fn knobs_turn_a1_while_pattern_controls_are_open() {
    let mut drive = app_capture_drive(vec![[0, 4, 90, 0]]);
    drive.app.goto(Screen::Pads);
    drive.app.pads.selected = 4;
    tidy_sound(&mut drive, 4);
    let volume = drive.app.pads.volume_db[4];
    tidy_routes(&mut drive, Source::Notes);
    drive.tap(KeyCode::Char('n'), M::ALT);
    value_knob(&mut drive, "edit.param_1", 127);
    value_knob(&mut drive, "edit.param_2", 0);
    value_knob(&mut drive, "edit.param_3", 0);
    value_knob(&mut drive, "pad.volume", 0);
    let pattern = drive.app.pads.pattern.as_ref().unwrap();
    assert_eq!(pattern.bars, 64);
    assert_eq!(pattern.quantize, "off");
    assert_eq!(pattern.strength, 0);
    assert_eq!(drive.app.pads.volume_db[4], volume, "the pad volume stays");
    assert!(
        tidy_audible(&mut drive, Source::Notes, 4),
        "a MIDI pad note plays its pad"
    );
}

/// Scenario "Pattern controls show restrictions in place".
#[test]
fn pattern_controls_show_restrictions_in_place() {
    let mut drive = app_capture_drive(vec![[0, 4, 90, 0]]);
    drive.app.goto(Screen::Pads);
    drive.tap(KeyCode::Char('n'), M::ALT);
    drive.tap(KeyCode::Down, M::NONE);
    drive.tap(KeyCode::Down, M::NONE);
    let view = drive.app.pattern_controls_view(&drive.keymap).unwrap();
    assert_eq!(
        (view.restriction.as_str(), view.refused),
        ("next hits, step 10", false)
    );
    drive.app.pattern_panel = None;
    app_capture_start(&mut drive, 1);
    drive.tap(KeyCode::Char('n'), M::ALT);
    let bars = drive.app.pads.pattern.as_ref().unwrap().bars;
    drive.tap(KeyCode::Right, M::NONE);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().bars, bars);
    let view = drive.app.pattern_controls_view(&drive.keymap).unwrap();
    assert_eq!(view.selected, 0);
    assert_eq!(
        (view.restriction.as_str(), view.refused),
        ("Idle only", true)
    );
    drive.app.pattern_panel = None;
    app_capture_finish(&mut drive, 1);
}

/// One frame of 120 BPM is 1/25 of a tick: 3840 ticks per 96000-frame bar.
const FRAMES_PER_TICK: usize = 25;

/// Records one press and release of a pad with the given modifiers, `at` ticks into a
/// fresh pass; returns the saved rows after the pass.
fn record_presses(drive: &mut Drive, presses: &[(usize, u32, M)]) -> Vec<[u32; 4]> {
    let pass = app_capture_snapshot(drive).pass + 1;
    app_capture_start(drive, pass);
    let mut now = 0;
    for &(pad, at, modifiers) in presses {
        let target = at as usize * FRAMES_PER_TICK;
        app_capture_frames(drive, target - now);
        now = target;
        let key = KeyCode::Char("1234qwerasdfzxcv".chars().nth(pad).unwrap());
        drive.key(key, modifiers, KeyEventKind::Press);
        drive.frame();
        app_capture_frames(drive, 256);
        drive.key(key, modifiers, KeyEventKind::Release);
        drive.frame();
        now += 256;
    }
    app_capture_frames(drive, 512);
    drive.app.flush_audio();
    app_capture_frames(drive, 0);
    app_capture_finish(drive, pass);
    drive.app.send(Command::TransportStop);
    app_capture_frames(drive, 0);
    tidy_hits(drive)
}

/// Sets and publishes the capture quantize: the audio thread places hits with it (98c).
fn set_quantize(drive: &mut Drive, quantize: &str, strength: u8) {
    let pattern = drive.app.pads.pattern.as_mut().unwrap();
    pattern.quantize = quantize.into();
    pattern.strength = strength;
    let command = drive.app.pads.recorded_pattern_command().unwrap();
    drive.app.send(command);
    app_capture_frames(drive, 0);
}

/// Scenario "A saved hit keeps the quantize of its capture".
#[test]
fn a_saved_hit_keeps_the_quantize_of_its_capture() {
    use engine::checks::{first_non_finite, max_step, onsets, peak};
    let tone = |hz: f32| {
        let len = 9_600;
        let wave: Vec<f32> = (0..len)
            .map(|i| {
                let fade = ((len - i) as f32 / 960.0).min(1.0);
                0.4 * fade * (std::f32::consts::TAU * hz * i as f32 / 48_000.0).sin()
            })
            .collect();
        Arc::new(Sample::mono(wave))
    };
    let mut drive = app_capture_drive(Vec::new());
    drive.app.goto(Screen::Pads);
    app_capture_sound(&mut drive, 4, tone(110.0), false);
    app_capture_sound(&mut drive, 5, tone(440.0), false);
    // Quantize 1/4 through pattern controls: from 1/16 two steps down
    drive.tap(KeyCode::Char('n'), M::ALT);
    drive.tap(KeyCode::Down, M::NONE);
    drive.tap(KeyCode::Left, M::NONE);
    drive.tap(KeyCode::Left, M::NONE);
    drive.tap(KeyCode::Esc, M::NONE);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().quantize, "1/4");
    let rows = record_presses(&mut drive, &[(4, 1040, M::SHIFT)]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], 960, "the hit lands on its 1/4 line");
    // Quantize 1/16 does not move it; the next hit lands on a 1/16 line
    drive.tap(KeyCode::Char('n'), M::ALT);
    drive.tap(KeyCode::Down, M::NONE);
    drive.tap(KeyCode::Right, M::NONE);
    drive.tap(KeyCode::Right, M::NONE);
    drive.tap(KeyCode::Esc, M::NONE);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().quantize, "1/16");
    assert_eq!(tidy_hits(&drive)[0][0], 960);
    let rows = record_presses(&mut drive, &[(5, 1700, M::SHIFT)]);
    let snare = rows.iter().find(|row| row[1] == 5).unwrap();
    assert_eq!(snare[0] % 240, 0, "{rows:?}");
    assert_ne!(snare[0] % 960, 0, "{rows:?}: between beats");
    assert_eq!(rows.iter().find(|row| row[1] == 4).unwrap()[0], 960);
    // playback plays the saved ticks
    let compiled = drive
        .app
        .pads
        .pattern
        .as_ref()
        .unwrap()
        .checked()
        .unwrap()
        .compile();
    let mut played: Vec<(u32, usize)> = compiled
        .events()
        .iter()
        .map(|hit| (hit.tick(), hit.pad()))
        .collect();
    played.sort();
    let mut saved: Vec<(u32, usize)> = rows.iter().map(|r| (r[0], r[1] as usize)).collect();
    saved.sort();
    assert_eq!(played, saved);
    // two loops for the owner: the low kick on beat 2, the high snare on its 1/16 line
    let bar = 96_000;
    // the manual hits of the passes ring out first
    app_capture_frames(&mut drive, 12_000);
    drive.app.send(Command::TransportPlay);
    let (left, right) = app_capture_frames(&mut drive, 2 * bar);
    for channel in [&left, &right] {
        assert_eq!(first_non_finite(channel), None);
        assert!(peak(channel) < 1.0, "no clipping");
        assert!(max_step(channel) < 0.1, "no click: {}", max_step(channel));
    }
    let expected: Vec<usize> = (0..2)
        .flat_map(|lap| {
            saved
                .iter()
                .map(move |&(tick, _)| lap * bar + tick as usize * FRAMES_PER_TICK)
        })
        .collect();
    let found = onsets(&left, 1e-3, 4_000);
    assert_eq!(found.len(), expected.len(), "{found:?} {expected:?}");
    for (got, want) in found.iter().zip(&expected) {
        assert!(got.abs_diff(*want) <= 1, "onset {got} for {want}");
    }
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/98c-quantize-on-input.wav");
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for (&l, &r) in left.iter().zip(&right) {
        writer.write_sample(l).unwrap();
        writer.write_sample(r).unwrap();
    }
    writer.finalize().unwrap();
}

/// Scenario "Strength and Off set the captured tick".
#[test]
fn strength_and_off_set_the_captured_tick() {
    use engine::pattern::Grid;
    for (grid, strength, placed) in [
        (Grid::Sixteenth, 0, 1000),
        (Grid::Sixteenth, 50, 980),
        (Grid::Sixteenth, 100, 960),
        (Grid::Off, 100, 1000),
    ] {
        assert_eq!(grid.place(strength, 1000, 3840), placed);
    }
    // the loop end wraps to zero
    assert_eq!(Grid::Quarter.place(100, 3800, 3840), 0);
    // the captured row of each setting, through the real capture path
    for (quantize, strength, placed) in [("1/16", 50, 980), ("off", 100, 1000)] {
        let mut drive = app_capture_drive(Vec::new());
        set_quantize(&mut drive, quantize, strength);
        let rows = record_presses(&mut drive, &[(4, 1000, M::SHIFT)]);
        assert!(
            rows[0][0].abs_diff(placed) <= 6,
            "{quantize} {strength}: {rows:?}"
        );
    }
    // a Gate duration stays as captured: the same held press with 1/16 and with Off
    let gate = |quantize: &str| {
        let mut drive = app_capture_drive(Vec::new());
        app_capture_sound(
            &mut drive,
            4,
            Arc::new(Sample::mono(vec![0.2; 96_000])),
            true,
        );
        set_quantize(&mut drive, quantize, 100);
        let rows = record_presses(&mut drive, &[(4, 1000, M::SHIFT)]);
        assert_eq!(rows.len(), 1, "{quantize}");
        (rows[0][0], rows[0][3])
    };
    let (quantized, quantized_gate) = gate("1/16");
    let (off, off_gate) = gate("off");
    assert_eq!(quantized, 960);
    assert!(off.abs_diff(1000) <= 6, "{off}: within one audio block");
    assert!(quantized_gate > 0);
    assert_eq!(quantized_gate, off_gate, "the Gate duration does not move");
}

/// Scenario "A settings change does not move playback".
#[test]
fn a_settings_change_does_not_move_playback() {
    use engine::checks::{first_non_finite, max_step, onsets, peak};
    let len = 2_400;
    let tone: Vec<f32> = (0..len)
        .map(|i| {
            // a sine from zero needs no fade-in; the fade-out ends the voice softly
            let fade = ((len - i) as f32 / 240.0).min(1.0);
            0.4 * fade * (std::f32::consts::TAU * 440.0 * i as f32 / 48_000.0).sin()
        })
        .collect();
    let ticks = [100u32, 1_000, 2_000, 3_000];
    let mut drive = app_capture_drive(ticks.iter().map(|&t| [t, 4, 127, 0]).collect());
    drive.app.goto(Screen::Pads);
    app_capture_sound(&mut drive, 4, Arc::new(Sample::mono(tone)), false);
    drive.tap(KeyCode::Char('n'), M::ALT);
    drive.tap(KeyCode::Down, M::NONE);
    drive.app.send(Command::TransportPlay);
    let bar = 96_000;
    let (mut left, mut right) = (Vec::new(), Vec::new());
    // every quantize step, then every strength step, one block of changes per loop part
    for step in 0..24 {
        let (l, r) = app_capture_frames(&mut drive, bar / 8);
        left.extend(l);
        right.extend(r);
        if step == 8 {
            drive.tap(KeyCode::Down, M::NONE);
        }
        drive.tap(
            if step % 12 < 6 {
                KeyCode::Right
            } else {
                KeyCode::Left
            },
            M::NONE,
        );
    }
    for channel in [&left, &right] {
        assert_eq!(first_non_finite(channel), None);
        assert!(peak(channel) < 1.0, "no clipping");
        assert!(max_step(channel) < 0.1, "no click: {}", max_step(channel));
    }
    let expected: Vec<usize> = (0..3)
        .flat_map(|lap| ticks.map(|t| lap * bar + t as usize * FRAMES_PER_TICK))
        .collect();
    let found = onsets(&left, 1e-3, 2_000);
    assert_eq!(found.len(), expected.len(), "{found:?}");
    for (got, want) in found.iter().zip(&expected) {
        assert!(got.abs_diff(*want) <= 1, "onset {got} for {want}");
    }
    let raw = drive.app.pads.pattern.as_ref().unwrap();
    assert_eq!(
        raw.hits.iter().map(|h| h[0]).collect::<Vec<_>>(),
        ticks,
        "saved ticks stay"
    );
}

/// Scenario "Two hits in one cell of one pass keep the last".
#[test]
fn two_hits_in_one_cell_of_one_pass_keep_the_last() {
    for (quantize, kept) in [("1/16", 1), ("1/32", 2)] {
        let mut drive = app_capture_drive(Vec::new());
        set_quantize(&mut drive, quantize, 100);
        // a plain press (velocity 80), then a Shift press (127) 50 ticks later
        let rows = record_presses(&mut drive, &[(4, 1000, M::NONE), (4, 1050, M::SHIFT)]);
        assert_eq!(rows.len(), kept, "{quantize}: {rows:?}");
        if kept == 1 {
            assert_eq!(rows[0][0], 960);
            assert_eq!(rows[0][2], 127, "the second hit stays");
        }
    }
}

/// Scenario "Quantize help names the next hits".
#[test]
fn quantize_help_names_the_next_hits() {
    let mut drive = app_capture_drive(vec![[0, 4, 90, 0]]);
    drive.app.goto(Screen::Pads);
    drive.tap(KeyCode::Char('n'), M::ALT);
    drive.tap(KeyCode::Down, M::NONE);
    let view = drive.app.pattern_controls_view(&drive.keymap).unwrap();
    assert_eq!(view.restriction, "next hits");
    drive.tap(KeyCode::Down, M::NONE);
    let view = drive.app.pattern_controls_view(&drive.keymap).unwrap();
    assert_eq!(view.restriction, "next hits, step 10");
    let f1 = crate::full_help(&drive.keymap, Screen::Pads, true);
    assert!(
        f1.iter()
            .any(|(_, text)| text.contains("next hits") && text.contains("not saved hits")),
        "{f1:?}"
    );
}

/// Scenario "Counts and undo name what they mean".
#[test]
fn counts_and_undo_name_what_they_mean() {
    let mut drive = app_capture_drive(vec![[0, 4, 90, 0], [240, 5, 80, 0], [480, 4, 90, 0]]);
    drive.app.goto(Screen::Pads);
    drive.tap(KeyCode::Char('e'), M::ALT);
    let mut pads = pads_view(&mut drive.app.pads, None, None, vec![], None, 48_000);
    drive.app.erase_pad_marks(&mut pads, None);
    assert_eq!(pads.pads[5].note.as_deref(), Some("1 hit"), "pad W");
    assert_eq!(pads.pads[4].note.as_deref(), Some("2 hits"), "pad Q");
    tidy_pad(&mut drive, Source::Keyboard, 5);
    assert!(
        drive.last_notice().text.contains("erased, 1 hit."),
        "{}",
        drive.last_notice().text
    );
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(drive.last_notice().text, "undo: erase pad W on A1");
    drive.tap(KeyCode::Esc, M::NONE);
    tidy_clear(&mut drive, Source::Keyboard);
    drive.tap(KeyCode::Enter, M::NONE);
    assert_eq!(drive.last_notice().text, "A1 cleared: 3 hits.");
    drive.app.pattern_panel = None;
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(drive.last_notice().text, "undo: clear on A1");
}

/// Scenario "Pattern keys follow the effective binding".
#[test]
fn pattern_keys_follow_the_effective_binding() {
    let mut drive = Drive::new();
    let shown = |drive: &Drive| {
        let groups = drive.app.pattern_help_groups(
            &drive.keymap,
            crate::help_groups(&drive.keymap, Screen::Pads, &drive.app.help_state()),
            None,
            true,
        );
        let group = groups.iter().find(|group| group.name == "PATTERN").unwrap();
        group
            .items
            .iter()
            .find(|(_, text)| text == "pattern keys")
            .cloned()
    };
    assert_eq!(shown(&drive), Some(("Alt N".into(), "pattern keys".into())));
    assert!(
        drive
            .app
            .full_help_keys(&drive.keymap, true)
            .iter()
            .any(|(key, text)| key.contains("Alt+N") && text.starts_with("pattern keys"))
    );
    drive
        .keymap
        .overlay("[global]\n'Alt+N' = 'none'\n'Alt+U' = 'pattern.controls'\n")
        .unwrap();
    assert_eq!(shown(&drive), Some(("Alt U".into(), "pattern keys".into())));
    drive
        .keymap
        .overlay("[global]\n'Alt+U' = 'none'\n")
        .unwrap();
    assert_eq!(shown(&drive), None);
    let info = action_info("screen.pattern");
    assert_eq!(
        (info.name, info.label, info.group),
        ("screen.pattern", "pattern", input::Group::Global)
    );
    drive.bind_action(94, "screen.pattern");
    let path = drive.save_registry_bindings();
    let mut restored = Drive::with_saved_bindings(false, Some(&path));
    restored.bytes(PAD_PORT, &[0xb0, 94, 127]);
    restored.bytes(PAD_PORT, &[0xb0, 94, 0]);
    restored.frame();
    assert!(restored.app.pattern_view.overview);
}

/// Scenario "Controller map names pattern in six cells".
#[test]
fn controller_map_names_pattern_in_six_cells() {
    let info = action_info("screen.pattern");
    assert_eq!((info.name, info.label), ("screen.pattern", "pattern"));
    assert_eq!(input::short_label(info.name), "ptrn");
}

/// Scenario "The pad edit help comes first".
#[test]
fn the_pad_edit_help_comes_first() {
    let mut drive = app_capture_drive(vec![[0, 4, 90, 0]]);
    drive.app.goto(Screen::Pads);
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(drive.app.edit);
    let groups = drive.app.pattern_help_groups(
        &drive.keymap,
        crate::help_groups(&drive.keymap, drive.app.screen, &drive.app.help_state()),
        None,
        true,
    );
    let names: Vec<&str> = groups.iter().map(|group| group.name).collect();
    let edit = names
        .iter()
        .position(|n| *n == "EDIT")
        .expect("an edit group");
    let a1 = names
        .iter()
        .position(|n| *n == "PATTERN")
        .expect("a PATTERN group");
    assert!(edit < a1, "{names:?}");
    // At 80 columns A1 drops whole after the edit keys; F1 lists every group.
    let shown = groups.len() - tui::help_dropped(&groups, 80);
    assert!(shown > edit && shown <= a1, "{names:?} shows {shown}");
    let rows = drive
        .app
        .full_help_recording_keys(&drive.keymap, true, None);
    assert!(
        rows.iter().any(|(key, _)| key.contains("Ctrl+R")),
        "{rows:?}"
    );
    // closed, A1 leads again
    drive.tap(KeyCode::Esc, M::NONE);
    let groups = drive.app.pattern_help_groups(
        &drive.keymap,
        crate::help_groups(&drive.keymap, drive.app.screen, &drive.app.help_state()),
        None,
        true,
    );
    assert_eq!(groups[0].name, "PATTERN");
}

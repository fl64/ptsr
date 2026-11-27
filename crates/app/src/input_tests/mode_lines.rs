//! Change 138: mode lines name the mode, the next step and the way out, in the words of the
//! last input device; PAD BANK + P1 leaves the mode first; STOP ends the output recording.

use super::*;
use std::sync::atomic::Ordering::Acquire;

fn tone() -> Arc<Sample> {
    Arc::new(Sample::mono(vec![0.2; 48_000]))
}

fn line(drive: &Drive) -> String {
    drive.app.mode_line(Instant::now()).unwrap_or_default()
}

fn button(drive: &mut Drive, cc: u8) -> Frame {
    drive.bytes(SMC, &[0xB0, cc, 127]);
    drive.bytes(SMC, &[0xB0, cc, 0]);
    drive.frame()
}

const RIGHT: u8 = 26;
const PLAY: u8 = 27;
const STOP: u8 = 28;

/// Renders until the audio thread accepted every queued command.
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
    panic!("the audio thread did not settle");
}

/// The screen as the program draws it, with the SMC-PAD knob strip and its mode line.
fn strip_rows(drive: &mut Drive, width: u16, height: u16) -> Vec<String> {
    strip_rows_at(drive, width, height, Instant::now())
}

fn strip_rows_at(drive: &mut Drive, width: u16, height: u16, now: Instant) -> Vec<String> {
    use ratatui::{Terminal, backend::TestBackend};
    let app = &mut drive.app;
    let mut feedback = app.status_feedback_at(now);
    feedback.knob_strip = app.smc_knob_strip(now);
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
        meters: ([-60.0; 2], [-60.0; 2], None, false),
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
    term.draw(|frame| tui::draw_feedback(frame, &status, &screen, &help, &feedback, None))
        .unwrap();
    term.backend()
        .buffer()
        .content()
        .chunks(width as usize)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect())
        .collect()
}

/// Scenario "Panic instructions fit both terminal sizes".
#[test]
fn panic_instructions_fit_both_terminal_sizes() {
    let mut drive = hold_drive();
    hold(&mut drive, true);
    drive.app.session_write_error = Some("Save refused.".into());
    assert_eq!(
        drive.app.output_stop_words(true),
        "STOP twice · stop and save"
    );
    let now = Instant::now();
    stop_at(&mut drive, SMC, now, 127);
    for (width, height) in [(80, 26), (120, 40)] {
        let rows = strip_rows_at(&mut drive, width, height, now).join("\n");
        assert!(rows.contains("STOP again"), "{rows}");
        assert!(rows.contains("release PAD BANK to cancel"), "{rows}");
        assert!(rows.contains("Save failed"), "{rows}");
    }
    hold(&mut drive, false);
    assert!(drive.app.mode_line(now).is_none());
}

/// A1 with one hit, restored, on the A1 overview.
fn a1_drive() -> Drive {
    let mut drive = reach_drive_from(hold_drive(), panel::View::Pattern, tone());
    drive.app.apply_session(session::Session {
        pattern: Some("A1".into()),
        patterns: vec![session::PatternState {
            slot: "A1".into(),
            bars: 1,
            quantize: "1/16".into(),
            strength: 100,
            hits: vec![[0, 0, 100, 0]],
        }],
        ..Default::default()
    });
    settle(&mut drive);
    drive.app.screen = Screen::Pads;
    drive.app.pattern_view.overview = true;
    drive.frame();
    drive
}

/// An output recording that runs, as the record key starts it.
fn recording(drive: &mut Drive) {
    drive.app.recording = Some((audio::SOURCE_OUTPUT, Instant::now()));
}

/// Scenario "The slice edit shows its close control" (screen-layout).
#[test]
fn the_slice_edit_shows_its_close_control() {
    let mut drive = hold_drive();
    drive.prepare_chop(tone(), 4);
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Chop, "edit slice")),
    );
    hold(&mut drive, false);
    assert!(!drive.app.panel.visible(), "the panel is hidden");
    let line = line(&drive);
    assert!(line.starts_with("EDIT slice 1 · BOUNDS"), "{line}");
    assert!(line.ends_with("PAD BANK + 1 close"), "{line}");
}

/// Scenario "The copy line keeps its source after a bank change" (screen-layout).
#[test]
fn the_copy_line_keeps_its_source_after_a_bank_change() {
    let mut drive = hold_drive();
    drive.load_pad(0, tone(), "kick");
    drive.frame();
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pads, "copy pad")),
    );
    hold(&mut drive, false);
    button(&mut drive, RIGHT);
    assert_eq!(drive.app.pads.bank, 1, "bank 2 is in sight");
    let line = line(&drive);
    assert!(line.starts_with("COPY kick B1/"), "{line}");
    assert!(line.ends_with("PAD BANK + 1 cancel"), "{line}");
}

/// Scenario "Tap tempo confirms the first tap" (screen-layout).
#[test]
fn tap_tempo_confirms_the_first_tap() {
    let mut drive = a1_drive();
    let bpm = drive.app.transport.bpm;
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pattern, "tap tempo")),
    );
    hold(&mut drive, false);
    assert_eq!(line(&drive), "TAP · 1 hit · tap again");
    assert_eq!(bpm, drive.app.transport.bpm, "one tap keeps the tempo");
}

#[test]
fn keyboard_tap_cue_follows_the_current_sequence() {
    let mut drive = Drive::offline();
    drive.tap(KeyCode::F(5), M::NONE);
    let before = drive.app.transport.bpm;
    let tap = |drive: &mut Drive| drive.tap(KeyCode::Char('b'), M::ALT);
    tap(&mut drive);
    let first = drive.app.transport.taps[0];
    let cue = Some("TAP · 1 hit · tap again");
    assert_eq!(drive.app.notice_at(first).as_deref(), cue);
    assert_eq!(drive.app.transport.bpm, before);

    drive.app.transport.tap_input_at = Some(first + Duration::from_millis(100));
    drive.app.apply(Action::TapTempo);
    assert_eq!(drive.app.transport.taps.len(), 1);
    assert_eq!(drive.app.transport.taps[0], first);
    assert_eq!(
        drive
            .app
            .notice_at(first + Duration::from_secs(2))
            .as_deref(),
        cue
    );
    assert_eq!(
        drive.app.notice_at(first + Duration::from_millis(2001)),
        None
    );

    drive.app.transport.taps[0] = Instant::now() - Duration::from_millis(400);
    drive.app.transport.tap_input_at = None;
    tap(&mut drive);
    assert!(drive.app.notice().unwrap().starts_with("Tempo "));
    drive.app.transport.taps[1] = Instant::now() - Duration::from_millis(2100);
    tap(&mut drive);
    assert_eq!(drive.app.frame_notice().as_deref(), cue);
    assert_eq!(drive.app.frame_notice_level, None);

    drive.tap(KeyCode::F(2), M::NONE);
    assert!(drive.app.transport.taps.is_empty());
    assert_ne!(drive.app.notice().as_deref(), cue);
}

/// Scenario "A1 shows the count-in and the recording" (screen-layout).
#[test]
fn a1_shows_the_count_in_and_the_recording() {
    use engine::capture::Phase;
    let mut drive = a1_drive();
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pattern, "record A1")),
    );
    hold(&mut drive, false);
    let mut seen = Vec::new();
    for _ in 0..4096 {
        drive.app.flush_audio();
        drive.render_blocks(8);
        drive.frame();
        match drive.app.pattern_record_state().map(|s| s.phase) {
            Some(Phase::CountIn) | Some(Phase::Recording) => {
                let now = line(&drive);
                if seen.last() != Some(&now) {
                    seen.push(now);
                }
            }
            _ => {}
        }
        if seen.iter().any(|l| l.starts_with("REC A1")) {
            break;
        }
    }
    assert!(drive.app.pattern_view.overview, "A1 stays on the screen");
    assert_eq!(
        seen.first().map(String::as_str),
        Some("COUNT 4"),
        "{seen:?}"
    );
    assert_eq!(
        seen.last().map(String::as_str),
        Some("REC A1 · hit pads · RECORD stop"),
        "{seen:?}"
    );
}

/// Scenario "Back cancels count-in on A1" (patterns).
/// Scenario "P1 cancels count-in before panel reveal" (command-panel).
#[test]
fn back_cancels_count_in_on_a1() {
    use super::super::patterns::app_capture_frames;
    use engine::capture::Phase;

    for (hold_setting, hold_ready) in [
        (None, true),
        (Some(false), true),
        (Some(true), true),
        (Some(true), false),
    ] {
        let mut drive = a1_drive();
        if let Some(enabled) = hold_setting {
            drive.app.midi_gestures.smc_pad_bank_opens_panel = enabled;
            drive.midi().set_pad_bank_shortcut(enabled);
        }
        if !hold_ready {
            drive.midi().recorded_smc_failure();
            drive.frame();
        }
        let hold_panel = hold_setting.map(|enabled| enabled && hold_ready);
        let back = |drive: &mut Drive| match hold_panel {
            None => drive.tap(KeyCode::Esc, M::NONE),
            Some(hold_panel) => {
                if hold_panel {
                    hold(drive, true);
                }
                let frame = cell(drive, 52);
                if hold_panel {
                    hold(drive, false);
                }
                frame
            }
        };
        drive.tap(KeyCode::Char('o'), M::NONE);
        assert_eq!(drive.app.pads.bank, 1);
        let sound = tone();
        drive.load_pad(16, sound.clone(), "warm-up");
        drive.app.send(Command::SetPadSound {
            pad: 16,
            sound: Some(sound),
        });
        drive.frame();
        drive.tap(KeyCode::Char('r'), M::CONTROL);
        app_capture_frames(&mut drive, 0);
        assert_eq!(
            drive.app.pattern_record_state().unwrap().phase,
            Phase::CountIn
        );

        if hold_panel.is_some() {
            app_capture_frames(&mut drive, 6000);
            let quiet = app_capture_frames(&mut drive, 128).0;
            assert_eq!(engine::checks::peak(&quiet), 0.0);

            drive.bytes(SMC, &[0x99, 48, 100]);
            drive.frame();
            let warm_up = app_capture_frames(&mut drive, 128).0;
            assert!(engine::checks::peak(&warm_up) > 0.0, "{hold_panel:?}");
            drive.bytes(SMC, &[0x89, 48, 0]);
            drive.frame();
            assert_eq!(
                drive.app.pattern_record_state().unwrap().phase,
                Phase::CountIn
            );
        }
        let cue = match hold_panel {
            None => "Esc cancel count",
            Some(false) => "P1 cancel count",
            Some(true) => "PAD BANK+P1 cancel count",
        };
        for width in [80, 120] {
            assert_eq!(
                drive
                    .app
                    .pattern_overview_at(&drive.keymap, width)
                    .next
                    .as_deref(),
                Some(cue)
            );
        }
        let selected = drive.app.pads.selected_abs();
        let before = drive.app.pads.pattern.as_ref().unwrap().clone();

        drive.tap(KeyCode::F(1), M::NONE);
        assert!(drive.app.help_open);
        back(&mut drive);
        assert!(!drive.app.help_open);
        app_capture_frames(&mut drive, 0);
        assert_eq!(
            drive.app.pattern_record_state().unwrap().phase,
            Phase::CountIn
        );

        drive.app.ask = Some(Ask::ClearPattern { lines: Vec::new() });
        back(&mut drive);
        assert!(drive.app.ask.is_none());
        app_capture_frames(&mut drive, 0);
        assert_eq!(
            drive.app.pattern_record_state().unwrap().phase,
            Phase::CountIn
        );

        assert!(!drive.app.panel.visible());
        back(&mut drive);
        assert!(!drive.app.panel.visible());
        assert!(drive.app.pattern_view.overview);
        app_capture_frames(&mut drive, 0);
        drive.frame();
        assert_eq!(drive.app.pattern_record_state().unwrap().phase, Phase::Idle);
        assert!(!drive.app.transport_state().playing);
        assert_eq!(drive.app.pads.bank, 1);
        assert_eq!(drive.app.pads.selected_abs(), selected);
        assert_eq!(drive.app.pads.pattern.as_ref(), Some(&before));
        assert!(drive.app.history.is_empty());
    }
}

/// Scenario "Step edit names its next step and close control" (screen-layout).
#[test]
fn step_edit_names_its_next_step_and_close_control() {
    let mut drive = a1_drive();
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pattern, "edit steps")),
    );
    hold(&mut drive, false);
    assert!(drive.app.pattern_view.edit);
    let line = line(&drive);
    assert!(line.starts_with("STEP "), "{line}");
    assert!(line.contains("hit a pad to add or remove"), "{line}");
    assert!(line.ends_with("PAD BANK + 1 close"), "{line}");
}

/// Scenario "A1 setup names its knobs" (screen-layout).
#[test]
fn a1_setup_names_its_knobs() {
    let mut drive = a1_drive();
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pattern, "A1 setup")),
    );
    hold(&mut drive, false);
    assert!(drive.app.pattern_panel.is_some());
    assert_eq!(
        line(&drive),
        "A1 SETUP · E1 row · E3 quantize · E4 strength · E5 bars · PAD BANK + 1 close"
    );
}

/// Scenario "A1 tells the owner how to close the current mode".
#[test]
fn a1_tells_the_owner_how_to_close_the_current_mode() {
    for (hold_panel, ready) in [(true, true), (false, true), (true, false)] {
        let mut drive = a1_drive();
        drive.app.midi_gestures.smc_pad_bank_opens_panel = hold_panel;
        drive.midi().set_pad_bank_shortcut(hold_panel);
        if !ready {
            drive.midi().recorded_smc_failure();
            drive.frame();
        }
        for setup in [true, false] {
            if setup {
                drive.tap(KeyCode::Char('n'), M::ALT);
                assert!(drive.app.pattern_panel.is_some());
            } else {
                drive.tap(KeyCode::F(7), M::NONE);
                assert!(drive.app.pattern_view.edit);
            }
            button(&mut drive, RIGHT);
            let held = hold_panel && ready;
            let expected = if held {
                "PAD BANK + 1 close"
            } else {
                "Open panel, P1 close"
            };
            let shown = line(&drive);
            assert!(shown.ends_with(expected), "{shown}");
            if !setup {
                assert!(shown.contains("hit a pad to add or remove"), "{shown}");
            }
            if held {
                hold(&mut drive, true);
            } else {
                cell(&mut drive, 52);
                assert!(drive.app.panel.visible());
                assert!(line(&drive).ends_with(" · P1 close"));
            }
            cell(&mut drive, 52);
            hold(&mut drive, false);
            drive.app.panel.hide();
            assert!(drive.app.pattern_panel.is_none());
            assert!(!drive.app.pattern_view.edit);
            assert!(drive.app.pattern_view.overview);
        }
    }
}

/// Scenario "Chop names slice pages before an assign" (screen-layout).
#[test]
fn chop_names_slice_pages_before_an_assign() {
    let mut drive = hold_drive();
    drive.prepare_chop(tone(), 20);
    let strip = drive.app.smc_knob_strip(Instant::now()).unwrap();
    assert_eq!(strip.list, "Slice page");
    drive.link_chop_pad(0, 0);
    drive.frame();
    let strip = drive.app.smc_knob_strip(Instant::now()).unwrap();
    assert_eq!(strip.list, "Bank", "after an assign the arrows turn banks");
}

/// Scenario "Pad edit keeps its close control at 80 by 26" (screen-layout).
#[test]
fn pad_edit_keeps_its_close_control_at_80_by_26() {
    let mut drive = hold_drive();
    drive.load_pad(0, tone(), "kick");
    drive.frame();
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pads, "edit pad")),
    );
    hold(&mut drive, false);
    assert!(drive.app.edit);
    for _ in 0..5 {
        let rows = strip_rows(&mut drive, 80, 26);
        let page = drive.app.pads.page().0;
        assert!(
            rows.iter().any(|row| row.contains("PAD BANK + 1 close")),
            "{page}: {}",
            rows.join("\n")
        );
        button(&mut drive, RIGHT);
    }
}

/// Scenario "P1 stops the A1 erase and stays on A1" (command-panel).
#[test]
fn p1_stops_the_a1_erase_and_stays_on_a1() {
    let mut drive = a1_drive();
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pattern, "erase hits")),
    );
    hold(&mut drive, false);
    assert!(drive.app.erase_pads);
    assert!(line(&drive).starts_with("ERASE A1"), "{}", line(&drive));
    assert!(line(&drive).ends_with("PAD BANK + 1 off"));
    hold(&mut drive, true);
    cell(&mut drive, 52); // device pad 1: back
    hold(&mut drive, false);
    assert!(!drive.app.erase_pads, "erase is off");
    assert!(drive.app.pattern_view.overview, "A1 stays on the screen");
    assert_eq!(drive.last_notice().text, "erase off");
}

/// Scenario "One P1 ends erasure and keeps the screen" (patterns).
#[test]
fn one_p1_ends_erasure_and_keeps_the_screen() {
    let mut drive = a1_drive();
    drive.tap(KeyCode::Char('e'), M::ALT);
    assert!(drive.app.erase_pads);
    hold(&mut drive, true);
    cell(&mut drive, 52);
    hold(&mut drive, false);
    assert!(!drive.app.erase_pads);
    assert_eq!(drive.app.screen, Screen::Pads);
    assert!(drive.app.pattern_view.overview);
    assert_eq!(drive.last_notice().text, "erase off");
    let hits = drive.app.pads.pattern.as_ref().unwrap().hits.len();
    drive.bytes(SMC, &[0x99, 36, 100]);
    drive.bytes(SMC, &[0x89, 36, 0]);
    drive.frame();
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits.len(),
        hits,
        "the next press erases no row"
    );
}

/// Scenario "Erasure in a pass shows its state" (patterns).
#[test]
fn erasure_in_a_pass_shows_its_state() {
    use engine::capture::Phase;
    let mut drive = a1_drive();
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pattern, "record A1")),
    );
    hold(&mut drive, false);
    for _ in 0..4096 {
        drive.app.flush_audio();
        drive.render_blocks(8);
        drive.frame();
        if drive
            .app
            .pattern_record_state()
            .is_some_and(|s| s.phase == Phase::Recording)
        {
            break;
        }
    }
    let erase = panel_note(slot_of(panel::View::Pattern, "erase hits"));
    hold(&mut drive, true);
    cell(&mut drive, erase);
    hold(&mut drive, false);
    settle(&mut drive);
    assert!(drive.app.pattern_erasing());
    let shown = line(&drive);
    assert!(shown.starts_with("ERASE A1"), "{shown}");
    assert!(shown.ends_with("PAD BANK + 1 off"), "{shown}");
    hold(&mut drive, true);
    cell(&mut drive, erase);
    hold(&mut drive, false);
    settle(&mut drive);
    assert!(!drive.app.pattern_erasing());
    assert!(!line(&drive).starts_with("ERASE"), "{}", line(&drive));
}

/// Scenario "P1 cancels a copy and stays on Pads" (command-panel).
#[test]
fn p1_cancels_a_copy_and_stays_on_pads() {
    let mut drive = hold_drive();
    drive.load_pad(0, tone(), "kick");
    drive.frame();
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pads, "copy pad")),
    );
    hold(&mut drive, false);
    assert!(drive.app.copy_in_hand.is_some());
    hold(&mut drive, true);
    cell(&mut drive, 52);
    hold(&mut drive, false);
    assert!(drive.app.copy_in_hand.is_none(), "the copy is cancelled");
    assert_eq!(drive.app.screen, Screen::Pads);
}

/// Scenario "Holding PLAY names the metronome" (transport).
#[test]
fn holding_play_names_the_metronome() {
    let mut drive = hold_drive();
    settle(&mut drive);
    // PLAY held past the hold time: the hold result, not the tap (input-devices)
    let start = Instant::now();
    for (millis, value) in [(0, 127), (400, 0)] {
        drive.midi().recorded_bytes_at(
            SMC,
            &[0xB0, PLAY, value],
            &drive.project,
            start + std::time::Duration::from_millis(millis),
        );
    }
    drive.frame();
    assert!(drive.app.transport.metronome);
    assert_eq!(drive.last_notice().text, "metronome on");
    settle(&mut drive);
    let rows = shared_feedback_rows(&mut drive, 120, 40, Instant::now());
    assert!(rows[1].contains("120.0 click"), "{}", rows[1]);
    assert!(!rows[1].contains(" M"), "{}", rows[1]);
}

/// Scenario "STOP on A1 pauses A1 and keeps the recording" (recording).
#[test]
fn stop_on_a1_pauses_a1_and_keeps_the_recording() {
    let mut drive = a1_drive();
    button(&mut drive, PLAY);
    settle(&mut drive);
    assert!(drive.app.transport_state().playing);
    recording(&mut drive);
    assert!(line(&drive).ends_with("STOP stops play · STOP end rec"));
    button(&mut drive, STOP);
    settle(&mut drive);
    assert!(!drive.app.transport_state().playing, "A1 pauses");
    assert!(drive.app.recording.is_some(), "the recording goes on");
    assert!(line(&drive).ends_with("· STOP end rec"), "{}", line(&drive));
    button(&mut drive, STOP);
    assert!(drive.app.recording.is_none(), "the second STOP ends it");
}

/// Scenario "One STOP saves when A1 does not play" (recording).
#[test]
fn one_stop_saves_when_a1_does_not_play() {
    let mut drive = hold_drive();
    drive.prepare_chop(tone(), 4);
    recording(&mut drive);
    let stop = button(&mut drive, STOP);
    assert!(drive.app.recording.is_none(), "one STOP ends the recording");
    assert!(
        !stop
            .ui
            .iter()
            .any(|command| matches!(command, Command::TransportStopContext)),
        "the stop ends the recording, not the transport"
    );
}

/// Scenario "Second STOP during the output recording saves it" (transport).
#[test]
fn second_stop_during_the_output_recording_saves_it() {
    let mut drive = a1_drive();
    button(&mut drive, PLAY);
    settle(&mut drive);
    for _ in 0..64 {
        drive.render_blocks(64);
        drive.frame();
    }
    recording(&mut drive);
    button(&mut drive, STOP);
    settle(&mut drive);
    let paused = drive.app.transport_state();
    assert!(!paused.playing);
    assert!(drive.app.recording.is_some());
    let stop = button(&mut drive, STOP);
    settle(&mut drive);
    assert!(drive.app.recording.is_none(), "the second tap saves");
    assert!(
        !stop
            .ui
            .iter()
            .any(|command| matches!(command, Command::TransportStopContext)),
        "the paused position stays"
    );
}

/// Scenario "STOP on Chop stops the preview before the recording" (transport).
#[test]
fn stop_on_chop_stops_the_preview_before_the_recording() {
    let mut drive = hold_drive();
    drive.prepare_chop(tone(), 4);
    drive.app.fake_playhead = Some(120);
    recording(&mut drive);
    let first = button(&mut drive, STOP);
    assert!(
        first
            .ui
            .iter()
            .any(|command| matches!(command, Command::Preview { sound: None, .. }))
    );
    assert!(drive.app.recording.is_some(), "the preview stops first");
    drive.app.fake_playhead = None;
    button(&mut drive, STOP);
    assert!(drive.app.recording.is_none(), "the second tap saves");
}

fn press_recording_stop(drive: &mut Drive, keyboard: bool) {
    if keyboard {
        drive.tap(KeyCode::Char('y'), M::ALT);
    } else {
        button(drive, STOP);
    }
}

fn stop_recording(drive: &mut Drive, keyboard: bool) {
    press_recording_stop(drive, keyboard);
    settle(drive);
}

fn record_test_take(drive: &mut Drive, files: &TransitionFiles) {
    drive.app.paths.samples = files.0.clone();
    recording(drive);
    let recorder = &drive.app.audio.as_ref().unwrap().recorder;
    recorder.start(audio::SOURCE_OUTPUT);
    recorder.append_test_output(&[0.25; 512]);
}

fn take_path(drive: &Drive) -> &Path {
    assert!(drive.app.recording.is_none());
    drive
        .app
        .browser
        .in_hand
        .as_deref()
        .expect("the take in hand")
}

/// Scenario "STOP pauses A1 and keeps the take on Files and Chop" (recording).
#[test]
fn stop_pauses_a1_and_keeps_the_take_on_files_and_chop() {
    let files = TransitionFiles::new();
    for keyboard in [false, true] {
        for (screen, assign) in [(6, false), (3, false), (3, true)] {
            let mut drive = a1_drive();
            drive.load_pad(0, tone(), "source");
            button(&mut drive, PLAY);
            settle(&mut drive);
            assert!(drive.app.transport_state().playing);

            record_test_take(&mut drive, &files);
            drive.tap(KeyCode::F(screen), M::NONE);
            assert_eq!(
                drive.app.screen,
                if screen == 6 {
                    Screen::Files
                } else {
                    Screen::Chop
                }
            );
            if assign {
                drive.tap(KeyCode::Char('p'), M::NONE);
                assert!(drive.app.plan.is_some());
            }
            stop_recording(&mut drive, keyboard);
            let paused = drive.app.transport_state();
            assert!(!paused.playing, "STOP must pause A1");
            assert!(drive.app.recording.is_some(), "the take keeps the tail");
            assert!(drive.app.browser.in_hand.is_none());

            drive
                .app
                .audio
                .as_ref()
                .unwrap()
                .recorder
                .append_test_output(&[0.125; 512]);
            stop_recording(&mut drive, keyboard);
            assert_eq!(drive.app.transport_state(), paused);
            assert_eq!(drive.app.plan.is_some(), assign);
            let mut wav = hound::WavReader::open(take_path(&drive)).unwrap();
            assert_eq!(wav.duration(), 1024);
            let samples: Vec<i16> = wav.samples().map(Result::unwrap).collect();
            assert_eq!(samples[400 * 2], 8192);
            assert_eq!(samples[700 * 2], 4096);
        }
    }
}

/// Scenario "Two STOP presses before the A1 pause save the take" (recording).
#[test]
fn two_stop_presses_before_the_a1_pause_save_the_take() {
    let files = TransitionFiles::new();
    for keyboard in [true, false] {
        let mut drive = a1_drive();
        button(&mut drive, PLAY);
        settle(&mut drive);
        drive.render_blocks(64);
        let playing = drive.app.transport_state();
        assert!(playing.playing && playing.ticks > 0.0);
        record_test_take(&mut drive, &files);
        drive.tap(KeyCode::F(6), M::NONE);

        press_recording_stop(&mut drive, keyboard);
        press_recording_stop(&mut drive, keyboard);
        assert_eq!(drive.app.transport_state(), playing);
        assert!(drive.app.recording.is_some());
        assert!(drive.app.browser.in_hand.is_none());

        settle(&mut drive);
        drive.app.flush_audio();
        let paused = drive.app.transport_state();
        assert!(!paused.playing);
        assert_eq!(paused.ticks, playing.ticks, "keyboard={keyboard}");
        assert_eq!(
            hound::WavReader::open(take_path(&drive))
                .unwrap()
                .duration(),
            512
        );
    }
}

#[test]
fn finishing_a_take_clears_its_pending_stop_before_the_next_take() {
    let files = TransitionFiles::new();
    let mut drive = a1_drive();
    button(&mut drive, PLAY);
    settle(&mut drive);
    record_test_take(&mut drive, &files);

    press_recording_stop(&mut drive, true);
    press_recording_stop(&mut drive, true);
    assert!(drive.app.transport_state().playing);
    drive.tap(KeyCode::Char('t'), M::CONTROL);
    let first_take = take_path(&drive).to_owned();
    record_test_take(&mut drive, &files);

    settle(&mut drive);
    drive.app.flush_audio();
    assert!(!drive.app.transport_state().playing);
    assert!(
        drive.app.recording.is_some(),
        "the new take keeps recording"
    );
    stop_recording(&mut drive, true);
    assert_ne!(take_path(&drive), first_take);
}

/// Scenario "STOP handles preview A1 and capture in order" (recording).
#[test]
fn stop_handles_preview_a1_and_capture_in_order() {
    let files = TransitionFiles::new();
    for keyboard in [false, true] {
        let mut drive = a1_drive();
        drive.load_pad(0, tone(), "source");
        button(&mut drive, PLAY);
        settle(&mut drive);
        assert!(drive.app.transport_state().playing);
        record_test_take(&mut drive, &files);
        drive.tap(KeyCode::F(3), M::NONE);
        button(&mut drive, PLAY);
        settle(&mut drive);
        assert!(drive.app.preview_frame().is_some());
        assert!(line(&drive).ends_with("STOP stop preview"));

        stop_recording(&mut drive, keyboard);
        assert!(drive.app.preview_frame().is_none());
        assert!(drive.app.transport_state().playing);
        assert!(drive.app.recording.is_some());

        stop_recording(&mut drive, keyboard);
        assert!(!drive.app.transport_state().playing);
        assert!(drive.app.recording.is_some());

        stop_recording(&mut drive, keyboard);
        assert!(take_path(&drive).is_file());
    }
}

/// Scenario "Saving the take cancels the pending preview" (transport).
#[test]
fn saving_the_take_cancels_the_pending_preview() {
    let files = TransitionFiles::new();
    for keyboard in [false, true] {
        let mut drive = hold_drive();
        drive.app.browser = browser::Browser::new(Some(files.0.clone()));
        drive.tap(KeyCode::F(6), M::NONE);
        drive.app.browser.selected = drive
            .app
            .browser
            .entries
            .iter()
            .position(|entry| entry.name == "tone.wav")
            .unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let (_reply_tx, reply_rx) = std::sync::mpsc::channel();
        drive.app.loader = Some((tx, reply_rx));
        button(&mut drive, PLAY);
        let load = rx.try_recv().unwrap();
        assert!(drive.app.preview_frame().is_none());
        assert!(!drive.app.transport_state().playing);
        record_test_take(&mut drive, &files);

        stop_recording(&mut drive, keyboard);
        assert!(take_path(&drive).is_file());
        drive.app.loader_reply(loader::Reply::Loaded {
            pad: PREVIEW,
            generation: load.generation,
            path: load.path,
            sound: tone(),
        });
        assert!(drive.app.preview.is_none());
        assert!(
            !drive
                .frame()
                .ui
                .iter()
                .any(|command| matches!(command, Command::Preview { sound: Some(_), .. }))
        );
    }
}

/// Scenario "The recording hint names one stop control" (recording).
#[test]
fn the_recording_hint_names_one_stop_control() {
    let mut drive = hold_drive();
    turn(&mut drive, 37, 65); // An SMC-PAD input.
    recording(&mut drive);
    let rows = strip_rows(&mut drive, 80, 26);
    let screen = rows.join("\n");
    assert!(screen.contains("STOP end rec"), "{screen}");
    assert!(!screen.contains("Ctrl+T"), "{screen}");
    let status = drive.app.notice_at(Instant::now()).unwrap();
    assert!(status.ends_with("STOP end rec"), "{status}");
}

/// With PAD BANK off, the idle line says how the commands show (F36).
#[test]
fn pad_bank_off_names_the_first_hit() {
    let mut drive = smc_drive();
    let strip = drive.app.smc_knob_strip(Instant::now()).unwrap();
    assert_eq!(strip.mode, None);
    assert_eq!(strip.idle, "PAD BANK: hit to show commands");
    drive.app.midi_gestures.smc_pad_bank_opens_panel = true;
    drive.frame();
    let strip = drive.app.smc_knob_strip(Instant::now()).unwrap();
    assert_eq!(strip.idle, "PAD BANK hold · PLAY play · STOP stop");
}

/// The save failure answers keep the words of its question in the line and the cells (F33).
#[test]
fn the_save_failure_answers_use_back_and_retry_save() {
    let mut drive = hold_drive();
    turn(&mut drive, 37, 65); // An SMC-PAD input.
    drive.app.ask = Some(Ask::ExitRecovery {
        lines: vec!["Save failed".into()],
        offered_at: Instant::now(),
    });
    drive.frame();
    assert_eq!(
        line(&drive),
        "ASK · PAD BANK + 4 retry save · PAD BANK + 1 back"
    );
    hold(&mut drive, true);
    let panel = drive.app.panel_snapshot().unwrap();
    assert_eq!(panel.labels[12].as_deref(), Some("back"));
    assert_eq!(panel.labels[15].as_deref(), Some("retry save"));
}

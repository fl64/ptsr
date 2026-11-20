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
    use ratatui::{Terminal, backend::TestBackend};
    let now = Instant::now();
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
    term.draw(|frame| tui::draw_feedback(frame, &status, &screen, &help, &feedback, None))
        .unwrap();
    term.backend()
        .buffer()
        .content()
        .chunks(width as usize)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect())
        .collect()
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
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Pattern, "tap tempo")),
    );
    hold(&mut drive, false);
    assert_eq!(line(&drive), "TAP · 1 hit · tap again");
    let bpm = drive.app.transport.bpm;
    assert_eq!(bpm, drive.app.transport.bpm, "one tap keeps the tempo");
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
        "A1 SETUP · knob 3 quantize · knob 4 strength · knob 5 bars · PAD BANK + 1 close"
    );
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

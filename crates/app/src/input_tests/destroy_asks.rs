//! Change 138: questions before work goes, the undo they name, remove all cuts and trim silence.

use super::*;

fn tone() -> Arc<Sample> {
    Arc::new(Sample::mono(vec![0.2; 48_000]))
}

/// Four hits of falling strength in one second.
fn hits() -> Arc<Sample> {
    let mut wave = vec![0.0_f32; 48_000];
    for (k, amp) in [0.9_f32, 0.6, 0.35, 0.2].into_iter().enumerate() {
        let at = k * 12_000 + 600;
        for i in 0..3_000 {
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            wave[at + i] = amp * (1.0 - i as f32 / 3_000.0) * sign;
        }
    }
    Arc::new(Sample::mono(wave))
}

/// 0.5 s of silence, 0.5 s of sound, 1 s of silence.
fn padded() -> Arc<Sample> {
    let mut wave = vec![0.0_f32; 96_000];
    for (i, frame) in wave[24_000..48_000].iter_mut().enumerate() {
        *frame = if i % 2 == 0 { 0.5 } else { -0.5 };
    }
    Arc::new(Sample::mono(wave))
}

/// Chop over `sound` with `n` cuts made by hand with the cut key.
fn hand_cuts(sound: Arc<Sample>, n: usize) -> Drive {
    let mut drive = hold_drive();
    drive.prepare_chop(sound.clone(), 1);
    let step = sound.len() / (n + 1);
    for k in 1..=n {
        drive.app.chop.as_mut().unwrap().set_cursor(k * step);
        drive.tap(KeyCode::Char('m'), M::NONE);
    }
    assert_eq!(drive.app.chop.as_ref().unwrap().markers.len(), n);
    drive
}

fn press(drive: &mut Drive, view: panel::View, label: &str) {
    hold(drive, true);
    cell(drive, panel_note(slot_of(view, label)));
}

fn answer(drive: &mut Drive, yes: bool) {
    cell(drive, panel_note(if yes { 16 } else { 13 }));
    hold(drive, false);
}

fn question(drive: &Drive) -> String {
    drive.app.ask.as_ref().expect("a question").lines()[0].clone()
}

fn markers(drive: &Drive) -> Vec<usize> {
    drive.app.chop.as_ref().unwrap().markers.clone()
}

/// Scenario "Equal parts asks before it replaces hand cuts" (command-panel).
#[test]
fn equal_parts_asks_before_it_replaces_hand_cuts() {
    let mut drive = hand_cuts(tone(), 4);
    press(&mut drive, panel::View::Chop, "equal parts");
    assert_eq!(
        question(&drive),
        "Replace 4 hand cuts with 2 equal parts? P2 undo brings them back."
    );
    answer(&mut drive, true);
    assert_eq!(drive.app.chop.as_ref().unwrap().slices().len(), 2);
    assert_eq!(
        drive.last_notice().text,
        "4 cuts replaced · P2 undo brings them back"
    );
    let mut keys = hand_cuts(tone(), 4);
    keys.tap(KeyCode::Char('y'), M::NONE);
    assert!(question(&keys).ends_with("Ctrl+Z brings them back."));
    keys.tap(KeyCode::Enter, M::NONE);
    assert_eq!(
        keys.last_notice().text,
        "4 cuts replaced · Ctrl+Z brings them back"
    );
}

/// Scenario "Equal parts over hand cuts asks" (chopping).
#[test]
fn equal_parts_over_hand_cuts_asks() {
    let mut drive = hand_cuts(tone(), 1);
    let cuts = markers(&drive);
    press(&mut drive, panel::View::Chop, "equal parts");
    answer(&mut drive, false);
    assert!(drive.app.ask.is_none());
    assert_eq!(markers(&drive), cuts, "the hand cut stays");
}

/// Scenario "Cut by hits replaces program cuts without a question" (command-panel).
#[test]
fn cut_by_hits_replaces_program_cuts_without_a_question() {
    let mut drive = hold_drive();
    drive.prepare_chop(hits(), 4);
    press(&mut drive, panel::View::Chop, "cut by hits");
    assert!(drive.app.ask.is_none(), "no question");
    assert!(drive.last_notice().text.starts_with("by hits at -6 dB"));
}

/// Scenario "By hits over hand cuts asks" (chopping).
#[test]
fn by_hits_over_hand_cuts_asks() {
    let mut drive = hand_cuts(hits(), 1);
    let cuts = markers(&drive);
    drive.tap(KeyCode::Char('h'), M::NONE);
    assert_eq!(
        question(&drive),
        "Replace 1 hand cuts with cuts by hits? Ctrl+Z brings them back."
    );
    drive.tap(KeyCode::Esc, M::NONE);
    assert_eq!(markers(&drive), cuts, "no keeps the hand cut");
}

/// Scenario "By hits names the SMC-PAD undo" (chopping).
#[test]
fn by_hits_names_the_smc_pad_undo() {
    let mut drive = hold_drive();
    drive.prepare_chop(hits(), 5);
    press(&mut drive, panel::View::Chop, "cut by hits");
    hold(&mut drive, false);
    assert!(
        drive
            .last_notice()
            .text
            .ends_with("P2 undo brings them back"),
        "{}",
        drive.last_notice().text
    );
}

/// Scenario "The threshold knob asks before it replaces hand cuts" (command-panel).
#[test]
fn the_threshold_knob_asks_before_it_replaces_hand_cuts() {
    let mut drive = hand_cuts(hits(), 2);
    let cuts = markers(&drive);
    turn(&mut drive, 36, 65);
    assert!(matches!(drive.app.ask, Some(Ask::Destroy { .. })));
    assert_eq!(markers(&drive), cuts, "the cuts stay until yes");
    hold(&mut drive, true);
    answer(&mut drive, true);
    assert_ne!(markers(&drive), cuts, "yes recuts by hits");
}

/// Scenario "Destructive cells are orange" (command-panel).
#[test]
fn destructive_cells_are_orange() {
    let mut drive = hold_drive();
    drive.prepare_chop(padded(), 4);
    hold(&mut drive, true);
    let panel = drive.app.panel_snapshot().unwrap();
    for label in [
        "equal parts",
        "cut by hits",
        "fewer hits",
        "remove all cuts",
        "trim silence",
    ] {
        let at = usize::from(slot_of(panel::View::Chop, label) - 1);
        assert!(panel.warn[at], "{label} is orange");
    }
    assert!(!panel.warn[usize::from(slot_of(panel::View::Chop, "cut here") - 1)]);
    hold(&mut drive, false);
    drive.app.goto(Screen::Pads);
    drive.frame();
    hold(&mut drive, true);
    let panel = drive.app.panel_snapshot().unwrap();
    assert!(panel.warn[usize::from(slot_of(panel::View::Pads, "clear pad") - 1)]);
}

/// Scenario "Remove all cuts without cuts is grey" (command-panel).
#[test]
fn remove_all_cuts_without_cuts_is_grey() {
    let mut drive = hold_drive();
    drive.prepare_chop(tone(), 1);
    hold(&mut drive, true);
    let at = usize::from(slot_of(panel::View::Chop, "remove all cuts") - 1);
    assert!(drive.app.panel_snapshot().unwrap().dim[at]);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Chop, "remove all cuts")),
    );
    assert_eq!(drive.last_notice().text, "no cuts to remove");
}

/// Scenario "Remove all cuts asks and can be undone" (chopping).
#[test]
fn remove_all_cuts_asks_and_can_be_undone() {
    let mut drive = hand_cuts(tone(), 6);
    let cuts = markers(&drive);
    press(&mut drive, panel::View::Chop, "remove all cuts");
    assert_eq!(question(&drive), "Remove 6 cuts? P2 undo brings them back.");
    answer(&mut drive, true);
    assert_eq!(drive.app.chop.as_ref().unwrap().slices().len(), 1);
    assert_eq!(
        drive.last_notice().text,
        "6 cuts removed · P2 undo brings them back"
    );
    hold(&mut drive, true);
    cell(&mut drive, panel_note(14));
    assert_eq!(markers(&drive), cuts, "one undo brings back the 6 cuts");
}

/// Scenario "The Delete key removes all cuts" (chopping).
#[test]
fn the_delete_key_removes_all_cuts() {
    let mut drive = hold_drive();
    drive.prepare_chop(hits(), 1);
    drive.tap(KeyCode::Char('h'), M::NONE);
    assert!(!markers(&drive).is_empty(), "cuts by hits");
    drive.tap(KeyCode::Delete, M::NONE);
    assert!(drive.app.ask.is_none(), "no question");
    assert_eq!(drive.app.chop.as_ref().unwrap().slices().len(), 1);
}

/// Scenario "Trim silence without cuts acts at once" (chopping).
#[test]
fn trim_silence_without_cuts_acts_at_once() {
    let mut drive = hold_drive();
    drive.prepare_chop(padded(), 1);
    press(&mut drive, panel::View::Chop, "trim silence");
    hold(&mut drive, false);
    assert!(drive.app.ask.is_none(), "no question");
    let range = drive.app.chop.as_ref().unwrap().range;
    assert_eq!((range.start, range.end), (24_000, 48_000));
    assert!(drive.last_notice().text.starts_with("silence trimmed"));
}

/// Scenario "Trim silence with cuts asks first" (chopping).
#[test]
fn trim_silence_with_cuts_asks_first() {
    let mut drive = hold_drive();
    drive.prepare_chop(padded(), 4);
    let (cuts, range) = (markers(&drive), drive.app.chop.as_ref().unwrap().range);
    press(&mut drive, panel::View::Chop, "trim silence");
    assert!(question(&drive).starts_with("Trim the silence?"));
    assert_eq!(
        drive.app.chop.as_ref().unwrap().range,
        range,
        "no trim before yes"
    );
    answer(&mut drive, true);
    assert_eq!(drive.app.chop.as_ref().unwrap().range.start, 24_000);
    hold(&mut drive, true);
    cell(&mut drive, panel_note(14));
    let chop = drive.app.chop.as_ref().unwrap();
    assert_eq!(
        (chop.range, chop.markers.clone()),
        (range, cuts),
        "one undo"
    );
}

/// Scenario "Trim silence without silence is grey" (chopping).
#[test]
fn trim_silence_without_silence_is_grey() {
    let mut drive = hold_drive();
    drive.prepare_chop(padded(), 1);
    drive.tap(KeyCode::Char('.'), M::ALT);
    hold(&mut drive, true);
    let at = usize::from(slot_of(panel::View::Chop, "trim silence") - 1);
    assert!(drive.app.panel_snapshot().unwrap().dim[at]);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Chop, "trim silence")),
    );
    assert_eq!(drive.last_notice().text, "no silence to trim");
}

/// Scenario "Fewer hits from Chop" (chopping).
#[test]
fn fewer_hits_from_chop() {
    let mut drive = hold_drive();
    drive.prepare_chop(hits(), 1);
    drive.app.chop.as_mut().unwrap().set_threshold(0.5);
    let before = drive.app.chop.as_ref().unwrap().slices().len();
    press(&mut drive, panel::View::Chop, "fewer hits");
    let chop = drive.app.chop.as_ref().unwrap();
    assert_eq!(chop.threshold_db, -18.0);
    assert!(chop.slices().len() <= before, "fewer slices");
}

/// Scenario "Fewer hits at −6 dB is grey" (chopping).
#[test]
fn fewer_hits_at_6_db_is_grey() {
    let mut drive = hold_drive();
    drive.prepare_chop(hits(), 1);
    drive.tap(KeyCode::Char('h'), M::NONE);
    hold(&mut drive, true);
    let at = usize::from(slot_of(panel::View::Chop, "fewer hits") - 1);
    assert!(drive.app.panel_snapshot().unwrap().dim[at]);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Chop, "fewer hits")),
    );
    assert_eq!(drive.last_notice().text, "by hits stays at -6 dB");
}

/// Scenario "Undo after cut by hits names the old slices" (command-panel).
#[test]
fn undo_after_cut_by_hits_names_the_old_slices() {
    let mut drive = hold_drive();
    drive.prepare_chop(hits(), 5);
    press(&mut drive, panel::View::Chop, "cut by hits");
    let now = drive.app.chop.as_ref().unwrap().slices().len();
    assert_ne!(now, 5);
    cell(&mut drive, panel_note(14));
    assert_eq!(drive.app.chop.as_ref().unwrap().slices().len(), 5);
    assert_eq!(
        drive.last_notice().text,
        format!("undo: 5 slices back (were {now})")
    );
}

/// Scenario "Undo after a join names the split" (command-panel).
#[test]
fn undo_after_a_join_names_the_split() {
    let mut drive = hold_drive();
    drive.prepare_chop(tone(), 4);
    drive.app.chop.as_mut().unwrap().select_slice(1);
    drive.tap(KeyCode::Backspace, M::NONE);
    assert_eq!(drive.last_notice().text, "slices 1 and 2 joined");
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(drive.last_notice().text, "undo: slices 1 and 2 split again");
}

/// Scenario "The undo notice replaces the action notice" (pad-playback).
#[test]
fn the_undo_notice_replaces_the_action_notice() {
    let mut drive = Drive::new();
    drive.load_pad(0, tone(), "kick");
    drive.tap(KeyCode::Backspace, M::NONE);
    assert!(drive.app.pads.file[0].is_none());
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(drive.last_notice().text, "undo: clear on pad 1");
    assert!(drive.app.pads.file[0].is_some());
}

/// Scenario "Copy onto a full pad names the undo" (command-panel).
#[test]
fn copy_onto_a_full_pad_names_the_undo() {
    let mut drive = Drive::new();
    drive.load_pad(0, tone(), "source");
    drive.load_pad(18, tone(), "old target");
    drive.tap(KeyCode::Char('c'), M::CONTROL);
    drive.tap(KeyCode::PageDown, M::NONE);
    drive.pad_press(Source::Keyboard, 2);
    drive.pad_release(Source::Keyboard, 2);
    let lines = drive.app.ask.as_ref().expect("a question").lines().to_vec();
    assert!(
        lines.iter().any(|l| l == "Ctrl+Z brings them back."),
        "{lines:?}"
    );
    assert!(
        !lines.iter().any(|l| l.contains("cannot be undone")),
        "{lines:?}"
    );
}

/// Scenario "Forget asks and says that it has no undo" (midi-learn).
#[test]
fn forget_asks_and_says_that_it_has_no_undo() {
    let mut drive = midi_tab_drive(|row| matches!(row, learn::MidiListRow::Action { .. }));
    let row = (0..drive.app.midi_list().len())
        .find(|&row| {
            drive.app.midi_row = row;
            drive.app.forgettable().is_some()
        })
        .expect("a row with a control");
    drive.app.midi_row = row;
    let learned = drive.app.learned.clone();
    let control = drive.app.forgettable().unwrap();
    drive.tap(KeyCode::Backspace, M::NONE);
    let line = question(&drive);
    assert!(line.contains(&control), "{line}");
    assert!(line.ends_with("This cannot be undone."), "{line}");
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.ask.is_none());
    assert_eq!(drive.app.learned, learned, "Esc keeps the binding");
}

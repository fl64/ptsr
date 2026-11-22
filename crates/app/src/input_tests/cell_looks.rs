//! Change 138: bright, orange, grey and empty panel cells.

use super::*;

fn looks(drive: &Drive) -> tui::CommandPanel {
    drive.app.panel_snapshot().unwrap()
}

/// The index of a shipped label in the snapshot of `view`.
fn at(view: panel::View, label: &str) -> usize {
    usize::from(slot_of(view, label) - 1)
}

fn tone() -> Arc<Sample> {
    Arc::new(Sample::mono(vec![0.2; 48_000]))
}

/// Scenario "Pad cells on an empty pad are grey" (command-panel).
#[test]
fn pad_cells_on_an_empty_pad_are_grey() {
    use panel::View::Pads;
    let mut drive = hold_drive();
    hold(&mut drive, true);
    let panel = looks(&drive);
    for label in ["copy pad", "chop pad", "clear pad"] {
        assert!(panel.dim[at(Pads, label)], "{label} is grey");
    }
    assert!(!panel.dim[at(Pads, "edit pad")], "edit pad acts");
    cell(&mut drive, panel_note(slot_of(Pads, "copy pad")));
    assert_eq!(
        drive.last_notice().text,
        "Copy what? This pad holds nothing."
    );
    assert!(drive.app.copy_in_hand.is_none(), "nothing is in hand");
    cell(&mut drive, panel_note(slot_of(Pads, "clear pad")));
    assert_eq!(drive.last_notice().text, "the pad is empty");
    hold(&mut drive, false);
    drive.load_pad(0, tone(), "kick");
    drive.frame();
    hold(&mut drive, true);
    let panel = looks(&drive);
    for label in ["copy pad", "chop pad", "clear pad"] {
        assert!(!panel.dim[at(Pads, label)], "{label} acts on a sound");
    }
}

/// Scenario "Load to pad on a folder is grey" (command-panel).
#[test]
fn load_to_pad_on_a_folder_is_grey() {
    use panel::View::Files;
    let mut drive = reach_drive_from(hold_drive(), Files, tone());
    drive.app.browser.dir = PathBuf::new();
    drive.app.browser.entries = vec![
        browser::Entry {
            name: "loops".into(),
            is_dir: true,
            favorite: None,
        },
        browser::Entry {
            name: "kick.wav".into(),
            is_dir: false,
            favorite: None,
        },
    ];
    drive.app.browser.selected = 0;
    drive.frame();
    hold(&mut drive, true);
    let panel = looks(&drive);
    for label in ["load to pad", "take file", "chop file"] {
        assert!(panel.dim[at(Files, label)], "{label} is grey on a folder");
    }
    let dir = drive.app.browser.dir.clone();
    cell(&mut drive, panel_note(slot_of(Files, "load to pad")));
    assert_eq!(drive.last_notice().text, "RIGHT opens the folder");
    assert_eq!(drive.app.browser.dir, dir, "the folder stays closed");
    assert_eq!(drive.app.screen, Screen::Files);
    drive.app.browser.selected = 1;
    drive.frame();
    let panel = looks(&drive);
    assert!(!panel.dim[at(Files, "load to pad")], "a file loads");
}

/// Scenario "Erase hits during the count-in is grey" (command-panel).
#[test]
fn erase_hits_during_the_count_in_is_grey() {
    use panel::View::Pattern;
    let mut drive = reach_drive_from(hold_drive(), Pattern, tone());
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
    // the pattern restoration completes before a recording can start
    for _ in 0..512 {
        use std::sync::atomic::Ordering::Acquire;
        drive.app.flush_audio();
        drive.render_blocks(1);
        drive.frame();
        if drive.app.pending_audio.is_empty()
            && drive.project.ready.load(Acquire) == drive.project.epoch.load(Acquire)
        {
            break;
        }
    }
    drive.app.screen = Screen::Pads;
    drive.app.pattern_view.overview = true;
    drive.frame();
    hold(&mut drive, true);
    assert!(!looks(&drive).dim[at(Pattern, "erase hits")], "A1 has hits");
    cell(&mut drive, panel_note(slot_of(Pattern, "record A1")));
    let mut count_in = false;
    for _ in 0..64 {
        drive.app.flush_audio();
        drive.render_blocks(1);
        drive.frame();
        if drive
            .app
            .pattern_record_state()
            .is_some_and(|s| s.phase == engine::capture::Phase::CountIn)
        {
            count_in = true;
            break;
        }
    }
    assert!(count_in, "the count-in runs");
    hold(&mut drive, true);
    let panel = looks(&drive);
    assert!(panel.dim[at(Pattern, "erase hits")]);
    assert!(panel.dim[at(Pattern, "edit steps")]);
    cell(&mut drive, panel_note(slot_of(Pattern, "erase hits")));
    assert_eq!(drive.last_notice().text, "wait for the count-in");
    assert!(!drive.app.erase_pads);
}

/// Scenario "Undo with nothing to undo is grey" (command-panel).
#[test]
fn undo_with_nothing_to_undo_is_grey() {
    let mut drive = reach_drive_from(hold_drive(), panel::View::Chop, tone());
    hold(&mut drive, true);
    // device pad 2, the undo of the bottom row, is cell 14
    assert!(looks(&drive).dim[13], "nothing to undo on a fresh chop");
    cell(&mut drive, 53);
    assert_eq!(drive.last_notice().text, "nothing to undo");
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Chop, "equal parts")),
    );
    assert!(!looks(&drive).dim[13], "a cut by parts can be undone");
}

/// Scenario "Grey, orange and empty cells look different" (screen-layout): the states
/// of real views; the drawing of each look is tested in `tui`.
#[test]
fn grey_orange_and_empty_cells_look_different() {
    use panel::View::{Chop, PadEdit, Pads};
    let mut drive = hold_drive();
    hold(&mut drive, true);
    let panel = looks(&drive);
    assert!(panel.dim[at(Pads, "copy pad")] && panel.warn[at(Pads, "clear pad")]);
    let mut drive = reach_drive_from(hold_drive(), PadEdit, tone());
    hold(&mut drive, true);
    let panel = looks(&drive);
    let empty = (0..12).find(|&i| panel.labels[i].is_none()).unwrap();
    assert!(
        !panel.dim[empty] && !panel.warn[empty],
        "a free cell has no look"
    );
    let mut drive = reach_drive_from(hold_drive(), Chop, tone());
    hold(&mut drive, true);
    let panel = looks(&drive);
    for label in ["equal parts", "cut by hits", "fewer hits"] {
        assert!(panel.warn[at(Chop, label)], "{label} is orange");
    }
    for label in ["cut here", "loop play", "put on pads"] {
        assert!(!panel.warn[at(Chop, label)], "{label} keeps its colour");
    }
}

/// Scenario "Play range shows its state" (command-panel): the state follows a cell press
/// from the SMC-PAD and a key at once; the three lines are drawn in the screen tests.
#[test]
fn play_range_shows_its_state() {
    use panel::View::Chop;
    let mut drive = reach_drive_from(hold_drive(), Chop, tone());
    hold(&mut drive, true);
    let range = at(Chop, "play range");
    assert_eq!(looks(&drive).states[range], Some("to end"));
    cell(&mut drive, panel_note(slot_of(Chop, "play range")));
    assert_eq!(looks(&drive).states[range], Some("to cut"));
    hold(&mut drive, false);
    drive.tap(KeyCode::Char('o'), M::NONE);
    hold(&mut drive, true);
    assert_eq!(looks(&drive).states[range], Some("slice"));
}

/// Scenario "Loop play shows its state at 80×26" (command-panel): the state of each state
/// cell follows its action; the two lines at 80×26 are drawn in the screen tests.
#[test]
fn loop_play_shows_its_state_at_80x26() {
    use panel::View::{Chop, ChopEdit, Files, Pads, Pattern};
    let mut drive = reach_drive_from(hold_drive(), Chop, tone());
    hold(&mut drive, true);
    for (label, before, after) in [("loop play", "off", "on"), ("cursor step", "even", "hits")] {
        assert_eq!(
            looks(&drive).states[at(Chop, label)],
            Some(before),
            "{label}"
        );
        cell(&mut drive, panel_note(slot_of(Chop, label)));
        assert_eq!(
            looks(&drive).states[at(Chop, label)],
            Some(after),
            "{label}"
        );
    }
    assert_eq!(
        looks(&drive).states[at(Chop, "cut here")],
        None,
        "an action has no state"
    );
    let mut drive = reach_drive_from(hold_drive(), ChopEdit, tone());
    hold(&mut drive, true);
    let as_pad = at(ChopEdit, "hear as pad");
    assert_eq!(looks(&drive).states[as_pad], Some("off"));
    cell(&mut drive, panel_note(slot_of(ChopEdit, "hear as pad")));
    assert_eq!(looks(&drive).states[as_pad], Some("on"));
    let mut drive = reach_drive_from(hold_drive(), Pads, tone());
    hold(&mut drive, true);
    for (label, states) in [
        ("play mode", ["1 shot", "gate"]),
        ("pitch gate", ["on", "off"]),
        ("mark pad", ["off", "on"]),
    ] {
        let before = looks(&drive).states[at(Pads, label)].expect(label);
        cell(&mut drive, panel_note(slot_of(Pads, label)));
        let after = looks(&drive).states[at(Pads, label)].expect(label);
        assert!(
            before != after && states.contains(&before) && states.contains(&after),
            "{label}: {before} -> {after}"
        );
    }
    let mut drive = reach_drive_from(hold_drive(), Pattern, tone());
    hold(&mut drive, true);
    let empty = at(Pattern, "hide empty");
    assert_eq!(looks(&drive).states[empty], Some("hidden"));
    cell(&mut drive, panel_note(slot_of(Pattern, "hide empty")));
    assert_eq!(looks(&drive).states[empty], Some("shown"));
    let mut drive = reach_drive_from(hold_drive(), Files, tone());
    hold(&mut drive, true);
    let listen = at(Files, "listen file");
    let on = looks(&drive).states[listen];
    cell(&mut drive, panel_note(slot_of(Files, "listen file")));
    assert_ne!(looks(&drive).states[listen], on, "listen file turns");
    assert!(looks(&drive).states[at(Files, "star file")].is_some());
}

/// Scenario "Every state cell shows one of its states" (command-panel): each cell of the
/// table shows a listed state, and the other cells of the view show none.
#[test]
fn every_state_cell_shows_one_of_its_states() {
    use panel::View::{Chop, ChopEdit, Files, PadEdit, Pads, Pattern};
    let table: [(panel::View, &str, &[&str]); 14] = [
        (Chop, "play range", &["to end", "to cut", "slice"]),
        (Chop, "loop play", &["on", "off"]),
        (Chop, "cursor step", &["even", "hits", "cuts"]),
        (ChopEdit, "hear as pad", &["on", "off"]),
        (Pads, "play mode", &["1 shot", "gate"]),
        (Pads, "pitch gate", &["on", "off"]),
        (Pads, "mark pad", &["on", "off"]),
        (Pads, "strip view", &["Sound", "Rhythm"]),
        (PadEdit, "play mode", &["1 shot", "gate"]),
        (PadEdit, "pitch gate", &["on", "off"]),
        (PadEdit, "pin sample", &["on", "off"]),
        (Pattern, "hide empty", &["shown", "hidden"]),
        (Files, "listen file", &["on", "off"]),
        (Files, "star file", &["on", "off"]),
    ];
    for view in [Chop, ChopEdit, Pads, PadEdit, Pattern, Files] {
        let mut drive = reach_drive_from(hold_drive(), view, tone());
        hold(&mut drive, true);
        let states = looks(&drive).states;
        let mut expected = [None; 16];
        for (_, label, listed) in table.iter().filter(|(v, ..)| *v == view) {
            let state = states[at(view, label)].unwrap_or_else(|| panic!("{label} has a state"));
            assert!(listed.contains(&state), "{label}: {state}");
            expected[at(view, label)] = Some(state);
        }
        assert_eq!(
            states, expected,
            "{view:?}: only the table cells show a state"
        );
    }
}

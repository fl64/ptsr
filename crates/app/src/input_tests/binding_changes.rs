//! Change 138: control bindings change from the keyboard only.

use super::*;

/// Scenario "Binding changes are keyboard only" (controller-reach).
#[test]
fn binding_changes_are_keyboard_only() {
    use panel::View;
    // No panel view shows a binding change: the screens view and every action view.
    let mut state = panel::State::default();
    let views = [
        View::Pads,
        View::PadEdit,
        View::Pattern,
        View::Files,
        View::Chop,
        View::ChopEdit,
        View::Fx,
        View::Settings,
    ];
    for page in [-1, 1] {
        state.move_view(page);
        for view in views {
            for cell in 1..=16 {
                let label = state.label(view, cell).unwrap_or_default();
                assert!(
                    !matches!(label, "learn" | "learn control" | "forget control"),
                    "{view:?} cell {cell} shows {label}"
                );
            }
        }
    }

    // From the SMC-PAD, choose refuses each binding row: a learn, a run, a reset.
    let rows: [fn(&learn::MidiListRow) -> bool; 3] = [
        |row| matches!(row, learn::MidiListRow::Action { .. }),
        |row| matches!(row, learn::MidiListRow::Set { .. }),
        |row| matches!(row, learn::MidiListRow::Reset { .. }),
    ];
    for pick in rows {
        let mut drive = midi_tab_drive(pick);
        labelled_cell(&mut drive, View::Settings, "choose");
        assert!(drive.app.ask.is_none() && drive.app.learning.is_none());
        assert!(drive.app.midi_run.is_none());
        assert_eq!(
            drive.last_notice().text,
            "Bindings: keyboard only (F10, Enter)"
        );
        // The keyboard keeps the same row: Enter learns, runs or asks.
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(
            drive.app.learning.is_some() || drive.app.midi_run.is_some() || drive.app.ask.is_some(),
            "Enter acts on the row"
        );
    }

    // F10 Learn opens from the keyboard on a work screen.
    let mut drive = hold_drive();
    drive.tap(KeyCode::F(10), M::NONE);
    assert!(drive.app.screen_learn.is_some());
}

/// Scenario "The controller can cancel a binding but cannot save it" (midi-learn).
#[test]
fn the_controller_can_cancel_a_binding_but_cannot_save_it() {
    let files = TransitionFiles::new();
    let path = files.0.join("config.toml");
    std::fs::write(&path, "[midi]\n").unwrap();
    let select =
        |row: &MidiListRow| matches!(row, MidiListRow::Action { info } if info.name == "bank.next");
    let mut drive = midi_tab_drive(select);
    drive.app.user_path = Some(path.clone());
    drive.midi().recorded_connect(SMC);
    drive.frame();
    let original = std::fs::read_to_string(&path).unwrap();
    drive.app.midi_row = drive.app.midi_list().iter().position(select).unwrap();
    drive.tap(KeyCode::Enter, M::NONE);
    let control = |drive: &mut Drive| {
        drive.bytes(SMC, &[0xb0, 88, 127]);
        drive.bytes(SMC, &[0xb0, 88, 0]);
        drive.frame();
        assert!(drive.app.midi_binding_pending());
    };
    control(&mut drive);
    for p4 in [39, 55] {
        cell(&mut drive, p4);
        assert!(drive.app.midi_binding_pending());
        assert_eq!(drive.last_notice().text, dispatch::BINDINGS_BY_KEYS);
        assert!(std::fs::read_to_string(&path).unwrap() == original);
    }
    cell(&mut drive, 52);
    assert!(drive.app.learning.is_none());
    assert!(std::fs::read_to_string(&path).unwrap() == original);

    drive.tap(KeyCode::Enter, M::NONE);
    control(&mut drive);
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(drive.app.learning.is_none());
    let saved = std::fs::read_to_string(&path).unwrap();
    assert_ne!(saved, original);
    let mut reconnected = Drive::with_saved_bindings_and_smc(false, Some(&path), true);
    reconnected.midi().recorded_connect(SMC);
    reconnected.frame();
    let map = learn::group_bindings(&reconnected.app.learned, "SMC-PAD", input::Group::Global);
    assert_eq!(
        map.get(&input::midi::Key::Cc(88)).unwrap().primary(),
        "bank.next"
    );

    for question in ["reset", "layout", "forget"] {
        let mut drive = midi_tab_drive(|row| matches!(row, MidiListRow::Reset { .. }));
        let path = files.0.join(format!("{question}.toml"));
        std::fs::write(&path, &saved).unwrap();
        drive.app.user_path = Some(path.clone());
        drive.midi().recorded_connect(SMC);
        drive.frame();
        let original = std::fs::read_to_string(&path).unwrap();
        match question {
            "reset" => {
                drive.tap(KeyCode::Enter, M::NONE);
            }
            "layout" => {
                learn::assign(
                    &mut drive.app.learned,
                    "SMC-PAD",
                    input::Group::Global,
                    input::midi::Key::Cc(30),
                    "projects.scroll",
                );
                drive.app.ask_smc_layout(None);
                drive.frame();
            }
            _ => {
                drive.app.ask_destroy(
                    crate::destroy::Act::Forget,
                    "Forget the control?".into(),
                    "it",
                );
                drive.frame();
            }
        }
        assert!(
            drive
                .app
                .ask
                .as_ref()
                .is_some_and(Ask::changes_midi_binding),
            "{question}"
        );
        let before = drive.app.learned.clone();
        for p4 in [39, 55] {
            cell(&mut drive, p4);
            assert!(drive.app.ask.is_some(), "{question}");
            let text = drive
                .app
                .question_view(&drive.keymap)
                .unwrap()
                .answers
                .join(" ");
            assert!(
                text.contains("Enter") && text.contains("P1") && !text.contains("P4"),
                "{text}"
            );
            assert_eq!(drive.app.learned, before, "{question}");
            assert!(
                std::fs::read_to_string(&path).unwrap() == original,
                "{question} changed the file"
            );
        }
        cell(&mut drive, 52);
        assert!(drive.app.ask.is_none());
        assert_eq!(drive.app.learned, before, "{question}");
    }
}

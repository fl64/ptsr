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

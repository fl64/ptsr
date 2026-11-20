//! Change 135: join a slice with its neighbour from the Chop edit panel.

use super::*;

/// A chop of `slices` slices with slice `selected` (0-based) selected, on Chop.
fn join_drive(slices: usize, selected: usize) -> Drive {
    let mut drive = hold_drive();
    drive.prepare_chop(Arc::new(Sample::mono(vec![0.2; 48_000])), slices);
    assert!(drive.app.chop.as_mut().unwrap().select_slice(selected));
    drive.frame();
    drive
}

/// Join with the keyboard (Backspace, Alt+Backspace) or with the panel: PAD BANK held,
/// `edit slice`, then `join left` or `join right`.
fn join(drive: &mut Drive, source: Source, right: bool) {
    match source {
        Source::Keyboard => {
            // Backspace joins left, Alt+Backspace joins right
            drive.tap(KeyCode::Backspace, if right { M::ALT } else { M::NONE });
        }
        _ => {
            hold(drive, true);
            if !drive.app.edit {
                cell(drive, panel_note(slot_of(panel::View::Chop, "edit slice")));
            }
            let (left, right_slot) = (
                slot_of(panel::View::ChopEdit, "join left"),
                slot_of(panel::View::ChopEdit, "join right"),
            );
            let labels = drive.app.panel_snapshot().unwrap().labels;
            assert_eq!(labels[usize::from(left - 1)].as_deref(), Some("join left"));
            assert_eq!(
                labels[usize::from(right_slot - 1)].as_deref(),
                Some("join right")
            );
            cell(drive, panel_note(if right { right_slot } else { left }));
            hold(drive, false);
        }
    }
}

fn slices(drive: &Drive) -> usize {
    drive.app.chop.as_ref().unwrap().slices().len()
}

/// Scenario "Join left names the joined slices" (chopping).
#[test]
fn join_left_names_the_joined_slices() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = join_drive(4, 2);
        join(&mut drive, source, false);
        assert_eq!(slices(&drive), 3, "{source:?}");
        assert_eq!(
            drive.last_notice().text,
            "slices 2 and 3 joined",
            "{source:?}"
        );
        let mut drive = join_drive(4, 2);
        join(&mut drive, source, true);
        assert_eq!(
            drive.last_notice().text,
            "slices 3 and 4 joined",
            "{source:?}"
        );
    }
}

/// Scenario "The first slice has no left neighbour" (chopping).
#[test]
fn the_first_slice_has_no_left_neighbour() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = join_drive(4, 0);
        if matches!(source, Source::Button) {
            hold(&mut drive, true);
            cell(
                &mut drive,
                panel_note(slot_of(panel::View::Chop, "edit slice")),
            );
            let dim = drive.app.panel_snapshot().unwrap().dim;
            assert!(dim[9] && !dim[10], "join left dim, join right lit");
            hold(&mut drive, false);
        }
        join(&mut drive, source, false);
        assert_eq!(slices(&drive), 4, "{source:?}");
        assert_eq!(
            drive.last_notice().text,
            "slice 1 has no slice on the left",
            "{source:?}"
        );
        let mut drive = join_drive(4, 3);
        join(&mut drive, source, true);
        assert_eq!(slices(&drive), 4, "{source:?}");
        assert_eq!(
            drive.last_notice().text,
            "slice 4 has no slice on the right"
        );
    }
}

/// Scenario "One slice has nothing to join" (chopping).
#[test]
fn one_slice_has_nothing_to_join() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = join_drive(1, 0);
        if matches!(source, Source::Button) {
            hold(&mut drive, true);
            cell(
                &mut drive,
                panel_note(slot_of(panel::View::Chop, "edit slice")),
            );
            let dim = drive.app.panel_snapshot().unwrap().dim;
            assert!(dim[9] && dim[10], "both join cells dim");
            hold(&mut drive, false);
        }
        join(&mut drive, source, true);
        assert_eq!(slices(&drive), 1, "{source:?}");
        assert_eq!(drive.last_notice().text, "nothing to join: one slice");
    }
}

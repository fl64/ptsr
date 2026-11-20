//! Change 138: the trim clear and the A1 pass undo keep controller routes without cells.

use super::*;
use crate::input_tests::patterns::{
    app_capture_finish, app_capture_frames, app_capture_start, settle_pattern,
};

/// Scenario "Trim clear and A1 pass undo have controller routes" (controller-reach).
#[test]
fn trim_clear_and_a1_pass_undo_have_controller_routes() {
    // Knobs 5 and 6 turned to the ends clear the trim.
    let mut drive = hold_drive();
    let sound = Arc::new(Sample::mono(vec![0.2; 48_000]));
    drive.prepare_chop(sound.clone(), 1);
    {
        let c = drive.app.chop.as_mut().unwrap();
        c.set_region_start(10_000);
        c.set_region_end(38_000);
    }
    drive.frame();
    for _ in 0..400 {
        turn(&mut drive, 34, 63); // Knob 5 down.
        turn(&mut drive, 35, 65); // Knob 6 up.
    }
    let range = drive.app.chop.as_ref().unwrap().range;
    assert_eq!(
        (range.start, range.end),
        (0, sound.len()),
        "the trim is clear"
    );

    // The undo cell on A1 undoes the last A1 pass.
    let mut drive = hold_drive();
    drive.app.apply_session(session::Session {
        pattern: Some("A1".into()),
        patterns: vec![session::PatternState {
            slot: "A1".into(),
            bars: 1,
            quantize: "1/16".into(),
            strength: 100,
            hits: Vec::new(),
        }],
        ..Default::default()
    });
    settle_pattern(&mut drive);
    hold(&mut drive, true);
    list_button(&mut drive, false);
    let a1 = slot_of_screen(&drive, "A1");
    cell(&mut drive, panel_note(a1));
    hold(&mut drive, false);
    app_capture_start(&mut drive, 1);
    app_capture_frames(&mut drive, 1500);
    drive.pad_press(Source::Keyboard, 0);
    app_capture_frames(&mut drive, 3000);
    drive.pad_release(Source::Keyboard, 0);
    app_capture_frames(&mut drive, 3000);
    drive.app.flush_audio();
    app_capture_frames(&mut drive, 0);
    app_capture_finish(&mut drive, 1);
    let hits = |drive: &Drive| drive.app.pads.pattern.as_ref().unwrap().hits.len();
    assert_eq!(hits(&drive), 1, "the pass recorded one hit");
    hold(&mut drive, true);
    cell(&mut drive, panel_note(14));
    hold(&mut drive, false);
    drive.app.flush_audio();
    app_capture_frames(&mut drive, 0);
    drive.frame();
    assert_eq!(hits(&drive), 0, "the undo cell took the pass back");
}

/// The cell of the screens view with `label`.
fn slot_of_screen(drive: &Drive, label: &str) -> u8 {
    let index = drive
        .app
        .panel_snapshot()
        .unwrap()
        .labels
        .iter()
        .position(|cell| cell.as_deref() == Some(label))
        .unwrap_or_else(|| panic!("{label} in the screens view"));
    u8::try_from(index + 1).unwrap()
}

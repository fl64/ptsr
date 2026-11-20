//! Change 136: the Chop zoom keeps the point that the owner touched last in place.

use super::*;

/// A chop of 4 slices over 4 s with slice 2 selected and the cursor inside it, apart from
/// its bounds, so a zoom around the cursor and one around a bound differ.
fn zoom_drive() -> Drive {
    let mut drive = hold_drive();
    drive.prepare_chop(Arc::new(Sample::mono(vec![0.2; 192_000])), 4);
    let c = drive.app.chop.as_mut().unwrap();
    assert!(c.select_slice(1));
    c.set_cursor(60_000);
    assert_eq!(c.selected, 1);
    drive.frame();
    drive
}

/// Where `pos` lies in the picture, 0 at the left edge and 1 at the right edge.
fn place(drive: &Drive, pos: usize) -> f64 {
    let v = drive.app.chop.as_ref().unwrap().view;
    (pos as f64 - v.start as f64) / (v.end - v.start) as f64
}

fn chop(drive: &Drive) -> &chop::Chop {
    drive.app.chop.as_ref().unwrap()
}

fn start(drive: &Drive) -> usize {
    chop(drive).slices()[chop(drive).selected].start
}

fn end(drive: &Drive) -> usize {
    chop(drive).slices()[chop(drive).selected].end
}

fn zoom_in(drive: &mut Drive) {
    drive.tap(KeyCode::Char('i'), M::NONE);
}

/// Asserts that `pos` keeps its place in the picture over `zoom`, and that the view shrank.
fn holds(drive: &mut Drive, pos: impl Fn(&Drive) -> usize, zoom: impl Fn(&mut Drive)) {
    let (before, len) = (
        place(drive, pos(drive)),
        chop(drive).view.end - chop(drive).view.start,
    );
    zoom(drive);
    let after = place(drive, pos(drive));
    assert!(
        chop(drive).view.end - chop(drive).view.start < len,
        "the view zoomed in"
    );
    assert!((before - after).abs() < 1e-3, "{before} → {after}");
}

/// Scenario "Zoom around the slice start" (chopping).
#[test]
fn zoom_around_the_slice_start() {
    let mut drive = zoom_drive();
    drive.tap(KeyCode::Right, M::CONTROL);
    assert_ne!(start(&drive), chop(&drive).cursor);
    holds(&mut drive, start, zoom_in);
}

/// Scenario "Zoom around the slice end with the knob" (chopping).
#[test]
fn zoom_around_the_slice_end_with_the_knob() {
    let mut drive = zoom_drive();
    turn(&mut drive, 33, 65);
    holds(&mut drive, end, |drive| turn(drive, 31, 65));
}

/// Scenario "Zoom around the region start" (chopping).
#[test]
fn zoom_around_the_region_start() {
    let mut drive = zoom_drive();
    turn(&mut drive, 34, 65);
    let region = |drive: &Drive| chop(drive).range.start;
    assert!(region(&drive) > 0, "the region knob moved the start");
    holds(&mut drive, region, zoom_in);
}

/// Scenario "A cursor move takes the anchor back" (chopping).
#[test]
fn a_cursor_move_takes_the_anchor_back() {
    for source in ["key", "knob"] {
        let mut drive = zoom_drive();
        drive.tap(KeyCode::Right, M::ALT);
        match source {
            "key" => drive.tap(KeyCode::Char('l'), M::NONE),
            _ => {
                turn(&mut drive, 30, 65);
                drive.frame()
            }
        };
        holds(&mut drive, |drive| chop(drive).cursor, zoom_in);
    }
}

/// Scenario "A pad hit takes the anchor back" (chopping).
#[test]
fn a_pad_hit_takes_the_anchor_back() {
    let mut drive = zoom_drive();
    drive.tap(KeyCode::Right, M::CONTROL);
    // the slice keys: device pad 15 plays slice 3
    drive.bytes(SMC, &[0x99, 35 + 15, 100]);
    drive.bytes(SMC, &[0x89, 35 + 15, 0]);
    drive.frame();
    assert_eq!(chop(&drive).selected, 2);
    holds(&mut drive, |drive| chop(drive).cursor, zoom_in);
}

/// Scenario "The anchor follows an undo" (chopping).
#[test]
fn the_anchor_follows_an_undo() {
    let mut drive = zoom_drive();
    let old = end(&drive);
    drive.tap(KeyCode::Right, M::ALT);
    assert_ne!(end(&drive), old);
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(end(&drive), old, "the undo moved the end back");
    holds(&mut drive, end, zoom_in);
}

/// Scenario "A deleted anchor gives the anchor to the cursor" (chopping).
#[test]
fn a_deleted_anchor_gives_the_anchor_to_the_cursor() {
    let mut drive = zoom_drive();
    drive.tap(KeyCode::Right, M::CONTROL);
    drive.tap(KeyCode::Backspace, M::NONE);
    assert_eq!(
        chop(&drive).slices().len(),
        3,
        "the cut at the slice start is gone"
    );
    holds(&mut drive, |drive| chop(drive).cursor, zoom_in);
}

/// Scenario "The cursor comes back after a zoom around a bound" (chopping).
#[test]
fn the_cursor_comes_back_after_a_zoom_around_a_bound() {
    let mut drive = zoom_drive();
    drive.tap(KeyCode::Right, M::ALT);
    for _ in 0..4 {
        zoom_in(&mut drive);
    }
    let cursor = chop(&drive).cursor;
    let v = chop(&drive).view;
    assert!(
        cursor < v.start || cursor > v.end,
        "the cursor left the view"
    );
    drive.tap(KeyCode::Char('l'), M::NONE);
    let (cursor, v) = (chop(&drive).cursor, chop(&drive).view);
    assert!(
        v.start <= cursor && cursor <= v.end,
        "the view shows the cursor"
    );
}

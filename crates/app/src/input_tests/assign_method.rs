//! Change 138: the Assign method cell, its question and apply waiting for the start pad.

use super::*;

/// A first-layer hit on the device pad printed `number` (1 bottom left, 16 top right).
fn hit(drive: &mut Drive, number: u8) -> Frame {
    drive.bytes(SMC, &[0x99, 35 + number, 100]);
    drive.bytes(SMC, &[0x89, 35 + number, 0]);
    drive.frame()
}

/// The pad index under the device pad printed `number`.
fn abs(number: usize) -> usize {
    let (row, column) = (3 - (number - 1) / 4, (number - 1) % 4);
    row * 4 + column
}

/// An open Assign plan by hand over five slices, from the SMC-PAD.
fn assigning() -> Drive {
    let mut drive = hold_drive();
    drive.prepare_chop(Arc::new(Sample::mono(vec![0.2; 48_000])), 5);
    hold(&mut drive, true);
    cell(
        &mut drive,
        panel_note(slot_of(panel::View::Chop, "put on pads")),
    );
    hold(&mut drive, false);
    assert!(matches!(drive.app.plan, Some(chop::Plan::ByHand(_))));
    drive
}

fn method_label(drive: &Drive) -> String {
    drive.app.panel_snapshot().unwrap().labels[3]
        .clone()
        .unwrap_or_default()
}

fn linked(drive: &Drive) -> usize {
    (0..PADS)
        .filter(|&pad| drive.app.pads.link[pad].is_some())
        .count()
}

/// Scenario "Assign shows its method" (command-panel).
#[test]
fn assign_shows_its_method() {
    let mut drive = assigning();
    hold(&mut drive, true);
    assert_eq!(method_label(&drive), "by hand");
    cell(&mut drive, panel_note(4));
    assert!(matches!(drive.app.plan, Some(chop::Plan::InOrder(None))));
    assert_eq!(method_label(&drive), "in order");
}

/// Scenario "Apply waits for the start pad" (command-panel).
#[test]
fn apply_waits_for_the_start_pad() {
    let mut drive = assigning();
    hold(&mut drive, true);
    cell(&mut drive, panel_note(4)); // In order.
    assert!(drive.app.panel_snapshot().unwrap().dim[15], "apply is grey");
    cell(&mut drive, panel_note(16));
    hold(&mut drive, false);
    assert_eq!(linked(&drive), 0, "no slice is placed");
    assert!(drive.app.plan.is_some());
    assert_eq!(drive.last_notice().text, "hit the start pad first");
}

/// Scenario "A switch with placed slices asks" (chopping).
#[test]
fn a_switch_with_placed_slices_asks() {
    // The SMC-PAD method cell, answered with device pad 4.
    let mut drive = assigning();
    hit(&mut drive, 1);
    hit(&mut drive, 2);
    assert_eq!(drive.app.placed_slices(), 2);
    hold(&mut drive, true);
    cell(&mut drive, panel_note(4));
    assert!(
        matches!(drive.app.plan, Some(chop::Plan::ByHand(_))),
        "asks first"
    );
    let question = drive.app.ask.as_ref().expect("a question").lines()[0].clone();
    assert!(question.contains("2 placed slices go"), "{question}");
    cell(&mut drive, panel_note(16));
    assert_eq!(drive.app.plan, Some(chop::Plan::InOrder(None)));
    assert_eq!(drive.app.placed_slices(), 0);
    assert_eq!(drive.last_notice().text, "in order · P2 undo brings 2 back");
    cell(&mut drive, panel_note(14));
    hold(&mut drive, false);
    assert!(matches!(drive.app.plan, Some(chop::Plan::ByHand(_))));
    assert_eq!(drive.app.placed_slices(), 2, "one undo brings both back");

    // Tab from the keyboard, answered with Enter.
    let mut keys = Drive::new();
    keys.prepare_chop(Arc::new(Sample::mono(vec![0.2; 48_000])), 5);
    keys.tap(KeyCode::Char('p'), M::NONE);
    keys.tap(KeyCode::Char('1'), M::NONE);
    keys.tap(KeyCode::Char('2'), M::NONE);
    assert_eq!(keys.app.placed_slices(), 2);
    keys.tap(KeyCode::Tab, M::NONE);
    assert!(keys.app.ask.is_some(), "Tab asks first");
    keys.tap(KeyCode::Enter, M::NONE);
    assert_eq!(keys.app.plan, Some(chop::Plan::InOrder(None)));
    assert_eq!(keys.last_notice().text, "in order · Ctrl+Z brings 2 back");
    keys.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(keys.app.placed_slices(), 2);
}

/// Scenario "Enter before the start pad places nothing" (chopping).
#[test]
fn enter_before_the_start_pad_places_nothing() {
    let mut keys = Drive::new();
    keys.prepare_chop(Arc::new(Sample::mono(vec![0.2; 48_000])), 5);
    keys.tap(KeyCode::Char('p'), M::NONE);
    keys.tap(KeyCode::Tab, M::NONE);
    assert_eq!(keys.app.plan, Some(chop::Plan::InOrder(None)));
    keys.tap(KeyCode::Enter, M::NONE);
    assert_eq!(keys.app.plan, Some(chop::Plan::InOrder(None)));
    assert_eq!(linked(&keys), 0);
    assert_eq!(keys.last_notice().text, "hit the start pad first");
}

/// Scenario "Enter after the start pad applies the plan" (chopping).
#[test]
fn enter_after_the_start_pad_applies_the_plan() {
    let mut drive = assigning();
    hold(&mut drive, true);
    cell(&mut drive, panel_note(4)); // In order.
    hold(&mut drive, false);
    hit(&mut drive, 9);
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(drive.app.plan.is_none(), "applied");
    let start = abs(9);
    for slice in 0..5 {
        assert!(
            drive.app.pads.link[start + slice].is_some(),
            "slice {} follows from pad 9",
            slice + 1
        );
    }
}

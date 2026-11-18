use super::*;

fn press(drive: &mut Drive, slot: u8) -> Frame {
    cell(drive, panel_note(slot))
}

fn reveal(drive: &mut Drive) {
    press(drive, 1);
    assert_eq!(drive.app.panel.owner(), Some(SMC));
}

fn target(drive: &Drive, name: &str) {
    let panel = drive.app.panel_snapshot().unwrap();
    assert!(panel.view.starts_with(name), "{}", panel.view);
    assert_eq!(panel.labels[12].as_deref(), Some("cancel"));
    assert_eq!(panel.labels[14].as_deref(), Some("menu"));
}

fn check_silent(frame: &Frame) {
    assert!(
        !frame
            .commands()
            .into_iter()
            .any(|cmd| matches!(cmd, Command::PadPress { .. }))
    );
}

/// Scenario "Panel target pads place items without a layer switch" (command-panel).
#[test]
fn panel_target_pads_place_items_without_a_layer_switch() {
    let source = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let existing = Arc::new(Sample::mono(vec![0.4; 4_800]));
    let mut drive = reach_drive(panel::View::Pads, source.clone());
    drive.load_pad(2, existing.clone(), "existing");
    reveal(&mut drive);

    press(&mut drive, 2);
    target(&drive, "COPY");
    check_silent(&press(&mut drive, 2));
    assert!(Arc::ptr_eq(
        drive.app.pads.sound[1].as_ref().unwrap(),
        &source
    ));
    assert!(drive.app.copy_in_hand.is_none());
    assert_eq!(drive.app.panel.owner(), Some(SMC));

    drive.app.pads.selected = 0;
    press(&mut drive, 2);
    check_silent(&press(&mut drive, 3));
    assert!(matches!(drive.app.ask, Some(Ask::CopyPad { .. })));
    press(&mut drive, 7);
    assert!(Arc::ptr_eq(
        drive.app.pads.sound[2].as_ref().unwrap(),
        &existing
    ));
    target(&drive, "COPY");
    press(&mut drive, 3);
    press(&mut drive, 8);
    assert!(Arc::ptr_eq(
        drive.app.pads.sound[2].as_ref().unwrap(),
        &source
    ));

    drive.app.pads.selected = 0;
    drive.load_pad(2, existing.clone(), "existing");
    press(&mut drive, 3);
    target(&drive, "SWAP");
    check_silent(&press(&mut drive, 3));
    assert!(Arc::ptr_eq(
        drive.app.pads.sound[0].as_ref().unwrap(),
        &existing
    ));
    assert!(Arc::ptr_eq(
        drive.app.pads.sound[2].as_ref().unwrap(),
        &source
    ));
    assert!(drive.app.pad_in_hand.is_none());

    let files = TransitionFiles::new();
    let mut file = reach_drive(panel::View::Files, source);
    file.app.loader = Some(loader::start(48_000));
    file.app.browser.dir = files.0.clone();
    file.app.browser.reread();
    file.app.browser.selected = file
        .app
        .browser
        .entries
        .iter()
        .position(|entry| entry.name == "tone.wav")
        .unwrap();
    reveal(&mut file);
    press(&mut file, 1);
    target(&file, "FILE");
    check_silent(&press(&mut file, 2));
    while file.app.pads.loading[1].is_some() {
        let (_, replies) = file.app.loader.as_ref().unwrap();
        let reply = replies
            .recv_timeout(std::time::Duration::from_millis(500))
            .unwrap();
        file.app.loader_reply(reply);
    }
    assert_eq!(
        file.app.pads.file[1].as_deref(),
        Some(files.0.join("tone.wav").as_path())
    );
    assert!(file.app.browser.in_hand.is_none());
}

/// Scenario "Assign applies from the target row" (command-panel).
#[test]
fn assign_applies_from_the_target_row() {
    let source = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive(panel::View::Chop, source.clone());
    reveal(&mut drive);
    press(&mut drive, 4);
    target(&drive, "ASSIGN");
    press(&mut drive, 3);
    assert!(drive.app.plan.is_some());
    assert!(drive.app.pads.link[2].is_none());

    press(&mut drive, 15);
    assert_eq!(
        drive.app.panel_snapshot().unwrap().labels[14].as_deref(),
        Some("target")
    );
    press(&mut drive, 14); // Undo the planned placement through the command menu.
    press(&mut drive, 15);
    target(&drive, "ASSIGN");
    press(&mut drive, 3);
    drive.bytes(SMC, &[0x99, 37, 100]); // Playing-layer P14.
    drive.bytes(SMC, &[0x89, 37, 0]);
    drive.frame();
    assert!(!drive.app.panel.visible());
    assert!(drive.app.plan.is_some());

    reveal(&mut drive);
    target(&drive, "ASSIGN");
    press(&mut drive, 16);
    assert!(drive.app.plan.is_none());
    assert!(drive.app.pads.link[2].is_some());
    assert!(drive.app.pads.link[13].is_some());

    assert_eq!(drive.app.screen, Screen::Pads);

    let mut cancel = reach_drive(panel::View::Chop, source);
    reveal(&mut cancel);
    press(&mut cancel, 4);
    target(&cancel, "ASSIGN");
    press(&mut cancel, 13);
    assert!(cancel.app.plan.is_none());
    assert_eq!(cancel.app.panel_snapshot().unwrap().view, "CHOP");
}

/// Scenario "Target cells retain connection and release priority" (command-panel).
#[test]
fn target_cells_retain_connection_and_release_priority() {
    const SECOND: &str = "SINCO SMC-PAD-Master 32:1";
    let source = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive(panel::View::Pads, source);
    drive.app.pads.gate[0] = true;
    drive.bytes(SMC, &[0x99, 48, 100]);
    drive.frame();
    assert!(drive.app.pads.held[0]);
    reveal(&mut drive);
    press(&mut drive, 2);
    drive.midi().recorded_connect(SECOND);
    drive.frame();
    drive.bytes(SECOND, &[0x99, panel_note(2), 100]);
    drive.bytes(SECOND, &[0x89, panel_note(2), 0]);
    drive.frame();
    assert!(drive.app.copy_in_hand.is_some());
    assert!(drive.app.pads.sound[1].is_none());
    assert_eq!(drive.app.panel.owner(), Some(SMC));

    drive.bytes(SMC, &[0x89, 48, 0]);
    drive.frame();
    assert!(!drive.app.pads.held[0]);
    press(&mut drive, 2);
    assert!(drive.app.pads.sound[1].is_some());
}

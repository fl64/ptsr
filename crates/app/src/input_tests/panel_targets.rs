use super::*;

fn press(drive: &mut Drive, slot: u8) -> Frame {
    cell(drive, panel_note(slot))
}

fn reveal(drive: &mut Drive) {
    if !drive.app.panel.visible() {
        press(drive, 1);
    }
    assert_eq!(drive.app.panel.owner(), Some(SMC));
}

fn target(drive: &Drive, name: &str) {
    let panel = drive.app.panel_snapshot().unwrap();
    assert!(panel.view.starts_with(name), "{}", panel.view);
    assert_eq!(panel.labels[12].as_deref(), Some("cancel"));
    assert_eq!(panel.labels[13].as_deref(), Some("undo"));
    assert_eq!(panel.labels[14].as_deref(), Some("menu"));
}

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

fn wait_loads(drive: &mut Drive) {
    while drive.app.pads.loading.iter().any(Option::is_some) {
        let (_, replies) = drive.app.loader.as_ref().unwrap();
        let reply = replies
            .recv_timeout(std::time::Duration::from_millis(500))
            .unwrap();
        drive.app.loader_reply(reply);
    }
}

fn check_silent(frame: &Frame) {
    assert!(
        !frame
            .commands()
            .into_iter()
            .any(|cmd| matches!(cmd, Command::PadPress { .. }))
    );
}

/// Scenario "All sixteen pads take an item in hand" (command-panel).
#[test]
fn all_sixteen_pads_take_an_item_in_hand() {
    let source = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let existing = Arc::new(Sample::mono(vec![0.4; 4_800]));

    let files = TransitionFiles::new();
    let mut file = reach_drive(panel::View::Files, source.clone());
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
    for number in [1u8, 16] {
        reveal(&mut file);
        press(&mut file, 1);
        target(&file, "FILE");
        check_silent(&hit(&mut file, number));
        wait_loads(&mut file);
        assert_eq!(
            file.app.pads.file[abs(usize::from(number))].as_deref(),
            Some(files.0.join("tone.wav").as_path()),
            "device pad {number}"
        );
        assert!(file.app.browser.in_hand.is_none());
    }

    let mut drive = reach_drive(panel::View::Pads, source.clone());
    drive.load_pad(abs(1), existing.clone(), "existing");
    reveal(&mut drive);
    press(&mut drive, 2);
    target(&drive, "COPY");
    check_silent(&hit(&mut drive, 16));
    assert!(Arc::ptr_eq(
        drive.app.pads.sound[abs(16)].as_ref().unwrap(),
        &source
    ));
    // An occupied target asks; pad 1 keeps, pad 4 replaces.
    drive.app.pads.selected = 0;
    reveal(&mut drive);
    press(&mut drive, 2);
    hit(&mut drive, 1);
    assert!(matches!(drive.app.ask, Some(Ask::CopyPad { .. })));
    hit(&mut drive, 1);
    assert!(Arc::ptr_eq(
        drive.app.pads.sound[abs(1)].as_ref().unwrap(),
        &existing
    ));
    hit(&mut drive, 1);
    hit(&mut drive, 4);
    assert!(Arc::ptr_eq(
        drive.app.pads.sound[abs(1)].as_ref().unwrap(),
        &source
    ));
    assert!(drive.last_notice().text.starts_with("copy placed on"));

    drive.app.pads.selected = 0;
    drive.load_pad(abs(1), existing.clone(), "existing");
    reveal(&mut drive);
    press(&mut drive, 3);
    target(&drive, "SWAP");
    check_silent(&hit(&mut drive, 1));
    assert!(Arc::ptr_eq(
        drive.app.pads.sound[0].as_ref().unwrap(),
        &existing
    ));
    assert!(drive.app.pad_in_hand.is_none());
    assert!(drive.last_notice().text.starts_with("swapped"));
}

/// Scenario "Assign places slices on the playing layer and applies on a hold" (command-panel).
#[test]
fn assign_places_slices_on_the_playing_layer_and_applies_on_a_hold() {
    let source = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive_from(hold_drive(), panel::View::Chop, source.clone());
    hold(&mut drive, true);
    cell(&mut drive, panel_note(4));
    target(&drive, "ASSIGN");
    assert_eq!(
        drive.app.panel_snapshot().unwrap().labels[15].as_deref(),
        Some("apply")
    );
    hold(&mut drive, false);
    check_silent(&hit(&mut drive, 1));
    check_silent(&hit(&mut drive, 16));
    assert!(drive.app.plan.is_some());
    assert!(drive.app.pads.link[abs(1)].is_none());
    hold(&mut drive, true);
    cell(&mut drive, panel_note(16));
    hold(&mut drive, false);
    assert!(drive.app.plan.is_none());
    assert!(drive.app.pads.link[abs(1)].is_some());
    assert!(drive.app.pads.link[abs(16)].is_some());
    assert!(drive.last_notice().text.contains("→"));

    let mut cancel = reach_drive_from(hold_drive(), panel::View::Chop, source);
    hold(&mut cancel, true);
    cell(&mut cancel, panel_note(4));
    hold(&mut cancel, false);
    hit(&mut cancel, 1);
    hold(&mut cancel, true);
    cell(&mut cancel, panel_note(13));
    hold(&mut cancel, false);
    assert!(cancel.app.plan.is_none());
    assert!(cancel.app.pads.link[abs(1)].is_none());
}

/// Scenario "A hold in target mode shows only the mode commands" (command-panel).
#[test]
fn a_hold_in_target_mode_shows_only_the_mode_commands() {
    let source = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive_from(hold_drive(), panel::View::Pads, source);
    hold(&mut drive, true);
    cell(&mut drive, panel_note(2));
    let panel = drive.app.panel_snapshot().unwrap();
    for (index, label) in panel.labels.iter().enumerate() {
        let want = match index {
            12 => Some("cancel"),
            13 => Some("undo"),
            14 => Some("menu"),
            _ => None,
        };
        assert_eq!(label.as_deref(), want, "cell {}", index + 1);
    }
    let before = drive.seen();
    cell(&mut drive, panel_note(1));
    assert_eq!(drive.seen(), before, "a dim cell does nothing");
    hold(&mut drive, false);
    assert!(
        drive.app.copy_in_hand.is_some(),
        "a release keeps the item in hand"
    );
}

/// Scenario "Target pads keep their connection and release" (command-panel).
#[test]
fn target_pads_keep_their_connection_and_release() {
    const SECOND: &str = "SINCO SMC-PAD-Master 32:1";
    let source = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive(panel::View::Pads, source);
    drive.app.pads.gate[0] = true;
    drive.app.publish_midi(&drive.shared);
    drive.bytes(SMC, &[0x99, 48, 100]);
    drive.frame();
    assert!(drive.app.pads.held[0]);
    reveal(&mut drive);
    press(&mut drive, 2);
    drive.midi().recorded_connect(SECOND);
    drive.frame();
    drive.bytes(SECOND, &[0x99, panel_note(13), 100]);
    drive.bytes(SECOND, &[0x89, panel_note(13), 0]);
    drive.frame();
    assert!(
        drive.app.copy_in_hand.is_some(),
        "another panel cannot cancel"
    );
    assert!(drive.app.pads.sound[1].is_none());

    drive.bytes(SMC, &[0x89, 48, 0]);
    drive.frame();
    assert!(!drive.app.pads.held[0]);
    hit(&mut drive, 14);
    assert!(drive.app.pads.sound[1].is_some());
}

/// Scenario "Assign apply names the pads" (controller-reach).
#[test]
fn assign_apply_names_the_pads() {
    let source = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive_from(hold_drive(), panel::View::Chop, source);
    hold(&mut drive, true);
    cell(&mut drive, panel_note(4));
    hold(&mut drive, false);
    for number in [5, 6, 9] {
        hit(&mut drive, number);
    }
    hold(&mut drive, true);
    cell(&mut drive, panel_note(16));
    hold(&mut drive, false);
    let text = drive.last_notice().text.clone();
    for pad in ["1→1-5", "2→1-6", "3→1-9"] {
        assert!(text.contains(pad), "{pad}: {text}");
    }
}

/// Scenario "LEFT and RIGHT change the Assign bank" (command-panel).
#[test]
fn left_and_right_change_the_assign_bank() {
    let source = Arc::new(Sample::mono(vec![0.2; 4_800]));
    let mut drive = reach_drive_from(hold_drive(), panel::View::Chop, source);
    hold(&mut drive, true);
    cell(&mut drive, panel_note(4));
    hold(&mut drive, false);
    assert_eq!(drive.app.pads.bank, 0);
    list_button(&mut drive, true);
    assert_eq!(drive.app.pads.bank, 1);
    hit(&mut drive, 1);
    assert!(drive.app.plan.is_some());
    hold(&mut drive, true);
    cell(&mut drive, panel_note(16));
    hold(&mut drive, false);
    assert!(
        drive.app.pads.link[16 + abs(1)].is_some(),
        "the slice lies on bank 2"
    );
    list_button(&mut drive, false);
}

/// Scenario "Assign places slices on the playing layer and applies on a hold" (command-panel),
/// across chops of one to three banks, empty, half and full target banks, by hand and in
/// order. Each run moves to bank 2 with RIGHT, places, applies and answers a replace
/// question with device pad 4. The mode line never names keys after SMC-PAD input.
#[test]
fn assign_works_for_any_chop_size_bank_occupancy_and_order() {
    let mut failures = Vec::new();
    for slices in [4usize, 20, 40] {
        for occupied in [0usize, 8, 16] {
            for in_order in [false, true] {
                let case = format!("{slices} slices, {occupied} pads taken, in order {in_order}");
                let sound = Arc::new(Sample::mono(vec![0.2; 48_000]));
                let mut drive = hold_drive();
                for pad in 16..16 + occupied {
                    drive.load_pad(pad, sound.clone(), "taken");
                }
                drive.prepare_chop(sound, slices);
                // Slice pages follow LEFT and RIGHT before Assign.
                for _ in 0..3 {
                    list_button(&mut drive, true);
                }
                list_button(&mut drive, false);
                list_button(&mut drive, false);
                list_button(&mut drive, false);
                hold(&mut drive, true);
                cell(&mut drive, panel_note(4));
                if in_order {
                    cell(&mut drive, panel_note(4)); // Order.
                }
                hold(&mut drive, false);
                if drive.app.plan.is_none() {
                    failures.push(format!("{case}: Assign does not open"));
                    continue;
                }
                list_button(&mut drive, true);
                if drive.app.pads.bank != 1 {
                    failures.push(format!("{case}: RIGHT does not reach bank 2"));
                    continue;
                }
                let hits: Vec<u8> = if in_order {
                    vec![13]
                } else {
                    [13u8, 14, 15].into_iter().take(slices.min(3)).collect()
                };
                for number in &hits {
                    hit(&mut drive, *number);
                }
                if let Some(line) = drive.app.mode_line(Instant::now())
                    && (line.contains("Enter") || line.contains("Esc"))
                {
                    failures.push(format!("{case}: keyboard words: {line}"));
                }
                hold(&mut drive, true);
                cell(&mut drive, panel_note(16));
                hold(&mut drive, false);
                if drive.app.ask.is_some() {
                    hit(&mut drive, 4); // Device pad 4 answers yes.
                }
                let expected = if in_order {
                    slices.min(PADS - (16 + abs(13)))
                } else {
                    hits.len()
                };
                let linked = (0..PADS)
                    .filter(|&pad| drive.app.pads.link[pad].is_some())
                    .count();
                if drive.app.plan.is_some() || drive.app.ask.is_some() {
                    failures.push(format!("{case}: Apply did not finish"));
                } else if linked != expected {
                    failures.push(format!(
                        "{case}: {linked} pads hold slices, want {expected}"
                    ));
                } else if !drive.last_notice().text.contains('→') {
                    failures.push(format!("{case}: the result names no pads"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

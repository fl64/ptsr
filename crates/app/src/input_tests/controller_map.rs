//! The F1 Controller tab (98j) through the production keyboard and recorded MIDI routes.

use super::*;
use crate::help::Page;

const SIZES: [(u16, u16); 2] = [(80, 26), (120, 40)];

/// A message of a control without a role: it only makes the device the last one used.
fn touch(drive: &mut Drive, port: &str) {
    drive.bytes(port, &[0xB0, 20, 0]);
    drive.frame();
}

fn open(drive: &mut Drive, size: (u16, u16)) {
    drive.app.resize_help(size.0, size.1);
    if !drive.app.help_open {
        drive.tap(KeyCode::F(1), M::NONE);
    }
    while drive.app.help_overlay.page != Page::Controller {
        drive.tap(KeyCode::Tab, M::NONE);
    }
}

fn view(drive: &mut Drive) -> &tui::Controller {
    let rows = full_help(&drive.keymap, drive.app.screen, true);
    drive.app.prepare_key_help(&drive.keymap, &rows, None);
    drive.app.help_overlay.controller_view.as_ref().unwrap()
}

fn screen(drive: &mut Drive, size: (u16, u16)) -> Vec<String> {
    drive.app.resize_help(size.0, size.1);
    let rows = full_help(&drive.keymap, drive.app.screen, true);
    drive.app.prepare_key_help(&drive.keymap, &rows, None);
    f1_message_screen(drive, size.0, size.1)
}

fn current(drive: &mut Drive) -> String {
    let v = view(drive);
    v.devices.get(v.current).cloned().unwrap_or_default()
}

/// (control, label) of each drawn control.
fn cells(drive: &mut Drive) -> Vec<(String, String)> {
    match &view(drive).body {
        tui::ControllerBody::Drawing { rows, .. } => rows
            .iter()
            .flatten()
            .flatten()
            .map(|c| (c.name.clone(), c.label.clone()))
            .collect(),
        _ => panic!("no drawing"),
    }
}

fn label(drive: &mut Drive, control: &str) -> String {
    cells(drive)
        .into_iter()
        .find(|(n, _)| n == control)
        .unwrap()
        .1
}

fn xsession(drive: &Drive) -> input::midi::Profile {
    drive
        .app
        .controller_devices()
        .into_iter()
        .find(|d| d.name == "X-Session Pro")
        .unwrap()
        .profile
}

#[test]
fn controller_tab_draws_the_device_top() {
    for size in SIZES {
        let mut drive = Drive::new();
        touch(&mut drive, XS_PORT);
        open(&mut drive, size);
        assert_eq!(current(&mut drive), "X-Session Pro");
        let rows = screen(&mut drive, size);
        let at = |text: &str| {
            rows.iter()
                .enumerate()
                .find_map(|(y, row)| row.find(text).map(|x| (y, x)))
                .unwrap_or_else(|| panic!("{text} in {rows:#?}"))
        };
        // places of the record: deck A left, the crossfader in the middle, deck B right
        let (k1, k10, cross) = (at("(volume)"), at("  k10   "), at("<-> "));
        assert!(k1.1 < cross.1 && cross.1 < k10.1, "{rows:#?}");
        assert!(k1.0 < cross.0, "{rows:#?}");
        assert!(at("=-=").0 < cross.0);
        // each label is the action of its control
        let profile = xsession(&drive);
        for (name, label) in cells(&mut drive) {
            let short = profile
                .control_action(Screen::Pads, false, &name)
                .map_or("-", input::short_label);
            assert_eq!(&label[1..label.len() - 1], short, "{name}");
        }
        // b2 shows the files screen; a raw press of b2 opens it
        assert_eq!(label(&mut drive, "b2"), "[files]");
        drive.tap(KeyCode::Esc, M::NONE);
        drive.bytes(XS_PORT, &[0x90, 44, 127]);
        drive.bytes(XS_PORT, &[0x80, 44, 0]);
        drive.frame();
        assert_eq!(drive.app.screen, Screen::Files);
    }
}

/// Scenario "Encoder roles follow the current screen" (screen-layout).
#[test]
fn smc_controller_view_shows_setup_status() {
    let mut drive = Drive::new();
    let port = "SINCO SMC-PAD-Master 28:1";
    drive.app.profiles.push(
        input::midi::Profile::from_toml(
            "smc-pad.toml",
            include_str!("../../../../config/devices/smc-pad.toml"),
        )
        .unwrap(),
    );
    drive.app.midi_event(crate::midi::Event::Connected {
        port: port.into(),
        profile: "SMC-PAD".into(),
    });
    drive.app.help_overlay.controller.last_port = Some(port.into());
    open(&mut drive, (80, 26));
    assert_eq!(current(&mut drive), "SMC-PAD");
    assert!(
        view(&mut drive)
            .detail
            .starts_with("encoders unavailable ·")
    );
    drive.app.midi_event(crate::midi::Event::SetupStatus {
        ready: false,
        message: "USB read timed out".into(),
    });
    assert!(
        view(&mut drive)
            .detail
            .starts_with("encoders unavailable ·")
    );
    drive.app.midi_event(crate::midi::Event::SetupStatus {
        ready: true,
        message: "SMC-PAD encoders ready".into(),
    });
    assert!(view(&mut drive).detail.starts_with("encoders ready ·"));
    let rows = screen(&mut drive, (80, 26)).join("\n");
    assert!(rows.contains("encoders ready"), "{rows}");
}

/// Scenario "A1 encoders follow the open controls on every screen" (controller-reach).
#[test]
fn smc_controller_tab_names_a1_encoder_roles_until_the_controls_close() {
    let port = "SINCO SMC-PAD-Master 28:1";
    let role = |drive: &mut Drive, control: &str| match &view(drive).body {
        tui::ControllerBody::Drawing { rows, .. } => rows
            .iter()
            .flatten()
            .flatten()
            .find(|cell| cell.name == control)
            .unwrap()
            .label
            .clone(),
        tui::ControllerBody::List { rows, .. } => rows
            .iter()
            .find(|(name, _)| name.starts_with(&format!("{control} ")))
            .unwrap()
            .1
            .clone(),
        tui::ControllerBody::Empty { .. } => panic!("no controller"),
    };
    for size in SIZES {
        let mut drive = Drive::smc_offline();
        touch(&mut drive, port);
        open(&mut drive, size);
        let knob_1 = role(&mut drive, "e1");
        assert!(knob_1.contains('-') && !knob_1.contains("pad"), "{knob_1}");
        drive.tap(KeyCode::Esc, M::NONE);
        drive.tap(KeyCode::Char('n'), M::ALT);
        assert!(drive.app.pattern_panel.is_some());
        open(&mut drive, size);
        assert!(role(&mut drive, "e2").contains("tempo"));
        assert!(role(&mut drive, "e5").contains("bars"));
        assert!(role(&mut drive, "e3").contains("quant"));
        assert!(role(&mut drive, "e4").contains("str"));
        drive.tap(KeyCode::Esc, M::NONE);
        drive.tap(KeyCode::Char('n'), M::ALT);
        open(&mut drive, size);
        let knob_1 = role(&mut drive, "e1");
        assert!(knob_1.contains('-') && !knob_1.contains("pad"), "{knob_1}");
    }
}

/// Scenario "Last knob input marks its bank" (screen-layout).
#[test]
fn last_knob_input_marks_its_bank() {
    let port = "SINCO SMC-PAD-Master 28:1";
    let mut drive = Drive::smc_offline();
    drive.midi().recorded_smc_setup();
    drive.frame();
    assert_eq!(
        drive.app.smc_knob_strip(Instant::now()).unwrap().last_bank,
        None
    );

    drive.bytes(port, &[0xB0, 30, 65]);
    drive.frame();
    assert_eq!(
        drive.app.smc_knob_strip(Instant::now()).unwrap().last_bank,
        Some(0)
    );
    drive.bytes(port, &[0xB0, 38, 64]);
    drive.frame();
    assert_eq!(
        drive.app.smc_knob_strip(Instant::now()).unwrap().last_bank,
        Some(0)
    );
    let volume = drive.app.pads.volume_db[0];
    drive.bytes(port, &[0xB0, 44, 65]);
    drive.frame();
    let strip = drive.app.smc_knob_strip(Instant::now()).unwrap();
    assert_eq!(strip.last_bank, Some(1));
    assert_eq!(strip.roles[1][6], ("-", false));
    assert_eq!(drive.app.pads.volume_db[0], volume);

    let connection = drive.midi().recorded_connection(port).unwrap();
    drive.midi().recorded_disconnect(port);
    drive.frame();
    assert!(drive.app.smc_knob_strip(Instant::now()).is_none());
    drive.midi().recorded_connect(port);
    drive.midi().recorded_smc_setup();
    drive.frame();
    drive.midi().recorded_bytes_from(
        port,
        &[0xB0, 45, 65],
        &drive.project,
        Some(crate::input_sources::Stamp {
            connection,
            at: Instant::now(),
        }),
    );
    drive.frame();
    assert_eq!(
        drive.app.smc_knob_strip(Instant::now()).unwrap().last_bank,
        None
    );
}

#[test]
fn controller_labels_follow_the_screen_and_the_edit() {
    let mut drive = Drive::new();
    touch(&mut drive, XS_PORT);
    open(&mut drive, (80, 26));
    assert_eq!(label(&mut drive, "k1"), "(volume)");
    assert_eq!(label(&mut drive, "k6"), "(-)");
    drive.tap(KeyCode::Esc, M::NONE);
    drive.app.goto(Screen::Chop);
    open(&mut drive, (80, 26));
    assert_eq!(label(&mut drive, "k1"), "(start)");
    assert_eq!(label(&mut drive, "k12"), "(zoom)");
    drive.tap(KeyCode::Esc, M::NONE);
    drive.tap(KeyCode::F(2), M::NONE);
    drive.tap(KeyCode::Enter, M::NONE);
    assert!(drive.app.edit);
    open(&mut drive, (80, 26));
    assert_eq!(label(&mut drive, "k1"), "(knob1)");
    assert_eq!(label(&mut drive, "b7"), "[back]");
    drive.tap(KeyCode::Esc, M::NONE);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(!drive.app.edit);
    // a learned binding stands over the profile role
    learn::assign(
        &mut drive.app.learned,
        "X-Session Pro",
        input::Group::Pads,
        input::midi::Key::Cc(24),
        "pad.pitch",
    );
    drive.app.sync_learned("X-Session Pro", input::Group::Pads);
    drive.frame();
    open(&mut drive, (80, 26));
    assert_eq!(label(&mut drive, "k1"), "(pitch)");
}

#[test]
fn arrows_select_controls_on_the_drawing() {
    let mut drive = Drive::new();
    touch(&mut drive, XS_PORT);
    open(&mut drive, (80, 26));
    assert!(
        view(&mut drive)
            .detail
            .starts_with("k1 · knob · pad.volume")
    );
    let mut walk = |code, expect: &str| {
        drive.tap(code, M::NONE);
        let detail = view(&mut drive).detail.clone();
        assert!(detail.starts_with(expect), "{code:?}: {detail}");
    };
    walk(KeyCode::Right, "k4 ·");
    // the empty middle places are skipped
    walk(KeyCode::Right, "k7 ·");
    walk(KeyCode::Right, "k10 ·");
    walk(KeyCode::Right, "k10 ·");
    walk(KeyCode::Down, "k11 ·");
    walk(KeyCode::Down, "k12 ·");
    // no control under k12: the nearest one of the next row
    walk(KeyCode::Down, "b3 · button");
    walk(KeyCode::Down, "pitch_b · fader");
    walk(KeyCode::Down, "b5 ·");
    walk(KeyCode::Down, "b9 ·");
    walk(KeyCode::Down, "b9 ·");
    walk(KeyCode::Left, "crossfade · crossfader");
    // b1 and b5 are both 3 places away: the left one wins
    walk(KeyCode::Up, "b1 ·");
    walk(KeyCode::Up, "fader_a ·");
}

#[test]
fn selection_does_not_act() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.load_pad(0, Arc::new(Sample::mono(vec![0.1; 4096])), "gate");
        drive.frame();
        assert_full_hit(&drive.pad_press(source, 0), 0);
        open(&mut drive, (80, 26));
        let before = drive.seen();
        for code in [KeyCode::Down, KeyCode::Right, KeyCode::Up, KeyCode::Left] {
            assert!(drive.tap(code, M::NONE).commands().is_empty(), "{code:?}");
            assert_eq!(drive.seen(), before);
        }
        assert_only_release(&drive.pad_release(source, 0), 0);
        assert!(!drive.app.pads.held[0]);
    }
}

#[test]
fn help_page_keys_change_the_device() {
    let mut drive = Drive::new();
    touch(&mut drive, PAD_PORT);
    open(&mut drive, (80, 26));
    assert_eq!(current(&mut drive), "Recorded pads");
    drive.tap(KeyCode::PageDown, M::ALT);
    assert_eq!(current(&mut drive), "X-Session Pro");
    let row = screen(&mut drive, (80, 26))
        .into_iter()
        .find(|r| r.contains("Recorded pads"))
        .unwrap();
    assert!(row.contains("[X-Session Pro]"), "{row}");
    drive.tap(KeyCode::PageUp, M::ALT);
    assert_eq!(current(&mut drive), "Recorded pads");
    // on Help the same keys page the key table
    drive.tap(KeyCode::Tab, M::NONE);
    assert_eq!(drive.app.help_overlay.page, Page::Help);
    let rows = full_help(&drive.keymap, drive.app.screen, true);
    drive.app.prepare_key_help(&drive.keymap, &rows, None);
    assert_eq!(drive.app.help_overlay.keys_scroll, 0);
    drive.tap(KeyCode::PageDown, M::ALT);
    drive.app.prepare_key_help(&drive.keymap, &rows, None);
    assert!(drive.app.help_overlay.keys_scroll > 0);
}

#[test]
fn tab_starts_on_the_last_used_device() {
    let mut drive = Drive::new();
    touch(&mut drive, XS_PORT);
    open(&mut drive, (80, 26));
    assert_eq!(current(&mut drive), "X-Session Pro");
    drive.tap(KeyCode::Esc, M::NONE);
    touch(&mut drive, PAD_PORT);
    open(&mut drive, (80, 26));
    assert_eq!(current(&mut drive), "Recorded pads");
}

#[test]
fn two_devices_of_one_profile_have_numbers() {
    let mut drive = Drive::new();
    drive
        .midi()
        .recorded_connect("USB X-Session:USB X-Session MIDI 1 24:0");
    drive.frame();
    open(&mut drive, (80, 26));
    assert_eq!(
        view(&mut drive).devices,
        ["Recorded pads", "X-Session Pro 1", "X-Session Pro 2"]
    );
}

#[test]
fn ports_of_one_device_share_a_page() {
    let mut drive = Drive::new();
    drive.midi().recorded_disconnect(XS_PORT);
    drive.frame();
    drive
        .midi()
        .recorded_connect("USB X-Session:USB X-Session MIDI 1 20:0");
    drive
        .midi()
        .recorded_connect("USB X-Session:USB X-Session MIDI 2 20:1");
    drive.frame();
    assert_eq!(drive.app.midi_devices.len(), 3);
    open(&mut drive, (80, 26));
    assert_eq!(view(&mut drive).devices, ["Recorded pads", "X-Session Pro"]);
}

#[test]
fn device_disconnects_while_its_page_is_open() {
    let mut drive = Drive::new();
    touch(&mut drive, XS_PORT);
    open(&mut drive, (80, 26));
    assert_eq!(current(&mut drive), "X-Session Pro");
    drive.midi().recorded_disconnect(XS_PORT);
    drive.frame();
    assert_eq!(current(&mut drive), "Recorded pads");
    drive.midi().recorded_connect(XS_PORT);
    drive.frame();
    assert_eq!(view(&mut drive).devices.len(), 2);
    assert_eq!(current(&mut drive), "Recorded pads");
    drive.midi().recorded_disconnect(XS_PORT);
    drive.midi().recorded_disconnect(PAD_PORT);
    drive.frame();
    assert!(matches!(
        view(&mut drive).body,
        tui::ControllerBody::Empty { .. }
    ));
}

#[test]
fn profile_without_a_layout_shows_a_list() {
    let mut drive = Drive::new();
    touch(&mut drive, PAD_PORT);
    open(&mut drive, (80, 26));
    let list = |drive: &mut Drive| match &view(drive).body {
        tui::ControllerBody::List {
            rows,
            selected,
            scroll,
        } => (rows.clone(), *selected, *scroll),
        _ => panic!("no list"),
    };
    let (rows, selected, scroll) = list(&mut drive);
    assert_eq!(rows.len(), 21);
    assert_eq!(rows[0], ("p1 pad".into(), "pad.1 · pad 1".into()));
    assert_eq!(rows[1].0, "p2 pad");
    assert_eq!(rows[16], ("back button".into(), "ui.back · back".into()));
    assert_eq!((selected, scroll), (0, 0));
    let visible = tui::controller_list_rows(26, false);
    for _ in 0..visible {
        drive.tap(KeyCode::Down, M::NONE);
    }
    let (_, selected, scroll) = list(&mut drive);
    assert_eq!((selected, scroll), (visible, 1));
    let text = screen(&mut drive, (80, 26)).join("\n");
    assert!(text.contains(&format!("p{} pad", visible + 1)), "{text}");
    assert!(!text.contains("p1 pad"), "{text}");
}

#[test]
fn layout_too_wide_for_the_frame_shows_a_list() {
    let mut drive = Drive::new();
    touch(&mut drive, XS_PORT);
    open(&mut drive, (70, 26));
    assert!(matches!(
        view(&mut drive).body,
        tui::ControllerBody::List { .. }
    ));
    drive.app.resize_help(80, 26);
    assert!(matches!(
        view(&mut drive).body,
        tui::ControllerBody::Drawing { .. }
    ));
}

#[test]
fn no_device_shows_the_known_profiles() {
    let mut drive = Drive::new();
    drive.midi().recorded_disconnect(XS_PORT);
    drive.midi().recorded_disconnect(PAD_PORT);
    drive.frame();
    for size in SIZES {
        open(&mut drive, size);
        let text = screen(&mut drive, size).join("\n");
        assert!(text.contains("No MIDI device is connected."), "{text}");
        assert!(text.contains("Known profiles:"));
        assert!(text.contains("  Recorded pads") && text.contains("  X-Session Pro"));
    }
}

#[test]
fn controller_page_keeps_the_f1_frame() {
    for size in SIZES {
        let mut drive = Drive::new();
        touch(&mut drive, XS_PORT);
        drive.tap(KeyCode::F(1), M::NONE);
        let mut frames = Vec::new();
        for _ in 0..3 {
            let rows = screen(&mut drive, size);
            let title = rows.iter().position(|r| r.contains("HELP")).unwrap();
            let footer = rows.iter().rposition(|r| r.contains("F1/Esc")).unwrap();
            let x = |text: &str| rows[title].find(text).unwrap();
            frames.push([
                title,
                footer,
                x("HELP"),
                x("MESSAGES"),
                x("CONTROLLER"),
                rows[footer].find("Tab/LEFT/RIGHT").unwrap(),
                rows[footer].find("F1/Esc").unwrap(),
            ]);
            drive.tap(KeyCode::Tab, M::NONE);
        }
        assert_eq!(drive.app.help_overlay.page, Page::Help);
        // Controller has the page keys of Help; Messages keeps its own close hint
        assert_eq!(frames[2], frames[0]);
        assert!(
            frames.iter().all(|f| f[..6] == frames[0][..6]),
            "{frames:?}"
        );
    }
}

/// Scenario "Shifted pads warn once on the Master connection" (input-devices).
#[test]
fn shifted_pads_warn_once_on_the_master_connection() {
    const SMC: &str = "SINCO SMC-PAD-Master 28:1";
    let mut drive = Drive::smc_offline();
    drive.midi().recorded_smc_setup();
    drive.frame();
    let warning_count = |drive: &Drive| {
        drive
            .app
            .notices
            .history()
            .iter()
            .filter(|notice| {
                notice.level == Level::Warn && notice.text == crate::controller_map::SHIFT_NOTICE
            })
            .count()
    };

    drive.bytes(PAD_PORT, &[0x99, 32, 100]);
    drive.bytes(SMC, &[0x90, 32, 100]);
    drive.frame();
    assert_eq!(warning_count(&drive), 0);
    for note in [32, 33] {
        drive.bytes(SMC, &[0x99, note, 100]);
        drive.bytes(SMC, &[0x89, note, 0]);
        assert_silent_target(&drive.frame());
    }
    assert_eq!(warning_count(&drive), 1);
    assert!(!drive.app.panel.visible());
    assert_eq!(drive.app.pads.selected, 0);

    drive.midi().recorded_disconnect(SMC);
    drive.midi().recorded_connect(SMC);
    drive.frame();
    drive.bytes(SMC, &[0x99, 32, 100]);
    drive.frame();
    assert_eq!(warning_count(&drive), 2);
}

/// Scenario "SMC-PAD help names the device repeat clock" (input-devices).
#[test]
fn smc_pad_help_names_the_device_repeat_clock() {
    let mut drive = Drive::smc_offline();
    touch(&mut drive, "SINCO SMC-PAD-Master 28:1");
    for size in SIZES {
        open(&mut drive, size);
        let text = screen(&mut drive, size).join("\n");
        assert!(
            text.contains("Device Note Repeat uses its own clock."),
            "{text}"
        );
        assert!(text.contains("Use ptsr Note Repeat"), "{text}");
    }
}

/// Scenario "Help shows pad 1 bottom left" (screen-help).
#[test]
fn help_shows_pad_1_bottom_left() {
    let mut drive = Drive::smc_offline();
    touch(&mut drive, "SINCO SMC-PAD-Master 28:1");
    let size = (120, 40);
    open(&mut drive, size);
    let rows = screen(&mut drive, size);
    let text = rows.join("\n");
    let row_of = |word: &str| {
        rows.iter()
            .position(|row| row.split(|c: char| !c.is_alphanumeric()).any(|w| w == word))
            .unwrap_or_else(|| panic!("{word}: {text}"))
    };
    assert_eq!(row_of("P13"), row_of("P16"), "{text}");
    assert!(row_of("P13") < row_of("P1"), "{text}");
    assert_eq!(row_of("P1"), row_of("P4"), "{text}");
    assert!(
        text.contains("PAD BANK hold: actions of this screen"),
        "{text}"
    );
}

/// Scenario "Chop help identifies the slice and region knobs" (screen-layout).
#[test]
fn chop_help_identifies_the_slice_and_region_knobs() {
    const SMC: &str = "SINCO SMC-PAD-Master 28:1";
    for size in SIZES {
        let mut drive = Drive::smc_offline();
        drive.midi().recorded_smc_setup();
        drive.frame();
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 48_000])), 4);
        drive.app.resize_help(size.0, size.1);
        // Hold PAD BANK, LEFT to the screens view, Help (cell 8).
        drive
            .midi()
            .recorded_pad_bank(SMC, std::time::Instant::now(), true);
        drive.frame();
        drive.bytes(SMC, &[0xB0, 25, 127]);
        drive.bytes(SMC, &[0xB0, 25, 0]);
        drive.frame();
        drive.bytes(SMC, &[0x99, 63, 100]);
        drive.bytes(SMC, &[0x89, 63, 0]);
        drive.frame();
        assert_eq!(drive.app.help_overlay.page, Page::Controller);
        assert_eq!(current(&mut drive), "SMC-PAD");
        assert_eq!(
            drive.app.smc_knob_strip(Instant::now()).unwrap().operation,
            Some("B1 3–4 slice · 5–6 region"),
        );
        for (key, detail) in [
            (KeyCode::Down, "B1 K5 · Chop region start"),
            (KeyCode::Right, "B1 K6 · Chop region end"),
            (KeyCode::Down, "B1 K4 · slice end"),
            (KeyCode::Left, "B1 K3 · slice start"),
        ] {
            drive.tap(key, M::NONE);
            let text = screen(&mut drive, size).join("\n");
            assert!(text.contains(detail), "{text}");
        }
    }
}

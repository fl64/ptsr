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
                rows[footer].find("Tab/Shift+Tab").unwrap(),
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

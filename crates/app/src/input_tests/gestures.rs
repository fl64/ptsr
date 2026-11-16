use super::*;

fn midi_app() -> App {
    let mut app = crate::tests::test_app();
    app.apply(Action::ScreenSettings);
    app.settings_tab = 2;
    app.midi_open.push(input::Group::Global);
    app.midi_devices.push(("X".into(), "default".into()));
    app.profiles.push(Profile::default_for("default"));
    app
}

fn pick(app: &mut App, action: &str, hold: bool) {
    app.midi_row = app
        .midi_list()
        .iter()
        .position(|row| matches!(row, MidiListRow::Action { info } if info.name == action))
        .unwrap();
    app.apply(Action::SettingsLearn);
    if hold {
        app.apply(Action::SettingsValue(1));
    }
}

fn hit(app: &mut App, note: u8) {
    app.midi_event(midi::Event::Message {
        port: "X".into(),
        profile: "default".into(),
        msg: input::midi::Msg::NoteOn {
            ch: 1,
            note,
            velocity: 127,
        },
        meaning: String::new(),
    });
}

#[test]
fn f12_learns_tap_and_hold_on_one_button() {
    let mut app = midi_app();
    pick(&mut app, "transport.toggle", false);
    hit(&mut app, 60);
    assert!(app.learning_line().unwrap().contains("(tap)"));
    app.apply(Action::SettingsLearn);
    pick(&mut app, "metronome.toggle", true);
    hit(&mut app, 60);
    assert!(app.learning_line().unwrap().contains("(hold)"));
    app.apply(Action::SettingsLearn);
    assert_eq!(
        app.learned["default"]["global"]["note 60"],
        config::MidiBinding::Gesture {
            tap: Some("transport.toggle".into()),
            hold: "metronome.toggle".into()
        }
    );
    for (action, slot) in [("transport.toggle", "tap"), ("metronome.toggle", "hold")] {
        let at = app
            .midi_list()
            .iter()
            .position(|row| matches!(row, MidiListRow::Action { info } if info.name == action))
            .unwrap();
        assert!(
            matches!(&app.midi_screen()[at], tui::MidiRow::Action { who, .. } if who.contains(slot)),
            "{action} must show {slot}"
        );
    }
}

#[test]
fn f12_pending_hold_warns_and_cancel_keeps_both_slots() {
    let mut app = midi_app();
    learn::assign_slot(
        &mut app.learned,
        "default",
        input::Group::Global,
        input::midi::Key::Note(60),
        learn::GestureSlot::Tap,
        "transport.toggle",
    );
    learn::assign_slot(
        &mut app.learned,
        "default",
        input::Group::Global,
        input::midi::Key::Note(60),
        learn::GestureSlot::Hold,
        "metronome.toggle",
    );
    let before = app.learned.clone();
    pick(&mut app, "help.toggle", true);
    hit(&mut app, 60);
    assert!(app.learning_line().unwrap().contains("replaces metronome"));
    app.apply(Action::Back);
    assert_eq!(app.learned, before);
}

#[test]
fn f12_hold_time_row_saves_the_new_value() {
    let mut app = midi_app();
    let dir = std::env::temp_dir().join(format!("ptsr-f12-hold-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.toml");
    app.user_path = Some(path.clone());
    app.midi_row = app
        .midi_list()
        .iter()
        .position(|row| matches!(row, MidiListRow::HoldTime))
        .unwrap();
    for _ in 0..2 {
        app.apply(Action::SettingsValue(1));
    }
    assert_eq!(app.midi_gestures.hold_time_ms, 400);
    assert!(
        matches!(&app.midi_screen()[app.midi_row], tui::MidiRow::Action { who, .. } if who == "400 ms")
    );
    assert_eq!(
        config::load_from(&path).config.midi_gestures.hold_time_ms,
        400
    );
    std::fs::remove_dir_all(dir).unwrap();
}

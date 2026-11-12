//! Read-only rhythm views through the real keyboard and raw MIDI routes.

use super::*;

fn pattern() -> session::PatternState {
    session::PatternState {
        slot: "A1".into(),
        bars: 2,
        quantize: "off".into(),
        strength: 50,
        hits: vec![
            [0, 0, 100, 0],
            [1, 0, 80, 960],
            [241, 4, 90, 0],
            [9000, 127, 50, 480],
        ],
    }
}

/// Scenario "Pattern view keys change only the view".
/// Scenario "Learned pattern view buttons change only the view".
#[test]
fn pattern_view_commands_preserve_raw_data_history_and_sound_routing() {
    for controller in [false, true] {
        for kitty in [false, true] {
            for pad in ['q', 'й'] {
                let mut drive = Drive::new();
                drive.app.screen = Screen::Pads;
                drive.app.pads.pattern = Some(pattern());
                drive.bind_action(90, "pattern.view_next");
                let before = drive.app.pads.pattern.clone();
                let history = drive.app.history.len();
                let dirty = drive.app.session_dirty;
                if controller {
                    drive.bytes(PAD_PORT, &[0xb0, 90, 127]);
                    drive.bytes(PAD_PORT, &[0xb0, 90, 0]);
                } else {
                    for kind in [KeyEventKind::Press, KeyEventKind::Release] {
                        drive.app.keyboard_event(
                            &drive.keymap,
                            &KeyEvent::new_with_kind(KeyCode::Up, M::ALT, kind),
                            kitty,
                            &mut drive.down,
                            &mut drive.held,
                            &mut drive.mods,
                        );
                    }
                }
                assert!(drive.frame().commands().is_empty());
                assert_eq!(
                    drive.app.pattern_view.division.grid(),
                    engine::pattern::Grid::SixteenthTriplet
                );
                assert_eq!(drive.app.pads.pattern, before);
                assert_eq!(drive.app.history.len(), history);
                assert_eq!(drive.app.session_dirty, dirty);
                if controller {
                    drive.bytes(PAD_PORT, &[0x90, 40, 77]);
                    drive.bytes(PAD_PORT, &[0x80, 40, 0]);
                } else {
                    drive.key(KeyCode::Char(pad), M::NONE, KeyEventKind::Press);
                    drive.key(KeyCode::Char(pad), M::NONE, KeyEventKind::Release);
                }
                assert!(
                    drive
                        .frame()
                        .commands()
                        .iter()
                        .any(|command| { matches!(command, Command::PadPress { pad: 4, .. }) })
                );
                assert_eq!(drive.app.pads.pattern, before);
                assert_eq!(drive.app.pattern_strip(&drive.keymap).label, "pad Q");
                if !controller {
                    // the view stops at 1/64 and at 1/4
                    for (code, end) in [
                        (KeyCode::Up, engine::pattern::Grid::SixtyFourth),
                        (KeyCode::Down, engine::pattern::Grid::Quarter),
                    ] {
                        for _ in 0..10 {
                            for kind in [KeyEventKind::Press, KeyEventKind::Release] {
                                drive.app.keyboard_event(
                                    &drive.keymap,
                                    &KeyEvent::new_with_kind(code, M::ALT, kind),
                                    kitty,
                                    &mut drive.down,
                                    &mut drive.held,
                                    &mut drive.mods,
                                );
                            }
                        }
                        assert_eq!(drive.app.pattern_view.division.grid(), end);
                    }
                    assert_eq!(drive.app.pads.pattern, before);
                    assert_eq!(drive.app.history.len(), history);
                    assert_eq!(drive.app.session_dirty, dirty);
                }
            }
        }
    }
}

#[test]
fn pattern_view_empty_state_uses_effective_recording_keys_without_creation() {
    let mut drive = Drive::new();
    drive
        .keymap
        .overlay("[global]\n\"Ctrl+R\" = \"none\"\nF9 = \"pattern.record\"\n")
        .unwrap();
    let before = drive.app.session_dirty;
    drive.tap(KeyCode::Up, M::ALT);
    let strip = drive.app.pattern_strip(&drive.keymap);
    assert!(strip.header.contains("No A1"));
    assert!(strip.header.contains("F9"));
    assert!(!strip.header.contains("Ctrl+R"));
    assert!(strip.cells.is_empty());
    assert!(strip.playhead.is_none());
    assert!(drive.app.pads.pattern.is_none());
    assert!(drive.app.history.is_empty());
    assert_eq!(drive.app.session_dirty, before);
    assert_eq!(
        Action::parse("pattern.step_edit"),
        None,
        "no placeholder edit control"
    );
}

/// Scenario "Pattern view keys preserve owner input rules".
#[test]
fn pattern_view_keys_respect_owner_layers_removals_and_input_owners() {
    let mut drive = Drive::new();
    let (shipped, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
    drive
        .keymap
        .overlay(
            "[screens.pads]\n\"Alt+Up\" = \"none\"\nF9 = \"pattern.view_next\"\n[edit]\nF9 = \"none\"\n",
        )
        .unwrap();
    let warnings = keymap_shadow_warnings(&shipped, &drive.keymap);
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("Alt+Up on pads hides pattern.view_next"))
    );
    drive.tap(KeyCode::Up, M::ALT);
    assert_eq!(
        drive.app.pattern_view.division.grid(),
        engine::pattern::Grid::Sixteenth
    );
    drive.tap(KeyCode::F(9), M::NONE);
    assert_eq!(
        drive.app.pattern_view.division.grid(),
        engine::pattern::Grid::SixteenthTriplet
    );
    drive.app.edit = true;
    drive.tap(KeyCode::F(9), M::NONE);
    assert_eq!(
        drive.app.pattern_view.division.grid(),
        engine::pattern::Grid::SixteenthTriplet
    );
    drive.app.edit = false;
    drive.app.help_open = true;
    drive.tap(KeyCode::Down, M::ALT);
    assert_eq!(
        drive.app.pattern_view.division.grid(),
        engine::pattern::Grid::SixteenthTriplet
    );
    drive.app.help_open = false;
    drive.app.pad_naming = Some((0, String::new()));
    drive.tap(KeyCode::Down, M::ALT);
    assert_eq!(
        drive.app.pattern_view.division.grid(),
        engine::pattern::Grid::SixteenthTriplet
    );
    drive.app.pad_naming = None;
    drive.app.screen = Screen::Files;
    drive.app.apply(Action::PatternViewNext);
    assert_eq!(drive.app.screen, Screen::Files);
    assert_eq!(
        drive.app.pattern_view.division.grid(),
        engine::pattern::Grid::SixteenthTriplet
    );
    assert!(drive.last_notice().text.contains("Open Pads"));
}

/// Scenario "Pattern playback plate follows the audio position".
#[test]
fn pattern_strip_uses_only_current_ready_audio_feedback_and_follows_pages() {
    use std::sync::atomic::Ordering::Acquire;
    let mut drive = Drive::offline();
    let raw = pattern();
    drive.app.send(Command::SetPattern {
        pattern: Some(Arc::new(raw.checked().unwrap().compile())),
    });
    drive.app.pads.pattern = Some(raw);
    drive.tap(KeyCode::Up, M::ALT);
    drive.tap(KeyCode::Char(' '), M::NONE);
    drive.render_blocks(500);
    let strip = drive.app.pattern_strip(&drive.keymap);
    assert!(
        strip
            .header
            .contains("A1 · bar 1/2 beat 3 · view 1/16 triplet · quantize off · bank 1"),
        "{}",
        strip.header
    );
    // after 500 blocks of 128 frames at 120 BPM the position stays just under tick 2560:
    // the 16th 1/16T cell of 160 ticks
    assert_eq!(strip.playhead, Some(15));
    assert_eq!(strip.cells.len(), 48);
    assert_eq!(strip.all.len(), 48);
    // the plate moves on with the audio position and wraps at the loop seam
    drive.render_blocks(300);
    let later = drive.app.pattern_strip(&drive.keymap).playhead.unwrap();
    assert!(later > 15, "{later}");
    drive.render_blocks(1100);
    let wrapped = drive.app.pattern_strip(&drive.keymap).playhead.unwrap();
    assert!(wrapped < later, "{wrapped} after {later}");
    let audio = drive.app.audio.as_ref().unwrap();
    audio.project.epoch.fetch_add(1, Acquire);
    let strip = drive.app.pattern_strip(&drive.keymap);
    assert!(strip.playhead.is_none());
    assert!(
        strip.header.contains("A1 · 2 bars · view 1/16 triplet"),
        "{}",
        strip.header
    );
    drive.app.pads.pattern.as_mut().unwrap().bars = 1;
    assert!(drive.app.pattern_strip(&drive.keymap).playhead.is_none());
}

#[test]
fn pattern_view_resets_on_session_change_without_entering_saved_data() {
    let mut drive = Drive::new();
    drive.tap(KeyCode::Up, M::ALT);
    assert_eq!(
        drive.app.pattern_view.division.grid(),
        engine::pattern::Grid::SixteenthTriplet
    );
    drive.app.apply_session(session::Session::default());
    assert_eq!(
        drive.app.pattern_view.division.grid(),
        engine::pattern::Grid::Sixteenth
    );
    assert!(
        drive
            .app
            .pattern_strip(&drive.keymap)
            .header
            .contains("No A1")
    );
}

/// Scenario "Bank row shows pattern banks and their hits".
#[test]
fn bank_row_shows_pattern_banks_and_their_hits() {
    use std::sync::atomic::Ordering::Acquire;
    let mut drive = Drive::offline();
    let raw = session::PatternState {
        slot: "A1".into(),
        bars: 2,
        quantize: "off".into(),
        strength: 100,
        // pad 1 of bank 1 at zero, pad Q of bank 3 at the eleventh 1/16 step, and a
        // bank 5 row beyond the two-bar loop, which stays hidden
        hits: vec![[0, 0, 100, 0], [2400, 36, 100, 0], [9000, 64, 100, 0]],
    };
    drive.app.send(Command::SetPattern {
        pattern: Some(Arc::new(raw.checked().unwrap().compile())),
    });
    drive.app.pads.pattern = Some(raw);
    let banks = drive.app.bank_pattern();
    assert_eq!(
        banks.recorded,
        [true, false, true, false, false, false, false, false]
    );
    assert_eq!(banks.hits, [0; tui::BANKS], "stopped: nothing lights");
    drive.tap(KeyCode::Char(' '), M::NONE);
    // 480 blocks of 128 frames at 120 BPM reach tick 2457, inside the step of tick 2400
    drive.render_blocks(480);
    let banks = drive.app.bank_pattern();
    assert_eq!(banks.hits[2], 1 << 4, "bank 3 lights pad Q");
    assert_eq!(banks.hits[0], 0, "the onset at zero is in an earlier step");
    assert_eq!(drive.app.pads.bank, 0, "the open bank does not change");
    // a stale position lights no pattern pad, the underlines stay
    let audio = drive.app.audio.as_ref().unwrap();
    audio.project.epoch.fetch_add(1, Acquire);
    let banks = drive.app.bank_pattern();
    assert_eq!(banks.hits, [0; tui::BANKS]);
    assert!(banks.recorded[0] && banks.recorded[2] && !banks.recorded[4]);
}

/// Scenario "Pattern view help names view keys and marks".
#[test]
fn pattern_view_help_names_view_keys_and_marks() {
    for overlay in [
        None,
        Some("[screens.pads]\n\"Alt+Up\" = \"none\"\nF9 = \"pattern.view_next\"\n"),
    ] {
        let mut drive = Drive::new();
        if let Some(overlay) = overlay {
            drive.keymap.overlay(overlay).unwrap();
        }
        drive.app.screen = Screen::Pads;
        let help = |drive: &Drive| {
            drive.app.pattern_help_groups(
                &drive.keymap,
                help_groups(&drive.keymap, Screen::Pads, &drive.app.help_state()),
                None,
                true,
            )
        };
        let view = |groups: &[tui::HelpGroup]| {
            groups
                .iter()
                .flat_map(|group| &group.items)
                .find(|(_, text)| text == "view")
                .map(|(keys, _)| keys.clone())
        };
        assert_eq!(view(&help(&drive)), None, "no A1: no view keys");
        drive.app.pads.pattern = Some(pattern());
        let keys = view(&help(&drive)).expect("view keys with A1");
        if overlay.is_none() {
            assert!(keys.contains('↑') && keys.contains('↓'), "{keys}");
        } else {
            assert!(keys.contains("F9"), "{keys}");
            assert!(
                !keys.contains('↑'),
                "a none binding has no fallback: {keys}"
            );
        }
        let rows = drive
            .app
            .full_help_recording_keys(&drive.keymap, true, None);
        let text = rows
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        for mark in [
            "■ hit",
            "~ off grid",
            "hits in one cell",
            "plate: playback",
            "all: any pad",
            "Underlined bank number",
        ] {
            assert!(text.contains(mark), "{mark}\n{text}");
        }
    }
}

/// Scenario "Strip header names view and quantize apart".
#[test]
fn strip_header_names_view_and_quantize_apart() {
    let mut drive = Drive::new();
    drive.app.screen = Screen::Pads;
    let mut raw = pattern();
    raw.quantize = "1/4".into();
    raw.strength = 100;
    drive.app.pads.pattern = Some(raw);
    drive.app.pads.bank = 3;
    drive.app.pads.selected = 4;
    let name = "rec-20261110-110509-a-long-take";
    drive.load_pad(52, Arc::new(Sample::mono(vec![0.1; 480])), name);
    let strip = drive.app.pattern_strip(&drive.keymap);
    assert!(
        strip
            .header
            .ends_with("view 1/16 · quantize 1/4 100 % · bank 4"),
        "{}",
        strip.header
    );
    assert_eq!(strip.label, "pad Q");
    assert!(!strip.header.contains(name), "{}", strip.header);
    // quantize off has no strength
    drive.app.pads.pattern.as_mut().unwrap().quantize = "off".into();
    let strip = drive.app.pattern_strip(&drive.keymap);
    assert!(
        strip.header.contains("· quantize off ·"),
        "{}",
        strip.header
    );
}

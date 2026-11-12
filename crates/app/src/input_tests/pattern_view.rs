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

/// Scenario "Overview bank and view changes leave A1 unchanged".
/// Scenario "Overview view keys share the Pads division".
#[test]
fn overview_bank_and_view_changes_leave_a1_unchanged() {
    for controller in [false, true] {
        let mut drive = Drive::new();
        drive.app.pads.pattern = Some(pattern());
        drive.bind_action(90, "pattern.view_next");
        drive.bind_action(91, "bank.next");
        let before = drive.app.pads.pattern.clone();
        let dirty = drive.app.session_dirty;
        let history = drive.app.history.len();
        drive.tap(KeyCode::Up, M::ALT);
        assert!(
            drive
                .app
                .pattern_strip(&drive.keymap)
                .header
                .contains("view 1/16 triplet")
        );
        if controller {
            drive.bind_action(92, "screen.pattern");
            drive.bytes(PAD_PORT, &[0xb0, 92, 127]);
            drive.bytes(PAD_PORT, &[0xb0, 92, 0]);
            drive.bytes(PAD_PORT, &[0xb0, 90, 127]);
            drive.bytes(PAD_PORT, &[0xb0, 90, 0]);
            drive.bytes(PAD_PORT, &[0xb0, 91, 127]);
            drive.bytes(PAD_PORT, &[0xb0, 91, 0]);
            drive.frame();
        } else {
            drive.tap(KeyCode::F(5), M::NONE);
            drive.tap(KeyCode::Up, M::ALT);
            drive.tap(KeyCode::Char('o'), M::NONE);
        }
        assert!(drive.app.pattern_view.overview);
        assert_eq!(drive.app.pads.bank, 1);
        let view = drive.app.pattern_overview(&drive.keymap);
        assert!(view.header.contains("view 1/32"), "{}", view.header);
        assert_eq!(view.rows.len(), 128);
        assert!(
            view.rows[16..32]
                .iter()
                .flatten()
                .all(|cell| cell.count == 0)
        );
        drive.tap(KeyCode::F(2), M::NONE);
        assert!(!drive.app.pattern_view.overview);
        assert!(
            drive
                .app
                .pattern_strip(&drive.keymap)
                .header
                .contains("view 1/32")
        );
        assert_eq!(drive.app.pads.pattern, before);
        assert_eq!(drive.app.session_dirty, dirty);
        assert_eq!(drive.app.history.len(), history);
    }
}

/// Scenario "Overview hides empty banks without losing access".
/// Scenario "Overview empty-bank key keeps pad and pattern state".
/// Scenario "Overview bank navigation reaches a hidden empty bank".
#[test]
fn overview_empty_bank_switch_and_navigation_keep_a1_unchanged() {
    let mut drive = Drive::new();
    let mut raw = pattern();
    raw.hits.extend([[960, 20, 100, 0], [9000, 40, 90, 0]]);
    drive.app.pads.pattern = Some(raw.clone());
    let dirty = drive.app.session_dirty;
    let history = drive.app.history.len();
    let modes = drive.app.pads.gate;
    drive.tap(KeyCode::F(5), M::NONE);
    let visible = drive.app.pattern_overview(&drive.keymap);
    assert!(!visible.hide_empty);
    assert_ne!(visible.active_pads[0], 0);
    assert_ne!(visible.active_pads[1], 0);
    assert_eq!(visible.active_pads[2], 0, "hidden raw hit is not visible");
    drive.tap(KeyCode::Char('h'), M::ALT);
    assert!(drive.app.pattern_view.hide_empty_banks);
    drive.tap(KeyCode::Char('o'), M::NONE);
    drive.tap(KeyCode::Char('o'), M::NONE);
    assert_eq!(drive.app.pads.bank, 2);
    let selected = drive.app.pattern_overview(&drive.keymap);
    assert!(selected.hide_empty);
    assert!(
        selected.rows[32..48]
            .iter()
            .flatten()
            .all(|cell| cell.count == 0)
    );
    drive.tap(KeyCode::Char('h'), M::ALT);
    assert!(!drive.app.pattern_view.hide_empty_banks);
    drive.tap(KeyCode::F(2), M::NONE);
    drive.tap(KeyCode::Char('h'), M::ALT);
    assert!(
        !drive.app.pattern_view.hide_empty_banks,
        "Pads does not toggle the view"
    );
    assert_eq!(drive.app.pads.pattern, Some(raw));
    assert_eq!(drive.app.pads.gate, modes);
    assert_eq!(drive.app.session_dirty, dirty);
    assert_eq!(drive.app.history.len(), history);
}

/// Scenario "Learned empty-bank button changes only visibility".
#[test]
fn learned_empty_bank_button_changes_only_visibility() {
    let mut drive = Drive::new();
    assert!(input::ACTIONS.iter().any(|info| {
        info.name == "pattern.overview_empty_toggle" && info.group == input::Group::Global
    }));
    drive.bind_action(93, "pattern.overview_empty_toggle");
    let path = drive.save_registry_bindings();
    let mut restored = Drive::with_saved_bindings(false, Some(&path));
    let before = restored.app.pads.pattern.clone();
    let modes = restored.app.pads.gate;
    restored.bytes(PAD_PORT, &[0xb0, 93, 127]);
    restored.bytes(PAD_PORT, &[0xb0, 93, 0]);
    restored.frame();
    assert!(!restored.app.pattern_view.hide_empty_banks);
    restored.tap(KeyCode::F(5), M::NONE);
    restored.bytes(PAD_PORT, &[0xb0, 93, 127]);
    restored.bytes(PAD_PORT, &[0xb0, 93, 0]);
    restored.frame();
    assert!(restored.app.pattern_view.hide_empty_banks);
    restored.tap(KeyCode::F(2), M::NONE);
    restored.bytes(PAD_PORT, &[0xb0, 93, 127]);
    restored.bytes(PAD_PORT, &[0xb0, 93, 0]);
    restored.frame();
    assert!(restored.app.pattern_view.hide_empty_banks);
    assert_eq!(restored.app.pads.pattern, before);
    assert_eq!(restored.app.pads.gate, modes);
    assert_eq!(restored.app.pads.bank, 0);
}

/// Scenario "Overview opens without editing or stopping A1".
/// Scenario "Overview distinguishes absent and empty A1".
#[test]
fn overview_opens_absent_empty_and_playing_a1_without_mutation() {
    let mut drive = Drive::new();
    let dirty = drive.app.session_dirty;
    drive.tap(KeyCode::F(5), M::NONE);
    let absent = drive.app.pattern_overview(&drive.keymap);
    assert!(absent.header.contains("No A1"));
    assert_eq!(absent.rows.len(), 128);
    assert!(absent.rows.iter().all(Vec::is_empty));
    assert!(drive.app.pads.pattern.is_none());
    drive.tap(KeyCode::F(2), M::NONE);
    drive.app.pads.pattern = Some(session::PatternState {
        hits: Vec::new(),
        ..pattern()
    });
    drive.tap(KeyCode::F(5), M::NONE);
    let empty = drive.app.pattern_overview(&drive.keymap);
    assert_eq!(empty.rows.len(), 128);
    assert!(
        empty.rows[..16]
            .iter()
            .flatten()
            .all(|cell| cell.count == 0)
    );
    assert_eq!(drive.app.session_dirty, dirty);
    assert!(drive.app.history.is_empty());
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(!drive.app.pattern_view.overview);

    let mut playing = Drive::offline();
    let raw = pattern();
    playing.app.send(Command::SetPattern {
        pattern: Some(Arc::new(raw.checked().unwrap().compile())),
    });
    playing.app.pads.pattern = Some(raw.clone());
    playing.tap(KeyCode::Char(' '), M::NONE);
    playing.render_blocks(20);
    let before = playing.app.transport_state();
    playing.tap(KeyCode::F(5), M::NONE);
    let after = playing.app.transport_state();
    assert_eq!(after.ticks, before.ticks);
    assert_eq!(after.playing, before.playing);
    assert_eq!(playing.app.pads.pattern, Some(raw));
    assert!(playing.app.pattern_view.overview);
    playing.tap(KeyCode::Esc, M::NONE);
    assert!(!playing.app.pattern_view.overview);
}

/// Scenario "Overview pad input keeps sound ownership".
#[test]
fn overview_pad_input_keeps_sound_ownership() {
    for source in [Source::Keyboard, Source::Notes] {
        let mut drive = Drive::offline();
        let sound = Arc::new(Sample::mono(vec![0.2; 48_000]));
        drive.app.pads.sound[0] = Some(sound.clone());
        drive.app.pads.gate[0] = true;
        drive.app.send(Command::SetPadSound {
            pad: 0,
            sound: Some(sound),
        });
        drive.app.send(Command::SetPadMode {
            pad: 0,
            mode: engine::PadMode::Gate,
        });
        drive.render_blocks(1);
        drive.tap(KeyCode::F(5), M::NONE);
        drive.pad_press(source, 0);
        let sounding = drive.render_blocks(10);
        assert!(sounding.0.iter().any(|sample| sample.abs() > 0.01));
        drive.app.apply(Action::BankSelect(1));
        drive.tap(KeyCode::F(4), M::NONE);
        let released = drive.pad_release(source, 0);
        assert!(
            !released
                .commands()
                .iter()
                .any(|command| matches!(command, Command::PadPress { pad: 16, .. }))
        );
        let tail = drive.render_blocks(200);
        assert!(
            tail.0[tail.0.len() - 128..]
                .iter()
                .all(|sample| sample.abs() < 0.001)
        );
        assert_eq!(drive.app.pads.bank, 1);
        assert!(!drive.app.pattern_view.overview);
    }
}

/// Scenario "Learned overview button follows screen and learning rules".
#[test]
fn learned_overview_button_survives_reload_and_releases_without_action() {
    let mut drive = Drive::new();
    assert!(
        input::ACTIONS
            .iter()
            .any(|info| info.name == "screen.pattern" && info.group == input::Group::Global)
    );
    let before = drive.app.learned.clone();
    drive.bind_action(92, "screen.pattern");
    let path = drive.save_registry_bindings();
    let mut restored = Drive::with_saved_bindings(false, Some(&path));
    assert_ne!(restored.app.learned, before);
    restored.bytes(PAD_PORT, &[0xb0, 92, 127]);
    restored.frame();
    assert!(restored.app.pattern_view.overview);
    restored.app.apply(Action::ScreenPads);
    restored.bytes(PAD_PORT, &[0xb0, 92, 0]);
    restored.frame();
    assert!(
        !restored.app.pattern_view.overview,
        "release does not reopen"
    );
}

/// Scenario "Overview help follows effective bindings".
#[test]
fn overview_help_follows_effective_bindings() {
    let mut drive = Drive::new();
    drive
        .keymap
        .overlay(
            "[global]\nF5 = \"none\"\nF8 = \"screen.pattern\"\n[screens.pads]\n\"Alt+Up\" = \"none\"\nF9 = \"pattern.view_next\"\n\"Alt+H\" = \"none\"\nF7 = \"pattern.overview_empty_toggle\"\n",
        )
        .unwrap();
    drive.tap(KeyCode::F(8), M::NONE);
    assert!(drive.app.pattern_view.overview);
    let groups = drive.app.pattern_help_groups(
        &drive.keymap,
        help_groups(&drive.keymap, Screen::Pads, &drive.app.help_state()),
        None,
        true,
    );
    let view = groups
        .iter()
        .flat_map(|group| &group.items)
        .find(|(_, label)| label == "view")
        .unwrap();
    assert!(
        view.0.contains("F9") && !view.0.contains("Alt Up"),
        "{}",
        view.0
    );
    let empty = groups
        .iter()
        .flat_map(|group| &group.items)
        .find(|(_, label)| label == "empty shown")
        .unwrap();
    assert!(empty.0.contains("F7") && !empty.0.contains("Alt H"));
    let rows = drive
        .app
        .full_help_recording_keys(&drive.keymap, true, None);
    let text = rows
        .iter()
        .map(|(_, label)| label.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        text.contains("Square: hit")
            && text.contains("current bank")
            && text.contains("Empty banks: shown")
    );
    drive.tap(KeyCode::F(1), M::NONE);
    assert!(drive.app.help_open);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.pattern_view.overview);
}

/// Scenario "Overview playhead follows audio without false motion".
#[test]
fn overview_playhead_follows_audio_without_false_motion() {
    use std::sync::atomic::Ordering::Acquire;
    let mut drive = Drive::offline();
    let raw = pattern();
    drive.app.send(Command::SetPattern {
        pattern: Some(Arc::new(raw.checked().unwrap().compile())),
    });
    drive.app.pads.pattern = Some(raw);
    drive.tap(KeyCode::Char(' '), M::NONE);
    drive.tap(KeyCode::F(5), M::NONE);
    drive.render_blocks(500);
    let early = drive.app.pattern_overview(&drive.keymap).playhead.unwrap();
    drive.render_blocks(300);
    let late = drive.app.pattern_overview(&drive.keymap).playhead.unwrap();
    assert!(late > early);
    let audio = drive.app.audio.as_ref().unwrap();
    audio.project.epoch.fetch_add(1, Acquire);
    assert!(drive.app.pattern_overview(&drive.keymap).playhead.is_none());
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
    assert!(
        drive
            .last_notice()
            .text
            .contains("Open Pads or the A1 overview")
    );
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

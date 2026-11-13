//! A1 step correction through the real keyboard and raw MIDI paths.

use super::*;

fn pattern(hits: Vec<[u32; 4]>) -> session::PatternState {
    session::PatternState {
        slot: "A1".into(),
        bars: 2,
        quantize: "off".into(),
        strength: 50,
        hits,
    }
}

/// Scenario "Step input adds at an empty cursor without sound".
#[test]
fn step_input_adds_keyboard_and_raw_midi_velocity_without_sound() {
    let mut drive = Drive::new();
    drive.app.pads.pattern = Some(pattern(vec![]));
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.key(KeyCode::Char('1'), M::NONE, KeyEventKind::Press);
    drive.key(KeyCode::Char('1'), M::NONE, KeyEventKind::Repeat);
    drive.key(KeyCode::Char('1'), M::NONE, KeyEventKind::Release);
    assert!(drive.frame().commands().is_empty());
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        vec![[0, 0, 80, 0]]
    );
    drive.tap(KeyCode::Char('l'), M::NONE);
    let shifted = drive.tap(KeyCode::Char('1'), M::SHIFT);
    assert!(shifted.commands().is_empty());
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits[1],
        [240, 0, 127, 0]
    );
    drive.tap(KeyCode::Char('l'), M::NONE);
    drive.bytes(PAD_PORT, &[0x90, 36, 93]);
    drive.bytes(PAD_PORT, &[0x90, 36, 93]);
    drive.bytes(PAD_PORT, &[0x90, 36, 0]);
    assert!(drive.frame().commands().is_empty());
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        vec![[0, 0, 80, 0], [240, 0, 127, 0], [480, 0, 93, 0]]
    );
    assert_eq!(drive.app.history.len(), 3);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.key(KeyCode::Char('1'), M::NONE, KeyEventKind::Press);
    assert!(
        drive
            .frame()
            .commands()
            .iter()
            .any(|command| matches!(command, Command::PadPress { .. }))
    );
}

/// Scenario "Step input keeps Gate length and saved tick".
#[test]
fn step_input_keeps_gate_length_and_saved_tick() {
    let mut drive = Drive::new();
    let mut raw = pattern(vec![]);
    raw.quantize = "1/4".into();
    raw.strength = 100;
    drive.app.pads.pattern = Some(raw);
    drive.app.pads.gate[0] = true;
    for _ in 0..4 {
        drive.app.pattern_view.division.change(true);
    }
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    for _ in 0..3 {
        drive.tap(KeyCode::Char('l'), M::NONE);
    }
    drive.tap(KeyCode::Char('1'), M::NONE);
    drive.tap(KeyCode::Char('2'), M::NONE);
    let raw = drive.app.pads.pattern.as_ref().unwrap();
    assert_eq!(raw.hits, vec![[180, 0, 80, 60], [180, 1, 80, 0]]);
    let compiled = raw.checked().unwrap().compile();
    assert_eq!(compiled.events()[0].tick(), 180);
    assert_eq!(compiled.events()[1].tick(), 180);
    drive.app.pattern_view.division = crate::pattern_view::Division::default();
    drive.app.pads.pattern.as_mut().unwrap().quantize = "off".into();
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        vec![[180, 0, 80, 60], [180, 1, 80, 0]]
    );
}

/// Scenario "Step input refuses a full or stale edit".
#[test]
fn step_input_refuses_full_and_stale_replacements() {
    let mut drive = Drive::new();
    let full = pattern(vec![[0, 0, 80, 0]; 4096]);
    drive.app.pads.pattern = Some(full.clone());
    let history = drive.app.history.len();
    assert!(
        drive
            .app
            .add_step_event(0, 240, 80, 0, &full)
            .unwrap_err()
            .contains("4096")
    );
    assert_eq!(drive.app.pads.pattern, Some(full));
    assert_eq!(drive.app.history.len(), history);
    let old = pattern(vec![]);
    let newer = pattern(vec![[0, 1, 90, 0]]);
    drive.app.pads.pattern = Some(newer.clone());
    assert!(!drive.app.add_step_event(0, 240, 80, 0, &old).unwrap());
    assert_eq!(drive.app.pads.pattern, Some(newer));
    assert_eq!(drive.app.history.len(), history);
}

#[test]
fn step_input_refuses_a_new_input_owner_with_a_next_action() {
    let mut drive = Drive::new();
    drive.app.pads.pattern = Some(pattern(vec![]));
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.app.pad_in_hand = Some(0);
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
    assert!(drive.app.history.is_empty());
    assert!(drive.last_notice().text.contains("Finish the current edit"));
}

/// Scenario "Correction feedback separates focus from edits".
#[test]
fn correction_feedback_separates_focus_add_remove_and_stale_target() {
    let mut drive = Drive::new();
    drive.app.pads.pattern = Some(pattern(vec![[240, 0, 90, 0], [241, 0, 70, 0]]));
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert!(drive.last_notice().text.contains("added one hit"));
    assert_eq!(drive.app.history.len(), 1);
    drive.tap(KeyCode::Char('l'), M::NONE);
    let notices = drive.app.notices.history().len();
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert_eq!(drive.app.history.len(), 1);
    assert_eq!(drive.app.notices.history().len(), notices);
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert!(drive.last_notice().text.contains("removed one hit"));
    assert_eq!(drive.app.history.len(), 2);
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    let current = drive.app.pads.pattern.clone().unwrap();
    drive.tap(KeyCode::Char('1'), M::NONE);
    drive.app.pads.pattern = Some(pattern(vec![
        [0, 0, 80, 0],
        [240, 0, 60, 0],
        [241, 0, 70, 0],
    ]));
    let history = drive.app.history.len();
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert_eq!(drive.app.history.len(), history);
    assert_ne!(drive.app.pads.pattern, Some(current));
}

/// Scenario "Correction help follows the next action".
#[test]
fn correction_help_shows_effective_undo_key_and_notice_names_action() {
    let mut drive = Drive::new();
    drive.app.pads.pattern = Some(pattern(vec![]));
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    let undo_key = |drive: &Drive| {
        drive
            .app
            .pattern_help_groups(&drive.keymap, vec![], None, true)
            .into_iter()
            .find(|group| group.name == "A1")
            .unwrap()
            .items
            .into_iter()
            .find(|(_, label)| label == "restore")
            .unwrap()
            .0
    };
    assert_eq!(undo_key(&drive), "Ctrl+Z");
    drive
        .keymap
        .overlay("[screens.pads]\n\"Ctrl+Z\" = \"none\"\n")
        .unwrap();
    assert_eq!(undo_key(&drive), "none");
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert!(drive.last_notice().text.contains("Undo removes it"));
    assert!(!drive.last_notice().text.contains("Ctrl+Z"));
}

/// Scenario "Step addition Undo restores the prior A1".
#[test]
fn step_addition_undo_removes_one_row_and_waits_for_newer_pad_edit() {
    let mut drive = Drive::new();
    drive.app.pads.pattern = Some(pattern(vec![]));
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.tap(KeyCode::Char('1'), M::NONE);
    drive.tap(KeyCode::Char('l'), M::NONE);
    drive.tap(KeyCode::Char('2'), M::NONE);
    assert_eq!(drive.app.history.len(), 2);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 2);
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        vec![[0, 0, 80, 0]]
    );
    drive.tap(KeyCode::Char('2'), M::NONE);
    assert_eq!(
        drive.app.pads.pattern_rows.as_ref().unwrap()[1]
            .id()
            .generation(),
        2
    );
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.tap(KeyCode::F(2), M::NONE);
    drive.tap(KeyCode::Char('h'), M::NONE);
    let gate = drive.app.pads.gate[1];
    drive.tap(KeyCode::Char('z'), M::ALT);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        vec![[0, 0, 80, 0]]
    );
    assert_eq!(drive.app.pads.gate[1], gate);
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    drive.tap(KeyCode::Char('z'), M::ALT);
    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
}

/// Scenario "Step addition saves without editor state".
#[test]
fn step_addition_save_refusal_keeps_undo_then_reopens_raw_row() {
    let files = TransitionFiles::new();
    let mut drive = files.drive(true);
    drive.app.pads.pattern = Some(pattern(vec![]));
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert_eq!(drive.app.history.len(), 1);
    let added = drive.app.pads.pattern.clone().unwrap();
    let path = drive.app.session_file();
    let occupied = path.with_extension("toml.tmp");
    std::fs::write(&occupied, "occupied").unwrap();
    assert!(drive.app.save_session().is_err());
    assert_eq!(drive.app.pads.pattern, Some(added.clone()));
    assert_eq!(drive.app.history.len(), 1);
    std::fs::remove_file(&occupied).unwrap();
    drive.app.save_session().unwrap();
    let saved = session::load_existing(&path).unwrap();
    assert_eq!(saved.version, 5);
    assert_eq!(saved.patterns, vec![added]);
    drive.app.apply_session(saved);
    assert!(!drive.app.pattern_view.edit);
    assert!(!drive.app.pattern_view.overview);
    assert_eq!(
        drive.app.pattern_view.division.grid(),
        engine::pattern::Grid::Sixteenth
    );
}

/// Scenario "Step addition survives publication pressure".
#[test]
fn step_additions_keep_order_behind_a_full_audio_queue() {
    use std::sync::atomic::Ordering::Acquire;
    let mut drive = Drive::offline();
    let sound = Arc::new(Sample::mono(vec![0.2; 48_000]));
    drive.load_pad(0, sound.clone(), "generated");
    drive.app.pads.gate[0] = true;
    drive.app.send(Command::SetPadSound {
        pad: 0,
        sound: Some(sound),
    });
    drive.app.send(Command::SetPadMode {
        pad: 0,
        mode: engine::PadMode::Gate,
    });
    let raw = pattern(vec![[0, 0, 100, 960]]);
    drive.app.send(Command::SetPattern {
        pattern: Some(Arc::new(raw.checked().unwrap().compile())),
    });
    drive.app.pads.pattern = Some(raw);
    drive.app.send(Command::TransportPlay);
    drive.render_blocks(1);
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    let epoch = drive.project.epoch.load(Acquire);
    for _ in 0..1024 {
        drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .unwrap();
    }
    drive.tap(KeyCode::Char('l'), M::NONE);
    drive.tap(KeyCode::Char('2'), M::NONE);
    drive.tap(KeyCode::Char('l'), M::NONE);
    drive.tap(KeyCode::Char('3'), M::NONE);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        vec![[0, 0, 100, 960], [240, 1, 80, 0], [480, 2, 80, 0]]
    );
    assert_eq!(drive.app.pending_audio.len(), 2);
    assert_eq!(drive.app.history.len(), 2);
    let tail = drive.render_blocks(10);
    assert!(
        engine::checks::peak(&tail.0) > 0.01,
        "the active Gate keeps its tail"
    );
    for _ in 0..512 {
        drive.render_blocks(1);
        drive.app.flush_audio();
        if drive.app.pending_audio.is_empty() {
            break;
        }
    }
    assert!(drive.app.pending_audio.is_empty());
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        vec![[0, 0, 100, 960], [240, 1, 80, 0]]
    );
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        vec![[0, 0, 100, 960]]
    );
}

/// Scenario "Step input adds at an empty cursor without sound".
/// Scenario "Step addition survives publication pressure".
#[test]
fn step_addition_render_has_one_timed_hit_and_silent_undo() {
    use engine::checks::{first_loud, first_non_finite, max_step, peak};
    let mut drive = Drive::offline();
    let wave = (0..12_000)
        .map(|index| {
            let attack = (index as f32 / 64.0).min(1.0);
            let release = ((12_000 - index) as f32 / 256.0).min(1.0);
            0.2 * attack * release
        })
        .collect();
    let tone = Arc::new(Sample::mono(wave));
    drive.load_pad(0, tone.clone(), "generated");
    drive.app.send(Command::SetPadSound {
        pad: 0,
        sound: Some(tone),
    });
    let mut raw = pattern(vec![]);
    raw.bars = 1;
    drive.app.send(Command::SetPattern {
        pattern: Some(Arc::new(raw.checked().unwrap().compile())),
    });
    drive.app.pads.pattern = Some(raw);
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.app.send(Command::TransportPlay);
    const BAR_BLOCKS: usize = 750;
    const BAR: usize = 96_000;
    let first = drive.render_blocks(BAR_BLOCKS);
    drive.tap(KeyCode::Char('l'), M::NONE);
    let added = drive.tap(KeyCode::Char('1'), M::NONE);
    assert!(added.commands().iter().all(|command| !matches!(
        command,
        Command::PadPress { .. } | Command::PadRelease { .. }
    )));
    let second = drive.render_blocks(BAR_BLOCKS);
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    let third = drive.render_blocks(BAR_BLOCKS);
    let mut left = first.0;
    left.extend(second.0);
    left.extend(third.0);
    let mut right = first.1;
    right.extend(second.1);
    right.extend(third.1);
    for channel in [&left, &right] {
        assert_eq!(first_non_finite(channel), None);
        assert!(peak(channel) < 1.0);
        assert!(max_step(channel) < 0.05, "click: {}", max_step(channel));
        assert!(peak(&channel[..BAR]) < 1e-5, "the empty bar is silent");
        assert!(
            peak(&channel[BAR..BAR + 6000]) < 1e-5,
            "the edit makes no immediate sound"
        );
        let onset = first_loud(channel, BAR + 6000..BAR + 6128, 1e-6).unwrap();
        assert!(onset.abs_diff(BAR + 6000) <= 1, "onset {onset}");
        assert!(peak(&channel[BAR + 7000..BAR + 9000]) > 0.01);
        assert!(
            peak(&channel[2 * BAR..]) < 1e-5,
            "Undo removes the next loop hit"
        );
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/123-step-add.wav");
    engine::offline::write_wav(&path, 48_000, &left, &right).unwrap();
}

/// Scenario "Step correction entry keeps A1 playing".
/// Scenario "Editor entry keys keep the overview route".
/// Scenario "Correction keys change the target without changing a pad".
/// Scenario "Step cursor stays independent of playback".
#[test]
fn step_correction_entry_and_cursor_keep_a1_unchanged() {
    let mut drive = Drive::new();
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    assert!(!drive.app.pattern_view.edit);
    drive.app.pads.pattern = Some(pattern(vec![[0, 0, 100, 0]]));
    drive.tap(KeyCode::F(7), M::NONE);
    assert!(drive.app.pattern_view.edit);
    let before = drive.app.pads.pattern.clone();
    let history = drive.app.history.len();
    drive.tap(KeyCode::Char('l'), M::NONE);
    assert_eq!(drive.app.pattern_view.cursor_cell, 1);
    drive.tap(KeyCode::Char('o'), M::NONE);
    assert_eq!(drive.app.pads.bank, 1);
    assert_eq!(drive.app.pattern_view.cursor_cell, 1);
    drive.tap(KeyCode::Up, M::ALT);
    assert_eq!(drive.app.pattern_view.cursor_cell, 1);
    assert_eq!(drive.app.pads.pattern, before);
    assert_eq!(drive.app.history.len(), history);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(!drive.app.pattern_view.edit);
    assert!(drive.app.pattern_view.overview);
    drive.tap(KeyCode::F(7), M::NONE);
    assert!(drive.app.pattern_view.edit);
    assert_eq!(drive.app.pattern_view.cursor_cell, 1);
    drive.tap(KeyCode::F(2), M::NONE);
    assert!(!drive.app.pattern_view.edit);
    assert!(!drive.app.pattern_view.overview);
}

/// Scenario "Step correction removes one hit without pad sound".
/// Scenario "Step removal Undo restores one hit".
/// Scenario "Correction feedback separates focus from removal".
#[test]
fn step_correction_removes_one_hit_and_undo_restores_it() {
    let mut drive = Drive::new();
    let raw = pattern(vec![[0, 0, 100, 0], [1, 0, 80, 0], [0, 1, 95, 0]]);
    drive.app.pads.pattern = Some(raw.clone());
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    let first = drive.tap(KeyCode::Char('1'), M::NONE);
    assert!(first.commands().is_empty());
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 3);
    assert_eq!(
        drive
            .app
            .pattern_view
            .selected_event
            .as_ref()
            .unwrap()
            .event
            .source,
        0
    );
    drive.tap(KeyCode::Char('k'), M::NONE);
    assert_eq!(
        drive
            .app
            .pattern_view
            .selected_event
            .as_ref()
            .unwrap()
            .event
            .source,
        1
    );
    let second = drive.tap(KeyCode::Char('1'), M::NONE);
    assert!(second.commands().is_empty());
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        vec![[0, 0, 100, 0], [0, 1, 95, 0]]
    );
    assert_eq!(drive.app.history.len(), 1);
    assert!(drive.last_notice().text.contains("one hit"));
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(drive.app.pads.pattern, Some(raw));
}

#[test]
fn pattern_undo_waits_for_a_newer_pad_edit() {
    let mut drive = Drive::new();
    let raw = pattern(vec![[0, 0, 100, 0]]);
    drive.app.pads.pattern = Some(raw.clone());
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
    drive.tap(KeyCode::Char('z'), M::ALT);
    assert_eq!(drive.app.pads.pattern, Some(raw.clone()));
    drive.tap(KeyCode::Char('1'), M::NONE);
    drive.tap(KeyCode::F(2), M::NONE);
    drive.tap(KeyCode::Char('h'), M::NONE);
    let gate = drive.app.pads.gate[0];
    drive.tap(KeyCode::Char('z'), M::ALT);
    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
    assert_eq!(drive.app.pads.gate[0], gate);
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    drive.tap(KeyCode::Char('z'), M::ALT);
    assert_eq!(drive.app.pads.pattern, Some(raw));
}

/// Scenario "Step correction chooses one of several hits".
/// Scenario "Step correction rejects a stale event identity".
#[test]
fn step_correction_rechecks_the_selected_row() {
    let mut drive = Drive::new();
    drive.app.pads.pattern = Some(pattern(vec![[0, 0, 100, 0], [0, 0, 70, 480]]));
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.tap(KeyCode::Char('1'), M::NONE);
    let replacement = pattern(vec![[0, 0, 60, 0], [0, 0, 70, 480]]);
    drive.app.pads.pattern = Some(replacement.clone());
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert_eq!(drive.app.pads.pattern, Some(replacement));
    assert!(drive.app.history.is_empty());
}

#[test]
fn a_single_hit_on_another_pad_needs_only_one_press() {
    let mut drive = Drive::new();
    drive.app.pads.pattern = Some(pattern(vec![[0, 0, 100, 0], [1, 0, 70, 0], [0, 1, 90, 0]]));
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert!(drive.app.pattern_view.selected_event.is_some());
    drive.tap(KeyCode::Char('2'), M::NONE);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits.len(), 2);
    assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits[0][1], 0);
}

#[test]
fn hidden_rows_stay_raw_and_project_switch_discards_the_choice() {
    let mut drive = Drive::new();
    let raw = pattern(vec![[0, 0, 100, 0], [9_000, 0, 80, 0]]);
    drive.app.pads.pattern = Some(raw.clone());
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        vec![[9_000, 0, 80, 0]]
    );
    drive.app.apply_session(session::Session::default());
    assert!(!drive.app.pattern_view.edit);
    assert!(drive.app.pattern_view.selected_event.is_none());
    assert_eq!(drive.app.pattern_view.cursor_cell, 0);
}

/// Scenario "Silent edit presses keep original sound ownership".
/// Scenario "Raw MIDI correction keeps command order".
/// Scenario "Keyboard and raw MIDI correct the same hit".
/// Scenario "Learned correction buttons persist".
#[test]
fn raw_midi_step_correction_orders_commands_and_keeps_pads_silent() {
    let mut drive = Drive::new();
    let raw = pattern(vec![[0, 0, 100, 0], [240, 0, 80, 0]]);
    drive.app.pads.pattern = Some(raw);
    drive.bind_action(94, "screen.pattern");
    drive.bind_action(95, "pattern.step_edit");
    drive.bind_action(96, "edit.value_inc");
    drive.bytes(PAD_PORT, &[0xb0, 94, 127]);
    drive.bytes(PAD_PORT, &[0xb0, 94, 0]);
    drive.bytes(PAD_PORT, &[0xb0, 95, 127]);
    drive.bytes(PAD_PORT, &[0xb0, 95, 0]);
    drive.bytes(PAD_PORT, &[0xb0, 96, 127]);
    drive.bytes(PAD_PORT, &[0xb0, 96, 0]);
    drive.bytes(PAD_PORT, &[0x90, 36, 110]);
    drive.bytes(PAD_PORT, &[0x90, 36, 110]);
    drive.bytes(PAD_PORT, &[0x90, 36, 0]);
    let frame = drive.frame();
    assert!(drive.app.pattern_view.edit);
    assert_eq!(drive.app.pattern_view.cursor_cell, 1);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        vec![[0, 0, 100, 0]]
    );
    assert!(frame.commands().iter().all(|command| !matches!(
        command,
        Command::PadPress { .. } | Command::PadRelease { .. }
    )));
    drive.tap(KeyCode::F(7), M::NONE);
    assert!(!drive.app.pattern_view.edit);
    drive.bytes(PAD_PORT, &[0x90, 36, 110]);
    let playing = drive.frame();
    assert!(
        playing
            .commands()
            .iter()
            .any(|command| matches!(command, Command::PadPress { .. }))
    );
    let path = drive.save_registry_bindings();
    let mut restored = Drive::with_saved_bindings(false, Some(&path));
    restored.app.pads.pattern = Some(pattern(vec![]));
    restored.bytes(PAD_PORT, &[0xb0, 94, 127]);
    restored.bytes(PAD_PORT, &[0xb0, 94, 0]);
    restored.bytes(PAD_PORT, &[0xb0, 95, 127]);
    restored.bytes(PAD_PORT, &[0xb0, 95, 0]);
    restored.frame();
    assert!(restored.app.pattern_view.edit);
}

/// Scenario "Correction keys respect owner bindings and input owners".
/// Scenario "Correction help follows the next action".
#[test]
fn correction_uses_effective_keys_and_help_keeps_its_priority() {
    let mut drive = Drive::new();
    drive.app.pads.pattern = Some(pattern(vec![[0, 0, 100, 0]]));
    drive
        .keymap
        .overlay("[screens.pads]\nF7 = \"none\"\nF9 = \"pattern.step_edit\"\n")
        .unwrap();
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    assert!(!drive.app.pattern_view.edit);
    drive.tap(KeyCode::F(9), M::NONE);
    assert!(drive.app.pattern_view.edit);
    let help = drive
        .app
        .full_help_recording_keys(&drive.keymap, true, None);
    assert!(
        help.iter()
            .any(|(key, text)| key.contains("F9") && text.contains("step correction"))
    );
    drive.tap(KeyCode::F(1), M::NONE);
    drive.tap(KeyCode::Char('l'), M::NONE);
    assert_eq!(drive.app.pattern_view.cursor_cell, 0);
    drive.tap(KeyCode::Esc, M::NONE);
    assert!(drive.app.pattern_view.edit);
    drive.app.pad_naming = Some((0, String::new()));
    drive.tap(KeyCode::Char('l'), M::NONE);
    assert_eq!(drive.app.pattern_view.cursor_cell, 0);
    drive.app.pad_naming = None;
    drive.tap(KeyCode::Char('l'), M::NONE);
    assert_eq!(drive.app.pattern_view.cursor_cell, 1);
}

/// Scenario "Raw MIDI correction rejects stale work".
#[test]
fn delayed_edit_midi_hit_stays_silent_after_exit() {
    let mut drive = Drive::new();
    let raw = pattern(vec![[0, 0, 100, 0]]);
    drive.app.pads.pattern = Some(raw.clone());
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.app.midi_event(midi::Event::PadHit {
        abs: 0,
        velocity: 1.0,
        ui_press: Some(999),
        sound: midi::PadSound::EditUnattempted,
    });
    assert_eq!(drive.app.pads.pattern, Some(raw));
    assert!(drive.app.sent.is_empty());
}

#[test]
fn sounding_gate_releases_original_pad_and_silent_key_stays_silent_after_exit() {
    let mut drive = Drive::new();
    drive.app.pads.pattern = Some(pattern(vec![[0, 17, 100, 0]]));
    drive.load_pad(0, Arc::new(Sample::mono(vec![0.2; 48_000])), "tone");
    drive.app.pads.gate[0] = true;
    drive.key(KeyCode::Char('1'), M::NONE, KeyEventKind::Press);
    assert!(
        drive
            .frame()
            .commands()
            .iter()
            .any(|command| matches!(command, Command::PadPress { pad: 0, .. }))
    );
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.tap(KeyCode::Char('o'), M::NONE);
    drive.key(KeyCode::Char('1'), M::NONE, KeyEventKind::Release);
    assert!(
        drive
            .frame()
            .commands()
            .iter()
            .any(|command| matches!(command, Command::PadRelease { pad: 0 }))
    );
    drive.key(KeyCode::Char('2'), M::NONE, KeyEventKind::Press);
    assert!(drive.frame().commands().is_empty());
    drive.tap(KeyCode::F(7), M::NONE);
    drive.key(KeyCode::Char('2'), M::NONE, KeyEventKind::Release);
    assert!(drive.frame().commands().is_empty());
}

/// Scenario "Step correction refuses conflicting work".
#[test]
fn correction_refuses_placement_and_existing_edit_without_mutation() {
    for blocker in 0..4 {
        let mut drive = Drive::new();
        let raw = pattern(vec![[0, 0, 100, 0]]);
        drive.app.pads.pattern = Some(raw.clone());
        drive.tap(KeyCode::F(5), M::NONE);
        match blocker {
            0 => drive.app.edit = true,
            1 => drive.app.pads.marking = true,
            2 => drive.app.pattern_panel = Some(0),
            _ => drive.app.pad_in_hand = Some(0),
        }
        drive.tap(KeyCode::F(7), M::NONE);
        assert!(!drive.app.pattern_view.edit, "blocker {blocker}");
        assert_eq!(drive.app.pads.pattern, Some(raw));
        assert!(drive.app.history.is_empty());
    }
}

/// Scenario "Step correction saves without editor state".
#[test]
fn step_correction_save_refusal_keeps_undo_then_reopens_raw_rows() {
    let files = TransitionFiles::new();
    let mut drive = files.drive(true);
    drive.app.pads.pattern = Some(pattern(vec![[0, 0, 100, 0], [240, 1, 90, 0]]));
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert_eq!(drive.app.history.len(), 1);
    let corrected = drive.app.pads.pattern.clone().unwrap();
    let path = drive.app.session_file();
    let occupied = path.with_extension("toml.tmp");
    std::fs::write(&occupied, "occupied").unwrap();
    assert!(drive.app.save_session().is_err());
    assert_eq!(drive.app.pads.pattern, Some(corrected.clone()));
    assert_eq!(drive.app.history.len(), 1);
    std::fs::remove_file(&occupied).unwrap();
    drive.app.save_session().unwrap();
    let saved = session::load_existing(&path).unwrap();
    assert_eq!(saved.version, 5);
    assert_eq!(saved.patterns, vec![corrected]);
    drive.app.apply_session(saved);
    assert!(!drive.app.pattern_view.edit);
    assert!(!drive.app.pattern_view.overview);
    assert_eq!(
        drive.app.pattern_view.division.grid(),
        engine::pattern::Grid::Sixteenth
    );
}

/// Scenario "Step removal survives publication pressure".
#[test]
fn two_step_removals_keep_order_behind_a_full_audio_queue() {
    use std::sync::atomic::Ordering::Acquire;
    let mut drive = Drive::offline();
    let raw = pattern(vec![[0, 0, 100, 0], [240, 1, 90, 0]]);
    drive.app.send(Command::SetPattern {
        pattern: Some(Arc::new(raw.checked().unwrap().compile())),
    });
    drive.app.pads.pattern = Some(raw.clone());
    drive.render_blocks(1);
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    let epoch = drive.project.epoch.load(Acquire);
    for _ in 0..1024 {
        drive
            .app
            .audio
            .as_mut()
            .unwrap()
            .commands
            .push(ProjectCommand {
                epoch,
                ready_at_send: true,
                command: ProjectMessage::Command(Command::SetPadGain {
                    pad: 127,
                    gain: 1.0,
                }),
            })
            .unwrap();
    }
    drive.tap(KeyCode::Char('1'), M::NONE);
    drive.tap(KeyCode::Char('l'), M::NONE);
    drive.tap(KeyCode::Char('2'), M::NONE);
    assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
    assert_eq!(drive.app.history.len(), 2);
    assert_eq!(drive.app.pending_audio.len(), 2);
    for _ in 0..512 {
        drive.render_blocks(1);
        drive.app.flush_audio();
        if drive.app.pending_audio.is_empty() {
            break;
        }
    }
    assert!(drive.app.pending_audio.is_empty());
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(
        drive.app.pads.pattern.as_ref().unwrap().hits,
        vec![[240, 1, 90, 0]]
    );
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(drive.app.pads.pattern, Some(raw));
}

#[test]
fn step_correction_render_removal_and_undo_keep_other_pad_playing() {
    use engine::checks::{first_non_finite, max_step, peak};
    let tone = |hz: f32| {
        let len = 24_000;
        let wave: Vec<f32> = (0..len)
            .map(|i| {
                let fade = ((len - i) as f32 / 2400.0).min(1.0);
                0.35 * fade * (std::f32::consts::TAU * hz * i as f32 / 48_000.0).sin()
            })
            .collect();
        Arc::new(Sample::mono(wave))
    };
    let mut drive = Drive::offline();
    let mut raw = pattern(vec![[0, 0, 127, 960], [1920, 1, 100, 480]]);
    raw.bars = 1;
    for (pad, hz) in [(0, 220.0), (1, 660.0)] {
        let sound = tone(hz);
        drive.load_pad(pad, sound.clone(), "generated");
        drive.app.send(Command::SetPadSound {
            pad,
            sound: Some(sound),
        });
    }
    drive.app.pads.gate[0] = true;
    drive.app.send(Command::SetPadMode {
        pad: 0,
        mode: engine::PadMode::Gate,
    });
    drive.app.send(Command::SetPattern {
        pattern: Some(Arc::new(raw.checked().unwrap().compile())),
    });
    drive.app.pads.pattern = Some(raw);
    drive.tap(KeyCode::F(5), M::NONE);
    drive.tap(KeyCode::F(7), M::NONE);
    drive.app.send(Command::TransportPlay);
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut add = |part: Stereo| {
        left.extend(part.0);
        right.extend(part.1);
    };
    const BAR_BLOCKS: usize = 750;
    const EXTRA_BLOCKS: usize = 32;
    add(drive.render_blocks(BAR_BLOCKS + EXTRA_BLOCKS));
    assert_eq!(drive.app.pattern_view.cursor_cell, 0);
    drive.tap(KeyCode::Char('1'), M::NONE);
    add(drive.render_blocks(BAR_BLOCKS));
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    add(drive.render_blocks(BAR_BLOCKS));
    let bar = 96_000;
    for channel in [&left, &right] {
        assert_eq!(first_non_finite(channel), None);
        assert!(peak(channel) < 1.0);
        assert!(max_step(channel) < 0.05, "click: {}", max_step(channel));
    }
    assert!(
        peak(&left[bar..bar + 4000]) > 0.02,
        "a started voice keeps playing: {}",
        peak(&left[bar..bar + 4000])
    );
    assert!(
        peak(&left[bar + 4096..bar + 8096]) > 0.02,
        "a Gate that started before removal keeps its tail"
    );
    assert!(
        peak(&left[2 * bar + 30_000..2 * bar + 34_000]) < 1e-4,
        "the removed hit leaves its next loop silent"
    );
    assert!(
        peak(&left[2 * bar + 48_000..2 * bar + 52_000]) > 0.02,
        "the other pad keeps its hit"
    );
    assert!(
        peak(&left[3 * bar..3 * bar + 4000]) > 0.02,
        "Undo restores the next onset"
    );
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/98a-step-correction.wav");
    engine::offline::write_wav(&path, 48_000, &left, &right).unwrap();
}

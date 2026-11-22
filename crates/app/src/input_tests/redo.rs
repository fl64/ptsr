use super::*;

fn undo(drive: &mut Drive, controller: bool) {
    if controller {
        hold(drive, true);
        cell(drive, panel_note(14));
        hold(drive, false);
    } else {
        drive.tap(KeyCode::Char('z'), M::CONTROL);
    }
}

fn redo(drive: &mut Drive, controller: bool) {
    if controller {
        hold(drive, true);
        if drive.app.panel_snapshot().is_some_and(|panel| panel.target) {
            cell(drive, panel_note(15));
        }
        cell(drive, panel_note(15));
        hold(drive, false);
    } else {
        drive.tap(KeyCode::Char('y'), M::CONTROL);
    }
}

fn test_sound() -> Arc<Sample> {
    Arc::new(Sample::mono(
        (0..8_192)
            .map(|frame| {
                let envelope = ((8_191 - frame).min(frame).min(240) as f32) / 240.0;
                0.25 * envelope * (frame as f32 * std::f32::consts::TAU * 220.0 / 48_000.0).sin()
            })
            .collect(),
    ))
}

/// Scenario "Redo restores a grouped pad edit and its sound".
#[test]
fn redo_restores_a_grouped_pad_edit_and_its_sound() {
    use engine::checks::{first_non_finite, max_step, peak};
    for controller in [false, true] {
        let mut drive = hold_drive();
        for pad in [0, 1] {
            let sound = test_sound();
            drive.load_pad(pad, sound.clone(), "tone");
            drive.app.pads.gate[pad] = false;
            drive.app.pads.pitch[pad] = 0;
            drive.app.send(Command::SetPadSound {
                pad,
                sound: Some(sound),
            });
        }
        drive.tap(KeyCode::Char('a'), M::CONTROL);
        assert_eq!(drive.app.pads.marked.len(), 16);
        for _ in 0..12 {
            drive.tap(KeyCode::Char('j'), M::NONE);
        }
        let edited = drive.app.pads.volume_db[0];
        assert!(edited < -6);
        assert!(
            drive.app.pads.volume_db[..16]
                .iter()
                .all(|&db| db == edited)
        );
        let hear = |drive: &mut Drive| {
            drive.render_blocks(3);
            drive.tap(KeyCode::Char('1'), M::NONE);
            drive.render_blocks(80)
        };
        let edited_sound = hear(&mut drive);
        undo(&mut drive, controller);
        assert!(drive.app.pads.volume_db[..16].iter().all(|&db| db == 0));
        let undone_sound = hear(&mut drive);
        redo(&mut drive, controller);
        assert!(
            drive.app.pads.volume_db[..16]
                .iter()
                .all(|&db| db == edited)
        );
        let redone_sound = hear(&mut drive);
        assert!(
            edited_sound == redone_sound,
            "the redone pad must sound the same"
        );
        assert!(peak(&undone_sound.0) > 2.0 * peak(&redone_sound.0));
        undo(&mut drive, controller);
        assert_eq!(drive.app.pads.volume_db[0], 0);
        if !controller {
            let left = [edited_sound.0, undone_sound.0, redone_sound.0].concat();
            let right = [edited_sound.1, undone_sound.1, redone_sound.1].concat();
            for samples in [&left, &right] {
                assert_eq!(first_non_finite(samples), None);
                assert!(peak(samples) < 1.0);
                assert!(max_step(samples) < 0.01);
                for hit in 0..3 {
                    assert_eq!(
                        peak(&samples[hit * 10_240 + 8_192..(hit + 1) * 10_240]),
                        0.0
                    );
                }
            }
            let path =
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/142-redo.wav");
            engine::offline::write_wav(&path, 48_000, &left, &right).unwrap();
        }
    }
}

#[test]
fn redo_swaps_a1_addresses_and_samples_again() {
    for controller in [false, true] {
        let mut drive = hold_drive();
        drive.load_pad(0, test_sound(), "one");
        drive.load_pad(1, test_sound(), "two");
        drive.app.pads.pattern = Some(session::PatternState {
            slot: "A1".into(),
            bars: 1,
            quantize: "off".into(),
            strength: 100,
            hits: vec![[0, 0, 100, 0], [240, 1, 110, 0]],
        });
        drive.tap(KeyCode::Char('x'), M::CONTROL);
        drive.tap(KeyCode::Char('2'), M::NONE);
        let swapped = drive.app.pads.pattern.clone();
        assert_eq!(swapped.as_ref().unwrap().hits[0][1], 1);
        undo(&mut drive, controller);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits[0][1], 0);
        redo(&mut drive, controller);
        assert_eq!(drive.app.pads.pattern, swapped);
        assert_eq!(drive.app.pads.custom_name[0].as_deref(), Some("two"));
        undo(&mut drive, controller);
        assert_eq!(drive.app.pads.pattern.as_ref().unwrap().hits[0][1], 0);
    }
}

/// Scenario "Redo restores an accepted A1 take".
#[test]
fn redo_restores_an_accepted_a1_take() {
    use super::super::patterns::app_capture_frames;

    for controller in [false, true] {
        for active in [false, true] {
            let mut drive = hold_drive();
            drive.tap(KeyCode::F(5), M::NONE);
            drive.tap(KeyCode::Char('r'), M::CONTROL);
            app_capture_frames(&mut drive, 96_000);
            if controller {
                cell(&mut drive, 48);
            } else {
                drive.tap(KeyCode::Char('1'), M::NONE);
            }
            app_capture_frames(&mut drive, 128);
            drive.app.flush_audio();
            let accepted = drive.app.pads.pattern.clone();
            assert_eq!(accepted.as_ref().unwrap().hits.len(), 1);
            if !active {
                drive.tap(KeyCode::Char('r'), M::CONTROL);
                app_capture_frames(&mut drive, 128);
                drive.app.flush_audio();
                app_capture_frames(&mut drive, 128);
                drive.app.flush_audio();
            }
            undo(&mut drive, controller);
            if active {
                redo(&mut drive, controller);
                for _ in 0..5 {
                    app_capture_frames(&mut drive, 128);
                    drive.app.flush_audio();
                }
            }
            assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
            redo(&mut drive, controller);
            app_capture_frames(&mut drive, 128);
            assert_eq!(drive.app.pads.pattern, accepted);
            undo(&mut drive, controller);
            assert!(drive.app.pads.pattern.as_ref().unwrap().hits.is_empty());
        }
    }
}

/// Scenario "Redo restores Chop and Assign edits".
#[test]
fn redo_restores_chop_and_assign_edits() {
    for controller in [false, true] {
        let mut drive = hold_drive();
        drive.prepare_chop(test_sound(), 4);
        drive.link_chop_pad(0, 0);
        drive.app.chop.as_mut().unwrap().cursor = 1500;
        drive.tap(KeyCode::Char('m'), M::NONE);
        let cut = drive.app.chop.as_ref().unwrap().saved();
        let linked_region = drive.app.pads.region[0];
        undo(&mut drive, controller);
        assert_ne!(drive.app.chop.as_ref().unwrap().saved(), cut);
        drive.tap(KeyCode::F(2), M::NONE);
        redo(&mut drive, controller);
        assert_eq!(drive.app.redo.len(), 1);
        drive.app.goto(Screen::Chop);
        redo(&mut drive, controller);
        assert_eq!(drive.app.chop.as_ref().unwrap().saved(), cut);
        assert_eq!(drive.app.pads.region[0], linked_region);

        drive.app.chop.as_mut().unwrap().select_slice(0);
        drive.tap(KeyCode::Char('g'), M::CONTROL);
        let gain = drive.app.pads.sound_gain_db[0];
        assert!(gain > 0.0);
        undo(&mut drive, controller);
        assert_eq!(drive.app.pads.sound_gain_db[0], 0.0);
        redo(&mut drive, controller);
        assert_eq!(drive.app.pads.sound_gain_db[0], gain);

        drive.tap(KeyCode::Char('p'), M::NONE);
        assert!(drive.app.plan.is_some());
        drive.tap(KeyCode::Char('3'), M::NONE);
        let plan = drive.app.plan.clone();
        undo(&mut drive, controller);
        assert_eq!(drive.app.pads.sound_gain_db[0], gain);
        assert_ne!(drive.app.plan, plan);
        if controller {
            hold(&mut drive, true);
            cell(&mut drive, panel_note(15));
            let panel = drive.app.panel_snapshot().unwrap();
            assert_eq!(panel.labels[14].as_deref(), Some("redo"));
            assert!(!panel.dim[14]);
            cell(&mut drive, panel_note(15));
            cell(&mut drive, panel_note(13));
            assert!(drive.app.panel_snapshot().unwrap().target);
            hold(&mut drive, false);
        } else {
            redo(&mut drive, false);
        }
        assert_eq!(drive.app.plan, plan);
    }
}

/// Scenario "A new edit replaces the future steps".
#[test]
fn a_new_edit_replaces_the_future_steps() {
    let mut drive = hold_drive();
    drive.load_pad(0, test_sound(), "tone");
    drive.tap(KeyCode::Char('j'), M::NONE);
    undo(&mut drive, false);
    drive.tap(KeyCode::Char('1'), M::NONE);
    assert_eq!(drive.app.redo_refusal(), None);
    drive.tap(KeyCode::Char('l'), M::NONE);
    assert_eq!(drive.app.redo_refusal(), Some("nothing to redo"));
    redo(&mut drive, true);
    assert_eq!(drive.last_notice().text, "nothing to redo");

    undo(&mut drive, false);
    assert!(!drive.app.redo.is_empty());
    drive.app.apply_session(session::Session::default());
    assert!(drive.app.redo.is_empty());
}

#[test]
fn redo_returns_to_the_fresh_chop_with_its_epoch() {
    for controller in [false, true] {
        for linked in [false, true] {
            let mut drive = hold_drive();
            let sound = test_sound();
            drive.prepare_chop(sound.clone(), 4);
            let old = drive.app.chop.as_ref().unwrap().saved();
            drive.load_pad(0, sound, "break");
            drive.app.goto(Screen::Pads);
            drive.tap(KeyCode::F(3), M::NONE);
            let fresh = drive.app.chop.as_ref().unwrap().saved();
            assert_ne!(fresh.epoch, old.epoch);
            if linked {
                drive.link_chop_pad(1, 0);
            }
            undo(&mut drive, controller);
            assert_eq!(drive.app.chop.as_ref().unwrap().saved(), old);
            redo(&mut drive, controller);
            assert_eq!(drive.app.chop.as_ref().unwrap().saved(), fresh);
            undo(&mut drive, controller);
            assert_eq!(drive.app.chop.as_ref().unwrap().saved(), old);
        }
    }
}

#[test]
fn redo_respects_an_owner_key_replacement() {
    let mut drive = hold_drive();
    drive.load_pad(0, test_sound(), "tone");
    drive.tap(KeyCode::Char('j'), M::NONE);
    let edited = drive.app.pads.volume_db[0];
    undo(&mut drive, false);
    drive
        .keymap
        .overlay("[global]\n'Ctrl+Y' = 'none'\nF9 = 'edit.redo'\n")
        .unwrap();
    drive.tap(KeyCode::Char('y'), M::CONTROL);
    assert_eq!(drive.app.pads.volume_db[0], 0);
    drive.tap(KeyCode::F(9), M::NONE);
    assert_eq!(drive.app.pads.volume_db[0], edited);
}

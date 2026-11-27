//! Change 138: `norm` sets the sound gain of the pads of the selected slice, not their volume.

use super::*;

/// A square wave at `peak`: its peak is exact, so the gain is exact too.
fn square(peak: f32) -> Arc<Sample> {
    Arc::new(Sample::mono(
        (0..48_000)
            .map(|i| if (i / 60) % 2 == 0 { peak } else { -peak })
            .collect(),
    ))
}

/// Chop over `sound` in two slices; pad 1 (abs 0) plays slice 1.
fn slice_on_pad(sound: Arc<Sample>) -> Drive {
    let mut drive = hold_drive();
    drive.prepare_chop(sound, 2);
    drive.link_chop_pad(0, 0);
    drive.app.chop.as_mut().unwrap().select_slice(0);
    drive
}

/// The undo cell: device pad 2 of the fixed bottom row.
fn slot_of_undo() -> u8 {
    14
}

fn press(drive: &mut Drive, view: panel::View, label: &str) {
    hold(drive, true);
    cell(drive, panel_note(slot_of(view, label)));
    hold(drive, false);
}

/// `edit slice`, then `norm` from the SMC-PAD.
fn norm(drive: &mut Drive) {
    if !drive.app.edit {
        press(drive, panel::View::Chop, "edit slice");
    }
    press(drive, panel::View::ChopEdit, "norm");
}

/// Scenario "Quiet break" (chopping): +5 dB of sound gain, the volume stays.
#[test]
fn quiet_break() {
    let mut drive = slice_on_pad(square(0.5));
    norm(&mut drive);
    assert!((drive.app.pads.sound_gain_db[0] - 5.0).abs() < 0.05);
    assert_eq!(drive.app.pads.volume_db[0], 0, "the pad volume stays");
    let said = drive.last_notice().text.clone();
    assert!(said.contains("+5 dB"), "{said}");
}

/// Scenario "Very quiet break" (chopping): +19 dB, past the +6 dB volume limit.
#[test]
fn very_quiet_break() {
    let mut drive = slice_on_pad(square(0.1));
    norm(&mut drive);
    assert!((drive.app.pads.sound_gain_db[0] - 19.0).abs() < 0.05);
    assert_eq!(drive.app.pads.volume_db[0], 0);
}

/// Scenario "Twice in a row" (chopping): measured from the sound each time.
#[test]
fn twice_in_a_row() {
    let mut drive = slice_on_pad(square(0.5));
    norm(&mut drive);
    norm(&mut drive);
    assert!((drive.app.pads.sound_gain_db[0] - 5.0).abs() < 0.05);
}

/// Scenario "Chop without a pad" (chopping): Ctrl+G with no pad of the slice says so.
#[test]
fn chop_without_a_pad() {
    let mut drive = hold_drive();
    drive.prepare_chop(square(0.5), 2);
    drive.tap(KeyCode::Char('g'), M::CONTROL);
    assert_eq!(drive.last_notice().text, "put the slice on a pad first");
    assert!(drive.app.pads.sound_gain_db.iter().all(|&db| db == 0.0));
}

/// Scenario "Assign ignores preview and gain changes" (chopping).
#[test]
fn assign_ignores_preview_and_gain_changes() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = slice_on_pad(square(0.5));
        let controls = [
            ('o', M::ALT, 90, "chop.loop_toggle"),
            ('p', M::ALT, 91, "chop.as_pad_toggle"),
            ('g', M::CONTROL, 92, "chop.normalize"),
        ];
        for (_, _, cc, action) in controls {
            drive.bind_action(cc, action);
        }
        drive.tap(KeyCode::Char('p'), M::NONE);
        assert!(drive.app.plan.is_some());
        drive.pad_press(source, 4);
        drive.pad_release(source, 4);
        let targets = drive.app.plan.as_ref().unwrap().targets(2);
        assert!(!targets.is_empty());
        let chop = drive.app.chop.as_ref().unwrap();
        let preview = (chop.looping, chop.as_pad);
        let history = drive.app.history.len();
        let gain = drive.app.pads.sound_gain_db[0];

        for (key, mods, cc, _) in controls {
            let frame = drive.registry_command(source, KeyCode::Char(key), mods, cc);
            assert!(frame.commands().is_empty());
            assert_eq!(
                drive.last_notice().text,
                "assigning: Enter applies, Esc leaves"
            );
            let chop = drive.app.chop.as_ref().unwrap();
            assert_eq!((chop.looping, chop.as_pad), preview);
            assert_eq!(drive.app.pads.sound_gain_db[0], gain);
            assert_eq!(drive.app.history.len(), history);
            assert_eq!(drive.app.plan.as_ref().unwrap().targets(2), targets);
        }
    }
}

#[test]
fn assign_loop_refusal_names_the_smc_controls() {
    let mut drive = slice_on_pad(square(0.5));
    press(&mut drive, panel::View::Chop, "put on pads");
    let targets = drive.app.plan.as_ref().unwrap().targets(2);
    let looping = drive.app.chop.as_ref().unwrap().looping;

    hold(&mut drive, true);
    cell(&mut drive, panel_note(15)); // P3 Menu opens Chop actions.
    let frame = cell(
        &mut drive,
        panel_note(slot_of(panel::View::Chop, "loop play")),
    );

    assert!(frame.commands().is_empty());
    assert_eq!(
        drive.last_notice().text,
        "assigning: PAD BANK + 4 applies, PAD BANK + 1 leaves"
    );
    assert_eq!(drive.app.chop.as_ref().unwrap().looping, looping);
    assert_eq!(drive.app.plan.as_ref().unwrap().targets(2), targets);
    hold(&mut drive, false);
}

/// Scenario "Normalize can be undone" (chopping): undo on Pads brings the old gain back.
#[test]
fn normalize_can_be_undone() {
    let mut drive = slice_on_pad(square(0.5));
    norm(&mut drive);
    assert!(drive.app.pads.sound_gain_db[0] > 4.0);
    drive.app.screen = Screen::Pads;
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(drive.app.pads.sound_gain_db[0], 0.0);
}

/// Scenario "Undo in the slice edit brings the level back" (chopping): device pad 2 on the
/// PAD BANK layer, right after `norm`, takes the gain back there; Pads cannot undo it again.
#[test]
fn undo_in_the_slice_edit_brings_the_level_back() {
    let mut drive = slice_on_pad(square(0.5));
    norm(&mut drive);
    assert!(drive.app.pads.sound_gain_db[0] > 4.0);
    hold(&mut drive, true);
    let slot = usize::from(slot_of_undo());
    assert!(
        !drive.app.panel_snapshot().unwrap().dim[slot - 1],
        "undo is not grey after norm"
    );
    cell(&mut drive, panel_note(slot as u8));
    hold(&mut drive, false);
    assert_eq!(drive.app.pads.sound_gain_db[0], 0.0);
    assert_eq!(
        drive.last_notice().text,
        format!("undo: level of pad {} back", tui::pads::pad_tag(0))
    );
    assert!(
        drive
            .app
            .history
            .iter()
            .all(|step| step.what != "normalize"),
        "the Pads history lost the step too"
    );
    assert!(drive.app.edit, "the slice edit stays open");
}

/// The same undo from the keyboard: Ctrl+Z in the slice edit after `norm`.
#[test]
fn ctrl_z_in_the_slice_edit_brings_the_level_back() {
    let mut drive = slice_on_pad(square(0.5));
    norm(&mut drive);
    let cuts = drive.app.chop.as_ref().unwrap().slices().len();
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(drive.app.pads.sound_gain_db[0], 0.0);
    assert_eq!(
        drive.app.chop.as_ref().unwrap().slices().len(),
        cuts,
        "no cut changes"
    );
    assert!(drive.last_notice().text.starts_with("undo: level of pad"));
}

/// A cut after `norm` is the newer step: the first undo takes the cut, the next the level.
#[test]
fn a_cut_after_norm_is_undone_first() {
    let mut drive = slice_on_pad(square(0.5));
    norm(&mut drive);
    press(&mut drive, panel::View::ChopEdit, "join right");
    let joined = drive.app.chop.as_ref().unwrap().slices().len();
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(drive.app.chop.as_ref().unwrap().slices().len(), joined + 1);
    assert!(
        drive.app.pads.sound_gain_db[0] > 4.0,
        "the level stays after the first undo"
    );
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(drive.app.pads.sound_gain_db[0], 0.0);
}

/// The reverse order: undo on Pads takes `norm` back, then undo on Chop does not again.
#[test]
fn a_pads_undo_of_norm_leaves_nothing_for_chop() {
    let mut drive = slice_on_pad(square(0.5));
    norm(&mut drive);
    let screen = drive.app.screen;
    drive.app.screen = Screen::Pads;
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(drive.app.pads.sound_gain_db[0], 0.0);
    drive.app.screen = screen;
    assert!(!drive.app.norm_is_next());
    drive.tap(KeyCode::Char('z'), M::CONTROL);
    assert_eq!(drive.app.pads.sound_gain_db[0], 0.0);
    assert!(!drive.last_notice().text.starts_with("undo: level"));
}

/// Scenario "Normalize without a pad is grey" (chopping).
#[test]
fn normalize_without_a_pad_is_grey() {
    let mut drive = hold_drive();
    drive.prepare_chop(square(0.5), 2);
    press(&mut drive, panel::View::Chop, "edit slice");
    hold(&mut drive, true);
    let slot = slot_of(panel::View::ChopEdit, "norm");
    assert!(drive.app.panel_snapshot().unwrap().dim[usize::from(slot - 1)]);
    cell(&mut drive, panel_note(slot));
    assert_eq!(drive.last_notice().text, "put the slice on a pad first");
}

/// Scenario "The sound gain is saved" (chopping): +5 dB comes back; no field reads 0 dB.
#[test]
fn the_sound_gain_is_saved() {
    let mut pads = pads::Pads::default();
    pads.file[0] = Some("kick.wav".into());
    pads.sound_gain_db[0] = 5.0;
    let saved = pads.to_session(None, 48_000);
    let text = toml::to_string(&saved).unwrap();
    let mut back = pads::Pads::default();
    back.restore(&toml::from_str(&text).unwrap(), 48_000);
    assert_eq!(back.sound_gain_db[0], 5.0);
    let old: session::Session =
        toml::from_str("version = 3\n\n[pads.5]\nvolume_db = -6\n").unwrap();
    let mut back = pads::Pads::default();
    back.restore(&old, 48_000);
    assert_eq!(back.sound_gain_db[5], 0.0);
}

/// Scenario "The sound gain raises the level before the volume" (pad-playback), the app
/// side: the engine gets the gain of `norm`, and the render of the pad before and after
/// goes to `renders/` for the owner. The peak lands at −1 dB, no clipping, no click.
#[test]
fn render_the_normalized_pad() {
    use engine::checks::{first_non_finite, max_step, onsets, peak};
    use engine::offline::{Event, Scenario, render, write_wav};
    let sr = 48_000u32;
    let ms = |n: usize| sr as usize / 1000 * n;
    // a quiet generated hit: a decaying 110 Hz tone at −13 dB
    let hit: Vec<f32> = (0..ms(400))
        .map(|i| {
            let t = i as f32 / sr as f32;
            0.224 * (-t * 9.0).exp() * (t * 110.0 * std::f32::consts::TAU).sin()
        })
        .collect();
    let sound = Arc::new(Sample::mono(hit));
    let mut drive = slice_on_pad(sound.clone());
    drive.app.chop.as_mut().unwrap().select_slice(0);
    norm(&mut drive);
    let db = drive.app.pads.sound_gain_db[0];
    let region = drive.app.pads.region[0];
    let play = |db: f32| {
        let events = [0, 500, 1000].into_iter().map(|t| Event {
            frame: ms(t + 50),
            cmd: Command::PadPress {
                pad: 0,
                velocity: 1.0,
            },
        });
        let mut all = vec![
            Event {
                frame: 0,
                cmd: Command::SetPadSound {
                    pad: 0,
                    sound: Some(sound.clone()),
                },
            },
            Event {
                frame: 0,
                cmd: Command::SetPadRegion { pad: 0, region },
            },
            Event {
                frame: 0,
                cmd: Command::SetSoundGain {
                    pad: 0,
                    gain: 10f32.powf(db / 20.0),
                },
            },
        ];
        all.extend(events);
        let sc = Scenario {
            sample_rate: sr,
            frames: ms(1600),
            events: all,
        };
        render(&mut engine::Engine::new(sr), &sc, 128)
    };
    let (bl, br) = play(0.0);
    let (al, ar) = play(db);
    for out in [&bl, &al] {
        assert_eq!(first_non_finite(out), None, "no NaN or infinities");
        assert!(peak(out) < 1.0, "no clipping: {}", peak(out));
        let hits = onsets(out, 1e-4, ms(100));
        assert_eq!(hits.len(), 3, "three hits: {hits:?}");
        for (got, want) in hits.iter().zip([50, 550, 1050]) {
            assert!(
                got.abs_diff(ms(want)) <= 1,
                "hit at {got}, want {}",
                ms(want)
            );
        }
    }
    let after = 20.0 * peak(&al).log10();
    assert!(
        (after + 1.0).abs() < 0.1,
        "the peak is at −1 dB: {after:.2}"
    );
    assert!(peak(&al) > peak(&bl) * 3.0, "louder after norm");
    // the steepest slope of the louder hit stays a tone, not a click
    assert!(max_step(&al) < 0.06, "no click: {}", max_step(&al));
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders");
    write_wav(&dir.join("138-normalize-before.wav"), sr, &bl, &br).unwrap();
    write_wav(&dir.join("138-normalize-after.wav"), sr, &al, &ar).unwrap();
}

/// Scenario "A new sound starts without the old gain" (pad-playback): a clear drops it.
#[test]
fn a_new_sound_starts_without_the_old_gain() {
    let mut drive = Drive::new();
    drive.load_pad(0, square(0.5), "kick");
    drive.app.pads.sound_gain_db[0] = 5.0;
    drive.tap(KeyCode::Backspace, M::NONE);
    drive.tap(KeyCode::Enter, M::NONE);
    assert_eq!(drive.app.pads.sound_gain_db[0], 0.0);
}

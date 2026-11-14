use super::*;

/// Scenario "Bank change" (pad-playback).
#[test]
fn bank_change_keeps_the_old_pad_sounding() {
    let mut drive = Drive::offline();
    let sound = Arc::new(Sample::mono(vec![0.2; 4096]));
    drive.load_pad(0, sound.clone(), "tone");
    drive.app.send(Command::SetPadSound {
        pad: 0,
        sound: Some(sound),
    });
    drive.render_blocks(1);

    drive.pad_press(Source::Keyboard, 0);
    let before = drive.render_blocks(1);
    assert!(engine::checks::peak(&before.0) > 0.0);

    drive.tap(KeyCode::Char('o'), M::NONE);
    assert_eq!(drive.app.pads.bank, 1);
    let view = pads_view(&mut drive.app.pads, None, None, Vec::new(), None, 48_000);
    assert!(view.pads[0].empty, "the grid shows the open bank");
    let after = drive.render_blocks(1);
    assert!(
        engine::checks::peak(&after.0) > 0.0,
        "bank 1 keeps sounding"
    );
}

//! Change 132: Chop from a pad, the slice page of a chop and one Undo for an apply.

use super::*;

/// Hold PAD BANK, hit the panel cell `slot`, release PAD BANK.
fn press_cell(drive: &mut Drive, slot: u8) -> Frame {
    hold(drive, true);
    let frame = cell(drive, panel_note(slot));
    hold(drive, false);
    frame
}

/// The keyboard key or the SMC-PAD cell that runs `chop` on Pads.
fn chop_from(drive: &mut Drive, source: Source) {
    match source {
        Source::Keyboard => {
            drive.tap(KeyCode::F(3), M::NONE);
        }
        _ => {
            press_cell(drive, 11);
        }
    }
}

fn break_sound() -> Arc<Sample> {
    Arc::new(Sample::mono(vec![0.2; 48_000]))
}

/// Scenario "The chop cell opens the slice of the pad" (chopping).
#[test]
fn the_chop_cell_opens_the_slice_of_the_pad() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = hold_drive();
        drive.prepare_chop(break_sound(), 40);
        drive.link_chop_pad(20, 19);
        drive.app.pads.bank = 1;
        drive.app.pads.selected = 4;
        drive.app.goto(Screen::Pads);
        drive.frame();
        chop_from(&mut drive, source);
        assert_eq!(drive.app.screen, Screen::Chop, "{source:?}");
        assert_eq!(drive.app.chop.as_ref().unwrap().selected, 19, "{source:?}");
        assert_eq!(drive.app.pads.bank, 1, "the bank of that pad, {source:?}");
    }
}

/// Scenario "The chop cell on an empty pad" (chopping).
#[test]
fn the_chop_cell_on_an_empty_pad() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = hold_drive();
        drive.app.pads.bank = 2;
        drive.app.pads.selected = 4;
        drive.app.goto(Screen::Pads);
        drive.frame();
        chop_from(&mut drive, source);
        assert_eq!(drive.app.screen, Screen::Pads, "{source:?}");
        let text = &drive.last_notice().text;
        assert!(text.starts_with("pad is empty"), "{text}");
        assert!(
            text.contains("bank 3, pad"),
            "the notice names the pad: {text}"
        );
    }
}

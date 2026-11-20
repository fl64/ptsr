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

/// The row over the Chop grid at 100×30, as the owner reads it.
fn chop_rows(drive: &mut Drive) -> String {
    let mut state = graphics::State::default();
    let frame =
        super::super::feedback::image_frame_at(drive, &mut state, false, (100, 30), Instant::now());
    frame.rows.join("\n")
}

/// The whole break on bank 3, pad 5 (keyboard pad D), selected on Pads.
fn whole_file_on_bank_3() -> Drive {
    let mut drive = hold_drive();
    drive.load_pad(36, break_sound(), "break");
    drive.app.pads.bank = 2;
    drive.app.pads.selected = 4;
    drive.app.goto(Screen::Pads);
    drive.frame();
    drive
}

fn cut_into(drive: &mut Drive, slices: usize) {
    let c = drive.app.chop.as_mut().unwrap();
    let step = c.sound.len() / slices;
    for n in 1..slices {
        c.cut(n * step);
    }
    assert_eq!(c.slices().len(), slices);
    drive.frame();
}

/// Scenario "A whole file opens on the first slice page" (chopping).
#[test]
fn a_whole_file_opens_on_the_first_slice_page() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = whole_file_on_bank_3();
        chop_from(&mut drive, source);
        assert_eq!(drive.app.screen, Screen::Chop, "{source:?}");
        cut_into(&mut drive, 40);
        let rows = chop_rows(&mut drive);
        assert!(
            rows.contains("slices 1–16 · page 1/3"),
            "{source:?}\n{rows}"
        );
    }
}

/// Scenario "A turned slice page keeps the pad bank" (chopping).
#[test]
fn a_turned_slice_page_keeps_the_pad_bank() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = whole_file_on_bank_3();
        chop_from(&mut drive, source);
        cut_into(&mut drive, 40);
        match source {
            Source::Keyboard => {
                drive.tap(KeyCode::PageDown, M::NONE);
            }
            _ => {
                // RIGHT on the device
                drive.bytes(SMC, &[0xB0, 26, 127]);
                drive.bytes(SMC, &[0xB0, 26, 0]);
                drive.frame();
            }
        }
        let rows = chop_rows(&mut drive);
        assert!(
            rows.contains("slices 17–32 · page 2/3"),
            "{source:?}\n{rows}"
        );
        hit_key(&mut drive, source);
        assert_eq!(drive.app.chop.as_ref().unwrap().selected, 16, "{source:?}");
        match source {
            Source::Keyboard => {
                drive.tap(KeyCode::Esc, M::NONE);
            }
            _ => {
                // PAD BANK held: LEFT shows the screens view, cell 1 is Pads
                hold(&mut drive, true);
                drive.bytes(SMC, &[0xB0, 25, 127]);
                drive.bytes(SMC, &[0xB0, 25, 0]);
                cell(&mut drive, panel_note(1));
                hold(&mut drive, false);
            }
        }
        assert_eq!(drive.app.screen, Screen::Pads, "{source:?}");
        assert_eq!(drive.app.pads.bank, 2, "Pads keeps bank 3, {source:?}");
    }
}

/// The first pad of the grid: keyboard key 1 or the device pad printed 13.
fn hit_key(drive: &mut Drive, source: Source) {
    match source {
        Source::Keyboard => {
            drive.key(KeyCode::Char('1'), M::NONE, KeyEventKind::Press);
            drive.key(KeyCode::Char('1'), M::NONE, KeyEventKind::Release);
            drive.frame();
        }
        _ => {
            drive.bytes(SMC, &[0x99, 35 + 13, 100]);
            drive.bytes(SMC, &[0x89, 35 + 13, 0]);
            drive.frame();
        }
    }
}

/// Scenario "Files opens the page of the selected slice" (chopping).
#[test]
fn files_opens_the_page_of_the_selected_slice() {
    let mut drive = hold_drive();
    let sound = break_sound();
    drive.prepare_chop(sound.clone(), 40);
    drive.app.chop.as_mut().unwrap().select_slice(34);
    drive.app.goto(Screen::Files);
    drive.app.show_file_chop("break.wav".into(), sound);
    drive.frame();
    assert_eq!(drive.app.screen, Screen::Chop);
    let rows = chop_rows(&mut drive);
    assert!(rows.contains("slices 33–40 · page 3/3"), "{rows}");
}

/// Scenario "Fewer slices show the last page" (chopping).
#[test]
fn fewer_slices_show_the_last_page() {
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = hold_drive();
        drive.prepare_chop(break_sound(), 40);
        let c = drive.app.chop.as_mut().unwrap();
        c.page = 2;
        c.select_slice(39);
        for _ in 0..30 {
            c.delete();
        }
        assert_eq!(c.slices().len(), 10);
        drive.frame();
        let rows = chop_rows(&mut drive);
        assert!(rows.contains("slices 1–10 · page 1/1"), "{rows}");
        hit_key(&mut drive, source);
        assert_eq!(drive.app.chop.as_ref().unwrap().selected, 0, "{source:?}");
    }
}

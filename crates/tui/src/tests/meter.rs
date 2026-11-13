//! The master meter and its clip row.

use super::*;

#[test]
fn meters_follow_the_sound() {
    let p = idle_pads();
    // L at -6 dB with its peak at -3, R at -12 dB with the peak under the bar
    let rows = render_pads_with(p, ([-6.0, -12.0], [-3.0, -12.0], None));
    // the meters top the right column (31)
    let y = frame_at(100, 26).side.y as usize;
    let (l, r) = (row_text(&rows, y), row_text(&rows, y + 1));
    let bars = |t: &str| t.chars().filter(|&c| c == '\u{2588}').count();
    assert!(bars(&l) > bars(&r), "the louder channel has the longer bar");
    assert!(l.contains("-6 dB"), "the dB number is next to the bar");
    assert!(l.contains('\u{258e}'), "the peak mark stands past the bar");
    assert!(
        !r.contains('\u{258e}'),
        "no mark when the peak is under the bar"
    );
}

#[test]
fn overload_latches() {
    let y = frame_at(100, 26).side.y as usize + 2;
    let rows = render_pads_with(
        idle_pads(),
        ([-6.0, -6.0], [-6.0, -6.0], Some(Duration::from_secs(3_723))),
    );
    assert!(row_text(&rows, y).contains("CLIP +01:02:03"));
    let light = rows[y]
        .iter()
        .find(|(s, _, _)| s == "C")
        .expect("the lit CLIP light");
    assert_eq!(light.2, MAGENTA, "the light is a magenta plate");
    let rows = render_pads_with(idle_pads(), ([-60.0, -60.0], [-60.0, -60.0], None));
    let side = frame_at(100, 26).side;
    assert!(
        rows[y][side.x as usize..side.right() as usize]
            .iter()
            .all(|(symbol, _, _)| symbol == " ")
    );
    assert!(
        rows[y].iter().all(|(_, fg, _)| *fg != MAGENTA),
        "a clean output leaves the light dark"
    );
}

/// Scenario "See what a bank holds": the bank row uses the pad fill mask.
#[test]
fn see_what_a_bank_holds() {
    // every pad lit: two full glyphs
    assert_eq!(bank_glyphs(0xFFFF), "⣿⣿");
    // nothing lit: two blank glyphs keep the row aligned
    assert_eq!(bank_glyphs(0), "\u{2800}\u{2800}");
    // pad 1 (row 0, col 0): top-left dot of the first glyph
    assert_eq!(bank_glyphs(1), "\u{2801}\u{2800}");
    // pad 4 (row 1, col 0): second dot of the left column
    assert_eq!(bank_glyphs(1 << 4), "\u{2802}\u{2800}");
    // pad 5 (row 1, col 1): second dot of the right column
    assert_eq!(bank_glyphs(1 << 5), "\u{2810}\u{2800}");
    // pad 8 (row 1, col 3): second dot of the right column of the second glyph
    assert_eq!(bank_glyphs(1 << 7), "\u{2800}\u{2810}");
    let mut pads = idle_pads();
    pads.bank_fill[0] = (1 << 0) | (1 << 4);
    let line: String = banks(&pads)
        .iter()
        .map(|span| span.content.as_ref())
        .collect();
    assert!(line.contains(&format!("[1 {}]", bank_glyphs(pads.bank_fill[0]))));
    assert!(line.contains(&format!("2{}", bank_glyphs(0))));
}

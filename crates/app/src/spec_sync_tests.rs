/// Scenario "Same sound again": the whole-file identity skips replacement.
#[test]
fn same_sound_again() {
    let mut p = super::pads::Pads::default();
    let kick = std::path::PathBuf::from("/m/kick.wav");
    p.file[0] = Some(kick.clone());
    p.region[0] = super::Region::WHOLE;
    assert!(super::same_whole_sound(&p, 0, &kick));
    // a slice of the same file is another sound: the question stays
    p.region[0] = super::Region {
        start: 100,
        end: 400,
    };
    assert!(!super::same_whole_sound(&p, 0, &kick));
    // another file is another sound
    assert!(!super::same_whole_sound(
        &p,
        0,
        std::path::Path::new("/m/snare.wav")
    ));
}

/// Scenario "Fill follows a change": the bank mask follows pad assignment and removal.
#[test]
fn fill_follows_a_change() {
    let mut pads = super::pads::Pads::default();
    let abs = 4 * 16 + 8;
    assert_eq!(super::bank_fill(&pads)[4], 0);
    pads.file[abs] = Some(std::path::PathBuf::from("/m/kick.wav"));
    assert_eq!(super::bank_fill(&pads)[4], 1 << 8);
    pads.file[abs] = None;
    assert_eq!(super::bank_fill(&pads)[4], 0);
}

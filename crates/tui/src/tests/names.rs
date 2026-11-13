//! User names in terminal-cell fields.

use super::*;

#[test]
fn long_names_stop_at_complete_words() {
    assert_eq!(clip_name("warm dusty snare sample", 12), "warm dusty…");
    assert_eq!(clip_name("warm dusty snare sample", 5), "warm…");
    assert_eq!(clip_name("warm dusty snare sample", 0), "");
    assert_eq!(
        clip_name("warm dusty snare sample", 24),
        "warm dusty snare sample"
    );
}

#[test]
fn an_overwide_first_word_keeps_graphemes_whole() {
    for width in 0..12 {
        let label = clip_name("音🎚️e\u{301}verylong sample", width);
        assert!(
            Span::raw(label.as_str()).width() <= width,
            "{width}: {label}"
        );
        assert!(!label.starts_with('\u{301}'));
        assert!(!label.chars().any(char::is_control));
    }
    assert_eq!(clip_name("音🎚️e\u{301}verylong sample", 3), "音…");
}

use std::path::Path;

use engine::checks::{first_loud, first_non_finite, onsets, peak};
use engine::offline::{Scenario, render, write_wav};
use engine::{Engine, click};

const SR: u32 = 48_000;
const EPS: f32 = 1e-6;

fn scenario() -> Scenario {
    Scenario::clicks(SR, 120.0, 8, 8)
}

/// Scenario "Render of a test pattern" (audio-engine).
#[test]
fn click_pattern_renders_on_time_and_clean() {
    let sc = scenario();
    let (left, right) = render(&mut Engine::new(SR), &sc, 128);
    let click_len = click(SR).len();

    for ch in [&left, &right] {
        assert_eq!(first_non_finite(ch), None);
        assert!(peak(ch) <= 0.9, "clipping: peak {}", peak(ch));

        let expected: Vec<_> = sc.events.iter().map(|e| e.frame).collect();
        let got = onsets(ch, EPS, click_len);
        assert_eq!(got.len(), expected.len(), "clicks: {got:?}");
        for (g, e) in got.iter().zip(&expected) {
            assert!(g.abs_diff(*e) <= 1, "click at {g}, expected {e}");
        }

        // silence between clicks and after the last one
        let ends = expected.iter().skip(1).copied().chain([sc.frames]);
        for (start, end) in expected.iter().zip(ends) {
            assert_eq!(first_loud(ch, start + click_len..end, EPS), None);
        }
    }

    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/01-click-pattern.wav");
    write_wav(&out, SR, &left, &right).unwrap();
}

#[test]
fn render_does_not_depend_on_block_size() {
    let sc = scenario();
    let (reference, _) = render(&mut Engine::new(SR), &sc, 128);
    for block in [1, 64, 1000, 4096] {
        let (got, _) = render(&mut Engine::new(SR), &sc, block);
        assert!(got == reference, "block {block} produced different audio");
    }
}

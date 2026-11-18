//! Decaying reverb tails through the real callback (129). Spec scenario "Decaying reverb
//! tails reach exact silence" has its unit test in `rt.rs`. This render is for the owner's
//! ears: `renders/129-reverb-tails.wav`, with the automated checks.

use std::path::Path;
use std::sync::Arc;

use engine::Engine;
use engine::checks::{first_non_finite, max_step, peak};
use engine::filter::{FilterKind, FilterSettings};
use engine::fx::FxSettings;
use engine::offline::write_wav;
use engine::rt::{Command, Realtime, Stats};

const SR: u32 = 48_000;
const BLOCK: usize = 512;

fn drum(tone: f32) -> Vec<f32> {
    (0..12_000)
        .map(|i| (i as f32 * tone).sin() * 0.5 * (-(i as f32) / 3_000.0).exp())
        .collect()
}

#[test]
fn reverb_tails_render_decays_cleanly() {
    let (mut tx, rx) = rtrb::RingBuffer::new(1024);
    let mut send = |command| tx.push(command).unwrap();
    send(Command::SetFx {
        slot: 0,
        settings: FxSettings::Reverb {
            size: 0.85,
            tone: 0.05,
        },
    });
    send(Command::SetFx {
        slot: 1,
        settings: FxSettings::Reverb {
            size: 0.16,
            tone: 0.76,
        },
    });
    for pad in 0..4 {
        send(Command::SetPadFilter {
            pad,
            settings: FilterSettings {
                kind: FilterKind::LowPass,
                cutoff_hz: 1_122.0,
                resonance: 0.02,
            },
        });
        send(Command::SetPadSends {
            pad,
            sends: [0.3, 0.2],
        });
    }
    let pads = vec![drum(0.05), drum(0.08), drum(0.03), drum(0.12)];
    let mut rt = Realtime::new(
        Engine::with_sounds(SR, engine::click(SR), pads),
        vec![rx],
        Arc::new(Stats::default()),
        SR,
    );
    let (mut left, mut right) = (Vec::new(), Vec::new());
    let (mut l, mut r) = (vec![0.0; BLOCK], vec![0.0; BLOCK]);
    let blocks = |seconds: usize| seconds * SR as usize / BLOCK;
    // Four seconds of fast hits, then twenty seconds of tails.
    for block in 0..blocks(24) {
        if block < blocks(4) && block % 8 == 0 {
            send(Command::PadPress {
                pad: (block / 8) % 4,
                velocity: 0.5,
            });
        }
        rt.process(&mut l, &mut r);
        left.extend_from_slice(&l);
        right.extend_from_slice(&r);
    }
    for buffer in [&left, &right] {
        assert_eq!(first_non_finite(buffer), None, "NaN or infinity");
        assert!(peak(buffer) < 1.0, "clips at {}", peak(buffer));
        assert!(max_step(buffer) < 0.5, "click step {}", max_step(buffer));
        let end = &buffer[buffer.len() - SR as usize..];
        assert!(
            peak(end) < 1e-4,
            "the last second still sounds at {}",
            peak(end)
        );
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/129-reverb-tails.wav");
    write_wav(&path, SR, &left, &right).unwrap();
}

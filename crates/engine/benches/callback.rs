//! Audio callback benchmark: 32 voices with pitch, envelope, filter, drive and both sends,
//! a reverb and a delay busy (56), 128-sample block,
//! 48 kHz. Target: at most 50 % of the block budget.
//! Run: `cargo bench -p engine`.

use std::sync::Arc;
use std::time::Instant;

use engine::rt::{Command, Realtime, Stats};
use engine::{Engine, MAX_VOICES};

const SR: u32 = 48_000;
const BLOCK: usize = 128;
const BLOCKS: usize = 20_000;

fn main() {
    // the sound is longer than the run, so every voice plays to the end
    let sound = (0..BLOCK * BLOCKS + 1)
        .map(|i| (i as f32 * 0.01).sin() * 0.01)
        .collect();
    // a press, a pitch change and an envelope for every pad: interpolation, glide and
    // envelope run the whole time
    let (mut tx, rx) = rtrb::RingBuffer::new(8 * MAX_VOICES);
    // different pads: hitting one pad again would cut its previous voice
    for pad in 0..MAX_VOICES {
        tx.push(Command::PadPress { pad, velocity: 1.0 }).unwrap();
        tx.push(Command::SetPadPitch { pad, semitones: 7 }).unwrap();
        tx.push(Command::SetPadEnvelope {
            pad,
            attack_ms: 20,
            decay_ms: 100,
            release_ms: 150,
        })
        .unwrap();
        // a resonant band filter on every voice (44)
        tx.push(Command::SetPadFilter {
            pad,
            settings: engine::filter::FilterSettings {
                kind: engine::filter::FilterKind::Band,
                cutoff_hz: 900.0,
                resonance: 0.5,
            },
        })
        .unwrap();
        // the drive and both sends on every voice (56)
        tx.push(Command::SetPadDrive { pad, drive: 0.6 }).unwrap();
        tx.push(Command::SetPadSends {
            pad,
            sends: [0.3, 0.3],
        })
        .unwrap();
    }
    // both slots busy: a reverb and a delay
    tx.push(Command::SetFx {
        slot: 0,
        settings: engine::fx::FxSettings::Reverb {
            size: 0.8,
            tone: 0.5,
        },
    })
    .unwrap();
    tx.push(Command::SetFx {
        slot: 1,
        settings: engine::fx::FxSettings::Delay {
            time_ms: 375.0,
            feedback: 0.6,
        },
    })
    .unwrap();
    let mut rt = Realtime::new(
        Engine::with_sounds(SR, vec![], vec![sound]),
        vec![rx],
        Arc::new(Stats::default()),
        SR,
    );
    let (mut l, mut r) = (vec![0.0; BLOCK], vec![0.0; BLOCK]);

    let mut times: Vec<f64> = (0..BLOCKS)
        .map(|_| {
            let t = Instant::now();
            rt.process(&mut l, &mut r);
            t.elapsed().as_secs_f64()
        })
        .collect();
    times.sort_by(f64::total_cmp);

    let budget = BLOCK as f64 / SR as f64;
    let pct = |t: f64| t / budget * 100.0;
    let mean = times.iter().sum::<f64>() / BLOCKS as f64;
    let p99 = times[BLOCKS * 99 / 100];
    let max = times[BLOCKS - 1];
    println!(
        "{MAX_VOICES} voices with pitch, envelope, filter, drive and sends, reverb and delay, block {BLOCK} @ {SR} Hz, budget {:.3} ms: mean {:.2} %, p99 {:.2} %, max {:.2} %",
        budget * 1e3,
        pct(mean),
        pct(p99),
        pct(max)
    );
    if pct(p99) > 50.0 {
        eprintln!("EXCEEDED: p99 is over 50 % of the budget");
        std::process::exit(1);
    }
}

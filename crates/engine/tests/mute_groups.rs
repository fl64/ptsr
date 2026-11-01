//! Change 70: mute groups — a hit cuts the other sounding pads of its group with the
//! short click-free fade, other groups and pads with no group ring on, groups span banks.

use std::path::Path;

use engine::checks::{first_non_finite, max_step, onsets, peak};
use engine::offline::{Event, Scenario, render, write_wav};
use engine::{Command, Engine};

const SR: u32 = 48_000;
const MS: usize = SR as usize / 1000;
/// The pad sines below rise by at most about 0.004 per sample, two of them together by
/// 0.008; a cut without a fade jumps by up to 0.1.
const NO_CLICK: f32 = 0.01;
/// The fade of a cut, 5 ms, and two frames of slack.
const CUT: usize = 5 * MS + 2;

fn at(frame: usize, cmd: Command) -> Event {
    Event { frame, cmd }
}

fn press(pad: usize) -> Command {
    Command::PadPress { pad, velocity: 1.0 }
}

fn group(pad: usize, group: u8) -> Command {
    Command::SetPadGroup { pad, group }
}

/// Every pad a long sine of its own pitch that starts at zero.
fn sines() -> Engine {
    let pads = (0..32)
        .map(|p| {
            let hz = 110.0 + 5.0 * p as f32;
            (0..2 * SR as usize)
                .map(|i| (i as f32 * hz * std::f32::consts::TAU / SR as f32).sin() * 0.1)
                .collect()
        })
        .collect();
    Engine::with_sounds(SR, vec![0.0; 10], pads)
}

fn run(events: Vec<Event>) -> Vec<f32> {
    let sc = Scenario {
        sample_rate: SR,
        frames: 600 * MS,
        events,
    };
    render(&mut sines(), &sc, 128).0
}

fn join(mut a: Vec<Event>, b: Vec<Event>) -> Vec<Event> {
    a.extend(b);
    a
}

/// `a` and `b` sound the same from `from` on.
fn same_from(a: &[f32], b: &[f32], from: usize) -> bool {
    a[from..]
        .iter()
        .zip(&b[from..])
        .all(|(x, y)| (x - y).abs() < 1e-6)
}

/// 70 "Closed hat cuts the open one": Q and W in group 1, Q then W 200 ms later; Q fades
/// out within 5 ms without a click, W plays as if alone.
#[test]
fn closed_hat_cuts_the_open_one() {
    let (q, w, hit) = (4, 5, 200 * MS + 17);
    let groups = || vec![at(0, group(q, 1)), at(0, group(w, 1))];
    let both = run(join(groups(), vec![at(0, press(q)), at(hit, press(w))]));
    let alone = run(join(groups(), vec![at(hit, press(w))]));
    assert!(
        peak(&both[100 * MS..200 * MS]) > 0.05,
        "Q sounds before the cut"
    );
    assert!(same_from(&both, &alone, hit + CUT), "Q is gone, W is whole");
    assert!(max_step(&both) < NO_CLICK, "jump {}", max_step(&both));
}

/// 70 "Other groups ring on": Q in 1, E in 2, R off, then W of 1; only Q stops.
#[test]
fn other_groups_ring_on() {
    let (q, w, e, r) = (4, 5, 6, 7);
    let set = || {
        vec![
            at(0, group(q, 1)),
            at(0, group(w, 1)),
            at(0, group(e, 2)),
            at(0, press(e)),
            at(10 * MS, press(r)),
        ]
    };
    let hit = 200 * MS;
    let with_q = run(join(set(), vec![at(0, press(q)), at(hit, press(w))]));
    let without_q = run(join(set(), vec![at(hit, press(w))]));
    assert!(
        same_from(&with_q, &without_q, hit + CUT),
        "E and R ring on, only Q stopped"
    );
}

/// 70 "Groups across banks": pad 1-1 and pad 2-1 in group 3; 2-1 cuts 1-1.
#[test]
fn groups_across_banks() {
    let hit = 200 * MS;
    let set = || vec![at(0, group(0, 3)), at(0, group(16, 3))];
    let both = run(join(set(), vec![at(0, press(0)), at(hit, press(16))]));
    let alone = run(join(set(), vec![at(hit, press(16))]));
    assert!(same_from(&both, &alone, hit + CUT), "pad 1-1 stopped");
}

/// A noise hat: `len` frames of noise that decays, the same every run.
fn hat(len: usize, amp: f32) -> Vec<f32> {
    let mut x: u32 = 0x1234_5678;
    (0..len)
        .map(|i| {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let n = (x >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0;
            let env = (-(i as f32) / len as f32 * 5.0).exp();
            // a 2 ms fade-in, so the hit itself is not a step
            n * env * amp * (i as f32 / (2 * MS) as f32).min(1.0)
        })
        .collect()
}

/// The render for the owner: the open hat rings, the closed hat over it; first with no
/// group (the open one rings on), then both in group 1 (the closed one cuts it).
#[test]
fn render_for_the_owner() {
    let (open, closed) = (4, 5);
    let mut pads = vec![vec![0.0; 10]; 32];
    pads[open] = hat(SR as usize, 0.4);
    pads[closed] = hat(60 * MS, 0.4);
    let mut events = Vec::new();
    let mut t = 0;
    let mut hits = Vec::new();
    for g in [0, 1] {
        events.push(at(t, group(open, g)));
        events.push(at(t, group(closed, g)));
        for _ in 0..4 {
            events.push(at(t, press(open)));
            events.push(at(t + 250 * MS, press(closed)));
            hits.extend([t, t + 250 * MS]);
            t += 500 * MS;
        }
        t += 1000 * MS;
    }
    let sc = Scenario {
        sample_rate: SR,
        frames: t,
        events,
    };
    let (l, r) = render(&mut Engine::with_sounds(SR, vec![0.0; 10], pads), &sc, 128);
    assert_eq!(first_non_finite(&l), None);
    assert!(peak(&l) <= 0.9, "peak {}", peak(&l));
    // every hit starts on its frame; with no group the open hat still rings at the
    // closed one, so only the onsets out of silence are counted
    let got = onsets(&l, 1e-6, 100 * MS);
    assert!(got.len() >= 4, "onsets {got:?}");
    for o in &got {
        assert!(hits.iter().any(|h| h.abs_diff(*o) <= 1), "onset at {o}");
    }
    // in the group the open hat is gone 5 ms after the closed one: only the closed
    // hat's 60 ms are left, then silence until the next open hat
    let second = 4 * 500 * MS + 1000 * MS;
    let closed_hit = second + 250 * MS;
    assert!(
        peak(&l[closed_hit + 70 * MS..second + 500 * MS]) < 1e-6,
        "the open hat rang on in the group"
    );
    let first_closed = 250 * MS;
    assert!(
        peak(&l[first_closed + 70 * MS..500 * MS]) > 0.01,
        "with no group the open hat rings on"
    );
    write_wav(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/70-mute-groups.wav"),
        SR,
        &l,
        &r,
    )
    .unwrap();
}

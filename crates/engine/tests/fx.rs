//! The pad drive and the shared effect slots (56): offline renders with the automated
//! checks. Spec scenarios: "Off is untouched", "Grit, not volume", "The echo", "The tail
//! outlives the pad", "Turn the time while it echoes", "Two pads, one room", "A new type in
//! the slot", "Two reverbs". The renders go to `renders/56-*.wav` for the owner's ears.

use std::path::Path;

use engine::checks::{first_non_finite, max_step, peak};
use engine::fx::FxSettings;
use engine::offline::{Event, Scenario, render, write_wav};
use engine::{Command, Engine};

const SR: u32 = 48_000;
const MS: usize = SR as usize / 1000;

fn at(frame: usize, cmd: Command) -> Event {
    Event { frame, cmd }
}

fn hit(pad: usize) -> Command {
    Command::PadPress { pad, velocity: 1.0 }
}

fn sends(pad: usize, a: f32, b: f32) -> Command {
    Command::SetPadSends { pad, sends: [a, b] }
}

fn reverb(slot: usize, size: f32, tone: f32) -> Command {
    Command::SetFx {
        slot,
        settings: FxSettings::Reverb { size, tone },
    }
}

fn delay(slot: usize, time_ms: f32, feedback: f32) -> Command {
    Command::SetFx {
        slot,
        settings: FxSettings::Delay { time_ms, feedback },
    }
}

fn run(sounds: Vec<Vec<f32>>, frames: usize, events: Vec<Event>) -> (Vec<f32>, Vec<f32>) {
    let mut engine = Engine::with_sounds(SR, vec![], sounds);
    let sc = Scenario {
        sample_rate: SR,
        frames,
        events,
    };
    render(&mut engine, &sc, 128)
}

/// No NaN, no infinity, no clipping.
fn sane(name: &str, (l, r): &(Vec<f32>, Vec<f32>)) {
    for b in [l, r] {
        assert_eq!(first_non_finite(b), None, "{name}: NaN or infinity");
        assert!(peak(b) < 1.0, "{name}: clips at {}", peak(b));
    }
}

fn save(name: &str, (l, r): &(Vec<f32>, Vec<f32>)) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../renders")
        .join(name);
    write_wav(&path, SR, l, r).unwrap();
}

#[test]
fn repeated_project_switch_keeps_effect_fade_continuous() {
    fn setup() -> Engine {
        let mut e = Engine::with_sounds(SR, vec![], vec![vec![0.2; 270 * MS]]);
        e.handle(0, delay(0, 250.0, 0.5));
        e.handle(0, sends(0, 1.0, 0.0));
        e.handle(0, hit(0));
        let mut l = vec![0.0; 300 * MS];
        let mut r = l.clone();
        e.process(&mut l, &mut r);
        assert!(l[l.len() - 1].abs() > 0.01, "effect tail must be audible");
        e.switch_project();
        e
    }

    let mut reference = setup();
    let mut repeated = setup();
    let mut expected = (vec![0.0; 512], vec![0.0; 512]);
    let mut actual = (vec![0.0; 512], vec![0.0; 512]);
    reference.process(&mut expected.0[..100], &mut expected.1[..100]);
    repeated.process(&mut actual.0[..100], &mut actual.1[..100]);
    repeated.switch_project();
    reference.process(&mut expected.0[100..], &mut expected.1[100..]);
    repeated.process(&mut actual.0[100..], &mut actual.1[100..]);
    sane("repeated project switch", &actual);
    for (actual, expected) in [(&actual.0, &expected.0), (&actual.1, &expected.1)] {
        for (frame, (a, b)) in actual.iter().zip(expected).enumerate() {
            assert_eq!(a, b, "effect fade changed at frame {frame}");
        }
    }
    assert!(actual.0[240..].iter().all(|s| *s == 0.0));
    assert!(actual.1[240..].iter().all(|s| *s == 0.0));
}

/// The last sample louder than `eps`.
fn last_loud(buf: &[f32], eps: f32) -> usize {
    buf.iter().rposition(|s| s.abs() > eps).unwrap_or(0)
}

/// A short burst: 2 ms of a 1 kHz tone, so an echo stands apart and is easy to find.
fn burst() -> Vec<f32> {
    (0..2 * MS)
        .map(|i| 0.5 * (std::f32::consts::TAU * 1000.0 * i as f32 / SR as f32).sin())
        .collect()
}

/// A kick: a sine falling from 150 to 50 Hz, dying in 300 ms.
fn kick() -> Vec<f32> {
    let mut phase = 0.0f32;
    (0..300 * MS)
        .map(|i| {
            let t = i as f32 / SR as f32;
            phase += std::f32::consts::TAU * (50.0 + 100.0 * (-t * 30.0).exp()) / SR as f32;
            0.7 * phase.sin() * (-t * 9.0).exp()
        })
        .collect()
}

/// A snare: noise and a 190 Hz body, dying in 150 ms (a fixed generator).
fn snare() -> Vec<f32> {
    let mut x: u32 = 0x9e37_79b9;
    (0..150 * MS)
        .map(|i| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            let t = i as f32 / SR as f32;
            let noise = x as f32 / u32::MAX as f32 * 2.0 - 1.0;
            let body = (std::f32::consts::TAU * 190.0 * t).sin();
            (0.4 * noise + 0.3 * body) * (-t * 25.0).exp()
        })
        .collect()
}

/// Energy of `buf` between `lo` and `hi` Hz: a plain DFT over the bins in that band.
fn band_energy(buf: &[f32], lo: f32, hi: f32) -> f64 {
    let n = buf.len();
    let bin = SR as f32 / n as f32;
    let (first, last) = ((lo / bin).ceil() as usize, (hi / bin).floor() as usize);
    (first.max(1)..=last)
        .map(|k| {
            let w = std::f64::consts::TAU * k as f64 / n as f64;
            let (re, im) = buf
                .iter()
                .enumerate()
                .fold((0.0, 0.0), |(re, im), (i, &s)| {
                    let a = w * i as f64;
                    (re + s as f64 * a.cos(), im - s as f64 * a.sin())
                });
            re * re + im * im
        })
        .sum()
}

/// "Off is untouched": the drive and the sends at 0 and both slots set leave a hit
/// sample for sample as an engine that never heard of them.
#[test]
fn off_is_untouched() {
    let frames = 600 * MS;
    let plain = run(vec![kick()], frames, vec![at(0, hit(0))]);
    let zeroed = run(
        vec![kick()],
        frames,
        vec![
            at(0, Command::SetPadDrive { pad: 0, drive: 0.0 }),
            at(0, sends(0, 0.0, 0.0)),
            at(0, reverb(0, 0.9, 0.2)),
            at(0, delay(1, 300.0, 0.8)),
            at(0, hit(0)),
        ],
    );
    assert_eq!(plain.0, zeroed.0);
    assert_eq!(plain.1, zeroed.1);
}

/// "Grit, not volume": the test tone with the drive at 80 % has more energy above its
/// third harmonic and a peak within 3 dB of the dry one.
#[test]
fn grit_not_volume() {
    let frames = 400 * MS;
    let tone = engine::test_tone(SR, 0);
    let dry = run(vec![tone.clone()], frames, vec![at(0, hit(0))]);
    let wet = run(
        vec![tone.clone()],
        frames,
        vec![
            at(0, Command::SetPadDrive { pad: 0, drive: 0.8 }),
            at(0, hit(0)),
        ],
    );
    sane("drive", &wet);
    let window = 4096;
    let (hi_dry, hi_wet) = (
        band_energy(&dry.0[..window], 400.0, 5_000.0),
        band_energy(&wet.0[..window], 400.0, 5_000.0),
    );
    assert!(hi_wet > 10.0 * hi_dry, "grit: {hi_wet} vs {hi_dry}");
    let db = 20.0 * (peak(&wet.0) / peak(&dry.0)).log10();
    assert!(db.abs() <= 3.0, "the loudness moves by {db} dB");
    // for the owner: the dry kick, then the driven one
    let mut demo_events = vec![at(0, hit(0))];
    demo_events.push(at(700 * MS, Command::SetPadDrive { pad: 0, drive: 0.8 }));
    demo_events.push(at(700 * MS, hit(0)));
    let demo = run(vec![kick()], 1_400 * MS, demo_events);
    sane("drive demo", &demo);
    save("56-drive.wav", &demo);
}

/// "The echo": a burst sent wholly to a delay of 250 ms with 50 % feedback echoes 250 ms
/// later within a sample, and again 250 ms after that about 6 dB quieter.
#[test]
fn the_echo() {
    let frames = 1_000 * MS;
    let start = 10 * MS;
    let out = run(
        vec![burst()],
        frames,
        vec![
            at(0, delay(1, 250.0, 0.5)),
            at(0, sends(0, 0.0, 1.0)),
            at(start, hit(0)),
        ],
    );
    sane("delay", &out);
    let echo = 250 * MS;
    let onset = |from: usize| {
        out.0[from..]
            .iter()
            .position(|s| s.abs() > 1e-4)
            .map(|i| from + i)
            .unwrap()
    };
    let dry = onset(start - MS);
    let first = onset(dry + 3 * MS);
    let second = onset(first + 3 * MS);
    assert!(first.abs_diff(dry + echo) <= 1, "{first} vs {}", dry + echo);
    assert!(second.abs_diff(dry + 2 * echo) <= 1);
    let level = |at: usize| peak(&out.0[at..at + 3 * MS]);
    let db = 20.0 * (level(second) / level(first)).log10();
    assert!((db + 6.02).abs() < 0.5, "the next echo is {db} dB");
    // for the owner: a snare into the delay
    let demo = run(
        vec![snare()],
        2_500 * MS,
        vec![
            at(0, delay(1, 300.0, 0.5)),
            at(0, sends(0, 0.0, 0.6)),
            at(0, hit(0)),
        ],
    );
    sane("delay demo", &demo);
    save("56-delay.wav", &demo);
}

/// "The tail outlives the pad": a one-shot of 100 ms sent 60 % to a reverb sounds on after
/// its end and dies down to silence within a few seconds.
#[test]
fn the_tail_outlives_the_pad() {
    let tone: Vec<f32> = engine::test_tone(SR, 0)[..100 * MS].to_vec();
    let frames = 6_000 * MS;
    let out = run(
        vec![tone],
        frames,
        vec![
            at(0, reverb(0, 0.6, 0.5)),
            at(0, sends(0, 0.6, 0.0)),
            at(0, hit(0)),
        ],
    );
    sane("reverb", &out);
    let after = &out.0[120 * MS..500 * MS];
    assert!(peak(after) > 1e-3, "the tail sounds after the pad");
    let end = last_loud(&out.0, 1e-4);
    assert!(end < 5_000 * MS, "silence after {} ms", end / MS);
    // for the owner: a snare into the reverb
    let demo = run(
        vec![snare()],
        3_000 * MS,
        vec![
            at(0, reverb(0, 0.6, 0.5)),
            at(0, sends(0, 0.6, 0.0)),
            at(0, hit(0)),
        ],
    );
    sane("reverb demo", &demo);
    save("56-reverb.wav", &demo);
}

/// "Turn the time while it echoes": the delay time jumps from 250 to 400 ms while echoes
/// sound; the output steps no more than the same echoes do without the turn.
#[test]
fn turn_the_time_while_it_echoes() {
    let tone: Vec<f32> = engine::test_tone(SR, 0);
    let frames = 2_000 * MS;
    let events = |turn: bool| {
        let mut e = vec![
            at(0, delay(1, 250.0, 0.6)),
            at(0, sends(0, 0.0, 1.0)),
            at(0, hit(0)),
        ];
        if turn {
            e.push(at(600 * MS, delay(1, 400.0, 0.6)));
        }
        e
    };
    let own = run(vec![tone.clone()], frames, events(false));
    let turned = run(vec![tone], frames, events(true));
    sane("turn", &turned);
    // after the turn only: before it the dry hit makes the largest steps of both
    let after = 600 * MS..frames;
    let (a, b) = (max_step(&own.0[after.clone()]), max_step(&turned.0[after]));
    assert!(a > 0.0, "the echoes sound after the turn");
    assert!(b <= a * 1.1, "a step of {b} against the sound's own {a}");
}

/// "Two pads, one room": two pads send to the same reverb; a larger size makes the tail
/// of either longer.
#[test]
fn two_pads_one_room() {
    let tail = |size: f32| {
        let out = run(
            vec![burst(), snare()],
            8_000 * MS,
            vec![
                at(0, reverb(0, size, 0.5)),
                at(0, sends(0, 0.5, 0.0)),
                at(0, sends(1, 0.5, 0.0)),
                at(0, hit(1)),
            ],
        );
        sane("room", &out);
        last_loud(&out.0, 1e-3)
    };
    let (small, large) = (tail(0.2), tail(0.9));
    assert!(
        large > small + 500 * MS,
        "{} vs {} ms",
        large / MS,
        small / MS
    );
}

/// "A new type in the slot": a delay echo sounds, then the slot turns to a reverb in the
/// middle of the echo: the echo fades out without a click.
#[test]
fn a_new_type_in_the_slot() {
    // 100 ms of tone: its echoes stand alone, 250 ms apart
    let tone: Vec<f32> = engine::test_tone(SR, 0)[..100 * MS].to_vec();
    let frames = 1_500 * MS;
    // inside the third echo (750..850 ms), long after the dry tone
    let switch = 790 * MS;
    let out = run(
        vec![tone],
        frames,
        vec![
            at(0, delay(1, 250.0, 0.7)),
            at(0, sends(0, 0.0, 1.0)),
            at(0, hit(0)),
            at(switch, reverb(1, 0.5, 0.5)),
        ],
    );
    sane("switch", &out);
    // the echo's own steps, in the echo before
    let own = max_step(&out.0[510 * MS..590 * MS]);
    let around = max_step(&out.0[switch - MS..switch + 20 * MS]);
    assert!(own > 0.0, "the echo sounds");
    assert!(
        around <= own * 1.1,
        "{around} at the switch, the echo's own {own}"
    );
}

/// "Two reverbs": both slots hold a reverb; the pad in the larger room rings longer. The
/// render is a kick in a small room and a snare in a hall, both to delays too.
#[test]
fn two_reverbs() {
    let tail = |pad: usize| {
        let out = run(
            vec![snare(), snare()],
            8_000 * MS,
            vec![
                at(0, reverb(0, 0.2, 0.5)),
                at(0, reverb(1, 0.9, 0.5)),
                at(0, sends(0, 0.5, 0.0)),
                at(0, sends(1, 0.0, 0.5)),
                at(0, hit(pad)),
            ],
        );
        sane("two reverbs", &out);
        last_loud(&out.0, 1e-3)
    };
    assert!(tail(1) > tail(0) + 500 * MS);
    // for the owner: a beat, the kick dry with drive, the snare in a reverb, then a delay
    let beat = 500 * MS;
    let mut events = vec![
        at(0, reverb(0, 0.7, 0.4)),
        at(0, delay(1, 375.0, 0.45)),
        at(0, Command::SetPadDrive { pad: 0, drive: 0.4 }),
        at(0, sends(0, 0.1, 0.0)),
        at(0, sends(1, 0.5, 0.0)),
    ];
    for b in 0..8 {
        events.push(at(b * beat, hit(if b % 2 == 0 { 0 } else { 1 })));
        if b == 4 {
            events.push(at(b * beat, sends(1, 0.5, 0.4)));
        }
    }
    let demo = run(vec![kick(), snare()], 8 * beat + 2_500 * MS, events);
    sane("two slots demo", &demo);
    save("56-two-slots.wav", &demo);
}

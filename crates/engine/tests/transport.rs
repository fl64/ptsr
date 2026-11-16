use engine::checks::{first_non_finite, max_step, onsets, peak};
use engine::offline::write_wav;
use engine::pattern::{Grid, Hit, RawPattern};
use engine::rt::{ProjectCommand, ProjectMessage, ProjectState, Realtime, Stats};
use engine::{Command, Engine, MAX_VOICES, Sample, click};
use std::sync::{
    Arc,
    atomic::Ordering::{Acquire, Release},
};

const SR: u32 = 48_000;

fn engine(rate: u32) -> Engine {
    Engine::with_sounds(rate, click(rate), vec![])
}

fn stereo(e: &mut Engine, frames: usize, block: usize) -> (Vec<f32>, Vec<f32>) {
    let mut l = vec![0.0; frames];
    let mut r = vec![0.0; frames];
    for (l, r) in l.chunks_mut(block).zip(r.chunks_mut(block)) {
        e.process(l, r);
    }
    (l, r)
}

fn audio(e: &mut Engine, frames: usize, block: usize) -> Vec<f32> {
    let (l, r) = stereo(e, frames, block);
    assert_eq!(l, r);
    l
}

fn start(e: &mut Engine, bpm: f64) {
    e.handle(0, Command::SetTempo { bpm });
    e.handle(0, Command::SetMetronome { enabled: true });
    e.handle(0, Command::TransportPlay);
}

#[test]
fn new_transport_starts_stopped() {
    let mut e = engine(SR);
    assert_eq!(e.transport(), Default::default());
    assert_eq!(e.transport().bar_beat(), (1, 1));
    assert_eq!(peak(&audio(&mut e, SR as usize, 128)), 0.0);
    assert_eq!(e.transport().ticks, 0.0);
}

#[test]
fn audio_frames_advance_the_running_clock() {
    for rate in [44_100, 48_000, 96_000] {
        for block in [1, 64, 128, 1000, 4096] {
            let mut e = engine(rate);
            e.handle(0, Command::TransportPlay);
            assert_eq!(peak(&audio(&mut e, 2 * rate as usize, block)), 0.0);
            assert!((e.transport().ticks - 3840.0).abs() < 1e-6);
            assert_eq!(e.transport().bar_beat(), (2, 1));
        }
    }
}

#[test]
fn stop_resets_position_and_restart_starts_on_the_first_beat() {
    let mut e = engine(SR);
    start(&mut e, 172.0);
    audio(&mut e, 10_000, 128);
    e.handle(0, Command::TransportStop);
    assert_eq!(e.transport().ticks, 0.0);
    assert!(!e.transport().playing);
    audio(&mut e, 2000, 128);
    e.handle(0, Command::TransportPlay);
    assert_eq!(audio(&mut e, 2000, 128)[..click(SR).len()], click(SR));
}

#[test]
fn consecutive_transport_requests_retain_their_order() {
    let (mut tx, rx) = rtrb::RingBuffer::new(16);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(engine(SR), vec![rx], stats.clone(), SR);
    let mut process = || rt.process(&mut [0.0; 128], &mut [0.0; 128]);
    for c in [Command::TransportToggle, Command::TransportToggle] {
        tx.push(c).unwrap();
    }
    process();
    assert_eq!(stats.transport(0).unwrap(), Default::default());
    tx.push(Command::TransportPlay).unwrap();
    process();
    for c in [
        Command::TransportPlay,
        Command::TransportToggle,
        Command::TransportToggle,
    ] {
        tx.push(c).unwrap();
    }
    process();
    let s = stats.transport(0).unwrap();
    assert!(s.playing);
    assert!(
        (s.ticks - 5.12).abs() < 1e-9,
        "toggles restart, explicit play does not"
    );
    tx.push(Command::TransportPlay).unwrap();
    process();
    assert!((stats.transport(0).unwrap().ticks - 10.24).abs() < 1e-9);
    for c in [Command::TransportStop, Command::TransportStop] {
        tx.push(c).unwrap();
    }
    process();
    assert_eq!(stats.transport(0).unwrap().ticks, 0.0);
}

fn check_beats(rate: u32, bpm: f64, block: usize, count: usize) -> Vec<f32> {
    let step = rate as f64 * 60.0 / bpm;
    let frames = (count as f64 * step).round() as usize;
    let mut e = engine(rate);
    start(&mut e, bpm);
    let out = audio(&mut e, frames, block);
    let sample = click(rate);
    let got = onsets(&out, 1e-6, sample.len());
    assert_eq!(got.len(), count, "{rate} {bpm} {block}: {got:?}");
    for (beat, &at) in got.iter().enumerate() {
        let expected = (beat as f64 * step).round() as usize;
        assert!(
            at.abs_diff(expected) <= 1,
            "{rate} {bpm} {block}: {at} != {expected}"
        );
        let velocity = if beat % 4 == 0 { 1.0 } else { 0.6 };
        for (&got, &sample) in out[at..at + sample.len()].iter().zip(&sample) {
            assert!((got - sample * velocity).abs() < 1e-7);
        }
        let next = got.get(beat + 1).copied().unwrap_or(frames);
        assert_eq!(peak(&out[at + sample.len()..next]), 0.0);
    }
    assert_eq!(first_non_finite(&out), None);
    assert!(peak(&out) < 1.0);
    assert!(max_step(&out) < 0.11);
    out
}

#[test]
fn metronome_beats_and_accents_follow_the_clock() {
    for rate in [44_100, 48_000, 96_000] {
        for block in [1, 64, 128, 1000, 4096] {
            check_beats(rate, 120.0, block, 32);
        }
    }
}

#[test]
fn fast_tempo_retains_exact_metronome_accents() {
    for bpm in [170.0, 172.0, 180.0] {
        for block in [1, 128, 4096] {
            check_beats(SR, bpm, block, 32);
        }
    }
}

#[test]
fn tempo_changes_without_resetting_the_beat() {
    let mut e = engine(SR);
    start(&mut e, 120.0);
    let mut out = audio(&mut e, 12_000, 128);
    let before = e.transport().ticks;
    e.handle(0, Command::SetTempo { bpm: 90.0 });
    assert_eq!(e.transport().ticks, before);
    out.extend(audio(&mut e, 100_000, 128));
    let got = onsets(&out, 1e-6, click(SR).len());
    assert_eq!(got.len(), 4);
    for (got, expected) in got.iter().zip([0, 28_000, 60_000, 92_000]) {
        assert!(got.abs_diff(expected) <= 1, "{got} != {expected}");
    }
}

#[test]
fn invalid_tempo_cannot_damage_the_clock() {
    let mut e = engine(SR);
    start(&mut e, 172.34);
    assert_eq!(e.transport().bpm, 172.3);
    for bpm in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        e.handle(0, Command::SetTempo { bpm });
        assert_eq!(e.transport().bpm, 172.3);
    }
    for (bpm, expected) in [(-1.0, 40.0), (999.0, 300.0)] {
        e.handle(0, Command::SetTempo { bpm });
        assert_eq!(e.transport().bpm, expected);
        assert_eq!(first_non_finite(&audio(&mut e, 1000, 128)), None);
        assert!(e.transport().ticks.is_finite());
    }
}

#[test]
fn muting_and_enabling_the_metronome_preserves_phase() {
    let mut e = engine(SR);
    start(&mut e, 120.0);
    audio(&mut e, 12_000, 128);
    e.handle(0, Command::SetMetronome { enabled: false });
    assert_eq!(peak(&audio(&mut e, 60_000, 128)), 0.0);
    e.handle(0, Command::SetMetronome { enabled: true });
    let out = audio(&mut e, 30_000, 128);
    assert_eq!(onsets(&out, 1e-6, click(SR).len()), [0, 24_000]);
    e.handle(0, Command::SetMetronome { enabled: false });
    audio(&mut e, 37_000, 128);
    e.handle(0, Command::SetMetronome { enabled: true });
    assert_eq!(
        onsets(&audio(&mut e, 12_000, 128), 1e-6, click(SR).len()),
        [5000]
    );
}

#[test]
fn empty_audio_blocks_preserve_the_first_beat() {
    let mut e = engine(SR);
    start(&mut e, 120.0);
    for _ in 0..4 {
        e.process(&mut [], &mut []);
    }
    assert_eq!(e.transport().ticks, 0.0);
    assert_eq!(audio(&mut e, 2000, 128)[..click(SR).len()], click(SR));
}

/// Scenario "Transport stop preserves a manually played tail".
#[test]
fn transport_stop_preserves_a_manually_played_tail() {
    let mut a = engine(SR);
    let mut b = engine(SR);
    let sound = Arc::new(Sample::mono(vec![0.1; 96_000]));
    for e in [&mut a, &mut b] {
        e.handle(
            0,
            Command::SetPadSound {
                pad: 0,
                sound: Some(sound.clone()),
            },
        );
        e.handle(
            0,
            Command::SetPadSends {
                pad: 0,
                sends: [0.5, 0.5],
            },
        );
        e.handle(
            0,
            Command::PadPress {
                pad: 0,
                velocity: 1.0,
            },
        );
        e.handle(
            0,
            Command::Preview {
                sound: Some(sound.clone()),
                region: Default::default(),
            },
        );
        e.handle(0, Command::TransportPlay);
        stereo(e, 40_000, 128);
    }
    a.handle(0, Command::TransportStop);
    assert_eq!(stereo(&mut a, 100_000, 128), stereo(&mut b, 100_000, 128));
}

/// Scenario "Context STOP pauses and PLAY continues A1". The render is longer than one second.
#[test]
fn context_stop_pauses_and_play_continues_a1_render() {
    let sound: Vec<f32> = (0..4800)
        .map(|i| {
            let fade = (i.min(4799 - i) as f32 / 240.0).min(1.0);
            0.2 * fade * (std::f32::consts::TAU * 220.0 * i as f32 / SR as f32).sin()
        })
        .collect();
    let pattern = Arc::new(
        RawPattern::new(
            1,
            Grid::Quarter,
            0,
            vec![
                Hit::new(0, 0, 127, 0).unwrap(),
                Hit::new(1920, 0, 127, 0).unwrap(),
            ],
        )
        .unwrap()
        .compile(),
    );
    let setup = || {
        let mut e = Engine::with_sounds(SR, click(SR), vec![sound.clone()]);
        e.handle(
            0,
            Command::SetPattern {
                pattern: Some(pattern.clone()),
            },
        );
        e.handle(0, Command::SetMetronome { enabled: true });
        e.handle(0, Command::TransportPlayContext);
        e
    };
    let mut reference = setup();
    let mut actual = setup();
    let before = audio(&mut actual, 125_000, 128);
    assert_eq!(before, audio(&mut reference, 125_000, 128));
    let held_tick = actual.transport().ticks;
    actual.handle(0, Command::TransportStopContext);
    assert!(!actual.transport().playing);
    assert_eq!(actual.transport().ticks, held_tick);
    let pause = audio(&mut actual, 10_000, 128);
    assert_eq!(peak(&pause), 0.0);
    assert_eq!(actual.transport().ticks, held_tick);
    actual.handle(0, Command::TransportPlayContext);
    let continued = audio(&mut actual, 25_000, 128);
    assert_eq!(continued, audio(&mut reference, 25_000, 128));
    assert_eq!(onsets(&continued, 1e-6, 4800), [19_000]);
    let mut render = before;
    render.extend(pause);
    render.extend(continued);
    assert_eq!(first_non_finite(&render), None);
    assert!(peak(&render) < 1.0 && max_step(&render) < 0.11);
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/92-command-panel.wav");
    write_wav(&path, SR, &render, &render).unwrap();
}

/// Scenario "Second context STOP resets to the start".
#[test]
fn second_context_stop_resets_to_the_start() {
    let mut e = engine(SR);
    let mut reference = engine(SR);
    let sound = Arc::new(Sample::mono(vec![0.05; 96_000]));
    for target in [&mut e, &mut reference] {
        target.handle(
            0,
            Command::SetPadSound {
                pad: 0,
                sound: Some(sound.clone()),
            },
        );
        target.handle(
            0,
            Command::PadPress {
                pad: 0,
                velocity: 1.0,
            },
        );
    }
    e.handle(0, Command::TransportPlayContext);
    assert_eq!(
        audio(&mut e, 30_000, 128),
        audio(&mut reference, 30_000, 128)
    );
    let held_tick = e.transport().ticks;
    e.handle(0, Command::TransportStopContext);
    assert_eq!(e.transport().ticks, held_tick);
    e.handle(0, Command::TransportStopContext);
    assert_eq!(e.transport().ticks, 0.0);
    assert_eq!(
        audio(&mut e, 10_000, 128),
        audio(&mut reference, 10_000, 128)
    );
    e.handle(0, Command::TransportPlayContext);
    assert_eq!(e.transport().ticks, 0.0);
}

/// Scenario "STOP during recording retains one pass".
#[test]
fn stop_during_recording_retains_one_pass() {
    use engine::capture::{Kind, Phase};
    let mut e = engine(SR);
    e.handle(
        0,
        Command::SetPattern {
            pattern: Some(Arc::new(
                RawPattern::new(1, Grid::Off, 100, vec![])
                    .unwrap()
                    .compile(),
            )),
        },
    );
    let mut events = e.enable_capture();
    e.handle(0, Command::TransportPlayContext);
    e.handle(0, Command::PatternRecordStart { pass: 1 });
    e.handle(
        0,
        Command::PadPress {
            pad: 0,
            velocity: 1.0,
        },
    );
    audio(&mut e, 128, 128);
    let accepted = e.transport().ticks;
    e.handle(0, Command::TransportStopContext);
    assert_eq!(e.recording().phase, Phase::Finishing);
    assert_eq!(e.recording().ticks, accepted);
    assert_eq!(e.transport().ticks, accepted);
    assert!(!e.transport().playing);
    let event = events.pop().unwrap();
    assert!(matches!(event.kind, Kind::Onset { pad: 0, .. }));
    assert!(events.pop().is_err());
}

#[test]
fn context_stop_cancels_count_in_without_starting_transport() {
    use engine::capture::Phase;
    let mut e = engine(SR);
    e.handle(
        0,
        Command::SetPattern {
            pattern: Some(Arc::new(
                RawPattern::new(1, Grid::Off, 100, vec![])
                    .unwrap()
                    .compile(),
            )),
        },
    );
    let _events = e.enable_capture();
    e.handle(0, Command::PatternRecordStart { pass: 1 });
    assert_eq!(e.recording().phase, Phase::CountIn);
    audio(&mut e, 128, 128);
    e.handle(0, Command::TransportStopContext);
    assert_eq!(e.recording().phase, Phase::Idle);
    assert!(!e.transport().playing);
    assert_eq!(e.transport().ticks, 0.0);
}

/// Scenario "Existing Space transport path keeps its reset".
#[test]
fn existing_space_transport_path_keeps_its_reset() {
    let mut e = engine(SR);
    e.handle(0, Command::TransportToggle);
    audio(&mut e, 30_000, 128);
    e.handle(0, Command::TransportToggle);
    assert_eq!(e.transport().ticks, 0.0);
    e.handle(0, Command::TransportPlayContext);
    assert_eq!(e.transport().ticks, 0.0);
    assert!(e.transport().playing);
}

#[test]
fn automatic_clicks_preserve_full_pad_polyphony() {
    let mut a = engine(SR);
    let mut b = engine(SR);
    for e in [&mut a, &mut b] {
        let sound = Arc::new(Sample::mono(vec![0.005; 48_000]));
        for pad in 0..MAX_VOICES {
            e.handle(
                0,
                Command::SetPadSound {
                    pad,
                    sound: Some(sound.clone()),
                },
            );
            e.handle(0, Command::PadPress { pad, velocity: 1.0 });
        }
    }
    start(&mut a, 300.0);
    let got = audio(&mut a, 48_000, 128);
    let pads = audio(&mut b, 48_000, 128);
    let metro = check_beats(SR, 300.0, 128, 5);
    for ((got, pad), metro) in got.iter().zip(pads).zip(metro) {
        assert!((got - pad - metro).abs() < 1e-7);
    }
}

/// Scenario "Late transport commands stay in their original project".
#[test]
fn late_transport_commands_stay_in_their_original_project() {
    let (mut tx, rx) = rtrb::RingBuffer::new(512);
    let stats = Arc::new(Stats::default());
    let project = Arc::new(ProjectState::default());
    let mut rt = Realtime::with_project(engine(SR), vec![rx], stats.clone(), SR, project.clone());
    let mut push = |epoch, ready, cmd| {
        tx.push(ProjectCommand {
            epoch,
            ready_at_send: ready,
            command: ProjectMessage::Command(cmd),
        })
        .unwrap()
    };
    push(0, true, Command::TransportPlay);
    rt.process(&mut [0.0; 128], &mut [0.0; 128]);
    assert!(stats.transport(0).unwrap().playing);
    for _ in 0..300 {
        push(0, true, Command::TransportToggle);
    }
    push(0, true, Command::TransportStopContext);
    push(0, true, Command::TransportPlayContext);
    push(0, true, Command::SetTempo { bpm: 299.0 });
    push(0, true, Command::SetMetronome { enabled: true });
    push(1, false, Command::SetTempo { bpm: 90.0 });
    push(1, false, Command::SetMetronome { enabled: false });
    push(1, false, Command::TransportPlay);
    tx.push(ProjectCommand {
        epoch: 1,
        ready_at_send: false,
        command: ProjectMessage::Ready,
    })
    .unwrap();
    project.epoch.store(1, Release);
    for _ in 0..300 {
        rt.process(&mut [0.0; 128], &mut [0.0; 128]);
    }
    assert_eq!(project.ready.load(Acquire), 1);
    assert!(stats.transport(0).is_none());
    let state = stats.transport(1).unwrap();
    assert_eq!(state.bpm, 90.0);
    assert!(!state.playing && !state.metronome);
    assert_eq!(state.ticks, 0.0);
}

#[test]
fn transport_owner_render() {
    let mut e = engine(SR);
    start(&mut e, 120.0);
    let mut out = audio(&mut e, 90_000, 128);
    e.handle(0, Command::SetTempo { bpm: 90.0 });
    out.extend(audio(&mut e, 66_000, 128));
    e.handle(0, Command::TransportStop);
    let stopped = out.len();
    out.extend(audio(&mut e, 24_000, 128));
    assert_eq!(peak(&out[stopped + click(SR).len()..]), 0.0);
    e.handle(0, Command::SetTempo { bpm: 172.0 });
    e.handle(0, Command::TransportPlay);
    let restart = out.len();
    out.extend(audio(
        &mut e,
        (SR as f64 * 60.0 / 172.0 * 8.0).round() as usize,
        128,
    ));
    e.handle(0, Command::TransportStop);
    out.extend(audio(&mut e, 24_000, 128));
    let got = onsets(&out, 1e-6, click(SR).len());
    let mut expected = vec![0, 24_000, 48_000, 72_000, 98_000, 130_000];
    expected
        .extend((0..8).map(|i| restart + (i as f64 * SR as f64 * 60.0 / 172.0).round() as usize));
    assert_eq!(got.len(), expected.len());
    for (i, (&at, &expected)) in got.iter().zip(&expected).enumerate() {
        assert!(at.abs_diff(expected) <= 1);
        let beat = if at >= restart { i - 6 } else { i };
        let velocity = if beat % 4 == 0 { 1.0 } else { 0.6 };
        assert!((peak(&out[at..at + click(SR).len()]) - peak(&click(SR)) * velocity).abs() < 1e-6);
    }
    assert_eq!(first_non_finite(&out), None);
    assert!(peak(&out) < 1.0 && max_step(&out) < 0.11);
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/95-transport.wav");
    write_wav(&path, SR, &out, &out).unwrap();
}

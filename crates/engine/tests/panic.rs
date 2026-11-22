use engine::checks::{first_non_finite, max_step, peak};
use engine::fx::FxSettings;
use engine::offline::{Event, Scenario, render, write_wav};
use engine::rt::{ProjectCommand, ProjectMessage, ProjectState, Realtime, Stats};
use engine::{Command, Engine, Region, Sample};
use std::path::Path;
use std::sync::{
    Arc,
    atomic::Ordering::{Acquire, Release},
};

const RATE: u32 = 48_000;
const FADE: usize = 240;

fn tone(hz: f32) -> Vec<f32> {
    (0..RATE)
        .map(|frame| (frame as f32 * hz * std::f32::consts::TAU / RATE as f32).sin() * 0.1)
        .collect()
}

fn hit() -> Command {
    Command::PadPress {
        pad: 0,
        velocity: 1.0,
    }
}

fn setup() -> Engine {
    let mut engine = Engine::with_sounds(RATE, vec![], vec![tone(110.0)]);
    engine.handle(
        0,
        Command::SetFx {
            slot: 0,
            settings: FxSettings::Delay {
                time_ms: 70.0,
                feedback: 0.5,
            },
        },
    );
    engine.handle(
        0,
        Command::SetPadSends {
            pad: 0,
            sends: [1.0, 0.0],
        },
    );
    engine
}

/// Scenario "Panic fades the whole sampler".
#[test]
fn panic_fades_the_whole_sampler() {
    let stop = 94 * 128;
    let restart = 220 * 128;
    let run = |panic| {
        let mut events = vec![
            Event {
                frame: 0,
                cmd: hit(),
            },
            Event {
                frame: 0,
                cmd: Command::TransportPlay,
            },
            Event {
                frame: 0,
                cmd: Command::Preview {
                    sound: Some(Arc::new(Sample::mono(tone(220.0)))),
                    region: Region::WHOLE,
                },
            },
        ];
        for (frame, cmd) in [(stop, Command::Panic), (restart, hit())] {
            let cmd = if panic {
                cmd
            } else {
                Command::SetMasterGain { gain: 1.0 }
            };
            events.push(Event { frame, cmd });
        }
        let mut engine = setup();
        let wave = render(
            &mut engine,
            &Scenario {
                sample_rate: RATE,
                frames: 40_000,
                events,
            },
            128,
        );
        (wave, engine.transport().playing)
    };
    let ((control, _), control_playing) = run(false);
    let ((left, right), playing) = run(true);
    assert!(control_playing && !playing);
    for (frame, (actual, expected)) in left[..stop].iter().zip(&control).enumerate() {
        assert_eq!(actual, expected, "before Panic at frame {frame}");
    }
    assert!(peak(&control[stop + FADE..restart]) > 0.05);
    for channel in [&left, &right] {
        assert_eq!(first_non_finite(channel), None);
        assert!(peak(channel) < 1.0);
        assert!(max_step(&channel[stop - 1..stop + FADE + 1]) < 0.01);
        assert_eq!(peak(&channel[stop + FADE..restart]), 0.0);
        assert!(peak(&channel[restart..restart + 128]) > 0.05);
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/141-panic.wav");
    write_wav(&path, RATE, &left, &right).unwrap();
}

#[test]
fn panic_takes_effect_after_both_queues_in_the_block() {
    let (mut ui, ui_rx) = rtrb::RingBuffer::new(8);
    let (mut midi, midi_rx) = rtrb::RingBuffer::new(8);
    let stats = Arc::new(Stats::default());
    let mut rt = Realtime::new(setup(), vec![ui_rx, midi_rx], stats.clone(), RATE);
    ui.push(Command::Panic).unwrap();
    midi.push(Command::TransportPlay).unwrap();
    midi.push(hit()).unwrap();
    let mut left = [0.0; 512];
    let mut right = left;
    rt.process(&mut left, &mut right);
    assert!(!stats.transport(0).unwrap().playing);
    assert_eq!(peak(&left[FADE..]), 0.0);
    midi.push(hit()).unwrap();
    rt.process(&mut left, &mut right);
    assert!(peak(&left) > 0.05);
}

/// Scenario "Project restore completes during a Panic fade".
#[test]
fn project_restore_completes_during_a_panic_fade() {
    for frames in [64, 512] {
        let (mut tx, rx) = rtrb::RingBuffer::new(8);
        let project = Arc::new(ProjectState::default());
        let stats = Arc::new(Stats::default());
        let mut rt = Realtime::with_project(setup(), vec![rx], stats, RATE, project.clone());
        let mut left = [0.0; 512];
        let mut right = left;
        let command = |command, epoch, ready_at_send| ProjectCommand {
            epoch,
            ready_at_send,
            command: ProjectMessage::Command(command),
        };
        tx.push(command(hit(), 0, true)).unwrap();
        for _ in 0..10 {
            rt.process(&mut left, &mut right);
        }
        assert!(peak(&left) > 0.05);
        tx.push(command(Command::Panic, 0, true)).unwrap();
        rt.process(&mut left[..frames], &mut right[..frames]);
        project.epoch.store(1, Release);
        tx.push(command(
            Command::SetPadSound {
                pad: 0,
                sound: Some(Arc::new(Sample::mono(tone(330.0)))),
            },
            1,
            false,
        ))
        .unwrap();
        tx.push(ProjectCommand {
            epoch: 1,
            ready_at_send: false,
            command: ProjectMessage::Ready,
        })
        .unwrap();
        for index in 0..256 {
            rt.process(&mut left, &mut right);
            if index > 0 {
                assert_eq!(peak(&left), 0.0);
            }
            if project.ready.load(Acquire) == 1 {
                break;
            }
        }
        assert_eq!(project.ready.load(Acquire), 1);
        tx.push(command(hit(), 1, true)).unwrap();
        rt.process(&mut left, &mut right);
        assert!(peak(&left) > 0.05);
    }
}

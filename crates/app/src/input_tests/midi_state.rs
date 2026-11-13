use super::*;

struct MidiStateLog(std::sync::Mutex<Vec<(std::thread::ThreadId, String)>>);

impl log::Log for MidiStateLog {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() == log::Level::Error && metadata.target().ends_with("::midi")
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let text = record.args().to_string();
            if text.starts_with("MIDI rejected new input:") {
                self.0
                    .lock()
                    .unwrap()
                    .push((std::thread::current().id(), text));
            }
        }
    }

    fn flush(&self) {}
}

fn midi_state_log() -> &'static MidiStateLog {
    static LOG: MidiStateLog = MidiStateLog(std::sync::Mutex::new(Vec::new()));
    static START: std::sync::Once = std::sync::Once::new();
    START.call_once(|| {
        log::set_logger(&LOG).unwrap();
        log::set_max_level(log::LevelFilter::Error);
    });
    &LOG
}

impl Drive {
    fn invalid_state_checks() -> Self {
        midi_state_log();
        let mut drive = Self::new();
        drive.bind_action(84, "bank.next");
        drive.bind_pad_button(4);
        drive.app.screen = Screen::Fx;
        drive.app.pads.bank = 2;
        drive.app.pads.selected = 7;
        drive.frame();
        assert_full_hit(&drive.pad_press(Source::Notes, 2), 34);
        assert_only_release(&drive.pad_release(Source::Notes, 2), 34);
        assert_full_hit(&drive.pad_press(Source::Button, 4), 36);
        assert_only_release(&drive.pad_release(Source::Button, 4), 36);
        drive.bytes(XS_PORT, &[0xb0, 24, 32]);
        let knob = drive.queued_events();
        assert!(
            knob.iter()
                .any(|event| matches!(event.event, midi::Event::Param { .. }))
        );
        for event in knob {
            drive.app.midi_ui_event(event);
        }
        drive.frame();
        drive
    }

    fn invalid_errors(&self) -> Vec<String> {
        let id = self.worker.as_ref().unwrap().thread().id();
        midi_state_log()
            .0
            .lock()
            .unwrap()
            .iter()
            .filter(|(thread, _)| *thread == id)
            .map(|(_, text)| text.clone())
            .collect()
    }

    fn rejected_input(&self) {
        for _ in 0..2 {
            for bytes in [
                [0x90, 38, 101],
                [0xb0, 80, 127],
                [0xb0, 84, 127],
                [0x80, 39, 0],
                [0x90, 38, 0],
                [0xb0, 80, 0],
            ] {
                self.bytes(PAD_PORT, &bytes);
            }
            self.bytes(XS_PORT, &[0xb0, 24, 127]);
        }
    }

    fn invalid_frame(&mut self) -> Frame {
        self.midi().recorded_fence();
        let direct = std::iter::from_fn(|| self.direct.as_mut()?.pop().ok()).collect();
        for event in self.midi().events.try_iter().collect::<Vec<_>>() {
            assert_eq!(
                event.sequence(),
                None,
                "invalid input cannot number UI work"
            );
            assert!(!matches!(
                event.event,
                midi::Event::PadHit { .. }
                    | midi::Event::Action(_)
                    | midi::Event::Param { .. }
                    | midi::Event::PadRelease { direct: false, .. }
            ));
            self.app.midi_ui_event(event);
        }
        // Unlike a normal UI frame, this observation must not publish valid state.
        Frame {
            direct,
            ui: std::mem::take(&mut self.app.sent),
        }
    }

    fn assert_invalid_input_rejected(&mut self, bits: u32) {
        let before = self.seen();
        self.shared.inject_focus(bits);
        assert_eq!(self.shared.focus(), Err(bits));
        self.rejected_input();
        assert!(self.invalid_frame().commands().is_empty());
        assert_eq!(self.seen(), before);
        assert!(self.app.midi_ui_held.is_empty());
        assert!(self.app.audio_held.iter().all(Option::is_none));
        assert!(self.app.pending_audio.is_empty());
        assert_eq!(
            self.invalid_errors(),
            [format!(
                "MIDI rejected new input: invalid state {bits:#010x}"
            )]
        );
        // Rejected hits must not create holds which later disconnect could release.
        self.midi().recorded_disconnect(PAD_PORT);
        assert!(self.invalid_frame().commands().is_empty());
        assert_eq!(self.shared.focus(), Err(bits));
        assert_eq!(self.invalid_errors().len(), 1);
    }
}

#[test]
fn unknown_midi_screen_rejects_new_input() {
    for code in 5..=7 {
        let mut drive = Drive::invalid_state_checks();
        let bits = (drive.app.focus().pack().unwrap() & !7) | code;
        drive.assert_invalid_input_rejected(bits);
    }
}

#[test]
fn other_invalid_midi_state_has_no_fallback() {
    for bank in 8..=15 {
        let mut drive = Drive::invalid_state_checks();
        let bits = (drive.app.focus().pack().unwrap() & !(15 << 3)) | (bank << 3);
        drive.assert_invalid_input_rejected(bits);
    }
    for bit in 20..32 {
        let mut drive = Drive::invalid_state_checks();
        let bits = drive.app.focus().pack().unwrap() | (1 << bit);
        drive.assert_invalid_input_rejected(bits);
    }
}

#[test]
fn valid_midi_state_resumes_without_rejected_input() {
    use engine::checks::peak;
    midi_state_log();
    let mut drive = Drive::offline();
    drive.parity_pad(0, false);
    drive.tap(KeyCode::F(2), M::NONE);
    drive.tap(KeyCode::Char('1'), M::CONTROL);
    drive.bind_action(84, "bank.next");
    drive.bind_pad_button(4);
    drive.render_blocks(32);
    drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
    drive.bytes(PAD_PORT, &[0xb0, 84, 0]);
    let earlier = drive.queued_events();
    let sequence = earlier.iter().find_map(|event| event.sequence()).unwrap();
    assert_eq!(drive.app.pads.bank, 0);
    drive.shared.inject_focus(5);
    drive.rejected_input();
    assert!(drive.invalid_frame().commands().is_empty());
    assert_eq!(drive.app.pads.bank, 0);
    assert_eq!(drive.invalid_errors().len(), 1);
    drive.app.publish_midi(&drive.shared);
    drive.bytes(PAD_PORT, &[0x90, 38, 101]);
    drive.bytes(PAD_PORT, &[0x80, 38, 0]);
    let next = drive.queued_events();
    assert_eq!(peak(&drive.render_blocks(2).0), 0.0);
    assert!(
        next.iter()
            .any(|event| event.sequence() == Some(sequence.wrapping_add(1))
                && matches!(
                    event.event,
                    midi::Event::PadHit {
                        sound: midi::PadSound::UiUnattempted { deferred: true },
                        ..
                    }
                ))
    );
    assert_eq!(
        next.iter()
            .filter(|event| event.sequence().is_some())
            .count(),
        1
    );
    for event in earlier.into_iter().chain(next) {
        drive.app.midi_ui_event(event);
    }
    let frame = drive.frame();
    assert_eq!(drive.app.pads.bank, 1);
    assert!(
        matches!(frame.commands()[..], [Command::PadPress {pad: 18, velocity},
        Command::PadRelease {pad: 18}] if *velocity == 101.0 / 127.0),
        "commands: {:?}",
        frame.commands()
    );
    assert!(drive.app.midi_ui_held.is_empty());
    drive.render_blocks(32);
    drive.bytes(PAD_PORT, &[0x90, 38, 77]);
    let resumed = drive.queued_events();
    assert!(resumed.iter().any(|event| event.sequence().is_none()
        && matches!(event.event,
        midi::Event::PadHit {abs: 18, velocity, sound: midi::PadSound::DirectAccepted, ..}
        if velocity == 77.0 / 127.0)));
    for event in resumed {
        drive.app.midi_ui_event(event);
    }
    assert!(drive.frame().ui.is_empty());
    assert!(peak(&drive.render_blocks(8).0) > 0.01);
    drive.bytes(PAD_PORT, &[0x80, 38, 0]);
    assert!(drive.frame().ui.is_empty());
    assert_eq!(drive.app.audio_held[18], None);
    assert_eq!(peak(&drive.render_blocks(64).0[4_096..]), 0.0);
    drive.shared.inject_focus(6);
    drive.rejected_input();
    assert!(drive.invalid_frame().commands().is_empty());
    drive.shared.inject_focus(7);
    drive.rejected_input();
    assert!(drive.invalid_frame().commands().is_empty());
    assert_eq!(
        drive.invalid_errors(),
        [
            "MIDI rejected new input: invalid state 0x00000005",
            "MIDI rejected new input: invalid state 0x00000006",
        ]
    );
    drive.app.publish_midi(&drive.shared);
    drive.bytes(PAD_PORT, &[0x90, 38, 90]);
    let hit = drive.queued_events();
    assert!(hit.iter().any(|event| event.sequence().is_none()
        && matches!(event.event,
        midi::Event::PadHit {abs: 18, velocity, sound: midi::PadSound::DirectAccepted, ..}
        if velocity == 90.0 / 127.0)));
}

#[test]
fn invalid_midi_state_preserves_original_releases() {
    midi_state_log();
    for source in [Source::Notes, Source::Button] {
        for release in [
            RecordedRelease::Off,
            RecordedRelease::Zero,
            RecordedRelease::Disconnect,
        ] {
            for full_ui_queue in [false, true] {
                let mut drive = Drive::new();
                drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 48_000])), 4);
                drive.link_chop_pad(18, 3);
                drive.link_chop_pad(34, 3);
                drive.tap(KeyCode::F(2), M::NONE);
                drive.tap(KeyCode::Char('2'), M::CONTROL);
                assert_full_hit(&drive.pad_press(source, 2), 18);
                drive.tap(KeyCode::F(3), M::NONE);
                drive.tap(KeyCode::Char('3'), M::CONTROL);
                let hit = drive.pad_press(source, 2);
                assert_full_hit(&hit, 34);
                assert!(hit.direct.is_empty());
                assert_eq!(drive.app.midi_ui_held.len(), 1);
                assert!(drive.app.pads.held[18] && drive.app.pads.held[34]);
                if full_ui_queue {
                    let mut filled = 0;
                    while drive
                        .app
                        .audio
                        .as_mut()
                        .unwrap()
                        .commands
                        .push(ProjectCommand {
                            epoch: 0,
                            ready_at_send: true,
                            command: ProjectMessage::Command(Command::SetPadGain {
                                pad: 127,
                                gain: 1.0,
                            }),
                        })
                        .is_ok()
                    {
                        filled += 1;
                    }
                    assert!(filled > 0);
                }
                drive.shared.inject_focus(5);
                drive.bytes(PAD_PORT, &[0x80, 40, 0]);
                assert!(drive.invalid_frame().commands().is_empty());
                drive.raw_ordered_release(source, release);
                let frame = drive.invalid_frame();
                assert!(matches!(
                    frame.direct[..],
                    [ProjectCommand {
                        epoch: 0,
                        command: ProjectMessage::Command(Command::PadRelease { pad: 18 }),
                        ..
                    }]
                ));
                assert!(matches!(frame.ui[..], [Command::PadRelease { pad: 34 }]));
                assert!(drive.app.midi_ui_held.is_empty());
                assert!(!drive.app.pads.held[18] && !drive.app.pads.held[34]);
                assert_eq!(drive.shared.focus(), Err(5));
                assert_eq!(drive.invalid_errors().len(), 1);
                if full_ui_queue {
                    assert_eq!(drive.app.pending_audio.len(), 1);
                    drive.render_blocks(1);
                    drive.app.flush_audio();
                    assert!(drive.app.pending_audio.is_empty());
                    assert_eq!(drive.shared.focus(), Err(5));
                }
            }
        }
    }
}

#[test]
fn invalid_midi_state_keeps_release_sources_separate() {
    midi_state_log();
    const OTHER: &str = "Recorded pads MIDI 2";
    let mut drive = Drive::new();
    drive.midi().recorded_connect(OTHER);
    drive.frame();
    for (bank, port, channel, note, abs) in [
        (1, PAD_PORT, 0, 38, 18),
        (2, PAD_PORT, 1, 38, 34),
        (3, OTHER, 0, 38, 50),
        (4, PAD_PORT, 0, 39, 67),
    ] {
        drive.tap(KeyCode::Char(char::from(b'1' + bank)), M::CONTROL);
        drive.bytes(port, &[0x90 | channel, note, 127]);
        assert_full_hit(&drive.frame(), abs);
    }
    drive.shared.inject_focus(5);
    for (port, status, note) in [
        (PAD_PORT, 0x82, 38),
        (OTHER, 0x81, 38),
        (PAD_PORT, 0x80, 40),
    ] {
        drive.bytes(port, &[status, note, 0]);
    }
    assert!(drive.invalid_frame().commands().is_empty());
    drive.bytes(PAD_PORT, &[0x80, 38, 0]);
    assert_only_release(&drive.invalid_frame(), 18);
    for abs in [34, 50, 67] {
        assert!(drive.app.pads.held[abs]);
    }
    drive.midi().recorded_disconnect(PAD_PORT);
    let frame = drive.invalid_frame();
    let mut released: Vec<_> = frame
        .commands()
        .into_iter()
        .map(|command| match command {
            Command::PadRelease { pad } => *pad,
            other => panic!("not a release: {other:?}"),
        })
        .collect();
    released.sort_unstable();
    assert_eq!(released, [34, 67]);
    assert!(drive.app.pads.held[50]);
    drive.midi().recorded_disconnect(OTHER);
    assert_only_release(&drive.invalid_frame(), 50);
    assert!(drive.app.pads.held.iter().all(|held| !held));
    assert_eq!(drive.invalid_errors().len(), 1);
}

#[test]
fn invalid_midi_state_retries_original_direct_releases() {
    midi_state_log();
    for release in [RecordedRelease::Off, RecordedRelease::Disconnect] {
        let mut drive = Drive::new();
        drive.tap(KeyCode::Char('2'), M::CONTROL);
        drive.bytes(PAD_PORT, &[0x90, 38, 127]);
        assert_full_hit(&drive.frame(), 18);
        for _ in 0..1024 {
            drive.bytes(PAD_PORT, &[0x90, 39, 90]);
        }
        for event in drive.queued_events() {
            assert_eq!(event.sequence(), None);
            drive.app.midi_ui_event(event);
        }
        drive.shared.inject_focus(5);
        drive.bytes(PAD_PORT, &[0x80, 40, 0]);
        drive.raw_ordered_release(Source::Notes, release);
        for event in drive.queued_events() {
            assert_eq!(event.sequence(), None);
            drive.app.midi_ui_event(event);
        }
        for _ in 0..1024 {
            let command = drive.direct.as_mut().unwrap().pop().unwrap();
            assert_eq!(command.epoch, 0);
            assert!(matches!(
                command.command,
                ProjectMessage::Command(Command::PadPress { pad: 19, .. })
            ));
        }
        drive.midi().recorded_fence();
        let frame = drive.invalid_frame();
        assert!(frame.ui.is_empty());
        assert!(
            frame
                .direct
                .iter()
                .all(|command| command.epoch == 0 && command.ready_at_send)
        );
        let mut released: Vec<_> = frame
            .commands()
            .into_iter()
            .map(|command| match command {
                Command::PadRelease { pad } => *pad,
                other => panic!("not a release: {other:?}"),
            })
            .collect();
        released.sort_unstable();
        assert_eq!(
            released,
            if matches!(release, RecordedRelease::Disconnect) {
                vec![18, 19]
            } else {
                vec![18]
            }
        );
        assert_eq!(drive.shared.focus(), Err(5));
        assert_eq!(drive.invalid_errors().len(), 1);
    }
}

#[test]
fn invalid_midi_state_cannot_release_a_new_project() {
    use engine::checks::{first_non_finite, peak};
    use std::sync::atomic::Ordering::Acquire;
    midi_state_log();
    for source in [Source::Notes, Source::Button] {
        for ui_route in [false, true] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            if !ui_route {
                drive.tap(KeyCode::F(2), M::NONE);
            }
            drive.velocity_hit(source, 127);
            assert!(peak(&drive.render_blocks(8).0) > 0.01);
            drive.shared.inject_focus(5);
            drive.app.apply_session(session::Session::default());
            for _ in 0..256 {
                drive.app.flush_audio();
                drive.render_blocks(1);
                if drive.project.ready.load(Acquire) == 1 && drive.app.pending_audio.is_empty() {
                    break;
                }
            }
            assert_eq!(drive.project.ready.load(Acquire), 1);
            assert!(drive.app.pending_audio.is_empty());
            drive.frame();
            drive.app.pads.track_keys = true;
            drive.parity_pad(0, false);
            drive.tap(KeyCode::F(2), M::NONE);
            drive.render_blocks(32);
            drive.velocity_hit(Source::Keyboard, 127);
            assert!(peak(&drive.render_blocks(32).0) > 0.01);
            assert_eq!(drive.app.audio_held[18], Some(1));
            drive.shared.inject_focus(5);
            drive.raw_ordered_release(source, RecordedRelease::Off);
            assert!(drive.invalid_frame().commands().is_empty());
            assert_eq!(drive.app.audio_held[18], Some(1));
            let wave = drive.render_blocks(32);
            assert_eq!(first_non_finite(&wave.0), None);
            assert!(peak(&wave.0) > 0.01 && peak(&wave.0) < 1.0);
            drive.key(KeyCode::Char('3'), M::SHIFT, KeyEventKind::Release);
            assert_only_release(&drive.invalid_frame(), 18);
            assert_eq!(peak(&drive.render_blocks(64).0[4_096..]), 0.0);
            assert_eq!(drive.shared.focus(), Err(5));
            assert_eq!(drive.invalid_errors().len(), 1);
        }
    }
}

#[test]
fn invalid_midi_state_keeps_audio_and_unrelated_holds_running() {
    use engine::checks::{first_non_finite, peak};
    midi_state_log();
    for full_midi_queue in [false, true] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        let pad = drive.app.pads.snapshot(18);
        for command in drive.app.pads.put_back(19, &pad) {
            drive.app.send(command);
        }
        drive.app.send(Command::SetPadSound {
            pad: 31,
            sound: None,
        });
        drive.tap(KeyCode::F(2), M::NONE);
        drive.render_blocks(32);
        drive.velocity_hit(Source::Notes, 127);
        drive.pad_press(Source::Keyboard, 3);
        assert!(peak(&drive.render_blocks(8).0) > 0.01);
        if full_midi_queue {
            // Fill the callback's MIDI queue with hits on an unloaded pad.
            for _ in 0..1024 {
                drive.bytes(PAD_PORT, &[0x90, 51, 127]);
            }
            let hits = drive.queued_events();
            assert_eq!(
                hits.iter()
                    .filter(|event| matches!(
                        event.event,
                        midi::Event::PadHit {
                            abs: 31,
                            sound: midi::PadSound::DirectAccepted,
                            ..
                        }
                    ))
                    .count(),
                1024
            );
            for event in hits {
                drive.app.midi_ui_event(event);
            }
        }
        drive.shared.inject_focus(5);
        drive.bytes(PAD_PORT, &[0x80, 38, 0]);
        assert!(drive.invalid_frame().commands().is_empty());
        assert!(!drive.app.pads.held[18] && drive.app.pads.held[19]);
        assert_eq!(drive.app.audio_held[19], Some(0));
        if full_midi_queue {
            drive.render_blocks(1);
            drive.midi().recorded_fence();
        }
        let wave = drive.render_blocks(32);
        assert_eq!(first_non_finite(&wave.0), None);
        assert!(
            peak(&wave.0) > 0.01 && peak(&wave.0) < 1.0,
            "full MIDI queue: {full_midi_queue}, peak: {}",
            peak(&wave.0)
        );
        drive.key(KeyCode::Char('4'), M::SHIFT, KeyEventKind::Release);
        assert_only_release(&drive.invalid_frame(), 19);
        assert_eq!(peak(&drive.render_blocks(64).0[4_096..]), 0.0);
        assert_eq!(drive.shared.focus(), Err(5));
        assert_eq!(drive.invalid_errors().len(), 1);
    }
}

#[test]
fn recorded_driver_uses_both_production_pad_routes() {
    let mut keys = Drive::new();
    keys.key(KeyCode::Char('q'), M::SHIFT, KeyEventKind::Press);
    let keyboard = keys.frame();
    assert!(keyboard.direct.is_empty());
    assert!(matches!(
        keyboard.ui[..],
        [Command::PadPress {
            pad: 4,
            velocity: 1.0
        }]
    ));
    assert_eq!(keys.app.pads.selected, 4);
    keys.key(KeyCode::Char('q'), M::SHIFT, KeyEventKind::Release);
    assert!(matches!(
        keys.frame().ui[..],
        [Command::PadRelease { pad: 4 }]
    ));

    let mut notes = Drive::new();
    notes.bytes(PAD_PORT, &[0x90, 40, 127]);
    let controller = notes.frame();
    assert!(
        controller.ui.is_empty(),
        "a direct hit must not sound twice"
    );
    assert!(matches!(
        controller.commands()[..],
        [Command::PadPress {
            pad: 4,
            velocity: 1.0
        }]
    ));
    assert_eq!(notes.app.pads.selected, 4);
    notes.bytes(PAD_PORT, &[0x80, 40, 0]);
    let release = notes.frame();
    assert!(release.ui.is_empty());
    assert!(matches!(
        release.commands()[..],
        [Command::PadRelease { pad: 4 }]
    ));
}

#[test]
fn recorded_driver_applies_seeded_buttons_and_explicit_frames() {
    let mut drive = Drive::new();
    drive.bytes(PAD_PORT, &[0xb0, 27, 127]);
    drive.frame();
    assert_eq!(drive.app.pads.bank, 1);
    drive.bytes(PAD_PORT, &[0xb0, 27, 0]);
    drive.frame();
    assert_eq!(
        drive.app.pads.bank, 1,
        "button release is not another bank turn"
    );
    drive.bytes(PAD_PORT, &[0x90, 36, 127]);
    let hit = drive.frame();
    assert!(matches!(
        hit.commands()[..],
        [Command::PadPress {
            pad: 16,
            velocity: 1.0
        }]
    ));
    drive.bytes(PAD_PORT, &[0x90, 36, 0]);
    drive.frame();
    drive.tap(KeyCode::PageUp, M::NONE);
    assert_eq!(drive.app.pads.bank, 0);
    drive.bytes(XS_PORT, &[0x90, 45, 127]);
    drive.frame();
    assert_eq!(
        drive.app.screen,
        Screen::Fx,
        "the shipped X-Session profile acts after App seeding"
    );
}

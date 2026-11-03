//! Recorded input through the production keyboard and MIDI routes. No device or sound card.

use super::*;
use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers as M};

const PAD_PORT: &str = "Recorded pads MIDI 1";
const XS_PORT: &str = "USB X-Session MIDI 1";

struct Frame {
    direct: Vec<ProjectCommand>,
    ui: Vec<Command>,
}

impl Frame {
    fn commands(&self) -> Vec<&Command> {
        self.direct
            .iter()
            .map(|message| {
                assert!(message.ready_at_send);
                let ProjectMessage::Command(command) = &message.command else {
                    panic!("a recorded pad frame must not contain a project marker");
                };
                command
            })
            .chain(self.ui.iter())
            .collect()
    }
}

struct Drive {
    app: App,
    keymap: Keymap,
    down: input::HeldKeys,
    held: HeldPads,
    mods: input::HeldModifiers,
    shared: Arc<midi::Shared>,
    project: Arc<ProjectState>,
    direct: Option<rtrb::Consumer<ProjectCommand>>,
    worker: Option<std::thread::JoinHandle<()>>,
    audio_rt: Option<engine::rt::Realtime>,
}

fn pad_profile() -> Profile {
    let mut text = String::from(
        "[device]\nname = \"Recorded pads\"\nmatch_port = \"Recorded pads\"\n\
         [controls.pads]\n",
    );
    for pad in 1..=16 {
        text.push_str(&format!("p{pad} = {{ note = {} }}\n", 35 + pad));
    }
    text.push_str(
        "[controls.buttons]\ncopy = { cc = 25 }\nback = { cc = 26 }\n\
         bank = { cc = 27 }\nmark = { cc = 28 }\nedit = { cc = 29 }\n\
         [screens.global]\ncopy = \"pad.copy\"\nback = \"ui.back\"\nbank = \"bank.next\"\n\
         [screens.pads]\nmark = \"pad.mark_toggle\"\nedit = \"pad.edit\"\n",
    );
    Profile::from_toml("recorded-pad-profile.toml", &text).unwrap()
}

impl Drive {
    fn new() -> Self {
        Self::with_audio(false)
    }

    fn offline() -> Self {
        Self::with_audio(true)
    }

    fn with_audio(offline: bool) -> Self {
        let mut app = tests::test_app();
        app.pads.track_keys = true;
        app.profiles = vec![
            pad_profile(),
            Profile::from_toml(
                "xsession-pro.toml",
                include_str!("../../../config/devices/xsession-pro.toml"),
            )
            .unwrap(),
        ];
        let shared = Arc::new(midi::Shared::default());
        app.publish_midi(&shared);
        // In render tests the MIDI writer owns the callback's real queue. Do not
        // capture and replay direct commands: that would hide UI-dependent timing.
        let (project, producer, direct, audio_rt) = if offline {
            let (mut audio, rt) = audio::offline(48_000);
            let project = audio.project.clone();
            let producer = audio.midi_commands.take().unwrap();
            app.audio = Some(audio);
            (project, producer, None, Some(rt))
        } else {
            let (producer, consumer) = rtrb::RingBuffer::new(1024);
            (
                Arc::new(ProjectState::default()),
                producer,
                Some(consumer),
                None,
            )
        };
        let (midi, worker) = midi::Midi::recorded(
            app.profiles.clone(),
            producer,
            project.clone(),
            shared.clone(),
        );
        app.midi = Some(midi);
        let (keymap, warnings) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        let mut drive = Self {
            app,
            keymap,
            down: Default::default(),
            held: Default::default(),
            mods: Default::default(),
            shared,
            project,
            direct,
            worker: Some(worker),
            audio_rt,
        };
        drive.midi().recorded_connect(PAD_PORT);
        drive.midi().recorded_connect(XS_PORT);
        drive.frame(); // Connected -> App::seed_device -> real learned-map updates.
        drive.midi().recorded_fence();
        assert!(drive.app.seeded.iter().any(|name| name == "Recorded pads"));
        assert!(drive.app.seeded.iter().any(|name| name == "X-Session Pro"));
        // A fixture with no user path reports that it cannot save the seeded map.
        // Neither input pass writes the owner's config or treats this as a user notice.
        drive.app.flash = None;
        drive
    }

    fn midi(&self) -> &midi::Midi {
        self.app.midi.as_ref().unwrap()
    }

    fn key(&mut self, code: KeyCode, modifiers: M, kind: KeyEventKind) {
        assert!(!self.app.keyboard_event(
            &self.keymap,
            &KeyEvent::new_with_kind(code, modifiers, kind),
            true,
            &mut self.down,
            &mut self.held,
            &mut self.mods,
        ));
    }

    fn tap(&mut self, code: KeyCode, modifiers: M) -> Frame {
        self.key(code, modifiers, KeyEventKind::Press);
        self.key(code, modifiers, KeyEventKind::Release);
        self.frame()
    }

    fn bytes(&self, port: &str, bytes: &[u8]) {
        self.midi().recorded_bytes(port, bytes, &self.project);
    }

    fn frame(&mut self) -> Frame {
        self.midi().recorded_fence();
        let direct = std::iter::from_fn(|| self.direct.as_mut()?.pop().ok()).collect();
        let events: Vec<_> = self.midi().events.try_iter().collect();
        for event in events {
            self.app.midi_ui_event(event);
        }
        self.app.publish_midi(&self.shared);
        self.app.frame_notice();
        Frame {
            direct,
            ui: std::mem::take(&mut self.app.sent),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Source {
    Keyboard,
    Notes,
    Button,
}

#[derive(Debug, PartialEq)]
struct Seen {
    screen: Screen,
    edit: bool,
    param: pads::Param,
    held: Vec<bool>,
    bank: usize,
    selected: usize,
    slice: Option<usize>,
    chop_cursor: Option<usize>,
    plan: Option<chop::Plan>,
    file_hand: Option<PathBuf>,
    swap_hand: Option<usize>,
    copy_hand: Option<usize>,
    marks: Vec<usize>,
    marking: bool,
    question: Vec<String>,
    notice: Option<String>,
    history: Vec<(String, Vec<usize>)>,
    pads: Vec<pads::PadSnap>,
}

impl Drive {
    fn seen(&self) -> Seen {
        Seen {
            screen: self.app.screen,
            edit: self.app.edit,
            param: self.app.pads.param,
            held: self.app.pads.held.to_vec(),
            bank: self.app.pads.bank,
            selected: self.app.pads.selected,
            slice: self.app.chop.as_ref().map(|chop| chop.selected),
            chop_cursor: self.app.chop.as_ref().map(|chop| chop.cursor),
            plan: self.app.plan.clone(),
            file_hand: self.app.browser.in_hand.clone(),
            swap_hand: self.app.pad_in_hand,
            copy_hand: self.app.copy_in_hand.as_ref().map(|(source, _)| *source),
            marks: self.app.pads.marked.iter().copied().collect(),
            marking: self.app.pads.marking,
            question: self
                .app
                .ask
                .as_ref()
                .map_or_else(Vec::new, |ask| ask.lines().to_vec()),
            notice: self.app.flash.as_ref().map(|(text, _)| text.clone()),
            history: self
                .app
                .history
                .iter()
                .map(|step| {
                    (
                        step.what.clone(),
                        step.pads.iter().map(|(pad, _)| *pad).collect(),
                    )
                })
                .collect(),
            pads: (0..PADS).map(|pad| self.app.pads.snapshot(pad)).collect(),
        }
    }

    fn bind_pad_button(&mut self, pad: usize) {
        learn::assign(
            &mut self.app.learned,
            "Recorded pads",
            input::Group::Global,
            input::midi::Key::Cc(80),
            &format!("pad.{}", pad + 1),
        );
        self.app.sync_learned("Recorded pads", input::Group::Global);
        self.midi().recorded_fence();
    }

    fn pad_press(&mut self, source: Source, pad: usize) -> Frame {
        match source {
            Source::Keyboard => self.key(
                KeyCode::Char("1234qwerasdfzxcv".chars().nth(pad).unwrap()),
                M::SHIFT,
                KeyEventKind::Press,
            ),
            Source::Notes => self.bytes(PAD_PORT, &[0x90, 36 + pad as u8, 127]),
            Source::Button => {
                self.bind_pad_button(pad);
                self.bytes(PAD_PORT, &[0xb0, 80, 127]);
            }
        }
        self.frame()
    }

    fn pad_release(&mut self, source: Source, pad: usize) -> Frame {
        match source {
            Source::Keyboard => self.key(
                KeyCode::Char("1234qwerasdfzxcv".chars().nth(pad).unwrap()),
                M::SHIFT,
                KeyEventKind::Release,
            ),
            Source::Notes => self.bytes(PAD_PORT, &[0x80, 36 + pad as u8, 0]),
            Source::Button => self.bytes(PAD_PORT, &[0xb0, 80, 0]),
        }
        self.frame()
    }

    fn prepare_chop(&mut self, sound: Arc<Sample>, slices: usize) {
        if self.app.audio.is_none() {
            let (audio, rt) = audio::offline(48_000);
            self.project = audio.project.clone();
            self.app.audio = Some(audio);
            self.audio_rt = Some(rt);
        }
        let step = sound.len() / slices;
        self.app.chop = Some(chop::Chop::from_saved(
            "break.wav".into(),
            sound.clone(),
            48_000,
            chop::Saved {
                range: Region {
                    start: 0,
                    end: sound.len(),
                },
                cuts: (1..slices)
                    .map(|slice| (slice as u32, slice * step))
                    .collect(),
                epoch: 1,
                next_id: slices as u32,
            },
        ));
        self.app.screen = Screen::Chop;
        self.frame();
    }

    fn link_chop_pad(&mut self, abs: usize, slice: usize) {
        let chop = self.app.chop.as_ref().unwrap();
        let (sound, region, link) = (
            chop.sound.clone(),
            chop.slices()[slice],
            (chop.start_id(slice), chop.epoch),
        );
        self.load_pad(abs, sound, "break");
        self.app.pads.region[abs] = region;
        self.app.pads.slice[abs] = Some(slice as u16 + 1);
        self.app.pads.link[abs] = Some(link);
        self.app.pads.filter[abs] = engine::filter::FilterKind::LowPass;
        self.app.pads.cutoff_hz[abs] = 1_200.0;
    }

    fn load_pad(&mut self, pad: usize, sound: Arc<Sample>, name: &str) {
        self.app.pads.sound[pad] = Some(sound);
        self.app.pads.file[pad] = Some(format!("{name}.wav").into());
        self.app.pads.custom_name[pad] = Some(name.into());
        self.app.pads.gate[pad] = true;
        self.app.pads.pitch[pad] = 3;
    }
}

fn assert_silent_target(frame: &Frame) {
    assert!(
        frame.direct.is_empty(),
        "a target must not enter the direct audio queue: {:?}",
        frame.direct
    );
    assert!(
        !frame.ui.iter().any(|command| matches!(
            command,
            Command::PadPress { .. } | Command::Preview { sound: Some(_), .. }
        )),
        "a target must not sound from the UI: {:?}",
        frame.ui
    );
}

impl Drop for Drive {
    fn drop(&mut self) {
        // Abandon the audio queue before joining, even if a failing test left it full.
        self.direct.take();
        self.audio_rt.take();
        self.app.midi.take();
        self.worker.take().unwrap().join().unwrap();
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

fn assert_full_hit(frame: &Frame, abs: usize) {
    assert!(
        matches!(frame.commands()[..], [Command::PadPress { pad, velocity }]
            if *pad == abs && *velocity == 1.0),
        "expected one full hit on pad {abs}: {:?}",
        frame.commands()
    );
}

fn paired_baseline(mut run: impl FnMut(&mut Drive, Source) -> Vec<Seen>) {
    let mut expected = None;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let states = run(&mut Drive::new(), source);
        if let Some(expected) = &expected {
            assert_eq!(&states, expected, "{source:?}");
        } else {
            expected = Some(states);
        }
    }
}

#[test]
fn baseline_pads_parity_from_raw_midi() {
    paired_baseline(|drive, source| {
        let mut states = Vec::new();
        for bank in 0..2 {
            if bank > 0 {
                drive.tap(KeyCode::PageDown, M::NONE);
            }
            assert_full_hit(&drive.pad_press(source, 5), bank * PADS_PER_BANK + 5);
            assert_eq!(drive.app.pads.selected, 5);
            assert!(drive.app.pads.held[bank * PADS_PER_BANK + 5]);
            states.push(drive.seen());
            let release = drive.pad_release(source, 5);
            assert!(
                matches!(release.commands()[..], [Command::PadRelease { pad }]
                if *pad == bank * PADS_PER_BANK + 5)
            );
            assert!(!drive.app.pads.held[bank * PADS_PER_BANK + 5]);
            states.push(drive.seen());
        }
        states
    });
}

#[test]
fn baseline_pad_edit_parity_from_raw_midi() {
    paired_baseline(|drive, source| {
        match source {
            Source::Keyboard => {
                drive.tap(KeyCode::Enter, M::NONE);
            }
            Source::Notes | Source::Button => {
                drive.bytes(PAD_PORT, &[0xb0, 29, 127]);
                drive.frame();
                drive.bytes(PAD_PORT, &[0xb0, 29, 0]);
                drive.frame();
            }
        }
        assert!(drive.app.edit);
        drive.tap(KeyCode::Tab, M::NONE);
        let mut states = vec![drive.seen()];
        assert_full_hit(&drive.pad_press(source, 4), 4);
        assert!(drive.app.edit, "a pad hit keeps edit open");
        states.push(drive.seen());
        drive.pad_release(source, 4);
        states.push(drive.seen());
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(!drive.app.edit);
        states.push(drive.seen());
        states
    });
}

#[test]
fn baseline_marking_parity_from_raw_midi() {
    paired_baseline(|drive, source| {
        match source {
            Source::Keyboard => {
                drive.tap(KeyCode::Char('m'), M::NONE);
            }
            Source::Notes | Source::Button => {
                drive.bytes(PAD_PORT, &[0xb0, 28, 127]);
                drive.frame();
                drive.bytes(PAD_PORT, &[0xb0, 28, 0]);
                drive.frame();
            }
        }
        assert!(drive.app.pads.marking);
        let mut states = Vec::new();
        for (bank, local, marked) in [(0, 4, vec![4]), (1, 2, vec![4, 18]), (0, 4, vec![18])] {
            drive.tap(KeyCode::Char(char::from(b'1' + bank)), M::CONTROL);
            assert_full_hit(
                &drive.pad_press(source, local),
                usize::from(bank) * PADS_PER_BANK + local,
            );
            assert_eq!(
                drive.app.pads.marked.iter().copied().collect::<Vec<_>>(),
                marked
            );
            states.push(drive.seen());
            drive.pad_release(source, local);
            states.push(drive.seen());
        }
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.pads.marked.is_empty());
        assert!(!drive.app.pads.marking);
        states.push(drive.seen());
        states
    });
}

#[test]
fn baseline_learn_keeps_pads_silent_from_raw_midi() {
    paired_baseline(|drive, source| {
        drive.tap(KeyCode::F(10), M::NONE);
        assert!(drive.app.screen_learn.is_some());
        let before = drive.seen();
        let hit = drive.pad_press(source, 4);
        assert!(hit.commands().is_empty(), "learn must not play a pad");
        assert_eq!(
            drive.seen(),
            before,
            "learning does not change the pad state"
        );
        drive.pad_release(source, 4);
        assert!(drive.app.screen_learn.is_some());
        // Keyboard picks an action; MIDI supplies a control. Those learner states
        // intentionally differ, but neither input may play or select the pad.
        vec![drive.seen()]
    });
}

/// Scenario "Swap target and undo parity from raw MIDI".
fn baseline_swap_parity(source: Source) {
    let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
    for occupied in [true, false] {
        let mut expected = None;
        for source in [Source::Keyboard, source] {
            let mut drive = Drive::new();
            drive.load_pad(0, sound.clone(), "source");
            if occupied {
                drive.load_pad(18, sound.clone(), "target");
            }
            let source_pad = drive.app.pads.snapshot(0);
            let target_pad = drive.app.pads.snapshot(18);
            drive.tap(KeyCode::Char('x'), M::CONTROL);
            drive.tap(KeyCode::PageDown, M::NONE);
            let hit = drive.pad_press(source, 2);
            assert_silent_target(&hit);
            assert!(matches!(
                hit.commands()[..],
                [Command::SwapPads { a: 0, b: 18 }]
            ));
            assert_eq!(drive.app.pads.snapshot(0), target_pad);
            assert_eq!(drive.app.pads.snapshot(18), source_pad);
            assert!(drive.app.pad_in_hand.is_none());
            assert_eq!(
                (drive.app.pads.selected_abs(), drive.app.history.len()),
                (18, 1),
                "{source:?}: a swap selects its target and makes one undo step"
            );
            let seen = drive.seen();
            if let Some(expected) = &expected {
                assert_eq!(&seen, expected, "{source:?}");
            } else {
                expected = Some(seen);
            }
            drive.pad_release(source, 2);
            assert_eq!(drive.app.history[0].what, "swap");
            assert_eq!(
                drive.app.history[0]
                    .pads
                    .iter()
                    .map(|(pad, _)| *pad)
                    .collect::<Vec<_>>(),
                vec![0, 18]
            );
            drive.tap(KeyCode::Char('z'), M::CONTROL);
            assert_eq!(drive.app.pads.snapshot(0), source_pad);
            assert_eq!(drive.app.pads.snapshot(18), target_pad);
            assert!(drive.app.history.is_empty());
        }
    }
}

#[test]
fn swap_cancellation_and_loading_refusal_parity_from_raw_midi() {
    let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
    paired_baseline(|drive, source| {
        drive.load_pad(0, sound.clone(), "source");
        drive.load_pad(18, sound.clone(), "target");
        let original: Vec<_> = (0..PADS).map(|pad| drive.app.pads.snapshot(pad)).collect();
        drive.tap(KeyCode::Char('x'), M::CONTROL);
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.pad_in_hand.is_none());
        let mut states = vec![drive.seen()];
        drive.tap(KeyCode::Char('x'), M::CONTROL);
        let same_source = drive.pad_press(source, 0);
        assert_silent_target(&same_source);
        assert!(same_source.commands().is_empty());
        assert!(drive.app.pad_in_hand.is_none());
        assert!(drive.app.history.is_empty());
        states.push(drive.seen());
        drive.pad_release(source, 0);
        drive.tap(KeyCode::Char('x'), M::CONTROL);
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.app.pads.loading[18] = Some(1);
        let refused = drive.pad_press(source, 2);
        assert_silent_target(&refused);
        assert!(refused.commands().is_empty());
        assert!(drive.app.pad_in_hand.is_none());
        assert!(drive.app.history.is_empty());
        assert_eq!(
            drive.app.flash.as_ref().unwrap().0,
            "Wait until loading finishes, then swap."
        );
        assert_eq!(
            (0..PADS)
                .map(|pad| drive.app.pads.snapshot(pad))
                .collect::<Vec<_>>(),
            original
        );
        states.push(drive.seen());
        states
    });
}

#[test]
fn swap_target_and_undo_parity_from_raw_midi_notes() {
    baseline_swap_parity(Source::Notes);
}

#[test]
fn swap_target_and_undo_parity_from_raw_midi_learned_button() {
    baseline_swap_parity(Source::Button);
}

/// Scenario "File target parity from raw MIDI".
fn baseline_file_hand_parity(source: Source) {
    let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
    let mut expected = None;
    for source in [Source::Keyboard, source] {
        let mut drive = Drive::new();
        drive.load_pad(18, sound.clone(), "old target");
        // An occupied target tests routing and the question without decoding a file.
        drive.app.browser.in_hand = Some("new file.wav".into());
        drive.tap(KeyCode::PageDown, M::NONE);
        let hit = drive.pad_press(source, 2);
        assert_silent_target(&hit);
        assert!(matches!(&drive.app.ask, Some(Ask::PutFile { abs: 18, .. })));
        assert_eq!(
            drive.app.pads.selected_abs(),
            18,
            "{source:?}: file target selection"
        );
        let seen = drive.seen();
        if let Some(expected) = &expected {
            assert_eq!(&seen, expected, "{source:?}");
        } else {
            expected = Some(seen);
        }
        drive.pad_release(source, 2);
        let question = drive.seen();
        assert_silent_target(&drive.pad_press(source, 3));
        assert_eq!(drive.seen(), question);
        drive.pad_release(source, 3);
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.ask.is_none());
        assert!(drive.app.browser.in_hand.is_some());
        assert!(drive.app.history.is_empty());
        assert_eq!(
            drive.app.pads.custom_name[18].as_deref(),
            Some("old target")
        );
        assert_silent_target(&drive.pad_press(source, 2));
        assert!(matches!(&drive.app.ask, Some(Ask::PutFile { abs: 18, .. })));
        drive.pad_release(source, 2);
    }
}

#[test]
fn file_target_parity_from_raw_midi_notes() {
    baseline_file_hand_parity(Source::Notes);
}

#[test]
fn file_target_parity_from_raw_midi_learned_button() {
    baseline_file_hand_parity(Source::Button);
}

/// Scenario "Question pad silence from raw MIDI".
fn baseline_question_keeps_pads_silent(source: Source) {
    let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
    for source in [Source::Keyboard, source] {
        let mut drive = Drive::new();
        for pad in [0, 1, 18] {
            drive.load_pad(pad, sound.clone(), "loaded");
        }
        drive.app.pads.marked.extend([0, 1]);
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::Backspace, M::NONE);
        assert!(matches!(&drive.app.ask, Some(Ask::ClearPads { .. })));
        let before = drive.seen();
        assert_silent_target(&drive.pad_press(source, 2));
        assert_eq!(
            drive.seen(),
            before,
            "{source:?}: a question keeps its state"
        );
        drive.pad_release(source, 2);
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.ask.is_none());
        assert_eq!(
            drive.app.pads.marked.iter().copied().collect::<Vec<_>>(),
            vec![0, 1]
        );
        let hit = drive.pad_press(source, 2);
        assert_full_hit(&hit, 18);
        match source {
            Source::Keyboard => assert!(hit.direct.is_empty()),
            Source::Notes | Source::Button => assert!(hit.ui.is_empty()),
        }
        drive.pad_release(source, 2);
    }
}

/// Scenario "Question route preserves held releases".
#[test]
fn question_route_preserves_held_releases() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
        drive.load_pad(0, sound.clone(), "held");
        drive.load_pad(16, sound, "target");
        assert_full_hit(&drive.pad_press(source, 0), 0);
        drive.tap(KeyCode::F(6), M::NONE);
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.app.browser.in_hand = Some("new file.wav".into());
        drive.frame();
        drive.tap(KeyCode::Enter, M::NONE);
        assert_eq!(drive.app.screen, Screen::Files);
        assert!(matches!(&drive.app.ask, Some(Ask::PutFile { abs: 16, .. })));
        assert!(drive.app.pads.held[0]);
        let question = drive.app.ask.as_ref().unwrap().lines().to_vec();
        let release = drive.pad_release(source, 0);
        assert!(
            matches!(release.commands()[..], [Command::PadRelease { pad: 0 }]),
            "{source:?}: {:?}",
            release.commands()
        );
        assert!(!drive.app.pads.held[0]);
        assert!(!drive.app.pads.held[16]);
        assert_eq!(drive.app.ask.as_ref().unwrap().lines(), question);
    }
}

#[test]
fn question_pad_silence_from_raw_midi_notes() {
    baseline_question_keeps_pads_silent(Source::Notes);
}

#[test]
fn question_pad_silence_from_raw_midi_learned_button() {
    baseline_question_keeps_pads_silent(Source::Button);
}

fn assert_slice_preview(frame: &Frame, sound: &Arc<Sample>, expected: Region) {
    assert!(frame.direct.is_empty(), "Chop must stay on the UI route");
    assert!(
        matches!(&frame.ui[..], [Command::PreviewAs { pad: None }, Command::Preview { sound: Some(actual), region }]
        if Arc::ptr_eq(actual, sound) && *region == expected),
        "one slice preview: {:?}",
        frame.ui
    );
}

/// Scenario "Chop parity before pad assignment".
#[test]
fn chop_parity_before_pad_assignment() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    paired_baseline(|drive, source| {
        drive.prepare_chop(sound.clone(), 20);
        let slices = drive.app.chop.as_ref().unwrap().slices();
        let mut states = Vec::new();
        for (bank, local, slice) in [(0, 5, 5), (1, 2, 18), (7, 3, 19)] {
            drive.tap(KeyCode::Char(char::from(b'1' + bank)), M::CONTROL);
            let hit = drive.pad_press(source, local);
            assert_slice_preview(&hit, &sound, slices[slice]);
            assert_eq!(
                drive.app.chop.as_ref().unwrap().selected,
                slice,
                "{source:?}"
            );
            states.push(drive.seen());
            drive.pad_release(source, local);
        }
        states
    });
}

/// Scenario "Chop parity with linked pads". Sound settings are checked by the render in 2.2.
#[test]
fn chop_parity_with_linked_pads() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    paired_baseline(|drive, source| {
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(5, 2);
        drive.link_chop_pad(18, 3);
        let original = drive.seen().pads;
        let mut states = Vec::new();
        for (bank, local, abs, slice) in [(0, 5, 5, 2), (1, 2, 18, 3)] {
            drive.tap(KeyCode::Char(char::from(b'1' + bank)), M::CONTROL);
            let hit = drive.pad_press(source, local);
            assert!(hit.direct.is_empty());
            assert_full_hit(&hit, abs);
            assert_eq!(drive.app.chop.as_ref().unwrap().selected, slice);
            assert_eq!(drive.app.pads.selected_abs(), abs);
            assert_eq!(drive.seen().pads, original);
            states.push(drive.seen());
            drive.pad_release(source, local);
        }
        states
    });
}

/// Scenario "Chop MIDI linked pad keeps velocity". The level check belongs to render 2.2.
#[test]
fn chop_midi_linked_pad_keeps_velocity() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        for velocity in [23, 101] {
            match source {
                Source::Notes => drive.bytes(PAD_PORT, &[0x90, 38, velocity]),
                Source::Button => drive.bytes(PAD_PORT, &[0xb0, 80, velocity]),
                Source::Keyboard => unreachable!(),
            }
            let hit = drive.frame();
            assert!(hit.direct.is_empty());
            assert!(
                matches!(hit.commands()[..], [Command::PadPress { pad: 18, velocity: actual }]
                if *actual == f32::from(velocity) / 127.0),
                "{source:?}: {:?}",
                hit.commands()
            );
            assert_eq!(drive.app.chop.as_ref().unwrap().selected, 3);
            drive.pad_release(source, 2);
        }
    }
}

/// Scenario "Chop parity in by-hand assignment".
#[test]
fn chop_parity_in_by_hand_assignment() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    paired_baseline(|drive, source| {
        drive.prepare_chop(sound.clone(), 4);
        drive.load_pad(18, sound.clone(), "occupied");
        let original = drive.seen().pads;
        let slices = drive.app.chop.as_ref().unwrap().slices();
        drive.tap(KeyCode::Char('p'), M::NONE);
        assert_eq!(drive.app.plan, Some(chop::Plan::ByHand(Vec::new())));
        let mut states = Vec::new();
        let mut targets = Vec::new();
        for (slice, (bank, local)) in [(0, 5), (1, 2), (1, 4), (0, 6)].into_iter().enumerate() {
            drive.tap(KeyCode::Char(char::from(b'1' + bank)), M::CONTROL);
            let hit = drive.pad_press(source, local);
            assert_slice_preview(&hit, &sound, slices[slice]);
            targets.push((slice, usize::from(bank) * PADS_PER_BANK + local));
            let mut expected = targets.clone();
            expected.sort_by_key(|target| target.1);
            assert_eq!(
                drive.app.plan.as_ref().unwrap().targets(4),
                expected,
                "{source:?}"
            );
            assert_eq!(
                drive.app.chop.as_ref().unwrap().selected,
                (slice + 1).min(3)
            );
            assert!(drive.app.ask.is_none());
            assert_eq!(
                drive.seen().pads,
                original,
                "a hit plans; it does not apply"
            );
            states.push(drive.seen());
            drive.pad_release(source, local);
        }
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(matches!(&drive.app.ask, Some(Ask::Plan { .. })));
        assert_eq!(
            drive.seen().pads,
            original,
            "only confirmation may replace the occupied target"
        );
        states.push(drive.seen());
        states
    });
}

/// Scenario "Chop parity while a question is open".
#[test]
fn chop_parity_while_a_question_is_open() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    paired_baseline(|drive, source| {
        drive.prepare_chop(sound.clone(), 4);
        drive.load_pad(18, sound.clone(), "occupied");
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::Char('p'), M::NONE);
        drive.pad_press(Source::Keyboard, 2);
        drive.pad_release(Source::Keyboard, 2);
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(matches!(&drive.app.ask, Some(Ask::Plan { .. })));
        let before = drive.seen();
        let hit = drive.pad_press(source, 4);
        assert!(hit.commands().is_empty());
        assert_eq!(drive.seen(), before);
        vec![drive.seen()]
    });
}

/// Scenario "Chop parity while learning".
#[test]
fn chop_parity_while_learning() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    paired_baseline(|drive, source| {
        drive.prepare_chop(sound.clone(), 4);
        drive.tap(KeyCode::Char('p'), M::NONE);
        drive.tap(KeyCode::F(10), M::NONE);
        assert!(drive.app.screen_learn.is_some());
        let before = drive.seen();
        let hit = drive.pad_press(source, 2);
        assert!(hit.commands().is_empty());
        assert_eq!(drive.seen(), before);
        vec![drive.seen()]
    });
}

fn check_chop_linked_gate_release(change_context: bool, source: Source) {
    for source in [Source::Keyboard, source] {
        let mut drive = Drive::new();
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 32_000])), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        let hit = drive.pad_press(source, 2);
        assert!(hit.direct.is_empty());
        assert_full_hit(&hit, 18);
        assert!(drive.app.pads.held[18]);
        if change_context {
            drive.tap(KeyCode::PageDown, M::NONE);
            drive.tap(KeyCode::F(4), M::NONE);
        }
        let release = drive.pad_release(source, 2);
        assert!(
            matches!(release.commands()[..], [Command::PadRelease { pad: 18 }]),
            "{source:?}, context changed: {change_context}, original held: {}: {:?}",
            drive.app.pads.held[18],
            release.commands()
        );
        assert!(!drive.app.pads.held[18]);
    }
}

/// Scenario "Chop linked gate release reaches audio".
#[test]
fn chop_linked_gate_release_reaches_audio() {
    check_chop_linked_gate_release(false, Source::Notes);
}

#[test]
fn chop_linked_button_gate_release_reaches_audio() {
    check_chop_linked_gate_release(false, Source::Button);
}

/// Scenario "Chop linked gate release keeps original pad".
#[test]
fn chop_linked_gate_release_keeps_original_pad() {
    check_chop_linked_gate_release(true, Source::Notes);
}

#[test]
fn chop_linked_button_gate_release_keeps_original_pad() {
    check_chop_linked_gate_release(true, Source::Button);
}

fn assert_only_release(frame: &Frame, pad: usize) {
    assert!(
        matches!(frame.commands()[..], [Command::PadRelease { pad: actual }] if *actual == pad),
        "release {pad}: {:?}",
        frame.commands()
    );
}

#[test]
fn chop_linked_gate_release_passes_question_and_learn_guards() {
    for source in [Source::Notes, Source::Button] {
        for learn in [false, true] {
            let mut drive = Drive::new();
            let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
            drive.prepare_chop(sound.clone(), 4);
            drive.link_chop_pad(18, 3);
            drive.load_pad(34, sound, "occupied");
            drive.tap(KeyCode::PageDown, M::NONE);
            assert_full_hit(&drive.pad_press(source, 2), 18);
            if learn {
                drive.tap(KeyCode::F(10), M::NONE);
                assert!(drive.app.screen_learn.is_some());
            } else {
                drive.tap(KeyCode::F(6), M::NONE);
                drive.tap(KeyCode::PageDown, M::NONE);
                drive.app.browser.in_hand = Some("new file.wav".into());
                drive.frame();
                drive.tap(KeyCode::Enter, M::NONE);
                assert!(matches!(&drive.app.ask, Some(Ask::PutFile { abs: 34, .. })));
            }
            assert_only_release(&drive.pad_release(source, 2), 18);
            assert!(drive.app.midi_ui_held.is_empty());
            assert!(!drive.app.pads.held[18]);
            assert!(if learn {
                drive.app.screen_learn.is_some()
            } else {
                drive.app.ask.is_some()
            });
        }
    }
}

#[test]
fn chop_linked_gate_disconnect_keeps_original_pad() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 32_000])), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_full_hit(&drive.pad_press(source, 2), 18);
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::F(4), M::NONE);
        drive.midi().recorded_disconnect(PAD_PORT);
        assert_only_release(&drive.frame(), 18);
        assert!(drive.app.midi_ui_held.is_empty());
        assert!(!drive.app.pads.held[18]);
    }
}

#[test]
fn chop_ui_holds_keep_port_channel_and_control_separate() {
    let other = "Recorded pads MIDI 2";
    let mut drive = Drive::new();
    drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 32_000])), 4);
    drive.link_chop_pad(18, 3);
    drive.link_chop_pad(19, 2);
    drive.midi().recorded_connect(other);
    drive.frame();
    drive.tap(KeyCode::PageDown, M::NONE);
    for (port, bytes, pad) in [
        (PAD_PORT, [0x90, 38, 127], 18),
        (PAD_PORT, [0x91, 38, 127], 18),
        (other, [0x90, 38, 127], 18),
        (PAD_PORT, [0x90, 39, 127], 19),
    ] {
        drive.bytes(port, &bytes);
        assert_full_hit(&drive.frame(), pad);
    }
    assert_eq!(drive.app.midi_ui_held.len(), 4);
    drive.tap(KeyCode::PageDown, M::NONE);
    drive.tap(KeyCode::F(4), M::NONE);
    drive.bytes(PAD_PORT, &[0x81, 38, 0]);
    assert_only_release(&drive.frame(), 18);
    assert_eq!(drive.app.midi_ui_held.len(), 3);
    drive.midi().recorded_disconnect(other);
    assert_only_release(&drive.frame(), 18);
    assert_eq!(drive.app.midi_ui_held.len(), 2);
    drive.bytes(PAD_PORT, &[0x90, 38, 0]); // note-on zero is the original note-off
    assert_only_release(&drive.frame(), 18);
    assert_eq!(drive.app.midi_ui_held.len(), 1);
    drive.bytes(PAD_PORT, &[0x80, 39, 0]);
    assert_only_release(&drive.frame(), 19);
    assert!(drive.app.midi_ui_held.is_empty());
    drive.bytes(PAD_PORT, &[0x80, 38, 0]);
    assert!(
        drive.frame().commands().is_empty(),
        "an untracked release does not sound"
    );
}

/// Scenario "Chop silent hits have no pad release".
#[test]
fn chop_silent_hits_have_no_pad_release() {
    for source in [Source::Notes, Source::Button] {
        for mode in ["preview", "plan", "question", "learn"] {
            let mut drive = Drive::new();
            let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
            drive.prepare_chop(sound.clone(), 4);
            drive.tap(KeyCode::F(2), M::NONE);
            drive.load_pad(18, sound.clone(), "held");
            drive.tap(KeyCode::PageDown, M::NONE);
            assert_full_hit(&drive.pad_press(Source::Keyboard, 2), 18);
            drive.prepare_chop(sound.clone(), 4);
            match mode {
                "plan" => {
                    drive.tap(KeyCode::Char('p'), M::NONE);
                }
                "question" => {
                    drive.load_pad(16, sound.clone(), "occupied");
                    drive.tap(KeyCode::Char('p'), M::NONE);
                    drive.pad_press(Source::Keyboard, 0);
                    drive.pad_release(Source::Keyboard, 0);
                    drive.tap(KeyCode::Enter, M::NONE);
                    assert!(matches!(&drive.app.ask, Some(Ask::Plan { .. })));
                }
                "learn" => {
                    drive.tap(KeyCode::F(10), M::NONE);
                }
                _ => {}
            }
            let hit = drive.pad_press(source, 2);
            assert!(
                !hit.commands().iter().any(|command| matches!(
                    command,
                    Command::PadPress { .. } | Command::PadRelease { .. }
                )),
                "{mode}: {:?}",
                hit.commands()
            );
            assert!(drive.app.midi_ui_held.is_empty());
            assert!(
                drive.pad_release(source, 2).commands().is_empty(),
                "{mode}, {source:?}"
            );
            assert!(
                drive.app.pads.held[18],
                "the older keyboard hold survives {mode}"
            );
            let release = drive.pad_release(Source::Keyboard, 2);
            assert!(
                !release.commands().is_empty(),
                "older keyboard hold: {mode}, {source:?}"
            );
            assert_only_release(&release, 18);
        }
    }
}

/// Scenario "Held pad release passes screen learn".
#[test]
fn held_pad_release_passes_screen_learn() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for linked in [false, true] {
            for change_context in [false, true] {
                let mut drive = Drive::new();
                let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
                drive.prepare_chop(sound.clone(), 4);
                drive.link_chop_pad(18, 3);
                if !linked {
                    drive.tap(KeyCode::F(2), M::NONE);
                }
                drive.tap(KeyCode::PageDown, M::NONE);
                assert_full_hit(&drive.pad_press(source, 2), 18);
                if change_context {
                    drive.tap(KeyCode::PageDown, M::NONE);
                    drive.tap(KeyCode::F(6), M::NONE);
                }
                drive.tap(KeyCode::F(10), M::NONE);
                assert!(drive.app.screen_learn.is_some());
                assert!(drive.tap(KeyCode::Char('q'), M::NONE).commands().is_empty());
                drive.bytes(PAD_PORT, &[0xb0, 25, 127]);
                assert!(drive.frame().commands().is_empty());
                let learn = drive.app.screen_learn.as_ref().unwrap();
                let (picked, pending) = (learn.picked.clone(), learn.pending.clone());
                assert_eq!(picked.as_ref().unwrap().0, "pad.5");
                assert!(pending.is_some());
                let bindings = drive.app.learned.clone();
                assert_only_release(&drive.pad_release(source, 2), 18);
                assert!(!drive.app.pads.held[18]);
                assert!(!drive.app.pads.held[34]);
                let learn = drive.app.screen_learn.as_ref().unwrap();
                assert_eq!(learn.picked, picked, "{source:?}, linked: {linked}");
                assert_eq!(learn.pending, pending);
                assert_eq!(drive.app.learned, bindings);
                assert!(drive.pad_release(source, 2).commands().is_empty());
                // The release also clears HeldPads, so the next real hit is not a repeat.
                drive.tap(KeyCode::F(10), M::NONE);
                drive.tap(KeyCode::F(2), M::NONE);
                assert_full_hit(
                    &drive.pad_press(source, 2),
                    if change_context { 34 } else { 18 },
                );
                drive.pad_release(source, 2);
            }
        }
    }
}

/// Scenario "New learn keys do not release an older pad".
#[test]
fn new_learn_keys_do_not_release_an_older_pad() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 32_000])), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_full_hit(&drive.pad_press(source, 2), 18);
        drive.tap(KeyCode::F(10), M::NONE);
        let bindings = drive.app.learned.clone();
        for local in [4, 2] {
            // A MIDI-held local pad key has never been pressed on the keyboard.
            // Do not press an already held keyboard key as if it were a new key.
            if matches!(source, Source::Keyboard) && local == 2 {
                continue;
            }
            assert!(
                drive
                    .pad_press(Source::Keyboard, local)
                    .commands()
                    .is_empty()
            );
            assert!(
                drive
                    .pad_release(Source::Keyboard, local)
                    .commands()
                    .is_empty()
            );
            assert!(drive.app.pads.held[18], "{source:?}, silent key {local}");
            assert_eq!(drive.app.audio_held[18], Some(0));
            assert_eq!(drive.app.learned, bindings);
        }
        assert_only_release(&drive.pad_release(source, 2), 18);
        assert!(!drive.app.pads.held[18]);
        assert!(drive.app.screen_learn.is_some());
        assert_eq!(drive.app.learned, bindings);
    }
}

#[test]
fn chop_blocked_button_does_not_steal_a_note_hold() {
    for learn in [false, true] {
        let mut drive = Drive::new();
        let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_full_hit(&drive.pad_press(Source::Notes, 2), 18);
        if learn {
            drive.tap(KeyCode::F(10), M::NONE);
        } else {
            drive.load_pad(16, sound, "occupied");
            drive.tap(KeyCode::Char('p'), M::NONE);
            drive.pad_press(Source::Keyboard, 0);
            drive.pad_release(Source::Keyboard, 0);
            drive.tap(KeyCode::Enter, M::NONE);
            assert!(drive.app.ask.is_some());
        }
        assert!(drive.pad_press(Source::Button, 2).commands().is_empty());
        assert!(drive.pad_release(Source::Button, 2).commands().is_empty());
        assert_eq!(drive.app.midi_ui_held.len(), 1);
        assert!(drive.app.pads.held[18]);
        assert_only_release(&drive.pad_release(Source::Notes, 2), 18);
        assert!(drive.app.midi_ui_held.is_empty());
    }
}

/// Scenario "Chop release keeps direct route and epoch" (route half).
#[test]
fn chop_release_keeps_direct_route_without_a_duplicate() {
    for source in [Source::Notes, Source::Button] {
        for same_pad in [false, true] {
            let mut drive = Drive::new();
            let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
            drive.load_pad(18, sound.clone(), "break");
            drive.tap(KeyCode::PageDown, M::NONE);
            let direct = drive.pad_press(source, 2);
            assert_full_hit(&direct, 18);
            assert!(direct.ui.is_empty());
            drive.prepare_chop(sound.clone(), 4);
            let target = if same_pad { 18 } else { 34 };
            drive.link_chop_pad(target, 3);
            if !same_pad {
                drive.tap(KeyCode::PageDown, M::NONE);
            }
            let ui = drive.pad_press(source, 2);
            assert_full_hit(&ui, target);
            assert!(ui.direct.is_empty());
            let release = drive.pad_release(source, 2);
            assert_eq!(release.direct.len(), 1);
            assert!(matches!(
                release.direct[0].command,
                ProjectMessage::Command(Command::PadRelease { pad: 18 })
            ));
            if same_pad {
                assert!(release.ui.is_empty());
            } else {
                assert!(matches!(release.ui[..], [Command::PadRelease { pad: 34 }]));
            }
            assert!(drive.app.midi_ui_held.is_empty());
        }
    }
}

/// Scenario "Chop release keeps direct route and epoch" (project half).
#[test]
fn chop_old_project_hit_and_release_cannot_touch_a_new_hold() {
    use std::sync::atomic::Ordering::{Acquire, Release};
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_full_hit(&drive.pad_press(source, 2), 18);
        let release: &[u8] = match source {
            Source::Notes => &[0x80, 38, 0],
            Source::Button => &[0xb0, 80, 0],
            Source::Keyboard => unreachable!(),
        };
        drive.bytes(PAD_PORT, release);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(3);
        }
        drive.bytes(
            PAD_PORT,
            match source {
                Source::Notes => &[0x90, 39, 127],
                Source::Button => &[0xb0, 80, 127],
                Source::Keyboard => unreachable!(),
            },
        );
        drive.midi().recorded_fence();
        let old: Vec<_> = drive.midi().events.try_iter().collect();
        assert!(
            old.iter()
                .any(|event| matches!(event.event, midi::Event::UiPadRelease { .. }))
        );
        assert!(
            old.iter()
                .any(|event| matches!(event.event, midi::Event::PadHit { .. }))
        );
        drive.app.apply_session(session::Session::default());
        assert!(drive.app.midi_ui_held.is_empty());
        assert_eq!(drive.project.epoch.load(Acquire), 1);
        let mut left = [0.0; 128];
        let mut right = [0.0; 128];
        for _ in 0..256 {
            drive.app.flush_audio();
            drive
                .audio_rt
                .as_mut()
                .unwrap()
                .process(&mut left, &mut right);
            if drive.project.ready.load(Acquire) == 1 && drive.app.pending_audio.is_empty() {
                break;
            }
        }
        assert_eq!(drive.project.ready.load(Acquire), 1);
        assert!(drive.app.pending_audio.is_empty());
        drive.app.pads.track_keys = true;
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(18, 3);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_full_hit(&drive.pad_press(source, 2), 18);
        assert_eq!(drive.app.midi_ui_held.len(), 1);
        for event in old {
            drive.app.midi_ui_event(event);
        }
        assert!(drive.frame().commands().is_empty());
        assert_eq!(drive.app.midi_ui_held.len(), 1);
        assert_eq!(drive.app.audio_held[18], Some(1));
        assert!(drive.app.pads.held[18]);
        assert_only_release(&drive.pad_release(source, 2), 18);
        assert!(drive.app.midi_ui_held.is_empty());
        drive.project.ready.store(0, Release);
        assert!(
            drive.pad_press(source, 2).commands().is_empty(),
            "not ready at input time"
        );
        drive.project.ready.store(1, Release);
        assert!(
            drive.pad_release(source, 2).commands().is_empty(),
            "rejected hit has no release"
        );
    }
}

/// Scenario "Chop rejected hits preserve accepted gate release".
#[test]
fn chop_rejected_hits_preserve_accepted_gate_release() {
    let mut drive = Drive::new();
    drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 32_000])), 4);
    drive.link_chop_pad(18, 3);
    drive.tap(KeyCode::PageDown, M::NONE);
    let (producer, mut consumer) = rtrb::RingBuffer::new(1);
    drive.app.audio.as_mut().unwrap().commands = producer;
    assert_full_hit(&drive.pad_press(Source::Notes, 2), 18);
    assert!(matches!(
        consumer.pop().unwrap().command,
        ProjectMessage::Command(Command::PadPress { pad: 18, .. })
    ));
    let original = drive.app.midi_ui_held.clone();
    drive
        .app
        .audio
        .as_mut()
        .unwrap()
        .commands
        .push(ProjectCommand {
            epoch: 0,
            ready_at_send: true,
            command: ProjectMessage::Command(Command::SetPadGain { pad: 0, gain: 1.0 }),
        })
        .unwrap();
    drive.pad_press(Source::Notes, 2); // same hold, rejected retrigger
    assert_eq!(
        drive.app.audio_held[18],
        Some(0),
        "the accepted hold must survive a rejected retrigger"
    );
    drive.pad_press(Source::Button, 2); // another control, also rejected
    assert_eq!(
        drive.app.midi_ui_held, original,
        "a rejected control does not own the older voice"
    );
    assert!(drive.pad_release(Source::Button, 2).commands().is_empty());
    assert_only_release(&drive.pad_release(Source::Notes, 2), 18);
    assert_eq!(drive.app.pending_audio.len(), 1);
    assert!(drive.app.midi_ui_held.is_empty());
    assert!(matches!(
        consumer.pop().unwrap().command,
        ProjectMessage::Command(Command::SetPadGain { .. })
    ));
    drive.app.flush_audio();
    assert!(matches!(
        consumer.pop().unwrap().command,
        ProjectMessage::Command(Command::PadRelease { pad: 18 })
    ));
    assert!(consumer.pop().is_err());
    assert!(drive.app.pending_audio.is_empty());
}

#[test]
fn chop_linked_gate_stops_in_offline_audio() {
    use engine::checks::{first_non_finite, peak};
    for (source, learn) in [Source::Keyboard, Source::Notes, Source::Button]
        .into_iter()
        .flat_map(|source| [(source, false), (source, true)])
    {
        let mut drive = Drive::new();
        let sound = Arc::new(Sample::mono(vec![0.25; 96_000]));
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(18, 3);
        let region = drive.app.pads.region[18];
        for command in [
            Command::SetPadSound {
                pad: 18,
                sound: Some(sound),
            },
            Command::SetPadRegion { pad: 18, region },
            Command::SetPadMode {
                pad: 18,
                mode: engine::PadMode::Gate,
            },
            Command::SetPadEnvelope {
                pad: 18,
                attack_ms: 0,
                decay_ms: 0,
                release_ms: 20,
            },
        ] {
            drive.app.send(command);
        }
        drive.frame();
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_full_hit(&drive.pad_press(source, 2), 18);
        let mut left = [0.0; 128];
        let mut right = [0.0; 128];
        let mut rendered = Vec::new();
        for _ in 0..8 {
            drive
                .audio_rt
                .as_mut()
                .unwrap()
                .process(&mut left, &mut right);
            rendered.extend_from_slice(&left);
            rendered.extend_from_slice(&right);
        }
        assert!(
            peak(&left) > 0.01,
            "{source:?}: accepted gate sounds before release"
        );
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::F(if learn { 6 } else { 4 }), M::NONE);
        if learn {
            drive.tap(KeyCode::F(10), M::NONE);
            assert!(drive.app.screen_learn.is_some());
        }
        assert_only_release(&drive.pad_release(source, 2), 18);
        for _ in 0..64 {
            drive
                .audio_rt
                .as_mut()
                .unwrap()
                .process(&mut left, &mut right);
            rendered.extend_from_slice(&left);
            rendered.extend_from_slice(&right);
        }
        assert_eq!(first_non_finite(&rendered), None);
        assert!(peak(&rendered) < 1.0);
        assert_eq!(
            peak(&left),
            0.0,
            "{source:?}: gate must stop, not play to the sample end"
        );
        assert_eq!(peak(&right), 0.0);
    }
}

/// Scenario "Copy parity from raw MIDI" and 83a's controller target contract.
#[test]
fn copy_parity_from_raw_midi() {
    let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
    let mut expected = None;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.load_pad(0, sound.clone(), "source");
        drive.tap(KeyCode::Char('c'), M::CONTROL);
        drive.tap(KeyCode::PageDown, M::NONE);
        let hit = drive.pad_press(source, 2);
        assert_silent_target(&hit);
        assert_eq!(drive.app.pads.selected_abs(), 18, "{source:?}");
        assert_eq!(drive.app.pads.custom_name[18].as_deref(), Some("source 2"));
        assert_eq!(drive.app.pads.custom_name[0].as_deref(), Some("source"));
        assert!(drive.app.copy_in_hand.is_none());
        assert_eq!(drive.app.history.len(), 1, "the target has one undo step");
        let seen = drive.seen();
        if let Some(expected) = &expected {
            assert_eq!(&seen, expected, "{source:?}");
        } else {
            expected = Some(seen);
        }
        drive.pad_release(source, 2);
        drive.tap(KeyCode::Char('z'), M::CONTROL);
        assert!(drive.app.pads.sound[18].is_none());
        assert!(drive.app.pads.sound[0].is_some());
    }
}

/// Scenario "Copy replacement parity from raw MIDI".
#[test]
fn copy_replacement_parity_from_raw_midi() {
    let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
    let mut expected = None;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.load_pad(0, sound.clone(), "source");
        drive.load_pad(18, sound.clone(), "old target");
        let old = drive.app.pads.snapshot(18);
        drive.tap(KeyCode::Char('c'), M::CONTROL);
        drive.tap(KeyCode::PageDown, M::NONE);
        assert_silent_target(&drive.pad_press(source, 2));
        assert_eq!(drive.app.pads.selected_abs(), 18, "{source:?}");
        assert_eq!(drive.app.pads.snapshot(18), old);
        assert!(
            drive
                .app
                .ask
                .as_ref()
                .unwrap()
                .lines()
                .iter()
                .any(|line| line.contains("old target"))
        );
        let question = drive.seen();
        drive.pad_release(source, 2);
        assert_silent_target(&drive.pad_press(source, 3));
        assert_eq!(
            drive.seen(),
            question,
            "a further hit must not answer or change the target"
        );
        drive.pad_release(source, 3);
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.ask.is_none());
        assert!(drive.app.copy_in_hand.is_some());
        assert_silent_target(&drive.pad_press(source, 2));
        drive.pad_release(source, 2);
        drive.tap(KeyCode::Enter, M::NONE);
        assert_eq!(drive.app.pads.custom_name[18].as_deref(), Some("source 2"));
        assert!(drive.app.copy_in_hand.is_none());
        assert!(drive.app.ask.is_none());
        assert_eq!(drive.app.history.len(), 1);
        let seen = drive.seen();
        if let Some(expected) = &expected {
            assert_eq!(&seen, expected, "{source:?}");
        } else {
            expected = Some(seen);
        }
    }
}

const RENDER_RATE: u32 = 48_000;
const RENDER_BLOCK: usize = 128;
const PARITY_HIT: usize = 32 * RENDER_BLOCK;
const PARITY_RELEASE: usize = PARITY_HIT + 128 * RENDER_BLOCK;
const PARITY_END: usize = PARITY_RELEASE + 128 * RENDER_BLOCK;
type Stereo = (Vec<f32>, Vec<f32>);

impl Drive {
    fn render_blocks(&mut self, blocks: usize) -> Stereo {
        let mut out = (Vec::new(), Vec::new());
        let (mut left, mut right) = ([0.0; RENDER_BLOCK], [0.0; RENDER_BLOCK]);
        for _ in 0..blocks {
            self.audio_rt
                .as_mut()
                .unwrap()
                .process(&mut left, &mut right);
            out.0.extend_from_slice(&left);
            out.1.extend_from_slice(&right);
            self.app.audio.as_mut().unwrap().free_garbage();
        }
        out
    }

    fn parity_pad(&mut self, pitch: i8, filtered: bool) -> Vec<Command> {
        use std::f64::consts::TAU;
        let sound = Arc::new(Sample::mono(
            (0..4 * RENDER_RATE as usize)
                .map(|i| {
                    let t = i as f64 / f64::from(RENDER_RATE);
                    // Each one-second slice starts at a zero crossing, also in
                    // the filter-off control. No hard sample edge hides routing checks.
                    (0.2 * (TAU * 220.0 * t).sin() + 0.08 * (TAU * 3_600.0 * t).sin()) as f32
                })
                .collect(),
        ));
        self.prepare_chop(sound, 4);
        self.link_chop_pad(18, 3);
        self.app.pads.pitch[18] = pitch;
        self.app.pads.volume_db[18] = -3;
        self.app.pads.attack_ms[18] = 0;
        self.app.pads.decay_ms[18] = 0;
        self.app.pads.release_ms[18] = 20;
        if !filtered {
            self.app.pads.filter[18] = engine::filter::FilterKind::Off;
        }
        let pad = self.app.pads.snapshot(18);
        let commands = self.app.pads.put_back(18, &pad);
        for command in &commands {
            self.app.send(command.clone());
        }
        self.tap(KeyCode::Char('2'), M::CONTROL);
        commands
    }

    fn velocity_hit(&mut self, source: Source, velocity: u8) -> Frame {
        match source {
            Source::Keyboard => {
                assert!(matches!(velocity, 80 | 127));
                self.key(
                    KeyCode::Char('3'),
                    if velocity == 127 { M::SHIFT } else { M::NONE },
                    KeyEventKind::Press,
                );
            }
            Source::Notes => self.bytes(PAD_PORT, &[0x90, 38, velocity]),
            Source::Button => {
                self.bind_pad_button(2);
                self.bytes(PAD_PORT, &[0xb0, 80, velocity]);
            }
        }
        self.frame()
    }
}

fn append_stereo(to: &mut Stereo, from: Stereo) {
    to.0.extend(from.0);
    to.1.extend(from.1);
}

fn check_parity_wave(wave: &Stereo) {
    use engine::checks::{first_loud, first_non_finite, max_step, peak};
    for channel in [&wave.0, &wave.1] {
        assert_eq!(first_non_finite(channel), None);
        assert!(peak(channel) < 1.0, "clipping");
        assert_eq!(peak(&channel[..PARITY_HIT]), 0.0);
        let onset = first_loud(channel, PARITY_HIT..PARITY_HIT + RENDER_BLOCK, 1e-6).unwrap();
        assert!(
            onset.abs_diff(PARITY_HIT) <= 1,
            "nearest-block onset: {onset}, head {:?}",
            &channel[PARITY_HIT..PARITY_HIT + 5]
        );
        assert!(peak(&channel[PARITY_RELEASE - RENDER_BLOCK..PARITY_RELEASE]) > 0.01);
        assert_eq!(
            peak(&channel[PARITY_RELEASE + 2_048..]),
            0.0,
            "gate tail ends"
        );
        assert!(max_step(channel) < 0.08, "unexpected sample jump");
    }
}

#[derive(Clone, Copy, Debug)]
enum RecordedRelease {
    Off,
    Zero,
    Disconnect,
}

/// Raw input -> production dispatch/App -> real queues -> callback. The separate
/// timestamped engine render is only an oracle; it does not feed this route.
fn render_chop_route(
    source: Source,
    velocity: u8,
    pitch: i8,
    filtered: bool,
    release: RecordedRelease,
) -> Stereo {
    let mut drive = Drive::offline();
    let settings = drive.parity_pad(pitch, filtered);
    let mut wave = drive.render_blocks(PARITY_HIT / RENDER_BLOCK);
    let hit = drive.velocity_hit(source, velocity);
    assert!(
        matches!(hit.ui[..], [Command::PadPress { pad: 18, velocity: v }]
        if v == f32::from(velocity) / 127.0)
    );
    assert_eq!(drive.app.chop.as_ref().unwrap().selected, 3);
    append_stereo(
        &mut wave,
        drive.render_blocks((PARITY_RELEASE - PARITY_HIT) / RENDER_BLOCK),
    );
    drive.tap(KeyCode::PageDown, M::NONE);
    drive.tap(KeyCode::F(6), M::NONE);
    drive.tap(KeyCode::F(10), M::NONE);
    let frame = match release {
        RecordedRelease::Off => drive.pad_release(source, 2),
        RecordedRelease::Zero => {
            assert!(matches!(source, Source::Notes));
            drive.bytes(PAD_PORT, &[0x90, 38, 0]);
            drive.frame()
        }
        RecordedRelease::Disconnect => {
            assert!(!matches!(source, Source::Keyboard));
            drive.midi().recorded_disconnect(PAD_PORT);
            drive.frame()
        }
    };
    assert_only_release(&frame, 18);
    assert!(!drive.app.pads.held[18]);
    assert!(!drive.app.pads.held[34]);
    assert!(drive.app.screen_learn.is_some());
    append_stereo(
        &mut wave,
        drive.render_blocks((PARITY_END - PARITY_RELEASE) / RENDER_BLOCK),
    );
    check_parity_wave(&wave);

    let mut engine = engine::Engine::new(RENDER_RATE);
    for setting in settings {
        engine.handle(0, setting);
    }
    let expected = engine::offline::render(
        &mut engine,
        &engine::offline::Scenario {
            sample_rate: RENDER_RATE,
            frames: PARITY_END,
            events: vec![
                engine::offline::Event {
                    frame: PARITY_HIT,
                    cmd: Command::PadPress {
                        pad: 18,
                        velocity: f32::from(velocity) / 127.0,
                    },
                },
                engine::offline::Event {
                    frame: PARITY_RELEASE,
                    cmd: Command::PadRelease { pad: 18 },
                },
            ],
        },
        RENDER_BLOCK,
    );
    assert_eq!(
        wave, expected,
        "one actual pad hit with all settings, {source:?}, {release:?}"
    );
    wave
}

/// Scenarios "Chop parity with linked pads", "Chop MIDI linked pad keeps velocity",
/// "Chop linked gate release keeps original pad", "Held pad release passes screen learn".
#[test]
fn render_84_input_parity() {
    use engine::checks::peak;
    let mut all = (Vec::new(), Vec::new());
    let keys = render_chop_route(Source::Keyboard, 127, 12, true, RecordedRelease::Off);
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let wave = render_chop_route(source, 127, 12, true, RecordedRelease::Off);
        assert_eq!(wave, keys, "matched full velocity: {source:?}");
        append_stereo(&mut all, wave);
    }
    let plain = render_chop_route(Source::Keyboard, 80, 12, true, RecordedRelease::Off);
    for source in [Source::Notes, Source::Button] {
        assert_eq!(
            render_chop_route(source, 80, 12, true, RecordedRelease::Off),
            plain
        );
        let low = render_chop_route(source, 23, 12, true, RecordedRelease::Off);
        let high = render_chop_route(source, 101, 12, true, RecordedRelease::Off);
        let ratio = peak(&low.0) / peak(&high.0);
        assert!(
            (ratio - 23.0 / 101.0).abs() < 1e-5,
            "velocity ratio {ratio}"
        );
        println!(
            "84 {source:?}: low peak {:.6}, high peak {:.6}, ratio {ratio:.6}",
            peak(&low.0),
            peak(&high.0)
        );
        append_stereo(&mut all, low);
        append_stereo(&mut all, high);
    }
    // Sensitivity controls: equality must not pass with ignored pitch or filter.
    let no_pitch = render_chop_route(Source::Notes, 127, 0, true, RecordedRelease::Zero);
    let no_filter = render_chop_route(Source::Notes, 127, 12, false, RecordedRelease::Off);
    let difference = |a: &[f32], b: &[f32]| {
        a.iter()
            .zip(b)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max)
    };
    assert!(difference(&keys.0, &no_pitch.0) > 0.05);
    assert!(difference(&keys.0, &no_filter.0) > 0.02);
    for source in [Source::Notes, Source::Button] {
        assert_eq!(
            render_chop_route(source, 127, 12, true, RecordedRelease::Disconnect),
            keys
        );
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/84-input-parity.wav");
    engine::offline::write_wav(&path, RENDER_RATE, &all.0, &all.1).unwrap();
    println!(
        "84 render: {} frames, peak {:.6}, matched routes sample-identical, onset {} (+/-1), finite/no-clip/tail-silence checked",
        all.0.len(),
        peak(&all.0).max(peak(&all.1)),
        PARITY_HIT
    );
}

/// Scenario "Hit from a controller": the real direct queue sounds before App sees
/// the event. Processing App later must not add another hit or release.
#[test]
fn offline_direct_midi_uses_the_next_block_without_ui_or_duplicate() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(12, true);
        drive.tap(KeyCode::F(2), M::NONE);
        let mut actual = drive.render_blocks(PARITY_HIT / RENDER_BLOCK);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.bytes(
            PAD_PORT,
            if matches!(source, Source::Notes) {
                &[0x90, 38, 127]
            } else {
                &[0xb0, 80, 127]
            },
        );
        drive.midi().recorded_fence();
        // No frame(): UI has not observed this hit.
        append_stereo(&mut actual, drive.render_blocks(1));
        assert!(engine::checks::peak(&actual.0[PARITY_HIT..]) > 0.01);
        assert!(
            drive.frame().ui.is_empty(),
            "UI must not repeat the direct hit"
        );
        append_stereo(
            &mut actual,
            drive.render_blocks((PARITY_RELEASE - PARITY_HIT) / RENDER_BLOCK - 1),
        );
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::F(6), M::NONE);
        drive.tap(KeyCode::F(10), M::NONE);
        drive.bytes(
            PAD_PORT,
            if matches!(source, Source::Notes) {
                &[0x90, 38, 0]
            } else {
                &[0xb0, 80, 0]
            },
        );
        drive.midi().recorded_fence();
        append_stereo(&mut actual, drive.render_blocks(1));
        assert!(
            drive.frame().ui.is_empty(),
            "UI must not repeat the direct release"
        );
        append_stereo(
            &mut actual,
            drive.render_blocks((PARITY_END - PARITY_RELEASE) / RENDER_BLOCK - 1),
        );
        let expected = render_chop_route(Source::Keyboard, 127, 12, true, RecordedRelease::Off);
        assert_eq!(
            actual, expected,
            "direct audio must match one UI hit, {source:?}"
        );
        check_parity_wave(&actual);
    }
}

/// Scenario "Chop rejected hits preserve accepted gate release": fill the actual
/// callback queue, not a replacement capture queue. The old voice must keep its phase.
#[test]
fn offline_full_queue_keeps_the_accepted_gate_and_delivers_pending_release() {
    use engine::checks::{first_non_finite, peak};
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(12, true);
        drive.render_blocks(32);
        drive.velocity_hit(source, 127);
        let first = drive.render_blocks(8);
        let original = drive.app.midi_ui_held.clone();
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
        assert_eq!(filled, 1024);
        drive.velocity_hit(source, 127);
        assert_eq!(drive.app.midi_ui_held, original);
        assert_eq!(drive.app.audio_held[18], Some(0));
        assert_only_release(&drive.pad_release(source, 2), 18);
        assert_eq!(drive.app.pending_audio.len(), 1);
        let after_full = drive.render_blocks(1);
        assert_eq!(drive.app.pending_audio.len(), 1);
        drive.app.flush_audio();
        assert!(drive.app.pending_audio.is_empty());
        let tail = drive.render_blocks(128);
        assert_eq!(first_non_finite(&tail.0), None);
        assert!(peak(&tail.0) < 1.0);
        assert_eq!(peak(&tail.0[4_096..]), 0.0, "pending release reaches audio");
        let mut reference = Drive::offline();
        reference.parity_pad(12, true);
        reference.render_blocks(32);
        reference.velocity_hit(Source::Keyboard, 127);
        assert_eq!(first, reference.render_blocks(8));
        assert_eq!(
            after_full,
            reference.render_blocks(1),
            "rejected hit must not retrigger"
        );
    }
    // A rejected first hit must not become a voice after queue space returns.
    let mut drive = Drive::offline();
    drive.parity_pad(12, true);
    drive.render_blocks(32);
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
    {}
    drive.velocity_hit(Source::Notes, 127);
    assert!(drive.app.midi_ui_held.is_empty());
    assert!(drive.pad_release(Source::Notes, 2).commands().is_empty());
    assert_eq!(peak(&drive.render_blocks(16).0), 0.0);
}

/// Scenario "Chop release keeps direct route and epoch": stale commands stay in
/// the real callback queues; stale UI events are delivered after the new gate starts.
#[test]
fn offline_project_switch_filters_both_routes_and_stale_releases() {
    use engine::checks::peak;
    use std::sync::atomic::Ordering::{Acquire, Release};
    for source in [Source::Notes, Source::Button] {
        for linked in [false, true] {
            let mut drive = Drive::offline();
            drive.parity_pad(12, true);
            if !linked {
                drive.tap(KeyCode::F(2), M::NONE);
            }
            drive.render_blocks(32);
            drive.velocity_hit(source, 127);
            assert!(peak(&drive.render_blocks(8).0) > 0.01);
            // Leave accepted old-epoch hit/release commands in either real queue.
            drive.pad_release(source, 2);
            drive.velocity_hit(source, 127);
            drive.bytes(
                PAD_PORT,
                if matches!(source, Source::Notes) {
                    &[0x80, 38, 0]
                } else {
                    &[0xb0, 80, 0]
                },
            );
            drive.bytes(
                PAD_PORT,
                if matches!(source, Source::Notes) {
                    &[0x90, 38, 127]
                } else {
                    &[0xb0, 80, 127]
                },
            );
            drive.midi().recorded_fence();
            let old: Vec<_> = drive.midi().events.try_iter().collect();
            assert!(
                old.iter()
                    .any(|event| matches!(event.event, midi::Event::PadHit { .. }))
            );
            assert!(old.iter().any(|event| matches!(
                event.event,
                midi::Event::UiPadRelease { .. } | midi::Event::PadRelease { .. }
            )));
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
            drive.parity_pad(12, true);
            assert_eq!(
                peak(&drive.render_blocks(32).0),
                0.0,
                "no queued old-project hit"
            );
            drive.velocity_hit(Source::Keyboard, 127);
            drive.render_blocks(8);
            for event in old {
                drive.app.midi_ui_event(event);
            }
            assert!(drive.frame().commands().is_empty());
            assert!(
                peak(&drive.render_blocks(32).0) > 0.01,
                "old release cannot stop new gate"
            );
            assert_eq!(drive.app.audio_held[18], Some(1));
            assert_only_release(&drive.pad_release(Source::Keyboard, 2), 18);
            assert_eq!(peak(&drive.render_blocks(64).0[4_096..]), 0.0);
            drive.project.ready.store(0, Release);
            drive.velocity_hit(source, 127);
            drive.project.ready.store(1, Release);
            drive.frame();
            assert!(drive.pad_release(source, 2).commands().is_empty());
            assert_eq!(
                peak(&drive.render_blocks(32).0),
                0.0,
                "not-ready hit cannot sound later"
            );
        }
    }
}

/// Scenarios "Chop parity before pad assignment", "Chop parity in by-hand assignment",
/// "Chop parity while a question is open" and "Chop parity while learning".
#[test]
fn offline_preview_assignment_and_guards_match_through_all_inputs() {
    use engine::checks::{first_non_finite, peak};
    let sound = Arc::new(Sample::mono(
        (0..96_000)
            .map(|i| (std::f32::consts::TAU * 220.0 * i as f32 / 48_000.0).sin() * 0.2)
            .collect(),
    ));
    for mode in ["preview", "plan", "question", "learn"] {
        let mut expected = None;
        for source in [Source::Keyboard, Source::Notes, Source::Button] {
            let mut drive = Drive::offline();
            drive.prepare_chop(sound.clone(), 4);
            drive.tap(KeyCode::PageDown, M::NONE);
            match mode {
                "plan" => {
                    drive.tap(KeyCode::Char('p'), M::NONE);
                }
                "question" => {
                    drive.load_pad(18, sound.clone(), "occupied");
                    drive.tap(KeyCode::Char('p'), M::NONE);
                    drive.pad_press(Source::Keyboard, 2);
                    drive.pad_release(Source::Keyboard, 2);
                    drive.tap(KeyCode::Enter, M::NONE);
                    assert!(drive.app.ask.is_some());
                }
                "learn" => {
                    drive.tap(KeyCode::F(10), M::NONE);
                }
                _ => {}
            }
            // Let the setup preview end before a guard is tested.
            assert_eq!(peak(&drive.render_blocks(512).0[60_000..]), 0.0);
            drive.pad_press(source, 2);
            let mut wave = drive.render_blocks(32);
            assert!(drive.app.audio_held.iter().all(Option::is_none));
            let release = drive.pad_release(source, 2);
            if matches!(source, Source::Keyboard) {
                // Pads::apply can return a keyboard release attempt even for a
                // preview. App::try_send rejects it without an accepted pad hold.
                assert!(
                    release
                        .commands()
                        .iter()
                        .all(|command| matches!(command, Command::PadRelease { .. }))
                );
            } else {
                assert!(release.commands().is_empty());
            }
            assert!(drive.app.pending_audio.is_empty());
            append_stereo(&mut wave, drive.render_blocks(224));
            for channel in [&wave.0, &wave.1] {
                assert_eq!(first_non_finite(channel), None);
                assert!(peak(channel) < 1.0);
                if matches!(mode, "question" | "learn") {
                    assert_eq!(peak(channel), 0.0, "guarded {source:?}, {mode}");
                } else {
                    assert!(peak(channel) > 0.01, "preview {source:?}, {mode}");
                    assert_eq!(peak(&channel[26_000..]), 0.0);
                }
            }
            if let Some(expected) = &expected {
                assert_eq!(&wave, expected, "{mode}, {source:?}");
            } else {
                expected = Some(wave);
            }
        }
    }
}

/// Scenario "Copy route preserves held MIDI releases".
#[test]
fn copy_route_preserves_held_midi_releases() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.load_pad(0, Arc::new(Sample::mono(vec![0.25; 1024])), "source");
        let press = drive.pad_press(source, 0);
        assert!(matches!(
            press.commands()[..],
            [Command::PadPress { pad: 0, .. }]
        ));
        drive.tap(KeyCode::Char('c'), M::CONTROL);
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::F(4), M::NONE);
        let release = drive.pad_release(source, 0);
        assert!(
            matches!(release.commands()[..], [Command::PadRelease { pad: 0 }]),
            "{source:?}: {:?}",
            release.commands()
        );
        assert!(!drive.app.pads.held[0]);
    }
}

fn pads_play_badge(drive: &mut Drive) -> bool {
    use ratatui::{Terminal, backend::TestBackend};
    let app = &mut drive.app;
    let view = pads_view(&mut app.pads, None, None, Vec::new(), None, app.rate);
    let screen = tui::Screen::Pads(app.mark_pads(view));
    let status = tui::Status {
        backend: None,
        recording: false,
        buffer_size: 128,
        sample_rate: 48_000,
        load_peak: 0.0,
        xruns: 0,
        master_db: 0,
        meters: ([-60.0; 2], [-60.0; 2], false),
        notice: None,
        no_releases: false,
        project: None,
    };
    let help = tui::Help {
        groups: &[],
        hint: None,
        full: None,
        question: None,
        learn: None,
    };
    let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
    term.draw(|frame| tui::draw(frame, &status, &screen, &help))
        .unwrap();
    let header = term
        .backend()
        .buffer()
        .content()
        .chunks(100)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .find(|row| row.contains("F2 PADS"))
        .unwrap();
    assert!(header.contains("PLAY"), "missing PLAY badge: {header}");
    header.contains("PLAY ▶")
}

/// Scenario "Pads PLAY badge follows keyboard and MIDI playback".
#[test]
fn pads_play_badge_follows_keyboard_and_midi_playback() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for release in [false, true] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            drive.app.pads.gate[18] = release;
            let snap = drive.app.pads.snapshot(18);
            for cmd in drive.app.pads.put_back(18, &snap) {
                drive.app.send(cmd);
            }
            drive.tap(KeyCode::F(2), M::NONE);
            drive.render_blocks(4);
            assert!(!pads_play_badge(&mut drive));
            drive.velocity_hit(source, 127);
            assert!(!pads_play_badge(&mut drive), "a queued hit is not playback");
            let hit = drive.render_blocks(1);
            assert!(pads_play_badge(&mut drive), "{source:?}, release {release}");
            assert!(engine::checks::peak(&hit.0) > 0.01);
            if release {
                drive.pad_release(source, 2);
                drive.render_blocks(1);
                assert!(pads_play_badge(&mut drive), "release fade still plays");
            }
            let end = drive.render_blocks(400);
            assert!(!pads_play_badge(&mut drive));
            if !release {
                assert!(drive.app.pads.held[18], "held does not mean sounding");
            }
            for channel in [&end.0, &end.1] {
                assert_eq!(engine::checks::first_non_finite(channel), None);
                assert!(engine::checks::peak(channel) < 1.0);
                assert_eq!(engine::checks::peak(&channel[49_000..]), 0.0);
            }
        }
    }
}

/// Scenario "Pads PLAY badge follows all sounding pads".
#[test]
fn pads_play_badge_follows_all_sounding_pads() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.app.pads.gate[18] = false;
        drive.load_pad(0, Arc::new(Sample::mono(vec![0.1; 96_000])), "second");
        for pad in [0, 18] {
            let snap = drive.app.pads.snapshot(pad);
            for cmd in drive.app.pads.put_back(pad, &snap) {
                drive.app.send(cmd);
            }
        }
        drive.tap(KeyCode::F(2), M::NONE);
        drive.render_blocks(4);
        drive.velocity_hit(source, 127);
        drive.render_blocks(1);
        drive.pad_release(source, 2);
        drive.tap(KeyCode::Char('1'), M::CONTROL);
        drive.pad_press(source, 0);
        drive.render_blocks(1);
        assert!(pads_play_badge(&mut drive));
        drive.pad_release(source, 0);
        drive.render_blocks(16);
        assert!(
            pads_play_badge(&mut drive),
            "the older pad in bank 2 still plays"
        );
        drive.tap(KeyCode::Char('3'), M::CONTROL);
        assert_eq!(drive.app.pads.selected_abs(), 32);
        assert!(
            pads_play_badge(&mut drive),
            "selected empty pad is not the sounding one"
        );
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.edit);
        assert!(pads_play_badge(&mut drive));
        drive.tap(KeyCode::Char('o'), M::CONTROL);
        assert!(drive.app.ui.hide_pads);
        assert!(pads_play_badge(&mut drive));
        drive.tap(KeyCode::F(4), M::NONE);
        drive.tap(KeyCode::F(2), M::NONE);
        assert!(
            pads_play_badge(&mut drive),
            "returning to F2 sees real pad activity"
        );
        drive.render_blocks(400);
        assert!(!pads_play_badge(&mut drive), "all pad voices ended");
    }
}

/// Scenario "Pads PLAY badge ignores silent requests and old projects".
#[test]
fn pads_play_badge_ignores_silent_requests_and_old_projects() {
    use std::sync::atomic::Ordering::{Acquire, Release};
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for guard in ["question", "learn", "queue", "not-ready", "no-audio"] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            drive.tap(KeyCode::F(2), M::NONE);
            drive.app.send(Command::SetPadSound {
                pad: 17,
                sound: None,
            });
            drive.render_blocks(4);
            match guard {
                "question" => {
                    drive.app.pads.marked.insert(18);
                    drive.app.ask_clear();
                    drive.frame();
                }
                "learn" => {
                    drive.tap(KeyCode::F(10), M::NONE);
                }
                "queue" if matches!(source, Source::Keyboard) => {
                    let audio = drive.app.audio.as_mut().unwrap();
                    while audio.commands.slots() > 0 {
                        audio
                            .commands
                            .push(ProjectCommand {
                                epoch: 0,
                                ready_at_send: true,
                                command: ProjectMessage::Command(Command::SetPadGain {
                                    pad: 0,
                                    gain: 1.0,
                                }),
                            })
                            .unwrap();
                    }
                }
                "queue" => {
                    // Fill the actual MIDI queue with hits of a pad with no sound.
                    for _ in 0..1024 {
                        drive.bytes(PAD_PORT, &[0x90, 37, 127]);
                    }
                    drive.frame();
                }
                "not-ready" => drive.project.ready.store(1, Release),
                "no-audio" => {
                    drive.app.audio.take();
                }
                _ => unreachable!(),
            }
            drive.velocity_hit(source, 127);
            assert!(
                !pads_play_badge(&mut drive),
                "{source:?}, {guard}, before callback"
            );
            if guard != "no-audio" {
                drive.project.ready.store(0, Release);
                let silent = drive.render_blocks(8);
                assert!(!pads_play_badge(&mut drive), "{source:?}, {guard}");
                assert_eq!(engine::checks::peak(&silent.0), 0.0);
            }
        }
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.render_blocks(4);
        drive.velocity_hit(source, 127);
        drive.render_blocks(1);
        assert!(pads_play_badge(&mut drive));
        drive.project.ready.store(1, Release);
        assert!(
            !pads_play_badge(&mut drive),
            "not ready hides an older snapshot"
        );
        drive.project.ready.store(0, Release);
        assert!(pads_play_badge(&mut drive));
        drive.app.apply_session(session::Session::default());
        assert!(
            !pads_play_badge(&mut drive),
            "old project is hidden before its fade ends"
        );
        for _ in 0..256 {
            drive.app.flush_audio();
            drive.render_blocks(1);
            assert!(!pads_play_badge(&mut drive));
            if drive.project.ready.load(Acquire) == 1 && drive.app.pending_audio.is_empty() {
                break;
            }
        }
        assert_eq!(drive.project.ready.load(Acquire), 1);
        // A new keyboard hit needs a release of the original physical press first.
        drive.pad_release(source, 2);
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.render_blocks(8);
        assert!(
            !drive.held.is_held(2),
            "fixture still holds the original keyboard pad"
        );
        drive.velocity_hit(source, 127);
        drive.render_blocks(1);
        assert!(
            pads_play_badge(&mut drive),
            "a valid new-project voice lights PLAY: {source:?}"
        );
        drive.app.audio.take();
        assert!(
            !pads_play_badge(&mut drive),
            "missing audio cannot reuse the snapshot"
        );
    }
}

/// Scenario "Pads PLAY badge ignores previews and effect tails".
#[test]
fn pads_play_badge_ignores_previews_and_effect_tails() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.app.send(Command::SetPadSends {
            pad: 18,
            sends: [0.0, 0.8],
        });
        drive.app.send(Command::SetFx {
            slot: 1,
            settings: engine::fx::FxSettings::Delay {
                time_ms: 20.0,
                feedback: 0.8,
            },
        });
        drive.render_blocks(4);
        for as_pad in [None, Some(18)] {
            let chop = drive.app.chop.as_ref().unwrap();
            let sound = chop.sound.clone();
            let region = chop.slices()[3];
            drive.app.send(Command::PreviewAs { pad: as_pad });
            drive.app.send(Command::Preview {
                sound: Some(sound),
                region,
            });
            let wave = drive.render_blocks(32);
            assert!(engine::checks::peak(&wave.0) > 0.01);
            assert!(
                !pads_play_badge(&mut drive),
                "preview, including borrowed pad settings, is not a pad hit"
            );
        }
        drive.app.send(Command::Preview {
            sound: None,
            region: Region::WHOLE,
        });
        drive.render_blocks(4);
        drive.velocity_hit(source, 127);
        drive.render_blocks(64);
        assert!(pads_play_badge(&mut drive));
        drive.pad_release(source, 2);
        let tail = drive.render_blocks(16);
        assert!(!pads_play_badge(&mut drive));
        assert!(
            engine::checks::peak(&tail.0[1024..]) > 1e-5,
            "effect tail still sounds"
        );
        for channel in [&tail.0, &tail.1] {
            assert_eq!(engine::checks::first_non_finite(channel), None);
            assert!(engine::checks::peak(channel) < 1.0);
        }
    }
}

fn displayed_pad_frame(drive: &Drive) -> Option<usize> {
    let chop = drive.app.chop.as_ref()?;
    let line = drive.app.chop_display(chop).playhead?;
    Some(chop.view.start + (line * (chop.view.end - chop.view.start) as f32).round() as usize)
}

fn check_pad_playback_line(source: Source) {
    for edit in [false, true] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.render_blocks(4);
        if edit {
            drive.tap(KeyCode::Enter, M::NONE);
            assert!(drive.app.edit);
        }
        assert_eq!(displayed_pad_frame(&drive), None);
        assert_full_hit(&drive.velocity_hit(source, 127), 18);
        let chop = drive.app.chop.as_ref().unwrap();
        let start = chop.slices()[3].start;
        let cursor = chop.cursor;
        assert_eq!(
            drive.app.chop_display(chop).mode,
            "LISTEN",
            "not lit before audio starts"
        );
        let mut wave = drive.render_blocks(1);
        assert_eq!(displayed_pad_frame(&drive), Some(start + RENDER_BLOCK));
        append_stereo(&mut wave, drive.render_blocks(3));
        assert_eq!(displayed_pad_frame(&drive), Some(start + 4 * RENDER_BLOCK));
        let chop = drive.app.chop.as_ref().unwrap();
        assert_eq!(
            chop.cursor, cursor,
            "progress must not drag the owner's cursor"
        );
        assert_eq!(
            drive.app.chop_display(chop).mode,
            "LISTEN ▶",
            "the owner-approved badge follows actual pad playback"
        );
        assert_eq!(drive.app.playhead(), None, "not a transport preview");
        assert!(
            drive.frame().commands().is_empty(),
            "drawing must not start another voice"
        );
        for samples in [&wave.0, &wave.1] {
            assert_eq!(engine::checks::first_non_finite(samples), None);
            assert!(engine::checks::peak(samples) < 1.0);
        }
    }
}

/// Scenario "Keyboard pad shows the Chop playback line".
#[test]
fn keyboard_pad_shows_the_chop_playback_line() {
    check_pad_playback_line(Source::Keyboard);
}

/// Scenario "MIDI pad shows the same Chop playback line".
#[test]
fn midi_pad_shows_the_same_chop_playback_line() {
    check_pad_playback_line(Source::Notes);
    check_pad_playback_line(Source::Button);
}

/// Scenario "Chop pad line follows pitch and retrigger".
#[test]
fn chop_pad_line_follows_pitch_and_retrigger() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for (pitch, step) in [(-12, 64), (0, 128), (12, 256)] {
            let mut drive = Drive::offline();
            drive.parity_pad(pitch, false);
            drive.render_blocks(4);
            drive.velocity_hit(source, 127);
            drive.render_blocks(3);
            assert_eq!(displayed_pad_frame(&drive), Some(144_000 + 3 * step));
            drive.pad_release(source, 2);
            assert_full_hit(&drive.velocity_hit(source, 127), 18);
            drive.render_blocks(1);
            assert_eq!(
                displayed_pad_frame(&drive),
                Some(144_000 + step),
                "{source:?}, pitch {pitch}"
            );
        }
    }
}

/// Scenario "Chop pad line ends with its voice".
#[test]
fn chop_pad_line_ends_with_its_voice() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for ending in ["end", "release", "hush"] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            drive.render_blocks(4);
            drive.velocity_hit(source, 127);
            drive.render_blocks(1);
            assert!(displayed_pad_frame(&drive).is_some());
            match ending {
                "release" => {
                    drive.pad_release(source, 2);
                    drive.render_blocks(1);
                    assert!(
                        displayed_pad_frame(&drive).is_some(),
                        "release still sounds"
                    );
                }
                "hush" => drive.app.send(Command::HushPads),
                _ => {}
            }
            let wave = drive.render_blocks(400);
            assert_eq!(displayed_pad_frame(&drive), None, "{source:?}, {ending}");
            assert_eq!(engine::checks::peak(&wave.0[49_000..]), 0.0);
        }
    }
}

/// Scenario "Chop pad line keeps transport and edits unchanged".
#[test]
fn chop_pad_line_keeps_transport_and_edits_unchanged() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.render_blocks(4);
        drive.velocity_hit(source, 127);
        drive.render_blocks(1);
        assert!(displayed_pad_frame(&drive).is_some());
        // Move within the same slice: the owner's cursor moves, no slice pick preview.
        drive.tap(KeyCode::Right, M::NONE);
        let cursor = drive.app.chop.as_ref().unwrap().cursor;
        drive.render_blocks(1);
        assert_eq!(drive.app.playhead(), None);
        let sounding = displayed_pad_frame(&drive).unwrap();
        assert_ne!(cursor, sounding);
        drive.tap(KeyCode::Char('m'), M::NONE);
        assert!(drive.app.chop.as_ref().unwrap().markers.contains(&cursor));
        assert!(!drive.app.chop.as_ref().unwrap().markers.contains(&sounding));
        assert_eq!(drive.app.chop.as_ref().unwrap().cursor, cursor);
        let play = drive.tap(KeyCode::Char(' '), M::NONE);
        assert!(
            play.ui
                .iter()
                .any(|cmd| matches!(cmd, Command::Preview { sound: Some(_), .. }))
        );
        assert!(
            !play
                .ui
                .iter()
                .any(|cmd| matches!(cmd, Command::PadPress { .. }))
        );
        // Existing cuts must be at least 10 ms apart; let the preview pass that gap.
        drive.render_blocks(8);
        assert_eq!(
            displayed_pad_frame(&drive),
            drive.app.playhead(),
            "Space line keeps priority"
        );
        let preview_at = drive.app.playhead().unwrap();
        drive.tap(KeyCode::Char('m'), M::NONE);
        assert!(
            drive
                .app
                .chop
                .as_ref()
                .unwrap()
                .markers
                .contains(&preview_at),
            "Space cut keeps its old rule"
        );
        let stop = drive.tap(KeyCode::Char(' '), M::NONE);
        assert!(
            stop.ui
                .iter()
                .any(|cmd| matches!(cmd, Command::Preview { sound: None, .. }))
        );
        drive.render_blocks(4);
        assert_eq!(drive.app.playhead(), None);
        assert!(
            displayed_pad_frame(&drive).is_some(),
            "stopping preview must not stop the pad"
        );
        drive.tap(KeyCode::Char('o'), M::ALT);
        drive.tap(KeyCode::Char(' '), M::NONE);
        drive.render_blocks(900);
        assert!(
            drive.app.playhead().is_some(),
            "existing Space loop still plays"
        );
        assert_eq!(displayed_pad_frame(&drive), drive.app.playhead());
    }
}

/// Scenario "Chop pad line rejects unrelated and silent hits".
#[test]
fn chop_pad_line_rejects_unrelated_and_silent_hits() {
    use std::sync::atomic::Ordering::{Acquire, Release};
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for guard in ["question", "learn", "queue"] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            drive.link_chop_pad(19, 2);
            let target = drive.app.pads.snapshot(19);
            for cmd in drive.app.pads.put_back(19, &target) {
                drive.app.send(cmd);
            }
            drive.render_blocks(4);
            drive.velocity_hit(Source::Keyboard, 127);
            drive.render_blocks(1);
            assert_eq!(displayed_pad_frame(&drive), Some(144_128));
            if guard == "question" {
                drive.app.pads.marked.insert(18);
                drive.app.ask_clear();
                drive.frame();
            } else if guard == "learn" {
                drive.tap(KeyCode::F(10), M::NONE);
            } else {
                let audio = drive.app.audio.as_mut().unwrap();
                while audio.commands.slots() > 0 {
                    audio
                        .commands
                        .push(ProjectCommand {
                            epoch: 0,
                            ready_at_send: true,
                            command: ProjectMessage::Command(Command::SetPadGain {
                                pad: 0,
                                gain: 1.0,
                            }),
                        })
                        .unwrap();
                }
            }
            drive.pad_press(source, 3);
            assert!(
                drive.app.audio_held[19].is_none(),
                "blocked hit has no audio hold"
            );
            drive.render_blocks(1);
            assert_eq!(
                displayed_pad_frame(&drive),
                Some(144_256),
                "older voice survives {guard}"
            );
        }
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        let sound = drive.app.chop.as_ref().unwrap().sound.clone();
        drive.load_pad(19, Arc::new(Sample::mono(vec![0.1; 96_000])), "other");
        let snap = drive.app.pads.snapshot(19);
        for cmd in drive.app.pads.put_back(19, &snap) {
            drive.app.send(cmd);
        }
        drive.render_blocks(4);
        drive.pad_press(source, 3);
        drive.render_blocks(1);
        assert_eq!(
            displayed_pad_frame(&drive),
            None,
            "another sample is not this waveform"
        );
        let chop = drive.app.chop.as_ref().unwrap();
        assert_eq!(drive.app.chop_display(chop).mode, "LISTEN");
        drive.velocity_hit(Source::Keyboard, 127);
        drive.render_blocks(1);
        assert!(displayed_pad_frame(&drive).is_some());
        drive.app.apply_session(session::Session::default());
        // Reuse the very same Sample allocation to test epoch, not just pointer filtering.
        drive.prepare_chop(sound, 4);
        drive.link_chop_pad(18, 3);
        assert_eq!(
            displayed_pad_frame(&drive),
            None,
            "old project position is hidden immediately"
        );
        let chop = drive.app.chop.as_ref().unwrap();
        assert_eq!(drive.app.chop_display(chop).mode, "LISTEN");
        for _ in 0..256 {
            drive.app.flush_audio();
            drive.render_blocks(1);
            assert_eq!(displayed_pad_frame(&drive), None);
            if drive.project.ready.load(Acquire) == 1 && drive.app.pending_audio.is_empty() {
                break;
            }
        }
        assert_eq!(drive.project.ready.load(Acquire), 1);
        drive.project.ready.store(0, Release);
        drive.velocity_hit(source, 127);
        drive.project.ready.store(1, Release);
        drive.render_blocks(4);
        assert_eq!(
            displayed_pad_frame(&drive),
            None,
            "not-ready hits cannot create a line"
        );
        let chop = drive.app.chop.as_ref().unwrap();
        assert_eq!(drive.app.chop_display(chop).mode, "LISTEN");
    }
}

/// Scenario "Chop listen badge stays lit beyond the waveform view".
#[test]
fn chop_pad_line_uses_the_visible_wave_and_current_chop() {
    let mut drive = Drive::offline();
    drive.parity_pad(0, false);
    drive.render_blocks(4);
    drive.velocity_hit(Source::Keyboard, 127);
    drive.render_blocks(4);
    for _ in 0..4 {
        drive.tap(KeyCode::Char('i'), M::NONE);
    }
    let chop = drive.app.chop.as_ref().unwrap();
    let view = drive.app.chop_display(chop);
    let expected = (144_512 - chop.view.start) as f32 / (chop.view.end - chop.view.start) as f32;
    assert_eq!(view.playhead, Some(expected));
    assert_eq!(displayed_pad_frame(&drive), Some(144_512));
    for _ in 0..4 {
        drive.tap(KeyCode::Char('i'), M::NONE);
    }
    assert_eq!(
        displayed_pad_frame(&drive),
        None,
        "do not clamp offscreen progress to an edge"
    );
    let chop = drive.app.chop.as_ref().unwrap();
    assert_eq!(
        drive.app.chop_display(chop).mode,
        "LISTEN ▶",
        "offscreen voice still plays"
    );
    for _ in 0..8 {
        drive.tap(KeyCode::Char('k'), M::NONE);
    }
    assert_eq!(displayed_pad_frame(&drive), Some(144_512));
    drive.app.pads.link[18].as_mut().unwrap().1 += 1;
    assert_eq!(
        displayed_pad_frame(&drive),
        None,
        "another chop of the same Sample is not this chop"
    );
    let chop = drive.app.chop.as_ref().unwrap();
    assert_eq!(drive.app.chop_display(chop).mode, "LISTEN");
}

/// Scenario "Chop listen badge lights only while playback runs".
#[test]
fn chop_listen_badge_lights_only_while_playback_runs() {
    use engine::checks::peak;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.app.send(Command::SetPadSends {
            pad: 18,
            sends: [0.0, 0.8],
        });
        drive.app.send(Command::SetFx {
            slot: 1,
            settings: engine::fx::FxSettings::Delay {
                time_ms: 20.0,
                feedback: 0.8,
            },
        });
        drive.render_blocks(4);
        let mode = |d: &Drive| d.app.chop_display(d.app.chop.as_ref().unwrap()).mode;
        assert_eq!(mode(&drive), "LISTEN");
        drive.velocity_hit(source, 127);
        assert_eq!(mode(&drive), "LISTEN", "queued input alone is not playback");
        drive.render_blocks(64);
        assert_eq!(mode(&drive), "LISTEN ▶");
        drive.pad_release(source, 2);
        let tail = drive.render_blocks(16);
        assert_eq!(mode(&drive), "LISTEN");
        assert!(
            peak(&tail.0[1_024..]) > 1e-5,
            "effect tail alone must not keep the badge lit"
        );
        drive.tap(KeyCode::Char(' '), M::NONE);
        drive.render_blocks(1);
        assert_eq!(mode(&drive), "LISTEN ▶");
        drive.tap(KeyCode::Char(' '), M::NONE);
        drive.render_blocks(4);
        assert_eq!(mode(&drive), "LISTEN");
    }
}

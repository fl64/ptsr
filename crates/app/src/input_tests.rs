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
        let project = Arc::new(ProjectState::default());
        let (producer, consumer) = rtrb::RingBuffer::new(1024);
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
            direct: Some(consumer),
            worker: Some(worker),
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
        let direct = std::iter::from_fn(|| self.direct.as_mut().unwrap().pop().ok()).collect();
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

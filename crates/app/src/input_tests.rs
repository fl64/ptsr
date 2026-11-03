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
            audio_rt: None,
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

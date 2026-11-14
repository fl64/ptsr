//! Recorded input through the production keyboard and MIDI routes. No device or sound card.

mod badges;
mod bank_scenarios;
mod controller_map;
mod copy;
mod feedback;
mod midi_state;
mod notices_help;
mod pattern_step;
mod pattern_view;
mod patterns;
mod registry;
mod saving_projects;
mod transport;

use super::*;
use engine::rt::{ProjectCommand, ProjectMessage, ProjectState};
use feedback::{input_device, shared_feedback_rows, shown_value, value_knob};
use notices_help::f1_message_screen;
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
        Self::with_saved_bindings(offline, None)
    }

    fn with_saved_bindings(offline: bool, path: Option<&Path>) -> Self {
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
        if let Some(path) = path {
            let loaded = config::load_from(path);
            assert_eq!(loaded.user_path.as_deref(), Some(path));
            app.user_path = loaded.user_path;
            app.learned = loaded.config.midi;
            assert!(learn::drop_dead(&mut app.learned).is_empty());
            app.seeded = loaded.config.devices.seeded;
            app.migrate_fx_bindings();
            let learned = midi::parse_learned(&app.learned);
            for profile in &mut app.profiles {
                midi::apply_learned(profile, &learned);
                profile.set_seeded(app.seeded.contains(&profile.name));
            }
        }
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
        drive.app.notices = Notices::new(Instant::now());
        drive
    }

    fn last_notice(&self) -> &Notice {
        self.app
            .notices
            .history()
            .front()
            .expect("a produced notice")
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
        self.app.input_sources.advance(Instant::now());
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
            notice: self
                .app
                .notices
                .history()
                .front()
                .map(|notice| notice.text.clone()),
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

    fn queued_events(&self) -> Vec<midi::UiEvent> {
        self.midi().recorded_fence();
        self.midi().events.try_iter().collect()
    }

    fn raw_ordered_hit(&self, source: Source, velocity: u8) {
        self.bytes(
            PAD_PORT,
            &match source {
                Source::Notes => [0x90, 38, velocity],
                Source::Button => [0xb0, 80, velocity],
                Source::Keyboard => panic!("raw MIDI scenario"),
            },
        );
    }

    fn raw_ordered_release(&self, source: Source, release: RecordedRelease) {
        if matches!(release, RecordedRelease::Disconnect) {
            self.midi().recorded_disconnect(PAD_PORT);
        } else {
            self.bytes(
                PAD_PORT,
                &match source {
                    Source::Notes if matches!(release, RecordedRelease::Off) => [0x80, 38, 0],
                    Source::Notes => [0x90, 38, 0],
                    Source::Button => [0xb0, 80, 0],
                    Source::Keyboard => panic!("raw MIDI scenario"),
                },
            );
        }
    }

    fn bind_action(&mut self, cc: u8, action: &str) {
        learn::assign(
            &mut self.app.learned,
            "Recorded pads",
            input::Group::Global,
            input::midi::Key::Cc(cc),
            action,
        );
        self.app.sync_learned("Recorded pads", input::Group::Global);
        self.midi().recorded_fence();
    }

    fn ordered_file(&mut self) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/85-ordered-files")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Relaxed)
            ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("new.wav");
        let tone = vec![0.1; 4096];
        engine::offline::write_wav(&path, RENDER_RATE, &tone, &tone).unwrap();
        self.tap(KeyCode::F(6), M::NONE);
        self.app.browser.dir = dir;
        self.app.browser.entries = vec![browser::Entry {
            name: "new.wav".into(),
            is_dir: false,
            favorite: None,
        }];
        self.app.browser.selected = 0;
        self.bind_action(82, "browser.hold");
        self.frame();
        path
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

const REGISTRY_SCREEN_COMMANDS: [(Screen, KeyCode, u8); 5] = [
    (Screen::Pads, KeyCode::F(2), 90),
    (Screen::Settings, KeyCode::F(12), 91),
    (Screen::Files, KeyCode::F(6), 92),
    (Screen::Chop, KeyCode::F(3), 93),
    (Screen::Fx, KeyCode::F(4), 94),
];

impl Drive {
    fn registry_screen_checks() -> Self {
        let mut drive = Self::new();
        for (number, action) in [
            (90, "screen.pads"),
            (91, "screen.settings"),
            (92, "screen.files"),
            (93, "screen.chop"),
            (94, "screen.fx"),
            (95, "help.toggle"),
            (96, "learn.toggle"),
            (97, "pad.edit"),
            (98, "chop.edit"),
            (99, "settings.tab_next"),
            (100, "browser.preview_toggle"),
            (101, "chop.loop_toggle"),
        ] {
            drive.bind_action(number, action);
        }
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 48_000])), 4);
        drive.link_chop_pad(0, 0);
        drive
    }

    fn save_registry_bindings(&self) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/86-registry-config")
            .join(format!(
                "{}-{}.toml",
                std::process::id(),
                NEXT.fetch_add(1, Relaxed)
            ));
        config::save_section(&path, "midi", &self.app.learned).unwrap();
        config::save_section(
            &path,
            "devices",
            &config::DevicesConfig {
                seeded: self.app.seeded.clone(),
            },
        )
        .unwrap();
        path
    }

    fn registry_command(&mut self, source: Source, code: KeyCode, mods: M, number: u8) -> Frame {
        match source {
            Source::Keyboard => return self.tap(code, mods),
            Source::Notes => {
                self.bytes(PAD_PORT, &[0x90, number, 127]);
                self.bytes(PAD_PORT, &[0x80, number, 0]);
            }
            Source::Button => {
                self.bytes(PAD_PORT, &[0xb0, number, 127]);
                self.bytes(PAD_PORT, &[0xb0, number, 0]);
            }
        }
        self.frame()
    }
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
            drive.last_notice().text,
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

#[test]
fn modes_85_text_and_project_picker_keep_source_boundaries() {
    for context in ["name", "filter", "project"] {
        for source in [Source::Keyboard, Source::Notes, Source::Button] {
            let mut drive = Drive::new();
            drive.load_pad(0, Arc::new(Sample::mono(vec![0.25; 1024])), "source");
            match context {
                "name" => {
                    drive.tap(KeyCode::Char('n'), M::CONTROL);
                    assert!(drive.app.pad_naming.is_some());
                }
                "filter" => {
                    drive.tap(KeyCode::F(6), M::NONE);
                    drive.tap(KeyCode::Char('/'), M::NONE);
                    assert!(drive.app.browser.editing);
                }
                "project" => {
                    drive.app.paths.projects = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../target/85-no-project-root");
                    drive.tap(KeyCode::Char('p'), M::CONTROL);
                    assert!(drive.app.project_picker.is_some());
                }
                _ => unreachable!(),
            }
            let hit = drive.pad_press(source, 4);
            if matches!(source, Source::Keyboard) {
                if context == "filter" {
                    assert!(matches!(
                        hit.commands()[..],
                        [Command::Preview {
                            sound: None,
                            region: Region::WHOLE,
                        }]
                    ));
                } else {
                    assert!(hit.commands().is_empty(), "{context}: typing is silent");
                }
                assert_eq!(drive.app.pads.selected, 0);
                match context {
                    "name" => assert_eq!(drive.app.pad_naming.as_ref().unwrap().1, "sourceq"),
                    "filter" => assert_eq!(drive.app.browser.filter, "q"),
                    "project" => assert_eq!(
                        drive.app.project_picker.as_ref().unwrap().naming.as_deref(),
                        Some("q")
                    ),
                    _ => unreachable!(),
                }
            } else {
                assert_full_hit(&hit, 4);
                assert!(hit.ui.is_empty(), "{context}: existing direct MIDI route");
                assert_eq!(drive.app.pads.selected, 4);
                assert!(drive.app.browser.filter.is_empty());
                if context == "name" {
                    assert_eq!(drive.app.pad_naming.as_ref().unwrap().1, "source");
                }
                if context == "project" {
                    assert!(drive.app.project_picker.as_ref().unwrap().naming.is_none());
                }
            }
            // Keyboard releases pass the text guards too. `sent` records this
            // attempt even without an accepted sound; keep that existing boundary.
            assert_only_release(&drive.pad_release(source, 4), 4);
        }
    }
}

#[test]
fn modes_85_folder_picker_consumes_actions_not_direct_midi_hits() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::new();
        drive.tap(KeyCode::F(12), M::NONE);
        drive.app.settings_tab = 3;
        // A fixed empty listing avoids reading the owner's folders. The production
        // picker handler must still consume resolved keyboard and controller actions.
        drive.app.path_picker = Some(PathPicker {
            which: 0,
            at: "target/85-picker".into(),
            entries: Vec::new(),
            selected: 0,
        });
        drive.frame();
        let hit = drive.pad_press(source, 4);
        if matches!(source, Source::Keyboard) {
            assert!(hit.commands().is_empty());
            assert_eq!(drive.app.pads.selected, 0);
        } else {
            assert_full_hit(&hit, 4);
            assert!(hit.ui.is_empty());
            assert_eq!(drive.app.pads.selected, 4);
        }
        assert!(drive.app.path_picker.is_some());
        let release = drive.pad_release(source, 4);
        if matches!(source, Source::Keyboard) {
            assert!(release.commands().is_empty());
        } else {
            assert_only_release(&release, 4);
        }
        drive.bytes(PAD_PORT, &[0xb0, 27, 127]);
        assert!(drive.frame().commands().is_empty());
        assert_eq!(drive.app.pads.bank, 0, "the picker consumes MIDI actions");
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.path_picker.is_none());
    }
}

#[test]
fn modes_85_f12_learning_does_not_acquire_f10_silence() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        for run in [false, true] {
            let mut drive = Drive::new();
            drive.tap(KeyCode::F(12), M::NONE);
            drive.app.settings_tab = 2;
            drive.app.midi_open = vec![input::Group::Pads, input::Group::Global];
            drive.app.midi_row = drive
                .app
                .midi_list()
                .iter()
                .position(|row| {
                    if run {
                        matches!(
                            row,
                            MidiListRow::Set {
                                name: "pads 1–16",
                                ..
                            }
                        )
                    } else {
                        matches!(row, MidiListRow::Action { info } if info.name == "pad.copy")
                    }
                })
                .unwrap();
            drive.tap(KeyCode::Enter, M::NONE);
            assert_eq!(drive.app.learning.is_some(), !run);
            assert_eq!(drive.app.midi_run.is_some(), run);
            assert!(drive.app.screen_learn.is_none());
            let hit = drive.pad_press(source, 4);
            assert_full_hit(&hit, 4);
            if matches!(source, Source::Keyboard) {
                assert!(hit.direct.is_empty());
                if !run {
                    assert!(drive.app.learning.as_ref().unwrap().pending.is_none());
                }
            } else {
                assert!(hit.ui.is_empty());
                if !run {
                    assert!(drive.app.learning.as_ref().unwrap().pending.is_some());
                }
            }
            assert_only_release(&drive.pad_release(source, 4), 4);
        }
    }
}

#[test]
fn modes_85_question_keeps_keyboard_and_midi_edit_resolution() {
    for keyboard in [true, false] {
        let mut drive = Drive::new();
        drive.load_pad(0, Arc::new(Sample::mono(vec![0.25; 1024])), "source");
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.edit);
        for (group, action) in [
            (input::Group::Pads, "pad.edit"),
            (input::Group::PadsEdit, "ui.back"),
        ] {
            learn::assign(
                &mut drive.app.learned,
                "Recorded pads",
                group,
                input::midi::Key::Cc(81),
                action,
            );
            drive.app.sync_learned("Recorded pads", group);
        }
        drive.app.pads.marked.insert(0);
        drive.app.ask_clear();
        drive.frame();
        assert!(!drive.app.edit_keys(), "a question disables keyboard edit");
        assert!(drive.app.ask.is_some());
        if keyboard {
            drive.tap(KeyCode::Enter, M::NONE);
            assert!(drive.app.pads.file[0].is_none(), "screen Enter confirms");
        } else {
            drive.bytes(PAD_PORT, &[0xb0, 81, 127]);
            drive.frame();
            assert!(drive.app.input_log.last().unwrap().ends_with("-> Back"));
            assert!(
                drive.app.pads.file[0].is_some(),
                "MIDI retains its edit layer"
            );
        }
        assert!(drive.app.ask.is_none());
        assert!(drive.app.edit);
    }
}

#[test]
fn modes_85_chop_precedes_hand_targets_and_keeps_edit() {
    let sound = Arc::new(Sample::mono(vec![0.25; 32_000]));
    paired_baseline(|drive, source| {
        drive.prepare_chop(sound.clone(), 4);
        drive.link_chop_pad(4, 2);
        drive.app.browser.in_hand = Some("new file.wav".into());
        drive.app.pad_in_hand = Some(0);
        drive.app.copy_in_hand = Some((0, drive.app.pads.snapshot(4)));
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.edit);
        let before = drive.seen().pads;
        let hit = drive.pad_press(source, 4);
        assert_full_hit(&hit, 4);
        assert!(hit.direct.is_empty());
        assert_eq!(drive.app.chop.as_ref().unwrap().selected, 2);
        assert_eq!(drive.seen().pads, before);
        assert!(drive.app.ask.is_none());
        assert!(drive.app.browser.in_hand.is_some());
        assert!(drive.app.copy_in_hand.is_some());
        assert_eq!(drive.app.pad_in_hand, Some(0));
        assert!(drive.app.edit);
        assert_only_release(&drive.pad_release(source, 4), 4);
        vec![drive.seen()]
    });
}

#[test]
fn modes_85_overlapping_hands_keep_source_specific_priority() {
    for with_file in [false, true] {
        for source in [Source::Keyboard, Source::Notes, Source::Button] {
            let mut drive = Drive::new();
            let sound = Arc::new(Sample::mono(vec![0.25; 1024]));
            drive.load_pad(0, sound.clone(), "source");
            drive.load_pad(4, sound, "target");
            drive.tap(KeyCode::Char('c'), M::CONTROL);
            drive.tap(KeyCode::Char('x'), M::CONTROL);
            assert!(drive.app.copy_in_hand.is_some());
            assert_eq!(drive.app.pad_in_hand, Some(0));
            if with_file {
                drive.tap(KeyCode::F(6), M::NONE);
                drive.app.browser.dir = "target/85-files".into();
                drive.app.browser.entries = vec![browser::Entry {
                    name: "new file.wav".into(),
                    is_dir: false,
                    favorite: None,
                }];
                drive.app.browser.selected = 0;
                drive.tap(KeyCode::Enter, M::NONE);
                drive.tap(KeyCode::F(2), M::NONE);
                assert!(drive.app.browser.in_hand.is_some());
            }
            let before = drive.seen().pads;
            assert_silent_target(&drive.pad_press(source, 4));
            if with_file && !matches!(source, Source::Keyboard) {
                assert!(matches!(drive.app.ask, Some(Ask::PutFile { abs: 4, .. })));
            } else {
                assert!(matches!(
                    drive.app.ask,
                    Some(Ask::CopyPad { source: 0, .. })
                ));
            }
            assert_eq!(drive.seen().pads, before);
            assert_eq!(drive.app.pads.selected_abs(), 4);
            assert!(drive.app.history.is_empty());
            assert!(drive.app.copy_in_hand.is_some());
            assert_eq!(drive.app.pad_in_hand, Some(0));
            let release = drive.pad_release(source, 4);
            if matches!(source, Source::Keyboard) {
                // A keyboard target keeps its existing release attempt. It did
                // not start an accepted audio hold and adds no new pad hit.
                assert_only_release(&release, 4);
            } else {
                assert!(release.commands().is_empty());
            }
        }
    }
}

#[test]
fn modes_85_marking_and_edit_still_coexist_and_sound() {
    paired_baseline(|drive, source| {
        drive.tap(KeyCode::Char('m'), M::NONE);
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.pads.marking);
        assert!(drive.app.edit);
        assert_full_hit(&drive.pad_press(source, 4), 4);
        assert!(drive.app.pads.marked.contains(&4));
        assert!(drive.app.edit);
        assert_only_release(&drive.pad_release(source, 4), 4);
        vec![drive.seen()]
    });
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

#[derive(Clone, Copy, Debug)]
enum MixedHoldGuard {
    Chop,
    Fx,
    Question,
    Learn,
}

fn render_mixed_chop_holds(
    source: Source,
    midi_first: bool,
    keyboard_release_first: bool,
    guard: MixedHoldGuard,
    reject_midi: bool,
) -> Stereo {
    use engine::checks::{first_loud, first_non_finite, peak};
    assert!(!matches!(source, Source::Keyboard));
    assert!(!reject_midi || !midi_first);
    let mut drive = Drive::offline();
    let mut settings = drive.parity_pad(0, true);
    let snap = drive.app.pads.snapshot(18);
    let other = drive.app.pads.put_back(34, &snap);
    for command in &other {
        drive.app.send(command.clone());
    }
    settings.extend(other);
    drive.frame();
    let mut wave = drive.render_blocks(32);
    let first_source = if midi_first { source } else { Source::Keyboard };
    let first = drive.pad_press(first_source, 2);
    assert!(first.direct.is_empty());
    assert!(matches!(
        first.ui[..],
        [Command::PadPress {
            pad: 18,
            velocity: 1.0
        }]
    ));
    append_stereo(&mut wave, drive.render_blocks(32));
    drive.tap(KeyCode::PageDown, M::NONE);
    assert_eq!(drive.app.pads.bank, 2);
    if reject_midi {
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
        assert_eq!(filled, 1024, "reject on the actual callback queue");
    }
    let second_source = if midi_first { Source::Keyboard } else { source };
    let second = drive.pad_press(second_source, 2);
    assert!(second.direct.is_empty());
    assert!(matches!(
        second.ui[..],
        [Command::PadPress {
            pad: 34,
            velocity: 1.0
        }]
    ));
    assert_eq!(drive.app.audio_held[18], Some(0));
    assert_eq!(drive.app.audio_held[34], (!reject_midi).then_some(0));
    if reject_midi {
        assert!(drive.app.midi_ui_held.is_empty());
    } else {
        assert_eq!(drive.app.midi_ui_held.len(), 1);
    }
    append_stereo(&mut wave, drive.render_blocks(32));
    match guard {
        MixedHoldGuard::Chop => {}
        MixedHoldGuard::Fx => {
            drive.tap(KeyCode::F(4), M::NONE);
        }
        MixedHoldGuard::Question => {
            drive.app.ask = Some(Ask::Plan { lines: Vec::new() });
            drive.frame();
        }
        MixedHoldGuard::Learn => {
            drive.tap(KeyCode::F(10), M::NONE);
        }
    }
    let (keyboard_pad, midi_pad) = if midi_first { (34, 18) } else { (18, 34) };
    for release_keyboard in [keyboard_release_first, !keyboard_release_first] {
        let release_source = if release_keyboard {
            Source::Keyboard
        } else {
            source
        };
        let release = drive.pad_release(release_source, 2);
        if reject_midi && !release_keyboard {
            assert!(
                release.commands().is_empty(),
                "a rejected hit has no audio release"
            );
        } else {
            let original = if release_keyboard {
                keyboard_pad
            } else {
                midi_pad
            };
            assert_only_release(&release, original);
            assert_eq!(drive.app.audio_held[original], None);
            assert!(!drive.app.pads.held[original]);
        }
        append_stereo(&mut wave, drive.render_blocks(64));
    }
    assert!(drive.app.midi_ui_held.is_empty());
    assert_eq!(drive.app.audio_held[18], None);
    assert_eq!(drive.app.audio_held[34], None);
    assert_eq!(
        drive.app.ask.is_some(),
        matches!(guard, MixedHoldGuard::Question)
    );
    assert_eq!(
        drive.app.screen_learn.is_some(),
        matches!(guard, MixedHoldGuard::Learn)
    );

    let hit_at = 32 * RENDER_BLOCK;
    let second_at = 64 * RENDER_BLOCK;
    let release_at = 96 * RENDER_BLOCK;
    let last_release_at = 160 * RENDER_BLOCK;
    let mut expected_events = vec![engine::offline::Event {
        frame: hit_at,
        cmd: Command::PadPress {
            pad: 18,
            velocity: 1.0,
        },
    }];
    if !reject_midi {
        expected_events.push(engine::offline::Event {
            frame: second_at,
            cmd: Command::PadPress {
                pad: 34,
                velocity: 1.0,
            },
        });
    }
    for (frame, release_keyboard) in [
        (release_at, keyboard_release_first),
        (last_release_at, !keyboard_release_first),
    ] {
        if !reject_midi || release_keyboard {
            expected_events.push(engine::offline::Event {
                frame,
                cmd: Command::PadRelease {
                    pad: if release_keyboard {
                        keyboard_pad
                    } else {
                        midi_pad
                    },
                },
            });
        }
    }
    let mut expected_engine = engine::Engine::new(RENDER_RATE);
    for command in settings {
        expected_engine.handle(0, command);
    }
    let expected = engine::offline::render(
        &mut expected_engine,
        &engine::offline::Scenario {
            sample_rate: RENDER_RATE,
            frames: wave.0.len(),
            events: expected_events,
        },
        RENDER_BLOCK,
    );
    assert_eq!(
        wave, expected,
        "{source:?}, {guard:?}, MIDI first {midi_first}, keyboard release first {keyboard_release_first}, rejected {reject_midi}"
    );
    for channel in [&wave.0, &wave.1] {
        assert_eq!(first_non_finite(channel), None);
        assert!(peak(channel) < 1.0);
        assert_eq!(peak(&channel[..hit_at]), 0.0);
        assert!(
            first_loud(channel, hit_at..hit_at + RENDER_BLOCK, 1e-6)
                .unwrap()
                .abs_diff(hit_at)
                <= 1
        );
        let sounding = peak(&channel[release_at + 2048..last_release_at]);
        if reject_midi && keyboard_release_first {
            assert_eq!(
                sounding, 0.0,
                "rejected MIDI must not start when space returns"
            );
        } else {
            assert!(
                sounding > 0.01,
                "the other accepted hold must keep sounding"
            );
        }
        assert_eq!(
            peak(&channel[last_release_at + 2048..]),
            0.0,
            "both gates release"
        );
    }
    wave
}

/// Scenario "Chop MIDI hit keeps held keyboard release target".
#[test]
fn chop_midi_hit_keeps_held_keyboard_release_target() {
    for source in [Source::Notes, Source::Button] {
        for guard in [
            MixedHoldGuard::Chop,
            MixedHoldGuard::Fx,
            MixedHoldGuard::Question,
            MixedHoldGuard::Learn,
        ] {
            for keyboard_first in [true, false] {
                render_mixed_chop_holds(source, false, keyboard_first, guard, false);
            }
        }
    }
}

/// Scenario "Chop rejected MIDI hit keeps held keyboard release target".
#[test]
fn chop_rejected_midi_hit_keeps_held_keyboard_release_target() {
    for source in [Source::Notes, Source::Button] {
        for guard in [
            MixedHoldGuard::Chop,
            MixedHoldGuard::Fx,
            MixedHoldGuard::Question,
            MixedHoldGuard::Learn,
        ] {
            for keyboard_first in [true, false] {
                render_mixed_chop_holds(source, false, keyboard_first, guard, true);
            }
        }
    }
}

/// Scenario "Chop keyboard hit keeps held MIDI release target".
#[test]
fn chop_keyboard_hit_keeps_held_midi_release_target() {
    for source in [Source::Notes, Source::Button] {
        for guard in [
            MixedHoldGuard::Chop,
            MixedHoldGuard::Fx,
            MixedHoldGuard::Question,
            MixedHoldGuard::Learn,
        ] {
            for keyboard_first in [true, false] {
                render_mixed_chop_holds(source, true, keyboard_first, guard, false);
            }
        }
    }
}

#[test]
fn render_85_mixed_holds() {
    let mut wave = (Vec::new(), Vec::new());
    for source in [Source::Notes, Source::Button] {
        for (midi_first, reject_midi) in [(false, false), (true, false), (false, true)] {
            append_stereo(
                &mut wave,
                render_mixed_chop_holds(
                    source,
                    midi_first,
                    true,
                    MixedHoldGuard::Chop,
                    reject_midi,
                ),
            );
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/85-mixed-holds.wav");
    engine::offline::write_wav(&path, RENDER_RATE, &wave.0, &wave.1).unwrap();
    println!(
        "85 mixed holds: {} frames, finite/no-clip/silence/onset/independent-release checked, sample-identical to timestamped oracle",
        wave.0.len()
    );
}

#[test]
fn offline_chop_to_fx_keeps_routes_velocity_banks_and_original_releases() {
    use engine::checks::{first_loud, first_non_finite, peak};
    let mut expected = None;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(12, true);
        let snap = drive.app.pads.snapshot(18);
        for command in drive.app.pads.put_back(34, &snap) {
            drive.app.send(command);
        }
        drive.frame();
        drive.render_blocks(32);
        let hit = drive.velocity_hit(source, 80);
        assert!(
            matches!(hit.ui[..], [Command::PadPress { pad: 18, velocity }]
            if velocity == 80.0 / 127.0)
        );
        let mut wave = drive.render_blocks(8);
        assert!(peak(&wave.0) > 0.01);
        assert!(first_loud(&wave.0, 0..RENDER_BLOCK, 1e-6).unwrap() <= 1);
        drive.tap(KeyCode::PageDown, M::NONE);
        drive.tap(KeyCode::F(4), M::NONE);
        assert_eq!(drive.app.pads.bank, 2);
        assert_eq!(drive.app.screen, Screen::Fx);
        assert_only_release(&drive.pad_release(source, 2), 18);
        let tail = drive.render_blocks(64);
        assert_eq!(peak(&tail.0[2048..]), 0.0);
        append_stereo(&mut wave, tail);

        // No frame handles this hit before the next callback. Keyboard writes
        // through App immediately; normal MIDI must use its real direct queue.
        match source {
            Source::Keyboard => drive.key(KeyCode::Char('3'), M::NONE, KeyEventKind::Press),
            Source::Notes => drive.bytes(PAD_PORT, &[0x90, 38, 80]),
            Source::Button => drive.bytes(PAD_PORT, &[0xb0, 80, 80]),
        }
        drive.midi().recorded_fence();
        let direct = drive.render_blocks(1);
        assert!(peak(&direct.0) > 0.01, "{source:?}: next block before UI");
        assert!(first_loud(&direct.0, 0..RENDER_BLOCK, 1e-6).unwrap() <= 1);
        append_stereo(&mut wave, direct);
        let notification = drive.frame();
        if matches!(source, Source::Keyboard) {
            assert!(
                matches!(notification.ui[..], [Command::PadPress { pad: 34, velocity }]
                if velocity == 80.0 / 127.0)
            );
        } else {
            assert!(notification.ui.is_empty(), "no second FX hit");
        }
        assert_eq!(drive.app.pads.selected_abs(), 34);
        assert!(!drive.app.pads.held[18]);
        assert!(drive.app.pads.held[34]);
        let release = drive.pad_release(source, 2);
        if matches!(source, Source::Keyboard) {
            assert_only_release(&release, 34);
        } else {
            assert!(release.ui.is_empty(), "no second direct release");
        }
        assert!(!drive.app.pads.held[34]);
        let tail = drive.render_blocks(64);
        assert_eq!(peak(&tail.0[2048..]), 0.0, "FX gate releases its own pad");
        append_stereo(&mut wave, tail);
        for channel in [&wave.0, &wave.1] {
            assert_eq!(first_non_finite(channel), None);
            assert!(peak(channel) < 1.0);
        }
        if let Some(expected) = &expected {
            assert_eq!(
                &wave, expected,
                "{source:?}: same sound across the route change"
            );
        } else {
            expected = Some(wave);
        }
    }
}

/// Scenario "Take a file and hit a target in one update".
#[test]
fn take_a_file_and_hit_a_target_in_one_update() {
    use engine::checks::peak;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        let path = drive.ordered_file();
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        let before = drive.app.pads.snapshot(18);
        match source {
            Source::Keyboard => {
                drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Press);
                drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Release);
                drive.key(KeyCode::Char('3'), M::NONE, KeyEventKind::Press);
                drive.key(KeyCode::Char('3'), M::NONE, KeyEventKind::Release);
            }
            Source::Notes | Source::Button => {
                drive.bytes(PAD_PORT, &[0xb0, 82, 127]);
                drive.bytes(
                    PAD_PORT,
                    if matches!(source, Source::Notes) {
                        &[0x90, 38, 80]
                    } else {
                        &[0xb0, 80, 80]
                    },
                );
                drive.midi().recorded_fence();
            }
        }
        let before_ui = drive.render_blocks(2);
        assert_eq!(
            peak(&before_ui.0),
            0.0,
            "{source:?}: target must not play old sound before UI"
        );
        let hit = drive.frame();
        assert_silent_target(&hit);
        assert!(
            matches!(drive.app.ask, Some(Ask::PutFile { abs: 18, .. })),
            "{source:?}: bank {}, selected {}, hand {:?}, question {:?}, ui {:?}",
            drive.app.pads.bank,
            drive.app.pads.selected,
            drive.app.browser.in_hand,
            drive.app.ask.as_ref().map(|ask| ask.lines()),
            hit.ui
        );
        assert_eq!(drive.app.pads.snapshot(18), before);
        assert_eq!(drive.app.browser.in_hand.as_ref(), Some(&path));
        drive.tap(KeyCode::Esc, M::NONE);
        assert!(drive.app.ask.is_none());
        assert_eq!(drive.app.pads.snapshot(18), before);
        assert_eq!(drive.app.browser.in_hand.as_ref(), Some(&path));
        let after = drive.render_blocks(8);
        assert_eq!(peak(&after.0), 0.0);
        assert!(drive.app.midi_ui_held.is_empty());
    }
}

fn render_mode_direct(source: Source, velocity: u8, edit_marking: bool) -> Stereo {
    const HIT: usize = 32 * RENDER_BLOCK;
    const OFF: usize = 34 * RENDER_BLOCK;
    const END: usize = 66 * RENDER_BLOCK;
    let mut drive = Drive::offline();
    let settings = drive.parity_pad(0, false);
    drive.tap(KeyCode::F(2), M::NONE);
    if edit_marking {
        drive.tap(KeyCode::Char('m'), M::NONE);
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.edit && drive.app.pads.marking);
    }
    if matches!(source, Source::Button) {
        drive.bind_pad_button(2);
    }
    let mut wave = drive.render_blocks(32);
    let notifications = if matches!(source, Source::Keyboard) {
        let hit = drive.velocity_hit(source, velocity);
        assert!(
            matches!(hit.ui[..], [Command::PadPress { pad: 18, velocity: v }] if v == f32::from(velocity)/127.0)
        );
        Vec::new()
    } else {
        drive.raw_ordered_hit(source, velocity);
        let events = drive.queued_events();
        assert!(events.iter().any(|e| matches!(e.event,
            midi::Event::PadHit { abs: 18, sound: midi::PadSound::DirectAccepted, velocity: v, .. }
            if v == f32::from(velocity)/127.0)));
        assert!(events.iter().all(|e| e.sequence().is_none()));
        events
    };
    let before_ui = drive.render_blocks(2);
    assert!(
        engine::checks::peak(&before_ui.0) > 0.005,
        "direct hit must not wait for App"
    );
    append_stereo(&mut wave, before_ui);
    for event in notifications {
        drive.app.midi_ui_event(event);
    }
    assert!(
        drive.frame().ui.is_empty(),
        "direct notification cannot start another sound"
    );
    if matches!(source, Source::Keyboard) {
        assert_only_release(&drive.pad_release(source, 2), 18);
    } else {
        drive.raw_ordered_release(source, RecordedRelease::Off);
        assert!(drive.frame().ui.is_empty());
    }
    append_stereo(&mut wave, drive.render_blocks(32));
    let mut engine = engine::Engine::new(RENDER_RATE);
    for cmd in settings {
        engine.handle(0, cmd);
    }
    let expected = engine::offline::render(
        &mut engine,
        &engine::offline::Scenario {
            sample_rate: RENDER_RATE,
            frames: END,
            events: vec![
                engine::offline::Event {
                    frame: HIT,
                    cmd: Command::PadPress {
                        pad: 18,
                        velocity: f32::from(velocity) / 127.0,
                    },
                },
                engine::offline::Event {
                    frame: OFF,
                    cmd: Command::PadRelease { pad: 18 },
                },
            ],
        },
        RENDER_BLOCK,
    );
    assert_eq!(
        wave, expected,
        "one nearest-block direct hit, original release and velocity"
    );
    for channel in [&wave.0, &wave.1] {
        assert_eq!(engine::checks::first_non_finite(channel), None);
        assert!(engine::checks::peak(channel) < 1.0);
        assert_eq!(engine::checks::peak(&channel[..HIT]), 0.0);
        assert!(
            engine::checks::first_loud(channel, HIT..HIT + RENDER_BLOCK, 1e-6)
                .unwrap()
                .abs_diff(HIT)
                <= 1
        );
        assert_eq!(engine::checks::peak(&channel[OFF + 2048..]), 0.0);
    }
    wave
}

fn render_mode_target(source: Source) -> Stereo {
    let mut drive = Drive::offline();
    drive.parity_pad(0, false);
    drive.ordered_file();
    if matches!(source, Source::Button) {
        drive.bind_pad_button(2);
    }
    let before = drive.app.pads.snapshot(18);
    let mut wave = drive.render_blocks(32);
    drive.bytes(PAD_PORT, &[0xb0, 82, 127]);
    drive.raw_ordered_hit(source, 80);
    drive.midi().recorded_fence();
    append_stereo(&mut wave, drive.render_blocks(2));
    assert_silent_target(&drive.frame());
    assert!(matches!(drive.app.ask, Some(Ask::PutFile { abs: 18, .. })));
    assert_eq!(drive.app.pads.snapshot(18), before);
    append_stereo(&mut wave, drive.render_blocks(32));
    for channel in [&wave.0, &wave.1] {
        assert_eq!(engine::checks::first_non_finite(channel), None);
        assert_eq!(
            engine::checks::peak(channel),
            0.0,
            "silent take-file target"
        );
    }
    wave
}

#[test]
fn render_85_input_modes() {
    let keyboard = render_mode_direct(Source::Keyboard, 80, false);
    let mut wave = keyboard.clone();
    for source in [Source::Notes, Source::Button] {
        append_stereo(&mut wave, render_mode_target(source));
        let direct = render_mode_direct(source, 80, true);
        assert_eq!(
            direct, keyboard,
            "matching velocity across keyboard, notes and learned button"
        );
        append_stereo(&mut wave, direct);
        append_stereo(
            &mut wave,
            render_ordered_gate(source, 80, RecordedRelease::Off, false, MixedHoldGuard::Fx),
        );
    }
    for source in [Source::Notes, Source::Button] {
        for velocity in [23, 101] {
            append_stereo(&mut wave, render_mode_direct(source, velocity, false));
        }
    }
    assert_eq!(wave.0.len(), 99_072);
    assert_eq!(wave.1.len(), wave.0.len());
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/85-input-modes.wav");
    engine::offline::write_wav(&path, RENDER_RATE, &wave.0, &wave.1).unwrap();
    println!(
        "85 input modes: 99072 frames, 2.064 s; silent targets, one direct/deferred onset, original releases, finite/no-clip and sample-identical timestamped oracles checked"
    );
}

/// Scenario "Hit from a controller": unchanged direct path, also with edit and marks.
#[test]
fn hit_from_a_controller() {
    for source in [Source::Notes, Source::Button] {
        for velocity in [23, 101] {
            for edit_marking in [false, true] {
                render_mode_direct(source, velocity, edit_marking);
            }
        }
    }
}

/// Scenario "Direct MIDI resumes after pending input".
#[test]
fn direct_midi_resumes_after_pending_input() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.tap(KeyCode::Char('1'), M::CONTROL);
        drive.bind_action(84, "bank.next");
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
        drive.raw_ordered_hit(source, 101);
        let mut events = drive.queued_events().into_iter();
        for event in events.by_ref() {
            let command = matches!(event.event, midi::Event::Action(Action::BankNext));
            drive.app.midi_ui_event(event);
            if command {
                break;
            }
        }
        // Retiring the command is not enough: the earlier UI hit is still pending.
        drive.raw_ordered_hit(source, 23);
        let following = drive.queued_events();
        assert!(following.iter().any(|event| matches!(
            event.event,
            midi::Event::PadHit {
                sound: midi::PadSound::UiUnattempted { deferred: true },
                ..
            }
        )));
        assert_eq!(engine::checks::peak(&drive.render_blocks(2).0), 0.0);
        for event in events.chain(following) {
            drive.app.midi_ui_event(event);
        }
        let ui = drive.frame();
        assert_eq!(
            ui.ui
                .iter()
                .filter(|c| matches!(c, Command::PadPress { pad: 18, .. }))
                .count(),
            2
        );
        assert!(!drive.app.midi_ui_held.is_empty());
        assert!(
            drive
                .app
                .midi_ui_held
                .values()
                .all(|pads| pads.iter().all(|hold| hold.abs == 18 && hold.deferred))
        );
        drive.render_blocks(4);
        drive.raw_ordered_hit(source, 101);
        let direct = drive.queued_events();
        assert!(direct.iter().any(|event| matches!(
            event.event,
            midi::Event::PadHit {
                sound: midi::PadSound::DirectAccepted,
                abs: 18,
                ..
            }
        )));
        // Stable direct sound arrives without a further UI update.
        assert!(engine::checks::peak(&drive.render_blocks(2).0) > 0.01);
        for event in direct {
            drive.app.midi_ui_event(event);
        }
        assert!(drive.frame().ui.is_empty());
        drive.raw_ordered_release(source, RecordedRelease::Off);
        let releases = drive.queued_events();
        assert_eq!(
            releases
                .iter()
                .filter(|e| matches!(
                    e.event,
                    midi::Event::PadRelease {
                        abs: 18,
                        direct: true
                    }
                ))
                .count(),
            1
        );
        for event in releases {
            drive.app.midi_ui_event(event);
        }
        let release = drive.frame();
        assert!(
            matches!(release.ui[..], [Command::PadRelease { pad: 18 }]),
            "accepted deferred hold retains one UI release, separate from the direct hold"
        );
        assert!(drive.app.midi_ui_held.is_empty());
    }
}

/// Scenario "Pending pad uses preceding commands in order".
#[test]
fn pending_pad_uses_preceding_commands_in_order() {
    for source in [Source::Notes, Source::Button] {
        for velocity in [23, 101] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            let pad = drive.app.pads.snapshot(18);
            for cmd in drive.app.pads.put_back(34, &pad) {
                drive.app.send(cmd);
            }
            drive.tap(KeyCode::F(2), M::NONE);
            drive.tap(KeyCode::Char('1'), M::CONTROL);
            drive.bind_action(84, "bank.next");
            if matches!(source, Source::Button) {
                drive.bind_pad_button(2);
            }
            drive.render_blocks(32);
            for _ in 0..2 {
                drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
                drive.raw_ordered_hit(source, velocity);
            }
            drive.midi().recorded_fence();
            assert_eq!(engine::checks::peak(&drive.render_blocks(2).0), 0.0);
            let hit = drive.frame();
            assert!(hit.direct.is_empty());
            let pads: Vec<_> = hit
                .ui
                .iter()
                .filter_map(|c| match c {
                    Command::PadPress { pad, velocity: v } => {
                        assert_eq!(*v, f32::from(velocity) / 127.0);
                        Some(*pad)
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(pads, [18, 34]);
            assert_eq!(
                drive.app.midi_ui_held.len(),
                1,
                "same unreleased control, stable candidate"
            );
            assert_eq!(
                drive
                    .app
                    .midi_ui_held
                    .values()
                    .next()
                    .unwrap()
                    .iter()
                    .map(|hold| hold.abs)
                    .collect::<Vec<_>>(),
                [18, 34]
            );
            drive.raw_ordered_release(source, RecordedRelease::Off);
            let off = drive.frame();
            assert!(off.direct.is_empty());
            assert!(matches!(
                off.ui[..],
                [
                    Command::PadRelease { pad: 18 },
                    Command::PadRelease { pad: 34 }
                ]
            ));
            assert!(drive.app.midi_ui_held.is_empty());
            let stopped = drive.render_blocks(32);
            assert_eq!(engine::checks::peak(&stopped.0[2048..]), 0.0);
        }
    }
    // A blocked/no-op command and F10 transitions must retire too.
    let mut drive = Drive::new();
    drive.bind_action(85, "ui.back");
    drive.bind_action(86, "learn.toggle");
    drive.bind_action(87, "screen.chop");
    drive.bytes(PAD_PORT, &[0xb0, 85, 127]);
    drive.bytes(PAD_PORT, &[0xb0, 87, 127]); // no file: remains Pads
    drive.raw_ordered_hit(Source::Notes, 23);
    assert!(matches!(
        drive.frame().ui[..],
        [Command::PadPress { pad: 2, .. }]
    ));
    drive.bytes(PAD_PORT, &[0xb0, 86, 127]);
    drive.raw_ordered_hit(Source::Notes, 101);
    assert_silent_target(&drive.frame());
    assert!(drive.app.screen_learn.is_some());
    drive.tap(KeyCode::F(10), M::NONE);
    drive.raw_ordered_hit(Source::Notes, 101);
    assert_eq!(
        drive.frame().direct.len(),
        1,
        "normal direct route resumes after the F10 guard"
    );
}

/// Scenario "A target question blocks a following pending hit".
#[test]
fn a_target_question_blocks_a_following_pending_hit() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        let pad = drive.app.pads.snapshot(18);
        for cmd in drive.app.pads.put_back(19, &pad) {
            drive.app.send(cmd);
        }
        let path = drive.ordered_file();
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
            drive.bind_action(81, "pad.4");
        }
        drive.render_blocks(32);
        let before = [drive.app.pads.snapshot(18), drive.app.pads.snapshot(19)];
        if matches!(source, Source::Keyboard) {
            for key in [KeyCode::Enter, KeyCode::Char('3'), KeyCode::Char('4')] {
                drive.key(key, M::NONE, KeyEventKind::Press);
                drive.key(key, M::NONE, KeyEventKind::Release);
            }
        } else {
            drive.bytes(PAD_PORT, &[0xb0, 82, 127]);
            drive.raw_ordered_hit(source, 23);
            drive.bytes(
                PAD_PORT,
                if matches!(source, Source::Notes) {
                    &[0x90, 39, 101]
                } else {
                    &[0xb0, 81, 101]
                },
            );
            let mut events = drive.queued_events().into_iter();
            for event in events.by_ref() {
                let take = matches!(event.event, midi::Event::Action(Action::BrowserHold));
                drive.app.midi_ui_event(event);
                if take {
                    break;
                }
            }
            drive.raw_ordered_hit(source, 101);
            let extra = drive.queued_events();
            assert!(extra.iter().any(|event| matches!(
                event.event,
                midi::Event::PadHit {
                    sound: midi::PadSound::UiUnattempted { deferred: true },
                    ..
                }
            )));
            for event in events.chain(extra) {
                drive.app.midi_ui_event(event);
            }
        }
        assert_silent_target(&drive.frame());
        assert!(matches!(drive.app.ask, Some(Ask::PutFile { abs: 18, .. })));
        assert_eq!(drive.app.pads.selected_abs(), 18);
        assert_eq!(
            [drive.app.pads.snapshot(18), drive.app.pads.snapshot(19)],
            before
        );
        assert_eq!(drive.app.browser.in_hand.as_ref(), Some(&path));
        assert!(drive.app.midi_ui_held.is_empty());
        drive.tap(KeyCode::Esc, M::NONE);
        assert_eq!(
            [drive.app.pads.snapshot(18), drive.app.pads.snapshot(19)],
            before
        );
        assert_eq!(drive.app.browser.in_hand.as_ref(), Some(&path));
        let wave = drive.render_blocks(32);
        assert_eq!(engine::checks::peak(&wave.0), 0.0);
        assert_eq!(engine::checks::first_non_finite(&wave.0), None);
    }
}

/// Scenario "Pending command order spans controllers".
#[test]
fn pending_command_order_spans_controllers() {
    const OTHER: &str = "Recorded pads B";
    let mut drive = Drive::offline();
    drive.midi().recorded_connect(OTHER);
    drive.frame();
    drive.bind_action(84, "bank.next");
    drive.bytes(PAD_PORT, &[0x90, 37, 101]);
    drive.bytes(OTHER, &[0x92, 38, 23]);
    let initial = drive.queued_events();
    assert_eq!(
        initial
            .iter()
            .filter(|e| matches!(
                e.event,
                midi::Event::PadHit {
                    sound: midi::PadSound::DirectAccepted,
                    ..
                }
            ))
            .count(),
        2
    );
    for event in initial {
        drive.app.midi_ui_event(event);
    }
    drive.frame();
    drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
    drive.bytes(OTHER, &[0x92, 38, 101]);
    drive.bytes(PAD_PORT, &[0x80, 37, 0]);
    drive.bytes(OTHER, &[0x82, 38, 0]);
    let events = drive.queued_events();
    let direct: Vec<_> = events
        .iter()
        .filter_map(|e| match e.event {
            midi::Event::PadRelease { abs, direct: true } => Some(abs),
            _ => None,
        })
        .collect();
    assert_eq!(direct, [1, 2]);
    for event in events {
        drive.app.midi_ui_event(event);
    }
    let frame = drive.frame();
    assert!(
        matches!(frame.ui[..], [Command::PadPress { pad: 18, velocity }, Command::PadRelease { pad: 18 }]
        if velocity == 101.0/127.0),
        "cross-port UI: {:?}, bank {}",
        frame.ui,
        drive.app.pads.bank
    );
    assert!(drive.app.midi_ui_held.is_empty());
}

/// Scenario "Pending command order spans controllers": take from A, target from B,
/// and original direct releases from both ports before any UI consumption.
#[test]
fn pending_file_target_spans_controllers_and_keeps_direct_releases() {
    const OTHER: &str = "Recorded pads B";
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        let pad = drive.app.pads.snapshot(18);
        for abs in [17, 19] {
            for cmd in drive.app.pads.put_back(abs, &pad) {
                drive.app.send(cmd);
            }
        }
        let path = drive.ordered_file();
        drive.midi().recorded_connect(OTHER);
        drive.frame();
        if matches!(source, Source::Button) {
            drive.bind_pad_button(3);
        }
        drive.render_blocks(32);
        drive.bytes(PAD_PORT, &[0x90, 37, 101]);
        drive.bytes(OTHER, &[0x92, 38, 23]);
        drive.frame();
        assert!(engine::checks::peak(&drive.render_blocks(4).0) > 0.01);
        drive.bytes(PAD_PORT, &[0xb0, 82, 127]);
        drive.bytes(
            OTHER,
            if matches!(source, Source::Notes) {
                &[0x92, 39, 101]
            } else {
                &[0xb2, 80, 101]
            },
        );
        drive.bytes(PAD_PORT, &[0x80, 37, 0]);
        drive.bytes(OTHER, &[0x82, 38, 0]);
        let events = drive.queued_events();
        let releases: Vec<_> = events
            .iter()
            .filter_map(|e| match e.event {
                midi::Event::PadRelease { abs, direct: true } => Some(abs),
                _ => None,
            })
            .collect();
        assert_eq!(releases, [17, 18]);
        // Both original direct gates stop while the command and target still wait.
        assert_eq!(
            engine::checks::peak(&drive.render_blocks(32).0[2048..]),
            0.0
        );
        let before = drive.app.pads.snapshot(19);
        for event in events {
            drive.app.midi_ui_event(event);
        }
        assert_silent_target(&drive.frame());
        assert!(matches!(drive.app.ask, Some(Ask::PutFile { abs: 19, .. })));
        assert_eq!(drive.app.pads.snapshot(19), before);
        assert_eq!(drive.app.browser.in_hand.as_ref(), Some(&path));
        assert!(drive.app.midi_ui_held.is_empty());
        let quiet = drive.render_blocks(32);
        assert_eq!(engine::checks::peak(&quiet.0), 0.0);
        assert_eq!(engine::checks::first_non_finite(&quiet.0), None);
    }
}

/// Scenario "Pending pad uses preceding commands in order": paired keyboard/MIDI
/// bank, cancelled hand and no-op commands leave the same ordinary pad hit.
#[test]
fn pending_cancelled_hand_matches_keyboard_command_order() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.ordered_file();
        drive.tap(KeyCode::Char('1'), M::CONTROL);
        drive.bind_action(84, "bank.next");
        drive.bind_action(85, "ui.back");
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        if matches!(source, Source::Keyboard) {
            for key in [KeyCode::Enter, KeyCode::Esc, KeyCode::PageDown] {
                drive.key(key, M::NONE, KeyEventKind::Press);
                drive.key(key, M::NONE, KeyEventKind::Release);
            }
            drive.key(KeyCode::Char('3'), M::NONE, KeyEventKind::Press);
        } else {
            for cc in [82, 85, 84] {
                drive.bytes(PAD_PORT, &[0xb0, cc, 127]);
            }
            drive.raw_ordered_hit(source, 80);
        }
        let frame = drive.frame();
        assert!(frame.direct.is_empty());
        assert!(
            matches!(frame.ui.iter().filter(|c| matches!(c, Command::PadPress { .. })).collect::<Vec<_>>()[..],
            [Command::PadPress { pad: 18, velocity }] if *velocity == 80.0/127.0)
        );
        assert!(drive.app.browser.in_hand.is_none());
        assert_eq!(drive.app.pads.bank, 1);
        assert!(engine::checks::peak(&drive.render_blocks(4).0) > 0.01);
        if matches!(source, Source::Keyboard) {
            drive.pad_release(source, 2);
        } else {
            drive.raw_ordered_release(source, RecordedRelease::Off);
            drive.frame();
        }
        assert_eq!(
            engine::checks::peak(&drive.render_blocks(32).0[2048..]),
            0.0
        );
    }
}

fn render_ordered_gate(
    source: Source,
    velocity: u8,
    release: RecordedRelease,
    early: bool,
    guard: MixedHoldGuard,
) -> Stereo {
    const HIT: usize = 34 * RENDER_BLOCK;
    const OFF: usize = 50 * RENDER_BLOCK;
    const END: usize = 90 * RENDER_BLOCK;
    let mut drive = Drive::offline();
    let settings = drive.parity_pad(0, false);
    drive.tap(KeyCode::F(2), M::NONE);
    drive.tap(KeyCode::Char('1'), M::CONTROL);
    drive.bind_action(84, "bank.next");
    if matches!(source, Source::Button) {
        drive.bind_pad_button(2);
    }
    let mut wave = drive.render_blocks(32);
    drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
    drive.raw_ordered_hit(source, velocity);
    if early {
        drive.raw_ordered_release(source, release);
    }
    drive.midi().recorded_fence();
    append_stereo(&mut wave, drive.render_blocks(2));
    assert_eq!(engine::checks::peak(&wave.0), 0.0, "no old-bank direct hit");
    let hit = drive.frame();
    assert!(hit.direct.is_empty());
    assert!(
        matches!(hit.ui.first(), Some(Command::PadPress { pad: 18, velocity: v }) if *v == f32::from(velocity)/127.0)
    );
    if early {
        assert!(matches!(
            hit.ui[..],
            [
                Command::PadPress { pad: 18, .. },
                Command::PadRelease { pad: 18 }
            ]
        ));
    } else {
        assert_eq!(drive.app.audio_held[18], Some(0));
        assert_eq!(
            drive
                .app
                .midi_ui_held
                .values()
                .next()
                .unwrap()
                .iter()
                .map(|hold| hold.abs)
                .collect::<Vec<_>>(),
            [18]
        );
    }
    append_stereo(&mut wave, drive.render_blocks(16));
    if !early {
        if !matches!(guard, MixedHoldGuard::Chop) {
            drive.tap(KeyCode::Char('3'), M::CONTROL);
        }
        match guard {
            MixedHoldGuard::Chop => {
                drive.tap(KeyCode::F(3), M::NONE);
            }
            MixedHoldGuard::Fx => {
                drive.tap(KeyCode::F(4), M::NONE);
            }
            MixedHoldGuard::Question => {
                drive.app.ask = Some(Ask::Plan { lines: vec![] });
                drive.frame();
            }
            MixedHoldGuard::Learn => {
                drive.tap(KeyCode::F(10), M::NONE);
            }
        }
        drive.raw_ordered_release(source, release);
        let off = drive.frame();
        assert_only_release(&off, 18);
    }
    assert!(drive.app.midi_ui_held.is_empty());
    append_stereo(&mut wave, drive.render_blocks((END - OFF) / RENDER_BLOCK));
    let mut engine = engine::Engine::new(RENDER_RATE);
    for cmd in settings {
        engine.handle(0, cmd);
    }
    let mut events = vec![engine::offline::Event {
        frame: HIT,
        cmd: Command::PadPress {
            pad: 18,
            velocity: f32::from(velocity) / 127.0,
        },
    }];
    if !early && matches!(guard, MixedHoldGuard::Chop) {
        events.push(engine::offline::Event {
            frame: OFF,
            cmd: Command::HushPads,
        });
    }
    events.push(engine::offline::Event {
        frame: if early { HIT } else { OFF },
        cmd: Command::PadRelease { pad: 18 },
    });
    let expected = engine::offline::render(
        &mut engine,
        &engine::offline::Scenario {
            sample_rate: RENDER_RATE,
            frames: END,
            events,
        },
        RENDER_BLOCK,
    );
    assert_eq!(
        wave, expected,
        "actual target, velocity, onset and release: {source:?}, {velocity}, {release:?}, early {early}, {guard:?}"
    );
    for channel in [&wave.0, &wave.1] {
        assert_eq!(engine::checks::first_non_finite(channel), None);
        assert!(engine::checks::peak(channel) < 1.0);
        assert_eq!(engine::checks::peak(&channel[..HIT]), 0.0);
        assert_eq!(engine::checks::peak(&channel[OFF + 2048..]), 0.0);
        if !early {
            let onset = engine::checks::first_loud(channel, HIT..HIT + RENDER_BLOCK, 1e-6).unwrap();
            assert!(onset.abs_diff(HIT) <= 1);
        }
    }
    wave
}

/// Scenario "Deferred gate receives an early release or disconnect".
#[test]
fn deferred_gate_receives_an_early_release_or_disconnect() {
    for source in [Source::Notes, Source::Button] {
        for velocity in [23, 101] {
            for release in [
                RecordedRelease::Off,
                RecordedRelease::Zero,
                RecordedRelease::Disconnect,
            ] {
                render_ordered_gate(source, velocity, release, true, MixedHoldGuard::Fx);
            }
        }
    }
}

/// Scenario "Deferred release follows its actual pad across modes".
#[test]
fn deferred_release_follows_its_actual_pad_across_modes() {
    for source in [Source::Notes, Source::Button] {
        for velocity in [23, 101] {
            for release in [
                RecordedRelease::Off,
                RecordedRelease::Zero,
                RecordedRelease::Disconnect,
            ] {
                for guard in [
                    MixedHoldGuard::Chop,
                    MixedHoldGuard::Fx,
                    MixedHoldGuard::Question,
                    MixedHoldGuard::Learn,
                ] {
                    render_ordered_gate(source, velocity, release, false, guard);
                }
            }
        }
    }
}

/// Scenario "Deferred silent and rejected hits preserve accepted holds".
#[test]
fn deferred_silent_and_rejected_hits_preserve_accepted_holds() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.tap(KeyCode::Char('1'), M::CONTROL);
        drive.bind_action(84, "bank.next");
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
        drive.raw_ordered_hit(source, 101);
        drive.frame();
        drive.render_blocks(4);
        let held = drive.app.midi_ui_held.clone();
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
        drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
        drive.raw_ordered_hit(source, 23);
        drive.frame();
        assert_eq!(
            drive.app.midi_ui_held, held,
            "rejected target must not replace accepted identity"
        );
        assert_eq!(drive.app.audio_held[18], Some(0));
        assert_eq!(drive.app.audio_held[34], None);
        drive.raw_ordered_release(source, RecordedRelease::Off);
        let off = drive.frame();
        assert_only_release(&off, 18);
        assert!(drive.app.midi_ui_held.is_empty());
        assert!(!drive.app.pending_audio.is_empty());
        let continuing = drive.render_blocks(1);
        assert!(
            engine::checks::peak(&continuing.0) > 0.01,
            "accepted gate survives rejected retrigger"
        );
        drive.app.flush_audio();
        let tail = drive.render_blocks(32);
        assert_eq!(engine::checks::first_non_finite(&tail.0), None);
        assert!(engine::checks::peak(&tail.0) < 1.0);
        assert_eq!(
            engine::checks::peak(&tail.0[2048..]),
            0.0,
            "pending release retries"
        );
    }
}

/// Scenario "Deferred gate receives an early release or disconnect": callback
/// already consumed an older direct hold's release before App starts the retrigger.
#[test]
fn early_direct_release_cannot_steal_a_later_deferred_hold() {
    for source in [Source::Notes, Source::Button] {
        for release in [
            RecordedRelease::Off,
            RecordedRelease::Zero,
            RecordedRelease::Disconnect,
        ] {
            let mut drive = Drive::offline();
            let settings = drive.parity_pad(0, false);
            drive.tap(KeyCode::F(2), M::NONE);
            drive.bind_action(85, "ui.back");
            if matches!(source, Source::Button) {
                drive.bind_pad_button(2);
            }
            let mut wave = drive.render_blocks(32);
            drive.raw_ordered_hit(source, 101);
            drive.midi().recorded_fence();
            append_stereo(&mut wave, drive.render_blocks(4));
            drive.frame();
            drive.bytes(PAD_PORT, &[0xb0, 85, 127]);
            drive.raw_ordered_hit(source, 23);
            drive.raw_ordered_release(source, release);
            drive.midi().recorded_fence();
            let old_release = drive.render_blocks(32);
            assert_eq!(engine::checks::peak(&old_release.0[2048..]), 0.0);
            append_stereo(&mut wave, old_release);
            let ui = drive.frame();
            let tail = drive.render_blocks(64);
            assert_eq!(
                engine::checks::peak(&tail.0[2048..]),
                0.0,
                "{source:?}, {release:?}: older direct release cannot retire a later UI gate"
            );
            append_stereo(&mut wave, tail);
            assert!(matches!(
                ui.ui[..],
                [
                    Command::PadPress { pad: 18, .. },
                    Command::PadRelease { pad: 18 }
                ]
            ));
            assert!(drive.app.midi_ui_held.is_empty());
            let mut engine = engine::Engine::new(RENDER_RATE);
            for cmd in settings {
                engine.handle(0, cmd);
            }
            let expected = engine::offline::render(
                &mut engine,
                &engine::offline::Scenario {
                    sample_rate: RENDER_RATE,
                    frames: 132 * RENDER_BLOCK,
                    events: vec![
                        engine::offline::Event {
                            frame: 32 * RENDER_BLOCK,
                            cmd: Command::PadPress {
                                pad: 18,
                                velocity: 101.0 / 127.0,
                            },
                        },
                        engine::offline::Event {
                            frame: 36 * RENDER_BLOCK,
                            cmd: Command::PadRelease { pad: 18 },
                        },
                        engine::offline::Event {
                            frame: 68 * RENDER_BLOCK,
                            cmd: Command::PadPress {
                                pad: 18,
                                velocity: 23.0 / 127.0,
                            },
                        },
                        engine::offline::Event {
                            frame: 68 * RENDER_BLOCK,
                            cmd: Command::PadRelease { pad: 18 },
                        },
                    ],
                },
                RENDER_BLOCK,
            );
            assert_eq!(
                wave, expected,
                "independent callback/UI progress and both accepted holds"
            );
            for channel in [&wave.0, &wave.1] {
                assert_eq!(engine::checks::first_non_finite(channel), None);
                assert!(engine::checks::peak(channel) < 1.0);
            }
        }
    }
}

/// Scenarios "Deferred silent and rejected hits preserve accepted holds" and
/// "Deferred release follows its actual pad across modes": both direct/UI orders.
#[test]
fn deferred_retriggers_keep_both_release_obligations_for_both_press_orders() {
    for source in [Source::Notes, Source::Button] {
        for ui_first in [false, true] {
            for same_pad in [false, true] {
                for disconnect in [false, true] {
                    let mut drive = Drive::offline();
                    drive.parity_pad(0, false);
                    let pad = drive.app.pads.snapshot(18);
                    for cmd in drive.app.pads.put_back(34, &pad) {
                        drive.app.send(cmd);
                    }
                    drive.tap(KeyCode::F(2), M::NONE);
                    drive.bind_action(84, "bank.next");
                    drive.bind_action(85, "ui.back");
                    if matches!(source, Source::Button) {
                        drive.bind_pad_button(2);
                    }
                    drive.render_blocks(32);
                    if ui_first {
                        drive.bytes(PAD_PORT, &[0xb0, 85, 127]);
                    }
                    drive.raw_ordered_hit(source, 101);
                    drive.frame();
                    drive.render_blocks(4);
                    if ui_first {
                        if !same_pad {
                            drive.tap(KeyCode::Char('3'), M::CONTROL);
                        }
                    } else {
                        drive.bytes(PAD_PORT, &[0xb0, if same_pad { 85 } else { 84 }, 127]);
                    }
                    drive.raw_ordered_hit(source, 23);
                    drive.frame();
                    drive.render_blocks(4);
                    let direct_abs = if ui_first && !same_pad { 34 } else { 18 };
                    let ui_abs = if !ui_first && !same_pad { 34 } else { 18 };
                    assert_eq!(
                        drive
                            .app
                            .midi_ui_held
                            .values()
                            .next()
                            .unwrap()
                            .iter()
                            .map(|hold| hold.abs)
                            .collect::<Vec<_>>(),
                        [ui_abs]
                    );
                    drive.raw_ordered_release(
                        source,
                        if disconnect {
                            RecordedRelease::Disconnect
                        } else {
                            RecordedRelease::Off
                        },
                    );
                    let events = drive.queued_events();
                    let direct: Vec<_> = events
                        .iter()
                        .filter_map(|e| match e.event {
                            midi::Event::PadRelease { abs, direct: true } => Some(abs),
                            _ => None,
                        })
                        .collect();
                    assert_eq!(direct, [direct_abs]);
                    for event in events {
                        drive.app.midi_ui_event(event);
                    }
                    let off = drive.frame();
                    assert!(
                        matches!(off.ui[..], [Command::PadRelease { pad }] if pad == ui_abs),
                        "each accepted deferred hold keeps its own release"
                    );
                    assert!(drive.app.midi_ui_held.is_empty());
                    let stopped = drive.render_blocks(32);
                    for channel in [&stopped.0, &stopped.1] {
                        assert_eq!(engine::checks::first_non_finite(channel), None);
                        assert!(engine::checks::peak(channel) < 1.0);
                        assert_eq!(engine::checks::peak(&channel[2048..]), 0.0);
                    }
                }
            }
        }
    }
}

/// Scenario "Deferred silent and rejected hits preserve accepted holds": silent,
/// preview and assignment retriggers, including a different physical control.
#[test]
fn silent_deferred_candidates_do_not_steal_an_accepted_hold() {
    for source in [Source::Notes, Source::Button] {
        for context in ["question", "learn", "preview", "assign"] {
            let mut drive = Drive::offline();
            drive.parity_pad(0, false);
            drive.tap(KeyCode::F(2), M::NONE);
            drive.tap(KeyCode::Char('1'), M::CONTROL);
            drive.bind_action(84, "bank.next");
            drive.bind_action(81, "pad.4");
            if matches!(source, Source::Button) {
                drive.bind_pad_button(2);
            }
            drive.render_blocks(32);
            drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
            drive.raw_ordered_hit(source, 101);
            drive.frame();
            drive.render_blocks(4);
            let held = drive.app.midi_ui_held.clone();
            drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
            let command = drive.queued_events();
            match context {
                "question" => {
                    drive.app.pads.marked.insert(18);
                    drive.tap(KeyCode::Backspace, M::NONE);
                    assert!(matches!(drive.app.ask, Some(Ask::ClearPads { .. })));
                }
                "learn" => {
                    drive.tap(KeyCode::F(10), M::NONE);
                }
                "preview" | "assign" => {
                    let selected = drive.app.pads.selected_abs();
                    drive.tap(KeyCode::F(3), M::NONE);
                    assert_eq!(
                        drive.app.screen,
                        Screen::Chop,
                        "{source:?}, {context}, selected {selected}, file {:?}",
                        drive.app.pads.file[selected]
                    );
                    if context == "assign" {
                        drive.tap(KeyCode::Char('p'), M::NONE);
                    } else {
                        // Bare-slice listen requires no linked pads anywhere, not
                        // merely an empty target on the newly selected bank.
                        drive.app.pads.link.fill(None);
                    }
                }
                _ => unreachable!(),
            }
            let learned = drive.app.learned.clone();
            drive.raw_ordered_hit(source, 23);
            drive.bytes(PAD_PORT, &[0xb0, 81, 101]);
            let hits = drive.queued_events();
            for event in command.into_iter().chain(hits) {
                drive.app.midi_ui_event(event);
            }
            assert_eq!(drive.app.midi_ui_held, held, "{source:?}, {context}");
            assert_eq!(
                drive.app.learned, learned,
                "deferred hit is not a learn confirmation"
            );
            drive.bytes(PAD_PORT, &[0xb0, 81, 0]);
            drive.frame();
            assert_eq!(
                drive.app.midi_ui_held, held,
                "another control cannot release the accepted hold"
            );
            drive.raw_ordered_release(source, RecordedRelease::Off);
            assert_only_release(&drive.frame(), 18);
            assert!(drive.app.midi_ui_held.is_empty());
            drive.app.stop_preview();
            let stopped = drive.render_blocks(64);
            assert_eq!(engine::checks::first_non_finite(&stopped.0), None);
            assert!(engine::checks::peak(&stopped.0) < 1.0);
            assert_eq!(engine::checks::peak(&stopped.0[4096..]), 0.0);
        }
    }
}

/// Scenario "Old deferred input cannot affect a new project".
#[test]
fn old_deferred_input_cannot_affect_a_new_project() {
    use std::sync::atomic::Ordering::Acquire;
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.bind_action(84, "bank.next");
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
        drive.raw_ordered_hit(source, 23);
        drive.raw_ordered_release(source, RecordedRelease::Off);
        let old = drive.queued_events();
        assert!(old.iter().any(|e| e.sequence().is_some()));
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
        let before = drive.app.pads.snapshot(18);
        for event in old {
            drive.app.midi_ui_event(event);
        }
        assert_eq!(drive.app.pads.snapshot(18), before);
        assert!(drive.app.midi_ui_held.is_empty());
        assert!(
            !drive
                .app
                .sent
                .iter()
                .any(|c| matches!(c, Command::PadPress { .. } | Command::PadRelease { .. }))
        );
        drive.frame();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        drive.raw_ordered_hit(source, 101);
        let next = drive.queued_events();
        assert!(
            next.iter().any(|e| matches!(
                e.event,
                midi::Event::PadHit {
                    abs: 18,
                    sound: midi::PadSound::DirectAccepted,
                    ..
                }
            )),
            "{source:?}: stale numbered work retires; focus {:?}, ready {}, epoch {}, events {:?}",
            drive.app.focus(),
            drive.project.ready.load(Acquire),
            drive.project.epoch.load(Acquire),
            next.iter()
                .map(|e| match &e.event {
                    midi::Event::PadHit { abs, sound, .. } => format!(
                        "hit {abs} {sound:?}, ready {}, seq {:?}",
                        e.preview_ready(),
                        e.sequence()
                    ),
                    midi::Event::Message { meaning, .. } => meaning.clone(),
                    _ => "other".into(),
                })
                .collect::<Vec<_>>()
        );
        assert!(engine::checks::peak(&drive.render_blocks(2).0) > 0.01);
        for event in next {
            drive.app.midi_ui_event(event);
        }
        assert!(drive.frame().ui.is_empty());
    }
}

/// Scenario "Take a file and hit a target in one update": empty targets finish loading.
#[test]
fn pending_file_on_an_empty_target_finishes_without_sound() {
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.app.clear_pads(&[18]);
        let path = drive.ordered_file();
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        if matches!(source, Source::Keyboard) {
            drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Press);
            drive.key(KeyCode::Enter, M::NONE, KeyEventKind::Release);
            drive.key(KeyCode::Char('3'), M::NONE, KeyEventKind::Press);
            drive.key(KeyCode::Char('3'), M::NONE, KeyEventKind::Release);
        } else {
            drive.bytes(PAD_PORT, &[0xb0, 82, 127]);
            drive.raw_ordered_hit(source, 101);
            drive.midi().recorded_fence();
        }
        assert_eq!(engine::checks::peak(&drive.render_blocks(2).0), 0.0);
        assert_silent_target(&drive.frame());
        assert!(drive.app.ask.is_none());
        assert!(drive.app.pads.loading[18].is_some());
        assert!(drive.app.browser.in_hand.is_none());
        while drive.app.pads.loading[18].is_some() {
            let reply = drive
                .app
                .loader
                .as_ref()
                .unwrap()
                .1
                .recv_timeout(std::time::Duration::from_secs(1))
                .unwrap();
            drive.app.loader_reply(reply);
        }
        assert_eq!(drive.app.pads.file[18].as_ref(), Some(&path));
        assert!(drive.app.pads.sound[18].is_some());
        assert_silent_target(&drive.frame());
        assert!(drive.app.midi_ui_held.is_empty());
        let quiet = drive.render_blocks(32);
        for channel in [&quiet.0, &quiet.1] {
            assert_eq!(engine::checks::peak(channel), 0.0);
            assert_eq!(engine::checks::first_non_finite(channel), None);
        }
    }
}

/// Scenario "Pending pad uses preceding commands in order": keep real Chop entry/exit.
#[test]
fn pending_chop_entry_and_exit_keep_fifo_routes() {
    for source in [Source::Notes, Source::Button] {
        let mut drive = Drive::offline();
        drive.parity_pad(0, false);
        drive.tap(KeyCode::F(2), M::NONE);
        drive.app.pads.selected = 2;
        drive.bind_action(87, "screen.chop");
        drive.bind_action(88, "screen.pads");
        if matches!(source, Source::Button) {
            drive.bind_pad_button(2);
        }
        drive.render_blocks(32);
        drive.bytes(PAD_PORT, &[0xb0, 87, 127]);
        drive.raw_ordered_hit(source, 23);
        drive.bytes(PAD_PORT, &[0xb0, 88, 127]);
        drive.raw_ordered_hit(source, 101);
        drive.midi().recorded_fence();
        assert_eq!(engine::checks::peak(&drive.render_blocks(2).0), 0.0);
        let frame = drive.frame();
        assert!(frame.direct.is_empty());
        let hits: Vec<_> = frame
            .ui
            .iter()
            .filter_map(|c| match c {
                Command::PadPress { pad, velocity } => Some((*pad, *velocity)),
                _ => None,
            })
            .collect();
        assert_eq!(hits, [(18, 23.0 / 127.0), (18, 101.0 / 127.0)]);
        assert!(frame.ui.iter().any(|c| matches!(c, Command::HushPads)));
        assert_eq!(drive.app.screen, Screen::Pads);
        drive.raw_ordered_release(source, RecordedRelease::Off);
        assert_only_release(&drive.frame(), 18);
        assert!(drive.app.midi_ui_held.is_empty());
        assert_eq!(
            engine::checks::peak(&drive.render_blocks(32).0[2048..]),
            0.0
        );
    }
}

/// Scenario "Pending pad uses preceding commands in order": old Ignore must not drop pending work.
#[test]
fn pending_hit_retires_under_an_old_f10_snapshot() {
    let mut drive = Drive::offline();
    drive.bind_action(84, "bank.next");
    drive.bytes(PAD_PORT, &[0xb0, 84, 127]);
    let command = drive.queued_events();
    drive.key(KeyCode::F(10), M::NONE, KeyEventKind::Press);
    drive.key(KeyCode::F(10), M::NONE, KeyEventKind::Release);
    drive.app.publish_midi(&drive.shared);
    assert_eq!(
        input::route_pad(drive.shared.focus().unwrap()),
        input::PadRoute::Ignore
    );
    drive.raw_ordered_hit(Source::Notes, 101);
    let hit = drive.queued_events();
    assert!(hit.iter().any(|e| matches!(
        e.event,
        midi::Event::PadHit {
            sound: midi::PadSound::UiUnattempted { deferred: true },
            ..
        }
    )));
    for event in command.into_iter().chain(hit) {
        drive.app.midi_ui_event(event);
    }
    assert_silent_target(&drive.frame());
    assert_eq!(
        drive.app.pads.bank, 0,
        "F10 blocks the command body but still retires its sequence"
    );
    assert!(drive.app.midi_ui_held.is_empty());
    drive.tap(KeyCode::F(10), M::NONE);
    drive.raw_ordered_hit(Source::Notes, 101);
    let resumed = drive.queued_events();
    assert!(resumed.iter().any(|e| matches!(
        e.event,
        midi::Event::PadHit {
            sound: midi::PadSound::DirectAccepted,
            ..
        }
    )));
    for event in resumed {
        drive.app.midi_ui_event(event);
    }
    assert!(drive.frame().ui.is_empty());
}

#[derive(Clone, Copy, Debug)]
enum LateChop {
    Linked,
    Bare,
    Assign,
    AssignOrder,
}

fn render_late_direct_notification(
    source: Source,
    rejected: bool,
    mode: LateChop,
    change_bank: bool,
    velocity: u8,
) -> Stereo {
    use engine::checks::{first_loud, first_non_finite, peak};
    let mut drive = Drive::offline();
    let mut settings = drive.parity_pad(0, true);
    // Offline Engine starts with built-in test tones, unlike the empty UI fixture.
    let clear_filler = Command::SetPadSound {
        pad: 31,
        sound: None,
    };
    drive.app.send(clear_filler.clone());
    settings.push(clear_filler);
    drive.tap(KeyCode::F(2), M::NONE);
    if matches!(source, Source::Button) {
        drive.bind_pad_button(2);
    }
    let mut wave = drive.render_blocks(32);
    if rejected {
        // Fill the production MIDI queue, not the UI queue. The filler pad is empty.
        for _ in 0..1024 {
            drive.bytes(PAD_PORT, &[0x91, 51, 127]);
        }
        drive.frame();
    }
    // Select the fixture's loaded pad before the delayed hit. Filler notifications
    // were consumed without a callback, so the real rejection queue is still full.
    drive.app.pads.selected = 2;
    let hit = if matches!(source, Source::Notes) {
        [0x90, 38, velocity]
    } else {
        [0xb0, 80, velocity]
    };
    drive.bytes(PAD_PORT, &hit);
    drive.midi().recorded_fence();
    let delayed: Vec<_> = drive.midi().events.try_iter().collect();
    assert!(delayed.iter().any(|event| matches!(event.event,
        midi::Event::PadHit { abs: 18, sound, velocity: v, ui_press: None }
        if sound == if rejected { midi::PadSound::DirectRejected } else { midi::PadSound::DirectAccepted }
            && v == f32::from(velocity) / 127.0)));
    append_stereo(&mut wave, drive.render_blocks(8));
    assert_eq!(peak(&wave.0[32 * RENDER_BLOCK..]) == 0.0, rejected);
    for kind in [KeyEventKind::Press, KeyEventKind::Release] {
        drive.key(KeyCode::F(3), M::NONE, kind);
    }
    assert_eq!(drive.app.screen, Screen::Chop);
    match mode {
        LateChop::Linked => {}
        LateChop::Bare => drive.app.pads.link[18] = None,
        LateChop::Assign | LateChop::AssignOrder => {
            for kind in [KeyEventKind::Press, KeyEventKind::Release] {
                drive.key(KeyCode::Char('p'), M::NONE, kind);
            }
            assert_eq!(drive.app.plan, Some(chop::Plan::ByHand(Vec::new())));
            if matches!(mode, LateChop::AssignOrder) {
                for kind in [KeyEventKind::Press, KeyEventKind::Release] {
                    drive.key(KeyCode::Tab, M::NONE, kind);
                }
                assert_eq!(drive.app.plan, Some(chop::Plan::InOrder(None)));
            }
        }
    }
    if change_bank {
        for kind in [KeyEventKind::Press, KeyEventKind::Release] {
            drive.key(KeyCode::Char('3'), M::CONTROL, kind);
        }
        assert_eq!(drive.app.pads.bank, 2);
    }
    drive.app.chop.as_mut().unwrap().select_slice(1);
    let (slice, cursor, plan) = (
        drive.app.chop.as_ref().unwrap().selected,
        drive.app.chop.as_ref().unwrap().cursor,
        drive.app.plan.clone(),
    );
    let entry = std::mem::take(&mut drive.app.sent);
    assert_eq!(
        entry
            .iter()
            .filter(|c| matches!(c, Command::HushPads))
            .count(),
        1
    );
    assert!(!entry.iter().any(|c| matches!(
        c,
        Command::PadPress { .. } | Command::Preview { sound: Some(_), .. }
    )));
    for event in delayed {
        drive.app.midi_ui_event(event);
    }
    let notification = drive.frame();
    assert!(
        notification.ui.is_empty(),
        "late {source:?}, rejected {rejected}, {mode:?}: {:?}",
        notification.ui
    );
    assert_eq!(drive.app.chop.as_ref().unwrap().selected, slice);
    assert_eq!(drive.app.chop.as_ref().unwrap().cursor, cursor);
    assert_eq!(drive.app.plan, plan);
    assert!(drive.app.midi_ui_held.is_empty());
    append_stereo(&mut wave, drive.render_blocks(64));
    drive.bytes(
        PAD_PORT,
        if matches!(source, Source::Notes) {
            &[0x80, 38, 0]
        } else {
            &[0xb0, 80, 0]
        },
    );
    drive.midi().recorded_fence();
    let releases: Vec<_> = drive.midi().events.try_iter().collect();
    if !rejected {
        assert!(releases.iter().any(|event| matches!(
            event.event,
            midi::Event::PadRelease {
                abs: 18,
                direct: true
            }
        )));
    }
    for event in releases {
        drive.app.midi_ui_event(event);
    }
    assert!(drive.frame().ui.is_empty(), "no extra UI audio release");
    let release_at = wave.0.len();
    append_stereo(&mut wave, drive.render_blocks(32));

    let mut expected_engine = engine::Engine::new(RENDER_RATE);
    for setting in settings {
        expected_engine.handle(0, setting);
    }
    let mut events = Vec::new();
    if !rejected {
        events.push(engine::offline::Event {
            frame: 32 * RENDER_BLOCK,
            cmd: Command::PadPress {
                pad: 18,
                velocity: f32::from(velocity) / 127.0,
            },
        });
    }
    events.extend(entry.into_iter().map(|cmd| engine::offline::Event {
        frame: 40 * RENDER_BLOCK,
        cmd,
    }));
    if !rejected {
        events.push(engine::offline::Event {
            frame: release_at,
            cmd: Command::PadRelease { pad: 18 },
        });
    }
    let expected = engine::offline::render(
        &mut expected_engine,
        &engine::offline::Scenario {
            sample_rate: RENDER_RATE,
            frames: wave.0.len(),
            events,
        },
        RENDER_BLOCK,
    );
    assert_eq!(
        wave, expected,
        "{source:?}, rejected {rejected}, {mode:?}, bank change {change_bank}, velocity {velocity}"
    );
    for channel in [&wave.0, &wave.1] {
        assert_eq!(first_non_finite(channel), None);
        assert!(peak(channel) < 1.0);
        assert_eq!(peak(&channel[..32 * RENDER_BLOCK]), 0.0);
        if rejected {
            assert_eq!(peak(channel), 0.0, "a rejected hit stays silent");
        } else {
            assert!(
                first_loud(channel, 32 * RENDER_BLOCK..33 * RENDER_BLOCK, 1e-6)
                    .unwrap()
                    .abs_diff(32 * RENDER_BLOCK)
                    <= 1
            );
        }
        assert_eq!(
            peak(&channel[40 * RENDER_BLOCK + 2048..]),
            0.0,
            "Chop hush without restart"
        );
    }
    wave
}

/// Scenario "Late accepted direct MIDI notification does not restart Chop".
#[test]
fn late_accepted_direct_midi_notification_does_not_restart_chop() {
    for source in [Source::Notes, Source::Button] {
        for mode in [
            LateChop::Linked,
            LateChop::Bare,
            LateChop::Assign,
            LateChop::AssignOrder,
        ] {
            for change_bank in [false, true] {
                for velocity in [23, 101] {
                    render_late_direct_notification(source, false, mode, change_bank, velocity);
                }
            }
        }
    }
}

/// Scenario "Late rejected direct MIDI notification stays silent on Chop".
#[test]
fn late_rejected_direct_midi_notification_stays_silent_on_chop() {
    for source in [Source::Notes, Source::Button] {
        for mode in [
            LateChop::Linked,
            LateChop::Bare,
            LateChop::Assign,
            LateChop::AssignOrder,
        ] {
            for change_bank in [false, true] {
                for velocity in [23, 101] {
                    render_late_direct_notification(source, true, mode, change_bank, velocity);
                }
            }
        }
    }
}

#[test]
fn render_85_late_midi() {
    let mut wave = (Vec::new(), Vec::new());
    for source in [Source::Notes, Source::Button] {
        for rejected in [false, true] {
            append_stereo(
                &mut wave,
                render_late_direct_notification(source, rejected, LateChop::Linked, false, 101),
            );
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/85-late-midi.wav");
    engine::offline::write_wav(&path, RENDER_RATE, &wave.0, &wave.1).unwrap();
    println!(
        "85 late MIDI: {} frames, finite/no-clip/silence/onset/hush checked, sample-identical to timestamped oracle",
        wave.0.len()
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

struct TransitionFiles(PathBuf);

impl TransitionFiles {
    fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/86c-transitions")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Relaxed)
            ));
        for name in ["A", "B"] {
            let folder = root.join(name);
            std::fs::create_dir_all(folder.join("samples")).unwrap();
            session::save_project(&folder.join("project.toml"), &session::Session::default())
                .unwrap();
        }
        let wave = vec![0.25; 4096];
        engine::offline::write_wav(&root.join("tone.wav"), 48_000, &wave, &wave).unwrap();
        Self(root)
    }

    fn drive(&self, project: bool) -> Drive {
        let mut drive = Drive::new();
        drive.bind_action(88, "project.pick");
        drive.bind_action(89, "pad.mode_next");
        drive.bind_action(90, "chop.cut");
        let file = self.0.join("tone.wav");
        let sound = Arc::new(Sample::mono(vec![0.25; 4096]));
        drive.load_pad(0, sound.clone(), "tone");
        drive.app.pads.file[0] = Some(file.clone());
        drive.app.chop = Some(chop::Chop::new(file, sound, 48_000));
        drive.app.paths.projects = self.0.clone();
        drive.app.browser = browser::Browser::new(Some(self.0.clone()));
        drive.app.preview_on = false;
        drive.app.session_path = self.0.join("scratch/session.toml");
        drive.app.session_read_error = None;
        if project {
            drive.app.project = Some(Project {
                name: "A".into(),
                folder: self.0.join("A"),
            });
            drive.app.project_cfg.last = Some("A".into());
        }
        drive.app.save_session().unwrap();
        let config_path = self.0.join("config.toml");
        config::save_section(&config_path, "project", &drive.app.project_cfg).unwrap();
        drive.app.user_path = Some(config_path);
        drive.frame();
        drive
    }

    fn save_feedback_drive(&self, project: bool, source: Source) -> Drive {
        let mut drive = self.drive(project);
        for (number, action) in [
            (91, "session.save"),
            (92, "bank.next"),
            (93, "master.volume_up"),
        ] {
            // Learning keeps one control per action in a group.
            let key = match source {
                Source::Notes => input::midi::Key::Note(number),
                _ => input::midi::Key::Cc(number),
            };
            learn::assign(
                &mut drive.app.learned,
                "Recorded pads",
                input::Group::Global,
                key,
                action,
            );
        }
        drive
            .app
            .sync_learned("Recorded pads", input::Group::Global);
        drive.midi().recorded_fence();
        drive
    }

    fn edit(drive: &mut Drive, source: Source) {
        drive.registry_command(source, KeyCode::Char('h'), M::NONE, 89);
        drive.tap(KeyCode::F(3), M::NONE);
        assert_eq!(drive.app.screen, Screen::Chop);
        for _ in 0..16 {
            drive.tap(KeyCode::Right, M::NONE);
        }
        drive.registry_command(source, KeyCode::Char('m'), M::NONE, 90);
        assert_eq!(drive.app.chop.as_ref().unwrap().saved().cuts.len(), 1);
        drive.tap(KeyCode::F(2), M::NONE);
        assert!(drive.app.session_dirty.is_some());
        assert!(!drive.app.history.is_empty());
    }

    fn pick(drive: &mut Drive, source: Source, name: &str, create: bool) {
        drive.registry_command(source, KeyCode::Char('p'), M::CONTROL, 88);
        assert!(drive.app.project_picker.is_some());
        // The current project picker has keyboard selection and confirmation only.
        // MIDI still enters through the production action and dispatch above.
        if create {
            for c in name.chars() {
                drive.tap(KeyCode::Char(c), M::NONE);
            }
        } else {
            let at = drive
                .app
                .project_picker
                .as_ref()
                .unwrap()
                .entries
                .iter()
                .position(|entry| entry == name)
                .unwrap();
            for _ in 0..at {
                drive.tap(KeyCode::Down, M::NONE);
            }
        }
    }

    fn picker(drive: &Drive) -> (Vec<String>, usize, Option<String>) {
        let picker = drive.app.project_picker.as_ref().unwrap();
        (
            picker.entries.clone(),
            picker.selected,
            picker.naming.clone(),
        )
    }

    fn state(drive: &Drive) -> Seen {
        let mut seen = drive.seen();
        seen.notice = None;
        seen
    }
}

impl Drop for TransitionFiles {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

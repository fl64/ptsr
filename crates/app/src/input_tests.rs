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

#[test]
fn existing_screen_bindings_survive_the_registry_change() {
    use input::{Group, midi::Key};
    let screens = [
        (Screen::Pads, KeyCode::F(2), "screen.pads"),
        (Screen::Settings, KeyCode::F(12), "screen.settings"),
        (Screen::Files, KeyCode::F(6), "screen.files"),
        (Screen::Chop, KeyCode::F(3), "screen.chop"),
        (Screen::Fx, KeyCode::F(4), "screen.fx"),
    ];
    let path = {
        let mut old = Drive::new();
        let maps = old.app.learned.get_mut("Recorded pads").unwrap();
        let global = maps.entry("global".into()).or_default();
        for key in [Key::Note as fn(u8) -> Key, Key::Cc] {
            for (index, (_, _, action)) in screens.iter().enumerate() {
                global.insert(key(90 + index as u8).to_string(), (*action).into());
            }
            for (number, action) in [
                (95, "bank.next"),
                (96, "bank.next"),
                (98, "help.toggle"),
                (100, "bank.prev"),
                (101, "ui.back"),
            ] {
                global.insert(key(number).to_string(), action.into());
            }
        }
        for (screen, _, _) in screens {
            let group = maps.entry(screen.key().into()).or_default();
            for key in [Key::Note as fn(u8) -> Key, Key::Cc] {
                group.insert(key(96).to_string(), "help.toggle".into());
                if matches!(screen, Screen::Pads | Screen::Chop) {
                    group.insert(
                        key(99).to_string(),
                        if screen == Screen::Pads {
                            "pad.edit"
                        } else {
                            "chop.edit"
                        }
                        .into(),
                    );
                }
            }
        }
        for group in [Group::PadsEdit, Group::ChopEdit] {
            let map = maps.entry(group.key().into()).or_default();
            for key in [Key::Note as fn(u8) -> Key, Key::Cc] {
                map.insert(key(96).to_string(), "edit.param_next".into());
            }
        }
        old.save_registry_bindings()
    };
    let saved = std::fs::read_to_string(&path).unwrap();
    let expected = config::load_from(&path).config.midi;
    for source in [Source::Keyboard, Source::Notes, Source::Button] {
        let mut drive = Drive::with_saved_bindings(false, Some(&path));
        assert_eq!(drive.app.learned, expected);
        let mut overrides = String::from(
            "[global]\n\"Ctrl+H\" = \"bank.next\"\n\"Ctrl+J\" = \"help.toggle\"\n\
             [edit]\n\"Ctrl+H\" = \"edit.param_next\"\n",
        );
        for name in ["pads", "settings", "files", "chop", "fx"] {
            overrides.push_str(&format!(
                "[screens.{name}]\n\"Ctrl+H\" = \"help.toggle\"\n\"Ctrl+J\" = \"none\"\n"
            ));
        }
        assert!(drive.keymap.overlay(&overrides).unwrap().is_empty());
        drive.prepare_chop(Arc::new(Sample::mono(vec![0.25; 48_000])), 4);
        drive.link_chop_pad(0, 0);
        drive.link_chop_pad(4, 1);
        for (index, (screen, key, _)) in screens.iter().enumerate() {
            drive.registry_command(source, *key, M::NONE, 90 + index as u8);
            assert_eq!(drive.app.screen, *screen);
            assert!(!drive.app.edit);
            let bank = drive.app.pads.bank;
            drive.registry_command(source, KeyCode::Char('h'), M::CONTROL, 96);
            assert!(drive.app.help_open);
            assert_eq!(drive.app.pads.bank, bank);
            drive.registry_command(source, KeyCode::F(1), M::NONE, 98);
            assert!(!drive.app.help_open);
            // Keyboard `none` blocks its global binding. MIDI 97 stays unassigned.
            drive.registry_command(source, KeyCode::Char('j'), M::CONTROL, 97);
            assert!(!drive.app.help_open);
            if matches!(screen, Screen::Pads | Screen::Chop) {
                drive.registry_command(source, KeyCode::Enter, M::NONE, 99);
                assert!(drive.app.edit);
                let before = (drive.app.pads.param, drive.app.slice_end);
                drive.registry_command(source, KeyCode::Char('h'), M::CONTROL, 96);
                assert_ne!((drive.app.pads.param, drive.app.slice_end), before);
                assert!(!drive.app.help_open);
                drive.registry_command(source, KeyCode::Esc, M::NONE, 101);
                assert!(!drive.app.edit);
            }
            if matches!(source, Source::Button) {
                drive.bind_pad_button(4);
            }
            match source {
                Source::Keyboard => drive.key(KeyCode::Char('q'), M::NONE, KeyEventKind::Press),
                Source::Notes => drive.bytes(PAD_PORT, &[0x90, 40, 80]),
                Source::Button => drive.bytes(PAD_PORT, &[0xb0, 80, 80]),
            }
            let hit = drive.frame();
            assert!(hit.commands().into_iter().any(|command| matches!(command,
                Command::PadPress { pad: 4, velocity } if *velocity == 80.0 / 127.0)));
            drive.pad_release(source, 4);
            let hit = drive.pad_press(source, 4);
            assert_eq!(drive.app.pads.selected_abs(), 4);
            if matches!(source, Source::Keyboard) || *screen == Screen::Chop {
                assert!(hit.direct.is_empty());
            } else {
                assert!(
                    !hit.ui
                        .iter()
                        .any(|command| matches!(command, Command::PadPress { .. }))
                );
            }
            assert!(hit.commands().into_iter().any(|command| matches!(command,
                Command::PadPress { pad: 4, velocity } if *velocity == 1.0)));
            drive.registry_command(source, KeyCode::F(4), M::NONE, 94);
            drive.registry_command(source, KeyCode::PageDown, M::NONE, 95);
            assert_eq!(drive.app.pads.bank, 1);
            let release = drive.pad_release(source, 4);
            assert!(
                release
                    .commands()
                    .into_iter()
                    .any(|command| matches!(command, Command::PadRelease { pad: 4 }))
            );
            assert!(
                !release
                    .commands()
                    .into_iter()
                    .any(|command| matches!(command, Command::PadRelease { pad: 20 }))
            );
            drive.registry_command(source, KeyCode::PageUp, M::NONE, 100);
            assert_eq!(drive.app.pads.bank, 0);
        }
        assert_eq!(std::fs::read_to_string(&path).unwrap(), saved);
    }
}

#[test]
fn fx_controls_keep_their_priority_through_the_registry() {
    use input::{Group, midi::Key};
    let path = {
        let mut old = Drive::new();
        for (number, action) in [(80, "edit.param_next"), (81, "edit.value_inc_fine")] {
            learn::assign(
                &mut old.app.learned,
                "X-Session Pro",
                Group::Fx,
                Key::Cc(number),
                action,
            );
        }
        old.save_registry_bindings()
    };
    let mut results = Vec::new();
    for keyboard in [true, false] {
        let mut drive = Drive::with_saved_bindings(false, Some(&path));
        let command = |drive: &mut Drive, code, mods, bytes: [u8; 3]| {
            if keyboard {
                drive.tap(code, mods);
            } else {
                drive.bytes(XS_PORT, &bytes);
                drive.bytes(XS_PORT, &[bytes[0], bytes[1], 0]);
                drive.frame();
            }
            assert!(!drive.app.edit);
        };
        command(&mut drive, KeyCode::F(4), M::NONE, [0x90, 45, 127]);
        assert_eq!(drive.app.screen, Screen::Fx);
        command(&mut drive, KeyCode::Tab, M::NONE, [0x90, 43, 127]);
        assert_eq!((drive.app.fx_slot, drive.app.fx_param), (1, 0));
        command(&mut drive, KeyCode::Char('k'), M::NONE, [0xb0, 80, 127]);
        assert_eq!(drive.app.fx_param, 1);
        let before = drive.app.pads.fx_pos(pads::Param::Fx2A);
        command(&mut drive, KeyCode::Char('l'), M::NONE, [0xb0, 81, 127]);
        assert!(drive.app.pads.fx_pos(pads::Param::Fx2A) > before);
        results.push((
            drive.app.fx_slot,
            drive.app.fx_param,
            drive.app.pads.fx_pos(pads::Param::Fx2A),
        ));
    }
    assert_eq!(results[0], results[1]);

    // A saved pre-FX profile inherits its old page control, not a later scope.
    let inherited = {
        let mut old = Drive::new();
        old.app
            .learned
            .get_mut("X-Session Pro")
            .unwrap()
            .remove("fx");
        for (group, number) in [
            (Group::PadsEdit, 82),
            (Group::ChopEdit, 83),
            (Group::Global, 84),
        ] {
            learn::assign(
                &mut old.app.learned,
                "X-Session Pro",
                group,
                Key::Cc(number),
                "edit.param_1",
            );
        }
        old.save_registry_bindings()
    };
    let mut drive = Drive::with_saved_bindings(false, Some(&inherited));
    drive.tap(KeyCode::F(4), M::NONE);
    drive.bytes(XS_PORT, &[0xb0, 83, 127]);
    drive.bytes(XS_PORT, &[0xb0, 84, 127]);
    drive.frame();
    assert_eq!(drive.app.pads.fx_pos(pads::Param::Fx1Kind), 0.25);
    drive.bytes(XS_PORT, &[0xb0, 82, 127]);
    drive.frame();
    assert_eq!(drive.app.pads.fx_pos(pads::Param::Fx1Kind), 0.75);
    learn::assign(
        &mut drive.app.learned,
        "X-Session Pro",
        Group::Fx,
        Key::Cc(82),
        "edit.param_2",
    );
    drive.app.sync_learned("X-Session Pro", Group::Fx);
    drive.bytes(XS_PORT, &[0xb0, 82, 0]);
    drive.frame();
    assert_eq!(drive.app.pads.fx_pos(pads::Param::Fx1Kind), 0.75);
    assert_eq!(drive.app.pads.fx_pos(pads::Param::Fx1A), 0.0);

    // Global changes reach FX without an old copied assignment.
    for number in [85, 86] {
        learn::assign(
            &mut drive.app.learned,
            "X-Session Pro",
            Group::Global,
            Key::Cc(number),
            "pad.volume",
        );
        drive.app.sync_learned("X-Session Pro", Group::Global);
    }
    let before = drive.app.pads.volume_db[0];
    drive.bytes(XS_PORT, &[0xb0, 85, 0]);
    drive.frame();
    assert_eq!(drive.app.pads.volume_db[0], before);
    drive.bytes(XS_PORT, &[0xb0, 86, 0]);
    drive.frame();
    assert!(drive.app.pads.volume_db[0] < before);
    assert!(
        learn::clear(&mut drive.app.learned, Group::Global, "pad.volume")
            .contains(&"X-Session Pro".into())
    );
    drive.app.sync_learned("X-Session Pro", Group::Global);
    let before = drive.app.pads.volume_db[0];
    drive.bytes(XS_PORT, &[0xb0, 86, 127]);
    drive.frame();
    assert_eq!(drive.app.pads.volume_db[0], before);

    drive
        .app
        .learned
        .get_mut("X-Session Pro")
        .unwrap()
        .get_mut("fx")
        .unwrap()
        .clear();
    let empty = drive.save_registry_bindings();
    drop(drive);
    let mut drive = Drive::with_saved_bindings(false, Some(&empty));
    assert!(drive.app.learned["X-Session Pro"]["fx"].is_empty());
    drive.tap(KeyCode::F(4), M::NONE);
    let before = (
        drive.app.fx_slot,
        drive.app.fx_param,
        drive.app.pads.fx_pos(pads::Param::Fx1Kind),
    );
    for (status, number) in [(0xb0, 24), (0xb0, 82), (0xb0, 83), (0xb0, 84), (0x90, 43)] {
        drive.bytes(XS_PORT, &[status, number, 127]);
        drive.bytes(XS_PORT, &[status, number, 0]);
        drive.frame();
    }
    assert_eq!(
        (
            drive.app.fx_slot,
            drive.app.fx_param,
            drive.app.pads.fx_pos(pads::Param::Fx1Kind)
        ),
        before
    );
    assert!(!drive.app.edit);
    assert!(drive.app.learned["X-Session Pro"]["fx"].is_empty());
}

#[test]
fn each_registered_screen_keeps_its_effective_help() {
    use ratatui::crossterm::event::ModifierKeyCode;
    for source in [Source::Keyboard, Source::Button] {
        let mut drive = Drive::registry_screen_checks();
        assert!(
            drive
                .keymap
                .overlay("[global]\nF8 = \"bank.next\"\nPageDown = \"none\"\n")
                .unwrap()
                .is_empty()
        );
        for (screen, key, number) in REGISTRY_SCREEN_COMMANDS {
            let (action, old_key, group_names, headings): (&str, &str, &[&str], &[&str]) =
                match screen {
                    Screen::Pads => (
                        "pad.edit",
                        "Enter",
                        &["PAD", "CHANGE", "BANK", "REC", ""],
                        &[
                            "PADS",
                            "BANK",
                            "CHOP",
                            "SOUND",
                            "SCREENS",
                            "GLYPHS ON A PAD",
                        ],
                    ),
                    Screen::Settings => (
                        "settings.row_next",
                        "K",
                        &["SETTINGS", "REC", ""],
                        &["PADS", "BANK", "CHOP", "SETTINGS", "SOUND", "SCREENS"],
                    ),
                    Screen::Files => (
                        "browser.hold",
                        "Enter",
                        &["FILES", "BANK", "REC", ""],
                        &[
                            "PADS",
                            "BANK",
                            "FILES",
                            "CHOP",
                            "SOUND",
                            "SCREENS",
                            "GLYPHS ON A PAD",
                        ],
                    ),
                    Screen::Chop => (
                        "chop.cut",
                        "M",
                        &["MOVE", "CUT", "PADS", "BOUNDS", "BANK", "REC", ""],
                        &["PADS", "BANK", "CHOP", "SOUND", "SCREENS"],
                    ),
                    Screen::Fx => (
                        "edit.page_next",
                        "Tab",
                        &["SHARED FX", "BANK", "REC", ""],
                        &["PADS", "BANK", "CHOP", "SOUND", "SCREENS"],
                    ),
                };
            assert!(
                drive
                    .keymap
                    .overlay(&format!(
                        "[screens.{}]\nF9 = \"{action}\"\n{old_key} = \"none\"\n",
                        screen.key()
                    ))
                    .unwrap()
                    .is_empty()
            );
            // F3 from Files uses its selected file. This check opens the loaded pad.
            drive.registry_command(source, KeyCode::F(2), M::NONE, 90);
            drive.registry_command(source, key, M::NONE, number);
            assert_eq!(drive.app.screen, screen);
            let groups = help_groups(&drive.keymap, screen, &drive.app.help_state());
            assert_eq!(
                groups.iter().map(|group| group.name).collect::<Vec<_>>(),
                group_names
            );
            assert!(
                groups
                    .iter()
                    .flat_map(|group| &group.items)
                    .any(|(keys, _)| keys.split(' ').any(|key| key == "F9")),
                "{screen:?}: override must be visible"
            );
            let normal =
                without_modifiers(help_groups(&drive.keymap, screen, &drive.app.help_state()));
            assert!(
                normal
                    .iter()
                    .flat_map(|group| &group.items)
                    .all(|(keys, _)| !keys.contains('+'))
            );
            for width in [100, 252] {
                assert_eq!(tui::help_rows(&groups, width), 3);
                assert_eq!(tui::help_rows(&normal, width), 3);
            }
            assert_eq!(tui::help_dropped(&groups, 252), 0);
            let dropped = tui::help_dropped(&groups, 100);
            assert!(dropped < groups.len());
            assert_eq!(groups[0].name, group_names[0]);
            if screen == Screen::Files {
                let kept = &groups[..groups.len() - dropped];
                assert!(kept.iter().any(|group| group.name == "BANK"));
                assert!(!kept.iter().any(|group| group.name.is_empty()));
            }
            drive.registry_command(source, KeyCode::F(1), M::NONE, 95);
            assert!(drive.app.help_open);
            let full = full_help(&drive.keymap, screen, true);
            assert_eq!(
                full.iter()
                    .filter(|(keys, _)| keys.is_empty())
                    .map(|(_, text)| text.as_str())
                    .collect::<Vec<_>>(),
                headings
            );
            assert!(
                full.iter()
                    .any(|(keys, text)| keys.split(' ').any(|key| key == "F8")
                        && text == "previous / next bank")
            );
            assert!(
                !full
                    .iter()
                    .any(|(keys, _)| keys.split(' ').any(|key| key == "PgDn"))
            );
            assert!(
                full.iter()
                    .any(|(keys, _)| keys.split(' ').any(|key| key == "F9"))
            );
            assert!(
                full_help(&drive.keymap, screen, false)
                    .iter()
                    .any(|(_, text)| text == "NO KEY RELEASES")
            );
            for (code, flags, expected, name) in [
                (
                    ModifierKeyCode::LeftControl,
                    M::CONTROL,
                    input::Modifier::Ctrl,
                    "CTRL",
                ),
                (
                    ModifierKeyCode::LeftAlt,
                    M::ALT,
                    input::Modifier::Alt,
                    "ALT",
                ),
                (
                    ModifierKeyCode::LeftShift,
                    M::SHIFT,
                    input::Modifier::Shift,
                    "SHIFT",
                ),
            ] {
                // Modifier help follows keyboard holds, also after a MIDI screen command.
                drive.key(KeyCode::Modifier(code), flags, KeyEventKind::Press);
                assert_eq!(drive.mods.get(), Some(expected));
                let held = modifier_groups(&drive.keymap, screen, expected, drive.app.ui.hide_pads);
                assert_eq!(held.len(), 1);
                assert_eq!(held[0].name, name);
                if expected == input::Modifier::Shift {
                    assert!(
                        held[0]
                            .items
                            .iter()
                            .any(|(_, text)| text == "pads at full velocity")
                    );
                    if screen == Screen::Chop {
                        assert!(
                            held[0]
                                .items
                                .contains(&("Shift+H".into(), "fewer hit cuts".into()))
                        );
                    }
                }
                drive.key(KeyCode::Modifier(code), M::NONE, KeyEventKind::Release);
                assert_eq!(drive.mods.get(), None);
            }
            drive.registry_command(source, KeyCode::F(1), M::NONE, 95);
            assert!(!drive.app.help_open);
            let texts = |drive: &Drive| {
                help_groups(&drive.keymap, screen, &drive.app.help_state())
                    .into_iter()
                    .flat_map(|group| group.items.into_iter().map(|(_, text)| text))
                    .collect::<Vec<_>>()
            };
            match screen {
                Screen::Pads => {
                    assert!(
                        texts(&drive)
                            .iter()
                            .any(|text| text.starts_with("mark: off"))
                    );
                    drive.registry_command(source, KeyCode::Char('m'), M::NONE, 28);
                    assert!(
                        texts(&drive)
                            .iter()
                            .any(|text| text.starts_with("mark: on"))
                    );
                    drive.registry_command(source, KeyCode::Char('m'), M::NONE, 28);
                }
                Screen::Files => {
                    assert!(texts(&drive).contains(&"preview: on".into()));
                    drive.registry_command(source, KeyCode::Char(' '), M::NONE, 100);
                    assert!(texts(&drive).contains(&"preview: off".into()));
                }
                Screen::Chop => {
                    drive.registry_command(source, KeyCode::Char('o'), M::ALT, 101);
                    assert!(texts(&drive).iter().any(|text| text.contains("loop: on")));
                }
                Screen::Fx => {
                    assert!(texts(&drive).contains(&"FX 1 selected".into()));
                    assert!(texts(&drive).contains(&"parameter".into()));
                    assert!(
                        full.iter()
                            .any(|(_, text)| text == "select the next / previous shared FX block")
                    );
                    assert!(
                        full.iter()
                            .any(|(_, text)| text == "select the next / previous effect parameter")
                    );
                }
                Screen::Settings => {
                    let tabs = "audio / input / MIDI / paths";
                    for expected in [
                        vec!["row", "change", tabs, "back"],
                        vec![tabs, "back"],
                        vec![
                            "learn",
                            "clear",
                            "row",
                            "open / close",
                            "groups",
                            tabs,
                            "back",
                        ],
                        vec!["row", "open", tabs, "back"],
                    ] {
                        let groups = help_groups(&drive.keymap, screen, &drive.app.help_state());
                        assert_eq!(
                            groups[0]
                                .items
                                .iter()
                                .map(|(_, text)| text.as_str())
                                .collect::<Vec<_>>(),
                            expected
                        );
                        assert_eq!(full_help(&drive.keymap, screen, true), full);
                        drive.registry_command(source, KeyCode::Tab, M::NONE, 99);
                    }
                }
            }
            if matches!(screen, Screen::Pads | Screen::Chop) {
                assert!(
                    drive
                        .keymap
                        .overlay(
                            "[edit]\nF7 = \"edit.param_next\"\nK = \"none\"\nDown = \"none\"\n"
                        )
                        .unwrap()
                        .is_empty()
                );
                let (key, number) = if screen == Screen::Pads {
                    (KeyCode::F(9), 97)
                } else {
                    (KeyCode::Enter, 98)
                };
                drive.registry_command(source, key, M::NONE, number);
                assert!(drive.app.edit);
                let edit = help_groups(&drive.keymap, screen, &drive.app.help_state());
                assert_eq!(edit.len(), 1);
                assert_eq!(edit[0].name, "EDIT");
                assert!(
                    edit[0]
                        .items
                        .iter()
                        .any(|(keys, _)| keys.split(' ').any(|key| key == "F7"))
                );
                assert!(
                    !edit[0]
                        .items
                        .iter()
                        .any(|(keys, _)| keys.split(' ').any(|key| matches!(key, "K" | "↓")))
                );
                assert!(
                    edit[0]
                        .items
                        .iter()
                        .any(|(_, text)| text == drive.app.help_state().page)
                );
                if screen == Screen::Chop {
                    assert_eq!(edit[0].items.len(), 4);
                    assert_eq!(edit[0].items[0].1, "BOUNDS");
                    assert!(!edit[0].items.iter().any(|(keys, _)| keys.contains("Tab")));
                }
                assert_eq!(tui::help_rows(&edit, 100), 3);
            }
        }
    }
}

#[test]
fn screen_learning_keeps_its_existing_entry_points() {
    use input::Group;
    for source in [Source::Keyboard, Source::Button] {
        for (screen, key, number, editing, group) in [
            (Screen::Pads, KeyCode::F(2), 90, false, Group::Pads),
            (Screen::Pads, KeyCode::F(2), 90, true, Group::PadsEdit),
            (Screen::Files, KeyCode::F(6), 92, false, Group::Files),
            (Screen::Chop, KeyCode::F(3), 93, false, Group::Chop),
            (Screen::Chop, KeyCode::F(3), 93, true, Group::ChopEdit),
            (Screen::Fx, KeyCode::F(4), 94, false, Group::Fx),
        ] {
            let mut drive = Drive::registry_screen_checks();
            drive.registry_command(source, key, M::NONE, number);
            if editing {
                drive.registry_command(
                    source,
                    KeyCode::Enter,
                    M::NONE,
                    if screen == Screen::Pads { 97 } else { 98 },
                );
            }
            assert_eq!(drive.app.edit, editing);
            let before = drive.app.learned.clone();
            drive.registry_command(source, KeyCode::F(10), M::NONE, 96);
            let learn = drive.app.screen_learn.as_ref().unwrap();
            assert_eq!(learn.group, group);
            assert_eq!(drive.app.learn_group(), group);
            assert_eq!(
                learn.page,
                if screen == Screen::Pads && editing {
                    Some(drive.app.pads.page().0)
                } else {
                    None
                }
            );
            assert!(drive.app.learn_lines().unwrap()[0].contains(group.title()));
            if screen == Screen::Chop {
                assert_eq!(drive.app.help_state().page, "BOUNDS");
                assert!(
                    drive
                        .app
                        .learn_params()
                        .iter()
                        .all(|(name, _)| !name.starts_with("edit.page"))
                );
            }
            if screen == Screen::Fx {
                assert!(!drive.app.edit);
                assert_eq!(group, Group::Fx);
            }
            drive.tap(KeyCode::Char('q'), M::NONE);
            assert_eq!(drive.app.learn_picked().unwrap().0, "pad.5");
            drive.bytes(PAD_PORT, &[0x90, 110, 100]);
            drive.bytes(PAD_PORT, &[0x80, 110, 0]);
            let pending = drive.frame();
            assert!(pending.commands().is_empty());
            assert_eq!(
                drive
                    .app
                    .screen_learn
                    .as_ref()
                    .unwrap()
                    .pending
                    .as_ref()
                    .unwrap()
                    .1,
                input::midi::Key::Note(110)
            );
            assert_eq!(drive.app.learned, before);
            // Screen learn captures MIDI as candidates. Esc cancels through its existing key path.
            drive.tap(KeyCode::Esc, M::NONE);
            assert!(drive.app.screen_learn.as_ref().unwrap().pending.is_none());
            drive.tap(KeyCode::Esc, M::NONE);
            assert!(drive.app.screen_learn.is_none());
            assert_eq!(drive.app.edit, editing);
            assert_eq!(drive.app.learned, before);
        }
        let mut drive = Drive::registry_screen_checks();
        drive.registry_command(source, KeyCode::F(12), M::NONE, 91);
        for tab in 0..4 {
            assert_eq!(drive.app.settings_tab, tab);
            let before = drive.app.learned.clone();
            drive.registry_command(source, KeyCode::F(10), M::NONE, 96);
            assert!(drive.app.screen_learn.is_none());
            assert!(drive.app.learning.is_none() && drive.app.midi_run.is_none());
            assert_eq!(drive.app.learned, before);
            assert_eq!(drive.app.screen_group(), Group::Files);
            assert_eq!(drive.app.layer_group(), Group::Files);
            drive.registry_command(source, KeyCode::Tab, M::NONE, 99);
        }
    }
}

#[test]
fn midi_list_keeps_screen_groups_and_their_order() {
    use input::{Group, midi::Scope};
    let groups = [
        Group::Global,
        Group::Pads,
        Group::PadsEdit,
        Group::Settings,
        Group::Files,
        Group::Chop,
        Group::ChopEdit,
        Group::Fx,
    ];
    let identities = [
        ("global", "GLOBAL", Scope::Global),
        ("pads", "PADS", Scope::Screen(Screen::Pads)),
        ("pads_edit", "PADS · EDIT", Scope::Edit(Screen::Pads)),
        ("settings", "SETTINGS", Scope::Screen(Screen::Settings)),
        ("files", "FILES", Scope::Screen(Screen::Files)),
        ("chop", "CHOP", Scope::Screen(Screen::Chop)),
        ("chop_edit", "CHOP · EDIT", Scope::Edit(Screen::Chop)),
        ("fx", "FX", Scope::Screen(Screen::Fx)),
    ];
    for source in [Source::Keyboard, Source::Button] {
        for (screen, key, number, group) in [
            (Screen::Pads, KeyCode::F(2), 90, Group::Pads),
            (Screen::Files, KeyCode::F(6), 92, Group::Files),
            (Screen::Chop, KeyCode::F(3), 93, Group::Chop),
            (Screen::Fx, KeyCode::F(4), 94, Group::Fx),
        ] {
            for editing in [false, true] {
                if editing && !matches!(screen, Screen::Pads | Screen::Chop) {
                    continue;
                }
                let mut drive = Drive::registry_screen_checks();
                let saved = drive.app.learned.clone();
                drive.registry_command(source, key, M::NONE, number);
                if editing {
                    drive.registry_command(
                        source,
                        KeyCode::Enter,
                        M::NONE,
                        if screen == Screen::Pads { 97 } else { 98 },
                    );
                    assert!(drive.app.edit);
                }
                drive.registry_command(source, KeyCode::F(12), M::NONE, 91);
                assert_eq!(drive.app.screen, Screen::Settings);
                assert!(!drive.app.edit);
                assert_eq!(drive.app.midi_open, [group]);
                drive.registry_command(source, KeyCode::Tab, M::NONE, 99);
                drive.registry_command(source, KeyCode::Tab, M::NONE, 99);
                assert_eq!(drive.app.settings_tab, 2);
                let rows = drive.app.midi_list();
                let listed: Vec<_> = rows
                    .iter()
                    .filter_map(|row| match row {
                        MidiListRow::Group { group } => Some(*group),
                        _ => None,
                    })
                    .collect();
                assert_eq!(listed, groups);
                assert_eq!(
                    listed
                        .iter()
                        .map(|group| (group.key(), group.title(), group.scope()))
                        .collect::<Vec<_>>(),
                    identities
                );
                let visible: Vec<_> = rows
                    .iter()
                    .filter_map(|row| match row {
                        MidiListRow::Action { info } => Some(info.group),
                        MidiListRow::Set { infos, .. } => Some(infos[0].group),
                        _ => None,
                    })
                    .collect();
                assert!(!visible.is_empty());
                assert!(visible.iter().all(|visible| *visible == group));
                assert_eq!(drive.app.learned, saved);
                assert!(drive.app.screen_learn.is_none());
            }
        }
    }
}

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
    for bit in 17..32 {
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

/// Scenario "Keyboard and controller save feedback agree".
/// Scenario "Repeated manual refusals each explain the reason".
/// Scenario "Successful retry clears the save warning".
#[test]
fn keyboard_and_controller_save_feedback_agree() {
    use tui::SaveStatus;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/86b-input-saves-{}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    let _cleanup = Cleanup(root.clone());
    for project in [false, true] {
        for problem in ["none", "write", "damaged", "newer"] {
            let path = root.join("project.toml");
            let mut expected = None;
            for source in [Source::Keyboard, Source::Button] {
                let mut drive = Drive::new();
                drive.bind_action(88, "session.save");
                drive.bind_action(89, "master.volume_up");
                drive.app.session_path = path.clone();
                drive.app.session_read_error = None;
                if project {
                    drive.app.project = Some(Project {
                        name: "test".into(),
                        folder: root.clone(),
                    });
                }
                session::save(&path, &session::Session::default()).unwrap();
                if matches!(problem, "damaged" | "newer") {
                    std::fs::write(
                        &path,
                        if problem == "damaged" {
                            "version = [\n"
                        } else {
                            "version = 99\n"
                        },
                    )
                    .unwrap();
                    drive.app.session_read_error = Some(session::load(&path).unwrap_err());
                }
                let tmp = path.with_extension("toml.tmp");
                if problem == "write" {
                    std::fs::write(&tmp, "occupied").unwrap();
                }
                let saved = std::fs::read(&path).unwrap();
                assert_eq!(drive.app.save_status(), SaveStatus::Clean);
                drive.registry_command(source, KeyCode::Char(']'), M::NONE, 89);
                assert_eq!(drive.app.save_status(), SaveStatus::Unsaved);
                let pending = drive.app.session_dirty;
                let before = drive.seen();
                for _ in 0..2 {
                    drive.app.flash = None;
                    drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 88);
                    let (notice, at) = drive.app.flash.as_ref().unwrap().clone();
                    assert_eq!(notice == "Session saved", problem == "none");
                    assert_eq!(
                        drive.app.notice_at(at + FLASH - Duration::from_nanos(1)),
                        Some(notice.clone())
                    );
                    assert_eq!(drive.app.notice_at(at + FLASH), None);
                    assert_eq!(
                        drive.app.session_dirty,
                        if problem == "none" { None } else { pending }
                    );
                    assert_eq!(drive.app.session_write_failed, problem == "write");
                    assert_eq!(
                        drive.app.save_status(),
                        match problem {
                            "none" => SaveStatus::Clean,
                            "write" => SaveStatus::Failed,
                            _ => SaveStatus::Unsaved,
                        }
                    );
                    assert_eq!(drive.app.screen, before.screen);
                    assert_eq!(drive.app.history.len(), before.history.len());
                    let bytes = std::fs::read(&path).unwrap();
                    if problem != "none" {
                        assert_eq!(bytes, saved);
                    } else {
                        assert_eq!(session::load(&path).unwrap().master_db, drive.app.master_db);
                    }
                    let result = (notice, bytes, drive.app.save_status());
                    if let Some(expected) = &expected {
                        assert_eq!(
                            &result, expected,
                            "{source:?}, {problem}, project {project}"
                        );
                    } else {
                        expected = Some(result);
                    }
                    drive.app.flash = Some(("another notice".into(), at));
                    assert_eq!(drive.app.notice_at(at), Some("another notice".into()));
                }
                if problem == "write" {
                    std::fs::remove_file(&tmp).unwrap();
                    drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 88);
                    assert_eq!(drive.app.flash.as_ref().unwrap().0, "Session saved");
                    assert_eq!(drive.app.save_status(), SaveStatus::Clean);
                    drive.registry_command(source, KeyCode::Char(']'), M::NONE, 89);
                    assert_eq!(drive.app.save_status(), SaveStatus::Unsaved);
                    drive.app.flash = None;
                    drive.app.autosave(
                        drive.app.session_dirty.unwrap() + SESSION_SAVE + Duration::from_nanos(1),
                    );
                    assert_eq!(drive.app.save_status(), SaveStatus::Clean);
                    assert!(drive.app.flash.is_none());
                    assert_eq!(session::load(&path).unwrap().master_db, drive.app.master_db);
                }
            }
        }
    }
}

/// Scenario "The save action appears in GLOBAL".
/// Scenario "Save learning waits for confirmation".
/// Scenario "The learned save button survives restart".
/// Scenario "The learned save button retries after write refusal".
#[test]
fn the_learned_save_button_survives_restart() {
    use input::{Group, midi::Key};
    use tui::SaveStatus;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/86b-learn-save-{}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    let _cleanup = Cleanup(root.clone());
    for source in [Source::Notes, Source::Button] {
        for project in [false, true] {
            let dir = root.join(format!("{source:?}-{project}"));
            std::fs::create_dir(&dir).unwrap();
            let config_path = dir.join("config.toml");
            let session_path = dir.join(if project {
                "project.toml"
            } else {
                "session.toml"
            });
            std::fs::write(&config_path, "").unwrap();
            session::save(&session_path, &session::Session::default()).unwrap();
            let key = match source {
                Source::Notes => Key::Note(88),
                Source::Button => Key::Cc(88),
                Source::Keyboard => unreachable!(),
            };
            let configure_session = |drive: &mut Drive| {
                drive.app.session_path = session_path.clone();
                drive.app.session_read_error = None;
                if project {
                    drive.app.project = Some(Project {
                        name: "learn save".into(),
                        folder: dir.clone(),
                    });
                }
            };
            let expected_bindings = {
                let mut drive = Drive::with_saved_bindings(false, Some(&config_path));
                configure_session(&mut drive);
                drive.tap(KeyCode::Char(']'), M::NONE);
                let pending_work = drive.app.session_dirty;
                drive.tap(KeyCode::F(12), M::NONE);
                for _ in 0..2 {
                    drive.tap(KeyCode::Tab, M::NONE);
                }
                assert_eq!(drive.app.settings_tab, 2);
                let global = drive
                    .app
                    .midi_list()
                    .iter()
                    .position(|row| {
                        matches!(
                            row,
                            MidiListRow::Group {
                                group: Group::Global
                            }
                        )
                    })
                    .unwrap();
                for _ in 0..global {
                    drive.tap(KeyCode::Down, M::NONE);
                }
                assert_eq!(drive.app.midi_row, global);
                drive.tap(KeyCode::Enter, M::NONE);
                assert!(drive.app.midi_open.contains(&Group::Global));
                let rows = drive.app.midi_list();
                let saves: Vec<_> = rows
                    .iter()
                    .enumerate()
                    .filter_map(|(at, row)| match row {
                        MidiListRow::Action { info } if info.name == "session.save" => {
                            assert_eq!(info.group, Group::Global);
                            assert_eq!(info.label, "save session");
                            assert!(!info.knob);
                            Some(at)
                        }
                        _ => None,
                    })
                    .collect();
                assert_eq!(saves.len(), 1);
                for _ in global..saves[0] {
                    drive.tap(KeyCode::Down, M::NONE);
                }
                assert_eq!(drive.app.midi_row, saves[0]);
                assert_eq!(
                    drive
                        .app
                        .midi_screen()
                        .iter()
                        .filter(|row| matches!(row,
                            tui::MidiRow::Action { label, .. } if label == "save session"
                        ))
                        .count(),
                    1
                );
                for confirm in [false, true] {
                    let old_bindings = drive.app.learned.clone();
                    let config_bytes = std::fs::read(&config_path).unwrap();
                    let session_bytes = std::fs::read(&session_path).unwrap();
                    drive.tap(KeyCode::Enter, M::NONE);
                    let learning = drive.app.learning.as_ref().unwrap();
                    assert_eq!(learning.info.name, "session.save");
                    assert!(learning.pending.is_none());
                    drive.bytes(
                        PAD_PORT,
                        &match source {
                            Source::Notes => [0x80, 88, 0],
                            Source::Button => [0xb0, 88, 0],
                            Source::Keyboard => unreachable!(),
                        },
                    );
                    drive.frame();
                    assert!(drive.app.learning.as_ref().unwrap().pending.is_none());
                    let frame = drive.registry_command(source, KeyCode::Null, M::NONE, 88);
                    assert!(frame.commands().is_empty());
                    assert_eq!(
                        drive.app.learning.as_ref().unwrap().pending,
                        Some(("Recorded pads".into(), key))
                    );
                    assert!(drive.app.midi_screen().iter().any(|row| matches!(row,
                        tui::MidiRow::Action { label, who, .. }
                            if label == "save session" && who.contains('→')
                    )));
                    assert_eq!(drive.app.learned, old_bindings);
                    assert_eq!(std::fs::read(&config_path).unwrap(), config_bytes);
                    assert_eq!(std::fs::read(&session_path).unwrap(), session_bytes);
                    assert_eq!(drive.app.session_dirty, pending_work);
                    drive.tap(
                        if confirm {
                            KeyCode::Enter
                        } else {
                            KeyCode::Esc
                        },
                        M::NONE,
                    );
                    assert!(drive.app.learning.is_none());
                    if confirm {
                        assert_eq!(
                            drive.app.learned["Recorded pads"]["global"][&key.to_string()],
                            "session.save"
                        );
                        assert_eq!(
                            config::load_from(&config_path).config.midi,
                            drive.app.learned
                        );
                    } else {
                        assert_eq!(drive.app.learned, old_bindings);
                        assert_eq!(std::fs::read(&config_path).unwrap(), config_bytes);
                    }
                    assert_eq!(std::fs::read(&session_path).unwrap(), session_bytes);
                    assert_eq!(drive.app.session_dirty, pending_work);
                }
                drive.app.learned.clone()
            };
            let mut drive = Drive::with_saved_bindings(false, Some(&config_path));
            configure_session(&mut drive);
            assert_eq!(drive.app.learned, expected_bindings);
            let config_bytes = std::fs::read(&config_path).unwrap();
            drive.load_pad(0, Arc::new(Sample::mono(vec![0.25; 1024])), "learn-save");
            drive.app.browser.dir = dir.clone();
            for (screen, code, _) in REGISTRY_SCREEN_COMMANDS {
                drive.tap(KeyCode::F(2), M::NONE);
                drive.tap(code, M::NONE);
                assert_eq!(drive.app.screen, screen);
                drive.tap(KeyCode::Char(']'), M::NONE);
                drive.registry_command(source, KeyCode::Null, M::NONE, 88);
                assert_eq!(drive.app.flash.as_ref().unwrap().0, "Session saved");
                assert_eq!(drive.app.save_status(), SaveStatus::Clean);
                assert_eq!(
                    session::load(&session_path).unwrap().master_db,
                    drive.app.master_db
                );
                assert_eq!(drive.app.screen, screen);
            }
            drive.tap(KeyCode::F(2), M::NONE);
            drive.tap(KeyCode::Char(']'), M::NONE);
            let pending_work = drive.app.session_dirty;
            let saved = std::fs::read(&session_path).unwrap();
            let tmp = session_path.with_extension("toml.tmp");
            std::fs::write(&tmp, "occupied").unwrap();
            for _ in 0..2 {
                drive.app.flash = None;
                drive.registry_command(source, KeyCode::Null, M::NONE, 88);
                assert_ne!(drive.app.flash.as_ref().unwrap().0, "Session saved");
                assert_eq!(drive.app.save_status(), SaveStatus::Failed);
                assert_eq!(drive.app.session_dirty, pending_work);
                assert_eq!(std::fs::read(&session_path).unwrap(), saved);
            }
            std::fs::remove_file(&tmp).unwrap();
            drive.registry_command(source, KeyCode::Null, M::NONE, 88);
            assert_eq!(drive.app.flash.as_ref().unwrap().0, "Session saved");
            assert_eq!(drive.app.save_status(), SaveStatus::Clean);
            drive.tap(KeyCode::Char(']'), M::NONE);
            drive.app.flash = None;
            drive.app.autosave(
                drive.app.session_dirty.unwrap() + SESSION_SAVE + Duration::from_nanos(1),
            );
            assert_eq!(drive.app.save_status(), SaveStatus::Clean);
            assert!(drive.app.flash.is_none());
            assert_eq!(
                session::load(&session_path).unwrap().master_db,
                drive.app.master_db
            );
            assert_eq!(std::fs::read(&config_path).unwrap(), config_bytes);
        }
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

/// Scenario "Project edits survive an immediate switch".
/// Scenario "Scratch edits save before opening or creating a project".
/// Scenario "Keyboard and MIDI project entry share transition safety".
#[test]
fn project_edits_survive_an_immediate_switch() {
    for source in [Source::Keyboard, Source::Button] {
        for project in [false, true] {
            for create in [false, true] {
                let files = TransitionFiles::new();
                let mut drive = files.drive(project);
                TransitionFiles::edit(&mut drive, source);
                let source_file = drive.app.session_file();
                let expected_gate = drive.app.pads.gate[0];
                let expected_chop = drive.app.chop.as_ref().unwrap().saved();
                let target = if create { "C" } else { "B" };
                TransitionFiles::pick(&mut drive, source, target, create);
                drive.tap(KeyCode::Enter, M::NONE);
                assert!(drive.app.project_picker.is_none());
                assert_eq!(drive.app.project.as_ref().unwrap().name, target);
                assert!(drive.app.session_dirty.is_none());
                assert!(drive.app.history.is_empty());
                let saved = session::load_existing(&source_file).unwrap();
                assert_eq!(saved.pads["0"].gate, expected_gate);
                assert_eq!(
                    saved.chops.values().next().unwrap(),
                    &expected_chop.to_state(drive.app.rate)
                );
                if project {
                    TransitionFiles::pick(&mut drive, source, "A", false);
                    drive.tap(KeyCode::Enter, M::NONE);
                    assert_eq!(drive.app.project.as_ref().unwrap().name, "A");
                    assert_eq!(drive.app.pads.gate[0], expected_gate);
                    assert_eq!(
                        drive.app.pending_chops[&files.0.join("tone.wav")],
                        expected_chop
                    );
                }
            }
        }
    }
}

/// Scenario "A save refusal keeps the selected target".
/// Scenario "A save refusal keeps the new project name".
/// Scenario "Pending protected work blocks a transition".
/// Scenario "A refused transition explains its reason".
#[test]
fn a_save_refusal_keeps_the_selected_target() {
    for source in [Source::Keyboard, Source::Button] {
        for project in [false, true] {
            for create in [false, true] {
                for protected in [false, true] {
                    let files = TransitionFiles::new();
                    let mut drive = files.drive(project);
                    TransitionFiles::edit(&mut drive, source);
                    let source_file = drive.app.session_file();
                    let tmp = source_file.with_extension("toml.tmp");
                    if protected {
                        std::fs::write(&source_file, "version = 99\n").unwrap();
                        drive.app.session_read_error =
                            Some(session::load(&source_file).unwrap_err());
                    } else {
                        std::fs::create_dir(&tmp).unwrap();
                    }
                    let reason = drive.app.session_read_error.clone();
                    let bytes = std::fs::read(&source_file).unwrap();
                    let config_bytes =
                        std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap();
                    let target_bytes = std::fs::read(files.0.join("B/project.toml")).unwrap();
                    let pending = drive.app.session_dirty;
                    let expected_gate = drive.app.pads.gate[0];
                    let generations = drive.app.load_gen.clone();
                    let cuts = drive.app.chop.as_ref().unwrap().saved();
                    let target = if create { "C" } else { "B" };
                    TransitionFiles::pick(&mut drive, source, target, create);
                    let before = TransitionFiles::state(&drive);
                    let picker = TransitionFiles::picker(&drive);
                    for _ in 0..2 {
                        drive.app.flash = None;
                        drive.tap(KeyCode::Enter, M::NONE);
                        assert_eq!(TransitionFiles::state(&drive), before);
                        assert_eq!(TransitionFiles::picker(&drive), picker);
                        assert_eq!(drive.app.chop.as_ref().unwrap().saved(), cuts);
                        assert_eq!(drive.app.load_gen, generations);
                        assert_eq!(drive.app.session_dirty, pending);
                        assert_eq!(drive.app.session_read_error, reason);
                        assert_eq!(drive.app.session_write_failed, !protected);
                        assert_eq!(
                            drive.app.project.as_ref().map(|p| p.name.as_str()),
                            project.then_some("A")
                        );
                        let notice = &drive.app.flash.as_ref().unwrap().0;
                        assert_ne!(notice, "Session saved");
                        assert!(!notice.starts_with("Project "));
                        assert!(drive.app.ask.is_none());
                        assert_eq!(std::fs::read(&source_file).unwrap(), bytes);
                        assert_eq!(
                            std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap(),
                            config_bytes
                        );
                        assert_eq!(
                            std::fs::read(files.0.join("B/project.toml")).unwrap(),
                            target_bytes
                        );
                        assert!(!files.0.join("C").exists());
                    }
                    if !protected {
                        std::fs::remove_dir(&tmp).unwrap();
                        drive.tap(KeyCode::Enter, M::NONE);
                        assert!(drive.app.project_picker.is_none());
                        assert_eq!(drive.app.project.as_ref().unwrap().name, target);
                        let saved = session::load_existing(&source_file).unwrap();
                        assert_eq!(saved.pads["0"].gate, expected_gate);
                        assert_eq!(
                            saved.chops.values().next().unwrap(),
                            &cuts.to_state(drive.app.rate)
                        );
                    }
                }
            }
        }
    }
}

/// Scenario "A target read refusal preserves the source".
#[test]
fn a_target_read_refusal_preserves_the_source() {
    for source in [Source::Keyboard, Source::Button] {
        for project in [false, true] {
            for fault in ["damaged", "newer", "unreadable", "missing"] {
                let files = TransitionFiles::new();
                let mut drive = files.drive(project);
                TransitionFiles::edit(&mut drive, source);
                TransitionFiles::pick(&mut drive, source, "B", false);
                let path = files.0.join("B/project.toml");
                match fault {
                    "damaged" => std::fs::write(&path, "version = [\n").unwrap(),
                    "newer" => std::fs::write(&path, "version = 99\n").unwrap(),
                    _ => {
                        std::fs::remove_file(&path).unwrap();
                        if fault == "unreadable" {
                            std::fs::create_dir(&path).unwrap();
                            std::fs::write(path.join("keep"), "unrelated").unwrap();
                        }
                    }
                }
                let target_bytes = std::fs::read(&path).ok();
                // A target refusal must happen before a source write attempt.
                let source_file = drive.app.session_file();
                std::fs::create_dir(source_file.with_extension("toml.tmp")).unwrap();
                drive.app.session_read_error = Some("source remains protected".into());
                let reason = drive.app.session_read_error.clone();
                let bytes = std::fs::read(&source_file).unwrap();
                let config_bytes = std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap();
                let pending = drive.app.session_dirty;
                let before = TransitionFiles::state(&drive);
                let picker = TransitionFiles::picker(&drive);
                let cuts = drive.app.chop.as_ref().unwrap().saved();
                let generations = drive.app.load_gen.clone();
                for _ in 0..2 {
                    drive.app.flash = None;
                    drive.tap(KeyCode::Enter, M::NONE);
                    assert!(
                        drive
                            .app
                            .flash
                            .as_ref()
                            .unwrap()
                            .0
                            .starts_with("Project not opened:")
                    );
                    assert_eq!(TransitionFiles::state(&drive), before);
                    assert_eq!(TransitionFiles::picker(&drive), picker);
                    assert_eq!(drive.app.chop.as_ref().unwrap().saved(), cuts);
                    assert_eq!(drive.app.load_gen, generations);
                    assert_eq!(drive.app.session_dirty, pending);
                    assert_eq!(drive.app.session_read_error, reason);
                    assert!(!drive.app.session_write_failed);
                    assert_eq!(
                        drive.app.project_cfg.last.as_deref(),
                        project.then_some("A")
                    );
                    assert_eq!(std::fs::read(&source_file).unwrap(), bytes);
                    assert_eq!(
                        std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap(),
                        config_bytes
                    );
                    assert_eq!(std::fs::read(&path).ok(), target_bytes);
                    assert_eq!(path.exists(), fault != "missing");
                    if fault == "unreadable" {
                        assert_eq!(
                            std::fs::read_to_string(path.join("keep")).unwrap(),
                            "unrelated"
                        );
                    }
                }
            }
        }
    }
}

/// Scenario "Selecting the current project keeps pending work".
#[test]
fn selecting_the_current_project_keeps_pending_work() {
    use std::os::unix::fs::symlink;
    use std::sync::atomic::Ordering::Acquire;
    for source in [Source::Keyboard, Source::Button] {
        for alias in [false, true] {
            let files = TransitionFiles::new();
            let mut drive = files.drive(true);
            TransitionFiles::edit(&mut drive, source);
            let path = drive.app.session_file();
            std::fs::create_dir(path.with_extension("toml.tmp")).unwrap();
            drive.tap(KeyCode::Char('s'), M::CONTROL);
            assert!(drive.app.session_write_failed);
            std::fs::write(&path, "version = [\n").unwrap();
            let (audio, _rt) = audio::offline(48_000);
            let epoch = audio.project.epoch.load(Acquire);
            drive.app.audio = Some(audio);
            let target = if alias {
                symlink(files.0.join("A"), files.0.join("alias")).unwrap();
                "alias"
            } else {
                "A"
            };
            TransitionFiles::pick(&mut drive, source, target, false);
            let before = TransitionFiles::state(&drive);
            let pending = drive.app.session_dirty;
            let generations = drive.app.load_gen.clone();
            let cuts = drive.app.chop.as_ref().unwrap().saved();
            let sound = drive.app.pads.sound[0].as_ref().unwrap().clone();
            let config_bytes = std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap();
            let frame = drive.tap(KeyCode::Enter, M::NONE);
            assert!(frame.commands().is_empty());
            assert!(drive.app.project_picker.is_none());
            assert_eq!(TransitionFiles::state(&drive), before);
            assert_eq!(drive.app.session_dirty, pending);
            assert!(drive.app.session_write_failed);
            assert_eq!(drive.app.load_gen, generations);
            assert_eq!(drive.app.chop.as_ref().unwrap().saved(), cuts);
            assert!(Arc::ptr_eq(
                drive.app.pads.sound[0].as_ref().unwrap(),
                &sound
            ));
            assert_eq!(
                drive
                    .app
                    .audio
                    .as_ref()
                    .unwrap()
                    .project
                    .epoch
                    .load(Acquire),
                epoch
            );
            assert_eq!(drive.app.project.as_ref().unwrap().name, "A");
            assert_eq!(std::fs::read(&path).unwrap(), b"version = [\n");
            assert_eq!(
                std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap(),
                config_bytes
            );
        }
    }
}

/// Scenario "A target creation refusal retains saved source work".
#[test]
fn a_target_creation_refusal_retains_saved_source_work() {
    for source in [Source::Keyboard, Source::Button] {
        for project in [false, true] {
            let files = TransitionFiles::new();
            let mut drive = files.drive(project);
            TransitionFiles::edit(&mut drive, source);
            let path = drive.app.session_file();
            std::fs::write(files.0.join("C"), "occupied").unwrap();
            TransitionFiles::pick(&mut drive, source, "C", true);
            let before = TransitionFiles::state(&drive);
            let picker = TransitionFiles::picker(&drive);
            let expected_gate = drive.app.pads.gate[0];
            let config_bytes = std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap();
            drive.tap(KeyCode::Enter, M::NONE);
            assert_eq!(TransitionFiles::state(&drive), before);
            assert_eq!(TransitionFiles::picker(&drive), picker);
            assert!(drive.app.session_dirty.is_none());
            assert!(!drive.app.session_write_failed);
            assert_eq!(
                drive.app.project_cfg.last.as_deref(),
                project.then_some("A")
            );
            assert_eq!(
                session::load_existing(&path).unwrap().pads["0"].gate,
                expected_gate
            );
            assert_eq!(
                std::fs::read_to_string(files.0.join("C")).unwrap(),
                "occupied"
            );
            assert_eq!(
                std::fs::read(drive.app.user_path.as_ref().unwrap()).unwrap(),
                config_bytes
            );
            std::fs::remove_file(files.0.join("C")).unwrap();
            drive.tap(KeyCode::Enter, M::NONE);
            assert!(drive.app.project_picker.is_none());
            assert_eq!(drive.app.project.as_ref().unwrap().name, "C");
        }
    }
}

#[test]
fn a_missing_source_folder_blocks_switching_until_restored() {
    for source in [Source::Keyboard, Source::Button] {
        for create in [false, true] {
            let files = TransitionFiles::new();
            let mut drive = files.drive(true);
            TransitionFiles::edit(&mut drive, source);
            let folder = files.0.join("A");
            let moved = files.0.join("moved");
            std::fs::rename(&folder, &moved).unwrap();
            let saved = std::fs::read(moved.join("project.toml")).unwrap();
            let target = if create { "C" } else { "B" };
            TransitionFiles::pick(&mut drive, source, target, create);
            let before = TransitionFiles::state(&drive);
            let picker = TransitionFiles::picker(&drive);
            let pending = drive.app.session_dirty;
            for _ in 0..2 {
                drive.tap(KeyCode::Enter, M::NONE);
                assert!(drive.app.flash.as_ref().unwrap().0.contains("is gone"));
                assert_eq!(TransitionFiles::state(&drive), before);
                assert_eq!(TransitionFiles::picker(&drive), picker);
                assert_eq!(drive.app.session_dirty, pending);
                assert!(drive.app.session_write_failed);
                assert!(!folder.exists());
                assert!(!files.0.join("C").exists());
                assert_eq!(std::fs::read(moved.join("project.toml")).unwrap(), saved);
            }
            std::fs::rename(&moved, &folder).unwrap();
            drive.tap(KeyCode::Enter, M::NONE);
            assert!(drive.app.project_picker.is_none());
            assert_eq!(drive.app.project.as_ref().unwrap().name, target);
            assert!(drive.app.session_dirty.is_none());
            assert!(!drive.app.session_write_failed);
        }
    }
}

/// Scenario "Clean protected scratch can leave without a write".
#[test]
fn clean_protected_scratch_can_leave_without_a_write() {
    for source in [Source::Keyboard, Source::Button] {
        let files = TransitionFiles::new();
        let mut drive = files.drive(false);
        let path = drive.app.session_file();
        std::fs::write(&path, "version = 99\n").unwrap();
        drive.app.session_read_error = Some(session::load(&path).unwrap_err());
        assert!(drive.app.session_dirty.is_none());
        TransitionFiles::pick(&mut drive, source, "B", false);
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.project_picker.is_none());
        assert_eq!(drive.app.project.as_ref().unwrap().name, "B");
        assert!(drive.app.session_read_error.is_none());
        assert_eq!(std::fs::read(&path).unwrap(), b"version = 99\n");
    }
}

struct PortableProject {
    root: PathBuf,
    file: PathBuf,
    internal: bool,
    states: Vec<session::ChopState>,
    pads: std::collections::BTreeMap<String, session::PadState>,
}

impl PortableProject {
    fn new(internal: bool) -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/86d-portable")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Relaxed)
            ));
        for name in ["A", "B"] {
            std::fs::create_dir_all(root.join(name).join("samples")).unwrap();
        }
        let file = if internal {
            root.join("A/samples/tone.wav")
        } else {
            root.join("tone.wav")
        };
        let wave: Vec<_> = (0..48_000)
            .map(|i| 0.12 + 0.07 * (i as f32 * 0.03).sin())
            .collect();
        engine::offline::write_wav(&file, RENDER_RATE, &wave, &wave).unwrap();
        let states: Vec<_> = [
            (3, vec![0.125, 0.375, 0.625, 0.875], vec![4, 8, 12, 16]),
            (7, vec![0.2, 0.4, 0.6, 0.8], vec![5, 9, 13, 17]),
            (11, vec![0.25, 0.5, 0.75], vec![6, 10, 14]),
        ]
        .into_iter()
        .map(|(epoch, cuts, ids)| session::ChopState {
            start: 0.0,
            end: 1.0,
            cuts,
            ids,
            epoch,
            next_id: 99,
        })
        .collect();
        let pads = [
            (0.125, 0.375, 4, 3, 2, -6, 3),
            (0.2, 0.4, 5, 7, 2, -3, -5),
            (0.5, 0.75, 10, 11, 3, 0, 7),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (start, end, link, epoch, slice, volume_db, pitch))| {
            (
                i.to_string(),
                session::PadState {
                    file: Some(file.clone()),
                    name: Some(format!("variant {}", i + 1)),
                    start: Some(start),
                    end: Some(end),
                    link: Some(link),
                    epoch: Some(epoch),
                    slice: Some(slice),
                    gate: false,
                    volume_db,
                    pitch,
                    release_ms: 10,
                    filter: "low-pass".into(),
                    cutoff_hz: Some(2_000.0),
                    reso: 10,
                    drive: 15,
                    ..Default::default()
                },
            )
        })
        .collect();
        let sess = session::Session {
            browser_dir: Some(root.join("A/samples")),
            pads,
            chops: [(file.to_string_lossy().into_owned(), states[2].clone())].into(),
            kept: [(file.to_string_lossy().into_owned(), states[..2].to_vec())].into(),
            ..Default::default()
        };
        session::save_project(&root.join("A/project.toml"), &sess).unwrap();
        session::save_project(&root.join("B/project.toml"), &session::Session::default()).unwrap();
        Self {
            root,
            file,
            internal,
            states,
            pads: sess.pads,
        }
    }

    fn drive(&self, source: Source) -> Drive {
        let mut drive = Drive::offline();
        for (cc, action) in [
            (88, "project.pick"),
            (89, "session.save"),
            (90, "screen.chop"),
            (91, "slice.start_right"),
        ] {
            drive.bind_action(cc, action);
        }
        drive.app.paths.projects = self.root.clone();
        drive.app.session_path = self.root.join("scratch.toml");
        drive.app.user_path = Some(self.root.join("config.toml"));
        drive.app.session_read_error = None;
        Self::open(&mut drive, source, "A");
        drive
    }

    fn open(drive: &mut Drive, source: Source, name: &str) {
        TransitionFiles::pick(drive, source, name, false);
        drive.tap(KeyCode::Enter, M::NONE);
        assert!(drive.app.project_picker.is_none());
        assert_eq!(drive.app.project.as_ref().unwrap().name, name);
        while drive.app.pads.loading.iter().any(Option::is_some) {
            let reply = drive
                .app
                .loader
                .as_ref()
                .unwrap()
                .1
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            drive.app.loader_reply(reply);
            drive.app.flush_audio();
            drive.render_blocks(1);
        }
        let epoch = drive
            .project
            .epoch
            .load(std::sync::atomic::Ordering::Acquire);
        // The production switch clears FX buffers in bounded steps. As in the
        // realtime switch tests, advance callbacks until the actual ready marker.
        for _ in 0..300 {
            drive.app.flush_audio();
            drive.render_blocks(1);
            if drive.app.pending_audio.is_empty()
                && drive
                    .project
                    .ready
                    .load(std::sync::atomic::Ordering::Acquire)
                    == epoch
            {
                break;
            }
        }
        assert!(drive.app.pending_audio.is_empty());
        assert_eq!(
            drive
                .project
                .ready
                .load(std::sync::atomic::Ordering::Acquire),
            epoch
        );
        drive.frame();
        drive.render_blocks(32);
    }

    fn f3(drive: &mut Drive, source: Source, pad: usize) {
        drive.tap(KeyCode::F(2), M::NONE);
        drive.pad_press(source, pad);
        drive.pad_release(source, pad);
        drive.registry_command(source, KeyCode::F(3), M::NONE, 90);
        assert_eq!(drive.app.screen, Screen::Chop);
    }

    fn check_variants(&self, drive: &mut Drive, source: Source, file: &Path) {
        for (pad, expected) in self.states.iter().enumerate() {
            Self::f3(drive, source, pad);
            let chop = drive.app.chop.as_ref().unwrap();
            assert_eq!(chop.file, file);
            assert_eq!(chop.saved().to_state(RENDER_RATE), *expected);
            drive.render_blocks(32);
        }
        let pads = drive.app.pads.to_session(None, RENDER_RATE).pads;
        for (key, expected) in &self.pads {
            let mut expected = expected.clone();
            expected.file = Some(file.to_path_buf());
            assert_eq!(pads[key], expected);
        }
    }

    fn save(drive: &mut Drive, source: Source) {
        drive.registry_command(source, KeyCode::Char('s'), M::CONTROL, 89);
        assert_eq!(drive.app.flash.as_ref().unwrap().0, "Session saved");
        assert!(drive.app.session_dirty.is_none());
    }

    fn check_stored(&self, name: &str) {
        let sess = session::load_existing(&self.root.join(name).join("project.toml")).unwrap();
        let stored = if self.internal {
            Path::new("samples/tone.wav")
        } else {
            &self.file
        };
        let key = stored.to_string_lossy();
        assert_eq!(sess.version, 3);
        assert_eq!(sess.chops.len(), 1);
        assert_eq!(sess.kept.len(), 1);
        assert_eq!(sess.chops[key.as_ref()], self.states[2]);
        assert_eq!(sess.kept[key.as_ref()], self.states[..2]);
        assert!(
            sess.pads
                .values()
                .all(|pad| pad.file.as_deref() == Some(stored))
        );
        assert_eq!(
            std::fs::read_dir(self.root.join(name).join("samples"))
                .unwrap()
                .count(),
            usize::from(self.internal)
        );
    }

    fn render(drive: &mut Drive, source: Source) -> Stereo {
        use engine::checks::{first_loud, first_non_finite, peak};
        let mut all = (Vec::new(), Vec::new());
        drive.tap(KeyCode::F(2), M::NONE);
        drive.render_blocks(32);
        for pad in 0..3 {
            let mut wave = drive.render_blocks(16);
            drive.pad_press(source, pad);
            append_stereo(&mut wave, drive.render_blocks(128));
            drive.pad_release(source, pad);
            for channel in [&wave.0, &wave.1] {
                assert_eq!(first_non_finite(channel), None);
                assert!(peak(channel) < 1.0);
                assert_eq!(peak(&channel[..2048]), 0.0);
                let onset = first_loud(channel, 2048..2176, 1e-6).unwrap();
                assert!(
                    onset.abs_diff(2048) <= 1,
                    "{source:?}, pad {pad}, onset {onset}"
                );
                assert!(peak(&channel[2048..]) > 0.01);
                assert_eq!(peak(&channel[channel.len() - 1024..]), 0.0);
            }
            append_stereo(&mut all, wave);
        }
        all
    }
}

impl Drop for PortableProject {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

/// Scenario "Internal variant paths move with the project".
/// Scenario "External variant paths remain absolute".
/// Scenario "Save and reopen after project relocation".
/// Scenario "Moving keeps unassigned cuts of older chops".
/// Scenario "Editing a moved chop changes only its linked pads".
/// Scenario "Moved slices keep their sound".
#[test]
fn portable_chops_keep_all_variants_and_sound() {
    let mut all = (Vec::new(), Vec::new());
    for source in [Source::Keyboard, Source::Button] {
        for internal in [true, false] {
            let files = PortableProject::new(internal);
            let bytes = std::fs::read(&files.file).unwrap();
            let mut drive = files.drive(source);
            files.check_variants(&mut drive, source, &files.file);
            let before = PortableProject::render(&mut drive, source);
            PortableProject::save(&mut drive, source);
            files.check_stored("A");
            std::fs::rename(files.root.join("A"), files.root.join("Moved")).unwrap();
            let moved_file = if internal {
                files.root.join("Moved/samples/tone.wav")
            } else {
                files.file.clone()
            };
            assert!(!files.root.join("A").exists());
            PortableProject::open(&mut drive, source, "Moved");
            files.check_variants(&mut drive, source, &moved_file);
            let after = PortableProject::render(&mut drive, source);
            assert_eq!(after, before, "{source:?}, internal {internal}");
            assert_eq!(std::fs::read(&moved_file).unwrap(), bytes);
            append_stereo(&mut all, before);
            append_stereo(&mut all, after);
            PortableProject::save(&mut drive, source);
            files.check_stored("Moved");
            PortableProject::open(&mut drive, source, "B");
            PortableProject::open(&mut drive, source, "Moved");
            files.check_variants(&mut drive, source, &moved_file);
            PortableProject::f3(&mut drive, source, 0);
            let other_pads = [drive.app.pads.snapshot(1), drive.app.pads.snapshot(2)];
            let original = drive.app.pads.region[0];
            drive.registry_command(source, KeyCode::Char('l'), M::CONTROL, 91);
            assert_ne!(drive.app.pads.region[0], original);
            assert_eq!(
                [drive.app.pads.snapshot(1), drive.app.pads.snapshot(2)],
                other_pads
            );
            let edited = drive.app.chop.as_ref().unwrap().saved();
            for pad in [1, 2] {
                PortableProject::f3(&mut drive, source, pad);
                assert_eq!(
                    drive
                        .app
                        .chop
                        .as_ref()
                        .unwrap()
                        .saved()
                        .to_state(RENDER_RATE),
                    files.states[pad]
                );
            }
            PortableProject::f3(&mut drive, source, 0);
            assert_eq!(drive.app.chop.as_ref().unwrap().saved(), edited);
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/86d-portable-chops.wav");
    engine::offline::write_wav(&path, RENDER_RATE, &all.0, &all.1).unwrap();
    println!(
        "86d render: {} frames, before/after sample-identical, finite/no-clip/silence/onset +/-1 checked",
        all.0.len()
    );
}

/// Scenario "Missing kept variants use the existing fallback".
/// Scenario "Stale absolute variant paths are not guessed".
#[test]
fn portable_chops_keep_old_project_compatibility() {
    for source in [Source::Keyboard, Source::Button] {
        for stale in [false, true] {
            let files = PortableProject::new(true);
            let path = files.root.join("A/project.toml");
            let mut sess = session::load_existing(&path).unwrap();
            sess.kept.clear();
            let absent = files.root.join("already-gone/tone.wav");
            if stale {
                sess.kept.insert(
                    absent.to_string_lossy().into_owned(),
                    files.states[..2].to_vec(),
                );
            }
            session::save_project(&path, &sess).unwrap();
            let mut drive = files.drive(source);
            if stale {
                assert!(
                    drive
                        .app
                        .pending_older
                        .iter()
                        .all(|(file, _)| file == &absent)
                );
                assert_eq!(drive.app.pending_older.len(), 2);
            } else {
                assert!(drive.app.pending_older.is_empty());
            }
            PortableProject::f3(&mut drive, source, 0);
            let saved = drive.app.chop.as_ref().unwrap().saved();
            assert_eq!(saved.epoch, 3);
            let cuts: Vec<_> = saved.cuts.iter().map(|(_, at)| *at).collect();
            assert_eq!(cuts, [6000, 18000]);
            assert_eq!(drive.app.pads.link[0], Some((4, 3)));
            assert_eq!(drive.app.pads.pitch[0], 3);
            assert_eq!(drive.app.pads.volume_db[0], -6);
            assert_eq!(session::load_existing(&path).unwrap().version, 3);
        }
    }
}

fn pads_play_badge(drive: &mut Drive) -> bool {
    use ratatui::{Terminal, backend::TestBackend};
    let app = &mut drive.app;
    let view = pads_view(&mut app.pads, None, None, Vec::new(), None, app.rate);
    let screen = tui::Screen::Pads(app.mark_pads(view));
    let status = tui::Status {
        save: app.save_status(),
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

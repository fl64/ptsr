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

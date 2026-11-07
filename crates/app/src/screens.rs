//! Deterministic text catalogue. This module exists only in the test binary.

use super::*;
use ratatui::{
    Terminal, TerminalOptions, Viewport, backend::CrosstermBackend, buffer::Buffer, layout::Rect,
};
use std::collections::BTreeSet;

const SIZES: [(u16, u16); 2] = [(120, 40), (80, 26)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Normal,
    Tone,
    Envelope,
    Filter,
    Source,
    Sends,
    Bounds,
    Fx2,
    Mark,
    Marked,
    File,
    Copy,
    Swap,
    Placements,
    Assign,
    AssignOrder,
    Learn,
    MidiLearn,
    Question,
    QuestionFile,
    QuestionProfile,
    QuestionClear,
    QuestionCopy,
    QuestionPlan,
    Recording,
    Value,
    Info,
    Done,
    Warn,
    Error,
    Help,
    Messages,
    Unsaved,
    SaveFailed,
    Safety,
    Device,
    Hidden,
    Text,
    Picker,
    PickerName,
    InputTab,
    MidiTab,
    PathsTab,
    FolderPicker,
    Slices128,
    Loop,
    AsPad,
}

impl Mode {
    const ALL: &'static [Self] = &[
        Self::Normal,
        Self::Tone,
        Self::Envelope,
        Self::Filter,
        Self::Source,
        Self::Sends,
        Self::Bounds,
        Self::Fx2,
        Self::Mark,
        Self::Marked,
        Self::File,
        Self::Copy,
        Self::Swap,
        Self::Placements,
        Self::Assign,
        Self::AssignOrder,
        Self::Learn,
        Self::MidiLearn,
        Self::Question,
        Self::QuestionFile,
        Self::QuestionProfile,
        Self::QuestionClear,
        Self::QuestionCopy,
        Self::QuestionPlan,
        Self::Recording,
        Self::Value,
        Self::Info,
        Self::Done,
        Self::Warn,
        Self::Error,
        Self::Help,
        Self::Messages,
        Self::Unsaved,
        Self::SaveFailed,
        Self::Safety,
        Self::Device,
        Self::Hidden,
        Self::Text,
        Self::Picker,
        Self::PickerName,
        Self::InputTab,
        Self::MidiTab,
        Self::PathsTab,
        Self::FolderPicker,
        Self::Slices128,
        Self::Loop,
        Self::AsPad,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Tone => "edit-tone",
            Self::Envelope => "edit-env",
            Self::Filter => "edit-filter",
            Self::Source => "edit-source",
            Self::Sends => "edit-fx",
            Self::Bounds => "edit-bounds",
            Self::Fx2 => "fx-2",
            Self::Mark => "mark",
            Self::Marked => "marked",
            Self::File => "file-in-hand",
            Self::Copy => "copy",
            Self::Swap => "swap",
            Self::Placements => "placements",
            Self::Assign => "assign-hand",
            Self::AssignOrder => "assign-order",
            Self::Learn => "learn",
            Self::MidiLearn => "midi-learn",
            Self::Question => "question",
            Self::QuestionFile => "question-file",
            Self::QuestionProfile => "question-profile",
            Self::QuestionClear => "question-clear",
            Self::QuestionCopy => "question-copy",
            Self::QuestionPlan => "question-plan",
            Self::Recording => "recording",
            Self::Value => "value",
            Self::Info => "info",
            Self::Done => "done",
            Self::Warn => "warn",
            Self::Error => "error",
            Self::Help => "help",
            Self::Messages => "messages",
            Self::Unsaved => "unsaved",
            Self::SaveFailed => "save-failed",
            Self::Safety => "safety",
            Self::Device => "input-device",
            Self::Hidden => "hidden-grid",
            Self::Text => "text",
            Self::Picker => "project-picker",
            Self::PickerName => "project-name",
            Self::InputTab => "input-tab",
            Self::MidiTab => "midi-tab",
            Self::PathsTab => "paths-tab",
            Self::FolderPicker => "folder-picker",
            Self::Slices128 => "128-slices",
            Self::Loop => "loop",
            Self::AsPad => "as-pad",
        }
    }

    fn applies(self, screen: Screen) -> bool {
        match self {
            Self::Tone | Self::Envelope | Self::Filter | Self::Source | Self::Sends => {
                screen == Screen::Pads
            }
            Self::Bounds
            | Self::Assign
            | Self::AssignOrder
            | Self::Slices128
            | Self::Loop
            | Self::AsPad
            | Self::QuestionPlan => screen == Screen::Chop,
            Self::Fx2 => screen == Screen::Fx,
            Self::Learn => screen.rules().screen_learn,
            Self::MidiLearn
            | Self::InputTab
            | Self::MidiTab
            | Self::PathsTab
            | Self::FolderPicker
            | Self::QuestionProfile => screen == Screen::Settings,
            Self::Hidden => matches!(screen, Screen::Pads | Screen::Files | Screen::Chop),
            Self::Text => matches!(screen, Screen::Pads | Screen::Files),
            _ => true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Case {
    screen: Screen,
    mode: Mode,
    size: (u16, u16),
}

impl Case {
    fn name(self) -> String {
        format!(
            "{}-{}-{}x{}.ansi",
            self.screen.key(),
            self.mode.name(),
            self.size.0,
            self.size.1
        )
    }
}

fn matrix() -> Vec<Case> {
    Screen::ALL
        .into_iter()
        .flat_map(|screen| {
            Mode::ALL
                .iter()
                .copied()
                .filter(move |mode| mode.applies(screen))
                .flat_map(move |mode| SIZES.map(|size| Case { screen, mode, size }))
        })
        .collect()
}

fn validate(cases: &[Case]) -> Result<(), String> {
    // Required names are independent of the fixture enum and its ALL list.
    let mut expected = BTreeSet::new();
    for screen in Screen::ALL {
        let mut modes = vec![
            "normal",
            "mark",
            "marked",
            "file-in-hand",
            "copy",
            "swap",
            "placements",
            "question",
            "question-file",
            "question-clear",
            "question-copy",
            "recording",
            "value",
            "info",
            "done",
            "warn",
            "error",
            "help",
            "messages",
            "unsaved",
            "save-failed",
            "safety",
            "input-device",
            "project-picker",
            "project-name",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        if screen.rules().screen_learn {
            modes.push("learn".into());
        }
        match screen {
            Screen::Pads => modes.extend(
                pads::page_names()
                    .into_iter()
                    .map(|page| format!("edit-{}", page.to_lowercase())),
            ),
            Screen::Chop => modes.extend(
                [
                    "edit-bounds",
                    "assign-hand",
                    "assign-order",
                    "128-slices",
                    "loop",
                    "as-pad",
                    "question-plan",
                ]
                .map(str::to_owned),
            ),
            Screen::Fx => modes.push("fx-2".into()),
            Screen::Settings => modes.extend(
                [
                    "midi-learn",
                    "input-tab",
                    "midi-tab",
                    "paths-tab",
                    "folder-picker",
                    "question-profile",
                ]
                .map(str::to_owned),
            ),
            Screen::Files => {}
        }
        if matches!(screen, Screen::Pads | Screen::Files | Screen::Chop) {
            modes.push("hidden-grid".into());
        }
        if matches!(screen, Screen::Pads | Screen::Files) {
            modes.push("text".into());
        }
        for mode in modes {
            for (width, height) in SIZES {
                expected.insert(format!("{}-{mode}-{width}x{height}.ansi", screen.key()));
            }
        }
    }
    let mut present = BTreeSet::new();
    for case in cases {
        let name = case.name();
        if !expected.contains(&name) {
            return Err(format!("unsupported catalogue entry: {name}"));
        }
        if !present.insert(name.clone()) {
            return Err(format!("duplicate catalogue entry: {name}"));
        }
    }
    let missing = expected.difference(&present).cloned().collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("missing catalogue entries: {}", missing.join(", ")))
    }
}

fn fixture(case: Case, now: Instant) -> App {
    let mut app = super::tests::test_app();
    app.loader = None;
    app.screen = case.screen;
    app.session_read_error = None;
    app.session_path = "/offline/session.toml".into();
    app.paths.projects = "/offline/projects".into();
    app.paths.samples = "/offline/samples".into();
    app.browser.dir = "/offline/samples".into();
    app.browser.entries = vec![
        browser::Entry {
            name: "Drums".into(),
            is_dir: true,
            favorite: None,
        },
        browser::Entry {
            name: "kick.wav".into(),
            is_dir: false,
            favorite: None,
        },
        browser::Entry {
            name: "音 e\u{301} long sample name.wav".into(),
            is_dir: false,
            favorite: None,
        },
    ];
    app.browser.selected = 1;
    let frames = if case.mode == Mode::Slices128 {
        96_000
    } else {
        48_000
    };
    let sound = Arc::new(Sample::mono(
        (0..frames)
            .map(|i| ((i % 128) as f32 - 64.0) / 256.0)
            .collect(),
    ));
    for (pad, name) in [
        (0, "kick"),
        (1, "snare"),
        (2, "音 e\u{301} sample"),
        (15, "slice 128"),
    ] {
        app.pads.sound[pad] = Some(sound.clone());
        app.pads.file[pad] = Some(format!("/offline/samples/{name}.wav").into());
        app.pads.custom_name[pad] = Some(name.into());
    }
    let mut chop = chop::Chop::new(
        "/offline/samples/音 e\u{301} long break name.wav".into(),
        sound,
        48_000,
    );
    if case.mode != Mode::Slices128 {
        for cut in [12_000, 24_000, 36_000] {
            assert_eq!(chop.cut(cut), chop::Cut::Placed);
        }
    }
    app.chop = Some(chop);
    app.notices = Notices::new(now);
    match case.mode {
        Mode::Tone | Mode::Envelope | Mode::Filter | Mode::Source | Mode::Sends => {
            app.edit = true;
            app.pads.param = match case.mode {
                Mode::Envelope => pads::Param::Attack,
                Mode::Filter => pads::Param::Filter,
                Mode::Source => pads::Param::Source,
                Mode::Sends => pads::Param::Drive,
                _ => pads::Param::Volume,
            };
        }
        Mode::Bounds => app.edit = true,
        Mode::Fx2 => app.fx_slot = 1,
        Mode::Mark => app.pads.marking = true,
        Mode::Marked => {
            app.pads.marked.extend([0, 1]);
        }
        Mode::File => app.browser.in_hand = Some("/offline/samples/new.wav".into()),
        Mode::Copy => app.copy_in_hand = Some((0, app.pads.snapshot(0))),
        Mode::Swap => app.pad_in_hand = Some(1),
        Mode::Placements => {
            app.browser.in_hand = Some("/offline/samples/new.wav".into());
            app.copy_in_hand = Some((0, app.pads.snapshot(0)));
            app.pad_in_hand = Some(1);
        }
        Mode::Assign => app.plan = Some(chop::Plan::ByHand(vec![(0, 0), (1, 1)])),
        Mode::AssignOrder => app.plan = Some(chop::Plan::InOrder(Some(0))),
        Mode::Learn => app.screen_learn = Some(app.learn_open()),
        Mode::MidiLearn => {
            app.settings_tab = 2;
            app.learning = Some(learn::Learning::new(action_info("pad.copy")));
        }
        Mode::Question => {
            app.ask = Some(Ask::ResetPads {
                pads: vec![0, 1],
                lines: vec![
                    "Reset 2 pads to clean?".into(),
                    "The sounds and their bounds stay.".into(),
                    "Ctrl+Z brings the settings back.".into(),
                    "Enter: reset · Esc: keep".into(),
                ],
            });
        }
        Mode::QuestionFile => {
            app.browser.in_hand = Some("/offline/samples/new.wav".into());
            app.ask = Some(Ask::PutFile {
                abs: 0,
                path: "/offline/samples/new.wav".into(),
                lines: vec![
                    "Replace pad 1 kick?".into(),
                    "The new sound replaces the current sound.".into(),
                    "Ctrl+Z brings the sound back.".into(),
                    "Enter: replace · Esc: keep".into(),
                ],
            });
        }
        Mode::QuestionProfile => {
            app.settings_tab = 2;
            app.ask = Some(Ask::ResetProfile {
                name: "Offline fixture".into(),
                port: "Offline fixture input".into(),
                lines: vec![
                    "Restore the device profile?".into(),
                    "The learned bindings are removed.".into(),
                    "F10: reset · Esc: keep".into(),
                ],
            });
        }
        Mode::QuestionClear => {
            app.ask = Some(Ask::ClearPads {
                pads: vec![0, 1],
                lines: vec![
                    "Clear 2 pads?".into(),
                    "The sounds are removed from these pads.".into(),
                    "Ctrl+Z brings the sounds back.".into(),
                    "Enter: clear · Esc: keep".into(),
                ],
            })
        }
        Mode::QuestionCopy => {
            app.copy_in_hand = Some((0, app.pads.snapshot(0)));
            app.pads.selected = 1;
            app.ask = Some(Ask::CopyPad {
                source: 0,
                lines: vec![
                    "Replace pad 2 snare with a copy of kick?".into(),
                    "Ctrl+Z brings the sound back.".into(),
                    "Enter: replace · Esc: keep".into(),
                ],
            });
        }
        Mode::QuestionPlan => {
            app.plan = Some(chop::Plan::InOrder(Some(0)));
            app.ask = Some(Ask::Plan {
                lines: vec![
                    "Put 4 slices on bank 1?".into(),
                    "This replaces 3 occupied pads.".into(),
                    "Ctrl+Z brings the sounds back.".into(),
                    "Enter: replace · Esc: keep".into(),
                ],
            });
        }
        Mode::Recording | Mode::Safety => {
            app.recording = Some((audio::SOURCE_OUTPUT, now - Duration::from_secs(12)))
        }
        Mode::Info | Mode::Done | Mode::Warn | Mode::Error => {
            let (level, text) = match case.mode {
                Mode::Info => (Level::Info, "Offline fixture: the target is unchanged"),
                Mode::Done => (Level::Done, "Offline fixture: operation complete"),
                Mode::Warn => (Level::Warn, "Offline fixture: wait for loading"),
                _ => (Level::Error, "Offline fixture: save refused"),
            };
            app.notices
                .push(Notice::new(level, Origin::Keyboard, text), now);
        }
        Mode::Help | Mode::Messages => {
            app.help_open = true;
            app.help_overlay.messages = case.mode == Mode::Messages;
            for (level, origin, text) in [
                (
                    Level::Info,
                    Origin::System,
                    "Offline fixture: session opened",
                ),
                (
                    Level::Done,
                    Origin::Keyboard,
                    "Offline fixture: operation complete",
                ),
                (
                    Level::Warn,
                    Origin::Device("Offline fixture MIDI".into()),
                    "Offline fixture: wait for loading",
                ),
            ] {
                app.notices.push(Notice::new(level, origin, text), now);
            }
        }
        Mode::Unsaved => app.session_dirty = Some(now),
        Mode::SaveFailed => app.session_write_failed = true,
        Mode::Hidden => app.ui.hide_pads = true,
        Mode::Text if case.screen == Screen::Pads => {
            app.pad_naming = Some((0, "fixture name".into()))
        }
        Mode::Text => {
            app.browser.editing = true;
            app.browser.filter = "kick".into();
            app.browser.entries.retain(|entry| entry.name == "kick.wav");
            app.browser.selected = 0;
        }
        Mode::Picker | Mode::PickerName => {
            app.project_picker = Some(ProjectPicker {
                entries: vec!["beats".into(), "live set".into()],
                selected: 0,
                naming: (case.mode == Mode::PickerName).then(|| "new set".into()),
            })
        }
        Mode::InputTab => app.settings_tab = 1,
        Mode::MidiTab => app.settings_tab = 2,
        Mode::PathsTab | Mode::FolderPicker => app.settings_tab = 3,
        Mode::Slices128 => {
            let c = app.chop.as_mut().unwrap();
            for i in 1..128 {
                assert_eq!(c.cut(i * 750), chop::Cut::Placed);
            }
            c.selected = 127;
        }
        Mode::Loop => app.chop.as_mut().unwrap().looping = true,
        Mode::AsPad => app.chop.as_mut().unwrap().as_pad = true,
        Mode::Normal | Mode::Value | Mode::Device => {}
    }
    if case.mode == Mode::Safety {
        app.session_write_failed = true;
    }
    app
}

fn render(case: Case, keymap: &Keymap) -> (Vec<u8>, Buffer) {
    let now = Instant::now();
    let mut app = fixture(case, now);
    let notice = app.frame_notice_at(now);
    let mut pads = pads_view(
        &mut app.pads,
        None,
        app.pad_in_hand,
        vec![0.2; 64],
        None,
        app.rate,
    );
    preview_pad_name(&mut pads, app.pads.bank, app.pad_naming.as_ref());
    pads.edit_key = keys(keymap, Screen::Pads, &[Action::PadEdit]);
    let pads = app.mark_pads(pads);
    let mut chop = app.chop_display(app.chop.as_ref().unwrap());
    if chop.slice_keys.is_none() {
        bank_waves(&mut chop.pads, &mut app.pads, app.rate);
    }
    chop.which = app.which_chop();
    app.learn_grid(&mut chop.pads.pads);
    let history = app.message_history();
    let learning = app.learning_view(keymap);
    let question = app.question_view(keymap);
    let groups = if app.screen_learn.is_some() {
        vec![]
    } else if app.project_picker.is_some() {
        picker_help(
            app.project_picker
                .as_ref()
                .is_some_and(|picker| picker.naming.is_some()),
        )
    } else {
        without_modifiers(help_groups(keymap, app.screen, &app.help_state()))
    };
    let keys = full_help(keymap, app.screen, true);
    let help = tui::Help {
        groups: &groups,
        hint: hint_for(&app),
        full: app
            .help_open
            .then(|| app.full_help_view(&keys, &history))
            .flatten(),
        question: question.as_ref(),
        learn: learning.as_ref(),
    };
    let feedback = tui::Feedback {
        inputs: vec![
            tui::InputSource {
                normal: "KB".into(),
                compact: "KB".into(),
                active: false,
            },
            tui::InputSource {
                normal: "M1".into(),
                compact: if case.mode == Mode::Device {
                    "M1"
                } else {
                    "  "
                }
                .into(),
                active: case.mode == Mode::Device,
            },
        ]
        .into(),
        recording_seconds: app.recording.map(|_| 12),
    };
    let status = tui::Status {
        badges: app.title_badges_at(now),
        save: app.save_status(),
        backend: (case.mode != Mode::Safety).then_some("offline"),
        recording: app.recording.is_some(),
        buffer_size: 128,
        sample_rate: 48_000,
        load_peak: 15.0,
        xruns: if case.mode == Mode::Safety { 3 } else { 0 },
        master_db: 0,
        meters: ([-18.0, -12.0], [-20.0, -16.0], false),
        notice: notice.as_deref().map(|text| tui::NoticeLine {
            text,
            level: app.frame_notice_level.map(|level| match level {
                Level::Info => tui::NoticeLevel::Info,
                Level::Done => tui::NoticeLevel::Done,
                Level::Warn => tui::NoticeLevel::Warn,
                Level::Error => tui::NoticeLevel::Error,
            }),
        }),
        no_releases: case.mode == Mode::Safety,
        project: Some("offline catalogue"),
    };
    let midi_rows = app.midi_screen();
    let midi_learning = app.learning_line();
    let hand = app.browser.in_hand.as_deref().map(file_name);
    let screen = if let Some(picker) = app.picker_view() {
        tui::Screen::ProjectPicker(picker)
    } else {
        match case.screen {
            Screen::Pads => tui::Screen::Pads(pads),
            Screen::Fx => tui::Screen::Fx(app.fx_view()),
            Screen::Chop => tui::Screen::Chop(Box::new(app.mark_chop(chop, None))),
            Screen::Files => tui::Screen::Files(tui::Files {
                dir: "/offline/samples",
                entries: app
                    .browser
                    .entries
                    .iter()
                    .map(|entry| (entry.name.as_str(), entry.is_dir, false))
                    .collect(),
                selected: app.browser.selected,
                in_hand: hand.as_deref(),
                error: None,
                pads,
                filter: &app.browser.filter,
                editing: app.browser.editing,
                preview: None,
                playhead: None,
            }),
            Screen::Settings => tui::Screen::Settings(tui::Settings {
                row: 0,
                graphics: false,
                output: Some("Offline fixture output"),
                output_missing: None,
                input: Some("Offline fixture input"),
                input_missing: None,
                input_level: 0.25,
                buffer_size: 128,
                buffer_choice: 128,
                routing: true,
                backend_jack: true,
                tab: app.settings_tab,
                midi_devices: &[],
                input_log: &[],
                midi_rows: &midi_rows,
                midi_row: 0,
                learning: midi_learning.as_deref(),
                paths: Some(("/offline/projects", "/offline/samples")),
                picker: (case.mode == Mode::FolderPicker).then(|| tui::PathPickerView {
                    title: " PICK THE PROJECTS ROOT ",
                    entries: vec!["..".into(), "beats".into()],
                    selected: 1,
                }),
            }),
        }
    };
    let value = (case.mode == Mode::Value).then(|| tui::Value {
        object: "Master".into(),
        parameter: "volume".into(),
        formatted: "+0 dB".into(),
        position: Some(0.85),
    });
    let mut bytes = b"\x1b[2J\x1b[H".to_vec();
    let buffer;
    {
        // A fixed viewport and vector writer never query or write to a real terminal.
        let backend = CrosstermBackend::new(&mut bytes);
        let mut term = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Fixed(Rect::new(0, 0, case.size.0, case.size.1)),
            },
        )
        .unwrap();
        let mut cache = tui::TextCache::default();
        buffer = term
            .draw(|frame| {
                tui::draw_cached(frame, &status, &screen, &help, &feedback, &mut cache, None);
                if let Some(value) = &value {
                    tui::value_popup(frame, value);
                }
            })
            .unwrap()
            .buffer
            .clone();
    }
    bytes.extend_from_slice(b"\x1b[0m\x1b[?25h");
    (bytes, buffer)
}

fn check_frame(case: Case, buffer: &Buffer) {
    let rows = buffer
        .content()
        .chunks(case.size.0 as usize)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), case.size.1 as usize);
    let text = rows.join("\n");
    let require = |label: &str| {
        assert!(
            text.contains(label),
            "{} clips required text {label:?}\n{text}",
            case.name()
        )
    };
    match case.mode {
        Mode::Picker | Mode::PickerName => require("PROJECT"),
        Mode::Help | Mode::Messages => {
            require("HELP");
            require("MESSAGES");
        }
        _ => assert!(
            rows[1].contains(case.screen.title()),
            "{} loses screen identity: {}",
            case.name(),
            rows[1]
        ),
    }
    match case.mode {
        Mode::Tone | Mode::Envelope | Mode::Filter | Mode::Source | Mode::Sends | Mode::Bounds => {
            require("EDIT")
        }
        Mode::Fx2 => require("FX 2 · selected"),
        Mode::Mark => require("MARK"),
        Mode::Marked => require("2 MARKED"),
        Mode::File => require("IN HAND"),
        Mode::Copy => require("COPY"),
        Mode::Swap => require("SWAP"),
        Mode::Placements => {
            require("COPY");
            require("SWAP");
            require("IN HAND");
        }
        Mode::Assign | Mode::AssignOrder => require("ASSIGN"),
        Mode::Learn | Mode::MidiLearn => require("LEARN"),
        Mode::Question
        | Mode::QuestionFile
        | Mode::QuestionClear
        | Mode::QuestionCopy
        | Mode::QuestionPlan => {
            require("ASK");
            require("KB");
            require("Ctrl+Z");
        }
        Mode::QuestionProfile => {
            require("ASK");
            require("KB");
            require("learned bindings");
        }
        Mode::PickerName => require("new set"),
        Mode::Recording => {
            require("REC");
            require("rec 12s");
        }
        Mode::Value => require("volume · Master"),
        Mode::Info => require("Offline fixture: the target is unchanged"),
        Mode::Done => require("Offline fixture: operation complete"),
        Mode::Warn => require("Offline fixture: wait for loading"),
        Mode::Error => require("Offline fixture: save refused"),
        Mode::Messages => require("Offline fixture: operation complete"),
        Mode::Unsaved => {
            require("*");
            assert!(!text.contains("Unsaved"));
        }
        Mode::SaveFailed => require("Save failed"),
        Mode::Safety => {
            for label in [
                "Save failed",
                "no audio",
                "rec",
                "xruns 3",
                "no key releases: gate pads play one-shot",
            ] {
                assert!(
                    rows[0].contains(label),
                    "{} clips safety {label:?}: {}",
                    case.name(),
                    rows[0]
                );
            }
        }
        Mode::Device => require("▸M1"),
        Mode::Text if case.screen == Screen::Pads => require("fixture name"),
        Mode::Text => require("kick"),
        Mode::FolderPicker => require("PICK THE PROJECTS ROOT"),
        Mode::Slices128 => require("128 slices"),
        Mode::Loop => require("LOOP"),
        Mode::AsPad => require("AS PAD"),
        _ => {}
    }
    for (index, cell) in buffer.content().iter().enumerate() {
        let x = index % case.size.0 as usize;
        assert!(
            x + ratatui::text::Line::from(cell.symbol()).width() <= case.size.0 as usize,
            "{} crosses its last column at ({x}, {}): {:?}\n{text}",
            case.name(),
            index / case.size.0 as usize,
            cell.symbol()
        );
        assert!(
            !cell.symbol().chars().any(char::is_control),
            "{} contains a text control character",
            case.name()
        );
    }
    assert!(
        rows[case.size.1 as usize - 3..]
            .iter()
            .any(|row| !row.trim().is_empty()),
        "{} loses the fixed help band",
        case.name()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_assignment_target_names_stay_in_the_sidebar() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        for size in SIZES {
            let case = Case {
                screen: Screen::Chop,
                mode: Mode::AssignOrder,
                size,
            };
            let (_, buffer) = render(case, &keymap);
            check_frame(case, &buffer);
            let rows = buffer
                .content()
                .chunks(size.0 as usize)
                .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                .collect::<Vec<_>>();
            let text = rows.join("\n");
            for name in ["kick", "snare", "音", "e\u{301} sample"] {
                assert!(
                    text.contains(name),
                    "a target name disappeared: {name}\n{text}"
                );
            }
        }
    }

    #[test]
    fn catalogue_covers_screens_and_existing_modes() {
        let cases = matrix();
        validate(&cases).unwrap();
        let (keymap, warnings) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        assert!(warnings.is_empty());
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/screens");
        std::fs::create_dir_all(&directory).unwrap();
        for case in &cases {
            let (bytes, buffer) = render(*case, &keymap);
            check_frame(*case, &buffer);
            assert!(
                !bytes.windows(3).any(|bytes| bytes == b"\x1b_G"),
                "ANSI is not a kitty-composition test"
            );
            std::fs::write(directory.join(case.name()), bytes).unwrap();
        }
        println!(
            "Catalogue: {} ANSI files at 120x40 and 80x26 in renders/screens/",
            cases.len()
        );
    }

    #[test]
    fn missing_catalogue_entries_fail_coverage() {
        let all = matrix();
        let mut missing = all.clone();
        let first = missing.remove(0);
        assert!(validate(&missing).unwrap_err().contains(&first.name()));
        missing.retain(|case| case.size != (80, 26));
        assert!(validate(&missing).unwrap_err().contains("80x26.ansi"));
        let mut duplicate = all.clone();
        duplicate.push(all[0]);
        assert!(validate(&duplicate).unwrap_err().contains("duplicate"));
        let mut invalid = all;
        invalid[0].size = (79, 26);
        assert!(validate(&invalid).unwrap_err().contains("unsupported"));
        let pages = Mode::ALL
            .iter()
            .copied()
            .filter(|mode| mode.applies(Screen::Pads) && mode.name().starts_with("edit-"))
            .count();
        assert_eq!(
            pages,
            pads::page_names().len(),
            "every current pad edit page needs a fixture"
        );
    }

    #[test]
    fn catalogue_bytes_are_repeatable_without_devices_or_project_writes() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        for case in matrix() {
            assert_eq!(
                render(case, &keymap).0,
                render(case, &keymap).0,
                "{} is not deterministic",
                case.name()
            );
        }
    }
}

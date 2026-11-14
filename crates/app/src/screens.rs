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
    QuietEmpty,
    ClipEvent,
    LongLabels,
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
    PatternActive,
    PatternStopped,
    PatternEmpty,
    PatternPending,
    PatternControls,
    PatternCountIn,
    PatternRecording,
    PatternErase,
    PatternFinishing,
    PatternSafety,
    PatternViewFine,
    PatternViewTriplet,
    PatternViewOffGrid,
    PatternViewMultiple,
    PatternViewHidden,
    PatternViewSafety,
    PatternOverviewAbsent,
    PatternOverviewEmpty,
    PatternOverviewStraight,
    PatternOverviewTriplet,
    PatternOverviewOffGrid,
    PatternOverviewMultiple,
    PatternOverviewHiddenHit,
    PatternOverviewSafety,
    PatternStepEmpty,
    PatternStepStraight,
    PatternStepTriplet,
    PatternStepOffGrid,
    PatternStepMultiple,
    PatternStepCursor,
    PatternStepSafety,
    PatternErasePads,
    PitchGateOff,
    PitchGateA1,
    PitchGateSource,
    PitchGatePitch,
}

impl Mode {
    const ALL: &'static [Self] = &[
        Self::Normal,
        Self::QuietEmpty,
        Self::ClipEvent,
        Self::LongLabels,
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
        Self::PatternActive,
        Self::PatternStopped,
        Self::PatternEmpty,
        Self::PatternPending,
        Self::PatternControls,
        Self::PatternCountIn,
        Self::PatternRecording,
        Self::PatternErase,
        Self::PatternFinishing,
        Self::PatternSafety,
        Self::PatternViewFine,
        Self::PatternViewTriplet,
        Self::PatternViewOffGrid,
        Self::PatternViewMultiple,
        Self::PatternViewHidden,
        Self::PatternViewSafety,
        Self::PatternOverviewAbsent,
        Self::PatternOverviewEmpty,
        Self::PatternOverviewStraight,
        Self::PatternOverviewTriplet,
        Self::PatternOverviewOffGrid,
        Self::PatternOverviewMultiple,
        Self::PatternOverviewHiddenHit,
        Self::PatternOverviewSafety,
        Self::PatternStepEmpty,
        Self::PatternStepStraight,
        Self::PatternStepTriplet,
        Self::PatternStepOffGrid,
        Self::PatternStepMultiple,
        Self::PatternStepCursor,
        Self::PatternStepSafety,
        Self::PatternErasePads,
        Self::PitchGateOff,
        Self::PitchGateA1,
        Self::PitchGateSource,
        Self::PitchGatePitch,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::QuietEmpty => "quiet-empty",
            Self::ClipEvent => "clip-event",
            Self::LongLabels => "long-labels",
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
            Self::PatternActive => "pattern-active",
            Self::PatternStopped => "pattern-stopped",
            Self::PatternEmpty => "pattern-empty",
            Self::PatternPending => "pattern-pending",
            Self::PatternControls => "pattern-controls",
            Self::PatternCountIn => "pattern-count-in",
            Self::PatternRecording => "pattern-recording",
            Self::PatternErase => "pattern-erase",
            Self::PatternFinishing => "pattern-finishing",
            Self::PatternSafety => "pattern-safety",
            Self::PatternViewFine => "pattern-view-fine",
            Self::PatternViewTriplet => "pattern-view-triplet",
            Self::PatternViewOffGrid => "pattern-view-off-grid",
            Self::PatternViewMultiple => "pattern-view-multiple",
            Self::PatternViewHidden => "pattern-view-hidden",
            Self::PatternViewSafety => "pattern-view-safety",
            Self::PatternOverviewAbsent => "pattern-overview-absent",
            Self::PatternOverviewEmpty => "pattern-overview-empty",
            Self::PatternOverviewStraight => "pattern-overview-straight",
            Self::PatternOverviewTriplet => "pattern-overview-triplet",
            Self::PatternOverviewOffGrid => "pattern-overview-off-grid",
            Self::PatternOverviewMultiple => "pattern-overview-multiple",
            Self::PatternOverviewHiddenHit => "pattern-overview-hidden-hit",
            Self::PatternOverviewSafety => "pattern-overview-safety",
            Self::PatternStepEmpty => "pattern-step-empty",
            Self::PatternStepStraight => "pattern-step-straight",
            Self::PatternStepTriplet => "pattern-step-triplet",
            Self::PatternStepOffGrid => "pattern-step-off-grid",
            Self::PatternStepMultiple => "pattern-step-multiple",
            Self::PatternStepCursor => "pattern-step-cursor",
            Self::PatternStepSafety => "pattern-step-safety",
            Self::PatternErasePads => "pattern-erase-pads",
            Self::PitchGateOff => "pitch-gate-off",
            Self::PitchGateA1 => "pitch-gate-a1",
            Self::PitchGateSource => "pitch-gate-source",
            Self::PitchGatePitch => "pitch-gate-pitch",
        }
    }

    fn applies(self, screen: Screen) -> bool {
        match self {
            Self::PatternViewFine
            | Self::PatternViewTriplet
            | Self::PatternViewOffGrid
            | Self::PatternViewMultiple
            | Self::PatternViewHidden
            | Self::PatternViewSafety
            | Self::PatternOverviewAbsent
            | Self::PatternOverviewEmpty
            | Self::PatternOverviewStraight
            | Self::PatternOverviewTriplet
            | Self::PatternOverviewOffGrid
            | Self::PatternOverviewMultiple
            | Self::PatternOverviewHiddenHit
            | Self::PatternOverviewSafety
            | Self::PatternStepEmpty
            | Self::PatternStepStraight
            | Self::PatternStepTriplet
            | Self::PatternStepOffGrid
            | Self::PatternStepMultiple
            | Self::PatternStepCursor
            | Self::PatternStepSafety
            | Self::PatternErasePads
            | Self::Tone
            | Self::Envelope
            | Self::Filter
            | Self::Source
            | Self::Sends
            | Self::PitchGateOff
            | Self::PitchGateA1
            | Self::PitchGateSource
            | Self::PitchGatePitch
            | Self::QuietEmpty
            | Self::ClipEvent
            | Self::LongLabels => screen == Screen::Pads,
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
            "pattern-active",
            "pattern-stopped",
            "pattern-empty",
            "pattern-pending",
            "pattern-controls",
            "pattern-count-in",
            "pattern-recording",
            "pattern-erase",
            "pattern-finishing",
            "pattern-safety",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        if screen.rules().screen_learn {
            modes.push("learn".into());
        }
        match screen {
            Screen::Pads => {
                modes.extend(
                    pads::page_names()
                        .into_iter()
                        .map(|page| format!("edit-{}", page.to_lowercase())),
                );
                modes.extend(
                    [
                        "quiet-empty",
                        "clip-event",
                        "long-labels",
                        "pattern-view-fine",
                        "pattern-view-triplet",
                        "pattern-view-off-grid",
                        "pattern-view-multiple",
                        "pattern-view-hidden",
                        "pattern-view-safety",
                        "pattern-overview-absent",
                        "pattern-overview-empty",
                        "pattern-overview-straight",
                        "pattern-overview-triplet",
                        "pattern-overview-off-grid",
                        "pattern-overview-multiple",
                        "pattern-overview-hidden-hit",
                        "pattern-overview-safety",
                        "pattern-step-empty",
                        "pattern-step-straight",
                        "pattern-step-triplet",
                        "pattern-step-off-grid",
                        "pattern-step-multiple",
                        "pattern-step-cursor",
                        "pattern-step-safety",
                        "pattern-erase-pads",
                        "pitch-gate-off",
                        "pitch-gate-a1",
                        "pitch-gate-source",
                        "pitch-gate-pitch",
                    ]
                    .map(str::to_owned),
                );
            }
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
            app.help_overlay.page = if case.mode == Mode::Messages {
                crate::help::Page::Messages
            } else {
                crate::help::Page::Help
            };
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
        Mode::QuietEmpty => app.pads = pads::Pads::default(),
        Mode::ClipEvent => {
            app.meters = meters::Meters::new(now - Duration::from_secs(3_723));
            app.meters.update((1.0, 0.7), now);
        }
        Mode::LongLabels => {
            app.pads.custom_name[0] = Some("warm dusty snare sample with 音 e\u{301}".into());
            app.pads.file[0] = Some("/offline/samples/warm dusty snare sample.wav".into());
            app.project = Some(Project {
                name: "warm dusty project collection".into(),
                folder: "/offline/project".into(),
            });
            app.session_write_failed = true;
        }
        Mode::Normal | Mode::Value | Mode::Device => {}
        Mode::PatternCountIn
        | Mode::PatternRecording
        | Mode::PatternErase
        | Mode::PatternFinishing => add_recording(&mut app, case.mode),
        Mode::PatternSafety => {
            add_recording(&mut app, Mode::PatternRecording);
            app.session_write_failed = true;
        }
        Mode::PatternControls => {
            app.apply(Action::PatternControls);
            app.apply(Action::PatternParamNext);
        }
        Mode::PitchGateOff | Mode::PitchGateA1 | Mode::PitchGateSource | Mode::PitchGatePitch => {
            set_pitch_gate_fixture(&mut app, case.mode);
        }
        Mode::PatternActive | Mode::PatternStopped | Mode::PatternEmpty | Mode::PatternPending => {
            add_pattern(&mut app, case.mode);
        }
        Mode::PatternViewFine
        | Mode::PatternViewTriplet
        | Mode::PatternViewOffGrid
        | Mode::PatternViewMultiple
        | Mode::PatternViewHidden
        | Mode::PatternViewSafety => add_pattern_view(&mut app, case.mode),
        Mode::PatternOverviewAbsent => {
            app.pads.pattern = None;
            app.pattern_view.overview = true;
        }
        Mode::PatternOverviewEmpty => {
            add_pattern(&mut app, Mode::PatternEmpty);
            app.pattern_view.overview = true;
        }
        Mode::PatternOverviewStraight
        | Mode::PatternOverviewTriplet
        | Mode::PatternOverviewOffGrid
        | Mode::PatternOverviewMultiple
        | Mode::PatternOverviewHiddenHit
        | Mode::PatternOverviewSafety => add_pattern_overview(&mut app, case.mode),
        Mode::PatternStepEmpty
        | Mode::PatternStepStraight
        | Mode::PatternStepTriplet
        | Mode::PatternStepOffGrid
        | Mode::PatternStepMultiple
        | Mode::PatternStepCursor
        | Mode::PatternStepSafety => add_pattern_step(&mut app, case.mode),
        Mode::PatternErasePads => {
            add_pattern_view(&mut app, Mode::PatternViewMultiple);
            app.apply(Action::PatternErase);
            assert!(app.erase_pads);
        }
    }
    if case.mode == Mode::Safety {
        app.session_write_failed = true;
    }
    app
}

fn set_pitch_gate_fixture(app: &mut App, mode: Mode) {
    app.pads.pitch_gate[0] = mode != Mode::PitchGateOff;
    app.pads.pitch[0] = if mode == Mode::PitchGatePitch { 0 } else { 12 };
    app.pads.link[0] = (mode != Mode::PitchGateSource).then_some((0, 0));
    app.pads.slice[0] = Some(1);
}

fn add_pattern(app: &mut App, mode: Mode) {
    let (audio, mut realtime) = audio::offline(48_000);
    app.audio = Some(audio);
    let raw = session::PatternState {
        slot: "A1".into(),
        bars: 1,
        quantize: "1/16".into(),
        strength: 100,
        hits: if mode == Mode::PatternEmpty {
            Vec::new()
        } else {
            vec![[0, 0, 100, 0]]
        },
    };
    app.send(Command::SetPadSound {
        pad: 0,
        sound: app.pads.sound[0].clone(),
    });
    app.send(Command::SetPattern {
        pattern: Some(Arc::new(raw.checked().unwrap().compile())),
    });
    app.pads.pattern = Some(raw.clone());
    if mode != Mode::PatternStopped {
        app.send(Command::TransportPlay);
    }
    realtime.process(&mut [0.0; 128], &mut [0.0; 128]);
    assert_eq!(app.transport_state().pattern_ticks, 3840);
    if mode == Mode::PatternPending {
        let mut next = raw;
        next.bars = 4;
        app.send(Command::SetPattern {
            pattern: Some(Arc::new(next.checked().unwrap().compile())),
        });
        app.pads.pattern = Some(next);
        assert_eq!(app.transport_state().pattern_ticks, 3840);
    }
}

fn add_pattern_view(app: &mut App, mode: Mode) {
    let (audio, mut rt) = audio::offline(48_000);
    app.audio = Some(audio);
    let raw = session::PatternState {
        slot: "A1".into(),
        bars: 2,
        quantize: "off".into(),
        strength: 100,
        hits: if mode == Mode::PatternViewMultiple {
            vec![[0, 0, 80, 0], [1, 0, 100, 960], [240, 0, 70, 0]]
        } else {
            vec![
                [1, 0, 100, 0],
                [241, 0, 80, 0],
                [1920, 0, 100, 0],
                [2001, 0, 80, 0],
            ]
        },
    };
    app.send(Command::SetPattern {
        pattern: Some(Arc::new(raw.checked().unwrap().compile())),
    });
    app.pads.pattern = Some(raw);
    let changes = match mode {
        Mode::PatternViewFine => 4,
        Mode::PatternViewTriplet => 1,
        _ => 0,
    };
    for _ in 0..changes {
        app.pattern_view.division.change(true);
    }
    app.ui.hide_pads = mode == Mode::PatternViewHidden;
    app.session_write_failed = mode == Mode::PatternViewSafety;
    if matches!(mode, Mode::PatternViewFine | Mode::PatternViewTriplet) {
        app.send(Command::TransportPlay);
        rt.process(&mut vec![0.0; 48_001], &mut vec![0.0; 48_001]);
    } else {
        rt.process(&mut [], &mut []);
    }
}

fn add_pattern_overview(app: &mut App, mode: Mode) {
    let view_mode = match mode {
        Mode::PatternOverviewTriplet => Mode::PatternViewTriplet,
        Mode::PatternOverviewMultiple => Mode::PatternViewMultiple,
        Mode::PatternOverviewSafety => Mode::PatternViewSafety,
        _ => Mode::PatternViewOffGrid,
    };
    add_pattern_view(app, view_mode);
    app.pattern_view.overview = true;
    let raw = app.pads.pattern.as_mut().unwrap();
    raw.hits.extend([[960, 20, 90, 0], [1920, 54, 70, 0]]);
    for (pad, name) in [
        (20, "rec-20261031-200404 20 minute take"),
        (54, "rec-20261101-091500 21 minute take"),
    ] {
        app.pads.sound[pad] = app.pads.sound[0].clone();
        app.pads.file[pad] = Some(format!("/offline/samples/{name}.wav").into());
        app.pads.custom_name[pad] = Some(name.into());
    }
    if mode == Mode::PatternOverviewHiddenHit {
        raw.bars = 1;
        raw.hits.push([5000, 20, 100, 0]);
    }
}

fn add_pattern_step(app: &mut App, mode: Mode) {
    match mode {
        Mode::PatternStepEmpty => {
            add_pattern(app, Mode::PatternEmpty);
            app.pattern_view.overview = true;
        }
        Mode::PatternStepTriplet => add_pattern_overview(app, Mode::PatternOverviewTriplet),
        Mode::PatternStepMultiple => add_pattern_overview(app, Mode::PatternOverviewMultiple),
        Mode::PatternStepSafety => add_pattern_overview(app, Mode::PatternOverviewSafety),
        _ => add_pattern_overview(app, Mode::PatternOverviewOffGrid),
    }
    app.pattern_view.edit = true;
    app.pattern_view.cursor_visited = true;
    app.pattern_view.cursor_cell = match mode {
        Mode::PatternStepEmpty => 3,
        Mode::PatternStepCursor => 20,
        Mode::PatternStepOffGrid => 1,
        _ => 0,
    };
    if mode == Mode::PatternStepMultiple {
        app.pads.selected = 0;
    }
    if mode == Mode::PatternStepSafety {
        app.pads.bank = 1;
        app.pads.selected = 4;
    }
}

fn add_recording(app: &mut App, mode: Mode) {
    let (audio, mut realtime) = audio::offline(48_000);
    app.audio = Some(audio);
    app.apply(Action::PatternRecord);
    realtime.process(&mut [], &mut []);
    assert_eq!(
        app.pattern_record_state().unwrap().phase,
        engine::capture::Phase::CountIn
    );
    if mode == Mode::PatternCountIn {
        return;
    }
    realtime.process(&mut vec![0.0; 96_000], &mut vec![0.0; 96_000]);
    assert_eq!(
        app.pattern_record_state().unwrap().phase,
        engine::capture::Phase::Recording
    );
    if mode == Mode::PatternErase {
        app.apply(Action::PatternErase);
        let pass = app.pattern_record_state().unwrap().pass;
        for pad in [0, 18] {
            app.send(Command::PatternEraseTarget {
                pass,
                pad,
                one_loop: false,
            });
        }
        realtime.process(&mut [], &mut []);
        assert!(app.pattern_record_state().unwrap().erasing);
    } else if mode == Mode::PatternFinishing {
        app.apply(Action::PatternRecord);
        realtime.process(&mut [], &mut []);
        assert_eq!(
            app.pattern_record_state().unwrap().phase,
            engine::capture::Phase::Finishing
        );
    }
}

fn render(case: Case, keymap: &Keymap) -> (Vec<u8>, Buffer) {
    render_at(case, keymap, Instant::now())
}

fn render_at(case: Case, keymap: &Keymap, now: Instant) -> (Vec<u8>, Buffer) {
    render_app(case, keymap, fixture(case, now), now)
}

fn render_app(case: Case, keymap: &Keymap, app: App, now: Instant) -> (Vec<u8>, Buffer) {
    render_app_layers(case, keymap, app, now, true)
}

fn render_app_layers(
    case: Case,
    keymap: &Keymap,
    mut app: App,
    now: Instant,
    show_pattern_controls: bool,
) -> (Vec<u8>, Buffer) {
    app.resize_help(case.size.0, case.size.1);
    let pattern_recording = app.pattern_record_state();
    let kitty = !matches!(
        case.mode,
        Mode::Safety | Mode::PatternSafety | Mode::PatternViewSafety | Mode::PatternStepSafety
    );
    let help_keys = app.full_help_recording_keys(keymap, kitty, pattern_recording);
    if app.help_open {
        app.prepare_key_help(keymap, &help_keys, pattern_recording);
    }
    let notice = app.frame_notice_at(now);
    let mut pads = pads_view(
        &mut app.pads,
        None,
        app.pad_in_hand,
        vec![0.2; 64],
        None,
        app.rate,
    );
    // The live view uses wall time for hit flashes. Catalogue frames use the fixture time.
    for (index, pad) in pads.pads.iter_mut().enumerate() {
        let abs = app.pads.bank * PADS_PER_BANK + index;
        pad.flash = app.pads.held[abs]
            || app.pads.hit_at[abs].is_some_and(|at| now.saturating_duration_since(at) < HIT_FLASH);
    }
    preview_pad_name(&mut pads, app.pads.bank, app.pad_naming.as_ref());
    pads.edit_key = keys(keymap, Screen::Pads, &[Action::PadEdit]);
    let mut pads = app.mark_pads_recording(pads, pattern_recording);
    pads.bank_pattern = app.bank_pattern();
    if app.screen == Screen::Pads {
        if app.pattern_view.overview {
            pads.overview = Some(app.pattern_overview_at(keymap, case.size.0));
        } else {
            pads.pattern = Some(app.pattern_strip(keymap));
        }
    }
    let mut chop = app.chop_display_recording(app.chop.as_ref().unwrap(), pattern_recording);
    if chop.slice_keys.is_none() {
        bank_waves(&mut chop.pads, &mut app.pads, app.rate);
    }
    chop.which = app.which_chop();
    app.learn_grid(&mut chop.pads.pads);
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
        let groups = app.pattern_help_groups(
            keymap,
            help_groups(keymap, app.screen, &app.help_state()),
            pattern_recording,
            kitty,
        );
        if app.pattern_view.overview {
            as_combos(groups)
        } else {
            without_modifiers(groups)
        }
    };
    let history = app.message_history();
    let help = tui::Help {
        groups: &groups,
        hint: hint_for(&app),
        full: app
            .help_open
            .then(|| app.full_help_view(&help_keys, &history))
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
        badges: view::FrameText::default()
            .badges_for(&app, now, pattern_recording)
            .to_vec(),
        save: app.save_status(),
        backend: (!matches!(
            case.mode,
            Mode::Safety | Mode::PatternSafety | Mode::PatternViewSafety | Mode::PatternStepSafety
        ))
        .then_some("offline"),
        recording: app.recording.is_some(),
        buffer_size: 128,
        sample_rate: 48_000,
        load_peak: 15.0,
        xruns: if matches!(
            case.mode,
            Mode::Safety | Mode::PatternSafety | Mode::PatternViewSafety | Mode::PatternStepSafety
        ) {
            3
        } else {
            0
        },
        master_db: 0,
        meters: if case.mode == Mode::ClipEvent {
            (
                app.meters.level_db(),
                app.meters.peak_db(),
                app.meters.clip_elapsed(),
            )
        } else {
            ([-18.0, -12.0], [-20.0, -16.0], None)
        },
        notice: notice.as_deref().map(|text| tui::NoticeLine {
            text,
            level: app.frame_notice_level.map(|level| match level {
                Level::Info => tui::NoticeLevel::Info,
                Level::Done => tui::NoticeLevel::Done,
                Level::Warn => tui::NoticeLevel::Warn,
                Level::Error => tui::NoticeLevel::Error,
            }),
        }),
        no_releases: !kitty,
        project: Some(
            app.project
                .as_ref()
                .map_or("offline catalogue", |project| project.name.as_str()),
        ),
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
            Screen::Chop => tui::Screen::Chop(Box::new(app.mark_chop_recording(
                chop,
                None,
                pattern_recording,
            ))),
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
    let pattern_controls = show_pattern_controls
        .then(|| app.pattern_controls_view(keymap))
        .flatten();
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
                if let Some(controls) = &pattern_controls {
                    tui::pattern_controls(frame, &screen, controls);
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
        Mode::PatternOverviewAbsent
        | Mode::PatternOverviewEmpty
        | Mode::PatternOverviewStraight
        | Mode::PatternOverviewTriplet
        | Mode::PatternOverviewOffGrid
        | Mode::PatternOverviewMultiple
        | Mode::PatternOverviewHiddenHit
        | Mode::PatternOverviewSafety => require("PATTERN · A1"),
        Mode::PatternStepEmpty
        | Mode::PatternStepStraight
        | Mode::PatternStepTriplet
        | Mode::PatternStepOffGrid
        | Mode::PatternStepMultiple
        | Mode::PatternStepCursor
        | Mode::PatternStepSafety => require("STEP EDIT · A1"),
        Mode::Help | Mode::Messages => {
            require("HELP");
            require("MESSAGES");
            if case.mode == Mode::Help && case.screen == Screen::Pads {
                require("PITCH GATE");
                require("PG n/a (PG on: needs linked slice)");
                require("linked slices with pitch above zero");
            }
        }
        _ => assert!(
            rows[1].contains(case.screen.title()),
            "{} loses screen identity: {}",
            case.name(),
            rows[1]
        ),
    }
    match case.mode {
        Mode::QuietEmpty => require("—"),
        Mode::ClipEvent => require("CLIP +01:02:03"),
        Mode::LongLabels => {
            require("whole file");
            require(if case.size.0 == 80 {
                "warm…"
            } else {
                "warm dusty snare…"
            });
            require("Save failed");
        }
        Mode::Tone | Mode::Envelope | Mode::Filter | Mode::Source | Mode::Sends => require("EDIT"),
        Mode::Bounds => {
            require("EDIT · BOUNDS");
            require("─ SLICE");
            require("cursor");
        }
        Mode::PitchGateOff | Mode::PitchGateA1 | Mode::PitchGateSource | Mode::PitchGatePitch => {
            let (label, explanation) = match case.mode {
                Mode::PitchGateOff => ("PG off", "PG off, manual unchanged"),
                Mode::PitchGateA1 => ("PG A1", "PG on: A1 only, manual unchanged"),
                Mode::PitchGateSource => ("PG n/a", "PG on: needs linked slice, manual unchanged"),
                Mode::PitchGatePitch => ("PG n/a", "PG on: needs Pitch > 0, manual unchanged"),
                _ => unreachable!(),
            };
            let mode = rows.iter().find(|row| row.contains("mode ")).unwrap();
            assert_eq!(
                mode.contains(label),
                case.mode != Mode::PitchGatePitch,
                "{text}"
            );
            let detail = match (case.size.0, case.mode) {
                (80, Mode::PitchGateSource) => "PG on: needs slice",
                (_, Mode::PitchGateSource) => "PG on: needs linked slice",
                (_, Mode::PitchGatePitch) => "PG on: needs Pitch > 0",
                (_, Mode::PitchGateA1) => "PG on: A1 only",
                _ => "PG off",
            };
            assert!(rows.iter().any(|row| row.contains(detail)), "{text}");
            // the help row explains Pitch Gate only for a linked slice with it on
            let applies = matches!(case.mode, Mode::PitchGateA1 | Mode::PitchGatePitch);
            let last = &rows[case.size.1 as usize - 1];
            assert_eq!(last.contains(explanation), applies, "{text}");
            assert_eq!(last.contains("PG "), applies, "{text}");
        }
        // the lit slot is checked by colour in the tui tests (98e)
        Mode::Fx2 => {
            require("EDIT · FX 1 FX 2");
            require("─ FX 2 ·");
        }
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
        Mode::PatternCountIn => require("COUNT 4"),
        Mode::PatternRecording => require("REC A1"),
        Mode::PatternErase => {
            require("ERASE A1");
            require("1-1");
        }
        Mode::PatternFinishing => require("Finishing A1"),
        Mode::PatternErasePads => {
            require("ERASE");
            require(" hits");
        }
        Mode::PatternViewFine => {
            require("view 1/64");
            require("A1 · bar 1/2 beat 3");
        }
        Mode::PatternViewTriplet => {
            require("view 1/16 triplet");
            require("A1 · bar 1/2 beat 3");
        }
        Mode::PatternViewOffGrid | Mode::PatternViewHidden => require("~"),
        Mode::PatternViewMultiple => require(" 2"),
        Mode::PatternViewSafety => {
            require("Save failed");
            require("no audio");
            require("xruns 3");
            require("~");
        }
        Mode::PatternOverviewAbsent => require("pattern A1 is empty"),
        Mode::PatternOverviewEmpty => require("view 1/16"),
        Mode::PatternOverviewStraight => require("steps 1-"),
        Mode::PatternOverviewTriplet => require("view 1/16 triplet"),
        Mode::PatternOverviewOffGrid | Mode::PatternOverviewHiddenHit => require("~"),
        Mode::PatternOverviewMultiple => require(" 2"),
        Mode::PatternOverviewSafety => {
            require("Save failed");
            require("PATTERN · A1");
        }
        Mode::PatternStepEmpty => require("empty cell"),
        Mode::PatternStepTriplet => require("triplet"),
        Mode::PatternStepOffGrid => require("~"),
        Mode::PatternStepMultiple => require(" 2"),
        Mode::PatternStepSafety => require("Save failed"),
        Mode::PatternSafety => {
            require("REC A1");
            for label in [
                "Save failed",
                "no audio",
                "xruns 3",
                "no key releases: gate pads play one-shot",
            ] {
                assert!(rows[0].contains(label), "{text}");
            }
        }
        Mode::PatternControls => {
            for label in [
                "A1 PATTERN",
                "bars",
                "▸ quantize",
                "strength",
                "CLEAR",
                "close",
            ] {
                require(label);
            }
        }
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
        Mode::PatternActive | Mode::PatternStopped | Mode::PatternEmpty | Mode::PatternPending => {
            require("A1");
            require("120.0");
            let compact = rows[1].replace(' ', "");
            assert!(compact.contains("A11:1"), "{}: {}", case.name(), rows[1]);
        }
        _ => {}
    }
    if matches!(case.mode, Mode::PatternViewFine | Mode::PatternViewTriplet) {
        let header = rows
            .iter()
            .position(|row| row.contains("A1 · bar"))
            .expect("pattern header");
        let dark = ratatui::style::Color::Rgb(0x07, 0x08, 0x0A);
        let plate = ratatui::style::Color::Rgb(0xBC, 0xBC, 0xBC);
        let width = case.size.0 as usize;
        let mut columns = Vec::new();
        // the all-pads row and the selected pad's row carry the same centred plate
        for row in [header + 1, header + 2] {
            let events = &buffer.content()[row * width..(row + 1) * width];
            let lit: Vec<usize> = (0..width).filter(|&x| events[x].bg == plate).collect();
            assert_eq!(lit.len(), 3, "{} needs one three-column plate", case.name());
            assert_eq!(lit[2] - lit[0], 2, "{} splits its plate", case.name());
            assert_eq!(events[lit[0]].symbol(), " ");
            assert_eq!(events[lit[2]].symbol(), " ");
            for &x in &lit {
                assert_eq!(events[x].fg, dark, "{} loses its plate text", case.name());
            }
            assert!(
                !events.iter().any(|cell| cell.symbol() == ">"),
                "playback has no arrow"
            );
            columns.push((lit[1], events[lit[1]].symbol().to_string()));
        }
        assert_eq!(columns[0].0, columns[1].0, "{} plates differ", case.name());
        // the bank row underlines bank 1, which holds the A1 onsets
        let bank_row = rows
            .iter()
            .position(|row| row.starts_with("bank"))
            .expect("bank row");
        let underlined: String = buffer.content()[bank_row * width..(bank_row + 1) * width]
            .iter()
            .filter(|cell| cell.modifier.contains(ratatui::style::Modifier::UNDERLINED))
            .map(|cell| cell.symbol())
            .collect();
        assert_eq!(underlined, "1", "{} bank row", case.name());
        assert_eq!(columns[0].1, "■", "{} all-pads row", case.name());
        assert_eq!(
            columns[1].1,
            if case.mode == Mode::PatternViewFine {
                "■"
            } else {
                "2"
            },
            "{} pad row",
            case.name()
        );
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

    /// Scenario "Long names stay readable at both sizes".
    #[test]
    fn long_names_stay_readable_at_both_sizes() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        for size in SIZES {
            let pads = Case {
                screen: Screen::Pads,
                mode: Mode::LongLabels,
                size,
            };
            let (_, frame) = render(pads, &keymap);
            check_frame(pads, &frame);

            let files = Case {
                screen: Screen::Files,
                mode: Mode::Normal,
                size,
            };
            let now = Instant::now();
            let mut app = fixture(files, now);
            app.browser.dir = "/offline/warm dusty folder with 音 e\u{301}".into();
            let name = "warm dusty sample with 音 e\u{301} and more words.wav";
            app.browser.entries = vec![browser::Entry {
                name: name.into(),
                is_dir: false,
                favorite: None,
            }];
            app.browser.selected = 0;
            app.browser.in_hand = Some(format!("/offline/{name}").into());
            let (_, frame) = render_app(files, &keymap, app, now);
            let text = frame
                .content()
                .chunks(size.0 as usize)
                .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(text.contains("← in hand"), "{text}");
            assert!(text.contains("warm dusty"), "{text}");
            assert!(text.contains('…'), "{text}");

            let picker = Case {
                screen: Screen::Pads,
                mode: Mode::Picker,
                size,
            };
            let now = Instant::now();
            let mut app = fixture(picker, now);
            app.project_picker.as_mut().unwrap().entries = vec![
                "warm dusty project with 音 e\u{301} and many more words repeated to fill the screen and another long sequence of project words"
                    .into(),
            ];
            let (_, frame) = render_app(picker, &keymap, app, now);
            let text = frame
                .content()
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();
            assert!(text.contains("warm dusty project"), "{text}");
            assert!(text.contains('…'), "{text}");
        }
    }

    /// Scenario "Screen words name the same concepts".
    #[test]
    fn screen_words_name_the_same_concepts() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        for size in SIZES {
            for screen in Screen::ALL {
                let case = Case {
                    screen,
                    size,
                    mode: Mode::Normal,
                };
                let (_, buffer) = render(case, &keymap);
                let text = buffer
                    .content()
                    .iter()
                    .map(|cell| cell.symbol())
                    .collect::<String>();
                assert!(!text.contains("rhythm"), "{}", case.name());
                assert!(text.contains("PATTERN"), "{}", case.name());
            }
            let case = Case {
                screen: Screen::Settings,
                size,
                mode: Mode::MidiLearn,
            };
            let now = Instant::now();
            let mut app = fixture(case, now);
            app.learning = Some(learn::Learning::new(action_info("screen.pattern")));
            let (_, buffer) = render_app(case, &keymap, app, now);
            let text = buffer
                .content()
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();
            assert!(text.contains("pattern"), "{}", case.name());
            assert!(!text.contains("rhythm"), "{}", case.name());
            assert_eq!(action_info("screen.pattern").group, input::Group::Global);
        }
    }

    /// Scenario "Pattern warnings explain limits without claiming success".
    #[test]
    fn pattern_warning_recovery_and_safety_remain_visible_with_long_project_names() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        for screen in Screen::ALL {
            for size in SIZES {
                for cause in ["phase", "newer", "full", "overflow"] {
                    for controls in [false, true] {
                        let case = Case {
                            screen,
                            size,
                            mode: Mode::PatternSafety,
                        };
                        let now = Instant::now();
                        let mut app = fixture(case, now);
                        app.project = Some(Project {
                            name: "very long owner project ".repeat(12),
                            folder: "/offline/project".into(),
                        });
                        let (audio, mut rt) = audio::offline(48000);
                        app.audio = Some(audio);
                        let raw = session::PatternState {
                            slot: "A1".into(),
                            bars: 1,
                            quantize: "off".into(),
                            strength: 100,
                            hits: if cause == "full" {
                                vec![[5000, 127, 1, 0]; 4096]
                            } else {
                                vec![]
                            },
                        };
                        let rows = (0..raw.hits.len())
                            .map(|slot| {
                                engine::pattern::PlaybackRow::existing(
                                    engine::pattern::RowId::new(slot, 1).unwrap(),
                                )
                            })
                            .collect::<Vec<_>>();
                        let pattern = app.pads.restore_recorded_pattern(&raw, &rows).unwrap();
                        app.send(pattern);
                        rt.process(&mut [], &mut []);
                        app.send(Command::TransportPlay);
                        assert!(
                            app.try_send(Command::PatternRecordStart { pass: 1 }),
                            "{screen:?}/{cause}/{controls}: pending {}, recording {:?}, accepted {:?}",
                            app.pending_audio.len(),
                            app.recording,
                            app.pattern_record_state()
                        );
                        rt.process(&mut [], &mut []);
                        if matches!(cause, "newer" | "full" | "overflow") {
                            for _ in 0..if cause == "overflow" { 640 } else { 1 } {
                                assert!(app.try_send(Command::PadPress {
                                    pad: 0,
                                    velocity: 1.0
                                }));
                            }
                            for _ in 0..3 {
                                rt.process(&mut [0.0; 128], &mut [0.0; 128]);
                            }
                            app.flush_audio();
                        }
                        if cause == "newer" {
                            app.send(Command::PatternRecordEnd { pass: 1 });
                            rt.process(&mut [], &mut []);
                            app.flush_audio();
                            rt.process(&mut [], &mut []);
                            assert_eq!(app.history.len(), 1);
                            let before = app.before();
                            app.pads.volume_db[5] -= 1;
                            app.note_step(before, "pad.5.volume".into());
                            assert_eq!(app.history.len(), 2);
                            app.apply(Action::PatternUndo);
                            assert_eq!(app.history.len(), 2);
                        } else if cause == "phase" {
                            app.apply(Action::PatternControls);
                            app.apply(Action::PatternValueInc);
                        }
                        // the newest notice: a pass that ended first has its own Done notice
                        let warning = app.notices.history().front().unwrap().clone();
                        assert_eq!(warning.level, Level::Warn, "{cause}: {warning:?}");
                        assert!(warning.text.contains(match cause {
                            "phase" => "Idle",
                            "newer" => "newer",
                            "full" => "row-limit refusals",
                            _ => "omitted onsets",
                        }));
                        if cause == "full" {
                            assert_eq!(app.pads.pattern.as_ref().unwrap().hits.len(), 4096);
                            assert_eq!(app.pattern_record_state().unwrap().captured_onsets, 0);
                        }
                        if cause == "overflow" {
                            assert_eq!(app.pads.pattern.as_ref().unwrap().hits.len(), 512);
                            assert_eq!(app.pattern_record_state().unwrap().dropped_onsets, 128);
                        }
                        if app.pattern_panel.is_some() != controls {
                            app.apply(Action::PatternControls);
                        }
                        let (_, frame) =
                            render_app(case, &keymap, app, now + Duration::from_millis(1));
                        let rows = frame
                            .content()
                            .chunks(size.0 as usize)
                            .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                            .collect::<Vec<_>>();
                        let text = rows.join("\n");
                        for safety in [
                            "Save failed",
                            "no audio",
                            "xruns 3",
                            "no key releases: gate pads play one-shot",
                        ] {
                            assert!(
                                rows[0].contains(safety),
                                "{screen:?}/{cause}/{controls}: {text}"
                            );
                        }
                        assert!(
                            rows[size.1 as usize - 4].contains(&warning.text),
                            "{screen:?}/{cause}/{controls}: {text}"
                        );
                        assert!(
                            rows[1].contains(if cause == "newer" { "A1" } else { "REC A1" }),
                            "{text}"
                        );
                        assert!(
                            text.contains(if controls {
                                "close"
                            } else if cause == "newer" {
                                "record A1"
                            } else {
                                "end A1"
                            }),
                            "{text}"
                        );
                        check_frame(
                            Case {
                                mode: Mode::Normal,
                                ..case
                            },
                            &frame,
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn pattern_short_help_uses_effective_edit_keys_none_and_visible_arrow_bindings() {
        for screen in Screen::ALL {
            for size in SIZES {
                for removed in [false, true] {
                    let (mut keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
                    keymap
                        .overlay(if removed {
                            r#"[edit]
"Ctrl+R"="none"
"Alt+E"="none"
"Alt+Z"="none"
I="none"
K="none"
Up="none"
Down="none"
J="none"
L="none"
Left="none"
Right="none"
"#
                        } else {
                            r#"[edit]
"Ctrl+R"="none"
"Alt+E"="pattern.record"
"Alt+Z"="none"
"Ctrl+J"="pattern.erase"
"Ctrl+U"="pattern.undo"
I="none"
K="none"
J="none"
L="none"
"#
                        })
                        .unwrap();
                    let case = Case {
                        screen,
                        size,
                        mode: Mode::PatternRecording,
                    };
                    let now = Instant::now();
                    let mut app = fixture(case, now);
                    app.edit = true;
                    let groups =
                        app.pattern_help_groups(&keymap, vec![], app.pattern_record_state(), true);
                    assert_eq!(
                        groups[0].items,
                        vec![
                            (
                                if removed { "none" } else { "Alt E" }.into(),
                                "end A1".into()
                            ),
                            (
                                if removed { "none" } else { "Ctrl J" }.into(),
                                "erase pads".into()
                            ),
                            (
                                if removed { "none" } else { "Ctrl U" }.into(),
                                "pass undo".into()
                            )
                        ]
                    );
                    let (_, frame) = render_app(case, &keymap, app, now);
                    let text = frame
                        .content()
                        .chunks(size.0 as usize)
                        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                        .collect::<Vec<_>>()
                        .join("\n");
                    for role in ["end A1", "erase pads", "pass undo"] {
                        assert!(text.contains(role), "{screen:?}/{removed}: {text}");
                    }
                    if removed {
                        assert!(text.matches("none").count() >= 3, "{text}");
                    } else {
                        for key in ["Alt E", "Ctrl J", "Ctrl U"] {
                            assert!(text.contains(key), "{text}");
                        }
                    }
                    let mut app = fixture(case, now);
                    app.apply(Action::PatternControls);
                    let controls = app.pattern_controls_view(&keymap).unwrap();
                    if removed {
                        assert_eq!(controls.controls[0], "none select  none change");
                    } else {
                        for key in ["↑", "↓", "←", "→"] {
                            assert!(controls.controls[0].contains(key));
                        }
                    }
                    let (_, frame) = render_app(case, &keymap, app, now);
                    let text = frame
                        .content()
                        .chunks(size.0 as usize)
                        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                        .collect::<Vec<_>>()
                        .join("\n");
                    assert!(
                        text.contains(if removed {
                            "none select  none change"
                        } else {
                            "↑/↓ select  ←/→ change"
                        }),
                        "{text}"
                    );
                    check_frame(
                        Case {
                            mode: Mode::Normal,
                            ..case
                        },
                        &frame,
                    );
                }
            }
        }
    }

    /// Scenario "Catalogue includes recording states and safety combinations".
    #[test]
    fn recording_feedback_catalogue_has_phases_controls_blockers_and_safety() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        let directory =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/screens/recording");
        std::fs::create_dir_all(&directory).unwrap();
        let mut count = 0;
        for screen in Screen::ALL {
            for size in SIZES {
                for mode in [
                    Mode::PatternCountIn,
                    Mode::PatternRecording,
                    Mode::PatternErase,
                    Mode::PatternFinishing,
                ] {
                    let case = Case { screen, size, mode };
                    let label = match mode {
                        Mode::PatternCountIn => "COUNT 4",
                        Mode::PatternRecording => "REC A1",
                        Mode::PatternErase => "ERASE A1",
                        _ => "Finishing A1",
                    };
                    for layer in ["hidden", "edit", "controls", "question"] {
                        let now = Instant::now();
                        let mut app = fixture(case, now);
                        match layer {
                            "hidden" => app.ui.hide_pads = true,
                            "edit" => app.edit = true,
                            "controls" => {
                                app.apply(Action::PatternControls);
                            }
                            "question" => {
                                app.ask = Some(Ask::ResetPads {
                                    pads: vec![0],
                                    lines: vec![
                                        "Reset this pad?".into(),
                                        "Enter: reset · Esc: keep".into(),
                                    ],
                                })
                            }
                            _ => unreachable!(),
                        }
                        let overlay = (layer == "controls").then(|| {
                            let model = match screen {
                                Screen::Pads | Screen::Files | Screen::Chop => {
                                    let pads =
                                        pads_view(&mut app.pads, None, None, vec![], None, 48_000);
                                    tui::Screen::Pads(app.mark_pads(pads))
                                }
                                _ => tui::Screen::Fx(app.fx_view()),
                            };
                            tui::pattern_controls_area(Rect::new(0, 0, size.0, size.1), &model)
                        });
                        let mut reference = fixture(case, now);
                        reference.ui.hide_pads = app.ui.hide_pads;
                        reference.edit = app.edit;
                        let (_, baseline) = render_app(case, &keymap, reference, now);
                        let (bytes, frame) = render_app(case, &keymap, app, now);
                        check_frame(
                            Case {
                                mode: Mode::Normal,
                                ..case
                            },
                            &frame,
                        );
                        let rows: Vec<String> = frame
                            .content()
                            .chunks(size.0 as usize)
                            .map(|r| r.iter().map(|c| c.symbol()).collect())
                            .collect();
                        assert!(
                            rows[1].contains(label),
                            "{screen:?}/{mode:?}/{layer}: {}",
                            rows.join("\n")
                        );
                        if layer == "controls" {
                            assert!(rows.iter().any(|r| r.contains("A1 PATTERN")));
                        }
                        if layer == "question" {
                            assert!(rows[1].contains("ASK"), "{}", rows[1]);
                            assert!(rows.iter().any(|r| r.contains("Reset this pad?")));
                        }
                        for y in [0, 2, 3] {
                            for x in 0..size.0 {
                                if overlay.is_some_and(|area| {
                                    x >= area.x
                                        && x < area.right()
                                        && y >= area.y
                                        && y < area.bottom()
                                }) {
                                    continue;
                                }
                                assert_eq!(
                                    frame[(x, y)],
                                    baseline[(x, y)],
                                    "{screen:?}/{mode:?}/{layer}: fixed cell {x},{y} outside the controls"
                                );
                            }
                        }
                        std::fs::write(
                            directory.join(format!("{}-{layer}.ansi", case.name())),
                            bytes,
                        )
                        .unwrap();
                        count += 1;
                    }
                }
                let case = Case {
                    screen,
                    size,
                    mode: Mode::PatternSafety,
                };
                let (bytes, frame) = render(case, &keymap);
                check_frame(case, &frame);
                std::fs::write(directory.join(case.name()), bytes).unwrap();
            }
        }
        assert_eq!(count, Screen::ALL.len() * SIZES.len() * 4 * 4);
        println!("Recording: {count} additional phase/overlay frames and 10 safety frames");
    }

    /// Scenario "Pattern controls fit without moving the pads".
    #[test]
    fn pattern_panel_preserves_grid_cells_and_safety_at_both_sizes() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        for screen in Screen::ALL {
            for size in SIZES {
                for hidden in [false, true] {
                    for safety in [false, true] {
                        let case = Case {
                            screen,
                            size,
                            mode: if safety { Mode::Safety } else { Mode::Normal },
                        };
                        let now = Instant::now();
                        let make_app = || {
                            let mut app = fixture(case, now);
                            app.ui.hide_pads = hidden;
                            if screen == Screen::Chop {
                                let chop = app.chop.as_ref().unwrap();
                                app.pads.file[0] = Some(chop.file.clone());
                                app.pads.sound[0] = Some(chop.sound.clone());
                                app.pads.region[0] = chop.slices()[0];
                                app.pads.link[0] = Some((chop.start_id(0), chop.epoch));
                                app.pads.slice[0] = Some(1);
                            }
                            app.apply(Action::PatternControls);
                            app.apply(Action::Back);
                            if screen == Screen::Chop {
                                assert!(
                                    app.chop_display(app.chop.as_ref().unwrap())
                                        .slice_keys
                                        .is_none(),
                                    "the reference fixture must show real pads"
                                );
                            }
                            app
                        };
                        let (_, closed) = render_app(case, &keymap, make_app(), now);
                        check_frame(case, &closed);
                        // Opening controls intentionally changes Chop's pad selection and dimming.
                        // Isolate the overlay from that input-role change, with full cell equality.
                        let mut base = make_app();
                        base.apply(Action::PatternControls);
                        let (_, reference) = render_app_layers(case, &keymap, base, now, false);
                        check_frame(case, &reference);
                        for selected in 0..3 {
                            let mut app = make_app();
                            app.apply(Action::PatternControls);
                            for _ in 0..selected {
                                app.apply(Action::PatternParamNext);
                            }
                            let controls = app.pattern_controls_view(&keymap).unwrap();
                            assert_eq!(controls.selected, selected);
                            let (_, actual) = render_app(case, &keymap, app, now);
                            check_frame(case, &actual);
                            let text = actual
                                .content()
                                .chunks(size.0 as usize)
                                .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                                .collect::<Vec<_>>()
                                .join("\n");
                            for label in [
                                "A1 PATTERN",
                                "bars",
                                "quantize",
                                "strength",
                                "CLEAR",
                                "close",
                            ] {
                                assert!(text.contains(label), "{text}");
                            }
                            let area = match screen {
                                Screen::Pads | Screen::Files | Screen::Chop => {
                                    // The same frame geometry is used by the production overlay.
                                    let mut app = make_app();
                                    let pads =
                                        pads_view(&mut app.pads, None, None, vec![], None, 48000);
                                    let pads = app.mark_pads(pads);
                                    tui::pattern_controls_area(
                                        Rect::new(0, 0, size.0, size.1),
                                        &tui::Screen::Pads(pads),
                                    )
                                }
                                _ => tui::pattern_controls_area(
                                    Rect::new(0, 0, size.0, size.1),
                                    &tui::Screen::Fx(make_app().fx_view()),
                                ),
                            };
                            for y in 0..size.1 {
                                for x in 0..size.0 {
                                    if !area.contains((x, y).into()) {
                                        assert_eq!(
                                            actual[(x, y)],
                                            reference[(x, y)],
                                            "{screen:?} {size:?} hidden={hidden} safety={safety} outside panel at {x},{y}"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Scenario "Pattern labels and hints stay whole".
    #[test]
    fn pattern_labels_and_hints_stay_whole() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        for size in SIZES {
            let case = Case {
                screen: Screen::Pads,
                size,
                mode: Mode::Normal,
            };
            let now = Instant::now();
            let text = |app: App| -> String {
                let (_, frame) = render_app(case, &keymap, app, now);
                frame
                    .content()
                    .chunks(size.0 as usize)
                    .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let make = || {
                let mut app = fixture(case, now);
                app.pads.pattern = Some(session::PatternState {
                    slot: "A1".into(),
                    bars: 1,
                    quantize: "1/4".into(),
                    strength: 100,
                    hits: vec![[0, 4, 90, 0], [960, 4, 90, 0], [1920, 5, 90, 0]],
                });
                app.pads.selected = 4;
                app
            };
            let shown = text(make());
            assert!(
                shown.contains("pad Q "),
                "{size:?}: the whole label\n{shown}"
            );
            for (rows, hint) in [(1, "next hits"), (2, "next hits, step 10"), (3, "Enter")] {
                let mut app = make();
                app.apply(Action::PatternControls);
                for _ in 0..rows {
                    app.apply(Action::PatternParamNext);
                }
                let shown = text(app);
                assert!(shown.contains(hint), "{size:?}: {hint}\n{shown}");
                assert!(shown.contains("Enter run"), "{size:?}\n{shown}");
            }
        }
    }

    /// Scenario "Pattern controls replace the pad panel".
    #[test]
    fn pattern_controls_replace_the_pad_panel() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        let rows = |buffer: &Buffer, area: Rect| -> String {
            (area.y..area.bottom())
                .map(|y| {
                    (area.x..area.right())
                        .map(|x| buffer[(x, y)].symbol())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        for screen in [Screen::Pads, Screen::Chop, Screen::Files, Screen::Fx] {
            for size in SIZES {
                let case = Case {
                    screen,
                    size,
                    mode: Mode::Normal,
                };
                let now = Instant::now();
                let mut closed = fixture(case, now);
                closed.apply(Action::PatternControls);
                closed.apply(Action::Back);
                let (_, closed) = render_app(case, &keymap, closed, now);
                let mut app = fixture(case, now);
                app.apply(Action::PatternControls);
                let area = {
                    let screen_view = match screen {
                        Screen::Fx => tui::Screen::Fx(app.fx_view()),
                        _ => {
                            let pads = pads_view(&mut app.pads, None, None, vec![], None, 48000);
                            tui::Screen::Pads(app.mark_pads(pads))
                        }
                    };
                    tui::pattern_controls_area(Rect::new(0, 0, size.0, size.1), &screen_view)
                };
                let (_, open) = render_app(case, &keymap, app, now);
                check_frame(case, &open);
                let text = rows(&open, area);
                for label in [
                    "A1 PATTERN",
                    "bars",
                    "quantize",
                    "strength",
                    "CLEAR",
                    "close",
                ] {
                    assert!(text.contains(label), "{screen:?} {size:?}:\n{text}");
                }
                for pad in ["volume", "attack", "cutoff", "TONE", "ENV"] {
                    assert!(!text.contains(pad), "{screen:?} {size:?} {pad}:\n{text}");
                }
                if screen == Screen::Pads {
                    assert!(
                        rows(&closed, area).contains("volume"),
                        "closing restores the pad panel"
                    );
                }
                // every cell outside the column stays; the help rows name the controls keys
                for y in 0..size.1 - 3 {
                    for x in 0..size.0 {
                        if !area.contains((x, y).into()) && screen != Screen::Chop {
                            assert_eq!(open[(x, y)], closed[(x, y)], "{screen:?} {size:?} {x},{y}");
                        }
                    }
                }
            }
        }
    }

    /// Scenario "Pattern controls use the pad edit frame".
    #[test]
    fn pattern_controls_use_the_pad_edit_frame() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        for size in SIZES {
            let case = Case {
                screen: Screen::Pads,
                size,
                mode: Mode::Normal,
            };
            let now = Instant::now();
            let mut edit = fixture(case, now);
            edit.edit = true;
            let (_, edit) = render_app(case, &keymap, edit, now);
            let mut app = fixture(case, now);
            app.apply(Action::PatternControls);
            let area = {
                let pads = pads_view(&mut app.pads, None, None, vec![], None, 48000);
                tui::pattern_controls_area(
                    Rect::new(0, 0, size.0, size.1),
                    &tui::Screen::Pads(app.mark_pads(pads)),
                )
            };
            let title = |buffer: &Buffer| -> String {
                (area.x..area.right())
                    .map(|x| buffer[(x, area.y)].symbol())
                    .collect()
            };
            let lit = |buffer: &Buffer, word: &str| {
                let row = title(buffer);
                let x = row.find(word).unwrap_or_else(|| panic!("{word} in {row}"));
                let x = area.x + row[..x].chars().count() as u16;
                buffer[(x, area.y)].bg
            };
            let frame_fg = |buffer: &Buffer| buffer[(area.x, area.y)].fg;
            let (_, bars) = render_app(case, &keymap, app, now);
            assert!(
                title(&bars).contains("PATTERN"),
                "{size:?}: {}",
                title(&bars)
            );
            assert_ne!(lit(&bars, "PATTERN"), lit(&bars, "CLEAR"), "{size:?}");
            // the pad edit frame: the same colour on its own panel corner
            let edit_corner = (0..size.1)
                .flat_map(|y| (0..size.0).map(move |x| (x, y)))
                .find(|&(x, y)| edit[(x, y)].symbol() == "┌" && edit[(x, y)].fg == frame_fg(&bars));
            assert!(
                edit_corner.is_some(),
                "{size:?}: the pad edit frame has the same colour"
            );
            let mut app = fixture(case, now);
            app.apply(Action::PatternControls);
            for _ in 0..3 {
                app.apply(Action::PatternParamNext);
            }
            let (_, clear) = render_app(case, &keymap, app, now);
            assert_eq!(
                lit(&clear, "CLEAR"),
                lit(&bars, "PATTERN"),
                "{size:?}: CLEAR lit"
            );
            assert_eq!(lit(&clear, "PATTERN"), lit(&bars, "CLEAR"), "{size:?}");
        }
    }

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

    /// Scenario "Pattern titles retain safety and actual playback slots".
    #[test]
    fn pattern_titles_retain_safety_and_actual_playback_slots() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        for case in matrix() {
            let now = Instant::now();
            let mut app = fixture(case, now);
            if !matches!(
                case.mode,
                Mode::PatternViewFine
                    | Mode::PatternViewTriplet
                    | Mode::PatternViewOffGrid
                    | Mode::PatternViewMultiple
                    | Mode::PatternViewHidden
                    | Mode::PatternViewSafety
                    | Mode::PatternOverviewAbsent
                    | Mode::PatternOverviewEmpty
                    | Mode::PatternOverviewStraight
                    | Mode::PatternOverviewTriplet
                    | Mode::PatternOverviewOffGrid
                    | Mode::PatternOverviewMultiple
                    | Mode::PatternOverviewHiddenHit
                    | Mode::PatternOverviewSafety
                    | Mode::PatternStepEmpty
                    | Mode::PatternStepStraight
                    | Mode::PatternStepTriplet
                    | Mode::PatternStepOffGrid
                    | Mode::PatternStepMultiple
                    | Mode::PatternStepCursor
                    | Mode::PatternStepSafety
            ) && !app
                .pattern_record_state()
                .is_some_and(|state| state.phase != engine::capture::Phase::Idle)
            {
                add_pattern(&mut app, Mode::PatternActive);
            }
            let (_, buffer) = render_app(case, &keymap, app, now);
            check_frame(case, &buffer);
            if !matches!(
                case.mode,
                Mode::Help | Mode::Messages | Mode::Picker | Mode::PickerName
            ) {
                let title = buffer.content()[case.size.0 as usize..2 * case.size.0 as usize]
                    .iter()
                    .map(|cell| cell.symbol())
                    .collect::<String>();
                assert!(
                    title.contains("A1"),
                    "{} loses pattern identity: {title}",
                    case.name()
                );
            }
            if case.screen == Screen::Pads && case.mode == Mode::Normal {
                let title = &buffer.content()[case.size.0 as usize..2 * case.size.0 as usize];
                let text = title.iter().map(|cell| cell.symbol()).collect::<String>();
                let start = text.find(" PLAY").unwrap();
                assert_eq!(
                    title[start..start + 10]
                        .iter()
                        .map(|cell| cell.symbol())
                        .collect::<String>(),
                    " PLAY ▶   "
                );
            }
        }
    }

    /// Scenario "Pitch Gate explanations fit the existing frame".
    /// Scenario "Pitch Gate help row appears only where it applies".
    #[test]
    fn pitch_gate_explanations_fit_the_existing_frame() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/screens");
        std::fs::create_dir_all(&directory).unwrap();
        let mut count = 0;
        for size in SIZES {
            for mode in [
                Mode::PitchGateOff,
                Mode::PitchGateA1,
                Mode::PitchGateSource,
                Mode::PitchGatePitch,
            ] {
                for hidden in [false, true] {
                    for page in
                        std::iter::once(None).chain(pads::page_names().into_iter().map(Some))
                    {
                        for safety in [false, true] {
                            let case = Case {
                                screen: Screen::Pads,
                                mode: if safety { Mode::Safety } else { Mode::Normal },
                                size,
                            };
                            let now = Instant::now();
                            let make_app = || {
                                let mut app = fixture(case, now);
                                app.ui.hide_pads = hidden;
                                app.edit = page.is_some();
                                if let Some(page) = page {
                                    let index =
                                        pads::page_names().iter().position(|p| *p == page).unwrap();
                                    app.pads.set_page_pos(
                                        (index as f32 + 0.5) / pads::page_names().len() as f32,
                                    );
                                }
                                app
                            };
                            let mut app = make_app();
                            set_pitch_gate_fixture(&mut app, mode);
                            // F1 gives the state and its reason for every pad
                            let f1 = app
                                .full_help_recording_keys(&keymap, true, None)
                                .into_iter()
                                .map(|(_, text)| text)
                                .collect::<Vec<_>>()
                                .join("\n");
                            let description = match mode {
                                Mode::PitchGateOff => "PG off",
                                Mode::PitchGateA1 => "PG on: A1 only",
                                Mode::PitchGateSource => "PG on: needs linked slice",
                                _ => "PG on: needs Pitch > 0",
                            };
                            assert!(f1.contains(description), "{mode:?}: {f1}");
                            let (bytes, before) = render_app(case, &keymap, app, now);
                            check_frame(case, &before);
                            let rows = before
                                .content()
                                .chunks(size.0 as usize)
                                .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                                .collect::<Vec<_>>();
                            let (label, reason) = match mode {
                                Mode::PitchGateOff => ("PG off", "PG off, manual unchanged"),
                                Mode::PitchGateA1 => ("PG A1", "PG on: A1 only, manual unchanged"),
                                Mode::PitchGateSource => {
                                    ("PG n/a", "PG on: needs linked slice, manual unchanged")
                                }
                                Mode::PitchGatePitch => {
                                    ("PG n/a", "PG on: needs Pitch > 0, manual unchanged")
                                }
                                _ => unreachable!(),
                            };
                            let mode_row = rows.iter().find(|row| row.contains("mode ")).unwrap();
                            assert_eq!(
                                mode_row.contains(label),
                                mode != Mode::PitchGatePitch,
                                "{rows:?}"
                            );
                            let panel_detail = if size.0 == 80 && mode == Mode::PitchGateSource {
                                "PG on: needs slice"
                            } else {
                                description
                            };
                            assert!(
                                rows.iter().any(|row| row.contains(panel_detail)),
                                "{rows:?}"
                            );
                            let applies = matches!(mode, Mode::PitchGateA1 | Mode::PitchGatePitch);
                            assert_eq!(
                                rows[size.1 as usize - 1].contains(reason),
                                applies,
                                "{rows:?}"
                            );
                            assert!(!rows[1].contains("PG"));
                            let (_, reference) = render_app(case, &keymap, make_app(), now);
                            let reference_rows = reference
                                .content()
                                .chunks(size.0 as usize)
                                .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                                .collect::<Vec<_>>();
                            for word in ["mode ", "scope ", "volume ", "pitch "] {
                                assert_eq!(
                                    rows.iter().position(|row| row.contains(word)),
                                    reference_rows.iter().position(|row| row.contains(word)),
                                    "fixed {word} row"
                                );
                            }
                            if let Some(page) = page {
                                assert!(rows.join("\n").contains(&format!("EDIT · {page}")));
                            }
                            let name = format!(
                                "pads-{}-{}-{}-{}-{}x{}.ansi",
                                mode.name(),
                                if hidden { "hidden" } else { "shown" },
                                page.unwrap_or("closed"),
                                if safety { "safety" } else { "normal" },
                                size.0,
                                size.1
                            );
                            std::fs::write(directory.join(name), bytes).unwrap();
                            count += 1;
                        }
                    }
                }
            }
        }
        println!("Pitch Gate: {count} additional ANSI frames with fixed rows and safety checks");
    }

    const PATTERN_VIEW_MODES: [Mode; 6] = [
        Mode::PatternViewFine,
        Mode::PatternViewTriplet,
        Mode::PatternViewOffGrid,
        Mode::PatternViewMultiple,
        Mode::PatternViewHidden,
        Mode::PatternViewSafety,
    ];

    /// Scenario "Catalogue covers the pattern view".
    #[test]
    fn catalogue_covers_the_pattern_view() {
        let cases = matrix();
        for mode in PATTERN_VIEW_MODES {
            for size in SIZES {
                assert!(
                    cases.iter().any(|case| case.screen == Screen::Pads
                        && case.mode == mode
                        && case.size == size),
                    "{mode:?} at {size:?}"
                );
            }
        }
        // a missing pattern-view entry fails coverage
        let mut missing = cases.clone();
        let gone = missing
            .iter()
            .position(|case| case.mode == Mode::PatternViewTriplet)
            .unwrap();
        let case = missing.remove(gone);
        assert!(validate(&missing).unwrap_err().contains(&case.name()));
        // offline fixtures: repeatable bytes, no devices, no project writes
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        for case in cases
            .iter()
            .filter(|case| PATTERN_VIEW_MODES.contains(&case.mode))
        {
            let now = Instant::now();
            let app = fixture(*case, now);
            assert!(app.session_path.starts_with("/offline"));
            let (first, first_frame) = render_at(*case, &keymap, now);
            let (second, second_frame) = render_at(*case, &keymap, now);
            if first != second {
                let at = first.iter().zip(&second).position(|(a, b)| a != b).unwrap();
                let cell = first_frame
                    .content()
                    .iter()
                    .zip(second_frame.content())
                    .position(|(a, b)| a != b);
                panic!(
                    "{} differs at byte {at}, cell {cell:?}: {:?} versus {:?}",
                    case.name(),
                    cell.map(|i| &first_frame.content()[i]),
                    cell.map(|i| &second_frame.content()[i])
                );
            }
        }
    }

    const OVERVIEW_MODES: [Mode; 8] = [
        Mode::PatternOverviewAbsent,
        Mode::PatternOverviewEmpty,
        Mode::PatternOverviewStraight,
        Mode::PatternOverviewTriplet,
        Mode::PatternOverviewOffGrid,
        Mode::PatternOverviewMultiple,
        Mode::PatternOverviewHiddenHit,
        Mode::PatternOverviewSafety,
    ];

    const STEP_MODES: [Mode; 7] = [
        Mode::PatternStepEmpty,
        Mode::PatternStepStraight,
        Mode::PatternStepTriplet,
        Mode::PatternStepOffGrid,
        Mode::PatternStepMultiple,
        Mode::PatternStepCursor,
        Mode::PatternStepSafety,
    ];

    /// Scenario "Catalogue covers correction without devices".
    /// Scenario "Edit screen keeps names, targets and safety visible".
    /// Scenario "Edit cursor does not follow playback".
    #[test]
    fn correction_catalogue_has_both_sizes_and_keeps_the_selected_bank() {
        let cases = matrix();
        for mode in STEP_MODES {
            for size in SIZES {
                let case = Case {
                    screen: Screen::Pads,
                    mode,
                    size,
                };
                assert!(
                    cases.iter().any(|listed| listed.screen == case.screen
                        && listed.mode == case.mode
                        && listed.size == case.size),
                    "{} is absent",
                    case.name()
                );
                let (bytes, buffer) = render(case, &Keymap::from_toml(DEFAULT_KEYMAP).unwrap().0);
                assert!(!bytes.is_empty());
                check_frame(case, &buffer);
                let rows = buffer
                    .content()
                    .chunks(size.0 as usize)
                    .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                    .collect::<Vec<_>>();
                assert!(rows.iter().any(|row| row.contains("STEP EDIT · A1")));
                assert!(
                    rows.iter()
                        .any(|row| row.contains("BANKS · pads with A1 hits"))
                );
                if mode == Mode::PatternStepSafety {
                    assert!(
                        rows.iter().any(|row| row.contains("Q rec-2026")),
                        "{}: {rows:#?}",
                        case.name()
                    );
                } else {
                    assert!(
                        rows.iter().any(|row| row.contains("1 kick")),
                        "{}: {rows:#?}",
                        case.name()
                    );
                }
            }
        }
        let mut missing = cases.clone();
        let gone = missing
            .iter()
            .position(|case| case.mode == Mode::PatternStepMultiple)
            .unwrap();
        let case = missing.remove(gone);
        assert!(validate(&missing).unwrap_err().contains(&case.name()));
    }

    /// Scenario "Pattern key names its destination".
    #[test]
    fn pattern_key_names_its_destination() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        let case = Case {
            screen: Screen::Files,
            mode: Mode::Normal,
            size: (120, 40),
        };
        let now = Instant::now();
        let mut app = fixture(case, now);
        let (_, files) = render_app(case, &keymap, app, now);
        let files_rows = files
            .content()
            .chunks(case.size.0 as usize)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
            .collect::<Vec<_>>();
        assert!(
            files_rows
                .iter()
                .any(|row| row.contains("F5") && row.contains("pattern")),
            "{:?}",
            &files_rows[files_rows.len() - 3..]
        );

        app = fixture(case, now);
        app.apply(Action::ScreenPattern);
        assert!(app.pattern_view.overview);
        let overview_case = Case {
            screen: Screen::Pads,
            ..case
        };
        let (_, overview) = render_app(overview_case, &keymap, app, now);
        let title = overview.content()[case.size.0 as usize..2 * case.size.0 as usize]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(title.contains("PATTERN · A1"), "{title}");
    }

    /// Scenario "Catalogue covers the overview without devices".
    #[test]
    fn catalogue_covers_the_overview_without_devices() {
        let cases = matrix();
        for mode in OVERVIEW_MODES {
            for size in SIZES {
                assert!(cases.iter().any(|case| {
                    case.screen == Screen::Pads && case.mode == mode && case.size == size
                }));
            }
        }
        let mut missing = cases.clone();
        let gone = missing
            .iter()
            .position(|case| case.mode == Mode::PatternOverviewTriplet)
            .unwrap();
        let case = missing.remove(gone);
        assert!(validate(&missing).unwrap_err().contains(&case.name()));
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        for case in cases
            .into_iter()
            .filter(|case| OVERVIEW_MODES.contains(&case.mode))
        {
            let app = fixture(case, Instant::now());
            assert!(app.session_path.starts_with("/offline"));
            let (first, _) = render_app(case, &keymap, app, Instant::now());
            assert_eq!(first, render(case, &keymap).0);
        }
    }

    /// Scenario "Overview keeps 16 rows and safety text visible".
    #[test]
    fn overview_keeps_sixteen_rows_and_safety_text_visible() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        for size in SIZES {
            let case = Case {
                screen: Screen::Pads,
                mode: Mode::PatternOverviewSafety,
                size,
            };
            let now = Instant::now();
            let mut app = fixture(case, now);
            app.pads.custom_name[0] = Some("long wide 界 name that must stay in one row".into());
            let (_, buffer) = render_app(case, &keymap, app, now);
            check_frame(case, &buffer);
            let rows = buffer
                .content()
                .chunks(size.0 as usize)
                .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                .collect::<Vec<_>>();
            assert!(rows[1].contains("PATTERN · A1"));
            assert!(rows[2].contains("BANKS · pads with A1 hits"));
            assert!(rows[2].contains("8:0 empty shown"), "{}", rows[2]);
            for number in 10..16 {
                assert!(rows[3].contains(&format!("{number} ")), "{}", rows[3]);
            }
            for (index, key) in "1234QWERASDFZXCV".chars().enumerate() {
                assert!(
                    rows[5 + index].contains(&format!(" {key} ")),
                    "{key}: {}",
                    rows[5 + index]
                );
            }
            assert!(rows[0].contains("Save failed"));
            let help = rows[size.1 as usize - 3..].join("\n");
            for text in ["PATTERN", "BANK", "CLOCK", "view", "empty shown"] {
                assert!(help.contains(text), "{text}: {help}");
            }
            assert!(
                rows[size.1 as usize - 4].contains("Alt+H hides empty banks."),
                "{}",
                rows[size.1 as usize - 4]
            );
        }
    }

    /// Scenario "Overview uses wide terminal space for names and steps".
    #[test]
    fn overview_uses_wide_terminal_space_for_names_and_steps() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        let case = Case {
            screen: Screen::Pads,
            mode: Mode::PatternOverviewStraight,
            size: (120, 40),
        };
        let (_, buffer) = render(case, &keymap);
        let rows = buffer
            .content()
            .chunks(case.size.0 as usize)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
            .collect::<Vec<_>>();
        assert!(
            rows.iter()
                .any(|row| row.contains("rec-20261031-200404 20 minute take"))
        );
        assert!(
            (105..119).any(|x| matches!(buffer[(x, 5)].symbol(), "·" | "■" | "~")),
            "time cells must use the right side"
        );

        let absent = Case {
            mode: Mode::PatternOverviewAbsent,
            ..case
        };
        let now = Instant::now();
        let mut app = fixture(absent, now);
        let long_name = "rec-20261031-200404 20 minute take from the room";
        app.pads.custom_name[0] = Some(long_name.into());
        let (_, buffer) = render_app(absent, &keymap, app, now);
        let title = buffer.content()[120..240]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(title.contains(long_name), "{title}");
        let row = buffer.content()[5 * 120..6 * 120]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(row.contains('…'), "{row}");
    }

    /// Scenario "Overview bank borders show identity without color alone".
    #[test]
    fn overview_bank_borders_show_identity_without_color_alone() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        let case = Case {
            screen: Screen::Pads,
            mode: Mode::PatternOverviewStraight,
            size: (120, 40),
        };
        let now = Instant::now();
        let app = fixture(case, now);
        let (_, buffer) = render_app(case, &keymap, app, now);
        assert_eq!(buffer[(0, 4)].symbol(), "┏");
        assert_eq!(buffer[(0, 4)].fg, tui::BANK_COLORS[0]);
        assert_eq!(buffer[(0, 22)].symbol(), "╭");
        assert_eq!(buffer[(0, 22)].fg, tui::BANK_COLORS[1]);
        let index = buffer.content()[2 * 120..3 * 120]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(index.contains("BANKS · pads with A1 hits"), "{index}");
        for bank in 1..=8 {
            assert!(index.contains(&format!("{bank}:")), "{index}");
        }
    }

    /// Scenario "Overview hides empty banks without losing access".
    /// Scenario "Overview bank navigation reaches a hidden empty bank".
    #[test]
    fn overview_hides_empty_bank_sections_without_losing_access() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        let case = Case {
            screen: Screen::Pads,
            mode: Mode::PatternOverviewStraight,
            size: (120, 80),
        };
        let now = Instant::now();
        let mut app = fixture(case, now);
        let text = |app: App| {
            let (_, buffer) = render_app(case, &keymap, app, now);
            buffer
                .content()
                .chunks(case.size.0 as usize)
                .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        };
        let all = text(fixture(case, now));
        assert!(all.contains("BANK 3 / 8"), "default shows empty banks");
        app.pattern_view.hide_empty_banks = true;
        let hidden = text(app);
        assert!(hidden.contains("BANK 1 / 8"));
        assert!(hidden.contains("BANK 2 / 8"));
        assert!(hidden.contains("BANK 4 / 8"));
        assert!(!hidden.contains("BANK 3 / 8"));
        assert!(hidden.contains("3:0"), "fixed index keeps the bank");

        let mut selected = fixture(case, now);
        selected.pattern_view.hide_empty_banks = true;
        selected.pads.bank = 2;
        let selected_text = text(selected);
        assert!(selected_text.contains("BANK 3 / 8"));
    }

    /// Scenario "Pattern strip distinguishes subdivisions and event density".
    #[test]
    fn pattern_strip_distinguishes_subdivisions_and_event_density() {
        let (keymap, _) = Keymap::from_toml(DEFAULT_KEYMAP).unwrap();
        let case = Case {
            screen: Screen::Pads,
            mode: Mode::Normal,
            size: (120, 40),
        };
        let raw = session::PatternState {
            slot: "A1".into(),
            bars: 2,
            quantize: "off".into(),
            strength: 100,
            // pad 1 on a line, between lines, twice on one tick; pad 6 of the bank alone
            hits: vec![
                [0, 0, 100, 0],
                [250, 0, 90, 480],
                [480, 0, 80, 0],
                [480, 0, 70, 0],
                [720, 5, 60, 0],
            ],
        };
        let shade = ratatui::style::Color::Rgb(0x1E, 0x22, 0x2A);
        for (finer, label, beat, pad, all) in [
            (
                0,
                "view 1/16",
                4,
                ["■", "~", "2", "·", "·"],
                ["■", "■", "■", "■", "·"],
            ),
            (
                1,
                "view 1/16 triplet",
                6,
                ["■", "~", "·", "2", "·"],
                ["■", "■", "·", "■", "■"],
            ),
        ] {
            let now = Instant::now();
            let mut app = fixture(case, now);
            app.pads.bank = 0;
            app.pads.selected = 0;
            app.pads.pattern = Some(raw.clone());
            for _ in 0..finer {
                app.pattern_view.division.change(true);
            }
            let header = app.pattern_strip(&keymap).header;
            assert!(header.contains(label), "{header}");
            assert_eq!(
                app.pads.pattern.as_ref(),
                Some(&raw),
                "drawing changes no raw row"
            );
            let (_, buffer) = render_app(case, &keymap, app, now);
            let width = case.size.0 as usize;
            let rows: Vec<&[ratatui::buffer::Cell]> = buffer.content().chunks(width).collect();
            let text =
                |row: &[ratatui::buffer::Cell]| row.iter().map(|c| c.symbol()).collect::<String>();
            let at = rows
                .iter()
                .position(|row| text(row).starts_with("all  "))
                .expect("all-pads row");
            assert!(text(rows[at - 1]).contains(label));
            for (row, expected) in [(rows[at], all), (rows[at + 1], pad)] {
                let glyphs: Vec<usize> = (5..width)
                    .filter(|&x| !matches!(row[x].symbol(), " " | "│"))
                    .collect();
                let shown: Vec<&str> = glyphs[..5].iter().map(|&x| row[x].symbol()).collect();
                assert_eq!(shown, expected, "{label}");
                // the second beat lies on the darker plate, the first does not
                assert_ne!(row[glyphs[0]].bg, shade, "{label}");
                assert_eq!(row[glyphs[beat]].bg, shade, "{label}");
                // a two-bar page has one bar line and no line between beats
                assert_eq!(
                    row.iter().filter(|c| c.symbol() == "│").count(),
                    1,
                    "{label}"
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
            let now = Instant::now();
            assert_eq!(
                render_at(case, &keymap, now).0,
                render_at(case, &keymap, now).0,
                "{} is not deterministic",
                case.name()
            );
        }
    }
}

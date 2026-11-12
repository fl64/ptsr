use super::*;
use crate::pads::Pads;
use engine::{Command, Engine, Sample};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Files(PathBuf);

impl Files {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../target/pattern-session-tests/{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn app(&self, project: bool) -> crate::App {
        let mut app = crate::tests::test_app();
        app.session_read_error = None;
        app.session_path = self.0.join("scratch/session.toml");
        if project {
            let folder = self.0.join("project");
            std::fs::create_dir_all(&folder).unwrap();
            app.project = Some(crate::Project {
                name: "project".into(),
                folder,
            });
        }
        app
    }
}

impl Drop for Files {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn pattern_text() -> String {
    "version = 5\nbpm = 172.3\nmetronome = true\npattern = 'A1'\n\n[[patterns]]\nslot = 'A1'\nbars = 1\nquantize = '1/16'\nstrength = 50\nhits = [[121, 0, 90, 960], [5001, 127, 127, 245760], [121, 0, 80, 0]]\n".into()
}

fn invalid_patterns() -> Vec<String> {
    let text = pattern_text();
    let mut cases = Vec::new();
    for version in 1..=4 {
        cases.push(text.replace("version = 5", &format!("version = {version}")));
        cases.push(format!("version = {version}\npattern = 'A1'\n"));
        cases.push(format!(
            "version = {version}\n[[patterns]]\nslot = 'A1'\nbars = 1\nhits = []\n"
        ));
    }
    for (field, values) in [
        (
            "bars = 1",
            vec!["bars = 0", "bars = 65", "bars = 1.0", "bars = '1'"],
        ),
        (
            "quantize = '1/16'",
            vec!["quantize = '1/64T'", "quantize = 'OFF'", "quantize = 16"],
        ),
        (
            "strength = 50",
            vec![
                "strength = -10",
                "strength = 101",
                "strength = 55",
                "strength = 50.0",
            ],
        ),
        (
            "pattern = 'A1'",
            vec!["pattern = 'A2'", "pattern = ''", "pattern = false"],
        ),
        (
            "slot = 'A1'",
            vec!["slot = 'A2'", "slot = 'a1'", "slot = 1"],
        ),
    ] {
        for value in values {
            cases.push(text.replace(field, value));
        }
    }
    let rows = "[[121, 0, 90, 960], [5001, 127, 127, 245760], [121, 0, 80, 0]]";
    for value in [
        "[[0, 0, 127]]",
        "[[0, 0, 127, 0, 0]]",
        "[0, 0, 127, 0]",
        "[[0.0, 0, 127, 0]]",
        "[[0, 0, '127', 0]]",
        "[[0, 0, 127, false]]",
        "[[-1, 0, 127, 0]]",
        "[[245760, 0, 127, 0]]",
        "[[4294967296, 0, 127, 0]]",
        "[[0, -1, 127, 0]]",
        "[[0, 128, 127, 0]]",
        "[[0, 4294967295, 127, 0]]",
        "[[0, 0, 0, 0]]",
        "[[0, 0, 128, 0]]",
        "[[0, 0, 256, 0]]",
        "[[0, 0, 4294967295, 0]]",
        "[[0, 0, 127, -1]]",
        "[[0, 0, 127, 245761]]",
    ] {
        cases.push(text.replace(rows, value));
    }
    for missing in [
        "pattern = 'A1'\n",
        "slot = 'A1'\n",
        "bars = 1\n",
        &format!("hits = {rows}\n"),
    ] {
        cases.push(text.replace(missing, ""));
    }
    cases.push("version = 5\npattern = 'A1'\n".into());
    cases.push(format!(
        "{text}\n[[patterns]]\nslot = 'A1'\nbars = 1\nhits = []\n"
    ));
    cases.push(format!(
        "{text}\n[[patterns]]\nslot = 'A2'\nbars = 1\nhits = []\n"
    ));
    cases.push(format!("{text}skip_lap = 1\n"));
    for value in ["0", "1", "'true'", "[]", "{}"] {
        cases.push(format!("{text}\n[pads.0]\npitch_gate = {value}\n"));
    }
    let overfull = vec!["[0, 0, 127, 0]"; engine::pattern::MAX_HITS + 1].join(",");
    cases.push(text.replace(rows, &format!("[{overfull}]")));
    cases
}

/// Scenario "Older format numbers cannot hide pattern data".
#[test]
fn pattern_production_loaders_reject_malformed_rows_slots_and_ranges() {
    let files = Files::new();
    let path = files.0.join("project.toml");
    for bytes in invalid_patterns() {
        std::fs::write(&path, &bytes).unwrap();
        assert!(load(&path).is_err(), "startup accepted: {bytes}");
        assert!(load_existing(&path).is_err(), "project accepted: {bytes}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), bytes);
    }
}

#[test]
fn pattern_loader_defaults_empty_and_exact_capacity_preserve_raw_order() {
    let files = Files::new();
    let path = files.0.join("project.toml");
    std::fs::write(&path, pattern_text()).unwrap();
    let original = load_existing(&path).unwrap();
    let raw = original.patterns[0].checked().unwrap();
    assert_eq!(raw.hits().len(), 3);
    assert_eq!(raw.hits()[1].tick(), 5001);
    assert_eq!(raw.hits()[1].pad(), 127);
    assert_eq!(raw.compile().events().len(), 1);
    save(&path, &original).unwrap();
    assert_eq!(load_existing(&path).unwrap(), original);
    for grid in [
        "off", "1/4", "1/8", "1/16", "1/32", "1/64", "1/8T", "1/16T", "1/32T",
    ] {
        for strength in (0..=100).step_by(10) {
            let mut s: Session = toml::from_str(&pattern_text()).unwrap();
            s.patterns[0].bars = 64;
            s.patterns[0].quantize = grid.into();
            s.patterns[0].strength = strength;
            s.patterns[0].hits = vec![[245759, 127, 127, 245760]; engine::pattern::MAX_HITS];
            save(&path, &s).unwrap();
            assert_eq!(load_existing(&path).unwrap(), s);
        }
    }
    std::fs::write(
        &path,
        "version = 5\npattern = 'A1'\n[[patterns]]\nslot = 'A1'\nbars = 64\nhits = []\n",
    )
    .unwrap();
    let empty = load_existing(&path).unwrap();
    assert_eq!(empty.patterns[0].quantize, "1/16");
    assert_eq!(empty.patterns[0].strength, 100);
    assert!(
        empty.patterns[0]
            .checked()
            .unwrap()
            .compile()
            .events()
            .is_empty()
    );
    assert!(PadState::default().pitch_gate);
    assert!(Pads::default().to_session(None, 48_000).pads.is_empty());
}

#[test]
fn invalid_in_memory_pattern_cannot_replace_a_valid_saved_file() {
    let files = Files::new();
    let path = files.0.join("project.toml");
    std::fs::write(&path, pattern_text()).unwrap();
    let mut s = load_existing(&path).unwrap();
    let before = std::fs::read(&path).unwrap();
    s.patterns[0].hits.push([0, 128, 100, 0]);
    assert!(save(&path, &s).is_err());
    assert!(save_project(&path, &s).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(!path.with_extension("toml.tmp").exists());
    s.patterns[0].hits.pop();
    s.pattern = None;
    assert!(save(&path, &s).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

/// Scenario "Pattern and Pitch Gate survive project and scratch restart".
#[test]
fn pattern_and_pitch_gate_survive_project_and_scratch_restart() {
    for project in [false, true] {
        let files = Files::new();
        let path = files.0.join("input.toml");
        std::fs::write(&path, format!("{}\n[pads.0]\nfile = 'samples/break.wav'\nname = 'break (copy 3)'\nslice = 2\nlink = 7\nepoch = 3\nstart = 0.1\nend = 0.2\ngroup = 7\npitch_gate = false\n\n[pads.127]\npitch_gate = false\n", pattern_text())).unwrap();
        let source = load_existing(&path).unwrap();
        let mut app = files.app(project);
        let folder = app.project.as_ref().map(|p| p.folder.clone());
        app.apply_session(crate::with_resolved_paths(source, folder.as_deref()));
        app.dirty();
        app.save_session().unwrap();
        assert!(app.session_dirty.is_none());
        let saved = load_existing(&app.session_file()).unwrap();
        assert_eq!(saved.version, 5);
        assert_eq!((saved.bpm, saved.metronome), (172.3, true));
        assert_eq!(saved.pattern.as_deref(), Some("A1"));
        assert_eq!(saved.patterns[0], *app.pads.pattern.as_ref().unwrap());
        assert_eq!(
            saved.patterns[0].hits,
            [[121, 0, 90, 960], [5001, 127, 127, 245760], [121, 0, 80, 0]]
        );
        assert!(!saved.pads["0"].pitch_gate && !saved.pads["127"].pitch_gate);
        assert_eq!(
            saved.pads["0"].file.as_deref(),
            Some(Path::new("samples/break.wav"))
        );
        assert_eq!(saved.pads["0"].name.as_deref(), Some("break (copy 3)"));
        assert_eq!(
            (
                saved.pads["0"].group,
                saved.pads["0"].link,
                saved.pads["0"].epoch
            ),
            (7, Some(7), Some(3))
        );
        let mut reopened = files.app(project);
        reopened.apply_session(crate::with_resolved_paths(saved, folder.as_deref()));
        assert_eq!(reopened.pads.pattern, app.pads.pattern);
        assert_eq!(reopened.pads.pitch_gate, app.pads.pitch_gate);
        assert!(reopened.pads.pitch_gate[17]);
        let clock = reopened.transport_state();
        assert_eq!(
            (clock.playing, clock.ticks, clock.bar_beat()),
            (false, 0.0, (1, 1))
        );
        assert_eq!((clock.bpm, clock.metronome), (172.3, true));
        reopened.apply_session(Session::default());
        assert!(reopened.pads.pattern.is_none());
        assert!(reopened.pads.pitch_gate.into_iter().all(|on| on));
    }
}

fn old_fixture(version: u32) -> String {
    let body = match version {
        1 => {
            "master_db = -3\n[pads.0]\nfile = 'samples/tone.wav'\nvolume_db = -6\n[pads.127]\nfile = '/outside/tone.wav'\nvolume_db = -9\n"
        }
        2 => {
            "master_db = -3\n[pads.0]\nfile = 'samples/break.wav'\nstart = 0.0\nend = 0.25\nslice = 1\n[pads.127]\nfile = 'samples/break.wav'\nstart = 0.25\nend = 0.5\nslice = 2\n"
        }
        3 | 4 => {
            "master_db = -6\nbank = 7\nbrowser_dir = 'samples'\n[pads.0]\nfile = 'samples/break.wav'\nname = 'break (copy 3)'\nstart = 0.0\nend = 0.1\nslice = 1\nlink = 0\nepoch = 3\ngroup = 7\npitch = 3\ngate = true\nattack_ms = 2\nrelease_ms = 100\nfilter = 'low-pass'\ncutoff_hz = 2000.0\nreso = 20\ndrive = 10\nsends = [10, 20]\n[pads.41]\nfile = 'samples/break.wav'\nstart = 0.1\nend = 0.25\nslice = 2\nlink = 7\nepoch = 3\nvolume_db = -9\n[pads.127]\nfile = '/outside/tone.wav'\nvolume_db = -9\n[chops.'samples/break.wav']\nstart = 0.0\nend = 0.5\ncuts = [0.1, 0.25]\nids = [7, 8]\nepoch = 3\nnext_id = 9\n[[kept.'samples/break.wav']]\nstart = 0.0\nend = 0.4\ncuts = [0.2]\nids = [1]\nepoch = 2\nnext_id = 2\n[[fx]]\nkind = 'delay'\ntime_ms = 90\nfeedback = 20\nsize = 50\ntone = 50\n[[fx]]\nkind = 'reverb'\nsize = 60\ntone = 40\ntime_ms = 250\nfeedback = 40\n"
        }
        _ => unreachable!(),
    };
    let transport = if version == 4 {
        "bpm = 92.3\nmetronome = true\n"
    } else {
        ""
    };
    format!("version = {version}\n{transport}{body}")
}

fn manual_render(s: &Session, triggered: &[usize]) -> (Vec<f32>, Vec<f32>) {
    let mut pads = Pads::default();
    let mut engine = Engine::with_sounds(48_000, vec![0.0], vec![Vec::new()]);
    for command in pads.restore(s, 48_000) {
        engine.handle(0, command);
    }
    engine.handle(
        0,
        Command::SetMasterGain {
            gain: 10f32.powf(s.master_db as f32 / 20.0),
        },
    );
    for pad in 0..engine::PADS {
        if pads.file[pad].is_some() {
            let wave = (0..24_000)
                .map(|frame| {
                    let phase = frame as f32 * std::f32::consts::TAU * (220.0 + pad as f32 * 7.0)
                        / 48_000.0;
                    0.02 * phase.sin()
                })
                .collect();
            engine.handle(
                0,
                Command::SetPadSound {
                    pad,
                    sound: Some(Arc::new(Sample::mono(wave))),
                },
            );
            engine.handle(
                0,
                Command::SetPadRegion {
                    pad,
                    region: pads.region[pad],
                },
            );
        }
    }
    // Settle parameter ramps before the reference hit, with no voices or clock running.
    engine.process(&mut [0.0; 128], &mut [0.0; 128]);
    for &pad in triggered {
        engine.handle(
            0,
            Command::PadPress {
                pad,
                velocity: 100.0 / 127.0,
            },
        );
    }
    let mut left = vec![0.0; 8000];
    let mut right = vec![0.0; 8000];
    for (index, (l, r)) in left.chunks_mut(128).zip(right.chunks_mut(128)).enumerate() {
        if index == 8 {
            for &pad in triggered {
                engine.handle(0, Command::PadRelease { pad });
            }
        }
        engine.process(l, r);
    }
    assert!(
        left.iter()
            .chain(&right)
            .all(|s| s.is_finite() && s.abs() < 1.0)
    );
    assert!(left.iter().any(|s| s.abs() > 0.0001));
    assert!(!engine.transport().playing);
    (left, right)
}

/// Scenarios "Format four gains pattern defaults without changing sound" and
/// "Earlier session migrations remain compatible with patterns".
#[test]
fn supported_format_migrations_preserve_manual_sound_and_ungrouped_overlap() {
    let files = Files::new();
    let path = files.0.join("project.toml");
    for version in 1..=4 {
        std::fs::write(&path, old_fixture(version)).unwrap();
        let before = load_existing(&path).unwrap();
        assert_eq!(before.version, version);
        assert!(before.pattern.is_none() && before.patterns.is_empty());
        assert!(before.pads.values().all(|p| p.pitch_gate));
        let ungrouped = if version >= 3 { 41 } else { 0 };
        assert_eq!(
            (
                before.pads[&ungrouped.to_string()].group,
                before.pads["127"].group
            ),
            (0, 0)
        );
        let triggered = if version >= 3 {
            vec![0, ungrouped, 127]
        } else {
            vec![0, 127]
        };
        let reference = manual_render(&before, &triggered);
        let overlap = manual_render(&before, &[ungrouped, 127]);
        let first = manual_render(&before, &[ungrouped]);
        let last = manual_render(&before, &[127]);
        for i in 0..overlap.0.len() {
            assert!(
                (overlap.0[i] - first.0[i] - last.0[i]).abs() <= 0.000001,
                "ungrouped overlap, format {version}, frame {i}"
            );
        }
        save(&path, &before).unwrap();
        let after = load_existing(&path).unwrap();
        assert_eq!(after.version, 5);
        assert!(after.pattern.is_none() && after.patterns.is_empty());
        assert_eq!(after.pads, before.pads);
        assert_eq!(after.chops, before.chops);
        assert_eq!(after.kept, before.kept);
        assert_eq!(after.fx, before.fx);
        assert_eq!(after.browser_dir, before.browser_dir);
        assert_eq!(
            (after.master_db, after.bank),
            (before.master_db, before.bank)
        );
        assert_eq!(
            (after.bpm, after.metronome),
            if version == 4 {
                (92.3, true)
            } else {
                (120.0, false)
            }
        );
        if version == 2 {
            assert_eq!(after.chops["samples/break.wav"].cuts, [0.25]);
            assert_eq!(
                (after.pads["0"].link, after.pads["127"].link),
                (Some(0), Some(1))
            );
        }
        let actual = manual_render(&after, &triggered);
        for (a, b) in reference
            .0
            .iter()
            .chain(&reference.1)
            .zip(actual.0.iter().chain(&actual.1))
        {
            assert!(
                (a - b).abs() <= 0.000001,
                "manual sound changed for format {version}"
            );
        }
    }
}

/// Scenario "Invalid or newer pattern startup files remain protected".
#[test]
fn invalid_or_newer_pattern_startup_files_remain_protected() {
    for project in [false, true] {
        let files = Files::new();
        for bytes in invalid_patterns()
            .into_iter()
            .chain(["version = 6\n".into()])
        {
            let mut app = files.app(project);
            let path = app.session_file();
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &bytes).unwrap();
            let reason = load(&path).unwrap_err();
            app.session_read_error = Some(reason.clone());
            app.pads.pitch_gate[0] = false;
            app.dirty();
            let pending = app.session_dirty;
            let error = app.save_session().unwrap_err();
            assert!(error.contains(&reason));
            app.autosave(
                pending.unwrap() + crate::SESSION_SAVE + std::time::Duration::from_secs(1),
            );
            app.save_on_exit();
            assert_eq!(app.session_dirty, pending);
            assert_eq!(app.session_read_error.as_ref(), Some(&reason));
            assert!(!app.session_write_failed);
            assert_eq!(std::fs::read_to_string(&path).unwrap(), bytes);
        }
    }
}

#[test]
fn pattern_save_failure_and_manual_retry_keep_raw_pending_work() {
    for project in [false, true] {
        let files = Files::new();
        let input = files.0.join("input.toml");
        std::fs::write(&input, pattern_text()).unwrap();
        let mut app = files.app(project);
        app.apply_session(load_existing(&input).unwrap());
        app.save_session().unwrap();
        let path = app.session_file();
        let saved = std::fs::read(&path).unwrap();
        let tmp = path.with_extension("toml.tmp");
        std::fs::create_dir(&tmp).unwrap();
        app.pads.pitch_gate[0] = false;
        app.pads
            .pattern
            .as_mut()
            .unwrap()
            .hits
            .push([240, 7, 120, 480]);
        app.dirty();
        let pending = app.session_dirty;
        app.autosave(pending.unwrap() + crate::SESSION_SAVE + std::time::Duration::from_secs(1));
        assert!(app.session_write_failed);
        assert_eq!(app.session_dirty, pending);
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        std::fs::remove_dir(&tmp).unwrap();
        app.pads.pattern.as_mut().unwrap().strength = 70;
        app.dirty();
        app.autosave(pending.unwrap() + std::time::Duration::from_secs(60));
        app.save_on_exit();
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        assert_eq!(app.session_dirty, pending);
        app.save_session().unwrap();
        assert!(!app.session_write_failed);
        assert!(app.session_dirty.is_none());
        let recovered = load_existing(&path).unwrap();
        assert_eq!(recovered.patterns[0], *app.pads.pattern.as_ref().unwrap());
        assert!(!recovered.pads["0"].pitch_gate);
        app.pads.pattern.as_mut().unwrap().bars = 2;
        app.dirty();
        app.save_on_exit();
        assert_eq!(load_existing(&path).unwrap().patterns[0].bars, 2);
    }
}

#[test]
fn missing_project_folder_keeps_pattern_work_until_manual_retry() {
    let files = Files::new();
    let input = files.0.join("input.toml");
    std::fs::write(&input, pattern_text()).unwrap();
    let mut app = files.app(true);
    app.apply_session(load_existing(&input).unwrap());
    app.save_session().unwrap();
    let folder = app.project.as_ref().unwrap().folder.clone();
    let moved = files.0.join("moved");
    std::fs::rename(&folder, &moved).unwrap();
    let original = std::fs::read(moved.join("project.toml")).unwrap();
    app.pads
        .pattern
        .as_mut()
        .unwrap()
        .hits
        .push([3839, 127, 120, 1000]);
    app.pads.pitch_gate[127] = false;
    app.dirty();
    let pending = app.session_dirty;
    let raw = app.pads.pattern.clone();
    assert!(app.save_session().unwrap_err().contains("is gone"));
    app.autosave(pending.unwrap() + std::time::Duration::from_secs(60));
    app.save_on_exit();
    assert!(app.session_write_failed);
    assert_eq!(app.session_dirty, pending);
    assert_eq!(app.pads.pattern, raw);
    assert!(!app.pads.pitch_gate[127]);
    assert!(!folder.exists());
    assert_eq!(std::fs::read(moved.join("project.toml")).unwrap(), original);
    std::fs::rename(&moved, &folder).unwrap();
    // An explicit retry saves the current pending work, not the last good snapshot.
    app.save_session().unwrap();
    let saved = load_existing(&folder.join("project.toml")).unwrap();
    assert_eq!(saved.patterns, raw.into_iter().collect::<Vec<_>>());
    assert!(!saved.pads["127"].pitch_gate);
    assert!(app.session_dirty.is_none() && !app.session_write_failed);
}

#[test]
fn pattern_target_refusal_precedes_source_save_or_replacement() {
    let files = Files::new();
    let input = files.0.join("input.toml");
    std::fs::write(&input, pattern_text()).unwrap();
    let target = files.0.join("target");
    std::fs::create_dir(&target).unwrap();
    let mut app = files.app(true);
    app.apply_session(load_existing(&input).unwrap());
    app.pads.pitch_gate[127] = false;
    app.save_session().unwrap();
    let original = std::fs::read(app.session_file()).unwrap();
    app.pads.pattern.as_mut().unwrap().strength = 60;
    app.dirty();
    let pending = app.session_dirty;
    let raw = app.pads.pattern.clone();
    let source = app.project.as_ref().unwrap().folder.clone();
    for bytes in invalid_patterns() {
        std::fs::write(target.join("project.toml"), &bytes).unwrap();
        assert!(!app.open_project(target.clone()));
        assert_eq!(app.project.as_ref().unwrap().folder, source);
        assert_eq!(app.pads.pattern, raw);
        assert!(!app.pads.pitch_gate[127]);
        assert_eq!(app.session_dirty, pending);
        assert_eq!(std::fs::read(app.session_file()).unwrap(), original);
        assert_eq!(
            std::fs::read_to_string(target.join("project.toml")).unwrap(),
            bytes
        );
    }
    save_project(&target.join("project.toml"), &Session::default()).unwrap();
    let tmp = app.session_file().with_extension("toml.tmp");
    std::fs::create_dir(&tmp).unwrap();
    assert!(!app.open_project(target.clone()));
    assert_eq!(app.pads.pattern, raw);
    assert_eq!(app.session_dirty, pending);
    assert_eq!(std::fs::read(app.session_file()).unwrap(), original);
    std::fs::remove_dir(tmp).unwrap();
    assert!(app.open_project(target));
    assert!(app.pads.pattern.is_none());
    let source_saved = load_existing(&source.join("project.toml")).unwrap();
    assert_eq!(source_saved.patterns, raw.into_iter().collect::<Vec<_>>());
    assert!(!source_saved.pads["127"].pitch_gate);
}

#[test]
fn pitch_gate_pad_snapshots_copy_clear_reset_and_restore_keep_state() {
    let mut pads = Pads::default();
    pads.pitch_gate[0] = false;
    let off = pads.snapshot(0);
    pads.pitch_gate[0] = true;
    assert_ne!(off, pads.snapshot(0));
    pads.put_back(0, &off);
    pads.put_back(127, &off);
    assert!(!pads.pitch_gate[0] && !pads.pitch_gate[127]);
    pads.swap(0, 1).unwrap();
    assert!(pads.pitch_gate[0] && !pads.pitch_gate[1]);
    pads.clear(1);
    assert!(!pads.pitch_gate[1]);
    pads.reset_clean(1);
    assert!(pads.pitch_gate[1]);
    pads.put_back(1, &off);
    let s = pads.to_session(None, 48_000);
    assert!(!s.pads["1"].pitch_gate && !s.pads["127"].pitch_gate);
    let mut restored = Pads::default();
    restored.restore(&s, 48_000);
    assert_eq!(restored.pitch_gate, pads.pitch_gate);
    restored.restore(&Session::default(), 48_000);
    assert!(restored.pitch_gate.into_iter().all(|on| on));
}

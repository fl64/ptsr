use super::{App, Project, SESSION_SAVE, session, tests::test_app};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Files(PathBuf);

impl Files {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../target/save-tests/{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn app(&self, project: bool) -> App {
        let mut app = test_app();
        app.session_read_error = None;
        app.session_path = self.0.join("scratch/session.toml");
        if project {
            let folder = self.0.join("project");
            std::fs::create_dir(&folder).unwrap();
            app.project = Some(Project {
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

fn block_save(app: &App) -> PathBuf {
    let tmp = app.session_file().with_extension("toml.tmp");
    std::fs::write(&tmp, b"keep this file").unwrap();
    tmp
}

#[test]
fn failed_autosave_retains_pending_work() {
    for project in [false, true] {
        let files = Files::new();
        let mut app = files.app(project);
        app.save_session().unwrap(); // Control: the same folder accepts a save.
        let path = app.session_file();
        let saved = std::fs::read(&path).unwrap();
        let tmp = block_save(&app);
        app.pads.pitch[0] = 3;
        app.dirty();
        let pending = app.session_dirty;
        let now = pending.unwrap();
        app.autosave(now + SESSION_SAVE);
        assert!(
            !app.session_write_failed,
            "keep the existing strict time boundary"
        );
        app.autosave(now + SESSION_SAVE + std::time::Duration::from_nanos(1));
        assert!(app.session_write_failed);
        assert!(app.session_read_error.is_none());
        assert_eq!(app.session_dirty, pending);
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        assert_eq!(std::fs::read(&tmp).unwrap(), b"keep this file");
        app.pads.pitch[1] = 7;
        app.dirty();
        std::fs::remove_file(tmp).unwrap();
        app.flash = None;
        app.autosave(now + std::time::Duration::from_secs(60));
        assert!(app.flash.is_none(), "no automatic retry or repeated notice");
        assert_eq!(app.session_dirty, pending);
        assert_eq!(std::fs::read(path).unwrap(), saved);
        assert_eq!((app.pads.pitch[0], app.pads.pitch[1]), (3, 7));
    }
}

#[test]
fn manual_retry_restores_autosave() {
    for project in [false, true] {
        let files = Files::new();
        let mut app = files.app(project);
        app.save_session().unwrap();
        let tmp = block_save(&app);
        app.pads.pitch[0] = 3;
        app.dirty();
        assert!(app.save_session().is_err());
        app.pads.pitch[1] = 7;
        app.dirty();
        std::fs::remove_file(tmp).unwrap();
        app.save_session().unwrap();
        assert!(!app.session_write_failed);
        assert!(app.session_dirty.is_none());
        let stored = session::load(&app.session_file()).unwrap();
        assert_eq!((stored.pads["0"].pitch, stored.pads["1"].pitch), (3, 7));
        app.pads.pitch[1] = 9;
        app.dirty();
        app.flash = None;
        app.autosave(app.session_dirty.unwrap() + SESSION_SAVE + std::time::Duration::from_secs(1));
        assert!(app.session_dirty.is_none());
        assert!(app.flash.is_none(), "successful autosave stays silent");
        assert_eq!(
            session::load(&app.session_file()).unwrap().pads["1"].pitch,
            9
        );
    }
}

#[test]
fn a_failed_save_does_not_retry_on_exit() {
    let files = Files::new();
    let mut app = files.app(false);
    app.save_session().unwrap();
    let saved = std::fs::read(app.session_file()).unwrap();
    let tmp = block_save(&app);
    app.pads.pitch[0] = 3;
    app.dirty();
    assert!(app.save_session().is_err());
    std::fs::remove_file(tmp).unwrap();
    app.flash = None;
    app.save_on_exit();
    assert!(app.flash.is_none());
    assert!(app.session_dirty.is_some());
    assert_eq!(std::fs::read(app.session_file()).unwrap(), saved);
    app.save_session().unwrap();
    app.pads.pitch[0] = 5;
    app.dirty();
    app.save_on_exit();
    assert_eq!(
        session::load(&app.session_file()).unwrap().pads["0"].pitch,
        5
    );
}

#[test]
fn damaged_or_newer_session_remains_protected() {
    for project in [false, true] {
        for bytes in [b"version = [\n".as_slice(), b"version = 99\n", b"\xff"] {
            let files = Files::new();
            let mut app = files.app(project);
            let path = app.session_file();
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, bytes).unwrap();
            if project {
                app.user_path = Some(files.0.join("config.toml"));
                app.open_project(path.parent().unwrap().to_path_buf());
            } else {
                app.session_read_error = Some(session::load(&path).unwrap_err());
            }
            let reason = app.session_read_error.clone().unwrap();
            app.pads.pitch[0] = 3;
            app.dirty();
            let pending = app.session_dirty;
            for _ in 0..2 {
                app.flash = None;
                let error = app.save_session().unwrap_err();
                assert!(error.contains(&reason));
                assert_eq!(app.flash.as_ref().unwrap().0, error);
                assert_eq!(app.session_read_error.as_ref(), Some(&reason));
                assert_eq!(app.session_dirty, pending);
                assert!(
                    !app.session_write_failed,
                    "read protection is not a write failure"
                );
                assert_eq!(std::fs::read(&path).unwrap(), bytes);
                app.dirty();
            }
            app.save_on_exit();
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
    }
}

#[test]
fn saving_does_not_recreate_a_deleted_project() {
    let files = Files::new();
    let mut app = files.app(true);
    app.save_session().unwrap();
    let folder = app.project.as_ref().unwrap().folder.clone();
    app.pads.file[0] = Some(folder.join("sample.wav"));
    app.pads.file[1] = Some(files.0.join("outside.wav"));
    std::fs::remove_dir_all(&folder).unwrap();
    app.dirty();
    let pending = app.session_dirty;
    for _ in 0..2 {
        app.flash = None;
        assert!(app.save_session().unwrap_err().contains("is gone"));
        assert!(app.flash.is_some());
        assert!(!folder.exists());
        assert_eq!(app.session_dirty, pending);
    }
    assert!(app.pads.missing[0]);
    assert!(!app.pads.missing[1]);
    assert_eq!(app.window_title(), "ptsr");
    assert!(app.session_read_error.is_none());
    std::fs::create_dir(&folder).unwrap();
    app.save_session().unwrap();
    assert!(app.session_dirty.is_none());
    assert!(!app.session_write_failed);
}

#[test]
fn an_occupied_temporary_path_preserves_the_saved_file() {
    use std::os::unix::fs::symlink;

    for kind in 0..5 {
        let files = Files::new();
        let path = files.0.join("session.toml");
        session::save(&path, &session::Session::default()).unwrap();
        let saved = std::fs::read(&path).unwrap();
        let tmp = path.with_extension("toml.tmp");
        let other = files.0.join("other");
        std::fs::write(&other, b"unrelated").unwrap();
        match kind {
            0 => std::fs::write(&tmp, b"occupied").unwrap(),
            1 => symlink(&path, &tmp).unwrap(),
            2 => symlink(&other, &tmp).unwrap(),
            3 => std::fs::hard_link(&path, &tmp).unwrap(),
            _ => std::fs::create_dir(&tmp).unwrap(),
        }
        assert!(
            session::save(
                &path,
                &session::Session {
                    master_db: -6,
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        assert_eq!(std::fs::read(&other).unwrap(), b"unrelated");
        assert!(
            std::fs::symlink_metadata(&tmp).is_ok(),
            "do not remove someone else's path"
        );
        if kind == 0 {
            assert_eq!(std::fs::read(tmp).unwrap(), b"occupied");
        }
    }
}

#[test]
fn replacement_refusal_removes_only_its_temporary_file() {
    let files = Files::new();
    let path = files.0.join("destination.toml");
    std::fs::create_dir(&path).unwrap();
    std::fs::write(path.join("keep"), b"unrelated").unwrap();
    assert!(session::save(&path, &session::Session::default()).is_err());
    assert_eq!(std::fs::read(path.join("keep")).unwrap(), b"unrelated");
    assert!(!path.with_extension("toml.tmp").exists());
}

#[test]
fn an_unreadable_session_stays_protected() {
    let files = Files::new();
    let mut app = files.app(false);
    let path = app.session_file();
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("keep"), b"unrelated").unwrap();
    app.session_read_error = Some(session::load(&path).unwrap_err());
    app.dirty();
    for _ in 0..2 {
        assert!(app.save_session().is_err());
        assert!(app.session_dirty.is_some());
        assert_eq!(std::fs::read(path.join("keep")).unwrap(), b"unrelated");
    }
}

#[test]
fn scratch_creates_its_parent_but_project_saving_does_not() {
    let files = Files::new();
    let path = files.0.join("absent/session.toml");
    assert!(session::save_project(&path, &session::Session::default()).is_err());
    assert!(!path.parent().unwrap().exists());
    session::save(&path, &session::Session::default()).unwrap();
    assert!(path.is_file());
}

//! Change 138: a new project with a date name from the SMC-PAD when no project exists.

use super::*;

/// A drive whose projects root holds no project.
fn empty_root() -> (Drive, PathBuf) {
    let root = TransitionFiles::new().0.join("empty");
    std::fs::create_dir_all(&root).unwrap();
    let mut drive = hold_drive();
    drive.app.paths.projects = root.clone();
    drive.app.project = None;
    drive.app.user_path = Some(root.join("config.toml"));
    (drive, root)
}

/// The project picker from the screens view, as the SMC-PAD opens it.
fn open_picker(drive: &mut Drive) {
    hold(drive, true);
    list_button(drive, false);
    let open = drive
        .app
        .panel_snapshot()
        .unwrap()
        .labels
        .iter()
        .position(|label| label.as_deref() == Some("open"))
        .expect("open in the screens view");
    cell(drive, panel_note(u8::try_from(open + 1).unwrap()));
    assert!(drive.app.project_picker.is_some());
}

fn today() -> String {
    let stamp = crate::stamp(std::time::SystemTime::now());
    format!("{}-{}-{}", &stamp[..4], &stamp[4..6], &stamp[6..8])
}

/// Scenario "New project with no projects" (projects).
#[test]
fn new_project_with_no_projects() {
    let (mut drive, root) = empty_root();
    open_picker(&mut drive);
    assert_eq!(
        drive.app.panel_snapshot().unwrap().labels[15].as_deref(),
        Some("new project")
    );
    cell(&mut drive, panel_note(16));
    hold(&mut drive, false);
    let name = format!("{}-1", today());
    assert_eq!(
        drive.app.project.as_ref().map(|p| p.name.clone()),
        Some(name.clone())
    );
    assert!(root.join(&name).join("project.toml").is_file());
    assert!(drive.app.project_picker.is_none());
}

/// Scenario "The date name takes the next number" (projects).
#[test]
fn the_date_name_takes_the_next_number() {
    let at = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_795_132_800 + 3_600);
    let (mut drive, root) = empty_root();
    assert_eq!(crate::browser::date_project_name(&root, at), "2026-11-20-1");
    std::fs::create_dir_all(root.join("2026-11-20-1")).unwrap();
    assert_eq!(crate::browser::date_project_name(&root, at), "2026-11-20-2");

    std::fs::create_dir_all(root.join(format!("{}-1", today()))).unwrap();
    open_picker(&mut drive);
    cell(&mut drive, panel_note(16));
    hold(&mut drive, false);
    let name = format!("{}-2", today());
    assert_eq!(
        drive.app.project.as_ref().map(|p| p.name.clone()),
        Some(name)
    );
}

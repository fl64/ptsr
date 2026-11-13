//! Files screen state: the current folder, its folders and audio files, the selection,
//! and the file "in hand".

use std::path::{Path, PathBuf};

use crate::loader::EXTENSIONS;

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    /// A favorite folder shown on top of every listing; Enter goes straight there.
    pub favorite: Option<PathBuf>,
}

pub struct Browser {
    pub dir: PathBuf,
    /// Everything in the folder.
    all: Vec<Entry>,
    /// What is shown: `all` narrowed by the filter.
    pub entries: Vec<Entry>,
    /// Shown entries contain this text, case aside; cleared when the folder changes.
    pub filter: String,
    /// The filter line is open: keys type text instead of playing pads.
    pub editing: bool,
    pub favorites: Vec<PathBuf>,
    pub selected: usize,
    /// Taken with Enter; the next pad hit puts it on that pad.
    pub in_hand: Option<PathBuf>,
    /// Why the folder could not be read, if it could not.
    pub error: Option<String>,
}

impl Browser {
    /// Opens `dir`, or `~/Music`, or the home folder, whichever exists first.
    pub fn new(dir: Option<PathBuf>) -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| "/".into());
        let dir = dir
            .filter(|d| d.is_dir())
            .or_else(|| Some(home.join("Music")).filter(|d| d.is_dir()))
            .unwrap_or(home);
        let mut b = Self {
            dir: PathBuf::new(),
            all: Vec::new(),
            entries: Vec::new(),
            filter: String::new(),
            editing: false,
            favorites: Vec::new(),
            selected: 0,
            in_hand: None,
            error: None,
        };
        b.open(dir, None);
        b
    }

    /// Lists `dir`: folders first, then audio files, by name; hidden entries are skipped.
    /// `select` puts the selection on that name if it is there.
    fn open(&mut self, dir: PathBuf, select: Option<&str>) {
        let mut entries: Vec<Entry> = match std::fs::read_dir(&dir) {
            Ok(rd) => {
                self.error = None;
                rd.flatten()
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().into_owned();
                        let is_dir = e.path().is_dir();
                        let audio = Path::new(&name)
                            .extension()
                            .and_then(|x| x.to_str())
                            .is_some_and(|x| EXTENSIONS.contains(&x.to_lowercase().as_str()));
                        (!name.starts_with('.') && (is_dir || audio)).then_some(Entry {
                            name,
                            is_dir,
                            favorite: None,
                        })
                    })
                    .collect()
            }
            Err(e) => {
                self.error = Some(format!("{}: {e}", dir.display()));
                Vec::new()
            }
        };
        entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        self.all = entries;
        self.dir = dir;
        self.filter.clear();
        self.editing = false;
        self.apply_filter(select);
    }

    /// A favorite shown on top of a listing, not the folder's own line here.
    fn is_shortcut(&self, e: &Entry) -> bool {
        e.favorite
            .as_ref()
            .is_some_and(|p| *p != self.dir.join(&e.name))
    }

    fn apply_filter(&mut self, select: Option<&str>) {
        let f = self.filter.to_lowercase();
        // a favorite inside this folder is not repeated on top: its own line wears the ★ (35)
        let here = |p: &PathBuf| p.parent() == Some(self.dir.as_path());
        let favorites = self
            .favorites
            .iter()
            .filter(|p| **p != self.dir && !here(p))
            .map(|p| Entry {
                name: p.file_name().map_or_else(
                    || p.display().to_string(),
                    |n| n.to_string_lossy().into_owned(),
                ),
                is_dir: true,
                favorite: Some(p.clone()),
            });
        let own = self.all.iter().cloned().map(|mut e| {
            let path = self.dir.join(&e.name);
            if e.is_dir && self.favorites.contains(&path) {
                e.favorite = Some(path);
            }
            e
        });
        let entries: Vec<Entry> = favorites
            .chain(own)
            .filter(|e| e.name.to_lowercase().contains(&f))
            .collect();
        self.entries = entries;
        self.selected = select
            .and_then(|s| {
                self.entries
                    .iter()
                    .position(|e| !self.is_shortcut(e) && e.name == s)
            })
            .unwrap_or(0);
    }

    /// Changes the filter; the selection stays on the same entry if it is still shown.
    pub fn set_filter(&mut self, filter: String) {
        let keep = self.current().map(|e| e.name.clone());
        self.filter = filter;
        self.apply_filter(keep.as_deref());
    }

    pub fn current(&self) -> Option<&Entry> {
        self.entries.get(self.selected)
    }

    pub fn step(&mut self, delta: isize) {
        if !self.entries.is_empty() {
            self.selected =
                (self.selected as isize + delta).clamp(0, self.entries.len() as isize - 1) as usize;
        }
    }

    /// Reads the folder again (66): a file saved there meanwhile shows; the selection and
    /// the filter stay.
    pub fn reread(&mut self) {
        let select = self.current().map(|e| e.name.clone());
        let filter = std::mem::take(&mut self.filter);
        self.open(self.dir.clone(), select.as_deref());
        if !filter.is_empty() {
            self.filter = filter;
            self.apply_filter(select.as_deref());
        }
    }

    /// Drops the filter (66); `false` when there was none.
    pub fn clear_filter(&mut self) -> bool {
        if self.filter.is_empty() {
            return false;
        }
        let select = self.current().map(|e| e.name.clone());
        self.filter.clear();
        self.editing = false;
        self.apply_filter(select.as_deref());
        true
    }

    /// Into the selected folder.
    pub fn enter(&mut self) {
        if let Some(e) = self.current().filter(|e| e.is_dir) {
            let dir = e.favorite.clone().unwrap_or_else(|| self.dir.join(&e.name));
            self.open(dir, None);
        }
    }

    /// Ctrl+B: the selected folder, or the folder being shown when a file is selected, becomes
    /// a favorite or stops being one. Returns the folder name and whether it is a favorite now.
    pub fn toggle_favorite(&mut self) -> (String, bool) {
        let (path, keep) = match self.current() {
            Some(e) if e.is_dir => (
                e.favorite.clone().unwrap_or_else(|| self.dir.join(&e.name)),
                (!self.is_shortcut(e)).then(|| e.name.clone()),
            ),
            e => (self.dir.clone(), e.map(|e| e.name.clone())),
        };
        let added = match self.favorites.iter().position(|p| *p == path) {
            Some(i) => {
                self.favorites.remove(i);
                false
            }
            None => {
                self.favorites.push(path.clone());
                true
            }
        };
        self.apply_filter(keep.as_deref());
        let name = path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        (name, added)
    }

    /// Up to the parent folder, with the folder we left selected.
    pub fn parent(&mut self) {
        let Some(parent) = self.dir.parent().map(Path::to_path_buf) else {
            return;
        };
        let left = self
            .dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned());
        self.open(parent, left.as_deref());
    }

    /// Enter on the selection: a folder opens, a file goes in hand.
    pub fn take(&mut self) {
        match self.current() {
            Some(e) if e.is_dir => self.enter(),
            Some(e) => self.in_hand = Some(self.dir.join(&e.name)),
            None => {}
        }
    }

    /// The selected file, or `None` when the selection is a folder (or the list is empty).
    pub fn selected_file(&self) -> Option<PathBuf> {
        self.current()
            .filter(|e| !e.is_dir)
            .map(|e| self.dir.join(&e.name))
    }
}

use crate::*;

impl App {
    /// The keys of the open folder picker (11): Up/Down choose, Right enters a folder,
    /// Left goes up (at `/` it stays), Enter picks the folder it is in, Esc cancels;
    /// the rest do nothing.
    pub(super) fn picker_key(&mut self, action: Action) {
        let Some(mut picker) = self.path_picker.take() else {
            return;
        };
        match action {
            Action::SettingsRow(step) => picker.move_cursor(step as isize),
            Action::SettingsValue(step) => {
                if step > 0 {
                    if let Some(dir) = picker.entries.get(picker.selected).cloned() {
                        picker.at = dir;
                        picker.entries = folders_of(&picker.at);
                        picker.selected = 0;
                    }
                } else if let Some(parent) = picker.at.parent().map(Path::to_path_buf) {
                    let from = picker.at.file_name().map(|n| n.to_owned());
                    picker.at = parent;
                    picker.entries = folders_of(&picker.at);
                    // the folder we came from stays under the cursor, as the files screen does
                    picker.selected = from
                        .and_then(|n| {
                            picker
                                .entries
                                .iter()
                                .position(|e| e.file_name() == Some(&n))
                        })
                        .unwrap_or(0);
                }
            }
            Action::SettingsLearn => {
                let (which, at) = (picker.which, picker.at.clone());
                self.paths_set(which, at);
                return; // picked: the picker stays closed
            }
            Action::Back => return, // cancelled: the picker stays closed
            _ => {}
        }
        self.path_picker = Some(picker);
    }

    pub(super) fn save_before_project_transition(&mut self) -> bool {
        self.session_dirty.is_none() || self.save_session().is_ok()
    }

    /// Validate the target before saving or replacing current work. Startup keeps its
    /// separate empty-session fallback for an absent session file.
    pub(super) fn open_project(&mut self, folder: PathBuf) -> bool {
        if self.refuse_pattern_project_change() {
            return false;
        }
        let same = self.project.as_ref().is_some_and(|p| {
            p.folder == folder
                || std::fs::canonicalize(&p.folder)
                    .ok()
                    .zip(std::fs::canonicalize(&folder).ok())
                    .is_some_and(|(current, target)| current == target)
        });
        if same {
            return true;
        }
        let sess = match session::load_existing(&folder.join("project.toml")) {
            Ok(s) => s,
            Err(e) => {
                log::error!("{e}");
                self.post_notice(
                    Level::Error,
                    (format!("Project not opened: {e}"), Instant::now()),
                );
                return false;
            }
        };
        if !self.save_before_project_transition() {
            return false;
        }
        self.session_read_error = None;
        let sess = with_resolved_paths(sess, Some(&folder));
        let name = file_name(&folder);
        self.project = Some(Project {
            name: name.clone(),
            folder,
        });
        self.apply_session(sess);
        self.post_notice(Level::Done, (format!("Project {name}"), Instant::now()));
        self.project_cfg.last = Some(name);
        self.save_project_cfg();
        true
    }

    /// Save pending source work before creating any target files. Existing project
    /// names use the same guarded open path; refusals keep the picker name available.
    pub(super) fn create_project(&mut self, name: String) -> bool {
        if self.refuse_pattern_project_change() {
            return false;
        }
        if name.trim().is_empty() {
            return false;
        }
        let folder = self.paths.projects.join(&name);
        if folder.join("project.toml").is_file() {
            return self.open_project(folder);
        }
        if !self.save_before_project_transition() {
            return false;
        }
        let result = std::fs::create_dir_all(&folder)
            .and_then(|()| std::fs::create_dir_all(folder.join("samples")))
            .map_err(|e| format!("{}: {e}", folder.display()))
            .and_then(|()| {
                session::save_project(&folder.join("project.toml"), &session::Session::default())
            });
        if let Err(e) = result {
            log::error!("{e}");
            self.post_notice(Level::Error, (e, Instant::now()));
            return false;
        }
        self.open_project(folder)
    }
    /// A key while the filter line is open; `false` lets it through (releases, Ctrl+key).
    pub(super) fn filter_key(&mut self, k: &event::KeyEvent) -> bool {
        use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
        if k.kind == KeyEventKind::Release || k.modifiers.contains(KeyModifiers::CONTROL) {
            return false;
        }
        let mut filter = self.browser.filter.clone();
        match k.code {
            KeyCode::Esc => {
                filter.clear();
                self.browser.editing = false;
            }
            KeyCode::Enter => self.browser.editing = false,
            KeyCode::Backspace => {
                filter.pop();
            }
            KeyCode::Char(c) => filter.push(c),
            // modifier keys and the like
            _ => return true,
        }
        if filter != self.browser.filter {
            self.browser.set_filter(filter);
            self.preview_selection();
        }
        true
    }

    /// A key while the project picker is open; `false` lets it through (releases, Ctrl+key).
    /// A character with the list on screen starts the name of a new project; the name line
    /// then takes every character until Enter (create and open) or Esc (back to the list).
    pub(super) fn project_picker_key(&mut self, k: &event::KeyEvent) -> bool {
        use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
        if k.kind == KeyEventKind::Release || k.modifiers.contains(KeyModifiers::CONTROL) {
            return false;
        }
        let Some(mut picker) = self.project_picker.take() else {
            return false;
        };
        if let Some(name) = &mut picker.naming {
            match k.code {
                KeyCode::Esc => picker.naming = None,
                KeyCode::Enter => {
                    if self.create_project(name.clone()) {
                        return true;
                    }
                }
                KeyCode::Backspace => {
                    name.pop();
                }
                KeyCode::Char(c) => name.push(c),
                // modifier keys and the like
                _ => {
                    self.project_picker = Some(picker);
                    return true;
                }
            }
            self.project_picker = Some(picker);
            return true;
        }
        match k.code {
            KeyCode::Up => picker.selected = picker.selected.saturating_sub(1),
            KeyCode::Down => {
                picker.selected = (picker.selected + 1).min(picker.entries.len().saturating_sub(1));
            }
            KeyCode::Enter => {
                if let Some(name) = picker.entries.get(picker.selected).cloned()
                    && self.open_project(self.paths.projects.join(name))
                {
                    return true;
                }
            }
            KeyCode::Esc => return true, // closed, nothing happens
            // any character is the start of a new project's name
            KeyCode::Char(c) => picker.naming = Some(c.to_string()),
            _ => {
                self.project_picker = Some(picker);
                return true;
            }
        }
        self.project_picker = Some(picker);
        true
    }

    /// (11) The window title: the open project joins the program's name while its folder
    /// is still on disk; a deleted folder drops the name.
    pub(super) fn window_title(&self) -> String {
        match self.project.as_ref().filter(|p| p.folder.is_dir()) {
            Some(p) => format!("ptsr — {}", p.name),
            None => "ptsr".into(),
        }
    }

    pub(super) fn stop_preview(&mut self) {
        // a load still on its way for the old selection becomes stale
        self.load_gen[PREVIEW] += 1;
        self.preview = None;
        if self.loop_sent.take().is_some() {
            self.send(Command::PreviewLoop { region: None });
        }
        self.send(Command::Preview {
            sound: None,
            region: Region::WHOLE,
        });
    }

    /// Plays the selected file if it is an audio file and the preview is on.
    pub(super) fn preview_selection(&mut self) {
        let file = self
            .browser
            .current()
            .filter(|e| !e.is_dir)
            .map(|e| self.browser.dir.join(&e.name));
        match file {
            Some(path) if self.preview_on && self.screen == Screen::Files => {
                self.load_gen[PREVIEW] += 1;
                self.preview = None;
                if let Some((tx, _)) = &self.loader {
                    let _ = tx.send(loader::Request {
                        pad: PREVIEW,
                        path,
                        generation: self.load_gen[PREVIEW],
                    });
                }
            }
            _ => self.stop_preview(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folders_first_audio_only_and_back_to_the_same_folder() {
        let root = std::env::temp_dir().join(format!("ptsr-browser-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for d in ["breaks", "Amen", ".hidden"] {
            std::fs::create_dir_all(root.join(d)).unwrap();
        }
        for f in ["kick.WAV", "notes.txt", "b.flac", ".x.wav"] {
            std::fs::write(root.join(f), b"").unwrap();
        }
        std::fs::write(root.join("breaks/amen.wav"), b"").unwrap();

        let mut b = Browser::new(Some(root.clone()));
        let names: Vec<_> = b.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["Amen", "breaks", "b.flac", "kick.WAV"]);

        b.step(1);
        b.take();
        assert_eq!(b.dir, root.join("breaks"));
        b.take();
        assert_eq!(b.in_hand, Some(root.join("breaks/amen.wav")));
        b.parent();
        assert_eq!(b.current().map(|e| e.name.as_str()), Some("breaks"));

        b.step(-10);
        assert_eq!(b.selected, 0);
        b.step(10);
        assert_eq!(b.selected, 3);

        // filter: case aside, folders too; cleared when the folder changes
        b.set_filter("B".into());
        let names: Vec<_> = b.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["breaks", "b.flac"]);
        b.set_filter("amen".into());
        assert_eq!(b.entries.len(), 1);
        b.set_filter("zzz".into());
        assert!(b.entries.is_empty() && b.current().is_none());
        b.set_filter(String::new());
        b.step(1);
        b.enter();
        assert!(b.filter.is_empty());

        // favorites: on top everywhere, Enter goes there, Ctrl+B again removes
        b.parent();
        let at = b.entries.iter().position(|e| e.name == "breaks").unwrap();
        b.step(at as isize - b.selected as isize);
        assert_eq!(b.toggle_favorite(), ("breaks".into(), true));
        assert_eq!(b.favorites, [root.join("breaks")]);
        b.enter();
        // inside the favorite itself it is not listed again
        assert!(b.entries.iter().all(|e| e.favorite.is_none()));
        // in the folder that holds it: one line, with its ★, and back on it after ← (35)
        b.parent();
        let lines: Vec<_> = b.entries.iter().filter(|e| e.name == "breaks").collect();
        assert_eq!(lines.len(), 1, "listed once");
        assert_eq!(lines[0].favorite, Some(root.join("breaks")), "with its ★");
        assert_eq!(b.current().map(|e| e.name.as_str()), Some("breaks"));
        assert_eq!(b.toggle_favorite(), ("breaks".into(), false));
        assert!(b.favorites.is_empty());
        // on a file: the folder being shown, and the file stays selected
        let at = b.entries.iter().position(|e| e.name == "breaks").unwrap();
        b.step(at as isize - b.selected as isize);
        b.enter();
        let file = b.current().map(|e| e.name.clone());
        assert_eq!(b.toggle_favorite(), ("breaks".into(), true));
        assert_eq!(b.favorites, [root.join("breaks")]);
        assert_eq!(b.current().map(|e| e.name.clone()), file);

        let missing = Browser::new(Some(root.join("nope")));
        assert_ne!(
            missing.dir,
            root.join("nope"),
            "falls back to ~/Music or home"
        );
        std::fs::remove_dir_all(&root).unwrap();
    }
}

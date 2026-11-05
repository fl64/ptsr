//! Session saving and retry scheduling on the UI thread.

use super::{App, Instant, PADS, PathBuf, SESSION_SAVE, session, tilde, with_stored_paths};

impl App {
    pub(super) fn dirty(&mut self) {
        self.session_dirty.get_or_insert_with(Instant::now);
    }

    pub(super) fn session_file(&self) -> PathBuf {
        self.project.as_ref().map_or_else(
            || self.session_path.clone(),
            |p| p.folder.join("project.toml"),
        )
    }

    fn automatic_save_allowed(&self) -> bool {
        self.session_read_error.is_none()
            && !self.session_write_failed
            && self.session_dirty.is_some()
    }

    pub(super) fn autosave(&mut self, now: Instant) {
        if self.automatic_save_allowed()
            && self
                .session_dirty
                .is_some_and(|at| now.saturating_duration_since(at) > SESSION_SAVE)
        {
            let _ = self.save_session();
        }
    }

    pub(super) fn save_on_exit(&mut self) {
        if self.automatic_save_allowed() {
            let _ = self.save_session();
        }
    }

    /// Only a successful replacement clears pending work and permits automatic saves.
    pub(super) fn save_session(&mut self) -> Result<(), String> {
        let result = match &self.session_read_error {
            Some(reason) => Err(format!("Session not saved: {reason}")),
            None => self.write_session(),
        };
        match &result {
            Ok(()) => {
                self.session_dirty = None;
                self.session_write_failed = false;
            }
            Err(reason) => {
                if self.session_read_error.is_none() {
                    self.session_write_failed = true;
                }
                log::error!("{reason}");
                self.flash = Some((reason.clone(), Instant::now()));
            }
        }
        result
    }

    fn write_session(&mut self) -> Result<(), String> {
        if let Some(folder) = self
            .project
            .as_ref()
            .map(|p| &p.folder)
            .filter(|f| !f.is_dir())
        {
            for abs in 0..PADS {
                if self.pads.file[abs]
                    .as_ref()
                    .is_some_and(|f| f.starts_with(folder))
                {
                    self.pads.missing[abs] = true;
                }
            }
            return Err(format!(
                "{} is gone; the session is not saved",
                tilde(&folder.display().to_string())
            ));
        }
        let mut s = self
            .pads
            .to_session(Some(self.browser.dir.clone()), self.rate);
        s.master_db = self.master_db;
        let open = self
            .chop
            .iter()
            .chain(self.chops.values())
            .chain(self.older.iter());
        for c in open {
            let key = c.file.to_string_lossy().into_owned();
            if self.is_newest(c) {
                s.chops.insert(key, c.saved().to_state(self.rate));
            } else if self.linked_to(&c.file, c.epoch) {
                s.kept
                    .entry(key)
                    .or_default()
                    .push(c.saved().to_state(self.rate));
            }
            // A deleted cut's pad uses the cut that now starts its slice.
            for p in s.pads.values_mut() {
                if p.file.as_ref() == Some(&c.file)
                    && p.epoch == Some(c.epoch)
                    && let Some(id) = p.link
                {
                    p.link = Some(c.start_id(c.slice_of(id)));
                }
            }
        }
        for (file, saved) in &self.pending_chops {
            s.chops
                .entry(file.to_string_lossy().into_owned())
                .or_insert_with(|| saved.to_state(self.rate));
        }
        for (file, saved) in &self.pending_older {
            if self.linked_to(file, saved.epoch) {
                s.kept
                    .entry(file.to_string_lossy().into_owned())
                    .or_default()
                    .push(saved.to_state(self.rate));
            }
        }
        if let Some(p) = &self.project {
            s = with_stored_paths(s, &p.folder);
            session::save_project(&self.session_file(), &s)
        } else {
            session::save(&self.session_file(), &s)
        }
    }
}

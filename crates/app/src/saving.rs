//! Session saving and retry scheduling on the UI thread.

use super::{
    App, Ask, Instant, Level, Notice, NoticeToken, Origin, PADS, PathBuf, SESSION_SAVE, session,
    tilde, with_stored_paths,
};
use std::sync::{Arc, atomic::Ordering::Acquire};

pub(super) struct PendingSave {
    project: Arc<engine::rt::ProjectState>,
    epoch: u64,
    origin: Origin,
    progress: NoticeToken,
}

pub(super) enum SaveNotice {
    Refused(NoticeToken),
    RecoveredUntil(Instant),
}

impl App {
    pub(super) fn request_quit(&mut self) {
        if matches!(self.ask, Some(Ask::ExitRecovery { .. })) {
            return;
        }
        self.quit_requested = true;
        self.flush_audio();
    }

    pub(super) fn try_finish_exit(&mut self) -> bool {
        if !self.quit_ready() || self.pending_manual_save.is_some() {
            return false;
        }
        if self.session_dirty.is_none() {
            return true;
        }

        let refusal = self
            .session_read_error
            .as_ref()
            .map(|reason| format!("Session not saved: {reason}"))
            .or_else(|| self.session_write_error.clone())
            .or_else(|| self.save_session().err());
        if let Some(reason) = refusal {
            self.show_exit_recovery(&reason);
            return false;
        }
        true
    }

    fn show_exit_recovery(&mut self, reason: &str) {
        let name = self.project.as_ref().map_or("Scratch", |p| p.name.as_str());
        let reason = reason.strip_prefix("Session not saved: ").unwrap_or(reason);
        let file_prefix = format!("{}: ", self.session_file().display());
        let reason = reason.strip_prefix(&file_prefix).unwrap_or(reason);
        let reason = if reason.ends_with(" is gone; the session is not saved") {
            "The project folder is missing."
        } else {
            reason.lines().next().unwrap_or(reason)
        };
        self.quit_requested = false;
        self.ask = Some(Ask::ExitRecovery {
            lines: vec![format!("Save failed · {name}"), reason.into()],
            offered_at: Instant::now(),
        });
        if let Some(audio) = &mut self.audio {
            match audio.pattern_capture.recover_exit(&self.pads) {
                Ok(Some(command)) => self.send(command),
                Ok(None) => {}
                Err(error) => self.post_notice(Level::Error, (error, Instant::now())),
            }
        }
    }

    pub(super) fn quit_ready(&self) -> bool {
        self.quit_requested
            && self
                .audio
                .as_ref()
                .is_none_or(|audio| !audio.pattern_capture.blocks_autosave())
    }

    pub(super) fn refuse_pattern_project_change(&mut self) -> bool {
        self.flush_audio();
        let phase = if self.quit_requested {
            Some("exit pending")
        } else {
            self.audio
                .as_ref()
                .and_then(|audio| audio.pattern_capture.blocking_phase())
        };
        if let Some(phase) = phase {
            self.post_notice(
                Level::Warn,
                (
                    format!("A1 {phase}: finish pattern work before opening projects."),
                    Instant::now(),
                ),
            );
            true
        } else {
            false
        }
    }

    pub(super) fn request_manual_save(&mut self) {
        let waiting = self.pending_manual_save.is_some();
        self.flush_audio();
        if waiting {
            return;
        }
        let origin = self.notice_origin.clone();
        if let Some(audio) = self
            .audio
            .as_ref()
            .filter(|a| a.pattern_capture.blocks_autosave())
        {
            let end = audio.pattern_capture.finish_for_save();
            let progress = self.notices.push_owned(
                Notice::new(Level::Info, origin.clone(), "Finishing A1 before saving."),
                Instant::now(),
            );
            self.pending_manual_save = Some(PendingSave {
                project: audio.project.clone(),
                epoch: audio.project.epoch.load(Acquire),
                origin,
                progress,
            });
            if let Some(end) = end {
                self.send(end);
            }
        } else {
            self.manual_save_now(origin);
        }
    }

    pub(super) fn finish_manual_save(&mut self) {
        let Some(pending) = self.pending_manual_save.as_ref() else {
            return;
        };
        let Some(audio) = self.audio.as_ref().filter(|a| {
            Arc::ptr_eq(&a.project, &pending.project)
                && a.project.epoch.load(Acquire) == pending.epoch
        }) else {
            self.cancel_manual_save();
            return;
        };
        if audio.pattern_capture.blocks_autosave() {
            return;
        }
        let pending = self.pending_manual_save.take().unwrap();
        self.notices.cancel_owned(&pending.progress, Instant::now());
        self.manual_save_now(pending.origin);
    }

    pub(super) fn cancel_manual_save(&mut self) {
        if let Some(pending) = self.pending_manual_save.take() {
            self.notices.cancel_owned(&pending.progress, Instant::now());
        }
    }

    fn manual_save_now(&mut self, origin: Origin) {
        let previous = std::mem::replace(&mut self.notice_origin, origin.clone());
        let recovering = matches!(self.ask, Some(Ask::ExitRecovery { .. }));
        match self.save_session() {
            Ok(()) => {
                let now = Instant::now();
                if recovering {
                    self.ask = None;
                    if let Some(SaveNotice::Refused(error)) = self.save_notice.take() {
                        self.notices.cancel_owned(&error, now);
                    }
                    self.save_notice = Some(SaveNotice::RecoveredUntil(
                        now + super::Duration::from_secs(5),
                    ));
                }
                self.produce_notice(Notice::manual_save_success(origin), now);
            }
            Err(reason) if recovering => self.show_exit_recovery(&reason),
            Err(_) => {}
        }
        self.notice_origin = previous;
    }

    pub(super) fn dirty(&mut self) {
        self.session_dirty.get_or_insert_with(Instant::now);
    }

    pub(super) fn session_file(&self) -> PathBuf {
        self.project.as_ref().map_or_else(
            || self.session_path.clone(),
            |p| p.folder.join("project.toml"),
        )
    }

    pub(super) fn save_status(&self) -> tui::SaveStatus {
        if self.session_write_error.is_some() {
            tui::SaveStatus::Failed
        } else if self.session_dirty.is_some() {
            tui::SaveStatus::Unsaved
        } else {
            tui::SaveStatus::Clean
        }
    }

    fn automatic_save_allowed(&self) -> bool {
        self.session_read_error.is_none()
            && self.session_write_error.is_none()
            && self.session_dirty.is_some()
    }

    pub(super) fn autosave(&mut self, now: Instant) {
        self.flush_audio();
        if self.quit_requested || self.pending_manual_save.is_some() {
            return;
        }
        if self
            .audio
            .as_ref()
            .is_some_and(|audio| audio.pattern_capture.blocks_autosave())
        {
            return;
        }
        if self.automatic_save_allowed()
            && self
                .session_dirty
                .is_some_and(|at| now.saturating_duration_since(at) > SESSION_SAVE)
        {
            let _ = self.save_session();
        }
    }

    pub(super) fn save_on_exit(&mut self) {
        self.quit_requested = true;
        self.flush_audio();
        if self.quit_ready() && self.automatic_save_allowed() {
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
                self.session_write_error = None;
            }
            Err(reason) => {
                if self.session_read_error.is_none() {
                    self.session_write_error = Some(reason.clone());
                }
                log::error!("{reason}");
                let now = Instant::now();
                if let Some(SaveNotice::Refused(error)) = self.save_notice.take() {
                    self.notices.cancel_owned(&error, now);
                }
                let notice = Notice::new(Level::Error, self.notice_origin.clone(), reason.clone());
                self.save_notice = Some(SaveNotice::Refused(self.notices.push_owned(notice, now)));
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
        s.bpm = self.transport.bpm;
        s.metronome = self.transport.metronome;
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

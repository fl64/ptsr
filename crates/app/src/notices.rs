//! Transient notice state. All mutation uses the caller's monotonic time.

use std::cmp::Reverse;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Duration, Instant};

const LIMIT: usize = 50;

// Declaration order is the display priority, from lowest to highest.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Level {
    Info,
    Done,
    Warn,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Origin {
    Keyboard,
    Device(String),
    System,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Lifetime {
    Timed(Duration),
    OwnerAction,
    Startup,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Notice {
    pub level: Level,
    pub origin: Origin,
    pub text: String,
    lifetime: Lifetime,
    recording_refusal: bool,
}

impl Notice {
    pub(super) fn new(level: Level, origin: Origin, text: impl Into<String>) -> Self {
        let lifetime = match level {
            Level::Info | Level::Done => Lifetime::Timed(Duration::from_secs(3)),
            Level::Warn => Lifetime::Timed(Duration::from_secs(6)),
            Level::Error => Lifetime::OwnerAction,
        };
        Self {
            level,
            origin,
            text: text.into(),
            lifetime,
            recording_refusal: false,
        }
    }

    pub(super) fn recording_refusal(origin: Origin, text: impl Into<String>) -> Self {
        Self {
            recording_refusal: true,
            ..Self::new(Level::Warn, origin, text)
        }
    }

    pub(super) fn startup(level: Level, text: impl Into<String>) -> Self {
        Self {
            lifetime: Lifetime::Startup,
            ..Self::new(level, Origin::System, text)
        }
    }

    pub(super) fn manual_save_success(origin: Origin) -> Self {
        Self {
            lifetime: Lifetime::Timed(Duration::from_secs(5)),
            ..Self::new(Level::Done, origin, "Session saved")
        }
    }
}

/// A retained identity cannot be reused while a frame still holds it.
#[derive(Clone)]
pub(super) struct NoticeToken(Rc<()>);

struct Entry {
    notice: Notice,
    remaining: Lifetime,
    token: NoticeToken,
}

pub(super) struct Notices {
    // Keep production order even when a newer, higher-level entry becomes active.
    entries: VecDeque<Entry>,
    active: Option<usize>,
    history: VecDeque<Notice>,
    visible: bool,
    at: Instant,
}

impl Notices {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            entries: VecDeque::new(),
            active: None,
            history: VecDeque::new(),
            visible: true,
            at: now,
        }
    }

    pub(super) fn recording_refusal_pending(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.notice.recording_refusal)
    }

    pub(super) fn current(&self) -> Option<&Notice> {
        self.visible
            .then(|| self.active.and_then(|i| self.entries.get(i)))
            .flatten()
            .map(|entry| &entry.notice)
    }

    pub(super) fn current_at(&self, now: Instant) -> Option<&Notice> {
        let entry = self.active.and_then(|i| self.entries.get(i))?;
        if !self.visible
            || matches!(entry.remaining, Lifetime::Timed(left)
            if now.saturating_duration_since(self.at) >= left)
        {
            return None;
        }
        Some(&entry.notice)
    }

    pub(super) fn history(&self) -> &VecDeque<Notice> {
        &self.history
    }

    #[cfg(test)]
    pub(super) fn deadline(&self) -> Option<Instant> {
        match self.entries.get(self.active?)?.remaining {
            Lifetime::Timed(left) if self.visible => self.at.checked_add(left),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(super) fn has_startup(&self) -> bool {
        self.entries
            .iter()
            .any(|e| e.remaining == Lifetime::Startup)
    }

    pub(super) fn record(&mut self, notice: Notice) {
        self.history.push_front(notice);
        if self.history.len() > LIMIT {
            self.history.pop_back();
        }
    }

    pub(super) fn pending_len(&self) -> usize {
        self.entries.len() - usize::from(self.active.is_some())
    }

    pub(super) fn push(&mut self, notice: Notice, now: Instant) {
        self.push_owned(notice, now);
    }

    pub(super) fn push_owned(&mut self, notice: Notice, now: Instant) -> NoticeToken {
        self.advance(now);
        self.record(notice.clone());
        let current = self
            .active
            .and_then(|i| self.entries.get(i))
            .map(|e| e.notice.level);
        let startup = notice.lifetime == Lifetime::Startup;
        if current == Some(notice.level)
            && !startup
            && let Some(i) = self.active
        {
            self.remove(i);
        }
        let level = notice.level;
        let i = self.entries.len();
        let token = NoticeToken(Rc::new(()));
        self.entries.push_back(Entry {
            remaining: notice.lifetime,
            notice,
            token: token.clone(),
        });
        if current.is_none_or(|old| level > old || (level == old && !startup)) {
            self.active = Some(i);
        }
        self.bound_pending();
        token
    }

    pub(super) fn cancel_owned(&mut self, token: &NoticeToken, now: Instant) {
        self.advance(now);
        if let Some(i) = self
            .entries
            .iter()
            .position(|e| Rc::ptr_eq(&e.token.0, &token.0))
        {
            self.remove(i);
            if self.active.is_none() {
                self.select_next();
            }
        }
    }

    pub(super) fn advance(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.at);
        self.at = self.at.max(now);
        if !self.visible {
            return;
        }
        let Some(i) = self.active else { return };
        let Some(entry) = self.entries.get_mut(i) else {
            return;
        };
        if let Lifetime::Timed(remaining) = &mut entry.remaining {
            *remaining = remaining.saturating_sub(elapsed);
            if remaining.is_zero() {
                self.remove(i);
                // A delayed update must not debit a notice that never appeared.
                self.select_next();
            }
        }
    }

    pub(super) fn set_visible(&mut self, visible: bool, now: Instant) {
        self.advance(now);
        self.visible = visible;
    }

    pub(super) fn error_token(&self) -> Option<NoticeToken> {
        let entry = self.entries.get(self.active?)?;
        (self.visible && entry.remaining == Lifetime::OwnerAction).then(|| entry.token.clone())
    }

    pub(super) fn acknowledge_shown(&mut self, token: Option<&NoticeToken>, now: Instant) {
        self.advance(now);
        if self
            .error_token()
            .zip(token)
            .is_some_and(|(active, shown)| Rc::ptr_eq(&active.0, &shown.0))
        {
            self.acknowledge(now);
        }
    }

    /// The caller classifies owner input and still routes that input normally.
    pub(super) fn acknowledge(&mut self, now: Instant) {
        self.advance(now);
        if !self.visible {
            return;
        }
        if let Some(i) = self.active
            && self
                .entries
                .get(i)
                .is_some_and(|e| e.remaining == Lifetime::OwnerAction)
        {
            self.remove(i);
            self.select_next();
        }
    }

    pub(super) fn dismiss_startup(&mut self, now: Instant) {
        self.advance(now);
        for i in (0..self.entries.len()).rev() {
            if self.entries[i].remaining == Lifetime::Startup {
                self.remove(i);
            }
        }
        if self.active.is_none() {
            self.select_next();
        }
    }

    fn select_next(&mut self) {
        self.active = self
            .entries
            .iter()
            .enumerate()
            .max_by_key(|(i, entry)| (entry.notice.level, Reverse(*i)))
            .map(|(i, _)| i);
    }

    fn bound_pending(&mut self) {
        if self.pending_len() > LIMIT
            && let Some(i) = self
                .entries
                .iter()
                .enumerate()
                .filter(|(i, _)| Some(*i) != self.active)
                .min_by_key(|(_, entry)| entry.notice.level)
                .map(|(i, _)| i)
        {
            self.remove(i);
        }
    }

    fn remove(&mut self, i: usize) {
        self.entries.remove(i);
        self.active = self.active.and_then(|active| match active.cmp(&i) {
            std::cmp::Ordering::Less => Some(active),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(active - 1),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_refusal_marker_preserves_hidden_error_priority_and_expires_with_its_notice() {
        let now = Instant::now();
        let mut notices = Notices::new(now);
        notices.push(
            Notice::new(Level::Error, Origin::System, "earlier error"),
            now,
        );
        notices.set_visible(false, now);
        notices.push(
            Notice::recording_refusal(Origin::Keyboard, "target refused"),
            now,
        );
        assert!(notices.recording_refusal_pending());
        assert!(notices.current().is_none());
        notices.set_visible(true, now);
        assert_eq!(notices.current().unwrap().text, "earlier error");
        let shown = notices.error_token();
        notices.acknowledge_shown(shown.as_ref(), now);
        assert_eq!(notices.current().unwrap().text, "target refused");
        assert_eq!(notices.deadline(), Some(now + Duration::from_secs(6)));
        notices.advance(now + Duration::from_secs(6));
        assert!(!notices.recording_refusal_pending());
        assert!(notices.current().is_none());
        let later = now + Duration::from_secs(7);
        notices.push(
            Notice::recording_refusal(Origin::Keyboard, "same text"),
            later,
        );
        assert!(notices.recording_refusal_pending());
        notices.push(Notice::new(Level::Warn, Origin::System, "same text"), later);
        assert!(
            !notices.recording_refusal_pending(),
            "text alone cannot classify a refusal"
        );
        assert_eq!(notices.current().unwrap().text, "same text");
        assert_eq!(notices.history().len(), 4);
    }

    #[test]
    fn pattern_manual_save_notice_cancellation_preserves_hidden_errors_and_history() {
        for visible in [false, true] {
            let now = Instant::now();
            let mut notices = Notices::new(now);
            let text = "Finishing A1 before saving.";
            let owned = notices.push_owned(Notice::new(Level::Info, Origin::Keyboard, text), now);
            notices.push(Notice::new(Level::Error, Origin::Keyboard, text), now);
            let error = notices.error_token().unwrap();
            notices.set_visible(visible, now);
            let history = notices.history().clone();
            notices.cancel_owned(&owned, now + Duration::from_secs(1));
            assert_eq!(notices.history(), &history);
            assert_eq!(notices.pending_len(), 0);
            assert_eq!(notices.current().is_some(), visible);
            notices.set_visible(true, now + Duration::from_secs(1));
            assert_eq!(notices.current().unwrap().level, Level::Error);
            notices.acknowledge_shown(Some(&error), now + Duration::from_secs(1));
            assert!(notices.current().is_none());
            notices.cancel_owned(&owned, now + Duration::from_secs(2));
            assert!(notices.current().is_none());
        }
    }

    #[test]
    fn pattern_manual_save_notice_cancellation_keeps_other_deadlines_and_stale_identities() {
        let now = Instant::now();
        let mut notices = Notices::new(now);
        let info = Notice::new(Level::Info, Origin::Keyboard, "Finishing A1 before saving.");
        let old = notices.push_owned(info.clone(), now);
        notices.push(info.clone(), now);
        let deadline = notices.deadline();
        let history = notices.history().clone();
        notices.cancel_owned(&old, now + Duration::from_secs(1));
        assert_eq!(notices.current(), Some(&info));
        assert_eq!(notices.deadline(), deadline);
        assert_eq!(notices.history(), &history);
        notices.advance(deadline.unwrap());
        assert!(notices.current().is_none());

        let at = now + Duration::from_secs(4);
        let owned = notices.push_owned(info, at);
        notices.push(
            Notice::new(Level::Warn, Origin::System, "Other warning"),
            at,
        );
        let deadline = notices.deadline().unwrap();
        notices.cancel_owned(&owned, at + Duration::from_secs(1));
        assert_eq!(notices.current().unwrap().text, "Other warning");
        assert_eq!(notices.deadline(), Some(deadline));
        notices.advance(deadline);
        assert!(notices.current().is_none());

        let expired = notices.push_owned(
            Notice::new(Level::Info, Origin::Keyboard, "Expired"),
            deadline,
        );
        notices.cancel_owned(&expired, deadline + Duration::from_secs(4));
        assert!(notices.current().is_none());
    }

    #[test]
    fn pattern_manual_save_notice_eviction_and_active_cancellation_keep_unrelated_entries() {
        let now = Instant::now();
        let mut notices = Notices::new(now);
        let owned = notices.push_owned(Notice::new(Level::Info, Origin::Keyboard, "Owned"), now);
        notices.cancel_owned(&owned, now);
        assert!(notices.current().is_none());
        let owned = notices.push_owned(Notice::new(Level::Info, Origin::Keyboard, "Evicted"), now);
        notices.push(Notice::new(Level::Error, Origin::System, "Keep error"), now);
        let error = notices.error_token().unwrap();
        for i in 0..=LIMIT {
            notices.push(
                Notice::new(Level::Info, Origin::System, format!("Other {i}")),
                now,
            );
        }
        let history = notices.history().clone();
        assert_eq!(notices.pending_len(), LIMIT);
        notices.cancel_owned(&owned, now);
        assert_eq!(notices.pending_len(), LIMIT);
        assert_eq!(notices.history(), &history);
        assert_eq!(notices.current().unwrap().text, "Keep error");
        notices.acknowledge_shown(Some(&error), now);
        assert_eq!(notices.current().unwrap().text, "Other 1");
    }
}

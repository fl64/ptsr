//! Transient notice state. All mutation uses the caller's monotonic time.

use std::cmp::Reverse;
use std::collections::VecDeque;
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Notice {
    pub level: Level,
    pub origin: Origin,
    pub text: String,
    lifetime: Lifetime,
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
        }
    }

    pub(super) fn manual_save_success(origin: Origin) -> Self {
        Self {
            lifetime: Lifetime::Timed(Duration::from_secs(5)),
            ..Self::new(Level::Done, origin, "Session saved")
        }
    }
}

struct Entry {
    notice: Notice,
    remaining: Lifetime,
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

    pub(super) fn current(&self) -> Option<&Notice> {
        self.visible
            .then(|| self.active.and_then(|i| self.entries.get(i)))
            .flatten()
            .map(|entry| &entry.notice)
    }

    pub(super) fn history(&self) -> &VecDeque<Notice> {
        &self.history
    }

    pub(super) fn pending_len(&self) -> usize {
        self.entries.len() - usize::from(self.active.is_some())
    }

    pub(super) fn push(&mut self, notice: Notice, now: Instant) {
        self.advance(now);
        self.history.push_front(notice.clone());
        if self.history.len() > LIMIT {
            self.history.pop_back();
        }

        let current = self
            .active
            .and_then(|i| self.entries.get(i))
            .map(|entry| entry.notice.level);
        if current == Some(notice.level)
            && let Some(i) = self.active
        {
            self.remove(i);
        }
        let level = notice.level;
        let i = self.entries.len();
        self.entries.push_back(Entry {
            remaining: notice.lifetime,
            notice,
        });
        if current.is_none_or(|old| level >= old) {
            self.active = Some(i);
        }
        self.bound_pending();
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
                // The next entry could not appear before this update. It keeps all
                // its remaining time, even when this update arrives after expiry.
                self.select_next();
            }
        }
    }

    pub(super) fn set_visible(&mut self, visible: bool, now: Instant) {
        self.advance(now);
        self.visible = visible;
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
                .is_some_and(|entry| entry.remaining == Lifetime::OwnerAction)
        {
            self.remove(i);
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

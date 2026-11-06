//! Process-local input identity. Observe existing routes without controlling them.

use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering::Relaxed},
    time::{Duration, Instant},
};

use crate::midi;
use input::midi::{Activity, Msg};

const HIGHLIGHT: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Connection(u64);

impl Connection {
    pub(super) fn fresh() -> Option<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self::take(&NEXT)
    }

    fn take(next: &AtomicU64) -> Option<Self> {
        next.fetch_update(Relaxed, Relaxed, |n| n.checked_add(1))
            .ok()
            .map(Self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Stamp {
    pub(super) connection: Connection,
    pub(super) at: Instant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Source {
    Keyboard,
    Device {
        port: String,
        connection: Connection,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Pending,
    Connected,
    Disconnected,
}

struct Device {
    connection: Connection,
    phase: Phase,
    positions: BTreeMap<(u8, u8), (u8, Instant)>,
}

#[derive(Default)]
pub(super) struct State {
    // Retain one generation per observed port, not a log of old connections.
    devices: BTreeMap<String, Device>,
    pub(super) latest: Option<(Source, Instant)>,
    pub(super) active: Option<Source>,
}

impl State {
    pub(super) fn connected(&self) -> impl Iterator<Item = (&str, Connection)> {
        self.devices.iter().filter_map(|(port, device)| {
            (device.phase == Phase::Connected).then_some((port.as_str(), device.connection))
        })
    }

    pub(super) fn snapshot(&self) -> Vec<tui::InputSource> {
        let mut inputs = vec![tui::InputSource {
            normal: "KB".into(),
            compact: "KB".into(),
            active: self.active == Some(Source::Keyboard),
        }];
        inputs.extend(self.connected().enumerate().map(|(i, (port, connection))| {
            let compact = format!("M{}", i + 1);
            tui::InputSource {
                normal: format!("{compact}:{port}"),
                compact,
                active: self.active.as_ref().is_some_and(|source| {
                    matches!(source, Source::Device { port: active, connection: id }
                        if active == port && *id == connection)
                }),
            }
        }));
        inputs
    }

    fn transition(&mut self, port: &str, connection: Connection, phase: Phase) -> bool {
        if let Some(device) = self.devices.get_mut(port) {
            if connection < device.connection
                || (connection == device.connection && device.phase == Phase::Disconnected)
            {
                return false;
            }
            if connection == device.connection {
                // A message can precede Connected, but cannot demote a known connection.
                if phase != Phase::Pending {
                    device.phase = phase;
                }
                return true;
            }
        }
        self.devices.insert(
            port.into(),
            Device {
                connection,
                phase,
                positions: BTreeMap::new(),
            },
        );
        true
    }

    fn movement(&mut self, port: &str, msg: Msg, at: Instant) -> bool {
        let Msg::Cc { ch, cc, value } = msg else {
            return false;
        };
        let Some(device) = self.devices.get_mut(port) else {
            return false;
        };
        let previous = device.positions.get(&(ch, cc)).copied();
        if previous.is_some_and(|(_, received)| at < received) {
            return false;
        }
        device.positions.insert((ch, cc), (value, at));
        previous.is_none_or(|(old, _)| old != value)
    }

    fn input(&mut self, source: Source, at: Instant) {
        if self
            .latest
            .as_ref()
            .is_none_or(|(_, previous)| at >= *previous)
        {
            self.latest = Some((source, at));
        }
    }

    pub(super) fn keyboard(&mut self, at: Instant) {
        self.input(Source::Keyboard, at);
        self.advance(at);
    }

    pub(super) fn observe(&mut self, event: &midi::UiEvent, now: Instant) {
        if let Some(stamp) = event.source() {
            match &event.event {
                midi::Event::Connected { port, .. } => {
                    self.transition(port, stamp.connection, Phase::Connected);
                }
                midi::Event::Disconnected { port } => {
                    self.transition(port, stamp.connection, Phase::Disconnected);
                }
                midi::Event::Message { msg, .. } => {
                    if let Some(port) = event.origin()
                        && self.transition(port, stamp.connection, Phase::Pending)
                    {
                        let activity = match event.activity() {
                            Activity::Press => true,
                            Activity::Movement => self.movement(port, *msg, stamp.at),
                            _ => false,
                        };
                        if activity {
                            self.input(
                                Source::Device {
                                    port: port.into(),
                                    connection: stamp.connection,
                                },
                                stamp.at,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
        self.advance(now);
    }

    /// A frame advances elapsed time even when no event arrives.
    pub(super) fn advance(&mut self, now: Instant) {
        let visible = self
            .latest
            .as_ref()
            .filter(|(source, at)| {
                now.checked_duration_since(*at)
                    .is_some_and(|elapsed| elapsed < HIGHLIGHT)
                    && match source {
                        Source::Keyboard => true,
                        Source::Device { port, connection } => self
                            .connected()
                            .any(|(name, id)| name == port && id == *connection),
                    }
            })
            .map(|(source, _)| source);
        if self.active.as_ref() != visible {
            self.active = visible.cloned();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_ids_never_wrap_or_reuse_an_exhausted_counter() {
        let next = AtomicU64::new(u64::MAX - 1);
        assert_eq!(Connection::take(&next), Some(Connection(u64::MAX - 1)));
        assert_eq!(Connection::take(&next), None);
        assert_eq!(Connection::take(&next), None);
    }

    #[test]
    fn older_connections_cannot_resurrect_or_replace_current_state() {
        let mut state = State::default();
        assert!(state.transition("port", Connection(1), Phase::Pending));
        assert!(state.transition("port", Connection(1), Phase::Connected));
        assert!(state.transition("port", Connection(1), Phase::Disconnected));
        assert!(!state.transition("port", Connection(1), Phase::Connected));
        assert!(!state.transition("port", Connection(1), Phase::Pending));
        assert!(state.transition("port", Connection(2), Phase::Connected));
        assert!(!state.transition("port", Connection(1), Phase::Disconnected));
        assert_eq!(
            state.connected().collect::<Vec<_>>(),
            [("port", Connection(2))]
        );
    }

    #[test]
    fn source_deadlines_use_original_time_without_new_input() {
        let mut state = State::default();
        let at = Instant::now();
        state.keyboard(at);
        state.advance(at + HIGHLIGHT - Duration::from_nanos(1));
        assert_eq!(state.active, Some(Source::Keyboard));
        state.advance(at + HIGHLIGHT);
        assert_eq!(state.active, None);
        state.advance(at + HIGHLIGHT + Duration::from_secs(3));
        assert_eq!(state.active, None);
        state.keyboard(at - Duration::from_nanos(1));
        state.advance(at + HIGHLIGHT);
        assert_eq!(
            state.active, None,
            "older input cannot replace the original timestamp"
        );
    }
}

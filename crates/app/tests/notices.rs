//! Controlled-time checks of the same source used by the application.
//! Production input and drawing checks live in their application and TUI modules.

#[path = "../src/notices.rs"]
mod notices;

use notices::{Level, Notice, Notices, Origin};
use std::time::{Duration, Instant};

fn text(queue: &Notices) -> Option<&str> {
    queue.current().map(|notice| notice.text.as_str())
}

fn push(queue: &mut Notices, level: Level, label: &str, now: Instant) {
    queue.push(Notice::new(level, Origin::System, label), now);
}

#[test]
fn startup_notices_keep_their_keyboard_acknowledgement_exception() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    queue.push(Notice::startup(Level::Warn, "first"), now);
    queue.push(Notice::startup(Level::Warn, "second"), now);
    assert_eq!(text(&queue), Some("first"));
    queue.push(Notice::startup(Level::Error, "read protection"), now);
    push(&mut queue, Level::Info, "pending", now);
    queue.advance(now + Duration::from_secs(100));
    queue.acknowledge(now + Duration::from_secs(100));
    assert_eq!(text(&queue), Some("read protection"));
    assert!(queue.error_token().is_none());
    assert!(queue.has_startup());
    queue.dismiss_startup(now + Duration::from_secs(100));
    assert!(!queue.has_startup());
    assert_eq!(text(&queue), Some("pending"));
    assert_eq!(queue.history().len(), 4);
    queue.advance(now + Duration::from_secs(103));
    assert!(queue.current().is_none());
}

#[test]
fn only_the_displayed_error_production_can_be_acknowledged() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    push(&mut queue, Level::Error, "same error", now);
    queue.acknowledge_shown(None, now);
    assert_eq!(text(&queue), Some("same error"));
    let old = queue.error_token();
    push(&mut queue, Level::Error, "same error", now);
    queue.acknowledge_shown(old.as_ref(), now);
    assert_eq!(text(&queue), Some("same error"));
    let displayed = queue.error_token();
    for i in 0..55 {
        push(&mut queue, Level::Info, &format!("info {i}"), now);
    }
    queue.set_visible(false, now);
    queue.acknowledge_shown(displayed.as_ref(), now);
    queue.set_visible(true, now);
    assert_eq!(text(&queue), Some("same error"));
    queue.acknowledge_shown(displayed.as_ref(), now);
    assert_eq!(text(&queue), Some("info 5"));
    queue.acknowledge_shown(displayed.as_ref(), now);
    assert_eq!(text(&queue), Some("info 5"));
}

#[test]
fn history_only_records_do_not_create_timed_or_pending_entries() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    for i in 0..60 {
        queue.record(Notice::new(
            Level::Warn,
            Origin::System,
            format!("keymap {i}"),
        ));
    }
    assert_eq!(queue.history().len(), 50);
    assert_eq!(queue.history().front().unwrap().text, "keymap 59");
    assert_eq!(queue.history().back().unwrap().text, "keymap 10");
    for second in 0..100 {
        queue.advance(now + Duration::from_secs(second));
        assert!(queue.current().is_none());
        assert_eq!(queue.pending_len(), 0);
        assert_eq!(queue.history().len(), 50);
    }
}

#[test]
fn notice_peeks_keep_deadlines_and_history_unchanged() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    queue.push(Notice::manual_save_success(Origin::Keyboard), now);
    assert_eq!(queue.deadline(), Some(now + Duration::from_secs(5)));
    assert_eq!(
        queue
            .current_at(now + Duration::from_secs(5) - Duration::from_nanos(1))
            .unwrap()
            .text,
        "Session saved"
    );
    assert!(queue.current_at(now + Duration::from_secs(5)).is_none());
    assert_eq!(text(&queue), Some("Session saved"));
    assert_eq!(queue.history().len(), 1);
}

#[test]
fn notice_model_preserves_levels_and_actual_origins() {
    let now = Instant::now();
    for level in [Level::Done, Level::Info, Level::Warn, Level::Error] {
        for origin in [
            Origin::Keyboard,
            Origin::Device("USB controller α MIDI 1".into()),
            Origin::Device("USB controller α MIDI 2".into()),
            Origin::System,
        ] {
            let mut queue = Notices::new(now);
            let notice = Notice::new(level, origin.clone(), "A result");
            queue.push(notice.clone(), now);
            assert_eq!(queue.current(), Some(&notice));
            assert_eq!(queue.history().front(), Some(&notice));
            assert_eq!(queue.current().unwrap().origin, origin);
            assert_eq!(queue.current().unwrap().level, level);
        }
    }
}

#[test]
fn timed_notices_use_their_visible_lifetime() {
    let now = Instant::now();
    for (notice, seconds) in [
        (Notice::new(Level::Done, Origin::Keyboard, "Done"), 3),
        (Notice::new(Level::Info, Origin::System, "Info"), 3),
        (Notice::new(Level::Warn, Origin::System, "Warn"), 6),
        (Notice::manual_save_success(Origin::Keyboard), 5),
    ] {
        let mut queue = Notices::new(now);
        queue.push(notice.clone(), now);
        let deadline = now + Duration::from_secs(seconds);
        queue.advance(deadline - Duration::from_nanos(1));
        assert_eq!(queue.current(), Some(&notice));
        queue.advance(deadline);
        assert_eq!(queue.current(), None);
        assert_eq!(queue.pending_len(), 0);
        assert_eq!(queue.history().front(), Some(&notice));
        queue.advance(deadline + Duration::from_secs(100));
        assert_eq!(queue.current(), None);
        assert_eq!(queue.history().len(), 1);
    }
}

#[test]
fn manual_save_success_keeps_its_origin_and_five_seconds_after_interruption() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    let saved = Notice::manual_save_success(Origin::Device("save controller".into()));
    assert_eq!(saved.level, Level::Done);
    assert_eq!(saved.text, "Session saved");
    queue.push(saved.clone(), now);
    push(
        &mut queue,
        Level::Error,
        "refused",
        now + Duration::from_secs(2),
    );
    queue.acknowledge(now + Duration::from_secs(100));
    assert_eq!(queue.current(), Some(&saved));
    queue.advance(now + Duration::from_secs(103) - Duration::from_nanos(1));
    assert_eq!(queue.current(), Some(&saved));
    queue.advance(now + Duration::from_secs(103));
    assert_eq!(queue.current(), None);
    assert_eq!(queue.history().len(), 2);
}

#[test]
fn an_informational_result_cannot_hide_an_error() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    push(&mut queue, Level::Error, "error", now);
    for (seconds, level, label) in [
        (1, Level::Info, "info"),
        (2, Level::Done, "done"),
        (3, Level::Warn, "warn"),
    ] {
        push(&mut queue, level, label, now + Duration::from_secs(seconds));
        assert_eq!(text(&queue), Some("error"));
    }
    assert_eq!(queue.pending_len(), 3);
    assert_eq!(queue.history().len(), 4);
    queue.advance(now + Duration::from_secs(100));
    assert_eq!(text(&queue), Some("error"));
    queue.acknowledge(now + Duration::from_secs(100));
    assert_eq!(text(&queue), Some("warn"));
    queue.advance(now + Duration::from_secs(106));
    assert_eq!(text(&queue), Some("done"));
    queue.advance(now + Duration::from_secs(109));
    assert_eq!(text(&queue), Some("info"));
    queue.advance(now + Duration::from_secs(112));
    assert_eq!(text(&queue), None);
    assert_eq!(queue.history().len(), 4);
}

#[test]
fn every_pair_of_notice_levels_obeys_priority() {
    let now = Instant::now();
    let levels = [Level::Info, Level::Done, Level::Warn, Level::Error];
    for (old_rank, old) in levels.iter().enumerate() {
        for (new_rank, new) in levels.iter().enumerate() {
            let mut queue = Notices::new(now);
            push(&mut queue, *old, "old", now);
            push(&mut queue, *new, "new", now + Duration::from_secs(1));
            assert_eq!(
                text(&queue),
                Some(if new_rank >= old_rank { "new" } else { "old" }),
                "{old:?} then {new:?}"
            );
            assert_eq!(queue.pending_len(), usize::from(old_rank != new_rank));
            assert_eq!(queue.history().len(), 2);
            assert_eq!(queue.history()[0].text, "new");
            assert_eq!(queue.history()[1].text, "old");
        }
    }
}

#[test]
fn interrupted_notices_retain_remaining_time() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    for (seconds, level, label) in [
        (0, Level::Info, "info"),
        (1, Level::Done, "done"),
        (2, Level::Warn, "warn"),
        (3, Level::Error, "error"),
        (4, Level::Info, "later info"),
    ] {
        push(&mut queue, level, label, now + Duration::from_secs(seconds));
    }
    queue.acknowledge(now + Duration::from_secs(100));
    for (seconds, label) in [
        (100, "warn"),
        (105, "done"),
        (107, "info"),
        (109, "later info"),
    ] {
        queue.advance(now + Duration::from_secs(seconds));
        assert_eq!(text(&queue), Some(label));
    }
    queue.advance(now + Duration::from_secs(112));
    assert_eq!(queue.current(), None);
    assert_eq!(queue.pending_len(), 0);
    assert_eq!(queue.history().len(), 5);
}

#[test]
fn equal_level_replacement_does_not_replay_the_old_notice() {
    let now = Instant::now();
    for level in [Level::Info, Level::Done, Level::Warn, Level::Error] {
        let mut queue = Notices::new(now);
        push(&mut queue, level, "old", now);
        push(&mut queue, level, "new", now + Duration::from_secs(1));
        assert_eq!(text(&queue), Some("new"));
        assert_eq!(queue.pending_len(), 0);
        if level == Level::Error {
            queue.acknowledge(now + Duration::from_secs(2));
        } else {
            queue.advance(now + Duration::from_secs(10));
        }
        assert_eq!(queue.current(), None);
        assert_eq!(queue.history().len(), 2);
    }
}

#[test]
fn queued_notices_retain_priority_and_production_order() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    push(&mut queue, Level::Error, "first error", now);
    push(&mut queue, Level::Warn, "old warn", now);
    push(&mut queue, Level::Warn, "waiting warn", now);
    queue.acknowledge(now);
    assert_eq!(text(&queue), Some("old warn"));
    push(&mut queue, Level::Warn, "replacement warn", now);
    push(&mut queue, Level::Error, "second error", now);
    queue.acknowledge(now);
    assert_eq!(text(&queue), Some("waiting warn"));
    queue.advance(now + Duration::from_secs(6));
    assert_eq!(text(&queue), Some("replacement warn"));
    queue.advance(now + Duration::from_secs(12));
    assert_eq!(text(&queue), None);
    assert_eq!(queue.history().len(), 5);
}

#[test]
fn expired_notices_do_not_resume_after_interruption() {
    let now = Instant::now();
    let deadline = Duration::from_secs(3);
    for arrival in [
        deadline - Duration::from_nanos(1),
        deadline,
        deadline + Duration::from_nanos(1),
    ] {
        let mut queue = Notices::new(now);
        push(&mut queue, Level::Info, "old", now);
        push(&mut queue, Level::Warn, "new", now + arrival);
        assert_eq!(text(&queue), Some("new"));
        assert_eq!(queue.pending_len(), usize::from(arrival < deadline));
        let end = now + arrival + Duration::from_secs(6);
        queue.advance(end);
        assert_eq!(text(&queue), (arrival < deadline).then_some("old"));
        queue.advance(end + Duration::from_nanos(1));
        assert_eq!(text(&queue), None);
        assert_eq!(queue.history().len(), 2);
    }
}

#[test]
fn pending_notice_overflow_remains_bounded() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    push(&mut queue, Level::Error, "active error", now);
    for (level, count) in [(Level::Warn, 10), (Level::Done, 20), (Level::Info, 25)] {
        for i in 0..count {
            push(&mut queue, level, &format!("{level:?} {i}"), now);
            assert_eq!(text(&queue), Some("active error"));
            assert!(queue.pending_len() <= 50);
            assert!(queue.history().len() <= 50);
        }
    }
    push(&mut queue, Level::Done, "new done", now);
    assert_eq!(queue.pending_len(), 50);
    assert_eq!(queue.history().len(), 50);
    assert!(queue.history().iter().any(|n| n.text == "Info 0"));
    assert!(!queue.history().iter().any(|n| n.text == "active error"));
    let history = queue.history().clone();
    let expected_history: Vec<String> = (6..10)
        .map(|i| format!("Warn {i}"))
        .chain((0..20).map(|i| format!("Done {i}")))
        .chain((0..25).map(|i| format!("Info {i}")))
        .chain(["new done".into()])
        .rev()
        .collect();
    assert_eq!(
        history.iter().map(|n| n.text.clone()).collect::<Vec<_>>(),
        expected_history
    );

    queue.acknowledge(now);
    let expected: Vec<(String, u64)> = (0..10)
        .map(|i| (format!("Warn {i}"), 6))
        .chain((0..20).map(|i| (format!("Done {i}"), 3)))
        .chain([("new done".into(), 3)])
        .chain((6..25).map(|i| (format!("Info {i}"), 3)))
        .collect();
    let mut at = now;
    for (i, (label, seconds)) in expected.iter().enumerate() {
        assert_eq!(text(&queue), Some(label.as_str()));
        assert_eq!(queue.pending_len(), expected.len() - i - 1);
        at += Duration::from_secs(*seconds);
        queue.advance(at);
    }
    assert_eq!(queue.current(), None);
    assert_eq!(queue.pending_len(), 0);
    assert_eq!(queue.history(), &history);
}

#[test]
fn overflow_can_discard_the_new_lowest_level_without_losing_history() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    push(&mut queue, Level::Error, "error", now);
    for i in 0..51 {
        push(&mut queue, Level::Warn, &format!("warn {i}"), now);
    }
    push(&mut queue, Level::Info, "new info", now);
    assert_eq!(text(&queue), Some("error"));
    assert_eq!(queue.pending_len(), 50);
    assert_eq!(queue.history().front().unwrap().text, "new info");
    queue.acknowledge(now);
    for i in 1..51 {
        assert_eq!(text(&queue), Some(format!("warn {i}").as_str()));
        queue.advance(now + Duration::from_secs(i * 6));
    }
    assert_eq!(queue.current(), None);
}

#[test]
fn interruption_and_overflow_keep_the_active_entry_identity() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    push(&mut queue, Level::Error, "error", now);
    for i in 0..50 {
        push(&mut queue, Level::Info, &format!("info {i}"), now);
    }
    queue.acknowledge(now);
    assert_eq!(text(&queue), Some("info 0"));
    push(&mut queue, Level::Warn, "active warn", now);
    assert_eq!(queue.pending_len(), 50);
    push(&mut queue, Level::Done, "done", now);
    assert_eq!(text(&queue), Some("active warn"));
    assert_eq!(queue.pending_len(), 50);
    queue.advance(now + Duration::from_secs(6));
    assert_eq!(text(&queue), Some("done"));
    queue.advance(now + Duration::from_secs(9));
    assert_eq!(text(&queue), Some("info 1"));
    queue.advance(now + Duration::from_secs(12));
    assert_eq!(text(&queue), Some("info 2"));
}

#[test]
fn hidden_notices_retain_their_visible_time() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    push(&mut queue, Level::Info, "info", now);
    queue.set_visible(false, now + Duration::from_secs(1));
    push(
        &mut queue,
        Level::Done,
        "done",
        now + Duration::from_secs(2),
    );
    push(
        &mut queue,
        Level::Warn,
        "warn",
        now + Duration::from_secs(3),
    );
    queue.advance(now + Duration::from_secs(100));
    assert_eq!(queue.current(), None);
    assert_eq!(queue.pending_len(), 2);
    assert_eq!(queue.history().len(), 3);
    queue.set_visible(true, now + Duration::from_secs(100));
    assert_eq!(text(&queue), Some("warn"));
    queue.advance(now + Duration::from_secs(106));
    assert_eq!(text(&queue), Some("done"));
    queue.advance(now + Duration::from_secs(109));
    assert_eq!(text(&queue), Some("info"));
    queue.advance(now + Duration::from_secs(111));
    assert_eq!(text(&queue), None);
    assert_eq!(queue.history().len(), 3);
}

#[test]
fn a_delayed_update_preserves_the_next_notice_lifetime() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    push(&mut queue, Level::Warn, "warn", now);
    push(&mut queue, Level::Info, "info", now);
    queue.advance(now + Duration::from_secs(60));
    assert_eq!(text(&queue), Some("info"));
    queue.advance(now + Duration::from_secs(63) - Duration::from_nanos(1));
    assert_eq!(text(&queue), Some("info"));
    queue.advance(now + Duration::from_secs(63));
    assert_eq!(text(&queue), None);
    assert_eq!(queue.history().len(), 2);
}

#[test]
fn explicit_acknowledgement_only_removes_the_visible_error() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    push(&mut queue, Level::Error, "error", now);
    push(&mut queue, Level::Done, "done", now);
    queue.set_visible(false, now);
    queue.acknowledge(now + Duration::from_secs(100));
    queue.set_visible(true, now + Duration::from_secs(100));
    assert_eq!(text(&queue), Some("error"));
    queue.acknowledge(now + Duration::from_secs(100));
    assert_eq!(text(&queue), Some("done"));
    queue.acknowledge(now + Duration::from_secs(101));
    assert_eq!(text(&queue), Some("done"));
    queue.advance(now + Duration::from_secs(103));
    assert_eq!(text(&queue), None);
    queue.acknowledge(now + Duration::from_secs(104));
    assert_eq!(text(&queue), None);
    assert_eq!(queue.history().len(), 2);
}

#[test]
fn timer_updates_and_reads_do_not_acknowledge_errors() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    push(&mut queue, Level::Error, "error", now);
    let history = queue.history().clone();
    for second in 0..1000 {
        queue.advance(now + Duration::from_secs(second));
        assert_eq!(text(&queue), Some("error"));
        assert_eq!(queue.pending_len(), 0);
        assert_eq!(queue.history(), &history);
    }
}

#[test]
fn notice_history_records_each_production_once_and_starts_empty() {
    let now = Instant::now();
    let mut queue = Notices::new(now);
    let repeated = Notice::new(Level::Done, Origin::Keyboard, "same result");
    queue.push(repeated.clone(), now);
    queue.push(repeated.clone(), now);
    assert_eq!(queue.history().len(), 2);
    assert!(queue.history().iter().all(|n| n == &repeated));
    assert_eq!(queue.pending_len(), 0);
    queue.advance(now + Duration::from_secs(3));
    let history = queue.history().clone();
    for _ in 0..100 {
        assert_eq!(queue.current(), None);
        assert_eq!(queue.history(), &history);
    }
    let restarted = Notices::new(now + Duration::from_secs(4));
    assert_eq!(restarted.current(), None);
    assert_eq!(restarted.pending_len(), 0);
    assert!(restarted.history().is_empty());
    assert_eq!(queue.history(), &history);
}

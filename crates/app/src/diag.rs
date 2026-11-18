//! Optional diagnostic mode (128): input lines, audio callback stalls and the kernel CPU limit.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};
use std::time::{Duration, Instant};

use engine::rt::{Stage, Stats};

use crate::config;

static ON: AtomicBool = AtomicBool::new(false);
/// The log file descriptor for the SIGXCPU handler; -1 while unset.
static LOG_FD: AtomicI32 = AtomicI32::new(-1);

const STALL: Duration = Duration::from_millis(100);
const WATCH_EVERY: Duration = Duration::from_millis(20);

/// Diagnostic mode from the command line or the environment.
fn requested(mut args: impl Iterator<Item = String>, env: Option<String>) -> bool {
    args.any(|arg| arg == "--diag") || env.is_some_and(|value| value == "1")
}

pub fn enabled() -> bool {
    ON.load(Relaxed)
}

/// Log to `$XDG_STATE_HOME/ptsr/ptsr.log`. The audio thread never logs.
pub fn init_log() -> PathBuf {
    let dir = config::xdg_dir("XDG_STATE_HOME", ".local/state");
    let path = dir.join("ptsr.log");
    let file = std::fs::create_dir_all(&dir).and_then(|_| {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
    });
    // the log did not open: run without it; the terminal is taken by the UI
    if let Ok(f) = file {
        let _ =
            simplelog::WriteLogger::init(log::LevelFilter::Info, simplelog::Config::default(), f);
    }
    if requested(std::env::args().skip(1), std::env::var("PTSR_DIAG").ok()) {
        ON.store(true, Relaxed);
        if let Ok(file) = std::fs::OpenOptions::new().append(true).open(&path) {
            use std::os::fd::IntoRawFd;
            LOG_FD.store(file.into_raw_fd(), Relaxed);
        }
        log::info!("diagnostic mode: on");
    }
    path
}

/// One input line in diagnostic mode; the text is built only when the mode is on.
pub fn input(line: impl FnOnce() -> String) {
    if let Some(line) = input_line(enabled(), line) {
        log::info!("{line}");
    }
}

fn input_line(on: bool, line: impl FnOnce() -> String) -> Option<String> {
    on.then(|| format!("input: {}", line()))
}

/// The warning for a callback that runs past the limit, once per callback.
fn stall_report(
    running: Option<(Duration, Option<Stage>)>,
    reported: &mut Option<Instant>,
    now: Instant,
) -> Option<String> {
    let Some((duration, stage)) = running else {
        *reported = None;
        return None;
    };
    // The start of this callback, so a new callback gets its own warning.
    let start = now.checked_sub(duration).unwrap_or(now);
    let same =
        |seen: Instant| seen.max(start).saturating_duration_since(seen.min(start)) < WATCH_EVERY;
    if duration < STALL || reported.is_some_and(same) {
        return None;
    }
    *reported = Some(start);
    Some(format!(
        "audio callback running for {} ms, stage {stage:?}",
        duration.as_millis()
    ))
}

/// Watch the audio callback and lower the soft real-time limit, in diagnostic mode only.
pub fn watch_audio(stats: &Arc<Stats>) {
    if !enabled() {
        return;
    }
    let stats = stats.clone();
    let spawned = std::thread::Builder::new()
        .name("ptsr-diag".into())
        .spawn(move || {
            let mut reported = None;
            loop {
                std::thread::sleep(WATCH_EVERY);
                let now = Instant::now();
                if let Some(line) = stall_report(stats.callback_running(now), &mut reported, now) {
                    log::warn!("{line}");
                }
            }
        });
    if let Err(e) = spawned {
        log::warn!("diagnostic watch did not start: {e}");
    }
    lower_rt_limit();
}

extern "C" fn on_sigxcpu(_: libc::c_int) {
    const LINE: &[u8] = b"[WARN] SIGXCPU: a real-time thread passed the soft CPU limit\n";
    let fd = LOG_FD.load(Relaxed);
    if fd >= 0 {
        // write(2) is async-signal-safe; the result cannot be reported from here.
        unsafe { libc::write(fd, LINE.as_ptr().cast(), LINE.len()) };
    }
}

/// Set the soft RLIMIT_RTTIME to half the hard limit, so SIGXCPU comes before SIGKILL.
fn lower_rt_limit() {
    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: plain libc calls with valid pointers; the handler only calls write(2).
    unsafe {
        if libc::getrlimit(libc::RLIMIT_RTTIME, &mut limit) != 0
            || limit.rlim_max == libc::RLIM_INFINITY
        {
            log::info!("diagnostic mode: no real-time CPU limit");
            return;
        }
        let handler = on_sigxcpu as extern "C" fn(libc::c_int) as libc::sighandler_t;
        if libc::signal(libc::SIGXCPU, handler) == libc::SIG_ERR {
            log::warn!("diagnostic mode: SIGXCPU handler not set");
            return;
        }
        let hard = limit.rlim_max;
        limit.rlim_cur = hard / 2;
        if libc::setrlimit(libc::RLIMIT_RTTIME, &limit) != 0 {
            log::warn!("diagnostic mode: soft real-time limit not set");
            return;
        }
        log::info!(
            "diagnostic mode: real-time CPU limit soft {} us, hard {hard} us",
            hard / 2
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario "Diagnostic mode reports a stalled callback" (audio-engine).
    #[test]
    fn diagnostic_mode_reports_a_stalled_callback() {
        let now = Instant::now();
        let mut reported = None;
        let quick = Some((Duration::from_millis(5), Some(Stage::Render)));
        assert_eq!(stall_report(quick, &mut reported, now), None);
        let slow = Some((Duration::from_millis(150), Some(Stage::Render)));
        let line = stall_report(slow, &mut reported, now).unwrap();
        assert!(line.contains("150 ms") && line.contains("Render"), "{line}");
        let later = now + WATCH_EVERY;
        let same = Some((Duration::from_millis(170), Some(Stage::Render)));
        assert_eq!(stall_report(same, &mut reported, later), None);
        assert_eq!(stall_report(None, &mut reported, later), None);
        let next = later + Duration::from_millis(500);
        assert!(stall_report(slow, &mut reported, next).is_some());
    }

    /// Scenario "Normal start keeps the log quiet" (audio-engine).
    #[test]
    fn normal_start_keeps_the_log_quiet() {
        let args = |list: &[&str]| list.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
        assert!(!requested(args(&[]).into_iter(), None));
        assert!(!requested(args(&["--other"]).into_iter(), Some("0".into())));
        assert!(requested(args(&["--diag"]).into_iter(), None));
        assert!(requested(args(&[]).into_iter(), Some("1".into())));
        let mut called = false;
        let line = input_line(false, || {
            called = true;
            String::new()
        });
        assert!(line.is_none() && !called);
    }

    /// Scenario "Diagnostic mode logs inputs" (audio-engine).
    #[test]
    fn diagnostic_mode_logs_inputs() {
        let key = input_line(true, || "action BankNext".into());
        assert_eq!(key.as_deref(), Some("input: action BankNext"));
        let midi = input_line(true, || "midi SMC-PAD-Master cc 30 65 -> step".into());
        assert_eq!(
            midi.as_deref(),
            Some("input: midi SMC-PAD-Master cc 30 65 -> step")
        );
    }
}

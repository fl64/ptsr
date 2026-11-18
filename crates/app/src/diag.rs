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
const SUMMARY_EVERY: Duration = Duration::from_secs(1);

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
            let mut summary = Summary::new(stats.callback_totals(), rt_threads());
            loop {
                std::thread::sleep(WATCH_EVERY);
                let now = Instant::now();
                if let Some(line) = stall_report(stats.callback_running(now), &mut reported, now) {
                    log::warn!("{line}");
                }
                if let Some(line) = summary.next(now, stats.callback_totals(), rt_threads()) {
                    log::info!("{line}");
                }
            }
        });
    if let Err(e) = spawned {
        log::warn!("diagnostic watch did not start: {e}");
    }
    lower_rt_limit();
}

/// A real-time thread of this process: id, name and CPU time in clock ticks.
#[derive(Clone, Debug, PartialEq)]
struct RtThread {
    tid: u32,
    name: String,
    ticks: u64,
}

/// Parse `/proc/self/task/<tid>/stat`; `None` for a normal (non real-time) thread.
fn parse_stat(tid: u32, stat: &str) -> Option<RtThread> {
    let open = stat.find('(')?;
    let close = stat.rfind(')')?;
    let name = stat.get(open + 1..close)?.to_string();
    // Fields after the name start with field 3 (state); see proc(5).
    let fields: Vec<&str> = stat.get(close + 1..)?.split_whitespace().collect();
    let field = |number: usize| fields.get(number - 3)?.parse::<u64>().ok();
    let policy = field(41)?;
    (policy != 0).then(|| RtThread {
        tid,
        name,
        ticks: field(14).unwrap_or(0) + field(15).unwrap_or(0),
    })
}

fn rt_threads() -> Vec<RtThread> {
    let Ok(tasks) = std::fs::read_dir("/proc/self/task") else {
        return Vec::new();
    };
    tasks
        .flatten()
        .filter_map(|entry| {
            let tid = entry.file_name().to_str()?.parse().ok()?;
            let stat = std::fs::read_to_string(entry.path().join("stat")).ok()?;
            parse_stat(tid, &stat)
        })
        .collect()
}

/// Once a second: callback count and time, and the CPU time of each real-time thread.
struct Summary {
    at: Instant,
    totals: (u64, u64),
    threads: Vec<RtThread>,
    tick_ms: u64,
}

impl Summary {
    fn new(totals: (u64, u64), threads: Vec<RtThread>) -> Self {
        // SAFETY: sysconf has no preconditions.
        let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        Self {
            at: Instant::now(),
            totals,
            threads,
            tick_ms: 1000 / u64::try_from(ticks).unwrap_or(100).max(1),
        }
    }

    fn next(&mut self, now: Instant, totals: (u64, u64), threads: Vec<RtThread>) -> Option<String> {
        if now.saturating_duration_since(self.at) < SUMMARY_EVERY {
            return None;
        }
        let calls = totals.0.saturating_sub(self.totals.0);
        let busy = totals.1.saturating_sub(self.totals.1) / 1000;
        let mut line = format!("diag: callbacks {calls}/s, callback busy {busy} ms/s");
        for thread in &threads {
            let before = self
                .threads
                .iter()
                .find(|old| old.tid == thread.tid)
                .map_or(thread.ticks, |old| old.ticks);
            let cpu = thread.ticks.saturating_sub(before) * self.tick_ms;
            line.push_str(&format!(
                "; rt {} {} cpu {cpu} ms/s",
                thread.tid, thread.name
            ));
        }
        if threads.is_empty() {
            line.push_str("; no rt threads");
        }
        self.at = now;
        self.totals = totals;
        self.threads = threads;
        Some(line)
    }
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

    /// Scenario "Diagnostic mode reports the real-time load each second" (audio-engine).
    #[test]
    fn diagnostic_mode_reports_the_real_time_load_each_second() {
        let fifo = "4321 (data-loop.0) S 1 2 3 4 5 6 7 8 9 10 30 20 0 0 -11 0 30 0 \
                    100 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 5 20 1 0 0 0 0";
        let normal = fifo.replace(" 20 1 0 0 0 0", " 0 0 0 0 0 0");
        let thread = parse_stat(4321, fifo).unwrap();
        assert_eq!((thread.name.as_str(), thread.ticks), ("data-loop.0", 50));
        assert_eq!(parse_stat(4321, &normal), None);

        let start = Instant::now();
        let mut summary = Summary {
            at: start,
            totals: (0, 0),
            threads: vec![thread.clone()],
            tick_ms: 10,
        };
        assert_eq!(summary.next(start, (90, 4_000), Vec::new()), None);
        let later = start + SUMMARY_EVERY;
        let busy = RtThread {
            ticks: 70,
            ..thread
        };
        let line = summary.next(later, (375, 9_000), vec![busy]).unwrap();
        assert_eq!(
            line,
            "diag: callbacks 375/s, callback busy 9 ms/s; rt 4321 data-loop.0 cpu 200 ms/s"
        );
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

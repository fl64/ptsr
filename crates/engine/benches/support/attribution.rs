//! Local observations around the offline benchmark, never inside sound processing.
use std::fs::{self, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use super::{Timing, gates_exceeded, measure};

pub const NOTES: &str = "Boundary CPU IDs and resource deltas cover a larger envelope than callback timing. Equal CPU IDs do not exclude intervening migrations.\nFrequency is a later scaling_cur_freq policy observation, not a frequency trace inside the callback.\nObservations can affect subsequent execution. No overhead is subtracted. Buffered rows are not crash-durable.\nKeep CSV files and actual measurement results local. Do not add them to Git.";

#[derive(Clone, Debug)]
pub struct Counters {
    pub voluntary: u64,
    pub involuntary: u64,
    pub minor_faults: u64,
    pub major_faults: u64,
}

#[derive(Clone, Debug)]
pub struct Boundary {
    pub cpu: Result<u32, String>,
    pub counters: Result<Counters, String>,
}

pub struct Observation {
    pub offset_ns: u128,
    pub timing: Timing,
    pub before: Boundary,
    pub after: Boundary,
}

pub struct Frequency {
    pub start_ns: u128,
    pub end_ns: u128,
    pub source: String,
    pub khz: Result<u64, String>,
}

struct Record {
    phase: &'static str,
    switch: u64,
    block: usize,
    observed: Observation,
    frequency: Frequency,
}

pub struct Trace {
    started: Instant,
    unix_start_ns: u128,
    budget: f64,
    records: Vec<Record>,
}

impl Trace {
    pub fn new(budget: f64) -> io::Result<Self> {
        if !budget.is_finite() || budget <= 0.0 {
            return Err(io::Error::other(
                "callback trace requires a positive finite budget",
            ));
        }
        let unix_start_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        Ok(Self {
            started: Instant::now(),
            unix_start_ns,
            budget,
            records: Vec::new(),
        })
    }

    pub fn observe_with(
        &self,
        mut boundary: impl FnMut() -> Boundary,
        timer: impl FnOnce() -> io::Result<Timing>,
    ) -> io::Result<Observation> {
        let offset_ns = self.started.elapsed().as_nanos();
        let before = boundary();
        let timing = timer()?;
        let after = boundary();
        Ok(Observation {
            offset_ns,
            timing,
            before,
            after,
        })
    }

    pub fn measure(
        &mut self,
        phase: &'static str,
        switch: u64,
        block: usize,
        call: impl FnOnce(),
    ) -> io::Result<Timing> {
        let observed = self.observe_with(read_boundary, || measure(call))?;
        let timing = observed.timing;
        let started = self.started;
        self.record_with(phase, switch, block, observed, |cpu| {
            read_frequency_at(Path::new("/sys/devices/system/cpu"), cpu, started)
        });
        Ok(timing)
    }

    pub fn record_with(
        &mut self,
        phase: &'static str,
        switch: u64,
        block: usize,
        observed: Observation,
        frequency: impl FnOnce(u32) -> Frequency,
    ) {
        if observed.timing.wall <= self.budget * 0.5 && observed.timing.cpu <= self.budget * 0.5 {
            return;
        }
        let frequency = match observed.after.cpu {
            Ok(cpu) => frequency(cpu),
            Err(ref error) => Frequency {
                start_ns: self.started.elapsed().as_nanos(),
                end_ns: self.started.elapsed().as_nanos(),
                source: "NA".into(),
                khz: Err(format!("after-CPU unavailable: {error}")),
            },
        };
        self.records.push(Record {
            phase,
            switch,
            block,
            observed,
            frequency,
        });
    }

    pub fn write(&self, out: &mut impl Write) -> io::Result<()> {
        writeln!(out, "# callback_trace_version=1")?;
        writeln!(out, "# run_start_unix_ns={}", self.unix_start_ns)?;
        writeln!(out, "# budget_ns={:.0}", self.budget * 1e9)?;
        for line in NOTES.lines() {
            writeln!(out, "# {line}")?;
        }
        writeln!(
            out,
            "phase,switch,block,envelope_offset_ns,wall_ns,cpu_ns,estimated_gap_ns,cpu_before,cpu_after,voluntary_delta,involuntary_delta,minor_faults_delta,major_faults_delta,frequency_read_start_ns,frequency_read_end_ns,frequency_source,frequency_khz"
        )?;
        for record in &self.records {
            let o = &record.observed;
            let f = &record.frequency;
            let fields = [
                record.phase.to_owned(),
                record.switch.to_string(),
                record.block.to_string(),
                o.offset_ns.to_string(),
                format!("{:.0}", o.timing.wall * 1e9),
                format!("{:.0}", o.timing.cpu * 1e9),
                format!("{:.0}", (o.timing.wall - o.timing.cpu).max(0.0) * 1e9),
                available(&o.before.cpu),
                available(&o.after.cpu),
                delta(o, |c| c.voluntary),
                delta(o, |c| c.involuntary),
                delta(o, |c| c.minor_faults),
                delta(o, |c| c.major_faults),
                f.start_ns.to_string(),
                f.end_ns.to_string(),
                f.source.clone(),
                available(&f.khz),
            ];
            for (i, field) in fields.iter().enumerate() {
                if i != 0 {
                    write!(out, ",")?;
                }
                // Quote every field: OS error messages can contain commas or quotes.
                write!(out, "\"{}\"", field.replace('"', "\"\""))?;
            }
            writeln!(out)?;
        }
        out.flush()
    }

    pub fn finish(
        &self,
        directory: &Path,
        steady_p99: f64,
        switch_p99: f64,
    ) -> io::Result<(PathBuf, bool)> {
        self.save(directory)
            .map(|path| (path, gates_exceeded(steady_p99, switch_p99)))
            .map_err(|e| io::Error::new(e.kind(), format!("callback diagnostic output: {e}")))
    }

    fn save(&self, directory: &Path) -> io::Result<PathBuf> {
        fs::create_dir_all(directory)?;
        for attempt in 0..64 {
            let path = directory.join(format!(
                "callback-{}-{}-{attempt}.csv",
                std::process::id(),
                self.unix_start_ns
            ));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => {
                    self.write(&mut BufWriter::new(file))?;
                    return Ok(path);
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "callback trace filenames are occupied",
        ))
    }
}

pub fn write_overhead(
    report: &super::Report,
    envelope_mean: f64,
    out: &mut impl Write,
) -> io::Result<()> {
    report.write("instrumented empty calls", out)?;
    writeln!(
        out,
        "observation-envelope mean: {:.3} us/call (boundary queries and record handling included)",
        envelope_mean * 1e6
    )?;
    writeln!(out, "{NOTES}")
}

fn available<T: std::fmt::Display>(value: &Result<T, String>) -> String {
    match value {
        Ok(value) => value.to_string(),
        Err(error) => format!("NA: {error}"),
    }
}

fn delta(o: &Observation, value: impl Fn(&Counters) -> u64) -> String {
    match (&o.before.counters, &o.after.counters) {
        (Ok(before), Ok(after)) => value(after)
            .checked_sub(value(before))
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NA: counter moved backwards".into()),
        (Err(error), _) => format!("NA: before: {error}"),
        (_, Err(error)) => format!("NA: after: {error}"),
    }
}

pub fn read_frequency_at(root: &Path, cpu: u32, started: Instant) -> Frequency {
    let source = root.join(format!("cpu{cpu}/cpufreq/scaling_cur_freq"));
    let start_ns = started.elapsed().as_nanos();
    let khz = fs::read_to_string(&source)
        .map_err(|e| e.to_string())
        .and_then(|text| {
            text.trim()
                .parse::<u64>()
                .ok()
                .filter(|&v| v > 0)
                .ok_or_else(|| "invalid positive kHz value".into())
        });
    let end_ns = started.elapsed().as_nanos();
    Frequency {
        start_ns,
        end_ns,
        source: source.to_string_lossy().into_owned(),
        khz,
    }
}

#[cfg(target_os = "linux")]
fn read_boundary() -> Boundary {
    // SAFETY: sched_getcpu has no arguments or caller-side preconditions.
    let cpu = unsafe { libc::sched_getcpu() };
    let cpu = u32::try_from(cpu).map_err(|_| io::Error::last_os_error().to_string());
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: usage points to writable storage of libc's native rusage layout.
    let status = unsafe { libc::getrusage(libc::RUSAGE_THREAD, usage.as_mut_ptr()) };
    let counters = if status == 0 {
        // SAFETY: successful getrusage initialized usage above.
        let usage = unsafe { usage.assume_init() };
        let convert = |n| u64::try_from(n).map_err(|_| "negative resource counter".to_string());
        (|| {
            Ok(Counters {
                voluntary: convert(usage.ru_nvcsw)?,
                involuntary: convert(usage.ru_nivcsw)?,
                minor_faults: convert(usage.ru_minflt)?,
                major_faults: convert(usage.ru_majflt)?,
            })
        })()
    } else {
        Err(io::Error::last_os_error().to_string())
    };
    Boundary { cpu, counters }
}

#[cfg(not(target_os = "linux"))]
fn read_boundary() -> Boundary {
    Boundary {
        cpu: Err("CPU ID requires Linux".into()),
        counters: Err("thread resource counters require Linux".into()),
    }
}

pub mod attribution;

use std::io::{self, Write};
use std::time::{Duration, Instant};

pub const TIMING_NOTES: &str = "Wall = elapsed time. Thread CPU = time accounted to this thread, including timer overhead.\nEstimated gap = max(wall - thread CPU, 0): estimate, not proof of a specific cause.\nExit checks unchanged: steady wall p99 or per-switch wall peak p99 > 50 % fails.\nA successful p99 check does not prove that all blocks met the target.";

#[derive(Clone, Copy, Debug)]
pub struct Timing {
    pub wall: f64,
    pub cpu: f64,
}

pub fn measure(call: impl FnOnce()) -> io::Result<Timing> {
    measure_with(call, thread_cpu_time)
}

pub fn measure_with(
    call: impl FnOnce(),
    mut cpu_clock: impl FnMut() -> io::Result<Duration>,
) -> io::Result<Timing> {
    let before = cpu_clock()
        .map_err(|e| io::Error::new(e.kind(), format!("thread CPU clock before callback: {e}")))?;
    let start = Instant::now();
    call();
    let wall = start.elapsed().as_secs_f64();
    let after = cpu_clock()
        .map_err(|e| io::Error::new(e.kind(), format!("thread CPU clock after callback: {e}")))?;
    let cpu = after
        .checked_sub(before)
        .ok_or_else(|| io::Error::other("thread CPU clock moved backwards"))?
        .as_secs_f64();
    Ok(Timing { wall, cpu })
}

#[cfg(target_os = "linux")]
pub fn thread_cpu_time() -> io::Result<Duration> {
    let mut value = std::mem::MaybeUninit::<libc::timespec>::uninit();
    // SAFETY: value points to writable storage with libc's native timespec layout.
    let status = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, value.as_mut_ptr()) };
    if status != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a successful clock_gettime initialized the timespec above.
    let value = unsafe { value.assume_init() };
    let seconds = u64::try_from(value.tv_sec)
        .map_err(|_| io::Error::other("thread CPU clock returned negative seconds"))?;
    let nanos = u32::try_from(value.tv_nsec)
        .ok()
        .filter(|&v| v < 1_000_000_000)
        .ok_or_else(|| io::Error::other("thread CPU clock returned invalid nanoseconds"))?;
    Ok(Duration::new(seconds, nanos))
}

#[cfg(not(target_os = "linux"))]
pub fn thread_cpu_time() -> io::Result<Duration> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "thread CPU clock requires Linux CLOCK_THREAD_CPUTIME_ID",
    ))
}

pub struct Summary {
    pub mean: f64,
    pub p99: f64,
    pub max: f64,
    pub count: usize,
    pub over_target: usize,
    budget: f64,
}

impl Summary {
    pub fn new(samples: &[f64], budget: f64) -> io::Result<Self> {
        if samples.is_empty()
            || !budget.is_finite()
            || budget <= 0.0
            || samples.iter().any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "timing summary needs samples, finite non-negative times and a positive finite budget",
            ));
        }
        let mut sorted = samples.to_vec();
        sorted.sort_by(f64::total_cmp);
        Ok(Self {
            mean: sorted.iter().sum::<f64>() / sorted.len() as f64,
            p99: sorted[sorted.len() * 99 / 100],
            max: sorted[sorted.len() - 1],
            count: sorted.len(),
            over_target: sorted
                .iter()
                .filter(|&&v| v / budget * 100.0 > 50.0)
                .count(),
            budget,
        })
    }

    fn write(&self, label: &str, out: &mut impl Write) -> io::Result<()> {
        let pct = |v: f64| v / self.budget * 100.0;
        writeln!(
            out,
            "  {label}: mean {:.2} %, p99 {:.2} %, max {:.2} %, samples {}, >50 % {}",
            pct(self.mean),
            pct(self.p99),
            pct(self.max),
            self.count,
            self.over_target
        )
    }
}

pub struct Report {
    pub wall: Summary,
    pub cpu: Summary,
    pub gap: Summary,
}

impl Report {
    pub fn new(samples: &[Timing], budget: f64) -> io::Result<Self> {
        let wall = samples.iter().map(|s| s.wall).collect::<Vec<_>>();
        let cpu = samples.iter().map(|s| s.cpu).collect::<Vec<_>>();
        // Pair before sorting: independent maxima could describe different callbacks.
        let gap = samples
            .iter()
            .map(|s| (s.wall - s.cpu).max(0.0))
            .collect::<Vec<_>>();
        Ok(Self {
            wall: Summary::new(&wall, budget)?,
            cpu: Summary::new(&cpu, budget)?,
            gap: Summary::new(&gap, budget)?,
        })
    }

    pub fn write(&self, label: &str, out: &mut impl Write) -> io::Result<()> {
        writeln!(out, "paired timing: {label}")?;
        self.wall.write("wall", out)?;
        self.cpu.write("thread CPU", out)?;
        self.gap.write("estimated gap", out)?;
        let status = |s: &Summary| {
            if s.over_target > 0 {
                "EXCEEDED"
            } else {
                "WITHIN"
            }
        };
        writeln!(
            out,
            "  maximum target (50 %): wall {}; thread CPU {}",
            status(&self.wall),
            status(&self.cpu)
        )
    }
}

pub fn write_overhead(report: &Report, envelope_mean: f64, out: &mut impl Write) -> io::Result<()> {
    report.write("empty calls (timer overhead)", out)?;
    writeln!(
        out,
        "empty-call means: wall {:.3} us, thread CPU {:.3} us, estimated gap {:.3} us",
        report.wall.mean * 1e6,
        report.cpu.mean * 1e6,
        report.gap.mean * 1e6
    )?;
    writeln!(
        out,
        "measurement-envelope mean: {:.3} us/call (sampling and storage included)",
        envelope_mean * 1e6
    )?;
    writeln!(out, "No overhead is subtracted from workload measurements.")
}

pub fn gates_exceeded(steady_wall_p99_pct: f64, switch_wall_peak_p99_pct: f64) -> bool {
    steady_wall_p99_pct > 50.0 || switch_wall_peak_p99_pct > 50.0
}

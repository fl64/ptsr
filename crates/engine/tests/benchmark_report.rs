#[path = "../benches/support/mod.rs"]
mod support;

use std::io;
use std::time::Duration;
use support::{Report, Summary, Timing, gates_exceeded, measure_with, thread_cpu_time};

const BUDGET: f64 = 0.002;

fn timing(wall: f64, cpu: f64) -> Timing {
    Timing { wall, cpu }
}

fn text(label: &str, samples: &[Timing]) -> String {
    let mut bytes = Vec::new();
    Report::new(samples, BUDGET)
        .unwrap()
        .write(label, &mut bytes)
        .unwrap();
    String::from_utf8(bytes).unwrap()
}

#[test]
fn benchmark_reports_both_clocks_for_every_phase() {
    let samples = [timing(0.0012, 0.0003), timing(0.0002, 0.0001)];
    for label in [
        "steady playback",
        "all project-switch callbacks",
        "first fade block",
        "second fade block",
        "clearing and restore",
    ] {
        assert_eq!(
            text(label, &samples),
            format!(
                "paired timing: {label}\n\
                 \x20 wall: mean 35.00 %, p99 60.00 %, max 60.00 %, samples 2, >50 % 1\n\
                 \x20 thread CPU: mean 10.00 %, p99 15.00 %, max 15.00 %, samples 2, >50 % 0\n\
                 \x20 estimated gap: mean 25.00 %, p99 45.00 %, max 45.00 %, samples 2, >50 % 0\n\
                 \x20 maximum target (50 %): wall EXCEEDED; thread CPU WITHIN\n"
            )
        );
    }
    // The first value, not only the last value, must reach each summary.
    let first = Report::new(&samples, BUDGET).unwrap();
    assert_eq!(first.wall.count, 2);
    assert_eq!(first.wall.max, 0.0012);
    let reversed = [samples[1], samples[0]];
    assert_eq!(text("phase", &samples), text("phase", &reversed));
}

#[test]
fn benchmark_exposes_a_rare_over_budget_block() {
    let mut samples = vec![timing(0.0002, 0.0001); 200];
    samples[0] = timing(0.0016, 0.0012);
    let report = Report::new(&samples, BUDGET).unwrap();
    assert_eq!(report.wall.p99, 0.0002);
    assert_eq!(report.wall.max, 0.0016);
    assert_eq!(report.wall.over_target, 1);
    assert_eq!(report.cpu.over_target, 1);
    assert!(!gates_exceeded(report.wall.p99 / BUDGET * 100.0, 10.0));
    let output = text("rare peak", &samples);
    assert!(output.contains("p99 10.00 %, max 80.00 %, samples 200, >50 % 1"));
    assert!(output.contains("wall EXCEEDED; thread CPU EXCEEDED"));
    assert!(support::TIMING_NOTES.contains("steady wall p99 or per-switch wall peak p99 > 50 %"));
    assert!(support::TIMING_NOTES.contains("does not prove that all blocks met the target"));
}

#[test]
fn benchmark_keeps_its_existing_p99_exit_checks() {
    assert!(!gates_exceeded(50.0, 50.0));
    assert!(gates_exceeded(50.000001, 0.0));
    assert!(gates_exceeded(0.0, 50.000001));
    let report = Report::new(&[timing(0.000999, 0.001001), timing(0.001, 0.002)], BUDGET).unwrap();
    assert_eq!(report.wall.over_target, 0);
    assert_eq!(report.cpu.over_target, 2);
    assert!(!gates_exceeded(report.wall.p99 / BUDGET * 100.0, 50.0));
    let exact = Summary::new(&[0.0, 0.001, 0.001000001], BUDGET).unwrap();
    assert_eq!(exact.over_target, 1);
    // Keep the original n*99/100 index, without percentile interpolation.
    let hundred: Vec<_> = (0..100).map(|i| i as f64 / 100_000.0).collect();
    assert_eq!(Summary::new(&hundred, BUDGET).unwrap().p99, hundred[99]);
}

#[test]
fn benchmark_bounds_the_estimated_delay() {
    // Independent sorting before subtraction would lose the 50% gap.
    let report = Report::new(&[timing(0.0014, 0.0004), timing(0.0004, 0.0005)], BUDGET).unwrap();
    assert!((report.gap.max - 0.001).abs() < 1e-12);
    assert!((report.gap.mean - 0.0005).abs() < 1e-12);
    assert_eq!(report.gap.over_target, 0);
    let zero = Report::new(&[timing(0.0001, 0.0002)], BUDGET).unwrap();
    assert_eq!(zero.gap.max, 0.0);
    assert!(text("gap", &[timing(0.0001, 0.0002)]).contains("estimated gap: mean 0.00 %"));
    assert!(support::TIMING_NOTES.contains("not proof of a specific cause"));
}

#[test]
fn benchmark_reports_timer_overhead_without_correction() {
    let mut calls = 0;
    let mut reads = [Duration::from_nanos(20), Duration::from_nanos(120)].into_iter();
    let sample = measure_with(|| calls += 1, || Ok(reads.next().unwrap())).unwrap();
    assert_eq!(calls, 1);
    assert_eq!(sample.cpu, 100e-9);
    let mut bytes = Vec::new();
    let report = Report::new(&[sample], BUDGET).unwrap();
    support::write_overhead(&report, 0.000002, &mut bytes).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("paired timing: empty calls (timer overhead)"));
    assert!(output.contains("thread CPU 0.100 us"));
    assert!(
        output.contains("measurement-envelope mean: 2.000 us/call (sampling and storage included)")
    );
    assert!(output.contains("No overhead is subtracted from workload measurements."));
    let raw = Report::new(&[timing(0.0011, 0.0009)], BUDGET).unwrap();
    assert_eq!(raw.wall.max, 0.0011);
    assert!(gates_exceeded(raw.wall.p99 / BUDGET * 100.0, 0.0));
}

#[test]
fn benchmark_clock_failure_is_explicit() {
    for failed_read in [0, 1] {
        let mut reads = 0;
        let mut calls = 0;
        let error = measure_with(
            || calls += 1,
            || {
                let current = reads;
                reads += 1;
                if current == failed_read {
                    Err(io::Error::from_raw_os_error(22))
                } else {
                    Ok(Duration::from_nanos(100))
                }
            },
        )
        .unwrap_err();
        assert_eq!(calls, failed_read);
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("thread CPU clock"));
        assert!(
            error
                .to_string()
                .contains(if failed_read == 0 { "before" } else { "after" })
        );
        // Exercise non-zero command exit and its diagnostic without a sound device.
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "clock_failure_probe_child", "--nocapture"])
            .env("PTSR_TEST_CLOCK_FAILURE", failed_read.to_string())
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("thread CPU clock"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("paired timing:"));
    }
    let mut values = [Duration::from_nanos(2), Duration::from_nanos(1)].into_iter();
    let error = measure_with(|| {}, || Ok(values.next().unwrap())).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("thread CPU clock moved backwards")
    );
}

#[test]
fn clock_failure_probe_child() -> io::Result<()> {
    let Ok(failed) = std::env::var("PTSR_TEST_CLOCK_FAILURE") else {
        return Ok(());
    };
    let mut read = 0;
    let sample = measure_with(
        || {},
        || {
            let current = read;
            read += 1;
            if current.to_string() == failed {
                Err(io::Error::from_raw_os_error(22))
            } else {
                Ok(Duration::ZERO)
            }
        },
    )?;
    let mut out = Vec::new();
    Report::new(&[sample], BUDGET)?.write("must not print", &mut out)?;
    println!("{}", String::from_utf8_lossy(&out));
    Ok(())
}

#[test]
fn benchmark_reporting_stays_outside_sound_processing() {
    let make_realtime = || {
        let (mut tx, rx) = rtrb::RingBuffer::new(4);
        tx.push(engine::rt::Command::PadPress {
            pad: 0,
            velocity: 1.0,
        })
        .unwrap();
        engine::rt::Realtime::new(
            engine::Engine::with_sounds(48_000, vec![], vec![vec![0.2; 512]]),
            vec![rx],
            std::sync::Arc::new(engine::rt::Stats::default()),
            48_000,
        )
    };
    let mut realtime = make_realtime();
    let mut direct = make_realtime();
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    let (mut direct_left, mut direct_right) = ([0.0; 128], [0.0; 128]);
    let mut reads = 0;
    let sample = measure_with(
        || realtime.process(&mut left, &mut right),
        || {
            reads += 1;
            Ok(Duration::from_nanos(reads * 10))
        },
    )
    .unwrap();
    assert_eq!(reads, 2);
    assert_eq!(sample.cpu, 10e-9);
    direct.process(&mut direct_left, &mut direct_right);
    assert_eq!(left, direct_left);
    assert_eq!(right, direct_right);
    assert!(left.iter().any(|v| *v != 0.0));
    assert!(left.iter().chain(&right).all(|v| v.is_finite()));
    // The real clock needs neither an audio server nor privileged scheduling.
    let before = thread_cpu_time().unwrap();
    let actual = support::measure(|| realtime.process(&mut left, &mut right)).unwrap();
    assert!(thread_cpu_time().unwrap() >= before);
    assert!(actual.cpu >= 0.0 && actual.wall >= 0.0);
}

use support::attribution::{Boundary, Counters, Frequency, Observation, Trace};

fn boundary(cpu: u32) -> Boundary {
    Boundary {
        cpu: Ok(cpu),
        counters: Ok(Counters {
            voluntary: 1,
            involuntary: 2,
            minor_faults: 3,
            major_faults: 0,
        }),
    }
}

fn observed(wall: f64, cpu: f64) -> Observation {
    Observation {
        offset_ns: 42,
        timing: timing(wall, cpu),
        before: boundary(2),
        after: boundary(3),
    }
}

fn frequency(cpu: u32) -> Frequency {
    assert_eq!(cpu, 3);
    Frequency {
        start_ns: 43,
        end_ns: 44,
        source: "fixture/scaling_cur_freq".into(),
        khz: Ok(1_000),
    }
}

fn trace_text(trace: &Trace) -> String {
    let mut out = Vec::new();
    trace.write(&mut out).unwrap();
    String::from_utf8(out).unwrap()
}

struct TraceFiles(std::path::PathBuf);

impl TraceFiles {
    fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/attribution-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Relaxed)
            ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TraceFiles {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn benchmark_retains_every_overrun_identity() {
    let mut trace = Trace::new(BUDGET).unwrap();
    let mut first = observed(0.0016, 0.0003);
    first.after.counters = Ok(Counters {
        voluntary: 2,
        involuntary: 6,
        minor_faults: 5,
        major_faults: 7,
    });
    trace.record_with("steady playback", 0, 0, first, frequency);
    trace.record_with("steady playback", 0, 9, observed(0.0008, 0.0012), frequency);
    trace.record_with(
        "first fade block",
        1,
        0,
        observed(0.0016, 0.0012),
        frequency,
    );
    trace.record_with(
        "second fade block",
        200,
        1,
        observed(0.0016, 0.0012),
        frequency,
    );
    trace.record_with(
        "clearing and restore",
        200,
        111,
        observed(0.0016, 0.0012),
        frequency,
    );
    for sample in [observed(0.001, 0.001), observed(0.0001, 0.0001)] {
        trace.record_with("below", 0, 1, sample, |_| {
            panic!("frequency read below target")
        });
    }
    let text = trace_text(&trace);
    let rows: Vec<_> = text.lines().filter(|line| line.starts_with('"')).collect();
    assert_eq!(rows.len(), 5);
    assert!(
        rows[0].starts_with(
            "\"steady playback\",\"0\",\"0\",\"42\",\"1600000\",\"300000\",\"1300000\""
        )
    );
    assert!(rows[0].contains("\"2\",\"3\",\"1\",\"4\",\"2\",\"7\""));
    assert!(
        rows[1].starts_with("\"steady playback\",\"0\",\"9\",\"42\",\"800000\",\"1200000\",\"0\"")
    );
    assert!(rows[2].starts_with("\"first fade block\",\"1\",\"0\""));
    assert!(rows[3].starts_with("\"second fade block\",\"200\",\"1\""));
    assert!(rows[4].starts_with("\"clearing and restore\",\"200\",\"111\""));
    assert!(text.contains("larger envelope than callback timing"));
    assert!(text.contains("Equal CPU IDs do not exclude intervening migrations"));
}

#[test]
fn benchmark_reports_unavailable_boundary_data() {
    for failure in 0..5 {
        let mut trace = Trace::new(BUDGET).unwrap();
        let mut sample = observed(0.0008, 0.0012);
        match failure {
            0 => sample.before.cpu = Err("CPU read refused".into()),
            1 => sample.after.cpu = Err("CPU read refused".into()),
            2 => sample.before.counters = Err("resource read refused".into()),
            3 => sample.after.counters = Err("resource read refused".into()),
            4 => sample.after.counters.as_mut().unwrap().minor_faults = 2,
            _ => unreachable!(),
        }
        let mut frequency_calls = 0;
        trace.record_with("steady playback", 0, 0, sample, |cpu| {
            frequency_calls += 1;
            frequency(cpu)
        });
        assert_eq!(frequency_calls, usize::from(failure != 1));
        let text = trace_text(&trace);
        assert!(text.contains("\"800000\",\"1200000\",\"0\""));
        assert!(text.contains(match failure {
            0 | 1 => "NA: CPU read refused",
            2 => "NA: before: resource read refused",
            3 => "NA: after: resource read refused",
            _ => "NA: counter moved backwards",
        }));
        if failure == 1 {
            assert!(text.contains("NA: after-CPU unavailable"));
        }
    }
}

#[test]
fn benchmark_labels_later_frequency_samples() {
    let files = TraceFiles::new();
    let path = files.0.join("cpu3/cpufreq/scaling_cur_freq");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let start = std::time::Instant::now();
    let missing = support::attribution::read_frequency_at(&files.0, 3, start);
    assert!(missing.khz.is_err());
    for value in ["1000\n", "0", "not a frequency", ""] {
        std::fs::write(&path, value).unwrap();
        let freq = support::attribution::read_frequency_at(&files.0, 3, start);
        assert!(freq.end_ns >= freq.start_ns);
        assert_eq!(std::path::Path::new(&freq.source), path);
        if value == "1000\n" {
            assert_eq!(freq.khz, Ok(1000));
        } else {
            assert!(freq.khz.is_err());
        }
    }
    let mut trace = Trace::new(BUDGET).unwrap();
    trace.record_with("steady playback", 0, 0, observed(0.0016, 0.0012), frequency);
    let text = trace_text(&trace);
    assert!(text.contains("\"43\",\"44\",\"fixture/scaling_cur_freq\",\"1000\""));
    assert!(text.contains("not a frequency trace inside the callback"));
    let mut trace = Trace::new(BUDGET).unwrap();
    trace.record_with("steady playback", 0, 0, observed(0.0016, 0.0012), |_| {
        missing
    });
    assert!(trace_text(&trace).contains("NA:"));
}

#[test]
fn benchmark_saves_records_before_exit_checks() {
    let files = TraceFiles::new();
    let mut trace = Trace::new(BUDGET).unwrap();
    let (empty, exceeded) = trace.finish(&files.0, 50.0, 50.0).unwrap();
    assert!(!exceeded);
    let bytes = std::fs::read(&empty).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert!(text.contains("phase,switch,block,"));
    assert!(!text.lines().any(|line| line.starts_with('"')));
    trace.record_with("steady playback", 0, 0, observed(0.0016, 0.0012), frequency);
    let (overrun, exceeded) = trace.finish(&files.0, 51.0, 0.0).unwrap();
    assert!(exceeded);
    assert_ne!(overrun, empty);
    assert_eq!(std::fs::read(&empty).unwrap(), bytes);
    assert!(
        std::fs::read_to_string(overrun)
            .unwrap()
            .contains("\"1600000\"")
    );
    for phase in ["steady", "switch"] {
        let destination = files.0.join(phase);
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "attribution_exit_probe_child", "--nocapture"])
            .env("PTSR_TEST_TRACE_OUTPUT", &destination)
            .env("PTSR_TEST_TRACE_GATE", phase)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("fixture p99 exceeded"));
        let csv = std::fs::read_dir(destination)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert!(
            std::fs::read_to_string(csv)
                .unwrap()
                .contains("\"steady playback\",\"0\",\"0\"")
        );
    }
}

#[test]
fn attribution_exit_probe_child() -> io::Result<()> {
    let Some(directory) = std::env::var_os("PTSR_TEST_TRACE_OUTPUT") else {
        return Ok(());
    };
    let mut trace = Trace::new(BUDGET)?;
    trace.record_with("steady playback", 0, 0, observed(0.0016, 0.0012), frequency);
    let steady = std::env::var("PTSR_TEST_TRACE_GATE").unwrap() == "steady";
    let (_, exceeded) = trace.finish(
        std::path::Path::new(&directory),
        if steady { 51.0 } else { 0.0 },
        if steady { 0.0 } else { 51.0 },
    )?;
    if exceeded {
        return Err(io::Error::other("fixture p99 exceeded"));
    }
    Ok(())
}

#[test]
fn benchmark_refuses_an_output_failure() {
    use std::io::Write;
    let files = TraceFiles::new();
    let blocked = files.0.join("not-a-directory");
    std::fs::write(&blocked, b"keep me").unwrap();
    let trace = Trace::new(BUDGET).unwrap();
    let error = trace.finish(&blocked, 0.0, 0.0).unwrap_err();
    assert!(error.to_string().contains("callback diagnostic output"));
    assert_eq!(std::fs::read(&blocked).unwrap(), b"keep me");
    struct Refuse {
        flush: bool,
    }
    impl Write for Refuse {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.flush {
                Ok(bytes.len())
            } else {
                Err(io::Error::other("fixture write refusal"))
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("fixture flush refusal"))
        }
    }
    for flush in [false, true] {
        let error = trace.write(&mut Refuse { flush }).unwrap_err();
        assert!(error.to_string().contains(if flush {
            "fixture flush refusal"
        } else {
            "fixture write refusal"
        }));
    }
}

#[test]
fn benchmark_observations_preserve_sound_and_gates() {
    let make_realtime = || {
        let (mut tx, rx) = rtrb::RingBuffer::new(4);
        tx.push(engine::rt::Command::PadPress {
            pad: 0,
            velocity: 1.0,
        })
        .unwrap();
        engine::rt::Realtime::new(
            engine::Engine::with_sounds(48_000, vec![], vec![vec![0.2; 512]]),
            vec![rx],
            std::sync::Arc::new(engine::rt::Stats::default()),
            48_000,
        )
    };
    let (mut traced, mut direct) = (make_realtime(), make_realtime());
    let (mut l, mut r, mut dl, mut dr) = ([0.0; 128], [0.0; 128], [0.0; 128], [0.0; 128]);
    let mut trace = Trace::new(BUDGET).unwrap();
    let mut calls = 0;
    let actual = trace
        .measure("steady playback", 0, 0, || {
            calls += 1;
            traced.process(&mut l, &mut r);
        })
        .unwrap();
    direct.process(&mut dl, &mut dr);
    assert_eq!(calls, 1);
    assert_eq!((l, r), (dl, dr));
    assert!(l.iter().any(|v| *v != 0.0));
    assert!(l.iter().chain(&r).all(|v| v.is_finite()));
    assert!(actual.cpu >= 0.0 && actual.wall >= 0.0);
    let mut boundaries = 0;
    let mut timer_calls = 0;
    let sample = trace
        .observe_with(
            || {
                boundaries += 1;
                boundary(3)
            },
            || {
                timer_calls += 1;
                Ok(timing(0.0008, 0.0012))
            },
        )
        .unwrap();
    assert_eq!((boundaries, timer_calls), (2, 1));
    assert_eq!((sample.timing.wall, sample.timing.cpu), (0.0008, 0.0012));
    let error = trace
        .observe_with(
            || boundary(3),
            || Err(io::Error::other("fixture clock error")),
        )
        .err()
        .unwrap();
    assert_eq!(error.to_string(), "fixture clock error");
    assert!(!gates_exceeded(50.0, 50.0));
    assert!(gates_exceeded(50.000001, 0.0));
    assert!(gates_exceeded(0.0, 50.000001));
}

#[test]
fn benchmark_reports_observation_overhead() {
    let report = Report::new(&[timing(0.0016, 0.0012)], BUDGET).unwrap();
    let mut bytes = Vec::new();
    support::write_overhead(&report, 0.000002, &mut bytes).unwrap();
    support::attribution::write_overhead(&report, 0.000004, &mut bytes).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("empty calls (timer overhead)"));
    assert!(text.contains("instrumented empty calls"));
    assert!(text.contains("observation-envelope mean: 4.000 us/call"));
    assert!(text.contains("Observations can affect subsequent execution"));
    assert!(text.contains("No overhead is subtracted"));
    assert!(text.contains("max 80.00 %"));
}

#[test]
fn benchmark_trace_escapes_csv_errors() {
    let mut trace = Trace::new(BUDGET).unwrap();
    let mut sample = observed(0.0016, 0.0012);
    sample.before.cpu = Err("refused, \"fixture\"".into());
    trace.record_with("steady playback", 0, 0, sample, frequency);
    assert!(trace_text(&trace).contains("\"NA: refused, \"\"fixture\"\"\""));
    for budget in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(Trace::new(budget).is_err());
    }
}

#[test]
fn benchmark_rejects_invalid_summary_inputs() {
    assert!(Summary::new(&[], BUDGET).is_err());
    assert!(Report::new(&[], BUDGET).is_err());
    for budget in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(Summary::new(&[0.001], budget).is_err());
    }
    for value in [-0.001, f64::NAN, f64::INFINITY] {
        assert!(Summary::new(&[value], BUDGET).is_err());
        assert!(Report::new(&[timing(0.001, value)], BUDGET).is_err());
    }
}

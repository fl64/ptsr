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

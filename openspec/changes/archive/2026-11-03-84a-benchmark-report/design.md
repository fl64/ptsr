## Context

See proposal.md for scope. The owner confirmed the plan, `libc` as a dev-dependency
and the current branch on 2026-11-03. Baseline: `80bbe4d`. Change 84 is archived;
84a is the only active change. Input modes 85 remains planned, not active.

At this baseline, `crates/engine/benches/callback.rs` measures 20,000 steady callbacks
and 200 project switches with Instant. The first two fade blocks and later clearing
blocks have separate summaries. The exit checks at lines 98-102 use steady wall p99
and per-switch wall peak p99, not the maximum. Keep their exact strict comparison.

The temporary probe in `target/bench-diagnostics/` linked the unchanged release engine
and copied the benchmark setup byte for byte. Across 200,000 paired measurements it
separated scheduler pauses from CPU time. One fast-core block took 5576.831 us wall
and 494.621 us CPU with 22 involuntary switches. On CPU 12, 15 of 40,000 blocks had CPU
time above half the budget, with a maximum of 61.733%. These are diagnostic observations,
not a resolved engine profile or a physical audio acceptance. Do not infer a universal
CPU-core policy or claim all earlier outliers have the same cause.

## Goals / Non-Goals

Use the production benchmark workload with two clocks and an honest report. Keep the
first block, queued setup commands, sustained voices and project transition ordering.
Do not move measurement work into Realtime, optimize audio, change a performance target
or use the new CPU measurements as a replacement exit gate.

## Decisions

### Benchmark-only support

Put the helper in `crates/engine/benches/support/mod.rs`, not in the engine public API.
Use it from callback.rs and from `crates/engine/tests/benchmark_report.rs` through a
path module. This makes calculations and the exact formatted report testable without
starting the full benchmark. No live App path calls it. Keep benchmark setup unchanged.

### Clock and pairing

Use libc's Linux CLOCK_THREAD_CPUTIME_ID and its native timespec, rather than copying
x86_64 FFI structures from the diagnostic. libc 0.2.191 is already locked and installed;
the approved addition is a dev-dependency, not a new runtime dependency or version bump.
Its installed manifest states MIT OR Apache-2.0 and Rust 1.65. No platform-specific
syscall numbers, perf permissions or privileged scheduler settings are needed.

Read the CPU clock before Instant starts and after Instant ends. CPU timing therefore
also contains the two wall-clock reads and small wrapper overhead. Keep both raw times;
clamp wall-minus-CPU at zero and call it an estimate. It can include scheduling and other
unaccounted time, and does not identify the cause. Pass a CPU-clock function into the
measurement helper so tests can simulate both read failures. Do not fabricate a value.
A clock failure propagates to main as an error naming the clock and a non-zero exit.

Measure 2,000 empty calls once before workload setup. Report the empty wall/CPU/gap
series and total measurement-envelope mean. Include sampling work and storage in that
envelope; do not subtract overhead from any audio result or adjust a threshold by it.
No warmup or sample removal is introduced into the existing audio workload.

### Statistics and output

Retain a paired wall/CPU duration for each callback. Compute each gap before sorting,
not by subtracting independently sorted series. Use the existing p99 index n*99/100.
Use all samples and count durations strictly above half the same block budget. Print
mean, p99, maximum, sample count and over-50% count for wall, CPU and estimated gap.
Print an explicit maximum-target status for each workload/phase. State separately that
exit status still uses steady wall p99 and per-switch wall peak p99. Preserve the old
wall summary lines and the original switch-peak calculation. A passing gate is not
an all-block pass. Empty timing sets and invalid budgets return errors, not panics.

Keep real per-switch wall peaks apart from paired callback measurements. Do not form
fake pairs by taking an independent CPU maximum and wall maximum from different blocks.
No new CSV output, core-affinity default or context-switch sampling is needed in the
shipped report; the earlier probe remains a separate local diagnostic.

### Verification

Use synthetic durations for all seven delta scenarios: first/last samples, exact 50%,
a single rare maximum, CPU greater than wall, overhead without subtraction, and CPU
read errors before/after the callback. Exercise the real Linux clock without asserting
machine speed. Check formatted output for every workload and phase and gate meaning.
The complete offline benchmark covers real callback wiring; existing no-allocation and
sound tests stay unchanged. No new sound feature or WAV is needed. Verify that
`renders/84-input-parity.wav` keeps its prior SHA-256 after the full suite.

## Risks / Trade-offs

- Clock reads perturb measurement slightly: show overhead, keep raw durations and avoid
  presenting CPU time as a perfect instruction-cost profile.
- Scheduler and CPU types affect results: show both distributions without changing
  affinity or claiming a single run proves a hard maximum.
- p99 success can hide rare deadline misses: print counts and target exceedance explicitly,
  retain the agreed exit policy, and keep the slow-core CPU issue open.

## Open questions

None for this report change. The source of slow-core CPU spikes is still a separate
investigation, not a deferred implementation decision or permission to optimize here.

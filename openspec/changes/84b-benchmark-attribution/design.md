## Context

The existing benchmark measures wall and thread CPU time around the real offline callback.
It sorts summary series independently. Summary maxima can describe different blocks.
Per-block records must retain the original pairing and identity.

## Goals / Non-Goals

Goals: retain slow-block evidence automatically, expose unavailable observations and measure observation overhead.
Non-goals: identify every possible cause, repair old traces, optimize sound, change scheduling or run live audio.
Only generic implementation and instructions belong in Git. All actual observations remain local.

## Decisions

1. Add a benchmark-only recorder around the existing measurement function.
   Read thread resource counters and CPU IDs before and after that measurement.
   These boundary observations cover a larger interval than the callback. Do not label their deltas as callback-only events.
2. Use existing `libc` support for Linux `getrusage(RUSAGE_THREAD)` and `sched_getcpu`.
   Record voluntary/involuntary switches and minor/major faults. Do not infer a waiting process or elapsed wait from counts.
3. Read `scaling_cur_freq` for the after-CPU only after an overrun.
   Record the observation start/end offsets and source. This value can be a policy estimate, not an actual instantaneous frequency.
   Equal boundary CPU IDs do not prove that no migration occurred between them.
4. Use monotonic offsets from a recorded Unix run-start time.
   Add phase, switch number and zero-based block number to each row.
5. Buffer overrun rows in memory. Write one exclusive, unique CSV file after measurement and before evaluating the existing exit gates.
   Keep a header and run metadata when there are no overruns. Do not overwrite older files.
   Output failures fail the command. A killed process can lose buffered rows, so this is not crash-durable tracing.
6. Keep all new queries, allocation and output outside `Realtime::process` and its wall/CPU timing interval.
   Report both the original empty-call overhead and the instrumented observation-envelope overhead.
   Query overhead can affect subsequent execution. Do not subtract it or call the instrumentation free.
7. Do not change ordinary p99 gates, the workload or first-block inclusion.
   Retain callback output identity with an automated direct-versus-observed sound test.

Rejected: logging inside the callback, because it violates real-time rules and contaminates measured work.
Rejected: a privileged scheduler tracer or a frequency polling thread, because this small diagnostic needs neither system changes nor concurrent I/O.
Rejected: recording only the largest blocks, because a smaller CPU overrun can have a different cause.

## Edge review

| Case | Expected result | Status | Test |
| --- | --- | --- | --- |
| Only CPU crosses 50%, or equality | Include CPU-only overruns, exclude equality | Closed in tests | `benchmark_retains_every_overrun_identity` |
| First block or later switch | Preserve phase and index without truncation | Closed in tests | `benchmark_retains_every_overrun_identity` |
| Missing CPU, resource counters or frequency | Explicit missing field and reason, never invented zero | Closed in tests | `benchmark_reports_unavailable_boundary_data`, `benchmark_labels_later_frequency_samples` |
| Counters move backwards | Mark delta unavailable | Closed in tests | `benchmark_reports_unavailable_boundary_data` |
| p99 failure after measurement | Flush the CSV before the failure | Closed in tests | `benchmark_saves_records_before_exit_checks` |
| Output failure or name collision | Clear error or fresh exclusive name, no replacement | Closed in tests | `benchmark_refuses_an_output_failure`, `benchmark_saves_records_before_exit_checks` |
| Migration away and back | Do not infer continuous CPU residence | Closed as a documented limitation | `benchmark_retains_every_overrun_identity` |
| Process killed before output | Buffered rows can be lost | Out of scope | Documented limitation |

## Open questions

None. The owner approved generic code and tests in Git, but not machine observations or actual results.
The callback maximum-budget failure remains separate work. This diagnostic does not activate its correction.

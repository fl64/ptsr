## ADDED Requirements

### Requirement: Callback benchmark timing report

The offline callback benchmark SHALL keep its existing 32-voice workload, sample rate,
block size, measured blocks and project-switch phases. For steady playback, all measured
project-switch callbacks and each switch phase, it SHALL report elapsed wall time,
thread CPU time and a non-negative estimate of their difference. Each timing series
SHALL show mean, p99, maximum, sample count and count strictly above 50% of the block
budget. The difference SHALL be named an estimate, not attributed to a specific cause.
An observed CPU time greater than wall time SHALL give a zero difference, not a negative
value. Timer overhead SHALL be reported from empty calls and SHALL NOT be subtracted.
No clock query, allocation or report output SHALL be added inside sound processing.

The 50% performance target SHALL stay unchanged. The command SHALL retain its existing
failure checks: steady wall-time p99 or per-switch wall-time peak p99 strictly above
50% fails. The report SHALL say when any measured wall or CPU maximum exceeds 50%,
even when those existing exit checks pass. A successful p99 check SHALL NOT be described
as proof that all blocks met the target. New output SHALL state which checks control
exit status. The existing wall-time summaries SHALL remain available.

If reading the thread CPU clock fails, the command SHALL stop with a clear error and
a non-zero exit status. It SHALL NOT replace missing CPU time with zero or silently
omit the CPU report. The benchmark SHALL need no sound device or controller, change
no system scheduling settings, and keep sound, live load reporting and saved data unchanged.

#### Scenario: Benchmark reports both clocks for every phase
- **WHEN** the offline benchmark measures steady playback and project switches
- **THEN** the report includes wall, thread CPU and estimated-gap statistics and counts
  for steady playback, all switch callbacks, and the three existing switch phases
- **AND** each summary includes every measured block, including the first one

#### Scenario: Benchmark exposes a rare over-budget block
- **WHEN** fewer than one percent of blocks exceed the 50% target and p99 stays below it
- **THEN** the maximum and exact exceedance count remain visible and the report names
  the target exceedance even though the existing p99 exit checks pass

#### Scenario: Benchmark keeps its existing p99 exit checks
- **WHEN** steady wall p99 or per-switch wall peak p99 crosses 50%
- **THEN** that check fails exactly as before, while equality at 50% still passes
- **AND** a maximum or CPU value alone does not replace these existing exit checks

#### Scenario: Benchmark bounds the estimated delay
- **WHEN** wall and CPU durations differ, including CPU time slightly greater than wall
- **THEN** the gap is the wall duration minus CPU duration clamped at zero, is labelled
  as an estimate, and is not claimed to prove which event caused it

#### Scenario: Benchmark reports timer overhead without correction
- **WHEN** the benchmark measures empty calls before the sound workload
- **THEN** it reports timer and full measurement-envelope overhead separately
- **AND** workload durations and the existing p99 checks use uncorrected measurements

#### Scenario: Benchmark clock failure is explicit
- **WHEN** a CPU clock read fails before or after a measured call
- **THEN** the command returns an error that names the thread CPU clock and fails
- **AND** no successful report with invented zero CPU time is emitted

#### Scenario: Benchmark reporting stays outside sound processing
- **WHEN** the benchmark runs without an audio device or controller
- **THEN** the extra clock reads and reporting wrap the same real sound callback
- **AND** no sound callback, live load, scheduling setting or audio output is changed

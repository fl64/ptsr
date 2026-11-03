# audio-engine Specification

## Purpose
Audio engine: low-latency sound output without blocking the audio thread, observability (latency, load, xruns) and offline render of a scenario to WAV with the same engine.

## Requirements

### Requirement: Low-latency sound output
The system SHALL output sound through JACK (PipeWire-JACK) with the buffer size set in the config (128 samples by default), and show the latency in the status line.

#### Scenario: Start with PipeWire running
- **WHEN** the owner starts the program on a system with PipeWire-JACK
- **THEN** the program connects to the sound server and shows in the status the buffer size, the sample rate and the latency in milliseconds

#### Scenario: Sound server unavailable
- **WHEN** JACK/PipeWire is unavailable
- **THEN** the program does not crash, shows a clear message in English and tries the fallback output

### Requirement: Audio thread does not block
The system SHALL process sound in the audio thread without memory allocation, locks or I/O; communication with other threads goes only through lock-free queues and atomics.

#### Scenario: UI hangs
- **WHEN** the UI thread does not respond for 2 seconds
- **THEN** the sound keeps playing without dropouts

### Requirement: Sound observability
The system SHALL show in the status line the audio thread load in percent and a counter of buffer underruns (xruns), and on the pads screen the master output level of each channel: a bar in dB, a peak-hold mark, and an overload light that latches while the output clips and goes out on the next hit that does not clip. The levels come from the mixed output itself, so they match what is heard and what an offline render contains.

#### Scenario: Buffer underrun
- **WHEN** the audio thread did not fill the buffer in time
- **THEN** the xrun counter goes up and is highlighted

#### Scenario: Meters follow the sound
- **WHEN** a pad plays at −6 dB peak and the owner watches the meters
- **THEN** both channel bars rise to −6 dB and fall when the sound ends, and the peak mark stays briefly at the top

#### Scenario: Overload latches
- **WHEN** layered pads push the output to a full-scale sample and then a single quiet pad plays
- **THEN** the overload light comes on with the clip and goes out only after the quiet hit

### Requirement: Offline render
The system SHALL be able to play a scenario of events without a sound card and save the result to WAV, using the same engine as live.

#### Scenario: Render of a test pattern
- **WHEN** a render of a scenario of 8 clicks with a step of 1/8 at 120 BPM is run
- **THEN** a WAV is created where each click starts at its own sample with ±1 precision, without clipping and NaN

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

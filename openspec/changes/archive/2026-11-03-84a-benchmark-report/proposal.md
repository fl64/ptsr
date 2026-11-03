## Why

A large callback time can mean slow sound processing or time spent waiting to run.
The current benchmark shows only elapsed time and checks p99, so a successful command
can still contain blocks above the 50% target. The owner needs these results apart.

## What Changes

- Keep the existing workload and wall-time summary for playback and project switches.
- Add thread CPU time, an estimated wall-minus-CPU gap and counts above 50%.
- Report each measured maximum and say when any block exceeds the target, even if
  the existing p99 exit checks pass. Do not describe that exit status as a maximum pass.
- Show empty-call timer overhead without subtracting it from the measurements.
- Fail with a clear error if the thread CPU clock cannot be read.
- Test calculation, output, clock errors and the unchanged p99 checks without relying
  on a machine being fast or on a scheduler pause happening during a test.

## Capabilities

### Modified Capabilities

- `audio-engine`: add an offline callback benchmark timing report. Live audio load
  meters, sound processing and the existing performance target stay unchanged.

## Impact

The engine benchmark, a benchmark-only helper and its integration tests, and owner
documentation. Add `libc` as an engine dev-dependency for the system clock ABI. Version
0.2.191 is already in Cargo.lock and installed locally; no package version update is
needed. Its MIT OR Apache-2.0 license is compatible with this project's GPL license.
It does not become an engine runtime dependency. No new audio render is needed because
no sound behavior changes; existing renders and allocation tests remain applicable.

The owner approved this work, the dev-dependency and continuation on
`plan/controller-first-patterns`. This is the branch exception for 84a. The baseline
is `80bbe4d`, the accepted change 84 archive. Push and PR need a separate request.

## Out of scope

Engine optimization, new audio-thread work, sound or key changes, a new performance
limit, stronger or weaker exit checks, system priority or affinity settings, live
sound, controller verification and input modes 85. A gap estimate does not identify
which program or kernel event caused a delay. CPU spikes on slow cores remain open.

## How to verify

1. Run `cargo test -p engine --test benchmark_report`. Check the synthetic timing,
   threshold, output and error cases.
2. Run `cargo bench -p engine --bench callback`. Read the wall and thread CPU lines,
   estimated gaps and counts above 50% for playback, project switching and its phases.
   Check the overhead line. A zero exit status alone is not a maximum-time pass.
3. Run `task check` and `task build`. No sound device or controller is needed.
4. Keyboard and controller behavior stay unchanged. No new manual input or listening
   pass is required for this report-only change. Existing `renders/84-input-parity.wav`
   remains available; its samples must stay unchanged.

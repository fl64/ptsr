## 0. Specifications

- [x] 0.1 Record the owner-approved report, unchanged target and exit policy, branch
  exception, dev-dependency license and diagnostic evidence. Validate strictly,
  run `task check` and commit these artifacts before code.

## 1. Benchmark report

- [x] 1.1 Add tests for all seven delta scenarios, including synthetic calculation,
  complete formatted output, threshold boundaries, clock errors and overhead. Record
  the failing baseline. Add the approved libc dev-dependency and benchmark-only helper;
  wrap steady and switch callbacks without changing workload, phases or wall p99 gates.
  Keep first blocks and every measured sample. Verify real CPU clocks and full offline
  output, run `task check`, inspect runtime-dependency and callback diffs, verify the
  unchanged parity render hash and commit this step with its task record.

Implementation check (2026-11-03): all seven delta scenarios have named tests in
`crates/engine/tests/benchmark_report.rs`; all 9 tests pass. The initial red run failed
because the support module did not exist (`/tmp/ptsr-84a-report-red.log`). The raw overhead
output test then failed on its missing CPU-time line (`/tmp/ptsr-84a-overhead-red.log`)
before the report was added. Both now pass without skips. Full `task check` passed
(`/tmp/ptsr-84a-implementation-check.log`). The full offline report has all six groups
and all three series: 2,000 empty calls, 20,000 steady blocks, 22,000 switch callbacks,
and phase counts 200 / 200 / 21,600. Workload setup is byte-identical to `cabf06f`.
The engine runtime tree remains hound and rtrb; libc is dev-only. Engine, App, input
and TUI sources are unchanged. The parity WAV SHA-256 remains
`f795d105ad3f098d03857203e0eaffed73e782dd1d51bcfdfec233f7d35a4da6`.
These runs do not close earlier CPU spikes or physical-controller checks.

## 2. Owner documentation

- [ ] 2.1 Explain the clocks, estimated gap, counts and unchanged gate in README and
  live checks. Update project/plan/OpenSpec context for active 84a, not active 85.
  Validate, run `task check`, build the release and commit the docs. No new render,
  keyboard/controller pass or listening test is needed for unchanged sound/input.
- [ ] 2.2 Ask the owner to read and accept the new offline report. Record actual results.
  Do not claim the slow-core performance issue or physical-controller checks are closed.
- [ ] 2.3 After explicit acceptance, archive with `openspec archive`, check all seven
  scenarios in the main specs, validate, run `task check` and commit separately.

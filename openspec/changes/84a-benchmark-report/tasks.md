## 0. Specifications

- [x] 0.1 Record the owner-approved report, unchanged target and exit policy, branch
  exception, dev-dependency license and diagnostic evidence. Validate strictly,
  run `task check` and commit these artifacts before code.

## 1. Benchmark report

- [ ] 1.1 Add tests for all seven delta scenarios, including synthetic calculation,
  complete formatted output, threshold boundaries, clock errors and overhead. Record
  the failing baseline. Add the approved libc dev-dependency and benchmark-only helper;
  wrap steady and switch callbacks without changing workload, phases or wall p99 gates.
  Keep first blocks and every measured sample. Verify real CPU clocks and full offline
  output, run `task check`, inspect runtime-dependency and callback diffs, verify the
  unchanged parity render hash and commit this step with its task record.

## 2. Owner documentation

- [ ] 2.1 Explain the clocks, estimated gap, counts and unchanged gate in README and
  live checks. Update project/plan/OpenSpec context for active 84a, not active 85.
  Validate, run `task check`, build the release and commit the docs. No new render,
  keyboard/controller pass or listening test is needed for unchanged sound/input.
- [ ] 2.2 Ask the owner to read and accept the new offline report. Record actual results.
  Do not claim the slow-core performance issue or physical-controller checks are closed.
- [ ] 2.3 After explicit acceptance, archive with `openspec archive`, check all seven
  scenarios in the main specs, validate, run `task check` and commit separately.

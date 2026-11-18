## 1. Specification

- [x] Record the owner decision, the start flag and the watch design.

## 2. Diagnostics

- [x] Record the callback start and stage in `Stats`.
- [x] Add the diagnostic flag, the watch thread, the input lines and the SIGXCPU line.
- [x] Test the stall report, the input lines and the normal start.
- [x] Run the callback benchmark.

## 3. Checks

- [x] Run format, Clippy, changed tests and the fast pre-push checks.
- [ ] Give the owner the start command and what to read in the log.

The changed run passed 1585 tests. The callback benchmark stays within the 50 % target:
32 voices p99 7.19 %, maximum 14.98 %. The SIGXCPU line needs a real-time thread
and has no automatic test.

## 1. Specification

- [x] Record the evidence from change 128 and the flush design.

## 2. Flush

- [x] Flush subnormal numbers at the start of each callback.
- [x] Test the exact silence after the tails and the flush in the callback thread.
- [x] Add the decaying-tail workload to the benchmark.
- [x] Render `renders/129-reverb-tails.wav` with the automatic checks.

## 3. Checks

- [x] Run format, Clippy, changed tests, the benchmark and the fast pre-push checks.
- [ ] Give the owner the build and the check.

The changed run passed 1582 tests. Without flushing, the exact silence test failed and
took 7.6 s instead of 0.37 s. The benchmark tails mean is 0.30 % with flushing and was
10.75 % without it. The 32-voice steady case gave p99 7.03 % and maximum 38.25 %.

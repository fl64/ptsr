# Tasks — 13

- [x] 1. `Stats` output peaks: `out_l`/`out_r` atomics, `record_output` from the audio callback after mixing and from the offline render, `take_output` resets; unit tests on the atomics and an offline check that the reported peak equals the rendered WAV peak; commit.
- [x] 2. App state: per-pad envelope (256 buckets) computed when a loaded sample lands on a pad; meter state reading `take_output` each tick with dB bars, ~40 dB/s decay, ~1.5 s peak hold, clip latch cleared by the next clean hit; unit tests on recorded sequences; commit.
- [x] 3. TUI: the 3-row strip above the grid (slot = pad column, empty = middle line) and the L/R meters at the top of the info column (dB number, bar, peak mark, overload light); TestBackend tests "What each pad holds", "Empty pads are lines", "Bank switch", "Meters follow the sound", "Overload latches"; commit.

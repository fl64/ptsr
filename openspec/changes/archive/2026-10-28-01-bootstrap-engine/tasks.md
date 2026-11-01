## 1. Skeleton
- [x] 1.1 Workspace `engine`, `input`, `tui`, `app`; build and `cargo test` pass
- [x] 1.2 Logs to a file, no logs from the audio thread

## 2. Sound
- [x] 2.1 Output through JACK, fallback `cpal`; choice in the config
- [x] 2.2 Command queue to the audio thread (`rtrb`), atomics for counters
- [x] 2.3 Built-in test sound (click), triggered by Space

## 3. Offline render and checks
- [x] 3.1 Render of a scenario to WAV
- [x] 3.2 Automatic checks: NaN/infinity, clipping, silence, timing ±1 sample
- [x] 3.3 `renders/01-click-pattern.wav`
- [x] 3.4 Audio callback benchmark

## 4. Terminal
- [x] 4.1 UI skeleton, status line (latency, audio thread CPU, xruns)
- [x] 4.2 kitty keyboard protocol; a warning if the terminal does not support it

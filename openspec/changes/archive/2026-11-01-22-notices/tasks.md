# Tasks — 22

- [x] 1. Startup notices clear on the first key event; the status keeps a "no releases" marker on terminals without the kitty protocol; recorded-event tests for the dismissal and the marker; commit.
- [x] 2. `Ctrl+S` shows the "Session saved" flash for 5 s; the autosave does not flash; tests for the flash and the silent autosave; commit.
- [x] 3. Live check by the agent in tmux (2026-10-31, no kitty protocol): the warning shows after the 5 s MIDI flash, one `Esc` clears it, the hint "Hit 1-4 Q-R A-F Z-V…" comes back, the status keeps "no releases"; `Ctrl+S` says "Session saved", still there at 4.5 s, gone at 6 s; nothing changed.
- [ ] 4. The owner confirms the feel; commit if anything changes.

# Tasks — 17

- [x] 1. MIDI knobs: `chop.zoom`, `chop.cursor_pos`, `chop.region_start_pos`, `chop.region_end_pos` with soft takeover, listed in MIDI learn under CHOP; X-Session k12 = `chop.zoom`; recorded-event tests for every scenario, the cursor buttons included; commit.
- [x] 2. Normalize: `Ctrl+N` on the chop screen sets the volume of the pad it was opened from to a −1 dB peak, rounded down, up to +6 dB; the status shows the gain and the limit; twice-in-a-row is stable; no pad — a message; tests for every scenario; an offline render of the normalized pad into `renders/17-normalize.wav` with the automated checks (no clipping); commit.

# Tasks — 09

- [x] 1. Cursor in `Chop` (frame position, playback writes it, arrows move by 1/32 of the view, snap to peaks, Ctrl skips, slice under cursor selected, `!` drawn, time in the slice line); M cuts on pause; Space plays from the cursor; logic tests; commit.
- [x] 2. Zoom: Ctrl+Minus/Ctrl+Equal around the cursor (plain -/= stay with slice params) with limits, the view follows the cursor; wave, numbers and dim draw from the view; logic tests; commit.
- [x] 3. Region: `I`/`O` set the edges, `Ctrl+I` clears, equal parts and playback respect the region, dim outside; logic tests; commit.
- [ ] 4. MIDI actions: `chop.zoom`, `chop.region_start`, `chop.region_end`, `chop.cursor` knobs and `chop.cursor_left`/`chop.cursor_right` buttons with soft takeover; keymap for the chop screen: ← → move the cursor; recorded-event tests; commit.
- [ ] 5. Normalize: `Ctrl+N` sets the pad volume to −1 dB peak, status shows the gain, twice-in-a-row is stable, no pad — a message; render of a normalized pad into `renders/`; commit.

Tasks 4 and 5 moved to change 17 (chop knobs and normalize).

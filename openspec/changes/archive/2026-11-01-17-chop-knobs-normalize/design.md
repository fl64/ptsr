# Design — 17

## Knobs

`chop.zoom` is a knob action (`Param::Zoom`): the knob position maps to the visible span between the region and the whole file on a log scale, with soft takeover, centered on the cursor. `chop.cursor` is a knob too: the position maps to a frame inside the region. `chop.cursor_left` / `chop.cursor_right` are button actions that move the cursor by one step, so a controller walks it like the arrows.

`chop.region_start_pos` / `chop.region_end_pos` (`Param::RegionStart` / `Param::RegionEnd`) map 0..1 of the file to the edge, clamped to keep the region at least 10 ms. The names carry `_pos`: `chop.region_start` / `chop.region_end` are the buttons that set an edge at the cursor (`,` `.`), and one name cannot be a button and a knob. The cursor knob is `chop.cursor_pos` for the same reason. The cursor buttons exist since 21; this change only tests them from a controller.

## Normalize

`Ctrl+N`: the peak of what the pad plays (its slice or the whole file), `gain_db = floor(−1 dBFS − peak_db)`, written into the volume of the pad the chop screen was opened from, clamped to the pad volume range (`pads::MAX_DB`, +6 dB); the status shows the gain and says when the limit stopped it. (Revised 2026-10-30: the first draft clamped at +12 dB, above the pad range.) When the chop was opened from the files screen (no pad), the status says there is no pad to set the volume on.

## Open questions

- The keyboard region (`,` `.` since 21) is at least one frame long, the knobs keep ~10 ms. Default: keep both; the knob is coarse and a one-frame region from a knob is useless.

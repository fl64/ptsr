# Design — 09

## The cursor

`Chop` grows a `cursor: usize` (a frame inside the range). The playing position writes into it while the sample plays, so there is only one position to think about. Arrows move it by `view_span / 32` frames (at least one frame); the slice under it becomes `selected`. Playback starts at the cursor; a pause leaves it. `cut(cursor)` already exists — M now feeds it on pause too.

Snap: when moving without Ctrl, look for the highest wave peak in a small window (±20 ms or ±2 steps, whichever is wider) around the new position and land on it. Cheap: the envelope is already in memory; no aubio until 04d.

Drawing: the `!` sits in the numbers row above the wave at the cursor column; the slice line gains `cursor 1.234 s`. When playing, the moving cursor is the same `!` (the old playback line goes away).

## Zoom and view

`view` already exists but never changes: `-` halves the visible span around the cursor (clamped at the whole file), `=` doubles it (clamped at the region when set, otherwise the file). Bounds are kept so the cursor stays inside the view; when an arrow walks the cursor past an edge, the view shifts just enough. The wave, the numbers row and the dim outside the region all draw from `view`/`range` as they already do.

MIDI: `chop.zoom` is a knob action (`Param::Zoom`): the knob position maps to the visible span between "region" and "whole file" on a log scale, with soft takeover, centered on the cursor. `chop.cursor` is a knob too: the position maps to a frame inside the region. `chop.cursor_left` / `chop.cursor_right` are button actions that move the cursor by one step, so a controller can walk it like the arrows.

## Region

`I` writes `range.start = cursor` (kept before `range.end` minus the minimal slice), `O` the end; `Ctrl+I` resets both to the whole file. Region edges live in the existing `range`, so equal parts, playback bounds and dimming already respect it; Tab now counts parts over the region (it already does over `range`). The knobs `chop.region_start` / `chop.region_end` (`Param::RegionStart` / `Param::RegionEnd`) map 0..1 of the file to the edge, clamped to keep the region at least ~10 ms.

## Normalize

`Ctrl+N`: peak of the loaded sample (already computed for the wave), `gain_db = −1 dBFS − peak_db`, written into the pad volume of the pad this chop belongs to (clamped to +12 dB); status shows the gain. When the chop was opened from the files screen (no pad), the status says there is no pad to set the volume on. The file on disk never changes.

## Keys

Free on the chop screen and away from the pad block: `-` `=` zoom, `I` `O` region, `Ctrl+I` clear, `Ctrl+N` normalize. `←` `→` change meaning on this screen: they move the cursor (they were marker.prev/next = slice select; the slice now follows the cursor). `marker.prev` / `marker.next` stay as actions for MIDI learn. `keymap.toml` default for the chop screen changes accordingly — a behavior change covered by this spec.

## Open questions

None carried: Space from the cursor, zoom knobs (buttons + `chop.zoom`), normalize via pad volume at −1 dB — approved by the owner.

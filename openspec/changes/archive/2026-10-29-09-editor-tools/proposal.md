# 09 — Editor tools: cursor, zoom, region, normalize

## Why

Chopping is blind today: the only cursor is the playing position, the screen always shows the whole file, and the part being cut can only be trimmed through the first slice. The owner wants to see where he is, work close up, mark what to cut, and bring quiet breaks up to level.

## What Changes

- **One editing cursor.** The playback position doubles as the editing cursor: on pause it stays and moves. ← → move it by 1/32 of the visible window; the slice under it becomes the selected one. Space plays from the cursor. M cuts under the cursor — also on pause. The cursor is drawn as `!` above the wave, its position shows in the slice line.
- **Cursor snaps to peaks.** Near a transient the cursor lands on the peak of the wave; Ctrl held moves without snapping.
- **Zoom.** `-` zooms out, `=` zooms in, both around the cursor; zoom out stops at the whole file, zoom in at the region. The view follows the cursor when it walks off-screen. A learned knob `chop.zoom` sets the zoom level with soft takeover.
- **Region.** `I` puts the region start under the cursor, `O` the region end, `Ctrl+I` clears back to the whole file. Tab cuts equal parts inside the region, Space plays to the region end, the wave outside the region stays dim. Learned knobs `chop.region_start` and `chop.region_end` move the edges with soft takeover.
- **Normalize.** `Ctrl+N` on the chop screen raises the pad volume so the loudest peak of the sample sits at −1 dB; the file on disk stays untouched and the status shows the gain (for example "+6.2 dB").

## Capabilities

Modified: `chopping` (cursor, zoom, region, normalize, play from the cursor), `midi-learn` (new chop actions: the `chop.zoom`, `chop.region_start`, `chop.region_end`, `chop.cursor` knobs and the `chop.cursor_left`, `chop.cursor_right` buttons).

## Out of scope

- Per-slice volume, fades and pitch (04b); auto-chop by hits and the tempo grid (04d).
- Normalizing a single slice; destructive edits of the file.
- Keyboard for the MIDI-only actions: they are learned on a controller.

## How to verify

By hand, on a break:

1. F3, press → : the `!` cursor moves from the start, the slice under it is selected, the slice line names the cursor time. M on pause adds a cut there.
2. Space plays from the cursor; press it again — the cursor stays where the music stopped.
3. Hold Ctrl and press → : the cursor walks in small steps without snapping; without Ctrl it lands on the nearest peak.
4. `-` a few times: the wave zooms out to the whole file; `=` zooms back in around the cursor; walking the cursor past the edge scrolls the view.
5. `I`, move, `O`: the region is set, Tab cuts 8 parts inside it, Space plays the region. `Ctrl+I` clears.
6. Learn `chop.zoom` and the two region knobs on the X-Session; turn them — zoom and edges follow, soft takeover holds.
7. `Ctrl+N` on a quiet break: the status says the gain, the pad sounds level; the file on disk is unchanged.

Listen for: no clicks from cuts made on pause; the gate of a playing pad is not affected.

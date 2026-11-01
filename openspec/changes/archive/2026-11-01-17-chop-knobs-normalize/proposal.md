# 17 — Chop knobs and normalize

## Why

Change 09 promised two things that were not built before it was archived: controller knobs for the chop cursor, zoom and region, and the normalize command. This change carries them on, so they stay in the plan.

## What changes

- MIDI knobs for the chop screen: `chop.zoom`, `chop.cursor_pos`, `chop.region_start_pos`, `chop.region_end_pos`, with soft takeover; the X-Session Pro gets the zoom on k12. The `chop.cursor_left` / `chop.cursor_right` buttons exist since 21 and are tested from a controller.
- `Ctrl+N` on the chop screen sets the volume of the pad it was opened from so the loudest peak sits at −1 dB, up to the pad volume limit of +6 dB. The file on disk does not change.

## Out of scope

Keyboard keys for the cursor, zoom and region: done in 09.

## How to verify

1. MIDI learn: put a knob on `chop.zoom`, turn it: the view goes from the whole file to a close-up smoothly; grab it from another position: the view does not jump.
2. Knobs on `chop.region_start` / `chop.region_end`: the dimmed edges follow, the value pops up, the region does not get shorter than ~10 ms.
3. A quiet break on a pad, F3, `Ctrl+N`: the status shows the gain (+5 dB for peaks at −6 dB); `Ctrl+N` again: the gain does not climb.
4. F3 from the files screen, `Ctrl+N`: the status says there is no pad.
5. Listen to `renders/` for the normalized pad.

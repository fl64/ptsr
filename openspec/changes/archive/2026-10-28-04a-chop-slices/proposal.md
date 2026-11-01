# 04a — Chop screen and slices on pads

## Why

The main feature of stage 1: cut a sample (usually a break) into slices and lay them out on pads, fast, with buttons only, by ear. This change gives the shortest working path: open the sample, cut it, fix the markers, put the slices on pads, play.

## What changes

- F3 opens the "Chop" screen for the sample of the selected pad. The waveform is drawn with Braille characters.
- Auto-chop into equal parts: Tab cycles 2, 4, 8, 16, 32.
- Chopping in time: Space plays the sample with a moving cursor, M sets a marker at the cursor.
- Slice editing: ← → select a slice and play it; its start moves coarsely or finely, the cut at its start can be deleted; Ctrl+Z undo.
- Knobs k1 and k2 (any knob, through MIDI learn) move the start and the end of the selected slice; on the first and the last slice they trim the sample itself.
- Two modes on the chop screen:
  - Listen: pad key k plays slice k, whatever lies on the pads.
  - Assign (Enter): the grid shows the real pads of the bank with what lies on them, PgUp/PgDn change banks, the plan shows which slice goes where and which pads it replaces. Tab switches between "in order" (all slices in a row from a pad you hit) and "by hand" (each pad hit takes the selected slice). Enter applies the plan, Esc goes back to listening.
- A pad with a slice is named after it: "break 5".
- A pad with a slice plays only its part of the sample. The source file does not change, the sample is kept in memory once for all its slices.
- Slices are saved in the session. Sessions from 03 open as before.
- All chop actions appear in MIDI learn (F12 → MIDI).

## Out of scope

Moved to 04b: auto-chop by hits, snapping to hits, tempo grid, slice parameters (attack, fade, pitch, volume, reverse), the large slice view, kitty graphics, the X-Session Pro chop profile. Later: saving slices to new files, resampling, time-stretch.

## How to verify

1. Put a break on pad 1, select pad 1, press F3.
2. Press Tab until it says 8: eight equal slices. Press pad keys `1 2 3 4 Q W E R`: each plays its slice, without a click at the end.
3. Press Space: the sample plays, the cursor moves. Press M on a hit: a new marker appears.
4. Select a slice with ← →: it plays. Move its start with `,` `.`, then with Ctrl held (smaller steps). Backspace deletes the cut, Ctrl+Z brings it back.
5. Select slice 1 and turn k1 on the X-Session: the start of the sample moves; turn k2 on the last slice: its end moves.
6. Press Enter: the pads of the bank show what lies on them and where the slices will go. Hit pad 13: the chain of slices starts there and goes on into bank 2; replaced pads are pink. Enter: done, the status says "slices placed on banks 1–2".
7. F3 again, Enter, Tab ("by hand"), Ctrl+3: select slice 1 and hit pad 1, then slice 3 and pad 2. Enter: only these two land in bank 3, named "break 1" and "break 3".
8. Quit and start again: the pads still play their slices under the same names.
9. Listen to `renders/04-chopped-beat.wav`: a beat from slices of one break, no clicks.

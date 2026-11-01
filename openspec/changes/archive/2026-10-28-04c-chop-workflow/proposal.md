# 04c — Chop workflow

## Why

After 04a a chop is one-off: once the slices are on pads, the cuts are gone. Fixing a late slice means chopping the whole break again and laying it out again. Getting to the chop screen also takes a detour: a file has to be put on some pad first, overwriting it. And cutting by ear always plays the sample from its very start.

## What changes

- The cuts of a file are remembered and the pads with its slices stay linked to them. F3 on any of these pads opens the whole file with all its cuts, that slice selected. Moving a cut changes the pads with the neighbouring slices at once, without laying them out again.
- F3 on the files screen (F6) opens the selected file on the chop screen directly; no pad is used as a draft. Where the slices go is decided in the assign mode.
- Space plays from the start of the selected slice to the end of the sample.
- In the listen mode PgUp/PgDn (and the bank buttons of a controller) turn pages of slices: page 1 plays slices 1–16 on the pads, page 2 slices 17–32.
- The cuts are saved in the session; sessions from 04a open with their slices.

## Out of scope

Hit detection, slice parameters, trimming one slice alone (04b). Several chops of one file.

## How to verify

1. F6, select a break, F3: the chop screen opens with this file; no pad changed.
2. Tab to 8, Enter, Enter: slices on pads 1–8.
3. Hit pad 3, F3: the whole break opens, slice 3 selected. Move its start 20 ms right, Esc.
4. Hit pads 2 and 3: slice 2 got longer, slice 3 starts later; no clicks.
5. On the chop screen select slice 5 and press Space: playback starts at slice 5.
6. Tab to 32, PgDn: the pads say 17–32 and play them; PgUp: back to 1–16.
7. Quit and start again: F3 on pad 3 shows the same cuts.

# Design 04a

Screen sketch: `docs/ui/chop.md` (the large slice view and the parameter pop-up are 04b).

## Non-destructive slices

A slice is a region `start..end` (in frames) of a sample that is already loaded. A pad holds a shared `Arc<Sample>` plus its region; nine slices of one break share one buffer. The engine gets the region together with the sample, so a voice reads only `start..end` and applies the usual 5 ms fade when it is cut or reaches the end of a slice that does not end in silence.

When a pad gets a new region while its voice still plays, the voice keeps the old `Arc` and region until it ends or is cut with a fade; the next hit uses the new region. The audio thread still never frees memory: old samples go to the garbage queue.

## Chop screen state

The UI thread owns the chop state: source pad, sample, marker list (sorted frame positions), selected marker, undo stack (snapshots of the marker list, bounded), playback cursor. Markers split the sample into slices; the sample start and end are implicit edges.

- Equal parts: Tab replaces the markers with N−1 equal cuts, N cycles 2, 4, 8, 16, 32. It is one undo step.
- Cut: M inserts a marker at the cursor. A marker closer than 10 ms to another one is not added.
- Selection is a slice, not a marker: ← → select the previous or next slice and play it. The cuts are the boundaries between slices; boundary 0 is the start of the chopped range, the last one its end.
- Nudge: `,` `.` move the start of the selected slice by 20 ms, by 4 ms while Ctrl is held. On the first slice this moves the start of the range (trims the sample). A boundary stops 1 ms short of its neighbours and of the edges of what was opened (the file, or the pad's slice); the screen always shows all of what was opened, the trimmed parts dim.
- Delete: Backspace removes the cut at the start of the selected slice, so it joins the previous one. Nothing happens on the first slice.
- Undo keeps the cuts and the range; a knob turn on one boundary is one undo step, however long it turns.
- Playback uses the preview slot of the engine; the cursor position comes back from the audio thread through an atomic.
- Pad keys on this screen play slices through the preview slot as well; the pads are not changed until the plan is applied. In listen mode pad key k plays slice k, so slices 17–32 are heard only with Space or by selecting them. Controller pad hits go to the UI on this screen instead of straight to the audio thread, the same way as with a file in hand.
- If the selected pad already holds a slice, F3 chops that slice, not the whole file.
- Leaving the screen keeps the markers; F3 on the same pad with the same sample brings them back. Enter clears them.

## Knobs: start and end of the selected slice

Two knob actions, `slice.start` and `slice.end` (k1 and k2 in the X-Session profile), set the position of the start or the end boundary of the selected slice inside its zone: from the boundary before it to the boundary after it (the edges of what was opened for the range ends), 1 ms away from each. The knob range spans the zone, so the step is the zone length / 127 (about 5 ms for a 330 ms slice); fine work stays with Ctrl+`,` `.`. Absolute knobs use soft takeover: after another slice is selected the boundary does not jump until the knob reaches its position, and the status hints which way to turn. A cut is shared by two slices, so moving the start of slice 3 also moves the end of slice 2; trimming one slice alone is 04b.

## Assign mode

Enter on the chop screen opens the assign mode; Esc goes back to listening, Enter applies the plan.

- The grid shows the real pads of the current bank: the name of what lies on each, empty pads dim. PgUp/PgDn and Ctrl+1…8 change the bank. The bank line marks banks that have something on them and banks the plan writes to.
- A planned pad shows the slice number (`→ 3`); a planned pad that holds something now is pink: it will be replaced. A summary line tells how many slices go to which banks, how many pads are replaced and how many slices are left out.
- Tab switches the way of planning:
  - In order (the default): slice 1 goes to the selected pad and the rest follow, across banks; slices after bank 8 pad 16 are left out. A pad hit moves the start of the chain to that pad, in any bank.
  - By hand: the plan starts empty. ← → select a slice (it plays), a pad hit puts the selected slice on that pad (replacing what the plan had there) and selects the next slice. Slices not put anywhere are not placed.
- A pad hit in the assign mode also plays the slice that goes to that pad.
- Ctrl+Z undoes the last plan step.
- What lies on the pads is read when the plan is applied, so a file put on a pad meanwhile is counted.

## Applying the plan

Each planned pad gets the source file path and region and keeps its mode and volume; the old pad content is released as usual. A load still on its way to a target pad is dropped. The screen then switches to the pads, so the slices can be played at once. The status says where the slices went ("slices placed on banks 1–2") and how many were left out.

## Slice names

A pad with a slice is shown as the file name and the slice number ("break 5"). The number is saved in the session (`slice`, optional, so no new session version).

## Session format v2

`version = 2`. A pad entry gets optional `start` and `end` in seconds of the file, so a slice survives a change of the audio rate. A v1 file has no slices and loads as whole-file pads; it is written back as v2 on the next save. A file with a version newer than 2 is refused, as before.

At start, pads that share a file wait for the first of them to load it: the file is read once and all its slices share one sample in memory. If that first pad gets another file before the load finishes, the next waiting pad loads the old file instead, so no pad stays "loading". The command queues hold 1024 commands, so a restore or a layout over all 128 pads is not cut short.

## Keys

From `config/keymap.toml` `[screens.chop]`. In 04a: Space, M, ← → (slice), `, .`, Ctrl+`,` Ctrl+`.` (fine), Backspace, Tab (equal parts; in the assign mode: in order / by hand), Ctrl+Z, Enter (assign mode, then apply), Esc, PgUp/PgDn and Ctrl+1…8 (bank in the assign mode). Knobs: `slice.start`, `slice.end`. A terminal sends Ctrl only together with another key, so the fine step is its own pair of actions (`marker.nudge_left_fine`, `marker.nudge_right_fine`); without the kitty keyboard protocol Ctrl+`,` does not reach the program and only the coarse step works. The slice parameter keys (↑ ↓ − =) and N (snap) do nothing until 04b.

## Risks

- Region changes while a voice plays (covered above, tested with an offline render).
- Session migration (tested on a saved v1 file).
- Loader: one decode per file even when nine pads reference it.

## Decisions from the owner (2026-10-28)

- Two modes: listening is not tied to the pads; assigning shows the real pads, banks can be changed, slices go in order by default or one by one by hand.
- Knobs move the shared cut (start and end of the selected slice); trimming a slice alone is 04b.
- A pad hit in the assign mode plays the slice that goes there.

## Open questions

None now.

# Design 04c

Builds on 04a (to be archived first): chop screen, listen and assign modes, slices as regions of a shared sample.

## A chop belongs to a file

One chop per file path: the range (start, end) and the cuts, in seconds. Every cut has a lasting id (the range start is id 0), and the chop has an epoch. A linked pad keeps the id of the cut at its slice start and the epoch; its slice number and region are worked out from the chop. When the cuts change, every linked pad of that file gets its new region (`SetPadRegion`); a playing voice keeps its old region, as in 04a.

- A cut that was deleted is remembered with its last position: its pads play the slice that now covers that place (the joined slice). Undo brings the cut back and the pads follow it again.
- Equal parts start a new epoch; pads of an older epoch are free (they keep their own start and end). Undo brings the old epoch back, so these pads are linked again. A new epoch number is never reused after an undo.
- The chops of all files stay in memory while the program runs; the one on the screen is the open one. A chop from the session waits until its file is loaded.

## Edits and linked pads

- Moving a cut or a range edge, a knob turn: regions of the linked pads follow.
- A new cut inside slice k: slices after k are renumbered +1; a pad with slice k keeps the first half.
- Joining slice k with k−1 (Backspace): a pad with slice k now plays the joined slice k−1; slices after k are renumbered −1.
- Undo restores the cuts; the linked pads follow again.
- Equal parts (Tab) replace all cuts at once, without asking: the linked pads of this file are unlinked and keep playing what they played (a free slice: file + start + end, as in 04a); the status says how many pads were unlinked. Ctrl+Z brings back the old cuts and links these pads again, so the step is safe without a confirmation.

## Opening

- F3 on a linked pad: the whole file opens with its chop, that slice selected.
- F3 on a pad with a free slice (from 04a or after unlinking): the whole file opens; if the file has no chop yet, the cuts are taken from the pad's slice edges.
- F3 on a pad with a whole file: the chop of the file, or a new one.
- F3 on the files screen: the selected file is loaded (as the preview is) and opens when ready; nothing changes on the pads until a plan is applied. The assign mode starts its chain from the selected pad.

## Playback

Space plays from the start of the selected slice to the end of the range.

## Pages of slices

In the listen mode the bank actions (PgUp/PgDn, Ctrl+1…8, controller bank buttons) turn pages of slices instead of banks: page p plays slices 16p+1…16p+16 on the pads, and the grid is labelled with these numbers. The page starts at the one with the selected slice when the screen opens. Selecting a slice with ← → turns to its page. In the assign mode the same actions change banks, as in 04a. A pad hit in the listen mode selects the slice it plays (owner request), so knobs and keys edit the slice just heard.

## Assign slice by slice (owner request)

Enter opens the assign mode by hand, not in order: the plan is empty and slice 1 is selected. A pad hit puts the selected slice on that pad and selects the next slice not planned yet (after the current one, else the first one left). When every slice is planned, a pad hit only plays what is planned there and the status says "all N slices placed: Enter applies". Ctrl+Z takes back the last step and selects the slice it planned.

## All in order (owner request)

Tab in the assign mode switches between "by hand" (the default) and "all in order"; Tab again goes back.

- All in order starts with no plan: the header says "IN ORDER · hit the pad to start from". The owner turns to a bank (PgUp/PgDn, Ctrl+1…8) and hits the start pad.
- From then on the grid shows the chain: slice 1 on the start pad, the rest after it, on into the next banks; slices after bank 8 pad 16 are left out. Another hit moves the start.
- The side panel lists what the chain would overwrite: "overwrites: bank 2 Q kick, W amen 3".

## Asking before overwriting (owner request)

A step that loses what lies on a pad and has no undo asks first. Today there are two: applying a plan over taken pads, and putting a file from the files screen on a taken pad (an empty pad, i.e. one with the test tone, does not ask). Cuts and plan steps are undone with Ctrl+Z and do not ask.

The question is a box over the screen, in plain words:

```
Replace what lies on 2 pads?
  bank 2  Q  kick     → slice 1
  bank 2  W  amen 3   → slice 2
The old sounds leave these pads; their files on disk stay.
This cannot be undone.
Enter: replace · Esc: keep
```

Enter (the key that applied the step, or the controller button of that action) confirms; Esc declines and changes nothing: the plan stays, a file stays in hand. While the question is open other keys do nothing. What lies on the pads is read again on Enter; if it changed, the question is shown again with the new list.

## Session v3

`[chops."<path>"]` with `start`, `end`, `cuts` (seconds). A pad entry keeps `file`, `slice` and, for a free slice, `start`/`end`. A v2 file: pads with `slice` and `start`/`end` of the same file are turned into a chop of that file built from their edges and linked to it; pads without `slice` stay free. A v1 file opens as before.

## Decisions from the owner (2026-10-28)

- Pages of slices in the listen mode through the bank actions.
- A pad hit in the listen mode selects its slice for editing.
- The assign mode starts by hand: slice 1 selected, each tap places it and moves on to the next.
- "All in order": Tab, then hit the start pad (any bank); the chain and what it overwrites are shown before Enter.
- Every step that loses what lies on a pad without undo asks first, in plain words: applying a plan over taken pads, a file from F6 on a taken pad.
- Tab over a chop with linked pads: "do it the logical way" — no confirmation; the pads unlink and keep their sound, undo relinks them.

## Open questions

None now.

# 27 — Every pad says what it plays: the whole file, a slice, or its own bounds

## Why

The owner (2026-10-30): "a sample sits on a pad, its slices on other pads; whichever pad I
pick, F3 drops me into the source sample with all its cuts. If I pick a fragment I expect
to edit that fragment. And why does the whole-file pad show cuts that have nothing to do
with it? It confuses me." The model is right — the cuts belong to the file and the pads
follow them (the linked school) — but it is hidden. The owner chose the linked school and
asked to make it as clear as on an Akai MPC, where every pad layer has a Slice parameter:
All, a slice number, or Pad (its own start and end), and the chop mode edits the slices of
the sample (MPC 1.8 release notes and the MPC forums; not checked on hardware).

## What changes

- **The pad source**: a pad with a sample plays `all` (the whole file), `slice N` (slice N
  of the file's cuts, follows them) or `pad` (its own start and end, follows nothing). The
  pad panel names it: `break.wav · slice 3`.
- **A SOURCE page in the pad edit**: source, start, end on knobs 1–3, after TONE and ENV.
  The source steps all → slice 1…N → pad. Moving the start or the end of a `slice` or
  `all` pad turns it into a `pad` pad at once, the status says so, and `Ctrl+Z` links it
  back. This is "I picked a fragment, I edit only it".
- **The chop screen names the owner of the cuts**: the header says "cuts of break.wav",
  every slice shows the pads that play it, and the slice of the pad F3 came from is
  selected. A pad that plays `all` or `pad` is not under any slice.
- The assign (`P`) keeps laying slices out as `slice N` pads (owner: no extra choice in
  the assign).

## Out of scope

A choice between linked and copied slices in the assign; showing a `pad` pad's piece over
the chop wave; slice parameters (04b).

## How to verify

1. Chop a break, lay slices 1–8 on pads 5–12. The pad panel of pad 7 says `slice 3`;
   pad 1 with the whole break says `all`.
2. F3 on pad 7: the header says "cuts of break.wav", slice 3 is selected, pad 7 stands
   under it and the other pads under their slices.
3. `Enter` on pad 7, walk to SOURCE, move the start: pad 7 says `pad`, the status says it
   no longer follows the cuts, pads 6 and 8 did not change. `Ctrl+Z`: `slice 3` again.
4. On SOURCE step the source of pad 1 to `slice 2`: it plays slice 2.

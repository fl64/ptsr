# 68 — State you can see

## Why

A second UX check of the owner's agent (2026-11-01) found five places where the program
changes or holds a state and does not show it:

1. The change scope (`Ctrl+A`: pad, N marked, bank, all pads) is named only in the pad edit
   panel. Outside it the owner cannot see that the next `H` or knob turn hits a whole bank.
2. `Ctrl+Y` (chop by hits) replaces hand-made cuts and says only "7 cuts at −12 dB".
3. `Ctrl+B` with a file selected marks the folder of the file, but the status reads as if
   nothing related to the file happened.
4. The chop screen header always says `LISTEN`; it does not show that `Space` plays.
5. The status marker `no releases` is jargon.

The owner: "ok, do it".

## What changes

- The help line of the pads screen shows the scope with the mark toggle:
  `mark: off · scope: bank 2`.
- `Ctrl+Y` says how many slices there are now and were before, and that `Ctrl+Z` brings
  them back.
- `Ctrl+B` on a file says it marked the folder of this file.
- The chop header reads `LISTEN ▶` while the chop screen plays.
- The status marker reads `no key releases: gate pads play one-shot`; F1 names the
  terminals that give key releases.

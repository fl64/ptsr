# 06b — Small UX fixes after a walkthrough

## Why

The agent went through every user path on a 140×36 terminal (cut off from audio). Everything works; in seven places the owner would stop and wonder what to do or what happened.

## What changes

- First start: when no pad holds a sample, the hint line says how to put one there.
- Empty pads are called "empty"; the test tone note is shown only in the pad info on the right.
- F1 help is grouped by purpose (pads, banks, screens, files, chop, settings), banks 1–8 are one row, descriptions fit the box.
- Swapping pads: the pad in hand is named on screen; arrows choose the other pad and Enter swaps, as with a file in hand (03c). A pad hit still swaps at once.
- Ctrl+B on a file marks the folder it is in as a favorite, and the status says so.
- MIDI learn list: PgUp/PgDn jump between groups (GLOBAL, PADS, FILES, CHOP, SETTINGS).
- The settings help line lists only the keys that work on the open tab.

## Out of scope

Taller pads on tall terminals (open question to the owner).

## How to verify

1. Start with an empty session: the hint says "Pads are empty: F6 opens files…".
2. Pads without samples say "empty"; the info on the right says "test tone A2".
3. F1: groups with headings, one row for banks 1–8.
4. Put a sample on pad W, Ctrl+X, the screen says "Pad W (…) in hand"; → Enter: W and E swap.
5. F6, select a file inside "breaks", Ctrl+B: the status says the folder became a favorite; one level up it has a ★.
6. F12, Tab to MIDI, PgDn: the selection jumps to the PADS group.
7. F12 on the Audio tab: the help line has no "learn" and "clear".

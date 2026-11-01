# 35 — The UX pass: the musician's walk, fixed

## Why

The owner (2026-10-31) asked to walk every path as a musician making a track and to fix
what is inconvenient ("fix all that you found"). The walk found: picking a given slice
for a given pad is not possible inside the assign mode; one `,` on a laid-out break moves
7 pads; after an undo the edit opens on a wrong slice; the chop hint says "start" on a
finished chop; a new project opens the files screen in an empty folder; the open project
is not marked and the picker shows the pad keys; the bank is forgotten on restart; the
pad in hand line is cut; a favorite is listed twice in its parent; `Ctrl+Z` and `Ctrl+Y`
read differently without kitty.

## What changes

- Assign mode: `J` `L` pick the previous and the next slice and play it; the cuts stay.
- A region leaves the pads outside it playing what they played (`pad`), undo links back.
- After an undo the slice under the cursor is selected.
- The chop hint follows the state: start, lay out, move bounds.
- A new project's files screen starts where the owner browsed last.
- The picker marks the open project and shows its own keys.
- The bank comes back after a restart and a project switch.
- The pad in hand line is short: `in hand: pad 3 · amen 3`.
- A favorite folder is listed once in the folder that holds it.
- Without kitty every modifier key reads as a combination.

## Not changed, on purpose

The flat line on an empty pad (29, the owner's choice); the 1 ms tap on envelope times
(21; the held key speeds up — to be felt live first); `Ctrl+P` on the help line (the last
group is F1 F3 F6 F12 by the spec, and the F3 line is full).

## How to verify

Recorded-event and drawn-screen tests per scenario; `task check` green; the owner walks
the chop, the layout, a new project and a restart.

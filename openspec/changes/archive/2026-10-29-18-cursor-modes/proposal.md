# 18 — Two cursor modes

## Why

The owner says the chop cursor answers badly to the arrows. Two causes:

- The snap "loudest frame on the way" lands right behind the cursor on a decaying hit: on a break, 12 presses of → moved the cursor by 12 frames.
- With the kitty keyboard protocol a held key comes as repeat events, and every repeat was dropped: a held arrow moves the cursor once.

## What changes

- Two cursor modes on the chop screen, switched with `N` ("snap to hits"), shown in the help line:
  - **even steps** (the default when the chop screen opens): ← → move by 1/32 of the view, no snap;
  - **hits**: ← → jump to the start of the next / previous hit, when that hit is on the screen; with no hit ahead on the screen the cursor steps evenly until a hit comes into view.
- Ctrl + ← → is a fine step (1/256 of the view) in both modes, no snap.
- A held key repeats for the cursor arrows, the cut nudge (`,` `.`) and zoom (`-` `=`). Pads and the other commands ignore the repeat, as before.

## Out of scope

Snapping of cuts made in time (M) to hits and the aubio onset detector: 04d. Until then hits come from a simple level-rise detector.

## How to verify

1. Chop screen on a break: hold →: the cursor runs in even steps; the help line says `snap: off`.
2. `N`: the help line says `snap: hits`; each → lands on the start of the next hit, ← on the previous one.
3. Zoom in on a long quiet tail with `N` on: → steps evenly until a hit appears on the screen, then jumps to it.
4. Ctrl + →: small even steps in both modes.

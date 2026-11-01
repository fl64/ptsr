# 49 — Loop the play on the chop screen

## Why

The owner (2026-10-31): "on the chop screen `O` says from where it plays; I need one more
key that makes `Space` loop the mode picked with `O`", "it plays until I press `Space`
again, that stops the loop", "it keeps playing while I move the bounds".

## What changes

- A loop switch on the chop screen, `Alt+O`: on, `Space` plays what `O` picked (to the end, to the
  cut, the slice) over and over until the next `Space`.
- The loop follows the bounds while it plays: a moved cut, slice bound or region edge is
  heard in the next turn of the loop, without a stop.
- The help line shows the switch with its state.


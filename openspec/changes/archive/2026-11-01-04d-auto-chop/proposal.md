# 04d — Auto-chop by hits

## Why

Equal parts (04a) rarely fall on the hits of a real break. The owner (2026-10-30): "I
need the auto split of a piece into samples, as samplers do". The program should find
the hits itself and cut there, and keep a cut made by ear on the hit.

## What changes

- **`Ctrl+Y` chops by hits**, the way `Y` chops into equal parts: the first press cuts
  at the loudest hits, every next press lowers the threshold so quieter hits get their
  cuts too, after the last step it starts over. Each press replaces the cuts and is one
  undo step: `Ctrl+Z` goes back a step.
- **The threshold on a knob**: the X-Session knob k11 (`chop.threshold`) adds or removes
  cuts at once, as the knob turns.
- **Snapping**: a cut made in time (`M`) moves to the nearest hit within 90 ms; `N` turns
  it off.
- **First the detector**: `aubio` or an own Rust detector, compared on the owner's breaks
  by ear before any code (task 0.2).

## Out of scope

The tempo grid (cuts on beats from the bars or a typed tempo): a later change of its own
(owner, 2026-10-30). Slice parameters, kitty graphics.

## How to verify

1. Take a break, F3, `Ctrl+Y`: cuts land on the loud hits; `Ctrl+Y` again: the ghost
   notes get their cuts; `Ctrl+Z`: back a step. Turn k11: cuts come and go at once.
2. `Space`, press `M` a little after a hit: the cut lands on the hit, the status says
   "marker snapped to hit". `N`, `M` again: the cut stays where it was pressed.
3. `renders/04d-chopped-by-hits.wav`: the break with a click on every cut.

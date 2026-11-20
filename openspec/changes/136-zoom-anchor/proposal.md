# Proposal

## Why

On 2026-11-20 the owner moved a slice bound and then zoomed. The zoom kept the cursor in
place, so the bound moved away from its place on the screen. The owner wants the zoom to
work around the point that the owner touched last.

## What Changes

- Chop keeps a zoom anchor: the cursor, the slice start, the slice end, the region start or the region end.
- A move of one of these points makes it the anchor. A cursor move, a pad hit and a slice selection make the cursor the anchor.
- Every zoom control keeps the anchor at its place on the screen: `I`, `K`, `↑`, `↓`, Ctrl/Alt+I, Ctrl/Alt+K, the SMC-PAD zoom knob and a learned zoom knob.
- After a zoom around a bound, the cursor can stay out of the view. The next cursor move brings the view back to the cursor.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chopping`: the zoom anchor and the zoom requirements that named the cursor.

## Impact

The change affects the Chop zoom only. Keys, knobs, cuts, saved files and sound stay unchanged.
No new dependency.

Owner checks, controller:

1. `CHOP` → turn knob 3 «start» → turn knob 2 «zoom» → the slice start stays at its place on the screen.
2. `CHOP` → turn knob 4 «end» → turn knob 2 → the slice end stays at its place.
3. `CHOP` → turn knob 5 «reg[» → turn knob 2 → the region start stays at its place.
4. `CHOP` → turn knob 1 «cursor» → turn knob 2 → the cursor stays at its place, as now.

Owner checks, keyboard: Ctrl+→ moves the slice start, then Ctrl+I zooms around it. `L`, then `I` zooms around the cursor.

Renders: none, because sound stays unchanged.

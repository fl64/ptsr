# Design — 04e

## Saving the cuts

The session is written when it is marked dirty; today only pad changes mark it, so a chop with no linked pads is never written. Any action on the chop screen that changes the cuts or the range marks the session dirty: the open chop is compared before and after the action (`Chop::saved()`), so every editing path (keys, knobs, undo) is covered by one check in `App::apply`.

## Unlinking on a cut inside a pad's slice

A pad follows a slice by the id of the cut at its start and the chop epoch. When `M` puts a cut inside slice k, the pads linked to slice k are unlinked: their link is cleared, their region stays as it was, the session is marked dirty. Pads linked to other slices are not touched, and moving or deleting a cut keeps today's behavior.

Undo must bring the links back. The undo entry of the cut remembers the unlinked pads with their links; undoing the cut restores them. This reuses the way equal parts unlink: the old epoch stays on the pad, and undo restores the epoch, here the per-pad link is restored instead.

## Labels and the info line

- The label of an unlinked pad is the `pad` source of 27: the panel says `<file> · pad`,
  the grid shows the file name without a slice number. (Revised 2026-10-30 after 27; the
  first draft had `<file> ·`.)
- The chop info line appends `on <pads>` for the selected slice, as bank and key like the
  labels at the slice numbers (`on 1Q, 2W`); nothing when no pad holds it.

## Open questions

1. **A cut inside a slice on a pad unlinks the pad.** Decided with the owner: the pad keeps exactly what it played; refining an already laid-out break then needs the slice placed again (`P`, slice, pad, `Enter`), or the pad's SOURCE page (27) puts it back on a slice. Default kept until the owner finds it in the way.

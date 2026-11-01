# Design 50

## Decisions (owner, 2026-10-31)

- 1 of 3 ("Q1.1"): a switch, off by default, `Alt+P`. Weighed: on by default (Q1.2), cuts
  are harder to place by ear under pitch and filter; always on (Q1.3).
- The pad of a slice laid on several pads: the first in pad order.
- The settings: volume, pitch, envelope (attack, decay; the release acts on a gate pad's
  key, the preview has none) and filter. The pad's gate mode does not apply: the preview
  plays its region out.
- The preview borrows the pad's settings without being the pad: a hit on that pad does not
  cut the preview, and a pad change while it plays is heard on it as on the pad.

## Open questions

- The loop (49) with "as pad": each turn starts the envelope again. Default: yes, a turn
  is a new hit.

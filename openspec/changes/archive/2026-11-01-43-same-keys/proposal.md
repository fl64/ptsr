# 43 — The same operation on the same keys on every screen

## Why

The owner (2026-10-31): "how logical are the key combinations between the modes; for
example the banks: on the pads they turn with the arrows, in the editor they do not;
align such operations everywhere". The inventory found five different sets of bank
keys on six screens, and on the chop screen `PageUp` `PageDown` mean the cut jump while
listening and the banks while assigning. The owner also asked to rework the help line at
the bottom: "a mess, unreadable".

## What changes

- `PageUp` `PageDown` and `Ctrl+1…8` turn the banks on every screen with pads: the pads
  screen, the pad edit, the chop screen (listening, assigning, slice edit), the files
  screen. `J` `L` (`←` `→`) on the pads screen keep turning the banks too.
- The chop cursor jump from cut to cut leaves `PageUp` `PageDown` and becomes a third
  step of the snap switch `N`: even steps, hit to hit, cut to cut.
- The help line at the bottom becomes columns in its 3 rows, one group a column, the same
  group order on every screen.
- The chop help says "remove cut" for `Backspace`.
- A held `I` `K` walks the settings rows.
- The pad edit panel shows TONE, ENV and SOURCE rows at once, no row hides on `Tab`.
- The X-Session profile does not change.

## Not in this change

Asked on the same day, each its own change: a filter on a pad (44); a learn walk over
every control on the screen to bind MIDI knobs and pads. A live wave while recording is
there already (owner: "I see it, it is ok").

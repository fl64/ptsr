# Design 28

## Groups

- `input::Group` gains `PadsEdit` and `ChopEdit` (config keys `pads_edit`, `chop_edit`;
  titles `PADS · EDIT`, `CHOP · EDIT`). Their actions: `edit.param_1`, `edit.param_2`,
  `edit.param_3` (knobs), `edit.param_prev`, `edit.param_next`, `ui.back` (buttons). An
  action name may now stand in more than one group; a learned entry is keyed by group
  and control as today.
- The learned config keeps its shape (profile → group key → control → action); the new
  group keys are additive, old configs load unchanged.
- Resolution inside an edit mode: the learned EDIT group of the screen first, then the
  profile's `[edit]` layer, then the screen and the global groups (learned over profile).
  Outside the edit the EDIT groups do nothing.
- CHOP · EDIT lists knob 3 too, though the BOUNDS page has two parameters until 04d brings
  the threshold: the layout of the knobs is the same on every screen (owner, 2026-10-30,
  default "show it").

## Folding

- The list is rows of two kinds: a group row and an action row. A closed group shows
  only its row with `· N actions · M learned`. The open set is UI state, not saved; F12
  opens with the group of the screen it came from (GLOBAL from the settings screen
  itself).
- Keys on the MIDI tab: `↑` `↓` (`I` `K`) rows, `→` (`L`) or `Enter` on a group row opens
  it, `←` (`J`) closes the group of the selected row and selects its row, `PgUp` `PgDn`
  jump group rows as today.

## Pads and banks in a row

- "pads 1–16" (actions `pad.1`…`pad.16` of GLOBAL) and "banks 1–8" (`bank.1`…`bank.8`)
  are one row each; the row shows `N of 16 learned`.
- `Enter` starts a run at the first not learned one (or 1 when all are): every control
  pressed is learned for the current one and the run moves on; a control already used in
  the run is ignored, so a double hit does not take two pads; the status says
  `pad 5 of 16: hit a control, Esc stops`. The run ends after the last one or on `Esc`;
  what is learned stays. `Backspace` on the row clears all 16 (8).

## Open questions

- A filter "learned only" was offered; not taken now.

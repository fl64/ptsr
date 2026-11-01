# Design 43

## The inventory (2026-10-31)

| | pads F2 | pad edit | chop F3 | assigning (P) | files F6 | settings F12 |
|---|---|---|---|---|---|---|
| bank | J L, ← →, PgUp PgDn, Ctrl+1…8 | PgUp PgDn, Ctrl+1…8 | Ctrl+1…8 only | PgUp PgDn, Ctrl+1…8 | PgUp PgDn, Ctrl+1…8 | MIDI tab: PgUp PgDn jump groups |
| J L | bank | value | cursor | slice | folder in / out | value |
| I K | — | parameter | zoom | deaf | list | row |
| PgUp PgDn | bank | bank | cut jump | bank | bank | MIDI groups |
| Enter | edit on | edit off | slice edit | apply | take in hand | learn |
| Tab | parameter | parameter | in order / by hand | — | — | tab |
| X-Session b1 b2 | bank | bank | play, cut | play, cut | — | — |

Deliberate by spec: `J` `L` walk what the screen shows and change the value in an edit
mode (21, "Key roles"); the project picker takes only the arrows, the letters type a
name (11).

## Decisions (owner, 2026-10-31)

- Banks: 1 of 3 ("Q1.1"). `PageUp` `PageDown` and `Ctrl+1…8` everywhere, `J` `L` on the
  pads screen stay. As samplers do it (assumption: from memory, SP-404 and MPC have bank
  buttons that work on every screen).
- The cut jump moves to the snap switch `N` as its third step.
- The help line: 1 of 3 ("Q2.1"), columns, and the owner: "use the three lines
  efficiently". So the help stays 3 rows; every group is a column of up to 3 rows of
  `key text`; the group name stands in colour left of its first row, not on a row of its
  own; the columns are split by `│`. The order is the same on every screen: the screen's
  own groups first, then BANK, REC, SCREENS. A group never breaks across columns.

```
 PAD  Enter edit   │ CHANGE M      mark: off │ BANK J L PgUp PgDn turn │ REC Ctrl+R mic: hold │ F3 chop
      H     mode   │        Ctrl+A scope     │      Ctrl+1…8      pick │     Ctrl+T output    │ F6 files
      Bksp  clear  │                         │                         │                      │ F1 help
```

- The chop help names `Backspace` "remove cut", not "join": the owner looked for a way
  to remove a cut and did not read it in "join"; the key worked once found.

- The X-Session profile stays: b1 b2 turn the banks on the pads screen and are play and
  cut on the chop screen ("Q3.1"); on the chop screen the banks turn from the keyboard.

- A held `I` `K` walks the settings rows, as in the files list ("Q6.1", owner: "ok").
- The pad edit panel shows the rows of all pages at once (owner: "show everything here so
  I do not scroll"): the SOURCE rows no longer take the place of the ENV rows; the panel
  grows by 3 rows into the empty space under it. The page still says what the knobs turn.
- The frame title of an edit names every page, the current one lit (`EDIT · TONE ENV
  SOURCE`): the owner on the SOURCE page, 2026-10-31: "it is unclear what can be
  scrolled".

## Open questions

- `Backspace` with the cursor in the first slice does nothing and says nothing: there is
  no cut on its left. Default: leave it; `Alt+Backspace` removes the right one.
- A terminal too narrow for all the columns. Default: the last groups drop off first
  (SCREENS, then REC), F1 shows them all.
- The chop screen at 140 columns (measured while coding, 2026-10-31): its 19 items in
  columns need about 180 columns with every name beside its group. A group of one or two
  items now puts its name on the free first row above them; then everything but the
  screens group fits, and the screens group drops off on the chop screen (F1 shows it),
  as the default for a narrow terminal says. The "Narrow terminal" scenario was written
  before the measure and says so now.
- Under kitty the chop help never showed `Backspace`: its item carried `Ctrl+Bksp` too,
  and the help hides combos until the modifier is held. The item now names only `Bksp`.
- A terminal too short for the taller pad panel. Default: the last rows (the pickup line)
  are cut at the bottom, as a long name is cut at the edge.

## Archive order

The MODIFIED requirements stand on the open versions of 21 ("Cursor modes", "Jump to a
cut", "Banks on J and L", "Help line fits the terminal", "The screen shows the layer") and 27 ("Source page"): 43 is
archived after 21 and 27.

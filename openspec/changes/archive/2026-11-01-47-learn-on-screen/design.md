# Design 47

## Decisions (owner, 2026-10-31)

- 1 of 3 ("Q1.1"): the learn mode on the screen, `F4`. Weighed: a keyboard key then a
  control (Q1.2), no way to reach parameters without a key; the F12 list beside the
  screen (Q1.3), nothing lit on the screen itself.

```
 LEARN · pads screen                         F4 / Esc: leave
 ┌1────────┐┌2────────┐...      ┌ pad Q ──────────────┐
 │ pad 1   ││ pad 2   │         │ volume  ◀ k1        │
 │ ◀ note 36│ ◀ ---   │         │ pitch   ◀ ---       │
 └─────────┘└─────────┘         │ ▶ attack ◀ ???      │
 bank ◀ b1 ▶ b2                 └─────────────────────┘
```

- The layer of a binding: 1 of 3 ("Q2.1"), the layer the screen is in now: the edit
  layer of 28 while an edit is open, else the screen's group. A bank button bound on the
  pads screen does not act on the chop screen until bound there too. Weighed: banks,
  recording, master and screens to GLOBAL (Q2.2); everything to GLOBAL (Q2.3).

- The walk: 1 of 2 ("Q3.1", "yes"): four zones, the grid by rows, the bank switch, the
  panel, the commands of the help line; `I` `K` jump zones, `J` `L` move inside one.
  Weighed: without the commands (Q3.2).

## Open questions

None yet.

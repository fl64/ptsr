## Context

The owner approved a mockup of the Pattern screen on 2026-11-27 (`docs/plans/PATTERNS.md`,
section "Pattern track"). The owner split the work into five changes. This change gives F5
the new look and keeps every current key and controller route.

## A screen for the owner, a Pads mode inside

Decision: the owner sees the Pattern screen as its own screen. It has the title `F5 PATTERN`
and its own layout in place of the pad grid and the pad inspector. Esc and F2 return to Pads.
The command panel shows the `Pattern` view. Inside the program, it stays the existing Pads
overview mode, with the same `[screens.pads]` keymap section and the same MIDI context.

Reason: owner keymaps bind pattern keys under `[screens.pads]` (F7, Alt+Up, Alt+Down, Alt+H, Alt+B).
A new screen section would drop those bindings without a warning. It would also need a config
migration, and AGENTS.md does not permit a format change without one.

Pad routing, bank selection, Space and the record keys act as on Pads.
The `Alt+N` panel, Step Edit and erase keep their input owners.
New code goes into modules. `crates/app/src/main.rs` and `crates/tui/src/lib.rs` do not grow.

## Layout

```text
 F5 PATTERN  ▶ 1.3  2 bars  120 BPM  bar 1/2                         [STEP EDIT]
            1  ·  ·  ·  2  ·  ·  ·  3  ·  ·  ·  4  ·  ·  ·
 1 kick     █  ·  ·  ·  ·  ·  █  ·  █  ·  ·  ·  ·  ·  ·  ·
 2 snare    ·  ·  ·  ·  █  ·  ·  ·  ·  ·  ·  ·  █  ·  ·  ▪
 ───────────────────────────────────────────────────────────
 Q hat      ▓  ·  ▪  ·  ▓  ·  ▪  ·  ▓  ·  ▪  ·  ▓  ·  ▪  ·
 velocity   ▆  ▁  ▁  ▁  ▁  ▁  ▆  ▁  ▆  ▁  ▁  ▁  ▁  ▁  ▁  ▁
 step 1.1 · kick · loud (vel 127)
 setup  bars 2 · tempo 120 · grid 1/16 · record quantize 1/16 100 % · metronome on · count-in 1 bar
```

- Row labels keep a fixed width that fits `Q ` and the name. The cells share the rest.
  The cell width is `max(3, rest / cells on the page)`. Extra columns stay at the right end.
- The header line puts the mode badge at the right end. A narrow terminal shortens the tempo and length text first.
- When the rows do not fit, the list scrolls to keep the selected pad row visible. The last row says how many rows it hides.
- The instruction cues of changes 140 and 164 use the detail line until the owner selects a cell, and during a pass.

## Velocity

The levels keep the thresholds of change 123: 1–42, 43–84 and 85–127. On-grid single hits use
`▪`, `▓` and `█`. `~` and the count digits stay, so the type of a cell stays visible without color.
The lane uses eight bar heights. It scales the velocity 1–127 to `▁`…`█`.

## Words

The Pattern screen, the Pads lane, the screens view cell and F5 help say `pattern`.
The record badge, notices and the panel action labels keep `A1` until the change that touches them.

## Routes that stay

| Action | Keyboard | SMC-PAD |
|---|---|---|
| Open the screen | F5 | screens view `pattern` |
| Setup | Alt+N | `A1 setup` cell, then knobs |
| Step Edit | F7 | `edit steps` cell |
| Record | Ctrl+R | RECORD |
| Erase | Alt+E | `erase hits` cell |
| Tap tempo | Alt+B | `tap tempo` cell |
| All rows | Alt+H | `hide empty` cell |
| Step and bar | J/L, I/K | E1, LEFT/RIGHT |

The press counts do not change.

## Open questions

- Alt+H adds all rows of the selected bank, not of all eight banks. Default: the selected bank only.
- An absent pattern cannot enter Step Edit today. Its second line names the setup control. Default: keep this until 176.
- Other visible `A1` words stay until the change that touches them. Default: yes.
- The count-in stays one bar and the setup line only shows it (owner decision, 2026-11-27).

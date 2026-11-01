# 14 — Key help that reads at a glance

## Why

The help line is a run of `[key] text` in one color: it takes two or three rows, the Ctrl combos crowd it, and the full help (F1) splits pairs of keys over rows, cuts long lines and repeats keys in the text. The owner asked for the help to react to a held modifier ("urgent", backlog item 5).

## What changes

- A state already shown on the screen is not repeated in the help line: on the pads screen the pad panel shows mode and volume, the help line says only "mode" and "volume".
- Keys are drawn as key caps: a light background with dark text instead of brackets, so a key and its meaning separate at once.
- Hold a modifier: the help line shows only the keys that work with it, nothing else. Hold Ctrl: the line shows the Ctrl combos of the screen (`CTRL R mic: hold  T output  X take pad  Z undo  1…8 bank`); release it and the usual line is back.
- Hold Shift: the line shows only that pads play at full velocity.
- Without a held modifier the line shows only keys without modifiers; on 140 columns it fits one row.
- In a terminal without the kitty protocol (no modifier events) nothing is hidden: the Ctrl combos stay in the usual line.
- F1: keys of one kind share a row (`← →  change value`, `PgUp PgDn  bank`), no line is cut, the text does not repeat its key.

## Out of scope

- Colors beyond the current palette (06d-colors).
- Help for the MIDI controller buttons.
- Shortening `Ctrl+` to `^`: F1 keeps `Ctrl+R`, the Ctrl view shows the bare letter under its CTRL label.

## How to verify

1. Pads screen, 140 columns: the help line is one row, keys look like caps.
2. Hold Ctrl: the line shows only the Ctrl combos of this screen, no other keys; release: the usual line is back. Same on F6, F3, F12.
3. Hold Shift: the line says pads play at full velocity; hit a pad with Shift: full velocity as before.
4. F1: `← →` one row, `↑ ↓` one row, `PgUp PgDn` one row; the longest line is whole; "record the microphone" has no "(hold Ctrl+R)".
5. In tmux (no kitty protocol): the Ctrl combos are in the usual line.

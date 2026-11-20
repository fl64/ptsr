# Proposal

## Why

On 2026-11-20 the owner did not understand which bound `cut -` removes, and asked that every
panel label say plainly what the press does. Many labels are short codes: `p.gate`, `all`,
`fill`, `snap`, `whole`, `normal`, `grid -`, `reg[`.

## What Changes

- Every action label on the panel uses two plain words where one word is not clear, for example `pitch gate`, `mark bank`, `fill pads`, `cursor step`.
- The cell shows the two words on two lines, as change 135 adds.
- `hide pads` shows `show pads` while the pads are hidden.
- The Chop knob strip names the region knobs `r.from` and `r.to`, and the threshold knob `hits`. The line of the last knob names them in full.
- The screens view and the control row stay unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `command-panel`: the shipped action table.
- `screen-layout`: the Chop knob labels.

## Impact

The change affects labels only. Actions, keys, profiles, saved files and sound stay unchanged.
No new dependency. The change needs change 135 first, because 135 adds labels in two lines.

Owner checks, controller:

1. `PADS` → hold [PAD BANK] → «edit pad», «pitch gate», «mark bank», «hide pads», «chop pad».
2. `CHOP` → hold [PAD BANK] → «cut here», «equal parts», «fill pads», «cursor step».
3. `CHOP` → turn knob 5 → the strip says «r.from» and the line says «region start».

Renders: none, because sound stays unchanged.

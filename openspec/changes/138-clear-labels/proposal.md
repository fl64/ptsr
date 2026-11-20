# Proposal

## Why

On 2026-11-20 the owner did not understand which bound `cut -` removes, and asked that every
panel label say plainly what the press does. Many labels are short codes: `p.gate`, `all`,
`fill`, `snap`, `whole`, `normal`, `grid -`, `reg[`.

## What Changes

- Every action label on the panel uses two plain words where one word is not clear, for example `pitch gate`, `mark bank`, `fill pads`, `cursor step`.
- The cell shows the two words on two lines, as change 135 adds.
- `hide pads` shows `show pads` while the pads are hidden.
- `whole sample` is dim while the region is the whole sample, and a press says what happened.
- The Chop knob strip names the region knobs `r.from` and `r.to`, and the threshold knob `hits`. The line of the last knob names them in full.
- A cell that cannot act now is dim, and a press says why.
- Cells that remove or replace work use the warning colour. Before hand cuts, a pad sound or a learned control goes, the program asks. After it, a notice names the undo.
- `level to peak` gets an undo step.
- The screens view and the control row stay unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `command-panel`: the shipped action table.
- `chopping`: the result of `whole sample`.
- `screen-layout`: the Chop knob labels.

## Impact

The change affects labels only. Actions, keys, profiles, saved files and sound stay unchanged.
No new dependency. The change needs change 135 first, because 135 adds labels in two lines.

Owner checks, controller:

1. `PADS` → hold [PAD BANK] → «edit pad», «pitch gate», «mark bank», «hide pads», «chop pad».
2. `CHOP` → hold [PAD BANK] → «cut here», «equal parts», «fill pads», «cursor step».
3. `CHOP` → turn knob 5 → the strip says «r.from» and the line says «region start».
4. `CHOP` → [PAD BANK]+[P5] «whole sample» → «region: whole sample» → press again → the cell is dim → «the sample is already whole».

Renders: none, because sound stays unchanged.

# Proposal

## Why

On 2026-11-20 the owner did not understand which bound `cut -` removes, and asked that every
panel label say plainly what the press does. Many labels are short codes: `p.gate`, `all`,
`fill`, `snap`, `whole`, `normal`, `grid -`, `reg[`.

## What Changes

- Every action label on the panel uses two plain words where one word is not clear, for example `pitch gate`, `mark bank`, `fill pads`, `cursor step`.
- The cell shows the two words on two lines, as change 135 adds.
- `hide pads` shows `show pads` while the pads are hidden.
- `reset trim` is dim while nothing is trimmed, and a press says what happened.
- Chop uses one word for the cut edges of the sound: trim. Knobs 5 and 6 are `trim<` and `trim>`, the cell is `reset trim`, the threshold knob is `hits`.
- A cell that cannot act now is dim, and a press says why.
- Cells that remove or replace work use the warning colour. Before hand cuts, a pad sound or a learned control goes, the program asks. After it, a notice names the undo.
- `level to peak` gets an undo step.
- Chop gets `remove all cuts` on P6 and the Delete key. It asks first and can be undone.
- A label line may use eight cells.
- The screens view and the control row stay unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `command-panel`: the shipped action table.
- `chopping`: the result of `reset trim`.
- `screen-layout`: the Chop knob labels.
- `controller-reach`: the Learn warning quotes the new label.

## Impact

The change affects labels only. Actions, keys, profiles, saved files and sound stay unchanged.
No new dependency. The change needs change 135 first, because 135 adds labels in two lines.

Owner checks, controller:

1. `PADS` → hold [PAD BANK] → «edit pad», «pitch gate», «mark bank», «hide pads», «chop pad».
2. `CHOP` → hold [PAD BANK] → «cut here», «equal parts», «fill pads», «cursor step».
3. `CHOP` → turn knob 5 → the start goes grey → the strip says «trim<» and the line says «trim start».
4. `CHOP` → turn knob 5 → [PAD BANK]+[P5] «reset trim» → «trim reset: the whole sample plays» → press again → the cell is dim → «nothing is trimmed».
5. `CHOP` → cut by hand → [PAD BANK]+[P6] «remove all cuts» → «Remove 6 cuts?» → [P4] → «6 cuts removed · P2 undo brings them back».

Renders: none, because sound stays unchanged.

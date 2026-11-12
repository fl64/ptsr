# Proposal

## Why

The owner asked for one look and one behavior for every edit element (`docs/plans/VISUAL.md`, section 2.9). The pad edit and the A1 pattern controls already share one look. F4 shows two separate cards, `▶ FX 1 · selected` and `FX 2`, without the edit frame and without block lines.

## What Changes

- F4 draws one frame in the edit color, titled `EDIT · FX 1 FX 2`. The selected slot is lit in the title, as the open page of the pad edit.
- Inside, block lines name each slot and its type: `─ FX 1 · reverb ─`, `─ FX 2 · delay ─`. The block line of the selected slot is lit, the other is dim.
- The rows keep their knob numbers 1–3. In the selected slot, the selected row is bright. The rows of the other slot are dim.
- Both slots stay visible. Tab, I/K, J/L and the knobs keep their roles.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pad-playback`: the shared FX screen uses the edit frame.

## Impact

Drawing of F4 only. No sound, input, knob or saved-format change.

## Out of scope

- The Chop slice edit (98f) and Settings (98g).

## How to check

1. Open F4 at 120x40 and at 80x26.
2. Check one frame with `EDIT · FX 1 FX 2` and FX 1 lit. Press Tab: FX 2 is lit.
3. Press I/K and J/L: the selected row is bright and the value changes.

# Proposal

## Why

Two Codex walks on 2026-11-19 checked 38 user stories and 129 SMC-PAD actions. Change 134
fixed the dead ends, the labels and the hints. Some screen problems remain. Text covers text,
hints are cut at 80×26, and a Learn conflict shows only after the save. One hint still names
a key after SMC-PAD input.

## What Changes

- Chop notices do not cover the time and threshold fields of the wave.
- At 80×26 the Settings row under the panel and the Assign mode line with apply and cancel stay readable.
- Learn names a panel cell that a new binding replaces, before the save.
- The marking hint names SMC-PAD controls after SMC-PAD input.
- In the slice edit, the panel cells `cut -` and `end -` become `join left` and `join right`. While the panel shows them, the wave marks the two bounds of the selected slice. A cell with no neighbour is dim and explains why. After a join, a notice names the joined slices.
- A panel label longer than seven cells breaks at its first space into two lines. Each line keeps at most seven cells.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `screen-layout`: Chop notices and the 80×26 layout with the panel.
- `controller-reach`: the Learn conflict warning.
- `screen-help`: the marking hint after SMC-PAD input.
- `chopping`: join the selected slice with its left or right neighbour.
- `command-panel`: the Chop edit actions table and labels in two lines.

## Impact

The change affects drawing and hint text only. Keys, saved files and sound stay unchanged.
No new dependency. The change follows change 134.

Owner checks, controller:

1. `CHOP` → `[PAD BANK]+[P15]` (hits) → the time and threshold stay readable.
2. 80×26 → `CHOP` → `[PAD BANK]+[P16]` → the line shows apply and cancel.
3. F12 MIDI: PAD BANK hold off → `PADS` → `LEARN` → pick pad mode → touch `[PAD BANK]+[P6]` → «replaces panel cell P6 rec out» before the save.

4. `CHOP` → hit the pad of slice 3 → `[PAD BANK]+[P7]` (edit) → «join left», «join right» and two marked bounds → `[PAD BANK]+[P15]` → «slices 2 and 3 joined».
5. `CHOP` → hit the pad of slice 1 → `[PAD BANK]+[P7]` → «join left» is dim → `[PAD BANK]+[P15]` → «slice 1 has no slice on the left».

Owner checks, keyboard: Esc and Enter keep their results.

Renders: none, because sound stays unchanged.

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

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `screen-layout`: Chop notices and the 80×26 layout with the panel.
- `controller-reach`: the Learn conflict warning.
- `screen-help`: the marking hint after SMC-PAD input.

## Impact

The change affects drawing and hint text only. Keys, saved files and sound stay unchanged.
No new dependency. The change follows change 134.

Owner checks, controller:

1. `CHOP` → `[PAD BANK]+[P15]` (hits) → the time and threshold stay readable.
2. 80×26 → `CHOP` → `[PAD BANK]+[P16]` → the line shows apply and cancel.
3. `LEARN` → touch `[PAD BANK]+[P6]` → «replaces panel cell P6 open» before the save.

Owner checks, keyboard: Esc and Enter keep their results.

Renders: none, because sound stays unchanged.

# Proposal

## Why

On 2026-11-19 the owner chopped a break into more than 16 slices from the SMC-PAD. The
Chop screen showed the last slices, because the slice page follows the pad bank. The
owner did not find the other pages. After Apply, one Undo did not bring back the old
pads. The owner asked to make Chop an action on a pad, as on the MPC and the SP-404.

## What Changes

- The Pads actions get `chop`. It opens Chop for the sound of the selected pad, as F3 does.
- Chop opens on the slice page of that pad, or on the first page for a whole file.
- One Undo after an Assign apply restores every pad of that apply.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chopping`: Chop from the selected pad, the first slice page and one Undo for an apply.
- `command-panel`: the Pads actions table.

## Impact

The change affects the Pads panel page, the Chop entry and the undo history of an Assign
apply. Keyboard keys and saved files stay unchanged. Sound stays unchanged. No new dependency.
The change needs change 134 first.

Owner checks, controller:

1. `PADS` → `[P1]` → `[PAD BANK]+[P11]` (chop) → `CHOP` with that sound → «slices 1–16».
2. `CHOP` → `[PAD BANK]+[P16]` → hit 2 used pads → `[PAD BANK]+[P4]` → `[P4]` → `[PAD BANK]+[P2]` → the old sounds come back.

Owner checks, keyboard: F3 on a pad keeps its result. Ctrl+Z after an apply restores all pads.

Renders: none, because sound stays unchanged.

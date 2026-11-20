# Proposal

## Why

On 2026-11-19 the owner chopped a break into more than 16 slices from the SMC-PAD. The
Chop screen showed the last slices, because the slice page follows the pad bank. The
owner did not find the other pages. After Apply, one Undo did not bring back the old
pads. The owner asked to make Chop an action on a pad, as on the MPC and the SP-404.

## What Changes

- The Pads actions get `chop` on device pad P7. It opens Chop for the sound of the selected pad, as F3 does.
- Each chop keeps its own slice page. Chop opens on the page of the selected slice, also from Files.
  A new chop or a whole file opens on slices 1–16.
- The page keys turn the slice page and do not change the pad bank of Pads.
- The row over the slice keys shows the page: `slices 17–32 · page 2/3`.
- One Undo after an Assign apply restores every pad of that apply.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chopping`: Chop from the selected pad, the own slice page of a chop and one Undo for an apply.
- `command-panel`: the Pads actions table.

## Impact

The change affects the Pads panel page, the Chop entry, the slice page and the undo history
of an Assign apply. Keyboard keys and saved files stay unchanged. Sound stays unchanged.
No new dependency.

Owner checks, controller:

1. `PADS` → bank 3 → `[P1]` with a whole break → `[PAD BANK]+[P7]` «chop» → `CHOP` → cut into 40 slices → `[PAD BANK]+[P1]` → `[PAD BANK]+[P7]` → «slices 1–16 · page 1/3».
2. `CHOP` → `RIGHT` → «slices 17–32 · page 2/3» → `[PAD BANK]+[P1]` → `PADS` shows bank 3.
3. `CHOP` → `[PAD BANK]+[P16]` «assign» → hit 2 used and 2 empty pads → `[PAD BANK]+[P4]` → `[P4]` → `PADS` → `[PAD BANK]+[P2]` «undo» → the old sounds come back, the 2 other pads are empty.

Owner checks, keyboard: F3 on a pad keeps its result. PgDn on the slice keys turns the page.
Ctrl+Z after an apply restores all pads.

Renders: none, because sound stays unchanged.

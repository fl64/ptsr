# Design

## Decisions

- **One frame.** The two slots are blocks of one edit, as the pages of the pad edit. Both stay visible, so the owner can compare them.
- **Title.** `EDIT · FX 1 FX 2`, the selected slot lit with the edit color background. A narrow column drops `EDIT ·` first, as the pad edit does.
- **Blocks.** A block line names the slot and its type. The lit block line uses the edit color, the other is dim.
- **Rows.** Rows keep the knob number, the label and the value. Learned controls keep the `◀ control` mark in the selected slot.
- **Size.** The frame takes the height of its content: two block lines and six rows. It does not fill unused rows.

## Open questions

None.

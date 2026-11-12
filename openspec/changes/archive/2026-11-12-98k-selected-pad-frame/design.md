# Design

## Decisions

- **Frame over state.** The selected pad gets the cyan frame before the hit, mark and grid-mode colors. Only a pad in hand keeps its magenta frame: the hand is a mode of the next pad hit.
- **Frame cells stay dark.** On a flash, the inside of the selected pad is white and its frame cells keep the panel background. Cyan lines on a white background are hard to see.
- **Key letter.** The key letter of the selected pad has the frame color, so it stays readable on the dark frame.

## Open questions

None.

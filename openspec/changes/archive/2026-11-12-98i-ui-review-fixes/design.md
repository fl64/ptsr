# Design

## Decisions

- **Label width.** The strip label column grows from 5 to 6 cells, so `pad Q` and a space fit. The cell grid moves one column right.
- **Narrow hints.** A row hint stays beside its value when it fits. Otherwise it takes the next line, dim and indented under the value. Refusals keep the warning color there.
- **Key line.** Pattern controls say `Esc/Alt+N close · Enter run`. Enter runs Clear A1 and closes from a value row, as before.
- **Undo names.** A pattern step keeps its name: the pass, `erase pad <key>` or `clear`. The undo notice uses it.
- **Help order.** While the pad edit is open, the A1 help group moves after the edit group.
- **Word wrap.** F1 descriptions and keys wrap at spaces. A word wider than the column wraps by graphemes, as before.

## Open questions

None.

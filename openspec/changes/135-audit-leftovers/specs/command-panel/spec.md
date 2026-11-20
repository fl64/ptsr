## MODIFIED Requirements

### Requirement: Each view has at most 12 actions

A screen or mode SHALL have one action page with at most 12 actions in cells 1 through 12.
An action that does not fit SHALL go to its own screen or mode.
The loader SHALL reject a second action page with a notice.
The shipped actions SHALL match the table. A dash is a dim cell.

#### Scenario: Shipped action pages match the table
- **WHEN** the layout test reads each view of the shipped panel file
- **THEN** each view has one page with the labels and actions of the table in cell order

#### Scenario: A second action page is rejected
- **WHEN** a panel file gives a view a second action page
- **THEN** a notice names the file and view, and the first page keeps its cells

| View | Actions in cell order |
| --- | --- |
| Pads | edit, copy, swap, mark, mode, remove, p.gate, all, grid, rec out, chop |
| Pad edit | mode, reset, p.gate, copy |
| A1 | step, grid -, grid +, params, record, erase, undoA1, clear, rows, tap |
| Files | to pad, load, —, star, —, listen, chop |
| Chop | cut, parts, hits, assign, loop, listen, snap, as pad, whole, —, edit, fill |
| Chop edit | normal, hits -, join left, join right, whole |
| Settings | —, learn, unbind |

### Requirement: Shipped panel labels use short words

Every shipped panel label SHALL use approved words in one or two lines.
The first space of a label SHALL start the second line.
Each line SHALL use at most seven cells.
A label SHALL have one meaning on every screen.
The label check SHALL include fixed cells and the shipped panel file.

#### Scenario: Shipped labels fit the panel glossary
- **WHEN** a shipped panel label line exceeds seven cells or uses an unknown word
- **THEN** the label check fails

#### Scenario: A two-word label uses two lines
- **WHEN** the panel shows `join left` at 80×26 and at 120×40
- **THEN** the cell shows «join» on its first line and «left» on its second line

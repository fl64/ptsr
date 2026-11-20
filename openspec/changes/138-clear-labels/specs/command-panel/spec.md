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
| Pads | edit pad, copy pad, swap pads, mark pad, play mode, clear pad, pitch gate, mark bank, hide pads, record output, chop pad |
| Pad edit | play mode, reset pad, pitch gate, copy pad |
| A1 | edit steps, steps wider, steps finer, A1 setup, record A1, erase hits, undo A1 pass, clear A1, hide empty, tap tempo |
| Files | take file, load to pad, —, star file, —, listen file, chop file |
| Chop | cut here, equal parts, cut by hits, put on pads, loop play, play range, cursor step, hear as pad, region whole, —, edit slice, fill pads |
| Chop edit | level to peak, fewer hits, join left, join right, region whole |
| Settings | —, learn control, forget control |

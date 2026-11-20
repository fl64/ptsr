## ADDED Requirements

### Requirement: A cell that cannot act now is dim

A panel cell SHALL be dim while its action cannot change anything, as the design table lists. A press on a dim cell SHALL show a notice that says why. The dim state SHALL follow the current state at once.

#### Scenario: Pad cells on an empty pad are dim
- **WHEN** the selected pad is empty and the owner holds PAD BANK on Pads
- **THEN** `copy pad`, `clear pad` and `chop pad` are dim, and a press on `copy pad` says «Copy what? This pad holds nothing.»

#### Scenario: Fewer hits at the top threshold is dim
- **WHEN** the chop is cut by hits at −6 dB and the owner holds PAD BANK in the slice edit
- **THEN** `fewer hits` is dim and a press says «by hits stays at -6 dB»

#### Scenario: Forget control without a learned control is dim
- **WHEN** the selected MIDI row has no learned control and the owner holds PAD BANK on Settings
- **THEN** `forget control` is dim and a press says that the row has no learned control

### Requirement: Destructive cells use a warning colour and ask first

A cell whose action replaces or removes the owner's work SHALL use the warning colour. Some actions replace cuts made by hand, a pad sound or a learned control. Before such an action, the program SHALL ask, with P4 yes and P1 no. After the action, a notice SHALL say what changed. It SHALL name the undo control, or say that no undo exists.

#### Scenario: Equal parts asks before it replaces hand cuts
- **WHEN** the owner made 4 cuts by hand and presses `equal parts`
- **THEN** a question asks «Replace 4 hand cuts with 2 equal parts?»
- **AND** after yes a notice says «4 cuts replaced · P2 undo brings them back»

#### Scenario: The threshold knob asks before it replaces hand cuts
- **WHEN** the owner made cuts by hand and turns knob 7 on Chop
- **THEN** a question asks before any cut changes, and the cuts stay until the owner answers yes

#### Scenario: Clear pad asks for a pad with a sound
- **WHEN** the selected pad holds a sound and the owner presses `clear pad`
- **THEN** a question asks first, and after yes a notice says «pad cleared · P2 undo brings it back»

#### Scenario: Forget control asks and says that it has no undo
- **WHEN** the selected MIDI row has a learned control and the owner presses `forget control`
- **THEN** a question says that no undo exists, and after yes a notice names the forgotten control

#### Scenario: Level to peak can be undone
- **WHEN** the owner presses `level to peak` and then undo on Pads
- **THEN** the pad volume returns to its value before the press

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
| Chop | cut here, equal parts, cut by hits, put on pads, loop play, play range, cursor step, hear as pad, reset trim, remove all cuts, edit slice, fill pads |
| Chop edit | level to peak, fewer hits, join left, join right, reset trim |
| Settings | —, learn control, forget control |

### Requirement: Shipped panel labels use short words

Every shipped panel label SHALL use approved words in one or two lines.
A label longer than eight cells SHALL break at its first space into two lines.
Each line SHALL use at most eight cells.
A label SHALL have one meaning on every screen.
The label check SHALL include fixed cells and the shipped panel file.

#### Scenario: Shipped labels fit the panel glossary
- **WHEN** a shipped panel label line exceeds eight cells or uses an unknown word
- **THEN** the label check fails

#### Scenario: A two-word label uses two lines
- **WHEN** the panel shows `join left` at 80×26 and at 120×40
- **THEN** the cell shows «join» on its first line and «left» on its second line

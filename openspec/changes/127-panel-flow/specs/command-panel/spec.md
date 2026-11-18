## ADDED Requirements

### Requirement: Panel views move with LEFT and RIGHT

The panel SHALL have a row of views: screens, then action pages of the current view.
A reveal SHALL show action page 1. LEFT and RIGHT SHALL move one view without a wrap.
A screen change SHALL show action page 1 of the new screen.
A question or the target view SHALL keep its cells, and LEFT and RIGHT SHALL do nothing.

#### Scenario: The panel opens on the screen actions
- **WHEN** the owner reveals the panel on Chop
- **THEN** the cells show Chop action page 1
- **AND** the header names the screens view on the left and action page 2 on the right

#### Scenario: LEFT and RIGHT move through the views
- **WHEN** the owner presses LEFT and RIGHT on the visible panel of Pads
- **THEN** LEFT from action page 1 shows the screens view and RIGHT shows action page 2
- **AND** a press past either end of the row keeps the current view

#### Scenario: A screen hit opens that screen with its actions
- **WHEN** the owner hits Files in the screens view
- **THEN** Files opens and the panel shows Files action page 1

### Requirement: The screens view opens screens and global actions

The screens view SHALL show pads, files, chop and A1 in row 1; fx, mixer, setup and help in row 2; save, open, learn and exit in row 3.
Mixer SHALL stay dim until it exists. Help SHALL open Controller Help.
Global actions SHALL appear only in the screens view.

#### Scenario: Global actions live in the screens view
- **WHEN** the owner visits the screens view and every action page of each current view
- **THEN** save, open, learn and exit appear only in the screens view
- **AND** no action page shows play or help

### Requirement: Action pages follow the shipped table

Each action page SHALL use cells 1 through 12 in physical order.
The shipped pages SHALL match the table. A dash is a dim cell.

#### Scenario: Shipped action pages match the table
- **WHEN** the layout test reads each view of the shipped panel file
- **THEN** each page has the labels and actions of the table in cell order

| View | Page 1 | Page 2 |
| --- | --- | --- |
| Pads | edit, copy, take, mark, mode, clear, gate, marks, grid, output | grid -, grid +, rows, step, A1 rec, erase, A1undo, knobs |
| Pad edit | mode, clear, gate, copy | — |
| A1 | step, grid -, grid +, knobs, record, erase, undo, clear, rows, tap | — |
| Files | take, load, parent, star, enter, auto, chop | — |
| Chop | cut, parts, hits, assign, loop, play, snap, as pad, start, end, whole, commit | hits -, cut -, end -, order, level, edit |
| Chop edit | level, hits -, cut -, end -, order | — |
| Settings | tab, learn, clear, —, row -, row + | — |

## MODIFIED Requirements

### Requirement: Panel fixed cells keep physical positions

The panel SHALL draw 16 cells in the same top-to-bottom order as the physical pads.
Each cell SHALL show the number printed on the device for that pad.
Its bottom row SHALL keep back, undo, redo and panic in cells 13 through 16.
Redo and Panic SHALL be dim and inactive until change 93a.

#### Scenario: Fixed cells stay on the bottom row
- **WHEN** the owner changes panel views or screens
- **THEN** cells 13 through 16 keep back, undo, redo and panic in physical left-to-right order
- **AND** Redo and Panic are dim and run no action

#### Scenario: Cells show the device numbers
- **WHEN** the panel shows at either supported size
- **THEN** the bottom-left cell shows 1 and the top-right cell shows 16

### Requirement: Panel configuration names logical actions

`config/panel.toml` SHALL name action pages, starting at page 1, with short labels and logical action IDs. The loader SHALL reject cells outside 1 through 12, repeated cells, unknown actions and labels wider than six terminal cells. It SHALL name the file and cell in a notice. A bad cell SHALL stay inactive while valid cells and pad play continue. A valid cell SHALL use normal action dispatch and existing confirmation checks.

#### Scenario: Invalid panel cells do not disable pad play
- **WHEN** a panel file names an unknown action and repeats a cell
- **THEN** each bad cell is inactive and a notice names its file and cell
- **AND** a valid screen cell still acts and first-bank pads still sound

#### Scenario: A destructive cell cannot confirm itself
- **WHEN** a panel cell requests a destructive action with an occupied target or other required question
- **THEN** the normal visible question opens without changing the target yet
- **AND** the same press cannot answer or bypass that question

### Requirement: SMC-PAD target cells select pads on the current layer

A visible target view SHALL use cells 1 to 12 as pad positions in the current bank.
It SHALL show that pads 13–16 use the playing layer.
Cell 13 SHALL cancel. Cell 14 SHALL keep undo. Cell 15 SHALL switch targets and commands.
Cell 16 SHALL apply Assign. Questions, ownership and original held releases SHALL keep priority.

#### Scenario: Panel target pads place items without a layer switch
- **WHEN** the owner takes a pad, a copy or a file and selects cells 1 through 12 on the visible target view
- **THEN** the matching pad gets the existing placement result without a layer change
- **AND** a replacement question stays available to cancel or accept

#### Scenario: Assign applies from the target row
- **WHEN** the owner selects Assign targets on the visible command layer or playing layer
- **THEN** the plan keeps those targets and cell 16 applies it on the visible target view
- **AND** cell 13 cancels, cell 14 stays undo and cell 15 gives access to the other commands

#### Scenario: Target cells retain connection and release priority
- **WHEN** another Master sends a target hit or a held layer 1 pad releases during target selection
- **THEN** only the owner can choose a target and the original held pad releases

## REMOVED Requirements

### Requirement: Panel pages expose screen and context commands

**Reason**: Change 127 replaces the main page, More, Next and Main with views that LEFT and RIGHT move.
**Migration**: `Panel views move with LEFT and RIGHT`, `The screens view opens screens and global actions` and `Action pages follow the shipped table`.

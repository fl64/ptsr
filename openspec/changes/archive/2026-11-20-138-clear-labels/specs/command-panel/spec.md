## ADDED Requirements

### Requirement: Panel cells have four looks

A panel cell SHALL have one of four looks. A bright cell SHALL act now. An orange cell SHALL act and replace or remove the owner's work. A grey cell SHALL have an action that cannot change anything now. An empty cell SHALL have no action. The look SHALL follow the current state at once. A press on a grey cell SHALL show a notice that says why and what to do first. The screens view and the bottom row SHALL keep their colours.

#### Scenario: Pad cells on an empty pad are grey
- **WHEN** the selected pad is empty and the owner holds PAD BANK on Pads
- **THEN** `copy pad`, `chop pad` and `clear pad` are grey
- **AND** a press on `copy pad` says «Copy what? This pad holds nothing.»

#### Scenario: Remove all cuts without cuts is grey
- **WHEN** the chop has no cut and the owner holds PAD BANK on Chop
- **THEN** `remove all cuts` is grey and a press says «no cuts to remove»

#### Scenario: Load to pad on a folder is grey
- **WHEN** the Files cursor is on a folder and the owner holds PAD BANK
- **THEN** `load to pad`, `take file` and `chop file` are grey
- **AND** a press on `load to pad` says «RIGHT opens the folder»

#### Scenario: Erase hits during the count-in is grey
- **WHEN** the A1 count-in runs and the owner holds PAD BANK on A1
- **THEN** `erase hits` and `edit steps` are grey and a press says «wait for the count-in»

#### Scenario: Destructive cells are orange
- **WHEN** the owner holds PAD BANK on Chop with cuts and a sound on the selected pad
- **THEN** `equal parts`, `cut by hits`, `fewer hits`, `remove all cuts` and `trim silence` are orange
- **AND** on Pads `clear pad` is orange

### Requirement: Destructive actions ask when work goes and name the undo

Before an action replaces cuts made by hand or removes a pad sound, the program SHALL ask, with P4 yes and P1 no. Cuts made by the program SHALL be replaced without a question. The question SHALL name what goes and the undo control. A question SHALL say «This cannot be undone.» only when no undo exists. After the action, a notice SHALL name the result and the undo control. The notice SHALL use the words of the last input device.

#### Scenario: Equal parts asks before it replaces hand cuts
- **WHEN** the owner made 4 cuts by hand and presses `equal parts`
- **THEN** a question asks «Replace 4 hand cuts with 2 equal parts? P2 undo brings them back.»
- **AND** after yes a notice says «4 cuts replaced · P2 undo brings them back»

#### Scenario: Cut by hits replaces program cuts without a question
- **WHEN** the chop has only cuts by parts and the owner presses `cut by hits`
- **THEN** the cuts change at once and no question shows

#### Scenario: The threshold knob asks before it replaces hand cuts
- **WHEN** the owner made cuts by hand and turns knob 7 on Chop
- **THEN** a question asks before any cut changes, and the cuts stay until the owner answers yes

#### Scenario: Clear pad asks for a pad with a sound
- **WHEN** the selected pad holds a sound and the owner presses `clear pad`
- **THEN** a question names the pad, and after yes a notice says «pad cleared · P2 undo brings it back»

#### Scenario: Copy onto a full pad names the undo
- **WHEN** the owner places a copy on a pad with a sound
- **THEN** the question names the undo and does not say «This cannot be undone.»

### Requirement: Undo says what came back

After undo on Chop, Pads or A1, a new notice SHALL replace the current notice. It SHALL say what came back, in the words of the action. With nothing to undo, the undo cell SHALL be grey and a press SHALL say «nothing to undo». Redo SHALL stay inactive until change 93a.

#### Scenario: Undo after cut by hits names the old slices
- **WHEN** `cut by hits` made 9 slices from 5 and the owner presses undo
- **THEN** the chop has 5 slices and the notice says «undo: 5 slices back (were 9)»

#### Scenario: Undo after a join names the split
- **WHEN** the owner joins slices 1 and 2 and presses undo
- **THEN** the notice says «undo: slices 1 and 2 split again»

#### Scenario: Undo with nothing to undo is grey
- **WHEN** nothing can be undone and the owner holds PAD BANK on Chop
- **THEN** the undo cell is grey and a press says «nothing to undo»

### Requirement: P1 leaves the active mode first

PAD BANK with device pad 1 SHALL first leave the active mode: erase, copy, swap, a file in hand or marking. The screen SHALL stay. The next press SHALL go back as usual.

#### Scenario: P1 stops the A1 erase and stays on A1
- **WHEN** erase is on in A1 and the owner presses PAD BANK with device pad 1
- **THEN** erase is off, A1 stays on the screen and a notice says «erase off»

#### Scenario: P1 cancels a copy and stays on Pads
- **WHEN** a copy is in hand on Pads and the owner presses PAD BANK with device pad 1
- **THEN** the copy is cancelled and Pads stays on the screen

### Requirement: State cells show the current state

A cell whose action changes a state SHALL show the current state below its label, in a colour that differs from the label. At 120×40 the cell SHALL show the two label words on two lines and the state on the third line. At 80×26 the cell SHALL show the label on one line and the state on the second line. The state SHALL follow the current state at once. The state cells SHALL be the cells of the table in "State cells and their states".

#### Scenario: Play range shows its state
- **WHEN** the owner holds PAD BANK on Chop at 120×40 and presses `play range`
- **THEN** the cell shows «play», «range» and «to end» before the press and «to cut» after it

#### Scenario: Loop play shows its state at 80×26
- **WHEN** the owner holds PAD BANK on Chop at 80×26 with the loop off and presses `loop play`
- **THEN** the cell shows «loop play» on its first line and «off» before the press and «on» after it on its second line

### Requirement: State cells and their states

The panel SHALL show a state for exactly these cells, with these states.

| Cell | States |
| --- | --- |
| play range | to end, to cut, slice |
| loop play | on, off |
| cursor step | even, hits, cuts |
| hear as pad | on, off |
| play mode | 1 shot, gate |
| pitch gate | on, off |
| mark pad | on, off |
| hide empty | shown, hidden |
| listen file | on, off |
| star file | on, off |

#### Scenario: Every state cell shows one of its states
- **WHEN** the owner holds PAD BANK on each view with a state cell
- **THEN** each cell of the table shows one of its listed states, and no other cell shows a state

## MODIFIED Requirements

### Requirement: The screens view opens screens and global actions

Row 1 of the screens view SHALL show pads, files, chop and A1. Row 2 SHALL show fx, mixer, setup and help. Row 3 SHALL show save, open and exit.
Mixer SHALL stay dim until it exists. Help SHALL open Controller Help.
Global actions SHALL appear only in the screens view.
The screens view SHALL NOT show learn, because control bindings change from the keyboard only.

#### Scenario: Global actions live in the screens view
- **WHEN** the owner visits the screens view and every action page of each current view
- **THEN** save, open and exit appear only in the screens view
- **AND** no action page shows play, help or learn

### Requirement: Each view has at most 12 actions

A screen or mode SHALL have one action page with at most 12 actions in cells 1 through 12.
Cells 1–4 SHALL be device pads 13–16, cells 5–8 device pads 9–12 and cells 9–12 device pads 5–8.
An action that does not fit SHALL go to its own screen or mode.
The loader SHALL reject a second action page with a notice.
The shipped actions SHALL match the table. A dash is an empty cell.

#### Scenario: Shipped action pages match the table
- **WHEN** the layout test reads each view of the shipped panel file
- **THEN** each view has one page with the labels and actions of the table by device pad

#### Scenario: A second action page is rejected
- **WHEN** a panel file gives a view a second action page
- **THEN** a notice names the file and view, and the first page keeps its cells

| View | P5–P8 | P9–P12 | P13–P16 |
| --- | --- | --- | --- |
| Pads | edit pad, copy pad, chop pad, clear pad | swap pads, mark pad, mark bank, play mode | pitch gate, record output, hide pads, — |
| Pad edit | play mode, copy pad, pitch gate, reset pad | tone page, env page, filter page, source page | fx page, —, —, — |
| A1 | record A1, edit steps, erase hits, tap tempo | steps wider, steps finer, A1 setup, clear A1 | hide empty, —, —, — |
| Files | load to pad, take file, listen file, chop file | star file, —, —, — | —, —, —, — |
| Chop | cut here, cut by hits, fewer hits, equal parts | put on pads, fill pads, edit slice, trim silence | remove all cuts, loop play, play range, cursor step |
| Chop edit | norm, join left, join right, hear as pad | —, —, —, — | —, —, —, — |
| Settings | choose, —, —, — | —, —, —, — | —, —, —, — |

### Requirement: Shipped panel labels use short words

Every shipped panel label SHALL use approved words in one or two lines.
A label of two words SHALL show each word on its own line.
Each line SHALL use at most eight cells.
A one-line cell, at a small size, SHALL show the label on its one line.
A label SHALL have one meaning on every screen.
The label check SHALL include fixed cells and the shipped panel file.

#### Scenario: Shipped labels fit the panel glossary
- **WHEN** a shipped panel label line exceeds eight cells or uses an unknown word
- **THEN** the label check fails

#### Scenario: A two-word label uses two lines
- **WHEN** the panel shows `join left` at 80×26 and at 120×40
- **THEN** the cell shows «join» on its first line and «left» on its second line

#### Scenario: A short two-word label also uses two lines
- **WHEN** the panel shows `copy pad` at 80×26 and at 120×40
- **THEN** the cell shows «copy» on its first line and «pad» on its second line

### Requirement: Panel configuration names logical actions

`config/panel.toml` SHALL name action pages, starting at page 1, with short labels and logical action IDs. The loader SHALL reject cells outside 1 through 12, repeated cells, unknown actions and labels with a line wider than eight terminal cells. It SHALL name the file and cell in a notice. A bad cell SHALL stay inactive while valid cells and pad play continue. A valid cell SHALL use normal action dispatch and existing confirmation checks.

#### Scenario: Invalid panel cells do not disable pad play
- **WHEN** a panel file names an unknown action and repeats a cell
- **THEN** each bad cell is inactive and a notice names its file and cell
- **AND** a valid screen cell still acts and first-bank pads still sound

#### Scenario: A destructive cell cannot confirm itself
- **WHEN** a panel cell requests a destructive action with an occupied target or other required question
- **THEN** the normal visible question opens without changing the target yet
- **AND** the same press cannot answer or bypass that question

### Requirement: A hold in target mode shows the mode commands

During a PAD BANK hold with an item in hand or an open Assign plan, the panel SHALL show the commands of the mode. Device pad 1 SHALL cancel, pad 2 SHALL undo and pad 3 SHALL open the menu. Pad 4 SHALL apply an Assign plan. During Assign, the top row SHALL show the current method, `by hand` or `in order`. In order, pad 4 SHALL be grey until the owner hits the start pad, and a press SHALL say «hit the start pad first». Other cells SHALL be empty. With PAD BANK `Off`, the panel SHALL show the same commands, and a first-bank hit SHALL hide the panel and place the item.

#### Scenario: A hold in target mode shows only the mode commands
- **WHEN** an item is in hand and the owner holds PAD BANK
- **THEN** the panel shows cancel, undo and menu on the control row and the other cells are empty
- **AND** a release without a hit keeps the item in hand

#### Scenario: Assign shows its method
- **WHEN** an Assign plan is open by hand and the owner holds PAD BANK
- **THEN** the method cell says `by hand`, and after a press it says `in order`

#### Scenario: Apply waits for the start pad
- **WHEN** an Assign plan is open in order without a start pad and the owner presses pad 4
- **THEN** no slice is placed and a notice says «hit the start pad first»

### Requirement: Cells behind a question show only the answers

While a question is visible and the panel shows, device pad 1 SHALL read `no` and device pad 4 SHALL read `yes`. All other cells SHALL be empty.

#### Scenario: The panel behind a question shows no and yes
- **WHEN** a replace question opens from a panel cell
- **THEN** the panel shows `no` on pad 1, `yes` on pad 4 and no other label

### Requirement: Every mode shows its way out on a hold

A PAD BANK hold SHALL show a panel in every mode, also in Help, the project list, the folder list, learning and text entry. The panel SHALL show only the cells that act in that mode. Device pad 1 SHALL close or cancel the mode, and the screen SHALL stay. In Help, cells 9 and 10 SHALL move. In a list, device pad 4 SHALL open the selected entry. While learning, device pad 4 SHALL stay a control to learn, and Enter SHALL save a pending choice. Other cells SHALL be empty.

#### Scenario: A hold in Help shows the close cell
- **WHEN** Help shows and the owner holds PAD BANK
- **THEN** the panel shows close on device pad 1 and up and down on cells 9 and 10
- **AND** device pad 1 closes Help

#### Scenario: No mode is a dead end for the SMC-PAD
- **WHEN** the owner enters each current mode in separate runs and holds PAD BANK
- **THEN** the panel shows a label on device pad 1
- **AND** device pad 1 during the hold leaves that mode or closes its question

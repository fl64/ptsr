# command-panel Specification

## Purpose
Provide SMC-PAD commands through the second pad bank while the first bank continues to play samples.

## Requirements

### Requirement: SMC-PAD second bank opens a command panel

The Master profile SHALL map 16 second-bank notes to panel cells in the first bank's physical rows. An owner binding MAY replace a cell outside a question. The first panel-bound press while hidden SHALL reveal the panel without an action. Later presses SHALL run one visible cell. Releases SHALL run no cell. A first-bank press SHALL hide the panel and keep its pad action and original release. PAD BANK SHALL require no host message.

#### Scenario: First second-bank hit only reveals the panel
- **WHEN** the owner starts transport and sends a second-bank press and release with the shipped panel binding while the panel is hidden
- **THEN** the panel appears in the pad-grid area and no cell action runs
- **AND** transport and sounding voices continue without a new hit or stop

#### Scenario: Two second-bank presses keep input order
- **WHEN** two second-bank presses reach the MIDI dispatcher before one UI redraw
- **THEN** the first reveals the panel without action and the second runs its visible cell once
- **AND** either release runs no cell

#### Scenario: First-bank hit leaves the panel and keeps its Gate
- **WHEN** the panel is visible and a first-bank Gate pad is pressed, then released after a bank or screen change
- **THEN** the press hides the panel and sounds once at its normal physical position
- **AND** the original release ends that Gate once without releasing another pad

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

### Requirement: Visible questions use the two agreed pads

While a question is visible, physical P7 SHALL cancel and P8 SHALL confirm on press, even with owner bindings and a hidden panel. Other second-bank cells SHALL not reveal or act. PLAY and STOP SHALL never answer the question, even with `ui.confirm` bound. Their context transport roles MAY act while the question stays visible. Keyboard and learned confirmation paths SHALL keep their rules.

#### Scenario: P7 and P8 answer a visible question on the first hit
- **WHEN** a question is visible and the owner presses P7 or P8 from a hidden panel in separate runs
- **THEN** P7 cancels and P8 confirms through the normal question action
- **AND** neither press first reveals the panel or acts as an ordinary cell

#### Scenario: A panel action followed by P8 confirms only the question
- **WHEN** a destructive panel cell opens a question and P8 arrives before the next redraw
- **THEN** P8 confirms only that visible question once
- **AND** no earlier cell action repeats and no second target changes

#### Scenario: PLAY and STOP do not answer a question
- **WHEN** a question is visible and PLAY or STOP sends a tap
- **THEN** the context playback action runs where available
- **AND** the question and its target stay unchanged

### Requirement: Panel state belongs to its controller

The panel SHALL belong to the Master connection that revealed it. Another controller SHALL keep its own pads, buttons and held releases. Disconnect SHALL clear the SMC-PAD panel and its mark. Reconnect SHALL start hidden. Invalid shared MIDI state SHALL reject new panel presses but preserve earlier accepted Gate releases.

#### Scenario: Two controllers keep separate routes during panel use
- **WHEN** the SMC-PAD panel is visible and an X-Session control acts between SMC-PAD cell presses
- **THEN** the X-Session keeps its current action and no panel cell runs from that input
- **AND** held pads on either connection keep their own release target

#### Scenario: Disconnect clears the panel without a stuck Gate
- **WHEN** the SMC-PAD disconnects while its panel shows and one of its Gate pads is held
- **THEN** the panel and mark disappear and that Gate receives its original release once
- **AND** another device's held control remains active and a reconnect starts with a hidden panel

### Requirement: Configured symmetric pairs share a panel row

The shipped panel map SHALL identify both roles of each symmetric action pair.
The lower, earlier or negative role SHALL be left of its neighbor.
The layout check SHALL reject a split or reversed pair.

#### Scenario: Layout rejects a split or reversed panel pair
- **WHEN** one member of a shipped panel pair moves to a different row or the pair reverses
- **THEN** the layout check reports that the positions are invalid

### Requirement: Shipped panel labels use short words

Every shipped panel label SHALL use an approved label of at most six cells.
The label check SHALL include fixed cells and the shipped panel file.

#### Scenario: Shipped labels fit the panel glossary
- **WHEN** a shipped panel label exceeds six cells or uses an unknown word
- **THEN** the label check fails

### Requirement: SMC-PAD target cells select pads on the current layer

A visible target view SHALL use cells 1 to 12 as pad positions in the current bank.
It SHALL show that pads 13–16 use the playing layer.
Cell 13 SHALL cancel. Cell 14 SHALL keep undo. Cell 15 SHALL switch targets and commands.
Cell 16 SHALL apply Assign. Questions, ownership and original held releases SHALL keep priority.
During Assign, the Chop actions SHALL show `order` in cell 4 instead of `assign`.

#### Scenario: Panel target pads place items without a layer switch
- **WHEN** the owner takes a pad, a copy or a file and selects cells 1 through 12 on the visible target view
- **THEN** the matching pad gets the existing placement result without a layer change
- **AND** a replacement question stays available to cancel or accept

#### Scenario: Assign applies from the target row
- **WHEN** the owner selects Assign targets on the visible command layer or playing layer
- **THEN** the plan keeps those targets and cell 16 applies it on the visible target view
- **AND** cell 13 cancels, cell 14 stays undo and cell 15 gives access to the other commands

#### Scenario: Assign shows order in the Chop actions
- **WHEN** the owner opens the menu from the Assign target view
- **THEN** cell 4 shows `order` and switches the plan as Tab does on the keyboard

#### Scenario: Target cells retain connection and release priority
- **WHEN** another Master sends a target hit or a held layer 1 pad releases during target selection
- **THEN** only the owner can choose a target and the original held pad releases

### Requirement: Panel views move with LEFT and RIGHT

The panel SHALL have two views: screens on the left and the actions of the current view on the right.
A reveal SHALL show the actions. LEFT SHALL show the screens view and RIGHT SHALL show the actions.
A screen change SHALL show the actions of the new screen.
A question or the target view SHALL keep its cells, and LEFT and RIGHT SHALL do nothing.

#### Scenario: The panel opens on the screen actions
- **WHEN** the owner reveals the panel on Chop
- **THEN** the cells show the Chop actions
- **AND** the header names the screens view on the left

#### Scenario: LEFT and RIGHT move through the views
- **WHEN** the owner presses LEFT and RIGHT on the visible panel of Pads
- **THEN** LEFT shows the screens view and RIGHT shows the actions again
- **AND** a second press in the same direction keeps the current view

#### Scenario: A screen hit opens that screen with its actions
- **WHEN** the owner hits Files in the screens view
- **THEN** Files opens and the panel shows the Files actions

### Requirement: The screens view opens screens and global actions

The screens view SHALL show pads, files, chop and A1 in row 1; fx, mixer, setup and help in row 2; save, open, learn and exit in row 3.
Mixer SHALL stay dim until it exists. Help SHALL open Controller Help.
Global actions SHALL appear only in the screens view.

#### Scenario: Global actions live in the screens view
- **WHEN** the owner visits the screens view and every action page of each current view
- **THEN** save, open, learn and exit appear only in the screens view
- **AND** no action page shows play or help

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
| Pads | edit, copy, take, mark, mode, clear, gate, marks, grid, output |
| Pad edit | mode, clear, gate, copy |
| A1 | step, grid -, grid +, knobs, record, erase, undo, clear, rows, tap |
| Files | take, load, parent, star, enter, auto, chop |
| Chop | cut, parts, hits, assign, loop, listen, snap, as pad, start, end, edit, commit |
| Chop edit | level, hits -, cut -, end -, whole |
| Settings | tab, learn, clear, —, row -, row + |

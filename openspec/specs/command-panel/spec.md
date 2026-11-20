# command-panel Specification

## Purpose
Provide SMC-PAD commands through the second pad bank while the first bank continues to play samples.

## Requirements

### Requirement: SMC-PAD second bank opens a command panel

Without the PAD BANK hold shortcut, the Master profile SHALL map 16 second-bank notes to panel cells in physical rows. An owner binding MAY replace a cell outside a question. A first panel-bound press while hidden SHALL only reveal the panel. Later presses SHALL run one visible cell. Releases SHALL run no cell. A first-bank press SHALL hide the panel and keep its pad action and release. With the shortcut on, the hold SHALL decide.

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
Its bottom row SHALL be the control row on every view: device pad 1 back, pad 2 undo,
pad 3 redo and pad 4 done. Redo SHALL be dim and inactive until change 93a.
Pad 4 SHALL be dim when the view has nothing to confirm or apply.
The panel SHALL NOT show Panic.

#### Scenario: Fixed cells stay on the bottom row
- **WHEN** the owner changes panel views or screens
- **THEN** device pads 1 through 4 keep back, undo, redo and done in physical left-to-right order
- **AND** Redo is dim and runs no action, and no cell shows Panic

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

Every shipped panel label SHALL use an approved label of at most seven cells.
A label SHALL have one meaning on every screen.
The label check SHALL include fixed cells and the shipped panel file.

#### Scenario: Shipped labels fit the panel glossary
- **WHEN** a shipped panel label exceeds seven cells or uses an unknown word
- **THEN** the label check fails

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
| Pads | edit, copy, swap, mark, mode, remove, p.gate, all, grid, rec out, chop |
| Pad edit | mode, reset, p.gate, copy |
| A1 | step, grid -, grid +, params, record, erase, undoA1, clear, rows, tap |
| Files | to pad, load, —, star, —, listen, chop |
| Chop | cut, parts, hits, assign, loop, listen, snap, as pad, whole, —, edit, fill |
| Chop edit | normal, hits -, cut -, end -, whole |
| Settings | —, learn, unbind |

### Requirement: Visible questions answer with pads 1 and 4

While a question is visible, device pad 1 SHALL cancel and device pad 4 SHALL confirm on press. This SHALL apply with owner bindings and a hidden panel. Both pads SHALL answer from either note bank without a hold. Other pads SHALL stay silent. PLAY and STOP SHALL never answer and MAY keep their transport roles. A release SHALL NOT answer. Keyboard answers SHALL keep their rules.

#### Scenario: Pad 1 and pad 4 answer a visible question on the first hit
- **WHEN** a question is visible and the owner presses device pad 1 or pad 4 from a hidden panel in separate runs
- **THEN** pad 1 cancels and pad 4 confirms through the normal question action
- **AND** neither press first reveals the panel or acts as an ordinary cell

#### Scenario: A question takes pads 1 and 4 without a hold
- **WHEN** a question is visible, the hold shortcut is on and PAD BANK is not held
- **THEN** device pads 1 and 4 from either note bank answer the question
- **AND** a hit on another pad stays silent and the question stays

#### Scenario: A panel action followed by pad 4 confirms only the question
- **WHEN** a destructive panel cell opens a question and device pad 4 arrives before the next redraw
- **THEN** pad 4 confirms only that visible question once
- **AND** no earlier cell action repeats and no second target changes

#### Scenario: PLAY and STOP do not answer a pad 1 and pad 4 question
- **WHEN** a question is visible and PLAY or STOP sends a tap
- **THEN** the context playback action runs where available
- **AND** the question and its target stay unchanged

#### Scenario: A held pad 4 release does not answer
- **WHEN** the owner holds device pad 4 as a Gate pad, a question opens and the owner releases the pad
- **THEN** the Gate sound ends once and the question stays open

### Requirement: Placement targets use the playing layer

An item in hand is a file, a recording, a copy or a swap. With an item in hand or an open Assign plan, a first-layer pad hit without a hold SHALL place the item on that pad. The existing placement result and questions SHALL apply. All 16 pads of the current bank SHALL be targets. Without a hold, LEFT and RIGHT SHALL change the target bank. Panel commands SHALL belong to the connection that owns the panel. Original held releases SHALL keep priority.

#### Scenario: All sixteen pads take an item in hand
- **WHEN** a file is in hand and the owner hits device pad 1, then takes another file and hits device pad 16
- **THEN** each pad gets its file with the existing placement result and no layer change

#### Scenario: Assign places slices on the playing layer and applies on a hold
- **WHEN** the owner opens Assign, hits device pads 1 and 16 without a hold, holds PAD BANK and presses pad 4
- **THEN** the plan places the next two slices on those pads and applies once
- **AND** pad 1 during the hold cancels the plan instead in a separate run

#### Scenario: LEFT and RIGHT change the Assign bank
- **WHEN** the owner opens Assign, releases PAD BANK and presses RIGHT
- **THEN** the next bank shows and a pad hit plans the next slice on that bank

#### Scenario: Target pads keep their connection and release
- **WHEN** another SMC-PAD sends a panel cancel or a held first-layer pad releases during target choice
- **THEN** the item stays in hand and the original held pad releases
- **AND** a later pad hit places the item as before

### Requirement: A hold in target mode shows the mode commands

During a PAD BANK hold with an item in hand or an open Assign plan, the panel SHALL show the commands of the mode. Device pad 1 SHALL cancel, pad 2 SHALL undo and pad 3 SHALL open the menu. Pad 4 SHALL apply an Assign plan. During Assign, the top row SHALL show `order`. Other cells SHALL be dim. With PAD BANK `Off`, the panel SHALL show the same commands, and a first-bank hit SHALL hide the panel and place the item.

#### Scenario: A hold in target mode shows only the mode commands
- **WHEN** an item is in hand and the owner holds PAD BANK
- **THEN** the panel shows cancel, undo and menu on the control row and dims the other cells
- **AND** a release without a hit keeps the item in hand

### Requirement: The panel title keeps the screen name

While the panel shows, the title SHALL keep the current screen name, and the header above the cells SHALL name the panel view. Badges, the BPM and the mode line SHALL keep their own columns. Text that does not fit SHALL be cut at its end and SHALL NOT overwrite other text. While the panel shows, the LEFT and RIGHT hint SHALL name the panel views.

#### Scenario: The panel title names the screen without overlap
- **WHEN** the panel shows on Chop with LISTEN on, and on Files with a file in hand, at both supported sizes
- **THEN** the title names CHOP or FILES and each badge reads in full or cut at its end
- **AND** the LEFT and RIGHT hint names the panel views

### Requirement: Cells behind a question show only the answers

While a question is visible and the panel shows, device pad 1 SHALL read `no` and device pad 4 SHALL read `yes`. All other cells SHALL be dim and empty.

#### Scenario: The panel behind a question shows no and yes
- **WHEN** a replace question opens from a panel cell
- **THEN** the panel shows `no` on pad 1, `yes` on pad 4 and no other label

### Requirement: Every mode shows its way out on a hold

A PAD BANK hold SHALL show a panel in every mode, also in Help, the project list, the folder list, learning and text entry. The panel SHALL show only the cells that act in that mode. Device pad 1 SHALL close or cancel the mode. In Help, cells 9 and 10 SHALL move. In a list, device pad 4 SHALL open the selected entry. While learning with a pending choice, device pad 4 SHALL save it. Other cells SHALL be dim.

#### Scenario: A hold in Help shows the close cell
- **WHEN** Help shows and the owner holds PAD BANK
- **THEN** the panel shows close on device pad 1 and up and down on cells 9 and 10
- **AND** device pad 1 closes Help

#### Scenario: No mode is a dead end for the SMC-PAD
- **WHEN** the owner enters each current mode in separate runs and holds PAD BANK
- **THEN** the panel shows a label on device pad 1
- **AND** device pad 1 during the hold leaves that mode or closes its question

### Requirement: A learned panel cell shows its new action

When the owner binds an action to a SMC-PAD panel cell, the panel SHALL show that action on the cell instead of the shipped label. The learning prompt SHALL name the cell by the number printed on the device.

#### Scenario: A cell bound by learning shows the learned action
- **WHEN** the owner binds pad mode to the panel cell of device pad 6 and shows the screens view
- **THEN** the cell shows the pad mode label and no longer shows open

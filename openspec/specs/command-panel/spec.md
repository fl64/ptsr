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
pad 3 redo and pad 4 done. Redo SHALL be dim when no applicable step waits.
Pad 4 SHALL be dim when the view has nothing to confirm or apply.
The panel SHALL NOT show Panic.

#### Scenario: Fixed cells stay on the bottom row
- **WHEN** the owner changes panel views or screens
- **THEN** device pads 1 through 4 keep back, undo, redo and done in physical left-to-right order
- **AND** P3 runs Redo when a step waits, and no cell shows Panic

#### Scenario: Cells show the device numbers
- **WHEN** the panel shows at either supported size
- **THEN** the bottom-left cell shows 1 and the top-right cell shows 16

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
| Pads | edit pad, copy pad, chop pad, clear pad | swap pads, mark pad, mark bank, play mode | pitch gate, record output, hide pads, strip view |
| Pad edit | play mode, copy pad, pitch gate, reset pad | tone page, env page, filter page, source page | fx page, —, —, pin sample |
| A1 | record A1, edit steps, erase hits, tap tempo | steps wider, steps finer, A1 setup, clear A1 | hide empty, —, —, — |
| Files | load to pad, take file, listen file, chop file | star file, —, —, — | —, —, —, — |
| Chop | cut here, cut by hits, fewer hits, equal parts | put on pads, fill pads, edit slice, trim silence | remove all cuts, loop play, play range, cursor step |
| Chop edit | norm, join left, join right, hear as pad | —, —, —, — | —, —, —, — |
| Settings | choose, —, —, — | —, —, —, — | —, —, —, — |

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

During a PAD BANK hold with an item in hand or an open Assign plan, the panel SHALL show the commands of the mode. Device pad 1 SHALL cancel, pad 2 SHALL undo and pad 3 SHALL open the menu. Other cells SHALL be empty. In that menu, P3 SHALL run Redo and P1 SHALL return to the targets. With PAD BANK `Off`, the panel SHALL show the same commands, and a first-bank hit SHALL hide the panel and place the item.

#### Scenario: A hold in target mode shows only the mode commands
- **WHEN** an item is in hand and the owner holds PAD BANK
- **THEN** the panel shows cancel, undo and menu on the control row and the other cells are empty
- **AND** a release without a hit keeps the item in hand

### Requirement: Assign shows its method and waits for the start pad

During a PAD BANK hold with an open Assign plan, device pad 4 SHALL apply the plan. The top row SHALL show the current method, `by hand` or `in order`. In order, pad 4 SHALL be grey until the owner hits the start pad, and a press SHALL say «hit the start pad first».

#### Scenario: Assign shows its method
- **WHEN** an Assign plan is open by hand and the owner holds PAD BANK
- **THEN** the method cell says `by hand`, and after a press it says `in order`

#### Scenario: Apply waits for the start pad
- **WHEN** an Assign plan is open in order without a start pad and the owner presses pad 4
- **THEN** no slice is placed and a notice says «hit the start pad first»

### Requirement: The panel title keeps the screen name

While the panel shows, the title SHALL keep the current screen name, and the header above the cells SHALL name the panel view. Badges, the BPM and the mode line SHALL keep their own columns. Text that does not fit SHALL be cut at its end and SHALL NOT overwrite other text. While the panel shows, the LEFT and RIGHT hint SHALL name the panel views.

#### Scenario: The panel title names the screen without overlap
- **WHEN** the panel shows on Chop with LISTEN on, and on Files with a file in hand, at both supported sizes
- **THEN** the title names CHOP or FILES and each badge reads in full or cut at its end
- **AND** the LEFT and RIGHT hint names the panel views

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

### Requirement: A learned panel cell shows its new action

When the owner binds an action to a SMC-PAD panel cell, the panel SHALL show that action on the cell instead of the shipped label. The learning prompt SHALL name the cell by the number printed on the device.

#### Scenario: A cell bound by learning shows the learned action
- **WHEN** the owner binds pad mode to the panel cell of device pad 6 and shows the screens view
- **THEN** the cell shows the pad mode label and no longer shows open

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

After undo on Chop, Pads or A1, a new notice SHALL replace the current notice. It SHALL say what came back, in the words of the action. With nothing to undo, the undo cell SHALL be grey and a press SHALL say «nothing to undo».

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

The panel SHALL show a state for these cells only.

| Cell | States |
| --- | --- |
| play range | to end, to cut, slice |
| loop play | on, off |
| cursor step | even, hits, cuts |
| hear as pad | on, off |
| play mode | 1 shot, gate |
| pitch gate | on, off |
| mark pad | on, off |
| strip view | Sound, Rhythm |
| pin sample | on, off |
| hide empty | shown, hidden |
| listen file | on, off |
| star file | on, off |

#### Scenario: Every state cell shows one of its states

- **WHEN** the owner holds PAD BANK on each view with a state cell
- **THEN** each cell of the table shows one of its listed states, and no other cell shows a state

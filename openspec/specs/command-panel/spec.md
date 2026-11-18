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

The panel SHALL draw 16 cells in the same top-to-bottom order as the physical pads. Its bottom row SHALL keep Back, Undo, Redo and Panic in cells 13 through 16. Redo and Panic SHALL be dim and inactive until change 93a.

#### Scenario: Fixed cells stay on the bottom row
- **WHEN** the owner changes panel pages or screens
- **THEN** cells 13 through 16 keep Back, Undo, Redo and Panic in physical left-to-right order
- **AND** Redo and Panic are dim and run no action

### Requirement: Panel pages expose screen and context commands

The main page SHALL offer Pads, Files, Chop, Pattern, FX, Settings, Help, More
and up to four screen actions. Unused cells SHALL stay dim.
More SHALL open context pages with up to ten actions each.
Cell 11 (Next) SHALL cycle them, and cell 12 (Main) SHALL return.
It SHALL show the page and count. A screen change or exit SHALL select Main.
LEFT and RIGHT SHALL keep their screen list.
Direct controls SHALL replace value, parameter and edit-page cells.
Keyboard and learn actions SHALL remain.

#### Scenario: Screen and context pages cover more than 16 commands
- **WHEN** the owner selects More, cycles through context pages and presses Main
- **THEN** the pages show their commands in physical order
- **AND** the page move retains the fixed bottom row, count and transport

#### Scenario: A screen switch resets the panel page
- **WHEN** a visible context page is open and the owner changes screen
- **THEN** the new screen shows the main page with its own cells
- **AND** LEFT and RIGHT move the current screen list

#### Scenario: Main cells follow the current view
- **WHEN** the owner visits the main page of each current view
- **THEN** cells 9 through 12 show the available actions for that view
- **AND** unused cells stay dim and change no value

#### Scenario: Direct page controls replace panel steps
- **WHEN** the owner changes an editor's page, parameter or value through the direct controls
- **THEN** the shipped panel omits the replaced steps
- **AND** the keyboard and learn actions remain available

### Requirement: Panel configuration names logical actions

`config/panel.toml` SHALL name current-screen and mode cells with short labels and logical action IDs. The loader SHALL reject cells outside 1 through 12, repeated cells and unknown actions. It SHALL also reject labels wider than six terminal cells and attempts to replace the fixed bottom row. It SHALL name the file and cell in a notice. A bad cell SHALL stay inactive while valid cells and pad play continue. A valid cell SHALL use normal action dispatch and existing confirmation checks.

#### Scenario: Invalid panel cells do not disable pad play
- **WHEN** a panel file names an unknown action and repeats a fixed cell
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

# command-panel Specification

## ADDED Requirements

### Requirement: SMC-PAD second bank opens a command panel

The measured Master profile SHALL route its 16 second-bank notes to the command panel in the same physical rows as the first bank. The first second-bank press while the panel is hidden SHALL show the panel and run no cell. Later presses SHALL run one visible cell each. Releases SHALL run no cell. A first-bank press SHALL hide the panel and keep its normal pad action and original release. PAD BANK itself SHALL require no host message.

#### Scenario: First second-bank hit only reveals the panel
- **WHEN** the owner starts transport and sends a second-bank press and release while the panel is hidden
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

The main page SHALL offer Pads, Files, Chop, Pattern, FX, Settings, Help and More, plus four current-screen actions. More SHALL open a context page. Each context page SHALL have up to ten actions, Next in cell 11 and Main in cell 12. Next SHALL cycle through context pages. The active page and count SHALL stay visible. A screen change or panel exit SHALL select the main page again. LEFT and RIGHT SHALL keep their current item-selection roles.

#### Scenario: Screen and context pages cover more than 16 commands
- **WHEN** the owner opens the panel, selects More, cycles through two context pages and returns with Main
- **THEN** each page shows its current-screen actions in the same physical cell order
- **AND** the bottom row stays fixed, the page count stays visible, and no page change stops transport

#### Scenario: A screen switch resets the panel page
- **WHEN** the context page is visible and a screen cell or the keyboard changes screens
- **THEN** the new screen shows the panel main page with its own four current-screen actions
- **AND** LEFT and RIGHT still select items instead of changing panel pages

#### Scenario: Main cells follow the current view
- **WHEN** the owner visits Pads, pad edit, Files, Chop, A1 overview, FX and Settings with the panel visible
- **THEN** cells 9 through 12 show and run the actions in the change design for that view
- **AND** an unavailable action stays dim without changing another view

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

While a question is visible, the SMC-PAD's physical P7 (cell 7) SHALL cancel it and P8 (cell 8) SHALL confirm it on press. These answers SHALL work when the panel was hidden before that press. Other second-bank cells SHALL not reveal the panel, answer the question or run background commands. PLAY and STOP SHALL keep their context transport roles and SHALL not answer the question. Keyboard and learned confirmation paths SHALL keep their current rules.

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

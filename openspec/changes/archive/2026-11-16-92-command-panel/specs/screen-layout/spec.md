# screen-layout Delta

## MODIFIED Requirements

### Requirement: SMC-PAD controller view follows the physical surface

The F1 controller view SHALL show the first hardware pad bank in the same top-to-bottom order as the screen pads. It SHALL show the second hardware pad bank as panel cells in the same physical rows. It SHALL show both sets of eight encoders and the five buttons in their physical groups. The view SHALL name each available current role and dim a control without a role. It SHALL not show SHIFT, PAD BANK or KNOB BANK as host commands.

#### Scenario: Controller view matches both pad banks
- **WHEN** the owner opens the SMC-PAD controller view and switches the local PAD BANK
- **THEN** the view identifies notes 48 through 51 as the top row and notes 36 through 39 as the bottom row of the first bank
- **AND** the second bank has the same physical row order and shows the current panel cell labels

#### Scenario: Encoder roles follow the current screen
- **WHEN** the owner views Pads, Files and Chop with the SMC-PAD selected
- **THEN** both encoder sets name the available roles for that screen
- **AND** unavailable roles are dim, while the controller status shows when encoder setup is incomplete

## ADDED Requirements

### Requirement: Panel replaces the pad grid without moving the frame

At 80 by 26 and 120 by 40, the command panel SHALL draw its 4 by 4 cell grid in the existing pad-grid rectangle. It SHALL keep the title, strip, bank row, right column, three-row help and status in their current positions. Cells SHALL use the same physical top-to-bottom order as first-bank pads. A cell label SHALL fit six terminal cells. The panel SHALL remain readable with kitty graphics on or off. It SHALL not conceal a question, text line or safety warning.

#### Scenario: Panel grid fits both supported sizes
- **WHEN** the panel shows its main and context pages at 80 by 26 and 120 by 40
- **THEN** all 16 cells, page title, fixed bottom row and safety indicators fit inside their existing areas
- **AND** no cell is obscured by a pad-wave image and leaving the panel restores the current wave

### Requirement: Panel state and unavailable cells stay visible

The title SHALL show `PANEL · SMC` for the active SMC-PAD connection. The status SHALL name the current page after a second-layer press. A new page or screen SHALL update the labels before the next cell acts. Disabled Redo, Panic and unsupported context cells SHALL appear dim with no active-action claim. A first reveal SHALL not show an action result. A disconnect or first-bank hit SHALL remove the panel mark.

#### Scenario: Panel feedback follows the active connection and page
- **WHEN** the owner reveals the panel, changes its page and then disconnects the SMC-PAD
- **THEN** the badge and labels follow that connection and page, then disappear on disconnect
- **AND** Redo and Panic remain visibly unavailable and no first-reveal command result appears

### Requirement: Panel catalogue includes its states

The offline screen catalogue SHALL cover panel main, context, question, unavailable-cell and disconnect states at both supported terminal sizes. It SHALL cover kitty images enabled and disabled where the grid intersects images. Missing entries SHALL fail the catalogue test. Generation SHALL not open audio or MIDI devices or edit a project.

#### Scenario: Catalogue covers panel pages and safety
- **WHEN** `task screens` generates the catalogue
- **THEN** both sizes include the panel pages, question priority and safety states
- **AND** a missing required entry fails coverage without device access

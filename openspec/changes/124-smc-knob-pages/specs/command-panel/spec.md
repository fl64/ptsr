## ADDED Requirements

### Requirement: Configured symmetric pairs share a panel row

The shipped panel map SHALL identify both roles of each symmetric action pair.
The lower, earlier or negative role SHALL be left of its neighbor.
The layout check SHALL reject a split or reversed pair.

#### Scenario: Layout rejects a split or reversed panel pair
- **WHEN** one member of a shipped panel pair moves to a different row or the pair reverses
- **THEN** the layout check reports that the positions are invalid

## MODIFIED Requirements

### Requirement: Panel pages expose screen and context commands

The main page SHALL offer Pads, Files, Chop, Pattern, FX, Settings, Help and More.
It SHALL provide up to four current-screen actions. Unused cells SHALL stay dim.
More SHALL open the first context page. A context page SHALL offer up to ten actions.
Next SHALL cycle context pages from cell 11. Main SHALL return from cell 12.
The program SHALL show the page and count.
A screen change or panel exit SHALL select the main page again.
LEFT and RIGHT SHALL retain their current screen list.
Direct controls SHALL replace shipped value, parameter and edit-page cells.
Their keyboard and MIDI learn actions SHALL remain available.

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

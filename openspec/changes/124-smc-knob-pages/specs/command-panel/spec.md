## ADDED Requirements

### Requirement: Shipped control pairs share a physical row

The shipped knob page tables and panel map SHALL identify both roles in a symmetric pair.
Each pair SHALL use adjacent cells in one physical row.
The lower, earlier or negative role SHALL be on the left.
The layout check SHALL cover slice bounds, region bounds, FX sends and other named pairs.
It SHALL cover No and Yes on P7 and P8, and Undo and Redo on the bottom row.

#### Scenario: Layout rejects a split or reversed pair
- **WHEN** a test puts one member of a shipped pair on another row or reverses the roles
- **THEN** the layout check reports the group and both positions

## MODIFIED Requirements

### Requirement: Panel pages expose screen and context commands

The main page SHALL offer Pads, Files, Chop, Pattern, FX, Settings, Help and More.
It SHALL provide up to four current-screen actions. Unused cells SHALL stay dim.
More SHALL open the first context page. Each context page SHALL have up to ten actions.
It SHALL keep Next in cell 11 and Main in cell 12.
Next SHALL cycle through context pages. The program SHALL show the page and count.
A screen change or panel exit SHALL select the main page again.
LEFT and RIGHT SHALL retain the current screen or mode list.
The shipped page SHALL remove value, parameter and edit-page cells that direct controls replace.
The program SHALL keep their keyboard and MIDI learn actions.

#### Scenario: Screen and context pages cover more than 16 commands
- **WHEN** the owner selects More, cycles through context pages and presses Main
- **THEN** each page shows its commands in the physical cell order
- **AND** the page move retains the fixed bottom row, count and transport

#### Scenario: A screen switch resets the panel page
- **WHEN** a visible context page is open and the owner changes screen
- **THEN** the new screen shows its main page and current cells
- **AND** LEFT and RIGHT move the current screen list

#### Scenario: Main cells follow the current view
- **WHEN** the owner visits Pads, pad edit, Files, Chop, Pattern, FX and Settings
- **THEN** cells 9–12 show and run the available actions for that view
- **AND** unused cells stay dim and change no value

#### Scenario: Direct page controls replace panel steps
- **WHEN** the owner opens an editor and changes its page, parameter and value
- **THEN** the buttons and knobs provide the direct routes
- **AND** the shipped panel omits the replaced steps
- **AND** the keyboard and learn actions remain available

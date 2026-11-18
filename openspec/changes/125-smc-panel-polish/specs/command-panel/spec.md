## MODIFIED Requirements

### Requirement: Panel pages expose screen and context commands

The main page SHALL show Pads, Files, Chop and A1 on row 1.
It SHALL show FX, Mixer, Setup and More on row 2.
Mixer SHALL stay dim until the mixer screen is available.
The main page SHALL have up to four current-view actions.
More SHALL open page 1. Help SHALL use P10 on that page in each view.
Next SHALL cycle context pages from P11. Main SHALL return from P12.
The panel SHALL show the current page and count.
A screen change or panel exit SHALL restore the main page.
LEFT and RIGHT SHALL keep the current list. Keyboard paths SHALL remain available.

#### Scenario: Screen and context pages cover more than 16 commands
- **WHEN** the owner selects More, cycles through context pages and presses Main
- **THEN** the pages show their commands in physical order
- **AND** the page move retains the fixed bottom row, count and transport

#### Scenario: A screen switch resets the panel page
- **WHEN** the owner changes screen from a visible context page
- **THEN** the new screen shows its main page and its own cells
- **AND** LEFT and RIGHT move the current list

#### Scenario: Main cells follow the current view
- **WHEN** the owner visits the main page of each current view
- **THEN** cells 9 through 12 show the available actions for that view
- **AND** unused cells stay dim and change no value

#### Scenario: Direct page controls replace panel steps
- **WHEN** the owner changes an editor's page, parameter or value through the direct controls
- **THEN** the shipped panel omits the replaced steps
- **AND** the keyboard and learn actions remain available

#### Scenario: The main panel gives Help one context route
- **WHEN** the owner uses the shipped main panel in each view
- **THEN** Mixer is dim and Settings uses P7
- **AND** More and P10 open Help

## ADDED Requirements

### Requirement: Shipped panel labels use short words

Every shipped panel label SHALL use an approved label of at most six cells.
The label check SHALL include fixed cells and the shipped panel file.

#### Scenario: Shipped labels fit the panel glossary
- **WHEN** a shipped panel label exceeds six cells or uses an unknown word
- **THEN** the label check fails

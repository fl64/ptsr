## ADDED Requirements

### Requirement: Shared FX screen

The system SHALL show both shared FX blocks on a separate screen and visibly identify
the selected block. Each block SHALL show its effect type and the parameters of that
type. The same three learned knobs SHALL control the type and two parameters of the
selected block. Switching blocks SHALL NOT change either effect. Each device SHALL use
its configured takeover mode for each block and control independently. Changes SHALL be
smooth and saved in the project; existing projects and learned bindings SHALL retain
their behavior. `Ctrl+Z` SHALL undo a shared effect change.

#### Scenario: Select the next shared effect
- **WHEN** FX 1 is selected and the owner presses Tab
- **THEN** FX 2 is selected and neither effect changes

#### Scenario: The same knobs control the selected effect
- **WHEN** the owner changes FX 1 with the three page knobs, selects FX 2 and turns them
- **THEN** only FX 2 changes after the selection

#### Scenario: Takeover is separate for shared effects
- **WHEN** a controller uses pickup, the owner changes FX 1, selects FX 2 and moves a knob
- **THEN** FX 2 waits for its own value before the knob changes it

#### Scenario: Undo a shared effect on the FX screen
- **WHEN** the owner changes a shared effect and presses Ctrl+Z
- **THEN** the previous effect setting returns

#### Scenario: Compact shared effect cards
- **WHEN** the owner opens F4 in a large or small terminal
- **THEN** each effect card contains only its title and three parameter rows
- **AND** the cards are beside each other when they fit, or stacked in a narrow terminal
- **AND** neither card expands to fill unused rows

#### Scenario: Clear a shared effect knob assignment
- **WHEN** the owner clears a page knob assignment on F4 and restarts the program
- **THEN** the cleared knob does not control the effect and no old assignment returns

#### Scenario: Replaced page knobs stay replaced
- **WHEN** an old pad edit page knob has a replacement and the owner opens F4
- **THEN** the replacement controls the effect and the previous knob stays inactive

## MODIFIED Requirements

### Requirement: The pad panel in blocks
The system SHALL group the parameter rows of the pad panel by page, in the page order
TONE, ENV, FILTER, SOURCE, FX: every block stands under a thin line with the page name,
the line of the page the edit is on is lit. `Tab` turns the pages in this order, `I` `K` walk
the parameters in this order, and the frame title names the pages in this order.

#### Scenario: Blocks
- **WHEN** the pad edit is open
- **THEN** the panel shows the lines TONE, ENV, FILTER, SOURCE, FX in this order, each over its rows

#### Scenario: SOURCE last
- **WHEN** the edit is on the reso of the FILTER page and the owner presses `K`
- **THEN** the source is picked, on the SOURCE page; `K` after the end of SOURCE picks the
  drive on FX, and `K` after the last parameter of FX goes back to the volume


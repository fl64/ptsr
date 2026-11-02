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

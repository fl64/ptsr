# pad-playback Delta

## MODIFIED Requirements

### Requirement: Shared FX screen

A separate FX screen SHALL show both shared blocks, their types and parameters, and identify the selected block.
The screen SHALL draw both blocks in one edit frame titled `EDIT · FX 1 FX 2`, with the selected block lit in the title and on its block line.
The same three learned knobs SHALL control its type and two parameters.
Block selection SHALL NOT change effects.
Configured takeover SHALL be independent per device, block and control.
Changes SHALL be smooth, project-saved and undoable with `Ctrl+Z`.
Existing projects and learned bindings SHALL keep their behavior.

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
- **WHEN** the owner opens F4 at 80x26 and 120x40, then presses Tab
- **THEN** one frame titled `EDIT · FX 1 FX 2` holds both blocks with their block lines and three rows each
- **AND** FX 1 is lit first and FX 2 after Tab, and the frame does not fill unused rows

#### Scenario: Clear a shared effect knob assignment
- **WHEN** the owner clears a page knob assignment on F4 and restarts the program
- **THEN** the cleared knob does not control the effect and no old assignment returns

#### Scenario: Replaced page knobs stay replaced
- **WHEN** an old pad edit page knob has a replacement and the owner opens F4
- **THEN** the replacement controls the effect and the previous knob stays inactive

## MODIFIED Requirements

### Requirement: Every action can be learned
The system SHALL list on the MIDI tab every action a key can do, grouped as GLOBAL (works on every screen) and one group per screen, except quitting the program; the list SHALL scroll so the selected row is always visible. The GLOBAL group includes the 16 pad hits ("pad 1" … "pad 16") that hit the pads of the current bank.

#### Scenario: Global action
- **WHEN** the owner learns a button for "help" in GLOBAL
- **THEN** the button opens the help on the pads screen and on the settings screen

#### Scenario: Screen group wins
- **WHEN** the same button is learned for "help" in GLOBAL and for "pad mode" in PADS
- **THEN** on the pads screen it switches the pad mode, on the settings screen it opens the help

#### Scenario: Pad hits in the list
- **WHEN** the MIDI tab shows the GLOBAL group
- **THEN** it lists "pad 1" … "pad 16" among the global actions

### Requirement: Learn a control for an action
The system SHALL let the owner pick an action on the MIDI tab of the settings screen (F12), wait for a control to be moved, assign that control to the action and save the assignment at once.

#### Scenario: Knob for pad volume
- **WHEN** the owner selects "pad volume", presses Enter and turns a knob, touching a neighbour knob briefly
- **THEN** the knob that moved most is assigned, the row shows it, and the knob changes the selected pad volume with soft takeover

#### Scenario: Button for the next bank
- **WHEN** the owner selects "bank next", presses Enter and presses a controller button
- **THEN** that button switches to the next bank; the control the profile had for it no longer does

#### Scenario: Cancel and clear
- **WHEN** the owner presses Esc while the program waits for a control, or Backspace on an assigned row
- **THEN** waiting stops without changes, or the learned assignment is removed and the profile assignment applies again

#### Scenario: Learn a pad hit
- **WHEN** the owner selects "pad 7", presses Enter and presses a note or a button on the controller
- **THEN** that control is assigned to "pad 7" and the row shows the control

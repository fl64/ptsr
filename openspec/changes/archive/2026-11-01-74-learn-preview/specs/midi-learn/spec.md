## MODIFIED Requirements

### Requirement: Learn a control for an action
The system SHALL let the owner pick an action on the MIDI tab of the settings screen (F12), wait for a control to be moved and show it pending on the row before it saves anything. The pending row shows what the control drives now: in the group of the row, the action that loses the control, in the taken colour with a warning; in the other groups, each action that the control keeps, in grey, with `stays` (for example `b9 on GLOBAL: bank next · stays`). `Enter` assigns the control to the action and saves it. `Esc` drops the pending control and changes nothing. Another control moved before `Enter` takes the place of the pending one.

#### Scenario: Knob for pad volume
- **WHEN** the owner selects "pad volume", presses Enter, turns a knob, touching a neighbour knob briefly, and presses Enter again
- **THEN** the knob that moved most is assigned, the row shows it, and the knob changes the selected pad volume with soft takeover

#### Scenario: Button for the next bank
- **WHEN** the owner selects "bank next", presses Enter, presses a controller button and presses Enter again
- **THEN** that button switches to the next bank; the control the row had before no longer does

#### Scenario: Cancel and clear
- **WHEN** the owner presses Esc while the program waits for a control, or Backspace on an assigned row
- **THEN** waiting stops without changes, or the binding is removed and the action has no
  control, also when it came from the profile

#### Scenario: Clear a profile role
- **WHEN** b3 of the X-Session Pro changes the pad mode, as seeded from the profile, and the
  owner presses Backspace on the "pad mode" row and restarts the program
- **THEN** b3 does nothing on the pads screen, and the row still has no control

#### Scenario: See what the button leaves before it is saved
- **WHEN** b9 switches the pad mode in the PADS group and turns to the next bank in the GLOBAL group, and on the MIDI tab of F12 the owner selects "pad mark" in PADS, presses Enter and presses b9
- **THEN** b9 stands pending on the "pad mark" row, the "pad mode" row lights in the taken colour with a warning that it loses b9, the line names `b9 on GLOBAL: bank next · stays` in grey, and nothing is saved yet

#### Scenario: Esc on F12 keeps all
- **WHEN** b9 stands pending on a row of the MIDI tab and the owner presses Esc
- **THEN** every binding stays as it was and the config file does not change

### Requirement: One control, one action
The system SHALL keep each learned control on a single action per group: assigning it to a new action in the group removes it from the old one in that group. The control keeps its actions in the other groups: on a screen, the group of the screen wins over the global group, so one control can do one thing on the pads screen and another thing on other screens.

#### Scenario: Reassign a button
- **WHEN** a button learned for "bank next" in the GLOBAL group is learned for "pad mode" in the PADS group
- **THEN** on the pads screen it switches the pad mode, on the other screens it still turns to the next bank, and the learn showed `bank next · stays` before the owner pressed Enter

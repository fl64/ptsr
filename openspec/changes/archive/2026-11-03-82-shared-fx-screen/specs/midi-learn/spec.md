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
- **WHEN** left Cue (b7) opens pad edit from the X-Session Pro profile, and the owner
  clears the "open pad edit only" row and restarts the program
- **THEN** b7 does nothing on the pads screen, and the row still has no control

#### Scenario: See what the button leaves before it is saved
- **WHEN** b9 switches the pad mode in the PADS group and turns to the next bank in the GLOBAL group, and on the MIDI tab of F12 the owner selects "pad mark" in PADS, presses Enter and presses b9
- **THEN** b9 stands pending on the "pad mark" row, the "pad mode" row lights in the taken colour with a warning that it loses b9, the line names `b9 on GLOBAL: bank next · stays` in grey, and nothing is saved yet

#### Scenario: Esc on F12 keeps all
- **WHEN** b9 stands pending on a row of the MIDI tab and the owner presses Esc
- **THEN** every binding stays as it was and the config file does not change

### Requirement: Reset to profile
The system SHALL offer on the MIDI tab, per connected device, an action that replaces the
device's bindings with the roles of its profile; it asks first, `Enter` resets, `Esc`
keeps.

#### Scenario: Reset to profile
- **WHEN** the owner clears b7 and learns b3 for "next bank", then selects "reset to
  profile" for the X-Session Pro and confirms the question with Enter
- **THEN** left Cue (b7) opens pad edit again and right Headphone (b3) opens FX

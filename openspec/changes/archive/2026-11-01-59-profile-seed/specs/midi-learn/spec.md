## MODIFIED Requirements

### Requirement: Learn a control for an action
The system SHALL let the owner pick an action on the MIDI tab of the settings screen (F12), wait for a control to be moved, assign that control to the action and save the assignment at once.

#### Scenario: Knob for pad volume
- **WHEN** the owner selects "pad volume", presses Enter and turns a knob, touching a neighbour knob briefly
- **THEN** the knob that moved most is assigned, the row shows it, and the knob changes the selected pad volume with soft takeover

#### Scenario: Button for the next bank
- **WHEN** the owner selects "bank next", presses Enter and presses a controller button
- **THEN** that button switches to the next bank; the control the row had before no longer does

#### Scenario: Cancel and clear
- **WHEN** the owner presses Esc while the program waits for a control, or Backspace on an assigned row
- **THEN** waiting stops without changes, or the binding is removed and the action has no
  control, also when it came from the profile

#### Scenario: Clear a profile role
- **WHEN** b3 of the X-Session Pro changes the pad mode, as seeded from the profile, and the
  owner presses Backspace on the "pad mode" row and restarts the program
- **THEN** b3 does nothing on the pads screen, and the row still has no control

### Requirement: Learned assignments are kept
The system SHALL save the bindings in the user config `~/.config/arasaka-404/config.toml`
and keep them after a restart; once a device is seeded (input-devices, "Device profiles")
they are its only actions, the profile file is not changed.

#### Scenario: Restart
- **WHEN** the owner learned controls and restarts the program
- **THEN** the controls do the same actions as before the restart

#### Scenario: Cleared stays cleared
- **WHEN** the owner cleared every row of the X-Session Pro and restarts the program
- **THEN** no control of it does anything but name itself on the input debug tab, and the
  profile roles are not copied again

## ADDED Requirements

### Requirement: Reset to profile
The system SHALL offer on the MIDI tab, per connected device, an action that replaces the
device's bindings with the roles of its profile; it asks first, `Enter` resets, `Esc`
keeps.

#### Scenario: Reset to profile
- **WHEN** the owner cleared b10 and learned b3 for "next bank", then picks "reset to
  profile" for the X-Session Pro and presses Enter
- **THEN** b10 opens the pad edit again and b3 changes the pad mode, as the profile has it

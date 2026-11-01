## MODIFIED Requirements

### Requirement: Learned assignments are kept
The system SHALL save the bindings in the user config `~/.config/ptsr/config.toml`
and keep them after a restart; once a device is seeded (input-devices, "Device profiles")
they are its only actions, the profile file is not changed.

#### Scenario: Restart
- **WHEN** the owner learned controls and restarts the program
- **THEN** the controls do the same actions as before the restart

#### Scenario: Cleared stays cleared
- **WHEN** the owner cleared every row of the X-Session Pro and restarts the program
- **THEN** no control of it does anything but name itself on the input debug tab, and the
  profile roles are not copied again

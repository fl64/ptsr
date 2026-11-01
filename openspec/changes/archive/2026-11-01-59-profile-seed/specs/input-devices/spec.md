## MODIFIED Requirements

### Requirement: Device profiles
The system SHALL describe each device with a TOML profile: controls (CC, notes, type) and
their starting logical actions per screen; a device without a profile works with the
default profile (notes 36–51 → pads 1–16). The starting actions are copied into the user
config the first time the device connects with no bindings there; from then on the user
config alone gives the device its actions, and the profile only names its controls.

#### Scenario: New controller
- **WHEN** a device without its own profile is connected
- **THEN** its notes 36–51 play pads, and the input debug screen shows what comes in, so that the owner can write a profile

#### Scenario: Profile with an error
- **WHEN** a profile file cannot be read (for example, it still has `??` from the template)
- **THEN** the program does not crash, the device works with the default profile, the status shows the file and the line with the error

#### Scenario: First connect
- **WHEN** the X-Session Pro connects and the user config has no bindings for it
- **THEN** the user config gets every role of its profile, b10 opens the pad edit, and the
  MIDI tab shows these rows as the owner's

#### Scenario: Learned wins
- **WHEN** the user config has the X-Session Pro b3 learned for "next bank" from before
  this change and the program starts
- **THEN** b3 switches to the next bank, the profile's "pad mode" on b3 is not added, and
  the other profile roles are added

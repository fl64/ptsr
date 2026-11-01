## MODIFIED Requirements

### Requirement: Output device selection
The system SHALL let the owner select an output device from the available ones on the settings screen (F12), apply the choice at once, without a restart and without a break in the sound, and save it in the user config.

#### Scenario: Switch to the speakers
- **WHEN** the sound goes to the headphones and the owner selects the laptop speakers
- **THEN** the next click sounds from the speakers, no more sound goes to the headphones

#### Scenario: Mono device
- **WHEN** the selected output device has one channel
- **THEN** both channels of the program go to it, no sound goes to other devices

#### Scenario: JACK is not available right now
- **WHEN** the config already has `backend = "jack"` and the sound server is not running
- **THEN** the device row says that JACK is not available right now; it does not tell the owner to change a config that is already right

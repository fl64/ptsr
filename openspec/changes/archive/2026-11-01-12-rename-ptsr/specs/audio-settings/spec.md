## MODIFIED Requirements

### Requirement: Device loss and return
The system SHALL, if the selected device is gone, output sound to the first available device and report this; when the device is back, it SHALL return the sound to it automatically. The saved choice does not change.

#### Scenario: Headphones turned off and on
- **WHEN** bluetooth headphones are selected, they disconnect, then connect again
- **THEN** without a restart the sound moves to the first available device, then returns to the headphones; the config still has the headphones

#### Scenario: Start without the selected device
- **WHEN** the program starts and the saved device is not there
- **THEN** the sound goes to the first available device, a message at the bottom names the saved device

#### Scenario: Another program connected to the sound
- **WHEN** another program registers its ports in JACK
- **THEN** the ptsr connections do not break and the sound does not stop

### Requirement: User config
The system SHALL read settings from `config/audio.toml` and on top of them from `~/.config/ptsr/config.toml`, and save the choice from the settings screen only to the user file, without damaging it if a write fails.

#### Scenario: User file with an error
- **WHEN** `~/.config/ptsr/config.toml` cannot be read
- **THEN** the program works with the default values, reports the error and does not overwrite this file

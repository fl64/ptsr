## ADDED Requirements

### Requirement: Output device selection
The system SHALL let the owner select an output device from the available ones on the settings screen (F12), apply the choice at once, without a restart and without a break in the sound, and save it in the user config.

#### Scenario: Switch to the speakers
- **WHEN** the sound goes to the headphones and the owner selects the laptop speakers
- **THEN** the next click sounds from the speakers, no more sound goes to the headphones

#### Scenario: Mono device
- **WHEN** the selected output device has one channel
- **THEN** both channels of the program go to it, no sound goes to other devices

### Requirement: Input device selection
The system SHALL let the owner select an input device and show the input signal level on the settings screen.

#### Scenario: Microphone
- **WHEN** the owner selected a microphone and speaks into it
- **THEN** the input level meter moves; the input does not get into the output

### Requirement: Buffer size on the fly
The system SHALL let the owner change the buffer size (64, 128, 256, 512) on the settings screen without a restart and show the actual size and the latency in the status line.

#### Scenario: Server did not accept the buffer
- **WHEN** the sound server did not accept the selected size
- **THEN** the status line shows the actual size, a message at the bottom says the size was not accepted

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
- **THEN** the arasaka-404 connections do not break and the sound does not stop

### Requirement: User config
The system SHALL read settings from `config/audio.toml` and on top of them from `~/.config/arasaka-404/config.toml`, and save the choice from the settings screen only to the user file, without damaging it if a write fails.

#### Scenario: User file with an error
- **WHEN** `~/.config/arasaka-404/config.toml` cannot be read
- **THEN** the program works with the default values, reports the error and does not overwrite this file

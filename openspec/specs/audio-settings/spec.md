# audio-settings Specification

## Purpose
Audio settings on the F12 screen: output and input device selection, buffer size on the fly, behavior when devices are lost, user config.

## Requirements

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
- **THEN** the device row says that JACK is not available right now. It does not ask the owner to change the correct config.

### Requirement: Input device selection
The system SHALL let the owner select an input device and show the input signal level on the settings screen. The input device is linked while the settings screen is open, so the meter moves, and unlinked when the screen is left, unless a microphone recording runs.

#### Scenario: Microphone
- **WHEN** the owner selected a microphone and speaks into it
- **THEN** the input level meter moves; the input does not get into the output

#### Scenario: Leaving the settings unlinks the microphone
- **WHEN** the settings screen is open and the owner switches to the pads screen
- **THEN** the input device is no longer linked to the program

### Requirement: Buffer size on the fly
The system SHALL take the sound server's buffer size as it is at start (the default, `buffer_size = 0`), let the owner change the buffer size (64, 128, 256, 512, 1024, auto) on the settings screen without a restart, and show the actual size and the latency in the status line. With auto chosen, the program never asks the server for another size.

#### Scenario: Start does not touch the graph
- **WHEN** the config has `buffer_size = 0` (or no line) and the server runs 1024
- **THEN** the program starts at 1024 without asking the server to change anything, and other programs' sound keeps playing without a hiccup

#### Scenario: An explicit size still applies
- **WHEN** the config has `buffer_size = 256`
- **THEN** the program asks the server for 256 at start, as before

#### Scenario: Server did not accept the buffer
- **WHEN** the sound server did not accept the selected size
- **THEN** the status line shows the actual size, a message at the bottom says the size was not accepted

#### Scenario: The buffer row explains itself
- **WHEN** the owner stands on the buffer row of the settings screen
- **THEN** it shows auto with the actual size in parentheses and one line saying what it is for: follow the server by default, go smaller only if pads feel late, bigger if clicks are heard

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

### Requirement: Held keys walk the settings rows
The system SHALL, on the settings screen, walk the rows while `I` or `K` (`↑` `↓`) is
held, as the file list does.

#### Scenario: Down the MIDI list
- **WHEN** the MIDI tab is open on its first row and the owner holds `K` for five repeats
- **THEN** the selection moves down six rows: the press and every repeat

### Requirement: Settings use the edit frame

F12 SHALL draw its body in one frame in the edit color, titled `SETTINGS · AUDIO INPUT MIDI PATHS`, with the open tab lit. A narrow title SHALL keep only the open tab. Labels SHALL be lowercase. The selected row SHALL be bright with `▸` on every tab. Explanation lines SHALL be dim. The keys SHALL keep their roles.

#### Scenario: Settings use the edit frame
- **WHEN** the owner opens F12 at 80x26 and 120x40 and presses Tab through all tabs, then I/K
- **THEN** one frame holds the tab body, its title lights the open tab, and labels are lowercase
- **AND** the selected row is bright with `▸`, and no row is inverse

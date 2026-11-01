## ADDED Requirements

### Requirement: Pad grid and banks
The system SHALL provide 16 pads in a 4×4 grid and 8 banks numbered 1–8; the bank is switched with a command from the keyboard or a controller: next/previous, or directly with Ctrl+1…8. On screen a bank is always shown with the word "bank" so it is not confused with the pad keys 1–4.

#### Scenario: Bank change
- **WHEN** the owner runs "next bank" on bank 1
- **THEN** bank 2 becomes active, the screen shows the pads of bank 2, sounding pads of bank 1 play to the end

#### Scenario: Direct bank choice
- **WHEN** the owner presses Ctrl+3
- **THEN** bank 3 becomes active; the plain 3 key still plays pad 3

### Requirement: Pads from the keyboard
The system SHALL play pads from the physical keys `1 2 3 4 / Q W E R / A S D F / Z X C V` in the English (US) and Russian layouts, passing both the press and the release; Shift on press gives full velocity.

#### Scenario: Russian layout
- **WHEN** the Russian layout is on and the owner presses the physical key Q
- **THEN** pad 5 triggers, not a command and not text input

#### Scenario: Pad block keys are not commands
- **WHEN** any screen assigns commands to the keyboard
- **THEN** no command is assigned to the pad block keys or to Shift

#### Scenario: Terminal without the kitty protocol
- **WHEN** the terminal does not support the extended keyboard protocol
- **THEN** pads play on press, gate mode sounds like one-shot, a warning is shown at the bottom

### Requirement: Pad modes
The system SHALL support the modes one-shot (plays to the end) and gate (plays while the pad is held), volume by velocity, and a pad volume that can be changed with a controller knob or with keys.

#### Scenario: Gate
- **WHEN** a pad in gate mode is pressed and released after 300 ms
- **THEN** the sound fades out right after the release, without a click

#### Scenario: Pad volume from the keyboard
- **WHEN** a pad is selected on the pads screen and the owner presses `−` or `=`
- **THEN** the pad volume changes by 1 dB, a sounding voice changes its volume smoothly, without a click

### Requirement: Pad response
The system SHALL start the pad sound in the nearest audio block after the event is received.

#### Scenario: Fast roll
- **WHEN** the owner hits one pad 16 times per second
- **THEN** every hit sounds, the previous voice of the same pad is cut without a click

### Requirement: Pads screen
The system SHALL show the pads screen on F2: a 4×4 grid with keys and sound names, the current bank, the selected pad with its mode and volume, and a flash of the pad on a hit. Hitting a pad selects it.

#### Scenario: Hit selects the pad
- **WHEN** the owner hits pad W
- **THEN** pad 6 flashes, becomes selected, its mode and volume are shown on the right

## ADDED Requirements

### Requirement: Several MIDI devices at once
The system SHALL accept MIDI from all connected devices at once and support connecting and disconnecting without a restart and without a break in the sound.

#### Scenario: Two controllers
- **WHEN** a controller with pads (for example, SMC-PAD) and an X-Session Pro are connected
- **THEN** the pads of the first one play pads, the X-Session knobs change parameters, both work at the same time

#### Scenario: Controller pulled out during play
- **WHEN** a device is disconnected while pads are sounding
- **THEN** the sound does not stop, the status shows the disconnect, after the device is connected again its profile is applied again

### Requirement: Controller pads without delay
The system SHALL send a pad hit from a controller straight to the audio engine, not through the screen refresh, and SHALL use the hit velocity 1–127.

#### Scenario: Hit from a controller
- **WHEN** the owner hits a controller pad while the screen is busy redrawing
- **THEN** the pad sounds in the nearest audio block, and the pads screen flashes and selects it

### Requirement: Device profiles
The system SHALL describe each device with a TOML profile: controls (CC, notes, type) and their logical actions per screen; a device without a profile works with the default profile (notes 36–51 → pads 1–16).

#### Scenario: New controller
- **WHEN** a device without its own profile is connected
- **THEN** its notes 36–51 play pads, and the input debug screen shows what comes in, so that the owner can write a profile

#### Scenario: Profile with an error
- **WHEN** a profile file cannot be read (for example, it still has `??` from the template)
- **THEN** the program does not crash, the device works with the default profile, the status shows the file and the line with the error

### Requirement: Soft takeover for absolute controls
The system SHALL not change a parameter with an absolute knob or fader until the control position crosses the current value of the parameter, and SHALL show on the screen which way to move it.

#### Scenario: Pad change
- **WHEN** the owner selects a pad with volume −12 dB, and the volume knob is at 0 dB
- **THEN** turning the knob does not change the volume until it passes −12 dB; the screen shows a direction arrow

### Requirement: Input debug
The system SHALL have a tab on the settings screen (F12) that shows the latest input events: device, type, number, value and the logical action it became.

#### Scenario: Reading the layout
- **WHEN** the owner turns a knob on the X-Session Pro on the input debug screen
- **THEN** the owner sees the CC number, the value and "not assigned" or the action name

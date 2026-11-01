## MODIFIED Requirements

### Requirement: Several MIDI devices at once
The system SHALL accept MIDI from all connected devices at once and support connecting and disconnecting without a restart and without a break in the sound.

#### Scenario: Two controllers
- **WHEN** a controller with pads (for example, SMC-PAD) and an X-Session Pro are connected
- **THEN** the pads of the first one play pads, the X-Session knobs change parameters, both work at the same time

#### Scenario: Controller pulled out during play
- **WHEN** a device is disconnected while pads are sounding
- **THEN** one-shot pads play to their end, a gate pad held by a button of that device fades out over its release as if the button were released, with no click; the status shows the disconnect; after the device is connected again its profile is applied again

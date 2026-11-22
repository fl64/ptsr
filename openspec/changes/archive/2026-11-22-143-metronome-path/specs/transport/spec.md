## ADDED Requirements

### Requirement: The click has an independent monitor path

The system SHALL send the automatic click and A1 count-in to the device after the master stage. The master meter and sources for recording and render SHALL exclude the click. Panic SHALL also stop the click.

#### Scenario: The click plays without moving the master meter

- **WHEN** the click plays in an empty project, in transport or during A1 count-in
- **THEN** the device plays the click at its scheduled time
- **AND** the master meter stays at zero

#### Scenario: Panic stops the click path

- **WHEN** the owner uses Panic while a click or count-in sounds
- **THEN** it stops with the other device sound within the Panic fade time

### Requirement: The click has its own level

The Audio settings SHALL save a click level from 0 to 100 percent. It SHALL start at 100 percent for an older configuration. Keyboard and the controller SHALL change the same setting. The device SHALL smooth the gain change. The level SHALL set the gain of the click and count-in while the beat position and master level stay the same.

#### Scenario: Click level changes independently

- **WHEN** the owner changes Click level during playback with the keyboard or a controller
- **THEN** only the click level changes and the next beat is on time
- **AND** the new level stays after restart

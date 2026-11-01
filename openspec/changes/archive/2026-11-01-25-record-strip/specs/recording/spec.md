## ADDED Requirements

### Requirement: The recording shows on the strip
The system SHALL draw the level of what is being recorded on the strip above the pad grid
while a recording runs, newest at the right, on the strip's decibel scale and in the
colour of the recording mark. It takes the place of the live output and of the pad edit
envelope; when the recording stops, the strip shows again what it showed before.

#### Scenario: The voice on the strip
- **WHEN** the owner holds Ctrl+R and speaks
- **THEN** the strip runs with the voice in the colour of the recording mark, and shows the output again after the release

#### Scenario: The output recording on the strip
- **WHEN** music plays in another program and the owner starts the output recording
- **THEN** the strip shows that music, as it goes into the file

#### Scenario: Recording in the pad edit
- **WHEN** the pad edit is open and the owner records
- **THEN** the strip shows the recording instead of the envelope, and the envelope comes back when the recording stops

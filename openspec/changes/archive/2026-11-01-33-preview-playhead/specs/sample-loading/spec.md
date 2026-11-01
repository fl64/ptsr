## ADDED Requirements

### Requirement: Preview playhead
The system SHALL draw, while the preview plays on the files screen, a thin green line over the wave of the file at the place that sounds now, moving with the sound; the line is gone when the preview stops or the file ends.

#### Scenario: The line runs with the sound
- **WHEN** the preview of a 4-second file has played for 2 seconds
- **THEN** the green line stands in the middle of the wave

#### Scenario: No line when silent
- **WHEN** the preview is off, or the file has played to its end
- **THEN** no green line is drawn over the wave

## MODIFIED Requirements

### Requirement: Input device selection
The system SHALL let the owner select an input device and show the input signal level on the settings screen. The input device is linked while the settings screen is open, so the meter moves, and unlinked when the screen is left, unless a microphone recording runs.

#### Scenario: Microphone
- **WHEN** the owner selected a microphone and speaks into it
- **THEN** the input level meter moves; the input does not get into the output

#### Scenario: Leaving the settings unlinks the microphone
- **WHEN** the settings screen is open and the owner switches to the pads screen
- **THEN** the input device is no longer linked to the program

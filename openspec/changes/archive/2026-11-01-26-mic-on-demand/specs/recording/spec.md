## ADDED Requirements

### Requirement: The microphone is linked only while recording
The system SHALL link the selected input device to the program only while a microphone
recording runs (and while the settings screen is open, see audio-settings); at start, at
quit and the rest of the time the program leaves the input device unlinked, so the sound
graph of other programs is not reclocked by the program's start or quit.

#### Scenario: Start leaves the microphone alone
- **WHEN** music plays on the headphones and the owner starts the program
- **THEN** the microphone is not linked to the program, the headphones keep driving the sound

#### Scenario: A recording links the microphone
- **WHEN** the owner holds Ctrl+R, speaks and releases it
- **THEN** the microphone is linked while the key is held, the recording has the voice, and after the release the microphone is unlinked

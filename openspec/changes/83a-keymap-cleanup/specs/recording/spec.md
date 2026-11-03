## MODIFIED Requirements

### Requirement: Record from the microphone

The system SHALL start recording the input ports on an Alt+R press and stop on its
release. Without key-release events, an Alt+R press SHALL start and the next press stop
recording. The status SHALL count seconds and the corner indicator SHALL show the
recording. Explicit owner bindings for `record.mic`, including old Ctrl+R, SHALL keep
the same hold/toggle behavior. Ctrl+T SHALL keep its output-recording toggle. Only one
recording SHALL run at a time, as before.

#### Scenario: Hold and speak
- **WHEN** the owner holds Alt+R, speaks and releases it
- **THEN** the status shows the length of the recording and says to hit a pad

#### Scenario: No microphone
- **WHEN** the owner presses the record key and no input ports are connected
- **THEN** the status says there is nothing to record from, no file appears, and the
  program keeps working

#### Scenario: One recording at a time
- **WHEN** output recording runs and the owner presses and releases Alt+R
- **THEN** output recording goes on; Ctrl+T during microphone recording is ignored likewise

#### Scenario: Microphone toggle without release events
- **WHEN** a terminal without release events sends Alt+R, then Alt+R again
- **THEN** microphone recording starts once and stops on the second press

#### Scenario: Holding the moved microphone key does not toggle
- **WHEN** a terminal with releases repeats Alt+R while it stays held
- **THEN** the recording continues until the release and does not stop on repeats

### Requirement: The recording shows on the strip

The system SHALL draw the level of what is being recorded on the strip above the pad grid
while a recording runs, newest at the right, on the strip's decibel scale and in the
colour of the recording mark. It takes the place of the live output and of the pad edit
envelope; when the recording stops, the strip shows again what it showed before.

#### Scenario: The voice on the strip
- **WHEN** the owner holds Alt+R and speaks
- **THEN** the strip runs with the voice in the colour of the recording mark, and shows
  the output again after the release

#### Scenario: The output recording on the strip
- **WHEN** music plays in another program and the owner starts output recording
- **THEN** the strip shows that music, as it goes into the file

#### Scenario: Recording in the pad edit
- **WHEN** the pad edit is open and the owner records
- **THEN** the strip shows the recording instead of the envelope, and the envelope comes
  back when recording stops

### Requirement: The microphone is linked only while recording

The system SHALL link the selected input device to the program only while a microphone
recording runs (and while the settings screen is open, see audio-settings); at start, at
quit and the rest of the time the program leaves the input device unlinked, so the sound
graph of other programs is not reclocked by the program's start or quit.

#### Scenario: Start leaves the microphone alone
- **WHEN** music plays on the headphones and the owner starts the program
- **THEN** the microphone is not linked to the program, the headphones keep driving the sound

#### Scenario: A recording links the microphone
- **WHEN** the owner holds Alt+R, speaks and releases it
- **THEN** the microphone is linked while the key is held, the recording has the voice,
  and after the release the microphone is unlinked

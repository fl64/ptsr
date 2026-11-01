# recording Specification

## ADDED Requirements

### Requirement: Record from the microphone
The system SHALL start recording the input ports when the record-microphone key is pressed and stop when it is released (or pressed again without release events); the status counts the seconds while recording, and the corner indicator shows it.

#### Scenario: Hold and speak
- **WHEN** the owner holds Ctrl+R, speaks and releases it
- **THEN** the status shows the length of the recording and says to hit a pad

#### Scenario: No microphone
- **WHEN** the owner presses the record key and no input ports are connected
- **THEN** the status says there is nothing to record from, no file appears, and the program keeps working

#### Scenario: One recording at a time
- **WHEN** the output recording runs and the owner presses and releases Ctrl+R
- **THEN** the output recording goes on; Ctrl+T during a microphone recording is ignored the same way

### Requirement: Record the output device
The system SHALL record what plays on the current output device while the record-output mode is on — ptsr and other programs alike — by tapping the device's monitor ports; the mode is toggled by the key or a learned control, and a device without monitors reports it instead of recording silence. The tap is unlinked from the monitor when the recording stops.

#### Scenario: Capture what plays
- **WHEN** music plays on the output device, the owner starts the output recording and stops it after a while
- **THEN** the recording is in hand and playing it back matches what was heard, other programs included

#### Scenario: No monitor on the device
- **WHEN** the output device exposes no monitor ports and the owner starts the output recording
- **THEN** the status says the device has no monitor to record and nothing starts

### Requirement: A recording lands in hand
The system SHALL put a finished recording in hand as a WAV file in the samples root, named by time, and a pad hit binds it with the usual overwrite question; Esc drops the recording. A silent recording is dropped with a message instead. The kept file fades in and out over 5 ms, so a pad playing it never clicks at its edges, even when the recording started in the middle of a loud sound.

#### Scenario: No click at the edges
- **WHEN** the owner starts recording while a loud sound plays, binds the recording to a pad and hits it
- **THEN** the pad starts and ends without a click

#### Scenario: Bind to a pad
- **WHEN** a recording is in hand and the owner hits pad Q
- **THEN** pad Q plays the recording; when Q held something, the replace question comes first

#### Scenario: Kept over a restart
- **WHEN** the owner binds a recording and restarts the program
- **THEN** the pad plays it again, the file is in the samples root

#### Scenario: Silence is not kept
- **WHEN** the owner records with the microphone muted
- **THEN** no file appears and the status says the recording was silent

### Requirement: Recording stops at the cap
The system SHALL stop a recording by itself after 30 seconds and report it; nothing after the cap is lost from the kept file.

#### Scenario: Half a minute
- **WHEN** the owner holds the record key for more than 30 seconds
- **THEN** the recording stops by itself, the status names the cap, and the recording is in hand

### Requirement: Record actions are learnable
The system SHALL list "record microphone" and "record output" in the GLOBAL group of the MIDI learn; a learned button toggles the corresponding recording.

#### Scenario: A button records
- **WHEN** the owner learns a controller button for "record microphone" and presses it twice
- **THEN** the first press starts the recording and the second stops it, as with the keyboard toggle

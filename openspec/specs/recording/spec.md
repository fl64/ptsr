# recording Specification

## Purpose

Recording the microphone and the output device into WAV files that land in hand, ready for a pad.

## Requirements

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

### Requirement: Record the output device

While output recording is active, the system SHALL record the sampler master and the other applications on the selected output before the sampler click. The key and learned control SHALL toggle the same recording. If the device has no monitor or the system cannot link the recording inputs, the system SHALL report the cause and refuse to start. When recording stops, the input links for the take SHALL close.

#### Scenario: Capture what plays

- **WHEN** music plays on the output device and the owner starts and stops output recording
- **THEN** the recording is in hand
- **AND** playback contains the sampler and other applications on the chosen output
- **AND** the recording has no sampler click

#### Scenario: No monitor on the device

- **WHEN** the output device exposes no monitor ports and the owner starts the output recording
- **THEN** the status says the device has no monitor to record and nothing starts

#### Scenario: Follow other playback sources during an output recording

- **WHEN** another application connects to or leaves the selected output during a recording
- **THEN** the system updates the recording links and keeps the take
- **AND** links from other outputs stay out of the capture

#### Scenario: Closing the output preserves the captured part

- **WHEN** the selected output disappears while output recording is active
- **THEN** the system stops the take and reports the loss
- **AND** retained audio follows the usual recording handoff

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

### Requirement: Pattern recording is separate from audio capture

Pattern recording SHALL store pad events, not microphone or output audio. Its commands SHALL NOT link microphone or monitor ports, create a WAV or put a file in hand. Existing Alt+R hold/toggle and Ctrl+T output capture SHALL keep their separate roles. A learned pattern-record button SHALL never select audio capture by screen or transport state.

#### Scenario: Pattern recording does not start microphone capture
- **WHEN** pattern recording starts, ends or cancels on any current screen
- **THEN** no microphone or output recording starts, no new capture link appears and no WAV lands in hand
- **AND** existing explicit microphone and output controls retain their meaning

### Requirement: Recording targets refuse conflicting starts

Microphone and output recording SHALL remain mutually exclusive. CountIn, Recording and Finishing of a pattern pass SHALL refuse audio-capture starts. Active audio capture SHALL refuse pattern recording. Refusal SHALL name the active target and preserve it. Releases from a refused microphone request SHALL NOT stop the pattern pass or output capture.

#### Scenario: Pattern recording refuses microphone and output starts
- **WHEN** microphone or output capture is requested during pattern count-in, recording or finishing
- **THEN** the request refuses with the active A1 phase and the pattern pass continues
- **AND** microphone release cannot end that pass

#### Scenario: Audio capture refuses a pattern start without changing targets
- **WHEN** microphone or output capture runs and pattern recording is requested
- **THEN** the request refuses and the original capture continues unchanged
- **AND** no empty A1 is created by the refused start

### Requirement: STOP ends the output recording

On any work screen, including Assign, each STOP during output recording SHALL do the first applicable action.

1. Stop an active preview.
2. Pause A1 if it plays. Keep its tail in the take.
3. Save the take and keep the A1 position.

If the next STOP arrives before A1 pauses, the app SHALL save the take when A1 pauses.
If recording continues after the pause, the mode line SHALL say «● REC OUTPUT 0:12 · STOP end rec».
On Pads, `record output` SHALL also end the take.

#### Scenario: STOP on A1 pauses A1 and keeps the recording
- **WHEN** the output recording runs, A1 plays and the owner presses STOP on A1
- **THEN** A1 pauses, the recording goes on and the mode line says «STOP end rec»
- **AND** a second STOP saves the recording and puts it in hand

#### Scenario: One STOP saves when A1 does not play
- **WHEN** the output recording runs on Chop, neither A1 nor a preview plays and the owner presses STOP
- **THEN** the app saves the recording and puts it in hand

#### Scenario: STOP pauses A1 and keeps the take on Files and Chop
- **WHEN** the output recording and A1 run on Files, Chop or Assign without a preview
- **THEN** the first STOP pauses A1 and the next STOP saves the take with its tail
- **AND** the paused position and any Assign plan stay in place

#### Scenario: Two STOP presses before the A1 pause save the take
- **WHEN** A1 plays during output recording without a preview
- **AND** the owner presses STOP twice before A1 pauses
- **THEN** A1 pauses at the current position
- **AND** the app saves the take and puts it in hand at that position

#### Scenario: STOP handles preview A1 and capture in order
- **WHEN** a preview and A1 play on Chop during output recording
- **THEN** the SMC-PAD hint names the preview step
- **AND** the first STOP stops only the preview, the second pauses A1 and the third saves the take

### Requirement: The output recording names one stop control

The mode line SHALL name the stop control of the last input device. No second hint SHALL name another stop control.

#### Scenario: The recording hint names one stop control
- **WHEN** the output recording runs at 80×26 after SMC-PAD input
- **THEN** the screen names STOP as the stop control and does not say «Ctrl+T to stop»

### Requirement: A held microphone take can end over a question

A keyboard microphone hold SHALL end when its key comes up in either modifier release order. An open question SHALL allow the release of the keyboard or SMC-PAD microphone hold. The release SHALL keep the question open and end only a take that this control started.

#### Scenario: A question lets the microphone hold end

- **WHEN** the owner starts a microphone hold and opens a question before letting go
- **THEN** releasing the keyboard chord in either order or releasing SMC-PAD RECORD stops the take
- **AND** the question stays open
- **AND** a hold that starts no take cannot end a take from another input

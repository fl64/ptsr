## ADDED Requirements

### Requirement: STOP ends the output recording

While the output recording runs, STOP SHALL end it on every screen. If A1 plays, the first STOP SHALL pause A1 and the recording SHALL go on, so that it keeps the tail of the sound. The mode line SHALL then say «● REC OUTPUT 0:12 · STOP end rec». The next STOP SHALL save the recording. If A1 does not play, one STOP SHALL save the recording. `record output` on Pads SHALL also end a running recording.

#### Scenario: STOP on A1 pauses A1 and keeps the recording
- **WHEN** the output recording runs, A1 plays and the owner presses STOP on A1
- **THEN** A1 pauses, the recording goes on and the mode line says «STOP end rec»
- **AND** a second STOP saves the recording and puts it in hand

#### Scenario: One STOP saves when A1 does not play
- **WHEN** the output recording runs on Chop, A1 does not play and the owner presses STOP
- **THEN** the recording is saved and in hand

### Requirement: The output recording names one stop control

The mode line SHALL name the stop control of the last input device. No second hint SHALL name another stop control.

#### Scenario: The recording hint names one stop control
- **WHEN** the output recording runs at 80×26 after SMC-PAD input
- **THEN** the screen names STOP as the stop control and does not say «Ctrl+T to stop»

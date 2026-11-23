## MODIFIED Requirements

### Requirement: STOP ends the output recording

During output recording, STOP SHALL first stop an active preview.
With no preview, STOP SHALL pause A1 if it plays. The recording SHALL keep the tail.
With both stopped, STOP SHALL save the recording at the current position.
This order SHALL apply on every work screen, including Assign.
After the pause the mode line SHALL say «● REC OUTPUT 0:12 · STOP end rec».
`record output` on Pads SHALL also end a running recording.

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

#### Scenario: STOP handles preview A1 and capture in order
- **WHEN** a preview and A1 play on Chop during output recording
- **THEN** the SMC-PAD hint names the preview step
- **AND** the first STOP stops only the preview, the second pauses A1 and the third saves the take

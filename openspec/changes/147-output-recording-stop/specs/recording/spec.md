## MODIFIED Requirements

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

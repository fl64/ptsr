# transport Delta

## ADDED Requirements

### Requirement: Context PLAY and STOP keep a paused position

The `play.context` action SHALL start a stopped transport at bar 1 or continue a paused transport from its accepted position. It SHALL not restart a running transport. The `stop.context` action SHALL pause a running transport at its accepted position on the first tap. A second tap while paused SHALL reset to bar 1, beat 1. These actions SHALL preserve manually played one-shot tails and existing effect tails. Space and the existing explicit stop and toggle actions SHALL keep their reset behavior.

#### Scenario: Context STOP pauses and PLAY continues A1
- **WHEN** A1 plays beyond bar 1, STOP pauses it and PLAY follows before another STOP
- **THEN** the stopped position stays at the accepted beat and PLAY continues from that position
- **AND** no early, missing or doubled pattern hit occurs at the pause or continuation boundary

#### Scenario: Second context STOP resets to the start
- **WHEN** A1 pauses beyond bar 1 and STOP is tapped again before PLAY
- **THEN** the stopped position returns to bar 1, beat 1
- **AND** PLAY starts from the first beat without restarting a manually played pad tail

#### Scenario: Existing Space transport path keeps its reset
- **WHEN** the owner uses Space to stop a running transport, then starts it again
- **THEN** the stopped position is bar 1, beat 1 and the restart begins at the first beat
- **AND** the new SMC-PAD pause state does not change the keyboard toggle path

### Requirement: Context buttons keep previews separate from transport

On Files and Chop, `play.context` SHALL start the selected preview only when no preview plays. It SHALL not stop or restart an active preview. `stop.context` SHALL stop that preview without starting it. On Files, STOP SHALL also cancel a pending preview load. With no preview, STOP MAY reset a paused transport but SHALL not start one. These actions SHALL not answer a visible question.

#### Scenario: PLAY does not toggle an active preview
- **WHEN** a selected Files or Chop preview plays and the owner taps PLAY twice
- **THEN** the preview keeps playing after the second tap without a new start
- **AND** one STOP tap ends the preview without changing the transport state

#### Scenario: Preview buttons leave a question intact
- **WHEN** a replacement question is visible and PLAY or STOP arrives on Files or Chop
- **THEN** the applicable preview starts or stops without confirming the question
- **AND** its target and answer remain available

#### Scenario: STOP cancels a pending Files preview
- **WHEN** the owner taps STOP on Files before the selected preview finishes loading
- **THEN** that load cannot start a preview after STOP
- **AND** a later PLAY can request the selected file again

### Requirement: Context pause safely closes recording

If `stop.context` pauses during A1 CountIn, it SHALL cancel CountIn with transport stopped at the accepted position. If it pauses during A1 Recording, it SHALL finish and retain the accepted pass as one Undo step before the pause takes effect. A pause SHALL not leave a hidden recording state or replay a delayed command after a project switch. A second STOP SHALL reset only after the completion is accepted.

#### Scenario: STOP during recording retains one pass
- **WHEN** an A1 pass records hits and context STOP arrives before the pass ends
- **THEN** recording finishes with its accepted hits and one Undo step
- **AND** transport pauses at the accepted position without an orphaned recording phase

## MODIFIED Requirements

### Requirement: Visible transport state

Every screen SHALL show tempo and play state. Playing transport SHALL show its current bar and beat. Enabled metronome SHALL have a visible indicator that says `click`. A metronome toggle SHALL show a notice «metronome on» or «metronome off». The screen SHALL use audio-thread position rather than estimate time itself. Transport feedback SHALL NOT hide a question, pad mode or save failure.

#### Scenario: Transport feedback follows accepted audio state
- **WHEN** transport plays and tempo changes while the owner switches screens
- **THEN** each screen shows the accepted tempo, play state and current bar and beat
- **AND** metronome state and existing safety feedback remain visible at supported terminal sizes

#### Scenario: Holding PLAY names the metronome
- **WHEN** the transport is stopped and the owner holds PLAY on the SMC-PAD
- **THEN** a notice says «metronome on» and the status shows `click`, not `M`

### Requirement: Context PLAY and STOP keep a paused position

The `play.context` action SHALL start a stopped transport at bar 1 or continue a paused transport from its accepted position. It SHALL not restart a running transport. The `stop.context` action SHALL pause a running transport at its accepted position on the first tap. A second tap while paused SHALL reset to bar 1, beat 1. While the output recording runs, a tap with nothing playing SHALL save the recording and SHALL NOT reset the position. These actions SHALL preserve manually played one-shot tails and existing effect tails. Space and the existing explicit stop and toggle actions SHALL keep their reset behavior.

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

#### Scenario: Second STOP during the output recording saves it
- **WHEN** the output recording runs, A1 plays beyond bar 1, and the owner taps STOP twice
- **THEN** the first tap pauses A1 and the second tap saves the recording
- **AND** the paused position stays at the accepted beat

### Requirement: Context buttons keep previews separate from transport

On Files and Chop, `play.context` SHALL start the selected preview only when no preview plays. It SHALL not stop or restart an active preview. `stop.context` SHALL stop that preview without starting it. On Files, STOP SHALL also cancel a pending preview load. With no preview, STOP MAY reset a paused transport but SHALL not start one. While the output recording runs and no preview or transport plays, STOP SHALL save the recording. These actions SHALL not answer a visible question.

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

#### Scenario: STOP on Chop stops the preview before the recording
- **WHEN** the output recording runs and a Chop preview plays, and the owner taps STOP twice
- **THEN** the first tap stops the preview and the second tap saves the recording

### Requirement: Pattern accepts taps for tempo

Pattern SHALL offer Tap Tempo on its action page, P8. Alt+B SHALL use
the same action. Tempo SHALL use the mean interval of up to four taps.
A gap longer than two seconds SHALL start a new sequence.
The first tap SHALL leave tempo as it was, and the mode line SHALL say «TAP · 1 hit · tap again». Intervals under 200 ms SHALL be ignored.
The program SHALL use MIDI receipt times. Tap Tempo SHALL act only in Pattern
or its controls.

#### Scenario: Pattern taps use the beat interval
- **WHEN** two Pattern tap presses arrive 400 ms apart
- **THEN** the tempo becomes 150 BPM after the second press
- **AND** the four most recent taps determine the average interval
- **AND** a gap longer than two seconds or a press outside Pattern changes no tempo
- **AND** Alt+B opens the same tap sequence

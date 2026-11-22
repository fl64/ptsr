# transport Specification

## Purpose

Provide a project beat clock for metronome playback and later pattern playback and recording. Keep timing independent of interface redraws.

## Requirements

### Requirement: Project beat clock

The system SHALL provide a 4/4 beat clock at 960 ticks per quarter note. Tempo SHALL range from 40.0 to 300.0 BPM in 0.1 BPM steps, with 120.0 BPM as the default. New sessions SHALL start stopped at bar 1, beat 1. Clock timing SHALL follow processed audio frames, not interface updates.

#### Scenario: New transport starts stopped
- **WHEN** the program opens a new session
- **THEN** tempo is 120.0 BPM and position is bar 1, beat 1 with playback stopped
- **AND** no metronome sounds

#### Scenario: Audio frames advance the running clock
- **WHEN** playback runs for two seconds at 120 BPM with no interface updates
- **THEN** the clock advances by four quarter notes independent of audio block size

### Requirement: Transport play and stop

Starting SHALL run from bar 1, beat 1. Stopping SHALL reset position to bar 1, beat 1 without stopping manually played pad tails, previews or effects. Repeated explicit start or stop requests SHALL be idempotent. Consecutive toggle requests SHALL act in their accepted order.

#### Scenario: Stop resets position and restart starts on the first beat
- **WHEN** the owner stops a running clock and starts it again
- **THEN** the stopped position is zero and the restarted clock starts on its first beat

#### Scenario: Transport stop preserves a manually played tail
- **WHEN** the owner stops transport while a one-shot pad sounds
- **THEN** the pad and its effect tail continue under the existing playback rules

#### Scenario: Consecutive transport requests retain their order
- **WHEN** two toggles reach the audio thread before its next block
- **THEN** both toggles act in order and the final play state equals the initial state
- **AND** repeated explicit start or stop requests do not restart an already running clock

### Requirement: Tempo changes preserve position

A tempo change SHALL take effect at an audio block boundary without resetting musical position or repeating a beat. Non-finite tempo commands SHALL leave the previous tempo unchanged. Finite commands outside the range SHALL stop at the nearest limit. Accepted tempo SHALL round to 0.1 BPM.

#### Scenario: Tempo changes without resetting the beat
- **WHEN** tempo changes from 120 to 90 BPM between beats
- **THEN** position remains continuous and later beats follow 90 BPM without a missing or doubled click

#### Scenario: Invalid tempo cannot damage the clock
- **WHEN** the engine receives non-finite tempo or a finite value outside 40–300 BPM
- **THEN** non-finite input keeps the previous tempo and finite input stops at the applicable range limit
- **AND** clock position and rendered output remain finite

### Requirement: Accented metronome

The metronome SHALL be off by default. While enabled and playing, it SHALL sound the existing click on each quarter note. Relative velocity SHALL be 1.0 on beat 1 and 0.6 on beats 2–4. Each onset SHALL be within one output frame of its beat at all supported block lengths. Muting the metronome SHALL NOT stop or reset the clock.

#### Scenario: Metronome beats and accents follow the clock
- **WHEN** eight bars play at 120 BPM with the metronome enabled
- **THEN** each of the 32 clicks starts within one output frame of its beat
- **AND** beats 2–4 have 60 percent of beat 1's amplitude with silence between click tails

#### Scenario: Fast tempo retains exact metronome accents
- **WHEN** the metronome runs at 170, 172 or 180 BPM
- **THEN** click timing remains within one output frame and every first beat has the stronger accent

#### Scenario: Muting and enabling the metronome preserves phase
- **WHEN** the owner disables the metronome, lets transport run and enables it between beats
- **THEN** the clock keeps its position and the next click occurs on the next beat
- **AND** missed clicks do not play later

### Requirement: Metronome does not consume pad voices

The beat clock and automatic metronome SHALL NOT allocate, free memory, wait for locks, log or access files in the audio callback. Automatic clicks SHALL NOT steal manually played pad voices. Zero-length processing SHALL NOT advance position or consume a beat.

#### Scenario: Transport remains allocation free under active control
- **WHEN** transport runs while tempo, metronome and play state change through real-time commands
- **THEN** the callback performs no allocation or deallocation

#### Scenario: Automatic clicks preserve full pad polyphony
- **WHEN** 32 pad voices sound while the automatic metronome runs
- **THEN** no pad voice starts fading merely because a metronome beat occurs

#### Scenario: Empty audio blocks preserve the first beat
- **WHEN** zero-length blocks occur before the first non-empty block after starting
- **THEN** position remains zero and the first click starts with the first output frame

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

### Requirement: Context STOP saves the output recording

While the output recording runs, a `stop.context` tap with nothing playing SHALL save the recording. It SHALL NOT reset the position.

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

### Requirement: Context pause safely closes recording

If `stop.context` pauses during A1 CountIn, it SHALL cancel CountIn with transport stopped at the accepted position. If it pauses during A1 Recording, it SHALL finish and retain the accepted pass as one Undo step before the pause takes effect. A pause SHALL not leave a hidden recording state or replay a delayed command after a project switch. A second STOP SHALL reset only after the completion is accepted.

#### Scenario: STOP during recording retains one pass
- **WHEN** an A1 pass records hits and context STOP arrives before the pass ends
- **THEN** recording finishes with its accepted hits and one Undo step
- **AND** transport pauses at the accepted position without an orphaned recording phase

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

### Requirement: The click has an independent monitor path

The system SHALL send the automatic click and A1 count-in to the device after the master stage. The master meter and sources for recording and render SHALL exclude the click. Panic SHALL also stop the click.

#### Scenario: The click plays without moving the master meter

- **WHEN** the click plays in an empty project, in transport or during A1 count-in
- **THEN** the device plays the click at its scheduled time
- **AND** the master meter stays at zero

#### Scenario: Panic stops the click path

- **WHEN** the owner uses Panic while a click or count-in sounds
- **THEN** it stops with the other device sound within the Panic fade time

### Requirement: The click has its own level

The Audio settings SHALL save a click level from 0 to 100 percent. It SHALL start at 100 percent for an older configuration. Keyboard and the controller SHALL change the same setting. The device SHALL smooth the gain change. The level SHALL set the gain of the click and count-in while the beat position and master level stay the same.

#### Scenario: Click level changes independently

- **WHEN** the owner changes Click level during playback with the keyboard or a controller
- **THEN** only the click level changes and the next beat is on time
- **AND** the new level stays after restart

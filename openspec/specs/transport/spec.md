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

Every screen SHALL show tempo and play state. Playing transport SHALL show its current bar and beat. Enabled metronome SHALL have a visible indicator. The screen SHALL use audio-thread position rather than estimate time itself. Transport feedback SHALL NOT hide a question, pad mode or save failure.

#### Scenario: Transport feedback follows accepted audio state
- **WHEN** transport plays and tempo changes while the owner switches screens
- **THEN** each screen shows the accepted tempo, play state and current bar and beat
- **AND** metronome state and existing safety feedback remain visible at supported terminal sizes

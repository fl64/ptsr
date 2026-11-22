## MODIFIED Requirements

### Requirement: Accented metronome

The metronome SHALL be off by default. While enabled and playing, it SHALL sound the existing click on each quarter note. Relative velocity SHALL be 1.0 on beat 1 and 0.6 on beats 2–4. Each onset SHALL be within one output frame of its beat plus the fixed device delay at all supported block lengths. Muting the metronome SHALL NOT stop or reset the clock.

#### Scenario: Metronome beats and accents follow the clock
- **WHEN** eight bars play at 120 BPM with the metronome enabled
- **THEN** each of the 32 clicks starts within one output frame of its beat plus the fixed device delay
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
- **THEN** position remains zero and the first click starts at the fixed device delay after the first output frame

### Requirement: The click has an independent monitor path

The system SHALL send the automatic click and A1 count-in to the device after the master stage. The click SHALL add to the limited master after the fixed device delay. Monitor headroom MAY reduce the cue gain to keep the combined device mix within full scale. The master meter and sources for recording and render SHALL exclude the click. Panic SHALL also stop the click.

#### Scenario: The click plays without moving the master meter

- **WHEN** the click plays in an empty project, in transport or during A1 count-in
- **THEN** the device plays the click at its scheduled time
- **AND** the master meter stays at zero

#### Scenario: Panic stops the click path

- **WHEN** the owner uses Panic while a click or count-in sounds
- **THEN** it stops with the other device sound within the Panic fade time

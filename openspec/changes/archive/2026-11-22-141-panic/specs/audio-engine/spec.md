## ADDED Requirements

### Requirement: Panic stops playback and retains accepted work

Panic SHALL stop transport and count-in at the audio boundary. All active samples, preview, metronome and effect tails SHALL fade to silence in at most five milliseconds. Panic SHALL preserve pad assignments, patterns and sound settings. It SHALL finish a live pattern pass at the stop boundary and finish an audio recording through its normal save path. Later hits SHALL play.

#### Scenario: Panic fades the whole sampler

- **WHEN** Panic reaches the audio boundary during playback with active voices, a preview, and effect tails
- **THEN** the clock stops and the old sound reaches silence in at most five milliseconds
- **AND** a later hit plays without a returning effect tail

#### Scenario: Panic retains the recorded take

- **WHEN** Panic stops an active A1 pass or a sound recording
- **THEN** the current take finishes through its usual path and retains accepted data
- **AND** a count-in ends without adding a hit

#### Scenario: Project restore completes during a Panic fade

- **WHEN** the project changes while Panic fades or clears old effects
- **THEN** the restored project can play and its output contains no old tail

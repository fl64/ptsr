## ADDED Requirements

### Requirement: Pad envelope
The system SHALL give each pad an envelope of three times in milliseconds: attack (fade-in at the start of the piece), decay (the last `decay` ms of the piece fade to silence, the piece still ends at the same place) and release (in gate mode the sound fades over it after the release). On the pads screen `Tab` steps through the parameters volume → pitch → attack → decay → release and `−` `=` change the picked one for the change scope; the changed value pops up. Every envelope change on a sounding pad is click-free.

#### Scenario: Attack at the start
- **WHEN** the owner raises the attack of a pad to 50 ms and hits it
- **THEN** the sound comes in softly over 50 ms, without a click

#### Scenario: Decay ends at the end of the piece
- **WHEN** the owner sets a decay of 200 ms on a one-shot pad
- **THEN** the last 200 ms of the piece fade to silence and the pad ends at the same place as before, in silence

#### Scenario: Release after the gate
- **WHEN** a gate pad with a release of 300 ms is released
- **THEN** the sound fades out over 300 ms, without a click

#### Scenario: The envelope follows the scope
- **WHEN** the scope is bank 2 and the owner raises the release
- **THEN** every pad of bank 2 gets the longer release and the panel names the scope

#### Scenario: Old session
- **WHEN** the session file was saved before the envelope existed
- **THEN** it opens and every pad sounds as it did: no attack, no decay, the same release as today

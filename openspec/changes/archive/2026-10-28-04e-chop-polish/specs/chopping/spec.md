## ADDED Requirements

### Requirement: Chop screen readability
The system SHALL show on the chop screen, without cutting text: the plan summary in full, the selected slice both on the waveform and on the grid of the listen mode, empty pads in the assign mode as only "empty", and in the overwrite question each pad as "bank B, pad K: what → new".

#### Scenario: Selected slice on the grid
- **WHEN** the owner hits pad W in the listen mode
- **THEN** pad W is highlighted as selected on the grid

#### Scenario: Plain overwrite question
- **WHEN** the plan puts slice 3 on pad 3 of bank 1 that holds "kick" and the owner applies it
- **THEN** the question says "bank 1, pad 3: kick → slice 3"

### Requirement: Equal parts go on after a restart
The system SHALL step equal parts on from the current number of slices when it is one of 2, 4, 8, 16, 32, also for a chop restored from the session.

#### Scenario: Eight slices after a restart
- **WHEN** a sample was cut into 8 equal parts, the program was restarted and the owner presses the equal-parts command
- **THEN** the sample is cut into 16 parts

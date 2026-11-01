## MODIFIED Requirements

### Requirement: Chop screen readability
The system SHALL show on the chop screen, without cutting text: the plan summary in full, the selected slice both on the waveform and, while the pads are shown, on the grid of the listen mode, empty pads in the assign mode as only "empty", and in the overwrite question each pad as "bank B, pad K: what → new".

#### Scenario: Selected slice on the grid
- **WHEN** the pads are shown and the owner hits pad W in the listen mode
- **THEN** pad W is highlighted as selected on the grid

#### Scenario: Selected slice with the pads hidden
- **WHEN** the pads are hidden and the owner hits pad W in the listen mode
- **THEN** the pad line names pad W and its slice, and the slice is selected on the waveform

#### Scenario: Plain overwrite question
- **WHEN** the plan puts slice 3 on pad 3 of bank 1 that holds "kick" and the owner applies it
- **THEN** the question says "bank 1, pad 3: kick → slice 3"

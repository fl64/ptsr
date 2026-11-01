## ADDED Requirements

### Requirement: The cuts belong to the file
The system SHALL show on the chop screen whose cuts these are and who plays them: the
header names the file ("cuts of break.wav"), every slice shows the pads that play it, and
opening the screen from a pad that plays a slice selects that slice. Pads that play `all`
or `pad` stand under no slice.

#### Scenario: Open from a slice pad
- **WHEN** slices 1–8 of a break lie on pads 5–12 and the owner presses F3 on pad 7
- **THEN** the header says "cuts of break.wav", slice 3 is selected, and every slice shows its pad

#### Scenario: Open from the whole-file pad
- **WHEN** pad 1 plays the whole break and the owner presses F3 on it
- **THEN** the same cuts are shown with their pads, and pad 1 stands under no slice

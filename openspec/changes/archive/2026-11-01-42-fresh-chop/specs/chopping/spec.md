## MODIFIED Requirements

### Requirement: The cuts belong to the file
The system SHALL show on the chop screen whose cuts these are and who plays them: the
header names the file ("cuts of break.wav"), every slice shows the pads that play it, and
opening the screen from a pad that plays a slice selects that slice. Pads that play `all`
or `pad` stand under no slice. Opening the screen from a pad that plays the whole file
starts a fresh chop of it (42): no cuts, the whole file; the pads of the old cuts keep
playing what they played and let go of them, and undo brings the old cuts and links back.

#### Scenario: Open from a slice pad
- **WHEN** slices 1–8 of a break lie on pads 5–12 and the owner presses F3 on pad 7
- **THEN** the header says "cuts of break.wav", slice 3 is selected, and every slice shows its pad

#### Scenario: Open from the whole-file pad
- **WHEN** slices of a break lie on pads 2-1…2-8, pad 1-1 plays the whole break and the owner presses F3 on pad 1-1
- **THEN** the break opens with no cuts, the pads 2-1…2-8 play what they played and say `pad`, and the status says that undo brings the old chop back

#### Scenario: Undo the fresh chop
- **WHEN** the owner presses `Ctrl+Z` right after a fresh chop opened
- **THEN** the old cuts are back and the pads 2-1…2-8 follow their slices again

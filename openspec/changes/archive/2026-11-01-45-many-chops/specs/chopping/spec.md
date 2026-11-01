## ADDED Requirements

### Requirement: Several chops of one file
The system SHALL keep every chop of a file that a pad plays a slice of. A fresh chop of
the file leaves the older chops and their pads as they are; F3 on a pad opens the chop its
slice came from, and moving a cut there moves only that chop's pads. The newest chop is
the one the files screen opens. A chop no pad plays a slice of is dropped once a newer
chop of the file exists. The chop screen header names the chop when the file has more
than one: `cuts of break.wav · chop 2 of 3`. The chops are saved with the project.

#### Scenario: Chop the same sample again
- **WHEN** slices 1–8 of a break lie on pads 2-1…2-8, the owner opens the whole-file pad 1-1 with F3, cuts the break in 4 and lays them on pads 3-1…3-4
- **THEN** pads 2-1…2-8 still say `slice 1`…`slice 8` and play what they played, pads 3-1…3-4 say `slice 1`…`slice 4`

#### Scenario: Back to an older pad
- **WHEN** after that the owner presses F3 on pad 2-3
- **THEN** the chop screen shows the 8 slices of the first chop with slice 3 selected, and the header says `chop 1 of 2`

#### Scenario: A cut moves only its own pads
- **WHEN** the owner moves the start of slice 3 in that first chop
- **THEN** pad 2-3 starts later and pads 3-1…3-4 do not change

#### Scenario: Saved and opened
- **WHEN** the project with both chops is saved and opened again
- **THEN** F3 on pad 2-3 opens the first chop and F3 on pad 3-2 the second, each with its cuts

#### Scenario: A project saved before
- **WHEN** a project saved before this change has pads linked to a chop that is no longer in it, and the owner presses F3 on one of them
- **THEN** the chop opens with the cuts at the starts and ends of those pads, and the pads say `slice N` again

## MODIFIED Requirements

### Requirement: The cuts belong to the file
The system SHALL show on the chop screen whose cuts these are and who plays them: the
header names the file ("cuts of break.wav"), every slice shows the pads that play it, and
opening the screen from a pad that plays a slice selects that slice. Pads that play `all`
or `pad` stand under no slice. Opening the screen from a pad that plays the whole file
starts a fresh chop of it (42): no cuts, the whole file; the older chop stays with its
pads, they keep their slices and follow its cuts (45), and undo brings the older chop
back on the screen.

#### Scenario: Open from a slice pad
- **WHEN** slices 1–8 of a break lie on pads 5–12 and the owner presses F3 on pad 7
- **THEN** the header says "cuts of break.wav", slice 3 is selected, and every slice shows its pad

#### Scenario: Open from the whole-file pad
- **WHEN** slices of a break lie on pads 2-1…2-8, pad 1-1 plays the whole break and the owner presses F3 on pad 1-1
- **THEN** the break opens with no cuts, the pads 2-1…2-8 play what they played and still say `slice 1`…`slice 8`, and the status says that the older chop stays with its pads

#### Scenario: Undo the fresh chop
- **WHEN** the owner presses `Ctrl+Z` right after a fresh chop opened
- **THEN** the older chop is on the screen again with its cuts, and the pads 2-1…2-8 follow their slices as all along

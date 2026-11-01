## ADDED Requirements

### Requirement: Open on your slice
The system SHALL open the chop screen from a pad that plays a slice with the view on that slice: the slice in the middle, half of its length shown on each side; from a pad that plays the whole file the view is left as it was. The zoom out shows the whole file again.

#### Scenario: Open on your slice
- **WHEN** slice 5 of a break lies on pad 2-Q and the owner presses F3 on it
- **THEN** the view shows slice 5 in its middle with half of it on each side, slice 5 is selected, and zooming out reaches the whole file

### Requirement: The source is marked
The system SHALL mark on the chop screen where the whole file lies: the bank row puts `◆` at every bank that holds a pad playing this file whole, and on the grid such a pad says `◆ all` and is not dimmed.

#### Scenario: The source is marked
- **WHEN** pad 1-1 plays the whole break, its slices lie on bank 2 and the owner opens the chop screen from pad 2-Q
- **THEN** the bank row shows `◆` at bank 1; after `Ctrl+1` pad 1-1 says `◆ all` and is not dimmed

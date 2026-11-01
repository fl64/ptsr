## MODIFIED Requirements

### Requirement: Open on your slice
The system SHALL open the chop screen from a pad that plays a part of the file with the
view on that part: the part in the middle, half of its length shown on each side, and the
slice where the part starts selected. This holds for a pad that follows the cuts of the
chop and for a pad that plays its own piece (its start or end moved by hand, or its
source set to the pad). From a pad that plays the whole file the view is left as it was. The zoom out
shows the whole file again.

#### Scenario: Open on your slice
- **WHEN** slice 5 of a break lies on pad 2-Q and the owner presses F3 on it
- **THEN** the view shows slice 5 in its middle with half of it on each side, slice 5 is selected, and zooming out reaches the whole file

#### Scenario: Open on your own piece
- **WHEN** pad 7 played slice 3 of a break, the owner moved its start 2 ms later on the
  SOURCE page, closed the edit and presses F3 on it
- **THEN** the view shows the piece of pad 7 in its middle with half of it on each side,
  slice 3 (where the piece starts) is selected, and the chop keeps its cuts

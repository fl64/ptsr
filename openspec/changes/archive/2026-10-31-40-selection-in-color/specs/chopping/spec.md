## MODIFIED Requirements

### Requirement: The chop wave is colored by frequency
The system SHALL color the wave of the selected slice of the chop screen per column by the band that stands out against its own average over the file — the low red, the mid green, the high blue, the band most above its average taking most of the color — and SHALL draw the other slices of the region grey at the same brightness, so the color itself shows where the owner is; outside the region the wave is dim. The picture mode uses the same colors; the strips of the other screens and the pad waves are not colored.

#### Scenario: Kick and hat by color
- **WHEN** a break with a kick and a hat is open on the chop screen and the slice with them is selected
- **THEN** the columns of the kick are red and the columns of the hat are blue, also when the three bands of the break are nearly even on average

#### Scenario: The selection stays visible
- **WHEN** slice 3 is selected
- **THEN** the wave of slice 3 is in color and the other slices are grey, with no background plate

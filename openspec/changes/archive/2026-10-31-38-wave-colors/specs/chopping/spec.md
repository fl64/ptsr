## ADDED Requirements

### Requirement: The chop wave is colored by frequency
The system SHALL color the wave of the chop screen per column by the band that stands out against its own average over the file — the low red, the mid green, the high blue, the band most above its average taking most of the color — and SHALL show the selected slice on a dark tint of the selection cyan, in the wave rows and in the row of slice numbers above them, so the colors stay full in every slice; outside the region the wave is dim. The picture mode uses the same colors; the strips of the other screens and the pad waves are not colored.

#### Scenario: Kick and hat by color
- **WHEN** a break with a kick and a hat is open on the chop screen
- **THEN** the columns of the kick are red and the columns of the hat are blue, also when the three bands of the break are nearly even on average

#### Scenario: The selection stays visible
- **WHEN** slice 3 is selected
- **THEN** the cells of slice 3 have a cyan-tinted background, unlike the other slices, in the wave and in the numbers row, and the wave colors of every slice are equally bright

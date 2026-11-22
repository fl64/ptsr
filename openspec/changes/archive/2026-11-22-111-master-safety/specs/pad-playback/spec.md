## MODIFIED Requirements

### Requirement: Pitch Gate uses the next grid boundary

An eligible note SHALL start its existing short cut at the first pattern grid line strictly after its compiled onset. Empty steps SHALL supply boundaries. Quantize Off SHALL use 1/16 for this gate. Strength SHALL NOT move grid lines. Straight and triplet grids SHALL retain their exact spacing. The loop end SHALL also be a boundary.

#### Scenario: Pitch Gate cuts at empty straight and triplet steps
- **WHEN** eligible notes play before empty steps on every supported straight or triplet grid
- **THEN** each cut starts within one output frame of the next grid line plus the fixed device delay, even with no hit on that line
- **AND** Quantize Off uses 1/16 and changing strength does not move the gate grid

#### Scenario: Pitch Gate respects off-grid onsets and the loop end
- **WHEN** eligible hits lie on a line, between lines or just before the loop seam
- **THEN** each gate uses the strictly later line or the loop end
- **AND** seam releases precede new tick-zero notes without cutting those new notes

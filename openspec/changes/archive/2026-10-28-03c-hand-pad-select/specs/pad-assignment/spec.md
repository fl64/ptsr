## ADDED Requirements

### Requirement: Choosing the pad with arrows
The system SHALL, while a file is in hand on the files screen, move the selected pad with the arrow keys instead of the file list and put the file on the selected pad with Enter.

#### Scenario: Arrows and Enter
- **WHEN** a file is in hand, pad 1 is selected and the owner presses → and Enter
- **THEN** the file goes to pad 2 (after the overwrite question if pad 2 holds a sample), the file list selection does not move

#### Scenario: A pad hit still works
- **WHEN** a file is in hand and the owner hits pad W
- **THEN** the file goes to pad W, as before

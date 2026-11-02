## MODIFIED Requirements

### Requirement: The mute group is seen on the pad
The system SHALL draw the mute group of a pad on its frame as its digit, dark on a light
plate (`1`…`8`), after the other glyphs of the pad, and F1 SHALL explain it. A pad with
the group off has no such digit.

#### Scenario: The group on the pad
- **WHEN** pad Q is in group 1 and pad W has the group off
- **THEN** the frame of pad Q shows `1` dark on a light plate, the frame of pad W shows no
  group digit, and F1 has a row for `1…8`

#### Scenario: The group digit keeps the frame
- **WHEN** pad Q plays a slice, has its envelope set and is in group 1
- **THEN** its top frame shows ` ◻⩘ ` and the plate `1` before the corner, and the corner
  stands in its own cell

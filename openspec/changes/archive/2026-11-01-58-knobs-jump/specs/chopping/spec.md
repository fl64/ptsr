## MODIFIED Requirements

### Requirement: Chop knobs
The system SHALL let learned controller knobs move the chop cursor, zoom and region: the
`chop.cursor_pos` knob puts the cursor inside the region, the `chop.zoom` knob sets the
visible span between the region and the whole file on a log scale around the cursor, and
the `chop.region_start_pos` / `chop.region_end_pos` knobs move the region edges, keeping
the region at least 10 ms long; absolute knobs catch their value as the knob response
says. The buttons `chop.cursor_left` / `chop.cursor_right` step the cursor as `J` `L` do.
All are listed in the MIDI learn under CHOP. The X-Session Pro profile puts `chop.zoom` on
k12.

#### Scenario: Zoom knob
- **WHEN** the owner turns the zoom knob
- **THEN** the view spans from the whole file to close-ups smoothly; with the response
  `pickup` grabbing the knob from another position does not jump the view, with `jump`
  the view goes to the knob position at once

#### Scenario: Region knobs
- **WHEN** the owner turns the learned region knobs
- **THEN** the edges follow as the knob response says, and the region never gets shorter
  than 10 ms

#### Scenario: Cursor knob
- **WHEN** the owner turns the learned cursor knob
- **THEN** the cursor moves inside the region, the slice under it is selected

#### Scenario: Cursor buttons
- **WHEN** the owner presses the learned "cursor right" button
- **THEN** the cursor moves one step right, as with `L`

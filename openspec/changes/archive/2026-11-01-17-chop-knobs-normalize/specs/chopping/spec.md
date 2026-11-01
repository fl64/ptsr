## ADDED Requirements

### Requirement: Chop knobs
The system SHALL let learned controller knobs move the chop cursor, zoom and region: the
`chop.cursor_pos` knob puts the cursor inside the region, the `chop.zoom` knob sets the
visible span between the region and the whole file on a log scale around the cursor, and
the `chop.region_start_pos` / `chop.region_end_pos` knobs move the region edges, keeping
the region at least 10 ms long; absolute knobs use soft takeover. The buttons
`chop.cursor_left` / `chop.cursor_right` step the cursor as `J` `L` do. All are listed in
the MIDI learn under CHOP. The X-Session Pro profile puts `chop.zoom` on k12.

#### Scenario: Zoom knob
- **WHEN** the owner turns the zoom knob
- **THEN** the view spans from the whole file to close-ups smoothly, and grabbing the knob from another position does not jump the view

#### Scenario: Region knobs
- **WHEN** the owner turns the learned region knobs
- **THEN** the edges follow with soft takeover, and the region never gets shorter than 10 ms

#### Scenario: Cursor knob
- **WHEN** the owner turns the learned cursor knob
- **THEN** the cursor moves inside the region, the slice under it is selected

#### Scenario: Cursor buttons
- **WHEN** the owner presses the learned "cursor right" button
- **THEN** the cursor moves one step right, as with `L`

### Requirement: Normalize
The system SHALL, on `Ctrl+N` on the chop screen, set the volume of the pad the chop
screen was opened from so the loudest peak of what that pad plays sits at −1 dB or just
below (the volume is whole dB, rounded down), within the pad volume range (up to +6 dB);
the status shows the gain and says when the limit stopped it. The file on disk never
changes. When the chop screen was opened without a pad, the status says there is no pad
to set the volume on.

#### Scenario: Quiet break
- **WHEN** the owner opens a pad whose peaks are 6 dB below full scale and presses `Ctrl+N`
- **THEN** the pad volume is +5 dB, the status shows "+5 dB", and the pad plays without clipping

#### Scenario: Very quiet break
- **WHEN** the peaks of the pad are 20 dB below full scale and the owner presses `Ctrl+N`
- **THEN** the pad volume is +6 dB and the status says the volume limit stopped it

#### Scenario: Twice in a row
- **WHEN** the owner presses `Ctrl+N` a second time
- **THEN** the gain is measured again from the sound, so the level does not climb twice

#### Scenario: Chop without a pad
- **WHEN** the chop screen was opened from the files screen and the owner presses `Ctrl+N`
- **THEN** the status says there is no pad to set the volume on

## MODIFIED Requirements

### Requirement: A1 tap tempo keeps the bank row visible

On A1, the notice row SHALL show the first keyboard tap and the BPM after an accepted second tap. The first cue SHALL expire after two seconds or on the second accepted tap. It SHALL take the place of an earlier tempo result. The bank and step rows SHALL remain visible. Warnings and errors SHALL keep priority.

#### Scenario: A1 tap tempo uses the notice row
- **WHEN** the owner taps tempo twice on A1 at 80x26 or 120x40
- **THEN** a notice names the new tempo and the bank and step rows remain complete
- **AND** a higher-priority warning keeps the notice row

#### Scenario: A1 shows the first keyboard tempo tap without a controller
- **WHEN** the owner presses Alt+B once on A1 without an SMC-PAD at 80x26 or 120x40
- **THEN** the notice row says «TAP · 1 hit · tap again» and the tempo stays the same
- **AND** the bank and step rows remain complete
- **AND** the cue disappears after its tap window or on an accepted second tap

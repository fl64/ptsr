## ADDED Requirements

### Requirement: Auto-chop by hits
The system SHALL chop a sample at the starts of hits on `Ctrl+Y`, with a threshold below
the loudest hit: every press steps the threshold lower (−6, −12, −18, −24, −30 dB, then
−6 again), so quieter hits get their cuts. Each press replaces the cuts as equal parts do
and is one undo step. The knob `chop.threshold` sets the threshold continuously, and the
cuts follow it at once. The status names the threshold and the number of cuts.

#### Scenario: Break by hits
- **WHEN** a break is on the chop screen and the owner presses Ctrl+Y
- **THEN** cuts land on the starts of the loud hits, and the status says how many

#### Scenario: More presses, more cuts
- **WHEN** the owner presses Ctrl+Y again
- **THEN** the quieter hits get their cuts too; Ctrl+Z brings back the cuts of the press before

#### Scenario: The threshold on a knob
- **WHEN** the owner turns the knob bound to `chop.threshold`
- **THEN** cuts are added or removed at once as the threshold moves

### Requirement: Snap to hits
The system SHALL, when snapping is on, move a marker set in time to the nearest hit within 90 ms.

#### Scenario: Cut slightly off the hit
- **WHEN** the sample plays, snapping is on, the owner presses "cut" 40 ms after a hit
- **THEN** the marker lands exactly on the hit, the status says "marker snapped to hit"

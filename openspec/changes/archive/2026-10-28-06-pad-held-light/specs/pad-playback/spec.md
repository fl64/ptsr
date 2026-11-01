## MODIFIED Requirements

### Requirement: Pads screen
The system SHALL show the pads screen on F2: a 4×4 grid with keys and sound names, the current bank, the selected pad with its mode and volume, and the pad lit while it is held (at least a short flash for a quick hit). Hitting a pad selects it.

#### Scenario: Hit selects the pad
- **WHEN** the owner hits pad W
- **THEN** pad 6 flashes, becomes selected, its mode and volume are shown on the right

#### Scenario: Held pad stays lit
- **WHEN** the owner holds pad Q for 2 seconds, from the keyboard or a controller
- **THEN** pad Q stays lit for the 2 seconds and goes dark right after the release

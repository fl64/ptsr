## ADDED Requirements

### Requirement: By hits says what it replaced
The system SHALL, when `Ctrl+Y` chops by hits over existing cuts, say in the status the
threshold, the number of slices now and before, and that `Ctrl+Z` brings them back:
`by hits at -12 dB: 7 slices (were 16) · Ctrl+Z brings them back`. With no cuts before
nothing was replaced: the line ends after the number of slices.

#### Scenario: By hits says what it replaced
- **WHEN** the owner cut a break into 16 slices by hand and presses `Ctrl+Y`
- **THEN** the status says `by hits at -6 dB: N slices (were 16) · Ctrl+Z brings them
  back`, and `Ctrl+Z` brings the 16 slices back

### Requirement: The header shows play
The system SHALL show `▶` in the chop screen header next to `LISTEN` while the chop
screen sounds, and drop it when the sound stops.

#### Scenario: The header shows play
- **WHEN** the chop screen is open and the owner presses `Space`
- **THEN** the header says `LISTEN ▶`; after the second `Space` it says `LISTEN`

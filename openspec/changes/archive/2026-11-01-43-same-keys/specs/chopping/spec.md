## ADDED Requirements

### Requirement: Banks on the chop screen
The system SHALL turn the banks on the chop screen with `PgUp` `PgDn` and `Ctrl+1`…`8`,
as on every screen with pads: in the listen mode, in the assign mode and in the slice
edit. Until a slice of this file lies on a pad, the same keys page the keyboard of slices.

#### Scenario: The next bank while chopping
- **WHEN** bank 1 is shown on the chop screen in the listen mode and the owner presses `PgDn`
- **THEN** bank 2 is shown and the cursor stays where it was

#### Scenario: The same keys while assigning
- **WHEN** the assign mode is open on bank 1 and the owner presses `PgDn`
- **THEN** bank 2 is shown, as in the listen mode

#### Scenario: Pages of the keyboard of slices
- **WHEN** a sample is cut into 20 slices, none lies on a pad, and the owner presses `PgDn`
- **THEN** the row over the grid says `slices 17–20` and the first pad key plays slice 17

## MODIFIED Requirements

### Requirement: Cursor modes
The system SHALL move the cursor with `J` `L` (and ← →) inside the region in one of three modes, switched in turn with the snap command `N` and shown in the help line: even steps, hit to hit, cut to cut; the chop screen opens in even steps. In even steps a tap moves the cursor by 1/128 of the visible window (never less than a frame) and a held key speeds up over 6 repeats to 1/32 of the window per repeat; the slice bounds step the same, so the cursor and the bounds move alike at any zoom. In hits mode it jumps to the start of the next or previous hit when that hit is on the screen, and moves in even steps while no hit lies ahead on the screen. In cuts mode it jumps to the next cut to the right, or to the end of the region when no cut lies ahead, and to the previous cut to the left, or to the start of the region; the slice under the cursor is selected, the page follows it and the slice plays. A finer step comes from zooming in: the step follows the view. The cut command cuts at the cursor also on pause.

#### Scenario: Steps follow the view
- **WHEN** the view shows the whole file and the owner presses →
- **THEN** the cursor moves by 1/128 of the file; after zooming in the same press moves it by 1/128 of the smaller view

#### Scenario: Even steps on a break
- **WHEN** the chop screen opens on a break of decaying hits and the owner presses → twelve times
- **THEN** the cursor moves twelve even steps, a peak does not hold it back

#### Scenario: Hit to hit
- **WHEN** hits mode is on and the owner presses → and then ←
- **THEN** the cursor lands on the start of the next hit and then back on the start of the previous one

#### Scenario: No hits on the screen
- **WHEN** hits mode is on and no hit lies ahead of the cursor on the screen
- **THEN** → moves the cursor in even steps until a hit comes into view, and then jumps to it

#### Scenario: Cut to cut
- **WHEN** cuts mode is on, the cursor stands inside slice 2 of 4 and the owner presses `L` twice and then `J`
- **THEN** the cursor lands on the start of slice 3, then of slice 4, then back on the start of slice 3, and each slice plays

#### Scenario: To the ends
- **WHEN** cuts mode is on, the cursor stands in the last slice and the owner presses `L`
- **THEN** the cursor lands on the end of the region; in the first slice `J` lands on its start

#### Scenario: N walks the three modes
- **WHEN** the chop screen opens and the owner presses `N` three times
- **THEN** the help line says `snap: hits`, then `snap: cuts`, then `snap: off`

#### Scenario: Fine step
- **WHEN** the owner zooms in four times with `I` and presses `L`
- **THEN** the cursor moves by 1/128 of the smaller view, a sixteenth of the step on the
  whole file

#### Scenario: Cut on pause
- **WHEN** the playback is stopped and the owner presses the cut command
- **THEN** a cut appears at the cursor, as it does while playing

## REMOVED Requirements

### Requirement: Jump to a cut
**Reason**: `PgUp` `PgDn` turn the banks on every screen (owner, 2026-10-31, "Q1.1").
**Migration**: the cuts mode of the snap command `N`: `J` `L` walk cut to cut.

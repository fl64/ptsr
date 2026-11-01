## REMOVED Requirements

### Requirement: Editing cursor
**Reason**: the snap to the loudest frame on the way holds the cursor on decaying hits.
**Migration**: replaced by "Cursor modes": even steps by default, hit to hit with the snap command.

## ADDED Requirements

### Requirement: Cursor modes
The system SHALL move the cursor with ← → inside the region in one of two modes, switched with the "snap to hits" command and shown in the help line; the chop screen opens in even steps. In even steps the cursor moves by 1/32 of the visible window (never less than a frame). In hits mode it jumps to the start of the next or previous hit when that hit is on the screen, and moves in even steps while no hit lies ahead on the screen. With Ctrl it moves by 1/256 of the window in both modes. The cut command cuts at the cursor also on pause.

#### Scenario: Steps follow the view
- **WHEN** the view shows the whole file and the owner presses →
- **THEN** the cursor moves by 1/32 of the file; after zooming in the same press moves it by 1/32 of the smaller view

#### Scenario: Even steps on a break
- **WHEN** the chop screen opens on a break of decaying hits and the owner presses → twelve times
- **THEN** the cursor moves twelve even steps, a peak does not hold it back

#### Scenario: Hit to hit
- **WHEN** hits mode is on and the owner presses → and then ←
- **THEN** the cursor lands on the start of the next hit and then back on the start of the previous one

#### Scenario: No hits on the screen
- **WHEN** hits mode is on and no hit lies ahead of the cursor on the screen
- **THEN** → moves the cursor in even steps until a hit comes into view, and then jumps to it

#### Scenario: Fine step
- **WHEN** the owner holds Ctrl and presses →
- **THEN** the cursor moves by 1/256 of the view in either mode, ignoring hits

#### Scenario: Cut on pause
- **WHEN** the playback is stopped and the owner presses the cut command
- **THEN** a cut appears at the cursor, as it does while playing

### Requirement: Held keys repeat on the chop screen
The system SHALL repeat the cursor arrows, the cut nudge and zoom while their key is held; pads and the other commands act once per press.

#### Scenario: Hold the arrow
- **WHEN** the owner holds → on the chop screen
- **THEN** the cursor keeps moving until the key is released

#### Scenario: A held pad does not roll
- **WHEN** the owner holds a pad key
- **THEN** the pad plays once

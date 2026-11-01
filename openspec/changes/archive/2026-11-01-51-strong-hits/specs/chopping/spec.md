## ADDED Requirements

### Requirement: One rule for the hits
The system SHALL use one hit threshold on the chop screen for the cursor's hits mode, the
cut that snaps to a hit, the marks of the hits and the chop by hits: a hit counts when it
is at most the threshold below the loudest hit of the region. The threshold is −12 dB
until `Ctrl+Y` or the threshold knob sets it, then theirs. In hits mode every counted hit
on the screen is marked with `▾` on the row over the wave, at its start; the marks show
where the cursor jumps and where `Ctrl+Y` at the current threshold cuts.

#### Scenario: The marks are the stops
- **WHEN** hits mode is on and the screen shows four marks
- **THEN** → from the start of the view stops at the four marks in turn

#### Scenario: Ctrl+Y cuts at the marks
- **WHEN** the owner presses `Ctrl+Y` and then switches hits mode on
- **THEN** a mark stands at every new cut and nowhere else in the region

#### Scenario: No marks in even steps
- **WHEN** the cursor walks in even steps or cut to cut
- **THEN** no marks are drawn

## MODIFIED Requirements

### Requirement: Cursor modes
The system SHALL move the cursor with `J` `L` (and ← →) inside the region in one of three modes, switched in turn with the snap command `N` and shown in the help line: even steps, hit to hit, cut to cut; the chop screen opens in even steps. In even steps a tap moves the cursor by 1/128 of the visible window (never less than a frame) and a held key speeds up over 6 repeats to 1/32 of the window per repeat; the slice bounds step the same, so the cursor and the bounds move alike at any zoom. In hits mode it jumps to the start of the next or previous strong hit when that hit is on the screen, and moves in even steps while no strong hit lies ahead on the screen; a strong hit is one the hit threshold keeps (see "One rule for the hits"). In cuts mode it jumps to the next cut to the right, or to the end of the region when no cut lies ahead, and to the previous cut to the left, or to the start of the region; the slice under the cursor is selected, the page follows it and the slice plays. A finer step comes from zooming in: the step follows the view. The cut command cuts at the cursor also on pause.

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

#### Scenario: Small rises are passed
- **WHEN** hits mode is on and between two hits of a break a sustained sound swells by a few dB
- **THEN** → goes from the first hit to the second, the swell does not stop the cursor

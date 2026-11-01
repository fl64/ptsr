## ADDED Requirements

### Requirement: Loop the play
The system SHALL switch a loop on and off with `Alt+O` on the chop screen and show its
state in the help line next to the play mode (`loop: on`). With the loop on, `Space`
plays what the play mode picks (to the end, to the cut, the slice) from its start again
each time it reaches its end, until `Space` stops it. A cut, a slice bound or a region
edge moved while the loop plays is heard from the next turn on, the play does not stop.
Every turn fades out and in at its seam, no click. With the loop off, `Space` plays once,
as before. The switch starts off when the chop screen opens.

#### Scenario: The slice over and over
- **WHEN** the play mode is "slice", the loop is on, the cursor stands in slice 3 and the owner presses `Space` and waits three times the length of slice 3
- **THEN** slice 3 plays three times in a row and goes on

#### Scenario: Space stops the loop
- **WHEN** the loop plays and the owner presses `Space`
- **THEN** the sound stops

#### Scenario: Move a bound while it loops
- **WHEN** the loop plays slice 3 and the owner moves the end of slice 3 to the left
- **THEN** the next turn ends at the new end, and the sound does not stop in between

#### Scenario: No click at the seam
- **WHEN** a slice that ends loud loops
- **THEN** the render has no step at the seam larger than the slice's own, no NaN, no clipping

#### Scenario: Off plays once
- **WHEN** the loop is off and the owner presses `Space` in the slice mode
- **THEN** the slice plays once and stops

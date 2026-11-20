## ADDED Requirements

### Requirement: The zoom anchor is the point touched last

Each chop SHALL keep a zoom anchor. The anchor SHALL be the cursor, the slice start, the slice end, the region start or the region end. A move of a slice bound or a region edge SHALL make that point the anchor. A cursor move, a pad hit, a slice selection and a new chop SHALL make the cursor the anchor. The anchor SHALL follow its point when an undo moves it. When the anchor point stops to exist, the cursor SHALL become the anchor.

#### Scenario: Zoom around the slice start
- **WHEN** the owner moves the start of the selected slice and then zooms in
- **THEN** the slice start stays at its place of the picture

#### Scenario: Zoom around the slice end with the knob
- **WHEN** the owner turns the slice end knob and then turns the zoom knob
- **THEN** the slice end stays at its place of the picture

#### Scenario: Zoom around the region start
- **WHEN** the owner moves the region start and then zooms in
- **THEN** the region start stays at its place of the picture

#### Scenario: A cursor move takes the anchor back
- **WHEN** the owner moves the slice end, then moves the cursor and then zooms in
- **THEN** the cursor stays at its place of the picture

#### Scenario: A pad hit takes the anchor back
- **WHEN** the owner moves the slice start, then selects another slice with a pad hit and then zooms in
- **THEN** the cursor stays at its place of the picture

#### Scenario: The anchor follows an undo
- **WHEN** the owner moves the slice end, undoes the move and then zooms in
- **THEN** the slice end at its old position stays at its place of the picture

#### Scenario: A deleted anchor gives the anchor to the cursor
- **WHEN** the owner moves the slice start, deletes that cut and then zooms in
- **THEN** the cursor stays at its place of the picture

#### Scenario: The cursor comes back after a zoom around a bound
- **WHEN** a zoom around the slice end puts the cursor out of the view and the owner moves the cursor
- **THEN** the view shifts so the cursor is visible again

## MODIFIED Requirements

### Requirement: Zoom around the cursor
The system SHALL let the owner zoom the wave view twice in or out around the zoom anchor. The view SHALL follow the cursor when the cursor moves. The view SHALL keep at least 256 frames and SHALL NOT be wider than the sample. The wave, numbers and dimming SHALL draw from the view.

#### Scenario: Zoom in and out
- **WHEN** the owner presses zoom-in twice and then zoom-out
- **THEN** the wave is a quarter of its full width and then back at the half of it; the cursor stays at the same place of the picture

#### Scenario: The view follows
- **WHEN** the cursor walks off the edge of the view
- **THEN** the view shifts so the cursor is visible again

### Requirement: Chop knobs
Learned Chop knobs SHALL use the configured response for absolute values:
- `chop.cursor_pos`: cursor within the region.
- `chop.zoom`: visible span from region to whole file, logarithmically around the zoom anchor.
- `chop.region_start_pos` / `chop.region_end_pos`: region edges, at least 10 ms apart.
MIDI learn SHALL list these under CHOP.
The X-Session Pro profile SHALL bind `chop.zoom` to k12.

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

### Requirement: The cursor and the zoom on the right hand
The system SHALL, on the chop screen outside the edit mode, move the cursor with `J` `L`
(and `←` `→`) and zoom the wave around the zoom anchor with `I` (in) and `K` (out), also
`↑` `↓`. The cursor step follows the view, so zooming in is its fine step; holding a key
repeats it. The slice under the
cursor is the selected one, as before; the slices are not walked with up and down.

#### Scenario: Zoom on I and K
- **WHEN** the chop screen is open and the owner presses `I` twice and then `K`
- **THEN** the view zooms in twice and out once around the cursor; the cursor stays

#### Scenario: Minus and equal do not zoom
- **WHEN** the chop screen is open and the owner presses `−` or `=`
- **THEN** the view does not change

### Requirement: Zoom without releasing the bound modifier

On Chop, the owner SHALL be able to move a slice bound and zoom without releasing Ctrl or Alt.
Ctrl/Alt+I SHALL zoom in and Ctrl/Alt+K SHALL zoom out around the zoom anchor.
Zoom SHALL preserve slice boundaries, the cursor and the selected slice.
It SHALL NOT change which bound Ctrl or Alt addresses.
Existing assignment and higher-priority input modes SHALL continue to block underlying zoom actions.

#### Scenario: Zoom between bound edits with the modifier held
- **WHEN** the owner holds Ctrl or Alt, moves a bound, presses and holds I, presses K and moves the bound again
- **THEN** the view zooms in and out without a modifier release, with slice edit open or closed
- **AND** zoom leaves the cursor, selection and boundaries unchanged, while each bound command edits only its original bound

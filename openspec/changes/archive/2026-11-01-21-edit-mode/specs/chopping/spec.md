## ADDED Requirements

### Requirement: Slice edit mode
The system SHALL open the edit mode of the selected slice with `Enter` on the chop screen
in the listen mode and close it with `Enter` or `Esc`; in the assign mode `Enter` stays
commit. The page is BOUNDS (slice start, slice end). `I` `K` pick the bound and `J` `L`
move it, the same keys as on the pads screen; a pad hit still sounds and selects the slice
it plays.

#### Scenario: Enter opens on the selected slice
- **WHEN** a slice is selected in the listen mode and the owner presses `Enter`
- **THEN** the screen names the BOUNDS page and shows what the knobs turn now

#### Scenario: Commit is untouched
- **WHEN** the assign mode is on and the owner presses `Enter`
- **THEN** the plan applies to the pads, as before

### Requirement: Assign from the keyboard
The system SHALL start the assign from the keyboard with `P` on the chop screen; `Enter`
opens the slice edit in the listen mode and stays the commit in the assign mode, and the
`chop.commit` action keeps "no plan — start, plan — apply" for device buttons. With the
slice edit open neither `P` nor a device commit starts the assign: `Esc` comes first.

#### Scenario: P starts the assign
- **WHEN** the chop screen is open in the listen mode and the owner presses `P`
- **THEN** the assign mode starts by hand from slice 1, as it did from `Enter` before

#### Scenario: No assign from the edit
- **WHEN** the slice edit is open and the owner presses `P` or the device commit button
- **THEN** the assign does not start and the slice edit stays open

### Requirement: The assign mode ignores the rest
The system SHALL, while the assign mode is on, ignore every chop-screen action except the
pads, `Tab` (by hand / in order), `Enter` (apply), `Esc` (back), the slice pick and the
undo: the cursor, zoom, cuts, the bounds, play and its mode, the equal parts and the region do not act
until the mode is left. An ignored key shows a hint in the status line: "assigning: Enter
applies, Esc leaves".

#### Scenario: No stray cut during the assign
- **WHEN** the assign mode is on and the owner presses `M`, `J` `L`, `I` `K`, `Ctrl+J`,
  `Alt+L`, `Y` and `,`
- **THEN** nothing is cut, zoomed or moved: the cuts, the cursor, the view and the plan
  stay as they were, and the status line says how to apply or leave

### Requirement: Slice bounds from the keyboard
The system SHALL move the bounds of the selected slice in the listen mode with a two-hand
chord: `Ctrl+J` `Ctrl+L` move its left bound (the start), `Alt+J` `Alt+L` its right bound
(the end), the arrows with the same modifiers too; a bound steps like the cursor (see
"Cursor modes"): a tap 1/128 of the visible wave, a held key speeds up to 1/32 per repeat;
the status line says which pads changed, as after a slice edit.

#### Scenario: Ctrl and Alt name the bound
- **WHEN** the chop screen is open in the listen mode and the owner presses `Ctrl+J`, then
  `Alt+L`
- **THEN** the start of the selected slice moves a step left, its end moves a step right,
  and the pads linked to the changed slices follow

### Requirement: The cursor and the zoom on the right hand
The system SHALL, on the chop screen outside the edit mode, move the cursor with `J` `L`
(and `←` `→`) and zoom the wave around the cursor with `I` (in) and `K` (out), also
`↑` `↓`. The cursor step follows the view, so zooming in is its fine step; holding a key
repeats it. The slice under the
cursor is the selected one, as before; the slices are not walked with up and down.

#### Scenario: Zoom on I and K
- **WHEN** the chop screen is open and the owner presses `I` twice and then `K`
- **THEN** the view zooms in twice and out once around the cursor; the cursor stays

#### Scenario: Minus and equal do not zoom
- **WHEN** the chop screen is open and the owner presses `−` or `=`
- **THEN** the view does not change

### Requirement: Cut and region at the cursor
The system SHALL put the actions at the cursor on the bottom row of the right hand: `M`
cuts at the cursor, `,` starts the region at the cursor, `.` ends it there, `Ctrl+,`
makes the region the whole sample again.

#### Scenario: Region on comma and dot
- **WHEN** the cursor stands at 0.5 s and the owner presses `,`, moves the cursor to 1.5 s
  and presses `.`
- **THEN** the region is 0.5 s – 1.5 s, and `Ctrl+,` brings back the whole sample

### Requirement: Equal parts on a letter
The system SHALL step the equal parts with `Y` on the chop screen, away from the keys the
right hand moves with; `Tab` switches in order / by hand in the assign mode.

#### Scenario: Y steps the parts
- **WHEN** the chop screen is open in the listen mode and the owner presses `Y`
- **THEN** the sample is cut into the next count of equal parts

### Requirement: Play modes
The system SHALL play on the chop screen in one of three modes, stepped with `O`: to the
end (from the cursor to the end of the region, or of the sample when no region is set),
to the cut (from the cursor to the end of the slice the cursor is in), the slice (the
whole slice the cursor is in). A green bar right above the wave always shows what play
will cover now; it follows the cursor and the cuts at once. The help line names the mode.
The chop screen opens in "to the end"; the mode is not saved.

#### Scenario: To the cut
- **WHEN** the cursor stands in the middle of slice 3, the mode is "to the cut" and the
  owner presses play
- **THEN** playback runs from the cursor to the end of slice 3 and stops there

#### Scenario: The slice
- **WHEN** the cursor stands in the middle of slice 3, the mode is "the slice" and the
  owner presses play
- **THEN** the whole slice 3 plays, from its start

#### Scenario: The bar shows what plays
- **WHEN** the owner presses `O` and moves the cursor into the next slice
- **THEN** the green bar above the wave changes at once to the span the next play covers,
  and the help line names the mode

### Requirement: Jump to a cut
The system SHALL, on the chop screen in the listen mode, put the cursor on the next cut to
the right with `PgDn`, or on the end of the region when no cut lies ahead, and on the
previous cut to the left with `PgUp`, or on the start of the region; the slice under the
cursor is selected and the page follows it. In the assign mode `PgUp` `PgDn` turn the
banks of pads, as before.

#### Scenario: Cut to cut
- **WHEN** the cursor stands inside slice 2 of 4 and the owner presses `PgDn` twice and
  then `PgUp`
- **THEN** the cursor lands on the start of slice 3, then of slice 4, then back on the
  start of slice 3

#### Scenario: To the ends
- **WHEN** the cursor stands in the last slice and the owner presses `PgDn`
- **THEN** the cursor lands on the end of the region; on the first slice `PgUp` lands on
  its start

## MODIFIED Requirements

### Requirement: Pages of slices
The system SHALL let the owner hear every slice from the pads in the listen mode by pages of 16 slices: the page follows the selected slice (the cursor, a jump, a slice pick), `Ctrl+1`…`8` pick a page; `PgUp` `PgDn` move the cursor between the cuts (see "Jump to a cut").

#### Scenario: Slices 17–32
- **WHEN** a sample is cut into 32 slices and the owner picks page 2 with `Ctrl+2` in the listen mode
- **THEN** the pads are labelled 17–32 and pad 1 plays slice 17

#### Scenario: Selected slice on another page
- **WHEN** the owner selects slice 20 with "next slice"
- **THEN** the page with slices 17–32 is shown



### Requirement: Slice editing
The system SHALL let the owner move the cursor, move the start of the selected slice finely (a tap) or speeding up (a held key), delete the cut at its start and undo the last actions, without a mouse. `Backspace` and `Ctrl+Backspace` delete the start of the selected slice, `Alt+Backspace` its end — the bound a held `Ctrl` / `Alt` lights; a bound that is the region's edge is not deleted. A cursor exactly on a cut selects the slice to its right, so `Backspace` deletes the cut under the cursor; the slice under the cursor plays when the cursor stops on it and stays selected while the cursor moves inside it.

#### Scenario: The cursor selects
- **WHEN** the owner presses → and the cursor enters the next slice
- **THEN** that slice becomes selected and highlighted, and plays once

#### Scenario: Select a slice
- **WHEN** the owner presses the "next slice" action (from a controller)
- **THEN** the next slice is selected, highlighted on the waveform and plays once

#### Scenario: Coarse move
- **WHEN** the owner holds the key that moves the start of the selected slice
- **THEN** the first repeat moves the cut by 1/128 of the visible wave, the step grows over
  6 repeats to 1/32 per repeat and stays there: about a screen in 1.3 s at the system's 25
  repeats a second, at any zoom

#### Scenario: Fine adjustment
- **WHEN** the owner taps the key that moves the start of the selected slice
- **THEN** the cut moves by 1/128 of the visible wave, about one cell of the screen; a tap is
  the fine step, holding speeds up

#### Scenario: Trim the sample start
- **WHEN** the first slice is selected and the owner moves its start to the right
- **THEN** the sample starts later, the silence before the first hit is not played; the file does not change

#### Scenario: Undo a delete
- **WHEN** the owner deletes the cut at the start of a slice and then presses undo
- **THEN** the cut is back at its old position

#### Scenario: Delete the lit bound
- **WHEN** slice 3 is selected and the owner holds Alt (its end lights) and presses Backspace
- **THEN** the cut between slices 3 and 4 is gone, slices 3 and 4 are one slice, and undo brings the cut back

#### Scenario: The cursor on a cut
- **WHEN** the owner jumps the cursor onto a cut with PgDn and presses Backspace
- **THEN** that cut is gone, and the slices on both sides of it are one slice

### Requirement: Chop region
The system SHALL let the owner set the start and the end of the playable region from the cursor with the region actions and clear it back to the whole sample; equal parts and playback cover only the region; the region is at least one frame long.

#### Scenario: Region from I to O
- **WHEN** the owner presses `,` at frame 30_000 and `.` at frame 60_000 and then runs equal parts
- **THEN** the parts cover the region from 30_000 to 60_000, and the outside is dimmed

#### Scenario: A region keeps pads linked to the slices inside it
- **WHEN** 16 slices of a break are on pads and the owner presses `,` in the middle of the break
- **THEN** the pads with slices inside the region keep following their slices; the pads whose cuts fell outside play the slice that now covers that place, as after a delete

#### Scenario: Play the region
- **WHEN** the region is set and the owner presses play
- **THEN** the sound stops at the region end, not at the end of the file

#### Scenario: Clear and undo
- **WHEN** the owner presses `Ctrl+,` to clear the region
- **THEN** the region is the whole sample again and undo restores the previous region

### Requirement: Cursor modes
The system SHALL move the cursor with `J` `L` (and ← →) inside the region in one of two modes, switched with the "snap to hits" command and shown in the help line; the chop screen opens in even steps. In even steps a tap moves the cursor by 1/128 of the visible window (never less than a frame) and a held key speeds up over 6 repeats to 1/32 of the window per repeat; the slice bounds step the same, so the cursor and the bounds move alike at any zoom. In hits mode it jumps to the start of the next or previous hit when that hit is on the screen, and moves in even steps while no hit lies ahead on the screen. A finer step comes from zooming in: the step follows the view. The cut command cuts at the cursor also on pause.

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

#### Scenario: Fine step
- **WHEN** the owner zooms in four times with `I` and presses `L`
- **THEN** the cursor moves by 1/128 of the smaller view, a sixteenth of the step on the
  whole file

#### Scenario: Cut on pause
- **WHEN** the playback is stopped and the owner presses the cut command
- **THEN** a cut appears at the cursor, as it does while playing

### Requirement: Chop screen
The system SHALL open the chop screen for the sample of the selected pad and show the waveform with cut markers, the selected slice, the cursor and, while the sample plays, the playhead. The cursor is the owner's: only the owner moves it (the cursor keys, the snap, a slice pick), it is drawn as `!` above the wave and named in the slice line; playback starts from it and the edits at the cursor use it. The playhead is a thin line that runs while the sample plays and is gone when it stops; it never moves the cursor.

#### Scenario: Open from a pad
- **WHEN** the selected pad has a sample and the owner presses F3
- **THEN** the chop screen shows the waveform of this sample, drawn with Braille characters

#### Scenario: Empty pad
- **WHEN** the selected pad has no sample and the owner presses F3
- **THEN** the screen does not change, the status says "pad is empty"

#### Scenario: The cursor is visible
- **WHEN** the chop screen is open
- **THEN** the `!` marks the cursor above the wave and the slice line names its time in the sample

#### Scenario: The playhead does not take the cursor
- **WHEN** the cursor stands at 1.0 s and the owner presses play and lets it run to 3.0 s
- **THEN** a thin line runs from 1.0 s to 3.0 s, the `!` and the slice line stay at 1.0 s


### Requirement: Chopping in time
The system SHALL play the sample with a moving playhead and set a marker at the playhead on the "cut" command while it plays (at the cursor when it does not); the cut does not move the cursor.

#### Scenario: Cut while playing
- **WHEN** the sample plays and the owner presses "cut"
- **THEN** a marker appears where the playhead is, the number of slices grows by one, and the cursor stays where it was

### Requirement: Play from the selected slice
The system SHALL start playback on the chop screen from the cursor, every time, as the play mode says (see "Play modes"); stopping does not move the cursor, so the next play starts from the same place.

#### Scenario: Listen from the middle
- **WHEN** the cursor stands at slice 5 and the owner presses play
- **THEN** playback starts there and goes on to the end of the region, or of the sample when no region is set

#### Scenario: The cursor survives a pause
- **WHEN** the owner stops the playback
- **THEN** the cursor stays where the owner put it and the next play starts from it again

## MODIFIED Requirements

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
- **WHEN** the snap is `cuts`, the owner moves the cursor onto a cut with `L` (or with the
  "next cut" action of a controller) and presses Backspace
- **THEN** that cut is gone, and the slices on both sides of it are one slice

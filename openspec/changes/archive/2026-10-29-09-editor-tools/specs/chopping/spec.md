## MODIFIED Requirements

### Requirement: Chop screen
The system SHALL open the chop screen for the sample of the selected pad and show the waveform with cut markers, the selected slice and the cursor: the position that plays and that edits use, drawn as `!` above the wave and named in the slice line.

#### Scenario: Open from a pad
- **WHEN** the selected pad has a sample and the owner presses F3
- **THEN** the chop screen shows the waveform of this sample, drawn with Braille characters

#### Scenario: Empty pad
- **WHEN** the selected pad has no sample and the owner presses F3
- **THEN** the screen does not change, the status says "pad is empty"

#### Scenario: The cursor is visible
- **WHEN** the chop screen is open
- **THEN** the `!` marks the cursor above the wave and the slice line names its time in the sample

### Requirement: Slice editing
The system SHALL let the owner move the cursor, move the start of the selected slice coarsely or finely, delete the cut at its start and undo the last actions, without a mouse; the slice under the cursor plays when the cursor stops on it and stays selected while the cursor moves inside it.

#### Scenario: The cursor selects
- **WHEN** the owner presses → and the cursor enters the next slice
- **THEN** that slice becomes selected and highlighted, and plays once

#### Scenario: Select a slice
- **WHEN** the owner presses the "next slice" action (from a controller)
- **THEN** the next slice is selected, highlighted on the waveform and plays once

#### Scenario: Coarse move
- **WHEN** the owner moves the start of the selected slice without a modifier
- **THEN** the cut moves by ~20 ms

#### Scenario: Fine adjustment
- **WHEN** the owner holds the "fine" modifier (Ctrl on the keyboard) and moves the start of the selected slice
- **THEN** the cut moves by ~4 ms; after the modifier is released the step is coarse again

#### Scenario: Trim the sample start
- **WHEN** the first slice is selected and the owner moves its start to the right
- **THEN** the sample starts later, the silence before the first hit is not played; the file does not change

#### Scenario: Undo a delete
- **WHEN** the owner deletes the cut at the start of a slice and then presses undo
- **THEN** the cut is back at its old position

### Requirement: Play from the selected slice
The system SHALL start playback on the chop screen from the cursor; on pause the cursor stays where it was, so play and edit share one position.

#### Scenario: Listen from the middle
- **WHEN** the cursor stands at slice 5 and the owner presses play
- **THEN** playback starts there and goes on to the end of the region, or of the sample when no region is set

#### Scenario: The cursor survives a pause
- **WHEN** the owner stops the playback
- **THEN** the cursor stays at the position where it stopped and the next play starts from there

### Requirement: Equal-parts auto-chop
The system SHALL cut the region into equal parts on the equal-parts command, stepping on from the current number of slices, and SHALL keep stepping after a restart; with no region set the region is the whole sample.

#### Scenario: Eight parts
- **WHEN** a sample is cut into 4 equal parts and the owner presses the equal-parts command
- **THEN** the sample is cut into 8 equal parts

#### Scenario: Parts of the region
- **WHEN** the region covers the second half of the sample and the owner presses the equal-parts command for 8 parts
- **THEN** the 8 cuts fall inside the region only

## ADDED Requirements

### Requirement: Editing cursor
The system SHALL move the cursor with ← → by 1/32 of the visible window (never less than a frame), snap it to the nearest wave peak in a small window around the new position, and skip the snap while Ctrl is held; the MIDI actions "cursor left" / "cursor right" and the `chop.cursor` knob move it the same way.

#### Scenario: Steps grow with the view
- **WHEN** the view shows the whole file and the owner presses → ten times, then zooms in and presses → ten times
- **THEN** the second ten steps cover a much smaller part of the sample

#### Scenario: Snap to the peak
- **WHEN** the cursor moves next to a drum hit without Ctrl
- **THEN** it lands on the peak of that hit, not between the waves

#### Scenario: No snap with Ctrl
- **WHEN** the owner holds Ctrl and moves the cursor across the same hit
- **THEN** the cursor passes in even steps, ignoring the peaks

#### Scenario: Cut on pause
- **WHEN** the playback is stopped and the owner presses the cut command
- **THEN** a cut appears at the cursor, as it does while playing

### Requirement: Zoom
The system SHALL zoom the view around the cursor with the zoom-out and zoom-in commands, stopping at the whole file and at the region, and SHALL shift the view just enough when the cursor walks off-screen; the `chop.zoom` knob sets the visible span between the region and the whole file with soft takeover.

#### Scenario: Zoom around the cursor
- **WHEN** the owner zooms in several times
- **THEN** the wave grows around the cursor and the cursor stays on screen

#### Scenario: Limits
- **WHEN** the owner zooms out past the whole file or zooms in past the region
- **THEN** the view stops at the whole file, or at the region when one is set

#### Scenario: The view follows
- **WHEN** the cursor walks off the edge of the view
- **THEN** the view shifts so the cursor is visible again

#### Scenario: Zoom knob
- **WHEN** the owner turns the learned zoom knob
- **THEN** the view spans from the whole file to close-ups smoothly, and grabbing the knob from another position does not jump the view

### Requirement: Region
The system SHALL mark the part of the sample being chopped: the region commands put the start or the end edge under the cursor, a clear command returns the whole file, equal parts and playback work inside the region, the wave outside it stays dim, and the learned knobs `chop.region_start` and `chop.region_end` move the edges with soft takeover.

#### Scenario: Mark and cut
- **WHEN** the owner puts the cursor on the first hit, presses the region-start command, moves to the last hit and presses the region-end command, then cuts equal parts
- **THEN** the cuts fall only between the two edges

#### Scenario: Play the region
- **WHEN** the region is set and the owner presses play
- **THEN** the sound stops at the region end, not at the end of the file

#### Scenario: Clear
- **WHEN** the owner presses the region-clear command
- **THEN** the region is the whole file again, the cuts stay

#### Scenario: Region knobs
- **WHEN** the owner turns the learned region knobs
- **THEN** the edges follow with soft takeover, the value pops up, and the region never gets shorter than one audible slice

### Requirement: Normalize
The system SHALL, on the normalize command, raise the volume of the pad this sample plays on so its loudest peak sits at −1 dB, show the gain in the status, and never change the file on disk; when the chop screen was opened without a pad, the status says there is nothing to normalize.

#### Scenario: Quiet break
- **WHEN** the owner opens a break whose peaks are 8 dB below full scale and presses normalize
- **THEN** the status shows about "+7 dB" and the pad plays at full level without clipping

#### Scenario: Twice in a row
- **WHEN** the owner presses normalize a second time
- **THEN** the gain is measured again from the file, so the level does not climb twice

#### Scenario: Chop without a pad
- **WHEN** the chop screen was opened from the files screen and the owner presses normalize
- **THEN** the status says there is no pad to set the volume on

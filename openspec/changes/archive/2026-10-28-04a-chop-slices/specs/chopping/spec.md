## ADDED Requirements

### Requirement: Chop screen
The system SHALL open the chop screen for the sample of the selected pad and show the waveform with cut markers, the selected slice and the playback position.

#### Scenario: Open from a pad
- **WHEN** the selected pad has a sample and the owner presses F3
- **THEN** the chop screen shows the waveform of this sample, drawn with Braille characters

#### Scenario: Empty pad
- **WHEN** the selected pad has no sample and the owner presses F3
- **THEN** the screen does not change, the status says "pad is empty"

### Requirement: Equal-parts auto-chop
The system SHALL chop a sample into equal parts (2, 4, 8, 16, 32); one command switches to the next count.

#### Scenario: Eight parts
- **WHEN** the owner presses the auto-chop command until the count is 8
- **THEN** the sample has 7 markers at equal distances, the header says "8 slices"

### Requirement: Chopping in time
The system SHALL play the sample with a moving cursor and set a marker at the current playback position on the "cut" command.

#### Scenario: Cut while playing
- **WHEN** the sample plays and the owner presses "cut"
- **THEN** a marker appears at the cursor position, the number of slices grows by one

### Requirement: Slice editing
The system SHALL let the owner select a slice, move its start coarsely or finely, delete the cut at its start and undo the last actions, without a mouse; selecting a slice plays it.

#### Scenario: Select a slice
- **WHEN** the owner presses "next slice"
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

### Requirement: Slice boundaries from knobs
The system SHALL let a knob set the start and another knob the end of the selected slice, inside the space between the neighbouring cuts, with soft takeover for absolute knobs.

#### Scenario: Turn the start
- **WHEN** slice 3 is selected and the owner turns the "slice start" knob
- **THEN** the cut between slices 2 and 3 follows the knob, from just after the start of slice 2 to just before the end of slice 3

#### Scenario: Another slice selected
- **WHEN** the owner selects another slice and turns the knob from a position far from the new cut
- **THEN** the cut does not jump; it starts to follow once the knob reaches its position, and the status says which way to turn

### Requirement: Listen mode
The system SHALL, while the chop screen is in the listen mode, play slice k on pad key k (and controller pad k), without regard to what lies on the pads and without changing them.

#### Scenario: Check a slice
- **WHEN** the sample is cut into 8 slices and the owner presses the third pad key
- **THEN** the third slice plays from its start to its end, without a click at the end

### Requirement: Assign mode
The system SHALL, on the "assign" command, show the real pads of the current bank with what lies on them, let the owner change banks, and show a plan: which slice goes to which pad and which pads it replaces. The plan is made in order or by hand and is applied by a separate command; leaving the mode changes nothing.

#### Scenario: See what is taken
- **WHEN** the owner enters the assign mode
- **THEN** each pad of the bank shows its sample name or that it is empty, the planned pads show the slice number, the pads that would be replaced are highlighted, and a summary says how many slices go to which banks and how many pads are replaced

#### Scenario: Nine slices
- **WHEN** a break is cut into 9 slices, pad 1 is selected and the plan is in order
- **THEN** the slices are planned on pads 1–9, pads 10–16 are not touched

#### Scenario: Start the chain on another pad
- **WHEN** the plan is in order and the owner switches to bank 3 and hits pad 5
- **THEN** slice 1 is planned on pad 5 of bank 3 and the rest follow from there

#### Scenario: More slices than pads to the end of the bank
- **WHEN** 9 slices are applied in order starting from pad 13 of bank 1
- **THEN** slices 1–4 go to pads 13–16 of bank 1, slices 5–9 to pads 1–5 of bank 2, the status says "slices placed on banks 1–2"

#### Scenario: Out of pads
- **WHEN** the slices do not fit before the end of bank 8
- **THEN** the extra slices are not assigned, the summary and the status say how many slices did not fit

#### Scenario: By hand
- **WHEN** the plan is by hand, the owner selects slice 1 and hits pad 1, then hits pad 2
- **THEN** slice 1 is planned on pad 1, slice 2 on pad 2 (the selection moved on after the first hit); slices not planned are not placed

#### Scenario: Hear the plan
- **WHEN** the owner hits a pad in the assign mode
- **THEN** the slice planned for that pad plays

#### Scenario: Leave without changes
- **WHEN** the owner leaves the assign mode without applying
- **THEN** no pad changes

#### Scenario: Slice playing on a pad is replaced
- **WHEN** a pad plays a slice and a plan with another slice for this pad is applied
- **THEN** the playing sound continues or stops with a fade, without a click; the next hit plays the new slice

### Requirement: Slices in the session
The system SHALL save slices in the session and restore them at start; sessions from earlier versions SHALL open with whole-file pads.

#### Scenario: Restart
- **WHEN** slices are on pads and the program is restarted
- **THEN** each pad plays the same slice as before and is named after it, e.g. "break 5"

#### Scenario: Old session
- **WHEN** the session file was saved before slices existed
- **THEN** it opens, every pad plays its whole file as before

### Requirement: Chop actions are learnable
The system SHALL list all chop screen actions, including the slice start and end knobs and the assign mode commands, in MIDI learn under their own group, so any controller control can perform them.

#### Scenario: Learn the cut
- **WHEN** the owner opens MIDI learn and selects "chop.cut" in the CHOP group
- **THEN** the next pressed controller button performs the cut on the chop screen

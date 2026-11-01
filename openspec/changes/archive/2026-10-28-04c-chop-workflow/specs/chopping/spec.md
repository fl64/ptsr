## MODIFIED Requirements

### Requirement: Assign mode
The system SHALL, on the "assign" command, show the real pads of the current bank with what lies on them, let the owner change banks, and show a plan: which slice goes to which pad and which pads it replaces. The plan is made by hand, pad by pad, or all in order from a start pad the owner hits; it is applied by a separate command, and an application that would overwrite pads asks first, showing what lies there. Leaving the mode changes nothing.

#### Scenario: See what is taken
- **WHEN** the owner enters the assign mode
- **THEN** each pad of the bank shows its sample name or that it is empty, the planned pads show the slice number, the pads that would be replaced are highlighted, and a summary says how many slices go to which banks and how many pads are replaced

#### Scenario: By hand
- **WHEN** the owner selects slice 1 and hits pad 1, then hits pad 2
- **THEN** slice 1 is planned on pad 1, slice 2 on pad 2 (the selection moved on after the first hit); slices not planned are not placed

#### Scenario: Start the chain on another pad
- **WHEN** the owner switches to "all in order", turns to bank 3 and hits pad 5
- **THEN** slice 1 is planned on pad 5 of bank 3 and the rest follow from there; nothing is planned before a start pad is hit

#### Scenario: Nine slices
- **WHEN** a break is cut into 9 slices, the plan is all in order from pad 1 of an empty bank 3 and the owner applies it
- **THEN** the slices go to pads 1–9 of bank 3 at once, pads 10–16 are not touched, the status says "slices placed on bank 3"

#### Scenario: More slices than pads to the end of the bank
- **WHEN** 9 slices are applied in order starting from pad 13 of bank 1
- **THEN** slices 1–4 go to pads 13–16 of bank 1, slices 5–9 to pads 1–5 of bank 2, the status says "slices placed on banks 1–2"

#### Scenario: Overwrite only after a question
- **WHEN** the plan covers pads Q and W of bank 2 that hold "kick" and "amen 3" and the owner applies it
- **THEN** nothing changes yet; a question shows which pads would be overwritten and what lies on them ("bank 2: Q kick, W amen 3"); confirming places the slices, declining leaves the pads and the plan as they were

#### Scenario: Out of pads
- **WHEN** the slices do not fit before the end of bank 8
- **THEN** the extra slices are not assigned, the summary and the status say how many slices did not fit

#### Scenario: Hear the plan
- **WHEN** the owner hits a pad in the assign mode
- **THEN** the slice planned for that pad plays

#### Scenario: Leave without changes
- **WHEN** the owner leaves the assign mode without applying
- **THEN** no pad changes

#### Scenario: Slice playing on a pad is replaced
- **WHEN** a pad plays a slice and a plan with another slice for this pad is applied
- **THEN** the playing sound continues or stops with a fade, without a click; the next hit plays the new slice

## ADDED Requirements

### Requirement: Linked slices
The system SHALL keep the cuts of a file after the slices are laid out and SHALL keep the pads with its slices linked to them, so a change of a cut changes these pads at once.

#### Scenario: Fix a slice after the layout
- **WHEN** slices 1–8 of a break are on pads 1–8 and the owner opens pad 3 with F3 and moves the start of slice 3 by 20 ms to the right
- **THEN** pad 2 plays 20 ms longer and pad 3 starts 20 ms later, without laying the slices out again

#### Scenario: Open the whole break from a slice
- **WHEN** the owner selects a pad with slice 5 of a break and presses F3
- **THEN** the chop screen shows the whole break with all its cuts and slice 5 selected

#### Scenario: Join two slices
- **WHEN** pads 2 and 3 hold slices 2 and 3 and the owner joins slice 3 with slice 2
- **THEN** pads 2 and 3 both play the joined slice, and the pads with later slices keep playing the same sound under numbers one lower

#### Scenario: Chop again into equal parts
- **WHEN** pads hold slices of a break and the owner replaces its cuts with equal parts
- **THEN** these pads keep playing what they played and are no longer linked, the status says how many pads were unlinked

#### Scenario: Undo the new chop
- **WHEN** the owner has replaced the cuts with equal parts and presses undo
- **THEN** the old cuts are back and the pads are linked to them again

#### Scenario: Restart
- **WHEN** the program is restarted
- **THEN** the cuts of each chopped file and the links of its pads are restored

### Requirement: Chop from the files screen
The system SHALL open the selected file of the files screen on the chop screen without putting it on a pad first.

#### Scenario: Chop a found break
- **WHEN** the owner selects a file on the files screen and presses F3
- **THEN** the chop screen opens with this file, no pad changes until a plan is applied

### Requirement: Play from the selected slice
The system SHALL start playback on the chop screen from the start of the selected slice.

#### Scenario: Listen from the middle
- **WHEN** slice 5 is selected and the owner presses play
- **THEN** playback starts at slice 5 and goes on to the end of the sample

### Requirement: Pages of slices
The system SHALL let the owner hear every slice from the pads in the listen mode by turning pages of 16 slices with the bank actions.

#### Scenario: Slices 17–32
- **WHEN** a sample is cut into 32 slices and the owner turns to the next page in the listen mode
- **THEN** the pads are labelled 17–32 and pad 1 plays slice 17

#### Scenario: Selected slice on another page
- **WHEN** the owner selects slice 20 with "next slice"
- **THEN** the page with slices 17–32 is shown

### Requirement: Pad hit selects the slice
The system SHALL, in the listen mode, make the slice played by a pad hit the selected slice, so the next edit acts on it.

#### Scenario: Fix the slice just heard
- **WHEN** the owner hits pad 6 on the page with slices 1–16 and then moves the slice start
- **THEN** slice 6 is selected and highlighted, and its start moves

### Requirement: Assign slice by slice
The system SHALL open the assign mode with an empty plan made by hand and slice 1 selected; each pad hit puts the selected slice on that pad and selects the next slice not yet planned, until all slices are planned.

#### Scenario: Tap pads in turn
- **WHEN** a sample is cut into 4 slices, the owner presses Enter and hits pads Q, 1, W, E
- **THEN** slice 1 is planned on Q, slice 2 on 1, slice 3 on W, slice 4 on E, and the status says that all 4 slices are placed and Enter applies

#### Scenario: All placed
- **WHEN** all slices are planned and the owner hits another pad
- **THEN** nothing is planned there, the pad plays what is planned for it if anything

#### Scenario: Undo a tap
- **WHEN** the owner undoes the last tap
- **THEN** that slice is no longer planned and is selected again

### Requirement: Ask before overwriting pads
The system SHALL, before any step that loses what lies on a pad and cannot be undone, show in plain words which pads lose what and what takes its place, and SHALL do the step only after the owner confirms; declining changes nothing.

#### Scenario: Apply a plan over taken pads
- **WHEN** a plan puts slices 1 and 2 on pads Q and W of bank 2 that hold "kick" and "amen 3" and the owner applies it
- **THEN** a question lists "bank 2 Q kick → slice 1" and "bank 2 W amen 3 → slice 2", says that the files on disk stay, and offers Enter to replace and Esc to keep

#### Scenario: Put a file on a taken pad
- **WHEN** a file is in hand on the files screen and the owner hits a pad that holds "kick"
- **THEN** a question says that "kick" on that pad will be replaced by the file; Esc keeps "kick" and the file stays in hand

#### Scenario: Steps that can be undone
- **WHEN** a step can be taken back with undo (cuts, the plan itself)
- **THEN** it is done without a question

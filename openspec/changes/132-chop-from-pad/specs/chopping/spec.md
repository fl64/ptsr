## ADDED Requirements

### Requirement: Chop opens from the selected pad

The Pads actions SHALL have `chop`. It SHALL open Chop for the sound of the selected pad, as F3 does. A whole file SHALL open on the first slice page. A pad of a slice SHALL open on the bank of that pad with its slice selected. On an empty pad, a notice SHALL name the pad and Pads SHALL stay.

#### Scenario: The chop cell opens the slice of the pad
- **WHEN** slice 20 of 40 lies on the selected pad and the owner presses the chop cell
- **THEN** Chop opens on that sound and the bank of that pad with slice 20 selected

#### Scenario: A whole file opens on the first slice page
- **WHEN** the selected pad on bank 3 plays a whole file with 40 slices on no pad
- **AND** the owner presses the chop cell
- **THEN** the row over the grid says `slices 1–16 · page 1/3`

#### Scenario: The chop cell on an empty pad
- **WHEN** the selected pad is empty and the owner presses the chop cell
- **THEN** a notice names the empty pad and Pads stays open

### Requirement: Each chop keeps its slice page

Each chop SHALL keep its own slice page. When Chop opens, from a pad or from Files, the page SHALL be the page of the selected slice. The page keys SHALL turn that page while the slice keys show and SHALL NOT change the pad bank. The row over the slice keys SHALL name the page and the page count. A page past the last slice SHALL show the last page.

#### Scenario: A turned slice page keeps the pad bank
- **WHEN** Pads shows bank 3, the owner opens a chop of 40 slices, presses PgDn and goes back to Pads
- **THEN** Chop said `slices 17–32 · page 2/3` and Pads shows bank 3

#### Scenario: Files opens the page of the selected slice
- **WHEN** a chop of 40 slices on no pad has slice 35 selected
- **AND** the owner opens that file in Chop from Files
- **THEN** the row over the grid says `slices 33–40 · page 3/3`

#### Scenario: Fewer slices show the last page
- **WHEN** the slice keys show page 3 of a chop and the owner deletes cuts until 10 slices stay
- **THEN** the row over the grid says `slices 1–10 · page 1/1` and the first pad key plays slice 1

### Requirement: One Undo restores an Assign apply

An Assign apply SHALL write one history step with every pad that it changes, as it was before the apply. One Undo SHALL restore every target: old sounds come back and empty targets become empty. The chop and its cuts SHALL stay.

#### Scenario: Undo brings back the replaced pads
- **WHEN** the owner applies four slices on two used and two empty pads of two banks and then presses Undo once
- **THEN** the two used pads play their old sounds, the two others are empty, and the chop keeps its cuts

## MODIFIED Requirements

### Requirement: Banks on the chop screen

Chop SHALL change banks with PgUp, PgDn and Ctrl+1…8 in listen, assignment and slice edit. Until a slice lies on a pad, those keys SHALL turn the slice page of the chop outside pattern work. They SHALL NOT change the pad bank then. During pattern controls or active recording, those keys SHALL select real pad banks. The open chop and earlier recorded addresses SHALL remain unchanged.

#### Scenario: The next bank while chopping
- **WHEN** bank 1 is shown on the chop screen in the listen mode and the owner presses PgDn
- **THEN** bank 2 is shown and the cursor stays where it was

#### Scenario: The same keys while assigning
- **WHEN** the assign mode is open on bank 1 and the owner presses PgDn
- **THEN** bank 2 is shown, as in the listen mode

#### Scenario: Pages of the keyboard of slices
- **WHEN** a sample is cut into 20 slices, none lies on a pad, no pattern work is active and the owner presses PgDn
- **THEN** the row over the grid says `slices 17–20 · page 2/2` and the first pad key plays slice 17

#### Scenario: Chop recording uses absolute pad banks
- **WHEN** recording on Chop changes from bank 1 to bank 8 and both banks receive manual pad hits
- **THEN** A1 stores the original absolute addresses and both banks keep their real pad assignments
- **AND** a later release retains its original bank address

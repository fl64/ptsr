## ADDED Requirements

### Requirement: Chop opens from the selected pad

The Pads actions SHALL have `chop`. It SHALL open Chop for the sound of the selected pad, as F3 does. Chop SHALL show the slice page of the slice that the pad plays, or the first page for a whole file. On an empty pad, a notice SHALL name the pad and Pads SHALL stay.

#### Scenario: The chop cell opens the pad sound on its page
- **WHEN** slice 20 of 40 lies on the selected pad and the owner presses the chop cell
- **THEN** Chop opens on that sound with slices 17–32 on the pads and slice 20 selected

#### Scenario: A whole file opens on the first slice page
- **WHEN** the selected pad on bank 2 plays a whole file cut into 40 slices and the owner presses the chop cell
- **THEN** Chop shows slices 1–16

#### Scenario: The chop cell on an empty pad
- **WHEN** the selected pad is empty and the owner presses the chop cell
- **THEN** a notice names the empty pad and Pads stays open

### Requirement: One Undo restores an Assign apply

An Assign apply SHALL write one history step with every target pad as it was before the apply. One Undo SHALL restore every target: old sounds come back and empty targets become empty. The chop and its cuts SHALL stay.

#### Scenario: Undo brings back the replaced pads
- **WHEN** the owner applies four slices on two used and two empty pads of two banks and then presses Undo once
- **THEN** the two used pads play their old sounds, the two others are empty, and the chop keeps its cuts

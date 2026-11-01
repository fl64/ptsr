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
- **THEN** nothing changes yet; a question shows which pads would be overwritten and what lies on them ("bank 2, pad Q: kick → slice 1", "bank 2, pad W: amen 3 → slice 2"); confirming places the slices, declining leaves the pads and the plan as they were

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

### Requirement: Ask before overwriting pads
The system SHALL, before any step that loses what lies on a pad and cannot be undone, show in plain words which pads lose what and what takes its place, and SHALL do the step only after the owner confirms; declining changes nothing.

#### Scenario: Apply a plan over taken pads
- **WHEN** a plan puts slices 1 and 2 on pads Q and W of bank 2 that hold "kick" and "amen 3" and the owner applies it
- **THEN** a question lists "bank 2, pad Q: kick → slice 1" and "bank 2, pad W: amen 3 → slice 2", says that the files on disk stay, and offers Enter to replace and Esc to keep

#### Scenario: Put a file on a taken pad
- **WHEN** a file is in hand on the files screen and the owner hits a pad that holds "kick"
- **THEN** a question says that "kick" on that pad will be replaced by the file; Esc keeps "kick" and the file stays in hand

#### Scenario: Steps that can be undone
- **WHEN** a step can be taken back with undo (cuts, the plan itself)
- **THEN** it is done without a question

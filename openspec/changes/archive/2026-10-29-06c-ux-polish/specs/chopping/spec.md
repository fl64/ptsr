## MODIFIED Requirements

### Requirement: Chop screen readability
The system SHALL show on the chop screen, without cutting text: the plan summary in full, the selected slice both on the waveform and on the grid of the listen mode, empty pads in the assign mode as only "empty", and in the overwrite question each pad as "bank B, pad K: what → new". Counts are plain English ("1 slice", "2 slices"), and the slice line labels its numbers: start and length.

#### Scenario: Selected slice on the grid
- **WHEN** the owner hits pad W in the listen mode
- **THEN** pad W is highlighted as selected on the grid

#### Scenario: Plain overwrite question
- **WHEN** the plan puts slice 3 on pad 3 of bank 1 that holds "kick" and the owner applies it
- **THEN** the question says "bank 1, pad 3: kick → slice 3"

#### Scenario: One slice in plain words
- **WHEN** a file is not cut yet and the chop screen is open
- **THEN** the header says "1 slice", not "1 slices"

#### Scenario: Labeled slice line
- **WHEN** slice 3 starts at 0.652 s and lasts 326 ms
- **THEN** the slice line reads "slice 3 · start 0.652 s · length 326 ms"

### Requirement: Ask before overwriting pads
The system SHALL, before any step that loses what lies on a pad and cannot be undone, show in plain words which pads lose what and what takes its place, and SHALL do the step only after the owner confirms; declining changes nothing. A step that would leave a pad with the same sound it already plays asks nothing.

#### Scenario: Apply a plan over taken pads
- **WHEN** a plan puts slices 1 and 2 on pads Q and W of bank 2 that hold "kick" and "amen 3" and the owner applies it
- **THEN** a question lists "bank 2 Q kick → slice 1" and "bank 2 W amen 3 → slice 2", says that the files on disk stay, and offers Enter to replace and Esc to keep

#### Scenario: Put a file on a taken pad
- **WHEN** a file is in hand on the files screen and the owner hits a pad that holds "kick"
- **THEN** a question says that "kick" on that pad will be replaced by the file; Esc keeps "kick" and the file stays in hand

#### Scenario: Steps that can be undone
- **WHEN** a step can be taken back with undo (cuts, the plan itself)
- **THEN** it is done without a question

#### Scenario: Same sound again
- **WHEN** the owner puts on a pad the very file this pad already plays, whole
- **THEN** no question appears and the pad does not change

### Requirement: Chop from the files screen
The system SHALL open the selected file of the files screen on the chop screen without putting it on a pad first, and the files screen SHALL name both paths from a selected file: to a pad and to the chop screen.

#### Scenario: Chop a found break
- **WHEN** the owner selects a file on the files screen and presses F3
- **THEN** the chop screen opens with this file, no pad changes until a plan is applied

#### Scenario: Both paths are named
- **WHEN** a file is selected on the files screen
- **THEN** the screen says in plain words that Enter takes it to a pad and F3 opens it for chopping

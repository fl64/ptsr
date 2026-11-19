## MODIFIED Requirements

### Requirement: Pickers accept SMC-PAD selection without typed text

In the project picker, SMC-PAD knob 1 SHALL move the selection. Device pad 4 SHALL confirm the selected project, and device pad 1 SHALL cancel. These modal routes SHALL take priority over the screen behind the picker. A visible question SHALL keep its pad 1 and pad 4 priority. A failed project transition SHALL keep the current project, picker and selection. The picker hint SHALL name `knob 1`.

#### Scenario: Controller opens a selected project
- **WHEN** the owner opens the project picker, turns knob 1 and presses device pad 4
- **THEN** the selected project opens after its required save succeeds
- **AND** a refused save keeps the old project and the picker selection

### Requirement: Paths folder picker accepts SMC-PAD selection

In the Paths folder picker, SMC-PAD knob 1 SHALL move the folder selection. LEFT and RIGHT SHALL move to parent and child folders. Device pad 4 SHALL select the current folder, and device pad 1 SHALL cancel. These routes SHALL not change the Settings value behind the picker.

#### Scenario: Controller selects a Paths folder
- **WHEN** the owner opens the Paths folder picker, turns knob 1, moves into a folder with RIGHT and presses device pad 4
- **THEN** only the selected folder becomes the path
- **AND** device pad 1 in a separate run cancels without changing the path

### Requirement: Help has a controller route

The help cell of the screens view SHALL open Controller Help on each view. While Help
shows, LEFT and RIGHT SHALL select its tabs. Cells 9 and 10 SHALL scroll or move the
selected control. Device pad 1 SHALL close Help. Help SHALL show the controls that work in its
current tab. A Help cell SHALL leave the screen behind Help in place. F1 and the keyboard
Help paths SHALL retain their results.

#### Scenario: Help cells navigate without acting behind Help
- **WHEN** the owner opens Help from the panel, uses LEFT, RIGHT, cells 9 and 10 and then device pad 1
- **THEN** only the Help tab, scroll position or selected controller control changes before Help closes
- **AND** the screen and its selected value remain unchanged

### Requirement: Controller can finish learning

With an F10 or MIDI-tab learning choice pending, device pad 4 SHALL save that choice and device pad 1 SHALL cancel it. With no choice pending, device pad 1 SHALL leave learning. The answer SHALL not replace the chosen control or run a panel action. With no choice pending, device pads 1 and 4 SHALL remain available as controls to learn. The learning prompt SHALL name these pads after an SMC-PAD input.

#### Scenario: Learning answer keeps its chosen control
- **WHEN** the owner selects an action, presses device pad 1 as the control to learn and then presses pad 4
- **THEN** the first press becomes the pending control and the second saves it once
- **AND** a separate pending choice followed by pad 1 cancels without a new choice or screen action

### Requirement: SMC-PAD Exit asks before quit

An explicit Exit context cell SHALL open a visible question without starting quit. Device pad 4 SHALL confirm and enter the existing exit-save path once. Device pad 1 SHALL cancel and keep the session open. Other panel cells SHALL NOT act behind the question. Ctrl+Q SHALL keep its current direct exit behavior. MIDI learn SHALL NOT offer Exit.

#### Scenario: Exit needs a second SMC-PAD press
- **WHEN** the owner hits Exit on the panel and presses device pad 1
- **THEN** the question closes and the session remains open
- **AND** a later Exit followed by device pad 4 enters the existing exit-save path once

## ADDED Requirements

### Requirement: A refused panel action says why

A panel cell that cannot act in the current state SHALL show a notice with the reason and the place where it works. It SHALL NOT change state.

#### Scenario: Learn on Settings explains the refusal
- **WHEN** the owner hits Learn in the screens view while Settings shows
- **THEN** a notice says where Learn works and the Settings screen stays unchanged

### Requirement: Placement results name their pads

A placement of files, recordings, copies, swaps or Assign slices SHALL show a result notice. The notice SHALL name the source and the target pads by the numbers on the grid.

#### Scenario: Assign apply names the pads
- **WHEN** the owner applies an Assign plan with three slices on device pads 5, 6 and 9
- **THEN** the notice names slices 1 to 3 and pads 5, 6 and 9

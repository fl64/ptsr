## MODIFIED Requirements

### Requirement: The list buttons follow the current SMC-PAD mode

LEFT and RIGHT SHALL select pad banks, pad edit pages, Pattern bars, Chop slices,
Settings tabs or FX slots in the corresponding view.
On Files, LEFT SHALL select the parent folder. RIGHT SHALL enter the selected folder.
The current Load and Take paths SHALL handle files.
While the panel of that controller shows, LEFT and RIGHT SHALL move the panel views instead.
The input overlay SHALL keep its priority.

#### Scenario: SMC-PAD list buttons name and move the same list
- **WHEN** LEFT and RIGHT act on Pads, pad edit, Pattern, Chop, FX or Settings with the panel hidden
- **THEN** each button moves the list for that mode
- **AND** the current selection remains within the list bounds

#### Scenario: File list buttons navigate folders
- **WHEN** the owner uses RIGHT with a selected file or folder and the panel hidden
- **THEN** a file remains selected and a folder opens
- **AND** LEFT returns to the parent folder

#### Scenario: A visible panel takes LEFT and RIGHT
- **WHEN** the panel shows on Chop and the owner presses RIGHT
- **THEN** the panel shows the next view and the selected slice stays the same

### Requirement: Help has a controller route

The help cell of the screens view SHALL open Controller Help on each view. While Help
shows, cells 7 and 8 SHALL select its tabs. Cells 9 and 10 SHALL scroll or move the
selected control. Back SHALL close Help. Help SHALL show the controls that work in its
current tab. A Help cell SHALL leave the screen behind Help in place. F1 and the keyboard
Help paths SHALL retain their results.

#### Scenario: Help cells navigate without acting behind Help
- **WHEN** the owner opens Help from the panel, uses cells 7 through 10 and then Back
- **THEN** only the Help tab, scroll position or selected controller control changes before Help closes
- **AND** the screen and its selected value remain unchanged

### Requirement: Controller routes keep short press paths

Frequent actions SHALL use a direct button, pad or knob, or action page 1.
From the visible panel, an action on the shown view SHALL take one press.
Each LEFT or RIGHT press to another view SHALL add one press.
A hidden panel SHALL add PAD BANK, and a reveal hit if the panel stays closed.
Count KNOB BANK, target hits and modal answers separately. A knob turn counts as zero.
The reach table SHALL list the route and counts for each story in `docs/USER-STORIES.md`.

#### Scenario: First entry and repeated commands have separate counts
- **WHEN** the owner reaches an action page 2 command from a hidden panel and then repeats it from a visible panel
- **THEN** the first count includes PAD BANK, RIGHT and a reveal hit when the shortcut is off or misses the tap
- **AND** the later count excludes PAD BANK and the reveal hit
- **AND** each LEFT or RIGHT press adds one to the path
- **AND** neither count treats a knob turn as a button press

#### Scenario: Every story has a recorded controller path
- **WHEN** the reach test reads every action story in `docs/USER-STORIES.md`
- **THEN** each story has an SMC-PAD route and a press count in `docs/CONTROLLER-PATHS.md`
- **AND** observation-only, text-only and other-device stories name their exception

## REMOVED Requirements

### Requirement: Panel buttons select context pages

**Reason**: Change 127 removes More, Next and Main. LEFT and RIGHT move the panel views.
**Migration**: `Panel views move with LEFT and RIGHT` in `command-panel`.

### Requirement: Added context cells provide missing actions

**Reason**: Change 127 moves global actions to the screens view and lists every action page in one table.
**Migration**: `Action pages follow the shipped table` in `command-panel`.

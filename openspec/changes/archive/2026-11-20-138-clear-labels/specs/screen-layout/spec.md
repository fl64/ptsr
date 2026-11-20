## ADDED Requirements

### Requirement: Chop knob labels name the trim and the hit level

On Chop, the knob strip SHALL name the trim knobs `trim<` and `trim>` and the threshold knob `hits`. The line of the last turned knob SHALL name them «trim start», «trim end» and «hit level».

#### Scenario: The trim knob names its edge
- **WHEN** the owner turns knob 5 on Chop
- **THEN** the knob strip says `trim<` and the line of the last knob says «trim start»

### Requirement: Modes show their state where they start

Each active mode SHALL have a mode line on the screen where it starts. The slice edit SHALL show `EDIT slice N · BOUNDS` and its close control. The copy line SHALL name the source sound, bank and pad. After the first tap, the line SHALL say «TAP · 1 hit · tap again». With PAD BANK set to Off and no mode, the line SHALL say «PAD BANK: hit to show commands».

#### Scenario: The slice edit shows its close control
- **WHEN** the owner opens the slice edit with the SMC-PAD and the panel is hidden
- **THEN** the mode line says `EDIT slice 1 · BOUNDS` and ends with «PAD BANK + 1 close»


#### Scenario: The copy line keeps its source after a bank change
- **WHEN** the owner copies pad 1 of bank 1 and moves to bank 2
- **THEN** the mode line names the source sound and B1/P1

#### Scenario: Tap tempo confirms the first tap
- **WHEN** the owner taps tempo once on A1
- **THEN** the mode line says «TAP · 1 hit · tap again»

### Requirement: A1 modes show their state on A1

A1 SHALL show `COUNT N` during the count-in and `REC A1` with its stop control during the recording. The step edit SHALL name the step, that a pad hit adds or removes a hit, and its close control. A1 setup SHALL name the knobs of its values and its close control.

#### Scenario: A1 shows the count-in and the recording
- **WHEN** the owner presses RECORD on A1
- **THEN** the A1 screen shows `COUNT 4` and then `REC A1` with the stop control

#### Scenario: Step edit names its next step and close control
- **WHEN** the owner opens the step edit on A1 with the SMC-PAD
- **THEN** the mode line names the step, says that a pad hit adds or removes a hit and ends with «PAD BANK + 1 close»

#### Scenario: A1 setup names its knobs
- **WHEN** the owner opens A1 setup with the SMC-PAD
- **THEN** the mode line names knob 3 quantize, knob 4 strength, knob 5 bars and «PAD BANK + 1 close»

## MODIFIED Requirements

### Requirement: Panel replaces the pad grid without moving the frame

At 80 by 26 and 120 by 40, the command panel SHALL draw its 4 by 4 cell grid in the existing pad-grid rectangle. It SHALL keep the title, strip, bank row, right column, three-row help and status in their current positions. Cells SHALL use the same physical top-to-bottom order as first-bank pads. Each line of a cell label SHALL fit eight terminal cells. The panel SHALL remain readable with kitty graphics on or off. It SHALL not conceal a question, text line or safety warning.

#### Scenario: Panel grid fits both supported sizes
- **WHEN** the panel shows its main and context pages at 80 by 26 and 120 by 40
- **THEN** all 16 cells, page title, fixed bottom row and safety indicators fit inside their existing areas
- **AND** no cell is obscured by a pad-wave image and leaving the panel restores the current wave

### Requirement: Panel state and unavailable cells stay visible

The title SHALL show `PANEL · SMC` for the active SMC-PAD connection.
A header row SHALL name the shown view and the views on its left and right.
Action cells, screen cells and the bottom row SHALL use three different colors.
An action cell that replaces or removes work SHALL use the orange warning color.
An action cell that cannot act now SHALL be grey. An empty cell SHALL show no label.
A new view or screen SHALL update the labels and looks before the next cell acts.
Disabled Redo and Panic SHALL appear grey with no active-action claim.
A first reveal SHALL not show an action result. A disconnect or first-bank hit SHALL remove the panel mark.

#### Scenario: Panel feedback follows the active connection and page
- **WHEN** the owner reveals the panel, changes its view and then disconnects the SMC-PAD
- **THEN** the badge, header and labels follow that connection and view, then disappear on disconnect
- **AND** Redo and Panic remain visibly unavailable and no first-reveal command result appears

#### Scenario: The header names the view and its neighbors
- **WHEN** the panel shows the Chop actions at either supported size
- **THEN** the header shows the screens view on the left and the Chop actions in the middle
- **AND** the action cells and the screen cells use different colors

#### Scenario: Grey, orange and empty cells look different
- **WHEN** the panel shows Pads with an empty selected pad, and then Chop with cuts, at either supported size
- **THEN** the pad cells that need a sound are grey, the Chop cut actions are orange and the free cells show no label

### Requirement: A panel cell hit lights the cell

A pad hit on a visible panel cell SHALL light that cell for 120 ms, the time of a pad hit flash.
An active cell SHALL light in its own color. A grey cell SHALL light grey, run no action and show a notice that says why. An empty cell SHALL light grey and run no action.
A hit that the panel does not take SHALL light no cell.

#### Scenario: A cell hit lights the cell
- **WHEN** the owner hits an active cell and then a grey cell on the visible panel
- **THEN** each hit lights its own cell for 120 ms, the active one in its color and the grey one grey
- **AND** the grey cell runs no action and a notice says why

### Requirement: A mode line shows the next step

A mode line SHALL show above the hints while a mode, an item in hand or an operation is active. It SHALL have four parts in this order: the mode, the item in hand or the target, the next step and the exit control. The line SHALL stay after the owner releases PAD BANK. It SHALL use the words of the last input device. With the panel shown, the LEFT and RIGHT hint SHALL name the panel views. With the panel hidden, the hint SHALL name what LEFT and RIGHT do on the screen, for example pad edit pages or slice pages. It SHALL NOT hide notices. Without an active mode the line SHALL be empty.

#### Scenario: The mode line shows an item in hand
- **WHEN** the owner takes a file with the SMC-PAD and releases PAD BANK
- **THEN** the mode line names the file, says to hit a pad and names PAD BANK and pad 1 as cancel

#### Scenario: The mode line shows a running recording
- **WHEN** an output recording runs
- **THEN** the mode line shows the recording time and the control that stops it

#### Scenario: The mode line shows the next Assign slice
- **WHEN** an Assign plan with two of nine slices placed is open
- **THEN** the mode line names slice 3 as the next slice, the apply control and the cancel control

#### Scenario: Chop names slice pages before an assign
- **WHEN** Chop has 20 slices and no assigned pad, and the panel is hidden
- **THEN** the LEFT and RIGHT hint names the slice page, not the bank

### Requirement: The mode line fits at 80 by 26

At 80×26, the mode line SHALL keep its exit control visible in full. The middle parts SHALL give way first. A line with apply and cancel SHALL keep both. Hint columns SHALL give way to the mode line.

#### Scenario: Assign keeps apply and cancel at 80 by 26
- **WHEN** the owner opens Assign and hits a pad at 80×26
- **THEN** the mode line names the apply and the cancel control in full

#### Scenario: Pad edit keeps its close control at 80 by 26
- **WHEN** the owner opens pad edit with the SMC-PAD on each of its five pages at 80×26
- **THEN** the mode line ends with «PAD BANK + 1 close» in full

## MODIFIED Requirements

### Requirement: Panel fixed cells keep physical positions

The panel SHALL draw 16 cells in the same top-to-bottom order as the physical pads.
Each cell SHALL show the number printed on the device for that pad.
Its bottom row SHALL be the control row on every view: device pad 1 back, pad 2 undo,
pad 3 redo and pad 4 done. Redo SHALL be dim when no applicable step waits.
Pad 4 SHALL be dim when the view has nothing to confirm or apply.
The panel SHALL NOT show Panic.

#### Scenario: Fixed cells stay on the bottom row
- **WHEN** the owner changes panel views or screens
- **THEN** device pads 1 through 4 keep back, undo, redo and done in physical left-to-right order
- **AND** P3 runs Redo when a step waits, and no cell shows Panic

#### Scenario: Cells show the device numbers
- **WHEN** the panel shows at either supported size
- **THEN** the bottom-left cell shows 1 and the top-right cell shows 16

### Requirement: Undo says what came back

After undo on Chop, Pads or A1, a new notice SHALL replace the current notice. It SHALL say what came back, in the words of the action. With nothing to undo, the undo cell SHALL be grey and a press SHALL say «nothing to undo».

#### Scenario: Undo after cut by hits names the old slices
- **WHEN** `cut by hits` made 9 slices from 5 and the owner presses undo
- **THEN** the chop has 5 slices and the notice says «undo: 5 slices back (were 9)»

#### Scenario: Undo after a join names the split
- **WHEN** the owner joins slices 1 and 2 and presses undo
- **THEN** the notice says «undo: slices 1 and 2 split again»

#### Scenario: Undo with nothing to undo is grey
- **WHEN** nothing can be undone and the owner holds PAD BANK on Chop
- **THEN** the undo cell is grey and a press says «nothing to undo»

### Requirement: A hold in target mode shows the mode commands

During a PAD BANK hold with an item in hand or an open Assign plan, the panel SHALL show the commands of the mode. Device pad 1 SHALL cancel, pad 2 SHALL undo and pad 3 SHALL open the menu. Other cells SHALL be empty. In that menu, P3 SHALL run Redo and P1 SHALL return to the targets. With PAD BANK `Off`, the panel SHALL show the same commands, and a first-bank hit SHALL hide the panel and place the item.

#### Scenario: A hold in target mode shows only the mode commands
- **WHEN** an item is in hand and the owner holds PAD BANK
- **THEN** the panel shows cancel, undo and menu on the control row and the other cells are empty
- **AND** a release without a hit keeps the item in hand

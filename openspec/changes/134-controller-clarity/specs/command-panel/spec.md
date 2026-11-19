## MODIFIED Requirements

### Requirement: Panel fixed cells keep physical positions

The panel SHALL draw 16 cells in the same top-to-bottom order as the physical pads.
Each cell SHALL show the number printed on the device for that pad.
Its bottom row SHALL be the control row on every view: device pad 1 back, pad 2 undo,
pad 3 redo and pad 4 done. Redo SHALL be dim and inactive until change 93a.
Pad 4 SHALL be dim when the view has nothing to confirm or apply.
The panel SHALL NOT show Panic.

#### Scenario: Fixed cells stay on the bottom row
- **WHEN** the owner changes panel views or screens
- **THEN** device pads 1 through 4 keep back, undo, redo and done in physical left-to-right order
- **AND** Redo is dim and runs no action, and no cell shows Panic

#### Scenario: Cells show the device numbers
- **WHEN** the panel shows at either supported size
- **THEN** the bottom-left cell shows 1 and the top-right cell shows 16

## REMOVED Requirements

### Requirement: Visible questions use the two agreed pads

**Reason**: Questions now use device pads 1 and 4, the same pads as Back and Apply.

**Migration**: Use the requirement "Visible questions answer with pads 1 and 4".

### Requirement: SMC-PAD target cells select pads on the current layer

**Reason**: The panel grid of 12 targets hid pads 1–4 and showed no pad names. The playing layer gives all 16 pads with their names.

**Migration**: Use the requirement "Placement targets use the playing layer". Pad hits without a hold place the item. A hold shows the mode commands.

## ADDED Requirements

### Requirement: Visible questions answer with pads 1 and 4

While a question is visible, device pad 1 SHALL cancel and device pad 4 SHALL confirm on press. This SHALL apply with owner bindings and a hidden panel. Both pads SHALL answer from either note bank without a hold. Other pads SHALL stay silent. PLAY and STOP SHALL never answer and MAY keep their transport roles. A release SHALL NOT answer. Keyboard answers SHALL keep their rules.

#### Scenario: Pad 1 and pad 4 answer a visible question on the first hit
- **WHEN** a question is visible and the owner presses device pad 1 or pad 4 from a hidden panel in separate runs
- **THEN** pad 1 cancels and pad 4 confirms through the normal question action
- **AND** neither press first reveals the panel or acts as an ordinary cell

#### Scenario: A question takes pads 1 and 4 without a hold
- **WHEN** a question is visible, the hold shortcut is on and PAD BANK is not held
- **THEN** device pads 1 and 4 from either note bank answer the question
- **AND** a hit on another pad stays silent and the question stays

#### Scenario: A panel action followed by pad 4 confirms only the question
- **WHEN** a destructive panel cell opens a question and device pad 4 arrives before the next redraw
- **THEN** pad 4 confirms only that visible question once
- **AND** no earlier cell action repeats and no second target changes

#### Scenario: PLAY and STOP do not answer a pad 1 and pad 4 question
- **WHEN** a question is visible and PLAY or STOP sends a tap
- **THEN** the context playback action runs where available
- **AND** the question and its target stay unchanged

#### Scenario: A held pad 4 release does not answer
- **WHEN** the owner holds device pad 4 as a Gate pad, a question opens and the owner releases the pad
- **THEN** the Gate sound ends once and the question stays open

### Requirement: Placement targets use the playing layer

An item in hand is a file, a recording, a copy or a swap. With an item in hand or an open Assign plan, a first-layer pad hit without a hold SHALL place the item on that pad. The existing placement result and questions SHALL apply. All 16 pads of the current bank SHALL be targets. Panel commands SHALL belong to the connection that owns the panel. Original held releases SHALL keep priority.

#### Scenario: All sixteen pads take an item in hand
- **WHEN** a file is in hand and the owner hits device pad 1, then takes another file and hits device pad 16
- **THEN** each pad gets its file with the existing placement result and no layer change

#### Scenario: Assign places slices on the playing layer and applies on a hold
- **WHEN** the owner opens Assign, hits device pads 1 and 16 without a hold, holds PAD BANK and presses pad 4
- **THEN** the plan places the next two slices on those pads and applies once
- **AND** pad 1 during the hold cancels the plan instead in a separate run

#### Scenario: Target pads keep their connection and release
- **WHEN** another SMC-PAD sends a panel cancel or a held first-layer pad releases during target choice
- **THEN** the item stays in hand and the original held pad releases
- **AND** a later pad hit places the item as before

### Requirement: A hold in target mode shows the mode commands

During a PAD BANK hold with an item in hand or an open Assign plan, the panel SHALL show the commands of the mode. Device pad 1 SHALL cancel, pad 2 SHALL undo and pad 3 SHALL open the menu. Pad 4 SHALL apply an Assign plan. During Assign, the top row SHALL show `order`. Other cells SHALL be dim. With PAD BANK `Off`, the panel SHALL show the same commands, and a first-bank hit SHALL hide the panel and place the item.

#### Scenario: A hold in target mode shows only the mode commands
- **WHEN** an item is in hand and the owner holds PAD BANK
- **THEN** the panel shows cancel, undo and menu on the control row and dims the other cells
- **AND** a release without a hit keeps the item in hand

### Requirement: The panel title keeps the screen name

While the panel shows, the title SHALL keep the current screen name, and the header above the cells SHALL name the panel view. Badges, the BPM and the mode line SHALL keep their own columns. Text that does not fit SHALL be cut at its end and SHALL NOT overwrite other text. While the panel shows, the LEFT and RIGHT hint SHALL name the panel views.

#### Scenario: The panel title names the screen without overlap
- **WHEN** the panel shows on Chop with LISTEN on, and on Files with a file in hand, at both supported sizes
- **THEN** the title names CHOP or FILES and each badge reads in full or cut at its end
- **AND** the LEFT and RIGHT hint names the panel views

### Requirement: Cells behind a question show only the answers

While a question is visible and the panel shows, device pad 1 SHALL read `no` and device pad 4 SHALL read `yes`. All other cells SHALL be dim and empty.

#### Scenario: The panel behind a question shows no and yes
- **WHEN** a replace question opens from a panel cell
- **THEN** the panel shows `no` on pad 1, `yes` on pad 4 and no other label

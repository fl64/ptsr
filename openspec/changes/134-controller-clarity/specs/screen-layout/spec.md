## ADDED Requirements

### Requirement: Pads use the device numbers with an SMC-PAD connection

While an SMC-PAD Master port has a connection, each pad frame SHALL show the device number of its position. Pad 1 is bottom left and pad 16 is top right. The keyboard key SHALL show in the frame corner. The side panel, messages and questions SHALL name pads by the same number with the bank. Without an SMC-PAD connection, the grid SHALL show the keyboard keys as now. Saved files and pad indexes SHALL stay unchanged.

#### Scenario: The printed pad 1 reads as pad 1
- **WHEN** an SMC-PAD has a connection and the owner hits the device pad printed 1
- **THEN** the bottom-left frame shows 1 with the key Z in its corner
- **AND** the side panel names pad 1 of the current bank

#### Scenario: Keyboard numbers return without an SMC-PAD
- **WHEN** the SMC-PAD disconnects
- **THEN** the grid shows the keyboard keys again and the selected pad stays the same

### Requirement: Page rows name the knob that turns them

A row can belong to a pad edit page, the FX screen or a picker. If an SMC-PAD knob turns the row, the row SHALL show that knob number. The row and the knob strip SHALL use the same parameter name. A row without a knob SHALL show no number.

#### Scenario: Pad edit rows show knobs 7, 5 and 3
- **WHEN** the owner opens pad edit TONE with an SMC-PAD connection
- **THEN** the rows read 7 volume, 5 pitch and 3 group, and the knob strip uses the same three words

#### Scenario: FX rows show their knobs
- **WHEN** the owner opens the FX screen with an SMC-PAD connection
- **THEN** each slot row shows the knob that turns it, and no row shows a number of another knob

### Requirement: A mode line shows the next step

A mode line SHALL show above the hints while a mode, an item in hand or an operation is active. It SHALL name the mode, the item or target, the next step and the cancel control. The line SHALL stay after the owner releases PAD BANK. It SHALL use the words of the last input device. It SHALL NOT hide notices. Without an active mode the line SHALL be empty.

#### Scenario: The mode line shows an item in hand
- **WHEN** the owner takes a file with the SMC-PAD and releases PAD BANK
- **THEN** the mode line names the file, says to hit a pad and names PAD BANK and pad 1 as cancel

#### Scenario: The mode line shows a running recording
- **WHEN** an output recording runs
- **THEN** the mode line shows the recording time and the control that stops it

#### Scenario: The mode line shows the next Assign slice
- **WHEN** an Assign plan with two of nine slices placed is open
- **THEN** the mode line names slice 3 as the next slice, the apply control and the cancel control

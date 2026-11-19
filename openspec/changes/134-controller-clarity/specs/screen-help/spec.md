## ADDED Requirements

### Requirement: Hints follow the last input device

The hint lines, the mode line, question answer rows and mode texts SHALL use the words of the last input device. After an SMC-PAD press, hit or knob turn, they SHALL name device pads, buttons and knobs that work through current production dispatch. After a key, they SHALL name keys as now. A release SHALL NOT change the device. A change of device SHALL redraw these texts at once. F1 SHALL keep both inputs. A question SHALL keep its target and safety text.

#### Scenario: SMC-PAD input switches hints to controller words
- **WHEN** the owner takes a file with the SMC-PAD
- **THEN** the hints and the in-hand text name pad hits and PAD BANK with pad 1, and name no arrow keys or Enter

#### Scenario: A key switches hints back
- **WHEN** after an SMC-PAD input the owner presses a keyboard key on Pads
- **THEN** the hints name keys again

#### Scenario: A question changes answer words with the device
- **WHEN** a question shows after a keyboard input and the owner then turns an SMC-PAD knob
- **THEN** the answer row names pad 1 and pad 4, and the question and its target stay

### Requirement: Controller Help draws the device as printed

Controller Help SHALL draw the SMC-PAD pads with the printed numbers, 1 bottom left and 16 top right. It SHALL show that a PAD BANK hold shows the actions of the current screen, and that the screens view is one LEFT press away.

#### Scenario: Help shows pad 1 bottom left
- **WHEN** the owner opens Controller Help with an SMC-PAD connection
- **THEN** the bottom-left pad reads 1, the top-right pad reads 16, and Help names the second layer as the actions of the current screen

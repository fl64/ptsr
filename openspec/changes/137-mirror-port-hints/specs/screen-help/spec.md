## MODIFIED Requirements

### Requirement: Hints follow the last input device

The hint lines, the mode line, question answer rows and mode texts SHALL use the words of the last input device. After an SMC-PAD press, hit or knob turn, they SHALL name device pads, buttons and knobs that work through current production dispatch. After a key, they SHALL name keys as now. A release SHALL NOT change the device. A message on the SMC-PAD Private port SHALL NOT change the device, because that port repeats the Master port. A change of device SHALL redraw these texts at once. F1 SHALL keep both inputs. A question SHALL keep its target and safety text.

#### Scenario: SMC-PAD input switches hints to controller words
- **WHEN** the owner takes a file with the SMC-PAD
- **THEN** the hints and the in-hand text name pad hits and PAD BANK with pad 1, and name no arrow keys or Enter

#### Scenario: A key switches hints back
- **WHEN** after an SMC-PAD input the owner presses a keyboard key on Pads
- **THEN** the hints name keys again

#### Scenario: A question changes answer words with the device
- **WHEN** a question shows after a keyboard input and the owner then turns an SMC-PAD knob
- **THEN** the answer row names pad 1 and pad 4, and the question and its target stay

#### Scenario: A mirrored hit keeps controller words
- **WHEN** the owner hits a pad and the Private port repeats the hit before or after the Master port
- **THEN** the hints keep SMC-PAD words

#### Scenario: A mirrored message after a key keeps keyboard words
- **WHEN** the owner presses a key and then only the Private port sends a message
- **THEN** the hints keep keyboard words

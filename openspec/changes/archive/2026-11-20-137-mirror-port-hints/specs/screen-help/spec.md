## ADDED Requirements

### Requirement: The SMC-PAD Private port does not change the input device

The SMC-PAD Private port repeats the Master port and runs no action. A message on the Private port SHALL NOT change the last input device.

#### Scenario: A mirrored hit keeps controller words
- **WHEN** the owner hits a pad and the Private port repeats the hit before or after the Master port
- **THEN** the hints keep SMC-PAD words

#### Scenario: A mirrored message after a key keeps keyboard words
- **WHEN** the owner presses a key and then only the Private port sends a message
- **THEN** the hints keep keyboard words

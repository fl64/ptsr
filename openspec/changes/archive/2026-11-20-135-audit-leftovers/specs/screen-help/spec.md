## ADDED Requirements

### Requirement: The marking hint follows the last input device

After an SMC-PAD input, the marking hint SHALL name the SMC-PAD control that drops the marks. After a key, it SHALL name the key.

#### Scenario: Marking after SMC-PAD input names PAD BANK and pad 1
- **WHEN** the owner turns marking on from the SMC-PAD panel
- **THEN** the hint names PAD BANK and pad 1 to drop the marks, and names no Esc

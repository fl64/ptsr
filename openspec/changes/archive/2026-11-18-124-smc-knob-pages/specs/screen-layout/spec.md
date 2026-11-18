## ADDED Requirements

### Requirement: The SMC-PAD knob strip shows both banks and the current list

The strip SHALL show both banks side by side at 80 by 26 and 120 by 40.
Each bank SHALL show rows 7–8, 5–6, 3–4 and 1–2 from top to bottom.
The strip SHALL name the page and dim unavailable roles.
The help row SHALL show the current role of LEFT and RIGHT.
The mark SHALL show the last valid knob bank. Disconnect SHALL clear the mark.
The F1 Controller tab SHALL show the same effective roles.

#### Scenario: The strip identifies page, list and roles at both sizes
- **WHEN** the owner views each supported size with the SMC-PAD connected
- **THEN** both knob banks fit in physical order
- **AND** the page, list and unavailable roles are visible

#### Scenario: Last knob input marks its bank
- **WHEN** valid knob movement arrives from bank 1 and then bank 2
- **THEN** the mark changes to the bank that sent the last turn
- **AND** a neutral value or an absent action changes no control value
- **AND** a disconnect clears the mark and stale input cannot set it again

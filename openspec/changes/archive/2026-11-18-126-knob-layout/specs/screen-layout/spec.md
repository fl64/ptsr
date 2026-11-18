## MODIFIED Requirements

### Requirement: The SMC-PAD knob strip shows both banks and the current list

The strip SHALL show both banks side by side at 80 by 26 and 120 by 40.
Each bank SHALL show rows 7–8, 5–6, 3–4 and 1–2 from top to bottom.
The strip SHALL name the page and dim unavailable roles.
The help row SHALL show the current role of LEFT and RIGHT.
The mark SHALL show the last valid knob and its bank. Disconnect SHALL clear it.
While the value pop-up shows, the help row SHALL name that knob, value, target and amount.
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

#### Scenario: The strip shows the turned knob and its value
- **WHEN** the owner turns bank 1 knob 7 in pad edit on TONE at either supported size
- **THEN** the strip marks knob 7 of bank 1
- **AND** the help row shows knob 7, volume, the pad and the new value until the pop-up ends

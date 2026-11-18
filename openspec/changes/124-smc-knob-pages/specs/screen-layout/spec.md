## ADDED Requirements

### Requirement: The SMC-PAD knob strip shows both banks and the current list

The knob strip SHALL show both banks side by side at 80 by 26 and 120 by 40.
Each bank SHALL show its physical rows in this order from top to bottom: 7–8, 5–6, 3–4, 1–2.
The strip SHALL name the page and dim unavailable roles.
The strip and one-row help SHALL name the LEFT and RIGHT roles.
The strip SHALL mark the bank of the last valid turn from its connection.
A disconnect SHALL clear the last input mark.
The F1 Controller tab SHALL show the same effective roles.

#### Scenario: The strip identifies page, list and roles at both sizes
- **WHEN** the owner views each supported size with the SMC-PAD connected
- **THEN** both knob banks fit in physical order
- **AND** the strip and help name the current page or list and unavailable roles stay dim

#### Scenario: Last knob input marks its bank
- **WHEN** valid knob movement arrives from bank 1 and then bank 2
- **THEN** the mark changes to the bank that sent the last turn
- **AND** no mark changes while a bank switch sends no message
- **AND** a disconnect clears the mark and stale input cannot set it again

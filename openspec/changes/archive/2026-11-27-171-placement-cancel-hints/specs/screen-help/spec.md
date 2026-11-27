## ADDED Requirements

### Requirement: An item in hand names the available way back

Copy, Swap and a file in hand SHALL show the applicable cancel control. When the hold shortcut is unavailable, P1 SHALL show the panel first and cancel on the next press. An open target menu SHALL name P1 as Back to targets. The keyboard hint SHALL name Esc. A short mode line SHALL keep the full `IN HAND` label.

#### Scenario: Placement hints follow reveal and target menu state
- **WHEN** an item is in hand and the owner uses SMC-PAD without a working hold shortcut
- **THEN** the hidden panel hint names two P1 presses in pad bank 2
- **AND** the target panel names pad bank 1 for placement and one P1 for cancel
- **AND** P1 in the target menu returns to the targets and keeps the item in hand

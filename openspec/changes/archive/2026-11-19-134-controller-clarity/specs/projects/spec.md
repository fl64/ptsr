## MODIFIED Requirements

### Requirement: The exit questions use physical SMC-PAD numbers

Both questions SHALL show the pad numbers printed on the device.
P1 SHALL cancel and P4 SHALL confirm from either pad bank without a PAD BANK hold.

#### Scenario: The printed SMC-PAD numbers answer both exit questions

- **WHEN** Exit or its recovery question is visible and the owner presses P1 or P4 in either SMC-PAD bank
- **THEN** the question shows the printed pad numbers
- **AND** P1 cancels or P4 confirms once, even with the PAD BANK shortcut off or unavailable

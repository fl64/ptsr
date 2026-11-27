## ADDED Requirements

### Requirement: A1 has one active edit mode

Step Edit and A1 Setup SHALL open one at a time. A request to open one over the other SHALL name the open mode and keep the current cursor, bank and selected event. Back SHALL close the visible question, Help or A1 mode first. On a Setup value row, Enter SHALL close Setup. During count-in, a Step Edit request SHALL keep the wait notice.

#### Scenario: Step Edit and Setup stay in order

- **WHEN** the owner tries to open one mode while the other is open with a key or SMC-PAD
- **THEN** the open mode stays visible and the notice names it
- **AND** Back over a question or Help returns to that mode before it can close the mode itself
- **AND** Enter on a Setup value row closes Setup

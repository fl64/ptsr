## MODIFIED Requirements

### Requirement: Assigning a sample by hitting a pad
The system SHALL let the owner take a file "in hand" on the files screen with `Ctrl+Enter` (`Enter` loads it onto the selected pad at once, 23) and put it on a pad by hitting that pad from the keyboard or a controller; Esc puts the file back. Without a file in hand, pad hits play as usual.

#### Scenario: Put a break on a pad
- **WHEN** the owner takes a break in hand on the files screen and hits pad W
- **THEN** the break is assigned to pad 6 of the current bank, the next hit on W plays it

#### Scenario: Change of mind
- **WHEN** a file is in hand and the owner presses Esc
- **THEN** no pad changes, and pad hits play again

## MODIFIED Requirements

### Requirement: Assigning a sample by hitting a pad
The system SHALL let the owner take a file "in hand" on the files screen with `Enter`. A hit on a pad from the keyboard or a controller puts the file on that pad. `Enter` again puts the file on the selected pad. `Ctrl+Enter` loads the file onto the selected pad at once, without the hand (41). `Esc` puts the file back. Without a file in hand, pad hits play as usual.

#### Scenario: Put a break on a pad
- **WHEN** the owner presses `Enter` on a break on the files screen and hits pad W
- **THEN** the break is assigned to pad 6 of the current bank, the next hit on W plays it

#### Scenario: Change of mind
- **WHEN** a file is in hand and the owner presses Esc
- **THEN** no pad changes, and pad hits play again

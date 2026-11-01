## ADDED Requirements

### Requirement: Assigning a sample by hitting a pad
The system SHALL let the owner take a file "in hand" on the files screen with Enter and put it on a pad by hitting that pad from the keyboard or a controller; Esc puts the file back. Without a file in hand, pad hits play as usual.

#### Scenario: Put a break on a pad
- **WHEN** the owner takes a break in hand on the files screen and hits pad W
- **THEN** the break is assigned to pad 6 of the current bank, the next hit on W plays it

#### Scenario: Change of mind
- **WHEN** a file is in hand and the owner presses Esc
- **THEN** no pad changes, and pad hits play again

### Requirement: Saving the session
The system SHALL save pad samples, modes and volumes of all banks and the last browser folder on every change and on exit, and restore them on the next start.

#### Scenario: Restart
- **WHEN** the owner closes and opens the program again
- **THEN** all pads of all banks sound the same as before closing, and the files screen opens in the same folder

#### Scenario: File is gone
- **WHEN** an assigned file is deleted from disk
- **THEN** the pad is marked as "missing file" and stays silent, the others work

# pad-assignment Specification

## Purpose
Putting samples on pads: a file in hand from the files screen goes to the next pad hit; the session keeps samples, modes and volumes of all pads.

## Requirements

### Requirement: Assigning a sample by hitting a pad
The system SHALL let the owner take a file "in hand" on the files screen with `Enter`. A hit on a pad from the keyboard or a controller puts the file on that pad. `Enter` again puts the file on the selected pad. `Ctrl+Enter` loads the file onto the selected pad at once, without the hand (41). `Esc` puts the file back. Without a file in hand, pad hits play as usual.

#### Scenario: Put a break on a pad
- **WHEN** the owner presses `Enter` on a break on the files screen and hits pad W
- **THEN** the break is assigned to pad 6 of the current bank, the next hit on W plays it

#### Scenario: Change of mind
- **WHEN** a file is in hand and the owner presses Esc
- **THEN** no pad changes, and pad hits play again

### Requirement: Saving the session
The system SHALL save pad samples, modes and volumes of all banks and the last browser folder on every change and on exit, and restore them on the next start; with a project open the session saves into that project's `project.toml` (sample paths inside the project folder relative to it), without a project — into the scratch session file as today.

#### Scenario: Restart
- **WHEN** the owner closes and opens the program again
- **THEN** all pads of all banks sound the same as before closing, and the files screen opens in the same folder

#### Scenario: File is gone
- **WHEN** an assigned file is deleted from disk
- **THEN** the pad is marked as "missing file" and stays silent, the others work

#### Scenario: Saved into the project
- **WHEN** a project is open and the session autosaves
- **THEN** the `project.toml` of that project changes and the scratch session file stays untouched

### Requirement: Swapping pads
The system SHALL let the owner take the selected pad "in hand" with Ctrl+X and swap it with the next pad that is hit; everything a pad has moves together: the sample and the part of it it plays, the mode, the volume, the pitch, the envelope, the filter, the drive and the sends. Esc cancels.

#### Scenario: Swap two pads
- **WHEN** pad Q is selected, the owner presses Ctrl+X and then hits pad V
- **THEN** Q now plays what V played and V plays what Q played, with their modes and volumes; the session is saved

#### Scenario: Every setting moves
- **WHEN** pad Q has a low-pass filter, pitch +3 and a drive, pad V has none of them, and
  the owner swaps them
- **THEN** pad V has the low-pass filter, pitch +3 and the drive, sounds with them, and pad
  Q has none of them

#### Scenario: Move to an empty pad
- **WHEN** the pad in hand is put on an empty pad
- **THEN** the sample moves there and the first pad becomes empty

#### Scenario: Swap while loading
- **WHEN** one of the two pads is still loading a file
- **THEN** the pads are not swapped and the status says to wait until loading finishes

### Requirement: A pad in hand is named
The system SHALL, while a pad is in hand, name it on the pads screen in a short line that fits the panel (`in hand: pad 3 · amen 3`; the help line says what to do) and swap it with the next pad hit; `Esc` puts it back.

#### Scenario: Swap by a hit
- **WHEN** pad W is in hand and the owner hits pad E
- **THEN** pads W and E swap, and the panel no longer names a pad in hand

## MODIFIED Requirements

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

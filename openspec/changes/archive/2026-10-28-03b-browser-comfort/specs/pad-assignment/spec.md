## ADDED Requirements

### Requirement: Swapping pads
The system SHALL let the owner take the selected pad "in hand" with Ctrl+X and swap it with the next pad that is hit; the sample, mode and volume move together. Esc cancels.

#### Scenario: Swap two pads
- **WHEN** pad Q is selected, the owner presses Ctrl+X and then hits pad V
- **THEN** Q now plays what V played and V plays what Q played, with their modes and volumes; the session is saved

#### Scenario: Move to an empty pad
- **WHEN** the pad in hand is put on an empty pad
- **THEN** the sample moves there and the first pad becomes empty

#### Scenario: Swap while loading
- **WHEN** one of the two pads is still loading a file
- **THEN** the pads are not swapped and the status says to wait until loading finishes

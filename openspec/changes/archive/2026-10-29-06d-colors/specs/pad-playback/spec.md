## ADDED Requirements

### Requirement: Bank colors
The system SHALL give each of the eight banks its own soft color, show every bank in the bank row in its color and draw the frames of the open bank's pads in that color; the colors of state (selected cyan, in hand magenta, hit white) stay above the bank color.

#### Scenario: Bank colors
- **WHEN** the owner switches from bank 1 to bank 2
- **THEN** the idle pad frames change from the color of bank 1 to the color of bank 2, and the bank row shows each of the eight banks in its own color

#### Scenario: Meanings stay
- **WHEN** a pad of bank 3 is selected and another one is hit
- **THEN** the selected pad's frame is cyan and the hit pad flashes white, not the bank color

## MODIFIED Requirements

### Requirement: Live output strip
The system SHALL show on the pads screen, in the strip above the grid, a live envelope of the master output at the height of the shared strip: the sound of the last few seconds scrolls right-to-left, what plays right now is at the right edge. The strip is scaled in decibels like the meters, so a peak's height on the strip matches its bar. A silent output is a thin middle line. The strip moves only while sound plays.

#### Scenario: I see what I play
- **WHEN** the owner hits a loud pad and then a quiet one
- **THEN** a tall wave runs from the right edge and a smaller one follows it, both moving left

#### Scenario: The strip agrees with the meters
- **WHEN** a peak of −6 dB plays and the owner compares the wave with the L/R bars
- **THEN** the wave rises to nearly the full height, like the bars, not to half of it

#### Scenario: Silence is a line
- **WHEN** nothing plays
- **THEN** the strip is a thin middle line and does not move

#### Scenario: Quiet and loud do not merge
- **WHEN** a quiet pad (−40 dB) plays after a loud one
- **THEN** the quiet wave is clearly lower than half the strip's height, not the same fat block

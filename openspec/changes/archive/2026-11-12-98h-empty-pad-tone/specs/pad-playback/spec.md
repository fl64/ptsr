# pad-playback Delta

## ADDED Requirements

### Requirement: The pad edit of an empty pad shows its test tone

In the pad edit of an empty pad, the strip SHALL show the pad's test tone as its piece, with the envelope line over it. The live output SHALL NOT show then.

#### Scenario: The pad edit of an empty pad shows its test tone
- **WHEN** the owner opens the pad edit on an empty pad and raises its attack
- **THEN** the strip shows the decaying test tone with the envelope line, not the live output
- **AND** the attack part of the line grows with the attack

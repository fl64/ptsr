## MODIFIED Requirements

### Requirement: A pad in hand is named
The system SHALL, while a pad is in hand, name it on the pads screen in a short line that fits the panel (`in hand: pad 3 · amen 3`; the help line says what to do) and swap it with the next pad hit; `Esc` puts it back.

#### Scenario: Swap by a hit
- **WHEN** pad W is in hand and the owner hits pad E
- **THEN** pads W and E swap, and the panel no longer names a pad in hand

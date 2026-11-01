## REMOVED Requirements

### Requirement: Choosing the pad with arrows
**Reason**: the arrows do not pick a pad (owner, 2026-10-30: "there are keys for that"); a pad hit puts the file in hand on that pad at once.
**Migration**: hit the pad; Enter on the files screen follows 23-files-enter.

### Requirement: Swapping with arrows
**Reason**: the arrows do not pick a pad; a pad hit swaps at once.
**Migration**: replaced by "A pad in hand is named": the panel names the pad in hand, a hit swaps.

## ADDED Requirements

### Requirement: A pad in hand is named
The system SHALL, while a pad is in hand, name it on the pads screen and swap it with the next pad hit; `Esc` puts it back.

#### Scenario: Swap by a hit
- **WHEN** pad W is in hand and the owner hits pad E
- **THEN** pads W and E swap, and the panel no longer names a pad in hand

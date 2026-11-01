## MODIFIED Requirements

### Requirement: Favorite the folder of a file
The system SHALL, on the favorite command with a file selected, mark the folder being
shown as a favorite (or unmark it) and say so in the status, naming it the folder of this
file.

#### Scenario: Ctrl+B on a file
- **WHEN** the owner is in "breaks" with a file selected and presses Ctrl+B
- **THEN** the status says "★ breaks: favorite (the folder of this file)" and one level up
  "breaks" has a star; a second Ctrl+B says "breaks: no longer a favorite (the folder of
  this file)"

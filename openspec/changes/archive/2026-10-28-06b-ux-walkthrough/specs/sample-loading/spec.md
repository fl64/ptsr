## ADDED Requirements

### Requirement: Favorite the folder of a file
The system SHALL, on the favorite command with a file selected, mark the folder being shown as a favorite (or unmark it) and say so in the status.

#### Scenario: Ctrl+B on a file
- **WHEN** the owner is in "breaks" with a file selected and presses Ctrl+B
- **THEN** the status says "★ breaks: favorite" and one level up "breaks" has a star

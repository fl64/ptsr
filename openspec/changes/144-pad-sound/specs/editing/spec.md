## ADDED Requirements

### Requirement: Pad edits keep the sound shape

Copy, swap, Undo and Redo SHALL retain the pad sound settings and the existing scope of each edit.

#### Scenario: Copy and undo preserve the pad sound shape

- **WHEN** the owner copies a shaped pad and then changes and undoes its settings
- **THEN** the copy has the same sound shape and the earlier settings return on Undo

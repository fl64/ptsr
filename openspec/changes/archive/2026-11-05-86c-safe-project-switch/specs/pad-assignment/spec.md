## ADDED Requirements

### Requirement: Preserve scratch work on project transitions

Pending scratch work SHALL save before another project opens or a new project is created.
If the source has pending work and read protection, the transition SHALL refuse without clearing either state.
A clean protected scratch session SHALL permit leaving without writing its source file.

#### Scenario: Scratch edits save before opening or creating a project
- **WHEN** scratch has pending edits and the owner opens or creates a project before autosave
- **THEN** the scratch file retains those edits before the target replaces the session

#### Scenario: Pending protected work blocks a transition
- **WHEN** a protected project or scratch session has pending work and the owner tries to open or create another project
- **THEN** the transition refuses without changing the source, pending work, undo or target

#### Scenario: Clean protected scratch can leave without a write
- **WHEN** scratch has read protection but no pending work and the owner opens another project
- **THEN** the target opens without a source save and the protected scratch file stays byte for byte unchanged

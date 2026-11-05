## ADDED Requirements

### Requirement: Project paths include every saved chop

Project path conversion SHALL include pads, the current chops and every saved older chop variant.
Paths inside the project SHALL store relative to its folder and resolve against its current folder on load.
External absolute paths SHALL stay unchanged. No sample SHALL be copied.
The saved format and scratch path behavior SHALL stay unchanged.

#### Scenario: Internal variant paths move with the project
- **WHEN** a project with several saved variants of an internal sample is saved, moved as a whole folder and opened from its new location
- **THEN** pads, newest chops and every older variant use the sample in the new folder
- **AND** saving stores their internal paths relative to that folder

#### Scenario: External variant paths remain absolute
- **WHEN** a project with several variants of an external sample is saved, moved and reopened
- **THEN** all saved variant keys and pad paths keep the same external absolute path
- **AND** the external file stays byte for byte unchanged and no sample copy appears in the project

#### Scenario: Save and reopen after project relocation
- **WHEN** the moved project is saved and opened again
- **THEN** all saved variants remain available with their full state and correct sample paths

### Requirement: Portable chop compatibility

Projects without saved older variants SHALL keep the existing pad-bound fallback.
Stale absolute paths from an earlier move SHALL NOT be replaced by guessed paths.
This change SHALL NOT migrate the saved format or search for replacement samples.

#### Scenario: Missing kept variants use the existing fallback
- **WHEN** an old project has a linked pad but no saved variant for its epoch and the owner opens that pad with F3
- **THEN** the existing fallback rebuilds the chop from its pad bounds without a format migration

#### Scenario: Stale absolute variant paths are not guessed
- **WHEN** a project already contains an older variant key that names an absent old absolute location
- **THEN** loading leaves that key unchanged instead of mapping it to a same-named sample in the new folder
- **AND** the existing fallback remains available for pads whose saved variant cannot be found

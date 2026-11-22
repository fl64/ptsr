## MODIFIED Requirements

### Requirement: Reserved keyboard chords

The shipped map SHALL bind Ctrl+Period to Panic and keep Ctrl+Y free until redo. Ctrl+R SHALL run pattern recording. Chop Ctrl+Y MAY explain its former role. Ctrl+B SHALL keep the Files favorite action. Owner bindings SHALL remain allowed to replace or remove these keys.

#### Scenario: Future commands stay unbound

- **WHEN** the owner loads the default map before redo ships
- **THEN** Ctrl+Y runs no action, except a moved-key notice where defined
- **AND** Ctrl+R records patterns and Ctrl+B changes the favorite on Files

#### Scenario: Panic uses the effective keyboard chord

- **WHEN** the owner presses an effective Panic chord during playback
- **THEN** the sampler stops sound once
- **AND** an explicit owner replacement or removal of Ctrl+Period takes effect

## ADDED Requirements

### Requirement: A held microphone take can end over a question

A keyboard microphone hold SHALL end when its key comes up in either modifier release order. An open question SHALL allow the release of the keyboard or SMC-PAD microphone hold. The release SHALL keep the question open and end only a take that this control started.

#### Scenario: A question lets the microphone hold end

- **WHEN** the owner starts a microphone hold and opens a question before letting go
- **THEN** releasing the keyboard chord in either order or releasing SMC-PAD RECORD stops the take
- **AND** the question stays open
- **AND** a hold that starts no take cannot end a take from another input

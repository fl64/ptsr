# recording Delta

## ADDED Requirements

### Requirement: Pattern recording is separate from audio capture

Pattern recording SHALL store pad events, not microphone or output audio. Its commands SHALL NOT link microphone or monitor ports, create a WAV or put a file in hand. Existing Alt+R hold/toggle and Ctrl+T output capture SHALL keep their separate roles. A learned pattern-record button SHALL never select audio capture by screen or transport state.

#### Scenario: Pattern recording does not start microphone capture
- **WHEN** pattern recording starts, ends or cancels on any current screen
- **THEN** no microphone or output recording starts, no new capture link appears and no WAV lands in hand
- **AND** existing explicit microphone and output controls retain their meaning

### Requirement: Recording targets refuse conflicting starts

Microphone and output recording SHALL remain mutually exclusive. CountIn, Recording and Finishing of a pattern pass SHALL refuse audio-capture starts. Active audio capture SHALL refuse pattern recording. Refusal SHALL name the active target and preserve it. Releases from a refused microphone request SHALL NOT stop the pattern pass or output capture.

#### Scenario: Pattern recording refuses microphone and output starts
- **WHEN** microphone or output capture is requested during pattern count-in, recording or finishing
- **THEN** the request refuses with the active A1 phase and the pattern pass continues
- **AND** microphone release cannot end that pass

#### Scenario: Audio capture refuses a pattern start without changing targets
- **WHEN** microphone or output capture runs and pattern recording is requested
- **THEN** the request refuses and the original capture continues unchanged
- **AND** no empty A1 is created by the refused start

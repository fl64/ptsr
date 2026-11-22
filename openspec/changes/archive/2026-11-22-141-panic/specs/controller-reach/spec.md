## ADDED Requirements

### Requirement: SMC-PAD runs Panic with two STOP presses

With the SMC-PAD command panel open, two distinct STOP presses from its owner connection within 500 ms SHALL run Panic. The first press SHALL leave playback active. Panel close, port disconnect, or expiry SHALL cancel the gesture. Ordinary STOP outside the panel SHALL keep its context action.

#### Scenario: Two STOP presses cut sound from the panel

- **WHEN** the owner opens the SMC-PAD panel and presses STOP twice within 500 ms
- **THEN** the first press arms Panic, and the second stops the sound without a question

#### Scenario: Panic needs two presses on one open panel

- **WHEN** the first press repeats, expires, comes from another connection, or the owner closes the panel before the next press
- **THEN** no Panic action completes
- **AND** ordinary STOP outside the panel keeps its context action

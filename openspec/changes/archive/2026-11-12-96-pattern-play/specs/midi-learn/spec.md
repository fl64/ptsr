# midi-learn Delta

## ADDED Requirements

### Requirement: Learn the Pitch Gate command

F12 MIDI SHALL list `pad.pitch_gate_toggle` as a PADS button action. F10 SHALL select it by its effective key without changing a pad. Existing pending confirmation, cancellation, clear and layer precedence SHALL apply. Screen learn SHALL refuse knobs or faders for this command. A learned press SHALL act once, with no action on release.

#### Scenario: Raw MIDI Pitch Gate button matches the keyboard
- **WHEN** learned note buttons and CC buttons send raw presses and releases through the production profile and dispatch on Pads
- **THEN** each press applies the same scope toggle and undo result as the keyboard
- **AND** release does not toggle again, while a PADS-only assignment does not execute on another screen

#### Scenario: Pitch Gate learning retains confirmation and control types
- **WHEN** F10 or F12 selects Pitch Gate and the owner moves, replaces, confirms, cancels or clears its pending control
- **THEN** existing learn rules retain their pending and saved states without executing Pitch Gate
- **AND** screen learn rejects an absolute knob or fader for the button command

### Requirement: Pitch Gate bindings preserve controller state

New bindings SHALL survive restart without reseeding devices or changing existing owner and factory roles. Playback and the toggle SHALL use current learned transport and PADS routes. The unfinished SMC-PAD example SHALL remain unverified. This change SHALL NOT require a hold, relative encoder or command panel.

#### Scenario: Pitch Gate binding survives restart without a controller reset
- **WHEN** the owner saves a learned Pitch Gate button alongside transport and existing X-Session assignments, then restarts
- **THEN** all those assignments keep their saved roles without profile reset or reseeding
- **AND** the SMC-PAD example gains no claimed factory mapping or physical acceptance

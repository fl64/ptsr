# midi-learn Delta

## ADDED Requirements

### Requirement: Learn pattern recording commands

F12 MIDI SHALL list `pattern.record`, `pattern.controls`, `pattern.erase` and `pattern.undo` as GLOBAL button actions. It SHALL list `pattern.param_prev`, `pattern.param_next`, `pattern.value_dec` and `pattern.value_inc` as buttons. Existing confirmation, clearing, persistence and layer precedence SHALL apply. Presses SHALL execute once. Releases SHALL NOT toggle commands or trigger pad sound.

#### Scenario: Learn and retain recording command buttons
- **WHEN** separate note or CC buttons are learned for all new pattern commands, confirmed and the program restarts
- **THEN** those commands retain their controls without changing existing owner or X-Session bindings
- **AND** pending assignments, Esc cancellation and Backspace clearing keep their existing rules

#### Scenario: Pattern recording buttons work on every screen
- **WHEN** learned pattern buttons send raw MIDI through production profiles and dispatch on every current screen
- **THEN** they match keyboard recording, controls, erasure and pass Undo behavior without changing screens
- **AND** each release produces no second command or extra hit

### Requirement: Learn absolute pattern parameters

F12 MIDI SHALL list `pattern.bars`, `pattern.quantize` and `pattern.strength` as GLOBAL knob parameters. Values 0–127 SHALL map to each permitted value list by nearest index, with ties later. Existing jump, pickup and scale response SHALL apply separately per device and control. Phase restrictions SHALL match keyboard controls.

#### Scenario: Absolute pattern knobs use their complete ranges
- **WHEN** learned knobs send 0, 64 and 127 in jump mode
- **THEN** bars become 1, 33 and 64, quantize becomes Off, 1/32 and 1/32T, and strength becomes 0, 50 and 100
- **AND** changes show the applied pattern parameter without changing raw timing

#### Scenario: Pattern takeover follows the current value and device
- **WHEN** two controllers use pickup or scale and the keyboard changes a pattern setting between their movements
- **THEN** each control follows its own current takeover state and parameter value
- **AND** pickup waiting and phase refusal do not report an unapplied value

### Requirement: Pattern controller input retains causal order

A recording or erasure command SHALL take effect before a following pad event from the same ordered production input stream. Deferred hits SHALL keep their original velocity, source, project and release. Commands or capture from an old project SHALL NOT change the new project's recording. Invalid shared focus SHALL reject new actions without losing old releases.

#### Scenario: Raw recording command precedes its following hit
- **WHEN** raw MIDI starts recording during playback and sends a pad press and release before the next UI update
- **THEN** the accepted hit belongs to that recording pass with its actual velocity and duration
- **AND** ending recording before another hit keeps that later hit out of the pass

#### Scenario: Raw erasure command precedes its target hold
- **WHEN** raw MIDI enables erasure and sends a target press, release and erasure-off command before a redraw
- **THEN** the target remains silent and erases only its accepted interval
- **AND** delayed notifications cannot convert it into a sounding or recorded pad

#### Scenario: Stale controller work cannot enter another recording pass
- **WHEN** old-project commands, capture or deferred notifications arrive after another project or pass becomes current
- **THEN** they cannot add rows, change phase or erase the new pattern
- **AND** releases still complete their original permitted sound ownership

### Requirement: Pattern learning does not imply factory support

New pattern actions SHALL use existing MIDI learning without resetting device bindings. The incomplete SMC-PAD example SHALL remain unverified. Pattern recording SHALL NOT silently select microphone capture by screen or transport state. This change SHALL NOT introduce factory RECORD holds, STOP layers or a tap-and-hold dispatcher.

#### Scenario: Learned pattern recording never changes into microphone capture
- **WHEN** a learned record button is used while stopped or playing on Pads, Files, Chop, FX and Settings
- **THEN** it always addresses A1 recording and never the microphone
- **AND** existing microphone controls remain separate without an invented SMC-PAD factory assignment

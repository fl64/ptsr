# MIDI learn delta

## ADDED Requirements

### Requirement: Learn transport controls

F12 MIDI SHALL list `transport.toggle`, `metronome.toggle`, `transport.tempo_up`, `transport.tempo_down` and their `_fine` forms as GLOBAL button actions. It SHALL list `transport.tempo` as a GLOBAL knob parameter. Existing confirmation, clear, layer precedence, input ordering and saved-binding rules SHALL apply. Press actions SHALL execute once, not again on release.

#### Scenario: Raw MIDI buttons control transport and metronome
- **WHEN** raw MIDI presses and releases reach learned transport and metronome buttons through the production profile and dispatch
- **THEN** their actions match the keyboard path and releases do not repeat them

#### Scenario: Raw MIDI tempo buttons match keyboard steps
- **WHEN** raw MIDI uses learned coarse or fine tempo buttons
- **THEN** each press changes tempo by 1 or 0.1 BPM with the same bounds as keyboard input

#### Scenario: Transport learn selects without playing
- **WHEN** the owner selects new transport controls in F12 MIDI or screen learn and moves a control
- **THEN** the binding remains pending until confirmation and selection does not execute the transport command
- **AND** a knob cannot bind to a button command through screen learn

### Requirement: Absolute tempo knob

A learned tempo knob SHALL map MIDI values 0–127 to 40.0–300.0 BPM, rounded to 0.1 BPM. Existing jump, pickup and scale takeover SHALL apply separately per device and control. A keyboard tempo change or project switch SHALL use the current tempo for takeover. Successful changes SHALL show the existing value popup with BPM and its range.

#### Scenario: Tempo knob maps its full range
- **WHEN** a learned absolute tempo knob sends raw MIDI values 0, 64 and 127 through production dispatch
- **THEN** tempo becomes 40.0, 171.0 and 300.0 BPM in jump mode
- **AND** the value popup identifies tempo in BPM

#### Scenario: Tempo pickup remains separate per controller
- **WHEN** two learned tempo knobs use pickup and the keyboard changes tempo between their movements
- **THEN** each knob must reach the current tempo under the existing per-control takeover rules
- **AND** moving one control does not unlock the other

### Requirement: Transport bindings preserve controller roles

Adding transport SHALL NOT reset, seed again, reassign or rewrite existing X-Session roles or owner bindings. Saved transport bindings SHALL survive restart. Transport commands from a previous project SHALL NOT change the new project's clock. The incomplete SMC-PAD template SHALL remain unverified.

#### Scenario: New transport bindings survive restart without a profile reset
- **WHEN** the owner saves transport bindings and restarts with existing X-Session assignments
- **THEN** the new bindings and all unchanged assignments retain their roles without reseeding

#### Scenario: Late transport commands stay in their original project
- **WHEN** a queued transport or tempo command belongs to the old project and a new project opens
- **THEN** that command does not start, stop or change the new project's clock

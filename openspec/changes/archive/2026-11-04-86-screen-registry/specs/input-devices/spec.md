## ADDED Requirements

### Requirement: Screen bindings keep their existing identity

The system SHALL keep the existing keyboard and MIDI binding identity for Pads, Settings, Files, Chop and FX.
The section names SHALL remain `pads`, `settings`, `files`, `chop` and `fx`.
Global bindings and edit bindings SHALL keep their current scope and priority.
Owner key overrides, explicit `none` entries and learned MIDI assignments SHALL keep their current effects without a reset or format migration.
Default screen keys SHALL remain F2 for Pads, F12 for Settings, F6 for Files, F3 for Chop and F4 for FX.
Screen changes SHALL preserve the existing held-pad release rules, input ordering and audio routes.

#### Scenario: Existing screen bindings survive the registry change
- **WHEN** each current screen receives recorded keyboard events and equivalent raw MIDI messages through existing profiles and learned assignments
- **THEN** each action keeps its existing screen, edit or global scope and priority
- **AND** owner key overrides, explicit `none` entries and saved MIDI assignments need no reset or conversion
- **AND** default screen keys, pad selection, hit velocity and original held-pad releases keep their existing behavior

### Requirement: FX keeps inherited controls without another edit mode

The system SHALL keep FX block, parameter and value controls available without a separate FX edit mode.
Existing page controls SHALL keep their FX roles and priority, including earlier owner assignments.
An explicit FX assignment SHALL keep its current priority over inherited controls.
An existing empty FX assignment group SHALL remain empty instead of restoring inherited bindings.
Global assignment changes and clearing SHALL keep their current effects on FX without stale copies.
This change SHALL NOT add a new edit layer or change the default safe controller buttons.

#### Scenario: FX controls keep their priority through the registry
- **WHEN** FX receives its existing block, parameter and value commands from keys or raw MIDI in separate runs
- **THEN** the same FX block, parameter and value change without a separate edit mode
- **AND** explicit FX assignments, inherited page controls, global changes and clearing keep their existing priority and effects
- **AND** an empty saved FX group does not restore inherited bindings

### Requirement: Invalid MIDI input state cannot select a fallback screen

If shared MIDI input state contains an unknown screen code or another invalid encoding, the system SHALL reject new actions from that state.
It SHALL NOT substitute Pads, another screen or an earlier valid state.
New pad hits, command presses and parameter messages SHALL cause no sound, action or value change while that state remains invalid.
The system SHALL record an error diagnostic outside the audio callback, without repeated logging for every message in one continuous invalid period.

Releases and disconnects for existing holds SHALL still reach their original absolute pads through their original routes.
Device, channel, control and project identities SHALL remain separate.
Untracked releases SHALL NOT release a pad selected by a fallback screen or bank.
Queued release retries SHALL continue under the existing audio queue rules.
This error SHALL NOT stop audio processing, create new pad holds or falsely acknowledge queued UI work.

After a valid state arrives, the system SHALL process new messages with that state and the existing pending-input ordering rules.
It SHALL NOT replay rejected messages or wait forever because it rejected them.

#### Scenario: Unknown MIDI screen rejects new input
- **WHEN** an invalid shared screen code reaches MIDI dispatch before raw pad, command and parameter messages
- **THEN** none of those messages starts sound, changes selection, runs a command or changes a parameter
- **AND** one error diagnostic identifies the invalid state, without a fallback screen or repeated logging for that continuous error

#### Scenario: Other invalid MIDI state has no fallback
- **WHEN** a shared state with an invalid bank or reserved bits reaches MIDI dispatch before raw pad, command and parameter messages
- **THEN** those messages have no sound or state effect and create no hold or new pending UI work
- **AND** the system records the invalid-state diagnostic without substituting a screen or bank

#### Scenario: Invalid MIDI state preserves original releases
- **WHEN** direct and UI-routed gate holds exist before shared state becomes invalid and their original controls release or disconnect
- **THEN** each accepted hold keeps its original absolute release target and route, including when a full audio queue delays release
- **AND** unrelated holds keep sounding, untracked releases cause no pad release and old-project releases do not affect a new project
- **AND** note-off, note-on zero and learned-button release retain their existing meanings

#### Scenario: Valid MIDI state resumes without rejected input
- **WHEN** the UI publishes valid state after dispatch rejected messages during an invalid period and then new raw MIDI input arrives
- **THEN** only the new messages act under the valid state and the existing pending-input ordering rules
- **AND** earlier queued UI work still needs its real acknowledgement, rejected messages never replay and rejection creates no lasting input barrier
- **AND** another invalid period produces its own error diagnostic

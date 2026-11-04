# Spec Delta

## REMOVED Requirements

### Requirement: The program name and its folders
**Reason**: Replaced by "Current program files", which keeps the same names and paths without the obsolete migration scenario.
**Migration**: Existing `ptsr` files need no conversion. Older program folders are not imported.

### Requirement: Copy settings from the old program name
**Reason**: The owner no longer needs automatic import from `arasaka-404`.
**Migration**: Use existing `ptsr` files or current defaults. Old folders remain untouched.

## ADDED Requirements

### Requirement: Current program files

The system SHALL use `ptsr` for its executable, window title, MIDI client and sound client (`ptsr:out_l`, `ptsr:out_r`, `ptsr:in_l`, `ptsr:in_r`).
Folders SHALL be `~/.config/ptsr/` for config, keymap and MIDI bindings, `~/.local/share/ptsr/` for session, and `~/.local/state/ptsr/` for log.
`XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `XDG_STATE_HOME` SHALL relocate them as before.

#### Scenario: Current ptsr files load
- **WHEN** current `ptsr` settings, keys and session exist alongside conflicting `arasaka-404` files
- **THEN** the current files load without importing or changing the old files
- **AND** current sound-device settings, MIDI bindings, owner keys and session pad state remain available

### Requirement: Legacy program folders are not imported

Startup SHALL NOT inspect, copy, rename or remove `arasaka-404` config or session folders, or show a settings-moved notice.
Absent current files SHALL use the existing default settings, shipped keys and empty scratch session.
Current files and XDG paths SHALL keep their existing behavior.

#### Scenario: Legacy folders stay unused without current files
- **WHEN** only legacy `arasaka-404` settings, keys and session files exist
- **THEN** current-file loaders use defaults, shipped keys and an empty session without creating current folders by copying legacy files
- **AND** legacy files remain unchanged

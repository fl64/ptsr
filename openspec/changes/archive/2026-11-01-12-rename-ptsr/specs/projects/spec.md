## ADDED Requirements

### Requirement: The program name and its folders
The system SHALL use the name `ptsr` for the program file, the window title, the sound server client (`ptsr:out_l`, `ptsr:out_r`, `ptsr:in_l`, `ptsr:in_r`), the MIDI client and its folders: `~/.config/ptsr/` (config, keymap, MIDI bindings), `~/.local/share/ptsr/` (session) and `~/.local/state/ptsr/` (log). The `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `XDG_STATE_HOME` variables move these folders as before.

At start, the system SHALL copy the files of the old folder `arasaka-404` into the new folder `ptsr` for the config and the session, if the new folder does not exist and the old folder exists. The system does not delete or change the old folders. The status line says once that the settings moved, and from where.

#### Scenario: The settings move at the first start
- **WHEN** `~/.config/arasaka-404/` holds `config.toml` and `keymap.toml`, `~/.local/share/arasaka-404/` holds `session.toml`, the `ptsr` folders do not exist and the owner starts ptsr
- **THEN** ptsr starts with the same audio device, MIDI bindings, keys and pads as before, the files are in `~/.config/ptsr/` and `~/.local/share/ptsr/`, the old folders stay as they were, and the status line says `settings moved from ~/.config/arasaka-404`

#### Scenario: The new folder wins
- **WHEN** `~/.config/ptsr/` exists and `~/.config/arasaka-404/` exists too
- **THEN** ptsr reads only `~/.config/ptsr/` and copies nothing

#### Scenario: A first start without old folders
- **WHEN** no `arasaka-404` folder and no `ptsr` folder exists
- **THEN** ptsr starts with the defaults, copies nothing and says nothing about a move

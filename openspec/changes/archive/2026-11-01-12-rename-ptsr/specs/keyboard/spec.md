## MODIFIED Requirements

### Requirement: The owner's keymap
The system SHALL build the shipped keymap into the program and read the owner's keys from
`keymap.toml` in the user config folder (`~/.config/ptsr/`) over it, whatever
folder the program starts in. A key bound in a section of the user file replaces the
shipped binding of that key in that section; the action `none` takes a key away. A pad key
bound to a command, alone or with Shift, is refused as in the shipped keymap and named on
the status line. A file that cannot be read is named on the status line with its line, and
the program runs with the shipped keys.

#### Scenario: A key of my own
- **WHEN** the user keymap has `"Ctrl+G" = "pad.scope_next"` under `[screens.pads]`
- **THEN** `Ctrl+G` steps the change scope on the pads screen, and `Ctrl+A` still does too

#### Scenario: A key taken away
- **WHEN** the user keymap has `"Ctrl+A" = "none"` under `[screens.pads]`
- **THEN** `Ctrl+A` does nothing on the pads screen

#### Scenario: A broken file
- **WHEN** the user keymap has a line that cannot be read
- **THEN** the status line names the file and the line, and every shipped key works

#### Scenario: From any folder
- **WHEN** the program starts from the home folder and the user keymap binds `Ctrl+G`
- **THEN** `Ctrl+G` works, and the shipped keys work as when started from the project folder

#### Scenario: Pad keys stay pads
- **WHEN** the user keymap binds `Q` to a command
- **THEN** `Q` still plays its pad, and the status line says a pad key cannot be a command

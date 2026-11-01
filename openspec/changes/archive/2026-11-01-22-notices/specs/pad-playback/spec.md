## MODIFIED Requirements

### Requirement: Pads from the keyboard
The system SHALL play pads from the physical keys `1 2 3 4 / Q W E R / A S D F / Z X C V` in the English (US) and Russian layouts, passing both the press and the release; Shift on press gives full velocity.

#### Scenario: Russian layout
- **WHEN** the Russian layout is on and the owner presses the physical key Q
- **THEN** pad 5 triggers, not a command and not text input

#### Scenario: Pad block keys are not commands
- **WHEN** any screen assigns commands to the keyboard
- **THEN** no command is assigned to the pad block keys alone or with Shift; combinations with Ctrl or Alt (`Ctrl+1`…`8` banks, `Ctrl+Z` undo) are commands

#### Scenario: Terminal without the kitty protocol
- **WHEN** the terminal does not support the extended keyboard protocol
- **THEN** pads play on press, gate mode sounds like one-shot; a warning is shown until the first keypress of any kind, after that the warning is gone, the hint line is back, and the status keeps a "no releases" marker for the session

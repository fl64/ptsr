# Spec Delta

## ADDED Requirements

### Requirement: A1 correction has its own key in the overview

F5 SHALL keep opening the A1 overview. F7 SHALL toggle correction only while that overview is open. F2 SHALL return to Pads and close correction. Unclaimed Esc SHALL close correction before it closes the overview. Other screens SHALL keep their existing keys.

#### Scenario: Editor entry keys keep the overview route
- **WHEN** the owner presses F5, F7, Esc and F2 with absent, empty or playing A1
- **THEN** F5 opens the overview, F7 enters or leaves correction when permitted, and Esc closes the nearest mode
- **AND** F2 returns to Pads without creating A1 or stopping playback

### Requirement: A1 correction keys use contextual actions

In correction, J/L and Left/Right SHALL select time cells. I/K and Up/Down SHALL select an event. Alt+Up and Alt+Down SHALL keep their shared view action. U/O SHALL keep bank control. Ctrl+Z and Alt+Z SHALL retain their permitted Undo roles. Outside correction, existing pad parameter keys SHALL keep their roles.

#### Scenario: Correction keys change the target without changing a pad
- **WHEN** the owner uses step, event, view and bank keys in correction, then leaves it and uses the same keys
- **THEN** correction selects its target without changing pad parameters or raw hits
- **AND** ordinary Pads and the read-only overview keep their existing actions

#### Scenario: Correction keys respect owner bindings and input owners
- **WHEN** the owner moves or disables a key, opens F1 or text, or holds an edit-layer key
- **THEN** effective bindings and input priority apply before correction actions
- **AND** pad-block keys remain pads and Shift on a pad never becomes a command

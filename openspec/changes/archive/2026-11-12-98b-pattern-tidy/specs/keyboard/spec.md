# keyboard Delta

## MODIFIED Requirements

### Requirement: Pattern control keys

Alt+N SHALL toggle compact pattern controls. I/K and Up/Down SHALL select Bars, Quantize, Strength or Clear A1. J/L and Left/Right SHALL change the selected value. On Clear A1, they SHALL change nothing. Enter on Clear A1 SHALL run Clear A1. Enter on a value row, Esc or Alt+N SHALL close controls. Alt+E SHALL toggle erasure inside a pass and on Pads outside a pass. Esc SHALL turn off erasure before its other roles. Otherwise, unclaimed Esc SHALL finish Recording or cancel CountIn. Alt+Z SHALL request pattern-pass Undo globally. Ctrl+Z on Pads SHALL undo through common history. Plain and Shift pad keys SHALL remain pads.

#### Scenario: Adjust a pattern without opening a step grid
- **WHEN** Alt+N opens controls on any current screen and direction keys change each parameter
- **THEN** those keys edit the shown pattern parameter without changing underlying screen parameters
- **AND** closing controls restores the original screen, bank, selection and edit page

#### Scenario: Pattern keys work in both layouts and protocols
- **WHEN** recording, controls, erasure and pattern Undo keys arrive in English or Russian with extended or legacy key input
- **THEN** their logical actions agree and plain or Shift pad keys never execute those commands
- **AND** repeats or releases do not toggle recording, controls or erasure again

#### Scenario: Enter runs Clear A1 from pattern controls
- **WHEN** the owner selects Clear A1 in pattern controls, presses L, then presses Enter
- **THEN** L changes nothing, and Enter shows the Clear A1 question without clearing
- **AND** Enter on Bars closes controls

#### Scenario: Esc leaves erasure first
- **WHEN** erasure is on during Recording and the owner presses Esc twice
- **THEN** the first Esc turns off erasure and recording continues
- **AND** the second Esc finishes Recording

## ADDED Requirements

### Requirement: Pattern controls own the edit controls

While pattern controls are open, keyboard commands that change a pad, a slice or an effect SHALL change nothing. These include the pad mode, Pitch Gate, pad clear, rename, marks and edit pages. Pad keys SHALL still play pads. Screen, bank, transport, recording, erasure and Undo keys SHALL keep their roles.

#### Scenario: Pad commands wait while pattern controls are open
- **WHEN** pattern controls are open on Pads and the owner presses H, Alt+G, Backspace and Tab, then a pad key
- **THEN** the pad mode, Pitch Gate, sound and panel page stay unchanged
- **AND** the pad key plays its pad

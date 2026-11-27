## MODIFIED Requirements

### Requirement: The SMC-PAD knob strip shows both banks and the current list

At 80x26 and 120x40, the strip SHALL show bank 1 as 7–8, 5–6, 3–4 and 1–2, and bank 2 master. When Rhythm needs a third row, pairs 3–4 and 1–2 SHALL share a row. It SHALL name the page and dim unavailable roles. Help SHALL show the roles of LEFT, RIGHT, PAD BANK, PLAY and STOP. The mark SHALL show the last knob and its bank until disconnect. The value pop-up SHALL name the knob, value, target and amount. F1 Controller SHALL show the same roles.

#### Scenario: The strip identifies page, list and roles at both sizes
- **WHEN** the owner views each supported size with the SMC-PAD connected
- **THEN** bank 1 fits in physical order and the bank 2 master line shows
- **AND** the page, list, button roles and unavailable roles are visible

#### Scenario: Last knob input marks its bank
- **WHEN** valid knob movement arrives from bank 1 and then bank 2
- **THEN** the mark changes to the bank that sent the last turn
- **AND** a neutral value or an absent action changes no control value
- **AND** a disconnect clears the mark and stale input cannot set it again

#### Scenario: The strip shows the turned knob and its value
- **WHEN** the owner turns bank 1 knob 7 in pad edit on TONE at either supported size
- **THEN** the strip marks knob 7 of bank 1
- **AND** the help row shows knob 7, volume, the pad and the new value until the pop-up ends

## ADDED Requirements

### Requirement: Rhythm stays visible with MIDI and the pad grid

At 80x26, the Pads screen SHALL show Rhythm and the full pad grid with SMC-PAD connected. A1 absent SHALL show the record route. An empty or populated A1 SHALL show the header and both event rows. The MIDI footer SHALL preserve the knob roles, the current view and its controls. The notice SHALL remain visible. Pad Edit, recording, and Full Help SHALL take priority over Rhythm.

#### Scenario: Compact Rhythm keeps its rows and MIDI roles
- **WHEN** the owner opens Pads Rhythm at 80x26 with the grid visible and an SMC-PAD connected
- **THEN** Rhythm, all sixteen pads, the notice and both MIDI banks are visible
- **AND** Pad Edit takes the strip until it closes

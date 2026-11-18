## MODIFIED Requirements

### Requirement: Panel state and unavailable cells stay visible

The title SHALL show `PANEL · SMC` for the active SMC-PAD connection.
A header row SHALL name the shown view and the views on its left and right.
Action cells, screen cells and the bottom row SHALL use three different colors.
A new view or screen SHALL update the labels before the next cell acts.
Disabled Redo, Panic and empty cells SHALL appear dim with no active-action claim.
A first reveal SHALL not show an action result. A disconnect or first-bank hit SHALL remove the panel mark.

#### Scenario: Panel feedback follows the active connection and page
- **WHEN** the owner reveals the panel, changes its view and then disconnects the SMC-PAD
- **THEN** the badge, header and labels follow that connection and view, then disappear on disconnect
- **AND** Redo and Panic remain visibly unavailable and no first-reveal command result appears

#### Scenario: The header names the view and its neighbors
- **WHEN** the panel shows the Chop actions at either supported size
- **THEN** the header shows the screens view on the left and the Chop actions in the middle
- **AND** the action cells and the screen cells use different colors

### Requirement: The SMC-PAD knob strip shows both banks and the current list

At 80 by 26 and 120 by 40, the strip SHALL show bank 1 as rows 7–8, 5–6, 3–4 and 1–2, and one bank 2 master line.
It SHALL name the page and dim unavailable roles.
The help row SHALL show the roles of LEFT, RIGHT, PAD BANK, PLAY and STOP.
The mark SHALL show the last knob and its bank until disconnect.
During the value pop-up, the help row SHALL name the knob, value, target and amount.
F1 Controller SHALL show the same roles.

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

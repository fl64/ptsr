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
- **WHEN** the panel shows action page 1 of Chop at either supported size
- **THEN** the header shows the screens view on the left, Chop actions 1 of 2 and action page 2 on the right
- **AND** the action cells and the screen cells use different colors

### Requirement: The SMC-PAD knob strip shows both banks and the current list

The strip SHALL show bank 1 as rows 7–8, 5–6, 3–4 and 1–2 from top to bottom at 80 by 26 and 120 by 40.
One line SHALL show the bank 2 master knob.
The strip SHALL name the page and dim unavailable roles.
The help row SHALL show the current role of LEFT and RIGHT, PAD BANK, PLAY and STOP.
The mark SHALL show the last valid knob and its bank. Disconnect SHALL clear it.
While the value pop-up shows, the help row SHALL name that knob, value, target and amount.
The F1 Controller tab SHALL show the same effective roles.

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

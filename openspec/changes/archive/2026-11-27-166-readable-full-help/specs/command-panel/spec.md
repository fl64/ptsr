## MODIFIED Requirements

### Requirement: Every mode shows its way out on a hold

A PAD BANK hold SHALL show the panel in all modes, including Help, project and folder lists, learning and text entry. In Full Help, the border SHALL show its controls. The panel SHALL show only cells that act. Device pad 1 SHALL close or cancel and keep the screen. In Help, cells 9 and 10 SHALL move. In a list, pad 4 SHALL open the selection. In learning, pad 4 SHALL stay a control to learn and Enter SHALL save the choice. Other cells SHALL be empty.

#### Scenario: A hold in Help shows the close cell
- **WHEN** Help shows and the owner holds PAD BANK
- **THEN** the help border shows device pad 1 for close and the navigation roles of device pads 5 and 6
- **AND** device pad 1 closes Help

#### Scenario: No mode is a dead end for the SMC-PAD
- **WHEN** the owner enters each current mode in separate runs and holds PAD BANK
- **THEN** the visible panel controls name device pad 1
- **AND** device pad 1 during the hold leaves that mode or closes its question

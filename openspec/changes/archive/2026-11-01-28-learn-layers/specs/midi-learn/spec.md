## MODIFIED Requirements

### Requirement: Every action can be learned
The system SHALL list on the MIDI tab every action a key can do, grouped as GLOBAL (works on every screen), one group per screen and one group per edit layer (PADS · EDIT, CHOP · EDIT: the page knobs 1–3, the previous and the next parameter, leaving the edit), except quitting the program; the list SHALL scroll so the selected row is always visible. Inside an edit mode the learned edit group of the screen goes first, then the device profile's edit layer, then the screen and the global groups.

#### Scenario: Global action
- **WHEN** the owner learns a button for "help" in GLOBAL
- **THEN** the button opens the help on the pads screen and on the settings screen

#### Scenario: Screen group wins
- **WHEN** the same button is learned for "help" in GLOBAL and for "pad mode" in PADS
- **THEN** on the pads screen it switches the pad mode, on the settings screen it opens the help

#### Scenario: Same knob, two layers
- **WHEN** a knob is learned for "pad volume" in PADS and for "knob 1 of the page" in PADS · EDIT, and the owner turns it with the pad edit closed and then open on the ENV page
- **THEN** it turns the volume first and the attack after

#### Scenario: Outside the edit
- **WHEN** a button is learned only for "next parameter" in PADS · EDIT and the owner presses it with no edit mode open
- **THEN** nothing happens

### Requirement: Jump between action groups
The system SHALL move the selection of the MIDI learn list to the previous or next group row with the bank actions (PgUp/PgDn).

#### Scenario: To the chop group
- **WHEN** the MIDI tab is open and the owner presses PgDn until the CHOP group
- **THEN** the CHOP group row is selected

## ADDED Requirements

### Requirement: Groups fold
The system SHALL show every group of the MIDI learn list as a row with its name, the number of its actions and how many are learned, and its actions under it only while it is open. The group of the screen the settings were opened from is open, the others are closed; `→` (`L`) or `Enter` on a group row opens it, `←` (`J`) closes the group of the selected row.

#### Scenario: Closed groups
- **WHEN** the owner opens F12 from the pads screen and goes to the MIDI tab
- **THEN** PADS is open, every other group is one row like "CHOP · 31 actions · 2 learned"

#### Scenario: Open a group
- **WHEN** the CHOP row is selected and the owner presses →
- **THEN** the actions of CHOP show under it; ← on one of them closes CHOP and selects its row

### Requirement: Pads and banks in a row
The system SHALL show pads 1–16 and banks 1–8 as one row each with how many of them are learned, and SHALL learn them one after another from that row: every control pressed goes to the next one, a control already taken in the same run is ignored, Esc stops and keeps what is learned, Backspace clears them all.

#### Scenario: Sixteen pads in a row
- **WHEN** the owner selects "pads 1–16", presses Enter and hits 16 controller pads in turn
- **THEN** controller pad k plays pad k, and the row says "16 of 16 learned"

#### Scenario: A double hit
- **WHEN** during the run the owner hits the same controller pad twice
- **THEN** the second hit is ignored and the run waits for another control

#### Scenario: Stop in the middle
- **WHEN** the owner presses Esc after 5 pads
- **THEN** pads 1–5 stay learned and the row says "5 of 16 learned"

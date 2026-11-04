## ADDED Requirements

### Requirement: Screen help keeps its current controls and learning groups

The system SHALL keep the existing bottom help and full help for Pads, Settings, Files, Chop and FX.
Each screen SHALL retain its current action groups, effective keys, toggle states and modifier help.
The registry SHALL NOT change group order, the three-row help band or the current width rules.

Screen learn SHALL remain available on Pads, Files, Chop and FX, with their current screen groups.
Pads edit SHALL retain PADS · EDIT. Chop edit SHALL retain CHOP · EDIT and its single bounds page.
FX SHALL retain its FX group without another edit group.
Settings SHALL retain its existing tab-specific help and MIDI learning list. F10 SHALL NOT open screen learn on Settings.
The MIDI list SHALL keep its existing group names and order.
Opening Settings from Pads, Files, Chop or FX SHALL keep the corresponding screen group open in that list.

#### Scenario: Each registered screen keeps its effective help
- **WHEN** the owner opens each current screen with keys or equivalent MIDI actions and then opens full help
- **THEN** bottom help and full help keep that screen's existing action groups, effective keys and modifier rules
- **AND** owner overrides, the three-row band, group order and narrow-terminal behavior remain unchanged
- **AND** Settings help still follows its current tab and FX still names its block and parameter controls

#### Scenario: Screen learning keeps its existing entry points
- **WHEN** the owner opens screen learn through keyboard or raw MIDI actions on Pads, Files, Chop and FX in separate runs
- **THEN** each screen uses its existing screen group, Pads edit uses PADS · EDIT and Chop edit uses CHOP · EDIT
- **AND** FX has no separate edit group, Chop retains one bounds page and cancellation saves no pending binding
- **AND** the same learn action on Settings does not open screen learn or add a new learning path

#### Scenario: MIDI list keeps screen groups and their order
- **WHEN** the owner opens Settings from Pads, Files, Chop or FX through keyboard or raw MIDI actions in separate runs
- **THEN** the MIDI tab opens the corresponding screen group and keeps all existing group titles and ordering
- **AND** global, screen and edit assignments keep their existing learning scopes without renaming saved groups

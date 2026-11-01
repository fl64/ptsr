## ADDED Requirements

### Requirement: The pads switch in the help
The system SHALL show in the help line of the pads, files and chop screens the pads switch with its state: `O pads: shown` or `O pads: hidden` among the Ctrl keys, which show while Ctrl is held; without the kitty protocol, where a held Ctrl is not seen, the full help (F1) lists the switch: the usual line of the chop screen has no room left for it within the three help rows.

#### Scenario: The help names the state
- **WHEN** the pads are hidden and the owner holds Ctrl on F6
- **THEN** the help line says `pads: hidden` next to `O`

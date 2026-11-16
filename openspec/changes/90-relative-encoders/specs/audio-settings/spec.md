# audio-settings Delta

## ADDED Requirements

### Requirement: A relative knob selects settings rows

On F12, a knob bound to `settings.row` SHALL move the selected row by signed steps and clamp at the visible list ends. The MIDI tab SHALL use its current expanded or folded row list. The turn SHALL not change a setting value or start MIDI learn. Outside F12, this target SHALL do nothing.

#### Scenario: Relative knob moves through MIDI rows
- **WHEN** F12 MIDI shows an expanded group and the owner turns a relative row knob in both directions
- **THEN** the highlight moves through the visible rows and stops at the list ends
- **AND** bindings and the knob response remain unchanged until a separate value or learn action

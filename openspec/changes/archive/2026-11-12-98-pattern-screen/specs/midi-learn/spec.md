# midi-learn Delta

## ADDED Requirements

### Requirement: Learn pattern view commands

F12 MIDI SHALL list `pattern.view_prev` and `pattern.view_next` as GLOBAL button actions. A learned button SHALL change the view through the same App action as the keyboard keys. It SHALL use existing confirmation, clearing, persistence and layer precedence. No factory device binding SHALL be introduced.

#### Scenario: Learned pattern view buttons change only the view
- **WHEN** a learned CC button for `pattern.view_next` arrives through raw MIDI profile routing on Pads
- **THEN** the view becomes finer, as with Alt+Up
- **AND** raw rows, history and dirty state stay unchanged, and a following MIDI pad note still plays its pad

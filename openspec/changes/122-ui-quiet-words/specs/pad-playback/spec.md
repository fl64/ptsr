# pad-playback Delta

## MODIFIED Requirements

### Requirement: Empty pads look empty

The system SHALL show a muted `—` in the grid cell of a pad without a sample. The cell SHALL NOT show the word `empty` or a flat wave. A meaningful mode or target corner mark SHALL remain visible. The selected pad's info SHALL still name its test tone. The pad edit SHALL still show the test tone piece.

#### Scenario: New pad
- **WHEN** a pad has no sample in an idle, marked, or target state
- **THEN** its grid cell shows a muted `—` without the word `empty` or a flat wave
- **AND** the relevant corner mark remains, and the selected info says `empty · test tone A2`

### Requirement: Pad waves

The system SHALL draw the wave of a loaded pad in its Pads grid cell with its envelope applied. It SHALL use up to three rows when the terminal has room. An empty pad SHALL have no wave. When no wave row fits, the grid SHALL omit pad waves. The Pads screen SHALL draw no separate large view of the selected pad.

#### Scenario: Every pad draws its wave
- **WHEN** Pads shows loaded and empty pads in a terminal 40 rows high
- **THEN** loaded pads show their waves in three rows under the mode line, and empty pads show no wave

#### Scenario: The wave follows the terminal
- **WHEN** the terminal gets lower step by step
- **THEN** loaded pad waves take three, then two, then one row, and the whole grid always fits

#### Scenario: The attack shows on the pad
- **WHEN** the owner turns the attack of a loaded pad up
- **THEN** the wave of that pad rises more slowly at its start

#### Scenario: Short terminal
- **WHEN** the terminal is 26 rows high
- **THEN** the pads are drawn without waves, and the whole grid fits

#### Scenario: No big view
- **WHEN** the owner selects a pad
- **THEN** nothing is drawn under the info panel

### Requirement: Pad source

The system SHALL give every pad with a sample one source. The pad panel SHALL name a whole-file source as `whole file`, a linked slice as `slice N`, and a pad's own bounded piece as `pad`. The internal source IDs SHALL stay `all`, `slice N`, and `pad`. The sound and cut-following rules SHALL stay unchanged.

#### Scenario: Slices laid out
- **WHEN** slices 1–8 of a break lie on pads 5–12 and the owner selects pad 7
- **THEN** the pad panel says `break.wav · slice 3`

#### Scenario: The whole file
- **WHEN** a pad plays a whole file and the owner selects it
- **THEN** the pad panel says `break.wav · whole file`
- **AND** the pad plays the same source as internal `all`

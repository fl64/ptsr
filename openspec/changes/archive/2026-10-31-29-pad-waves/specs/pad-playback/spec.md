## ADDED Requirements

### Requirement: Pad waves
The system SHALL draw in every pad cell of the pads screen the wave of what the pad plays with its envelope applied, in as many rows as the terminal has room for, up to three; an empty pad draws a flat line. When not even one row fits, the pads are drawn without waves. The pads screen draws no separate big view of the selected pad.

#### Scenario: Every pad draws its wave
- **WHEN** the pads screen is open in a terminal 40 rows high and pads hold samples
- **THEN** every pad with a sample shows its wave in three rows under its mode line, and an empty pad shows a flat line

#### Scenario: The wave follows the terminal
- **WHEN** the terminal gets lower step by step
- **THEN** the waves take three, then two, then one row, and the whole grid always fits

#### Scenario: The attack shows on the pad
- **WHEN** the owner turns the attack of a pad up
- **THEN** the wave of that pad rises more slowly at its start

#### Scenario: Short terminal
- **WHEN** the terminal is 26 rows high
- **THEN** the pads are drawn without waves, as before, and the whole grid fits

#### Scenario: No big view
- **WHEN** a pad is selected
- **THEN** nothing is drawn under the info panel

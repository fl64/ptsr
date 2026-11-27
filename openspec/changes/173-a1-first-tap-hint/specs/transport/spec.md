## MODIFIED Requirements

### Requirement: Pattern accepts taps for tempo

Pattern SHALL offer Tap Tempo on its action page, P8. Alt+B SHALL use
the same action. Tempo SHALL use the mean interval of up to four taps.
A gap longer than two seconds SHALL start a new sequence.
The first tap SHALL keep the tempo and show «TAP · 1 hit · tap again». The program SHALL ignore intervals under 200 ms.
The program SHALL use MIDI receipt times. Tap Tempo SHALL act only in Pattern
or its controls.

#### Scenario: Pattern taps use the beat interval
- **WHEN** two Pattern tap presses arrive 400 ms apart
- **THEN** the tempo becomes 150 BPM after the second press
- **AND** the four most recent taps determine the average interval
- **AND** a gap longer than two seconds or a press outside Pattern changes no tempo
- **AND** Alt+B opens the same tap sequence

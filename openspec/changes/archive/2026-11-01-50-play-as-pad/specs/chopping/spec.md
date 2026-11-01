## ADDED Requirements

### Requirement: Play as the pad
The system SHALL switch "as pad" on and off with `Alt+P` on the chop screen; it starts
off. While it is on, what the chop screen plays (`Space`, a cursor move into a slice, a
slice pick) sounds with the volume, pitch, envelope and filter of the pad the slice under
the cursor lies on, the first such pad in pad order; a slice on no pad plays bare. A hit
on that pad does not cut the preview. The help line shows `as pad` next to the play mode
while the switch is on.

#### Scenario: Slice 3 as its pad
- **WHEN** slice 3 lies on pad W with pitch −12 and a low-pass at 500 Hz, "as pad" is on and the owner plays slice 3
- **THEN** the render is an octave lower and has less than a tenth of the energy above 2 kHz that slice 3 has bare

#### Scenario: Off plays bare
- **WHEN** "as pad" is off and the owner plays slice 3
- **THEN** the render is the bare slice, sample for sample

#### Scenario: A slice on no pad
- **WHEN** "as pad" is on and the slice under the cursor lies on no pad
- **THEN** it plays bare

#### Scenario: The switch in the help line
- **WHEN** the owner presses `Alt+P` on the chop screen
- **THEN** the help line says `as pad` next to the play mode; `Alt+P` again takes it away

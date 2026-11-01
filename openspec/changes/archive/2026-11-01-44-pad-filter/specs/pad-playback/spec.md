## ADDED Requirements

### Requirement: Pad filter
The system SHALL give every pad a filter: the type (off, low-pass, high-pass, band), the
cutoff from 20 Hz to 20 kHz and the resonance from 0 to 100 %. A new pad and a pad of an
older project have the filter off; with the filter off the pad sounds exactly as without
it. A change on a sounding pad is heard at once and glides, no click. The filter follows
the change scope (the marked pads), as the volume and the pitch do, and is saved with the
pad in the project. The cutoff steps by a semitone on a tap and by an octave on a held
key; the resonance by 1 % and 10 %; a knob turns the cutoff along the octaves.

#### Scenario: Behind a wall
- **WHEN** a pad with a hi-hat has the low-pass at 1 kHz and is hit
- **THEN** the render keeps less than a tenth of the energy above 4 kHz the same hit has with the filter off; no clipping, no NaN

#### Scenario: The rumble goes
- **WHEN** a pad with a kick has the high-pass at 60 Hz and is hit
- **THEN** the render keeps less than a tenth of the energy below 30 Hz the same hit has with the filter off

#### Scenario: Off is untouched
- **WHEN** a pad has the filter off and is hit
- **THEN** the render is the same, sample for sample, as before the filter existed

#### Scenario: Turn while it sounds
- **WHEN** a long pad in gate sounds with the low-pass at 200 Hz and the cutoff is turned up to 8 kHz in one step
- **THEN** the sound opens up over a few milliseconds, no step between two neighbouring samples is larger than the sound's own

#### Scenario: Saved and opened
- **WHEN** pad W has the band filter at 2 kHz with 40 % resonance, the project is saved and opened again
- **THEN** pad W has the band filter at 2 kHz with 40 % resonance and sounds as before

#### Scenario: An older project
- **WHEN** a project saved before the filter existed is opened
- **THEN** every pad has the filter off

#### Scenario: Marked pads together
- **WHEN** pads Q and W are marked and the owner turns the cutoff on Q down an octave
- **THEN** the cutoff of W moves down an octave too

### Requirement: Filter curve on the strip
The system SHALL, while the FILTER page of the pad edit is picked, draw in the strip above
the grid the filter's response in place of the wave and the envelope: the frequency from
20 Hz on the left to 20 kHz on the right along the octaves, with half an octave of room at
each end, the level that passes upwards, the top rising with the resonance hump; the whole
curve, the hump too, stays inside the strip at any cutoff and resonance;
a caption names the type, the cutoff and the resonance, and a few frequencies stand under
the curve. The cutoff and the resonance hump are where they sound; the curve follows every
change at once. With the filter off the curve is a flat line at full level and the caption
says `filter off`. On the other pages the strip shows the wave with the envelope, as
before.

#### Scenario: The cutoff moves on the curve
- **WHEN** the FILTER page of a pad with the low-pass at 1.2 kHz is open and the owner turns the cutoff down an octave
- **THEN** the drop of the curve moves left to 600 Hz and the caption says `low-pass · 600 Hz`

#### Scenario: The resonance is a hump
- **WHEN** the resonance of that pad goes from 0 to 60 %
- **THEN** a hump rises on the curve at the cutoff

#### Scenario: The hump stays on the screen
- **WHEN** the low-pass is at 20 kHz with 100 % resonance, or at 20 Hz
- **THEN** the top of the hump is inside the strip, not cut at its edge

#### Scenario: Off is flat
- **WHEN** the FILTER page of a pad with the filter off is open
- **THEN** the strip shows a flat line at full level and `filter off`

#### Scenario: Back to the wave
- **WHEN** the owner presses `Tab` from FILTER to TONE
- **THEN** the strip shows the pad's wave with its envelope again

## MODIFIED Requirements

### Requirement: Pad pitch
The system SHALL play each pad at its own pitch from −24 to +24 semitones in steps of 1, faster and shorter when higher, like a record; a change on a sounding pad glides without a click. On the pads screen `I` `K` pick the parameter, and `Tab` turns a whole page (see "Pad edit mode"). `J` `L` change the picked parameter in the edit mode.

#### Scenario: An octave up
- **WHEN** pitch is picked and the owner raises the selected pad by 12 semitones
- **THEN** the pad plays an octave higher and half as long

#### Scenario: Glide while it sounds
- **WHEN** a long pad sounds and the owner changes its pitch
- **THEN** the sound moves to the new pitch smoothly, without a click

#### Scenario: Pick the parameter
- **WHEN** the volume is picked and the owner presses `K` on the pads screen
- **THEN** the highlighted parameter in the pad panel moves from volume to pitch, and `J` `L` in the edit mode now change the pitch

### Requirement: Pad envelope
The system SHALL give each pad an envelope of three times in milliseconds: attack (fade-in at the start of the piece), decay (the last `decay` ms of the piece fade to silence, the piece still ends at the same place) and release (in gate mode the sound fades over it after the release). The envelope is the ENV page of the pad edit: `Tab` turns to it, and `I` `K` pick attack, decay or release. `J` `L` change the picked one in the edit mode (a tap 1 ms, a held key 5 ms per repeat) for the change scope; the changed value pops up. Every envelope change on a sounding pad is click-free.

#### Scenario: Attack at the start
- **WHEN** the owner raises the attack of a pad to 50 ms and hits it
- **THEN** the sound comes in softly over 50 ms, without a click

#### Scenario: Decay ends at the end of the piece
- **WHEN** the owner sets a decay of 200 ms on a one-shot pad
- **THEN** the last 200 ms of the piece fade to silence and the pad ends at the same place as before, in silence

#### Scenario: Release after the gate
- **WHEN** a gate pad with a release of 300 ms is released
- **THEN** the sound fades out over 300 ms, without a click

#### Scenario: The envelope follows the scope
- **WHEN** the scope is bank 2 and the owner raises the release
- **THEN** every pad of bank 2 gets the longer release and the panel names the scope

#### Scenario: Old session
- **WHEN** the session file was saved before the envelope existed
- **THEN** it opens and every pad sounds as it did: no attack, no decay, the same release as today

### Requirement: Pad edit mode
The system SHALL open the edit mode of the selected pad with `Enter` on the pads screen.
`Enter` or `Esc` closes it. The pages and their parameters are as the editing spec says
(requirement "Parameter pages"). `Tab` turns to the next page and `Shift+Tab` to the
previous page. `I` `K` (`↑` `↓`) pick the previous or the next parameter, and the page
follows the picked parameter. `Tab` and `I` `K` pick with the mode open or closed. `J` `L`
(`←` `→`) change the picked parameter in the mode. With the mode closed, `J` `L` turn the
banks (see "Banks on J and L").

#### Scenario: The page follows the picked parameter
- **WHEN** the owner enters the pad edit mode and presses `K` until decay is picked
- **THEN** the page is ENV and the screen shows that the knobs turn attack, decay and release

#### Scenario: The knobs turn the page
- **WHEN** the pad edit mode is open on the ENV page and the owner turns knob 2
- **THEN** the decay of the selected pad (or of the change scope) follows the knob with soft takeover

### Requirement: Source page
The system SHALL put a SOURCE page into the pad edit after TONE and ENV: the source, the
start and the end, on knobs 1–3 and the value keys. The source steps all, slice 1…N, pad.
Moving the start or the end of a pad that plays `all` or a slice makes it a `pad` pad with
its current bounds first; the status says the pad no longer follows the cuts, and undo
links it back. The pad panel shows the rows of all pages at once, so no row hides when
the page changes; the page decides only what the knobs turn and carries their numbers. A
window that is too short for every row folds the rows (see "The pad panel fits the
window").

#### Scenario: Edit only this fragment
- **WHEN** pad 7 plays slice 3 and the owner opens its edit, walks to SOURCE and moves the start right
- **THEN** pad 7 starts later and says `pad`, pads 6 and 8 do not change, and the status says pad E (pad 7) no longer follows the cuts and `Ctrl+Z` links it back

#### Scenario: Undo links it back
- **WHEN** the owner presses Ctrl+Z after that
- **THEN** pad 7 says `slice 3` again and plays slice 3

#### Scenario: Pick another slice
- **WHEN** a pad plays the whole break and the owner steps its source to `slice 2`
- **THEN** the pad plays slice 2 and follows its cuts

#### Scenario: Every row in sight
- **WHEN** the window is tall enough for every row, the edit of a pad is open on the ENV
  page and the owner presses `Tab` until the page is SOURCE
- **THEN** the panel shows the rows of every page the whole time, from volume to end, and
  the knob numbers 1–3 move from attack, decay, release to source, start, end

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
- **WHEN** the FILTER page is open and the owner presses `Tab` until the page is TONE
- **THEN** the strip shows the pad's wave with its envelope again

### Requirement: The pad panel in blocks
The system SHALL group the parameter rows of the pad panel by page, in the page order
TONE, ENV, FILTER, SOURCE, FX: every block stands under a thin line with the page name,
the line of the page the edit is on is lit. While the edit is on a shared slot's page,
that slot's block (`FX 1 * shared`, `FX 2 * shared`) stands in place of the FX block. `Tab` turns the pages in this order, `I` `K` walk
the parameters in this order, and the frame title names the pages in this order.

#### Scenario: Blocks
- **WHEN** the pad edit is open
- **THEN** the panel shows the lines TONE, ENV, FILTER, SOURCE, FX in this order, each over its rows

#### Scenario: SOURCE last
- **WHEN** the edit is on the reso of the FILTER page and the owner presses `K`
- **THEN** the source is picked, on the SOURCE page; `K` after the end of SOURCE picks the
  drive on FX, and `K` after the last parameter of FX2* goes back to the volume

## ADDED Requirements

### Requirement: Pad edit mode
The system SHALL open the edit mode of the selected pad with `Enter` on the pads screen:
the pages TONE (volume, pitch) and ENV (attack, decay, release), `Tab` walking all five
parameters (`I` `K` too inside the mode), `J` `L` changing the picked one, the page
following the picked one, `Enter` or `Esc` closing. `Tab` picks the parameter with the
mode open or closed.

#### Scenario: The page follows the picked parameter
- **WHEN** the owner enters the pad edit mode and presses `Tab` until decay is picked
- **THEN** the page is ENV and the screen shows that the knobs turn attack, decay and release

#### Scenario: The knobs turn the page
- **WHEN** the pad edit mode is open on the ENV page and the owner turns knob 2
- **THEN** the decay of the selected pad (or of the change scope) follows the knob with soft takeover

### Requirement: Pad envelope over the wave
The system SHALL, while the pad edit mode is open, draw in the strip above the grid the
wave of the selected pad's piece, dimmed, and over it the pad's envelope as a bright
line: the attack rises from zero at the start, the decay falls to zero at the end of the
piece. The release acts only in gate mode, from the moment the key lets go: for a gate
pad it is drawn apart from the wave, at the right edge after a gap, as a dotted fall from
full to silence named "key up", its length to the scale of the piece; for a one-shot pad
it is not drawn, and the panel shows the release dimmed with "(gate)": the value is kept
and acts as soon as the pad is gate. The part of the picked parameter is drawn in its own
colour; the line follows every change, the mode too, at once. This replaces the live output while the mode is open; `Esc`
brings the live output back; the level meters stay.

#### Scenario: The attack is a slope
- **WHEN** the pad edit mode is open and the owner raises the attack of the pad
- **THEN** the line in the strip rises from zero over a longer part of the wave, and the
  attack part is drawn in the colour of the picked parameter

#### Scenario: The release follows the mode
- **WHEN** the edit mode of a one-shot pad with a release of 250 ms is open and the owner
  presses `H`
- **THEN** before: no release is drawn and the panel says "release 250 ms (gate)" dimmed;
  after: a dotted fall named "key up" stands at the right edge apart from the wave

#### Scenario: The live output comes back
- **WHEN** the pad edit mode is open and the owner presses `Esc`
- **THEN** the strip shows the live output again

### Requirement: Banks on J and L
The system SHALL turn the banks with `J` `L` (and `←` `→`) on the pads screen outside the
edit mode; the arrows and the letters do not pick a pad, a pad hit does.

#### Scenario: L turns to the next bank
- **WHEN** bank 1 is open on the pads screen and the owner presses `L`
- **THEN** bank 2 is open and the selected pad is the same place in it

## MODIFIED Requirements

### Requirement: Pad modes
The system SHALL support the modes one-shot (plays to the end) and gate (plays while the pad is held), volume by velocity, and a pad volume and pitch that can be changed with a controller knob or with keys, for the change scope. `H` switches the mode of the selected pad — and of the scope — in place of `Enter`, which now opens the edit mode.

#### Scenario: Gate
- **WHEN** a pad in gate mode is pressed and released after 300 ms
- **THEN** the sound fades out right after the release, without a click

#### Scenario: Pad volume from the keyboard
- **WHEN** the edit mode of a pad is open, volume is the picked parameter and the owner presses `J` or `L`
- **THEN** the pad volume changes by 1 dB, a sounding voice changes its volume smoothly, without a click

#### Scenario: Mode from the keyboard
- **WHEN** a pad is selected and the owner presses `H`
- **THEN** the pad switches to its next mode, one-shot or gate

### Requirement: Pad pitch
The system SHALL play each pad at its own pitch from −24 to +24 semitones in steps of 1, faster and shorter when higher, like a record; a change on a sounding pad glides without a click. `Tab` on the pads screen picks the parameter that `J` `L` change in the edit mode: volume, pitch, attack, decay or release.

#### Scenario: An octave up
- **WHEN** pitch is picked and the owner raises the selected pad by 12 semitones
- **THEN** the pad plays an octave higher and half as long

#### Scenario: Glide while it sounds
- **WHEN** a long pad sounds and the owner changes its pitch
- **THEN** the sound moves to the new pitch smoothly, without a click

#### Scenario: Pick the parameter
- **WHEN** the owner presses `Tab` on the pads screen
- **THEN** the highlighted parameter in the pad panel moves from volume to pitch, and `J` `L` in the edit mode now change the pitch

### Requirement: Change scope
The system SHALL apply volume, pitch and mode changes to the change scope, stepped with `Ctrl+A`: the selected pad (or the marked pads when there are marks), the current bank, all pads. The panel always names the scope. Volume and pitch change relatively: every pad in the scope moves by the same step and the differences between them stay; a pad at a limit stays there and the pop-up says how many pads hit it. The mode is set to the next mode of the selected pad on every pad of the scope, with `H`. Clearing a pad acts on the selected pad only.

#### Scenario: A bank lower
- **WHEN** the scope is bank 2 and the owner lowers the pitch twice
- **THEN** every pad of bank 2 is two semitones lower and a melody laid out from slices keeps its shape

#### Scenario: All pads quieter
- **WHEN** the scope is all pads and the owner lowers the volume by 3 dB
- **THEN** all 128 pads are 3 dB quieter, pads on other banks too

#### Scenario: A limit
- **WHEN** the scope holds a pad at +24 and others at 0 and the owner raises the pitch
- **THEN** the others go up by one, the pad at +24 stays, and the pop-up says that 1 pad is at the limit

#### Scenario: The scope is always named
- **WHEN** the owner presses `Ctrl+A`
- **THEN** the panel shows the next scope on its "change" row: "change  pad Q", "change  bank 2" or "change  all pads"

#### Scenario: Mode for a bank
- **WHEN** the scope is bank 1 and the owner presses `H`
- **THEN** every pad of bank 1 gets the next mode of the selected pad

#### Scenario: Clear stays on one pad
- **WHEN** the scope is a bank and the owner presses `Backspace`
- **THEN** only the selected pad is cleared

### Requirement: Pad envelope
The system SHALL give each pad an envelope of three times in milliseconds: attack (fade-in at the start of the piece), decay (the last `decay` ms of the piece fade to silence, the piece still ends at the same place) and release (in gate mode the sound fades over it after the release). On the pads screen `Tab` steps through the parameters volume → pitch → attack → decay → release and `J` `L` change the picked one in the edit mode (a tap 1 ms, a held key 5 ms per repeat) for the change scope; the changed value pops up. Every envelope change on a sounding pad is click-free.

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

### Requirement: Marking pads
The system SHALL turn marking on and off with `M`, the marker key; while it is on, a pad hit marks or unmarks the pad and the pad still sounds. Marked pads are framed in the grid and become the scope instead of the selected pad; `Esc` drops all marks.

#### Scenario: Three kicks quieter
- **WHEN** the owner presses `M`, hits three pads, presses `M` and lowers the volume by 4 dB
- **THEN** the panel says "change: 3 marked" and only these three pads are 4 dB quieter

#### Scenario: Unmark
- **WHEN** marking is on and the owner hits a marked pad again
- **THEN** that pad is no longer marked

#### Scenario: Drop the marks
- **WHEN** pads are marked and the owner presses `Esc`
- **THEN** no pad is marked and the scope is the selected pad again

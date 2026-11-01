## MODIFIED Requirements

### Requirement: Pad modes
The system SHALL support the modes one-shot (plays to the end) and gate (plays while the pad is held), volume by velocity, and a pad volume and pitch that can be changed with a controller knob or with keys, for the change scope.

#### Scenario: Gate
- **WHEN** a pad in gate mode is pressed and released after 300 ms
- **THEN** the sound fades out right after the release, without a click

#### Scenario: Pad volume from the keyboard
- **WHEN** a pad is selected on the pads screen, volume is the picked parameter and the owner presses `−` or `=`
- **THEN** the pad volume changes by 1 dB, a sounding voice changes its volume smoothly, without a click

### Requirement: Pads screen
The system SHALL show the pads screen on F2: a 4×4 grid with keys and sound names, the current bank, the selected pad with its mode, volume and pitch with the picked parameter highlighted, the change scope, marked pads framed, and the pad lit while it is held (at least a short flash for a quick hit). Hitting a pad selects it.

#### Scenario: Hit selects the pad
- **WHEN** the owner hits pad W
- **THEN** pad 6 flashes, becomes selected, its mode, volume and pitch are shown on the right

#### Scenario: Held pad stays lit
- **WHEN** the owner holds pad Q for 2 seconds, from the keyboard or a controller
- **THEN** pad Q stays lit for the 2 seconds and goes dark right after the release

## ADDED Requirements

### Requirement: Pad pitch
The system SHALL play each pad at its own pitch from −24 to +24 semitones in steps of 1, faster and shorter when higher, like a record; a change on a sounding pad glides without a click. `Tab` on the pads screen picks the parameter that `−` `=` change: volume or pitch.

#### Scenario: An octave up
- **WHEN** pitch is picked and the owner raises the selected pad by 12 semitones
- **THEN** the pad plays an octave higher and half as long

#### Scenario: Glide while it sounds
- **WHEN** a long pad sounds and the owner changes its pitch
- **THEN** the sound moves to the new pitch smoothly, without a click

#### Scenario: Pick the parameter
- **WHEN** the owner presses `Tab` on the pads screen
- **THEN** the highlighted parameter in the pad panel moves from volume to pitch, and `−` `=` now change the pitch

### Requirement: Change scope
The system SHALL apply volume, pitch and mode changes to the change scope, stepped with `Ctrl+A`: the selected pad (or the marked pads when there are marks), the current bank, all pads. The panel always names the scope. Volume and pitch change relatively: every pad in the scope moves by the same step and the differences between them stay; a pad at a limit stays there and the pop-up says how many pads hit it. The mode is set to the next mode of the selected pad on every pad of the scope. Clearing a pad acts on the selected pad only.

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
- **THEN** the panel shows the next scope: "change: pad Q", "change: bank 2" or "change: all pads"

#### Scenario: Mode for a bank
- **WHEN** the scope is bank 1 and the owner presses `Enter`
- **THEN** every pad of bank 1 gets the next mode of the selected pad

#### Scenario: Clear stays on one pad
- **WHEN** the scope is a bank and the owner presses `Backspace`
- **THEN** only the selected pad is cleared

### Requirement: Marking pads
The system SHALL turn marking on and off with `M`; while it is on, a pad hit marks or unmarks the pad and the pad still sounds. Marked pads are framed in the grid and become the scope instead of the selected pad; `Esc` drops all marks.

#### Scenario: Three kicks quieter
- **WHEN** the owner presses `M`, hits three pads, presses `M` and lowers the volume by 4 dB
- **THEN** the panel says "change: 3 marked" and only these three pads are 4 dB quieter

#### Scenario: Unmark
- **WHEN** marking is on and the owner hits a marked pad again
- **THEN** that pad is no longer marked

#### Scenario: Drop the marks
- **WHEN** pads are marked and the owner presses `Esc`
- **THEN** no pad is marked and the scope is the selected pad again

### Requirement: Pad knobs for the scope
The system SHALL let the learned `pad.volume` and `pad.pitch` knobs set the selected pad with soft takeover and move the other pads of the scope by the same amount.

#### Scenario: Pitch knob on a bank
- **WHEN** the scope is bank 2 and the owner turns the learned pitch knob up by 3 semitones
- **THEN** the selected pad and every other pad of bank 2 are 3 semitones higher

### Requirement: Pad pitch in the session
The system SHALL keep the pitch of every pad in the session; a session saved before pitch existed SHALL open with every pad at pitch 0.

#### Scenario: Restart
- **WHEN** pads have different pitches and the program is restarted
- **THEN** every pad plays at the same pitch as before

#### Scenario: Old session
- **WHEN** the session file was saved before pitch existed
- **THEN** it opens, every pad plays at pitch 0

# pad-playback Specification

## Purpose
Pads: 16 pads in 8 banks played from the keyboard, one-shot and gate modes, velocity, pad volume and pitch, click-free voice cuts, and the F2 pads screen.

## Requirements

### Requirement: Shared FX screen

A separate FX screen SHALL show both shared blocks with their types and parameters in one edit frame titled `EDIT · FX 1 FX 2`.
The selected block SHALL be lit in the title and on its block line.
The same three learned knobs SHALL control its type and two parameters.
Block selection SHALL NOT change effects.
Takeover SHALL be separate per device, block and control.
Changes SHALL be smooth, project-saved and undoable with `Ctrl+Z`.
Existing projects and learned bindings SHALL keep their behavior.

#### Scenario: Select the next shared effect
- **WHEN** FX 1 is selected and the owner presses Tab
- **THEN** FX 2 is selected and neither effect changes

#### Scenario: The same knobs control the selected effect
- **WHEN** the owner changes FX 1 with the three page knobs, selects FX 2 and turns them
- **THEN** only FX 2 changes after the selection

#### Scenario: Takeover is separate for shared effects
- **WHEN** a controller uses pickup, the owner changes FX 1, selects FX 2 and moves a knob
- **THEN** FX 2 waits for its own value before the knob changes it

#### Scenario: Undo a shared effect on the FX screen
- **WHEN** the owner changes a shared effect and presses Ctrl+Z
- **THEN** the previous effect setting returns

#### Scenario: Compact shared effect cards
- **WHEN** the owner opens F4 at 80x26 and 120x40, then presses Tab
- **THEN** one frame titled `EDIT · FX 1 FX 2` holds both blocks with their block lines and three rows each
- **AND** FX 1 is lit first and FX 2 after Tab, and the frame does not fill unused rows

#### Scenario: Clear a shared effect knob assignment
- **WHEN** the owner clears a page knob assignment on F4 and restarts the program
- **THEN** the cleared knob does not control the effect and no old assignment returns

#### Scenario: Replaced page knobs stay replaced
- **WHEN** an old pad edit page knob has a replacement and the owner opens F4
- **THEN** the replacement controls the effect and the previous knob stays inactive

### Requirement: Pad grid and banks
The system SHALL provide 16 pads in a 4×4 grid and 8 banks numbered 1–8; the bank is switched with a command from the keyboard or a controller: next/previous, or directly with Ctrl+1…8. On screen a bank is always shown with the word "bank" so it is not confused with the pad keys 1–4.

#### Scenario: Bank change
- **WHEN** the owner runs "next bank" on bank 1
- **THEN** bank 2 becomes active, the screen shows the pads of bank 2, sounding pads of bank 1 play to the end

#### Scenario: Direct bank choice
- **WHEN** the owner presses Ctrl+3
- **THEN** bank 3 becomes active; the plain 3 key still plays pad 3

### Requirement: Pads from the keyboard
The system SHALL play pads from the physical keys `1 2 3 4 / Q W E R / A S D F / Z X C V` in the English (US) and Russian layouts, passing both the press and the release; Shift on press gives full velocity.

#### Scenario: Russian layout
- **WHEN** the Russian layout is on and the owner presses the physical key Q
- **THEN** pad 5 triggers, not a command and not text input

#### Scenario: Pad block keys are not commands
- **WHEN** any screen assigns commands to the keyboard
- **THEN** no command is assigned to the pad block keys alone or with Shift; combinations with Ctrl or Alt (`Ctrl+1`…`8` banks, `Ctrl+Z` undo) are commands

#### Scenario: Terminal without the kitty protocol
- **WHEN** the terminal does not support the extended keyboard protocol
- **THEN** pads play on press, gate mode sounds like one-shot; a warning is shown until the first keypress of any kind, after that the warning is gone, the hint line is back, and the status keeps a "no releases" marker for the session

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

### Requirement: Pad response
The system SHALL start the pad sound in the nearest audio block after the event is received.

#### Scenario: Fast roll
- **WHEN** the owner hits one pad 16 times per second
- **THEN** every hit sounds, the previous voice of the same pad is cut without a click

### Requirement: Pads screen
The system SHALL show the pads screen on F2: a 4×4 grid with keys and sound names, the current bank, the selected pad with its mode, volume and pitch with the picked parameter highlighted, the change scope, marked pads with a dark yellow fill and a bright yellow frame, and the pad lit while it is held (at least a short flash for a quick hit). Hitting a pad selects it.

#### Scenario: Hit selects the pad
- **WHEN** the owner hits pad W
- **THEN** pad 6 flashes, becomes selected, its mode, volume and pitch are shown on the right

#### Scenario: Held pad stays lit
- **WHEN** the owner holds pad Q for 2 seconds, from the keyboard or a controller
- **THEN** pad Q stays lit for the 2 seconds and goes dark right after the release

#### Scenario: A marked pad is yellow
- **WHEN** pad E of bank 3 is marked
- **THEN** pad E has a dark yellow fill and a bright yellow frame, its name stays readable,
  and the other pads of bank 3 keep the bank color

### Requirement: Empty pads look empty

The system SHALL show a muted `—` in the grid cell of a pad without a sample. The cell SHALL NOT show the word `empty` or a flat wave. A meaningful mode or target corner mark SHALL remain visible. The selected pad's info SHALL still name its test tone. The pad edit SHALL still show the test tone piece.

#### Scenario: New pad
- **WHEN** a pad has no sample in an idle, marked, or target state
- **THEN** its grid cell shows a muted `—` without the word `empty` or a flat wave
- **AND** the relevant corner mark remains, and the selected info says `empty · test tone A2`

### Requirement: Bank colors
The system SHALL give each of the eight banks its own soft color, show every bank in the bank row in its color and draw the frames of the open bank's pads in that color; the colors of state (selected cyan, in hand magenta, marked yellow, hit white) stay above the bank color.

#### Scenario: Bank colors
- **WHEN** the owner switches from bank 1 to bank 2
- **THEN** the idle pad frames change from the color of bank 1 to the color of bank 2, and the bank row shows each of the eight banks in its own color

#### Scenario: Meanings stay
- **WHEN** a pad of bank 3 is selected and another one is hit
- **THEN** the selected pad's frame is cyan and the hit pad flashes white, not the bank color

### Requirement: A learned control hits the pad
The system SHALL make a control learned for "pad N" hit pad N of the current bank on every screen, with the velocity of the note or the value of the CC press; the release of the control (note off, CC 0) ends a gate pad.

#### Scenario: Hit on any screen
- **WHEN** a button is learned for "pad 1" and the owner presses it on the pads screen and on the files screen
- **THEN** pad 1 of the current bank plays both times

#### Scenario: Velocity
- **WHEN** the control is a note and the owner presses it with a low velocity
- **THEN** the pad plays quieter than a full-velocity press

#### Scenario: Gate by release
- **WHEN** pad 1 is in gate mode, plays from the learned control and the owner releases it (note off, CC 0)
- **THEN** the sound fades out right after the release, without a click

#### Scenario: Banks change under the button
- **WHEN** the owner switches from bank 1 to bank 3 and presses the control learned for "pad 2"
- **THEN** pad 2 of bank 3 plays

### Requirement: Live output strip
The system SHALL show on the pads screen, in the strip above the grid, a live envelope of the master output at the height of the shared strip: the sound of the last few seconds scrolls right-to-left, what plays right now is at the right edge. The strip is scaled in decibels like the meters, so a peak's height on the strip matches its bar. A silent output is a thin middle line. The strip moves only while sound plays.

#### Scenario: I see what I play
- **WHEN** the owner hits a loud pad and then a quiet one
- **THEN** a tall wave runs from the right edge and a smaller one follows it, both moving left

#### Scenario: The strip agrees with the meters
- **WHEN** a peak of −6 dB plays and the owner compares the wave with the L/R bars
- **THEN** the wave rises to nearly the full height, like the bars, not to half of it

#### Scenario: Silence is a line
- **WHEN** nothing plays
- **THEN** the strip is a thin middle line and does not move

#### Scenario: Quiet and loud do not merge
- **WHEN** a quiet pad (−40 dB) plays after a loud one
- **THEN** the quiet wave is clearly lower than half the strip's height, not the same fat block

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

### Requirement: Change scope
Pad changes SHALL affect marked pads, otherwise the selected pad.
The panel's second row under pad/bank SHALL always name `scope   pad Q` or `scope   17 marked`, as help does (68).
Volume and pitch SHALL move all scope pads by the same relative step, preserving differences except at limits.
Limited pads SHALL stay at their limit, and the popup SHALL count them.
`H` SHALL set all scope pads to the selected pad's next mode.

#### Scenario: A bank lower
- **WHEN** all 16 pads of bank 2 are marked and the owner lowers the pitch twice
- **THEN** every pad of bank 2 is two semitones lower and a melody laid out from slices keeps its shape

#### Scenario: All pads quieter
- **WHEN** the owner presses `Ctrl+A` on each of the eight banks and lowers the volume by 3 dB
- **THEN** all 128 pads are 3 dB quieter, pads on other banks too

#### Scenario: A limit
- **WHEN** the scope holds a pad at +24 and others at 0 and the owner raises the pitch
- **THEN** the others go up by one, the pad at +24 stays, and the pop-up says that 1 pad is at the limit

#### Scenario: The scope is always named
- **WHEN** no pad is marked and the owner presses `Ctrl+A` on bank 2
- **THEN** the panel's second row changes from "scope   pad Q" to "scope   16 marked"; no
  row under the blocks names it

#### Scenario: Mode for a bank
- **WHEN** all 16 pads of bank 1 are marked and the owner presses `H`
- **THEN** every pad of bank 1 gets the next mode of the selected pad

### Requirement: Clearing the pad change scope

Outside edit, `Backspace` SHALL clear only the selected pad without marks, or all marked pads with sound after confirmation.
The question box SHALL name `Clear 3 pads?`, say files on disk stay, and offer `Enter: clear · Esc: keep`.
A multi-pad clear SHALL make one undo step.

#### Scenario: Clear stays on one pad
- **WHEN** no pad is marked, all pads of the bank hold sounds, the edit is closed and the
  owner presses `Backspace`
- **THEN** only the selected pad is cleared, and nothing asks

#### Scenario: Clear the marked pads
- **WHEN** pads Q, W and E hold sounds and are marked, the edit is closed, the owner
  presses `Backspace` and then `Enter`
- **THEN** the question says `Clear 3 pads?` before anything changes, and after `Enter`
  pads Q, W and E are empty; one `Ctrl+Z` brings back the three sounds

#### Scenario: Esc keeps the marked pads
- **WHEN** pads are marked, the owner presses `Backspace` and then `Esc`
- **THEN** every pad keeps its sound and the marks stay

### Requirement: Marking pads
`M` SHALL toggle marking.
While on, pad hits SHALL toggle the pad's mark and still sound.
`Ctrl+A` SHALL mark all 16 open-bank pads and enable marking, or unmark them if all are already marked.
Marks SHALL survive bank changes and replace the selected pad as the change scope.

#### Scenario: Three kicks quieter
- **WHEN** the owner presses `M`, hits three pads, presses `M` and lowers the volume by 4 dB
- **THEN** the panel says "scope   3 marked" and only these three pads are 4 dB quieter

#### Scenario: Unmark
- **WHEN** marking is on and the owner hits a marked pad again
- **THEN** that pad is no longer marked

### Requirement: Mark state and cancellation

Esc SHALL drop all marks and disable marking. In pad edit, the first Esc SHALL close edit and keep marks.
The shared yellow badge SHALL say `MARK · hit pads · N marked` while marking is on, otherwise `N MARKED · Esc drops` while marks remain.
Its compact form SHALL retain mode and count.
Moving the plate into the shared badge model SHALL NOT change marking, sound, scope or cancellation.

#### Scenario: Drop the marks
- **WHEN** pads are marked, edit is closed and the owner presses Esc
- **THEN** no pad is marked and the selected pad becomes the change scope again

#### Scenario: The bank, then a few more
- **WHEN** the owner marks bank 2, unmarks Q and W, then marks 1, 2 and 3 on bank 5
- **THEN** 17 pads remain marked across those banks and the badge reports that count

#### Scenario: Ctrl+A again unmarks the bank
- **WHEN** all pads of bank 2 and pad 1 of bank 5 are marked and the owner presses Ctrl+A on bank 2
- **THEN** only bank 5 pad 1 stays marked and marking remains on

#### Scenario: Esc in the edit keeps the marks
- **WHEN** three pads are marked, edit is open and the owner presses Esc
- **THEN** edit closes with all three marks retained and a second Esc drops them

#### Scenario: The mark plate
- **WHEN** the owner presses M, marks three pads and presses M again
- **THEN** the shared yellow badge changes from `MARK · hit pads · 3 marked` to `3 MARKED · Esc drops`
- **AND** Esc removes it without changing the shared screen rows

### Requirement: Pad knobs for the scope
The system SHALL let the learned `pad.volume` and `pad.pitch` knobs set the selected pad with soft takeover and move the other pads of the scope by the same amount.

#### Scenario: Pitch knob on a bank
- **WHEN** all 16 pads of bank 2 are marked and the owner turns the learned pitch knob up by 3 semitones
- **THEN** the selected pad and every other pad of bank 2 are 3 semitones higher

### Requirement: Pad pitch in the session
The system SHALL keep the pitch of every pad in the session; a session saved before pitch existed SHALL open with every pad at pitch 0.

#### Scenario: Restart
- **WHEN** pads have different pitches and the program is restarted
- **THEN** every pad plays at the same pitch as before

#### Scenario: Old session
- **WHEN** the session file was saved before pitch existed
- **THEN** it opens, every pad plays at pitch 0

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

### Requirement: Pad envelope
Each pad SHALL have three envelope times in milliseconds:
- Attack: fade in at the piece's start.
- Decay: fade the last `decay` ms to silence without moving the piece's end.
- Release: fade after gate release for that duration.
Changes on sounding pads SHALL be click-free.

#### Scenario: Attack at the start
- **WHEN** the owner raises the attack of a pad to 50 ms and hits it
- **THEN** the sound comes in softly over 50 ms, without a click

#### Scenario: Decay ends at the end of the piece
- **WHEN** the owner sets a decay of 200 ms on a one-shot pad
- **THEN** the last 200 ms of the piece fade to silence and the pad ends at the same place as before, in silence

#### Scenario: Release after the gate
- **WHEN** a gate pad with a release of 300 ms is released
- **THEN** the sound fades out over 300 ms, without a click

### Requirement: Pad envelope controls

The envelope SHALL use the ENV page, reached with `Tab`.
`I` `K` SHALL select attack, decay or release.
In edit, `J` `L` SHALL change it for the scope by 1 ms per tap or 5 ms per held repeat, with a value popup.

#### Scenario: The envelope follows the scope
- **WHEN** all 16 pads of bank 2 are marked and the owner raises the release
- **THEN** every pad of bank 2 gets the longer release and the panel names the scope

#### Scenario: Old session
- **WHEN** the session file was saved before the envelope existed
- **THEN** it opens and every pad sounds as it did: no attack, no decay, the same release as today

### Requirement: Pad edit mode

Enter on Pads SHALL edit the selected pad.
Enter/Esc SHALL close edit.
Pages SHALL follow editing, "Parameter pages".
Tab/Shift+Tab SHALL change pages forward/back.
I/K and Up/Down SHALL select previous/next parameters, with the page following.
Tab, I/K, J/L and their arrow forms SHALL work with edit open or closed.
J/L and Left/Right SHALL change values.
U/O SHALL change banks in both states, per "Keyboard bank controls".

#### Scenario: The page follows the picked parameter
- **WHEN** the owner enters pad edit and presses K until decay is picked
- **THEN** the page is ENV and the screen shows what the knobs turn

#### Scenario: The knobs turn the page
- **WHEN** pad edit is open on ENV and the owner turns knob 2
- **THEN** decay of the selected pad or change scope follows the knob with soft takeover

#### Scenario: Value keys work with pad edit closed
- **WHEN** Pads is open with edit closed, volume is selected and the owner presses L
- **THEN** volume rises by 1 dB for the change scope, selection and bank stay, and edit stays closed

#### Scenario: Value arrows work with pad edit closed
- **WHEN** Pads is open with edit closed and the owner presses Left or Right
- **THEN** the selected value changes as with J or L, without turning banks

### Requirement: Pad envelope over the wave
During pad edit, the strip SHALL show the selected piece's dim wave with a bright envelope line.
Attack SHALL rise from zero at the start and decay fall to zero at the piece's end.
The selected parameter's part SHALL use its own colour.
The line SHALL follow parameter and mode changes immediately.

#### Scenario: The attack is a slope
- **WHEN** the pad edit mode is open and the owner raises the attack of the pad
- **THEN** the line in the strip rises from zero over a longer part of the wave, and the
  attack part is drawn in the colour of the picked parameter

### Requirement: Envelope release display follows pad mode

Release SHALL act only after gate key release.
For gate pads, it SHALL appear separately after a gap at the right edge: a dotted full-to-silent fall named "key up", scaled to the piece.
One-shot pads SHALL hide that line and dim release with "(gate)" in the panel.
The value SHALL remain stored and act as soon as the pad becomes gate.

#### Scenario: The release follows the mode
- **WHEN** the edit mode of a one-shot pad with a release of 250 ms is open and the owner
  presses `H`
- **THEN** before: no release is drawn and the panel says "release 250 ms (gate)" dimmed;
  after: a dotted fall named "key up" stands at the right edge apart from the wave

### Requirement: Pad edit strip replaces live output

Pad edit SHALL replace live output in the strip while open.
`Esc` SHALL restore live output.
Level meters SHALL remain visible.

#### Scenario: The live output comes back
- **WHEN** the pad edit mode is open and the owner presses `Esc`
- **THEN** the strip shows the live output again

### Requirement: Pad source

The system SHALL give every pad with a sample one source. The pad panel SHALL name a whole-file source as `whole file`, a linked slice as `slice N`, and a pad's own bounded piece as `pad`. The internal source IDs SHALL stay `all`, `slice N`, and `pad`. The sound and cut-following rules SHALL stay unchanged.

#### Scenario: Slices laid out
- **WHEN** slices 1–8 of a break lie on pads 5–12 and the owner selects pad 7
- **THEN** the pad panel says `break.wav · slice 3`

#### Scenario: The whole file
- **WHEN** a pad plays a whole file and the owner selects it
- **THEN** the pad panel says `break.wav · whole file`
- **AND** the pad plays the same source as internal `all`

### Requirement: Source page
Pad edit SHALL have SOURCE after TONE and ENV: source, start and end on knobs 1–3 and value keys.
Source SHALL step through all, slice 1…N, pad.
Moving start/end of an `all` or slice source SHALL first make it a `pad` source with its current bounds.
Status SHALL say it no longer follows cuts, and undo SHALL restore the link.

#### Scenario: Edit only this fragment
- **WHEN** pad 7 plays slice 3 and the owner opens its edit, walks to SOURCE and moves the start right
- **THEN** pad 7 starts later and says `pad`, pads 6 and 8 do not change, and the status says pad E (pad 7) no longer follows the cuts and `Ctrl+Z` links it back

#### Scenario: Undo links it back
- **WHEN** the owner presses Ctrl+Z after that
- **THEN** pad 7 says `slice 3` again and plays slice 3

#### Scenario: Pick another slice
- **WHEN** a pad plays the whole break and the owner steps its source to `slice 2`
- **THEN** the pad plays slice 2 and follows its cuts

### Requirement: Pad page rows stay visible

The pad panel SHALL show all page rows at once, without hiding rows on page changes.
Pages SHALL select only knob targets and show knob numbers.
Short windows SHALL fold rows per "The pad panel fits the window".

#### Scenario: Every row in sight
- **WHEN** the window is tall enough for every row, the edit of a pad is open on the ENV
  page and the owner presses `Tab` until the page is SOURCE
- **THEN** the panel shows the rows of every page the whole time, from volume to end, and
  the knob numbers 1–3 move from attack, decay, release to source, start, end

### Requirement: The bank comes back
The system SHALL keep the open bank in the session and open it again on the next start and when the project opens again.

#### Scenario: Restart on bank 2
- **WHEN** the owner works on bank 2 and restarts the program
- **THEN** the pads screen opens on bank 2

### Requirement: Pad filter
Each pad SHALL have off/low-pass/high-pass/band filter types, 20 Hz–20 kHz cutoff and 0–100 % resonance.
New pads and older-project pads SHALL default to off, sounding exactly as without a filter.
Sounding-pad changes SHALL act immediately and glide without clicks.
Filters SHALL follow the change scope like volume/pitch and be project-saved with the pad.

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

### Requirement: Pad filter control steps

Cutoff SHALL step by a semitone per tap and an octave per held repeat.
Resonance SHALL step by 1 % and 10 % respectively.
Knob cutoff SHALL follow octaves.

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
FILTER edit SHALL replace wave/envelope with filter response.
Frequency SHALL run 20 Hz left to 20 kHz right along octaves, with half an octave extra at each end.
Passed level SHALL rise upwards, with the top rising for the resonance hump.
The whole curve SHALL fit at any cutoff/resonance and place them where they sound.
A caption SHALL name type, cutoff and reso, with frequencies below.
Changes SHALL show immediately.

#### Scenario: The cutoff moves on the curve
- **WHEN** the FILTER page of a pad with the low-pass at 1.2 kHz is open and the owner turns the cutoff down an octave
- **THEN** the drop of the curve moves left to 600 Hz and the caption says `low-pass · 600 Hz`

#### Scenario: The resonance is a hump
- **WHEN** the resonance of that pad goes from 0 to 60 %
- **THEN** a hump rises on the curve at the cutoff

#### Scenario: The hump stays on the screen
- **WHEN** the low-pass is at 20 kHz with 100 % resonance, or at 20 Hz
- **THEN** the top of the hump is inside the strip, not cut at its edge

### Requirement: Filter strip off and page states

With filter off, the curve SHALL be flat at full level with caption `filter off`.
Other pages SHALL show the wave with its envelope as before.

#### Scenario: Off is flat
- **WHEN** the FILTER page of a pad with the filter off is open
- **THEN** the strip shows a flat line at full level and `filter off`

#### Scenario: Back to the wave
- **WHEN** the FILTER page is open and the owner presses `Tab` until the page is TONE
- **THEN** the strip shows the pad's wave with its envelope again

### Requirement: The pad panel in blocks
The system SHALL group the parameter rows of the pad panel by page, in the page order
TONE, ENV, FILTER, SOURCE, FX: every block stands under a thin line with the page name,
the line of the page the edit is on is lit. `Tab` turns the pages in this order, `I` `K` walk
the parameters in this order, and the frame title names the pages in this order.

#### Scenario: Blocks
- **WHEN** the pad edit is open
- **THEN** the panel shows the lines TONE, ENV, FILTER, SOURCE, FX in this order, each over its rows

#### Scenario: SOURCE last
- **WHEN** the edit is on the reso of the FILTER page and the owner presses `K`
- **THEN** the source is picked, on the SOURCE page; `K` after the end of SOURCE picks the
  drive on FX, and `K` after the last parameter of FX goes back to the volume

### Requirement: Pad drive
The system SHALL give every pad a drive from 0 to 100 %: turning it up adds grit and keeps
the loudness about the same. A new pad and a pad of an older project have the drive at 0;
at 0 the pad sounds exactly as without it. A change on a sounding pad glides, no click.
The drive follows the change scope and is saved with the pad in the project.

#### Scenario: Off is untouched
- **WHEN** a pad has the drive and both sends at 0 and is hit
- **THEN** the render is the same, sample for sample, as before the effects existed

#### Scenario: Grit, not volume
- **WHEN** the test tone pad has the drive at 80 % and is hit
- **THEN** the render has more energy above the third harmonic than at 0 %, its peak stays
  within 3 dB of the peak at 0 %; no clipping, no NaN

### Requirement: Shared effect slots
Project pads SHALL share FX 1 and FX 2.
Each pad SHALL send 0–100 % of its post-drive sound to each slot, with slot output added to its dry sound.
New and older-project pads SHALL default to zero sends.
Slots SHALL offer reverb (size, tone) or delay (10–2000 ms, feedback 0–90 %).
New projects SHALL use reverb in FX 1 and delay in FX 2.
Slots SHALL be project-saved.

#### Scenario: The echo
- **WHEN** a short pad sends 100 % to a delay with the time at 250 ms and the feedback at
  50 % and is hit once
- **THEN** the render has an echo 250 ms after the hit, within 1 sample, and the next echo
  250 ms later about 6 dB quieter; no clipping, no NaN

### Requirement: Shared effect tails and parameter changes

Slots SHALL continue after their input pads stop until their tails end.
Parameter changes SHALL glide without clicks.
Type changes SHALL fade the old tail out and start the new effect empty, retaining each type's parameters for later return.

#### Scenario: The tail outlives the pad
- **WHEN** a one-shot pad of 100 ms sends 60 % to a reverb and is hit
- **THEN** the render has sound after the pad's end, and it dies down to silence within a
  few seconds

#### Scenario: Turn the time while it echoes
- **WHEN** the delay echoes and its time is turned from 250 to 400 ms in one step
- **THEN** no step between two neighbouring samples is larger than the sound's own

#### Scenario: Two pads, one room
- **WHEN** pads Q and W send to FX 1 with a reverb and the owner increases its size on the FX screen
- **THEN** the tail of a hit on W is longer too

#### Scenario: A new type in the slot
- **WHEN** FX 2 sounds with a delay tail and its type is turned to reverb, then back to delay
- **THEN** the delay tail fades out without a click, and the delay has the time and the
  feedback it had before

#### Scenario: Two reverbs
- **WHEN** FX 1 and FX 2 both hold a reverb, pad Q sends to FX 1 with size 20 % and pad W
  to FX 2 with size 90 %
- **THEN** the tail of W is longer than the tail of Q

#### Scenario: Saved and opened
- **WHEN** pad W has the drive at 30 % and send 1 at 40 %, FX 2 holds a reverb with size
  70 %, the project is saved and opened again
- **THEN** all of it is as before and sounds as before

#### Scenario: An older project
- **WHEN** a project saved before the effects existed is opened
- **THEN** every pad has the drive and both sends at 0, FX 1 holds a reverb and FX 2 a delay

### Requirement: Shared effects callback budget

With 32 voices and both slots, the audio callback SHALL stay within 50 % of the buffer budget (128 samples at 48 kHz).

#### Scenario: Marked pads together
- **WHEN** pads Q and W are marked and the owner turns send 1 on Q to 30 %
- **THEN** send 1 of W is 30 % too, and changes on the shared FX screen change no pad

### Requirement: Undo on the pads

Ctrl+Z on Pads and Files SHALL undo one change per press through 100 steps:
- Volume, pitch, envelope, filter, drive and sends.
- Mode, swap, copy, clear, reset, file assignment, source and shared FX.
A knob turn or value-key run on one parameter SHALL make one step for all touched pads.
Restored pads SHALL sound as before.

#### Scenario: Undo a knob turn
- **WHEN** the owner turns volume of Q from 0 to -12 dB in one turn and presses Ctrl+Z
- **THEN** Q is at 0 dB and sounds so after one press

#### Scenario: Undo over the scope
- **WHEN** all 16 pads are marked, the owner raises their cutoff and presses Ctrl+Z
- **THEN** all 16 cutoffs return and the status says `undo: cutoff on 16 pads`

#### Scenario: Undo a swap
- **WHEN** the owner swaps Q and V with Ctrl+X and presses Ctrl+Z
- **THEN** Q and V play their previous sounds with their settings

#### Scenario: Undo a clear
- **WHEN** the owner clears Q with Backspace and presses Ctrl+Z
- **THEN** Q has its file back and sounds as before

#### Scenario: Undo a shared slot
- **WHEN** the owner changes FX 2 from delay to reverb and presses Ctrl+Z
- **THEN** FX 2 holds the delay again with its time and feedback

### Requirement: Pad undo history and feedback

Undo status SHALL name what returned or say `nothing to undo`.
Selection, banks, scope, pages and marks SHALL add no step.
History SHALL last until exit.

#### Scenario: Nothing to undo
- **WHEN** the program has just started and the owner presses Ctrl+Z on Pads
- **THEN** nothing changes and the status says `nothing to undo`

#### Scenario: What changes no pad
- **WHEN** the owner raises Q's volume, presses Ctrl+A twice, turns the bank, selects W
  and presses Ctrl+Z
- **THEN** Q's volume returns; bank, selection and marks stay as they are

#### Scenario: Undo links it back
- **WHEN** pad 7 plays slice 3, the owner moves its start on SOURCE and presses Ctrl+Z
- **THEN** pad 7 says `slice 3` and plays it again

### Requirement: Pad copy undo restores only its target

A copy placement SHALL restore the target's full previous state in one undo step without changing its source.
Cancelled copies SHALL add no step.

#### Scenario: Undo one pad copy
- **WHEN** the owner copies Q onto occupied W after confirmation and presses Ctrl+Z
- **THEN** W's previous sound, source link, name and all settings return in one step
- **AND** Q stays unchanged throughout

### Requirement: Mute groups
The system SHALL give each pad a mute group, off or 1–8, off by default. A hit of a pad
stops every other sounding pad of the same group with a short click-free fade (the one a
pad uses to cut its own last hit), whatever their mode and release; pads in other groups
and pads with the group off ring on. Groups are shared by all banks. Sounds of the chop
screen are in no group.

#### Scenario: Closed hat cuts the open one
- **WHEN** the open hat on pad Q and the closed hat on pad W are both in group 1, the
  owner hits Q and 200 ms later W
- **THEN** the open hat fades out within 5 ms of the W hit without a click, and the closed
  hat plays in full

#### Scenario: Other groups ring on
- **WHEN** pad Q is in group 1, pad E in group 2 and pad R has the group off, and the owner
  hits Q, E, R and then W of group 1
- **THEN** only Q stops; E and R ring on

#### Scenario: Groups across banks
- **WHEN** pad 1-1 and pad 2-1 are in group 3 and the owner hits 1-1, turns to bank 2 and
  hits 2-1
- **THEN** pad 1-1 stops

### Requirement: The mute group is set like any pad parameter
The system SHALL set the mute group as the third parameter of the TONE page of the pad
edit, by the value keys (off, 1, …, 8, then it stops at 8) and by knob 3 of the page, for
the change scope: every pad of the scope takes the group the selected pad steps to. The
panel shows it on its own row of the TONE block, under the pitch: `mute    off` or
`mute    1`. `Ctrl+Z` undoes a change of it as any pad change.

#### Scenario: A group for the marked pads
- **WHEN** the owner presses `M`, hits pads Q, W and E, presses `M`, opens the edit,
  steps to "mute group" and presses `L` once
- **THEN** pads Q, W and E are in group 1, pad R keeps the group off, and the panel says
  `mute    1` under the pitch

#### Scenario: Undo a group
- **WHEN** the owner puts pad Q into group 2 and presses `Ctrl+Z`
- **THEN** pad Q has the group off again

### Requirement: The mute group is seen on the pad
The system SHALL show the mute group of a pad at the end of its mode line, as
`mute N`, and F1 SHALL explain it. A pad with the group off shows no group.

#### Scenario: The group on the pad
- **WHEN** pad Q is one-shot at +1 dB in group 2 and pad W has the group off
- **THEN** the mode line of pad Q says `⇢ +1 mute 2`, the one of pad W has no `mute`,
  no frame carries a group digit, and F1 has a row for `mute N`

### Requirement: The mute group in the project
The system SHALL save the mute group with the pad in the project and restore it on open;
a project saved before mute groups opens with every group off. Swapping two pads
(`Ctrl+X`) swaps their groups with the rest of the pad.

#### Scenario: The group is saved
- **WHEN** pad Q is in group 1, the owner saves, restarts and opens the project
- **THEN** pad Q is in group 1 and cuts the others of group 1

#### Scenario: An older project
- **WHEN** the owner opens a project saved before mute groups
- **THEN** every pad has the group off and every pad rings on as before

### Requirement: The pad panel fits the window
The pad panel SHALL fit any window height and width.
Short windows SHALL fold unopened page blocks to `▸` title lines (`─ SOURCE ▸ ──`), lowest first, until they fit.
Taller windows SHALL unfold them as the panel grows with the grid, not the strip.
Top pad, scope, file and mode rows SHALL always stay visible.

#### Scenario: A shorter window folds the blocks
- **WHEN** the window is 100×40, the edit of a pad with effects is open on TONE
- **THEN** the TONE, ENV and FILTER rows are seen, SOURCE and FX show only their titles
  with `▸`, and the scope row is seen; on SOURCE the SOURCE rows are seen and FILTER folds

### Requirement: Folded pad panel scrolls when necessary

If folded blocks still do not fit, they SHALL scroll to keep the selected parameter visible.
First/last block rows SHALL say `▲ N more` / `▼ N more` for hidden rows above/below.

#### Scenario: A very short window scrolls
- **WHEN** the window is 100×30 and the owner walks with `K` from volume to the end of the
  parameters
- **THEN** the picked parameter is in sight at every step, `▼ N more` stands at the
  bottom while rows are below, `▲ N more` at the top while rows are above, and the scope
  row is seen the whole time

### Requirement: Narrow pad panel labels

When labels and values do not fit, labels SHALL shorten (`vol`, `pit`, `atk`…) and the frame title SHALL name only the open page and its position (`TONE 1/7`).

#### Scenario: A narrow window
- **WHEN** the window is 70×40 and the edit is open on TONE
- **THEN** the rows say `vol`, `pit`, `mute` with their whole values, and the frame title
  says `TONE 1/7`

### Requirement: The closed panel names the edit key
The system SHALL, while the pad edit is closed, end the first row of the pad panel with the
key that opens it, as the keymap has it: `pad W  bank 1   Enter: edit`. In the edit the
frame with its pages tells the mode, and the hint is gone.

#### Scenario: The closed panel names the edit key
- **WHEN** the pads screen is open with the pad edit closed
- **THEN** the first row of the panel says `Enter: edit`; after `Enter` the panel is framed
  and the hint is gone

### Requirement: Marks in the bank row
The system SHALL show the marks in the bank row: a bank with marked pads shows in its
glyphs only the marked pads, as yellow dots, the open bank and the closed banks alike, in
the bank row above the grid and in the pad line of the hidden grid. A bank with no marks
shows its fill as before. While pads of the open bank are held or flash, its glyphs show
those pads in white, and the yellow marks again after.

#### Scenario: Marks in another bank are seen
- **WHEN** all 16 pads of bank 2 are marked, the owner turns to bank 5, hits three pads
  with marking on and turns back to bank 2
- **THEN** the bank row shows bank 2 with all dots yellow and bank 5 with three yellow dots
  at the places of its marked pads, and the other banks show their fill

#### Scenario: A hit over the marks
- **WHEN** bank 2 has marks and the owner holds pad Q of bank 2 with marking off
- **THEN** while pad Q is held or flashes the glyphs of bank 2 show only pad Q in white,
  and the yellow marks after

#### Scenario: No marks, the fill
- **WHEN** no pad is marked
- **THEN** every bank in the bank row shows its fill, a dot for every pad with a sound

### Requirement: Reset a pad to clean
In pad edit, `Backspace` SHALL reset scope pads to freshly loaded defaults: volume 0 dB, one-shot, pitch 0, default envelope, filter, drive, sends and mute group.
File and source (whole file, linked slice or own bounded piece) SHALL stay.
Empty pads SHALL be skipped.
Sounding pads SHALL reset without clicks.
Reset SHALL make one undo step.
Edit help SHALL name `Bksp reset`.

#### Scenario: A pad back to clean
- **WHEN** pad Q plays slice 3 with the pitch -5, the filter low-pass and the volume -6 dB,
  the owner opens the edit, presses `Backspace` and `Enter`
- **THEN** pad Q plays slice 3 with the pitch 0, the filter off, the volume 0 dB, one-shot,
  and its top frame carries only `◻`

### Requirement: Pad reset confirmation

Reset SHALL ask in the question box, naming `Reset pad Q to clean?` or `Reset 17 pads to clean?`, then what stays and `Enter: reset · Esc: keep`.
`Enter` SHALL reset, `Esc` SHALL keep, and other keys SHALL do nothing while it asks.

#### Scenario: The question names the pads
- **WHEN** 17 pads are marked, the owner opens the edit and presses `Backspace`
- **THEN** the question box says `Reset 17 pads to clean?` and `Enter: reset · Esc: keep`,
  and no pad has changed yet

#### Scenario: Esc keeps
- **WHEN** the reset question is open and the owner presses `Esc`
- **THEN** the question is gone, every pad keeps its settings and the edit stays open

#### Scenario: Undo a reset
- **WHEN** 16 marked pads with different settings are reset and the owner presses
  `Ctrl+Z`
- **THEN** every one of the 16 pads has its own settings back and the status says
  `undo: reset on 16 pads`

#### Scenario: The edit names the reset key
- **WHEN** the pad edit is open
- **THEN** the help line names `Bksp reset`

#### Scenario: A sounding pad resets without a click
- **WHEN** pad Q sounds with the volume −12 dB and the filter low-pass, and the owner
  resets it
- **THEN** the sound goes on and comes to 0 dB and an open filter without a click

#### Scenario: Backspace outside the edit
- **WHEN** no pad is marked, the edit is closed and the owner presses `Backspace` on the
  pads screen
- **THEN** only the selected pad is cleared, as before, and nothing asks

### Requirement: The mode line of a pad
Below the pad name, its mode line SHALL show `⇥` for a gate pad (play until release).
A one-shot pad (play to end) is the default and SHALL show no mode glyph.
It SHALL append nonzero volume, active `mute N`, then `✱` for extra settings per editing, "The pad marks its settings".
No separators SHALL appear, so `⇥ -24 mute 8 ✱` fits 13 cells.
The panel SHALL name the mode in words (`mode    one-shot`), and F1 SHALL name `⇥` and state that one-shot has no mark.

#### Scenario: The mode as a glyph
- **WHEN** pad Q is one-shot at +1 dB and pad W is gate at 0 dB
- **THEN** the mode line of pad Q says `+1` and the one of pad W says `⇥`

#### Scenario: F1 names the mode glyphs
- **WHEN** the owner opens the full help (F1) on the pads screen
- **THEN** it names `⇥` gate and states that a one-shot pad has no mark

### Requirement: Name a pad

`Ctrl+N` on Pads SHALL open name input for the selected pad, starting with its displayed name.
The grid SHALL preview typed text immediately, including an empty line for empty input.
Both keyboard layouts SHALL type without playing pads.
Renaming SHALL NOT change sound.

#### Scenario: Rename a pad
- **WHEN** the owner selects a pad, presses `Ctrl+N`, types a name and presses `Enter`
- **THEN** the grid shows the name and the sound stays unchanged

#### Scenario: See the pad name while typing
- **WHEN** the owner types or deletes characters in the pad name line
- **THEN** the pad shows the text immediately, before it is saved

### Requirement: Pad name input fits the pad

A visible cursor SHALL mark the input's end, including empty names.
Input SHALL fit inside every pad width.
For overwide text, the visible part SHALL follow the end while keeping the full name.

#### Scenario: The pad name cursor
- **WHEN** the name input is open, including when it is empty or wider than the pad
- **THEN** a cursor shows at the end of the visible input inside the frame
- **AND** closing the input removes the cursor

#### Scenario: A long pad name while typing
- **WHEN** the owner types a name wider than the pad, including wide characters
- **THEN** the end of the text is visible inside the frame and the full name is kept

### Requirement: Pad name confirmation and persistence

`Enter` SHALL save the name.
`Esc` SHALL cancel the preview and restore the previous name.
An empty name on `Enter` SHALL restore the sample file name.
Custom names SHALL appear in the grid and be saved in project or scratch session.
Older sessions without names SHALL show sample file names.

#### Scenario: Cancel the pad name
- **WHEN** the owner edits a pad name and presses `Esc`
- **THEN** the name stays as it was

#### Scenario: Restore the file name
- **WHEN** the owner clears the pad name and presses `Enter`
- **THEN** the pad shows the file name again

#### Scenario: Pad name after restart
- **WHEN** the owner names a pad, saves the project and restarts
- **THEN** the same name shows on that pad

### Requirement: Pad names follow sound state changes

Successful sound replacement or clearing SHALL clear the custom name.
Failed loads SHALL keep the previous name.
Swaps SHALL move names with sounds.
Pad settings resets SHALL keep names.
Undo SHALL restore the name with pad state.

#### Scenario: Replace a named pad sound
- **WHEN** a new sample replaces the sound of a named pad
- **THEN** the pad shows the new sample file name

#### Scenario: A failed replacement keeps the name
- **WHEN** replacement of a named pad sound fails
- **THEN** the old sound and name stay

#### Scenario: Swap named pads
- **WHEN** two pads are swapped
- **THEN** each custom name moves with its sound

#### Scenario: Clear and undo a named pad
- **WHEN** the owner clears a named pad and undoes the clear
- **THEN** the sound and its custom name return

### Requirement: Drive sound stays the same after optimization

The system SHALL preserve the drive transfer curve and the smooth change of drive on a
sounding pad. An offline comparison against the previous formula SHALL differ by at most
0.000001 per sample for fixed drive and for a drive that changes during playback. The
comparison SHALL contain no non-finite samples or clipping. The benchmark SHALL keep
32 voices sounding for its full measurement, including when pitch is above zero.

#### Scenario: The optimized drive matches the previous sound
- **WHEN** the same sample is played with fixed and changing drive using the old formula
  and the optimized engine
- **THEN** the rendered outputs differ by at most 0.000001 per sample and contain no
  non-finite samples or clipping

#### Scenario: All benchmark voices stay sounding
- **WHEN** the 32-voice benchmark runs with pitch above zero
- **THEN** all voices have sample data for the whole measurement

### Requirement: Keyboard bank controls

The system SHALL turn banks with PageUp/PageDown and Ctrl+1…8 on every screen with
pads, including in edit. On Pads, U/O SHALL also turn banks with edit open or closed,
keeping the selected pad position. J/L and Left/Right SHALL change values, not banks,
in the shipped map. Owner keymaps that explicitly bind those keys to bank actions
SHALL keep their bindings. Controller bank actions SHALL stay unchanged. Neither
bank keys nor arrows SHALL choose another pad position; a pad hit chooses it.

#### Scenario: L turns to the next bank
- **WHEN** the owner explicitly maps L on Pads to `bank.next` and presses it on bank 1
- **THEN** bank 2 opens with the same pad position selected

#### Scenario: The next bank while editing a pad
- **WHEN** edit of pad W in bank 1 is open and the owner presses PageDown
- **THEN** bank 2 opens, edit stays open on W and J/L still change its selected value

#### Scenario: The same keys on the files screen
- **WHEN** bank 1 is open on Files and the owner presses PageDown
- **THEN** bank 2 is shown on the grid and the file list does not move

#### Scenario: U and O turn banks on Pads
- **WHEN** Pads is on bank 1 with edit closed and the owner presses O, then U
- **THEN** bank 2 opens and then bank 1, with the selected position unchanged

#### Scenario: U and O turn banks during pad edit
- **WHEN** pad edit is open and the owner presses O or U
- **THEN** the bank turns, edit stays open and J/L keep changing the selected value

### Requirement: Pads playback badge

F2 SHALL show PLAY beside its title, 10 cells including padding: ` PLAY     ` idle, ` PLAY ▶   ` during actual ready-current-project pad playback.
Idle SHALL be dim on normal background.
Playback SHALL use the existing blue background and play icon.
It SHALL follow actual voices including gate release fades, not held keys, flashes or queued requests.
Keyboard, raw MIDI and learned buttons SHALL give the same result through production routes.

#### Scenario: Pads PLAY badge follows keyboard and MIDI playback
- **WHEN** a keyboard pad, raw MIDI note or learned pad button triggers a pad on F2
  and the real audio callback advances it
- **THEN** PLAY lights blue with the play icon until the voice ends naturally or its
  gate release fade finishes, then returns to dim without the icon
- **AND** a queued hit alone or a held key after natural sound end does not keep it lit

### Requirement: Pads PLAY badge scope and layout

PLAY SHALL include every bank with edit open/closed and grid hidden/shown, regardless of selection.
It SHALL coexist with shared mode badges while retaining its ten-cell slot, blue active style and actual-voice meaning.
Following title content SHALL keep its columns as playback changes.
Sound, input routes, keys, transport, F3 playback rules and saved data SHALL stay unchanged.

#### Scenario: Pads PLAY badge follows all sounding pads
- **WHEN** several pads sound while selection, bank, edit, grid visibility or title mode badges change
- **THEN** PLAY stays active until the last current-project pad voice ends, regardless of selection or bank
- **AND** mode badges do not turn a preview, effect tail or old-project voice into pad playback

### Requirement: Pads PLAY badge excludes other sound

Previews, effect tails alone, rejected/blocked hits and old-project voices SHALL NOT light PLAY.
An unready project or missing audio SHALL show idle.

#### Scenario: Pads PLAY badge ignores silent requests and old projects
- **WHEN** question or learn blocks a hit, its audio queue rejects it, the project is
  not ready, audio is absent, or the project changes while an old pad still sounds
- **THEN** those requests and old voices do not light PLAY
- **AND** a valid current-project voice still lights it until that voice ends

#### Scenario: Pads PLAY badge ignores previews and effect tails
- **WHEN** no pad voice remains but a sample preview or an effect tail still sounds
- **THEN** PLAY is dim without the icon even though the output is not silent

#### Scenario: Pads PLAY badge has fixed width
- **WHEN** F2 changes between silence and pad playback with pad edit open or closed,
  the grid hidden or shown, and a MARK plate present or absent
- **THEN** PLAY keeps its 10-cell width and the following title content does not move
- **AND** MARK keeps its existing text and colors and the shared screen frame stays put

### Requirement: Pad mode corner marks

File placement and copy SHALL show `◎` on available targets and `!` on occupied targets that already require confirmation.
Swap SHALL show `◎` without inventing a replacement question.
Chop assignment SHALL show planned slice numbers on target pads.
These marks SHALL supplement readable pad identity and existing mark dots, not change actions, targets or saved state.

#### Scenario: Placement marks distinguish confirmation from swap
- **WHEN** file placement, copy and swap each target empty and occupied pads through keyboard and raw MIDI runs
- **THEN** corner marks reflect existing target and confirmation behavior
- **AND** file/copy replacement keeps its existing question while an occupied swap stays direct

#### Scenario: Assignment numbers appear on target pads
- **WHEN** the owner plans slices onto pads on Chop through keyboard or raw MIDI
- **THEN** target pads show their actual planned slice numbers without applying the plan
- **AND** changing a target or cancelling assignment updates the numbers without changing sound state

### Requirement: Yellow grid mode outline

A visible pad grid SHALL use a yellow mode outline during marking, placement, copy, swap, assignment or learning.
Yellow SHALL mean that pads do more than ordinary playback, not that they are necessarily silent.
Existing hit flashes, selection, marked-pad fill and bank membership SHALL remain distinguishable.
Leaving the mode SHALL restore the ordinary outline without moving cells or changing note routing.

#### Scenario: Marking changes appearance without silencing pads
- **WHEN** the owner enables marking and plays pads through keyboard or raw MIDI
- **THEN** the mode outline is yellow, marks change and the pads still sound as before
- **AND** selected, held and flashing pads retain their existing distinctions

#### Scenario: Learning and placement keep existing sound rules
- **WHEN** an existing learning, copy, swap, file placement or assignment mode starts and ends
- **THEN** its outline and badge agree with the current state
- **AND** existing note blocking, audition, velocity and release ownership remain unchanged

### Requirement: Mode feedback with hidden pads

Hiding the grid SHALL retain the shared mode badge and existing pad-line behavior.
Mode marks SHALL NOT force the grid open except for Chop assignment under its existing rule.
Closing assignment SHALL restore the prior grid visibility.
Mode feedback SHALL NOT move the help band, value area, status or right column.

#### Scenario: Hidden grids retain visible mode identity
- **WHEN** the owner hides the grid and starts marking or a pending file, copy or swap operation
- **THEN** its mode remains visible in the title while the pad line and hidden layout remain intact
- **AND** starting and closing assignment temporarily shows the grid, then restores the hidden state

### Requirement: Per-pad slice Pitch Gate

Each pad SHALL have a saved Pitch Gate boolean, true by default. It SHALL act only on sequenced notes whose onset uses a linked slice and positive pitch. Manual hits, Chop previews, whole-file sources, detached pad pieces and non-positive pitch SHALL NOT receive its cut. Changes SHALL affect future note eligibility, not deadlines of sounding notes.

#### Scenario: Pattern Pitch Gate eligibility preserves manual sound
- **WHEN** equivalent linked slices, whole files and detached pieces play manually and through A1 at negative, zero and positive pitch
- **THEN** only positively pitched linked slices with Pitch Gate On receive an automatic step cut
- **AND** manual hits and previews sound the same with the switch On or Off

#### Scenario: Pitch Gate changes apply to future note eligibility
- **WHEN** pitch, source or Pitch Gate changes after a note starts
- **THEN** that note retains its existing deadline and the next note uses the new eligibility

### Requirement: Pitch Gate uses the next grid boundary

An eligible note SHALL start its existing short cut at the first pattern grid line strictly after its compiled onset. Empty steps SHALL supply boundaries. Quantize Off SHALL use 1/16 for this gate. Strength SHALL NOT move grid lines. Straight and triplet grids SHALL retain their exact spacing. The loop end SHALL also be a boundary.

#### Scenario: Pitch Gate cuts at empty straight and triplet steps
- **WHEN** eligible notes play before empty steps on every supported straight or triplet grid
- **THEN** each cut starts within one output frame of the next grid line, even with no hit on that line
- **AND** Quantize Off uses 1/16 and changing strength does not move the gate grid

#### Scenario: Pitch Gate respects off-grid onsets and the loop end
- **WHEN** eligible hits lie on a line, between lines or just before the loop seam
- **THEN** each gate uses the strictly later line or the loop end
- **AND** seam releases precede new tick-zero notes without cutting those new notes

### Requirement: Pitch Gate preserves the source and cut path

Pitch Gate SHALL use the existing 5 ms short fade, not the pad's gate-release duration. The cut SHALL NOT move source bounds, extend playback, insert a universal attack or change mute groups. Natural source end SHALL remain authoritative. An earlier ordinary release SHALL start its envelope, but Pitch Gate SHALL still cap that fading voice. A short cut SHALL preserve current fade gain.

#### Scenario: Pitch Gate cuts without extending a slice or changing attack
- **WHEN** eligible slices end before or after their step boundary, including a gate pad with a long release
- **THEN** short slices end naturally and longer slices use the existing short fade at the boundary
- **AND** source bounds, attack, group membership and unrelated voices stay unchanged

#### Scenario: Pitch Gate caps an ordinary release tail
- **WHEN** an eligible gate note has its saved release before, on or after the Pitch Gate boundary
- **THEN** an earlier ordinary release starts its envelope and the Pitch Gate boundary still starts its short cut if that note remains
- **AND** coincident deadlines use the short cut without raising the current fade gain

### Requirement: Pitch Gate follows pad state and undo

Pitch Gate SHALL follow the existing pad change scope, save, swap, copy snapshot and undo rules. Reset to clean SHALL set it to true without changing the source. Clear SHALL retain it like other retained pad parameters. Cancelled or refused operations SHALL leave it unchanged. A scope toggle SHALL set every scope pad to the selected pad's inverse value in one undo step.

#### Scenario: Pitch Gate scope reset copy and undo preserve pad state
- **WHEN** keyboard and raw MIDI toggle mixed scope values, copy a snapshot, reset to clean, clear and undo
- **THEN** each operation follows the specified Pitch Gate state and existing source and undo rules
- **AND** unrelated pads and cancelled operations stay unchanged

### Requirement: The pad edit of an empty pad shows its test tone

In the pad edit of an empty pad, the strip SHALL show the pad's test tone as its piece, with the envelope line over it. The live output SHALL NOT show then.

#### Scenario: The pad edit of an empty pad shows its test tone
- **WHEN** the owner opens the pad edit on an empty pad and raises its attack
- **THEN** the strip shows the decaying test tone with the envelope line, not the live output
- **AND** the attack part of the line grows with the attack

### Requirement: Selected pad keeps its frame

The selected pad SHALL keep its cyan frame while it flashes, while it is held and while it is marked.
A flash SHALL fill only its inside with white. The frame cells SHALL keep the panel background.
A marked and selected pad SHALL keep its dark yellow fill. A pad in hand SHALL keep its magenta frame.
Other pads SHALL flash and show marks as before.

#### Scenario: Selected pad keeps its frame on a hit
- **WHEN** A1 plays and a hit of the selected pad flashes, and another pad flashes at the same time
- **THEN** the selected pad has a cyan frame on the panel background and a white inside
- **AND** the other pad has a white frame and a white inside

#### Scenario: Selected marked pad keeps its frame
- **WHEN** marking is on and the selected pad is marked
- **THEN** the selected pad has a cyan frame and a dark yellow inside, and another marked pad has a yellow frame

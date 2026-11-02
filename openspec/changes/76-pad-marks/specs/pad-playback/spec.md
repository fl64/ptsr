## ADDED Requirements

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
The system SHALL reset with `Backspace` in the pad edit every pad of the change scope to a
freshly loaded pad: volume 0 dB, one-shot, pitch 0, the envelope, the filter, the drive,
the sends and the mute group at their defaults. The file and the source of the pad (the
whole file, a slice or its own piece with its bounds) stay. An empty pad in the scope is
skipped. It asks first in the question box, as the other questions do, naming the pads:
`Reset pad Q to clean?` or `Reset 17 pads to clean?`, then what stays and
`Enter: reset · Esc: keep`; `Enter` resets, `Esc` keeps, and other keys do nothing while
it asks. A reset sounding pad changes without a click. The reset is one undo step. The
help line of the pad edit names the key: `Bksp reset`.

#### Scenario: A pad back to clean
- **WHEN** pad Q plays slice 3 with the pitch -5, the filter low-pass and the volume -6 dB,
  the owner opens the edit, presses `Backspace` and `Enter`
- **THEN** pad Q plays slice 3 with the pitch 0, the filter off, the volume 0 dB, one-shot,
  and its top frame carries only `◻`

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
- **WHEN** the edit is closed and the owner presses `Backspace` on the pads screen
- **THEN** only the selected pad is cleared, as before, and nothing asks

## MODIFIED Requirements

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

### Requirement: Bank colors
The system SHALL give each of the eight banks its own soft color, show every bank in the bank row in its color and draw the frames of the open bank's pads in that color; the colors of state (selected cyan, in hand magenta, marked yellow, hit white) stay above the bank color.

#### Scenario: Bank colors
- **WHEN** the owner switches from bank 1 to bank 2
- **THEN** the idle pad frames change from the color of bank 1 to the color of bank 2, and the bank row shows each of the eight banks in its own color

#### Scenario: Meanings stay
- **WHEN** a pad of bank 3 is selected and another one is hit
- **THEN** the selected pad's frame is cyan and the hit pad flashes white, not the bank color

### Requirement: Change scope
The system SHALL apply the pad changes to the change scope: the marked pads, or the selected pad when no pad is marked. The panel always names the scope on its second row, under the pad and its bank:
`scope   pad Q` or `scope   17 marked`, the same word as the help line (68). Volume and pitch change relatively: every pad in the scope moves by the same step and the differences between them stay; a pad at a limit stays there and the pop-up says how many pads hit it. The mode is set to the next mode of the selected pad on every pad of the scope, with `H`. Clearing a pad with `Backspace` outside the edit acts on the selected pad only.

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

#### Scenario: Clear stays on one pad
- **WHEN** all pads of a bank are marked, the edit is closed and the owner presses `Backspace`
- **THEN** only the selected pad is cleared

### Requirement: Marking pads
The system SHALL turn marking on and off with `M`, the marker key; while it is on, a pad hit marks or unmarks the pad and the pad still sounds. `Ctrl+A` marks the 16 pads of the open bank and turns marking on; when all 16 are marked already, it unmarks them. Marks stay when the bank turns. While marking is on or pads are marked, the title line of the pads screen shows a yellow plate next to `F2 PADS`: `MARK · hit pads · 17 marked` while marking is on, `17 MARKED · Esc drops` while it is off. Marked pads become the scope instead of the selected pad. `Esc` drops all marks and turns marking off; in the pad edit the first `Esc` closes the edit and keeps the marks.

#### Scenario: Three kicks quieter
- **WHEN** the owner presses `M`, hits three pads, presses `M` and lowers the volume by 4 dB
- **THEN** the panel says "scope   3 marked" and only these three pads are 4 dB quieter

#### Scenario: Unmark
- **WHEN** marking is on and the owner hits a marked pad again
- **THEN** that pad is no longer marked

#### Scenario: Drop the marks
- **WHEN** pads are marked, the edit is closed and the owner presses `Esc`
- **THEN** no pad is marked and the scope is the selected pad again

#### Scenario: The bank, then a few more
- **WHEN** on bank 2 the owner presses `Ctrl+A`, hits pads Q and W, turns to bank 5 and
  hits pads 1, 2 and 3
- **THEN** 17 pads are marked: bank 2 without Q and W, and pads 1, 2 and 3 of bank 5

#### Scenario: Ctrl+A again unmarks the bank
- **WHEN** all 16 pads of bank 2 and pad 1 of bank 5 are marked and the owner presses
  `Ctrl+A` on bank 2
- **THEN** only pad 1 of bank 5 stays marked and marking stays on

#### Scenario: Esc in the edit keeps the marks
- **WHEN** three pads are marked, the edit is open and the owner presses `Esc`
- **THEN** the edit closes and the three pads stay marked; a second `Esc` drops the marks

#### Scenario: The mark plate
- **WHEN** the owner presses `M` on the pads screen and hits three pads, then presses `M`
- **THEN** the title line shows `MARK · hit pads · 3 marked` on yellow while marking is
  on, then `3 MARKED · Esc drops`; after `Esc` the plate is gone

### Requirement: Pad knobs for the scope
The system SHALL let the learned `pad.volume` and `pad.pitch` knobs set the selected pad with soft takeover and move the other pads of the scope by the same amount.

#### Scenario: Pitch knob on a bank
- **WHEN** all 16 pads of bank 2 are marked and the owner turns the learned pitch knob up by 3 semitones
- **THEN** the selected pad and every other pad of bank 2 are 3 semitones higher

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
- **WHEN** all 16 pads of bank 2 are marked and the owner raises the release
- **THEN** every pad of bank 2 gets the longer release and the panel names the scope

#### Scenario: Old session
- **WHEN** the session file was saved before the envelope existed
- **THEN** it opens and every pad sounds as it did: no attack, no decay, the same release as today

### Requirement: Undo on the pads
The system SHALL undo the last change of the pads with `Ctrl+Z` on the pads and files
screens, one step a press, back through the last 100 steps: a parameter of a pad (volume,
pitch, envelope, filter, drive, sends), the mode, a swap, a clear, a reset, a file put on a pad,
the source of a pad, a shared effect slot. A knob turn or a run of value keys on one
parameter is one step, on every pad it touched. An undone pad sounds as it did before the
step. The status line says what came back (`undo: volume on 16 pads`), or `nothing to
undo`. The selection, the bank, the scope, the pages and the marks change no pad and are
no steps. The history lives until the program closes.

#### Scenario: Undo a knob turn
- **WHEN** the owner turns the volume knob of pad Q from 0 to −12 dB in one turn and
  presses `Ctrl+Z`
- **THEN** pad Q is at 0 dB again and sounds so, after one press

#### Scenario: Undo over the scope
- **WHEN** all 16 pads of the bank are marked, the owner raises the cutoff of the 16 pads
  and presses `Ctrl+Z`
- **THEN** every one of the 16 pads has its cutoff back, and the status says
  `undo: cutoff on 16 pads`

#### Scenario: Undo a swap
- **WHEN** the owner swaps pads Q and V with `Ctrl+X` and presses `Ctrl+Z`
- **THEN** Q and V play what they played before, with their settings

#### Scenario: Undo a clear
- **WHEN** the owner clears pad Q with `Backspace` and presses `Ctrl+Z`
- **THEN** pad Q has its file back and sounds as before

#### Scenario: Undo a shared slot
- **WHEN** the owner turns FX 2 from a delay to a reverb and presses `Ctrl+Z`
- **THEN** FX 2 holds the delay again with its time and feedback

#### Scenario: Nothing to undo
- **WHEN** the program has just started and the owner presses `Ctrl+Z` on the pads screen
- **THEN** nothing changes and the status says `nothing to undo`

#### Scenario: What changes no pad
- **WHEN** the owner raises the volume of pad Q, then presses `Ctrl+A` twice, turns
  the bank and selects pad W, and presses `Ctrl+Z`
- **THEN** the volume of pad Q comes back; the bank, the selection and the marks stay as
  they are

#### Scenario: Undo links it back
- **WHEN** pad 7 plays slice 3, the owner moves its start on the SOURCE page and presses
  `Ctrl+Z`
- **THEN** pad 7 says `slice 3` again and plays slice 3

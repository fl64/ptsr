## MODIFIED Requirements

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

### Requirement: Pads PLAY badge scope and layout

PLAY SHALL include every bank with edit open/closed and grid hidden/shown, regardless of selection.
It SHALL coexist with shared mode badges while retaining its ten-cell slot, blue active style and actual-voice meaning.
Following title content SHALL keep its columns as playback changes.
Sound, input routes, keys, transport, F3 playback rules and saved data SHALL stay unchanged.

#### Scenario: Pads PLAY badge follows all sounding pads
- **WHEN** several pads sound while selection, bank, edit, grid visibility or title mode badges change
- **THEN** PLAY stays active until the last current-project pad voice ends, regardless of selection or bank
- **AND** mode badges do not turn a preview, effect tail or old-project voice into pad playback

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Change scope
The system SHALL apply volume, pitch and mode changes to the change scope, stepped with `Ctrl+A`: the selected pad (or the marked pads when there are marks), the current bank, all pads. The panel always names the scope on its second row, under the pad and its bank:
`scope   pad Q`, `scope   3 marked`, `scope   bank 2`, `scope   all pads`, the same word as
the help line (68). Volume and pitch change relatively: every pad in the scope moves by the same step and the differences between them stay; a pad at a limit stays there and the pop-up says how many pads hit it. The mode is set to the next mode of the selected pad on every pad of the scope, with `H`. Clearing a pad acts on the selected pad only.

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
- **THEN** the panel shows the next scope on its second row, under the pad and its bank:
  "scope   pad Q", "scope   bank 2" or "scope   all pads"; no row under the blocks names it

#### Scenario: Mode for a bank
- **WHEN** the scope is bank 1 and the owner presses `H`
- **THEN** every pad of bank 1 gets the next mode of the selected pad

#### Scenario: Clear stays on one pad
- **WHEN** the scope is a bank and the owner presses `Backspace`
- **THEN** only the selected pad is cleared

## ADDED Requirements

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
The system SHALL draw the mute group of a pad on its frame as a circled digit `①`…`⑧`,
with the other glyphs of the pad, and F1 SHALL explain the glyph. A pad with the group
off has no such glyph.

#### Scenario: The group on the pad
- **WHEN** pad Q is in group 1 and pad W has the group off
- **THEN** the frame of pad Q shows `①`, the frame of pad W shows no circled digit, and F1
  has a row for `①…⑧`

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
The system SHALL fit the pad panel to the window, whatever its height and width. When the
window is too short for every row, the blocks of the pages that are not open fold to their
title line with `▸` (`─ SOURCE ▸ ──`), the lowest first, until the panel fits; a taller
window unfolds them as far as the panel grows (it grows with the pad grid, not with the
strip). When even the folded panel does not fit, the blocks scroll so that
the picked parameter is in sight, and the first and the last row of the blocks say
`▲ N more` and `▼ N more` where rows are out of sight. The pad rows on top (pad, scope,
file, mode) stay in every case. When the panel is too narrow for the labels
and the values, the labels shorten (`vol`, `pit`, `atk`…) and the frame title names only
the open page and its place, `TONE 1/7`.

#### Scenario: A shorter window folds the blocks
- **WHEN** the window is 100×40, the edit of a pad with effects is open on TONE
- **THEN** the TONE, ENV and FILTER rows are seen, SOURCE and FX show only their titles
  with `▸`, and the scope row is seen; on SOURCE the SOURCE rows are seen and FILTER folds

#### Scenario: A very short window scrolls
- **WHEN** the window is 100×30 and the owner walks with `K` from volume to the end of the
  parameters
- **THEN** the picked parameter is in sight at every step, `▼ N more` stands at the
  bottom while rows are below, `▲ N more` at the top while rows are above, and the scope
  row is seen the whole time

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

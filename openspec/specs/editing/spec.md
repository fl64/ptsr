# editing Specification

## Purpose
TBD - created by archiving change 21-edit-mode. Update Purpose after archive.

## Requirements

### Requirement: Edit mode
The system SHALL open an edit mode for the selected object with `Enter` and close it with
`Esc`, a second `Enter` or by switching the screen. In the edit mode the page knobs turn the object's
parameters; pads keep playing, and a pad hit still sounds and selects its pad. The edit
mode and the assign mode never stand together. `Esc` closes the help (F1) before the edit
mode.

#### Scenario: Enter opens, Esc closes
- **WHEN** a pad is selected on the pads screen and the owner presses `Enter` and then `Esc`
- **THEN** while the mode is open the screen names the page and what every knob turns, and after `Esc` the knobs do what they did before

#### Scenario: The music plays on
- **WHEN** a pad sounds and the owner enters the edit mode and turns the decay knob
- **THEN** the sound keeps playing and the decay changes on it without a click

#### Scenario: Esc closes the help first
- **WHEN** the edit mode is open, the owner opens the help with F1 and presses `Esc`
- **THEN** the help closes and the edit mode stays open

### Requirement: Parameter pages
The system SHALL group the parameters of the edited object into pages of at most 3; knob k
turns the k-th parameter of the current page, on every screen. In the edit, and on the pads
screen with the edit closed too, `Tab` turns the page forward and `Shift+Tab` back, in a
ring, and the pick lands on the first parameter of the new page; `I` `K` (`↑` `↓`) walk all parameters of the object one by one, on the
pads screen with the edit closed too, and the page follows the picked parameter. A knob bound to the page (`edit.page`) splits its travel
into equal parts, one per page in this order, and turns to the page under it at once, no
pickup; the pick lands on the first parameter of that page, and a turn that stays inside
the part of the current page keeps the pick. An object with one page keeps it on `Tab`
and on the page knob. The pads screen
has TONE (volume, pitch, mute group), ENV (attack, decay, release), FILTER (filter, cutoff, reso),
SOURCE (source, start, end), FX (drive, send 1, send 2), in this order; the chop screen has BOUNDS (slice start,
slice end). Shared effects are controlled on the FX screen. The panel shows the FX block,
each send naming the kind of its slot.

#### Scenario: The envelope page fills three knobs
- **WHEN** the picked parameter of a pad is attack, decay or release
- **THEN** the page is ENV and the knobs turn attack, decay and release, in this order

#### Scenario: Tab turns the page
- **WHEN** the pad edit is open on the pitch of TONE and the owner presses `Tab` twice
- **THEN** the page is FILTER, the filter type is picked, and knob 1 turns it

#### Scenario: Shift+Tab turns back
- **WHEN** the pad edit is open on TONE and the owner presses `Shift+Tab`
- **THEN** the page is FX and the drive is picked

#### Scenario: A knob picks the page
- **WHEN** the pad edit is open on TONE and the page knob is turned to the middle of its
  travel, then a little further inside the same part
- **THEN** the page is FILTER (the third of five) at once, the filter type is picked, and the
  second turn keeps it

#### Scenario: One page
- **WHEN** the slice edit is open on the slice end and the owner presses `Tab`
- **THEN** the page stays BOUNDS and the slice end stays picked

#### Scenario: Up down walks the parameters
- **WHEN** the edit mode is open and the owner presses `K` and then `I`
- **THEN** the next parameter is picked and then the previous one, and the page follows
  each pick

#### Scenario: The filter page
- **WHEN** the owner presses `K` past the end of the ENV page
- **THEN** the page is FILTER and the knobs turn the filter type, the cutoff and the
  resonance, in this order

#### Scenario: The FX page
- **WHEN** the pad edit is on the FX page
- **THEN** knob 1 turns the drive, knob 2 send 1, knob 3 send 2, and each send line names
  the effect of its slot

#### Scenario: A slot page
- **WHEN** the owner opens the shared FX screen and selects FX 2 with a delay
- **THEN** the controls change the shared slot type, time and feedback rather than the pad

#### Scenario: Up down with the edit closed
- **WHEN** the pads screen is open, the pad edit is closed, the pick is on the volume and
  the owner presses `K` twice, `↓` once and then `I`
- **THEN** the pick goes to the pitch, the mute group, the attack and back to the mute
  group, and the edit stays closed

#### Scenario: Tab with the edit closed
- **WHEN** the pads screen is open, the pad edit is closed, the pick is on the volume and
  the owner presses `Tab` twice and then `Shift+Tab`
- **THEN** the pick goes to the attack of ENV, the filter type of FILTER, and back to the
  attack of ENV

### Requirement: Key roles
The system SHALL split the work between the hands and give every key one meaning per
layer: the left fingers play the pads, the left thumb holds `Ctrl` and `Alt` for the
right hand's keys; the right hand moves and changes with the direction letters `J` `L`
`I` `K` (the arrows are the same keys) and gives the commands. In the edit mode `I` `K`
pick the parameter and `J` `L` change the picked value on every screen; outside it they
walk what the screen shows (the chop cursor and zoom, the banks on the pads screen). A
tap is the fine step, holding repeats the coarse step. A command letter means the same on
every screen it is bound on; no command sits on the left half of the keyboard. `−` `=`
and the arrows do not pick a pad.

#### Scenario: The same keys on a pad and on a slice
- **WHEN** the owner edits a pad on the pads screen and a slice on the chop screen
- **THEN** `I` `K` pick the parameter and `J` `L` change it on both screens, and the
  arrows do the same as the letters

#### Scenario: A tap is fine, holding is coarse
- **WHEN** the edit mode of a pad is open on the attack and the owner taps `L`, then
  holds it
- **THEN** the tap raises the attack by 1 ms and every repeat of the held key by 5 ms

#### Scenario: Arrows change the picked value
- **WHEN** the edit mode is open and the owner presses `→`
- **THEN** the picked value moves a fine step and the selection stays

#### Scenario: Enter closes too
- **WHEN** the edit mode of a pad is open and the owner presses `Enter`
- **THEN** the mode closes, as with `Esc`

#### Scenario: Minus and equal are free
- **WHEN** no edit mode is open and the owner presses `−` on the pads screen
- **THEN** nothing changes

#### Scenario: No command on the left half
- **WHEN** the default keymap is loaded
- **THEN** no command is bound to `1`–`5`, `Q`–`T`, `A`–`G`, `Z`–`B` without `Ctrl` or
  `Alt`, and every command letter but the direction letters `J` `L` `I` `K` and the marker
  key `M` means the same on every screen it is bound on

### Requirement: The screen shows the layer
The system SHALL show at all times which layer is active, which object and which
parameter are picked, and, in the edit mode, which parameter every knob turns now:

- the edit mode changes the frame colour of the panel it edits and names the mode and
  every page of the object in the frame title, the current page lit
  (`EDIT · TONE ENV FILTER SOURCE FX`), so the owner sees where `I` `K` and
  `Tab` lead; a title wider than the panel drops `EDIT ·` first and keeps every page, the
  frame colour still telling the mode;
- the picked parameter is highlighted in the panel; on the chop screen the bound that the
  keys move now is drawn in its own colour, the lines stay thin: the bound of a held
  modifier (`Ctrl` the left one, `Alt` the right one), else the picked bound in the edit
  mode, else the bound that just moved;
- in the edit mode every parameter of the page carries its knob number in the panel;
- a knob turn or a value key lights the changed parameter for about a second, in and out
  of the edit mode;
- the help line of an edit mode says that a pad hit listens.

#### Scenario: The frame tells the mode
- **WHEN** the owner presses `Enter` on a pad and then `Esc`
- **THEN** the pad panel frame changes colour and its title says
  `EDIT · TONE ENV FILTER SOURCE FX` with TONE lit, and after `Esc` the frame is
  as before

#### Scenario: The title shows where Tab leads
- **WHEN** the pad edit is open on TONE and the owner presses `Tab` until FX
- **THEN** the lit page in the title moves through every page in order, the other pages
  stay named in it

#### Scenario: The knob numbers
- **WHEN** the pad edit mode is open on the ENV page
- **THEN** the panel shows the knob number 1 next to attack, 2 next to decay, 3 next to
  release, and the help line names the ENV page

#### Scenario: A knob lights what it turns
- **WHEN** the owner turns the knob bound to the pad volume outside the edit mode
- **THEN** the volume line of the pad panel lights for about a second

#### Scenario: The picked bound is visible
- **WHEN** the slice edit is open and the slice end is the picked parameter
- **THEN** the end line of the selected slice is drawn in the bound colour, its start
  line in the slice colour, both thin

#### Scenario: A held modifier lights its bound
- **WHEN** the chop screen is open and the owner holds `Ctrl`, lets it go and holds `Alt`
- **THEN** the start line of the selected slice takes the bound colour while `Ctrl` is
  held, then the end line while `Alt` is held; no line gets thicker; when the cursor
  stands on that line, the cursor takes the bound colour too, so the lit bound never
  hides under it

### Requirement: The pad marks its settings
The system SHALL end the mode line of a pad (under its name) with `✱` when the pitch, the
envelope (attack, decay, release), the filter (its type is not off; the cutoff and the
reso of a filter that is off count for nothing), the drive or a send differ from a fresh
pad. The volume, the mode and the mute group are on the mode line already and do not
count; the source (a slice or the pad's own piece) does not count either. The top frame
of a pad carries no glyph. The pad panel draws a changed value bright and a default value
dim, and the block line with a change carries `●`. F1 names `✱`.

#### Scenario: A filter marks the pad
- **WHEN** pad R plays a whole file, the owner turns its filter to low-pass and closes the edit
- **THEN** the mode line of pad R ends with `✱`, and its top frame carries no glyph

#### Scenario: A filter that is off marks nothing
- **WHEN** pad R plays a whole file with the filter off and the cutoff at 1.1 kHz
- **THEN** its mode line has no `✱`, the FILTER line carries no `●`, and the cutoff is dim

#### Scenario: A fresh pad has no mark
- **WHEN** a sample is loaded on a fresh pad
- **THEN** its mode line is `⇢` alone

#### Scenario: The mark goes with the setting
- **WHEN** the pitch of a pad is +2 and the owner turns it back to 0
- **THEN** the `✱` is gone from its mode line

#### Scenario: The panel names the settings
- **WHEN** a pad that plays a whole file, with the pitch -5 and the filter low-pass, is selected
- **THEN** the panel draws the pitch and the filter bright, the envelope dim, and the
  TONE and FILTER block lines carry `●`, the ENV and SOURCE lines do not

#### Scenario: A slice is not a setting
- **WHEN** slices 1–8 of a break lie on pads 1–8 with nothing else changed
- **THEN** no pad of the eight shows `✱`

#### Scenario: Volume and group are not counted twice
- **WHEN** pad Q is at -6 dB in mute group 2 with nothing else changed
- **THEN** its mode line is `⇢ -6 mute 2`, with no `✱`

#### Scenario: A send marks the pad
- **WHEN** the owner turns send 2 of pad Q to 20 %
- **THEN** the mode line of pad Q ends with `✱`

#### Scenario: F1 names the mark
- **WHEN** the owner opens the full help with F1 on the pads screen
- **THEN** it names `✱` as "more settings, see the panel", and no frame glyph

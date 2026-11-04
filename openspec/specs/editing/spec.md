# editing Specification

## Purpose
Edit pad and slice parameters through shared pages, keys and controller knobs. Show the active layer, selected parameter and changed settings.

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
The system SHALL group edited parameters into pages of at most 3.
On every screen, knob k SHALL turn parameter k of the current page.
Pads SHALL use these pages in order:
- TONE: volume, pitch, mute group.
- ENV: attack, decay, release.
- FILTER: filter, cutoff, reso.
- SOURCE: source, start, end.
- FX: drive, send 1, send 2.
Chop SHALL use BOUNDS: slice start, slice end.

#### Scenario: The envelope page fills three knobs
- **WHEN** the picked parameter of a pad is attack, decay or release
- **THEN** the page is ENV and the knobs turn attack, decay and release, in this order

### Requirement: Parameter page keys

In edit, and on Pads with edit closed, `Tab` SHALL select the next page and `Shift+Tab` the previous page, cyclically.
A page change SHALL select its first parameter.
An object with one page SHALL keep its page and selected parameter on `Tab`.

#### Scenario: Tab turns the page
- **WHEN** the pad edit is open on the pitch of TONE and the owner presses `Tab` twice
- **THEN** the page is FILTER, the filter type is picked, and knob 1 turns it

#### Scenario: Shift+Tab turns back
- **WHEN** the pad edit is open on TONE and the owner presses `Shift+Tab`
- **THEN** the page is FX and the drive is picked

### Requirement: Parameter page knob

The `edit.page` knob SHALL divide its travel equally among pages in their defined order.
It SHALL select the page at its position immediately, without pickup, and select that page's first parameter.
A turn within the current page's part SHALL keep the selected parameter.
An object with one page SHALL keep that page on the page knob.

#### Scenario: A knob picks the page
- **WHEN** the pad edit is open on TONE and the page knob is turned to the middle of its
  travel, then a little further inside the same part
- **THEN** the page is FILTER (the third of five) at once, the filter type is picked, and the
  second turn keeps it

#### Scenario: One page
- **WHEN** the slice edit is open on the slice end and the owner presses `Tab`
- **THEN** the page stays BOUNDS and the slice end stays picked

### Requirement: Parameter selection follows the page

In edit, and on Pads with edit closed, `I` `K` (`↑` `↓`) SHALL select each object parameter in sequence.
The page SHALL follow the selected parameter.
Shared effects SHALL use the FX screen.
The pad panel SHALL show the FX block, with each send naming its slot's effect kind.

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

I/K and Up/Down SHALL select the parameter in edit.
On Pads and FX, J/L and Left/Right SHALL change the selected value with edit open or closed.
On Chop outside slice edit, they SHALL keep cursor and zoom roles.
In slice edit, they SHALL select and change bounds.
On Files, they SHALL keep file navigation.
U/O SHALL change banks on Pads, including during pad edit.

#### Scenario: The same keys on a pad and on a slice
- **WHEN** the owner edits a pad on Pads and a slice on Chop
- **THEN** I/K pick the parameter and J/L change it on both screens
- **AND** the arrows do the same as the letters

### Requirement: Value key steps

A value key tap SHALL make a fine step.
Holding the key SHALL repeat the coarse step.

#### Scenario: A tap is fine, holding is coarse
- **WHEN** pad edit is open on attack and the owner taps L, then holds it
- **THEN** the tap raises attack by 1 ms and each repeat of the held key by 5 ms

#### Scenario: Arrows change the picked value
- **WHEN** edit is open and the owner presses Right
- **THEN** the picked value moves a fine step and the selection stays

#### Scenario: Enter closes too
- **WHEN** pad edit is open and the owner presses Enter
- **THEN** edit closes, as with Esc

### Requirement: Command keys preserve pad access

The system SHALL reserve left fingers for pads and the left thumb's Ctrl/Alt for right-hand commands.
Bound Ctrl/Alt chords SHALL keep their action or role, except named test exceptions.
Plain letters MAY have screen roles, such as H for pad mode on Pads and hit cuts on Chop.
Commands SHALL NOT use pad block keys alone or with Shift.
Minus, equal and arrows SHALL NOT select a pad.

#### Scenario: Minus and equal are free
- **WHEN** no edit is open and the owner presses Minus on Pads
- **THEN** nothing changes

#### Scenario: No command on the left half
- **WHEN** the default keymap is loaded
- **THEN** no command uses `1`–`5`, `Q`–`T`, `A`–`G`, `Z`–`B` without Ctrl or Alt
- **AND** each bound Ctrl/Alt chord keeps the same action role across screens, except
  named and explained test cases

### Requirement: The screen shows the layer
The system SHALL always show the active layer, selected object and selected parameter.
Edit SHALL change the edited panel's frame colour and name the mode and all pages in its title, with the current page lit.
The pad title SHALL use `EDIT · TONE ENV FILTER SOURCE FX`.
A title wider than the panel SHALL omit `EDIT ·` first but keep all pages.
Frame colour SHALL still identify edit mode.

#### Scenario: The frame tells the mode
- **WHEN** the owner presses `Enter` on a pad and then `Esc`
- **THEN** the pad panel frame changes colour and its title says
  `EDIT · TONE ENV FILTER SOURCE FX` with TONE lit, and after `Esc` the frame is
  as before

#### Scenario: The title shows where Tab leads
- **WHEN** the pad edit is open on TONE and the owner presses `Tab` until FX
- **THEN** the lit page in the title moves through every page in order, the other pages
  stay named in it

### Requirement: Edit parameter labels

The panel SHALL highlight the selected parameter.
In edit, it SHALL show each current-page parameter's knob number and identify what each knob turns.
The edit help line SHALL say that a pad hit listens.

#### Scenario: The knob numbers
- **WHEN** the pad edit mode is open on the ENV page
- **THEN** the panel shows the knob number 1 next to attack, 2 next to decay, 3 next to
  release, and the help line names the ENV page

### Requirement: Changed parameter flash

A knob turn or value key SHALL light the changed parameter for about a second, with edit open or closed.

#### Scenario: A knob lights what it turns
- **WHEN** the owner turns the knob bound to the pad volume outside the edit mode
- **THEN** the volume line of the pad panel lights for about a second

### Requirement: Active slice bound colour

On Chop, the system SHALL colour the bound that keys currently move, while keeping bound lines thin.
A held modifier SHALL take priority: `Ctrl` for the left bound, `Alt` for the right bound.
Without a held modifier, it SHALL colour the selected bound in edit, otherwise the bound that last moved.

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
The pad mode line SHALL end with `✱` when pitch, envelope (attack, decay, release), an active filter, drive or a send differs from a fresh pad.
An off filter's cutoff and reso SHALL NOT count.
Volume, mode and mute group SHALL NOT count again.
The source, whether a slice or the pad's own piece, SHALL NOT count.
The pad's top frame SHALL carry no glyph.
F1 SHALL name `✱`.

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

### Requirement: Pad panel marks changed values

The pad panel SHALL draw changed values bright and default values dim.
A block line with a change SHALL carry `●`.
An off filter's cutoff and reso SHALL remain dim and SHALL NOT mark the FILTER block.

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

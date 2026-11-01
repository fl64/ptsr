## MODIFIED Requirements

### Requirement: Parameter pages
The system SHALL group the parameters of the edited object into pages of at most 3; knob k
turns the k-th parameter of the current page, on every screen. In the edit, `Tab` turns the
page forward and `Shift+Tab` back, in a ring, and the pick lands on the first parameter of
the new page; `I` `K` (`↑` `↓`) walk all parameters of the object one by one and the page
follows the picked parameter. A knob bound to the page (`edit.page`) splits its travel
into equal parts, one per page in this order, and turns to the page under it at once, no
pickup; the pick lands on the first parameter of that page, and a turn that stays inside
the part of the current page keeps the pick. An object with one page keeps it on `Tab`
and on the page knob. The pads screen
has TONE (volume, pitch), ENV (attack, decay, release), FILTER (filter, cutoff, reso),
SOURCE (source, start, end), FX (drive, send 1, send 2), FX1* and FX2* (the slot's type and
the two parameters of that type), in this order; the chop screen has BOUNDS (slice start,
slice end). A page with a star sets the shared slot, not the pad: its block line says
`shared`, the change line says `all pads`. The panel shows the FX block, each send naming
the kind of its slot; while the edit is on a slot's page, the block of that slot stands in
its place, so the panel is no taller than with the FX block.

#### Scenario: The envelope page fills three knobs
- **WHEN** the picked parameter of a pad is attack, decay or release
- **THEN** the page is ENV and the knobs turn attack, decay and release, in this order

#### Scenario: Tab turns the page
- **WHEN** the pad edit is open on the pitch of TONE and the owner presses `Tab` twice
- **THEN** the page is FILTER, the filter type is picked, and knob 1 turns it

#### Scenario: Shift+Tab turns back
- **WHEN** the pad edit is open on TONE and the owner presses `Shift+Tab`
- **THEN** the page is FX2* and the type of FX 2 is picked

#### Scenario: A knob picks the page
- **WHEN** the pad edit is open on TONE and the page knob is turned to the middle of its
  travel, then a little further inside the same part
- **THEN** the page is SOURCE (the fourth of seven) at once, the source is picked, and the
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
- **WHEN** the pad edit is on FX2* with a delay in FX 2 and the owner turns knob 1 one step
- **THEN** FX 2 holds a reverb, knobs 2 and 3 turn its size and tone, the block line says
  `FX 2 * shared` where the FX block stood, and the change line says `all pads` in a
  window of 52 rows; on the TONE page the panel shows the FX block again

### Requirement: The screen shows the layer
The system SHALL show at all times which layer is active, which object and which
parameter are picked, and, in the edit mode, which parameter every knob turns now:

- the edit mode changes the frame colour of the panel it edits and names the mode and
  every page of the object in the frame title, the current page lit
  (`EDIT · TONE ENV FILTER SOURCE FX FX1* FX2*`), so the owner sees where `I` `K` and
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
  `EDIT · TONE ENV FILTER SOURCE FX FX1* FX2*` with TONE lit, and after `Esc` the frame is
  as before

#### Scenario: The title shows where Tab leads
- **WHEN** the pad edit is open on TONE and the owner presses `Tab` until FX2*
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

### Requirement: The pad shows what is set on it
The system SHALL show, without the edit, which blocks of a pad differ from a fresh pad:
the pad cell carries on its top frame a glyph for every changed block, in this order:
`↕` the pitch, `◣` the envelope (attack, decay, release), `∿` the filter on (its
type is not off; the cutoff and the reso of a filter that is off count for nothing), `◻` a part of the file (a slice or the pad's own piece; a pad that plays the whole file has none), `↯` the drive, then for send 1
and send 2 above 0 the glyph of the effect in that slot: `◌` reverb, `⋯` delay. An
unchanged block gives no glyph. The pad panel draws a changed value bright and a default
value dim, and the block line with a change carries `●`. The volume stays on the mode line
and takes no glyph. F1 names the glyphs.

#### Scenario: A filter on the pad
- **WHEN** pad R plays a whole file, the owner turns its filter to low-pass and closes the edit
- **THEN** the top frame of pad R carries `∿` and no other glyph

#### Scenario: A filter that is off
- **WHEN** pad R plays a whole file with the filter off and the cutoff at 1.1 kHz
- **THEN** its frame carries no `∿`, the FILTER line carries no `●`, and the cutoff is dim

#### Scenario: A clean pad
- **WHEN** a sample is loaded on a fresh pad
- **THEN** its top frame carries no glyph

#### Scenario: Back to the default
- **WHEN** the pitch of a pad is +2 and the owner turns it back to 0
- **THEN** the `↕` glyph is gone from the pad frame

#### Scenario: The panel shows the change
- **WHEN** a pad that plays a whole file, with the pitch -5 and the filter low-pass, is selected
- **THEN** the panel draws the pitch and the filter bright, the envelope dim, and the
  TONE and FILTER block lines carry `●`, the ENV and SOURCE lines do not

#### Scenario: Chopped pads carry the square
- **WHEN** slices 1–8 of a break lie on pads 1–8 and pad 9 plays the whole break
- **THEN** pads 1–8 carry `◻` and pad 9 does not

#### Scenario: The send lights its glyph
- **WHEN** FX 1 holds a reverb, FX 2 a delay, and the owner turns send 2 of pad Q to 20 %
- **THEN** the top frame of pad Q carries `⋯`; with both sends above 0 it carries `◌⋯`

#### Scenario: Two reverbs on the frame
- **WHEN** both slots hold a reverb and pad Q sends to both
- **THEN** the top frame of pad Q carries `◌◌`

#### Scenario: F1 names the glyphs
- **WHEN** the owner opens the full help with F1 on the pads screen
- **THEN** the help names every glyph: `↕` pitch, `◣` envelope, `∿` filter, `◻` a part of the
  file, `↯` drive, `◌` reverb, `⋯` delay

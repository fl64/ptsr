## MODIFIED Requirements

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

## MODIFIED Requirements

### Requirement: The pad shows what is set on it
The system SHALL show, without the edit, which blocks of a pad differ from a fresh pad:
the pad cell carries on its top frame a glyph for every changed block, in this order:
`⇕` the pitch, `⩘` the envelope (attack, decay, release), `∿` the filter on (its
type is not off; the cutoff and the reso of a filter that is off count for nothing), `◻` a part of the file (a slice or the pad's own piece; a pad that plays the whole file has none), `↯` the drive, then for send 1
and send 2 above 0 the glyph of the effect in that slot: `◌` reverb, `⋯` delay. An
unchanged block gives no glyph. The pad panel draws a changed value bright and a default
value dim, and the block line with a change carries `●`. The volume stays on the mode line
and takes no glyph. Every glyph takes one cell in the terminal and never covers the frame:
a character of ambiguous width (as `◣` or `①`) is not used. F1 names the glyphs.

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
- **THEN** the `⇕` glyph is gone from the pad frame

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
- **THEN** the help names every glyph: `⇕` pitch, `⩘` envelope, `∿` filter, `◻` a part of the
  file, `↯` drive, `◌` reverb, `⋯` delay

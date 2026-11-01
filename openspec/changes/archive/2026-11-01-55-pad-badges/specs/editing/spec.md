## ADDED Requirements

### Requirement: The pad shows what is set on it
The system SHALL show, without the edit, which blocks of a pad differ from a fresh pad:
the pad cell carries on its top frame a glyph for every changed block, in this order:
`↕` the pitch, `◣` the envelope (attack, decay, release), `∿` the filter on (its
type is not off; the cutoff and the reso of a filter that is off count for nothing), `◻` a part of the file (a slice or the pad's own piece; a pad that plays the whole file has none). An unchanged block gives no
glyph. The pad panel draws a changed value bright and a default value dim, and the block
line with a change carries `●`. The volume stays on the mode line and takes no glyph. F1
names the glyphs.

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

#### Scenario: F1 names the glyphs
- **WHEN** the owner opens the full help with F1 on the pads screen
- **THEN** the help names every glyph: `↕` pitch, `◣` envelope, `∿` filter, `◻` a part of the
  file

#### Scenario: Chopped pads carry the square
- **WHEN** slices 1–8 of a break lie on pads 1–8 and pad 9 plays the whole break
- **THEN** pads 1–8 carry `◻` and pad 9 does not

## REMOVED Requirements

### Requirement: The pad shows what is set on it
**Reason**: seven glyphs on every pad frame read as clutter (owner, 2026-11-02: "it all looks like garbage, a pile"); the grid shows what plays, the pad panel shows how a pad is set.
**Migration**: one mark `✱` on the mode line says that a pad has settings (77, "The pad marks its settings"); the panel names them in words, bright where they differ from a fresh pad.

## ADDED Requirements

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
- **THEN** its mode line is `⇥` alone

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
- **THEN** its mode line is `⇥ -6 mute 2`, with no `✱`

#### Scenario: A send marks the pad
- **WHEN** the owner turns send 2 of pad Q to 20 %
- **THEN** the mode line of pad Q ends with `✱`

#### Scenario: F1 names the mark
- **WHEN** the owner opens the full help with F1 on the pads screen
- **THEN** it names `✱` as "more settings, see the panel", and no frame glyph

# editing Delta

## MODIFIED Requirements

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
- **THEN** its mode line is empty

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
- **THEN** its mode line is `-6 mute 2`, with no `✱`

#### Scenario: A send marks the pad
- **WHEN** the owner turns send 2 of pad Q to 20 %
- **THEN** the mode line of pad Q ends with `✱`

#### Scenario: F1 names the mark
- **WHEN** the owner opens the full help with F1 on the pads screen
- **THEN** it names `✱` as "more settings, see the panel", and no frame glyph

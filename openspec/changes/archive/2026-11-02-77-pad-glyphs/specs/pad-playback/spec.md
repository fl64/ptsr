## ADDED Requirements

### Requirement: The mode line of a pad
The system SHALL draw under the name of a pad its mode line: the mode as a glyph, `⇥`
one-shot (it plays to the end) or `⇢` gate (it plays while held), then the volume when it
is not 0 dB, then `mute N` when the pad is in a mute group, then `✱` when it has more
settings (editing, "The pad marks its settings"): `⇥ +1 mute 2`, `⇢ -6 ✱`, `⇥`; no
separator, so `⇥ -24 mute 8 ✱` fits the 13 cells of the narrowest pad. The pad panel names the mode in words (`mode    one-shot`), and F1 names
the two glyphs.

#### Scenario: The mode as a glyph
- **WHEN** pad Q is one-shot at +1 dB and pad W is gate at 0 dB
- **THEN** the mode line of pad Q says `⇥ +1` and the one of pad W says `⇢`

#### Scenario: F1 names the mode glyphs
- **WHEN** the owner opens the full help (F1) on the pads screen
- **THEN** it names `⇥` one-shot and `⇢` gate

## MODIFIED Requirements

### Requirement: The mute group is seen on the pad
The system SHALL show the mute group of a pad at the end of its mode line, as
`mute N`, and F1 SHALL explain it. A pad with the group off shows no group.

#### Scenario: The group on the pad
- **WHEN** pad Q is one-shot at +1 dB in group 2 and pad W has the group off
- **THEN** the mode line of pad Q says `⇥ +1 mute 2`, the one of pad W has no `mute`,
  no frame carries a group digit, and F1 has a row for `mute N`

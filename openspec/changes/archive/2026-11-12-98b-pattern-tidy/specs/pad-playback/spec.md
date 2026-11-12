# pad-playback Delta

## MODIFIED Requirements

### Requirement: The mode line of a pad
Below the pad name, its mode line SHALL show `⇥` for a gate pad (play until release).
A one-shot pad (play to end) is the default and SHALL show no mode glyph.
It SHALL append nonzero volume, active `mute N`, then `✱` for extra settings per editing, "The pad marks its settings".
No separators SHALL appear, so `⇥ -24 mute 8 ✱` fits 13 cells.
The panel SHALL name the mode in words (`mode    one-shot`), and F1 SHALL name `⇥` and state that one-shot has no mark.

#### Scenario: The mode as a glyph
- **WHEN** pad Q is one-shot at +1 dB and pad W is gate at 0 dB
- **THEN** the mode line of pad Q says `+1` and the one of pad W says `⇥`

#### Scenario: F1 names the mode glyphs
- **WHEN** the owner opens the full help (F1) on the pads screen
- **THEN** it names `⇥` gate and states that a one-shot pad has no mark

## MODIFIED Requirements

### Requirement: Pads pattern event strip

F2 Rhythm SHALL show a header, an all-pads row, and the selected pad's row. A strip of four or more rows SHALL keep an output wave row. Extra rows SHALL show other pads with hits in the open bank. The frame and safety status SHALL keep their positions. For absent A1 the strip SHALL show the recording instruction.

#### Scenario: Pattern strip fits the shared frame

- **WHEN** F2 shows Rhythm with absent, empty, or populated A1 at 80x26 and 120x40
- **THEN** its event rows fit, and the outer frame and warnings stay visible
- **AND** Chop and Files draw no event rows

#### Scenario: Rhythm shows other pad rows when space permits

- **WHEN** several pads in the open bank have A1 hits and the Rhythm view has spare rows
- **THEN** the extra rows name their pads and follow the same moving playback marker

## ADDED Requirements

### Requirement: Sound strip shows pad sample regions

F2 Sound SHALL show the full focus sample wave and the ranges of pads that use it. It SHALL highlight each sounding region and show its playback position. It SHALL group voices by loaded sample and keep all new playback position marks in the strip. It SHALL use the current project and loaded sample identities.

#### Scenario: Sound strip groups multiple slices of one sample

- **WHEN** two pads with different regions of the same sample sound in the current project
- **THEN** the Sound strip shows one full wave with both ranges highlighted and both playback lines

#### Scenario: Sound strip rejects replaced samples and old projects

- **WHEN** a pad sample changes or the app opens a new project during playback
- **THEN** a voice from the replaced pad sample or old project does not mark the loaded wave

### Requirement: Sound strip fits other sounding samples

Sound SHALL show other sounding samples in order of their newest voice. Each named row SHALL show its wave and positions. The strip SHALL show up to one other sample at 80x26 and two at 120x40. A count and pad labels SHALL identify the remaining sounding samples. The pad labels SHALL use the current input labels and name another bank when necessary.

#### Scenario: Sound strip names hidden sounding pads

- **WHEN** four distinct samples sound at 80x26 and 120x40
- **THEN** one and two other samples get named rows respectively
- **AND** the count and pad labels identify the remaining samples and pads

### Requirement: Sound strip focus follows or pins

Sound SHALL follow the newest sounding pad after a 150 ms view hold. A pin SHALL select and retain the loaded sample of the selected pad. It SHALL clear on a second pin action or when its sample is gone. In silence the strip SHALL keep the last available wave and show `silent`.

#### Scenario: Sound strip keeps the last wave in silence

- **WHEN** the last sounding pad voice ends
- **THEN** its sample wave stays visible and the strip shows `silent`

#### Scenario: Sound strip holds focus and pins the selected sample

- **WHEN** another sample sounds within the hold period or while a pin is active
- **THEN** the focus stays until the hold ends or the owner clears the pin

### Requirement: Sound strip position follows the callback

Sound SHALL use positions from completed audio blocks. A restart SHALL replace the position of the new voice with its own start and progress. The envelope level SHALL set the region brightness. Recording and pad edit SHALL have priority over Sound and Rhythm, and the chosen view SHALL return after exit.

#### Scenario: Sound strip follows a restarted voice and its fade

- **WHEN** a pad restarts while its former voice fades
- **THEN** the newest voice position starts in its own region, and the older fade loses brightness

#### Scenario: Sound strip gives priority to edit and recording

- **WHEN** the owner edits a pad or records from Pads
- **THEN** the edit or recording view takes the strip, and the previous view returns after exit

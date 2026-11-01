## ADDED Requirements

### Requirement: Graphic waveform
The system SHALL draw the waveforms with Braille characters by default and SHALL draw them with kitty graphics when the owner switches the picture on in the settings (F12) and the terminal supports it: the chop screen waveform. The switch is kept in the config and takes effect at the next start.

#### Scenario: Braille by default
- **WHEN** the owner starts the program in a terminal with graphics and has not switched the picture on
- **THEN** the waveforms are drawn with Braille characters

#### Scenario: The picture switched on
- **WHEN** the owner sets "Wave" to "picture" on F12 and starts the program again in a terminal with graphics
- **THEN** the waveforms are drawn as pictures

#### Scenario: The picture does not blink
- **WHEN** the owner moves the cursor across slice bounds on a terminal with graphics
- **THEN** the wave stays on the screen the whole time: a changed picture appears over the old one before the old one is taken off

#### Scenario: Terminal without graphics
- **WHEN** the terminal does not support kitty graphics
- **THEN** the waveforms are drawn with Braille characters, all functions are available

> The X-Session chop profile moved to 21-edit-mode (the parameter standard); it lands
> after the key roles and the edit pages are in place.


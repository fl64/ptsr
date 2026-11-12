# audio-settings Delta

## ADDED Requirements

### Requirement: Settings use the edit frame

F12 SHALL draw its body in one frame in the edit color, titled `SETTINGS · AUDIO INPUT MIDI PATHS`, with the open tab lit. A narrow title SHALL keep only the open tab. Labels SHALL be lowercase. The selected row SHALL be bright with `▸` on every tab. Explanation lines SHALL be dim. The keys SHALL keep their roles.

#### Scenario: Settings use the edit frame
- **WHEN** the owner opens F12 at 80x26 and 120x40 and presses Tab through all tabs, then I/K
- **THEN** one frame holds the tab body, its title lights the open tab, and labels are lowercase
- **AND** the selected row is bright with `▸`, and no row is inverse

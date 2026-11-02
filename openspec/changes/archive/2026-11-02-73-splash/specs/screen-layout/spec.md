## ADDED Requirements

### Requirement: The splash in the wave strip
The system SHALL show at start, in the wave strip above the pad grid (the strip that shows
the master output as a wave), a splash instead of the silent wave, and keep it until the
first sound. The splash shows the logo, `ptsr - PT's SampleR`, the version of the build and
a loading line. The logo is this text, character by character:

```
           
  _ _/_ __
 /_// _\/ 
/         
```

The session file is read before the first screen. The loading line names the number of
pads whose samples are loading and says `ready` when loading is complete, or names a file
that did not load. Sample loading continues under the splash, and the rest of the screen
shows as always. When the strip has fewer than four rows, the splash shows `ptsr` and the
loading line instead of the full logo, name and version.

The splash is visible only on the pads screen. Other screens show their own content. If the
owner returns to the pads screen before the first sound, the splash is visible again.
The pad edit shows its envelope or filter curve in place of the splash. When the edit
closes before the first sound, the splash shows again.
The first sound ends the splash, and it never returns: a note that plays (a pad hit from
the keys or from the controller), a sample preview, or a recording that starts. A sound
request ends the splash only when the audio queue accepts it while the project is ready.
No audio, a full queue, an unready project, or a hit on a pad with no sound does not end
it. No key and no time closes the splash by itself; from the first sound on,
the strip shows the wave.

#### Scenario: The splash at start
- **WHEN** the owner starts ptsr
- **THEN** the wave strip shows the logo, `ptsr - PT's SampleR`, the version and the
  loading line, and the pads, the help line and the status line show as always

#### Scenario: A pad hit during the splash
- **WHEN** the splash shows and the owner hits pad `Q`
- **THEN** pad `Q` plays at once and the strip shows the wave from then on

#### Scenario: A command key during the splash
- **WHEN** the splash shows and the owner presses F3
- **THEN** the chop screen shows its own content; when the owner returns to the pads screen
  before the first sound, the splash shows again

#### Scenario: The pad edit takes the strip from the splash
- **WHEN** the owner opens the pad edit before the first sound
- **THEN** the strip shows the envelope or filter curve instead of the splash
- **AND WHEN** the owner closes the edit before the first sound
- **THEN** the splash shows again

#### Scenario: A missing sample on the splash
- **WHEN** the session names a sample file that does not exist and the owner starts ptsr
- **THEN** the loading line names that file

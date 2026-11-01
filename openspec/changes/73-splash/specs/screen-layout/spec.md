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

The loading line names what loads now (the session, the number of samples) and says `ready`
when all is loaded, or names a file that did not load. Loading goes on under the splash,
and the rest of the screen shows as always.

The first sound ends the splash, and it never returns: a note that plays (a pad hit from
the keys or from the controller) or a recording that starts. A hit on a pad with no sound
does not end it. No key and no time closes the splash by itself; from the first sound on,
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
- **THEN** the chop screen opens and the splash stays until the first sound

#### Scenario: A missing sample on the splash
- **WHEN** the session names a sample file that does not exist and the owner starts ptsr
- **THEN** the loading line names that file

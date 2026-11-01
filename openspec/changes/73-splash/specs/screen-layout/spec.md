## ADDED Requirements

### Requirement: The splash screen
The system SHALL show at start, over the screen of the last session, a splash box for 6.4
seconds: as wide as the screen and in the middle of its height. The box shows these lines,
centered, from top to bottom: the logo, `ptsr - PT's SampleR`, the version of the build, a
loading line and `any key: start`. The logo is this text, character by character:

```
          
  _ _/_ __
 /_// _\/ 
/         
```

The loading line names what loads now (the session, the number of samples) and says `ready` when all is loaded, or names a file that did not load. The interface stays behind the box and shows around it. Loading goes on under the splash.

Any key or controller event closes the splash and does its own action at once. When the time ends, the box closes; the screen of the session is behind it.

#### Scenario: The splash at start
- **WHEN** the owner starts ptsr
- **THEN** a box as wide as the screen shows, centered, the logo, `ptsr - PT's SampleR`,
  the version, the loading line and `any key: start`, over the pads screen of the last
  session; 6.4 seconds later the box closes and the pads screen stays

#### Scenario: A pad hit during the splash
- **WHEN** the splash shows and the owner hits pad `Q`
- **THEN** the splash closes and pad `Q` plays at once

#### Scenario: A command key during the splash
- **WHEN** the splash shows and the owner presses F3
- **THEN** the splash closes and the chop screen opens

#### Scenario: A missing sample on the splash
- **WHEN** the session names a sample file that does not exist and the owner starts ptsr
- **THEN** the loading line names that file

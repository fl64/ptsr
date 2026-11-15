# audio-settings Delta

## ADDED Requirements

### Requirement: Set the MIDI button hold time

F12 MIDI SHALL show a `hold time` row. J and L SHALL change the time in 50 ms steps. The range SHALL be 200 to 600 ms. The default SHALL be 300 ms.

The program SHALL save the value under `[midi_gestures]` and apply it to all connected devices. A changed value SHALL affect gestures that start after the change. An invalid saved value SHALL use 300 ms and show a warning.

#### Scenario: Set and keep hold time
- **WHEN** the owner opens F12 MIDI, changes hold time from 300 to 400 ms, and restarts the program
- **THEN** the row shows 400 ms and a new button hold starts at 400 ms
- **AND** a tap released before 400 ms runs on release

#### Scenario: Reject an invalid saved hold time
- **WHEN** the user config contains a hold time outside 200 to 600 ms
- **THEN** the program uses 300 ms and shows a warning
- **AND** MIDI buttons and the other settings remain usable

#### Scenario: Existing hold keeps its start threshold
- **WHEN** a button starts a hold at 300 ms and the owner changes the setting to 400 ms before releasing it
- **THEN** that button uses the 300 ms threshold
- **AND** the next button hold uses 400 ms

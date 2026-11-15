# input-devices Delta

## ADDED Requirements

### Requirement: MIDI buttons can have a tap and a hold

A device profile SHALL accept a plain action string in a screen or edit binding. It SHALL also accept a table with `hold` and an optional `tap`. A plain string SHALL run on the first press. A button with `hold` SHALL wait. Release before the hold time sends `tap` once if present. At the hold time, it sends `hold` down once. Release after hold sends its up event once and SHALL NOT send tap.

#### Scenario: Plain button acts on press
- **WHEN** a test profile gives a button a plain `transport.toggle` action and the owner presses it
- **THEN** transport changes on press without waiting for the hold time
- **AND** release changes nothing

#### Scenario: PLAY tap and hold use different actions
- **WHEN** a test profile gives PLAY a `transport.toggle` tap and `metronome.toggle` hold, and the owner taps it before the threshold and then holds it past the threshold
- **THEN** the tap changes transport only on release
- **AND** the hold changes the metronome once and does not change transport

#### Scenario: RECORD hold ends microphone capture
- **WHEN** a test profile gives RECORD a `record.mic` hold, and the owner holds it past the threshold and releases it
- **THEN** microphone recording starts at hold down and stops at release
- **AND** no tap action runs

### Requirement: Hold actions keep their action semantics

A toggle action SHALL act only on hold down. `record.mic` SHALL start on hold down and stop on hold up. The program SHALL refuse an unknown action, including an unavailable `layer.*` action. The message SHALL name the profile file and control.

#### Scenario: Program refuses an unknown layer action
- **WHEN** a profile gives a button a `layer.panel` hold before the panel exists
- **THEN** the program refuses that binding and names its profile file and control
- **AND** other valid bindings in the profile keep their actions

### Requirement: Another control starts a pending hold

A new press or real movement of another control on the same port SHALL start a pending hold before that control acts. The controls MAY use different MIDI channels. A release, unchanged CC value, or repeated press of the held control SHALL NOT start another hold. Timed decisions SHALL use MIDI receipt times. At the threshold, a hold wins over a release. The MIDI worker SHALL process already received messages before due timers.

#### Scenario: A pad on another channel starts the hold
- **WHEN** a button on channel 1 is down and a pad on channel 10 is hit on the same port before the hold threshold
- **THEN** the button hold starts before the pad hit acts
- **AND** the later button release sends hold up and no tap

#### Scenario: Two ports keep separate holds
- **WHEN** the owner holds buttons with the same channel and number on two ports, and uses another control on only the first port
- **THEN** only the first port starts its hold
- **AND** each port's release completes only its own gesture

#### Scenario: Threshold and repeat use one gesture
- **WHEN** a button sends a press, a repeated press, and a release at or after the hold threshold
- **THEN** one hold down and one hold up occur, with no tap
- **AND** the repeated press does not restart the timer

### Requirement: Gesture state ends with its device

Device disconnect SHALL send up for every active hold on that port and SHALL discard pending taps. It SHALL keep the existing direct pad release behavior. An invalid shared state SHALL refuse new gesture presses but SHALL still allow an existing hold to end. The program cannot detect a missing release message alone. A repeated press SHALL NOT create a second action.

#### Scenario: Disconnect ends an active hold
- **WHEN** a port disconnects while RECORD hold is recording the microphone and another button awaits a tap
- **THEN** microphone recording receives one up action and stops
- **AND** the pending tap does not run and the port reconnects with no held gesture

#### Scenario: Missing release does not duplicate a hold
- **WHEN** a device sends a hold press without its release and then repeats the same press
- **THEN** no second hold or tap runs
- **AND** disconnect ends the active hold once

#### Scenario: Invalid state keeps original release
- **WHEN** the worker accepts a hold before the shared input state becomes invalid and its button releases
- **THEN** its original up action runs once
- **AND** no new gesture starts in the invalid state

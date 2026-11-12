## ADDED Requirements

### Requirement: Device profile layout

A device profile MAY have a `[layout]` section with `rows`: lists of control names of the same profile, in the order of the device top. An empty string SHALL be an empty place.
The `[controls]` group of a control SHALL give its kind: pad, button, knob, fader or crossfader (`crossfaders`).
An invalid layout SHALL NOT change the controls and actions of the device. The status SHALL name the file and the error.

#### Scenario: Profile layout gives places and kinds
- **WHEN** a profile has a `[layout]` with knobs, buttons, faders, a crossfader and empty places
- **THEN** the parse gives each place its control name and kind, and each empty place stays empty

#### Scenario: Invalid layout keeps the device working
- **WHEN** a layout names an unknown control or a control two times
- **THEN** the device keeps its controls and actions, the layout is not used, and the status names the file and the error

#### Scenario: Crossfader group acts as a fader
- **WHEN** the X-Session Pro crossfader is in `[controls.crossfaders]` and the owner moves it
- **THEN** the crossfader sends the same action as before this change

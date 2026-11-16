# input-devices Delta

## MODIFIED Requirements

### Requirement: Device profiles

The system SHALL describe each device with a TOML profile: controls and their starting logical actions per screen. A device without a profile SHALL use the default notes 36 through 51 for pads 1 through 16. The system SHALL copy starting actions into the user config on first connection with no saved bindings. After that, the user config SHALL give the device its actions and the profile SHALL name its controls. For new SMC-PAD panel cells and PLAY/STOP roles, a missing saved binding SHALL use the shipped action at runtime. This exception SHALL not change the user config. An explicit owner action or `none` SHALL take precedence.

#### Scenario: New controller
- **WHEN** a device without its own profile is connected
- **THEN** its notes 36 through 51 play pads, and the input debug screen shows what comes in, so that the owner can write a profile

#### Scenario: Profile with an error
- **WHEN** a profile file cannot be read
- **THEN** the program does not crash, the device works with the default profile, and the status shows the file and line with the error

#### Scenario: First connect
- **WHEN** the X-Session Pro connects and the user config has no bindings for it
- **THEN** the user config gets every role of its profile and left Cue opens the pad edit
- **AND** the MIDI tab shows these rows as the owner's

#### Scenario: Learned wins
- **WHEN** the user config has the X-Session Pro b3 learned for "next bank" before the profile changes
- **THEN** b3 switches to the next bank, its new FX role is not added on that control
- **AND** other profile roles are added only if the device has not already been initialized

#### Scenario: Existing SMC-PAD keeps saved roles when panel defaults arrive
- **WHEN** an initialized SMC-PAD map has a saved action on cell 3, explicit `none` on cell 4 and no binding on cell 5
- **THEN** cells 3 and 4 keep the owner's choices and cell 5 uses its shipped panel role
- **AND** the program does not rewrite the saved map or change any X-Session binding

### Requirement: SMC-PAD pads follow their physical rows

The supported SMC-PAD Master preset SHALL map channel 10 notes 36 through 51 to screen pads in physical rows from bottom to top. It SHALL keep each velocity and both note-off forms. Notes 52 through 67 SHALL be distinct second-bank command cells in the same physical order. PAD BANK and SHIFT SHALL have no invented host action.

#### Scenario: Physical pad rows match the screen
- **WHEN** raw Master MIDI presses and releases physical pads in the four corners of the first bank
- **THEN** each hit sounds once on the corresponding screen corner at its received velocity
- **AND** both note-off forms end the original Gate once

#### Scenario: Second pad bank stays distinct
- **WHEN** raw Master MIDI presses all 16 second-bank notes on channel 10
- **THEN** it recognizes 16 distinct cells ordered from the screen top row to bottom row
- **AND** no second-bank hit plays a sample

### Requirement: SMC-PAD buttons keep measured identities

The supported preset SHALL recognize LEFT, RIGHT, PLAY, STOP and RECORD as channel 1 CC 25 through 29. Press SHALL use the received nonzero value and release SHALL use zero. LEFT and RIGHT SHALL use current selection actions where available. PLAY SHALL use the context play action for a tap and the metronome action for a hold. STOP SHALL use context stop on press. RECORD SHALL keep microphone hold and use pattern recording for an available tap. Saved owner assignments SHALL take precedence over these starting roles.

#### Scenario: Button press and release use the current screen
- **WHEN** the Master port sends press and release for each of CC 25 through 29 on Pads, Files and Chop
- **THEN** each button keeps its name, LEFT and RIGHT select available items, and RECORD ends microphone capture on release
- **AND** PLAY never stops active playback and STOP never starts it

#### Scenario: Owner bindings survive profile installation
- **WHEN** a saved SMC-PAD control has an owner assignment and the new profile loads
- **THEN** that assignment keeps its effect without a reset or migration

## ADDED Requirements

### Requirement: An explicit MIDI none disables a new panel default

A saved SMC-PAD MIDI binding MAY use the action `none` for a panel cell or PLAY/STOP button. The program SHALL keep this value in the user config and SHALL run no action for that control. An explicit `none` SHALL not disable a different control, screen or device. F12 clearing a binding SHALL remove that override and restore the shipped fallback for the new controls.

#### Scenario: Explicit none keeps one SMC-PAD cell inactive
- **WHEN** the owner saves `none` for one second-bank cell and restarts ptsr
- **THEN** that cell stays inactive while other second-bank cells keep their shipped roles
- **AND** F12 shows the override and clearing it restores only that cell's shipped role

### Requirement: SMC-PAD context buttons keep their gesture timing

PLAY's tap SHALL act on release before 300 ms. Its metronome hold SHALL act once at the threshold. Another control on the same port SHALL start a pending hold before it acts. STOP SHALL have no starting hold and SHALL act on press. A missing physical SHIFT message SHALL provide no shortcut or octave/transpose action in ptsr. This change SHALL add no STOP hold setting.

#### Scenario: PLAY tap and hold have separate results
- **WHEN** PLAY is tapped, then held past 300 ms while a pad arrives on its other MIDI channel
- **THEN** the tap starts or continues playback once and the hold toggles the metronome once before the pad acts
- **AND** the held release does not start or stop playback

#### Scenario: STOP acts on press by default
- **WHEN** a raw STOP press and release arrive with the shipped profile
- **THEN** context stop acts once on press and release changes nothing
- **AND** no panel shortcut opens

#### Scenario: Local SHIFT cannot trigger a host command
- **WHEN** the owner uses SHIFT with a pad, encoder or button and no distinct SHIFT MIDI message exists
- **THEN** ptsr routes only the messages the hardware actually sends
- **AND** it does not infer a panel shortcut, octave change or transpose action

# input-devices Delta

## MODIFIED Requirements

### Requirement: Working profiles exclude examples

The profile loader SHALL read only files with the `.toml` extension. It SHALL ignore `.toml.example` files without a profile or error diagnostic. The shipped SMC-PAD profile SHALL match only the measured Master input. A separate empty profile SHALL match the mirrored Private input and suppress its default pad routing. Invalid actual `.toml` files SHALL retain their file-and-line diagnostics and default device routing.

#### Scenario: Shipped profiles load without template errors
- **WHEN** the program loads the actual shipped device directory
- **THEN** all working profiles load without excluded errors
- **AND** the SMC-PAD Master and Private profiles load with distinct exact port matches

#### Scenario: An example does not replace a working profile
- **WHEN** a profile directory contains a valid `.toml` and an unfinished `.toml.example`
- **THEN** the valid profile keeps its actions and the example adds no diagnostic or assignments
- **AND** a device without a valid matching profile keeps default notes 36 through 51

#### Scenario: Invalid working profiles retain default routing
- **WHEN** an actual `.toml` profile contains an invalid value
- **THEN** its diagnostic names the file and line
- **AND** its device still uses default routing without a crash

## ADDED Requirements

### Requirement: SMC-PAD pads follow their physical rows

The supported SMC-PAD Master preset SHALL map channel 10 notes 36 through 51 to screen pads in physical rows from bottom to top. It SHALL keep each velocity and both note-off forms. Notes 52 through 67 SHALL be distinct second-bank buttons in the same physical order, without command actions in this change. PAD BANK and SHIFT SHALL have no invented host action.

#### Scenario: Physical pad rows match the screen
- **WHEN** raw Master MIDI presses and releases physical pads in the four corners of the first bank
- **THEN** each hit sounds once on the corresponding screen corner at its received velocity
- **AND** both note-off forms end the original gate once

#### Scenario: Second pad bank stays distinct
- **WHEN** raw Master MIDI presses all 16 second-bank notes on channel 10
- **THEN** it recognizes 16 distinct controls ordered from the screen top row to bottom row
- **AND** no second-bank hit plays a sample or runs a command in this change

### Requirement: Mirrored SMC-PAD ports make one action

The measured Private input SHALL not act on its mirrored notes or CC values. The Master input SHALL keep its pad, button and encoder routes. The generic third `SINCO` port SHALL not match either SMC-PAD profile. Another device SHALL keep its own profile and saved assignments.

#### Scenario: Two SMC-PAD ports do not double a hit
- **WHEN** identical note-on, note-off and button bytes arrive through Private and Master
- **THEN** each Master control acts once and Private adds no sound or action

#### Scenario: X-Session remains independent
- **WHEN** the X-Session Pro and both SMC-PAD ports send their own controls in one input sequence
- **THEN** each device keeps its own profile, actions and held state
- **AND** unplugging the SMC-PAD ends its held gate without ending an X-Session hold

### Requirement: SMC-PAD encoder setup guards relative actions

Before relative actions start, ptsr SHALL save the active preset's 16 encoder records. It SHALL change only these records and enable relative actions only after matching read-back. It SHALL refuse an unsupported or changing preset. On a partial write, it SHALL restore the saved records and verify them. A failure SHALL name its cause, keep relative actions off and keep pads and buttons usable. It SHALL not reset the device or change other presets.

#### Scenario: Verified relative encoder setup
- **WHEN** the active preset has the measured absolute encoder records and read-back matches all 16 relative records after a write
- **THEN** CC 30 through 45 use relative steps for the Master profile
- **AND** every other preset byte stays equal to its original value

#### Scenario: Absolute bytes cannot become false steps
- **WHEN** setup has not succeeded and the Master port sends absolute CC values
- **THEN** no relative value, page or selection changes
- **AND** the setup status explains why the encoders are inactive

#### Scenario: Interrupted setup restores original records
- **WHEN** a preset change, write error or read-back mismatch occurs after one encoder record changes
- **THEN** setup stops, attempts to restore all original encoder records and verifies the result
- **AND** the status reports a failed restore if verification differs

### Requirement: SMC-PAD buttons start safe actions

The supported preset SHALL recognize LEFT, RIGHT, PLAY, STOP and RECORD as channel 1 CC 25 through 29. Press SHALL use the received nonzero value and release SHALL use zero. LEFT and RIGHT SHALL select the current screen item. PLAY SHALL start or continue the current screen item. STOP SHALL stop the current transport or preview. RECORD SHALL hold microphone recording. These controls SHALL not answer a destructive question. Saved owner assignments SHALL take precedence over the starting roles.

#### Scenario: Button press and release use the current screen
- **WHEN** the Master port sends a press and release for each of CC 25 through 29 while the owner visits Pads, Files and Chop
- **THEN** each configured action occurs once per intended gesture and RECORD ends on its release
- **AND** no button answers a pending destructive question

#### Scenario: Owner bindings survive profile installation
- **WHEN** a saved SMC-PAD control has an owner assignment and the new profile loads
- **THEN** that assignment keeps its effect without a reset or migration

### Requirement: SMC-PAD raw replay uses production routing

The virtual MIDI replay command SHALL feed recorded bytes through the shipped profiles, production parser, port selection and App dispatch. It SHALL cover both SMC-PAD inputs and the X-Session Pro. Channel pressure SHALL run no action. The replay path SHALL not substitute a test-only action router.

#### Scenario: Replay covers the measured controller stream
- **WHEN** a replay sends pads, both encoder banks, button pairs, pressure and disconnect through the real input route
- **THEN** its action and gate trace matches the specified routes
- **AND** pressure runs no action

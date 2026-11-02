## ADDED Requirements

### Requirement: Documented SMC-PAD normal MIDI profile

The system SHALL ship a valid SMC-PAD profile for the documented normal Note and absolute
CC setup. It SHALL document the expected notes, channels, physical pad orientation,
knob modes, buttons and source limitations. It SHALL NOT claim all presets use that map.
The sixteen pads SHALL match the screen by physical position. Knobs 1–3 SHALL control
the current edit page or selected FX block. Left and Right SHALL select banks on Pads,
slices on Chop, pages in edit mode, and blocks on FX. Knob 4 SHALL select the page in
edit mode and the block on FX. Knob 8 SHALL control master volume. The second knob set
SHALL use the same roles with independent takeover identity. The local PAD BANK and
Shift buttons SHALL NOT acquire invented host actions.

#### Scenario: SMC-PAD physical grid
- **WHEN** the documented normal setup sends all sixteen notes 36–51 with varying velocity
- **THEN** each hit reaches the screen pad at the same physical grid position with that velocity

#### Scenario: SMC-PAD note release across screens
- **WHEN** the normal profile starts a gate pad, the screen or bank changes, and the pad releases
- **THEN** the original pad receives its release exactly once

#### Scenario: SMC-PAD absolute page knobs
- **WHEN** the normal profile sends absolute CC30–32, then selects another FX block and repeats the movement
- **THEN** only the selected block changes according to jump, pickup or scale

#### Scenario: SMC-PAD second knob set
- **WHEN** CC38–45 follows CC30–37 in the normal setup
- **THEN** the same logical roles are available with separate takeover for each physical control message

#### Scenario: SMC-PAD button release is not another press
- **WHEN** a normal navigation button sends CC127 and then CC0
- **THEN** the navigation runs once and the release does not change an unrelated parameter

#### Scenario: SMC-PAD unsupported messages stay harmless
- **WHEN** pressure, clock or unassigned second-set pad notes arrive
- **THEN** they do not change unrelated pad, FX or project settings

### Requirement: Documented X-Session physical controls

The system SHALL document X-Session Pro controls by their physical labels and retain
existing numeric control identifiers. The physical defaults SHALL follow change 82,
without default delete or commit buttons. The left three numbered knobs SHALL control
page parameters in edit mode and on FX. On FX, left backward and forward SHALL select the
previous and next block. Left Volume SHALL control selected pad volume, left Pitch pad
pitch, and right Volume master volume. Mirrored crossfader CC20 SHALL stay unassigned.
The new defaults SHALL NOT replace existing seeded or learned bindings without a
confirmed reset to the profile.

#### Scenario: X-Session faders use physical roles
- **WHEN** an initialized profile receives CC11, CC12 and CC14 movement
- **THEN** pad volume, pad pitch and master volume follow those controls respectively

#### Scenario: X-Session shared FX controls
- **WHEN** the FX screen receives CC24–26 and block button presses
- **THEN** the selected FX block uses the page trio and the other block stays unchanged

#### Scenario: Updated profiles preserve learned settings
- **WHEN** the new profiles load with an existing seeded device and learned assignments
- **THEN** those assignments stay unchanged until the owner confirms a reset

### Requirement: Controller MIDI emulation tests

The system SHALL test both shipped profiles with raw MIDI sequences without hardware.
The tests SHALL cover parsing, action resolution, profile initialization, dispatch,
application state, release, takeover, undo, persistence and simultaneous controllers.
Hardware feel and actual preset or port identity SHALL remain explicit manual checks.

#### Scenario: Emulated SMC-PAD and X-Session together
- **WHEN** SMC-PAD notes and X-Session parameter movements are interleaved with a screen change
- **THEN** pad velocity and release remain correct and only the intended parameter changes

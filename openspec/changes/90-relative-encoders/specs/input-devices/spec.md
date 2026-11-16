# input-devices Delta

## ADDED Requirements

### Requirement: Relative MIDI controls use signed steps

A device profile SHALL accept `type = "relative"` and an `encoding` on a CC knob. The supported encodings SHALL be `offset64`, `twos` and `signbit`. Each SHALL produce the signed steps in the change design. A neutral value SHALL do nothing. Controls without `type = "relative"` SHALL remain absolute.

#### Scenario: Offset64 direction and neutral
- **WHEN** a relative `offset64` knob sends raw CC values 65, 63 and 64
- **THEN** its target receives +1, -1 and no step in that order
- **AND** the neutral value does not acknowledge a notice or start a pending button hold

#### Scenario: Other relative encodings
- **WHEN** test profiles use `twos` and `signbit` with positive, negative and neutral raw CC bytes
- **THEN** each target receives the signed counts defined for that encoding
- **AND** a relative step does not use the absolute-knob response setting

### Requirement: Invalid relative controls keep default routing

An unknown encoding, a missing encoding or an incompatible binding SHALL refuse the profile. The diagnostic SHALL name its file and control. The device SHALL retain default routing.

#### Scenario: Invalid relative profile keeps default routing
- **WHEN** a profile gives a CC knob an unknown or absent relative encoding or binds it to an incompatible action
- **THEN** the program names the file and control in its diagnostic
- **AND** the device keeps default pad routing without a crash

### Requirement: Learned relative controls keep their physical type

A profile SHALL retain a relative knob's encoding after the program copies its starting roles into the user config. A learned compatible target SHALL override the starting target and use signed steps. An unnamed CC learned as a parameter SHALL stay absolute. An absolute knob SHALL keep its jump, pickup and scale behavior.

#### Scenario: Relative control keeps a learned role
- **WHEN** the owner learns a new parameter for a named relative knob after the program copies the profile roles
- **THEN** raw relative CC values move the new parameter by steps without pickup
- **AND** the old profile target does not move

#### Scenario: Absolute knob keeps its response
- **WHEN** an X-Session Pro absolute knob turns with jump, pickup or scale selected
- **THEN** it keeps the current position response and does not enter the relative step path

### Requirement: Relative speed and fine mode stay per device

The system SHALL count detents of one port, channel and CC in a 40 ms receipt-time window. It SHALL use multipliers 1, 2, 4 and 8 for one, two, three and at least four detents. A neutral CC SHALL not count as a detent. A disconnect SHALL clear speed history.

#### Scenario: Fast turn and fine button
- **WHEN** one knob sends four detents within 40 ms, then the owner holds a global fine button during four more detents
- **THEN** the first group grows by multipliers 1, 2, 4 and 8
- **AND** each detent under fine mode uses multiplier 1 until the button releases

### Requirement: A held button selects fine steps

A global `knob.fine` button action SHALL disable acceleration on its port while held. Its original release, an invalid-state release or a disconnect SHALL clear fine mode. Fine mode SHALL not change stored precision.

#### Scenario: Two devices keep separate step state
- **WHEN** two ports send the same CC and only the first port holds its fine button
- **THEN** the second port keeps its own speed and normal steps
- **AND** disconnecting the first port clears only its own fine and speed state

### Requirement: Relative steps keep input order

A relative step that changes a bank, page or selection SHALL reach the UI before a later pad hit in the same MIDI stream. A neutral step SHALL create no pending-input barrier. Existing direct pad releases SHALL keep their original route, target and project identity.

#### Scenario: Relative bank step precedes a pad hit
- **WHEN** a raw relative bank step and a pad hit arrive before one UI refresh
- **THEN** the hit uses the new bank and sounds once with its original velocity
- **AND** the original release still reaches that pad after a screen change

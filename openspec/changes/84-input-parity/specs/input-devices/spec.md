## ADDED Requirements

### Requirement: Controller pad parity on Chop

On the Chop screen, the system SHALL handle a controller pad hit as the same keyboard
pad hit in the current bank, using the existing listen or assign mode. Before slices
are linked to pads, both inputs SHALL use the same slice page and selection. With
linked slices, both SHALL play the actual pad and select its linked slice. The system
SHALL preserve MIDI hit velocity when it plays that pad. During assignment, both inputs
SHALL put the selected slice into the plan at the hit target and preview it, without
applying the plan or bypassing an overwrite question. A question or screen learn SHALL
keep pads silent and prevent plan changes as for the keyboard. Normal performance
MIDI pad hits outside the UI modes SHALL keep their direct audio route; the UI SHALL
NOT sound a hit a second time.

#### Scenario: Chop parity before pad assignment
- **WHEN** a chopped sample has no linked pads and the owner hits the same keyboard and
  controller pad on the same bank page in separate runs
- **THEN** the same slice is selected and previewed once in each run

#### Scenario: Chop parity with linked pads
- **WHEN** a pad has a linked slice, changed pitch and filter, and the owner hits it on
  Chop from the keyboard and controller with matching velocities in separate runs
- **THEN** the same pad sound plays once with its settings and the same slice is selected

#### Scenario: Chop MIDI linked pad keeps velocity
- **WHEN** the owner hits a linked pad on Chop from MIDI with low and high velocities
- **THEN** each hit plays once with its original velocity and the low hit is quieter

#### Scenario: Chop parity in by-hand assignment
- **WHEN** a by-hand plan is open and the owner hits the same targets in two banks from
  the keyboard and controller in separate runs
- **THEN** the plans and selected slices match after each hit, each planned slice is
  previewed once, and the plan is not applied by those hits

#### Scenario: Chop parity while a question is open
- **WHEN** an overwrite question is open on Chop and keyboard or controller pads are hit
- **THEN** no pad or preview sounds, the plan stays unchanged and the question stays open

#### Scenario: Chop parity while learning
- **WHEN** screen learn is open on Chop and keyboard or controller pads are hit
- **THEN** no pad or preview sounds and no assign plan changes

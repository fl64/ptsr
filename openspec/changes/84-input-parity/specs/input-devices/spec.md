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

### Requirement: Silent MIDI copy targeting

On Pads, the system SHALL route controller pad hits to the UI while a copy or its
replacement question is open. It SHALL select the same single target in the current
bank as a keyboard hit, without a direct audio hit or an extra UI sound. Empty targets,
occupied targets, cancellation and confirmation SHALL follow the existing Copy a pad
requirement. A new target hit SHALL NOT confirm replacement. A release of a pad held
before copying SHALL still release the original pad after a bank or screen change.
Normal performance hits outside these modes SHALL keep their direct audio route.

#### Scenario: Copy parity from raw MIDI
- **WHEN** a copy is in hand and the owner hits an empty target in another bank with
  a keyboard pad, a MIDI note or a learned pad button in separate runs
- **THEN** the same target is selected and receives the copy, the source stays unchanged
  and no hit sound is sent through either audio route

#### Scenario: Copy replacement parity from raw MIDI
- **WHEN** a copy is in hand and the owner hits an occupied target with a keyboard pad,
  a MIDI note or a learned pad button in separate runs
- **THEN** the same target is selected and named in the question without sound or replacement
- **AND** further pad hits stay silent and do not confirm; Esc keeps the copy in hand,
  and Enter after a new target hit replaces only the target named in that question

#### Scenario: Copy route preserves held MIDI releases
- **WHEN** a MIDI gate pad is held before a copy is taken, the bank and screen change
  and the original note or learned pad button is released
- **THEN** the original pad receives its release, with no hit or release of the new bank's pad

### Requirement: MIDI file and swap target parity

On Pads, a controller note or learned pad button used as a file-in-hand or swap target
SHALL select the same pad in the current bank as a keyboard hit. The target hit SHALL
NOT also sound. A successful swap SHALL make the same one-step undo for both pads as
the keyboard. Existing file loading, questions, confirmation commands, cancellation,
same-source swap and loading refusal SHALL stay unchanged.

#### Scenario: File target parity from raw MIDI
- **WHEN** a file is in hand and the owner hits an occupied target in another bank
  with a keyboard pad, raw MIDI note or learned pad button in separate runs
- **THEN** the same target is selected and named in the replacement question without sound
- **AND** Esc keeps the old target and file in hand without an undo step

#### Scenario: Swap target and undo parity from raw MIDI
- **WHEN** the owner takes a loaded pad with Ctrl+X and hits a target in another bank
  with a keyboard pad, raw MIDI note or learned pad button in separate runs
- **THEN** the same two pads swap without a hit sound, the target is selected and the hand clears
- **AND** one Ctrl+Z restores both pads; cancellation and loading refusal change no pad
  and add no undo step

### Requirement: Silent MIDI pad hits under a question

While a question is open, the system SHALL route new controller pad hits to the UI
instead of the direct audio queue. As for keyboard hits, they SHALL NOT sound, select
another pad, change marks or answer the question. Releases of pads held before the
question SHALL still reach their original pad after a bank or screen change. Closing
the question SHALL restore the normal direct MIDI pad route when no other UI-only
pad mode remains. A file or copy still in hand SHALL keep its silent targeting route.
This rule SHALL NOT change which commands confirm a question or silence normal marking.

#### Scenario: Question pad silence from raw MIDI
- **WHEN** the owner marks two loaded pads, opens their clear question and hits a pad
  in another bank with a keyboard pad, raw MIDI note or learned pad button in separate runs
- **THEN** neither audio route gets a new hit, the question and pad state stay unchanged,
  and Esc keeps the sounds and marks
- **AND** a new hit after Esc uses the normal performance route once

#### Scenario: Question route preserves held releases
- **WHEN** a gate pad is held, the bank and screen change, a replacement question
  opens and the owner releases the original key, note or learned pad button
- **THEN** only the original pad receives its release; the question stays open and no
  new-bank pad sounds or releases

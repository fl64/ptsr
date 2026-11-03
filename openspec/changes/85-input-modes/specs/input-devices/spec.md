## MODIFIED Requirements

### Requirement: Controller pads without delay

In normal performance, with no earlier controller command or UI-routed pad hit still
waiting for the UI, the system SHALL send a controller pad hit straight to the audio
engine, not through the screen refresh. It SHALL use the original hit velocity 1–127.
The existing UI-only routes for Chop, objects in hand and questions, and the silent
screen-learn route SHALL remain. The UI SHALL NOT sound an accepted direct hit again,
and SHALL NOT retry a direct hit rejected by the audio queue as a new UI hit.

While an earlier controller command or UI-routed hit is waiting for the UI, later
controller pad hits SHALL wait for that work instead of sounding with its old state.
This exception SHALL end when the UI has consumed that earlier work and published
its resulting input state. It SHALL NOT make ordinary performance depend on a screen
refresh when no such work is pending. A pending command SHALL NOT delay the existing
direct release of a pad already accepted by the audio queue or stop sound processing.

#### Scenario: Hit from a controller
- **WHEN** the owner hits a controller pad in normal performance, with no earlier
  controller command or UI-routed hit pending, while the screen is busy redrawing
- **THEN** the pad sounds in the nearest audio block, and the pads screen flashes and selects it
- **AND** later UI handling adds no second hit

#### Scenario: Direct MIDI resumes after pending input
- **WHEN** the UI has consumed all earlier controller commands and UI-routed pad hits,
  published normal performance with no object in hand or question, and a new pad is hit
- **THEN** the new hit reaches the next audio block without waiting for another UI update
- **AND** accepted hits sound once and direct queue rejection does not become a UI retry

## ADDED Requirements

### Requirement: Keyboard and MIDI keep separate Chop release targets

On Chop, the system SHALL keep a held keyboard pad's original absolute release
target separate from MIDI pad holds. A later MIDI hit at the same pad position
in another bank SHALL NOT replace that keyboard target, whether the audio queue
accepts or rejects the MIDI hit. A later keyboard hit SHALL NOT replace an accepted
MIDI hold's release target. Each original release SHALL reach only its own original
absolute pad after a bank or screen change, with a question open, or with F10 open.
A rejected MIDI hit SHALL acquire no audio hold. The other Chop behavior, velocity,
selection, marks and existing direct MIDI release rules SHALL remain unchanged.

#### Scenario: Chop MIDI hit keeps held keyboard release target
- **WHEN** a linked gate pad is held from the keyboard on Chop, the bank changes,
  and a raw MIDI note or learned pad button hits and holds the same pad position
- **THEN** both pads sound once with their original velocities, and releasing either
  original control stops only its own absolute pad while the other keeps sounding
- **AND** both release orders work on Chop, after a screen change, with a question
  open and with F10 open

#### Scenario: Chop rejected MIDI hit keeps held keyboard release target
- **WHEN** a keyboard gate is held on Chop, the bank changes, and the real audio
  queue rejects a raw-note or learned-button hit at the same pad position
- **THEN** the rejected hit starts no sound and acquires no audio hold; its release
  does not release the keyboard pad, whose original key still releases it
- **AND** both release orders work after a screen change, with a question open and
  with F10 open, without replaying the rejected hit after queue space returns

#### Scenario: Chop keyboard hit keeps held MIDI release target
- **WHEN** a linked gate pad is held on Chop from a raw MIDI note or learned button,
  the bank changes, and the keyboard hits and holds the same pad position
- **THEN** both original controls keep their own absolute release targets, and each
  release stops only its own pad while the other keeps sounding
- **AND** both release orders work on Chop, after a screen change, with a question
  open and with F10 open

### Requirement: Controller commands precede following pad hits

The system SHALL consume controller commands and following UI-routed pad hits in
the order accepted by its MIDI dispatcher. A pad hit that waits for earlier input
SHALL use the resulting bank and pad behavior when the UI reaches that hit, while
keeping the pad position resolved from its original control and its original velocity.
It SHALL NOT be interpreted a second time through a changed profile. This ordering
SHALL apply across connected controllers, not only within one device.

A command that takes a file SHALL make a following pad hit a silent file target,
even when both arrive before the next UI update. Empty targets, replacement questions,
cancellation and confirmation SHALL follow the existing file assignment rules. A
pending hit SHALL NOT confirm replacement. If a command has no effect or is blocked,
the following hit SHALL use the state that remains, not be dropped or act twice.
The keyboard SHALL keep its current command-then-pad behavior. Profile changes,
text input, pickers and F12 learn SHALL keep their existing input boundaries.

#### Scenario: Take a file and hit a target in one update
- **WHEN** Files has a selected file and the take-file command is followed by a pad hit
  without an intervening screen refresh, using keyboard events, raw MIDI notes or a
  learned pad button in separate runs, with controller input queued before UI handling
- **THEN** the same target is selected without a pad sound from either audio route
- **AND** an empty target receives the file, an occupied target asks before replacement,
  and Esc keeps the occupied pad and file in hand

#### Scenario: Pending pad uses preceding commands in order
- **WHEN** several controller commands are followed by a pad hit before a UI update,
  including a bank change, a cancelled hand state or a command that has no effect
- **THEN** the hit uses the bank and pad behavior left by those commands, as the same
  keyboard action sequence does, with its original velocity and no duplicate sound
- **AND** a command that has no effect does not leave later performance hits waiting

#### Scenario: A target question blocks a following pending hit
- **WHEN** taking a file is followed by hits on two occupied pads without an intervening
  screen refresh, from keyboard events, raw notes or learned pad buttons in separate
  runs, with the controller sequence queued before UI handling
- **THEN** the first hit selects its target and opens its replacement question
- **AND** the second hit stays silent, does not select another target and does not
  confirm or replace anything; Esc keeps the old sounds and the file in hand

#### Scenario: Pending command order spans controllers
- **WHEN** a take-file command from one controller is accepted before a pad hit from
  another controller and neither has reached the UI yet
- **THEN** that hit is a silent target for the file, not a performance hit
- **AND** pads already accepted directly by either device keep their original releases

### Requirement: Deferred MIDI hits keep their release identity

For a pad hit that waits for the UI and then starts an actual pad sound, the system
SHALL retain the original device, channel, control and project identity. Its original
release or device disconnect SHALL release only the absolute pad actually started,
even if the bank, screen, question or F10 state later changes. A release received
before the UI handles the hit SHALL still follow that hit and SHALL NOT leave a gate
sounding. Each accepted hold SHALL have its existing release behavior without an
extra release through the other audio route. The hit SHALL keep its original velocity.

A silent target, blocked hit, bare preview or audio-queue-rejected hit SHALL NOT acquire
a pad audio hold. A rejected retrigger SHALL NOT remove an older accepted hold. An
accepted UI hold's release SHALL wait for audio queue space, as on Chop. Old-project
hits and releases SHALL NOT affect the new project. Discarding old pending events
SHALL NOT leave later valid performance waiting forever. These rules SHALL preserve
the existing Chop and direct MIDI release behavior.

#### Scenario: Deferred gate receives an early release or disconnect
- **WHEN** a controller command is followed by a gate hit and its note-off, note-on zero,
  learned-button CC zero or device disconnect before the UI consumes the sequence,
  and the resulting mode permits that pad to sound
- **THEN** the original velocity reaches one pad hit and that accepted hold receives
  its original release, without a second hit, a duplicate release or a stuck gate

#### Scenario: Deferred release follows its actual pad across modes
- **WHEN** a deferred MIDI hit starts a gate after a preceding bank change and the
  owner then changes bank or screen, opens a question or F10, and releases its control
- **THEN** only the absolute pad actually started receives the release, as for the
  equivalent held keyboard pad
- **AND** other devices, channels and controls keep their own holds and release routes

#### Scenario: Deferred silent and rejected hits preserve accepted holds
- **WHEN** an older accepted gate is held and deferred controls target a file, are
  blocked, preview a bare slice or have their pad hits rejected by a full audio queue
- **THEN** those hits acquire no new pad audio hold and do not release the older hold;
  another control cannot acquire or release that older hold
- **AND** a rejected retrigger keeps its older accepted hold, whose own release waits
  for queue space instead of being lost

#### Scenario: Old deferred input cannot affect a new project
- **WHEN** the project changes with controller commands, deferred hits and releases
  still waiting, then the new project becomes ready and receives valid input
- **THEN** old input neither sounds nor changes nor releases pads in the new project
- **AND** consumed stale input does not keep the new project's normal direct MIDI route
  waiting, and new accepted holds keep their own release identity

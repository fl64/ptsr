# input-devices Specification

## Purpose
MIDI controllers: several devices at once with hotplug, TOML device profiles with a default profile, controller pads straight to the audio engine, soft takeover, and the F12 input debug tab.

## Requirements

### Requirement: Several MIDI devices at once
The system SHALL accept MIDI from all connected devices at once and support connecting and disconnecting without a restart and without a break in the sound.

#### Scenario: Two controllers
- **WHEN** a controller with pads (for example, SMC-PAD) and an X-Session Pro are connected
- **THEN** the pads of the first one play pads, the X-Session knobs change parameters, both work at the same time

#### Scenario: Controller pulled out during play
- **WHEN** a device is disconnected while pads are sounding
- **THEN** one-shot pads play to their end, a gate pad held by a button of that device fades out over its release as if the button were released, with no click; the status shows the disconnect; after the device is connected again its profile is applied again

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

### Requirement: Device profiles
The system SHALL describe each device with a TOML profile: controls (CC, notes, type) and
their starting logical actions per screen; a device without a profile works with the
default profile (notes 36–51 → pads 1–16). The starting actions are copied into the user
config the first time the device connects with no bindings there; from then on the user
config alone gives the device its actions, and the profile only names its controls.

#### Scenario: New controller
- **WHEN** a device without its own profile is connected
- **THEN** its notes 36–51 play pads, and the input debug screen shows what comes in, so that the owner can write a profile

#### Scenario: Profile with an error
- **WHEN** a profile file cannot be read (for example, it still has `??` from the template)
- **THEN** the program does not crash, the device works with the default profile, the status shows the file and the line with the error

#### Scenario: First connect
- **WHEN** the X-Session Pro connects and the user config has no bindings for it
- **THEN** the user config gets every role of its profile, left Cue (b7, note 58) opens the pad edit, and the
  MIDI tab shows these rows as the owner's

#### Scenario: Learned wins
- **WHEN** the user config has the X-Session Pro b3 learned for "next bank" from before
  this change and the program starts
- **THEN** b3 switches to the next bank, its new FX screen role is not added on that control,
  and other profile roles are added only if the device has not already been initialized

### Requirement: Soft takeover for absolute controls
The system SHALL drive a parameter from an absolute knob or fader the way the knob
response setting says, one setting for every absolute control of every device, kept in
the user config; the device profiles do not choose it. The setting has three values:

- `jump` (the default): the parameter goes to the control position at the first move;
- `pickup`: the parameter does not change until the control position crosses its current
  value, and the screen shows which way to move the control;
- `scale`: the parameter moves from the first move, in the direction of the move, by the
  share of its remaining room that the control covers of its own remaining travel, so the
  two meet at the end stop and the control follows the value from then on.

Every change glides, no click. The response starts over for a control when it drives a new
target (another pad, another page).

#### Scenario: Pad change
- **WHEN** the response is `jump`, the owner selects a pad with volume −12 dB, the volume
  knob stands at 0 dB and the owner turns it one step
- **THEN** the volume is at the knob position at once, no direction arrow is shown

#### Scenario: Pickup in the settings
- **WHEN** the response is `pickup`, the pad volume is −12 dB and the knob stands at 0 dB
- **THEN** turning the knob does not change the volume until it passes −12 dB; the screen
  shows a direction arrow

#### Scenario: Scale meets at the stop
- **WHEN** the response is `scale`, a parameter is at 20 % and the knob stands at 60 %, and
  the owner turns the knob up to its end in small steps
- **THEN** the parameter rises with the first step and with every step, never falls, and
  reaches 100 % with the knob

### Requirement: Input debug
The system SHALL have a tab on the settings screen (F12) that shows the latest input events: device, type, number, value and the logical action it became.

#### Scenario: Reading the layout
- **WHEN** the owner turns a knob on the X-Session Pro on the input debug screen
- **THEN** the owner sees the CC number, the value and "not assigned" or the action name

### Requirement: Edit mode knobs in profiles
The system SHALL bind knob actions `edit.param_1`, `edit.param_2`, `edit.param_3` in
the `[edit]` layer of a device profile: in an edit mode they turn the 1st, 2nd and 3rd
parameter of the current page, as the knob response says; outside it the knobs
do their screen roles from the profile. A slot with no parameter on the page does nothing.
An envelope time follows the knob on a square curve over 0–5000 ms, so the short times
near zero get most of the travel.

#### Scenario: X-Session Pro in the pad edit mode
- **WHEN** the pad edit mode is open and the owner turns the knob bound to `edit.param_1`
- **THEN** the first parameter of the current page follows the knob, and the screen says which one

#### Scenario: The ENV page on the knobs
- **WHEN** the pad edit mode shows the ENV page and the owner turns knob 2
- **THEN** with the response `jump` the decay of the selected pad goes to the knob position at once, and the panel
  lights it

#### Scenario: The knob keeps its role outside the edit
- **WHEN** no edit mode is open and the owner turns knob 1 on the pads screen
- **THEN** it changes the volume of the selected pad, as the screen section of the profile says

### Requirement: Edit layer on device buttons
The system SHALL let device buttons do what the edit keys do, so the whole edit loop runs
from a controller without the keyboard: a screen button opens the edit (`pad.edit`,
`chop.edit`, as `Enter`), and the `[edit]` layer of the profile, active while an edit mode
is open, binds closing it (`ui.back`), the page turn (`edit.page_prev`, `edit.page_next`,
as `Shift+Tab` `Tab`; the page knob `edit.page`) and the parameter pick (`edit.param_prev`, `edit.param_next`, as
`↑` `↓`). The layer is the same on every screen and stands over the screen roles and the
learned controls while the mode is open. The X-Session Pro profile uses left Cue
(b7, note 58) to open and close pad edit, right Cue (b8, note 59) for slice edit,
left backward / forward (b4 / b1, notes 46 / 43) for pages, and k1–k3 for page parameters.
It has no default page knob; the owner can bind one in F12. Play keeps its preview role.

#### Scenario: Edit a pad from the controller only
- **WHEN** a pad is selected and the owner presses left Cue, then left forward, turns knob 1 and presses left Cue
- **THEN** the edit mode opens, the ENV page is shown, the attack follows the knob, and
  the edit mode closes

#### Scenario: Pages from the controller
- **WHEN** the pad edit is open on TONE and the owner presses left forward twice and then left backward
- **THEN** the page is ENV, then FILTER, then ENV again, and knobs 1–3 turn the
  parameters of the page shown

#### Scenario: The buttons get their roles back
- **WHEN** the edit mode is closed on the chop screen and the owner presses left backward (b4)
- **THEN** the previous slice is selected, as before the edit

### Requirement: Chopping from the X-Session Pro
The system SHALL use physical controls for safe default chopping: either Play starts or
stops preview, left Cue cuts, left backward / forward selects slices, and right Cue opens
slice edit. Knobs 1 and 2 set slice bounds, knob 3 the cursor, knobs 4 and 5 the region,
and right Mid / Low the threshold / zoom. Default buttons SHALL NOT delete cuts, apply
layouts or confirm replacement. These actions SHALL remain available through keyboard
commands and explicit MIDI learn. Existing learned commit controls SHALL keep their role:
a commit starts the sequential plan, the next commit applies it and confirms overwrite.

#### Scenario: X-Session Pro only
- **WHEN** the owner has their hands only on the X-Session Pro
- **THEN** they can start the sample, cut, select slices, and move the cursor and region; layout uses the keyboard or an explicitly learned control

#### Scenario: Lay out from the controller
- **WHEN** a sample is cut into 8 slices, pad Q of bank 1 is selected and the owner presses a control explicitly learned as commit twice
- **THEN** slices 1–8 lie on Q W E R A S D F of bank 1; the `P` key still starts the assign by hand

### Requirement: Knob response setting
The system SHALL show the knob response on the MIDI tab of the settings screen (F12) as
one row with `jump`, `pickup` and `scale`, change it with the value keys, apply it at once
and keep it in the user config; a config without it reads as `jump`.

#### Scenario: Pick pickup
- **WHEN** the owner opens F12, goes to the MIDI tab, picks the knob response row and
  steps it to `pickup`
- **THEN** the next knob turn waits for the value, and after a restart the row still says
  `pickup`

#### Scenario: An older config
- **WHEN** the user config has no knob response
- **THEN** the row says `jump`

### Requirement: Every binding acts
The system SHALL bind keys and controls only to actions it has. A binding in the keymap,
the owner's keymap or a device profile that names an action the program does not have is
named on the status line at start with its file, key and name; the binding does nothing
and the rest loads. MIDI bindings in the owner's config that name no action are dropped
at start, the config is saved without them, and the status says how many went.

#### Scenario: The shipped bindings all act
- **WHEN** the program loads its keymap and the X-Session Pro profile
- **THEN** every binding names an action the program has; `Ctrl+K` is free and knobs
  k6–k10 of the chop screen are free

#### Scenario: An unknown action is named
- **WHEN** the owner's keymap binds `Ctrl+G` to `pad.flyaway`
- **THEN** the status names the file, `Ctrl+G` and `pad.flyaway`, and every other key works

#### Scenario: Dead bindings leave the config
- **WHEN** the owner's config binds X-Session `cc 29` on the chop screen to `slice.pitch`
- **THEN** at start the binding is dropped, the config no longer has it, the status says
  one binding to a missing action was dropped, and the next start says nothing

### Requirement: X-Session physical default map

The system SHALL ship the factory channel-1 map documented in the active change design.
It SHALL keep numeric identifiers and document physical labels. Left Headphone SHALL
open Files and right Headphone FX. Left backward / forward SHALL select pads on Pads,
files on Files, slices on Chop, pages in edit and blocks on FX. Right backward / forward
SHALL select banks. Left Cue SHALL open pad edit, hold a file on Files and cut on Chop.
Right Cue SHALL go back, except that it opens Chop from Files and slice edit from Chop.
Either Cue SHALL close an open edit. Either Play SHALL toggle preview on Files and Chop,
including inside edit, and SHALL be unassigned elsewhere. Left Volume CC11 and Pitch CC12
SHALL control selected pad volume and pitch; right Volume CC14 SHALL control master
volume on all screens. Right Pitch CC15 and crossfader CC17 / CC20 SHALL stay unassigned.
Cue SHALL use safe actions `pad.edit_only`, `chop.edit_only` and `browser.take_only`.
They SHALL NOT apply a pending plan, place a held file or answer a confirmation.
Existing keyboard and learned actions SHALL retain their behavior. FX SHALL inherit page
controls and retain prior user page assignments, even without an initialization marker.
Global control changes and clearing SHALL affect FX without stale screen copies.
A released command button SHALL NOT run its action again. Profile initialization SHALL
keep both default controls for a shared action, without replacing a pre-existing user
assignment. Explicit MIDI learn SHALL keep its existing replacement behavior.
Existing initialized maps
SHALL stay unchanged until a confirmed Reset to profile; cancel SHALL keep them.

#### Scenario: X-Session physical navigation and preview
- **WHEN** raw factory button messages arrive on Pads, Files, Chop, edit and FX
- **THEN** navigation, Cue, Headphone and Play follow the physical default map, and releases do nothing

#### Scenario: X-Session faders use physical roles
- **WHEN** raw CC11, CC12 and CC14 messages arrive on each screen
- **THEN** pad volume, pad pitch and master volume follow the controls; CC15, CC17 and CC20 do nothing

#### Scenario: X-Session safe default buttons
- **WHEN** every factory button is pressed in every screen and edit layer
- **THEN** no default resolves to cut deletion, layout application or overwrite confirmation

#### Scenario: X-Session replacement preserves existing assignments
- **WHEN** an initialized old map loads with the replacement profile, then reset is cancelled and later confirmed
- **THEN** the old map stays through loading and cancellation; only confirmation installs the new map

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

A linked gate pad started by the Chop UI route SHALL receive one audio release for its
original absolute pad when its original MIDI note or learned button is released, even
after a bank/screen change, question or learn opens. Disconnect SHALL also release that
pad. Release tracking SHALL keep device, channel, control and project epoch separate.
A bare preview, assignment target or blocked hit SHALL NOT create a pad audio release.
Original direct hits SHALL keep their existing release route without a duplicate UI
release. Stale project events SHALL NOT hit or release pads in the new project.
A queue-rejected UI hit SHALL NOT create a pad hold or remove an older accepted hold.
The system SHALL retain an accepted UI hold's release until the audio queue accepts it.

#### Scenario: Chop linked gate release reaches audio
- **WHEN** a linked gate pad is hit on Chop from a raw MIDI note or learned pad button
  and the original control is released without a bank or screen change
- **THEN** that pad receives one audio release, as for the keyboard, and its held state clears

#### Scenario: Chop linked gate release keeps original pad
- **WHEN** a linked gate pad is held on Chop, the bank and screen change, and the
  original note or learned pad button is released, or its device disconnects
- **THEN** only the original absolute pad receives its release and its held state clears
- **AND** a question or screen learn does not block that release

#### Scenario: Chop silent hits have no pad release
- **WHEN** a controller pad previews an unlinked slice, sets an assignment target or
  is blocked by a question or screen learn, then its control is released
- **THEN** no pad audio release is sent for that silent or preview-only hit
- **AND** an already held pad from another control keeps its original release route

#### Scenario: Chop release keeps direct route and epoch
- **WHEN** direct and UI-routed MIDI hits are released after route or project changes
- **THEN** accepted direct hits still receive one direct release, UI-started linked pads
  receive no duplicate release, and old-project events do not hit or release new-project pads

#### Scenario: Chop rejected hits preserve accepted gate release
- **WHEN** a UI-started gate pad is held, the UI audio queue fills, and retriggers from
  the same or another MIDI control are rejected before those controls release
- **THEN** the rejected hits acquire no new hold, the accepted hold still has its original
  release, and that one release waits for queue space instead of being lost

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

### Requirement: Held pad release through screen learn

The system SHALL pass the release of a keyboard pad held before F10 through screen
learn, as it does for an original MIDI note or learned pad button. It SHALL release
only the original absolute pad, including after a bank or screen change before learn
opens. New learn-mode hits and their releases SHALL NOT sound or release another
control's held pad. An original release SHALL NOT change the learn pick or pending
control, confirm a binding or leave learn. Existing binding and confirmation rules
SHALL stay unchanged.

#### Scenario: Held pad release passes screen learn
- **WHEN** a gate pad is held from the keyboard, a raw MIDI note or a learned pad button,
  the bank or screen changes, F10 opens learn and the original control is released
- **THEN** only the original absolute pad receives one audio release and its held state clears
- **AND** learn stays open with the same pick, pending control and saved bindings

#### Scenario: New learn keys do not release an older pad
- **WHEN** a gate pad is held before F10 and another pad key is pressed and released in learn
- **THEN** the new key sends no pad audio command, the older hold keeps its release route
  and only its original control releases it
- **AND** a silent keyboard key cannot release a MIDI hold of the same local pad

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

### Requirement: Late direct MIDI notifications do not start Chop actions

The system SHALL retain whether a MIDI pad hit was routed to the UI, accepted by
its direct audio queue, or rejected by that queue. If Chop opens before a direct
hit notification reaches App, that notification SHALL remain a report of the
original direct hit. It SHALL NOT start another pad sound, preview a bare slice,
select a Chop slice or change an assignment plan. A rejected direct hit SHALL NOT
be retried after queue space returns. Keep the original absolute pad and direct
release route. The existing Chop-entry hush SHALL remain. Actual UI-routed Chop
hits SHALL retain their current listen, preview, assignment and release behavior.
Other text, picker, question, learn and hand priorities SHALL remain unchanged.

#### Scenario: Late accepted direct MIDI notification does not restart Chop
- **WHEN** a raw note or learned pad button starts a direct pad on Pads, Chop opens
  from the keyboard before the hit notification is consumed, and the UI then
  consumes it in linked listen, bare-slice listen or assignment mode
- **THEN** that hit sounds once before Chop entry and keeps its original velocity;
  Chop entry hushes it without a later restart, preview or assignment action
- **AND** its original control retains the direct release of its original absolute
  pad, including after a bank change; no UI audio hold or duplicate release is added

#### Scenario: Late rejected direct MIDI notification stays silent on Chop
- **WHEN** the actual MIDI audio queue rejects a raw-note or learned-button hit on
  Pads, space returns, Chop opens before the hit notification is consumed, and the
  UI consumes it in linked listen, bare-slice listen or assignment mode
- **THEN** the rejected hit stays silent, no slice is selected or previewed, and
  no assignment action or UI audio hold results
- **AND** its release does not release another accepted hold or add a UI audio release

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
sounding. Each accepted hold SHALL keep its own release identity. An accepted deferred
hit is a new UI hold: an older direct release of the same absolute pad SHALL NOT
remove its release, even when audio processes that older release before UI delivery.
This does not repeat the older direct hold's release. Existing direct-first merging
for non-deferred Chop hits SHALL stay unchanged. The hit SHALL keep its original velocity.

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
- **AND** if an older direct hold of that same pad releases before UI delivery, the
  later accepted deferred hit still receives its own UI release; no old hold is released twice

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

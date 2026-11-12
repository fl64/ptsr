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

Without pending earlier controller commands or UI-routed hits, normal controller pad hits SHALL reach audio directly, independent of screen refresh.
They SHALL keep velocity 1–127.
Chop, held objects and questions SHALL keep their UI routes.
Screen learn SHALL stay silent.
The UI SHALL NOT repeat an accepted direct hit or retry an audio-queue-rejected direct hit as a new UI hit.

#### Scenario: Hit from a controller
- **WHEN** the owner hits a controller pad in normal performance while the screen is busy redrawing
- **AND** no earlier controller command or UI-routed hit is pending
- **THEN** the pad sounds in the nearest audio block, and the pads screen flashes and selects it
- **AND** later UI handling adds no second hit

### Requirement: Pending MIDI work delays later hits only

Later controller pad hits SHALL wait for earlier controller commands or UI-routed hits, without sounding under old state.
Direct performance SHALL resume after UI consumption and publication of the resulting input state.
Pending commands SHALL NOT delay existing direct releases of audio-accepted pads or stop sound processing.

#### Scenario: Direct MIDI resumes after pending input
- **WHEN** the UI consumes all earlier controller commands and UI-routed pad hits
- **AND** it publishes normal performance with no object in hand or question
- **AND** the owner hits a new pad
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
One user-config setting SHALL control all devices' absolute knobs and faders, not device profiles: `jump`, `pickup` or `scale`.
Default `jump` SHALL set the parameter to the control position on its first move.
Every change SHALL glide without a click.
A new target (pad or page) SHALL restart that control's response.

#### Scenario: Pad change
- **WHEN** the response is `jump`, the owner selects a pad with volume −12 dB, the volume
  knob stands at 0 dB and the owner turns it one step
- **THEN** the volume is at the knob position at once, no direction arrow is shown

### Requirement: Pickup response

With `pickup`, the parameter SHALL stay unchanged until the control crosses its current value.
The screen SHALL show which direction to move the control.

#### Scenario: Pickup in the settings
- **WHEN** the response is `pickup`, the pad volume is −12 dB and the knob stands at 0 dB
- **THEN** turning the knob does not change the volume until it passes −12 dB; the screen
  shows a direction arrow

### Requirement: Scale response

With `scale`, the parameter SHALL move from the first move in that direction.
It SHALL cover the same share of its remaining range as the control covers of its remaining travel.
Both SHALL meet at the end stop, after which the control SHALL follow the value.

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
Device buttons SHALL support the full edit loop without keys.
Screen buttons SHALL open edit (`pad.edit`, `chop.edit`, as `Enter`).
While edit is open, `[edit]` SHALL override screen and learned roles consistently across screens:
- `ui.back`: close.
- `edit.page_prev`, `edit.page_next`: pages, as `Shift+Tab`, `Tab`.
- `edit.page`: page knob.
- `edit.param_prev`, `edit.param_next`: parameters, as `↑`, `↓`.

#### Scenario: Edit a pad from the controller only
- **WHEN** a pad is selected and the owner presses left Cue, then left forward, turns knob 1 and presses left Cue
- **THEN** the edit mode opens, the ENV page is shown, the attack follows the knob, and
  the edit mode closes

### Requirement: X-Session edit controls

The X-Session Pro SHALL use left Cue (b7, note 58) to open/close pad edit and right Cue (b8, note 59) for slice edit.
Left backward/forward (b4/b1, notes 46/43) SHALL change pages, with k1–k3 for page parameters.
No page knob SHALL be assigned by default.
F12 SHALL allow one.
Play SHALL keep its preview role.

#### Scenario: Pages from the controller
- **WHEN** the pad edit is open on TONE and the owner presses left forward twice and then left backward
- **THEN** the page is ENV, then FILTER, then ENV again, and knobs 1–3 turn the
  parameters of the page shown

#### Scenario: The buttons get their roles back
- **WHEN** the edit mode is closed on the chop screen and the owner presses left backward (b4)
- **THEN** the previous slice is selected, as before the edit

### Requirement: Chopping from the X-Session Pro
Default Chop controls SHALL use either Play for preview, left Cue for cuts, left backward/forward for slices and right Cue for slice edit.
Knobs 1/2 SHALL set slice bounds, 3 cursor, 4/5 region, and right Mid/Low threshold/zoom.
Default buttons SHALL NOT delete cuts, apply layouts or confirm replacement.
Those actions SHALL remain available through keys and explicit MIDI learn.

#### Scenario: X-Session Pro only
- **WHEN** the owner has their hands only on the X-Session Pro
- **THEN** they can start the sample, cut, select slices, and move the cursor and region; layout uses the keyboard or an explicitly learned control

### Requirement: Learned Chop commit retains its role

Existing learned commit controls SHALL start the sequential plan, then apply it and confirm overwrite on the next commit.

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

The factory channel-1 map SHALL keep numeric IDs and documented physical labels.
Left/right Headphone SHALL open Files/FX.
Left backward/forward SHALL select Pads pads, Files files, Chop slices, edit pages or FX blocks.
Right backward/forward SHALL select banks.
Either Play SHALL toggle preview on Files/Chop, including edit, and remain unassigned elsewhere.
Command releases SHALL NOT act.

#### Scenario: X-Session physical navigation and preview
- **WHEN** raw factory button messages arrive on Pads, Files, Chop, edit and FX
- **THEN** navigation, Cue, Headphone and Play follow the physical default map, and releases do nothing

### Requirement: X-Session physical faders

Left Volume CC11 and Pitch CC12 SHALL control selected pad volume and pitch.
Right Volume CC14 SHALL control master volume on all screens.
Right Pitch CC15 and crossfader CC17/CC20 SHALL stay unassigned.

#### Scenario: X-Session faders use physical roles
- **WHEN** raw CC11, CC12 and CC14 messages arrive on each screen
- **THEN** pad volume, pad pitch and master volume follow the controls; CC15, CC17 and CC20 do nothing

### Requirement: X-Session safe Cue actions

Left Cue SHALL open pad edit, hold a file on Files or cut on Chop.
Right Cue SHALL go back, except Files→Chop and Chop→slice edit.
Either Cue SHALL close open edit.
Cue SHALL use `pad.edit_only`, `chop.edit_only` and `browser.take_only`, without applying plans, placing held files or answering confirmations.
Existing keyboard and learned actions SHALL retain their behavior.

#### Scenario: X-Session safe default buttons
- **WHEN** every factory button is pressed in every screen and edit layer
- **THEN** no default resolves to cut deletion, layout application or overwrite confirmation

### Requirement: X-Session profile initialization

FX SHALL inherit page controls, keep prior page assignments without an initialization marker, and reflect global changes/clearing without stale copies.
Profile initialization SHALL keep both defaults for a shared action without replacing owner assignments.
Explicit MIDI learn SHALL keep its replacement behavior.
Initialized maps SHALL change only on confirmed Reset to profile.
Cancellation SHALL keep them.

#### Scenario: X-Session replacement preserves existing assignments
- **WHEN** an initialized old map loads with the replacement profile, then reset is cancelled and later confirmed
- **THEN** the old map stays through loading and cancellation; only confirmation installs the new map

### Requirement: Controller pad parity on Chop

A linked gate pad started through the Chop UI SHALL receive one release for its original absolute pad on original-note/button release or disconnect.
This SHALL hold after bank/screen changes, questions or learn.
Tracking SHALL separate device, channel, control and project epoch.

#### Scenario: Chop linked gate release reaches audio
- **WHEN** a linked gate pad is hit on Chop from a raw MIDI note or learned pad button
  and the original control is released without a bank or screen change
- **THEN** that pad receives one audio release, as for the keyboard, and its held state clears

#### Scenario: Chop linked gate release keeps original pad
- **WHEN** a linked gate pad is held on Chop, the bank and screen change, and the
  original note or learned pad button is released, or its device disconnects
- **THEN** only the original absolute pad receives its release and its held state clears
- **AND** a question or screen learn does not block that release

### Requirement: Chop silent MIDI hits have no release

Bare previews, assignment targets and blocked hits SHALL NOT create pad audio releases.
Normal performance MIDI hits outside UI modes SHALL keep the direct audio route without a second UI sound.

#### Scenario: Chop silent hits have no pad release
- **WHEN** a controller pad previews an unlinked slice, sets an assignment target or
  is blocked by a question or screen learn, then its control is released
- **THEN** no pad audio release is sent for that silent or preview-only hit
- **AND** an already held pad from another control keeps its original release route

### Requirement: Chop direct releases and project identity

Original direct hits SHALL keep their release route without duplicate UI releases.
Stale project events SHALL NOT hit or release pads in a new project.

#### Scenario: Chop release keeps direct route and epoch
- **WHEN** direct and UI-routed MIDI hits are released after route or project changes
- **THEN** accepted direct hits still receive one direct release, UI-started linked pads
  receive no duplicate release, and old-project events do not hit or release new-project pads

### Requirement: Chop UI audio queue preserves holds

A queue-rejected UI hit SHALL NOT create a pad hold or remove an older accepted hold.
An accepted UI hold's release SHALL remain pending until the audio queue accepts it.

#### Scenario: Chop rejected hits preserve accepted gate release
- **WHEN** a UI-started gate pad is held, the UI audio queue fills, and retriggers from
  the same or another MIDI control are rejected before those controls release
- **THEN** the rejected hits acquire no new hold, the accepted hold still has its original
  release, and that one release waits for queue space instead of being lost

### Requirement: Chop MIDI and keyboard listen parity

On Chop, controller and keyboard pad hits SHALL use the same current bank and existing listen or assign mode.
Before links exist, both SHALL use the same slice page and selection.
With links, both SHALL play the actual pad and select its linked slice.
MIDI SHALL retain original hit velocity when playing that pad.

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

### Requirement: Chop MIDI and keyboard assignment parity

In assignment, controller and keyboard pad hits SHALL plan the selected slice at the hit target and preview it without applying or bypassing overwrite questions.
Questions and screen learn SHALL keep pads silent and prevent plan changes as for keys.

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

A keyboard pad held before F10 SHALL release through screen learn as original MIDI notes and learned pad buttons do.
Only its original absolute pad SHALL release, even after bank/screen changes before learn.
New learn hits/releases SHALL NOT sound or release another control's hold.
Original releases SHALL NOT change learn selection or pending control, bind or leave learn.
Existing binding and confirmation SHALL stay unchanged.

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

On Pads, controller hits during copy or its replacement question SHALL use the UI.
They SHALL select the keyboard's same single current-bank target, silently on both routes.
Empty/occupied targets, cancellation and confirmation SHALL follow "Copy a pad".
New target hits SHALL NOT confirm replacement.
Normal performance outside these modes SHALL keep its direct route.

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

### Requirement: Copy targeting keeps held releases

A pad held before copying SHALL still release its original absolute pad after a bank or screen change.

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

During questions, new controller pad hits SHALL use the UI, not direct audio.
As for keys, they SHALL NOT sound, select pads, change marks or answer.
Closing SHALL restore direct MIDI when no UI-only mode remains.
A held file/copy SHALL keep silent targeting.
Confirmation commands and normal marking sound SHALL stay unchanged.

#### Scenario: Question pad silence from raw MIDI
- **WHEN** the owner marks two loaded pads, opens their clear question and hits a pad
  in another bank with a keyboard pad, raw MIDI note or learned pad button in separate runs
- **THEN** neither audio route gets a new hit, the question and pad state stay unchanged,
  and Esc keeps the sounds and marks
- **AND** a new hit after Esc uses the normal performance route once

### Requirement: Questions keep held releases

Pads held before a question SHALL still release their original pad after a bank or screen change.

#### Scenario: Question route preserves held releases
- **WHEN** a gate pad is held, the bank and screen change, a replacement question
  opens and the owner releases the original key, note or learned pad button
- **THEN** only the original pad receives its release; the question stays open and no
  new-bank pad sounds or releases

### Requirement: Late direct MIDI notifications do not start Chop actions

The system SHALL retain each MIDI hit's UI route or direct-queue acceptance/rejection.
After Chop opens, a late direct notification SHALL remain a report of the original hit.
It SHALL NOT add pad sound, bare preview, slice selection or plan changes.
The original absolute pad, direct release route and Chop-entry hush SHALL remain.

#### Scenario: Late accepted direct MIDI notification does not restart Chop
- **WHEN** a raw note or learned pad button starts a direct pad on Pads
- **AND** Chop opens from the keyboard before the UI consumes the hit notification
- **AND** the UI then consumes it in linked listen, bare-slice listen or assignment mode
- **THEN** that hit sounds once before Chop entry and keeps its original velocity
- **AND** Chop entry hushes it without a later restart, preview or assignment action
- **AND** its original control retains the direct release of its original absolute pad, including after a bank change
- **AND** no UI audio hold or duplicate release is added

### Requirement: Late rejected MIDI hits stay rejected

Rejected direct hits SHALL NOT retry after queue space returns.
Actual UI-routed Chop hits SHALL keep listen, preview, assignment and release behavior.
Text, picker, question, learn and hand priorities SHALL stay unchanged.

#### Scenario: Late rejected direct MIDI notification stays silent on Chop
- **WHEN** the actual MIDI audio queue rejects a raw-note or learned-button hit on Pads
- **AND** space returns and Chop opens before the UI consumes the hit notification
- **AND** the UI consumes it in linked listen, bare-slice listen or assignment mode
- **THEN** the rejected hit stays silent, no slice is selected or previewed, and no assignment action or UI audio hold results
- **AND** its release does not release another accepted hold or add a UI audio release

### Requirement: Keyboard and MIDI keep separate Chop release targets

On Chop, keyboard and MIDI holds SHALL keep separate original absolute release targets.
A later MIDI hit at the same position in another bank SHALL NOT replace the keyboard target, accepted or rejected.
A later keyboard hit SHALL NOT replace an accepted MIDI target.
Each original release SHALL reach only its own original pad, including after bank/screen changes and during questions or F10.

#### Scenario: Chop MIDI hit keeps held keyboard release target
- **WHEN** the keyboard holds a linked gate pad on Chop and the bank changes
- **AND** a raw MIDI note or learned pad button hits and holds the same pad position
- **THEN** both pads sound once with their original velocities
- **AND** releasing either original control stops only its own absolute pad while the other keeps sounding
- **AND** both release orders work on Chop, after a screen change, with a question open and with F10 open

### Requirement: Rejected MIDI hits acquire no Chop hold

Rejected MIDI hits SHALL acquire no audio hold.
Other Chop behavior, velocity, selection, marks and existing direct MIDI releases SHALL stay unchanged.

#### Scenario: Chop rejected MIDI hit keeps held keyboard release target
- **WHEN** the keyboard holds a gate pad on Chop and the bank changes
- **AND** the real audio queue rejects a raw-note or learned-button hit at the same pad position
- **THEN** the rejected hit starts no sound and acquires no audio hold
- **AND** its release does not release the keyboard pad, whose original key still releases it
- **AND** both release orders work after a screen change, with a question open and with F10 open
- **AND** the rejected hit does not replay after queue space returns

#### Scenario: Chop keyboard hit keeps held MIDI release target
- **WHEN** a raw MIDI note or learned button holds a linked gate pad on Chop and the bank changes
- **AND** the keyboard hits and holds the same pad position
- **THEN** both original controls keep their own absolute release targets
- **AND** each release stops only its own pad while the other keeps sounding
- **AND** both release orders work on Chop, after a screen change, with a question open and with F10 open

### Requirement: Controller commands precede following pad hits

The UI SHALL consume controller commands and following UI-routed hits in MIDI-dispatcher acceptance order.
Each waiting hit SHALL use the resulting bank and pad behavior, keeping its original resolved pad position and velocity.
It SHALL NOT be reinterpreted through a changed profile.
Keyboard command-then-pad behavior SHALL stay unchanged.

#### Scenario: Take a file and hit a target in one update
- **WHEN** Files has a selected file and the take-file command is followed by a pad hit without an intervening screen refresh
- **AND** separate runs use keyboard events, raw MIDI notes or a learned pad button
- **AND** controller input is queued before UI handling
- **THEN** the same target is selected without a pad sound from either audio route
- **AND** an empty target receives the file and an occupied target asks before replacement
- **AND** Esc keeps the occupied pad and file in hand

### Requirement: Blocked controller commands preserve following hits

After a blocked or ineffective command, the next hit SHALL use the remaining state without being dropped or acting twice.
Profile changes, text input, pickers and F12 learn SHALL keep their existing input boundaries.

#### Scenario: Pending pad uses preceding commands in order
- **WHEN** several controller commands are followed by a pad hit before a UI update
- **AND** those commands include a bank change, a cancelled hand state or a command that has no effect
- **THEN** the hit uses the bank and pad behavior left by those commands, as the same keyboard action sequence does
- **AND** it keeps its original velocity without duplicate sound
- **AND** a command that has no effect does not leave later performance hits waiting

### Requirement: Pending file targets stay silent

A take-file command SHALL make a following hit a silent file target, even before the next UI update.
Empty targets, replacement questions, cancellation and confirmation SHALL follow existing file assignment rules.
Pending hits SHALL NOT confirm replacement.

#### Scenario: A target question blocks a following pending hit
- **WHEN** taking a file is followed by hits on two occupied pads without an intervening screen refresh
- **AND** separate runs use keyboard events, raw notes or learned pad buttons
- **AND** the controller sequence is queued before UI handling
- **THEN** the first hit selects its target and opens its replacement question
- **AND** the second hit stays silent, does not select another target and does not confirm or replace anything
- **AND** Esc keeps the old sounds and the file in hand

### Requirement: Pending MIDI order spans devices

MIDI-dispatcher acceptance order SHALL apply across connected controllers, not only within one device.

#### Scenario: Pending command order spans controllers
- **WHEN** one controller's take-file command is accepted before another controller's pad hit and neither has reached the UI yet
- **THEN** that hit is a silent target for the file, not a performance hit
- **AND** pads already accepted directly by either device keep their original releases

### Requirement: Deferred MIDI hits keep their release identity

A deferred hit that starts a pad SHALL keep its original device, channel, control and project release identity and velocity.
Its original release or disconnect SHALL release only the absolute pad started, despite later bank, screen, question or F10 changes.
A release received before UI hit handling SHALL still follow that hit without leaving a gate sounding.

#### Scenario: Deferred gate receives an early release or disconnect
- **WHEN** a controller command is followed by a gate hit and its release before the UI consumes the sequence
- **AND** that release is note-off, note-on zero, learned-button CC zero or device disconnect
- **AND** the resulting mode permits that pad to sound
- **THEN** the original velocity reaches one pad hit and that accepted hold receives its original release
- **AND** no second hit, duplicate release or stuck gate occurs
- **AND** if an older direct hold of that same pad releases before UI delivery, the later deferred hit keeps its own UI release
- **AND** no old hold is released twice

### Requirement: Deferred holds stay independent of older direct holds

Each accepted deferred hit SHALL create its own UI hold.
An older direct release of the same pad SHALL NOT remove its release, even if audio processes that release before UI delivery.
It SHALL NOT repeat the old hold's release.
Direct-first merging for non-deferred Chop hits and existing Chop/direct MIDI releases SHALL stay unchanged.

#### Scenario: Deferred release follows its actual pad across modes
- **WHEN** a deferred MIDI hit starts a gate after a preceding bank change
- **AND** the owner then changes bank or screen, opens a question or F10, and releases its control
- **THEN** only the absolute pad actually started receives the release, as for the equivalent held keyboard pad
- **AND** other devices, channels and controls keep their own holds and release routes

### Requirement: Deferred silent hits and queue pressure

Silent targets, blocked hits, bare previews and audio-queue-rejected hits SHALL NOT acquire pad audio holds.
Rejected retriggers SHALL NOT remove older accepted holds.
Accepted UI releases SHALL wait for audio queue space, as on Chop.

#### Scenario: Deferred silent and rejected hits preserve accepted holds
- **WHEN** an older accepted gate is held
- **AND** deferred controls target a file, are blocked, preview a bare slice or have their pad hits rejected by a full audio queue
- **THEN** those hits acquire no new pad audio hold and do not release the older hold
- **AND** another control cannot acquire or release that older hold
- **AND** a rejected retrigger keeps its older accepted hold, whose own release waits for queue space instead of being lost

### Requirement: Deferred project identity

Old-project hits and releases SHALL NOT affect a new project.
Discarding stale pending events SHALL NOT leave later valid performance waiting forever.

#### Scenario: Old deferred input cannot affect a new project
- **WHEN** the project changes with controller commands, deferred hits and releases still waiting
- **AND** the new project then becomes ready and receives valid input
- **THEN** old input neither sounds nor changes nor releases pads in the new project
- **AND** consumed stale input does not keep the new project's normal direct MIDI route waiting
- **AND** new accepted holds keep their own release identity

### Requirement: Screen bindings keep their existing identity

Keyboard/MIDI sections SHALL stay `pads`, `settings`, `files`, `chop`, `fx`, with global/edit scope and priority unchanged.
Owner overrides, explicit `none` and learned MIDI assignments SHALL keep their effects without reset or migration.
Defaults SHALL stay F2 Pads, F12 Settings, F6 Files, F3 Chop, F4 FX.
Screen changes SHALL preserve held-pad releases, input order and audio routes.

#### Scenario: Existing screen bindings survive the registry change
- **WHEN** each current screen receives recorded keyboard events and equivalent raw MIDI messages through existing profiles and learned assignments
- **THEN** each action keeps its existing screen, edit or global scope and priority
- **AND** owner key overrides, explicit `none` entries and saved MIDI assignments need no reset or conversion
- **AND** default screen keys, pad selection, hit velocity and original held-pad releases keep their existing behavior

### Requirement: FX keeps inherited controls without another edit mode

FX block, parameter and value controls SHALL need no separate edit mode.
Page controls and earlier owner assignments SHALL keep FX roles and priority.
Explicit FX assignments SHALL keep priority over inherited controls.
Saved empty groups SHALL NOT restore inheritance.
Global changes/clearing SHALL affect FX without stale copies.
No new edit layer or changed safe default buttons SHALL be added.

#### Scenario: FX controls keep their priority through the registry
- **WHEN** FX receives its existing block, parameter and value commands from keys or raw MIDI in separate runs
- **THEN** the same FX block, parameter and value change without a separate edit mode
- **AND** explicit FX assignments, inherited page controls, global changes and clearing keep their existing priority and effects
- **AND** an empty saved FX group does not restore inherited bindings

### Requirement: Invalid MIDI input state cannot select a fallback screen

Unknown screen codes or other invalid shared MIDI encodings SHALL reject new pad hits, commands and parameter messages without sound, action or value changes.
The system SHALL NOT substitute Pads, another screen or earlier valid state.
It SHALL log one error outside audio processing per continuous invalid period, not per message.

#### Scenario: Unknown MIDI screen rejects new input
- **WHEN** an invalid shared screen code reaches MIDI dispatch before raw pad, command and parameter messages
- **THEN** none of those messages starts sound, changes selection, runs a command or changes a parameter
- **AND** one error diagnostic identifies the invalid state, without a fallback screen or repeated logging for that continuous error

#### Scenario: Other invalid MIDI state has no fallback
- **WHEN** a shared state with an invalid bank or reserved bits reaches MIDI dispatch before raw pad, command and parameter messages
- **THEN** those messages have no sound or state effect and create no hold or new pending UI work
- **AND** the system records the invalid-state diagnostic without substituting a screen or bank

### Requirement: Invalid MIDI state keeps original releases

Existing holds SHALL release/disconnect through original routes to original absolute pads, with device, channel, control and project separate.
Untracked releases SHALL NOT release fallback-screen/bank pads.
Queued retries SHALL follow existing audio queue rules.
Invalid state SHALL NOT stop audio, create holds or falsely acknowledge queued UI work.

#### Scenario: Invalid MIDI state preserves original releases
- **WHEN** direct and UI-routed gate holds exist before shared state becomes invalid and their original controls release or disconnect
- **THEN** each accepted hold keeps its original absolute release target and route, including when a full audio queue delays release
- **AND** unrelated holds keep sounding, untracked releases cause no pad release and old-project releases do not affect a new project
- **AND** note-off, note-on zero and learned-button release retain their existing meanings

### Requirement: Valid MIDI input resumes without replay

After valid state arrives, new messages SHALL use it and existing pending-input ordering.
Rejected messages SHALL NOT replay or leave input waiting forever.

#### Scenario: Valid MIDI state resumes without rejected input
- **WHEN** the UI publishes valid state after dispatch rejected messages during an invalid period and then new raw MIDI input arrives
- **THEN** only the new messages act under the valid state and the existing pending-input ordering rules
- **AND** earlier queued UI work still needs its real acknowledgement, rejected messages never replay and rejection creates no lasting input barrier
- **AND** another invalid period produces its own error diagnostic

### Requirement: Working profiles exclude examples

The profile loader SHALL read only files with the `.toml` extension.
It SHALL ignore unfinished `.toml.example` files without a profile or error diagnostic.
The shipped SMC-PAD template SHALL use `.toml.example` until measured hardware data defines a working profile.
Invalid actual `.toml` files SHALL retain their file-and-line diagnostics and default device routing.

#### Scenario: Shipped profiles load without template errors
- **WHEN** the program loads the actual shipped device directory
- **THEN** all working profiles load without excluded errors
- **AND** the unfinished SMC-PAD example creates no profile or error diagnostic

#### Scenario: An example does not replace a working profile
- **WHEN** a profile directory contains a valid `.toml` and an unfinished `.toml.example`
- **THEN** the valid profile keeps its actions and the example adds no diagnostic or assignments
- **AND** a device without a valid matching profile keeps default notes 36 through 51

#### Scenario: Invalid working profiles retain default routing
- **WHEN** an actual `.toml` profile contains an invalid value
- **THEN** its diagnostic names the file and line
- **AND** its device still uses default routing without a crash

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

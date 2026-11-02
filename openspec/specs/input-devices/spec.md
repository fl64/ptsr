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
The system SHALL send a pad hit from a controller straight to the audio engine, not through the screen refresh, and SHALL use the hit velocity 1–127.

#### Scenario: Hit from a controller
- **WHEN** the owner hits a controller pad while the screen is busy redrawing
- **THEN** the pad sounds in the nearest audio block, and the pads screen flashes and selects it

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

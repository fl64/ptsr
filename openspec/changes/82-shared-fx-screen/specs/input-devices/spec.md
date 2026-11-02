## MODIFIED Requirements

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

## ADDED Requirements

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

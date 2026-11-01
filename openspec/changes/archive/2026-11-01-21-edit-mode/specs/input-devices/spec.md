## ADDED Requirements

### Requirement: Edit mode knobs in profiles
The system SHALL bind knob actions `edit.param_1`, `edit.param_2`, `edit.param_3` in
the `[edit]` layer of a device profile: in an edit mode they turn the 1st, 2nd and 3rd
parameter of the current page with soft takeover; outside it the knobs do their screen
roles from the profile. A slot with no parameter on the page does nothing. An envelope
time follows the knob on a square curve over 0–5000 ms, so the short times near zero get
most of the travel.

#### Scenario: X-Session Pro in the pad edit mode
- **WHEN** the pad edit mode is open and the owner turns the knob bound to `edit.param_1`
- **THEN** the first parameter of the current page follows the knob, and the screen says which one

#### Scenario: The ENV page on the knobs
- **WHEN** the pad edit mode shows the ENV page and the owner turns knob 2
- **THEN** the decay of the selected pad follows the knob after the pickup, and the panel lights it

#### Scenario: The knob keeps its role outside the edit
- **WHEN** no edit mode is open and the owner turns knob 1 on the pads screen
- **THEN** it changes the volume of the selected pad, as the screen section of the profile says

### Requirement: Edit layer on device buttons
The system SHALL let device buttons do what the edit keys do, so the whole edit loop runs
from a controller without the keyboard: a screen button opens the edit (`pad.edit`,
`chop.edit`, as `Enter`), and the `[edit]` layer of the profile, active while an edit mode
is open, binds closing it (`ui.back`) and the parameter pick (`edit.param_prev`,
`edit.param_next`, as `↑` `↓`). The layer is the same on every screen and stands over the
screen roles and the learned controls while the mode is open. The X-Session Pro profile
puts the toggle on b10, the parameter pick on b3 and b4, the page on k1–k3.

#### Scenario: Edit a pad from the controller only
- **WHEN** a pad is selected and the owner presses b10, then b4, turns knob 1 and presses b10
- **THEN** the edit mode opens, the next parameter is picked, the first parameter of its
  page follows the knob, and the edit mode closes

#### Scenario: The buttons get their roles back
- **WHEN** the edit mode is closed on the chop screen and the owner presses b3
- **THEN** the previous slice is selected, as before the edit

### Requirement: Chopping from the X-Session Pro
The system SHALL perform the chopping actions from the X-Session Pro according to its
device profile: play and stop, cut, walk the slices, delete a cut, undo, snap, the auto
mode and commit — the slice bounds on knobs 1 and 2, the cursor on knob 3, the region
start and end on knobs 4 and 5. The commit with no plan starts the assign in order from
the selected pad (slice 1 there, the rest after it); the commit again applies it, and it
confirms the overwrite question too — no pad hit is needed.

#### Scenario: X-Session Pro only
- **WHEN** the owner has their hands only on the X-Session Pro
- **THEN** they can start the sample, cut, move the cursor and the region, and lay the slices out on pads

#### Scenario: Lay out from the controller
- **WHEN** a sample is cut into 8 slices, pad Q of bank 1 is selected and the owner presses the commit button twice
- **THEN** slices 1–8 lie on Q W E R A S D F of bank 1; the `P` key still starts the assign by hand

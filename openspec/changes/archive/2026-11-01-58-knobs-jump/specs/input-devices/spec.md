## MODIFIED Requirements

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

## ADDED Requirements

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

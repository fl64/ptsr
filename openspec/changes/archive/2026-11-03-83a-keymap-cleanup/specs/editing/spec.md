## MODIFIED Requirements

### Requirement: Key roles

The system SHALL split work between the hands: left fingers play pads; the left thumb
holds Ctrl and Alt for the right hand's commands. I/K and Up/Down SHALL pick the
parameter in edit. On Pads and FX, J/L and Left/Right SHALL change the selected value
with edit open or closed. On Chop outside slice edit they SHALL keep cursor and zoom
roles; in slice edit they SHALL pick and change bounds. On Files they SHALL keep file
navigation. U/O SHALL turn banks on Pads, including during pad edit. A tap SHALL be a
fine step and holding SHALL repeat the coarse step. Ctrl/Alt chords SHALL keep the same
action or role where bound, with named test exceptions; plain letters MAY have screen
roles, such as H for pad mode on Pads and hit cuts on Chop. No command SHALL use a pad
block key alone or with Shift. Minus, equal and arrows SHALL NOT pick a pad.

#### Scenario: The same keys on a pad and on a slice
- **WHEN** the owner edits a pad on Pads and a slice on Chop
- **THEN** I/K pick the parameter and J/L change it on both screens
- **AND** the arrows do the same as the letters

#### Scenario: A tap is fine, holding is coarse
- **WHEN** pad edit is open on attack and the owner taps L, then holds it
- **THEN** the tap raises attack by 1 ms and each repeat of the held key by 5 ms

#### Scenario: Arrows change the picked value
- **WHEN** edit is open and the owner presses Right
- **THEN** the picked value moves a fine step and the selection stays

#### Scenario: Enter closes too
- **WHEN** pad edit is open and the owner presses Enter
- **THEN** edit closes, as with Esc

#### Scenario: Minus and equal are free
- **WHEN** no edit is open and the owner presses Minus on Pads
- **THEN** nothing changes

#### Scenario: No command on the left half
- **WHEN** the default keymap is loaded
- **THEN** no command uses `1`–`5`, `Q`–`T`, `A`–`G`, `Z`–`B` without Ctrl or Alt
- **AND** each bound Ctrl/Alt chord keeps the same action role across screens, except
  named and explained test cases

## MODIFIED Requirements

### Requirement: Auto-chop by hits

The system SHALL chop a sample at hit starts on H, with a threshold below the loudest
hit. Each press SHALL step lower (-6, -12, -18, -24, -30 dB, then -6 again). Shift+H,
`chop.hits_back`, SHALL step to the next higher value of that list, stopping at -6 dB
without wrapping; before a hit threshold is set it SHALL start at -6 dB. A continuous
knob value below -6 dB SHALL step to the next higher listed value. A knob value already
at or above -6 dB SHALL stay on reverse, so reverse never adds quieter hits. Either
command SHALL replace cuts with the same pad-link safeguards as equal parts and make
one undo step when cuts or threshold change. A clamped reverse press SHALL change
nothing and add no undo step. The knob `chop.threshold` SHALL still set the threshold
continuously and recut at once. The status SHALL name the threshold and slice count.

#### Scenario: Break by hits
- **WHEN** a break is on Chop and the owner presses H
- **THEN** cuts land on the starts of the loud hits, and the status says how many

#### Scenario: More presses, more cuts
- **WHEN** the owner presses H again
- **THEN** quieter hits get cuts too; Ctrl+Z brings back the cuts of the press before

#### Scenario: The threshold on a knob
- **WHEN** the owner turns the knob bound to `chop.threshold`
- **THEN** cuts are added or removed at once as the threshold moves

#### Scenario: Reverse the hit threshold
- **WHEN** the hit threshold is -18 dB and the owner presses Shift+H
- **THEN** the threshold becomes -12 dB and quieter cuts disappear
- **AND** one Ctrl+Z restores the previous cuts and -18 dB threshold

#### Scenario: Reverse hit threshold stops at the first step
- **WHEN** the threshold is -6 dB and the owner presses Shift+H again
- **THEN** cuts and threshold stay, no undo step is added, and the flash names -6 dB

#### Scenario: Reverse a continuous hit threshold
- **WHEN** the threshold knob has set -15 dB and the owner presses Shift+H
- **THEN** the threshold becomes -12 dB and the cuts follow the same hit rule

#### Scenario: Reverse keeps a higher knob threshold
- **WHEN** the threshold knob has set -3 dB and the owner presses Shift+H
- **THEN** the threshold and cuts stay and no undo step is added

#### Scenario: First reverse hit command
- **WHEN** no hit threshold has been explicitly set and the owner presses Shift+H
- **THEN** the threshold becomes -6 dB, hit cuts are made and the status names the threshold

### Requirement: Normalize

The system SHALL, on Ctrl+G on Chop, set the volume of the pad the screen was opened
from so the loudest peak of what that pad plays sits at -1 dB or just below (the volume
is whole dB, rounded down), within the pad volume range (up to +6 dB). The status SHALL
show the gain and say when the limit stopped it. The file on disk SHALL NOT change.
When Chop was opened without a pad, the status SHALL say there is no pad to normalize.

#### Scenario: Quiet break
- **WHEN** the owner opens a pad whose peaks are 6 dB below full scale and presses Ctrl+G
- **THEN** the pad volume is +5 dB, the status shows "+5 dB", and the pad plays without clipping

#### Scenario: Very quiet break
- **WHEN** the pad peaks are 20 dB below full scale and the owner presses Ctrl+G
- **THEN** the pad volume is +6 dB and the status says the volume limit stopped it

#### Scenario: Twice in a row
- **WHEN** the owner presses Ctrl+G a second time
- **THEN** gain is measured again from the sound, so the level does not climb twice

#### Scenario: Chop without a pad
- **WHEN** Chop was opened from Files and the owner presses Ctrl+G
- **THEN** the status says there is no pad to normalize

### Requirement: One rule for the hits

The system SHALL use one hit threshold on Chop for cursor hits mode, cuts snapped to
hits, hit marks and hit chopping. A hit counts when it is at most the threshold below
the loudest hit of the region. The threshold SHALL stay -12 dB until H, Shift+H, a
learned hit command or the threshold knob sets it, then use their value. In hits mode,
each counted hit on screen SHALL be marked with `▾` at its start. These marks SHALL
show where the cursor jumps and where hit chopping cuts at that threshold. Explicit
owner keymaps for `chop.hits` SHALL use this same rule, including the former Ctrl+Y.

#### Scenario: The marks are the stops
- **WHEN** hits mode is on and the screen shows four marks
- **THEN** Right from the start of the view stops at the four marks in turn

#### Scenario: Ctrl+Y cuts at the marks
- **WHEN** the owner explicitly maps Ctrl+Y to `chop.hits`, presses it and turns hits mode on
- **THEN** a mark stands at every new cut and nowhere else in the region

#### Scenario: No marks in even steps
- **WHEN** the cursor walks in even steps or cut to cut
- **THEN** no hit marks are drawn

#### Scenario: H cuts at the marks by default
- **WHEN** the default map is loaded, the owner presses H and turns hits mode on
- **THEN** a mark stands at every new cut and nowhere else in the region

### Requirement: By hits says what it replaced

The system SHALL, when H or Shift+H chops by hits over existing cuts, name the threshold,
new and previous slice counts, and Ctrl+Z in the status:
`by hits at -12 dB: 7 slices (were 16) · Ctrl+Z brings them back`. With no cuts before,
the line SHALL end after the slice count. A clamped reverse command SHALL report its
threshold without claiming it replaced cuts.

#### Scenario: By hits says what it replaced
- **WHEN** the owner cut a break into 16 slices by hand and presses H
- **THEN** the status says `by hits at -6 dB: N slices (were 16) · Ctrl+Z brings them back`
- **AND** Ctrl+Z brings the 16 slices back

## ADDED Requirements

### Requirement: Named assign order action

On Chop, the system SHALL bind Tab to `chop.assign_order`. It SHALL switch by-hand and
in-order assignment only while an assign plan is open, with the same plan and undo
behavior as the former `edit.param_next`. It SHALL do nothing in listen or slice edit
and SHALL NOT apply a plan. Existing owner keymaps and learned controls for
`edit.param_next` and `chop.auto_mode_next` SHALL keep switching the order while
assigning. Threshold commands SHALL stay ignored while assigning.

#### Scenario: Tab has the named assign order action
- **WHEN** a by-hand plan is open and the owner presses Tab
- **THEN** the plan switches to in-order with no target until one is selected
- **AND** the next Tab returns to an empty by-hand plan without applying either plan

#### Scenario: Old parameter binding still switches assign order
- **WHEN** a learned controller button or owner key sends `edit.param_next` while assigning
- **THEN** it switches by-hand and in-order as Tab does

#### Scenario: Old auto mode binding still switches assign order
- **WHEN** an owner binding sends `chop.auto_mode_next` while assigning
- **THEN** it keeps its former order-switching behavior and does not apply the plan

#### Scenario: Assign order does not edit a listening slice
- **WHEN** `chop.assign_order` is sent in listen or slice edit
- **THEN** cuts, selection, bounds and playback stay unchanged and no plan opens

#### Scenario: New hit keys stay deaf while assigning
- **WHEN** an assign plan is open and the owner presses H or Shift+H
- **THEN** cuts, threshold and plan stay unchanged and the status names apply and back

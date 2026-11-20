## ADDED Requirements

### Requirement: Trim silence

Chop SHALL have `trim silence` on device pad P12. A sound is a sample louder than −50 dB below the peak of the file. A press SHALL move the trim start to the first sound and the trim end to the last sound. Without cuts, it SHALL act at once. With cuts, it SHALL ask first, because cuts in the trimmed parts go. The cell SHALL be grey while the trim already sits on the first and the last sound. One undo SHALL bring back the trim and the cuts.

#### Scenario: Trim silence without cuts acts at once
- **WHEN** a sample without cuts has 0.5 s of silence at the start and 1 s at the end, and the owner presses `trim silence`
- **THEN** the trim start sits at the first sound and the trim end at the last sound, and no question shows

#### Scenario: Trim silence with cuts asks first
- **WHEN** the chop has cuts and the owner presses `trim silence`
- **THEN** a question asks before the trim changes, and after yes one undo brings back the trim and the cuts

#### Scenario: Trim silence without silence is grey
- **WHEN** the trim already sits on the first and the last sound and the owner holds PAD BANK on Chop
- **THEN** `trim silence` is grey and a press says «no silence to trim»

### Requirement: Remove all cuts

Chop SHALL have `remove all cuts` on device pad P13 and on the Delete key. It SHALL be grey while the chop has no cut. When cuts made by hand would go, a press SHALL ask «Remove N cuts?». P4 is yes and P1 is no. Cuts made by the program SHALL go without a question. After it, the chop SHALL have one slice. Pads linked to the old slices SHALL keep their sound, as after `equal parts`. A notice SHALL say «N cuts removed · P2 undo brings them back».

#### Scenario: Remove all cuts asks and can be undone
- **WHEN** the chop has 6 cuts made by hand and the owner presses `remove all cuts` and answers yes
- **THEN** the chop has one slice and a notice says «6 cuts removed · P2 undo brings them back»
- **AND** one undo brings back the 6 cuts

#### Scenario: The Delete key removes all cuts
- **WHEN** the chop has cuts by hits only and the owner presses Delete on Chop
- **THEN** the chop has one slice without a question

### Requirement: Fewer hits acts on Chop

`fewer hits` SHALL be a Chop cell on device pad P7, not a slice edit cell. It SHALL act as Shift+H. It SHALL be grey while the threshold is −6 dB or while the chop has no cut by hits. Shift+H on a chop without a cut by hits SHALL change no cut and SHALL say «cut by hits first».

#### Scenario: Fewer hits from Chop
- **WHEN** the chop is cut by hits at −20 dB and the owner presses `fewer hits` on Chop
- **THEN** the chop is cut by hits at a higher threshold with fewer slices

#### Scenario: Fewer hits before cut by hits is grey
- **WHEN** the chop has no cut by hits and the owner holds PAD BANK on Chop
- **THEN** `fewer hits` is grey and a press says «cut by hits first»

#### Scenario: Shift+H before cut by hits changes nothing
- **WHEN** the chop has no cut by hits and the owner presses Shift+H on Chop
- **THEN** no cut changes and a notice says «cut by hits first»

### Requirement: In order waits for the start pad

In the assign mode "all in order", apply SHALL wait until the owner hits the start pad. Before that hit, `Enter` and device pad 4 SHALL place no slice, and a notice SHALL say «hit the start pad first». After the hit, the plan SHALL lay slice 1 on that pad and the rest after it. The next apply SHALL apply it as any plan, with the overwrite question for full pads.

#### Scenario: Enter with no pad hit
- **WHEN** the owner opens the assign mode, switches to "in order" with `Tab` and presses `Enter`
- **THEN** no slice is placed and a notice says «hit the start pad first»

#### Scenario: The start pad sets the order
- **WHEN** in order, the owner hits pad Q of bank 2 and presses `Enter`
- **THEN** the plan with slice 1 on pad Q of bank 2 is applied

## MODIFIED Requirements

### Requirement: Normalize

The system SHALL, on Ctrl+G on Chop or on `norm` in the slice edit, set the sound gain of each pad that plays the selected slice. The loudest peak of what that pad plays SHALL sit at -1 dB. The sound gain SHALL be separate from the pad volume and SHALL NOT be limited by the pad volume range. The gain SHALL be at most +30 dB. The status SHALL show the gain and say when the limit stopped it. The file on disk SHALL NOT change. The gain SHALL be saved per pad in the project. Undo on Pads SHALL bring back the old gain. When no pad plays the selected slice, `norm` SHALL be grey and a press SHALL say «put the slice on a pad first».

#### Scenario: Quiet break
- **WHEN** the owner selects a slice whose pad peaks 6 dB below full scale and presses `norm`
- **THEN** the pad sound gain is +5 dB, the pad volume stays, the status shows "+5 dB", and the pad plays without clipping

#### Scenario: Very quiet break
- **WHEN** the pad peaks are 20 dB below full scale and the owner presses `norm`
- **THEN** the pad sound gain is +19 dB, beyond the +6 dB pad volume limit

#### Scenario: Twice in a row
- **WHEN** the owner presses `norm` a second time
- **THEN** gain is measured again from the sound, so the level does not climb twice

#### Scenario: Chop without a pad
- **WHEN** Chop was opened from Files, no pad plays the selected slice and the owner presses Ctrl+G
- **THEN** the status says there is no pad to normalize

#### Scenario: Normalize can be undone
- **WHEN** the owner presses `norm` and then undo
- **THEN** the pad sound gain returns to its value before the press

#### Scenario: Normalize without a pad is grey
- **WHEN** no pad plays the selected slice and the owner holds PAD BANK in the slice edit
- **THEN** `norm` is grey and a press says «put the slice on a pad first»

#### Scenario: The sound gain is saved
- **WHEN** a pad has a sound gain of +5 dB and the owner saves and opens the project again
- **THEN** the pad sound gain is +5 dB, and a project without the gain opens with 0 dB

### Requirement: Equal-parts auto-chop
The system SHALL chop a sample into equal parts (2, 4, 8, 16, 32); one command switches to the next count. When cuts made by hand would go, the command SHALL ask first, with P4 yes and P1 no. Cuts made by the program SHALL be replaced without a question.

#### Scenario: Eight parts
- **WHEN** the owner presses the auto-chop command until the count is 8
- **THEN** the sample has 7 markers at equal distances, the header says "8 slices"

#### Scenario: Equal parts over hand cuts asks
- **WHEN** the owner made a cut by hand and presses the equal-parts command and answers no
- **THEN** the cuts stay as they were

### Requirement: Ask before overwriting pads
The system SHALL, before any step that loses what lies on a pad and cannot be undone, show in plain words which pads lose what and what takes its place, and SHALL do the step only after the owner confirms; declining changes nothing. A step that replaces cuts made by hand SHALL ask too, and its question SHALL name the undo.

#### Scenario: Apply a plan over taken pads
- **WHEN** a plan puts slices 1 and 2 on pads Q and W of bank 2 that hold "kick" and "amen 3" and the owner applies it
- **THEN** a question lists "bank 2, pad Q: kick → slice 1" and "bank 2, pad W: amen 3 → slice 2", says that the files on disk stay, and offers Enter to replace and Esc to keep

#### Scenario: Put a file on a taken pad
- **WHEN** a file is in hand on the files screen and the owner hits a pad that holds "kick"
- **THEN** a question says that "kick" on that pad will be replaced by the file; Esc keeps "kick" and the file stays in hand

#### Scenario: Steps that can be undone
- **WHEN** a step can be taken back with undo and removes no cut made by hand (cuts by the program, the plan itself)
- **THEN** it is done without a question

#### Scenario: Same sound again
- **WHEN** the owner puts on a pad the same whole file that the pad already plays
- **THEN** no question appears and the pad does not change

### Requirement: Auto-chop by hits

H SHALL cut at hit starts using a threshold below the loudest hit.
Each press SHALL cycle -6, -12, -18, -24, -30 dB, then -6 again.
`chop.threshold` SHALL set a continuous threshold and recut immediately.
H and Shift+H SHALL replace cuts with equal-parts pad-link safeguards and make one undo step when cuts or threshold change.
When cuts made by hand would go, H, Shift+H and the threshold knob SHALL ask first and SHALL change no cut before yes.
The status SHALL name threshold and slice count.

#### Scenario: Break by hits
- **WHEN** a break is on Chop and the owner presses H
- **THEN** cuts land on the starts of the loud hits, and the status says how many

#### Scenario: More presses, more cuts
- **WHEN** the owner presses H again
- **THEN** quieter hits get cuts too; Ctrl+Z brings back the cuts of the press before

#### Scenario: The threshold on a knob
- **WHEN** the owner turns the knob bound to `chop.threshold`
- **THEN** cuts are added or removed at once as the threshold moves

#### Scenario: By hits over hand cuts asks
- **WHEN** the owner made a cut by hand and presses H
- **THEN** a question asks before the cuts change, and no asks keeps the hand cut

### Requirement: By hits says what it replaced

The system SHALL, when H or Shift+H chops by hits over existing cuts, name the threshold,
new and previous slice counts, and the undo control in the status:
`by hits at -12 dB: 7 slices (were 16) · Ctrl+Z brings them back` after keyboard input, and
`… · P2 undo brings them back` after SMC-PAD input. With no cuts before,
the line SHALL end after the slice count. A clamped reverse command SHALL report its
threshold without claiming it replaced cuts.

#### Scenario: By hits says what it replaced
- **WHEN** the owner cut a break into 16 slices by hand, presses H and answers yes
- **THEN** the status says `by hits at -6 dB: N slices (were 16) · Ctrl+Z brings them back`
- **AND** Ctrl+Z brings the 16 slices back

#### Scenario: By hits names the SMC-PAD undo
- **WHEN** the chop has 5 slices by parts and the owner presses `cut by hits` on the SMC-PAD panel
- **THEN** the status ends with «P2 undo brings them back»

### Requirement: Named assign order action

On Chop, the system SHALL bind Tab to `chop.assign_order`. It SHALL switch by-hand and
in-order assignment only while an assign plan is open, with the same plan and undo
behavior as the former `edit.param_next`. With placed slices, a switch SHALL ask first,
name the placed slices that go and name the undo; one undo SHALL bring them back. It SHALL do nothing in listen or slice edit
and SHALL NOT apply a plan. Existing owner keymaps and learned controls for
`edit.param_next` and `chop.auto_mode_next` SHALL keep switching the order while
assigning. Threshold commands SHALL stay ignored while assigning.

#### Scenario: Tab has the named assign order action
- **WHEN** a by-hand plan with no placed slice is open and the owner presses Tab
- **THEN** the plan switches to in-order with no target until one is selected
- **AND** the next Tab returns to an empty by-hand plan without applying either plan

#### Scenario: A switch with placed slices asks
- **WHEN** a by-hand plan has 2 of 5 slices placed and the owner presses Tab and answers yes
- **THEN** the plan is in order with 0 placed and a notice says «in order · P2 undo brings 2 back»
- **AND** one undo brings back the 2 placed slices by hand

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

## REMOVED Requirements

### Requirement: Enter starts the order from the selected pad
**Reason**: The owner decided that apply waits until the owner hits the start pad (Q27, walk finding F22).
**Migration**: Hit the start pad, then press `Enter` or device pad 4. See "In order waits for the start pad".

## ADDED Requirements

### Requirement: Copy a pad

On Pads, the system SHALL take a snapshot of the selected loaded pad with Ctrl+C or
`pad.copy` and show `COPY · hit a pad`. The next keyboard or controller pad hit SHALL
select a single target in any bank without also playing it. The source SHALL stay
unchanged. The copy SHALL share sample data and keep the file, bounds, slice link,
mode, volume, pitch, envelope, filter, drive, mute group and sends. Its custom name
SHALL be the displayed source name plus ` 2`. Marks and held state SHALL NOT be copied.
An occupied target SHALL ask before replacement. Esc in that question SHALL keep the
target and copy in hand; Esc outside it or a hit on the original source SHALL cancel.
One successful placement SHALL clear the hand, save the change and make one undo step
for the target. A cancelled or refused copy SHALL change no pad and add no undo step.
Empty, missing or loading sources and loading targets SHALL be refused with a notice.

#### Scenario: Copy the selected pad with all settings
- **WHEN** Q has a loaded slice, custom name, gate mode, volume, pitch, envelope, filter,
  drive, mute group and sends, and the owner presses Ctrl+C then hits empty W
- **THEN** W keeps all Q's sound settings, bounds and slice link, with its name ending in ` 2`
- **AND** Q stays unchanged, the copy hand clears and the target hit does not play

#### Scenario: Copy shares sample data
- **WHEN** a loaded pad is copied to another pad
- **THEN** both use the same sample allocation, not a decoded or duplicated sample buffer
- **AND** a linked copied slice follows later cut edits as its source does

#### Scenario: Copy asks before replacing a pad
- **WHEN** a copy is in hand and the owner hits an occupied pad
- **THEN** no pad changes and a question names the target's current sound and the copy

#### Scenario: Confirm a pad copy replacement
- **WHEN** the copy replacement question is open and the owner presses Enter
- **THEN** only the named target is replaced, the source stays and the copy hand clears

#### Scenario: Decline a pad copy replacement
- **WHEN** the copy replacement question is open and the owner presses Esc
- **THEN** the target stays unchanged and the copy remains in hand for another target

#### Scenario: A changed copy target asks again
- **WHEN** a target changes after the copy question was shown and the owner presses Enter
- **THEN** a new question names the current target and no replacement occurs yet

#### Scenario: Esc drops a pending copy
- **WHEN** a copy is in hand with no question open and the owner presses Esc
- **THEN** the hand clears, no pad changes and no undo step is added

#### Scenario: Copy onto the source cancels
- **WHEN** a copy is in hand and the owner hits its original source pad
- **THEN** the hand clears without playing, changing a pad or adding an undo step

#### Scenario: Copy target from a controller in another bank
- **WHEN** a copy is in hand, the bank turns and a controller note or learned pad button hits a target
- **THEN** that bank's target receives the same copy as a keyboard target would
- **AND** no direct pad sound is triggered and existing held-pad releases still work

#### Scenario: Copy keeps the settings from pickup
- **WHEN** the owner takes a copy of Q, changes Q's pitch and then puts the copy on W
- **THEN** W has the pitch from pickup, not Q's later pitch

#### Scenario: Copy refuses an unavailable source or loading target
- **WHEN** the source is empty, missing or loading, or a chosen target is loading
- **THEN** no copy is applied, a plain notice explains why, and pads and existing hand state stay

#### Scenario: Copy changes one target despite marks
- **WHEN** pads are marked, a copy is in hand and the owner hits W
- **THEN** only W is the copy target, no marked-pad group is replaced and marks stay

#### Scenario: A copied pad survives restart
- **WHEN** a copy is placed, the project or scratch session is saved and reopened
- **THEN** the source and copy keep their samples, source links, names and settings

#### Scenario: The copy sounds like the source
- **WHEN** source and copy are rendered separately with matching velocity and the same engine state
- **THEN** their output matches sample for sample, with no non-finite samples or clipping
- **AND** the render checks expected silence and hit timing within one sample

## MODIFIED Requirements

### Requirement: A pad in hand is named

While a pad is in hand for a swap, the system SHALL name it on Pads in a short line
that fits the panel (`in hand: pad 3 · amen 3`), with the next step in help. The next
pad hit SHALL swap it and Esc SHALL put it back. A copy in hand SHALL instead use
`COPY · hit a pad` and the separate copy rules; it SHALL NOT swap the source.

#### Scenario: Swap by a hit
- **WHEN** W is in hand for a swap and the owner hits E
- **THEN** W and E swap, and the panel no longer names a pad in hand

# pad-assignment Specification

## Purpose
Putting samples on pads: a file in hand from the files screen goes to the next pad hit; the session keeps samples, modes and volumes of all pads.

## Requirements

### Requirement: Assigning a sample by hitting a pad
The system SHALL let the owner take a file "in hand" on the files screen with `Enter`. A hit on a pad from the keyboard or a controller puts the file on that pad. `Enter` again puts the file on the selected pad. `Ctrl+Enter` loads the file onto the selected pad at once, without the hand (41). `Esc` puts the file back. Without a file in hand, pad hits play as usual.

#### Scenario: Put a break on a pad
- **WHEN** the owner presses `Enter` on a break on the files screen and hits pad W
- **THEN** the break is assigned to pad 6 of the current bank, the next hit on W plays it

#### Scenario: Change of mind
- **WHEN** a file is in hand and the owner presses Esc
- **THEN** no pad changes, and pad hits play again

### Requirement: Saving the session
The system SHALL save pad samples, modes and volumes of all banks and the last browser folder on every change and on exit, and restore them on the next start; with a project open the session saves into that project's `project.toml` (sample paths inside the project folder relative to it), without a project — into the scratch session file as today.

#### Scenario: Restart
- **WHEN** the owner closes and opens the program again
- **THEN** all pads of all banks sound the same as before closing, and the files screen opens in the same folder

#### Scenario: File is gone
- **WHEN** an assigned file is deleted from disk
- **THEN** the pad is marked as "missing file" and stays silent, the others work

#### Scenario: Saved into the project
- **WHEN** a project is open and the session autosaves
- **THEN** the `project.toml` of that project changes and the scratch session file stays untouched

### Requirement: Swapping pads
The system SHALL let the owner take the selected pad "in hand" with Ctrl+X and swap it with the next pad that is hit; everything a pad has moves together: the sample and the part of it it plays, the mode, the volume, the pitch, the envelope, the filter, the drive and the sends. Esc cancels.

#### Scenario: Swap two pads
- **WHEN** pad Q is selected, the owner presses Ctrl+X and then hits pad V
- **THEN** Q now plays what V played and V plays what Q played, with their modes and volumes; the session is saved

#### Scenario: Every setting moves
- **WHEN** pad Q has a low-pass filter, pitch +3 and a drive, pad V has none of them, and
  the owner swaps them
- **THEN** pad V has the low-pass filter, pitch +3 and the drive, sounds with them, and pad
  Q has none of them

#### Scenario: Move to an empty pad
- **WHEN** the pad in hand is put on an empty pad
- **THEN** the sample moves there and the first pad becomes empty

#### Scenario: Swap while loading
- **WHEN** one of the two pads is still loading a file
- **THEN** the pads are not swapped and the status says to wait until loading finishes

### Requirement: A pad in hand is named

While a pad is in hand for a swap, the system SHALL name it on Pads in a short line
that fits the panel (`in hand: pad 3 · amen 3`), with the next step in help. The next
pad hit SHALL swap it and Esc SHALL put it back. A copy in hand SHALL instead use
`COPY · hit a pad` and the separate copy rules; it SHALL NOT swap the source.

#### Scenario: Swap by a hit
- **WHEN** W is in hand for a swap and the owner hits E
- **THEN** W and E swap, and the panel no longer names a pad in hand

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

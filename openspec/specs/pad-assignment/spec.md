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

The system SHALL save all banks' pad samples, modes, volumes and the last browser folder, and restore them at startup.
Changes SHALL save after editing and on exit.
Projects SHALL use `project.toml` and relative internal paths. Scratch SHALL use its session file.
Only successful replacement SHALL clear pending work.
Write failure SHALL retain work and saved bytes, allow manual retry and pause automatic and exit saves.
A successful manual save SHALL restore autosave for later edits.

#### Scenario: Restart
- **WHEN** the owner closes and opens the program again after successful saving
- **THEN** all pads of all banks sound the same as before closing, and the files screen opens in the same folder

#### Scenario: File is gone
- **WHEN** an assigned file is deleted from disk
- **THEN** the pad is marked as "missing file" and stays silent, the others work

#### Scenario: Saved into the project
- **WHEN** a project is open and the session autosaves
- **THEN** the `project.toml` of that project changes and the scratch session file stays untouched

#### Scenario: Failed autosave retains pending work
- **WHEN** an autosave fails after the owner changes a pad
- **THEN** the pad change remains pending and the last saved file stays byte for byte unchanged
- **AND** later edits retain all pending work without resuming automatic retries

#### Scenario: Manual retry restores autosave
- **WHEN** a write problem ends and the owner requests a manual save after a failed save
- **THEN** the current work saves and the pending-change and write-failure states clear
- **AND** a later edit saves through ordinary autosave

#### Scenario: A failed save does not retry on exit
- **WHEN** the owner exits while a write failure remains unresolved
- **THEN** exit saving does not retry the write or report success

#### Scenario: An occupied temporary path preserves the saved file
- **WHEN** a save cannot safely create or write its temporary file
- **THEN** the save fails without changing the last saved file or another file at the temporary path

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

On Pads, Ctrl+C or `pad.copy` SHALL snapshot the selected loaded pad and show `COPY · hit a pad`.
The next keyboard/controller pad hit SHALL silently select one target in any bank, leaving the source unchanged.
The copy SHALL share sample data.
It SHALL keep file, bounds, slice link, mode, volume, pitch, envelope, filter, drive, mute group and sends.
Its name SHALL follow the numbered pad copy rules.
Marks and held state SHALL NOT be copied.

#### Scenario: Copy the selected pad with all settings
- **WHEN** Q has a loaded slice, custom name, gate mode, volume, pitch, envelope, filter, drive, mute group and sends
- **AND** the owner presses Ctrl+C then hits empty W
- **THEN** W keeps Q's sound settings, bounds and slice link, with its name following the numbered pad copy rules
- **AND** Q stays unchanged, the copy hand clears and the target hit does not play

#### Scenario: Copy shares sample data
- **WHEN** the owner copies a loaded pad to another pad
- **THEN** both use the same sample allocation, not a decoded or duplicated sample buffer
- **AND** a linked copied slice follows later cut edits as its source does

### Requirement: Pad copy confirmation and cancellation

Occupied targets SHALL ask before replacement.
Esc in that question SHALL keep the target and copy in hand.
Esc outside it or a hit on the original source SHALL cancel.
Cancelled or refused copies SHALL change no pad and add no undo step.

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

### Requirement: Pad copy placement and refusal

Empty, missing or loading sources and loading targets SHALL be refused with a notice.
One successful placement SHALL clear the hand, save the change and make one undo step for the target.

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

### Requirement: Preserve scratch work on project transitions

Pending scratch work SHALL save before another project opens or a new project is created.
If the source has pending work and read protection, the transition SHALL refuse without clearing either state.
A clean protected scratch session SHALL permit leaving without writing its source file.

#### Scenario: Scratch edits save before opening or creating a project
- **WHEN** scratch has pending edits and the owner opens or creates a project before autosave
- **THEN** the scratch file retains those edits before the target replaces the session

#### Scenario: Pending protected work blocks a transition
- **WHEN** a protected project or scratch session has pending work and the owner tries to open or create another project
- **THEN** the transition refuses without changing the source, pending work, undo or target

#### Scenario: Clean protected scratch can leave without a write
- **WHEN** scratch has read protection but no pending work and the owner opens another project
- **THEN** the target opens without a source save and the protected scratch file stays byte for byte unchanged

### Requirement: Numbered pad copy names

Copies SHALL use `<base> (copy N)`.
The base SHALL be the full source name at pickup, before clipping, without its final canonical ` (copy N)` marker.
Canonical `N` SHALL be a positive ASCII decimal number without leading zeroes or a machine-integer size limit.
The marker SHALL use the shown case and spaces.
Only that final marker SHALL identify a copy family, including in a manual name.
The system SHALL preserve all other base text, including Unicode and slice numbers.

#### Scenario: Repeated source copies use numbered names
- **WHEN** the owner copies `kick` three times to empty pads, with no previous copy names occupied
- **THEN** the targets receive `kick (copy 1)`, `kick (copy 2)` and `kick (copy 3)`
- **AND** the source remains named `kick`

#### Scenario: A copy of a copy replaces its marker
- **WHEN** `kick (copy 1)` exists and the owner copies it to an empty target, with `kick (copy 2)` available
- **THEN** the new copy uses `kick (copy 2)`, without another copy marker
- **AND** the source name stays unchanged

#### Scenario: Slice numbers remain separate from copy numbers
- **WHEN** the owner copies a pad shown as `break 2`, with `break 2 (copy 1)` available
- **THEN** the copy uses `break 2 (copy 1)` and retains the same slice link

#### Scenario: Only a canonical final copy marker identifies the base
- **WHEN** the owner copies `kick (copy 01)`, `kick (Copy 1)` or `kick (copy 0)`
- **THEN** each new copy retains the full source text as its base and receives a canonical marker
- **AND** copying `kick (copy 999999999999999999999999999999)` uses `kick` as its base without integer overflow

### Requirement: Copy names avoid occupied names in all banks

The system SHALL choose the smallest positive `N` with an unoccupied complete candidate name.
It SHALL compare full names exactly before clipping, across all banks of the current project or scratch session.
The scan SHALL include the source, missing pads and loading pads.
The target's current name SHALL remain occupied until replacement succeeds.
Names in other projects SHALL NOT reserve numbers.
The system SHALL NOT use a saved counter or change another pad's name to make a candidate available.

#### Scenario: A name in another bank reserves its number
- **WHEN** `kick (copy 1)` exists in another bank and the owner copies `kick`, with `kick (copy 2)` available
- **THEN** the new copy uses `kick (copy 2)`
- **AND** the result is the same when the existing pad is missing or loading

#### Scenario: Copy numbers reuse the smallest gap
- **WHEN** `kick (copy 1)` and `kick (copy 3)` exist, but `kick (copy 2)` does not
- **THEN** the next copy in that family uses `kick (copy 2)`
- **AND** the same rule applies when deletion or undo frees that name
- **AND** a copy of `kick (copy 3)` also uses `kick (copy 2)` if that gap remains free

#### Scenario: Replacement includes the current target name
- **WHEN** the only numbered name is `kick (copy 1)` on an occupied target and the owner confirms copying `kick` onto it
- **THEN** that target becomes `kick (copy 2)` and the source stays unchanged

#### Scenario: Other projects do not reserve copy numbers
- **WHEN** only a different project contains `kick (copy 1)` and the owner copies `kick` in the current work
- **THEN** the new copy can use `kick (copy 1)` in the current project or scratch session

#### Scenario: Unicode copy names use full text rather than clipped cells
- **WHEN** two long Unicode names share a clipped screen prefix but differ in their full text
- **THEN** a copy reserves numbers only against exact full candidate names
- **AND** stored base-name text stays unchanged, including Unicode and case

### Requirement: Copy name selection follows placement state

The base name SHALL remain the name from the pickup snapshot if the source changes later.
The system SHALL scan current pad names when it selects or confirms a target.
A replacement question SHALL show the name that a successful confirmation applies.
If current names change the proposed copy name, confirmation SHALL show an updated question without replacing the target yet.

#### Scenario: Copy naming keeps the pickup name
- **WHEN** the owner takes a copy of `kick`, renames the source to `snare` and then selects an empty target
- **THEN** the new copy uses the `kick` base with a currently available number
- **AND** the source keeps its later name

#### Scenario: A changed proposed copy name asks again
- **WHEN** an occupied-target question proposes `kick (copy 1)` and another pad obtains that name before confirmation
- **THEN** confirmation asks again with the next available name and leaves the target unchanged
- **AND** a later confirmation applies exactly the name in the updated question if it remains available

### Requirement: Cancelled and undone copies leave numbers available

Cancellation and refusal SHALL reserve no number and change no pad.
Undo SHALL restore the target's prior name with its existing single copy undo step.

#### Scenario: Cancelled and refused copies do not consume numbers
- **WHEN** the owner cancels or receives a refusal before copying `kick` successfully, with `kick (copy 1)` available
- **THEN** the successful copy uses `kick (copy 1)`
- **AND** undo restores the previous target and permits reuse of that available name

### Requirement: Copy naming preserves stored work

The system SHALL preserve existing names on load, save and reopen.
Copying SHALL preserve names on every pad except the target.
This SHALL include old ` 2` chains and manual names.
New names SHALL use the existing custom-name field without a format change.
Keyboard and controller paths SHALL use the same naming rules.
Sound, source links, sample files, confirmation controls and name clipping SHALL stay unchanged.

#### Scenario: Old numeric suffixes stay literal
- **WHEN** the owner opens work containing `rec 2 2 2` and copies that pad, with its first candidate available
- **THEN** the original stays `rec 2 2 2` and the new copy uses `rec 2 2 2 (copy 1)`
- **AND** saving and reopening do not clean or renumber other existing names

#### Scenario: A canonical manual source name stays unchanged
- **WHEN** the owner manually names a pad `kick (copy 7)` and copies it, with `kick (copy 1)` available
- **THEN** the new copy uses `kick (copy 1)` and the source remains `kick (copy 7)`

#### Scenario: Restored copy names determine the next available number
- **WHEN** the owner saves work containing `kick (copy 1)`, reopens it and copies that pad, with `kick (copy 2)` available
- **THEN** the stored copy keeps its name and the new copy uses `kick (copy 2)`
- **AND** this applies to both named projects and scratch sessions

## MODIFIED Requirements

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

## ADDED Requirements

### Requirement: Numbered pad copy names

Copies SHALL use `<base> (copy N)`.
The base SHALL be the full source name at pickup, before clipping, without its final canonical ` (copy N)` marker.
Canonical `N` SHALL be a positive ASCII decimal number without leading zeroes or a machine-integer size limit.
The marker SHALL use the shown case and spaces.
Only that final marker SHALL identify a copy family.
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

The system SHALL choose the smallest positive `N` whose complete candidate name differs from every current pad name.
It SHALL compare full names exactly, before screen clipping, across every bank of the current project or scratch session.
The scan SHALL include the source, the occupied target, missing pads and loading pads.
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

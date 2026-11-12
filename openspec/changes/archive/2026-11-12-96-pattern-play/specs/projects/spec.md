# projects Delta

## MODIFIED Requirements

### Requirement: Saved transport preferences

Project and scratch session format 5 SHALL save `bpm` and `metronome`. Tempo SHALL be finite and within 40–300 BPM, rounded to 0.1 BPM. New sessions SHALL use 120 BPM and metronome off. Playback position and running state SHALL NOT be saved. Tempo and metronome edits SHALL use existing dirty-state, manual-save and retry rules.

#### Scenario: Project transport preferences survive a restart
- **WHEN** the owner saves 172.3 BPM with the metronome enabled and restarts
- **THEN** the project restores those preferences stopped at bar 1, beat 1

#### Scenario: Scratch transport preferences survive a restart
- **WHEN** the owner changes tempo and metronome without an open project and saves
- **THEN** reopening scratch restores its preferences without starting playback

### Requirement: Transport session migration

Formats 1–4 SHALL remain readable and use 120 BPM and metronome off when transport fields are absent. Existing pad, chop, group, name, path and effect state SHALL remain intact through migration and saving as format 5. Newer formats and invalid transport values SHALL retain existing load refusal and read protection.

#### Scenario: Older sessions gain transport defaults without changing pads
- **WHEN** supported format 1, 2 or 3 sessions load and then save
- **THEN** format 5 contains transport defaults and preserves the existing migrated pad and chop state
- **AND** groups, effects, copy names and stored paths keep their existing meanings

#### Scenario: Invalid transport settings protect the session file
- **WHEN** a target session contains non-finite tempo, tempo outside the supported range or an invalid metronome value
- **THEN** loading refuses it without replacing current work
- **AND** startup load refusal protects the original file from later save requests

## ADDED Requirements

### Requirement: Saved pattern and Pitch Gate state

Format 5 SHALL save zero patterns or one `A1` entry, its raw hits, bars, quantize, strength and selection. It SHALL save the per-pad `pitch_gate` boolean, with true as its absent default. Project and scratch saves SHALL use their existing safe replacement, dirty-state, retry and read-protection rules. Runtime notes, deadlines and running position SHALL NOT be saved.

#### Scenario: Pattern and Pitch Gate survive project and scratch restart
- **WHEN** project or scratch work saves A1 with raw off-grid and hidden hits, custom playback settings and mixed Pitch Gate values
- **THEN** reopening restores those exact data with transport stopped at tick zero
- **AND** the existing tempo, metronome, sounds, sources and mute groups retain their meanings

### Requirement: Pattern migration preserves older work

Formats 1–4 without patterns SHALL restore no pattern. A format 1–4 file with pattern selection or entries SHALL be refused without discarding those data. Format 4 SHALL retain its saved transport preferences. Earlier formats SHALL keep their existing migration rules. Absent Pitch Gate state SHALL default to true without changing manual playback. Saving as format 5 SHALL preserve names, groups, assignments, bounds, paths, chops, effects and ungrouped overlap.

#### Scenario: Format four gains pattern defaults without changing sound
- **WHEN** a format 4 project with tempo, metronome, linked slices and manual mute groups loads and saves as format 5
- **THEN** no pattern starts, transport preferences remain and absent Pitch Gate values become true
- **AND** existing manual renders match their pre-migration sound and ungrouped pads still overlap

#### Scenario: Older format numbers cannot hide pattern data
- **WHEN** a file declares format 1, 2, 3 or 4 but contains a pattern selection or entry
- **THEN** startup and project loading refuse it without changing the original bytes
- **AND** startup read protection and current project work remain intact

#### Scenario: Earlier session migrations remain compatible with patterns
- **WHEN** fixtures for formats 1, 2 and 3 load and save as format 5
- **THEN** their existing migrations preserve pad and chop behavior with no pattern and the specified transport defaults

### Requirement: Pattern file validation protects work

Loading SHALL reject invalid pattern ranges, malformed hit rows, duplicate or unsupported slots, mismatched selection and non-boolean Pitch Gate values. Invalid raw data SHALL NOT be silently clamped or discarded. A target refusal SHALL retain the current session and pending work. A startup refusal SHALL protect the file from later saves, including newer format versions.

#### Scenario: Invalid pattern targets do not replace a playing project
- **WHEN** a selected target contains invalid pattern settings, rows, selection, slots or Pitch Gate values
- **THEN** target validation refuses it before source saving or replacement
- **AND** the current rhythm, pending work, undo and picker selection remain unchanged

#### Scenario: Invalid or newer pattern startup files remain protected
- **WHEN** startup cannot read an invalid pattern session or a session newer than format 5
- **THEN** later saves explain the refusal and leave the original bytes unchanged

### Requirement: Patterns remain isolated across project transitions

A successful project transition SHALL clear source pattern playback and pending automatic releases before target readiness. The target SHALL open stopped with its own pattern data. Old queued patterns, swaps and releases SHALL NOT change it. A read or required source-save refusal SHALL leave the source rhythm running. Selecting the current folder SHALL NOT reload or reset it.

#### Scenario: Pattern project switches reject stale work under queue pressure
- **WHEN** projects with different patterns switch while command or retirement queues are full and old messages arrive late
- **THEN** only the target pattern can start after readiness and no source hit or deadline reaches it
- **AND** switching to an empty project produces no sequenced sound

#### Scenario: Refused or same-folder selection preserves the pattern
- **WHEN** a running project transition fails target reading or source saving, or selects its current canonical folder
- **THEN** its pattern and clock continue without a reset and current work remains available

# Projects delta

## ADDED Requirements

### Requirement: Saved transport preferences

Project and scratch session format 4 SHALL save `bpm` and `metronome`. Tempo SHALL be finite and within 40–300 BPM, rounded to 0.1 BPM. New sessions SHALL use 120 BPM and metronome off. Playback position and running state SHALL NOT be saved. Tempo and metronome edits SHALL use existing dirty-state, manual-save and retry rules.

#### Scenario: Project transport preferences survive a restart
- **WHEN** the owner saves 172.3 BPM with the metronome enabled and restarts
- **THEN** the project restores those preferences stopped at bar 1, beat 1

#### Scenario: Scratch transport preferences survive a restart
- **WHEN** the owner changes tempo and metronome without an open project and saves
- **THEN** reopening scratch restores its preferences without starting playback

### Requirement: Transport session migration

Formats 1–3 SHALL remain readable and use 120 BPM and metronome off when transport fields are absent. Existing pad, chop, group, name, path and effect state SHALL remain intact through migration and saving as format 4. Newer formats and invalid transport values SHALL retain existing load refusal and read protection.

#### Scenario: Older sessions gain transport defaults without changing pads
- **WHEN** supported format 1, 2 or 3 sessions load and then save
- **THEN** format 4 contains transport defaults and preserves the existing migrated pad and chop state
- **AND** groups, effects, copy names and stored paths keep their existing meanings

#### Scenario: Invalid transport settings protect the session file
- **WHEN** a target session contains non-finite tempo, tempo outside the supported range or an invalid metronome value
- **THEN** loading refuses it without replacing current work
- **AND** startup load refusal protects the original file from later save requests

### Requirement: Transport isolation during project changes

A validated project switch SHALL stop and reset the source clock, discard its queued transport commands and restore the target's preferences before new transport starts. A refused read or required source save SHALL leave the current clock and preferences unchanged. Selecting the current project SHALL NOT restart it.

#### Scenario: Different projects retain independent transport settings
- **WHEN** the owner switches between projects with different transport preferences while the audio queue is full
- **THEN** the target opens stopped with its own preferences and no old command changes them later

#### Scenario: Refused project switch preserves running transport
- **WHEN** target validation or a required source save refuses a project switch while transport runs
- **THEN** the source keeps its tempo, metronome preference and running position without a reset

#### Scenario: Selecting the current project preserves its running clock
- **WHEN** the owner selects the already open project or its canonical folder alias
- **THEN** the picker closes without stopping, resetting or restoring transport

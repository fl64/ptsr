# Proposal: one looped pattern

## Why

The beat clock runs, but it cannot play a saved rhythm. Add one loop before recording and step editing.

## What Changes

- Play one pattern, `A1`, through the existing transport. Start at its beginning and repeat until Stop.
- Enter hits in `project.toml` while the program is closed. Recording belongs to 97. The pattern screen belongs to 98.
- Keep raw hit timing. Apply quantize only to playback, with straight and triplet grids through 1/64.
- Address pads across all eight banks. A pad swap moves its hits with its sound, including hidden hits.
- Add a saved per-pad Pitch Gate switch. For sequenced pitched-up slices, it starts a short cut at the next grid line.
- Empty steps also supply that boundary. Quantize Off uses 1/16. Manual hits never receive this automatic cut.
- Show the pattern slot and loop position beside the existing clock. Keep actual pad playback separate from transport state.
- **BREAKING:** save project and scratch sessions as format 5. Read formats 1–4 without changing their existing sounds or settings.

## Capabilities

### New Capabilities

- `patterns`: one loop, raw hits, playback quantize, precise scheduling and safe note releases.

### Modified Capabilities

- `projects`: format 5 patterns, Pitch Gate state, migration and project isolation.
- `pad-assignment`: pattern addresses follow successful swaps, with atomic undo and saved state.
- `pad-playback`: slice Pitch Gate, ownership-safe automatic releases and saved pad operations.
- `keyboard`: Pitch Gate toggle on Pads, with effective-key help and existing input priorities.
- `midi-learn`: learn the same toggle without changing factory or owner assignments.
- `screen-layout`: accepted pattern position, visible Pitch Gate state and sequenced pad playback feedback.

## Impact

Implementation will affect engine scheduling, command retirement, session storage, pad snapshots, undo, input dispatch and screen views.
No new dependency is planned. The existing clock, metronome, mute groups, effects and source bounds remain authoritative.

Keep the current callback workload and limits. Add a dense-pattern workload without replacing the current measurements.
Historical maximum-budget failures remain unresolved. A successful p99 exit does not close them.

### Input prerequisites

Use the implemented keyboard and MIDI-learn routes. Pitch Gate needs a button, not a hold, encoder or new command panel.
Do not implement changes 88–93a or invent SMC-PAD factory controls here. Their physical passes remain open.
TOML entry is an explicit text-only exception for this intermediate change.

### Out of scope

No recording, count-in, step screen, slot switching, swing, note repeat, locks, redo, controller panel or MIDI clock.
No universal fades, automatic mute groups, fractional source starts, boundary snapping or changes to existing pad attack.
Change 109 stays canceled. Chop-click diagnosis, separate metronome traces and deferred sound corrections stay outside 96.

## How to verify

These procedures apply after implementation. Use a disposable project and a backup before saving format 5.
Do not edit a session file while ptsr runs. The program does not watch it for external changes.

### Keyboard

1. Load generated kick and snare sounds on bank 1 pads `1` and `2`. Save and close ptsr.
2. Use the [format example](design.md#saved-format) to add the two-bar `A1` pattern. Reopen that project.
3. Press Space on Pads. Hear the two-bar rhythm repeat without a doubled hit at its beginning.
4. Change tempo with Ctrl+Up/Down. Verify that pattern and metronome keep the same beat.
5. Swap pads `1` and `2` through Ctrl+X. Hear the same rhythm with the same sounds. Undo with Ctrl+Z.
6. Select a linked slice at +12 semitones. Toggle Pitch Gate with Alt+G on Pads. Compare its sequenced tail with Off.
7. Hit that slice manually. Verify that Alt+G does not impose a step cut on the manual hit.
8. Stop, save and reopen. Verify slot, settings and hits, with transport stopped at the beginning.

### Controller

1. In F12 MIDI, learn buttons for GLOBAL transport toggle and PADS Pitch Gate toggle. Keep existing assignments.
2. On Pads, start and stop the pattern with the learned button. Verify that releasing it does not toggle again.
3. Select the slice with a controller pad. Toggle Pitch Gate with its learned button and compare with the keyboard result.
4. Use the existing learned swap and undo actions to repeat the pad swap procedure, including a target in another bank.
5. Save and restart. Verify that learned buttons retain their roles. Report the physical device and results separately.

An unmeasured SMC-PAD remains unverified. Raw MIDI tests do not establish a hardware pass.

### Listening

Generated local files. Listening acceptance remains open:

- `renders/96-pattern-play.wav`: four two-bar loops, gate release, tempo change and a pattern replacement.
- `renders/96-pitch-gate.wav`: matching Pitch Gate On and Off sections, an empty step, a loop seam and a manual hit.

Listen for stable timing, intended short tails and unchanged attacks. Report unwanted clicks with the exact section and action.

# Tasks

## 1. Specify and compare

- [x] 1.1 Record the overview, velocity, and step-edit behavior in proposal, design, and spec deltas. Resolve conflicts with the current `patterns` requirements before code.
- [x] 1.2 Validate OpenSpec and STE text. Commit the specification before code.
- [x] 1.3 Record the 122 screen catalogue as the comparison base. It has 826 local ANSI files at `477a169`, with hashes in `/tmp/ptsr-123-base-screens.sha256`.

## 2. Overview projection

- [x] 2.1 Show rows with visible hits plus the selected pad by default. Keep all eight banks in the index. Make Alt+H show every row and keep the selected bank in view. Test hidden hits, empty banks, navigation, and keyboard/raw MIDI bindings.
- [x] 2.2 Use fixed three-cell steps and aligned numbers at 80x26 and 120x40 for every division. Keep beat, bar, triplet, cursor, and playhead marks.
- [x] 2.3 Show three velocity levels while keeping off-grid and count marks. Test thresholds, collisions, and numeric detail.

## 3. Step edit

- [x] 3.1 Add a silent pad hit to an empty cell with keyboard and raw MIDI velocity. Keep Gate and OneShot lengths, repeat and release behavior, and original sound ownership.
- [x] 3.2 Keep explicit choice for multiple hits. Give each add or remove one Undo step and verified ordered publication. Test full capacity, stale work, project change, save refusal, and reload.
- [x] 3.3 Add the named catalogue fixtures and a verified offline step-add render. Compare the unchanged catalogue areas with the base.
- [ ] 3.4 Mark the time column across all rows and the pressed pad as the row target. Name bank, pad, and step in feedback. Add keyboard, raw MIDI, TUI, and catalogue tests at both sizes.

## 4. Publish and hand over

- [x] 4.1 Run focused commands before each code commit, then `task screens`, `task openspec:check`, `task check`, and `task build` on the final code state.
- [x] 4.2 Review the diff for sound, input, saved-format, and large-file changes. Give exact keyboard, controller, view, and listening steps. Keep unreported hardware and listening results open.

## Implementation evidence

The row-filter step passed the 16 recorded pattern-view input tests, including the learned raw MIDI toggle. It passed 150 TUI tests, Clippy, the file-growth gate, and all 25 screen tests. The selected bottom row stays visible at both sizes in full mode. The catalogue has 826 names with no missing or new file. All 30 changed base frames are A1 overview or step frames.

The fixed-cell step passed 151 TUI tests, the focused app screen test, Clippy, the file-growth gate, and all 25 screen tests. All 826 catalogue names remain. The 30 changed frames are A1 overview or step frames.

The velocity step passed 152 TUI tests, 25 pattern-view tests, Clippy, and the file-growth gate. The 80-column title first clipped `triplet`. The `vel` label fixed it. All 25 screen tests then passed. The catalogue still has 826 names, and its 30 changed frames are A1 overview or step frames.

The addition step passed keyboard and raw MIDI tests for velocity, repeats, release, Gate length, stale edits, and the 4096-row limit. Undo, save refusal, reload, and ordered audio publication passed. The offline render test covers silence, level, timing within one sample, finite output, and clicks. The callback benchmark stayed below 50 % at 32 voices. The catalogue has 830 frames. Four new frames show velocity and addition at both sizes. No old frame is missing. The 30 changed old frames are A1 overview or step frames.

The final `task check` passed with 1,107 app tests and the 32-voice benchmark. `task build`, `task screens`, OpenSpec validation, and the file-growth gate passed. Early screen runs found a clipped `triplet` title and an old hint assertion. A later screen run passed with both corrections. The owner trials in `docs/LIVE-CHECKS.md` stay open.

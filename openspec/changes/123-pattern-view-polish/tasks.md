# Tasks

## 1. Specify and compare

- [x] 1.1 Record the overview, velocity, and step-edit behavior in proposal, design, and spec deltas. Resolve conflicts with the current `patterns` requirements before code.
- [x] 1.2 Validate OpenSpec and STE text. Commit the specification before code.
- [x] 1.3 Record the 122 screen catalogue as the comparison base. It has 826 local ANSI files at `477a169`, with hashes in `/tmp/ptsr-123-base-screens.sha256`.

## 2. Overview projection

- [ ] 2.1 Show rows with visible hits plus the selected pad by default. Keep all eight banks in the index. Make Alt+H show every row and keep the selected bank in view. Test hidden hits, empty banks, navigation, and keyboard/raw MIDI bindings.
- [ ] 2.2 Use fixed three-cell steps and aligned numbers at 80x26 and 120x40 for every division. Keep beat, bar, triplet, cursor, and playhead marks.
- [ ] 2.3 Show three velocity levels while keeping off-grid and count marks. Test thresholds, collisions, and numeric detail.

## 3. Step edit

- [ ] 3.1 Add a silent pad hit to an empty cell with keyboard and raw MIDI velocity. Keep Gate and OneShot lengths, repeat and release behavior, and original sound ownership.
- [ ] 3.2 Keep explicit choice for multiple hits. Give each add or remove one Undo step and verified ordered publication. Test full capacity, stale work, project change, save refusal, and reload.
- [ ] 3.3 Add the named catalogue fixtures and a verified offline step-add render. Compare the unchanged catalogue areas with the base.

## 4. Publish and hand over

- [ ] 4.1 Run focused commands before each code commit, then `task screens`, `task openspec:check`, `task check`, and `task build` on the final code state.
- [ ] 4.2 Review the diff for sound, input, saved-format, and large-file changes. Give exact keyboard, controller, view, and listening steps. Keep unreported hardware and listening results open.

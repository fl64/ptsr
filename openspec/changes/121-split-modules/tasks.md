# Tasks

## 1. Prepare the change

- [x] 1.1 Draft this proposal, design, and task list for owner review.
- [x] 1.2 Run `task openspec:check`, STE lint, and `task check`. Commit the artifacts separately.
- [x] 1.3 Ask the owner to accept this source-only scope before code moves. On 2026-11-13 the owner asked to continue after the choice between this change and pattern slots.

## 2. Freeze the comparison base

- [x] 2.1 Generate the clean base screen catalogue. Record its file count and compare method. Keep generated files out of Git.
- [x] 2.2 Check the existing keyboard, raw MIDI, sound, and callback results before code moves.

## 3. Move terminal drawing

- [x] 3.1 Move Settings drawing as a pilot. Compare both catalogue sizes. Run `task check` and commit.
- [x] 3.2a Move the shared pad frame, bank row, grid, and title into a widget module. Compare the catalogue. Run `task check` and commit.
- [x] 3.2b Move shared badges, status, and meters into a widget module. Compare the catalogue. Run `task check` and commit.
- [x] 3.2c Move shared wave drawing into a widget module. Compare the catalogue. Run `task check` and commit.
- [x] 3.3 Move Help, Files, FX, Chop, Pads, and pattern drawing into screen modules. Compare the catalogue after each move. Run `task check` and commit each screen move.

## 4. Move application code and tests

- [x] 4.1 Move each screen's App methods near its existing module. Keep method bodies, input priority, and keys. Run format, Clippy, and recorded input tests before each remaining commit.
- [ ] 4.2 Split `input_tests.rs` by topic without changing test bodies or names. Run format, Clippy, and focused input tests before each topic commit.

## 5. Check and hand over

- [ ] 5.1 Compare the complete final ANSI catalogue with the base. Run `task screens`, `task openspec:check`, `task check`, and `task build`.
- [ ] 5.2 Review the diff for behavior changes and file growth. Give the owner the keyboard, controller, view, and listening steps from the proposal. Keep unreported manual checks open.

## Baseline and pilot evidence

The clean worktree at `2909c43` produced 820 ANSI files with `task screens`. A local SHA-256 manifest uses each relative file name as its key. After the Settings move, all 820 names and bytes match the base. `cargo test -p tui --lib` passed 148 tests. The Settings function bodies did not change. The pre-move `task check` passed. We do not claim a hardware or listening result.

The first post-move `task check` failed in the callback benchmark. Active erasure wall time reached 55.27 % of the buffer budget in two callbacks. Thread CPU time reached 29.58 %. All earlier checks in that run passed. Keep this failure visible even if a later run passes.

The next full `task check` passed. In active erasure, wall time reached 10.84 % and no callback exceeded 50 %. This result does not resolve the earlier maximum-budget failures.

The shared pad-frame widget move passed `cargo test -p tui --lib` and `task check`. All 820 ANSI file names and bytes still match the base.

The shared status and meter move passed the same TUI tests and `task check`. All 820 ANSI file names and bytes still match the base. The existing title widget draws badges. This move leaves that code in `widgets.rs`.

The shared wave move passed 148 TUI tests and `task check`. All 820 ANSI file names and bytes match the base. The active erasure wall maximum was 19.88 % of the buffer budget in this run.

The Help screen move passed 148 TUI tests and `task check`. All 820 ANSI file names and bytes match the base. The first full run found an unused import. The second run passed after we limited the import to tests.

The FX screen move passed 148 TUI tests and `task check`. All 820 ANSI file names and bytes match the base.

The Files screen and project picker move passed 148 TUI tests and `task check`. All 820 ANSI file names and bytes match the base.

The Chop screen move passed 148 TUI tests and `task check`. All 820 ANSI file names and bytes match the base.

The Pads screen and pad cell move passed 148 TUI tests and `task check`. All 820 ANSI file names and bytes match the base.

The pattern strip and overview move passed 148 TUI tests and `task check`. All 820 ANSI file names and bytes match the base. The screen drawing moves are complete.

The Chop App method move passed `task check`. The method tokens match the old block after we remove the new module visibility and the two commas that Rustfmt added to function signatures. The first full run found an item after the test module. The next run found an empty line after a doc comment. Both layout fixes passed the final full run.

The folder and project picker App method move passed `task check`. Its method tokens match the old block after we remove the new module visibility and whitespace.

The pad load, copy, swap, and placement App method move passed `task check`. Its method tokens match the old block after we remove the new module visibility and whitespace.

The screen learn App method move passed `task check`. Its method tokens match the old block after we remove the new module visibility, whitespace, and Rustfmt signature commas.

The MIDI tab App method move passed `task check`. Its method tokens match the old block after we remove the new module visibility and whitespace.

The remaining Chop App method move passed `task check`. Its method tokens match the old block after we remove the new module visibility and whitespace.

The pad edit App method move passed `task check`. Its method tokens match the old block after we remove the new module visibility and whitespace. The first build found that the pad and MIDI modules both define `Param`. We kept the MIDI value method and settings method in `main.rs` for their own moves.

The FX selection and MIDI slot App method move passed `task check`. Its method tokens match the old blocks after we remove the new module visibility and whitespace.

The settings App method move passed `task check`. Its method tokens match the old blocks after we remove the new module visibility and whitespace.

The remaining Files App method move passed format, Clippy, 448 recorded input tests, and the browser unit test. Its method tokens match the old blocks after we remove the new module visibility and whitespace.

The final pad input App method move passed format, Clippy, and 448 recorded input tests. Its method tokens match the old blocks after we remove the new module visibility and whitespace. Screen-specific App methods now live near their modules. Shared session, recording, routing, and process methods remain in `main.rs`.

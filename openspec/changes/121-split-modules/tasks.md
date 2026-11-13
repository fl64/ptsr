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
- [ ] 3.3 Move Help, Files, FX, Chop, Pads, and pattern drawing into screen modules. Compare the catalogue after each move. Run `task check` and commit each screen move.

## 4. Move application code and tests

- [ ] 4.1 Move each screen's App methods near its existing module. Keep method bodies, input priority, and keys. Run recorded input tests and `task check` before each commit.
- [ ] 4.2 Split `input_tests.rs` by topic without changing test bodies or names. Run `task check` before each topic commit.

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

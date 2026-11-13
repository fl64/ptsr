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
- [ ] 3.2 Move shared badges, status, pad grid, and wave drawing into widget modules. Compare the catalogue. Run `task check` and commit each logical move.
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

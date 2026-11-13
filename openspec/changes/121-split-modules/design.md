## Context

The base is `2909c43` on `master`. `crates/tui/src/lib.rs` has 11,613 lines, `crates/app/src/main.rs` has 22,145 lines, and `crates/app/src/input_tests.rs` has 12,755 lines. App already has modules for Chop, Pads, the browser, views, and pattern input tests. The terminal renderer keeps screen data, draw functions, shared widgets, and tests in one file.

The change has no spec delta because it defines no new behavior. The main specifications remain the behavior contract.

## Decisions

### Keep a clean comparison base

Generate the ANSI catalogue from a clean worktree at the base commit. Generate it again after each terminal drawing move. Compare the file names and bytes at both supported sizes. Keep both worktrees' ignored output local. Do not add generated ANSI files to Git.

Run the existing recorded keyboard and raw MIDI tests after application or test moves. Run the existing offline sound tests and callback benchmark through `task check` before each commit.

### Move one area at a time

Start with Settings drawing as the small pilot. Move its draw code without editing its logic. Then move shared widgets and the other screens in reviewable groups: Help, Files, FX, Chop, Pads, and pattern views. Keep the public `tui` API stable. Change visibility only where a module boundary requires it.

Move App methods into the existing related modules after the drawing moves. Do not change method bodies during a move. Move recorded input tests by topic last. Keep each test body and name unchanged.

If a move exposes a behavior defect, stop that move. Do not include the repair in change 121.

### Keep commits atomic

One commit moves one logical area. Each commit includes only its own imports and visibility changes. Run `task check` before each commit. The two large files must lose lines against the PR base.

## Open questions

None. This design uses the source-only scope above.
